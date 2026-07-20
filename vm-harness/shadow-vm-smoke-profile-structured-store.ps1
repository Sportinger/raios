if (-not $StructuredStoreDiskImage -or -not $StructuredStoreFixture -or -not $PersistDiskImage) {
    throw "structured-store profile requires separate C1 store and valid-a BOOTCTL fixtures"
}

$fixtureReady = (
    [bool]$StructuredStoreFixture.valid -and
    [bool]$StructuredStoreFixture.disposable_qemu_only -and
    $StructuredStoreFixture.store_state -eq "empty_unformatted" -and
    (Test-Path -LiteralPath $PersistDiskImage)
)
Add-Predicate `
    -Name "structured-store:host_fixture_ready" `
    -Expected "a dedicated empty C1 store plus a separate valid-a SEED_DATA BOOTCTL fixture" `
    -Passed $fixtureReady `
    -Actual $(if ($fixtureReady) { $StructuredStoreDiskImage } else { $StructuredStoreFixture | ConvertTo-Json -Compress -Depth 6 })

if (-not $fixtureReady) {
    throw "Structured-store fixture is not the dedicated empty QEMU test image"
}

# The first boot owns format/open/append. Stop it explicitly and boot the same
# fixture again so the replay proof cannot be a same-boot self-report.
foreach ($marker in @(
    "C1_STRUCTURED_STORE_FIXTURE_ACCEPTED",
    "C1_STRUCTURED_STORE_FORMAT_OPEN_OK",
    "C1_VAULT_COMPLETE_REPLAY_BOUND",
    "C1_VAULT_CORE_POLICY_BOUND",
    "C1_STRUCTURED_STORE_APPEND_FLUSH_READBACK_OK"
)) {
    Assert-LogContains `
        -Name "structured-store:$marker" `
        -Needle $marker `
        -TimeoutSeconds $TimeoutSeconds
}

$firstBootLog = $SerialLog
if (-not $QemuPid) {
    throw "structured-store profile cannot reboot without the first QEMU process"
}
$firstQemuPid = $QemuPid
Stop-Process -Id $firstQemuPid -Force -ErrorAction Stop
if ($script:QemuProcess) {
    try {
        $script:QemuProcess.WaitForExit(5000) | Out-Null
    }
    catch {
    }
}
if (Get-Process -Id $firstQemuPid -ErrorAction SilentlyContinue) {
    throw "structured-store first QEMU process did not stop before reboot"
}

$rebootLog = Join-Path $RunDir "serial-structured-store-reboot.log"
$rebootParams = $runParams.Clone()
$rebootParams.StopExisting = $false
$rebootParams.SerialLog = $rebootLog
$rebootOutput = & $RunScript @rebootParams
$SerialLog = $rebootLog
$QemuPid = $null
foreach ($line in $rebootOutput) {
    if ($line -match '^qemu pid:\s*(\d+)') {
        $QemuPid = [int]$Matches[1]
    }
}
if (-not $QemuPid) {
    throw "structured-store reboot did not return a QEMU pid"
}
try {
    $script:QemuProcess = Get-Process -Id $QemuPid -ErrorAction Stop
}
catch {
    $script:QemuProcess = $null
}

Assert-LogContains -Name "structured-store:reboot_serial_console_ready" -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds
Assert-LogContains `
    -Name "structured-store:C1_STRUCTURED_STORE_REBOOT_REPLAY_OK" `
    -Needle "C1_STRUCTURED_STORE_REBOOT_REPLAY_OK" `
    -TimeoutSeconds $TimeoutSeconds
Assert-LogContains `
    -Name "structured-store:reboot_vault_complete_replay_bound" `
    -Needle "C1_VAULT_COMPLETE_REPLAY_BOUND" `
    -TimeoutSeconds $TimeoutSeconds
Assert-LogContains `
    -Name "structured-store:reboot_vault_core_policy_bound" `
    -Needle "C1_VAULT_CORE_POLICY_BOUND" `
    -TimeoutSeconds $TimeoutSeconds

# Keep one report with a serial hash covering both boots; no first-boot report
# is produced while the test plate is mid-transaction.
$combinedLog = Join-Path $RunDir "serial-structured-store-combined.log"
$firstBootContent = Get-Content -LiteralPath $firstBootLog -Raw -ErrorAction Stop
$rebootContent = Get-Content -LiteralPath $rebootLog -Raw -ErrorAction Stop
Set-Content -LiteralPath $combinedLog -Value ($firstBootContent + [Environment]::NewLine + $rebootContent) -Encoding UTF8
$SerialLog = $combinedLog
