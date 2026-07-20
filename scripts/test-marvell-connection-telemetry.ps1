param([switch]$SkipBuild)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Require([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Require-Match([string]$Text, [string]$Pattern, [string]$Message) {
    Require ([regex]::IsMatch($Text, $Pattern, [Text.RegularExpressions.RegexOptions]::Singleline)) $Message
}

function Require-NoMatch([string]$Text, [string]$Pattern, [string]$Message) {
    Require (-not [regex]::IsMatch($Text, $Pattern, [Text.RegularExpressions.RegexOptions]::Singleline)) $Message
}

function Slice-Between([string]$Text, [string]$Start, [string]$End) {
    $startAt = $Text.IndexOf($Start, [StringComparison]::Ordinal)
    Require ($startAt -ge 0) "missing fixture start: $Start"
    $endAt = $Text.IndexOf($End, $startAt + $Start.Length, [StringComparison]::Ordinal)
    Require ($endAt -gt $startAt) "missing fixture end: $End"
    $Text.Substring($startAt, $endAt - $startAt)
}

$corePath = Join-Path $RepoRoot "crates\raios-core\src\marvell_wifi_cmd.rs"
$supplicantPath = Join-Path $RepoRoot "crates\raios-core\src\marvell_wifi_supplicant.rs"
$driverPath = Join-Path $RepoRoot "seed-kernel\src\marvell_wifi_pcie.rs"
$pciPath = Join-Path $RepoRoot "seed-kernel\src\pci.rs"
$uiPath = Join-Path $RepoRoot "seed-kernel\src\shell_host\wifi_flow.rs"
$core = Get-Content -LiteralPath $corePath -Raw
$supplicant = Get-Content -LiteralPath $supplicantPath -Raw
$driver = Get-Content -LiteralPath $driverPath -Raw
$pci = Get-Content -LiteralPath $pciPath -Raw
$ui = Get-Content -LiteralPath $uiPath -Raw

$allocator = Slice-Between $core "pub struct SingleBssHostCmdSequenceAllocator" "pub struct MarvellResponseHeader"
Require-Match $allocator 'next:\s*u8' "allocator is not physically bounded to the low sequence byte"
Require-Match $allocator 'Self\s*\{\s*next:\s*1\s*\}' "allocator does not start at sequence 1"
Require-Match $allocator 'last\s*>\s*HOST_CMD_SEQUENCE_MASK.*base\s*=\s*1' "allocator does not wrap before BSS bits"
Require-Match $core 'single_bss_sequence_windows_start_advance_and_wrap_without_context_bits.*\[251,\s*252,\s*253,\s*254,\s*255\].*reserve_window\(5\),\s*1' "boundary-wrap positive test missing"
Require-Match $core 'connection_builders_reject_old_bss_one_sequence_context.*0x0100' "old BSS1 command negative missing"
Require-Match $supplicant 'connection_supplicant_builders_reject_old_bss_one_sequence_context.*0x0100' "old BSS1 supplicant negative missing"
Require-Match $core 'host_cmd_epoch_rejects_stale_high_and_accepts_only_low_then_done.*host_cmd_done_low_after_clear.*host_cmd_done_is_current' "HostCmd epoch unit predicate missing"
Require-Match $core 'host_cmd_empty_response_is_typed_without_scratch_readback_contract.*is_empty' "HostCmd empty-response unit predicate missing"
Require-Match $core 'connection_pci_enable_and_post_enable_mmio_liveness_are_fail_closed.*pci_memory_bus_master_enabled.*ConnectionMmioLiveness::Ready.*MmioUnavailable.*FirmwareNotReady' "PCI/MMIO liveness unit predicate missing"

$runtime = Slice-Between $driver "struct ConnectionRuntime" "struct FirmwareJob"
Require-Match $runtime 'sequence_allocator:\s*SingleBssHostCmdSequenceAllocator' "connection runtime bypasses low-byte allocator"
Require-NoMatch $runtime 'next_seq:\s*u16|0x100' "connection runtime retains wide/BSS1 sequence state"
$start = Slice-Between $driver "fn start_association_inner" "fn same_connection_target"
Require-Match $start 'reserve_window\(CONNECTION_HOST_CMD_PHASE_COUNT\)' "connection does not reserve one association-only command window"
$phase = Slice-Between $driver "fn connection_phase_seq" "fn connection_phase_command"
foreach ($offset in 0..2) {
    Require-Match $phase ("=>\s*" + $offset) "connection phase offset $offset missing"
}
Require-NoMatch $phase 'RegisterRings|MacControl|PCIE_DESC_DETAILS_CMD|MAC_CONTROL_CMD' "association sequence still owns firmware-epoch init commands"

$poll = Slice-Between $driver "pub fn poll_connection" "fn next_connection_phase"
$doneAt = $poll.IndexOf("host_cmd_done_is_current", [StringComparison]::Ordinal)
$captureAt = $poll.IndexOf("connection_response_diagnostic", [StringComparison]::Ordinal)
$quarantineAt = $poll.IndexOf("quarantine_connection_job(&job)", $captureAt, [StringComparison]::Ordinal)
Require (($doneAt -ge 0) -and ($captureAt -gt $doneAt) -and ($quarantineAt -gt $captureAt)) "response telemetry is not confined to CMD_DONE before quarantine"
Require-Match $poll 'cmd_done_low_baseline.*HOST_INTR_CMD_DONE.*StaleCommandDone' "completion can bypass the proven-low epoch"
Require-Match $poll 'status == u32::MAX.*MmioUnavailable.*host_cmd_done_is_current' "unavailable HOST_INT_STATUS can reach completion parsing"
Require-Match $poll 'clear_connection_response_mailbox.*_cleanup_flush_status\s*=\s*read_reg\(mmio,\s*PCIE_HOST_INT_STATUS\)' "response address cleanup lacks a status-register posted-write flush"
Require-NoMatch $driver 'RegisterReadbackMismatch|mailbox_register_readback_mismatch|HostCmdMailboxRegisters|read_connection_mailbox|observed\.matches' "runtime scratch readback contract still gates HostCmd"
Require-NoMatch $driver 'read_reg\(mmio,\s*(?:CMDRSP_ADDR_LO|CMDRSP_ADDR_HI|CMD_ADDR_LO|CMD_ADDR_HI|CMD_SIZE)\)' "runtime HostCmd reads a write-only/untrusted scratch register"

$arm = Slice-Between $driver "fn arm_connection_command" "pub fn poll_connection"
$pciAt = $arm.IndexOf("let vendor_device = job.pci_address.read_u32(0x00)", [StringComparison]::Ordinal)
$preEnableAt = $arm.IndexOf("pre_enable_status = read_reg", [StringComparison]::Ordinal)
$enableAt = $arm.IndexOf("ensure_pci_memory_bus_master", [StringComparison]::Ordinal)
$livenessAt = $arm.IndexOf("connection_mmio_liveness", [StringComparison]::Ordinal)
$livenessReadyAt = $arm.IndexOf("ConnectionMmioLiveness::Ready", $livenessAt, [StringComparison]::Ordinal)
$maskAt = $arm.IndexOf("PCIE_HOST_INT_MASK, 0", [StringComparison]::Ordinal)
$zeroAt = $arm.IndexOf("clear_connection_response_mailbox", [StringComparison]::Ordinal)
$pendingAt = $arm.IndexOf("let pending = read_reg", [StringComparison]::Ordinal)
$ackAt = $arm.IndexOf("!pending", [StringComparison]::Ordinal)
$lowAt = $arm.IndexOf("host_cmd_done_low_after_clear", [StringComparison]::Ordinal)
$staleAt = $arm.IndexOf("return Err(ConnectionTransportError::StaleCommandDone)", $lowAt, [StringComparison]::Ordinal)
$writeAt = $arm.IndexOf("CMDRSP_ADDR_LO", $staleAt, [StringComparison]::Ordinal)
$flushAt = $arm.IndexOf("let program_flush_status = read_reg(mmio, PCIE_HOST_INT_STATUS)", [StringComparison]::Ordinal)
$flushUnavailableAt = $arm.IndexOf("program_flush_status == u32::MAX", [StringComparison]::Ordinal)
$flushHighAt = $arm.IndexOf("program_flush_status & HOST_INTR_CMD_DONE", [StringComparison]::Ordinal)
$doorbellAt = $arm.IndexOf("CPU_INTR_DOOR_BELL", [StringComparison]::Ordinal)
Require (($pciAt -ge 0) -and ($preEnableAt -gt $pciAt) -and ($enableAt -gt $preEnableAt) -and ($livenessAt -gt $enableAt) -and ($livenessReadyAt -gt $livenessAt) -and ($maskAt -gt $livenessReadyAt) -and ($zeroAt -gt $maskAt) -and ($pendingAt -gt $zeroAt) -and ($ackAt -gt $pendingAt) -and ($lowAt -gt $ackAt) -and ($staleAt -gt $lowAt) -and ($writeAt -gt $staleAt) -and ($flushAt -gt $writeAt) -and ($flushUnavailableAt -gt $flushAt) -and ($flushHighAt -gt $flushUnavailableAt) -and ($doorbellAt -gt $flushHighAt)) "HostCmd arm order does not restore PCI ownership and prove post-enable MMIO/status epochs before doorbell"
Require-Match $arm 'CONNECTION_CMD_DONE_CLEAR_POLLS.*SHORT_POLL_DELAY_US.*StaleCommandDone' "CMD_DONE clear wait is not bounded/fail-closed"
Require-Match $arm 'pending == u32::MAX.*MmioUnavailable' "initial unavailable HOST_INT_STATUS is not a typed no-doorbell failure"
Require-Match $arm 'CONNECTION_MMIO_LIVENESS_POLLS.*FW_STATUS.*PCIE_HOST_INT_STATUS.*FirmwareNotReadyAfterEnable.*MmioUnavailable' "post-enable firmware/MMIO liveness probe is not bounded and typed"
$preDoorbell = $arm.Substring(0, $doorbellAt)
Require-Match $preDoorbell 'PciCommandEnableFailed.*FirmwareNotReadyAfterEnable.*MmioUnavailable' "PCI/MMIO failures do not remain before doorbell"
$afterDoorbell = $arm.Substring($doorbellAt)
Require-NoMatch $afterDoorbell 'host_cmd_done_low_after_clear|StaleCommandDone' "an immediate post-doorbell completion is incorrectly rejected"

$pciPlan = Slice-Between $core "pub enum PciCommandEnablePlan" "pub enum ConnectionMmioLiveness"
Require-Match $pciPlan 'command == u16::MAX.*Unavailable.*pci_memory_bus_master_enabled\(command\).*AlreadyEnabled.*Write\(command \| 0x0006\)' "PCI command planner is not 0406-idempotent/FFFF-fail-closed"
$pciPrimitive = Slice-Between $pci "fn pci_config_write_command_u16_checked" "fn pci_config_write_u32"
Require-Match $pciPrimitive 'PCI_LOCK\.lock.*vendor_device == u32::MAX.*command == u16::MAX.*command != expected_command.*outw\(CONFIG_DATA, value\).*readback' "command-only PCI write is not atomically rechecked and verified"
Require-NoMatch $pciPrimitive 'outl\(CONFIG_DATA,\s*(?:current|value)\)|mask\s*=|&\s*mask' "command-only PCI path performs forbidden Status dword RMW"
$pciEnsure = Slice-Between $driver "fn ensure_pci_memory_bus_master" "fn arm_connection_command"
Require-Match $pciEnsure 'AlreadyEnabled.*command_after:\s*command_before.*PciCommandEnablePlan::Write.*write_command_u16_checked.*pci_memory_bus_master_enabled\(readback\)' "0406 no-write or 0402->0406 verified behavior missing"
Require-NoMatch $pciEnsure 'enable_bus_master|write_u16\(' "checked PCI helper can fall back to unsafe dword-RMW"

$hwPoll = Slice-Between $driver "pub fn poll_hw_spec" "pub fn poll_scan_ext"
Require-Match $hwPoll 'phase = runtime\.snapshot\.stage.*prepare_hw_spec_dma\(&job, phase\).*parse_hw_spec_dma_response\(phase.*clear_connection_response_mailbox.*match parsed.*Err\(result\).*disable_bus_master' "firmware-epoch init does not execute bounded command/response stages or quarantine failures"
Require-Match $hwPoll 'CmdDoneTimeout.*finish_hw_spec_locked.*write_hw_spec_failure.*return true' "init timeout does not stop the current epoch"
Require-Match $hwPoll 'EmptyResponseOnCommandDone|parse_hw_spec_dma_response' "zero command response is not routed to typed quarantine"
Require-Match $hwPoll 'next_init_phase\(phase\).*next == HwSpecStage::Ready.*publish_data_ring_pointers.*HwSpecResult::Done.*HwSpecStage::Ready' "InitReady is observable before every init response succeeds"
$initPrepare = Slice-Between $driver "fn prepare_hw_spec_dma" "fn prepare_scan_dma"
Require-Match $initPrepare 'HwSpecStage::PcieDescDetails.*build_pcie_desc_details.*HwSpecStage::FuncInit.*build_func_init.*HwSpecStage::GetHwSpec.*build_get_hw_spec.*HwSpecStage::MacControl.*build_mac_control' "upstream init command order/builders are incomplete"
$initOrder = Slice-Between $driver "fn next_init_phase" "fn parse_hw_spec_dma_response"
Require-Match $initOrder 'PcieDescDetails\s*=>\s*HwSpecStage::FuncInit.*FuncInit\s*=>\s*HwSpecStage::GetHwSpec.*GetHwSpec\s*=>\s*HwSpecStage::MacControl.*MacControl\s*=>\s*HwSpecStage::Ready' "firmware-epoch init order differs from PCIE_DESC->FUNC_INIT->GET_HW_SPEC->MAC_CONTROL"
$initParser = Slice-Between $driver "fn parse_hw_spec_dma_response" "fn copy_block_into_dma"
Require-Match $initParser 'is_empty.*EmptyResponseOnCommandDone.*parse_pcie_desc_details_response.*parse_func_init_response.*parse_hw_spec_response.*parse_mac_control_response' "init response validation/zero-response boundary is incomplete"
$scanStart = Slice-Between $driver "pub fn start_scan_ext_24ghz" "pub fn poll_hw_spec"
Require-Match $scanStart 'hw_spec_snapshot\(\)\.is_ready\(\).*HwSpecNotReady' "scan can start before firmware epoch InitReady"
Require-Match $start 'hw_spec_snapshot\(\).*is_ready\(\).*HwSpecNotReady' "association can start before firmware epoch InitReady"
$associationPrepare = Slice-Between $driver "fn prepare_connection_dma" "fn parse_connection_dma_response"
Require-NoMatch $associationPrepare 'build_pcie_desc_details|build_mac_control|PCIE_DESC_DETAILS_CMD|MAC_CONTROL_CMD' "association resends firmware-epoch descriptor/MAC init commands"
$traceQueue = Slice-Between $driver "fn queue_fixed_hw_failure_trace" "struct DmaBlock"
Require-Match $traceQueue 'HwFailureTrace::new.*HwFailureSubsystem::MarvellWifiPcie.*HwFailureTraceStep.*usb::queue_hw_failure_trace' "fatal WiFi paths do not queue the fixed typed trace"
Require-NoMatch $traceQueue 'flush_pending|msc_|disk|append_|write_sector|read_sector' "WiFi failure callback performs forbidden media I/O"
foreach ($finishName in @("fn finish_locked", "fn finish_hw_spec_locked", "fn finish_scan_locked", "fn finish_connection_locked")) {
    $finishAt = $driver.IndexOf($finishName, [StringComparison]::Ordinal)
    Require ($finishAt -ge 0) "missing fatal finish hook: $finishName"
    $queueAt = $driver.IndexOf("queue_fixed_hw_failure_trace", $finishAt, [StringComparison]::Ordinal)
    Require (($queueAt -gt $finishAt) -and (($queueAt - $finishAt) -lt 2500)) "fatal finish path does not queue typed trace: $finishName"
}
$scanPoll = Slice-Between $driver "pub fn poll_scan_ext" "fn clear_connection_response_mailbox"
Require-Match $scanPoll 'parse_scan_dma_response.*clear_connection_response_mailbox.*ScanCmdResult::Done.*disable_bus_master.*ScanCmdResult::LiveResultParseFailed.*Err\(error\).*disable_bus_master' "scan success does not retain BME or scan failures no longer quarantine it"

$diagnostic = Slice-Between $driver "fn connection_response_diagnostic" "fn write_connection_response_failure"
Require-Match $diagnostic 'expected_command.*expected_sequence' "diagnostic lacks expected command/sequence"
Require-NoMatch $diagnostic 'connect_cmd_ptr|target|ssid|bssid|passphrase|pmk|security_ie|authority|receipt|attempt' "diagnostic reaches request, target, or authority data"
$parser = Slice-Between $driver "fn parse_connection_dma_response" "fn connection_phase_seq"
Require-Match $parser 'redacted_response_header' "DMA parser bypasses fixed-header projection"
Require-Match $parser 'is_empty.*EmptyResponseOnCommandDone' "zero HostCmd header is not classified as empty_response_on_cmd_done"
$header = Slice-Between $core "pub fn redacted_response_header" "pub const fn has_single_bss_host_cmd_context"
Require-Match $header 'interface_len.*interface_type.*command.*host_command_size.*sequence.*result' "fixed response header is incomplete"
Require-NoMatch $header 'payload|ssid|bssid|passphrase|pmk|authority|receipt|attempt' "fixed response projection contains forbidden metadata"

$quarantine = Slice-Between $driver "fn quarantine_connection_job" "fn clear_connection_secret_dma"
Require-Match $quarantine 'disable_bus_master.*clear_connection_response_mailbox.*PCIE_HOST_INT_STATUS.*clear_connection_secret_dma.*CONNECTION_REBOOT_REQUIRED\.store\(true' "response failure quarantine does not clear DMA ownership with a status flush and require reboot"
$start = Slice-Between $driver "fn start_association_inner" "fn same_connection_target"
Require-Match $start 'connection_reboot_required.*ConnectionResult::RebootRequired' "same-boot retry can re-enter association after quarantine"
$uiFailure = Slice-Between $ui "fn draw_failed" "fn connection_progress"
Require-Match $uiFailure 'failed_stage.*error_class.*interface_len.*interface_type.*command.*host_command_size.*sequence.*result.*expected_command.*expected_sequence' "Genesis response diagnostics are incomplete"
Require-Match $uiFailure 'pci_config_valid.*pci=.*cmd=.*pre_enable_status_valid.*live.*firmware_status_valid.*post_enable_status_valid.*pre_clear_status_valid.*post_clear_status_valid.*program_flush_status_valid.*first_poll_status_valid' "Genesis PCI/MMIO telemetry lacks explicit validity"
Require-Match $uiFailure 'Transport quarantined - reboot required.*Reboot req\.' "Genesis failure view does not make reboot-required quarantine clear"
Require-NoMatch $uiFailure 'ssid|bssid|passphrase|pmk|security_ie|authority|receipt|attempt' "Genesis failure view exposes secret or target metadata"
$retry = Slice-Between $ui "State::Failed(_) =>" "fn submit_vault_password"
Require-Match $retry 'connection_reboot_required.*self\.begin\(\)' "failure action can retry without checking transport quarantine"

if (-not $SkipBuild) {
    $oldCargoHome = $env:CARGO_HOME
    $oldTarget = $env:CARGO_TARGET_DIR
    try {
        $env:CARGO_HOME = Join-Path $env:USERPROFILE ".cargo"
        $env:CARGO_TARGET_DIR = Join-Path $env:TEMP "raios-marvell-connection-fixture-target"
        & cargo test --locked -p raios-core marvell_wifi
        if ($LASTEXITCODE -ne 0) { throw "focused Marvell core tests failed" }
    }
    finally {
        $env:CARGO_HOME = $oldCargoHome
        $env:CARGO_TARGET_DIR = $oldTarget
    }
}

Write-Output "MARVELL_CONNECTION_TELEMETRY_FIXTURE status=pass seq_field=low8 bss=0 payload_logged=false"
