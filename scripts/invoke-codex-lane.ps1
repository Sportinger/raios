[CmdletBinding(DefaultParameterSetName = "Dispatch")]
param(
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$OrderPath = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$ExpectedMachineId = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$ExpectedManifestSha256 = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string[]]$RequiredFactPath = @(),
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$Sandbox = "",
    [Parameter(Mandatory = $true, ParameterSetName = "Dispatch")][string]$ReportPath = "",
    [Parameter(Mandatory = $true, ParameterSetName = "SelfTest")][switch]$SelfTest
)

$ErrorActionPreference = "Stop"
$LauncherRepoRoot = Split-Path -Parent $PSScriptRoot
$LauncherSelfTest = [bool]$SelfTest
$MachineManifestMaxBytes = 65536 # Fixed whole-file ceiling; the reader allocates one extra detection byte.
$LaneOrderMaxBytes = 65536       # Fixed whole-file ceiling for the authored lane order.
$MachineContextMaxChars = 16384
$ReservedMachineContextToken = "raios-machine-context"
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

function Invoke-CodexLaneGate {
    param(
        [string]$Order, [string]$MachineId, [string]$Digest, [string[]]$FactPaths,
        [string]$SandboxName, [string]$Report, [object]$Schema,
        [string]$ManifestPathOverride = "", [string]$ChildCommand = "codex", [string[]]$ChildPrefixArguments = @(),
        [switch]$SimulateStdinWriteFailure, [switch]$SimulateStdinCloseFailure, [switch]$SimulateWaitFailure,
        [string]$PostStartReadyPath = "", [AllowNull()][scriptblock]$ReaderFactory = $null
    )
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
    if (-not $manifest.curated_context_ready) { Throw-LaneGateDenial "manifest_not_context_ready" "Manifest '$MachineId' is valid but not curated-context ready" $manifest.missing_required_facts }

    $selectedFacts = @()
    foreach ($path in $FactPaths) {
        $fact = Resolve-MachineManifestFact $manifest $path
        if (-not $fact.found) { Throw-LaneGateDenial "required_fact_path_unknown" "Required fact path '$path' does not exist" $path }
        if ($fact.status -cne "observed") { Throw-LaneGateDenial "required_fact_unknown" "Required fact '$path' is not observed" $fact }
        $selectedFacts += [ordered]@{ path = $path; status = $fact.status; value = $fact.value; provenance = $fact.provenance }
    }
    try {
        $context = [ordered]@{
            schema = "raios.machine_curated_context.v1"
            machine_manifest = "$MachineId@sha256:$actualDigest"
            required_fact_paths = @($FactPaths)
            facts = @($selectedFacts)
        }
        $contextJson = $context | ConvertTo-Json -Depth 20 -Compress
        $contextJson = $contextJson.Replace('<', '\u003c').Replace('>', '\u003e')
        if ([string]::IsNullOrWhiteSpace($contextJson) -or $contextJson.Length -gt $MachineContextMaxChars) { throw "Rendered machine context exceeds the $MachineContextMaxChars-character bound" }
        $prompt = $orderText.TrimEnd() + "`r`n`r`n<raios-machine-context>`r`n" + $contextJson + "`r`n</raios-machine-context>"
    } catch {
        if ($_.Exception.Data["reason"]) { throw }
        Throw-LaneGateDenial "prompt_render_failed" "Bounded lane prompt could not be rendered" $_.Exception.Message
    }
    try { $resolvedChild = (Get-Command $ChildCommand -ErrorAction Stop).Source }
    catch { Throw-LaneGateDenial "codex_command_missing" "Codex child command is unavailable" $ChildCommand }

    $arguments = @($ChildPrefixArguments) + @("exec", "-s", $SandboxName, "-C", $LauncherRepoRoot, "--ephemeral", "-o", $reportFullPath, "-")
    $startInfo = New-Object Diagnostics.ProcessStartInfo
    $startInfo.FileName = $resolvedChild
    $startInfo.Arguments = (@($arguments | ForEach-Object { ConvertTo-NativeArgument ([string]$_) }) -join " ")
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardInput = $true
    $process = New-Object Diagnostics.Process
    $process.StartInfo = $startInfo
    try { $started = $process.Start() }
    catch {
        $startError = $_.Exception.Message
        $process.Dispose()
        Throw-LaneGateDenial "codex_child_start_failed" "Codex child process could not be started" $startError
    }
    if (-not $started) {
        $process.Dispose()
        Throw-LaneGateDenial "codex_child_start_failed" "Codex child process did not start"
    }
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
'@
        Add-Type -TypeDefinition $growingReaderSource -Language CSharp
        $env:RAIOS_FAKE_CHILD_COUNT = $countPath; $env:RAIOS_FAKE_CHILD_ARGS = $argsPath; $env:RAIOS_FAKE_CHILD_PROMPT = $promptPath
        $qemuPath = Join-Path $LauncherRepoRoot "hardware\manifests\qemu-q35-shadow.v1.json"
        $surfacePath = Join-Path $LauncherRepoRoot "hardware\manifests\surface-pro-4.v1.json"
        $qemu = Get-Content -Raw -LiteralPath $qemuPath | ConvertFrom-Json
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
        $expectedPrompt = $orderContent.TrimEnd() + "`r`n`r`n<raios-machine-context>`r`n" + $expectedContextJson + "`r`n</raios-machine-context>"
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

        $malformed = Join-Path $tempRoot "malformed.json"; [IO.File]::WriteAllText($malformed, '{', (New-Object Text.UTF8Encoding($false))); $malformedDigest = Get-TestFileDigest $malformed
        Run-Denial "malformed-json" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $malformedDigest $required "workspace-write" $report $Schema -ManifestPathOverride $malformed -ChildCommand $fake } "manifest_json_invalid"
        $m = Copy-JsonObject (Get-Content -Raw -LiteralPath $qemuPath | ConvertFrom-Json); $m | Add-Member unexpected $true; $path = Write-Mutant "additional" $m; $digest = Get-TestFileDigest $path
        Run-Denial "additional-property" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $digest $required "workspace-write" $report $Schema -ManifestPathOverride $path -ChildCommand $fake } "manifest_schema_invalid" "additional_property"
        $m = Copy-JsonObject (Get-Content -Raw -LiteralPath $qemuPath | ConvertFrom-Json); $m.curated_context_ready = "true"; $path = Write-Mutant "wrong-type" $m; $digest = Get-TestFileDigest $path
        Run-Denial "wrong-json-type" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $digest $required "workspace-write" $report $Schema -ManifestPathOverride $path -ChildCommand $fake } "manifest_schema_invalid" "type_mismatch"
        Run-Denial "wrong-machine" { Invoke-CodexLaneGate $order "qemu-q35-shadow" $surfaceDigest $required "workspace-write" $report $Schema -ManifestPathOverride $surfacePath -ChildCommand $fake } "machine_id_mismatch"
        Run-Denial "digest-drift" { Invoke-CodexLaneGate $order "qemu-q35-shadow" ("0" * 64) $required "workspace-write" $report $Schema -ManifestPathOverride $qemuPath -ChildCommand $fake } "manifest_digest_mismatch"
        Run-Denial "surface-not-ready" { Invoke-CodexLaneGate $order "surface-pro-4" $surfaceDigest @("/cpu/model") "read-only" $report $Schema -ManifestPathOverride $surfacePath -ChildCommand $fake } "manifest_not_context_ready"
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
    $result = Invoke-CodexLaneGate $OrderPath $ExpectedMachineId $ExpectedManifestSha256 $RequiredFactPath $Sandbox $ReportPath $launcherSchema
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
