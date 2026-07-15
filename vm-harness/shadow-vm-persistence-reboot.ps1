param(
    [int]$SerialTcpPort = 4665,
    [string]$Image = "",
    [string]$ReportDir = "$PSScriptRoot\..\release\vm-reports",
    [int]$TimeoutSeconds = 180,
    [int]$SerialWriteChunkSize = 256,
    [int]$SerialWriteDelayMilliseconds = 0,
    [switch]$KeepRunDir
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"
$PackageScript = Join-Path $RepoRoot "scripts\package-stage0.ps1"
$Builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
$RunId = "shadow-persistence-reboot-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
$RunDir = Join-Path $env:TEMP "raios-$RunId"
$SerialLog = Join-Path $RunDir "serial-merged.log"
$ReportPath = Join-Path $ReportDir "$RunId.json"
$ReportHashPath = "$ReportPath.sha256"
$QemuPid = $null
$TempImage = $false
$Result = "failed"
$Failures = New-Object System.Collections.Generic.List[object]
$Predicates = New-Object System.Collections.Generic.List[object]
$ExecutedCommands = New-Object System.Collections.Generic.List[object]
# Write-Report does @($script:VisualEvidence.ToArray()); a null list is a method
# call on null, and PowerShell misreports it against a nearby line in the report
# hashtable. shadow-vm-smoke.ps1 has always initialized this; this script never did.
$script:VisualEvidence = New-Object System.Collections.Generic.List[object]
$StartedAt = [DateTime]::UtcNow
$script:SerialTcpClient = $null
$script:SerialTcpDrainStream = $null
$script:SerialTcpStream = $null
$QemuArgList = @()
$HardwareProfile = $null
$Profile = "persistence-reboot"
$ResolvedImage = $null
$ScratchImage = $null
$AuditRollbackTargetImage = $null
$PersistDiskImage = $null
$ResolvedArtifact = $null
$ResolvedManifest = $null
$ManifestValidation = $null
$Network = $true
$MonitorTcpPort = 0
$QmpTcpPort = 0
$FixtureReadyPath = Join-Path $RunDir "w7-fixture-ready.json"
$FixtureResultPath = Join-Path $RunDir "w7-fixture-result.json"
$FixtureModePath = Join-Path $RunDir "w7-fixture-mode.txt"
$script:SerialLogCachePath = $null
$script:SerialLogCacheLength = [int64]-1
$script:SerialLogCacheWriteTicks = [int64]-1
$script:SerialLogCacheContent = $null
$script:QemuProcess = $null
$script:QemuProcessBeforeTeardown = $null
$script:QemuProcessAfterTeardown = $null
$script:QemuTeardownAction = "not_started"
$script:SerialTransportFailure = $null

. (Join-Path $PSScriptRoot "shadow-vm-smoke-support.ps1")

function Convert-CompactJson {
    param(
        [object]$Value,
        [int]$Depth = 8
    )
    if ($null -eq $Value) {
        return "null"
    }
    return ($Value | ConvertTo-Json -Compress -Depth $Depth)
}

function Assert-ReportPredicate {
    param(
        [string]$Prefix,
        [string]$Name,
        [string]$Expected,
        [bool]$Passed,
        [string]$Actual,
        [string]$FailureMessage
    )
    Add-Predicate -Name "${Prefix}:$Name" -Expected $Expected -Passed $Passed -Actual $Actual
    if (-not $Passed) {
        throw $FailureMessage
    }
}

function Invoke-NativeCommandForReport {
    param(
        [string]$FilePath,
        [string[]]$Arguments
    )
    $stderrPath = Join-Path $RunDir ("native-{0}.err" -f [guid]::NewGuid().ToString("N"))
    try {
        $output = @(& $FilePath @Arguments 2> $stderrPath | ForEach-Object { "$_" })
        $exitCode = $LASTEXITCODE
        $errorOutput = @(Get-Content -LiteralPath $stderrPath -ErrorAction SilentlyContinue)
    }
    finally {
        Remove-Item -LiteralPath $stderrPath -Force -ErrorAction SilentlyContinue
    }
    return [pscustomobject]@{
        ExitCode = $exitCode
        Output = @($output)
        Error = @($errorOutput)
    }
}

function Send-AgentCommandTagged {
    param(
        [string]$Prefix,
        [string]$Command,
        [string]$ExpectedMarker,
        [string]$Name
    )
    Send-AgentCommand -Command $Command -ExpectedMarker $ExpectedMarker -Name "${Prefix}:$Name"
}

function Send-AgentCommandTaggedPaced {
    param(
        [string]$Prefix,
        [string]$Command,
        [string]$ExpectedMarker,
        [string]$Name
    )
    $oldChunkSize = $script:SerialWriteChunkSize
    $oldDelayMilliseconds = $script:SerialWriteDelayMilliseconds
    try {
        $script:SerialWriteChunkSize = 32
        $script:SerialWriteDelayMilliseconds = 10
        Send-AgentCommandTagged -Prefix $Prefix -Command $Command -ExpectedMarker $ExpectedMarker -Name $Name
    }
    finally {
        $script:SerialWriteChunkSize = $oldChunkSize
        $script:SerialWriteDelayMilliseconds = $oldDelayMilliseconds
    }
}

function New-MarkerImage {
    param(
        [string]$Path,
        [string]$Marker
    )
    $sector0 = New-Object byte[] 512
    $markerBytes = [System.Text.Encoding]::ASCII.GetBytes($Marker)
    [Array]::Copy($markerBytes, 0, $sector0, 0, $markerBytes.Length)
    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Create, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::Read)
    try {
        $stream.SetLength(1MB)
        $stream.Write($sector0, 0, $sector0.Length)
    }
    finally {
        $stream.Dispose()
    }
}

function Get-PersistInspection {
    param([string]$Path)
    $json = & python $Builder --inspect-json $Path
    if ($LASTEXITCODE -ne 0) {
        throw "Persist image inspection failed: $Path"
    }
    return ($json -join [Environment]::NewLine) | ConvertFrom-Json
}

function Invoke-PersistTool {
    param([string[]]$Arguments)
    $toolArguments = @($Builder) + $Arguments
    $result = Invoke-NativeCommandForReport -FilePath "python" -Arguments $toolArguments
    if ($result.ExitCode -ne 0) {
        throw "Persist image tool failed ($($Arguments -join ' ')): $((@($result.Output) + @($result.Error)) -join [Environment]::NewLine)"
    }
    return ($result.Output -join [Environment]::NewLine) | ConvertFrom-Json
}

function Assert-Boot1MemoryAppend {
    param(
        [string]$Label,
        $Record,
        [string]$GoldenPayloadSha256
    )
    $appendOk = (
        $Record -and
        $Record.durable_append -eq "appended" -and
        [bool]$Record.performed
    )
    Assert-ReportPredicate -Prefix "boot1" -Name "mem-record-$Label-appended" -Expected "durable_append == appended and performed == true" -Passed $appendOk -Actual $(if ($appendOk) { "matched" } else { Convert-CompactJson $Record 10 }) -FailureMessage "Expected memory record $Label to append"

    $goldenOk = ($Record -and $Record.payload_sha256 -eq $GoldenPayloadSha256)
    Assert-ReportPredicate -Prefix "boot1" -Name "mem-record-$Label-payload-golden" -Expected "payload_sha256 == $GoldenPayloadSha256" -Passed $goldenOk -Actual $(if ($goldenOk) { "matched" } else { [string]$Record.payload_sha256 }) -FailureMessage "Expected memory record $Label payload_sha256 to match the pinned golden"
}

function Invoke-Boot1DurableMemoryWrites {
    param([string]$Prefix)

    $observationBlob = "dm0uc21va2Uub2JzZXJ2YXRpb24Kb2JzZXJ2ZWQKYWdlbnQgYXV0aG9yZWQgZHVyYWJsZSBvYnNlcnZhdGlvbiB2aWEgbWVtb3J5Lm9ic2VydmF0aW9uX2xvZ19hcHBlbmQKdm0uc21va2UubWVtb3J5LWFnZW50LW9ic2VydmF0aW9u"

    Send-AgentCommandTagged -Prefix $Prefix -Command "agent memory.record_log_append" -ExpectedMarker "RAIOS_AGENT_END memory.record_log_append" -Name "mem-record-Z-send"
    $z = (Get-LastAgentResponseJson -Method "memory.record_log_append").body.result
    Assert-Boot1MemoryAppend -Label "Z" -Record $z -GoldenPayloadSha256 "sha256:1e0d230ecc56b4a970dd09c6ab6bbc10748aa369934dc9fa412a1f0e1a77ba8f"

    Send-AgentCommandTagged -Prefix $Prefix -Command "agent memory.decision_problem_log_append" -ExpectedMarker "RAIOS_AGENT_END memory.decision_problem_log_append" -Name "mem-record-APB-send"
    $decisionProblem = (Get-LastAgentResponseJson -Method "memory.decision_problem_log_append").body.result
    $records = @($decisionProblem.records)
    Assert-Boot1MemoryAppend -Label "A" -Record $records[0] -GoldenPayloadSha256 "sha256:9b39ac7309d7b63c95062c78d8c02bb717f25a589c437a54b627598c24198cb0"
    Assert-Boot1MemoryAppend -Label "P" -Record $records[1] -GoldenPayloadSha256 "sha256:3b010268c4e45a30bb79fee27f4cf07cead1a4542709fdc1ca9c195a7ee9a249"
    Assert-Boot1MemoryAppend -Label "B" -Record $records[2] -GoldenPayloadSha256 "sha256:5f27b06d961fe866f7c025a5a4b7ee3ffb76ee6e51db111f437c73647bb7a262"

    Send-AgentCommandTaggedPaced -Prefix $Prefix -Command "agent memory.observation_log_append $observationBlob" -ExpectedMarker "RAIOS_AGENT_END memory.observation_log_append" -Name "mem-record-obs-send"
    $observation = (Get-LastAgentResponseJson -Method "memory.observation_log_append").body.result
    Assert-Boot1MemoryAppend -Label "obs" -Record $observation -GoldenPayloadSha256 "sha256:75ea5ab92fc9dafe908bae204e5a357947e47ba7e231aaddc0c19854288e198d"

    Send-AgentCommandTagged -Prefix $Prefix -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "mem-scan-final"
    $scan = (Get-LastAgentResponseJson -Method "durable.record_log_scan").body.result
    $captured = (
        $scan.schema -eq "raios.durable_record_log_scan.v0" -and
        $scan.status -eq "valid" -and
        [int64]$scan.count -gt 0 -and
        [int64]$scan.tail_seq -gt 0 -and
        [int64]$scan.head_seq -eq 1 -and
        $null -ne $scan.head_frame_sha256 -and
        $null -ne $scan.tail_frame_sha256
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-scan-captured" -Expected "final boot-1 RECLOG scan captured count/tail/head/tail frame hashes" -Passed $captured -Actual $(if ($captured) { "count=$($scan.count) tail_seq=$($scan.tail_seq)" } else { Convert-CompactJson $scan 10 }) -FailureMessage "Expected boot 1 final memory scan capture"
    return [pscustomobject]@{
        CountFinal = [int64]$scan.count
        TailSeqFinal = [int64]$scan.tail_seq
        HeadSeq = [int64]$scan.head_seq
        HeadFrameSha256 = [string]$scan.head_frame_sha256
        TailFrameSha256 = [string]$scan.tail_frame_sha256
    }
}

function Get-DurableRecordById {
    param(
        $Records,
        [string]$Id
    )
    $matches = @($Records | Where-Object { $_.id -eq $Id })
    if ($matches.Count -gt 0) { return $matches[0] }
    return $null
}

function Get-DurableRecordIndex {
    param(
        $Records,
        [string]$Id
    )
    $recordArray = @($Records)
    for ($i = 0; $i -lt $recordArray.Count; $i += 1) {
        if ($recordArray[$i].id -eq $Id) { return $i }
    }
    return -1
}

function Get-MemoryContextDurableRecords {
    param($Context)
    return @($Context.evidence | Where-Object { $_.id -like "durable_record.*" } | ForEach-Object {
        [pscustomobject]@{
            id = $_.facts.record_id
            kind = $_.kind
            entity = $_.facts.entity
            predicate = $_.facts.predicate
            classification = $_.classification
            authority = $_.facts.authority
            scope = $_.facts.scope
            exportable = $_.facts.exportable
            status = $_.status
            reason = $_.reason
        }
    })
}

function Get-MemoryContextOmissions {
    param($Context)
    return @($Context.evidence | Where-Object { $_.id -like "omission.*" } | ForEach-Object {
        [pscustomobject]@{
            id = $_.id
            kind = $_.kind
            status = $_.status
            reason = $_.reason
            ids = @($_.facts.ids)
        }
    })
}

function Assert-Boot2MemorySurvives {
    param(
        [string]$Prefix,
        [object]$Boot1MemoryScan
    )

    Send-AgentCommandTagged -Prefix $Prefix -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "mem-reclog-scan"
    $scan = (Get-LastAgentResponseJson -Method "durable.record_log_scan").body.result

    $chainValid = $scan.status -eq "valid" -and
        [int64]$scan.count -gt [int64]$Boot1MemoryScan.CountFinal -and
        [int64]$scan.tail_seq -gt [int64]$Boot1MemoryScan.TailSeqFinal -and
        [int64]$scan.head_seq -eq 1 -and
        [string]$scan.head_frame_sha256 -eq [string]$Boot1MemoryScan.HeadFrameSha256
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-reclog-chain-valid-after-autoload" -Expected "valid boot-2 chain keeps the boot-1 head and has a newer automatic-autoload tail" -Passed $chainValid -Actual $(if ($chainValid) { "count=$($scan.count) tail_seq=$($scan.tail_seq)" } else { Convert-CompactJson $scan 10 }) -FailureMessage "Boot 2 RECLOG did not preserve the boot-1 head and continue after autoload"

    Send-AgentCommandTagged -Prefix $Prefix -Command "agent memory.context" -ExpectedMarker "RAIOS_AGENT_END memory.context" -Name "mem-broker-context"
    $contextResponse = Get-LastAgentResponseJson -Method "memory.context"
    $context = $contextResponse
    $records = Get-MemoryContextDurableRecords -Context $context
    $omitted = Get-MemoryContextOmissions -Context $context

    $zId = "mem.capability_denial.module_load_ephemeral_durable.current_boot.v0"
    $aId = "mem.decision.module_sharing_confirmed_vision.current_boot.v0"
    $pRecId = "mem.problem.memory_mutation_denied.current_boot.v0"
    $bId = "mem.decision.module_sharing_evidence_gated.current_boot.v0"
    $obsId = "mem.observation.agent.current_boot.00000001.v0"

    $visibleIds = @($records | ForEach-Object { [string]$_.id })
    $visibleSet = (
        $visibleIds -contains $zId -and
        $visibleIds -contains $pRecId -and
        $visibleIds -contains $bId -and
        $visibleIds -contains $obsId -and
        -not ($visibleIds -contains $aId)
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-broker-visible-set" -Expected "Z/P/B/obs visible in durable_records and A hidden" -Passed $visibleSet -Actual $(if ($visibleSet) { "matched" } else { $visibleIds -join "," }) -FailureMessage "Boot 2 memory.context visible durable set was wrong"

    $supersededFold = @($omitted | Where-Object { $_.kind -eq "durable_superseded" })[0]
    $supersededIds = if ($supersededFold) { @($supersededFold.ids) } else { @() }
    $aHidden = ($supersededIds -contains $aId)
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-broker-A-hidden-by-B" -Expected "A id appears in durable_superseded omitted fold" -Passed $aHidden -Actual $(if ($aHidden) { "matched" } else { Convert-CompactJson $omitted 12 }) -FailureMessage "Boot 2 broker did not fold A as superseded"

    $zRecord = Get-DurableRecordById -Records $records -Id $zId
    $zAuditVisible = ($zRecord -and $zRecord.kind -eq "capability_denial")
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-broker-audit-Z-visible" -Expected 'Z visible with kind == "capability_denial"' -Passed $zAuditVisible -Actual $(if ($zAuditVisible) { "matched" } else { Convert-CompactJson $zRecord 8 }) -FailureMessage "Boot 2 broker did not expose Z audit denial"

    $idxZ = Get-DurableRecordIndex -Records $records -Id $zId
    $idxP = Get-DurableRecordIndex -Records $records -Id $pRecId
    $idxB = Get-DurableRecordIndex -Records $records -Id $bId
    $idxObs = Get-DurableRecordIndex -Records $records -Id $obsId
    $frameSeqRanked = ($idxZ -ge 0 -and $idxP -gt $idxZ -and $idxB -gt $idxP -and $idxObs -gt $idxB)
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-broker-frameseq-ranked" -Expected "durable_records are ordered by frame seq as Z,P,B,obs after A is superseded" -Passed $frameSeqRanked -Actual "Z=$idxZ P=$idxP B=$idxB obs=$idxObs" -FailureMessage "Boot 2 broker durable records were not frame-seq ranked"

    $classificationOk = ($records.Count -gt 0)
    foreach ($record in $records) {
        if ($record.classification -ne "local_only" -or $record.exportable -ne $false) {
            $classificationOk = $false
            break
        }
    }
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-broker-classification" -Expected "every durable record is local_only and exportable == false" -Passed $classificationOk -Actual $(if ($classificationOk) { "matched" } else { Convert-CompactJson $records 12 }) -FailureMessage "Boot 2 broker durable record classification/export flags were wrong"

    $exportClosed = ($context.facts.provider_export -eq "disabled")
    Assert-ReportPredicate -Prefix $Prefix -Name "mem-broker-export-closed" -Expected 'memory.context contains provider_export == "disabled"' -Passed $exportClosed -Actual $(if ($exportClosed) { "provider_export=disabled" } else { Convert-CompactJson $context 12 }) -FailureMessage "Boot 2 broker provider export was not closed"
}

function Start-RaiosVm {
    param(
        [string]$Label,
        [string]$PredicatePrefix,
        [int]$Port,
        [int]$MonitorPort,
        [int]$QmpPort,
        [switch]$NetworkBoot,
        [string]$PersistPath
    )

    $stageImage = Join-Path $RunDir "raios-stage0-$Label.img"
    $scratch = Join-Path $RunDir "raios-stage0-$Label-scratch.img"
    $audit = Join-Path $RunDir "raios-stage0-$Label-audit-rollback-target.img"
    $log = Join-Path $RunDir "serial-$Label.log"
    Copy-Item -LiteralPath $ResolvedImage -Destination $stageImage -Force
    New-MarkerImage -Path $scratch -Marker "RAIOS_SCRATCH_V0"
    New-MarkerImage -Path $audit -Marker "RAIOS_AUDITRB_V0"

    $script:SerialLog = $log
    $script:SerialTcpPort = $Port
    $script:QmpTcpPort = $QmpPort
    $script:MonitorTcpPort = $MonitorPort
    $script:SerialLogCachePath = $null
    $script:SerialLogCacheLength = [int64]-1
    $script:SerialLogCacheWriteTicks = [int64]-1
    $script:SerialLogCacheContent = $null

    $runParams = @{
        Image = $stageImage
        ScratchImage = $scratch
        AuditRollbackTargetImage = $audit
        PersistDiskPath = $PersistPath
        PersistCacheMode = "writethrough"
        SerialMode = "tcp"
        SerialTcpPort = $Port
        SerialLog = $log
        Headless = $true
        UsbXhciInput = $true
        Cpu = "max"
        Nic = $(if ($NetworkBoot) { "e1000" } else { "none" })
    }
    $argList = @(
        "-Image", $stageImage,
        "-ScratchImage", $scratch,
        "-AuditRollbackTargetImage", $audit,
        "-PersistDiskPath", $PersistPath,
        "-PersistCacheMode", "writethrough",
        "-SerialMode", "tcp",
        "-SerialTcpPort", "$Port",
        "-SerialLog", $log,
        "-Headless",
        "-UsbXhciInput",
        "-Cpu", "max",
        "-Nic", $(if ($NetworkBoot) { "e1000" } else { "none" })
    )
    if ($MonitorPort -gt 0) {
        $runParams.MonitorTcpPort = $MonitorPort
        $argList += @("-MonitorTcpPort", "$MonitorPort")
    }
    if ($QmpPort -gt 0) {
        $runParams.QmpTcpPort = $QmpPort
        $argList += @("-QmpTcpPort", "$QmpPort")
    }
    if ($NetworkBoot) {
        $w7Fwd = "tcp:10.0.2.100:8443-tcp:127.0.0.1:8443"
        $runParams.GuestFwd = $w7Fwd
        $argList += @("-GuestFwd", $w7Fwd)
    }
    $script:QemuArgList += @("${Label}:") + $argList

    $runOutput = & $RunScript @runParams
    $qemuPid = $null
    foreach ($line in $runOutput) {
        if ($line -match '^qemu pid:\s*(\d+)') {
            $qemuPid = [int]$Matches[1]
        }
    }
    if (-not $qemuPid) {
        throw "Could not parse QEMU pid from runner output for $Label"
    }
    $script:QemuPid = $qemuPid
    try {
        $script:QemuProcess = Get-Process -Id $qemuPid -ErrorAction Stop
    }
    catch {
        $script:QemuProcess = $null
    }
    $script:QemuTeardownAction = "running"

    Assert-LogContains -Name "${PredicatePrefix}:$Label-serial-console-ready" -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "${PredicatePrefix}:$Label-framebuffer-ready" -Needle "status FRAMEBUFFER: READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "${PredicatePrefix}:$Label-usb-xhci-ready" -Needle "status USB-XHCI: READY" -TimeoutSeconds $TimeoutSeconds
    if ($NetworkBoot) {
        Assert-LogContains -Name "${PredicatePrefix}:$Label-e1000-ready" -Needle "e1000 network initialised; DHCP polling enabled" -TimeoutSeconds $TimeoutSeconds
        Assert-LogContains -Name "${PredicatePrefix}:$Label-dhcp-ready" -Needle "DHCP lease acquired:" -TimeoutSeconds $TimeoutSeconds
    }

    return [pscustomobject]@{
        Label = $Label
        PredicatePrefix = $PredicatePrefix
        SerialLog = $log
        SerialTcpPort = $Port
        MonitorPort = $MonitorPort
        QmpPort = $QmpPort
        Pid = $qemuPid
        Process = $script:QemuProcess
        StageImage = $stageImage
        ScratchImage = $scratch
        AuditRollbackTargetImage = $audit
        PersistDisk = $PersistPath
        CleanupPaths = @($stageImage, $scratch, $audit)
    }
}

function Stop-RaiosVmCleanly {
    param(
        [object]$Vm,
        [string]$Name
    )
    Close-SerialTcpConnection
    $script:QemuPid = $Vm.Pid
    $script:QemuProcess = $Vm.Process
    $passed = $false
    $actual = ""
    $client = $null
    try {
        $client = [System.Net.Sockets.TcpClient]::new()
        $connect = $client.BeginConnect("127.0.0.1", $Vm.MonitorPort, $null, $null)
        if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(5))) {
            throw "timed out connecting to QEMU monitor port $($Vm.MonitorPort)"
        }
        $client.EndConnect($connect)
        $stream = $client.GetStream()
        $bytes = [System.Text.Encoding]::ASCII.GetBytes("quit`n")
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()
        if ($null -ne $Vm.Process) {
            $passed = $Vm.Process.WaitForExit(15000)
        }
        else {
            $deadline = [DateTime]::UtcNow.AddSeconds(15)
            do {
                try {
                    Get-Process -Id $Vm.Pid -ErrorAction Stop | Out-Null
                    Start-Sleep -Milliseconds 200
                }
                catch {
                    $passed = $true
                    break
                }
            } while ([DateTime]::UtcNow -lt $deadline)
        }
        $actual = if ($passed) { "qmp_quit_exited" } else { "qmp_quit_sent_but_process_still_running" }
    }
    catch {
        $actual = $_.Exception.Message
    }
    finally {
        if ($client) {
            $client.Close()
        }
    }
    $script:QemuProcessBeforeTeardown = Get-QemuProcessSnapshot -Observation "${Name}_after_qmp_quit"
    $script:QemuTeardownAction = "qmp_quit"
    $script:QemuProcessAfterTeardown = Get-QemuProcessSnapshot -Observation "${Name}_after_qmp_quit_wait"
    Add-Predicate -Name "$($Vm.PredicatePrefix):$Name-clean-qmp-quit" -Expected "qmp_monitor_quit_exits_qemu" -Passed $passed -Actual $actual
    if (-not $passed) {
        Stop-RaiosVmForce -Vm $Vm
        throw "QEMU did not exit cleanly for ${Name}: $actual"
    }
}

function Stop-RaiosVmForce {
    param([object]$Vm)
    Close-SerialTcpConnection
    if ($Vm -and $Vm.Pid) {
        try {
            $proc = Get-Process -Id $Vm.Pid -ErrorAction Stop
            Stop-Process -Id $Vm.Pid -Force -ErrorAction SilentlyContinue
            $proc.WaitForExit(5000) | Out-Null
        }
        catch {
        }
    }
}

function Remove-RunImages {
    param([object]$Vm)
    if ($Vm -and $Vm.CleanupPaths) {
        Remove-Item -LiteralPath $Vm.CleanupPaths -Force -ErrorAction SilentlyContinue
    }
}

function Get-ProviderAutoloadMarker {
    param([int64]$BeforeOffset = -1)

    # Boot readiness only guarantees hardware-init lines; the provider-autoload
    # marker is emitted later in boot (after entropy seed, core policy,
    # project-app autoload, and — for an active install — the reverify+reload of
    # the guest from the persist disk). Wait for the line before reading, so a
    # freshly started VM is not sampled before it has autoloaded.
    $null = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
    $log = Get-SerialLogContent -Path $SerialLog
    $pattern = '^GRANTED_CANDIDATE_PROVIDER_AUTOLOAD result=(accepted|denied) phase=(autoloaded|not_installed|rolled_back|denied) reason=([a-z0-9_]+) posture=(Normal|Probation|Safe|PersistenceUnavailable) candidate_sha256=(sha256:[0-9a-f]{64}|none) promotion_transaction_sha256=(sha256:[0-9a-f]{64}|none) artifact_persist_frame_sha256=(sha256:[0-9a-f]{64}|none) m6_reverified=(true|false) w6_signature_verified=(true|false) loaded=(true|false) running=(true|false) run_count=([0-9]+) cross_reboot_proven=(true|false)$'
    $found = New-Object System.Collections.Generic.List[object]
    $offset = 0
    foreach ($rawLine in @($log -split "`n")) {
        $line = $rawLine.TrimEnd()
        if ($line.StartsWith("GRANTED_CANDIDATE_PROVIDER_AUTOLOAD ", [System.StringComparison]::Ordinal)) {
            $match = [regex]::Match($line, $pattern)
            if (-not $match.Success) {
                throw "Provider-autoload marker did not match the merged emitter: $line"
            }
            $found.Add([pscustomobject]@{
                Line = $line
                Offset = [int64]$offset
                Result = $match.Groups[1].Value
                Phase = $match.Groups[2].Value
                Reason = $match.Groups[3].Value
                Posture = $match.Groups[4].Value
                CandidateSha256 = $match.Groups[5].Value
                PromotionTransactionSha256 = $match.Groups[6].Value
                ArtifactPersistFrameSha256 = $match.Groups[7].Value
                M6Reverified = $match.Groups[8].Value -eq "true"
                W6SignatureVerified = $match.Groups[9].Value -eq "true"
                Loaded = $match.Groups[10].Value -eq "true"
                Running = $match.Groups[11].Value -eq "true"
                RunCount = [int]$match.Groups[12].Value
                CrossRebootProven = $match.Groups[13].Value -eq "true"
            }) | Out-Null
        }
        $offset += $rawLine.Length + 1
    }
    if ($found.Count -ne 1) {
        throw "Expected exactly one provider-autoload marker, found $($found.Count): $(Get-SerialLogTail -Path $SerialLog)"
    }
    $marker = $found[0]
    if ($BeforeOffset -ge 0 -and $marker.Offset -ge $BeforeOffset) {
        throw "Provider-autoload marker arrived after the first command boundary: $($marker.Line)"
    }
    return $marker
}

function Assert-Boot1ActivationAndInstall {
    param([object]$Activation, [object]$Install)

    $binding = $Activation.ActivationResponse.source_binding
    $lifecycle = $Activation.LastLifecycleResponse
    $runOnce = $binding.source_kind -eq "w7" -and
        $binding.candidate_sha256 -eq $Activation.CandidateSha256 -and
        $binding.receipt_sha256 -eq $Activation.W7ReceiptSha256 -and
        $binding.receiver_identity.content_sha256 -eq $Activation.CandidateSha256 -and
        $binding.receiver_identity.retained_candidate_sha256 -eq $Activation.CandidateSha256 -and
        $binding.receiver_identity.catalog_finalize_candidate_sha256 -eq $Activation.CandidateSha256 -and
        $binding.computed_grant.computed_grant_hash -eq $Activation.M6ComputedGrantSha256 -and
        $binding.local_attestation.attestation_reference_hash -eq $Activation.M6AttestationReferenceSha256 -and
        $binding.local_attestation.signature_verified -eq $true -and
        $lifecycle.loaded -eq $true -and $lifecycle.running -eq $true -and
        [int]$lifecycle.run_count -eq 1 -and $Activation.GuestLogCount -eq 1 -and
        $lifecycle.durable_promotion_transaction.performed -eq $false -and
        $lifecycle.durable_artifact_persist.performed -eq $false
    Assert-ReportPredicate -Prefix "boot1" -Name "b12c-w7-run-once" -Expected "the exact W7/receiver/M6 binding runs once after its physical approval and has zero durable effect before W6" -Passed $runOnce -Actual $(if ($runOnce) { "candidate=$($Activation.CandidateSha256) run_count=1 durable=false" } else { Convert-CompactJson $Activation 24 }) -FailureMessage "Boot 1 did not establish the exact one-shot W7/M6 activation"

    $installWindowLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$Install.InstallOffset)
    $installed = $Install.CandidateSha256 -eq $Activation.CandidateSha256 -and
        $Install.W7ReceiptSha256 -eq $Activation.W7ReceiptSha256 -and
        $Install.ActivationApprovalSha256 -eq $Activation.ActivationChallengeSha256 -and
        $Install.Preview.install_source -eq "granted_candidate" -and
        $Install.Preview.receipt_kind -eq "w7_acquisition" -and
        $Install.Preview.w4_project_receipt_present -eq $false -and
        $Install.Preview.candidate_sha256 -eq $Activation.CandidateSha256 -and
        $Install.Preview.receipt_sha256 -eq $Activation.W7ReceiptSha256 -and
        $Install.Preview.activation_approval_sha256 -eq $Activation.ActivationChallengeSha256 -and
        $Install.SignedPreview.signature_verified -eq $true -and
        $Install.ActionSignatureMessageSha256 -ne $Activation.M6AttestationReferenceSha256 -and
        $Install.InstallEnvelopeSha256 -eq $Install.Preview.install_envelope_sha256 -and
        $Install.InstallActionSha256 -match '^sha256:[0-9a-f]{64}$' -and
        $Install.PromotionTransactionSha256 -match '^sha256:[0-9a-f]{64}$' -and
        $Install.ArtifactPersistFrameSha256 -match '^sha256:[0-9a-f]{64}$' -and
        $Install.PhysicalApprovalSha256 -match '^sha256:[0-9a-f]{64}$' -and
        $Install.PhysicalApprovalSha256 -ne $Activation.ActivationChallengeSha256 -and
        $binding.computed_grant.computed_grant_hash -eq $Activation.M6ComputedGrantSha256 -and
        $binding.local_attestation.manifest_reference_hash -eq $Activation.M6ManifestReferenceSha256 -and
        $binding.local_attestation.artifact_reference_hash -eq $Activation.M6ArtifactReferenceSha256 -and
        $binding.local_attestation.vm_report_reference_hash -eq $Activation.M6VmReportReferenceSha256 -and
        $binding.local_attestation.attestation_reference_hash -eq $Activation.M6AttestationReferenceSha256 -and
        [int64]$Install.PostReclogScan.count -eq ([int64]$Install.PreReclogScan.count + 3) -and
        [int64]$Install.Sequence -eq ([int64]$Install.PreReclogScan.tail_seq + 2) -and
        $Install.RunCount -eq 1 -and $Install.DurableWrites -eq $true -and
        ($Activation.ClickCount + $Install.ClickCount) -eq 3 -and
        # The candidate ran exactly once during physical activation; the W6
        # install commits durably but must NOT re-run the guest. Count in the
        # post-install window only (the whole boot also contains selftest and
        # recovery-probe guest logs), matching the network/m6d no-rerun pins.
        ([regex]::Matches($installWindowLog, [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count -eq 0 -and
        ([regex]::Matches($installWindowLog, [regex]::Escape("GRANTED_CANDIDATE_INSTALL_COMMIT result=accepted"))).Count -eq 1
    Assert-ReportPredicate -Prefix "boot1" -Name "w6-second-click-installed" -Expected "a distinct W6 pointer approval appends authorization/promote/artifact frames for the exact running W7 candidate without a second run" -Passed $installed -Actual $(if ($installed) { $Install.MarkerLine } else { Convert-CompactJson $Install 24 }) -FailureMessage "Boot 1 W6 install did not commit the exact one-shot W7 candidate"
}

function Assert-Boot1HostProvenance {
    param([object]$Inspection, [object]$Activation, [object]$Install)

    $frames = @($Inspection.reclog_frames)
    $authorization = @($frames | Where-Object { $_.payload_json.schema -eq "raios.install_authorization.v0" -and [int64]$_.seq -eq ([int64]$Install.Sequence - 1) })[0]
    $promote = @($frames | Where-Object { "sha256:$($_.frame_sha256)" -eq $Install.PromotionTransactionSha256 })[0]
    $persist = @($frames | Where-Object { "sha256:$($_.frame_sha256)" -eq $Install.ArtifactPersistFrameSha256 })[0]
    $artifact = @($Inspection.artifact_persist_records | Where-Object { "sha256:$($_.reclog_frame_sha256)" -eq $Install.ArtifactPersistFrameSha256 })[0]
    $authHash = if ($authorization) { "sha256:$($authorization.frame_sha256)" } else { "none" }
    $ok = $Inspection.reclog_scan.status -eq "valid" -and
        $authorization.payload_json.w7_receipt_sha256 -eq $Activation.W7ReceiptSha256 -and
        $authorization.payload_json.activation_approval_sha256 -eq $Activation.ActivationChallengeSha256 -and
        $authorization.payload_json.receiver_content_sha256 -eq $Activation.CandidateSha256 -and
        $authorization.payload_json.receiver_candidate_sha256 -eq $Activation.CandidateSha256 -and
        $authorization.payload_json.catalog_candidate_sha256 -eq $Activation.CandidateSha256 -and
        $authorization.payload_json.install_envelope_sha256 -eq $Install.InstallEnvelopeSha256 -and
        $authorization.payload_json.install_action_sha256 -eq $Install.InstallActionSha256 -and
        $authorization.payload_json.install_action_signature_message_sha256 -eq $Install.ActionSignatureMessageSha256 -and
        $authorization.payload_json.physical_approval_sha256 -eq $Install.PhysicalApprovalSha256 -and
        $authorization.payload_json.install_signature_der -eq $Install.SignatureHex -and
        [int64]$authorization.payload_json.install_signature_len -eq ([int64]$Install.SignatureHex.Length / 2) -and
        [int64]$promote.seq -eq [int64]$Install.Sequence -and
        $promote.payload_json.transaction_kind -eq "promote" -and
        $promote.payload_json.install_authorization_frame_sha256 -eq $authHash -and
        $promote.payload_json.manifest_reference_hash -eq $Activation.M6ManifestReferenceSha256 -and
        $promote.payload_json.artifact_reference_hash -eq $Activation.M6ArtifactReferenceSha256 -and
        $promote.payload_json.vm_report_reference_hash -eq $Activation.M6VmReportReferenceSha256 -and
        $promote.payload_json.attestation_reference_hash -eq $Activation.M6AttestationReferenceSha256 -and
        [int64]$persist.seq -eq ([int64]$Install.Sequence + 1) -and
        $persist.payload_json.schema -eq "raios.artifact_persist.v0" -and
        $persist.payload_json.promotion_transaction_sha256 -eq $Install.PromotionTransactionSha256 -and
        $artifact.binding_ok -eq $true -and
        $artifact.artifact_sha256 -eq $Activation.CandidateSha256 -and
        $artifact.actual_artifact_sha256 -eq $Activation.CandidateSha256.Substring(7) -and
        $artifact.promotion_transaction_sha256 -eq $Install.PromotionTransactionSha256 -and
        $Inspection.first_artstor_blob.valid -eq $true -and
        $Inspection.first_artstor_blob.payload_sha256 -eq $Activation.CandidateSha256.Substring(7) -and
        [int64]$Inspection.first_artstor_blob.payload_len -eq 4205
    $dump = [ordered]@{ inspection = $Inspection; activation = $Activation; install = $Install }
    Assert-ReportPredicate -Prefix "boot1" -Name "persist-disk-has-w6-provenance" -Expected "clean-quit host inspection finds the linked authorization/promote/artifact frames and exact 4205-byte ARTSTOR payload" -Passed $ok -Actual $(if ($ok) { "authorization=$authHash promote=$($Install.PromotionTransactionSha256) artifact=$($Install.ArtifactPersistFrameSha256)" } else { Convert-CompactJson $dump 30 }) -FailureMessage "Boot 1 kept disk did not contain the exact linked W7/M6/W6 provenance"
}

function Assert-Boot2Autoload {
    param([object]$Activation, [object]$Install, [object]$Boot1Inspection)

    # Boot readiness only guarantees hardware-init lines; the provider-autoload
    # marker is emitted later (after entropy seed, core policy, project-app
    # autoload, and — for an active install — the reverify+reload of the guest
    # from the persist disk). Wait for the marker line before reading it once.
    $markerWaited = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
    Assert-ReportPredicate -Prefix "boot2" -Name "provider-autoload-marker-emitted" -Expected "boot 2 emits a provider-autoload marker before the first serial command" -Passed $markerWaited -Actual $(if ($markerWaited) { "emitted" } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Boot 2 never emitted a provider-autoload marker"
    $firstCommandOffset = Get-SerialLogOffset
    $marker = Get-ProviderAutoloadMarker -BeforeOffset $firstCommandOffset
    $log = Get-SerialLogContent -Path $SerialLog
    $readyOffset = $log.IndexOf("SERIAL CONSOLE READY", [System.StringComparison]::Ordinal)
    $before = $marker.Result -eq "accepted" -and $marker.Phase -eq "autoloaded" -and
        $marker.Reason -eq "repromotion_reverified_m6_gate_loaded_and_started" -and
        $marker.Posture -eq "Normal" -and $marker.M6Reverified -and $marker.W6SignatureVerified -and
        $marker.Loaded -and $marker.Running -and $marker.RunCount -eq 1 -and $marker.CrossRebootProven -and
        $readyOffset -ge 0 -and $marker.Offset -gt $readyOffset -and $marker.Offset -lt $firstCommandOffset
    Assert-ReportPredicate -Prefix "boot2" -Name "provider-autoload-before-command" -Expected "the exact accepted Normal/autoloaded marker follows boot readiness and precedes the first tagged serial command" -Passed $before -Actual $(if ($before) { $marker.Line } else { Convert-CompactJson $marker 12 }) -FailureMessage "Boot 2 provider autoload was not complete before the first command"

    $manual = @($ExecutedCommands | Where-Object { $_.command -eq "agent repromotion.run" }).Count -eq 0
    Assert-ReportPredicate -Prefix "boot2" -Name "no-manual-repromotion-command" -Expected "no executed command is agent repromotion.run before the positive autoload assertion" -Passed $manual -Actual $(if ($manual) { "agent repromotion.run absent" } else { Convert-CompactJson @($ExecutedCommands) 8 }) -FailureMessage "Boot 2 positive path used manual repromotion"

    $authFrame = @($Boot1Inspection.reclog_frames | Where-Object { $_.payload_json.schema -eq "raios.install_authorization.v0" -and [int64]$_.seq -eq ([int64]$Install.Sequence - 1) })[0]
    $promoteFrame = @($Boot1Inspection.reclog_frames | Where-Object { "sha256:$($_.frame_sha256)" -eq $Install.PromotionTransactionSha256 })[0]
    $chain = $marker.CandidateSha256 -eq $Activation.CandidateSha256 -and
        $marker.PromotionTransactionSha256 -eq $Install.PromotionTransactionSha256 -and
        $marker.ArtifactPersistFrameSha256 -eq $Install.ArtifactPersistFrameSha256 -and
        $authFrame.payload_json.w7_receipt_sha256 -eq $Activation.W7ReceiptSha256 -and
        $authFrame.payload_json.receiver_content_sha256 -eq $Activation.CandidateSha256 -and
        $authFrame.payload_json.receiver_candidate_sha256 -eq $Activation.CandidateSha256 -and
        $authFrame.payload_json.catalog_candidate_sha256 -eq $Activation.CandidateSha256 -and
        $authFrame.payload_json.install_envelope_sha256 -eq $Install.InstallEnvelopeSha256 -and
        $authFrame.payload_json.install_action_sha256 -eq $Install.InstallActionSha256 -and
        $authFrame.payload_json.install_action_signature_message_sha256 -eq $Install.ActionSignatureMessageSha256 -and
        $authFrame.payload_json.physical_approval_sha256 -eq $Install.PhysicalApprovalSha256 -and
        $authFrame.payload_json.install_signature_der -eq $Install.SignatureHex -and
        [int64]$authFrame.payload_json.install_signature_len -eq ([int64]$Install.SignatureHex.Length / 2) -and
        $promoteFrame.payload_json.manifest_reference_hash -eq $Activation.M6ManifestReferenceSha256 -and
        $promoteFrame.payload_json.artifact_reference_hash -eq $Activation.M6ArtifactReferenceSha256 -and
        $promoteFrame.payload_json.vm_report_reference_hash -eq $Activation.M6VmReportReferenceSha256 -and
        $promoteFrame.payload_json.attestation_reference_hash -eq $Activation.M6AttestationReferenceSha256
    $chainDump = [ordered]@{ marker = $marker; authorization = $authFrame; promotion = $promoteFrame; activation = $Activation; install = $Install }
    Assert-ReportPredicate -Prefix "boot2" -Name "autoload-chain-exact" -Expected "autoload selects the exact boot-1 candidate/promote/artifact plus W7 receiver, M6 references, and W6 envelope/action/signature provenance" -Passed $chain -Actual $(if ($chain) { $marker.Line } else { Convert-CompactJson $chainDump 30 }) -FailureMessage "Boot 2 autoload did not preserve the exact boot-1 authority chain"

    Send-AgentCommandTagged -Prefix "boot2" -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "autoload-inventory"
    $inventory = Get-LastAgentResponseJson -Method "service.inventory"
    Send-AgentCommandTagged -Prefix "boot2" -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "autoload-slot"
    $slot = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
    $rows = @($inventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" })
    $row = $rows[0]
    $liveSlot = @($slot.evidence | Where-Object id -eq "live_granted_service_slot")[0].facts
    $live = $rows.Count -eq 1 -and [int64]$row.generation -eq [int64]$Install.Generation -and
        $row.last_run_outcome -eq "success" -and $row.service_slot_activation_active -eq $true -and
        $row.trust_tier -eq "dev_key_not_owner_sealed" -and $row.PSObject.Properties.Name -notcontains "run_count" -and
        $slot.facts.runtime.live_granted_service_slot_present -eq $true -and
        $liveSlot.service_slot_allocated -eq $true -and $liveSlot.running -eq $true -and
        $liveSlot.trust_tier -eq "dev_key_not_owner_sealed" -and
        ([regex]::Matches((Get-SerialLogContent -Path $SerialLog), [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count -eq 1
    $liveDump = [ordered]@{ inventory = $inventory; slot = $slot; log = (Get-SerialLogContent -Path $SerialLog) }
    Assert-ReportPredicate -Prefix "boot2" -Name "autoload-service-live" -Expected "autoload alone creates one healthy inventory row, one positive slot, and exactly one boot-local guest run" -Passed $live -Actual $(if ($live) { "inventory/slot live; guest_log=1" } else { Convert-CompactJson $liveDump 24 }) -FailureMessage "Boot 2 autoloaded service was not live exactly once"
    return $marker
}

function Assert-Boot2Rollback {
    param([object]$Activation, [object]$Install, [object]$Boot1MemoryScan)

    Send-AgentCommandTagged -Prefix "boot2" -Command "service.rollback_preview svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_preview" -Name "rollback-preview"
    $previewResponse = Get-LastAgentResponseJson -Method "service.rollback_preview"
    $preview = $previewResponse.body.result
    Send-AgentCommandTagged -Prefix "boot2" -Command "service.rollback_apply svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply" -Name "rollback-apply"
    $applyResponse = Get-LastAgentResponseJson -Method "service.rollback_apply"
    $apply = $applyResponse.body.result
    $verification = $apply.rollback_verification
    $unpromote = $verification.durable_unpromote_transaction
    Send-AgentCommandTagged -Prefix "boot2" -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "rollback-reclog"
    $scanResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
    $scan = $scanResponse.body.result
    $ok = $preview.code -eq "ok" -and $preview.reason -eq "rollback_plan_recorded_ram_only" -and
        $preview.rollback_plan.artifact_hash -eq $Activation.CandidateSha256 -and
        $apply.code -eq "ok" -and $apply.reason -eq "unpromoted_dev_key_granted_external_wasm_current_boot" -and
        $apply.loaded -eq $false -and $apply.running -eq $false -and
        $verification.stopped -eq $true -and $verification.drop_clear_bytes -eq $true -and
        $verification.free_slot -eq $true -and $verification.remove_inventory -eq $true -and
        $verification.restore_hash_verified -eq $true -and
        $verification.reprojected_service_inventory_hash -eq $verification.pre_load_service_inventory_hash -and
        $unpromote.transaction_kind -eq "unpromote" -and $unpromote.performed -eq $true -and
        $unpromote.readback_sha256 -eq $unpromote.frame_sha256 -and $unpromote.reparse_valid -eq $true -and
        $unpromote.signature_verified -eq $true -and $unpromote.grant_binds_capability -eq $true -and
        $scan.status -eq "valid" -and $scan.tail_frame_sha256 -eq $unpromote.frame_sha256 -and
        # Boot 2 appends four frames by this point: the autoload re-promotion
        # links the existing authorization and appends promote + artifact, then
        # re-running the guest records its wasm import-grant as a third durable
        # memory record; rollback then appends the unpromote.
        [int64]$scan.count -eq ([int64]$Boot1MemoryScan.CountFinal + 4)
    $dump = [ordered]@{ preview = $previewResponse; apply = $applyResponse; scan = $scanResponse }
    Assert-ReportPredicate -Prefix "boot2" -Name "rollback-restores-and-persists" -Expected "rollback restores the exact pre-load inventory and appends a readback-verified linked unpromote as the newest RECLOG frame" -Passed $ok -Actual $(if ($ok) { "restored=$($verification.reprojected_service_inventory_hash) unpromote=$($unpromote.frame_sha256)" } else { Convert-CompactJson $dump 28 }) -FailureMessage "Boot 2 rollback did not restore and persist the unpromote"
    return [pscustomobject]@{ FrameSha256 = [string]$unpromote.frame_sha256; GuestScan = $scan; Response = $applyResponse }
}

function Assert-Boot2HostChain {
    param([object]$Boot1Inspection, [object]$Boot2Inspection, [object]$Rollback)

    $boot1Frames = @($Boot1Inspection.reclog_frames)
    $boot2Frames = @($Boot2Inspection.reclog_frames)
    $prefixOk = $boot2Frames.Count -gt $boot1Frames.Count
    for ($i = 0; $prefixOk -and $i -lt $boot1Frames.Count; $i += 1) {
        $prefixOk = [int64]$boot2Frames[$i].seq -eq [int64]$boot1Frames[$i].seq -and
            $boot2Frames[$i].frame_sha256 -eq $boot1Frames[$i].frame_sha256 -and
            $boot2Frames[$i].payload_sha256 -eq $boot1Frames[$i].payload_sha256
    }
    Assert-ReportPredicate -Prefix "boot2" -Name "mem-reclog-boot1-prefix-survives" -Expected "every boot-1 RECLOG sequence/frame/payload hash remains an identical prefix and the head hash is unchanged" -Passed $prefixOk -Actual $(if ($prefixOk) { "prefix_frames=$($boot1Frames.Count) head=$($Boot2Inspection.reclog_scan.head_frame_sha256)" } else { Convert-CompactJson $Boot2Inspection.reclog_frames 20 }) -FailureMessage "Boot 2 changed the boot-1 RECLOG prefix"

    # The autoload re-promotion links the EXISTING boot-1 authorization frame
    # (no duplicate), so the first two new frames are promote then artifact;
    # re-running the guest then records its wasm import-grant as a third durable
    # memory record, and boot-2 rollback appends the unpromote as the newest tail.
    $firstNew = $boot2Frames[$boot1Frames.Count]
    $secondNew = $boot2Frames[$boot1Frames.Count + 1]
    $tailOk = $Boot2Inspection.reclog_scan.status -eq "valid" -and
        [int64]$firstNew.seq -eq ([int64]$Boot1Inspection.reclog_scan.tail_seq + 1) -and
        $firstNew.payload_json.schema -eq "raios.promotion_transaction.v0" -and
        $firstNew.payload_json.transaction_kind -eq "promote" -and
        [int64]$secondNew.seq -eq ([int64]$firstNew.seq + 1) -and
        $secondNew.payload_json.schema -eq "raios.artifact_persist.v0" -and
        [int64]$Boot2Inspection.reclog_scan.tail_seq -gt [int64]$Boot1Inspection.reclog_scan.tail_seq -and
        "sha256:$($Boot2Inspection.reclog_scan.tail_frame_sha256)" -eq $Rollback.FrameSha256
    $tailDump = [ordered]@{ boot1 = $Boot1Inspection.reclog_scan; boot2 = $Boot2Inspection.reclog_scan; first_new = $firstNew; second_new = $secondNew; rollback = $Rollback }
    Assert-ReportPredicate -Prefix "boot2" -Name "mem-reclog-tail-continues-after-autoload" -Expected "the valid chain's first new frame follows the immutable boot-1 tail, autoload appends promote/artifact, and rollback leaves a newer unpromote tail" -Passed $tailOk -Actual $(if ($tailOk) { "first_new_seq=$($firstNew.seq) tail_seq=$($Boot2Inspection.reclog_scan.tail_seq)" } else { Convert-CompactJson $tailDump 24 }) -FailureMessage "Boot 2 RECLOG tail did not continue validly after autoload"
}

function Assert-Boot3RollbackState {
    param([object]$Activation, [object]$Rollback)

    # Same wait as boot 2: the rolled-back marker is emitted after the resolver
    # runs during early boot; read it only once it has appeared.
    $markerWaited = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
    Assert-ReportPredicate -Prefix "boot3" -Name "provider-autoload-marker-emitted" -Expected "boot 3 emits a provider-autoload marker before the first serial command" -Passed $markerWaited -Actual $(if ($markerWaited) { "emitted" } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Boot 3 never emitted a provider-autoload marker"
    $firstCommandOffset = Get-SerialLogOffset
    $marker = Get-ProviderAutoloadMarker -BeforeOffset $firstCommandOffset
    $preCommandLog = Get-SerialLogContent -Path $SerialLog
    $preceded = $marker.Result -eq "accepted" -and $marker.Phase -eq "rolled_back" -and
        $marker.Reason -eq "granted_candidate_install_rolled_back" -and $marker.Posture -eq "Normal" -and
        $marker.CandidateSha256 -eq $Activation.CandidateSha256 -and
        $marker.PromotionTransactionSha256 -eq $Rollback.FrameSha256 -and
        $marker.ArtifactPersistFrameSha256 -eq "none" -and
        -not $marker.M6Reverified -and -not $marker.W6SignatureVerified -and
        -not $marker.Loaded -and -not $marker.Running -and $marker.RunCount -eq 0 -and
        -not $marker.CrossRebootProven -and $marker.Offset -lt $firstCommandOffset
    Assert-ReportPredicate -Prefix "boot3" -Name "rollback-resolution-precedes-command" -Expected "the exact accepted rolled_back marker names the newest unpromote and precedes the first command with no load/run" -Passed $preceded -Actual $(if ($preceded) { $marker.Line } else { Convert-CompactJson $marker 12 }) -FailureMessage "Boot 3 did not resolve the durable unpromote before commands"

    $noAutoload = -not $preCommandLog.Contains("GRANTED_CANDIDATE_PROVIDER_AUTOLOAD result=accepted phase=autoloaded") -and
        -not $preCommandLog.Contains("RAIOS_AGENT_BEGIN service.load_ephemeral") -and
        -not $preCommandLog.Contains("RAIOS_AGENT_BEGIN service.start") -and
        -not $preCommandLog.Contains("WASM_GUEST_LOG echo counter=")
    Assert-ReportPredicate -Prefix "boot3" -Name "no-autoload-after-rollback" -Expected "rolled-back resolution emits no retain/load/start path and no guest log" -Passed $noAutoload -Actual $(if ($noAutoload) { "no load/start/guest log" } else { $preCommandLog }) -FailureMessage "Boot 3 autoloaded after the durable rollback"

    Send-AgentCommandTagged -Prefix "boot3" -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "reclog-scan"
    $scanResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
    Send-AgentCommandTagged -Prefix "boot3" -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "inventory"
    $inventory = Get-LastAgentResponseJson -Method "service.inventory"
    Send-AgentCommandTagged -Prefix "boot3" -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "slot"
    $slot = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
    $absent = @($inventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" }).Count -eq 0 -and
        $slot.facts.runtime.live_granted_service_slot_present -eq $false -and
        @($slot.evidence | Where-Object id -eq "live_granted_service_slot").Count -eq 0
    $dump = [ordered]@{ inventory = $inventory; slot = $slot }
    Assert-ReportPredicate -Prefix "boot3" -Name "service-remains-absent" -Expected "inventory lacks the granted service and the runtime slot fact remains absent" -Passed $absent -Actual $(if ($absent) { "inventory/slot absent" } else { Convert-CompactJson $dump 22 }) -FailureMessage "Boot 3 resurrected the rolled-back service"
    return [pscustomobject]@{ Marker = $marker; GuestScan = $scanResponse.body.result }
}

function Assert-Boot3HostChain {
    param([object]$Inspection, [object]$Boot3State, [object]$Rollback)

    $transactions = @($Inspection.promotion_transaction_records)
    $newest = @($transactions | Sort-Object seq | Select-Object -Last 1)[0]
    $scan = $Inspection.reclog_scan
    $guest = $Boot3State.GuestScan
    $ok = $scan.status -eq "valid" -and $guest.status -eq "valid" -and
        [int64]$scan.count -eq [int64]$guest.count -and [int64]$scan.head_seq -eq [int64]$guest.head_seq -and
        [int64]$scan.tail_seq -eq [int64]$guest.tail_seq -and
        "sha256:$($scan.head_frame_sha256)" -eq [string]$guest.head_frame_sha256 -and
        "sha256:$($scan.tail_frame_sha256)" -eq [string]$guest.tail_frame_sha256 -and
        $newest.transaction_kind -eq "unpromote" -and "sha256:$($newest.reclog_frame_sha256)" -eq $Rollback.FrameSha256 -and
        "sha256:$($scan.tail_frame_sha256)" -eq $Rollback.FrameSha256
    $dump = [ordered]@{ host = $Inspection; guest = $guest; newest = $newest; rollback = $Rollback }
    Assert-ReportPredicate -Prefix "boot3" -Name "reclog-chain-valid-after-unpromote" -Expected "host and guest agree on the valid RECLOG head/tail/count and the linked unpromote remains newest" -Passed $ok -Actual $(if ($ok) { "count=$($scan.count) tail=$($Rollback.FrameSha256)" } else { Convert-CompactJson $dump 28 }) -FailureMessage "Boot 3 host/guest RECLOG scans did not preserve the newest unpromote"
}

function Assert-RepromotionDeniedChild {
    param(
        [string]$Label,
        [string]$MutationFlag,
        [string]$ExpectedMismatch,
        [string]$ExpectedReason,
        [int]$Port
    )
    $prefix = $Label
    $childVm = $null
    $childPersist = Join-Path $RunDir "persist-$Label.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $childPersist -Force
    try {
        Invoke-PersistTool -Arguments @($MutationFlag, $childPersist) | Out-Null
        $inspection = Get-PersistInspection -Path $childPersist
        $records = @($inspection.artifact_persist_records)
        $mismatches = if ($records.Count -gt 0) { @($records[0].binding_mismatches) } else { @() }
        $landed = $mismatches -contains $ExpectedMismatch
        Assert-ReportPredicate -Prefix $prefix -Name "$Label-tamper-landed" -Expected "$ExpectedMismatch mismatch visible in inspect-json" -Passed $landed -Actual $(Convert-CompactJson $records 10) -FailureMessage "$Label tamper did not land"

        $childVm = Start-RaiosVm -Label "denyhash-$Label" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -QmpPort 0 -PersistPath $childPersist
        # No command is sent before this read, so the marker precedes the first
        # command by construction; Get-ProviderAutoloadMarker waits for it.
        $marker = Get-ProviderAutoloadMarker
        $expectedPhase = if ($ExpectedReason -eq "no_w6_authorized_install") { "not_installed" } else { "denied" }
        $expectedResult = if ($ExpectedReason -eq "no_w6_authorized_install") { "accepted" } else { "denied" }
        $deniedOk = $marker.Result -eq $expectedResult -and $marker.Phase -eq $expectedPhase -and
            $marker.Reason -eq $ExpectedReason -and -not $marker.M6Reverified -and
            -not $marker.W6SignatureVerified -and -not $marker.Loaded -and
            -not $marker.Running -and $marker.RunCount -eq 0 -and -not $marker.CrossRebootProven
        Assert-ReportPredicate -Prefix $prefix -Name "provider-autoload-denied-hash-mismatch" -Expected "automatic provider autoload rejects the mutated hash/link before authority, with both signature outcomes false and no fallback" -Passed $deniedOk -Actual $(if ($deniedOk) { $marker.Line } else { Convert-CompactJson $marker 12 }) -FailureMessage "$Label did not fail closed in automatic provider autoload"

        Send-AgentCommandTagged -Prefix $prefix -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "inventory-after-denial"
        $inventory = Get-LastAgentResponseJson -Method "service.inventory"
        Send-AgentCommandTagged -Prefix $prefix -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "slot-after-denial"
        $slot = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
        $childLog = Get-SerialLogContent -Path $SerialLog
        $noAnswer = @($inventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" }).Count -eq 0 -and
            $slot.facts.runtime.live_granted_service_slot_present -eq $false -and
            @($slot.evidence | Where-Object id -eq "live_granted_service_slot").Count -eq 0 -and
            -not $childLog.Contains("WASM_GUEST_LOG echo counter=")
        $dump = [ordered]@{ marker = $marker; inventory = $inventory; slot = $slot; log = $childLog }
        Assert-ReportPredicate -Prefix $prefix -Name "no-service-answer" -Expected "the denied automatic path leaves no guest log, inventory row, or active slot" -Passed $noAnswer -Actual $(if ($noAnswer) { "guest/inventory/slot absent" } else { Convert-CompactJson $dump 22 }) -FailureMessage "$Label denial still left a live service"
    }
    finally {
        Stop-RaiosVmForce -Vm $childVm
        Remove-RunImages -Vm $childVm
        Remove-Item -LiteralPath $childPersist -Force -ErrorAction SilentlyContinue
    }
}

function Assert-SafeChild {
    param([int]$Port)
    $prefix = "safe"
    $safeVm = $null
    $safePersist = Join-Path $RunDir "persist-safe.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $safePersist -Force
    try {
        Invoke-PersistTool -Arguments @("--image", $safePersist, "--seed-bootctl", "both-invalid") | Out-Null
        $inspection = Get-PersistInspection -Path $safePersist
        $safePosture = $inspection.bootctl_read.decision.posture -eq "Safe"
        Assert-ReportPredicate -Prefix $prefix -Name "bootctl-safe-posture-landed" -Expected "both-invalid bootctl inspect-json posture Safe" -Passed $safePosture -Actual $(Convert-CompactJson $inspection.bootctl_read.decision 6) -FailureMessage "SAFE child bootctl patch did not land"

        $safeVm = Start-RaiosVm -Label "safe" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -QmpPort 0 -PersistPath $safePersist
        # No command is sent before this read, so the marker precedes the first
        # command by construction; Get-ProviderAutoloadMarker waits for it.
        $marker = Get-ProviderAutoloadMarker
        $skipped = $marker.Result -eq "denied" -and $marker.Phase -eq "denied" -and
            $marker.Reason -eq "provider_autoload_posture_not_normal" -and $marker.Posture -eq "Safe" -and
            $marker.CandidateSha256 -eq "none" -and $marker.PromotionTransactionSha256 -eq "none" -and
            -not $marker.M6Reverified -and -not $marker.W6SignatureVerified -and
            -not $marker.Loaded -and -not $marker.Running -and $marker.RunCount -eq 0 -and -not $marker.CrossRebootProven
        Assert-ReportPredicate -Prefix $prefix -Name "provider-autoload-skipped" -Expected "Safe posture denies provider autoload before record intake with no authority or service effect" -Passed $skipped -Actual $(if ($skipped) { $marker.Line } else { Convert-CompactJson $marker 12 }) -FailureMessage "Safe child did not skip provider autoload before record intake"

        Send-AgentCommandTagged -Prefix $prefix -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "inventory-after-skip"
        $inventory = Get-LastAgentResponseJson -Method "service.inventory"
        Send-AgentCommandTagged -Prefix $prefix -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "slot-after-skip"
        $slot = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
        $safeLog = Get-SerialLogContent -Path $SerialLog
        $noAnswer = @($inventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" }).Count -eq 0 -and
            $slot.facts.runtime.live_granted_service_slot_present -eq $false -and
            -not $safeLog.Contains("WASM_GUEST_LOG echo counter=")
        Assert-ReportPredicate -Prefix $prefix -Name "no-service-answer" -Expected "Safe provider-autoload skip leaves no guest log, inventory row, or active slot" -Passed $noAnswer -Actual $(if ($noAnswer) { "guest/inventory/slot absent" } else { Convert-CompactJson ([ordered]@{ inventory = $inventory; slot = $slot; log = $safeLog }) 22 }) -FailureMessage "Safe provider-autoload skip still left a live service"
    }
    finally {
        Stop-RaiosVmForce -Vm $safeVm
        Remove-RunImages -Vm $safeVm
        Remove-Item -LiteralPath $safePersist -Force -ErrorAction SilentlyContinue
    }
}

function Assert-Boot2LoadArtifactByHash {
    param(
        [string]$ArtifactSha,
        [int]$Port
    )
    # M8D-2 authority flip: a FRESH VM booted off the boot-1 persist snapshot runs ONLY
    # recovery.load_artifact_by_hash. A wrong (well-formed but unknown) hash denies
    # artifact_not_in_local_store and cannot answer; the correct boot-1 artifact hash drives
    # the FULL M6 gate to RE-INSTATE the RAM-only current-boot service, appends the durable
    # raios.recovery_load.v0 audit AFTER the gate, and reports cross_reboot_proven=true. It
    # re-instates an already-attested LOCAL artifact only — grants NOTHING NEW.
    $prefix = "boot2-loadhash"
    $loadVm = $null
    $loadPersist = Join-Path $RunDir "persist-loadhash.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $loadPersist -Force
    try {
        # Probation skips automatic provider autoload but intentionally remains a
        # writable recovery posture, so the explicit load-by-hash proof stays real.
        Invoke-PersistTool -Arguments @("--image", $loadPersist, "--seed-bootctl", "pending") | Out-Null
        $loadVm = Start-RaiosVm -Label "loadhash" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -QmpPort 0 -PersistPath $loadPersist
        $null = Get-ProviderAutoloadMarker

        # (1) WRONG-HASH negative FIRST (nothing loaded yet): a well-formed but unknown hash
        # denies artifact_not_in_local_store, reads no blob, and loads/starts nothing.
        $wrongHash = "sha256:2222222222222222222222222222222222222222222222222222222222222222"
        Send-AgentCommandTagged -Prefix $prefix -Command "agent recovery.load_artifact_by_hash $wrongHash" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash" -Name "wrong-hash-load"
        $wrong = Get-LastAgentResponseJson -Method "recovery.load_artifact_by_hash"
        $w = $wrong.body.result
        $wrongOk = (
            $w.schema -eq "raios.recovery_load.v0" -and
            $w.method -eq "recovery.load_artifact_by_hash" -and
            $w.status -eq "capability_denied" -and
            $w.reason -eq "artifact_not_in_local_store" -and
            $w.requested_hash_valid -eq $true -and
            $w.record_found -eq $false -and
            $w.reverify_attempted -eq $false -and
            $w.authorizes_load -eq $false -and
            $w.cross_reboot_proven -eq $false -and
            $w.service_loaded -eq $false -and
            $w.durable_write_attempted -eq $false -and
            $w.mutates_live_state -eq $false -and
            $w.owner_sealed -eq $false -and
            $w.persistence_claimed -eq $false
        )
        Assert-ReportPredicate -Prefix $prefix -Name "wrong-hash-not-in-local-store" -Expected "recovery.load_artifact_by_hash <unknown hash> denies artifact_not_in_local_store and grants nothing" -Passed $wrongOk -Actual $(if ($wrongOk) { "denied artifact_not_in_local_store" } else { Convert-CompactJson $w 10 }) -FailureMessage "Expected recovery.load_artifact_by_hash <unknown> to deny artifact_not_in_local_store"

        $wrongStartOffset = Get-SerialLogOffset
        Send-AgentCommandTagged -Prefix $prefix -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "wrong-hash-service-start"
        $afterWrong = (Get-SerialLogContent -Path $SerialLog).Substring([int]$wrongStartOffset)
        $wrongNoAnswer = -not $afterWrong.Contains("WASM_GUEST_LOG echo counter=")
        Assert-ReportPredicate -Prefix $prefix -Name "wrong-hash-no-service-answer" -Expected "an unknown-hash denial leaves the service unable to answer" -Passed $wrongNoAnswer -Actual $(if ($wrongNoAnswer) { "no_guest_log_after_offset:$wrongStartOffset" } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Unknown-hash denial still allowed a service answer"

        # (2) POSITIVE: the correct boot-1 artifact hash re-instates the service through the
        # FULL M6 gate and durably audits the reinstatement.
        $loadOffset = Get-SerialLogOffset
        Send-AgentCommandTagged -Prefix $prefix -Command "agent recovery.load_artifact_by_hash $ArtifactSha" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash" -Name "correct-hash-load"
        $load = Get-LastAgentResponseJson -Method "recovery.load_artifact_by_hash"
        $l = $load.body.result
        $loadOk = (
            $l.schema -eq "raios.recovery_load.v0" -and
            $l.method -eq "recovery.load_artifact_by_hash" -and
            $l.status -eq "reinstated_current_boot" -and
            $l.record_found -eq $true -and
            $l.reverify_attempted -eq $true -and
            $l.reverified -eq $true -and
            $l.reconstructed_wasm_valid -eq $true -and
            $l.load_granted -eq $true -and
            $l.service_loaded -eq $true -and
            $l.service_started -eq $true -and
            $l.start_run_outcome -eq "success" -and
            $l.authorizes_load -eq $true -and
            $l.cross_reboot_proven -eq $true -and
            $l.durable_write_attempted -eq $true -and
            $l.durable_append -eq "appended" -and
            $l.durable_append_performed -eq $true -and
            $l.owner_sealed -eq $false -and
            $l.persistence_claimed -eq $false -and
            $l.grants_new_capability -eq $false -and
            $l.promotion_authority_is_placeholder -eq $true -and
            $l.accepts_external_bytes -eq $false -and
            $l.accepts_url -eq $false -and
            $l.fetches -eq $false
        )
        Assert-ReportPredicate -Prefix $prefix -Name "correct-hash-reinstates-service" -Expected "recovery.load_artifact_by_hash <boot1 hash> re-verifies + M6-gate loads/starts the service (cross_reboot_proven=true) and appends the durable reinstatement audit; grants nothing new" -Passed $loadOk -Actual $(if ($loadOk) { "reinstated cross_reboot_proven=true durable_append=$($l.durable_append)" } else { Convert-CompactJson $l 12 }) -FailureMessage "Expected recovery.load_artifact_by_hash <boot1 hash> to re-instate the service via the full M6 gate"

        # (3) The recovery reinstatement itself ran the guest live (one
        # WASM_GUEST_LOG in the load window; the load response already reported
        # service_started/success). Post-B1.2b a subsequent SERIAL service.start
        # cannot re-run the granted candidate — it is denied
        # physical_approval_preview_missing while the recovery-reinstated service
        # stays running at run_count=1 with no second guest run.
        $afterLoad = (Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)
        $ranLiveOnReinstate = ([regex]::Matches($afterLoad, [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count -eq 1 -and
            $l.service_started -eq $true -and $l.start_run_outcome -eq "success"
        $startOffset = Get-SerialLogOffset
        Send-AgentCommandTagged -Prefix $prefix -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "reinstated-service-start"
        $start = Get-LastAgentResponseJson -Method "service.start"
        $startResult = $start.body.result
        $afterStart = (Get-SerialLogContent -Path $SerialLog).Substring([int]$startOffset)
        $answers = (
            $ranLiveOnReinstate -and
            $start.t -eq "response" -and
            $startResult.loaded -eq $true -and $startResult.running -eq $true -and
            [int]$startResult.run_count -eq 1 -and
            $startResult.code -eq "capability_denied" -and
            $startResult.reason -eq "physical_approval_preview_missing" -and
            $startResult.last_run_evidence.run_outcome -eq "success" -and
            $startResult.last_run_evidence.log_line_emitted -eq $true -and
            -not $afterStart.Contains("WASM_GUEST_LOG echo counter=")
        )
        Assert-ReportPredicate -Prefix $prefix -Name "reinstated-service-answers-live" -Expected "recovery reinstatement runs the guest live once; a later serial service.start is denied physical_approval_preview_missing and does not re-run the still-running service" -Passed $answers -Actual $(if ($answers) { "reinstated live once; serial restart gated" } else { Convert-CompactJson $startResult 8 }) -FailureMessage "Expected the hash-reinstated service to answer live once and the serial restart to stay gated"
    }
    finally {
        Stop-RaiosVmForce -Vm $loadVm
        Remove-RunImages -Vm $loadVm
        Remove-Item -LiteralPath $loadPersist -Force -ErrorAction SilentlyContinue
    }
}

function Assert-LoadByHashTamperDeniedChild {
    param([int]$Port)
    # Reuse a --tamper-persist-record persist image, but select the record by its NOW
    # MISMATCHING (tampered) hash: the exact-hash select FINDS the record, yet the FULL
    # reverify denies because the blob payload no longer binds the record's claimed
    # artifact_sha256. This proves the load gates on the FULL reverify path (not on
    # hash-selection alone), re-instates nothing, writes NOTHING durable, and cannot answer.
    $prefix = "boot2-loadhash-tamper"
    $tamperVm = $null
    $tamperPersist = Join-Path $RunDir "persist-loadhash-tamper.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $tamperPersist -Force
    try {
        Invoke-PersistTool -Arguments @("--tamper-persist-record", $tamperPersist) | Out-Null
        Invoke-PersistTool -Arguments @("--image", $tamperPersist, "--seed-bootctl", "pending") | Out-Null
        $inspection = Get-PersistInspection -Path $tamperPersist
        $records = @($inspection.artifact_persist_records)
        $tamperedSha = if ($records.Count -gt 0) { [string]$records[0].artifact_sha256 } else { "" }
        $mismatches = if ($records.Count -gt 0) { @($records[0].binding_mismatches) } else { @() }
        $landed = ($mismatches -contains "artifact_sha256") -and ($tamperedSha -ne "")
        Assert-ReportPredicate -Prefix $prefix -Name "tamper-landed" -Expected "artifact_sha256 mismatch visible in inspect-json with a selectable tampered hash" -Passed $landed -Actual $(Convert-CompactJson $records 10) -FailureMessage "load-by-hash tamper did not land"

        $tamperVm = Start-RaiosVm -Label "loadhash-tamper" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -QmpPort 0 -PersistPath $tamperPersist
        $null = Get-ProviderAutoloadMarker
        Send-AgentCommandTagged -Prefix $prefix -Command "agent recovery.load_artifact_by_hash $tamperedSha" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash" -Name "tamper-load"
        $load = Get-LastAgentResponseJson -Method "recovery.load_artifact_by_hash"
        $t = $load.body.result
        $deniedOk = (
            $t.schema -eq "raios.recovery_load.v0" -and
            $t.method -eq "recovery.load_artifact_by_hash" -and
            $t.status -eq "capability_denied" -and
            $t.record_found -eq $true -and
            $t.reverify_attempted -eq $true -and
            $t.reverified -eq $false -and
            ([string]$t.reverify_reason -match "sha_mismatch|hash_mismatch") -and
            $t.service_loaded -eq $false -and
            $t.authorizes_load -eq $false -and
            $t.cross_reboot_proven -eq $false -and
            $t.durable_write_attempted -eq $false -and
            $t.load_still_denied -eq $true -and
            $t.owner_sealed -eq $false -and
            $t.persistence_claimed -eq $false
        )
        Assert-ReportPredicate -Prefix $prefix -Name "tamper-reverify-denied" -Expected "load-by-hash of a tampered-hash record is FOUND but the full reverify denies (sha mismatch); nothing loaded, nothing written durable" -Passed $deniedOk -Actual $(if ($deniedOk) { "denied $([string]$t.reverify_reason)" } else { Convert-CompactJson $t 12 }) -FailureMessage "Expected load-by-hash of a tampered record to deny via full reverify"

        $startOffset = Get-SerialLogOffset
        Send-AgentCommandTagged -Prefix $prefix -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "tamper-service-start"
        $afterStart = (Get-SerialLogContent -Path $SerialLog).Substring([int]$startOffset)
        $noAnswer = -not $afterStart.Contains("WASM_GUEST_LOG echo counter=")
        Assert-ReportPredicate -Prefix $prefix -Name "tamper-no-service-answer" -Expected "a reverify-denied load leaves the service unable to answer" -Passed $noAnswer -Actual $(if ($noAnswer) { "no_guest_log_after_offset:$startOffset" } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Tampered load-by-hash denial still allowed a service answer"
    }
    finally {
        Stop-RaiosVmForce -Vm $tamperVm
        Remove-RunImages -Vm $tamperVm
        Remove-Item -LiteralPath $tamperPersist -Force -ErrorAction SilentlyContinue
    }
}

function Assert-MemoryTornReclogChild {
    param(
        [object]$Boot1MemoryScan,
        [int]$Port
    )
    $prefix = "boot2-mem-torn"
    $tornVm = $null
    $tornPersist = Join-Path $RunDir "persist-memory-torn.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $tornPersist -Force
    try {
        Invoke-PersistTool -Arguments @("--corrupt-memory-frame", $tornPersist) | Out-Null
        $tornVm = Start-RaiosVm -Label "memory-torn" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -PersistPath $tornPersist

        Send-AgentCommandTagged -Prefix $prefix -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "record-log-scan"
        $scanResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
        $scan = $scanResponse.body.result
        $reclogTorn = (
            $scan.status -ne "valid" -or
            [int64]$scan.count -lt [int64]$Boot1MemoryScan.CountFinal
        )
        Assert-ReportPredicate -Prefix $prefix -Name "reclog-torn" -Expected "corrupted tail frame is not accepted as a fully valid boot-1 chain" -Passed $reclogTorn -Actual $(if ($reclogTorn) { "status=$($scan.status) count=$($scan.count)" } else { Convert-CompactJson $scan 10 }) -FailureMessage "Torn memory child accepted the corrupted RECLOG tail as valid"

        Send-AgentCommandTagged -Prefix $prefix -Command "agent memory.context" -ExpectedMarker "RAIOS_AGENT_END memory.context" -Name "memory-context"
        $contextResponse = Get-LastAgentResponseJson -Method "memory.context"
        $context = $contextResponse
        $vmStillAnswers = (
            $scanResponse.t -eq "response" -and
            $contextResponse.schema -eq "raios.evidence_response.v1" -and
            $contextResponse.family -eq "memory.context" -and
            $null -ne $context
        )
        Assert-ReportPredicate -Prefix $prefix -Name "vm-still-answers" -Expected "child answers both durable.record_log_scan and memory.context after RECLOG tail corruption" -Passed $vmStillAnswers -Actual $(if ($vmStillAnswers) { "answered" } else { "scan=$($scanResponse.t) context=$($contextResponse.schema)/$($contextResponse.family)" }) -FailureMessage "Torn memory child stopped answering after scan/context"

        $serial = Get-SerialLogContent -Path $SerialLog
        $contextEnded = $serial.LastIndexOf("RAIOS_AGENT_END memory.context", [System.StringComparison]::Ordinal) -ge 0
        Assert-ReportPredicate -Prefix $prefix -Name "broker-no-panic" -Expected "memory.context returned RAIOS_AGENT_END" -Passed $contextEnded -Actual $(if ($contextEnded) { "RAIOS_AGENT_END memory.context" } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Torn memory child broker did not return a complete memory.context response"
    }
    finally {
        Stop-RaiosVmForce -Vm $tornVm
        Remove-RunImages -Vm $tornVm
        Remove-Item -LiteralPath $tornPersist -Force -ErrorAction SilentlyContinue
    }
}

function Merge-SerialLogs {
    $merged = Join-Path $RunDir "serial-merged.log"
    Remove-Item -LiteralPath $merged -Force -ErrorAction SilentlyContinue
    $logs = @(Get-ChildItem -LiteralPath $RunDir -Filter "serial-*.log" -ErrorAction SilentlyContinue | Sort-Object Name)
    foreach ($log in $logs) {
        Add-Content -LiteralPath $merged -Encoding UTF8 -Value "===== $($log.Name) ====="
        if (Test-Path -LiteralPath $log.FullName) {
            Add-Content -LiteralPath $merged -Encoding UTF8 -Value (Get-Content -Raw -LiteralPath $log.FullName)
        }
    }
    $script:SerialLog = $merged
}

New-Item -ItemType Directory -Force -Path $RunDir | Out-Null

$Boot1PersistSnapshot = Join-Path $RunDir "persist-after-boot1.img"
$boot1Vm = $null
$boot2Vm = $null
$boot3Vm = $null
$boot1MemoryScan = $null
$activation = $null
$install = $null
$rollback = $null

try {
    $ResolvedArtifact = Join-Path $RepoRoot "seed-kernel\artifacts\svc.demo.echo.wasm"
    if (-not (Test-Path -LiteralPath $ResolvedArtifact)) {
        throw "Candidate artifact missing: $ResolvedArtifact"
    }

    $env:CARGO_HOME = (Resolve-Path (Join-Path $RepoRoot ".cargo-home")).Path
    $env:CARGO_TARGET_DIR = Join-Path $RepoRoot "target"
    # cargo writes progress to stderr; the report helper's stderr redirect turns
    # that into a terminating NativeCommandError under Stop preference. Run it
    # plainly like shadow-vm-smoke.ps1 does and gate on the exit code only.
    & cargo build --locked -p ota-tools --bin dev-promotion-signer
    $signerBuildExit = $LASTEXITCODE
    Assert-ReportPredicate -Prefix "boot1" -Name "rust-signer-built" -Expected "cargo build --locked -p ota-tools --bin dev-promotion-signer exits 0" -Passed ($signerBuildExit -eq 0) -Actual $(if ($signerBuildExit -eq 0) { "built" } else { "exit=$signerBuildExit" }) -FailureMessage "dev-promotion-signer build failed"

    if ($Image) {
        throw "persistence-reboot requires a per-run temporary pin-bearing image; do not pass -Image"
    }
    $fixtureWrapper = Join-Path $PSScriptRoot "net8-w7-tls-fixture.ps1"
    $null = & $fixtureWrapper -Action Start -ArtifactPath $ResolvedArtifact -ReadyPath $FixtureReadyPath -ResultPath $FixtureResultPath -ModePath $FixtureModePath
    $fixtureDeadline = [DateTime]::UtcNow.AddSeconds(60)
    while (-not (Test-Path -LiteralPath $FixtureReadyPath -PathType Leaf) -and [DateTime]::UtcNow -lt $fixtureDeadline) {
        Start-Sleep -Milliseconds 50
    }
    if (-not (Test-Path -LiteralPath $FixtureReadyPath -PathType Leaf)) {
        throw "NET-8 TLS fixture did not publish its atomic ready file"
    }
    $ready = Get-Content -LiteralPath $FixtureReadyPath -Raw | ConvertFrom-Json
    if ($ready.schema -ne "raios.net8_w7_tls_fixture_ready.v0" -or
        $ready.sni -ne "w7.test.raios" -or [int]$ready.host_port -ne 8443 -or
        [int]$ready.artifact_length -ne 4205 -or
        $ready.artifact_sha256 -ne "f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2" -or
        $ready.path -ne "/raios/cas/sha256/f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2" -or
        $ready.spki_sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "NET-8 TLS fixture ready file failed validation: $(Convert-CompactJson $ready 10)"
    }
    $pinEnvName = "RAIOS_NET8_W7_PIN_$PID"
    [Environment]::SetEnvironmentVariable($pinEnvName, [string]$ready.spki_sha256, "Process")
    $ResolvedImage = Join-Path $RunDir "raios-stage0-persistence-reboot-pin.img"
    $TempImage = $true
    & $PackageScript -Profile release -Image $ResolvedImage -UseTempEsp -EmbedNet8W7SpkiPinFromEnv -Net8W7SpkiPinEnvVar $pinEnvName
    [Environment]::SetEnvironmentVariable($pinEnvName, $null, "Process")
    if ($LASTEXITCODE -ne 0) {
        throw "Pin-bearing image packaging failed with exit code $LASTEXITCODE"
    }

    $PersistDiskImage = Join-Path $RunDir "kept-persist.img"
    $builderResult = Invoke-NativeCommandForReport -FilePath "python" -Arguments @($Builder, "--self-check", "--seed-bootctl", "valid-a", $PersistDiskImage)
    Assert-ReportPredicate -Prefix "boot1" -Name "kept-persist-disk-built" -Expected "self-check GPT persist disk with valid-a bootctl" -Passed ($builderResult.ExitCode -eq 0) -Actual $(if ($builderResult.ExitCode -eq 0) { "built" } else { (@($builderResult.Output) + @($builderResult.Error)) -join [Environment]::NewLine }) -FailureMessage "Persist disk build failed"
    $PersistDiskImage = Assert-PersistDiskPathSafe -Path $PersistDiskImage
    $initialInspection = Get-PersistInspection -Path $PersistDiskImage
    $initialOk = (
        [bool]$initialInspection.gpt_header_valid -and
        [bool]$initialInspection.gpt_crc_checked -and
        [bool]$initialInspection.data_superblock_valid -and
        $initialInspection.bootctl_read.decision.posture -eq "Normal" -and
        [int]$initialInspection.reclog_scan.count -eq 0
    )
    Assert-ReportPredicate -Prefix "boot1" -Name "kept-persist-disk-empty-normal" -Expected "valid GPT, Normal bootctl, empty RECLOG" -Passed $initialOk -Actual $(Convert-CompactJson $initialInspection 8) -FailureMessage "Initial kept persist disk was not empty Normal posture"

    $HardwareProfile = New-HardwareProfile -Nic "e1000" -ScratchDrive $true -AuditRollbackTargetDrive $true -PersistDrive $true

    $boot1Vm = Start-RaiosVm -Label "boot1" -PredicatePrefix "boot1" -Port $SerialTcpPort -MonitorPort ($SerialTcpPort + 100) -QmpPort ($SerialTcpPort + 200) -NetworkBoot -PersistPath $PersistDiskImage
    $activation = @(Invoke-W7M6PhysicalActivation)[-1]
    $install = @(Invoke-SignedGrantedCandidateInstall -Activation $activation -NamePrefix "boot1")[-1]
    Assert-Boot1ActivationAndInstall -Activation $activation -Install $install
    $boot1MemoryScan = Invoke-Boot1DurableMemoryWrites -Prefix "boot1"
    Stop-RaiosVmCleanly -Vm $boot1Vm -Name "boot1"

    $postBoot1Inspection = Get-PersistInspection -Path $PersistDiskImage
    $postBoot1Records = @($postBoot1Inspection.artifact_persist_records)
    Assert-Boot1HostProvenance -Inspection $postBoot1Inspection -Activation $activation -Install $install
    Copy-Item -LiteralPath $PersistDiskImage -Destination $Boot1PersistSnapshot -Force

    $boot2Vm = Start-RaiosVm -Label "boot2" -PredicatePrefix "boot2" -Port ($SerialTcpPort + 1) -MonitorPort ($SerialTcpPort + 101) -QmpPort 0 -PersistPath $PersistDiskImage
    $null = Assert-Boot2Autoload -Activation $activation -Install $install -Boot1Inspection $postBoot1Inspection
    Assert-Boot2MemorySurvives -Prefix "boot2" -Boot1MemoryScan $boot1MemoryScan
    $rollback = Assert-Boot2Rollback -Activation $activation -Install $install -Boot1MemoryScan $boot1MemoryScan
    Stop-RaiosVmCleanly -Vm $boot2Vm -Name "boot2"
    $postBoot2Inspection = Get-PersistInspection -Path $PersistDiskImage
    Assert-Boot2HostChain -Boot1Inspection $postBoot1Inspection -Boot2Inspection $postBoot2Inspection -Rollback $rollback

    $boot3Vm = Start-RaiosVm -Label "boot3" -PredicatePrefix "boot3" -Port ($SerialTcpPort + 2) -MonitorPort ($SerialTcpPort + 102) -QmpPort 0 -PersistPath $PersistDiskImage
    $boot3State = Assert-Boot3RollbackState -Activation $activation -Rollback $rollback
    Stop-RaiosVmCleanly -Vm $boot3Vm -Name "boot3"
    Assert-Boot3HostChain -Inspection (Get-PersistInspection -Path $PersistDiskImage) -Boot3State $boot3State -Rollback $rollback

    Assert-RepromotionDeniedChild -Label "corrupt-artstor-blob" -MutationFlag "--corrupt-artstor-blob" -ExpectedMismatch "artstor_blob_frame_sha256" -ExpectedReason "blob_hash_mismatch" -Port ($SerialTcpPort + 10)
    # The merged resolver rejects a tampered artifact-persist link before reverify,
    # so its exact marker is not_installed/no_w6_authorized_install (plan gap).
    Assert-RepromotionDeniedChild -Label "tamper-persist-record" -MutationFlag "--tamper-persist-record" -ExpectedMismatch "artifact_sha256" -ExpectedReason "no_w6_authorized_install" -Port ($SerialTcpPort + 11)
    Assert-SafeChild -Port ($SerialTcpPort + 12)

    # M8D-2 recovery.load_artifact_by_hash authority flip: a fresh boot off the boot-1
    # snapshot re-instates the persisted service by its content hash (wrong hash denies +
    # no answer; correct hash re-instates + answers live), and a tampered-hash record is
    # found but reverify-denied (no load, no answer).
    Assert-Boot2LoadArtifactByHash -ArtifactSha ([string]$postBoot1Records[0].artifact_sha256) -Port ($SerialTcpPort + 13)
    Assert-LoadByHashTamperDeniedChild -Port ($SerialTcpPort + 14)
    Assert-MemoryTornReclogChild -Boot1MemoryScan $boot1MemoryScan -Port ($SerialTcpPort + 15)

    $Result = "passed"
}
catch {
    $Failures.Add($_.Exception.Message) | Out-Null
    # Print it here: the rethrow surfaces only after the finally block, where a
    # second failure can replace it. The first failure is the one worth reading.
    Write-Host "PRIMARY FAILURE: $($_.Exception.Message)"
    throw
}
finally {
    Close-SerialTcpConnection
    Stop-RaiosVmForce -Vm $boot1Vm
    Stop-RaiosVmForce -Vm $boot2Vm
    Stop-RaiosVmForce -Vm $boot3Vm
    if (Test-Path -LiteralPath $FixtureReadyPath -PathType Leaf) {
        try {
            $fixturePid = [int](Get-Content -LiteralPath $FixtureReadyPath -Raw | ConvertFrom-Json).process_id
            & (Join-Path $PSScriptRoot "net8-w7-tls-fixture.ps1") -Action Stop -ProcessId $fixturePid
        }
        catch {
        }
    }
    [Environment]::SetEnvironmentVariable("RAIOS_NET8_W7_PIN_$PID", $null, "Process")
    Merge-SerialLogs

    # A throw in here would REPLACE the exception that is already unwinding and
    # the real failure would never be printed. shadow-vm-smoke.ps1 has guarded
    # this since forever; this script had not, so every failure arrived disguised
    # as a report-write error.
    try {
        Write-Report `
            -FinalResult $Result `
            -ResolvedImage $ResolvedImage `
            -ResolvedArtifact $ResolvedArtifact `
            -ResolvedManifest $ResolvedManifest `
            -QemuArgList $QemuArgList `
            -HardwareProfile $HardwareProfile `
            -StartedAt $StartedAt
    }
    catch {
        $Result = "failed"
        Write-Host "report write FAILED: $($_.Exception.Message)"
    }

    if ($Result -eq "passed" -and -not $KeepRunDir) {
        Remove-Item -LiteralPath $RunDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "persistence reboot result: $Result"
    Write-Host "report: $ReportPath"
    Write-Host "report sha256: $ReportHashPath"
    Write-Host "run dir: $RunDir"
}
