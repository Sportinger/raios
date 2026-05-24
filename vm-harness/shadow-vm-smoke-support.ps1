# Support functions for shadow-vm-smoke.ps1.
# This file is dot-sourced and intentionally uses variables from the caller script scope.

function Resolve-OptionalPath {
    param([string]$Path)
    if (-not $Path) {
        return $null
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Path does not exist: $Path"
    }
    return (Resolve-Path -LiteralPath $Path).Path
}

function Get-FileSha256OrNull {
    param([string]$Path)
    if (-not $Path) {
        return $null
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextSha256 {
    param([string]$Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $sha.ComputeHash($bytes)
        return ([BitConverter]::ToString($hash) -replace "-", "").ToLowerInvariant()
    }
    finally {
        $sha.Dispose()
    }
}

function ConvertTo-ReportJson {
    param([object]$Value)
    return ($Value | ConvertTo-Json -Depth 20 -Compress)
}

function Get-NullablePath {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }
    return $Path
}

function Get-SerialLogContent {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }

    try {
        $item = Get-Item -LiteralPath $Path -ErrorAction Stop
    }
    catch {
        return $null
    }

    $length = [int64]$item.Length
    $writeTicks = [int64]$item.LastWriteTimeUtc.Ticks
    if (
        $script:SerialLogCachePath -eq $Path -and
        $script:SerialLogCacheLength -eq $length -and
        $script:SerialLogCacheWriteTicks -eq $writeTicks -and
        $null -ne $script:SerialLogCacheContent
    ) {
        return $script:SerialLogCacheContent
    }

    $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
    if ($null -eq $content) {
        return $null
    }

    $script:SerialLogCachePath = $Path
    $script:SerialLogCacheLength = $length
    $script:SerialLogCacheWriteTicks = $writeTicks
    $script:SerialLogCacheContent = $content
    return $content
}

function Get-SerialLogTail {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        return "serial log not created"
    }

    $content = Get-SerialLogContent -Path $Path
    if ($null -eq $content) {
        return "serial log unreadable"
    }

    $limit = 1600
    if ($content.Length -le $limit) {
        return $content
    }
    return $content.Substring($content.Length - $limit)
}

function New-HardwareProfile {
    param([string]$Nic)

    $networkDevice = if ($Nic -eq "e1000") {
        "e1000_user"
    }
    else {
        "none"
    }

    return [ordered]@{
        profile = "raios.shadow_vm.q35_xhci.v0"
        machine = "q35"
        memory = "512M"
        cpu = "max"
        firmware = "edk2-x86_64"
        boot_drive = "ide_raw_image"
        display = "none"
        serial = "tcp_chardev_with_log"
        input = @(
            "qemu-xhci",
            "usb-kbd",
            "usb-tablet"
        )
        network = $networkDevice
    }
}

function Wait-ForLogText {
    param(
        [string]$Path,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        $content = Get-SerialLogContent -Path $Path
        if ($null -ne $content -and $content -clike "*$Needle*") {
            return $true
        }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)

    return $false
}

function Get-SerialLogOffset {
    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -ne $content) {
        return [int64]$content.Length
    }
    return [int64]0
}

function Wait-ForLogTextAfterOffset {
    param(
        [string]$Path,
        [string]$Needle,
        [int64]$Offset,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        Drain-SerialTcpOutput -Stream $script:SerialTcpDrainStream
        $content = Get-SerialLogContent -Path $Path
        if ($null -ne $content) {
            $start = [int64]$Offset
            if ($start -lt 0) {
                $start = [int64]0
            }
            if ($start -gt [int64]$content.Length) {
                $start = [int64]$content.Length
            }
            $after = if ($start -eq [int64]0) {
                $content
            }
            elseif ($start -lt [int64]$content.Length) {
                $content.Substring([int]$start)
            }
            else {
                ""
            }
            if ($after -clike "*$Needle*") {
                return $true
            }
        }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)

    return $false
}

function Drain-SerialTcpOutput {
    param(
        [System.Net.Sockets.NetworkStream]$Stream
    )

    if ($null -eq $Stream) {
        return
    }

    $buffer = New-Object byte[] 4096
    try {
        while ($Stream.DataAvailable) {
            $null = $Stream.Read($buffer, 0, $buffer.Length)
        }
    }
    catch {
        return
    }
}

function Add-Predicate {
    param(
        [string]$Name,
        [string]$Expected,
        [bool]$Passed,
        [string]$Actual = ""
    )

    $Predicates.Add([ordered]@{
        name = $Name
        expected = $Expected
        passed = $Passed
        actual = $Actual
    }) | Out-Null

    if (-not $Passed) {
        $Failures.Add($Name) | Out-Null
    }
}

function Assert-LogContains {
    param(
        [string]$Name,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $passed = Wait-ForLogText -Path $SerialLog -Needle $Needle -TimeoutSeconds $TimeoutSeconds
    $actual = if ($passed) { "found" } else { Get-SerialLogTail -Path $SerialLog }
    Add-Predicate -Name $Name -Expected "serial_contains:$Needle" -Passed $passed -Actual $actual
    if (-not $passed) {
        throw "Timed out waiting for '$Needle' in $SerialLog"
    }
}

function Assert-LogContainsFields {
    param(
        [string]$NamePrefix,
        [object[]]$Fields,
        [int]$TimeoutSeconds
    )

    foreach ($field in $Fields) {
        Assert-LogContains -Name "$NamePrefix$($field.Suffix)" -Needle $field.Needle -TimeoutSeconds $TimeoutSeconds
    }
}

function Assert-LogDoesNotContain {
    param(
        [string]$Name,
        [string]$Needle
    )

    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -eq $content) {
        $content = ""
    }
    $passed = -not ($content.Contains($Needle))
    $actual = if ($passed) { "absent" } else { "found" }
    Add-Predicate -Name $Name -Expected "serial_not_contains:$Needle" -Passed $passed -Actual $actual
    if (-not $passed) {
        throw "Unexpected '$Needle' in $SerialLog"
    }
}

function Write-SerialTcpText {
    param(
        [System.Net.Sockets.NetworkStream]$Stream,
        [string]$Text
    )

    $bytes = [System.Text.Encoding]::ASCII.GetBytes($Text)
    $chunkSize = [Math]::Max(1, $SerialWriteChunkSize)
    for ($offset = 0; $offset -lt $bytes.Length; $offset += $chunkSize) {
        $count = [Math]::Min($chunkSize, $bytes.Length - $offset)
        $Stream.Write($bytes, $offset, $count)
        if ($SerialWriteDelayMilliseconds -gt 0) {
            Start-Sleep -Milliseconds $SerialWriteDelayMilliseconds
        }
    }
}

function Send-SerialText {
    param(
        [int]$Port,
        [string]$Text,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    $client = $null
    while ([DateTime]::UtcNow -lt $deadline) {
        $candidate = [System.Net.Sockets.TcpClient]::new()
        $candidate.NoDelay = $true
        try {
            $connect = $candidate.BeginConnect("127.0.0.1", $Port, $null, $null)
            if ($connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromMilliseconds(500))) {
                $candidate.EndConnect($connect)
                $client = $candidate
                break
            }
        }
        catch {
            $candidate.Close()
            Start-Sleep -Milliseconds 100
            continue
        }
        $candidate.Close()
        Start-Sleep -Milliseconds 100
    }
    if (-not $client) {
        throw "Timed out connecting to QEMU serial TCP port $Port"
    }

    try {
        $stream = $client.GetStream()
        Write-SerialTcpText -Stream $stream -Text $Text
        $stream.Flush()
        Start-Sleep -Milliseconds 250
    }
    finally {
        $client.Close()
    }
}

function Send-AgentCommand {
    param(
        [string]$Command,
        [string]$ExpectedMarker,
        [string]$Name = ""
    )

    $commandStartedAt = [DateTime]::UtcNow
    $startOffset = Get-SerialLogOffset
    $predicateName = if ($Name.Length -gt 0) { $Name } else { "command:$Command" }
    $passed = $false
    $sent = $false
    $client = $null
    $stream = $null
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
        while ([DateTime]::UtcNow -lt $deadline) {
            $candidate = [System.Net.Sockets.TcpClient]::new()
            $candidate.NoDelay = $true
            try {
                $connect = $candidate.BeginConnect("127.0.0.1", $SerialTcpPort, $null, $null)
                if ($connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromMilliseconds(500))) {
                    $candidate.EndConnect($connect)
                    $client = $candidate
                    break
                }
            }
            catch {
                $candidate.Close()
                Start-Sleep -Milliseconds 100
                continue
            }
            $candidate.Close()
            Start-Sleep -Milliseconds 100
        }
        if (-not $client) {
            throw "Timed out connecting to QEMU serial TCP port $SerialTcpPort"
        }

        $stream = $client.GetStream()
        $script:SerialTcpDrainStream = $stream
        Write-SerialTcpText -Stream $stream -Text "$Command`r"
        $sent = $true
        $stream.Flush()
        Start-Sleep -Milliseconds 50

        $passed = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $ExpectedMarker -Offset $startOffset -TimeoutSeconds $TimeoutSeconds
        $actual = if ($passed) { "found_after_offset:$startOffset" } else { Get-SerialLogTail -Path $SerialLog }
        Add-Predicate -Name $predicateName -Expected "serial_contains_after_offset:$ExpectedMarker" -Passed $passed -Actual $actual
        if (-not $passed) {
            throw "Timed out waiting for '$ExpectedMarker' in $SerialLog after offset $startOffset"
        }
    }
    finally {
        $commandEndedAt = [DateTime]::UtcNow
        if ($sent) {
            $ExecutedCommands.Add([ordered]@{
                command = $Command
                name = $predicateName
                expected_marker = $ExpectedMarker
                response_offset = $startOffset
                duration_ms = ([int][Math]::Round(($commandEndedAt - $commandStartedAt).TotalMilliseconds))
                sent = $true
                passed = $passed
            }) | Out-Null
        }
        $script:SerialTcpDrainStream = $null
        if ($null -ne $stream) {
            Drain-SerialTcpOutput -Stream $stream
        }
        if ($null -ne $client) {
            $client.Close()
        }
    }
}

function Get-LastAgentResponseJson {
    param(
        [string]$Method
    )

    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -eq $content) {
        throw "No serial log content found in $SerialLog"
    }
    $begin = "RAIOS_AGENT_BEGIN $Method"
    $end = "RAIOS_AGENT_END $Method"
    $beginIndex = $content.LastIndexOf($begin, [System.StringComparison]::Ordinal)
    if ($beginIndex -lt 0) {
        throw "No agent response for method '$Method' found in $SerialLog"
    }
    $jsonStart = $content.IndexOf("{", $beginIndex, [System.StringComparison]::Ordinal)
    $endIndex = $content.IndexOf($end, $jsonStart, [System.StringComparison]::Ordinal)
    if ($jsonStart -lt 0 -or $endIndex -lt 0) {
        throw "Incomplete agent response for method '$Method' found in $SerialLog"
    }
    $json = $content.Substring($jsonStart, $endIndex - $jsonStart).Trim()
    return $json | ConvertFrom-Json
}

function Assert-CurrentBootEventId {
    param(
        [string]$Name,
        [string]$Value
    )

    $passed = $Value -match '^event\.current_boot\.[0-9]{8}$'
    Add-Predicate -Name $Name -Expected "current_boot_event_id" -Passed $passed -Actual $Value
    if (-not $passed) {
        throw "Expected current-boot event id for '$Name', got '$Value'"
    }
}

function Write-Report {
    param(
        [string]$FinalResult,
        [string]$ResolvedImage,
        [string]$ResolvedArtifact,
        [string]$ResolvedManifest,
        [string[]]$QemuArgList,
        [object]$HardwareProfile,
        [DateTime]$StartedAt
    )

    New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null

    $serialHash = Get-FileSha256OrNull -Path $SerialLog
    $errLog = [System.IO.Path]::ChangeExtension($SerialLog, ".err.txt")
    $endedAt = [DateTime]::UtcNow
    $networkMode = if ($Network) { "e1000_user_enabled" } else { "disabled" }
    $baseImageSha256 = Get-FileSha256OrNull -Path $ResolvedImage
    $artifactSha256 = Get-FileSha256OrNull -Path $ResolvedArtifact
    $manifestSha256 = Get-FileSha256OrNull -Path $ResolvedManifest
    $qemuArgsCanonical = ConvertTo-ReportJson -Value @($QemuArgList)
    $qemuArgsSha256 = Get-TextSha256 -Text $qemuArgsCanonical
    $hardwareProfileSha256 = Get-TextSha256 -Text (ConvertTo-ReportJson -Value $HardwareProfile)
    $executedCommandDetails = @($ExecutedCommands.ToArray())
    $executedCommandNames = @($executedCommandDetails | ForEach-Object { $_.command })

    $report = [ordered]@{
        schema = "raios.vm_test_report.v0"
        result = $FinalResult
        profile = $Profile
        generated_at_utc = ($endedAt.ToString("o"))
        started_at_utc = ($StartedAt.ToString("o"))
        duration_ms = ([int][Math]::Round(($endedAt - $StartedAt).TotalMilliseconds))
        run_id = $RunId
        sandbox_policy = [ordered]@{
            hypervisor = "qemu-system-x86_64"
            headless = $true
            shared_folders = "none"
            host_filesystem_mounts = "none"
            network = $networkMode
            boot_media = "temporary_or_explicit_image"
            qemu_killed_after_run = $true
        }
        base_image = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedImage)
            sha256 = $baseImageSha256
            temporary = $TempImage
        }
        candidate_artifact = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedArtifact)
            sha256 = $artifactSha256
        }
        candidate_manifest = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedManifest)
            sha256 = $manifestSha256
            validation = $ManifestValidation
        }
        hardware_profile = $HardwareProfile
        qemu = [ordered]@{
            script = $RunScript
            args = @($QemuArgList)
            args_canonical_json = $qemuArgsCanonical
            args_sha256 = $qemuArgsSha256
            serial_tcp_port = $SerialTcpPort
            serial_write = [ordered]@{
                chunk_bytes = $SerialWriteChunkSize
                inter_chunk_delay_ms = $SerialWriteDelayMilliseconds
            }
            pid = $QemuPid
        }
        evidence_binding = [ordered]@{
            base_image_sha256 = $baseImageSha256
            candidate_artifact_sha256 = $artifactSha256
            candidate_manifest_sha256 = $manifestSha256
            hardware_profile_sha256 = $hardwareProfileSha256
            qemu_args_sha256 = $qemuArgsSha256
            serial_log_sha256 = $serialHash
            predicate_count = $Predicates.Count
            predicate_passed_count = @($Predicates.ToArray() | Where-Object { $_.passed }).Count
            predicate_failed_count = @($Predicates.ToArray() | Where-Object { -not $_.passed }).Count
            result = $FinalResult
        }
        commands = $executedCommandNames
        executed_commands = $executedCommandDetails
        predicates = @($Predicates.ToArray())
        serial_log = [ordered]@{
            path = $SerialLog
            sha256 = $serialHash
        }
        stderr_log = [ordered]@{
            path = $errLog
            sha256 = (Get-FileSha256OrNull -Path $errLog)
        }
        failures = @($Failures.ToArray())
    }

    $json = $report | ConvertTo-Json -Depth 20
    Set-Content -LiteralPath $ReportPath -Value $json -Encoding UTF8
    $reportHash = Get-FileSha256OrNull -Path $ReportPath
    Set-Content -LiteralPath $ReportHashPath -Value "$reportHash  $ReportPath" -Encoding ASCII
}

