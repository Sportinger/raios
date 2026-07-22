[CmdletBinding(DefaultParameterSetName = "Dispatch")]
param(
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$OrderPath = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$ExpectedMachineId = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$ExpectedManifestSha256 = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string[]]$RequiredFactPath = @(),
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$Sandbox = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$ReportPath = "",
    [Parameter(ParameterSetName = "Dispatch")][AllowEmptyString()][string]$GovernanceException = "",
    [Parameter(Mandatory = $true, ParameterSetName = "SelfTest")][switch]$SelfTest
)

$ErrorActionPreference = "Stop"
$LauncherRepoRoot = Split-Path -Parent $PSScriptRoot
$LauncherSelfTest = [bool]$SelfTest
$MachineManifestMaxBytes = 65536 # Fixed whole-file ceiling; the reader allocates one extra detection byte.
$LaneOrderMaxBytes = 65536       # Fixed whole-file ceiling for the authored lane order.
$MachineContextMaxChars = 16384
$ReservedMachineContextToken = "raios-machine-context"
$Adr0045ExceptionToken = "ADR-0045-H26"
$Adr0045R2ExceptionToken = "ADR-0045-H26-R2"
$Adr0045R3ExceptionToken = "ADR-0045-H26-R3"
$Adr0045R3RecoveryToken = "ADR-0045-H26-R3-DISPATCH-RECOVERY-1"
$Adr0045MachineId = "surface-pro-4"
$Adr0045ManifestSha256 = "08c8d977f48f5a846edecaf31cc4d205291105dc5c821960df21621e17b36189"
$Adr0045FactPath = "/devices/2/identity"
$Adr0045FactValue = "Marvell 88W8897"
$Adr0045OrderMarker = "Governance exception: ADR-0045-H26"
$Adr0045R2OrderMarker = "Governance exception: ADR-0045-H26-R2"
$Adr0045R3OrderMarker = "Governance exception: ADR-0045-H26-R3"
$Adr0045DecisionRelativePath = "docs/architecture/decisions/0045-authorize-one-h26-surface-development-test.md"
$Adr0045DecisionSha256 = "a190c50925cb3e15659384146529851e1d2ca26b7692584dd16f46effc439061"
$Adr0045R2DecisionSha256 = "71c6eb3b177738b3b8a338d699b2b1e0a277323926d7c0c8d857b31187f37e84"
$Adr0045R3DecisionSha256 = "3f3b0c42b7d1f48fc19381bf5a2d0eb9ca0857c0c94cde7a77e8674658e0506e"
$Adr0045R3RecoveryDecisionSha256 = "d1ab00cc939c307b1e763a4afd868cf34de9c740300b24f0c532bf90292bba8b"
$Adr0045ClaimRelativePath = "target\state\adr0045-h26.claim"
$Adr0045R2ClaimRelativePath = "target\state\adr0045-h26-r2.claim"
$Adr0045R3ClaimRelativePath = "target\state\adr0045-h26-r3.claim"
$Adr0045R3RecoveryClaimRelativePath = "target\state\adr0045-h26-r3-dispatch-recovery-1.claim"
$Adr0045R3RecoveryOrderSha256 = "0ef25b8ce5fdefe15790ec83ff7803c3597b4f9933008fc27674367f18172585"
$Adr0045R3FailureLogSha256 = "4d8bf26aecef9ff08ef78485691f81909a6b03dd762cb8510492c26bb083e7c8"
$Adr0045R3ClaimSha256 = "b32ef3b8e56f3eba19e846160cf84ffe7c014b6933126d8e766c7d1c03bcf11d"
$Adr0045R3RecoveryLauncherCommit = "4c77bdaf03a42ab0e543ca389e1310d7bcf5baf2"
$Adr0045R3RecoveryLauncherSha256 = "2124404d8a7616767d87f1260b8f5fe61d6afb6c73419022567337e483ad700d"
$Adr0045R3RecoveryShimPath = "C:\Users\admin\AppData\Roaming\npm\codex.ps1"
$Adr0045R3RecoveryShimSha256 = "0c149db80ed0bf442c810146b0ad0163b74982fe4542d673f56c354d7b8229cb"
$Adr0045R3RecoveryHostPath = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
$Adr0045R3RecoveryHostSha256 = "7600ffe12da441fe89d035b13801e8e91d064bc544a27b19a5cf49f6ab8b18f5"
. (Join-Path $PSScriptRoot "check-hardware-manifests.ps1")

function Throw-LaneGateDenial {
    param([string]$Reason, [string]$Message, [AllowNull()][object]$Details = $null, [bool]$ChildStarted = $false)
    $exception = New-Object System.InvalidOperationException($Message)
    $exception.Data["reason"] = $Reason
    $exception.Data["child_started"] = $ChildStarted
    if ($null -ne $Details) { $exception.Data["details"] = $Details }
    throw $exception
}

function Get-Sha256Hex {
    param([byte[]]$Bytes, [int]$Count = -1)
    if ($Count -lt 0) { $Count = $Bytes.Length }
    if ($Count -gt $Bytes.Length) { throw "SHA-256 byte count exceeds the supplied buffer" }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($Bytes, 0, $Count))).Replace("-", "").ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Read-Utf8Text {
    param([byte[]]$Bytes, [int]$Count, [string]$Reason, [string]$Label)
    try {
        $encoding = New-Object System.Text.UTF8Encoding($false, $true)
        $text = $encoding.GetString($Bytes, 0, $Count)
        if ($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) { $text = $text.Substring(1) }
        return $text
    } catch { Throw-LaneGateDenial $Reason "$Label is not valid UTF-8" $_.Exception.Message }
}

function Read-BoundedFileBytes {
    param(
        [string]$Path, [int64]$MaxBytes, [string]$TooLargeReason, [string]$ReadFailureReason, [string]$Label,
        [AllowNull()][scriptblock]$ReaderFactory = $null
    )
    if ($MaxBytes -lt 0 -or $MaxBytes -ge [int]::MaxValue) { throw "Unsupported bounded-reader ceiling '$MaxBytes'" }
    $capacity = [int]($MaxBytes + 1); [byte[]]$buffer = [byte[]]::new($capacity)
    $reader = $null; $readFailure = $null; $bytesRead = 0
    try {
        # FileShare.Read denies writers and delete/replacement while this handle
        # is alive on platforms that enforce sharing modes. The path is opened
        # once; no metadata length is trusted and no reopen occurs.
        $reader = if ($null -ne $ReaderFactory) { & $ReaderFactory $Path } else {
            [IO.FileStream]::new($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read, 4096, [IO.FileOptions]::SequentialScan)
        }
        while ($bytesRead -lt $capacity) {
            $requested = [Math]::Min(4096, $capacity - $bytesRead)
            $read = [int]$reader.Read($buffer, $bytesRead, $requested)
            if ($read -lt 0 -or $read -gt $requested) { throw "Reader returned invalid byte count '$read' for request '$requested'" }
            if ($read -eq 0) { break }
            $bytesRead += $read
        }
    } catch { $readFailure = $_.Exception }
    finally {
        if ($null -ne $reader) {
            try { $reader.Dispose() }
            catch { if ($null -eq $readFailure) { $readFailure = $_.Exception } }
        }
    }
    if ($null -ne $readFailure) { Throw-LaneGateDenial $ReadFailureReason "$Label could not be read through its bounded handle" $readFailure.Message }
    if ($bytesRead -gt $MaxBytes) {
        Throw-LaneGateDenial $TooLargeReason "$Label exceeds the fixed $MaxBytes-byte ceiling" ([ordered]@{
            max_bytes = $MaxBytes; observed_at_least_bytes = $bytesRead; allocated_bytes = $capacity
        })
    }
    return [pscustomobject]@{ buffer = $buffer; count = $bytesRead; allocated_bytes = $capacity }
}

function Get-AbsolutePath {
    param([string]$Path)
    if ([IO.Path]::IsPathRooted($Path)) { return [IO.Path]::GetFullPath($Path) }
    return [IO.Path]::GetFullPath((Join-Path (Get-Location).Path $Path))
}

function ConvertTo-NativeArgument {
    param([string]$Value)
    if ($Value.Length -eq 0) { return '""' }
    if ($Value -notmatch '[\s"]') { return $Value }
    $quoted = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $quoted = [regex]::Replace($quoted, '(\\+)$', '$1$1')
    return '"' + $quoted + '"'
}

function Resolve-CodexChildStartPlan {
    param([string]$ChildCommand, [string[]]$ChildArguments, [string]$PowerShellHostPathOverride = "")
    try { $resolved = @(Get-Command $ChildCommand -ErrorAction Stop)[0] }
    catch { Throw-LaneGateDenial "codex_command_missing" "Codex child command is unavailable" $ChildCommand }

    $commandPath = [string]$resolved.Path
    if ([string]::IsNullOrWhiteSpace($commandPath)) { $commandPath = [string]$resolved.Source }
    try { $commandPath = [IO.Path]::GetFullPath($commandPath) }
    catch { Throw-LaneGateDenial "codex_command_unsupported" "Codex child command does not resolve to a canonical file" $ChildCommand }
    $extension = [IO.Path]::GetExtension($commandPath).ToLowerInvariant()

    if ($resolved.CommandType -eq [Management.Automation.CommandTypes]::Application -and $extension -ceq ".exe") {
        return [pscustomobject]@{ file_name = $commandPath; arguments = [string[]]@($ChildArguments); command_type = [string]$resolved.CommandType; command_path = $commandPath }
    }
    if ($resolved.CommandType -ne [Management.Automation.CommandTypes]::ExternalScript -or $extension -cne ".ps1") {
        Throw-LaneGateDenial "codex_command_unsupported" "Codex child command must resolve to a native .exe or a PowerShell .ps1 shim" ([ordered]@{ command_type = [string]$resolved.CommandType; path = $commandPath })
    }

    $hostCandidate = if ($PowerShellHostPathOverride) { $PowerShellHostPathOverride } elseif ($PSVersionTable.PSEdition -ceq "Core") {
        Join-Path $PSHOME "pwsh.exe"
    } else { Join-Path $PSHOME "powershell.exe" }
    try { $hostPath = [IO.Path]::GetFullPath($hostCandidate) }
    catch { Throw-LaneGateDenial "codex_powershell_host_missing" "Native PowerShell host for the Codex shim is unavailable" $hostCandidate }
    if ([IO.Path]::GetExtension($hostPath).ToLowerInvariant() -cne ".exe" -or -not [IO.File]::Exists($hostPath)) {
        Throw-LaneGateDenial "codex_powershell_host_missing" "Native PowerShell host for the Codex shim is unavailable" $hostPath
    }
    try { [byte[]]$scriptBytes = [IO.File]::ReadAllBytes($commandPath) }
    catch { Throw-LaneGateDenial "codex_command_read_failed" "PowerShell Codex shim could not be snapshotted" $_.Exception.Message }
    $scriptSha256 = Get-Sha256Hex $scriptBytes
    $scriptBase64 = [Convert]::ToBase64String($scriptBytes)
    $scriptDirectoryBase64 = [Convert]::ToBase64String((New-Object Text.UTF8Encoding($false)).GetBytes((Split-Path -Parent $commandPath)))
    $payloadJson = [ordered]@{ script_path = $commandPath; script_sha256 = $scriptSha256; script_base64 = $scriptBase64; script_directory_base64 = $scriptDirectoryBase64; arguments = [string[]]@($ChildArguments) } | ConvertTo-Json -Compress
    $payloadBase64 = [Convert]::ToBase64String((New-Object Text.UTF8Encoding($false)).GetBytes($payloadJson))
    $bootstrap = '$utf8=New-Object Text.UTF8Encoding($false);[Console]::InputEncoding=$utf8;[Console]::OutputEncoding=$utf8;$OutputEncoding=$utf8;$json=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String("' + $payloadBase64 + '"));$data=$json|ConvertFrom-Json;$argv=[string[]]$data.arguments;$script=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String([string]$data.script_base64));$dir64=[string]$data.script_directory_base64;$needle=''$basedir=Split-Path $MyInvocation.MyCommand.Definition -Parent'';$replacement=''$basedir=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String("''+$dir64+''"))'';$script=$script.Replace($needle,$replacement);$block=[ScriptBlock]::Create($script);$raw=[Console]::In.ReadToEnd();if(-not $raw.EndsWith("`r`n")){exit 125};$raw.Substring(0,$raw.Length-2)|& $block @argv;exit $LASTEXITCODE'
    $encodedCommand = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($bootstrap))
    return [pscustomobject]@{
        file_name = $hostPath
        arguments = [string[]]@("-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", $encodedCommand)
        command_type = [string]$resolved.CommandType
        command_path = $commandPath
        command_sha256 = $scriptSha256
    }
}

function Assert-LaneGateInputs {
    param([string]$Order, [string]$MachineId, [string]$Digest, [string[]]$FactPaths, [string]$SandboxName, [string]$Report)
    if ([string]::IsNullOrWhiteSpace($Order)) { Throw-LaneGateDenial "order_path_missing" "OrderPath is required" }
    if ($MachineId -cnotmatch '^[a-z0-9][a-z0-9._-]*$') { Throw-LaneGateDenial "expected_machine_id_invalid" "ExpectedMachineId is not a stable machine id" }
    if ($Digest -cnotmatch '^[0-9a-fA-F]{64}$') { Throw-LaneGateDenial "expected_digest_invalid" "ExpectedManifestSha256 must be 64 hexadecimal characters" }
    if (@($FactPaths).Count -eq 0 -or @($FactPaths | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -gt 0) { Throw-LaneGateDenial "required_fact_paths_missing" "At least one non-empty RequiredFactPath is required" }
    if (@($FactPaths).Count -gt 128 -or @($FactPaths | Where-Object { $_.Length -gt 256 }).Count -gt 0) { Throw-LaneGateDenial "required_fact_paths_too_large" "RequiredFactPath is limited to 128 values of at most 256 characters" }
    $seenFactPaths = New-Object 'System.Collections.Generic.HashSet[string]' ([StringComparer]::Ordinal)
    foreach ($path in @($FactPaths)) { if (-not $seenFactPaths.Add($path)) { Throw-LaneGateDenial "required_fact_paths_duplicate" "RequiredFactPath values must be ordinally unique" } }
    if ($SandboxName -cnotin @("read-only", "workspace-write")) { Throw-LaneGateDenial "sandbox_invalid" "Sandbox must be read-only or workspace-write" }
    if ([string]::IsNullOrWhiteSpace($Report)) { Throw-LaneGateDenial "report_path_missing" "ReportPath is required" }
}

function Test-TextContainsExactLine {
    param([string]$Text, [string]$ExpectedLine)
    $reader = New-Object IO.StringReader($Text)
    try {
        while ($null -ne ($line = $reader.ReadLine())) {
            if ($line -ceq $ExpectedLine) { return $true }
        }
        return $false
    } finally { $reader.Dispose() }
}

function Get-Adr0045ExceptionBinding {
    param([string]$Token)
    if ($Token -ceq $Adr0045ExceptionToken) {
        return [pscustomobject]@{
            token = $Adr0045ExceptionToken
            order_marker = $Adr0045OrderMarker
            decision_sha256 = $Adr0045DecisionSha256
            claim_relative_path = $Adr0045ClaimRelativePath
            claim_schema = "raios.adr0045_h26_claim.v1"
        }
    }
    if ($Token -ceq $Adr0045R2ExceptionToken) {
        return [pscustomobject]@{
            token = $Adr0045R2ExceptionToken
            order_marker = $Adr0045R2OrderMarker
            decision_sha256 = $Adr0045R2DecisionSha256
            claim_relative_path = $Adr0045R2ClaimRelativePath
            claim_schema = "raios.adr0045_h26_r2_claim.v1"
        }
    }
    if ($Token -ceq $Adr0045R3ExceptionToken) {
        return [pscustomobject]@{
            token = $Adr0045R3ExceptionToken
            order_marker = $Adr0045R3OrderMarker
            decision_sha256 = $Adr0045R3DecisionSha256
            claim_relative_path = $Adr0045R3ClaimRelativePath
            claim_schema = "raios.adr0045_h26_r3_claim.v1"
        }
    }
    if ($Token -ceq $Adr0045R3RecoveryToken) {
        return [pscustomobject]@{
            token = $Adr0045R3RecoveryToken
            order_marker = $Adr0045R3OrderMarker
            decision_sha256 = $Adr0045R3RecoveryDecisionSha256
            claim_relative_path = $Adr0045R3RecoveryClaimRelativePath
            claim_schema = "raios.adr0045_h26_r3_dispatch_recovery_1_claim.v1"
            is_recovery = $true
        }
    }
    Throw-LaneGateDenial "governance_exception_invalid" "GovernanceException is not an exact authorized ADR-0045 token" $Token
}

function Open-Adr0045AuthorityLease {
    param([string]$Path, [string]$ExpectedDigest, [string]$Token)
    $capacity = [int]($LaneOrderMaxBytes + 1)
    [byte[]]$buffer = [byte[]]::new($capacity)
    $stream = $null; $readFailure = $null; $bytesRead = 0
    try {
        # FileShare.Read admits only other readers: writes, replacement, and
        # deletion remain denied for the lifetime of this exact FileStream.
        $stream = [IO.FileStream]::new($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read, 4096, [IO.FileOptions]::SequentialScan)
        while ($bytesRead -lt $capacity) {
            $requested = [Math]::Min(4096, $capacity - $bytesRead)
            $read = [int]$stream.Read($buffer, $bytesRead, $requested)
            if ($read -lt 0 -or $read -gt $requested) { throw "ADR reader returned invalid byte count '$read' for request '$requested'" }
            if ($read -eq 0) { break }
            $bytesRead += $read
        }
    } catch { $readFailure = $_.Exception }
    if ($null -ne $readFailure) {
        if ($null -ne $stream) { $stream.Dispose() }
        Throw-LaneGateDenial "governance_exception_adr_read_failed" "ADR-0045 authority file could not be read through its leased handle" $readFailure.Message
    }
    if ($bytesRead -gt $LaneOrderMaxBytes) {
        $stream.Dispose()
        Throw-LaneGateDenial "governance_exception_adr_too_large" "ADR-0045 authority file exceeds the fixed $LaneOrderMaxBytes-byte ceiling" ([ordered]@{
            max_bytes = $LaneOrderMaxBytes; observed_at_least_bytes = $bytesRead; allocated_bytes = $capacity
        })
    }
    $digest = Get-Sha256Hex $buffer $bytesRead
    if ($digest -cne $ExpectedDigest) {
        $stream.Dispose()
        Throw-LaneGateDenial "governance_exception_adr_digest_mismatch" "$Token authority file does not match its pinned digest" ([ordered]@{
            expected = $ExpectedDigest; actual = $digest; path = $Path
        })
    }
    return [pscustomobject]@{ path = $Path; stream = $stream; buffer = $buffer; count = $bytesRead; digest = $digest }
}

function Open-ExactRecoveryEvidenceLease {
    param([string]$Path, [string]$ExpectedDigest, [string]$Label, [int64]$MaxBytes = 8388608)
    $fullPath = if ([IO.Path]::IsPathRooted($Path)) { [IO.Path]::GetFullPath($Path) } else { [IO.Path]::GetFullPath((Join-Path $LauncherRepoRoot $Path)) }
    $stream = $null
    try {
        $stream = [IO.FileStream]::new($fullPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read, 4096, [IO.FileOptions]::SequentialScan)
        if ($stream.Length -gt $MaxBytes) { throw "$Label exceeds its evidence bound" }
        [byte[]]$bytes = [byte[]]::new([int]$stream.Length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -le 0) { throw "$Label ended before its leased length" }
            $offset += $read
        }
        $digest = Get-Sha256Hex $bytes
        if ($digest -cne $ExpectedDigest) { throw "$Label digest mismatch: expected=$ExpectedDigest actual=$digest" }
        return [pscustomobject]@{ path = $fullPath; stream = $stream; bytes = $bytes; digest = $digest }
    } catch {
        if ($null -ne $stream) { $stream.Dispose() }
        Throw-LaneGateDenial "governance_recovery_evidence_mismatch" "H26 R3 dispatch-recovery evidence is unavailable or changed" $_.Exception.Message
    }
}

function Open-Adr0045R3RecoveryEvidence {
    param([string]$OrderFullPath, [string]$InitialOrderSha256, [object]$StartPlan, [string]$Prompt)
    $expectedOrderPath = [IO.Path]::GetFullPath((Join-Path $LauncherRepoRoot "target\lanes\h26-r3-race-repair-order.md"))
    if (-not [string]::Equals($OrderFullPath, $expectedOrderPath, [StringComparison]::OrdinalIgnoreCase)) {
        Throw-LaneGateDenial "governance_recovery_order_path_mismatch" "H26 R3 recovery requires the original order path" $OrderFullPath
    }
    if ($InitialOrderSha256 -cne $Adr0045R3RecoveryOrderSha256) {
        Throw-LaneGateDenial "governance_recovery_order_digest_mismatch" "The prompt source is not the exact authorized R3 order" ([ordered]@{ expected = $Adr0045R3RecoveryOrderSha256; actual = $InitialOrderSha256 })
    }
    if (-not [string]::Equals([string]$StartPlan.command_path, $Adr0045R3RecoveryShimPath, [StringComparison]::OrdinalIgnoreCase) -or
        -not [string]::Equals([string]$StartPlan.file_name, $Adr0045R3RecoveryHostPath, [StringComparison]::OrdinalIgnoreCase)) {
        Throw-LaneGateDenial "governance_recovery_start_plan_mismatch" "H26 R3 recovery requires the pinned Windows shim and native host" ([ordered]@{ shim = $StartPlan.command_path; host = $StartPlan.file_name })
    }
    if ([string]$StartPlan.command_sha256 -cne $Adr0045R3RecoveryShimSha256) {
        Throw-LaneGateDenial "governance_recovery_start_plan_mismatch" "The snapshotted Codex shim does not match its pinned bytes" $StartPlan.command_sha256
    }
    try {
        $gitPath = (Get-Command git.exe -ErrorAction Stop).Source
        $gitInfo = New-Object Diagnostics.ProcessStartInfo
        $gitInfo.FileName = $gitPath
        $gitInfo.Arguments = "cat-file blob $($Adr0045R3RecoveryLauncherCommit):scripts/invoke-codex-lane.ps1"
        $gitInfo.WorkingDirectory = $LauncherRepoRoot
        $gitInfo.UseShellExecute = $false; $gitInfo.RedirectStandardOutput = $true; $gitInfo.RedirectStandardError = $true
        $gitProcess = New-Object Diagnostics.Process; $gitProcess.StartInfo = $gitInfo
        if (-not $gitProcess.Start()) { throw "git cat-file did not start" }
        $memory = New-Object IO.MemoryStream
        $gitProcess.StandardOutput.BaseStream.CopyTo($memory)
        $gitError = $gitProcess.StandardError.ReadToEnd(); $gitProcess.WaitForExit()
        if ($gitProcess.ExitCode -ne 0) { throw "git cat-file failed: $gitError" }
        $launcherBlobSha256 = Get-Sha256Hex $memory.ToArray()
        $memory.Dispose(); $gitProcess.Dispose()
        if ($launcherBlobSha256 -cne $Adr0045R3RecoveryLauncherSha256) { throw "Pinned launcher blob digest mismatch: $launcherBlobSha256" }
    } catch {
        Throw-LaneGateDenial "governance_recovery_launcher_mismatch" "The reviewed launcher commit/file binding is unavailable or changed" $_.Exception.Message
    }
    $specs = @(
        @("target\state\adr0045-h26.claim", "01f1049bca67c2581eecb1045902391dd8c61958ad41f1eac1941d3d54e8e904", "R1 claim"),
        @("target\state\adr0045-h26-r2.claim", "c614af70542f785d375e9936ca41bf7d1eadbcce070733a04a7355819623fb6a", "R2 claim"),
        @("target\state\adr0045-h26-r3.claim", $Adr0045R3ClaimSha256, "R3 claim"),
        @("target\lanes\h26-r3-race-repair.stdout.log", $Adr0045R3FailureLogSha256, "pre-child failure record"),
        @("target\lanes\h26-r3-race-repair-order.md", $Adr0045R3RecoveryOrderSha256, "R3 order"),
        @("seed-kernel\src\wifi.rs", "690aa68efaa835fa1df59cfd7316472828e438976e68238e021b6b9c0496f91e", "wifi.rs"),
        @("seed-kernel\src\marvell_wifi_pcie.rs", "d53f1eeedd66fe529d2ad55ab6f821e731135971b17c4b6f584f1105c68c5595", "marvell_wifi_pcie.rs"),
        @("scripts\test-marvell-connection-telemetry.ps1", "fddb8474d46d53b802a5e93b9780b4343a305400c20bdf5026d380608174c31f", "connection predicate"),
        @("scripts\test-wifi-ephemeral-physical.ps1", "f9e818260b369f17b984af06ba86cc795adca14415228265609eddcea228ac65", "ephemeral predicate"),
        @($Adr0045R3RecoveryShimPath, $Adr0045R3RecoveryShimSha256, "codex.ps1"),
        @($Adr0045R3RecoveryHostPath, $Adr0045R3RecoveryHostSha256, "powershell.exe")
    )
    $leases = New-Object System.Collections.Generic.List[object]
    try {
        foreach ($spec in $specs) { $leases.Add((Open-ExactRecoveryEvidenceLease $spec[0] $spec[1] $spec[2])) }
        $failureLease = $leases[3]
        $failure = (New-Object Text.UTF8Encoding($false, $true)).GetString($failureLease.bytes) | ConvertFrom-Json
        if ($failure.accepted -ne $false -or $failure.child_started -ne $false -or $failure.reason -cne "codex_child_start_failed") {
            throw "Failure record does not prove accepted=false, child_started=false, codex_child_start_failed"
        }
        $promptSha256 = Get-Sha256Hex ((New-Object Text.UTF8Encoding($false)).GetBytes($Prompt))
        $planJson = [ordered]@{ host = $Adr0045R3RecoveryHostPath; host_sha256 = $Adr0045R3RecoveryHostSha256; shim = $Adr0045R3RecoveryShimPath; shim_sha256 = $Adr0045R3RecoveryShimSha256; arguments = [string[]]@($StartPlan.arguments); stdin_sha256 = $promptSha256 } | ConvertTo-Json -Compress
        $planSha256 = Get-Sha256Hex ((New-Object Text.UTF8Encoding($false)).GetBytes($planJson))
        return [pscustomobject]@{ leases = $leases; prompt_sha256 = $promptSha256; start_plan_sha256 = $planSha256 }
    } catch {
        foreach ($lease in $leases) { try { $lease.stream.Dispose() } catch {} }
        if ($_.Exception.Data["reason"]) { throw }
        Throw-LaneGateDenial "governance_recovery_evidence_mismatch" "H26 R3 recovery evidence content is invalid" $_.Exception.Message
    }
}

function New-Adr0045OneShotClaim {
    param([string]$Path, [object]$Binding, [string]$MachineId, [string]$ManifestDigest, [string]$FactPath, [string]$DecisionDigest, [AllowNull()][object]$RecoveryEvidence = $null)
    $claimFullPath = Get-AbsolutePath $Path
    $claimParent = Split-Path -Parent $claimFullPath
    try { $null = [IO.Directory]::CreateDirectory($claimParent) }
    catch { Throw-LaneGateDenial "governance_exception_claim_state_unavailable" "ADR-0045 one-shot claim directory is unavailable" $_.Exception.Message }

    $binding = [ordered]@{
        schema = $Binding.claim_schema
        token = $Binding.token
        machine = $MachineId
        manifest_sha256 = $ManifestDigest
        fact_path = $FactPath
        adr_sha256 = $DecisionDigest
    }
    if ($null -ne $RecoveryEvidence) {
        $binding["r1_claim_sha256"] = "01f1049bca67c2581eecb1045902391dd8c61958ad41f1eac1941d3d54e8e904"
        $binding["r2_claim_sha256"] = "c614af70542f785d375e9936ca41bf7d1eadbcce070733a04a7355819623fb6a"
        $binding["recovered_claim_sha256"] = $Adr0045R3ClaimSha256
        $binding["failure_log_sha256"] = $Adr0045R3FailureLogSha256
        $binding["order_sha256"] = $Adr0045R3RecoveryOrderSha256
        $binding["launcher_commit"] = $Adr0045R3RecoveryLauncherCommit
        $binding["launcher_sha256"] = $Adr0045R3RecoveryLauncherSha256
        $binding["wifi_rs_sha256"] = "690aa68efaa835fa1df59cfd7316472828e438976e68238e021b6b9c0496f91e"
        $binding["marvell_wifi_pcie_rs_sha256"] = "d53f1eeedd66fe529d2ad55ab6f821e731135971b17c4b6f584f1105c68c5595"
        $binding["connection_predicate_sha256"] = "fddb8474d46d53b802a5e93b9780b4343a305400c20bdf5026d380608174c31f"
        $binding["ephemeral_predicate_sha256"] = "f9e818260b369f17b984af06ba86cc795adca14415228265609eddcea228ac65"
        $binding["start_plan_sha256"] = $RecoveryEvidence.start_plan_sha256
        $binding["stdin_sha256"] = $RecoveryEvidence.prompt_sha256
    }
    $bindingBytes = (New-Object Text.UTF8Encoding($false)).GetBytes(($binding | ConvertTo-Json -Depth 10 -Compress))
    $claimStream = $null
    try {
        # CreateNew is the atomic authorization boundary. FileShare.Read keeps
        # the completed binding observable while denying mutation/deletion.
        $claimStream = [IO.FileStream]::new($claimFullPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    } catch [IO.IOException] {
        if ([IO.File]::Exists($claimFullPath)) {
            Throw-LaneGateDenial "governance_exception_already_consumed" "$($Binding.token) implementation dispatch has already been claimed" $claimFullPath
        }
        Throw-LaneGateDenial "governance_exception_claim_create_failed" "ADR-0045 one-shot claim could not be created atomically" $_.Exception.Message
    } catch {
        Throw-LaneGateDenial "governance_exception_claim_create_failed" "ADR-0045 one-shot claim could not be created atomically" $_.Exception.Message
    }
    try {
        $claimStream.Write($bindingBytes, 0, $bindingBytes.Length)
        $claimStream.Flush($true)
    } catch {
        $persistFailure = $_.Exception.Message
        $claimStream.Dispose()
        # Fail closed: CreateNew already won, so an incomplete claim is never
        # removed or made reusable automatically.
        Throw-LaneGateDenial "governance_exception_claim_persist_failed" "ADR-0045 one-shot claim binding could not be persisted; authorization remains consumed" $persistFailure
    }
    return [pscustomobject]@{ path = $claimFullPath; stream = $claimStream; binding = $binding }
}

function Assert-Adr0045GovernanceException {
    param(
        [string]$OrderText, [string]$MachineId, [string]$ActualManifestDigest, [string[]]$FactPaths,
        [string]$SandboxName, [object]$Binding, [string]$DecisionPathOverride = ""
    )
    if ($MachineId -cne $Adr0045MachineId) {
        Throw-LaneGateDenial "governance_exception_machine_mismatch" "$($Binding.token) is bound to machine '$Adr0045MachineId'" $MachineId
    }
    if ($ActualManifestDigest -cne $Adr0045ManifestSha256) {
        Throw-LaneGateDenial "governance_exception_manifest_digest_mismatch" "$($Binding.token) is bound to the pinned Surface manifest digest" ([ordered]@{
            expected = $Adr0045ManifestSha256; actual = $ActualManifestDigest
        })
    }
    if (@($FactPaths).Count -ne 1 -or $FactPaths[0] -cne $Adr0045FactPath) {
        Throw-LaneGateDenial "governance_exception_fact_paths_mismatch" "$($Binding.token) requires exactly its singleton fact path" ([ordered]@{
            expected = @($Adr0045FactPath); actual = @($FactPaths)
        })
    }
    if ($SandboxName -cne "workspace-write") {
        Throw-LaneGateDenial "governance_exception_sandbox_mismatch" "$($Binding.token) requires exactly the workspace-write sandbox" $SandboxName
    }
    if (-not (Test-TextContainsExactLine $OrderText $Binding.order_marker)) {
        Throw-LaneGateDenial "governance_exception_order_marker_missing" "Lane order lacks the exact $($Binding.token) governance marker line" $Binding.order_marker
    }
    $decisionFullPath = if ($DecisionPathOverride) { Get-AbsolutePath $DecisionPathOverride } else {
        Join-Path $LauncherRepoRoot $Adr0045DecisionRelativePath
    }
    if (-not [IO.File]::Exists($decisionFullPath)) {
        Throw-LaneGateDenial "governance_exception_adr_not_found" "ADR-0045 authority file does not exist" $decisionFullPath
    }
    return Open-Adr0045AuthorityLease $decisionFullPath $Binding.decision_sha256 $Binding.token
}

function Invoke-CodexLaneGate {
    param(
        [string]$Order, [string]$MachineId, [string]$Digest, [string[]]$FactPaths,
        [string]$SandboxName, [string]$Report, [object]$Schema,
        [string]$GovernanceException = "", [string]$Adr0045DecisionPathOverride = "", [string]$Adr0045ClaimPathOverride = "",
        [AllowNull()][scriptblock]$Adr0045PreStartHook = $null,
        [string]$ManifestPathOverride = "", [string]$ChildCommand = "codex", [string[]]$ChildPrefixArguments = @(),
        [string]$PowerShellHostPathOverride = "",
        [switch]$SimulateStdinWriteFailure, [switch]$SimulateStdinCloseFailure, [switch]$SimulateWaitFailure,
        [string]$PostStartReadyPath = "", [AllowNull()][scriptblock]$ReaderFactory = $null
    )
    $exceptionBinding = if ($GovernanceException -ceq "") { $null } else { Get-Adr0045ExceptionBinding $GovernanceException }
    $useAdr0045Exception = $null -ne $exceptionBinding
    if (($Adr0045ClaimPathOverride -or $null -ne $Adr0045PreStartHook) -and -not $LauncherSelfTest) {
        Throw-LaneGateDenial "governance_exception_selftest_hook_forbidden" "ADR-0045 claim overrides and pre-start hooks are available only inside launcher selftests"
    }
    if ($PowerShellHostPathOverride -and -not $LauncherSelfTest) {
        Throw-LaneGateDenial "codex_powershell_host_override_forbidden" "PowerShell host override is available only inside launcher selftests"
    }
    Assert-LaneGateInputs $Order $MachineId $Digest $FactPaths $SandboxName $Report
    $orderFullPath = Get-AbsolutePath $Order
    $reportFullPath = Get-AbsolutePath $Report
    $manifestFullPath = if ($ManifestPathOverride) { Get-AbsolutePath $ManifestPathOverride } else {
        Join-Path $LauncherRepoRoot "hardware\manifests\$MachineId.v1.json"
    }
    if (-not [IO.File]::Exists($orderFullPath)) { Throw-LaneGateDenial "order_path_not_found" "Lane order does not exist" $orderFullPath }
    if (-not [IO.File]::Exists($manifestFullPath)) { Throw-LaneGateDenial "manifest_path_not_found" "Machine manifest does not exist" $manifestFullPath }
    $reportParent = Split-Path -Parent $reportFullPath
    if (-not [IO.Directory]::Exists($reportParent)) { Throw-LaneGateDenial "report_parent_not_found" "Report parent directory does not exist" $reportParent }
    if ($reportFullPath -eq $orderFullPath -or $reportFullPath -eq $manifestFullPath) { Throw-LaneGateDenial "report_path_conflict" "ReportPath cannot overwrite an input" }

    $orderRead = Read-BoundedFileBytes $orderFullPath $LaneOrderMaxBytes "lane_order_too_large" "order_read_failed" "Lane order" $ReaderFactory
    $initialOrderSha256 = Get-Sha256Hex $orderRead.buffer $orderRead.count
    $orderText = Read-Utf8Text $orderRead.buffer $orderRead.count "order_utf8_invalid" "Lane order"
    if ([string]::IsNullOrWhiteSpace($orderText)) { Throw-LaneGateDenial "lane_order_empty" "Lane order is empty" }
    if ($orderText.IndexOf($ReservedMachineContextToken, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
        Throw-LaneGateDenial "lane_order_reserved_marker" "Lane order contains the reserved machine-context marker token" $ReservedMachineContextToken
    }

    # The manifest is read exactly once. Digest, validation, selection, and prompt
    # rendering all consume these bounded bytes or the single object parsed from them.
    $manifestRead = Read-BoundedFileBytes $manifestFullPath $MachineManifestMaxBytes "manifest_too_large" "manifest_read_failed" "Machine manifest" $ReaderFactory
    $actualDigest = Get-Sha256Hex $manifestRead.buffer $manifestRead.count
    $manifestText = Read-Utf8Text $manifestRead.buffer $manifestRead.count "manifest_utf8_invalid" "Machine manifest"
    try { $manifest = $manifestText | ConvertFrom-Json }
    catch { Throw-LaneGateDenial "manifest_json_invalid" "Machine manifest is not valid JSON" $_.Exception.Message }
    $validation = Test-MachineManifest $manifest $Schema
    if (-not $validation.valid) { Throw-LaneGateDenial "manifest_schema_invalid" "Machine manifest failed structural or semantic validation" $validation.errors }
    if ($validation.machine_id -cne $MachineId) { Throw-LaneGateDenial "machine_id_mismatch" "Manifest machine '$($validation.machine_id)' does not match expected '$MachineId'" }
    if ($actualDigest -cne $Digest.ToLowerInvariant()) { Throw-LaneGateDenial "manifest_digest_mismatch" "Manifest SHA-256 does not match the lane order" ([ordered]@{ expected = $Digest.ToLowerInvariant(); actual = $actualDigest }) }
    $adrLease = $null; $claimLease = $null; $recoveryEvidence = $null; $process = $null; $processStarted = $false
    try {
        if ($useAdr0045Exception) {
            $adrLease = Assert-Adr0045GovernanceException $orderText $MachineId $actualDigest $FactPaths $SandboxName $exceptionBinding $Adr0045DecisionPathOverride
        }
        if (-not $manifest.curated_context_ready -and -not $useAdr0045Exception) { Throw-LaneGateDenial "manifest_not_context_ready" "Manifest '$MachineId' is valid but not curated-context ready" $manifest.missing_required_facts }

        $selectedFacts = @()
        foreach ($path in $FactPaths) {
            $fact = if ($useAdr0045Exception -and $path -ceq $Adr0045FactPath) {
                $identity = @($manifest.devices)[2].identity
                [ordered]@{ found = ($null -ne $identity); path = $path; status = $identity.status; value = $identity.value; provenance = $identity.provenance }
            } else { Resolve-MachineManifestFact $manifest $path }
            if (-not $fact.found) { Throw-LaneGateDenial "required_fact_path_unknown" "Required fact path '$path' does not exist" $path }
            if ($useAdr0045Exception -and $fact.status -cne "observed") {
                Throw-LaneGateDenial "governance_exception_fact_status_mismatch" "ADR-0045-H26 requires an observed fact" $fact
            }
            if ($fact.status -cne "observed") { Throw-LaneGateDenial "required_fact_unknown" "Required fact '$path' is not observed" $fact }
            if ($useAdr0045Exception -and (-not ($fact.value -is [string]) -or $fact.value -cne $Adr0045FactValue)) {
                Throw-LaneGateDenial "governance_exception_fact_value_mismatch" "ADR-0045-H26 requires the pinned Marvell identity" $fact
            }
            $selectedFacts += [ordered]@{ path = $path; status = $fact.status; value = $fact.value; provenance = $fact.provenance }
        }
        try {
            $context = [ordered]@{
                schema = "raios.machine_curated_context.v1"
                machine_manifest = "$MachineId@sha256:$actualDigest"
                required_fact_paths = @($FactPaths)
                facts = @($selectedFacts)
            }
            if ($useAdr0045Exception) {
                $context["governance_exception"] = $exceptionBinding.token
                $context["governance_exception_adr_sha256"] = $adrLease.digest
            }
            $contextJson = $context | ConvertTo-Json -Depth 20 -Compress
            $contextJson = $contextJson.Replace('<', '\u003c').Replace('>', '\u003e')
            if ([string]::IsNullOrWhiteSpace($contextJson) -or $contextJson.Length -gt $MachineContextMaxChars) { throw "Rendered machine context exceeds the $MachineContextMaxChars-character bound" }
            $prompt = $orderText.TrimEnd() + "`r`n`r`n<raios-machine-context>`r`n" + $contextJson + "`r`n</raios-machine-context>`r`n"
        } catch {
            if ($_.Exception.Data["reason"]) { throw }
            Throw-LaneGateDenial "prompt_render_failed" "Bounded lane prompt could not be rendered" $_.Exception.Message
        }
        # Freeze the native executable and structured shim prefix before any
        # one-shot claim is created. Nothing is resolved again after this gate.
        $childArguments = @($ChildPrefixArguments) + @("exec", "-s", $SandboxName, "-C", $LauncherRepoRoot, "--ephemeral", "-o", $reportFullPath, "-")
        $startPlan = Resolve-CodexChildStartPlan $ChildCommand $childArguments $PowerShellHostPathOverride
        $arguments = @($startPlan.arguments)
        $startInfo = New-Object Diagnostics.ProcessStartInfo
        $startInfo.FileName = $startPlan.file_name
        $startInfo.Arguments = (@($arguments | ForEach-Object { ConvertTo-NativeArgument ([string]$_) }) -join " ")
        $startInfo.UseShellExecute = $false
        $startInfo.RedirectStandardInput = $true
        $process = New-Object Diagnostics.Process
        $process.StartInfo = $startInfo

        if ($useAdr0045Exception -and $exceptionBinding.is_recovery -eq $true) {
            $recoveryEvidence = Open-Adr0045R3RecoveryEvidence $orderFullPath $initialOrderSha256 $startPlan $prompt
        }

        $startFailureMessage = if ($useAdr0045Exception) {
            "Codex child process could not be started; ADR-0045 claim remains consumed"
        } else { "Codex child process could not be started" }
        $startFalseMessage = if ($useAdr0045Exception) {
            "Codex child process did not start; ADR-0045 claim remains consumed"
        } else { "Codex child process did not start" }
        if ($useAdr0045Exception) {
            $claimCandidatePath = if ($Adr0045ClaimPathOverride) { $Adr0045ClaimPathOverride } else { Join-Path $LauncherRepoRoot $exceptionBinding.claim_relative_path }
            $claimFullPath = Get-AbsolutePath $claimCandidatePath
            $claimConflicts = @($orderFullPath, $manifestFullPath, $reportFullPath, $adrLease.path) | Where-Object {
                [string]::Equals($_, $claimFullPath, [StringComparison]::OrdinalIgnoreCase)
            }
            if (@($claimConflicts).Count -gt 0) {
                Throw-LaneGateDenial "governance_exception_claim_path_conflict" "ADR-0045 claim path conflicts with a dispatch input or output" $claimFullPath
            }
            if ($null -ne $Adr0045PreStartHook) {
                $hookState = [pscustomobject]@{
                    authority_path = $adrLease.path
                    authority_sha256 = $adrLease.digest
                    authority_bytes_base64 = [Convert]::ToBase64String($adrLease.buffer, 0, $adrLease.count)
                    prompt = $prompt
                    claim_path = $claimFullPath
                }
                try { $null = & $Adr0045PreStartHook $hookState }
                catch { Throw-LaneGateDenial "governance_exception_prestart_hook_failed" "ADR-0045 selftest pre-start hook failed" $_.Exception.Message }
            }
            $claimLease = New-Adr0045OneShotClaim $claimFullPath $exceptionBinding $MachineId $actualDigest $FactPaths[0] $adrLease.digest $recoveryEvidence
        }

        # Nothing that can widen or revalidate authority occurs between the
        # atomic one-shot claim and Process.Start.
        try { $started = $process.Start() }
        catch { Throw-LaneGateDenial "codex_child_start_failed" $startFailureMessage $_.Exception.Message }
        if (-not $started) { Throw-LaneGateDenial "codex_child_start_failed" $startFalseMessage }
        $processStarted = $true

        $postStartReason = ""; $postStartMessage = ""; $postStartCause = ""; $childExitCode = $null
        $terminationAttempted = $false; $childReaped = $false
        $cleanupErrors = New-Object System.Collections.Generic.List[string]
        try {
            $promptBytes = (New-Object Text.UTF8Encoding($false)).GetBytes($prompt)
            if (($SimulateStdinWriteFailure -or $SimulateStdinCloseFailure -or $SimulateWaitFailure) -and
                -not [string]::IsNullOrWhiteSpace($PostStartReadyPath)) {
                $readyDeadline = [DateTime]::UtcNow.AddSeconds(5)
                while (-not [IO.File]::Exists($PostStartReadyPath) -and [DateTime]::UtcNow -lt $readyDeadline) { Start-Sleep -Milliseconds 10 }
                if (-not [IO.File]::Exists($PostStartReadyPath)) { throw [IO.IOException]::new("Fake child did not publish its post-start sentinel") }
            }
            try {
                if ($SimulateStdinWriteFailure) {
                    throw [IO.IOException]::new("Simulated stdin write failure after child start")
                }
                $process.StandardInput.BaseStream.Write($promptBytes, 0, $promptBytes.Length)
            } catch {
                $postStartReason = "codex_child_stdin_write_failed"; $postStartMessage = "Codex child stdin write failed after process start"; $postStartCause = $_.Exception.Message
            }
            if (-not $postStartReason) {
                try {
                    if ($SimulateStdinCloseFailure) { throw [IO.IOException]::new("Simulated stdin close failure after child start") }
                    $process.StandardInput.Close()
                    if ($null -ne $recoveryEvidence) {
                        foreach ($lease in $recoveryEvidence.leases) { $lease.stream.Dispose() }
                        $recoveryEvidence = $null
                    }
                }
                catch { $postStartReason = "codex_child_stdin_close_failed"; $postStartMessage = "Codex child stdin close failed after process start"; $postStartCause = $_.Exception.Message }
            }
            if (-not $postStartReason) {
                try {
                    if ($SimulateWaitFailure) { throw [IO.IOException]::new("Simulated wait failure after child start") }
                    $process.WaitForExit(); $childExitCode = $process.ExitCode
                }
                catch { $postStartReason = "codex_child_wait_failed"; $postStartMessage = "Codex child wait failed after process start"; $postStartCause = $_.Exception.Message }
            }
        } catch {
            if (-not $postStartReason) {
                $postStartReason = "codex_child_post_start_failed"; $postStartMessage = "Codex child dispatch failed after process start"; $postStartCause = $_.Exception.Message
            }
        } finally {
            if ($postStartReason) {
                try {
                    if (-not $process.HasExited) { $terminationAttempted = $true; $process.Kill() }
                } catch { $cleanupErrors.Add("terminate: $($_.Exception.Message)") }
            }
            try { $process.WaitForExit(); $childReaped = $process.HasExited }
            catch { $cleanupErrors.Add("reap: $($_.Exception.Message)") }
            try { $process.Dispose() }
            catch { $cleanupErrors.Add("dispose: $($_.Exception.Message)") }
        }
        if ($postStartReason) {
            $details = [ordered]@{ cause = $postStartCause; termination_attempted = $terminationAttempted; child_reaped = $childReaped; cleanup_errors = [object[]]$cleanupErrors }
            Throw-LaneGateDenial $postStartReason $postStartMessage $details $true
        }
        return [ordered]@{ schema = "raios.codex_lane_dispatch.v1"; accepted = ($childExitCode -eq 0); child_started = $true;
            child_exit_code = [int]$childExitCode; machine_manifest = "$MachineId@sha256:$actualDigest"; report_path = $reportFullPath }
    } finally {
        try {
            if ($null -ne $process -and -not $processStarted) { $process.Dispose() }
        } finally {
            try {
                if ($null -ne $recoveryEvidence) {
                    foreach ($lease in $recoveryEvidence.leases) { try { $lease.stream.Dispose() } catch {} }
                }
                if ($null -ne $claimLease) { $claimLease.stream.Dispose() }
            } finally {
                if ($null -ne $adrLease) { $adrLease.stream.Dispose() }
            }
        }
    }
}

function Invoke-CodexLaneSelfTests {
    param([object]$Schema)
    $tempRoot = Join-Path ([IO.Path]::GetTempPath()) ("raios-codex-lane-" + [Guid]::NewGuid().ToString("N"))
    $null = [IO.Directory]::CreateDirectory($tempRoot)
    $oldCount = $env:RAIOS_FAKE_CHILD_COUNT; $oldArgs = $env:RAIOS_FAKE_CHILD_ARGS; $oldPrompt = $env:RAIOS_FAKE_CHILD_PROMPT
    try {
        $order = Join-Path $tempRoot "order.md"; $report = Join-Path $tempRoot "report.md"
        $fake = Join-Path $tempRoot "fake-codex.exe"; $countPath = Join-Path $tempRoot "count.txt"; $argsPath = Join-Path $tempRoot "args.txt"; $promptPath = Join-Path $tempRoot "prompt.txt"
        $orderContent = "# Fake hardware lane`r`nOnly use the gated facts."
        [IO.File]::WriteAllText($order, $orderContent, (New-Object Text.UTF8Encoding($false)))
        $fakeSource = @'
using System;
using System.IO;
using System.Text;
public static class FakeCodexChild {
    public static int Main(string[] args) {
        var utf8 = new UTF8Encoding(false);
        var countPath = Environment.GetEnvironmentVariable("RAIOS_FAKE_CHILD_COUNT");
        var argsPath = Environment.GetEnvironmentVariable("RAIOS_FAKE_CHILD_ARGS");
        var promptPath = Environment.GetEnvironmentVariable("RAIOS_FAKE_CHILD_PROMPT");
        var count = File.Exists(countPath) ? Int32.Parse(File.ReadAllText(countPath)) : 0;
        File.WriteAllText(countPath, (count + 1).ToString(), utf8);
        File.WriteAllLines(argsPath, args, utf8);
        using (var reader = new StreamReader(Console.OpenStandardInput(), utf8)) {
            File.WriteAllText(promptPath, reader.ReadToEnd(), utf8);
        }
        return 0;
    }
}
'@
        Add-Type -TypeDefinition $fakeSource -Language CSharp -OutputAssembly $fake -OutputType ConsoleApplication
        $growingReaderSource = @'
using System;
using System.IO;
public sealed class DeterministicGrowingReadStream : Stream {
    private long position;
    private long visibleLength;
    private readonly long finalLength;
    private readonly int chunkSize;
    private readonly byte fillByte;
    public bool GrowthTriggered { get; private set; }
    public int ReadCalls { get; private set; }
    public int MaxRequested { get; private set; }
    public DeterministicGrowingReadStream(long initialLength, long finalLength, int chunkSize, byte fillByte) {
        this.visibleLength = initialLength;
        this.finalLength = finalLength;
        this.chunkSize = chunkSize;
        this.fillByte = fillByte;
    }
    public override int Read(byte[] buffer, int offset, int count) {
        ReadCalls++;
        if (count > MaxRequested) MaxRequested = count;
        if (position >= visibleLength && !GrowthTriggered && finalLength > visibleLength) {
            visibleLength = finalLength;
            GrowthTriggered = true;
        }
        if (position >= visibleLength) return 0;
        int result = (int)Math.Min(Math.Min((long)count, visibleLength - position), (long)chunkSize);
        for (int i = 0; i < result; i++) buffer[offset + i] = fillByte;
        position += result;
        return result;
    }
    public override bool CanRead { get { return true; } }
    public override bool CanSeek { get { return false; } }
    public override bool CanWrite { get { return false; } }
    public override long Length { get { throw new NotSupportedException(); } }
    public override long Position { get { return position; } set { throw new NotSupportedException(); } }
    public override void Flush() { }
    public override long Seek(long offset, SeekOrigin origin) { throw new NotSupportedException(); }
    public override void SetLength(long value) { throw new NotSupportedException(); }
    public override void Write(byte[] buffer, int offset, int count) { throw new NotSupportedException(); }
}
public sealed class Adr0045ClaimRaceResult {
    public int WinnerCount;
    public string[] UnexpectedErrors;
}
public static class Adr0045AtomicClaimRace {
    public static Adr0045ClaimRaceResult Run(string path, byte[] binding) {
        var start = new System.Threading.ManualResetEventSlim(false);
        var errors = new string[2];
        int winners = 0;
        System.Threading.Thread[] threads = new System.Threading.Thread[2];
        for (int i = 0; i < threads.Length; i++) {
            int slot = i;
            threads[i] = new System.Threading.Thread(() => {
                start.Wait();
                try {
                    using (var stream = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read)) {
                        stream.Write(binding, 0, binding.Length);
                        stream.Flush(true);
                        System.Threading.Interlocked.Increment(ref winners);
                    }
                } catch (IOException) {
                    // The losing CreateNew is the expected outcome.
                } catch (Exception ex) {
                    errors[slot] = ex.GetType().FullName + ": " + ex.Message;
                }
            });
            threads[i].Start();
        }
        start.Set();
        foreach (var thread in threads) thread.Join();
        return new Adr0045ClaimRaceResult { WinnerCount = winners, UnexpectedErrors = errors };
    }
}
'@
        Add-Type -TypeDefinition $growingReaderSource -Language CSharp
        $env:RAIOS_FAKE_CHILD_COUNT = $countPath; $env:RAIOS_FAKE_CHILD_ARGS = $argsPath; $env:RAIOS_FAKE_CHILD_PROMPT = $promptPath
        $qemuPath = Join-Path $LauncherRepoRoot "hardware\manifests\qemu-q35-shadow.v1.json"
        $surfacePath = Join-Path $LauncherRepoRoot "hardware\manifests\surface-pro-4.v1.json"
        $qemu = Get-Content -Raw -LiteralPath $qemuPath | ConvertFrom-Json
        $surface = Get-Content -Raw -LiteralPath $surfacePath | ConvertFrom-Json
        $exceptionOrderContent = $orderContent + [Environment]::NewLine + $Adr0045OrderMarker
        $r2ExceptionOrderContent = $orderContent + [Environment]::NewLine + $Adr0045R2OrderMarker
        $r3ExceptionOrderContent = $orderContent + [Environment]::NewLine + $Adr0045R3OrderMarker
        $cases = New-Object System.Collections.Generic.List[object]
        function Reset-Sentinel { Remove-Item -LiteralPath $countPath, $argsPath, $promptPath -Force -ErrorAction SilentlyContinue }
        function Get-InvocationCount { if (Test-Path -LiteralPath $countPath) { return [int](Get-Content -Raw -LiteralPath $countPath) }; return 0 }
        function Write-Mutant([string]$Name, [object]$Value) { $path = Join-Path $tempRoot "$Name.json"; [IO.File]::WriteAllText($path, ($Value | ConvertTo-Json -Depth 100), (New-Object Text.UTF8Encoding($false))); return $path }
        function Get-TestFileDigest([string]$Path) {
            $read = Read-BoundedFileBytes $Path $MachineManifestMaxBytes "test_fixture_too_large" "test_fixture_read_failed" "Selftest fixture"
            return Get-Sha256Hex $read.buffer $read.count
        }
        $qemuDigest = Get-TestFileDigest $qemuPath; $surfaceDigest = Get-TestFileDigest $surfacePath
        function Run-Denial([string]$Name, [scriptblock]$Action, [string]$ExpectedReason, [string]$ExpectedDetailCode = "", [scriptblock]$AdditionalAssertion = $null) {
            Reset-Sentinel; $reason = "accepted"; $details = $null; $exceptionChildStarted = $false
            try { $null = & $Action } catch { $reason = [string]$_.Exception.Data["reason"]; $details = $_.Exception.Data["details"]; $exceptionChildStarted = [bool]$_.Exception.Data["child_started"] }
            $detailCodes = @($details | ForEach-Object { if ($null -ne $_.code) { $_.code } })
            $additionalPassed = $null -eq $AdditionalAssertion -or [bool](& $AdditionalAssertion $details)
            $passed = $reason -ceq $ExpectedReason -and -not $exceptionChildStarted -and (Get-InvocationCount) -eq 0 -and (-not $ExpectedDetailCode -or (Test-OrdinalStringIn $ExpectedDetailCode $detailCodes)) -and $additionalPassed
            $cases.Add([ordered]@{ name = $Name; expected_reason = $ExpectedReason; actual_reason = $reason; child_started = $exceptionChildStarted; child_invocation_count = Get-InvocationCount; passed = $passed; detail_codes = $detailCodes; denial_details = $details; additional_assertion = $additionalPassed })
        }

        Reset-Sentinel
        $required = @("/cpu/model", "/memory/total_bytes", "/devices/xhci/identity")
        $dispatch = Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "workspace-write" $report $Schema -ChildCommand $fake
        $receivedArgs = [IO.File]::ReadAllLines($argsPath); $receivedPrompt = Get-Content -Raw -LiteralPath $promptPath
        $match = [regex]::Match($receivedPrompt, '(?s)<raios-machine-context>\r?\n(.+?)\r?\n</raios-machine-context>')
        $parsedContext = if ($match.Success) { $match.Groups[1].Value | ConvertFrom-Json } else { $null }
        $facts = @{}; if ($null -ne $parsedContext) { foreach ($fact in @($parsedContext.facts)) { $facts[$fact.path] = $fact } }
        $expectedFacts = @(); foreach ($path in $required) { $fact = Resolve-MachineManifestFact $qemu $path; $expectedFacts += [ordered]@{ path = $path; status = $fact.status; value = $fact.value; provenance = $fact.provenance } }
        $expectedContext = [ordered]@{ schema = "raios.machine_curated_context.v1"; machine_manifest = "qemu-q35-shadow@sha256:$qemuDigest"; required_fact_paths = @($required); facts = @($expectedFacts) }
        $expectedContextJson = ($expectedContext | ConvertTo-Json -Depth 20 -Compress).Replace('<', '\u003c').Replace('>', '\u003e')
        $expectedPrompt = $orderContent.TrimEnd() + "`r`n`r`n<raios-machine-context>`r`n" + $expectedContextJson + "`r`n</raios-machine-context>`r`n"
        $expectedArgs = @("exec", "-s", "workspace-write", "-C", $LauncherRepoRoot, "--ephemeral", "-o", ([IO.Path]::GetFullPath($report)), "-")
        $argsExact = (@($receivedArgs) -join "`0") -ceq ($expectedArgs -join "`0")
        $promptExact = $receivedPrompt -ceq $expectedPrompt
        $positivePassed = (Get-InvocationCount) -eq 1 -and $dispatch.child_started -and $argsExact -and $promptExact -and $match.Success -and
            $parsedContext.machine_manifest -ceq "qemu-q35-shadow@sha256:$qemuDigest" -and
            (@($parsedContext.required_fact_paths) -join ',') -ceq ($required -join ',') -and
            $facts["/cpu/model"].value -ceq "max" -and [int64]$facts["/memory/total_bytes"].value -eq 536870912 -and
            $facts["/devices/xhci/identity"].value -ceq "qemu-xhci" -and
            @($facts.Values | Where-Object { $_.provenance.machine_id -cne "qemu-q35-shadow" -or [string]::IsNullOrWhiteSpace($_.provenance.source_ref) }).Count -eq 0
        $positiveDetails = @("dispatch=$($dispatch.child_started)", "argv_exact=$argsExact", "stdin_exact=$promptExact", "arg_count=$(@($receivedArgs).Count)", "stdin_marker=$(@($receivedArgs)[-1])", "match=$($match.Success)", "manifest=$($parsedContext.machine_manifest)",
            "paths=$(@($parsedContext.required_fact_paths) -join ',')", "cpu=$($facts['/cpu/model'].value)",
            "memory=$($facts['/memory/total_bytes'].value)", "device=$($facts['/devices/xhci/identity'].value)",
            "bad_provenance=$(@($facts.Values | Where-Object { $_.provenance.machine_id -cne 'qemu-q35-shadow' -or [string]::IsNullOrWhiteSpace($_.provenance.source_ref) }).Count)")
        $cases.Add([ordered]@{ name = "positive-one-fake-child-with-exact-context"; expected_reason = "accepted"; actual_reason = if ($positivePassed) { "accepted" } else { "positive_assertion_failed" }; child_invocation_count = Get-InvocationCount; passed = $positivePassed; detail_codes = $positiveDetails })

        $shim = Join-Path $tempRoot "fake-codex.ps1"
        $shimArgsPath = Join-Path $tempRoot "fake-codex-shim-args.txt"
        $escapedFake = $fake.Replace("'", "''")
        $escapedShimArgsPath = $shimArgsPath.Replace("'", "''")
        $shimSource = "[IO.File]::WriteAllLines('$escapedShimArgsPath', [string[]]`$args)`r`nif (`$MyInvocation.ExpectingInput) { `$input | & '$escapedFake' @args } else { & '$escapedFake' @args }`r`nexit `$LASTEXITCODE`r`n"
        [IO.File]::WriteAllText($shim, $shimSource, (New-Object Text.UTF8Encoding($false)))
        Reset-Sentinel
        $shimPlan = Resolve-CodexChildStartPlan $shim $expectedArgs
        $shimDispatch = Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "workspace-write" $report $Schema -ChildCommand $shim
        $shimArgs = [IO.File]::ReadAllLines($argsPath); $shimPrompt = Get-Content -Raw -LiteralPath $promptPath
        $shimBoundaryArgs = [IO.File]::ReadAllLines($shimArgsPath)
        $shimArgsExact = (@($shimArgs) -join "`0") -ceq ($expectedArgs -join "`0")
        $shimBoundaryArgsExact = (@($shimBoundaryArgs) -join "`0") -ceq ($expectedArgs -join "`0")
        $shimPromptExact = $shimPrompt -ceq $expectedPrompt
        $shimPassed = (Get-InvocationCount) -eq 1 -and $shimDispatch.child_started -and $shimArgsExact -and $shimBoundaryArgsExact -and $shimPromptExact -and
            [IO.Path]::GetExtension([string]$shimPlan.file_name).ToLowerInvariant() -ceq ".exe" -and
            [IO.Path]::GetExtension([string]$shimPlan.command_path).ToLowerInvariant() -ceq ".ps1" -and
            ([string]$shimPlan.file_name -cne [string]$shimPlan.command_path)
        $cases.Add([ordered]@{ name = "windows-powershell-shim-one-child-exact-argv-stdin"; expected_reason = "native_host_exact_handoff";
            actual_reason = if ($shimPassed) { "native_host_exact_handoff" } else { "shim_handoff_assertion_failed" };
            child_started = $shimDispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $shimPassed;
            detail_codes = @("native=$($shimPlan.file_name)", "shim=$($shimPlan.command_path)", "shim_boundary_argv_exact=$shimBoundaryArgsExact", "child_argv_exact=$shimArgsExact", "stdin_exact=$shimPromptExact", "stdin_expected_chars=$($expectedPrompt.Length)", "stdin_actual_chars=$($shimPrompt.Length)") })

        $unsupported = Join-Path $tempRoot "fake-codex.cmd"
        [IO.File]::WriteAllText($unsupported, "@echo off`r`nexit /b 0`r`n", (New-Object Text.UTF8Encoding($false)))
        Run-Denial "windows-command-script-is-never-direct-file-name" {
            Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "workspace-write" $report $Schema -ChildCommand $unsupported
        } "codex_command_unsupported"
        $missingHost = Join-Path $tempRoot "missing-powershell.exe"
        Run-Denial "windows-powershell-host-missing-before-child-or-claim" {
            Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "workspace-write" $report $Schema -ChildCommand $shim -PowerShellHostPathOverride $missingHost
        } "codex_powershell_host_missing"

        $malformed = Join-Path $tempRoot "malformed.json"; [IO.File]::WriteAllText($malformed, '{', (New-Object Text.UTF8Encoding($false))); $malformedDigest = Get-TestFileDigest $malformed
        Run-Denial "malformed-json" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $malformedDigest $required "workspace-write" $report $Schema -ManifestPathOverride $malformed -ChildCommand $fake } "manifest_json_invalid"
        $m = Copy-JsonObject (Get-Content -Raw -LiteralPath $qemuPath | ConvertFrom-Json); $m | Add-Member unexpected $true; $path = Write-Mutant "additional" $m; $digest = Get-TestFileDigest $path
        Run-Denial "additional-property" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $digest $required "workspace-write" $report $Schema -ManifestPathOverride $path -ChildCommand $fake } "manifest_schema_invalid" "additional_property"
        $m = Copy-JsonObject (Get-Content -Raw -LiteralPath $qemuPath | ConvertFrom-Json); $m.curated_context_ready = "true"; $path = Write-Mutant "wrong-type" $m; $digest = Get-TestFileDigest $path
        Run-Denial "wrong-json-type" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $digest $required "workspace-write" $report $Schema -ManifestPathOverride $path -ChildCommand $fake } "manifest_schema_invalid" "type_mismatch"
        Run-Denial "wrong-machine" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $surfaceDigest $required "workspace-write" $report $Schema -ManifestPathOverride $surfacePath -ChildCommand $fake } "machine_id_mismatch"
        Run-Denial "digest-drift" { Invoke-CodexLaneGate $order "qemu-q35-shadow" ("0" * 64) $required "workspace-write" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake } "manifest_digest_mismatch"
        Run-Denial "surface-not-ready" { Invoke-CodexLaneGate $order "surface-pro-4" $surfaceDigest @("/cpu/model") "read-only" $report $Schema -ManifestPathOverride $surfacePath -ChildCommand $fake } "manifest_not_context_ready"

        $exceptionFactPaths = @($Adr0045FactPath)
        $authoritySourcePath = Join-Path $LauncherRepoRoot $Adr0045DecisionRelativePath
        $currentAuthorityBytes = [IO.File]::ReadAllBytes($authoritySourcePath)
        $strictUtf8 = New-Object Text.UTF8Encoding($false, $true)
        $currentAuthorityText = $strictUtf8.GetString($currentAuthorityBytes)
        $recoverySectionMarker = "## R3-Dispatch-Recovery-Erweiterung (2026-07-22)"
        $recoverySectionIndex = $currentAuthorityText.IndexOf($recoverySectionMarker, [StringComparison]::Ordinal)
        if ($recoverySectionIndex -lt 2 -or $currentAuthorityText[$recoverySectionIndex - 1] -ne "`n" -or $currentAuthorityText[$recoverySectionIndex - 2] -ne "`n") {
            throw "ADR-0045 R3 authority prefix cannot be reconstructed exactly inside the selftest"
        }
        $r3AuthorityBytes = $strictUtf8.GetBytes($currentAuthorityText.Substring(0, $recoverySectionIndex - 1))
        $r3AuthorityText = $strictUtf8.GetString($r3AuthorityBytes)
        $r3SectionMarker = "## R3-Erweiterung (2026-07-22)"
        $r3SectionIndex = $r3AuthorityText.IndexOf($r3SectionMarker, [StringComparison]::Ordinal)
        if ($r3SectionIndex -lt 2 -or $r3AuthorityText[$r3SectionIndex - 1] -ne "`n" -or $r3AuthorityText[$r3SectionIndex - 2] -ne "`n") {
            throw "ADR-0045 R2 authority prefix cannot be reconstructed exactly inside the selftest"
        }
        $r2AuthorityBytes = $strictUtf8.GetBytes($r3AuthorityText.Substring(0, $r3SectionIndex - 1))
        $r2AuthorityText = $strictUtf8.GetString($r2AuthorityBytes)
        $r2SectionMarker = "## R2-Erweiterung (2026-07-22)"
        $r2SectionIndex = $r2AuthorityText.IndexOf($r2SectionMarker, [StringComparison]::Ordinal)
        if ($r2SectionIndex -lt 2 -or $r2AuthorityText[$r2SectionIndex - 1] -ne "`n" -or $r2AuthorityText[$r2SectionIndex - 2] -ne "`n") {
            throw "ADR-0045 R1 authority prefix cannot be reconstructed exactly inside the selftest"
        }
        # R1/R2 remain bound to their earlier authority prefixes. Reconstruct
        # only those exact committed prefixes in the temp directory; production
        # receives neither a reconstructed ADR nor an authority-digest override.
        $authorityBytes = $strictUtf8.GetBytes($r2AuthorityText.Substring(0, $r2SectionIndex - 1))
        if ((Get-Sha256Hex $authorityBytes) -cne $Adr0045DecisionSha256 -or
            (Get-Sha256Hex $r2AuthorityBytes) -cne $Adr0045R2DecisionSha256 -or
            (Get-Sha256Hex $r3AuthorityBytes) -cne $Adr0045R3DecisionSha256) {
            throw "ADR-0045 R1/R2/R3 selftest authority fixtures do not match their hard-coded production digests"
        }
        $exceptionAdrPath = Join-Path $tempRoot "adr0045-authority.md"
        $r2ExceptionAdrPath = Join-Path $tempRoot "adr0045-r2-authority.md"
        $r3ExceptionAdrPath = Join-Path $tempRoot "adr0045-r3-authority.md"
        $exceptionClaimPath = Join-Path $tempRoot "adr0045-positive.claim"
        [IO.File]::WriteAllBytes($exceptionAdrPath, $authorityBytes)
        [IO.File]::WriteAllBytes($r2ExceptionAdrPath, $r2AuthorityBytes)
        [IO.File]::WriteAllBytes($r3ExceptionAdrPath, $r3AuthorityBytes)
        [IO.File]::WriteAllText($order, $exceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Reset-Sentinel
        $exceptionDispatch = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
            -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionClaimPath `
            -ManifestPathOverride $surfacePath -ChildCommand $fake
        $exceptionArgs = [IO.File]::ReadAllLines($argsPath); $exceptionPrompt = Get-Content -Raw -LiteralPath $promptPath
        $exceptionMatch = [regex]::Match($exceptionPrompt, '(?s)<raios-machine-context>\r?\n(.+?)\r?\n</raios-machine-context>')
        $exceptionContext = if ($exceptionMatch.Success) { $exceptionMatch.Groups[1].Value | ConvertFrom-Json } else { $null }
        $exceptionClaim = if ([IO.File]::Exists($exceptionClaimPath)) { Get-Content -Raw -LiteralPath $exceptionClaimPath | ConvertFrom-Json } else { $null }
        $exceptionIdentity = @($surface.devices)[2].identity
        $exceptionSelectedFact = [ordered]@{ status = $exceptionIdentity.status; value = $exceptionIdentity.value; provenance = $exceptionIdentity.provenance }
        $expectedExceptionContext = [ordered]@{
            schema = "raios.machine_curated_context.v1"
            machine_manifest = "$Adr0045MachineId@sha256:$Adr0045ManifestSha256"
            required_fact_paths = @($Adr0045FactPath)
            facts = @([ordered]@{ path = $Adr0045FactPath; status = $exceptionSelectedFact.status; value = $exceptionSelectedFact.value; provenance = $exceptionSelectedFact.provenance })
        }
        $expectedExceptionContext["governance_exception"] = $Adr0045ExceptionToken
        $expectedExceptionContext["governance_exception_adr_sha256"] = $Adr0045DecisionSha256
        $expectedExceptionJson = ($expectedExceptionContext | ConvertTo-Json -Depth 20 -Compress).Replace('<', '\u003c').Replace('>', '\u003e')
        $crlf = [string][char]13 + [string][char]10
        $expectedExceptionPrompt = $exceptionOrderContent.TrimEnd() + $crlf + $crlf + "<raios-machine-context>" + $crlf + $expectedExceptionJson + $crlf + "</raios-machine-context>" + $crlf
        $expectedExceptionArgs = @("exec", "-s", "workspace-write", "-C", $LauncherRepoRoot, "--ephemeral", "-o", ([IO.Path]::GetFullPath($report)), "-")
        $exceptionArgsExact = (@($exceptionArgs) -join [char]0) -ceq ($expectedExceptionArgs -join [char]0)
        $exceptionPromptExact = $exceptionPrompt -ceq $expectedExceptionPrompt
        $exceptionForbiddenFacts = @($exceptionContext.facts | Where-Object { $_.path -like "/cpu/*" -or $_.path -like "/memory/*" }).Count
        $exceptionClaimExact = $null -ne $exceptionClaim -and $exceptionClaim.schema -ceq "raios.adr0045_h26_claim.v1" -and
            $exceptionClaim.token -ceq $Adr0045ExceptionToken -and $exceptionClaim.machine -ceq $Adr0045MachineId -and
            $exceptionClaim.manifest_sha256 -ceq $Adr0045ManifestSha256 -and $exceptionClaim.fact_path -ceq $Adr0045FactPath -and
            $exceptionClaim.adr_sha256 -ceq $Adr0045DecisionSha256
        $exceptionPositivePassed = $surfaceDigest -ceq $Adr0045ManifestSha256 -and (Get-InvocationCount) -eq 1 -and
            $exceptionDispatch.child_started -and $exceptionArgsExact -and $exceptionPromptExact -and $exceptionMatch.Success -and
            $exceptionContext.governance_exception -ceq $Adr0045ExceptionToken -and
            $exceptionContext.governance_exception_adr_sha256 -ceq $Adr0045DecisionSha256 -and $exceptionClaimExact -and
            @($exceptionContext.required_fact_paths).Count -eq 1 -and $exceptionContext.required_fact_paths[0] -ceq $Adr0045FactPath -and
            @($exceptionContext.facts).Count -eq 1 -and $exceptionContext.facts[0].status -ceq "observed" -and
            $exceptionContext.facts[0].value -ceq $Adr0045FactValue -and $exceptionForbiddenFacts -eq 0
        $cases.Add([ordered]@{ name = "adr0045-exact-positive-one-fake-child"; expected_reason = "accepted";
            actual_reason = if ($exceptionPositivePassed) { "accepted" } else { "positive_assertion_failed" };
            child_started = $exceptionDispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $exceptionPositivePassed;
            detail_codes = @("manifest=$surfaceDigest", "argv_exact=$exceptionArgsExact", "stdin_exact=$exceptionPromptExact",
                "exception=$($exceptionContext.governance_exception)", "adr_digest=$($exceptionContext.governance_exception_adr_sha256)",
                "claim_binding_exact=$exceptionClaimExact", "fact_count=$(@($exceptionContext.facts).Count)", "forbidden_fact_count=$exceptionForbiddenFacts") })

        $replayBefore = Get-InvocationCount; $replayReason = "accepted"; $replayChildStarted = $false
        try {
            $null = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
                -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionClaimPath `
                -ManifestPathOverride $surfacePath -ChildCommand $fake
        } catch {
            $replayReason = [string]$_.Exception.Data["reason"]
            $replayChildStarted = [bool]$_.Exception.Data["child_started"]
        }
        $replayAfter = Get-InvocationCount
        $replayPassed = $replayReason -ceq "governance_exception_already_consumed" -and -not $replayChildStarted -and
            $replayBefore -eq 1 -and $replayAfter -eq $replayBefore
        $cases.Add([ordered]@{ name = "adr0045-second-identical-dispatch-denied"; expected_reason = "governance_exception_already_consumed";
            actual_reason = $replayReason; child_started = $replayChildStarted; child_invocation_count = $replayAfter; passed = $replayPassed;
            detail_codes = @("before=$replayBefore", "after=$replayAfter", "additional=$($replayAfter - $replayBefore)") })

        Reset-Sentinel
        $raceClaimPath = Join-Path $tempRoot "adr0045-race.claim"
        $claimBindingBytes = [IO.File]::ReadAllBytes($exceptionClaimPath)
        $raceResult = [Adr0045AtomicClaimRace]::Run($raceClaimPath, $claimBindingBytes)
        $raceUnexpectedErrors = @($raceResult.UnexpectedErrors | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        $raceContentExact = [IO.File]::Exists($raceClaimPath) -and
            [Convert]::ToBase64String([IO.File]::ReadAllBytes($raceClaimPath)) -ceq [Convert]::ToBase64String($claimBindingBytes)
        $racePassed = $raceResult.WinnerCount -eq 1 -and $raceUnexpectedErrors.Count -eq 0 -and $raceContentExact -and (Get-InvocationCount) -eq 0
        $cases.Add([ordered]@{ name = "adr0045-create-new-race-single-winner"; expected_reason = "one_atomic_winner";
            actual_reason = if ($racePassed) { "one_atomic_winner" } else { "claim_race_assertion_failed" };
            child_started = $false; child_invocation_count = Get-InvocationCount; passed = $racePassed;
            detail_codes = @("winners=$($raceResult.WinnerCount)", "content_exact=$raceContentExact") + $raceUnexpectedErrors })

        Reset-Sentinel
        $leaseAdrPath = Join-Path $tempRoot "adr0045-leased-authority.md"
        $leaseClaimPath = Join-Path $tempRoot "adr0045-lease.claim"
        $leaseReplacementPath = Join-Path $tempRoot "adr0045-replacement.md"
        $leaseBackupPath = Join-Path $tempRoot "adr0045-replaced-backup.md"
        [IO.File]::WriteAllBytes($leaseAdrPath, $authorityBytes)
        [IO.File]::WriteAllText($leaseReplacementPath, "# revoked authority", (New-Object Text.UTF8Encoding($false)))
        $hookObservation = [pscustomobject]@{ calls = 0; replace_attempted = $false; replace_failed = $false; replace_error = "";
            delete_attempted = $false; delete_failed = $false; delete_error = ""; snapshot_digest = ""; prompt_exact = $false }
        $preStartMutationHook = {
            param($state)
            $hookObservation.calls++
            $hookObservation.snapshot_digest = Get-Sha256Hex ([Convert]::FromBase64String($state.authority_bytes_base64))
            $hookObservation.prompt_exact = $state.authority_sha256 -ceq $Adr0045DecisionSha256 -and $state.prompt -ceq $expectedExceptionPrompt
            $hookObservation.replace_attempted = $true
            try { [IO.File]::Replace($leaseReplacementPath, $state.authority_path, $leaseBackupPath) }
            catch { $hookObservation.replace_failed = $true; $hookObservation.replace_error = $_.Exception.GetType().Name }
            $hookObservation.delete_attempted = $true
            try { [IO.File]::Delete($state.authority_path) }
            catch { $hookObservation.delete_failed = $true; $hookObservation.delete_error = $_.Exception.GetType().Name }
        }.GetNewClosure()
        $leaseDispatch = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
            -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $leaseAdrPath -Adr0045ClaimPathOverride $leaseClaimPath `
            -Adr0045PreStartHook $preStartMutationHook -ManifestPathOverride $surfacePath -ChildCommand $fake
        $leasePrompt = Get-Content -Raw -LiteralPath $promptPath
        $leaseAuthorityExact = [IO.File]::Exists($leaseAdrPath) -and (Get-TestFileDigest $leaseAdrPath) -ceq $Adr0045DecisionSha256
        $leasePassed = $leaseDispatch.child_started -and (Get-InvocationCount) -eq 1 -and $hookObservation.calls -eq 1 -and
            $hookObservation.replace_attempted -and $hookObservation.replace_failed -and
            $hookObservation.delete_attempted -and $hookObservation.delete_failed -and
            $hookObservation.snapshot_digest -ceq $Adr0045DecisionSha256 -and $hookObservation.prompt_exact -and
            $leaseAuthorityExact -and $leasePrompt -ceq $expectedExceptionPrompt
        $cases.Add([ordered]@{ name = "adr0045-prestart-lease-blocks-replace-delete"; expected_reason = "accepted_with_mutation_denied";
            actual_reason = if ($leasePassed) { "accepted_with_mutation_denied" } else { "lease_assertion_failed" };
            child_started = $leaseDispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $leasePassed;
            detail_codes = @("replace_failed=$($hookObservation.replace_failed):$($hookObservation.replace_error)",
                "delete_failed=$($hookObservation.delete_failed):$($hookObservation.delete_error)",
                "snapshot_digest=$($hookObservation.snapshot_digest)", "authority_exact=$leaseAuthorityExact",
                "prompt_exact=$($hookObservation.prompt_exact -and $leasePrompt -ceq $expectedExceptionPrompt)") })

        $r1ClaimBytesBeforeR2 = [IO.File]::ReadAllBytes($exceptionClaimPath)
        $r2ClaimPath = Join-Path $tempRoot "adr0045-r2-positive.claim"
        [IO.File]::WriteAllText($order, $r2ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Reset-Sentinel
        $r2Dispatch = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
            -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r2ClaimPath `
            -ManifestPathOverride $surfacePath -ChildCommand $fake
        $r2Args = [IO.File]::ReadAllLines($argsPath); $r2Prompt = Get-Content -Raw -LiteralPath $promptPath
        $r2Match = [regex]::Match($r2Prompt, '(?s)<raios-machine-context>\r?\n(.+?)\r?\n</raios-machine-context>')
        $r2Context = if ($r2Match.Success) { $r2Match.Groups[1].Value | ConvertFrom-Json } else { $null }
        $r2Claim = if ([IO.File]::Exists($r2ClaimPath)) { Get-Content -Raw -LiteralPath $r2ClaimPath | ConvertFrom-Json } else { $null }
        $expectedR2Context = [ordered]@{
            schema = "raios.machine_curated_context.v1"
            machine_manifest = "$Adr0045MachineId@sha256:$Adr0045ManifestSha256"
            required_fact_paths = @($Adr0045FactPath)
            facts = @([ordered]@{ path = $Adr0045FactPath; status = $exceptionSelectedFact.status; value = $exceptionSelectedFact.value; provenance = $exceptionSelectedFact.provenance })
        }
        $expectedR2Context["governance_exception"] = $Adr0045R2ExceptionToken
        $expectedR2Context["governance_exception_adr_sha256"] = $Adr0045R2DecisionSha256
        $expectedR2Json = ($expectedR2Context | ConvertTo-Json -Depth 20 -Compress).Replace('<', '\u003c').Replace('>', '\u003e')
        $expectedR2Prompt = $r2ExceptionOrderContent.TrimEnd() + $crlf + $crlf + "<raios-machine-context>" + $crlf + $expectedR2Json + $crlf + "</raios-machine-context>" + $crlf
        $r2ArgsExact = (@($r2Args) -join [char]0) -ceq ($expectedExceptionArgs -join [char]0)
        $r2PromptExact = $r2Prompt -ceq $expectedR2Prompt
        $r2ClaimExact = $null -ne $r2Claim -and $r2Claim.schema -ceq "raios.adr0045_h26_r2_claim.v1" -and
            $r2Claim.token -ceq $Adr0045R2ExceptionToken -and $r2Claim.machine -ceq $Adr0045MachineId -and
            $r2Claim.manifest_sha256 -ceq $Adr0045ManifestSha256 -and $r2Claim.fact_path -ceq $Adr0045FactPath -and
            $r2Claim.adr_sha256 -ceq $Adr0045R2DecisionSha256
        $r2PositivePassed = (Get-InvocationCount) -eq 1 -and $r2Dispatch.child_started -and $r2ArgsExact -and
            $r2PromptExact -and $r2Match.Success -and $r2Context.governance_exception -ceq $Adr0045R2ExceptionToken -and
            $r2Context.governance_exception_adr_sha256 -ceq $Adr0045R2DecisionSha256 -and $r2ClaimExact -and
            @($r2Context.required_fact_paths).Count -eq 1 -and $r2Context.required_fact_paths[0] -ceq $Adr0045FactPath -and
            @($r2Context.facts).Count -eq 1 -and $r2Context.facts[0].status -ceq "observed" -and
            $r2Context.facts[0].value -ceq $Adr0045FactValue
        $cases.Add([ordered]@{ name = "adr0045-r2-exact-positive-one-fake-child"; expected_reason = "accepted";
            actual_reason = if ($r2PositivePassed) { "accepted" } else { "positive_assertion_failed" };
            child_started = $r2Dispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $r2PositivePassed;
            detail_codes = @("argv_exact=$r2ArgsExact", "stdin_exact=$r2PromptExact", "claim_binding_exact=$r2ClaimExact",
                "exception=$($r2Context.governance_exception)", "adr_digest=$($r2Context.governance_exception_adr_sha256)") })

        $r2ReplayBefore = Get-InvocationCount; $r2ReplayReason = "accepted"; $r2ReplayChildStarted = $false
        try {
            $null = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
                -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r2ClaimPath `
                -ManifestPathOverride $surfacePath -ChildCommand $fake
        } catch {
            $r2ReplayReason = [string]$_.Exception.Data["reason"]
            $r2ReplayChildStarted = [bool]$_.Exception.Data["child_started"]
        }
        $r2ReplayAfter = Get-InvocationCount
        $r2ReplayPassed = $r2ReplayReason -ceq "governance_exception_already_consumed" -and -not $r2ReplayChildStarted -and
            $r2ReplayBefore -eq 1 -and $r2ReplayAfter -eq $r2ReplayBefore
        $cases.Add([ordered]@{ name = "adr0045-r2-second-identical-dispatch-denied"; expected_reason = "governance_exception_already_consumed";
            actual_reason = $r2ReplayReason; child_started = $r2ReplayChildStarted; child_invocation_count = $r2ReplayAfter; passed = $r2ReplayPassed;
            detail_codes = @("before=$r2ReplayBefore", "after=$r2ReplayAfter", "additional=$($r2ReplayAfter - $r2ReplayBefore)") })

        Reset-Sentinel
        $r2RaceClaimPath = Join-Path $tempRoot "adr0045-r2-race.claim"
        $r2ClaimBindingBytes = [IO.File]::ReadAllBytes($r2ClaimPath)
        $r2RaceResult = [Adr0045AtomicClaimRace]::Run($r2RaceClaimPath, $r2ClaimBindingBytes)
        $r2RaceUnexpectedErrors = @($r2RaceResult.UnexpectedErrors | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        $r2RaceContentExact = [IO.File]::Exists($r2RaceClaimPath) -and
            [Convert]::ToBase64String([IO.File]::ReadAllBytes($r2RaceClaimPath)) -ceq [Convert]::ToBase64String($r2ClaimBindingBytes)
        $r2RacePassed = $r2RaceResult.WinnerCount -eq 1 -and $r2RaceUnexpectedErrors.Count -eq 0 -and $r2RaceContentExact -and (Get-InvocationCount) -eq 0
        $cases.Add([ordered]@{ name = "adr0045-r2-create-new-race-single-winner"; expected_reason = "one_atomic_winner";
            actual_reason = if ($r2RacePassed) { "one_atomic_winner" } else { "claim_race_assertion_failed" };
            child_started = $false; child_invocation_count = Get-InvocationCount; passed = $r2RacePassed;
            detail_codes = @("winners=$($r2RaceResult.WinnerCount)", "content_exact=$r2RaceContentExact") + $r2RaceUnexpectedErrors })

        Reset-Sentinel
        $r2LeaseAdrPath = Join-Path $tempRoot "adr0045-r2-leased-authority.md"
        $r2LeaseClaimPath = Join-Path $tempRoot "adr0045-r2-lease.claim"
        $r2LeaseReplacementPath = Join-Path $tempRoot "adr0045-r2-replacement.md"
        $r2LeaseBackupPath = Join-Path $tempRoot "adr0045-r2-replaced-backup.md"
        [IO.File]::WriteAllBytes($r2LeaseAdrPath, $r2AuthorityBytes)
        [IO.File]::WriteAllText($r2LeaseReplacementPath, "# revoked R2 authority", (New-Object Text.UTF8Encoding($false)))
        $r2HookObservation = [pscustomobject]@{ calls = 0; replace_attempted = $false; replace_failed = $false; replace_error = "";
            delete_attempted = $false; delete_failed = $false; delete_error = ""; snapshot_digest = ""; prompt_exact = $false }
        $r2PreStartMutationHook = {
            param($state)
            $r2HookObservation.calls++
            $r2HookObservation.snapshot_digest = Get-Sha256Hex ([Convert]::FromBase64String($state.authority_bytes_base64))
            $r2HookObservation.prompt_exact = $state.authority_sha256 -ceq $Adr0045R2DecisionSha256 -and $state.prompt -ceq $expectedR2Prompt
            $r2HookObservation.replace_attempted = $true
            try { [IO.File]::Replace($r2LeaseReplacementPath, $state.authority_path, $r2LeaseBackupPath) }
            catch { $r2HookObservation.replace_failed = $true; $r2HookObservation.replace_error = $_.Exception.GetType().Name }
            $r2HookObservation.delete_attempted = $true
            try { [IO.File]::Delete($state.authority_path) }
            catch { $r2HookObservation.delete_failed = $true; $r2HookObservation.delete_error = $_.Exception.GetType().Name }
        }.GetNewClosure()
        $r2LeaseDispatch = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
            -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2LeaseAdrPath -Adr0045ClaimPathOverride $r2LeaseClaimPath `
            -Adr0045PreStartHook $r2PreStartMutationHook -ManifestPathOverride $surfacePath -ChildCommand $fake
        $r2LeasePrompt = Get-Content -Raw -LiteralPath $promptPath
        $r2LeaseAuthorityExact = [IO.File]::Exists($r2LeaseAdrPath) -and (Get-TestFileDigest $r2LeaseAdrPath) -ceq $Adr0045R2DecisionSha256
        $r2LeasePassed = $r2LeaseDispatch.child_started -and (Get-InvocationCount) -eq 1 -and $r2HookObservation.calls -eq 1 -and
            $r2HookObservation.replace_attempted -and $r2HookObservation.replace_failed -and
            $r2HookObservation.delete_attempted -and $r2HookObservation.delete_failed -and
            $r2HookObservation.snapshot_digest -ceq $Adr0045R2DecisionSha256 -and $r2HookObservation.prompt_exact -and
            $r2LeaseAuthorityExact -and $r2LeasePrompt -ceq $expectedR2Prompt
        $cases.Add([ordered]@{ name = "adr0045-r2-prestart-lease-blocks-replace-delete"; expected_reason = "accepted_with_mutation_denied";
            actual_reason = if ($r2LeasePassed) { "accepted_with_mutation_denied" } else { "lease_assertion_failed" };
            child_started = $r2LeaseDispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $r2LeasePassed;
            detail_codes = @("replace_failed=$($r2HookObservation.replace_failed):$($r2HookObservation.replace_error)",
                "delete_failed=$($r2HookObservation.delete_failed):$($r2HookObservation.delete_error)",
                "snapshot_digest=$($r2HookObservation.snapshot_digest)", "authority_exact=$r2LeaseAuthorityExact") })

        $r2ClaimBytesBeforeR3 = [IO.File]::ReadAllBytes($r2ClaimPath)
        $r3ClaimPath = Join-Path $tempRoot "adr0045-r3-positive.claim"
        [IO.File]::WriteAllText($order, $r3ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        $r3UnsupportedClaimPath = Join-Path $tempRoot "adr0045-r3-unsupported-command.claim"
        Run-Denial "adr0045-r3-unsupported-command-denies-before-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
                -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3UnsupportedClaimPath `
                -ManifestPathOverride $surfacePath -ChildCommand $unsupported
        } "codex_command_unsupported" "" { -not [IO.File]::Exists($r3UnsupportedClaimPath) }
        $r3MissingHostClaimPath = Join-Path $tempRoot "adr0045-r3-missing-host.claim"
        Run-Denial "adr0045-r3-missing-host-denies-before-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
                -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3MissingHostClaimPath `
                -ManifestPathOverride $surfacePath -ChildCommand $shim -PowerShellHostPathOverride $missingHost
        } "codex_powershell_host_missing" "" { -not [IO.File]::Exists($r3MissingHostClaimPath) }
        Reset-Sentinel
        $r3Dispatch = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
            -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3ClaimPath `
            -ManifestPathOverride $surfacePath -ChildCommand $fake
        $r3Args = [IO.File]::ReadAllLines($argsPath); $r3Prompt = Get-Content -Raw -LiteralPath $promptPath
        $r3Match = [regex]::Match($r3Prompt, '(?s)<raios-machine-context>\r?\n(.+?)\r?\n</raios-machine-context>')
        $r3Context = if ($r3Match.Success) { $r3Match.Groups[1].Value | ConvertFrom-Json } else { $null }
        $r3Claim = if ([IO.File]::Exists($r3ClaimPath)) { Get-Content -Raw -LiteralPath $r3ClaimPath | ConvertFrom-Json } else { $null }
        $expectedR3Context = [ordered]@{
            schema = "raios.machine_curated_context.v1"
            machine_manifest = "$Adr0045MachineId@sha256:$Adr0045ManifestSha256"
            required_fact_paths = @($Adr0045FactPath)
            facts = @([ordered]@{ path = $Adr0045FactPath; status = $exceptionSelectedFact.status; value = $exceptionSelectedFact.value; provenance = $exceptionSelectedFact.provenance })
        }
        $expectedR3Context["governance_exception"] = $Adr0045R3ExceptionToken
        $expectedR3Context["governance_exception_adr_sha256"] = $Adr0045R3DecisionSha256
        $expectedR3Json = ($expectedR3Context | ConvertTo-Json -Depth 20 -Compress).Replace('<', '\u003c').Replace('>', '\u003e')
        $expectedR3Prompt = $r3ExceptionOrderContent.TrimEnd() + $crlf + $crlf + "<raios-machine-context>" + $crlf + $expectedR3Json + $crlf + "</raios-machine-context>" + $crlf
        $r3ArgsExact = (@($r3Args) -join [char]0) -ceq ($expectedExceptionArgs -join [char]0)
        $r3PromptExact = $r3Prompt -ceq $expectedR3Prompt
        $r3ClaimExact = $null -ne $r3Claim -and $r3Claim.schema -ceq "raios.adr0045_h26_r3_claim.v1" -and
            $r3Claim.token -ceq $Adr0045R3ExceptionToken -and $r3Claim.machine -ceq $Adr0045MachineId -and
            $r3Claim.manifest_sha256 -ceq $Adr0045ManifestSha256 -and $r3Claim.fact_path -ceq $Adr0045FactPath -and
            $r3Claim.adr_sha256 -ceq $Adr0045R3DecisionSha256
        $r3PositivePassed = (Get-InvocationCount) -eq 1 -and $r3Dispatch.child_started -and $r3ArgsExact -and
            $r3PromptExact -and $r3Match.Success -and $r3Context.governance_exception -ceq $Adr0045R3ExceptionToken -and
            $r3Context.governance_exception_adr_sha256 -ceq $Adr0045R3DecisionSha256 -and $r3ClaimExact -and
            @($r3Context.required_fact_paths).Count -eq 1 -and $r3Context.required_fact_paths[0] -ceq $Adr0045FactPath -and
            @($r3Context.facts).Count -eq 1 -and $r3Context.facts[0].status -ceq "observed" -and
            $r3Context.facts[0].value -ceq $Adr0045FactValue
        $cases.Add([ordered]@{ name = "adr0045-r3-exact-positive-one-fake-child"; expected_reason = "accepted";
            actual_reason = if ($r3PositivePassed) { "accepted" } else { "positive_assertion_failed" };
            child_started = $r3Dispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $r3PositivePassed;
            detail_codes = @("argv_exact=$r3ArgsExact", "stdin_exact=$r3PromptExact", "claim_binding_exact=$r3ClaimExact",
                "exception=$($r3Context.governance_exception)", "adr_digest=$($r3Context.governance_exception_adr_sha256)") })

        $r3ReplayBefore = Get-InvocationCount; $r3ReplayReason = "accepted"; $r3ReplayChildStarted = $false
        try {
            $null = Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema `
                -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3ClaimPath `
                -ManifestPathOverride $surfacePath -ChildCommand $fake
        } catch {
            $r3ReplayReason = [string]$_.Exception.Data["reason"]
            $r3ReplayChildStarted = [bool]$_.Exception.Data["child_started"]
        }
        $r3ReplayAfter = Get-InvocationCount
        $r3ReplayPassed = $r3ReplayReason -ceq "governance_exception_already_consumed" -and -not $r3ReplayChildStarted -and
            $r3ReplayBefore -eq 1 -and $r3ReplayAfter -eq $r3ReplayBefore
        $cases.Add([ordered]@{ name = "adr0045-r3-second-identical-dispatch-denied"; expected_reason = "governance_exception_already_consumed";
            actual_reason = $r3ReplayReason; child_started = $r3ReplayChildStarted; child_invocation_count = $r3ReplayAfter; passed = $r3ReplayPassed;
            detail_codes = @("before=$r3ReplayBefore", "after=$r3ReplayAfter", "additional=$($r3ReplayAfter - $r3ReplayBefore)") })

        Reset-Sentinel
        $r3RaceClaimPath = Join-Path $tempRoot "adr0045-r3-race.claim"
        $r3ClaimBindingBytes = [IO.File]::ReadAllBytes($r3ClaimPath)
        $r3RaceResult = [Adr0045AtomicClaimRace]::Run($r3RaceClaimPath, $r3ClaimBindingBytes)
        $r3RaceUnexpectedErrors = @($r3RaceResult.UnexpectedErrors | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        $r3RaceContentExact = [IO.File]::Exists($r3RaceClaimPath) -and
            [Convert]::ToBase64String([IO.File]::ReadAllBytes($r3RaceClaimPath)) -ceq [Convert]::ToBase64String($r3ClaimBindingBytes)
        $r3RacePassed = $r3RaceResult.WinnerCount -eq 1 -and $r3RaceUnexpectedErrors.Count -eq 0 -and $r3RaceContentExact -and (Get-InvocationCount) -eq 0
        $cases.Add([ordered]@{ name = "adr0045-r3-create-new-race-single-winner"; expected_reason = "one_atomic_winner";
            actual_reason = if ($r3RacePassed) { "one_atomic_winner" } else { "claim_race_assertion_failed" };
            child_started = $false; child_invocation_count = Get-InvocationCount; passed = $r3RacePassed;
            detail_codes = @("winners=$($r3RaceResult.WinnerCount)", "content_exact=$r3RaceContentExact") + $r3RaceUnexpectedErrors })

        $r1Binding = Get-Adr0045ExceptionBinding $Adr0045ExceptionToken
        $r2Binding = Get-Adr0045ExceptionBinding $Adr0045R2ExceptionToken
        $r3Binding = Get-Adr0045ExceptionBinding $Adr0045R3ExceptionToken
        [IO.File]::WriteAllText($order, $r2ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r2-cannot-use-r1-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $exceptionClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_already_consumed"
        [IO.File]::WriteAllText($order, $exceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r1-cannot-use-r2-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $r2ClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_already_consumed"
        [IO.File]::WriteAllText($order, $r3ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r3-cannot-use-r1-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $exceptionClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_already_consumed"
        Run-Denial "adr0045-r3-cannot-use-r2-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r2ClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_already_consumed"
        [IO.File]::WriteAllText($order, $exceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r1-cannot-use-r3-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $r3ClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_already_consumed"
        [IO.File]::WriteAllText($order, $r2ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r2-cannot-use-r3-claim" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r3ClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_already_consumed"
        $r1ClaimStillExact = [Convert]::ToBase64String([IO.File]::ReadAllBytes($exceptionClaimPath)) -ceq [Convert]::ToBase64String($r1ClaimBytesBeforeR2)
        $r2ClaimStillExact = [Convert]::ToBase64String([IO.File]::ReadAllBytes($r2ClaimPath)) -ceq [Convert]::ToBase64String($r2ClaimBytesBeforeR3)
        $r3ClaimStillExact = [Convert]::ToBase64String([IO.File]::ReadAllBytes($r3ClaimPath)) -ceq [Convert]::ToBase64String($r3ClaimBindingBytes)
        $crossBindingPassed = $r1Binding.claim_relative_path -ceq "target\state\adr0045-h26.claim" -and
            $r2Binding.claim_relative_path -ceq "target\state\adr0045-h26-r2.claim" -and
            $r3Binding.claim_relative_path -ceq "target\state\adr0045-h26-r3.claim" -and
            $r1Binding.claim_relative_path -cne $r2Binding.claim_relative_path -and
            $r1Binding.claim_relative_path -cne $r3Binding.claim_relative_path -and
            $r2Binding.claim_relative_path -cne $r3Binding.claim_relative_path -and
            $r1ClaimStillExact -and $r2ClaimStillExact -and $r3ClaimStillExact
        $cases.Add([ordered]@{ name = "adr0045-r1-r2-r3-cross-claim-isolation"; expected_reason = "isolated_immutable_claims";
            actual_reason = if ($crossBindingPassed) { "isolated_immutable_claims" } else { "cross_claim_assertion_failed" };
            child_started = $false; child_invocation_count = Get-InvocationCount; passed = $crossBindingPassed;
            detail_codes = @("r1_path=$($r1Binding.claim_relative_path)", "r2_path=$($r2Binding.claim_relative_path)",
                "r3_path=$($r3Binding.claim_relative_path)", "r1_unchanged=$r1ClaimStillExact",
                "r2_unchanged=$r2ClaimStillExact", "r3_unchanged=$r3ClaimStillExact") })

        $r2NegativeClaimPath = Join-Path $tempRoot "adr0045-r2-negative.claim"
        [IO.File]::WriteAllText($order, $r2ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r2-wrong-token" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException "ADR-0045-H26-R4" -Adr0045ClaimPathOverride $r2NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_invalid"
        Run-Denial "adr0045-r2-wrong-manifest-digest" {
            Invoke-CodexLaneGate $order $Adr0045MachineId ("0" * 64) $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r2NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "manifest_digest_mismatch"
        Run-Denial "adr0045-r2-wrong-fact-path" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest @("/cpu/model") "workspace-write" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r2NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_fact_paths_mismatch"
        Run-Denial "adr0045-r2-wrong-sandbox" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "read-only" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r2NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_sandbox_mismatch"
        Run-Denial "adr0045-r2-old-adr-digest" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $r2NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_adr_digest_mismatch"
        $m = Copy-JsonObject $surface; ($m.devices | Where-Object { $_.id -ceq "wifi" }).identity.value = "Marvell 88W8898"
        $path = Write-Mutant "adr0045-r2-wrong-fact-value" $m; $digest = Get-TestFileDigest $path
        Run-Denial "adr0045-r2-wrong-fact-value" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $digest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r2NegativeClaimPath -ManifestPathOverride $path -ChildCommand $fake
        } "governance_exception_manifest_digest_mismatch"
        [IO.File]::WriteAllText($order, $exceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r2-wrong-order-marker" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R2ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r2NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_order_marker_missing"
        [IO.File]::WriteAllText($order, $exceptionOrderContent, (New-Object Text.UTF8Encoding($false)))

        $r3NegativeClaimPath = Join-Path $tempRoot "adr0045-r3-negative.claim"
        [IO.File]::WriteAllText($order, $r3ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r3-wrong-token" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException "ADR-0045-H26-R4" -Adr0045ClaimPathOverride $r3NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_invalid"
        Run-Denial "adr0045-r3-wrong-manifest-digest" {
            Invoke-CodexLaneGate $order $Adr0045MachineId ("0" * 64) $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "manifest_digest_mismatch"
        Run-Denial "adr0045-r3-wrong-fact-path" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest @("/cpu/model") "workspace-write" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_fact_paths_mismatch"
        Run-Denial "adr0045-r3-wrong-sandbox" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "read-only" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_sandbox_mismatch"
        Run-Denial "adr0045-r3-old-adr-digest" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r2ExceptionAdrPath -Adr0045ClaimPathOverride $r3NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_adr_digest_mismatch"
        $m = Copy-JsonObject $surface; ($m.devices | Where-Object { $_.id -ceq "wifi" }).identity.value = "Marvell 88W8898"
        $path = Write-Mutant "adr0045-r3-wrong-fact-value" $m; $digest = Get-TestFileDigest $path
        Run-Denial "adr0045-r3-wrong-fact-value" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $digest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3NegativeClaimPath -ManifestPathOverride $path -ChildCommand $fake
        } "governance_exception_manifest_digest_mismatch"
        [IO.File]::WriteAllText($order, $r2ExceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-r3-wrong-order-marker" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045R3ExceptionToken -Adr0045DecisionPathOverride $r3ExceptionAdrPath -Adr0045ClaimPathOverride $r3NegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_order_marker_missing"
        [IO.File]::WriteAllText($order, $exceptionOrderContent, (New-Object Text.UTF8Encoding($false)))

        $exceptionNegativeClaimPath = Join-Path $tempRoot "adr0045-negative.claim"
        Run-Denial "adr0045-explicit-empty-token-stays-not-ready" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException "" -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "manifest_not_context_ready"
        Run-Denial "adr0045-wrong-token" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException "ADR-0045-H25" -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_invalid"
        Run-Denial "adr0045-wrong-machine" {
            Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $qemuPath -ChildCommand $fake
        } "governance_exception_machine_mismatch"
        Run-Denial "adr0045-wrong-manifest-digest" {
            Invoke-CodexLaneGate $order $Adr0045MachineId ("0" * 64) $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "manifest_digest_mismatch"
        Run-Denial "adr0045-wrong-fact-path" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest @("/cpu/model") "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_fact_paths_mismatch"
        Run-Denial "adr0045-additional-fact-path" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest @($Adr0045FactPath, "/cpu/model") "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_fact_paths_mismatch"

        $m = Copy-JsonObject $surface; ($m.devices | Where-Object { $_.id -ceq "wifi" }).identity.value = "Marvell 88W8898"
        $path = Write-Mutant "adr0045-wrong-fact-value" $m; $digest = Get-TestFileDigest $path
        Run-Denial "adr0045-wrong-fact-value" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $digest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $path -ChildCommand $fake
        } "governance_exception_manifest_digest_mismatch"
        $m = Copy-JsonObject $surface; $wifiIdentity = ($m.devices | Where-Object { $_.id -ceq "wifi" }).identity; $wifiIdentity.status = "unknown"; $wifiIdentity.value = $null
        $path = Write-Mutant "adr0045-wrong-fact-status" $m; $digest = Get-TestFileDigest $path
        Run-Denial "adr0045-wrong-fact-status" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $digest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $path -ChildCommand $fake
        } "manifest_schema_invalid" "missing_fact_not_declared"

        [IO.File]::WriteAllText($order, $orderContent, (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-missing-order-marker" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_order_marker_missing"
        [IO.File]::WriteAllText($order, ($orderContent + [Environment]::NewLine + "Governance exception: ADR-0045-H25"), (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-wrong-order-marker" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $exceptionAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_order_marker_missing"

        [IO.File]::WriteAllText($order, $exceptionOrderContent, (New-Object Text.UTF8Encoding($false)))
        $missingAdrPath = Join-Path $tempRoot "missing-adr0045.md"
        Run-Denial "adr0045-missing-adr-file" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $missingAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_adr_not_found"
        $changedAdrPath = Join-Path $tempRoot "changed-adr0045.md"
        [IO.File]::WriteAllText($changedAdrPath, "# changed authority", (New-Object Text.UTF8Encoding($false)))
        Run-Denial "adr0045-changed-adr-file" {
            Invoke-CodexLaneGate $order $Adr0045MachineId $surfaceDigest $exceptionFactPaths "workspace-write" $report $Schema -GovernanceException $Adr0045ExceptionToken -Adr0045DecisionPathOverride $changedAdrPath -Adr0045ClaimPathOverride $exceptionNegativeClaimPath -ManifestPathOverride $surfacePath -ChildCommand $fake
        } "governance_exception_adr_digest_mismatch"
        [IO.File]::WriteAllText($order, $orderContent, (New-Object Text.UTF8Encoding($false)))

        Run-Denial "required-fact-path-missing" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest @("/devices/not-present/identity") "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake } "required_fact_path_unknown"
        Run-Denial "required-fact-pointer-case" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest @("/CPU/model") "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake } "required_fact_path_unknown"
        Run-Denial "required-fact-unknown" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest @("/devices/q35/bdf") "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake } "required_fact_unknown"
        $m = Copy-JsonObject $qemu; $m.cpu.model.provenance.source_ref = "x" * 513; $path = Write-Mutant "schema-string-bound" $m; $digest = Get-TestFileDigest $path
        Run-Denial "schema-string-bound" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $digest @("/cpu/model") "read-only" $report $Schema -ManifestPathOverride $path -ChildCommand $fake } "manifest_schema_invalid" "string_too_long"

        function Run-RepinnedControlledMutation([string]$Name, [scriptblock]$Mutation) {
            $mutant = Copy-JsonObject $qemu; & $Mutation $mutant
            $mutantPath = Write-Mutant $Name $mutant; $mutantDigest = Get-TestFileDigest $mutantPath
            $testOrder = $order; $testReport = $report; $testSchema = $Schema; $testChild = $fake
            Run-Denial $Name ({ Invoke-CodexLaneGate $testOrder "qemu-q35-shadow" $mutantDigest @("/cpu/model") "read-only" $testReport $testSchema -ManifestPathOverride $mutantPath -ChildCommand $testChild }.GetNewClosure()) "manifest_schema_invalid" "qemu_controlled_truth_drift"
        }
        Run-RepinnedControlledMutation "repinned-invented-bdf" { param($x) $x.devices[1].bdf.status = "observed"; $x.devices[1].bdf.value = "0000:00:01.0" }
        Run-RepinnedControlledMutation "repinned-invented-resource" { param($x) $x.devices[1].resources = @([pscustomobject]@{ kind = "mmio"; start = 4096; length = 4096; status = "observed"; provenance = Copy-JsonObject $x.devices[1].provenance }) }
        Run-RepinnedControlledMutation "repinned-invented-feature" { param($x) $f = Copy-JsonObject $x.cpu.features[0]; $f.name = "invented-feature"; $x.cpu.features = @($x.cpu.features) + @($f) }
        Run-RepinnedControlledMutation "repinned-invented-device" { param($x) $d = Copy-JsonObject $x.devices[1]; $d.id = "invented-device"; $x.devices = @($x.devices) + @($d) }
        Run-RepinnedControlledMutation "repinned-invented-region" { param($x) $x.memory.regions[0].length_bytes = 268435456; $r = Copy-JsonObject $x.memory.regions[0]; $r.id = "invented-region"; $r.start_bytes = 268435456; $x.memory.regions = @($x.memory.regions) + @($r) }
        Run-RepinnedControlledMutation "repinned-false-provenance" { param($x) $x.cpu.model.provenance.source_ref = "invented source" }

        Reset-Sentinel
        $exactOrderBytes = (New-Object Text.UTF8Encoding($false)).GetBytes("x" * $LaneOrderMaxBytes)
        [IO.File]::WriteAllBytes($order, $exactOrderBytes)
        $exactOrderDispatch = Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest @("/cpu/model") "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake
        $exactOrderPassed = $exactOrderBytes.Length -eq $LaneOrderMaxBytes -and $exactOrderDispatch.child_started -and (Get-InvocationCount) -eq 1
        $cases.Add([ordered]@{ name = "lane-order-exact-boundary"; expected_reason = "accepted"; actual_reason = if ($exactOrderPassed) { "accepted" } else { "boundary_assertion_failed" }; child_started = $exactOrderDispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $exactOrderPassed; detail_codes = @("bytes=$($exactOrderBytes.Length)") })
        [IO.File]::WriteAllText($order, $orderContent, (New-Object Text.UTF8Encoding($false)))

        Reset-Sentinel
        $qemuTextBytes = (New-Object Text.UTF8Encoding($false)).GetBytes((Get-Content -Raw -LiteralPath $qemuPath))
        [byte[]]$exactManifestBytes = [byte[]]::new($MachineManifestMaxBytes)
        [Array]::Copy($qemuTextBytes, $exactManifestBytes, $qemuTextBytes.Length)
        for ($i = $qemuTextBytes.Length; $i -lt $exactManifestBytes.Length; $i++) { $exactManifestBytes[$i] = 0x20 }
        $exactManifestPath = Join-Path $tempRoot "exact-boundary-manifest.json"; [IO.File]::WriteAllBytes($exactManifestPath, $exactManifestBytes)
        $exactManifestDigest = Get-Sha256Hex $exactManifestBytes $exactManifestBytes.Length
        $exactManifestDispatch = Invoke-CodexLaneGate $order "qemu-q35-shadow" $exactManifestDigest @("/cpu/model") "read-only" $report $Schema -ManifestPathOverride $exactManifestPath -ChildCommand $fake
        $exactManifestPassed = $exactManifestBytes.Length -eq $MachineManifestMaxBytes -and $exactManifestDispatch.child_started -and (Get-InvocationCount) -eq 1
        $cases.Add([ordered]@{ name = "manifest-exact-boundary"; expected_reason = "accepted"; actual_reason = if ($exactManifestPassed) { "accepted" } else { "boundary_assertion_failed" }; child_started = $exactManifestDispatch.child_started; child_invocation_count = Get-InvocationCount; passed = $exactManifestPassed; detail_codes = @("bytes=$($exactManifestBytes.Length)") })

        $m = Copy-JsonObject $qemu; $m.machine.display_name = "x" * $MachineManifestMaxBytes; $path = Write-Mutant "whole-manifest-oversize" $m
        Run-Denial "whole-manifest-oversize-unselected-field" { Invoke-CodexLaneGate $order "qemu-q35-shadow" ("0" * 64) @("/cpu/model") "read-only" $report $Schema -ManifestPathOverride $path -ChildCommand $fake } "manifest_too_large"

        [IO.File]::WriteAllText($order, ("x" * ($LaneOrderMaxBytes + 1)), (New-Object Text.UTF8Encoding($false)))
        Run-Denial "lane-order-oversize" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake } "lane_order_too_large"
        [IO.File]::WriteAllText($order, $orderContent, (New-Object Text.UTF8Encoding($false)))

        $orderGrowthState = [pscustomobject]@{ stream = $null; opens = @{} }
        $orderGrowthFactory = {
            param($candidatePath)
            if (-not $orderGrowthState.opens.ContainsKey($candidatePath)) { $orderGrowthState.opens[$candidatePath] = 0 }
            $orderGrowthState.opens[$candidatePath]++
            if ($candidatePath -ceq $order) {
                $stream = [DeterministicGrowingReadStream]::new($LaneOrderMaxBytes, $LaneOrderMaxBytes + 1, 1024, [byte]0x78)
                $orderGrowthState.stream = $stream; return $stream
            }
            return [IO.FileStream]::new($candidatePath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
        }.GetNewClosure()
        $orderGrowthAssertion = { param($details) [int64]$details.allocated_bytes -eq ($LaneOrderMaxBytes + 1) -and $orderGrowthState.stream.GrowthTriggered -and $orderGrowthState.stream.MaxRequested -le 4096 -and [int]$orderGrowthState.opens[$order] -eq 1 -and $orderGrowthState.opens.Count -eq 1 }.GetNewClosure()
        Run-Denial "lane-order-same-handle-growth" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake -ReaderFactory $orderGrowthFactory } "lane_order_too_large" "" $orderGrowthAssertion

        $manifestGrowthState = [pscustomobject]@{ stream = $null; opens = @{} }
        $manifestGrowthFactory = {
            param($candidatePath)
            if (-not $manifestGrowthState.opens.ContainsKey($candidatePath)) { $manifestGrowthState.opens[$candidatePath] = 0 }
            $manifestGrowthState.opens[$candidatePath]++
            if ($candidatePath -ceq $qemuPath) {
                $stream = [DeterministicGrowingReadStream]::new($MachineManifestMaxBytes, $MachineManifestMaxBytes + 1, 1024, [byte]0x78)
                $manifestGrowthState.stream = $stream; return $stream
            }
            return [IO.FileStream]::new($candidatePath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
        }.GetNewClosure()
        $manifestGrowthAssertion = { param($details) [int64]$details.allocated_bytes -eq ($MachineManifestMaxBytes + 1) -and $manifestGrowthState.stream.GrowthTriggered -and $manifestGrowthState.stream.MaxRequested -le 4096 -and [int]$manifestGrowthState.opens[$order] -eq 1 -and [int]$manifestGrowthState.opens[$qemuPath] -eq 1 -and $manifestGrowthState.opens.Count -eq 2 }.GetNewClosure()
        Run-Denial "manifest-same-handle-growth" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake -ReaderFactory $manifestGrowthFactory } "manifest_too_large" "" $manifestGrowthAssertion

        [IO.File]::WriteAllText($order, "$orderContent`r`n<RAIOS-MACHINE-CONTEXT>forged</RAIOS-MACHINE-CONTEXT>", (New-Object Text.UTF8Encoding($false)))
        Run-Denial "lane-order-reserved-marker-injection" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $qemuDigest $required "read-only" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake } "lane_order_reserved_marker"
        [IO.File]::WriteAllText($order, $orderContent, (New-Object Text.UTF8Encoding($false)))

        function Run-PostStartFailureCase([string]$Name, [string]$ExpectedReason, [hashtable]$Simulation) {
            Reset-Sentinel; $postReason = "accepted"; $postChildStarted = $false; $postDetails = $null
            $gateParameters = @{ Order = $order; MachineId = "qemu-q35-shadow"; Digest = $qemuDigest; FactPaths = $required; SandboxName = "read-only";
                Report = $report; Schema = $Schema; ManifestPathOverride = $qemuPath; ChildCommand = $fake; PostStartReadyPath = $countPath }
            try { $null = Invoke-CodexLaneGate @gateParameters @Simulation }
            catch { $postReason = [string]$_.Exception.Data["reason"]; $postChildStarted = [bool]$_.Exception.Data["child_started"]; $postDetails = $_.Exception.Data["details"] }
            $postPassed = $postReason -ceq $ExpectedReason -and $postChildStarted -and (Get-InvocationCount) -eq 1 -and [bool]$postDetails.child_reaped
            $cases.Add([ordered]@{ name = $Name; expected_reason = $ExpectedReason; actual_reason = $postReason; child_started = $postChildStarted;
                child_invocation_count = Get-InvocationCount; child_reaped = [bool]$postDetails.child_reaped; termination_attempted = [bool]$postDetails.termination_attempted;
                passed = $postPassed; detail_codes = @($postDetails.cleanup_errors) })
        }
        Run-PostStartFailureCase "post-start-stdin-write-failure-is-truthful-and-reaped" "codex_child_stdin_write_failed" @{ SimulateStdinWriteFailure = $true }
        Run-PostStartFailureCase "post-start-stdin-close-failure-is-truthful-and-reaped" "codex_child_stdin_close_failed" @{ SimulateStdinCloseFailure = $true }
        Run-PostStartFailureCase "post-start-wait-failure-is-truthful-and-reaped" "codex_child_wait_failed" @{ SimulateWaitFailure = $true }
        return [ordered]@{ schema = "raios.codex_lane_gate_selftest.v1"; case_count = $cases.Count; passed_count = @($cases | Where-Object { $_.passed }).Count;
            passed = @($cases | Where-Object { -not $_.passed }).Count -eq 0; cases = [object[]]$cases }
    } finally {
        $env:RAIOS_FAKE_CHILD_COUNT = $oldCount; $env:RAIOS_FAKE_CHILD_ARGS = $oldArgs; $env:RAIOS_FAKE_CHILD_PROMPT = $oldPrompt
        $resolvedTemp = [IO.Path]::GetFullPath($tempRoot); $systemTemp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
        if ($resolvedTemp.StartsWith($systemTemp, [StringComparison]::OrdinalIgnoreCase) -and [IO.Directory]::Exists($resolvedTemp)) { Remove-Item -LiteralPath $resolvedTemp -Recurse -Force }
    }
}

try {
    $launcherSchema = Get-MachineManifestSchema
    if ($LauncherSelfTest) {
        $result = Invoke-CodexLaneSelfTests $launcherSchema
        $result | ConvertTo-Json -Depth 50
        if (-not $result.passed) { exit 1 }; exit 0
    }
    $result = Invoke-CodexLaneGate $OrderPath $ExpectedMachineId $ExpectedManifestSha256 $RequiredFactPath $Sandbox $ReportPath $launcherSchema -GovernanceException $GovernanceException
    $result | ConvertTo-Json -Depth 20
    exit [int]$result.child_exit_code
} catch {
    $reason = [string]$_.Exception.Data["reason"]
    if ([string]::IsNullOrWhiteSpace($reason)) { $reason = "launcher_internal_error" }
    $childStarted = [bool]$_.Exception.Data["child_started"]
    [ordered]@{ schema = "raios.codex_lane_dispatch.v1"; accepted = $false; child_started = $childStarted; reason = $reason;
        message = $_.Exception.Message; details = $_.Exception.Data["details"] } | ConvertTo-Json -Depth 50
    exit 1
}
