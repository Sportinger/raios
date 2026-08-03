param(
    [int]$SerialTcpPort = 4665,
    [string]$Image = "",
    [string]$ReportDir = "$PSScriptRoot\..\release\vm-reports",
    [int]$TimeoutSeconds = 180,
    [int]$SerialWriteChunkSize = 256,
    [int]$SerialWriteDelayMilliseconds = 0,
    [switch]$KeepRunDir,
    [switch]$ProgramPersistence,
    [switch]$GrantProjectionOnly,
    [switch]$RollbackGrantDeltaOnly,
    [switch]$RollbackBoundaryOnly,
    [ValidateSet("all", "intent", "revoke", "commit")]
    [string]$RollbackBoundaryPhase = "all"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"
$PackageScript = Join-Path $RepoRoot "scripts\package-stage0.ps1"
$Builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
$RollbackFixtureMode = $RollbackGrantDeltaOnly -or $RollbackBoundaryOnly
$RunKind = if ($RollbackBoundaryOnly) { "rollback-boundaries" } elseif ($RollbackGrantDeltaOnly) { "rollback-grant-delta" } elseif ($GrantProjectionOnly) { "grant-reboot" } else { "persistence-reboot" }
$RunId = "shadow-$RunKind-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
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
$Profile = $RunKind
$ResolvedImage = $null
$ScratchImage = $null
$AuditRollbackTargetImage = $null
$PersistDiskImage = $null
$InitialRollbackFixtureSnapshot = $null
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
    # A failure dump must never abort the run: if the object holds a value
    # ConvertTo-Json cannot render, fall back per key so the real predicate
    # failure still surfaces with as much data as possible.
    try {
        return ($Value | ConvertTo-Json -Compress -Depth $Depth)
    }
    catch {
        if ($Value -is [System.Collections.IDictionary]) {
            $parts = New-Object System.Collections.Generic.List[string]
            foreach ($key in $Value.Keys) {
                $entry = $Value[$key]
                try {
                    $parts.Add('"' + $key + '":' + ($entry | ConvertTo-Json -Compress -Depth $Depth))
                }
                catch {
                    $parts.Add('"' + $key + '":"UNSERIALIZABLE:' + $entry.GetType().FullName + '"')
                }
            }
            return "{" + ($parts -join ",") + "}"
        }
        return "UNSERIALIZABLE_DUMP: $($_.Exception.Message) :: $($Value.GetType().FullName)"
    }
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

function Assert-RollbackMutationReclogUnchanged {
    param(
        [string]$Prefix,
        [string]$ServiceId,
        [object]$BeforeInspection,
        [object]$AfterInspection
    )

    # Serial output cannot prove that a storage write did not happen.  Compare
    # the host-inspected, immutable RECLOG frames themselves: the service-scoped
    # rollback transaction multiset must be identical and no new rollback frame
    # of any service may appear after the denied command/boot boundary.
    $rollbackPredicates = @(
        "wasm_import.rollback_intent.v1",
        "wasm_import.revoked.v1",
        "wasm_import.rollback_commit.v1"
    )
    $beforeFrames = @($BeforeInspection.reclog_frames | Sort-Object seq)
    $afterFrames = @($AfterInspection.reclog_frames | Sort-Object seq)
    $beforeServiceFrames = @($beforeFrames | Where-Object {
        $_.payload_json.predicate -in $rollbackPredicates -and
        ($_.payload_json.entity -eq $ServiceId -or $_.payload_json.value.service_id -eq $ServiceId)
    })
    $afterServiceFrames = @($afterFrames | Where-Object {
        $_.payload_json.predicate -in $rollbackPredicates -and
        ($_.payload_json.entity -eq $ServiceId -or $_.payload_json.value.service_id -eq $ServiceId)
    })
    $beforeFrameIdentity = @($beforeServiceFrames | ForEach-Object {
        "{0}|{1}|{2}|{3}|{4}" -f $_.seq, $_.frame_sha256, $_.payload_sha256,
            $_.payload_json.predicate, $_.payload_json.value.transaction_id
    } | Sort-Object)
    $afterFrameIdentity = @($afterServiceFrames | ForEach-Object {
        "{0}|{1}|{2}|{3}|{4}" -f $_.seq, $_.frame_sha256, $_.payload_sha256,
            $_.payload_json.predicate, $_.payload_json.value.transaction_id
    } | Sort-Object)
    $serviceMultisetUnchanged = $beforeFrameIdentity.Count -eq $afterFrameIdentity.Count -and
        @(Compare-Object -ReferenceObject $beforeFrameIdentity -DifferenceObject $afterFrameIdentity -SyncWindow 0).Count -eq 0
    $prefixUnchanged = $afterFrames.Count -ge $beforeFrames.Count
    for ($index = 0; $prefixUnchanged -and $index -lt $beforeFrames.Count; $index++) {
        $prefixUnchanged = [int64]$beforeFrames[$index].seq -eq [int64]$afterFrames[$index].seq -and
            $beforeFrames[$index].frame_sha256 -eq $afterFrames[$index].frame_sha256 -and
            $beforeFrames[$index].payload_sha256 -eq $afterFrames[$index].payload_sha256
    }
    $newRollbackFrames = @($afterFrames | Where-Object {
        [int64]$_.seq -gt [int64]$BeforeInspection.reclog_scan.tail_seq -and
        $_.payload_json.predicate -in $rollbackPredicates
    })
    $durableNoMutation = $BeforeInspection.reclog_scan.status -eq "valid" -and
        $AfterInspection.reclog_scan.status -eq "valid" -and
        $prefixUnchanged -and $serviceMultisetUnchanged -and $newRollbackFrames.Count -eq 0
    $actual = [ordered]@{
        before_scan = $BeforeInspection.reclog_scan
        after_scan = $AfterInspection.reclog_scan
        service_id = $ServiceId
        before_service_rollback_frame_count = $beforeServiceFrames.Count
        after_service_rollback_frame_count = $afterServiceFrames.Count
        service_rollback_frame_multiset_unchanged = $serviceMultisetUnchanged
        preexisting_service_rollback_frames = $beforeFrameIdentity
        post_service_rollback_frames = $afterFrameIdentity
        new_intent_revoke_commit_frames = $newRollbackFrames
        precommand_frame_prefix_unchanged = $prefixUnchanged
    }
    Assert-ReportPredicate -Prefix $Prefix -Name "durable-reclog-no-rollback-mutation" -Expected "host-safe post-denial RECLOG preserves the exact service rollback intent/revoke/commit multiset and appends zero new rollback frames" -Passed $durableNoMutation -Actual $(Convert-CompactJson $actual 20) -FailureMessage "$Prefix appended or altered a durable rollback intent/revoke/commit frame after denial"
    return [pscustomobject]@{
        Passed = $durableNoMutation
        BeforeServiceFrames = $beforeServiceFrames
        AfterServiceFrames = $afterServiceFrames
        NewRollbackFrames = $newRollbackFrames
    }
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

function Invoke-Boot1DurableRevoke {
    param([string]$Prefix)

    $offset = Get-SerialLogOffset
    Send-AgentCommandTagged -Prefix $Prefix -Command "revoke.selftest" -ExpectedMarker "RAIOS_REVOKE selftest=pass surface=env.counter_get first_call=ok revoked=1 next_call=denied host_effect_delta=0 peer_surface=ok same_instance=1 logged=1" -Name "durable-revoke"
    $tail = (Get-SerialLogContent -Path $SerialLog).Substring([int]$offset)
    $match = [regex]::Match($tail, 'RAIOS_REVOKE_DURABLE selftest=pass replay=0 invalid_projection=0 append_before_flip=1 projection=(sha256:[0-9a-f]{64}) call_attempted=1 gate=env\.counter_get denied_next=1 host_effect_delta=0')
    Assert-ReportPredicate -Prefix $Prefix -Name "durable-revoke-append-before-flip" -Expected "typed grant+revoke append through RECLOG before RAM flip; next call denied with zero effect" -Passed $match.Success -Actual $(if ($match.Success) { $match.Value } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Boot 1 durable revoke predicate failed"
    return $(if ($match.Success) { $match.Groups[1].Value } else { "" })
}

function Assert-Boot2DurableRevokeReplay {
    param(
        [string]$Prefix,
        [string]$Boot1Projection
    )

    Assert-LogContains -Name "${Prefix}:durable-revoke-projection-ready" -Needle "cap.projection " -TimeoutSeconds $TimeoutSeconds
    $bootSerial = Get-SerialLogContent -Path $SerialLog
    $bootProjection = [regex]::Match($bootSerial, 'cap\.projection (sha256:[0-9a-f]{64}) valid=1 events=([2-9][0-9]*) reason=fold_valid')
    $bootFoldOk = $bootProjection.Success -and $bootProjection.Groups[1].Value -eq $Boot1Projection
    Assert-ReportPredicate -Prefix $Prefix -Name "durable-revoke-boot-fold" -Expected "boot emits the identical validated cap.projection before service instantiation" -Passed $bootFoldOk -Actual $(if ($bootProjection.Success) { $bootProjection.Value } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Boot 2 capability projection did not replay identically"

    $offset = Get-SerialLogOffset
    Send-AgentCommandTagged -Prefix $Prefix -Command "revoke.selftest" -ExpectedMarker "RAIOS_REVOKE_DURABLE selftest=pass replay=1 invalid_projection=0 append_before_flip=1" -Name "durable-revoke-replay"
    $tail = (Get-SerialLogContent -Path $SerialLog).Substring([int]$offset)
    $replay = [regex]::Match($tail, 'RAIOS_REVOKE_DURABLE selftest=pass replay=1 invalid_projection=0 append_before_flip=1 projection=(sha256:[0-9a-f]{64}) call_attempted=1 gate=env\.counter_get denied_next=1 host_effect_delta=0')
    $replayOk = $replay.Success -and $replay.Groups[1].Value -eq $Boot1Projection
    Assert-ReportPredicate -Prefix $Prefix -Name "durable-revoke-next-call-denied" -Expected "same durable revoke refold denies the first post-reboot env.counter_get effect; delta=0" -Passed $replayOk -Actual $(if ($replay.Success) { $replay.Value } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Boot 2 durable revoke did not deny the next host call"

    Send-AgentCommandTagged -Prefix $Prefix -Command "host_import.selftest" -ExpectedMarker "RAIOS_HOSTIMPORT selftest=pass" -Name "durable-revoke-peer-still-answers"
    $peerLog = Get-SerialLogContent -Path $SerialLog
    $peerOk = $peerLog.Contains("RAIOS_HOSTIMPORT selftest=pass") -and -not $peerLog.Contains("*** KERNEL PANIC ***")
    Assert-ReportPredicate -Prefix $Prefix -Name "durable-revoke-peer-healthy" -Expected "unrelated host-import peer answers after replay denial and kernel does not panic" -Passed $peerOk -Actual $(Get-SerialLogTail -Path $SerialLog) -FailureMessage "Boot 2 durable revoke replay damaged the peer or panicked"
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
    Start-Sleep -Milliseconds 100
    if (-not (Get-Process -Id $qemuPid -ErrorAction SilentlyContinue)) {
        $earlyError = Get-Content -LiteralPath "$log.err.txt" -Raw -ErrorAction SilentlyContinue
        throw "harness_infrastructure_failure: QEMU exited before serial readiness for ${Label}: $earlyError"
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
    $newRecords = @($Install.PostReclogScan.records | Where-Object { [int64]$_.seq -gt [int64]$Install.PreReclogScan.tail_seq } | Sort-Object seq)
    $focusedFrameShape = -not $RollbackFixtureMode -or (
        $newRecords.Count -eq 5 -and
        $newRecords[0].schema -eq "raios.install_authorization.v0" -and
        $newRecords[1].schema -eq "raios.promotion_transaction.v0" -and
        $newRecords[2].schema -eq "raios.artifact_persist.v0" -and
        $newRecords[3].schema -eq "raios.memory_record.v0" -and
        $newRecords[4].schema -eq "raios.memory_record.v0" -and
        $newRecords[1].frame_sha256 -eq $Install.PromotionTransactionSha256 -and
        $newRecords[2].frame_sha256 -eq $Install.ArtifactPersistFrameSha256
    )
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
        [int64]$Install.PostReclogScan.count -eq ([int64]$Install.PreReclogScan.count + $(if ($RollbackFixtureMode) { 5 } else { 3 })) -and
        $focusedFrameShape -and
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

function Assert-RollbackSourceGrantFrames {
    param([object]$Inspection, [object]$Activation, [object]$Install)
    $frames = @($Inspection.reclog_frames | Sort-Object seq)
    $auth = @($frames | Where-Object { [int64]$_.seq -eq ([int64]$Install.Sequence - 1) })[0]
    $promote = @($frames | Where-Object { [int64]$_.seq -eq [int64]$Install.Sequence })[0]
    $persist = @($frames | Where-Object { [int64]$_.seq -eq ([int64]$Install.Sequence + 1) })[0]
    $grants = @($frames | Where-Object { [int64]$_.seq -in @(([int64]$Install.Sequence + 2), ([int64]$Install.Sequence + 3)) })
    $surfaces = @($grants | ForEach-Object { $_.payload_json.value.host_import_id } | Sort-Object)
    $bound = $grants.Count -eq 2
    foreach ($grant in $grants) {
        $bound = $bound -and $grant.payload_json.predicate -eq "wasm_import.granted.v1" -and
            $grant.payload_json.value.service_id -eq "svc.dev.granted_candidate" -and
            [int64]$grant.payload_json.value.domain_instance -eq 1 -and
            $grant.payload_json.value.binding_sha256 -eq "sha256:18c5db9cba3fe1471112505ffcb999e46cacc0ca89c10fe189865b3fa76e12fa"
    }
    $ok = $auth.payload_json.schema -eq "raios.install_authorization.v0" -and
        [int64]$auth.payload_json.install_envelope_version -eq 2 -and
        $auth.payload_json.install_candidate_sha256 -eq $Activation.CandidateSha256 -and
        $auth.payload_json.computed_grant_sha256 -eq $Activation.M6ComputedGrantSha256 -and
        $auth.payload_json.attestation_reference_sha256 -eq $Activation.M6AttestationReferenceSha256 -and
        $auth.payload_json.grant_target_schema -eq "raios.grant_target_snapshot.v1" -and
        [int64]$auth.payload_json.grant_target_count -eq 2 -and
        $auth.payload_json.grant_target_snapshot_sha256 -match '^sha256:[0-9a-f]{64}$' -and
        $promote.payload_json.schema -eq "raios.promotion_transaction.v0" -and
        $promote.payload_json.install_authorization_frame_sha256 -eq "sha256:$($auth.frame_sha256)" -and
        [int64]$promote.payload_json.install_envelope_version -eq 2 -and
        $promote.payload_json.grant_target_schema -eq $auth.payload_json.grant_target_schema -and
        [int64]$promote.payload_json.grant_target_count -eq 2 -and
        $promote.payload_json.grant_target_snapshot_sha256 -eq $auth.payload_json.grant_target_snapshot_sha256 -and
        $persist.payload_json.schema -eq "raios.artifact_persist.v0" -and
        $persist.payload_json.promotion_transaction_sha256 -eq "sha256:$($promote.frame_sha256)" -and
        $persist.payload_json.service_id -eq $auth.payload_json.service_id -and
        $persist.payload_json.artifact_sha256 -eq $auth.payload_json.install_candidate_sha256 -and
        $persist.payload_json.grant_target_schema -eq $auth.payload_json.grant_target_schema -and
        [int64]$persist.payload_json.grant_target_count -eq 2 -and
        $persist.payload_json.grant_target_snapshot_sha256 -eq $auth.payload_json.grant_target_snapshot_sha256 -and
        $persist.payload_json.artifact_sha256 -eq $Activation.CandidateSha256 -and $bound -and
        $grants[0].payload_json.value.host_import_id -eq "env.log" -and
        $grants[1].payload_json.value.host_import_id -eq "env.counter_get" -and
        $surfaces.Count -eq 2 -and $surfaces[0] -eq "env.counter_get" -and $surfaces[1] -eq "env.log"
    Assert-ReportPredicate -Prefix "boot1" -Name "source-five-frame-binding" -Expected "the five new frames carry one exact signed V2 service+artifact+grant+attestation+two-import snapshot authorization, linked promote/persist, then bound env.log and env.counter_get grants" -Passed $ok -Actual $(Convert-CompactJson ([ordered]@{ authorization = $auth; promote = $promote; persist = $persist; grants = $grants }) 22) -FailureMessage "Boot 1 source five-frame sequence was not exactly typed and V2-bound"
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

function Invoke-RollbackGrantDeltaApply {
    $before = Get-SerialLogOffset
    Send-AgentCommandTagged -Prefix "rollback" -Command "service.rollback_apply svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply" -Name "apply"
    $response = Get-LastAgentResponseJson -Method "service.rollback_apply"
    $result = $response.body.result
    $window = (Get-SerialLogContent -Path $SerialLog).Substring([int]$before)
    $markers = [regex]::Matches($window, 'RAIOS_ROLLBACK_GRANT result=accepted transaction=([0-9a-f]{64}) delta=1 intent=1 revokes=1 commit=1 ram_install=1 counter_denied=1 host_effect_delta=0 teardown_after_commit=1 target_reverified=1 rebuilt=1')
    $ok = $markers.Count -eq 1 -and
        $result.code -eq "ok" -and
        $result.reason -eq "rollback_grant_delta_committed_before_rebuild" -and
        $result.loaded -eq $true -and $result.running -eq $true -and
        $result.rollback_apply_authorized -eq $true -and
        $window.Contains("WASM_GUEST_LOG rollback target log-only")
    Assert-ReportPredicate -Prefix "rollback" -Name "delta-commit-before-rebuild" -Expected "one signed counter-grant revoke is durably committed and RAM-installed, the real counter gate denies with zero effect, then the unchanged M6 path starts the log-only predecessor" -Passed $ok -Actual $(if ($ok) { $markers[0].Value } else { Convert-CompactJson ([ordered]@{ response = $response; window = $window }) 20 }) -FailureMessage "Rollback grant delta did not commit and rebuild in the required order"

    Send-AgentCommandTagged -Prefix "rollback" -Command "host_import.selftest" -ExpectedMarker "RAIOS_HOSTIMPORT selftest=pass" -Name "peer-isolation"

    $retryBefore = Get-SerialLogOffset
    Send-AgentCommandTagged -Prefix "rollback" -Command "service.rollback_apply svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply" -Name "retry-after-commit"
    $retry = Get-LastAgentResponseJson -Method "service.rollback_apply"
    $retryWindow = (Get-SerialLogContent -Path $SerialLog).Substring([int]$retryBefore)
    $retryOk = $retry.body.result.code -eq "capability_denied" -and
        $retry.body.result.reason -in @("rollback_transaction_already_committed", "no_recorded_promotion_to_roll_back") -and
        $retry.body.result.loaded -eq $true -and $retry.body.result.running -eq $true -and
        -not $retryWindow.Contains("RAIOS_ROLLBACK_GRANT result=accepted")
    Assert-ReportPredicate -Prefix "rollback" -Name "committed-retry-idempotent" -Expected "a retry after the committed transaction cannot reverse the rollback or append another accepted transaction" -Passed $retryOk -Actual $(Convert-CompactJson $retry 16) -FailureMessage "Committed rollback retry was not idempotently refused"
    return [pscustomobject]@{ TransactionId = $markers[0].Groups[1].Value; Response = $response }
}

function Assert-RollbackGrantDeltaHostOrdering {
    param([object]$Inspection)
    $frames = @($Inspection.reclog_frames)
    $intent = @($frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.rollback_intent.v1" } | Select-Object -Last 1)[0]
    $commit = @($frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.rollback_commit.v1" } | Select-Object -Last 1)[0]
    $revokes = @($frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.revoked.v1" -and $_.payload_json.value.host_import_id -eq "env.counter_get" })
    # This host RECLOG assertion is the durable idempotence authority.
    $ordered = $Inspection.reclog_scan.status -eq "valid" -and $null -ne $intent -and $null -ne $commit -and
        $revokes.Count -eq 1 -and [int64]$intent.seq -lt [int64]$revokes[0].seq -and [int64]$revokes[0].seq -lt [int64]$commit.seq
    Assert-ReportPredicate -Prefix "rollback" -Name "host-chain-intent-revoke-commit" -Expected "host inspection sees a valid RECLOG with exactly one env.counter_get revoke strictly between intent and commit" -Passed $ordered -Actual $(Convert-CompactJson ([ordered]@{ scan = $Inspection.reclog_scan; intent = $intent; revokes = $revokes; commit = $commit }) 20) -FailureMessage "Rollback RECLOG ordering was not intent -> one counter revoke -> commit"
}

function Assert-RollbackTargetReboot {
    $null = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
    $marker = Get-ProviderAutoloadMarker -BeforeOffset (Get-SerialLogOffset)
    $log = Get-SerialLogContent -Path $SerialLog
    $ok = $marker.Result -eq "accepted" -and $marker.Phase -eq "autoloaded" -and
        $marker.Reason -eq "rollback_commit_target_autoloaded" -and
        $marker.CandidateSha256 -eq "sha256:33ea9dc8f8ecd039236673fafdffcf63e8691a1d352cffee4ddf47531cb5756c" -and
        $marker.Loaded -and $marker.Running -and $marker.RunCount -eq 1 -and
        $log.Contains("WASM_GUEST_LOG rollback target log-only") -and
        -not $log.Contains("WASM_GUEST_LOG echo counter=")
    Assert-ReportPredicate -Prefix "reboot" -Name "committed-target-autoload" -Expected "the committed exact predecessor re-verifies and emits its log after reboot while the revoked counter source does not run" -Passed $ok -Actual $(if ($ok) { $marker.Line } else { Convert-CompactJson ([ordered]@{ marker = $marker; log = $log }) 16 }) -FailureMessage "Committed rollback target did not survive reboot"
    Send-AgentCommandTagged -Prefix "reboot" -Command "host_import.selftest" -ExpectedMarker "RAIOS_HOSTIMPORT selftest=pass" -Name "peer-isolation"
}

function Assert-RollbackFixtureMutationDenied {
    param([string]$Label, [string]$MutationFlag, [int]$Port, [string]$ExpectedReason = "")
    $childPersist = Join-Path $RunDir "persist-$Label.img"
    $childVm = $null
    $childFixturePid = 0
    $baseResolvedImage = $script:ResolvedImage
    Copy-Item -LiteralPath $InitialRollbackFixtureSnapshot -Destination $childPersist -Force
    try {
        $mutation = Invoke-PersistTool -Arguments @($MutationFlag, $childPersist)
        $mutationImageHash = (Get-FileHash -LiteralPath $childPersist -Algorithm SHA256).Hash
        $outer = Get-PersistInspection -Path $childPersist
        Assert-ReportPredicate -Prefix $Label -Name "outer-chain-valid" -Expected "the semantic predecessor mutation preserves a fully valid outer RECLOG chain" -Passed ($outer.reclog_scan.status -eq "valid") -Actual $(Convert-CompactJson $outer.reclog_scan 8) -FailureMessage "$Label damaged the outer RECLOG chain"
        if ($MutationFlag -eq "--duplicate-authenticated-install") {
            $originalPayloads = @($mutation.original_payload_sha256)
            $duplicatePayloads = @($mutation.duplicate_payload_sha256)
            $payloadsMatch = $originalPayloads.Count -eq 3 -and $duplicatePayloads.Count -eq 3
            for ($index = 0; $payloadsMatch -and $index -lt 3; $index++) {
                $payloadsMatch = $originalPayloads[$index] -eq $duplicatePayloads[$index]
            }
            $mutationProof = $mutation.operation -eq "duplicate_authenticated_install" -and
                $mutation.semantic_boundary -eq "rollback_target_authenticated_order_duplicate" -and
                $mutation.signed_install_payloads_unchanged -eq $true -and
                $mutation.duplicated_install_payloads_byte_identical -eq $true -and
                $mutation.unsigned_reclog_framing_rechained -eq $true -and
                $mutation.outer_reclog_chain_valid -eq $true -and
                $mutation.post_reclog_scan.status -eq "valid" -and
                @($mutation.original_seq).Count -eq 3 -and @($mutation.duplicate_seq).Count -eq 3 -and
                $payloadsMatch
            Assert-ReportPredicate -Prefix $Label -Name "authentic-triple-byte-preserved" -Expected "one linked authorization+promotion+persist triple is payload-byte-identical while only unsigned seq/prev/frame RECLOG framing is rechained" -Passed $mutationProof -Actual $(Convert-CompactJson $mutation 14) -FailureMessage "$Label did not prove byte-preserved signed install duplication"
        }
        foreach ($path in @($FixtureReadyPath, $FixtureResultPath, "$FixtureReadyPath.stdout.log", "$FixtureReadyPath.stderr.log")) {
            Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
        }
        $fixtureWrapper = Join-Path $PSScriptRoot "net8-w7-tls-fixture.ps1"
        $null = & $fixtureWrapper -Action Start -ArtifactPath $ResolvedArtifact -ReadyPath $FixtureReadyPath -ResultPath $FixtureResultPath -ModePath $FixtureModePath
        $fixtureDeadline = [DateTime]::UtcNow.AddSeconds(60)
        while (-not (Test-Path -LiteralPath $FixtureReadyPath -PathType Leaf) -and [DateTime]::UtcNow -lt $fixtureDeadline) {
            Start-Sleep -Milliseconds 50
        }
        if (-not (Test-Path -LiteralPath $FixtureReadyPath -PathType Leaf)) {
            throw "$Label local W7 fixture did not become ready"
        }
        $childFixturePid = [int](Get-Content -LiteralPath $FixtureReadyPath -Raw | ConvertFrom-Json).process_id
        $childReady = Get-Content -LiteralPath $FixtureReadyPath -Raw | ConvertFrom-Json
        if ($childReady.spki_sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "harness_infrastructure_failure: $Label local fixture published an invalid SPKI"
        }
        $childPinEnvName = "RAIOS_NET8_W7_PIN_CHILD_$PID"
        [Environment]::SetEnvironmentVariable($childPinEnvName, [string]$childReady.spki_sha256, "Process")
        $childResolvedImage = Join-Path $RunDir "raios-stage0-$Label-pin.img"
        & $PackageScript -Profile release -Image $childResolvedImage -UseTempEsp -EmbedNet8W7SpkiPinFromEnv -Net8W7SpkiPinEnvVar $childPinEnvName
        [Environment]::SetEnvironmentVariable($childPinEnvName, $null, "Process")
        if ($LASTEXITCODE -ne 0) {
            throw "harness_infrastructure_failure: $Label pin-bearing stage packaging failed"
        }
        if ((Get-FileHash -LiteralPath $childPersist -Algorithm SHA256).Hash -ne $mutationImageHash) {
            throw "harness_infrastructure_failure: $Label stage packaging changed the mutation image"
        }
        $script:ResolvedImage = $childResolvedImage
        $childVm = Start-RaiosVm -Label $Label -PredicatePrefix $Label -Port $Port -MonitorPort ($Port + 100) -QmpPort ($Port + 200) -NetworkBoot -PersistPath $childPersist
        $script:ResolvedImage = $baseResolvedImage
        $qmpProbe = [System.Net.Sockets.TcpClient]::new()
        try {
            $qmpConnect = $qmpProbe.BeginConnect("127.0.0.1", ($Port + 200), $null, $null)
            if (-not $qmpConnect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(5))) {
                throw "harness_infrastructure_failure: $Label QMP did not become ready"
            }
            $qmpProbe.EndConnect($qmpConnect)
        }
        finally {
            $qmpProbe.Dispose()
        }
        $childActivation = @(Invoke-W7M6PhysicalActivation)[-1]
        $childInstall = @(Invoke-SignedGrantedCandidateInstall -Activation $childActivation -NamePrefix $Label)[-1]
        Send-AgentCommandTagged -Prefix $Label -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "source-identity-before-denial"
        $beforeInventory = Get-LastAgentResponseJson -Method "service.inventory"
        $beforeSourceRows = @($beforeInventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" })
        $beforeSourceRow = if ($beforeSourceRows.Count -eq 1) { $beforeSourceRows[0] } else { $null }
        $beforeRollbackInspection = Get-PersistInspection -Path $childPersist
        $sourceAuthorizationBefore = @($beforeRollbackInspection.reclog_frames | Where-Object {
            [int64]$_.seq -eq ([int64]$childInstall.Sequence - 1)
        })[0]
        $sourcePersistFrameBefore = @($beforeRollbackInspection.reclog_frames | Where-Object {
            "sha256:$($_.frame_sha256)" -eq $childInstall.ArtifactPersistFrameSha256
        })[0]
        $sourcePersistBefore = @($beforeRollbackInspection.artifact_persist_records | Where-Object {
            "sha256:$($_.reclog_frame_sha256)" -eq $childInstall.ArtifactPersistFrameSha256
        })[0]
        $sourceGrantFramesBefore = @($beforeRollbackInspection.reclog_frames | Where-Object {
            $_.payload_json.predicate -eq "wasm_import.granted.v1" -and
            [int64]$_.seq -in @(([int64]$childInstall.Sequence + 2), ([int64]$childInstall.Sequence + 3))
        } | Sort-Object seq)
        $sourceLifecycleBefore = $childActivation.LastLifecycleResponse
        $sourceIdentityBefore = [ordered]@{
            service_id = "svc.dev.granted_candidate"
            artifact_sha256 = [string]$sourcePersistBefore.artifact_sha256
            artifact_persist_frame_sha256 = "sha256:$($sourcePersistFrameBefore.frame_sha256)"
            artstor_blob_frame_sha256 = [string]$sourcePersistBefore.artstor_blob_frame_sha256
            promotion_transaction_sha256 = [string]$sourcePersistBefore.promotion_transaction_sha256
            install_authorization_frame_sha256 = "sha256:$($sourceAuthorizationBefore.frame_sha256)"
            install_generation = [int64]$childInstall.Generation
            inventory_generation = [int64]$beforeSourceRow.generation
            load_generation = [int64]$sourceLifecycleBefore.generation
            load_event_id = [string]$sourceLifecycleBefore.load_event_id
            start_event_id = [string]$sourceLifecycleBefore.start_event_id
            grant_target_schema = [string]$sourcePersistFrameBefore.payload_json.grant_target_schema
            grant_target_count = [int64]$sourcePersistFrameBefore.payload_json.grant_target_count
            grant_target_snapshot_sha256 = [string]$sourcePersistFrameBefore.payload_json.grant_target_snapshot_sha256
            grant_frame_sha256 = @($sourceGrantFramesBefore | ForEach-Object { "sha256:$($_.frame_sha256)" })
            grant_payload_sha256 = @($sourceGrantFramesBefore | ForEach-Object { "sha256:$($_.payload_sha256)" })
            granted_host_imports = @($sourceGrantFramesBefore | ForEach-Object { [string]$_.payload_json.value.host_import_id } | Sort-Object)
            grant_binding_sha256 = @($sourceGrantFramesBefore | ForEach-Object { [string]$_.payload_json.value.binding_sha256 } | Sort-Object)
        }
        $sourceBeforeCaptured = $beforeSourceRows.Count -eq 1 -and
            $childInstall.CandidateSha256 -eq $childInstall.ArtifactSha256 -and
            $sourceIdentityBefore.artifact_sha256 -eq $childInstall.CandidateSha256 -and
            $sourceIdentityBefore.artifact_persist_frame_sha256 -eq $childInstall.ArtifactPersistFrameSha256 -and
            $sourceIdentityBefore.promotion_transaction_sha256 -eq $childInstall.PromotionTransactionSha256 -and
            $sourceIdentityBefore.install_generation -eq $sourceIdentityBefore.inventory_generation -and
            $sourceIdentityBefore.install_generation -eq $sourceIdentityBefore.load_generation -and
            $sourceLifecycleBefore.loaded -eq $true -and $sourceLifecycleBefore.running -eq $true -and
            -not [string]::IsNullOrEmpty($sourceIdentityBefore.load_event_id) -and
            -not [string]::IsNullOrEmpty($sourceIdentityBefore.start_event_id) -and
            $sourceIdentityBefore.grant_target_schema -eq "raios.grant_target_snapshot.v1" -and
            $sourceIdentityBefore.grant_target_snapshot_sha256 -match '^sha256:[0-9a-f]{64}$' -and
            $sourceGrantFramesBefore.Count -eq 2 -and
            $sourceIdentityBefore.granted_host_imports.Count -eq 2 -and
            $sourceIdentityBefore.granted_host_imports[0] -eq "env.counter_get" -and
            $sourceIdentityBefore.granted_host_imports[1] -eq "env.log"
        Assert-ReportPredicate -Prefix $Label -Name "source-identity-captured-before-denial" -Expected "before rollback_apply, capture the exact installed artifact, generation, load/start event identities, and signed grant projection" -Passed $sourceBeforeCaptured -Actual $(Convert-CompactJson ([ordered]@{ identity = $sourceIdentityBefore; inventory = $beforeInventory; install = $childInstall }) 20) -FailureMessage "$Label did not expose a complete exact source identity before denied rollback"
        $before = Get-SerialLogOffset
        Send-AgentCommandTagged -Prefix $Label -Command "service.rollback_apply svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply" -Name "rollback-denied"
        $response = Get-LastAgentResponseJson -Method "service.rollback_apply"
        $afterRollbackInspection = Get-PersistInspection -Path $childPersist
        $null = Assert-RollbackMutationReclogUnchanged -Prefix $Label -ServiceId "svc.dev.granted_candidate" -BeforeInspection $beforeRollbackInspection -AfterInspection $afterRollbackInspection
        $window = (Get-SerialLogContent -Path $SerialLog).Substring([int]$before)
        $reasonMatches = [string]::IsNullOrEmpty($ExpectedReason) -or
            $response.body.result.reason -eq $ExpectedReason
        $denied = $response.body.result.code -eq "capability_denied" -and
            $reasonMatches -and
            $response.body.result.loaded -eq $true -and $response.body.result.running -eq $true -and
            $response.body.result.rollback_apply_authorized -eq $false -and
            $response.body.result.rollback_verification.stopped -eq $false -and
            $response.body.result.rollback_verification.drop_clear_bytes -eq $false -and
            $response.body.result.rollback_verification.free_slot -eq $false -and
            $response.body.result.rollback_verification.remove_inventory -eq $false -and
            $response.body.result.rollback_verification.durable_unpromote_transaction.performed -eq $false -and
            -not $window.Contains("RAIOS_ROLLBACK_GRANT result=accepted") -and
            -not $window.Contains('"predicate": "wasm_import.rollback_intent.v1"') -and
            -not $window.Contains("phase=intent") -and
            -not $window.Contains("phase=revoke") -and
            -not $window.Contains("phase=commit") -and
            -not $window.Contains("teardown_after_commit=1")
        Assert-ReportPredicate -Prefix $Label -Name "deny-before-teardown-or-effect" -Expected $(if ($ExpectedReason) { "rejection reason=$ExpectedReason leaves rollback authorization, teardown, durable unpromote, and guest effect false" } else { "the mutated predecessor is rejected before rollback teardown, durable unpromote, or guest effect" }) -Passed $denied -Actual $(Convert-CompactJson ([ordered]@{ expected_reason = $ExpectedReason; response = $response; window = $window }) 18) -FailureMessage "$Label was not rejected at the expected boundary before teardown or effect"
        Send-AgentCommandTagged -Prefix $Label -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "source-live-after-denial"
        $inventory = Get-LastAgentResponseJson -Method "service.inventory"
        $sourceRows = @($inventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" })
        $sourceRow = if ($sourceRows.Count -eq 1) { $sourceRows[0] } else { $null }
        $sourceAuthorizationAfter = @($afterRollbackInspection.reclog_frames | Where-Object {
            [int64]$_.seq -eq ([int64]$childInstall.Sequence - 1)
        })[0]
        $sourcePersistFrameAfter = @($afterRollbackInspection.reclog_frames | Where-Object {
            "sha256:$($_.frame_sha256)" -eq $childInstall.ArtifactPersistFrameSha256
        })[0]
        $sourcePersistAfter = @($afterRollbackInspection.artifact_persist_records | Where-Object {
            "sha256:$($_.reclog_frame_sha256)" -eq $childInstall.ArtifactPersistFrameSha256
        })[0]
        $sourceGrantFramesAfter = @($afterRollbackInspection.reclog_frames | Where-Object {
            $_.payload_json.predicate -eq "wasm_import.granted.v1" -and
            [int64]$_.seq -in @(([int64]$childInstall.Sequence + 2), ([int64]$childInstall.Sequence + 3))
        } | Sort-Object seq)
        $sourceIdentityAfter = [ordered]@{
            service_id = [string]$response.body.result.service_id
            artifact_sha256 = [string]$sourcePersistAfter.artifact_sha256
            artifact_persist_frame_sha256 = "sha256:$($sourcePersistFrameAfter.frame_sha256)"
            artstor_blob_frame_sha256 = [string]$sourcePersistAfter.artstor_blob_frame_sha256
            promotion_transaction_sha256 = [string]$sourcePersistAfter.promotion_transaction_sha256
            install_authorization_frame_sha256 = "sha256:$($sourceAuthorizationAfter.frame_sha256)"
            install_generation = [int64]$childInstall.Generation
            inventory_generation = [int64]$sourceRow.generation
            load_generation = [int64]$response.body.result.generation
            load_event_id = [string]$response.body.result.load_event_id
            start_event_id = [string]$response.body.result.start_event_id
            grant_target_schema = [string]$sourcePersistFrameAfter.payload_json.grant_target_schema
            grant_target_count = [int64]$sourcePersistFrameAfter.payload_json.grant_target_count
            grant_target_snapshot_sha256 = [string]$sourcePersistFrameAfter.payload_json.grant_target_snapshot_sha256
            grant_frame_sha256 = @($sourceGrantFramesAfter | ForEach-Object { "sha256:$($_.frame_sha256)" })
            grant_payload_sha256 = @($sourceGrantFramesAfter | ForEach-Object { "sha256:$($_.payload_sha256)" })
            granted_host_imports = @($sourceGrantFramesAfter | ForEach-Object { [string]$_.payload_json.value.host_import_id } | Sort-Object)
            grant_binding_sha256 = @($sourceGrantFramesAfter | ForEach-Object { [string]$_.payload_json.value.binding_sha256 } | Sort-Object)
        }
        $sourceIdentityUnchanged = (Convert-CompactJson $sourceIdentityBefore 16) -eq (Convert-CompactJson $sourceIdentityAfter 16)
        $sourceLive = $sourceIdentityUnchanged -and $sourceRows.Count -eq 1 -and
            $sourceRow.service_slot_activation_active -eq $true -and $sourceRow.running -eq $true -and
            $sourceRow.last_run_outcome -eq "success" -and $response.body.result.loaded -eq $true -and
            $response.body.result.running -eq $true -and
            $response.body.result.rollback_plan.artifact_hash -eq $sourceIdentityBefore.artifact_sha256 -and
            [int64]$response.body.result.rollback_plan.generation -eq $sourceIdentityBefore.install_generation -and
            $response.body.result.rollback_plan.load_event_id -eq $sourceIdentityBefore.load_event_id
        Assert-ReportPredicate -Prefix $Label -Name "source-identity-remains-live" -Expected "after denial, the exact pre-command artifact, generation, load/start identities, and grant projection remain the one loaded/running source" -Passed $sourceLive -Actual $(Convert-CompactJson ([ordered]@{ before = $sourceIdentityBefore; after = $sourceIdentityAfter; inventory = $inventory; response = $response }) 22) -FailureMessage "$Label replaced or disturbed the exact live source identity after denial"
        Send-AgentCommandTagged -Prefix $Label -Command "host_import.selftest" -ExpectedMarker "RAIOS_HOSTIMPORT selftest=pass" -Name "peer-isolation"
        Stop-RaiosVmCleanly -Vm $childVm -Name $Label
        $childVm = $null
    }
    finally {
        $script:ResolvedImage = $baseResolvedImage
        if ($childVm) { Stop-RaiosVmForce -Vm $childVm -Name "$Label-failure" }
        if ($childFixturePid -gt 0) {
            & (Join-Path $PSScriptRoot "net8-w7-tls-fixture.ps1") -Action Stop -ProcessId $childFixturePid
        }
    }
}

function Assert-RollbackRecoveryMutationDenied {
    param([string]$Label, [string]$MutationFlag, [string]$ExpectedReason, [int]$Port)
    $childPersist = Join-Path $RunDir "persist-$Label.img"
    $childVm = $null
    Copy-Item -LiteralPath $PersistDiskImage -Destination $childPersist -Force
    try {
        $mutation = Invoke-PersistTool -Arguments @($MutationFlag, $childPersist)
        $outer = Get-PersistInspection -Path $childPersist
        $outerValid = $outer.reclog_scan.status -eq "valid" -and
            $mutation.signed_install_payloads_unchanged -eq $true -and
            $mutation.unsigned_recovery_and_reclog_resealed -eq $true
        Assert-ReportPredicate -Prefix $Label -Name "resealed-outer-chain-valid" -Expected "the storage attacker fully rechains unsigned recovery records while every signed install payload remains byte-identical" -Passed $outerValid -Actual $(Convert-CompactJson ([ordered]@{ mutation = $mutation; scan = $outer.reclog_scan }) 14) -FailureMessage "$Label did not preserve the signed installs and valid outer RECLOG"
        $recoverySourceIdentityBefore = [ordered]@{
            artifact_persist = @($outer.artifact_persist_records | Where-Object {
                $_.service_id -eq "svc.dev.granted_candidate"
            } | Sort-Object reclog_frame_sha256 | ForEach-Object {
                "{0}|{1}|{2}|{3}|{4}|{5}" -f $_.reclog_frame_sha256, $_.artifact_sha256,
                    $_.promotion_transaction_sha256, $_.grant_target_schema,
                    $_.grant_target_count, $_.grant_target_snapshot_sha256
            })
            install_authorization = @($outer.reclog_frames | Where-Object {
                $_.payload_json.schema -eq "raios.install_authorization.v0" -and
                $_.payload_json.service_id -eq "svc.dev.granted_candidate"
            } | Sort-Object seq | ForEach-Object {
                "{0}|{1}|{2}|{3}|{4}|{5}" -f $_.seq, $_.frame_sha256,
                    $_.payload_json.install_candidate_sha256, $_.payload_json.generation,
                    $_.payload_json.grant_target_count, $_.payload_json.grant_target_snapshot_sha256
            })
        }

        $before = Get-SerialLogOffset
        $childVm = Start-RaiosVm -Label $Label -PredicatePrefix $Label -Port $Port -MonitorPort ($Port + 100) -QmpPort 0 -PersistPath $childPersist
        $null = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
        $afterRecoveryInspection = Get-PersistInspection -Path $childPersist
        $null = Assert-RollbackMutationReclogUnchanged -Prefix $Label -ServiceId "svc.dev.granted_candidate" -BeforeInspection $outer -AfterInspection $afterRecoveryInspection
        $recoverySourceIdentityAfter = [ordered]@{
            artifact_persist = @($afterRecoveryInspection.artifact_persist_records | Where-Object {
                $_.service_id -eq "svc.dev.granted_candidate"
            } | Sort-Object reclog_frame_sha256 | ForEach-Object {
                "{0}|{1}|{2}|{3}|{4}|{5}" -f $_.reclog_frame_sha256, $_.artifact_sha256,
                    $_.promotion_transaction_sha256, $_.grant_target_schema,
                    $_.grant_target_count, $_.grant_target_snapshot_sha256
            })
            install_authorization = @($afterRecoveryInspection.reclog_frames | Where-Object {
                $_.payload_json.schema -eq "raios.install_authorization.v0" -and
                $_.payload_json.service_id -eq "svc.dev.granted_candidate"
            } | Sort-Object seq | ForEach-Object {
                "{0}|{1}|{2}|{3}|{4}|{5}" -f $_.seq, $_.frame_sha256,
                    $_.payload_json.install_candidate_sha256, $_.payload_json.generation,
                    $_.payload_json.grant_target_count, $_.payload_json.grant_target_snapshot_sha256
            })
        }
        $recoverySourceIdentityUnchanged = $recoverySourceIdentityBefore.artifact_persist.Count -gt 0 -and
            $recoverySourceIdentityBefore.install_authorization.Count -gt 0 -and
            (Convert-CompactJson $recoverySourceIdentityBefore 16) -eq (Convert-CompactJson $recoverySourceIdentityAfter 16)
        Assert-ReportPredicate -Prefix $Label -Name "quarantined-source-provenance-unchanged" -Expected "the recovery quarantine retains the exact durable artifact, signed-install generation, and grant-projection provenance without claiming a live service" -Passed $recoverySourceIdentityUnchanged -Actual $(Convert-CompactJson ([ordered]@{ before = $recoverySourceIdentityBefore; after = $recoverySourceIdentityAfter }) 18) -FailureMessage "$Label changed durable source provenance while quarantining the forged recovery"
        $window = (Get-SerialLogContent -Path $SerialLog).Substring([int]$before)
        $marker = Get-ProviderAutoloadMarker -BeforeOffset (Get-SerialLogOffset)
        $projectionIndex = $window.IndexOf("cap.projection ")
        $autoloadIndex = $window.IndexOf("GRANTED_CANDIDATE_PROVIDER_AUTOLOAD ")
        $deniedBeforeEffect = $marker.Result -eq "denied" -and
            $marker.Phase -eq "denied" -and $marker.Reason -eq $ExpectedReason -and
            -not $marker.Loaded -and -not $marker.Running -and $marker.RunCount -eq 0 -and
            $projectionIndex -ge 0 -and $projectionIndex -lt $autoloadIndex -and
            $window.Contains("rollback_quarantine=1") -and
            -not $window.Contains("WASM_GUEST_LOG rollback target log-only") -and
            -not $window.Contains("WASM_GUEST_LOG echo counter=")
        Assert-ReportPredicate -Prefix $Label -Name "quarantine-before-autoload-or-effect" -Expected "typed reason=$ExpectedReason is installed as quarantine before provider autoload, with neither source nor target guest effect" -Passed $deniedBeforeEffect -Actual $(Convert-CompactJson ([ordered]@{ marker = $marker; window = $window }) 18) -FailureMessage "$Label did not fail closed before autoload/effect"
        Stop-RaiosVmCleanly -Vm $childVm -Name $Label
        $childVm = $null
    }
    finally {
        if ($childVm) { Stop-RaiosVmForce -Vm $childVm -Name "$Label-failure" }
    }
}

function Invoke-RollbackBoundaryCase {
    param([ValidateSet("intent", "revoke", "commit")][string]$Phase, [int]$Port)
    $prefix = "boundary-$Phase"
    $persist = Join-Path $RunDir "persist-boundary-$Phase.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $persist -Force
    $vm = $null
    $recoveryVm = $null
    $stableVm = $null
    try {
        $vm = Start-RaiosVm -Label "$prefix-source" -PredicatePrefix $prefix -Port $Port -MonitorPort ($Port + 100) -QmpPort 0 -PersistPath $persist
        $null = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD result=accepted phase=autoloaded reason=repromotion_reverified_m6_gate_loaded_and_started" -TimeoutSeconds $TimeoutSeconds
        $startOffset = Get-SerialLogOffset
        $stream = Get-SerialTcpStream -Port $Port -TimeoutSeconds $TimeoutSeconds
        $script:SerialTcpDrainStream = $stream
        Write-SerialTcpText -Stream $stream -Text "service.rollback_apply svc.dev.granted_candidate`r"
        $stream.Flush()
        $needle = if ($Phase -eq "revoke") { "phase=revoke index=1 durable=1" } else { "phase=$Phase durable=1" }
        $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
        $seen = $false
        do {
            $text = Get-SerialLogContent -Path $SerialLog
            if ($text.Length -ge $startOffset -and $text.Substring([int]$startOffset).Contains($needle)) {
                $seen = $true
                break
            }
            Start-Sleep -Milliseconds 1
        } while ([DateTime]::UtcNow -lt $deadline)
        Assert-ReportPredicate -Prefix $prefix -Name "durable-marker-observed" -Expected "$Phase marker is emitted only after that durable append/readback" -Passed $seen -Actual $(Get-SerialLogTail -Path $SerialLog) -FailureMessage "$prefix durable boundary marker not observed"
        Stop-RaiosVmForce -Vm $vm
        $processStopped = -not (Get-Process -Id $vm.Pid -ErrorAction SilentlyContinue)
        Assert-ReportPredicate -Prefix $prefix -Name "source-process-stopped" -Expected "the exact source QEMU process exits immediately after the durable milestone marker" -Passed $processStopped -Actual "pid=$($vm.Pid) stopped=$processStopped" -FailureMessage "harness_infrastructure_failure: $prefix source QEMU survived the requested cut"
        $cutLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$startOffset)
        $laterMarkerAbsent = if ($Phase -eq "intent") {
            -not $cutLog.Contains("phase=revoke index=1 durable=1") -and -not $cutLog.Contains("phase=commit durable=1")
        } elseif ($Phase -eq "revoke") {
            -not $cutLog.Contains("phase=commit durable=1")
        } else {
            $true
        }
        Assert-ReportPredicate -Prefix $prefix -Name "later-marker-absent" -Expected "the saved serial cut contains no transaction milestone after $Phase" -Passed $laterMarkerAbsent -Actual $cutLog -FailureMessage "$prefix serial cut already contained a later durable milestone"
        $vm = $null
        $script:SerialTcpDrainStream = $null
        Close-SerialTcpConnection

        $cut = Get-PersistInspection -Path $persist
        $cutIntent = @($cut.reclog_frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.rollback_intent.v1" }).Count
        $cutRevoke = @($cut.reclog_frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.revoked.v1" -and $_.payload_json.value.host_import_id -eq "env.counter_get" }).Count
        $cutCommit = @($cut.reclog_frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.rollback_commit.v1" }).Count
        $cutOk = $cut.reclog_scan.status -eq "valid" -and $cutIntent -eq 1 -and
            $cutRevoke -eq $(if ($Phase -eq "intent") { 0 } else { 1 }) -and
            $cutCommit -eq $(if ($Phase -eq "commit") { 1 } else { 0 })
        Assert-ReportPredicate -Prefix $prefix -Name "exact-cut-state" -Expected "disk cut is exactly the requested durable milestone with no later frame" -Passed $cutOk -Actual "intent=$cutIntent revoke=$cutRevoke commit=$cutCommit" -FailureMessage "$prefix cut crossed the requested transaction milestone"

        $recoveryVm = Start-RaiosVm -Label "$prefix-recovery" -PredicatePrefix $prefix -Port ($Port + 1) -MonitorPort ($Port + 101) -QmpPort 0 -PersistPath $persist
        $null = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
        $recoveryLog = Get-SerialLogContent -Path $SerialLog
        $recovered = $recoveryLog.Contains("GRANTED_CANDIDATE_PROVIDER_AUTOLOAD result=accepted phase=autoloaded reason=rollback_commit_target_autoloaded") -and
            $recoveryLog.Contains("WASM_GUEST_LOG rollback target log-only") -and
            -not $recoveryLog.Contains("WASM_GUEST_LOG echo counter=") -and
            ($Phase -eq "commit" -or $recoveryLog.Contains("RAIOS_ROLLBACK_RECOVERY "))
        Assert-ReportPredicate -Prefix $prefix -Name "restart-converges-before-guest" -Expected "restart completes/reuses the transaction before any source guest call and autoloads only the target" -Passed $recovered -Actual $(Get-SerialLogTail -Path $SerialLog) -FailureMessage "$prefix did not converge on first restart"
        Send-AgentCommandTagged -Prefix $prefix -Command "host_import.selftest" -ExpectedMarker "RAIOS_HOSTIMPORT selftest=pass" -Name "peer-after-recovery"
        Stop-RaiosVmCleanly -Vm $recoveryVm -Name "$prefix-recovery"
        $recoveryVm = $null
        $afterRecovery = Get-PersistInspection -Path $persist
        $revokes = @($afterRecovery.reclog_frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.revoked.v1" -and $_.payload_json.value.host_import_id -eq "env.counter_get" })
        $commits = @($afterRecovery.reclog_frames | Where-Object { $_.payload_json.predicate -eq "wasm_import.rollback_commit.v1" })
        $oneShot = $afterRecovery.reclog_scan.status -eq "valid" -and $revokes.Count -eq 1 -and $commits.Count -eq 1
        Assert-ReportPredicate -Prefix $prefix -Name "one-revoke-one-commit" -Expected "recovery leaves exactly one counter revoke and one commit" -Passed $oneShot -Actual "revokes=$($revokes.Count) commits=$($commits.Count)" -FailureMessage "$prefix duplicated revoke or commit"

        $stableCount = [int64]$afterRecovery.reclog_scan.count
        $stableTail = [string]$afterRecovery.reclog_scan.tail_frame_sha256
        $stableVm = Start-RaiosVm -Label "$prefix-stable" -PredicatePrefix $prefix -Port ($Port + 2) -MonitorPort ($Port + 102) -QmpPort 0 -PersistPath $persist
        $null = Wait-ForLogText -Path $SerialLog -Needle "GRANTED_CANDIDATE_PROVIDER_AUTOLOAD result=accepted phase=autoloaded reason=rollback_commit_target_autoloaded" -TimeoutSeconds $TimeoutSeconds
        Stop-RaiosVmCleanly -Vm $stableVm -Name "$prefix-stable"
        $stableVm = $null
        $stable = Get-PersistInspection -Path $persist
        $stableOk = [int64]$stable.reclog_scan.count -eq $stableCount -and [string]$stable.reclog_scan.tail_frame_sha256 -eq $stableTail
        Assert-ReportPredicate -Prefix $prefix -Name "second-restart-stable" -Expected "a second restart appends no revoke/commit and preserves the exact tail" -Passed $stableOk -Actual "before=$stableCount/$stableTail after=$($stable.reclog_scan.count)/$($stable.reclog_scan.tail_frame_sha256)" -FailureMessage "$prefix was not stable on second restart"
    }
    finally {
        Stop-RaiosVmForce -Vm $vm
        Stop-RaiosVmForce -Vm $recoveryVm
        Stop-RaiosVmForce -Vm $stableVm
    }
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

function Assert-WasmGrantSemanticTamperChild {
    param([int]$Port)

    $prefix = "boot2-cap-link-tamper"
    $child = $null
    $tamperedPersist = Join-Path $RunDir "persist-cap-link-tamper.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $tamperedPersist -Force
    try {
        $mutation = Invoke-PersistTool -Arguments @("--tamper-wasm-grant-event", $tamperedPersist)
        $semanticLanded = $mutation.semantic_boundary -eq "parent_grant_sha256" -and
            $mutation.post_reclog_scan.status -eq "valid" -and
            $mutation.post_reclog_scan.valid_prefix_chain -eq $true -and
            $mutation.post_reclog_scan.full_region_valid -eq $true
        Assert-ReportPredicate -Prefix $prefix -Name "semantic-tamper-landed" -Expected "revoke parent_grant_sha256 changed while the outer RECLOG remains fully valid" -Passed $semanticLanded -Actual $(Convert-CompactJson $mutation 8) -FailureMessage "Wasm grant semantic tamper did not land"

        $child = Start-RaiosVm -Label "cap-link-tamper" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -QmpPort 0 -PersistPath $tamperedPersist
        $null = Wait-ForLogText -Path $SerialLog -Needle "cap.projection " -TimeoutSeconds $TimeoutSeconds
        $serial = Get-SerialLogContent -Path $SerialLog
        $foldDenied = [regex]::IsMatch($serial, 'cap\.projection sha256:[0-9a-f]{64} valid=0 events=0 reason=grant_fold_malformed_link')
        Assert-ReportPredicate -Prefix $prefix -Name "semantic-fold-denied" -Expected "fully framed but malformed revoke link yields a denied/empty boot projection" -Passed $foldDenied -Actual $(Get-SerialLogTail -Path $SerialLog) -FailureMessage "Malformed grant link was accepted by the boot fold"

        Send-AgentCommandTagged -Prefix $prefix -Command "revoke.selftest" -ExpectedMarker "RAIOS_REVOKE_DURABLE selftest=pass replay=0 invalid_projection=1 append_before_flip=0" -Name "revoked-call-zero-effect"
        $tamperCall = [regex]::Match((Get-SerialLogContent -Path $SerialLog), 'RAIOS_REVOKE_DURABLE selftest=pass replay=0 invalid_projection=1 append_before_flip=0 projection=sha256:[0-9a-f]{64} call_attempted=1 gate=env\.counter_get denied_next=1 host_effect_delta=0')
        Assert-ReportPredicate -Prefix $prefix -Name "invalid-history-real-gate" -Expected "malformed history constructs denied Store, instantiates Wasm, and attempts the real env.counter_get gate with zero effect" -Passed $tamperCall.Success -Actual $(if ($tamperCall.Success) { $tamperCall.Value } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "Invalid projection did not reach the real host call gate"
        Send-AgentCommandTagged -Prefix $prefix -Command "host_import.selftest" -ExpectedMarker "RAIOS_HOSTIMPORT selftest=pass" -Name "peer-still-answers"
        $after = Get-SerialLogContent -Path $SerialLog
        $noPanic = $after.Contains("RAIOS_HOSTIMPORT selftest=pass") -and -not $after.Contains("*** KERNEL PANIC ***")
        Assert-ReportPredicate -Prefix $prefix -Name "zero-effect-no-peer-damage" -Expected "authority stays denied with host_effect_delta=0; peer selftest still answers and no panic occurs" -Passed $noPanic -Actual $(Get-SerialLogTail -Path $SerialLog) -FailureMessage "Semantic grant corruption damaged the peer or panicked"
    }
    finally {
        Stop-RaiosVmForce -Vm $child
        Remove-RunImages -Vm $child
        Remove-Item -LiteralPath $tamperedPersist -Force -ErrorAction SilentlyContinue
    }
}

function Get-ProgramEditorFixture {
    $base64 = @'
UlVJUAEAIACwAAAAAQMAAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAABABQACAAIAPAAGAAAAAAAcmFpT1MgRURJVCAgRjEyPUVYSVQEAAAACAAoAGACgAEAAAAAAwAFAAgAtAFgACQAAQAAAENMRUFSAAAAAQAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
'@ -replace '\s', ''
    $bytes = [Convert]::FromBase64String($base64)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $actual = ([BitConverter]::ToString($sha.ComputeHash($bytes)) -replace "-", "").ToLowerInvariant()
    }
    finally {
        $sha.Dispose()
    }
    if ($bytes.Length -ne 176 -or $actual -ne "34f726d13818d174e23ef0614ca183a2967b9449c8cf4447151aef13d277d815") {
        throw "Canonical editor RUIP fixture drifted: bytes=$($bytes.Length) sha256=$actual"
    }
    return [pscustomobject]@{
        Bytes = $bytes
        Hash = "sha256:$actual"
    }
}

function Get-ProgramInstallReadyMarker {
    param([string]$ProgramSha256)

    $null = Wait-ForLogText -Path $SerialLog -Needle "PROGRAM_INSTALL_READY " -TimeoutSeconds $TimeoutSeconds
    $pattern = '^PROGRAM_INSTALL_READY result=(accepted) physical_approval=(genesis_pointer) program_sha256=(sha256:[0-9a-f]{64}) activation_approval_sha256=(sha256:[0-9a-f]{64}) engine=(svc\.user\.shell) persistence_authority=(false) reason=(program_current_boot_approved)$'
    $lines = @((Get-SerialLogContent -Path $SerialLog) -split "`n" | ForEach-Object { $_.TrimEnd() } | Where-Object {
        $_.StartsWith("PROGRAM_INSTALL_READY ", [System.StringComparison]::Ordinal)
    })
    if ($lines.Count -ne 1) {
        throw "Expected exactly one PROGRAM_INSTALL_READY marker, found $($lines.Count): $(Get-SerialLogTail -Path $SerialLog)"
    }
    $match = [regex]::Match([string]$lines[0], $pattern)
    if (-not $match.Success -or $match.Groups[3].Value -ne $ProgramSha256) {
        throw "PROGRAM_INSTALL_READY did not match the merged emitter: $($lines[0])"
    }
    return [pscustomobject]@{
        Line = [string]$lines[0]
        ProgramSha256 = [string]$match.Groups[3].Value
        ActivationApprovalSha256 = [string]$match.Groups[4].Value
    }
}

function Get-ProgramAutoloadMarker {
    param([int64]$BeforeOffset = -1)

    $null = Wait-ForLogText -Path $SerialLog -Needle "PROGRAM_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
    $log = Get-SerialLogContent -Path $SerialLog
    $pattern = '^PROGRAM_AUTOLOAD result=(accepted|denied) phase=(autoloaded|not_installed|rolled_back|denied) reason=([a-z0-9_]+) posture=(Normal|Probation|Safe|PersistenceUnavailable) program_sha256=(sha256:[0-9a-f]{64}|none) promotion_transaction_sha256=(sha256:[0-9a-f]{64}|none) program_persist_frame_sha256=(sha256:[0-9a-f]{64}|none) w6_signature_verified=(true|false) canonical_verified=(true|false) workspace_reloaded=(true|false) shell_started=(false) cross_reboot_proven=(true|false)$'
    $found = New-Object System.Collections.Generic.List[object]
    $offset = 0
    foreach ($rawLine in @($log -split "`n")) {
        $line = $rawLine.TrimEnd()
        if ($line.StartsWith("PROGRAM_AUTOLOAD ", [System.StringComparison]::Ordinal)) {
            $match = [regex]::Match($line, $pattern)
            if (-not $match.Success) {
                throw "PROGRAM_AUTOLOAD did not match the merged emitter: $line"
            }
            $found.Add([pscustomobject]@{
                Line = $line
                Offset = [int64]$offset
                Result = [string]$match.Groups[1].Value
                Phase = [string]$match.Groups[2].Value
                Reason = [string]$match.Groups[3].Value
                Posture = [string]$match.Groups[4].Value
                ProgramSha256 = [string]$match.Groups[5].Value
                PromotionTransactionSha256 = [string]$match.Groups[6].Value
                ProgramPersistFrameSha256 = [string]$match.Groups[7].Value
                W6SignatureVerified = $match.Groups[8].Value -eq "true"
                CanonicalVerified = $match.Groups[9].Value -eq "true"
                WorkspaceReloaded = $match.Groups[10].Value -eq "true"
                ShellStarted = $false
                CrossRebootProven = $match.Groups[12].Value -eq "true"
            }) | Out-Null
        }
        $offset += $rawLine.Length + 1
    }
    if ($found.Count -ne 1) {
        throw "Expected exactly one PROGRAM_AUTOLOAD marker, found $($found.Count): $(Get-SerialLogTail -Path $SerialLog)"
    }
    $marker = $found[0]
    if ($BeforeOffset -ge 0 -and $marker.Offset -ge $BeforeOffset) {
        throw "PROGRAM_AUTOLOAD arrived after the first command boundary: $($marker.Line)"
    }
    return $marker
}

function Get-ProgramRollbackCommitMarker {
    param([int64]$AfterOffset)

    $log = (Get-SerialLogContent -Path $SerialLog).Substring([int]$AfterOffset)
    $lines = @($log -split "`n" | ForEach-Object { $_.TrimEnd() } | Where-Object {
        $_.StartsWith("PROGRAM_ROLLBACK_COMMIT ", [System.StringComparison]::Ordinal)
    })
    $pattern = '^PROGRAM_ROLLBACK_COMMIT result=(accepted) program_sha256=(sha256:[0-9a-f]{64}) promotion_transaction_sha256=(sha256:[0-9a-f]{64}) unpromote_transaction_sha256=(sha256:[0-9a-f]{64}) workspace_removed=(true|false) durable_writes=(true) reason=(program_unpromoted)$'
    if ($lines.Count -ne 1) {
        throw "Expected exactly one PROGRAM_ROLLBACK_COMMIT marker, found $($lines.Count): $log"
    }
    $match = [regex]::Match([string]$lines[0], $pattern)
    if (-not $match.Success) {
        throw "PROGRAM_ROLLBACK_COMMIT did not match the merged emitter: $($lines[0])"
    }
    return [pscustomobject]@{
        Line = [string]$lines[0]
        ProgramSha256 = [string]$match.Groups[2].Value
        PromotionTransactionSha256 = [string]$match.Groups[3].Value
        UnpromoteTransactionSha256 = [string]$match.Groups[4].Value
        WorkspaceRemoved = $match.Groups[5].Value -eq "true"
    }
}

function Get-UiProgramHostRecord {
    param([object]$Inspection)
    return @($Inspection.reclog_frames | Where-Object {
        $_.payload_json.schema -eq "raios.artifact_persist.v0" -and
        $_.payload_json.subject_kind -eq "ui_program"
    } | Select-Object -Last 1)[0]
}

function Test-ReclogPrefix {
    param([object[]]$PrefixFrames, [object[]]$Frames)
    if ($Frames.Count -lt $PrefixFrames.Count) {
        return $false
    }
    for ($i = 0; $i -lt $PrefixFrames.Count; $i += 1) {
        if ([int64]$Frames[$i].seq -ne [int64]$PrefixFrames[$i].seq -or
            $Frames[$i].frame_sha256 -ne $PrefixFrames[$i].frame_sha256 -or
            $Frames[$i].payload_sha256 -ne $PrefixFrames[$i].payload_sha256) {
            return $false
        }
    }
    return $true
}

function Invoke-UiProgramPersistLinkTamper {
    param([string]$Path, [Parameter(Mandatory = $true)][string]$ProgramHashHex)

    # Match the target frame parse-free: substring-search the raw payload bytes
    # for the ui_program artifact_persist schema AND the known canonical program
    # hash. This is immune to NUL padding and to any seq-numbering difference
    # between the inspect and raw-parse helpers.
    $code = @'
import json, runpy, sys
m = runpy.run_path(sys.argv[1])
image = m["Path"](sys.argv[2])
program_hash = sys.argv[3]
m["assert_not_release_output"](image)
info = m["validate_existing_gpt_image"](image)
seed = m["seed_data_first_lba_from_info"](info)
def as_text(payload):
    if isinstance(payload, (bytes, bytearray)):
        return payload.decode("latin-1")
    return str(payload)
with image.open("r+b") as handle:
    frames = m["parse_reclog_frames_for_inspection"](m["read_reclog_region"](handle, seed))
    target = next((i for i, frame in enumerate(frames) if "artifact_persist.origin_boot.ui_program.v0" in as_text(frame["payload"]) and program_hash in as_text(frame["payload"])), None)
    if target is None:
        raise ValueError("no ui_program artifact_persist RECLOG record found for the program hash")
    record = m["payload_json"](frames[target]["payload"])
    field = "promotion_transaction_sha256"
    old = str(record[field])
    new = m["flip_hash_byte"](old)
    payload = m["replace_json_hash"](frames[target]["payload"], field, old, new)
    m["rewrite_reclog_frames"](handle, seed, frames, target, payload)
    handle.flush()
    m["os"].fsync(handle.fileno())
post = m["inspect_image"](image)
if (post.get("reclog_scan") or {}).get("status") != "valid":
    raise ValueError("tampered ui_program RECLOG no longer scans as valid")
print(json.dumps({"operation": "tamper_ui_program_persist_link", "field": field, "old_value": old, "new_value": new, "seq": frames[target]["seq"]}))
'@
    # Never pass this multi-line bracket-heavy script through `python -c` as a
    # native arg: PowerShell mangles it (the sole -c call in this harness).
    # Write it to a temp .py and run it by path, like every other python call.
    $scriptPath = Join-Path $RunDir "tamper-ui-program-link.py"
    [System.IO.File]::WriteAllText($scriptPath, $code, (New-Object System.Text.UTF8Encoding($false)))
    $result = Invoke-NativeCommandForReport -FilePath "python" -Arguments @($scriptPath, $Builder, $Path, $ProgramHashHex)
    if ($result.ExitCode -ne 0) {
        throw "UI-program link tamper failed: $((@($result.Output) + @($result.Error)) -join [Environment]::NewLine)"
    }
    return ($result.Output -join [Environment]::NewLine) | ConvertFrom-Json
}

function Assert-ProgramWorkspaceEmpty {
    param([string]$Prefix)

    Send-AgentCommandTagged -Prefix $Prefix -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "workspace"
    $response = Get-LastAgentResponseJson -Method "program.workspace"
    $workspace = $response.body.result
    $ok = $response.v -eq "raios.agent.v0" -and
        $workspace.method -eq "program.workspace" -and $workspace.status -eq "empty" -and
        $workspace.present -eq $false -and [int64]$workspace.byte_len -eq 0 -and
        $null -eq $workspace.program_sha256 -and $null -eq $workspace.source -and
        [int64]$workspace.pending_byte_len -eq 0 -and [int64]$workspace.pending_chunk_count -eq 0 -and
        $null -eq $workspace.pending_provider_request_id -and
        $workspace.authorizes_load -eq $false -and $workspace.authorizes_execution -eq $false -and
        $workspace.execution_attempted -eq $false -and $workspace.writes_persistent_state -eq $false
    return [pscustomobject]@{ Ok = $ok; Response = $response; Workspace = $workspace }
}

function Invoke-ProgramPersistenceProof {
    $script:Profile = "program-persistence-reboot"
    $script:Network = $false
    $fixture = Get-ProgramEditorFixture
    $programHash = [string]$fixture.Hash
    $activationNeedle = "PROGRAM_CURRENT_BOOT_ACTIVATION physical_approval=pointer program_sha256=$programHash engine=svc.user.shell capability_surface=ui_only wasm=true result=accepted"

    $env:CARGO_HOME = (Resolve-Path (Join-Path $RepoRoot ".cargo-home")).Path
    $env:CARGO_TARGET_DIR = Join-Path $RepoRoot "target"
    & cargo build --locked -p ota-tools --bin dev-promotion-signer
    if ($LASTEXITCODE -ne 0) {
        throw "dev-promotion-signer build failed with exit code $LASTEXITCODE"
    }

    if ($Image) {
        $script:ResolvedImage = (Resolve-Path -LiteralPath $Image).Path
    }
    else {
        $script:ResolvedImage = Join-Path $RunDir "raios-stage0-program-persistence.img"
        $script:TempImage = $true
        & $PackageScript -Profile release -Image $ResolvedImage -UseTempEsp
        if ($LASTEXITCODE -ne 0) {
            throw "Key-free program-persistence image packaging failed with exit code $LASTEXITCODE"
        }
    }

    $script:PersistDiskImage = Join-Path $RunDir "kept-program-persist.img"
    $builderResult = Invoke-NativeCommandForReport -FilePath "python" -Arguments @($Builder, "--self-check", "--seed-bootctl", "valid-a", $PersistDiskImage)
    if ($builderResult.ExitCode -ne 0) {
        throw "Program-persistence disk build failed: $((@($builderResult.Output) + @($builderResult.Error)) -join [Environment]::NewLine)"
    }
    $script:PersistDiskImage = Assert-PersistDiskPathSafe -Path $PersistDiskImage
    $initialInspection = Get-PersistInspection -Path $PersistDiskImage
    if (-not ([bool]$initialInspection.gpt_header_valid -and [bool]$initialInspection.gpt_crc_checked -and
        [bool]$initialInspection.data_superblock_valid -and $initialInspection.bootctl_read.decision.posture -eq "Normal" -and
        [int64]$initialInspection.reclog_scan.count -eq 0)) {
        throw "Initial program persist disk was not empty Normal: $(Convert-CompactJson $initialInspection 30)"
    }
    $script:HardwareProfile = New-HardwareProfile -Nic "none" -ScratchDrive $true -AuditRollbackTargetDrive $true -PersistDrive $true

    $script:boot1Vm = Start-RaiosVm -Label "program-boot1" -PredicatePrefix "program-boot1" -Port $SerialTcpPort -MonitorPort ($SerialTcpPort + 100) -QmpPort ($SerialTcpPort + 200) -PersistPath $PersistDiskImage
    $chunkCount = Send-GenesisUiProgramBytes -Bytes $fixture.Bytes -NamePrefix "program-boot1:editor-delivery"
    Send-AgentCommandTagged -Prefix "program-boot1" -Command "program.submit_finalize" -ExpectedMarker "RAIOS_AGENT_END program.submit_finalize" -Name "editor-finalize"
    $finalizeResponse = Get-LastAgentResponseJson -Method "program.submit_finalize"
    $finalize = $finalizeResponse.body.result
    $runOffset = Get-SerialLogOffset
    Send-QemuAbsolutePointerClick -X 27017 -Y 6559
    if (-not (Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $activationNeedle -Offset $runOffset -TimeoutSeconds $TimeoutSeconds)) {
        throw "Boot 1 editor run click did not activate the exact restored program"
    }
    $ready = Get-ProgramInstallReadyMarker -ProgramSha256 $programHash
    foreach ($key in @("h", "i")) {
        $inputOffset = Get-SerialLogOffset
        Send-GenesisUiKey -KeyName $key
        if (-not (Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -Offset $inputOffset -TimeoutSeconds $TimeoutSeconds)) {
            throw "Boot 1 editor input '$key' did not update the rendered frame"
        }
    }
    Send-AgentCommandTagged -Prefix "program-boot1" -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "editor-inventory"
    $runInventory = Get-LastAgentResponseJson -Method "service.inventory"
    Send-GenesisUiKey -KeyName "f12"
    if (-not (Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL EXIT F12 genesis" -Offset $runOffset -TimeoutSeconds $TimeoutSeconds)) {
        throw "Boot 1 F12 did not return the editor to Genesis before install"
    }
    $script:install = @(Invoke-SignedUiProgramInstall -ProgramSha256 $programHash -ActivationApprovalSha256 $ready.ActivationApprovalSha256 -NamePrefix "program-boot1:editor")[-1]
    $shellRows = @($runInventory.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
    $boot1Log = Get-SerialLogContent -Path $SerialLog
    $activationOffset = $boot1Log.IndexOf($activationNeedle, [System.StringComparison]::Ordinal)
    $readyOffset = $boot1Log.IndexOf($ready.Line, [System.StringComparison]::Ordinal)
    $boot1Ok = $finalizeResponse.v -eq "raios.agent.v0" -and $finalize.accepted -eq $true -and
        $finalize.program_sha256 -eq $programHash -and [int64]$finalize.byte_len -eq 176 -and
        [int64]$finalize.serial_chunk_count -eq [int64]$chunkCount -and
        $ready.ProgramSha256 -eq $programHash -and $ready.ActivationApprovalSha256 -match '^sha256:[0-9a-f]{64}$' -and
        $activationOffset -ge 0 -and $readyOffset -gt $activationOffset -and
        $shellRows.Count -eq 1 -and $shellRows[0].scope -eq "current_boot" -and $shellRows[0].running -eq $true -and
        $install.PrepareResponse.v -eq "raios.agent.v0" -and $install.SignatureResponse.v -eq "raios.agent.v0" -and
        $install.DenialResponse.v -eq "raios.agent.v0" -and
        $install.Preview.install_source -eq "ui_program" -and $install.Preview.receipt_kind -eq "ruip_canonical" -and
        $install.Preview.candidate_sha256 -eq $programHash -and
        $install.Preview.activation_approval_sha256 -eq $ready.ActivationApprovalSha256 -and
        $install.SignedPreview.signature_verified -eq $true -and
        $install.SignedPreview.action_signature_message_sha256 -eq $install.ActionSignatureMessageSha256 -and
        $install.SignedPreview.action_signature_message_sha256 -ne $ready.ActivationApprovalSha256 -and
        $install.Denial.reason -eq "project_install_physical_pointer_approval_required" -and
        (1 + $install.ClickCount) -eq 2 -and
        $install.ProgramSha256 -eq $programHash -and $install.ActivationApprovalSha256 -eq $ready.ActivationApprovalSha256 -and
        $install.InstallEnvelopeSha256 -eq $install.SignedPreview.install_envelope_sha256 -and
        $install.InstallActionSha256 -match '^sha256:[0-9a-f]{64}$' -and
        $install.PromotionTransactionSha256 -match '^sha256:[0-9a-f]{64}$' -and
        $install.ProgramPersistFrameSha256 -match '^sha256:[0-9a-f]{64}$' -and
        [int64]$install.CanonicalProgramByteLen -eq 176 -and $install.ParsedPayloadSha256 -eq $programHash -and
        # A clean program-only boot writes NO pre-install durable record: the
        # editor RUIP renders through the shell guest but neither echoes a guest
        # log nor requests a wasm import grant (unlike the granted candidate), so
        # RECLOG is empty (count 0) until the three install frames land.
        [int64]$install.PreReclogScan.count -eq 0 -and
        [int64]$install.PostReclogScan.count -eq ([int64]$install.PreReclogScan.count + 3) -and
        [int64]$install.Sequence -eq ([int64]$install.PreReclogScan.tail_seq + 2) -and
        [int64]$install.PostReclogScan.tail_seq -eq ($install.Sequence + 1) -and
        $install.PostReclogScan.tail_frame_sha256 -eq $install.ProgramPersistFrameSha256 -and
        [int64]$install.PostArtifactScan.ui_program_persist_record_count -eq ([int64]$install.PreArtifactScan.ui_program_persist_record_count + 1) -and
        $install.InstalledArtifactRecord.canonical_program_sha256 -eq $programHash -and
        $install.InstalledArtifactRecord.parsed_payload_sha256 -eq $programHash -and
        $install.ActivationCountAfterClick -eq $install.ActivationCountBeforeClick -and
        $install.ShellActiveCountAfterClick -eq $install.ShellActiveCountBeforeClick -and
        ([regex]::Matches($install.AfterClickLog, [regex]::Escape("PROGRAM_CURRENT_BOOT_ACTIVATION "))).Count -eq 0
    $boot1Dump = [ordered]@{ finalize = $finalizeResponse; ready = $ready; inventory = $runInventory; install = $install }
    Assert-ReportPredicate -Prefix "program-boot1" -Name "approved-editor-installed" -Expected "two distinct physical clicks run then W6-install the exact 176-byte editor; three linked frames and one UI ARTSTOR record land without install-time rerun" -Passed $boot1Ok -Actual $(if ($boot1Ok) { $install.MarkerLine } else { Convert-CompactJson $boot1Dump 30 }) -FailureMessage "Boot 1 did not physically run and durably install the exact editor"

    Stop-RaiosVmCleanly -Vm $boot1Vm -Name "program-boot1"
    $script:postBoot1Inspection = Get-PersistInspection -Path $PersistDiskImage
    $boot1Frames = @($postBoot1Inspection.reclog_frames)
    $authorization = @($boot1Frames | Where-Object { [int64]$_.seq -eq ([int64]$install.Sequence - 1) })[0]
    $promotion = @($boot1Frames | Where-Object { "sha256:$($_.frame_sha256)" -eq $install.PromotionTransactionSha256 })[0]
    $persist = Get-UiProgramHostRecord -Inspection $postBoot1Inspection
    $blob = $postBoot1Inspection.first_artstor_blob
    $hostBoot1Ok = $postBoot1Inspection.reclog_scan.status -eq "valid" -and
        $authorization.payload_json.schema -eq "raios.install_authorization.v0" -and $authorization.payload_json.subject_kind -eq "ui_program" -and
        $authorization.payload_json.canonical_program_sha256 -eq $programHash -and $authorization.payload_json.activation_approval_sha256 -eq $ready.ActivationApprovalSha256 -and
        $promotion.payload_json.schema -eq "raios.promotion_transaction.v0" -and $promotion.payload_json.subject_kind -eq "ui_program" -and
        $promotion.payload_json.transaction_kind -eq "promote" -and
        $promotion.payload_json.install_authorization_frame_sha256 -eq "sha256:$($authorization.frame_sha256)" -and
        [int64]$persist.seq -eq ([int64]$promotion.seq + 1) -and
        "sha256:$($persist.frame_sha256)" -eq $install.ProgramPersistFrameSha256 -and
        $persist.payload_json.canonical_program_sha256 -eq $programHash -and
        [int64]$persist.payload_json.canonical_program_byte_len -eq 176 -and
        $persist.payload_json.install_authorization_frame_sha256 -eq "sha256:$($authorization.frame_sha256)" -and
        $persist.payload_json.promotion_transaction_sha256 -eq $install.PromotionTransactionSha256 -and
        $blob.valid -eq $true -and [int64]$blob.payload_len -eq 176 -and
        "sha256:$($blob.payload_sha256)" -eq $programHash -and
        "sha256:$($blob.frame_sha256)" -eq $persist.payload_json.artstor_blob_frame_sha256
    Assert-ReportPredicate -Prefix "program-boot1" -Name "persist-disk-has-w6-provenance" -Expected "clean-quit host inspection finds linked ui_program authorization/promote/persist frames and the exact 176-byte ARTSTOR blob" -Passed $hostBoot1Ok -Actual $(if ($hostBoot1Ok) { "authorization=sha256:$($authorization.frame_sha256) promote=$($install.PromotionTransactionSha256) persist=$($install.ProgramPersistFrameSha256)" } else { Convert-CompactJson $postBoot1Inspection 30 }) -FailureMessage "Boot 1 host inspection did not find exact UI-program W6 provenance"
    Copy-Item -LiteralPath $PersistDiskImage -Destination $Boot1PersistSnapshot -Force

    $script:boot2Vm = Start-RaiosVm -Label "program-boot2" -PredicatePrefix "program-boot2" -Port ($SerialTcpPort + 1) -MonitorPort ($SerialTcpPort + 101) -QmpPort ($SerialTcpPort + 201) -PersistPath $PersistDiskImage
    $null = Wait-ForLogText -Path $SerialLog -Needle "PROGRAM_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
    $boot2FirstCommandOffset = Get-SerialLogOffset
    $boot2Marker = Get-ProgramAutoloadMarker -BeforeOffset $boot2FirstCommandOffset
    $boot2Before = $boot2Marker.Result -eq "accepted" -and $boot2Marker.Phase -eq "autoloaded" -and
        $boot2Marker.Reason -eq "program_autoloaded" -and $boot2Marker.Posture -eq "Normal" -and
        $boot2Marker.ProgramSha256 -eq $programHash -and
        $boot2Marker.PromotionTransactionSha256 -eq $install.PromotionTransactionSha256 -and
        $boot2Marker.ProgramPersistFrameSha256 -eq $install.ProgramPersistFrameSha256 -and
        $boot2Marker.W6SignatureVerified -and $boot2Marker.CanonicalVerified -and $boot2Marker.WorkspaceReloaded -and
        -not $boot2Marker.ShellStarted -and $boot2Marker.CrossRebootProven -and
        $boot2Marker.Offset -lt $boot2FirstCommandOffset
    Assert-ReportPredicate -Prefix "program-boot2" -Name "autoload-before-command" -Expected "exact accepted autoload marker restores the 176-byte editor before the first command with W6/canonical/workspace true and shell false" -Passed $boot2Before -Actual $(if ($boot2Before) { $boot2Marker.Line } else { Convert-CompactJson $boot2Marker 20 }) -FailureMessage "Boot 2 program autoload did not precede the first command exactly"

    Send-AgentCommandTagged -Prefix "program-boot2" -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "workspace-restored"
    $workspaceResponse = Get-LastAgentResponseJson -Method "program.workspace"
    $workspace = $workspaceResponse.body.result
    $workspaceOk = $workspaceResponse.v -eq "raios.agent.v0" -and $workspace.method -eq "program.workspace" -and
        $workspace.status -eq "ready" -and $workspace.present -eq $true -and
        $workspace.source -eq "durable" -and $workspace.retention -eq "durable" -and
        [int64]$workspace.byte_len -eq 176 -and $workspace.program_sha256 -eq $programHash -and
        [int64]$workspace.pending_byte_len -eq 0 -and [int64]$workspace.pending_chunk_count -eq 0 -and
        $null -eq $workspace.pending_provider_request_id -and
        $workspace.signing_attempted -eq $false -and $workspace.load_attempted -eq $false -and
        $workspace.execution_attempted -eq $false -and $workspace.authorizes_load -eq $false -and
        $workspace.authorizes_execution -eq $false -and $workspace.writes_persistent_state -eq $false
    Assert-ReportPredicate -Prefix "program-boot2" -Name "workspace-restored-exact" -Expected "raios.agent.v0 program.workspace reports source/retention durable, exact 176-byte editor, no pending delivery, and no execution authority" -Passed $workspaceOk -Actual $(if ($workspaceOk) { "$programHash durable inert" } else { Convert-CompactJson $workspaceResponse 20 }) -FailureMessage "Boot 2 program.workspace did not truthfully expose the exact durable inert editor"

    $boot2RunOffset = Get-SerialLogOffset
    Send-QemuAbsolutePointerClick -X 27017 -Y 6559
    if (-not (Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $activationNeedle -Offset $boot2RunOffset -TimeoutSeconds $TimeoutSeconds)) {
        throw "Boot 2 restored editor click did not activate the exact program"
    }
    $editorUpdates = $true
    foreach ($key in @("h", "i", "ret", "r", "backspace")) {
        $inputOffset = Get-SerialLogOffset
        Send-GenesisUiKey -KeyName $key
        if (-not (Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -Offset $inputOffset -TimeoutSeconds $TimeoutSeconds)) {
            $editorUpdates = $false
            break
        }
    }
    Send-AgentCommandTagged -Prefix "program-boot2" -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "restored-editor-inventory"
    $boot2Inventory = Get-LastAgentResponseJson -Method "service.inventory"
    $boot2ShellRows = @($boot2Inventory.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
    $boot2RunLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$boot2RunOffset)
    $boot2RunOk = $editorUpdates -and
        ([regex]::Matches($boot2RunLog, [regex]::Escape($activationNeedle))).Count -eq 1 -and
        $boot2ShellRows.Count -eq 1 -and $boot2ShellRows[0].scope -eq "current_boot" -and
        $boot2ShellRows[0].persistence -eq "none" -and $boot2ShellRows[0].running -eq $true -and
        [int64]$boot2ShellRows[0].host_import_count -eq 6
    Assert-ReportPredicate -Prefix "program-boot2" -Name "physical-click-runs-restored-editor" -Expected "one fresh Genesis click activates the exact restored editor once; its bounded current-boot shell renders and updates through physical input" -Passed $boot2RunOk -Actual $(if ($boot2RunOk) { "$activationNeedle; editor frame updated" } else { Convert-CompactJson ([ordered]@{ inventory = $boot2Inventory; log = $boot2RunLog }) 24 }) -FailureMessage "Boot 2 physical click did not run the restored editor exactly once"

    Send-GenesisUiKey -KeyName "f12"
    if (-not (Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL EXIT F12 genesis" -Offset $boot2RunOffset -TimeoutSeconds $TimeoutSeconds)) {
        throw "Boot 2 F12 did not return the restored editor to Genesis"
    }
    Send-AgentCommandTagged -Prefix "program-boot2" -Command "program.rollback_preview $programHash" -ExpectedMarker "RAIOS_AGENT_END program.rollback_preview" -Name "rollback-preview"
    $previewResponse = Get-LastAgentResponseJson -Method "program.rollback_preview"
    $preview = $previewResponse.body.result
    $rollbackOffset = Get-SerialLogOffset
    Send-AgentCommandTagged -Prefix "program-boot2" -Command "program.rollback_apply $programHash" -ExpectedMarker "RAIOS_AGENT_END program.rollback_apply" -Name "rollback-apply"
    $applyResponse = Get-LastAgentResponseJson -Method "program.rollback_apply"
    $apply = $applyResponse.body.result
    $rollbackMarker = Get-ProgramRollbackCommitMarker -AfterOffset $rollbackOffset
    Send-AgentCommandTagged -Prefix "program-boot2" -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "workspace-after-rollback"
    $afterRollbackWorkspaceResponse = Get-LastAgentResponseJson -Method "program.workspace"
    $afterRollbackWorkspace = $afterRollbackWorkspaceResponse.body.result
    Send-AgentCommandTagged -Prefix "program-boot2" -Command "program.rollback_apply $programHash" -ExpectedMarker "RAIOS_AGENT_END program.rollback_apply" -Name "rollback-apply-second-denied"
    $secondApplyResponse = Get-LastAgentResponseJson -Method "program.rollback_apply"
    $secondApply = $secondApplyResponse.body.result
    Send-AgentCommandTagged -Prefix "program-boot2" -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "rollback-reclog"
    $rollbackScanResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
    $rollbackScan = $rollbackScanResponse.body.result
    $rollbackOk = $previewResponse.v -eq "raios.agent.v0" -and $preview.accepted -eq $true -and
        $preview.reason -eq "program_rollback_ready" -and $preview.program_sha256 -eq $programHash -and
        $preview.writes_persistent_state -eq $false -and
        $applyResponse.v -eq "raios.agent.v0" -and $apply.accepted -eq $true -and
        $apply.reason -eq "program_unpromoted" -and $apply.program_sha256 -eq $programHash -and
        $apply.writes_persistent_state -eq $true -and
        $rollbackMarker.ProgramSha256 -eq $programHash -and
        $rollbackMarker.PromotionTransactionSha256 -eq $install.PromotionTransactionSha256 -and
        $rollbackMarker.UnpromoteTransactionSha256 -match '^sha256:[0-9a-f]{64}$' -and $rollbackMarker.WorkspaceRemoved -and
        $afterRollbackWorkspaceResponse.v -eq "raios.agent.v0" -and $afterRollbackWorkspace.status -eq "empty" -and
        $afterRollbackWorkspace.present -eq $false -and $null -eq $afterRollbackWorkspace.program_sha256 -and
        $secondApplyResponse.v -eq "raios.agent.v0" -and $secondApply.accepted -eq $false -and
        $secondApply.reason -eq "ui_program_already_rolled_back" -and $secondApply.writes_persistent_state -eq $false -and
        $rollbackScan.status -eq "valid" -and $rollbackScan.tail_frame_sha256 -eq $rollbackMarker.UnpromoteTransactionSha256
    $rollbackDump = [ordered]@{ preview = $previewResponse; apply = $applyResponse; marker = $rollbackMarker; workspace = $afterRollbackWorkspaceResponse; second_apply = $secondApplyResponse; scan = $rollbackScanResponse }
    Assert-ReportPredicate -Prefix "program-boot2" -Name "rollback-tombstone-readback" -Expected "F12-stopped exact-hash preview/apply appends one linked unpromote, removes only the durable workspace entry, and denies a second apply" -Passed $rollbackOk -Actual $(if ($rollbackOk) { $rollbackMarker.Line } else { Convert-CompactJson $rollbackDump 30 }) -FailureMessage "Boot 2 program rollback did not append and read back the one-shot tombstone"

    Stop-RaiosVmCleanly -Vm $boot2Vm -Name "program-boot2"
    $script:postBoot2Inspection = Get-PersistInspection -Path $PersistDiskImage
    $boot2Frames = @($postBoot2Inspection.reclog_frames)
    $prefixOk = Test-ReclogPrefix -PrefixFrames $boot1Frames -Frames $boot2Frames
    $tailFrame = @($boot2Frames | Select-Object -Last 1)[0]
    $boot2ChainOk = $prefixOk -and $boot2Frames.Count -gt $boot1Frames.Count -and
        $postBoot2Inspection.reclog_scan.status -eq "valid" -and
        $tailFrame.payload_json.schema -eq "raios.promotion_transaction.v0" -and
        $tailFrame.payload_json.subject_kind -eq "ui_program" -and $tailFrame.payload_json.transaction_kind -eq "unpromote" -and
        "sha256:$($tailFrame.frame_sha256)" -eq $rollbackMarker.UnpromoteTransactionSha256
    Assert-ReportPredicate -Prefix "program-boot2" -Name "boot1-prefix-immutable" -Expected "every boot-1 frame is byte-identical; actual boot-2 additions continue a valid chain and end at the linked UI-program unpromote without assuming a count" -Passed $boot2ChainOk -Actual $(if ($boot2ChainOk) { "prefix=$($boot1Frames.Count) additions=$($boot2Frames.Count - $boot1Frames.Count) tail=sha256:$($tailFrame.frame_sha256)" } else { Convert-CompactJson $postBoot2Inspection 30 }) -FailureMessage "Boot 2 changed the boot-1 prefix or produced an invalid newer tail"

    $script:boot3Vm = Start-RaiosVm -Label "program-boot3" -PredicatePrefix "program-boot3" -Port ($SerialTcpPort + 2) -MonitorPort ($SerialTcpPort + 102) -QmpPort 0 -PersistPath $PersistDiskImage
    $null = Wait-ForLogText -Path $SerialLog -Needle "PROGRAM_AUTOLOAD " -TimeoutSeconds $TimeoutSeconds
    $boot3FirstCommandOffset = Get-SerialLogOffset
    $boot3Marker = Get-ProgramAutoloadMarker -BeforeOffset $boot3FirstCommandOffset
    $boot3PreCommandLog = Get-SerialLogContent -Path $SerialLog
    $boot3Resolved = $boot3Marker.Result -eq "accepted" -and $boot3Marker.Phase -eq "rolled_back" -and
        $boot3Marker.Reason -eq "ui_program_install_rolled_back" -and $boot3Marker.Posture -eq "Normal" -and
        $boot3Marker.ProgramSha256 -eq $programHash -and
        $boot3Marker.PromotionTransactionSha256 -eq $rollbackMarker.UnpromoteTransactionSha256 -and
        $boot3Marker.ProgramPersistFrameSha256 -eq "none" -and
        -not $boot3Marker.W6SignatureVerified -and -not $boot3Marker.CanonicalVerified -and
        -not $boot3Marker.WorkspaceReloaded -and -not $boot3Marker.ShellStarted -and
        -not $boot3Marker.CrossRebootProven -and $boot3Marker.Offset -lt $boot3FirstCommandOffset
    Assert-ReportPredicate -Prefix "program-boot3" -Name "rollback-resolved-before-command" -Expected "exact rolled_back marker names the newest unpromote before commands with canonical/workspace/shell flags false" -Passed $boot3Resolved -Actual $(if ($boot3Resolved) { $boot3Marker.Line } else { Convert-CompactJson $boot3Marker 20 }) -FailureMessage "Boot 3 did not resolve the durable UI-program rollback before commands"

    $boot3Workspace = Assert-ProgramWorkspaceEmpty -Prefix "program-boot3"
    Send-AgentCommandTagged -Prefix "program-boot3" -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "inventory"
    $boot3Inventory = Get-LastAgentResponseJson -Method "service.inventory"
    Send-AgentCommandTagged -Prefix "program-boot3" -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "reclog"
    $boot3ScanResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
    $boot3Scan = $boot3ScanResponse.body.result
    $boot3Empty = $boot3Workspace.Ok -and
        @($boot3Inventory.facts.services | Where-Object { $_.id -eq "svc.user.shell" }).Count -eq 0 -and
        -not $boot3PreCommandLog.Contains("PROGRAM_CURRENT_BOOT_ACTIVATION ") -and
        -not $boot3PreCommandLog.Contains("PERSONAL SHELL ACTIVE current_boot proof") -and
        -not $boot3PreCommandLog.Contains("PERSONAL SHELL FRAME UPDATED")
    Assert-ReportPredicate -Prefix "program-boot3" -Name "workspace-empty-no-shell" -Expected "program.workspace and inventory remain empty after rollback; no activation, shell-active, or frame marker occurs" -Passed $boot3Empty -Actual $(if ($boot3Empty) { "workspace and svc.user.shell absent" } else { Convert-CompactJson ([ordered]@{ workspace = $boot3Workspace.Response; inventory = $boot3Inventory; pre_command_log = $boot3PreCommandLog }) 24 }) -FailureMessage "Boot 3 resurrected a rolled-back program or shell"
    Stop-RaiosVmCleanly -Vm $boot3Vm -Name "program-boot3"

    foreach ($child in @(
        [pscustomobject]@{ Label = "program-corrupt-blob"; Port = $SerialTcpPort + 10; Kind = "blob" },
        [pscustomobject]@{ Label = "program-tamper-link"; Port = $SerialTcpPort + 11; Kind = "link" }
    )) {
        $childVm = $null
        $childPersist = Join-Path $RunDir "$($child.Label).img"
        Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $childPersist -Force
        try {
            if ($child.Kind -eq "blob") {
                Invoke-PersistTool -Arguments @("--corrupt-artstor-blob", $childPersist) | Out-Null
            }
            else {
                $null = Invoke-UiProgramPersistLinkTamper -Path $childPersist -ProgramHashHex ($programHash -replace '^sha256:', '')
            }
            $mutated = Get-PersistInspection -Path $childPersist
            $childVm = Start-RaiosVm -Label $child.Label -PredicatePrefix $child.Label -Port $child.Port -MonitorPort 0 -QmpPort 0 -PersistPath $childPersist
            $childMarker = Get-ProgramAutoloadMarker
            $childWorkspace = Assert-ProgramWorkspaceEmpty -Prefix $child.Label
            $childLog = Get-SerialLogContent -Path $SerialLog
            if ($child.Kind -eq "blob") {
                $childOk = $childMarker.Result -eq "denied" -and $childMarker.Phase -eq "denied" -and
                    $childMarker.Reason -eq "blob_hash_mismatch" -and $childMarker.ProgramSha256 -eq $programHash -and
                    $childMarker.PromotionTransactionSha256 -eq $install.PromotionTransactionSha256 -and
                    $childMarker.ProgramPersistFrameSha256 -eq $install.ProgramPersistFrameSha256 -and
                    $childMarker.W6SignatureVerified -and -not $childMarker.CanonicalVerified -and
                    -not $childMarker.WorkspaceReloaded -and -not $childMarker.CrossRebootProven -and
                    $childWorkspace.Ok -and -not $childLog.Contains("PROGRAM_CURRENT_BOOT_ACTIVATION ")
                Assert-ReportPredicate -Prefix "program-corrupt-blob" -Name "autoload-denied-no-workspace" -Expected "mutated ARTSTOR frame yields exact blob_hash_mismatch after W6 link verification, with no workspace or activation" -Passed $childOk -Actual $(if ($childOk) { $childMarker.Line } else { Convert-CompactJson ([ordered]@{ marker = $childMarker; workspace = $childWorkspace.Response; inspection = $mutated; log = $childLog }) 30 }) -FailureMessage "Corrupt UI-program blob did not fail closed before workspace restore"
            }
            else {
                $childOk = $mutated.reclog_scan.status -eq "valid" -and
                    $childMarker.Result -eq "accepted" -and $childMarker.Phase -eq "not_installed" -and
                    $childMarker.Reason -eq "no_w6_authorized_program_install" -and
                    $childMarker.ProgramSha256 -eq "none" -and $childMarker.PromotionTransactionSha256 -eq "none" -and
                    $childMarker.ProgramPersistFrameSha256 -eq "none" -and -not $childMarker.W6SignatureVerified -and
                    -not $childMarker.CanonicalVerified -and -not $childMarker.WorkspaceReloaded -and
                    -not $childMarker.CrossRebootProven -and $childWorkspace.Ok -and
                    -not $childLog.Contains("PROGRAM_CURRENT_BOOT_ACTIVATION ")
                Assert-ReportPredicate -Prefix "program-tamper-link" -Name "autoload-denied-no-fallback" -Expected "valid rebuilt RECLOG with changed program-persist promotion link resolves exact accepted/not_installed with no fallback, workspace, or activation" -Passed $childOk -Actual $(if ($childOk) { $childMarker.Line } else { Convert-CompactJson ([ordered]@{ marker = $childMarker; workspace = $childWorkspace.Response; inspection = $mutated; log = $childLog }) 30 }) -FailureMessage "Tampered UI-program persist link restored a workspace or fell back"
            }
        }
        finally {
            Stop-RaiosVmForce -Vm $childVm
            Remove-RunImages -Vm $childVm
            Remove-Item -LiteralPath $childPersist -Force -ErrorAction SilentlyContinue
        }
    }

    $safeVm = $null
    $safePersist = Join-Path $RunDir "program-safe.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $safePersist -Force
    try {
        Invoke-PersistTool -Arguments @("--image", $safePersist, "--seed-bootctl", "both-invalid") | Out-Null
        $safeInspection = Get-PersistInspection -Path $safePersist
        $safeVm = Start-RaiosVm -Label "program-safe" -PredicatePrefix "program-safe" -Port ($SerialTcpPort + 12) -MonitorPort 0 -QmpPort 0 -PersistPath $safePersist
        $safeMarker = Get-ProgramAutoloadMarker
        $safeWorkspace = Assert-ProgramWorkspaceEmpty -Prefix "program-safe"
        $safeOk = $safeInspection.bootctl_read.decision.posture -eq "Safe" -and
            $safeMarker.Result -eq "denied" -and $safeMarker.Phase -eq "denied" -and
            $safeMarker.Reason -eq "program_autoload_posture_not_normal" -and $safeMarker.Posture -eq "Safe" -and
            $safeMarker.ProgramSha256 -eq "none" -and $safeMarker.PromotionTransactionSha256 -eq "none" -and
            $safeMarker.ProgramPersistFrameSha256 -eq "none" -and -not $safeMarker.W6SignatureVerified -and
            -not $safeMarker.CanonicalVerified -and -not $safeMarker.WorkspaceReloaded -and
            -not $safeMarker.CrossRebootProven -and $safeWorkspace.Ok
        Assert-ReportPredicate -Prefix "program-safe" -Name "autoload-skipped-no-workspace" -Expected "Safe posture emits exact program_autoload_posture_not_normal before record/blob intake and leaves workspace empty" -Passed $safeOk -Actual $(if ($safeOk) { $safeMarker.Line } else { Convert-CompactJson ([ordered]@{ marker = $safeMarker; workspace = $safeWorkspace.Response; inspection = $safeInspection }) 30 }) -FailureMessage "Safe posture did not skip UI-program autoload before workspace restore"
    }
    finally {
        Stop-RaiosVmForce -Vm $safeVm
        Remove-RunImages -Vm $safeVm
        Remove-Item -LiteralPath $safePersist -Force -ErrorAction SilentlyContinue
    }

    $finalInspection = Get-PersistInspection -Path $PersistDiskImage
    $finalFrames = @($finalInspection.reclog_frames)
    $finalAuthorization = @($finalFrames | Where-Object { $_.payload_json.schema -eq "raios.install_authorization.v0" -and $_.payload_json.subject_kind -eq "ui_program" })[0]
    $finalPromote = @($finalFrames | Where-Object { $_.payload_json.schema -eq "raios.promotion_transaction.v0" -and $_.payload_json.subject_kind -eq "ui_program" -and $_.payload_json.transaction_kind -eq "promote" })[0]
    $finalPersist = Get-UiProgramHostRecord -Inspection $finalInspection
    $finalUnpromote = @($finalFrames | Where-Object { $_.payload_json.schema -eq "raios.promotion_transaction.v0" -and $_.payload_json.subject_kind -eq "ui_program" -and $_.payload_json.transaction_kind -eq "unpromote" } | Select-Object -Last 1)[0]
    $finalChainOk = $finalInspection.reclog_scan.status -eq "valid" -and $boot3Scan.status -eq "valid" -and
        [int64]$finalInspection.reclog_scan.count -eq [int64]$boot3Scan.count -and
        "sha256:$($finalInspection.reclog_scan.head_frame_sha256)" -eq $boot3Scan.head_frame_sha256 -and
        "sha256:$($finalInspection.reclog_scan.tail_frame_sha256)" -eq $boot3Scan.tail_frame_sha256 -and
        [int64]$finalPromote.seq -eq ([int64]$finalAuthorization.seq + 1) -and
        $finalPromote.payload_json.install_authorization_frame_sha256 -eq "sha256:$($finalAuthorization.frame_sha256)" -and
        [int64]$finalPersist.seq -eq ([int64]$finalPromote.seq + 1) -and
        $finalPersist.payload_json.promotion_transaction_sha256 -eq "sha256:$($finalPromote.frame_sha256)" -and
        $finalPersist.payload_json.install_authorization_frame_sha256 -eq "sha256:$($finalAuthorization.frame_sha256)" -and
        $finalUnpromote.payload_json.install_authorization_frame_sha256 -eq "sha256:$($finalAuthorization.frame_sha256)" -and
        $finalUnpromote.payload_json.canonical_program_sha256 -eq $programHash -and
        "sha256:$($finalUnpromote.frame_sha256)" -eq $rollbackMarker.UnpromoteTransactionSha256 -and
        "sha256:$($finalInspection.reclog_scan.tail_frame_sha256)" -eq $rollbackMarker.UnpromoteTransactionSha256 -and
        (Test-ReclogPrefix -PrefixFrames $boot1Frames -Frames $finalFrames)
    $finalDump = [ordered]@{ host = $finalInspection; guest = $boot3ScanResponse; authorization = $finalAuthorization; promote = $finalPromote; persist = $finalPersist; unpromote = $finalUnpromote }
    Assert-ReportPredicate -Prefix "program-reclog-chain" -Name "authorization-promote-persist-unpromote-valid" -Expected "host and guest agree on the valid immutable authorization/promote/persist/.../unpromote chain, exact links and newest tombstone" -Passed $finalChainOk -Actual $(if ($finalChainOk) { "count=$($finalInspection.reclog_scan.count) tail=$($rollbackMarker.UnpromoteTransactionSha256)" } else { Convert-CompactJson $finalDump 30 }) -FailureMessage "Final UI-program RECLOG chain did not retain exact linked provenance and newest tombstone"
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

function Invoke-GrantProjectionOnlyProof {
    if ($Image) {
        throw "grant-projection-only requires a per-run current kernel image; do not pass -Image"
    }

    $env:CARGO_HOME = (Resolve-Path (Join-Path $RepoRoot ".cargo-home")).Path
    $env:CARGO_TARGET_DIR = Join-Path $RepoRoot "target"
    $script:ResolvedImage = Join-Path $RunDir "raios-stage0-grant-reboot.img"
    $script:TempImage = $true
    & $PackageScript -Profile release -Image $ResolvedImage -UseTempEsp
    $packageExit = $LASTEXITCODE
    Assert-ReportPredicate -Prefix "grant-setup" -Name "current-kernel-image-built" -Expected "current release kernel packaged into a per-run image without NET-8 pinning" -Passed ($packageExit -eq 0) -Actual $(if ($packageExit -eq 0) { "built" } else { "exit=$packageExit" }) -FailureMessage "Focused grant-reboot image packaging failed"

    $script:PersistDiskImage = Join-Path $RunDir "grant-persist.img"
    $builderResult = Invoke-NativeCommandForReport -FilePath "python" -Arguments @($Builder, "--self-check", "--seed-bootctl", "valid-a", $PersistDiskImage)
    Assert-ReportPredicate -Prefix "grant-setup" -Name "clean-persist-disk-built" -Expected "self-checked GPT persist disk with valid-a bootctl" -Passed ($builderResult.ExitCode -eq 0) -Actual $(if ($builderResult.ExitCode -eq 0) { "built" } else { (@($builderResult.Output) + @($builderResult.Error)) -join [Environment]::NewLine }) -FailureMessage "Focused grant-reboot persist disk build failed"
    $script:PersistDiskImage = Assert-PersistDiskPathSafe -Path $PersistDiskImage
    $initialInspection = Get-PersistInspection -Path $PersistDiskImage
    $initialOk = [bool]$initialInspection.gpt_header_valid -and
        [bool]$initialInspection.gpt_crc_checked -and
        [bool]$initialInspection.data_superblock_valid -and
        $initialInspection.bootctl_read.decision.posture -eq "Normal" -and
        [int]$initialInspection.reclog_scan.count -eq 0 -and
        $initialInspection.reclog_scan.valid_prefix_chain -eq $true -and
        $initialInspection.reclog_scan.full_region_valid -eq $true
    Assert-ReportPredicate -Prefix "grant-setup" -Name "clean-persist-disk-empty-normal" -Expected "valid GPT, Normal bootctl, empty and fully valid RECLOG" -Passed $initialOk -Actual $(Convert-CompactJson $initialInspection 8) -FailureMessage "Focused grant-reboot disk was not clean Normal posture"

    $script:HardwareProfile = New-HardwareProfile -Nic "none" -ScratchDrive $true -AuditRollbackTargetDrive $true -PersistDrive $true

    $script:boot1Vm = Start-RaiosVm -Label "grant-boot1" -PredicatePrefix "grant-boot1" -Port $SerialTcpPort -MonitorPort ($SerialTcpPort + 100) -QmpPort 0 -PersistPath $PersistDiskImage
    $boot1Projection = Invoke-Boot1DurableRevoke -Prefix "grant-boot1"
    Stop-RaiosVmCleanly -Vm $boot1Vm -Name "grant-boot1"

    $boot1Inspection = Get-PersistInspection -Path $PersistDiskImage
    $boot1OuterValid = $boot1Inspection.reclog_scan.status -eq "valid" -and
        [int]$boot1Inspection.reclog_scan.count -eq 2 -and
        $boot1Inspection.reclog_scan.valid_prefix_chain -eq $true -and
        $boot1Inspection.reclog_scan.full_region_valid -eq $true
    Assert-ReportPredicate -Prefix "grant-boot1" -Name "grant-revoke-outer-reclog-valid" -Expected "exactly grant+revoke are durable in a fully valid outer RECLOG" -Passed $boot1OuterValid -Actual $(Convert-CompactJson $boot1Inspection.reclog_scan 8) -FailureMessage "Boot 1 grant/revoke outer RECLOG was not fully valid"
    Copy-Item -LiteralPath $PersistDiskImage -Destination $Boot1PersistSnapshot -Force

    $script:boot2Vm = Start-RaiosVm -Label "grant-boot2" -PredicatePrefix "grant-boot2" -Port ($SerialTcpPort + 1) -MonitorPort ($SerialTcpPort + 101) -QmpPort 0 -PersistPath $PersistDiskImage
    Assert-Boot2DurableRevokeReplay -Prefix "grant-boot2" -Boot1Projection $boot1Projection
    Stop-RaiosVmCleanly -Vm $boot2Vm -Name "grant-boot2"

    Assert-WasmGrantSemanticTamperChild -Port ($SerialTcpPort + 2)
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
    if ($ProgramPersistence) {
        Invoke-ProgramPersistenceProof
        $Result = "passed"
        return
    }

    if ($GrantProjectionOnly) {
        Invoke-GrantProjectionOnlyProof
        $Result = "passed"
        return
    }

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
    $rollbackFixtureTool = Join-Path $env:CARGO_TARGET_DIR "debug\dev-rollback-fixture.exe"
    $rollbackFixtureWasm = Join-Path $RepoRoot "vm-harness\fixtures\rollback-target-log-only.wasm"
    if ($RollbackFixtureMode) {
        & cargo build --locked -p ota-tools --bin dev-rollback-fixture
        $fixtureToolBuildExit = $LASTEXITCODE
        Assert-ReportPredicate -Prefix "boot1" -Name "rollback-fixture-tool-built" -Expected "the fixed offline rollback fixture tool builds with Cargo.lock unchanged" -Passed ($fixtureToolBuildExit -eq 0 -and (Test-Path -LiteralPath $rollbackFixtureTool)) -Actual "exit=$fixtureToolBuildExit tool=$rollbackFixtureTool" -FailureMessage "dev-rollback-fixture build failed"
    }

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
    $builderArgs = @($Builder, "--self-check", "--seed-bootctl", "valid-a")
    if ($RollbackFixtureMode) {
        $builderArgs += @("--seed-dev-rollback-fixture", $rollbackFixtureWasm, "--dev-rollback-fixture-tool", $rollbackFixtureTool)
    }
    $builderArgs += $PersistDiskImage
    $builderResult = Invoke-NativeCommandForReport -FilePath "python" -Arguments $builderArgs
    Assert-ReportPredicate -Prefix "boot1" -Name "kept-persist-disk-built" -Expected "self-check GPT persist disk with valid-a bootctl" -Passed ($builderResult.ExitCode -eq 0) -Actual $(if ($builderResult.ExitCode -eq 0) { "built" } else { (@($builderResult.Output) + @($builderResult.Error)) -join [Environment]::NewLine }) -FailureMessage "Persist disk build failed"
    $PersistDiskImage = Assert-PersistDiskPathSafe -Path $PersistDiskImage
    $initialInspection = Get-PersistInspection -Path $PersistDiskImage
    $fixtureV2Ok = $true
    if ($RollbackFixtureMode) {
        $fixtureFrames = @($initialInspection.reclog_frames)
        $fixtureAuth = @($fixtureFrames | Where-Object { $_.payload_json.record_kind -eq "install_authorization" })[0]
        $fixturePromote = @($fixtureFrames | Where-Object { $_.payload_json.transaction_kind -eq "promote" })[0]
        $fixturePersist = @($fixtureFrames | Where-Object { $_.payload_json.record_kind -eq "artifact_persist" })[0]
        $fixtureV2Ok = [int64]$fixtureAuth.payload_json.install_envelope_version -eq 2 -and
            $fixtureAuth.payload_json.grant_target_schema -eq "raios.grant_target_snapshot.v1" -and
            [int64]$fixtureAuth.payload_json.grant_target_count -eq 1 -and
            $fixtureAuth.payload_json.grant_target_snapshot_sha256 -match '^sha256:[0-9a-f]{64}$' -and
            $fixtureAuth.payload_json.computed_grant_sha256 -eq $fixturePromote.payload_json.computed_grant_hash -and
            $fixtureAuth.payload_json.attestation_reference_sha256 -eq $fixturePromote.payload_json.attestation_reference_hash -and
            $fixturePromote.payload_json.install_authorization_frame_sha256 -eq "sha256:$($fixtureAuth.frame_sha256)" -and
            [int64]$fixturePromote.payload_json.install_envelope_version -eq 2 -and
            $fixturePromote.payload_json.grant_target_snapshot_sha256 -eq $fixtureAuth.payload_json.grant_target_snapshot_sha256 -and
            $fixturePersist.payload_json.service_id -eq $fixtureAuth.payload_json.service_id -and
            $fixturePersist.payload_json.artifact_sha256 -eq $fixtureAuth.payload_json.install_candidate_sha256 -and
            $fixturePersist.payload_json.grant_target_entries -eq "env.log:host_call" -and
            [int64]$fixturePersist.payload_json.grant_target_count -eq 1 -and
            $fixturePersist.payload_json.grant_target_snapshot_sha256 -eq $fixtureAuth.payload_json.grant_target_snapshot_sha256
    }
    $initialOk = (
        [bool]$initialInspection.gpt_header_valid -and
        [bool]$initialInspection.gpt_crc_checked -and
        [bool]$initialInspection.data_superblock_valid -and
        $initialInspection.bootctl_read.decision.posture -eq "Normal" -and
        [int]$initialInspection.reclog_scan.count -eq $(if ($RollbackFixtureMode) { 4 } else { 0 }) -and
        $fixtureV2Ok
    )
    Assert-ReportPredicate -Prefix "boot1" -Name "kept-persist-disk-empty-normal" -Expected $(if ($RollbackFixtureMode) { "valid GPT, Normal bootctl, exact four-record log-only predecessor with independently signed V2 snapshot authority" } else { "valid GPT, Normal bootctl, empty RECLOG" }) -Passed $initialOk -Actual $(Convert-CompactJson $initialInspection 8) -FailureMessage "Initial kept persist disk did not match the selected fixture or signed V2 context"
    if ($RollbackFixtureMode) {
        $InitialRollbackFixtureSnapshot = Join-Path $RunDir "persist-initial-rollback-fixture.img"
        Copy-Item -LiteralPath $PersistDiskImage -Destination $InitialRollbackFixtureSnapshot -Force
    }

    $HardwareProfile = New-HardwareProfile -Nic "e1000" -ScratchDrive $true -AuditRollbackTargetDrive $true -PersistDrive $true

    $boot1Vm = Start-RaiosVm -Label "boot1" -PredicatePrefix "boot1" -Port $SerialTcpPort -MonitorPort ($SerialTcpPort + 100) -QmpPort ($SerialTcpPort + 200) -NetworkBoot -PersistPath $PersistDiskImage
    $activation = @(Invoke-W7M6PhysicalActivation)[-1]
    $install = @(Invoke-SignedGrantedCandidateInstall -Activation $activation -NamePrefix "boot1")[-1]
    Assert-Boot1ActivationAndInstall -Activation $activation -Install $install
    if ($RollbackBoundaryOnly) {
        Stop-RaiosVmCleanly -Vm $boot1Vm -Name "boot1"
        $boot1Vm = $null
        $postBoot1Inspection = Get-PersistInspection -Path $PersistDiskImage
        Assert-RollbackSourceGrantFrames -Inspection $postBoot1Inspection -Activation $activation -Install $install
        Copy-Item -LiteralPath $PersistDiskImage -Destination $Boot1PersistSnapshot -Force
        if ($RollbackBoundaryPhase -in @("all", "intent")) {
            Invoke-RollbackBoundaryCase -Phase "intent" -Port ($SerialTcpPort + 20)
        }
        if ($RollbackBoundaryPhase -in @("all", "revoke")) {
            Invoke-RollbackBoundaryCase -Phase "revoke" -Port ($SerialTcpPort + 30)
        }
        if ($RollbackBoundaryPhase -in @("all", "commit")) {
            Invoke-RollbackBoundaryCase -Phase "commit" -Port ($SerialTcpPort + 40)
        }
        $Result = "passed"
        return
    }
    if ($RollbackGrantDeltaOnly) {
        Stop-RaiosVmCleanly -Vm $boot1Vm -Name "boot1"
        $boot1Vm = $null
        $postBoot1Inspection = Get-PersistInspection -Path $PersistDiskImage
        Assert-RollbackSourceGrantFrames -Inspection $postBoot1Inspection -Activation $activation -Install $install
        Copy-Item -LiteralPath $PersistDiskImage -Destination $Boot1PersistSnapshot -Force

        $boot2Vm = Start-RaiosVm -Label "rollback-boot" -PredicatePrefix "rollback" -Port ($SerialTcpPort + 1) -MonitorPort ($SerialTcpPort + 101) -QmpPort 0 -PersistPath $PersistDiskImage
        $null = Assert-Boot2Autoload -Activation $activation -Install $install -Boot1Inspection $postBoot1Inspection
        $rollback = Invoke-RollbackGrantDeltaApply
        Stop-RaiosVmCleanly -Vm $boot2Vm -Name "rollback-boot"
        $boot2Vm = $null
        Assert-RollbackGrantDeltaHostOrdering -Inspection (Get-PersistInspection -Path $PersistDiskImage)

        $boot3Vm = Start-RaiosVm -Label "rollback-reboot" -PredicatePrefix "reboot" -Port ($SerialTcpPort + 2) -MonitorPort ($SerialTcpPort + 102) -QmpPort 0 -PersistPath $PersistDiskImage
        Assert-RollbackTargetReboot
        Stop-RaiosVmCleanly -Vm $boot3Vm -Name "rollback-reboot"
        $boot3Vm = $null

        Assert-RollbackFixtureMutationDenied -Label "rollback-snapshot-reseal" -MutationFlag "--reseal-rollback-target-snapshot" -ExpectedReason "signed_snapshot_context_mismatch" -Port ($SerialTcpPort + 10)
        Assert-RollbackFixtureMutationDenied -Label "rollback-snapshot-bitflip" -MutationFlag "--tamper-rollback-target-snapshot" -Port ($SerialTcpPort + 11)
        Assert-RollbackFixtureMutationDenied -Label "rollback-artstor-link-tamper" -MutationFlag "--tamper-rollback-target-artstor-link" -Port ($SerialTcpPort + 12)
        Assert-RollbackFixtureMutationDenied -Label "rollback-signature-tamper" -MutationFlag "--tamper-rollback-target-signature" -Port ($SerialTcpPort + 13)
        Assert-RollbackFixtureMutationDenied -Label "rollback-authenticated-install-duplicate" -MutationFlag "--duplicate-authenticated-install" -ExpectedReason "rollback_target_authenticated_order_duplicate" -Port ($SerialTcpPort + 14)
        Assert-RollbackRecoveryMutationDenied -Label "rollback-forged-unchanged-projection" -MutationFlag "--forge-rollback-unchanged-projection" -ExpectedReason "rollback_transition_delta_count_mismatch" -Port ($SerialTcpPort + 15)
        Assert-RollbackRecoveryMutationDenied -Label "rollback-mismatched-pair" -MutationFlag "--mismatch-rollback-intent-commit" -ExpectedReason "rollback_recovery_pair_mismatch" -Port ($SerialTcpPort + 16)
        $Result = "passed"
        return
    }
    $boot1CapProjection = Invoke-Boot1DurableRevoke -Prefix "boot1"
    $boot1MemoryScan = Invoke-Boot1DurableMemoryWrites -Prefix "boot1"
    Stop-RaiosVmCleanly -Vm $boot1Vm -Name "boot1"

    $postBoot1Inspection = Get-PersistInspection -Path $PersistDiskImage
    $postBoot1Records = @($postBoot1Inspection.artifact_persist_records)
    Assert-Boot1HostProvenance -Inspection $postBoot1Inspection -Activation $activation -Install $install
    Copy-Item -LiteralPath $PersistDiskImage -Destination $Boot1PersistSnapshot -Force

    $boot2Vm = Start-RaiosVm -Label "boot2" -PredicatePrefix "boot2" -Port ($SerialTcpPort + 1) -MonitorPort ($SerialTcpPort + 101) -QmpPort 0 -PersistPath $PersistDiskImage
    $null = Assert-Boot2Autoload -Activation $activation -Install $install -Boot1Inspection $postBoot1Inspection
    Assert-Boot2DurableRevokeReplay -Prefix "boot2" -Boot1Projection $boot1CapProjection
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
    Assert-WasmGrantSemanticTamperChild -Port ($SerialTcpPort + 16)

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
