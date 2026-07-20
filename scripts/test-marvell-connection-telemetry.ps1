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
$offAt = $poll.IndexOf("terminal_quiesce_and_cleanup_while_gated", $doneAt, [StringComparison]::Ordinal)
$captureAt = $poll.IndexOf("connection_response_diagnostic", [StringComparison]::Ordinal)
Require (($doneAt -ge 0) -and ($offAt -gt $doneAt) -and ($captureAt -gt $offAt)) "connection parses response before verified BME-off cleanup"
Require-Match $poll 'begin_host_command_epoch_while_gated.*prepare_connection_dma.*arm_connection_command' "connection mutates DMA before the verified-off epoch"
Require-NoMatch $driver 'pci::(?:enable|disable)_bus_master|clear_connection_response_mailbox' "driver retains an unchecked BME or partial-mailbox cleanup path"
Require-NoMatch $driver 'read_reg\(mmio,\s*(?:CMDRSP_ADDR_LO|CMDRSP_ADDR_HI|CMD_ADDR_LO|CMD_ADDR_HI|CMD_SIZE)\)' "runtime HostCmd reads write-only scratch registers"

function Test-VerifiedQuiesce([string]$Text) {
    $surface = Slice-Between $Text "fn verified_quiesce_while_gated" "fn enable_memory_space_while_verified_off"
    $vendorAt = $surface.IndexOf('read_u32(0x00)', [StringComparison]::Ordinal)
    $commandAt = $surface.IndexOf('read_u16(0x04)', [StringComparison]::Ordinal)
    $requestAt = $surface.IndexOf('(command & !0x0004) | (1 << 10)', [StringComparison]::Ordinal)
    $checkedAt = $surface.IndexOf('write_command_u16_checked', [StringComparison]::Ordinal)
    $readbackAt = $surface.IndexOf('readback & 0x0004', [StringComparison]::Ordinal)
    $tokenAt = $surface.IndexOf('Ok(VerifiedOff(()))', [StringComparison]::Ordinal)
    ($vendorAt -ge 0) -and ($commandAt -gt $vendorAt) -and ($requestAt -gt $commandAt) -and ($checkedAt -gt $requestAt) -and ($readbackAt -gt $checkedAt) -and ($tokenAt -gt $readbackAt) -and
        [regex]::IsMatch($surface, 'DeviceUnavailable.*poison_dma_epoch_while_gated.*CommandChanged.*poison_dma_epoch_while_gated', [Text.RegularExpressions.RegexOptions]::Singleline)
}

Require (Test-VerifiedQuiesce $driver) "verified quiesce is not checked/readback-gated"
$uncheckedQuiesce = $driver.Replace('address.write_command_u16_checked(vendor_device, command, requested)', 'pci::disable_bus_master(address)')
Require (-not (Test-VerifiedQuiesce $uncheckedQuiesce)) "unchecked BME-off failure injection was accepted"

function Test-VerifiedCleanup([string]$Text) {
    $surface = Slice-Between $Text "fn cleanup_host_command_mailbox_after_verified_off" "fn begin_host_command_epoch_while_gated"
    $needles = @('CMD_SIZE, 0', 'CMD_ADDR_LO, 0', 'CMD_ADDR_HI, 0', 'CMDRSP_ADDR_LO, 0', 'CMDRSP_ADDR_HI, 0', 'read_reg(mmio, PCIE_HOST_INT_STATUS)', 'flush == u32::MAX', 'poison_dma_epoch_while_gated')
    $cursor = -1
    foreach ($needle in $needles) {
        $next = $surface.IndexOf($needle, $cursor + 1, [StringComparison]::Ordinal)
        if ($next -le $cursor) { return $false }
        $cursor = $next
    }
    [regex]::IsMatch($surface, '_verified_off:\s*&VerifiedOff')
}

Require (Test-VerifiedCleanup $driver) "full mailbox cleanup is not token-gated, ordered, flushed, and poisoned on all-ones"
$badCleanup = $driver.Replace('write_reg(mmio, CMD_SIZE, 0);', 'write_reg(mmio, CMDRSP_ADDR_LO, 0);')
Require (-not (Test-VerifiedCleanup $badCleanup)) "partial/reordered cleanup failure injection was accepted"

function Test-TransactionalPublication([string]$Text) {
    $surface = Slice-Between $Text "fn publish_host_command_while_gated" "fn terminal_quiesce_and_cleanup_while_gated"
    $needles = @('PublicationModel::new', 'PublicationStep::ResponseLow', 'PublicationStep::ResponseHigh', 'PublicationStep::CommandLow', 'PublicationStep::CommandHigh', 'PublicationStep::CommandSize', 'PublicationStep::RingsPublished', 'program_flush_status', 'ensure_pci_memory_bus_master', 'enable_bme()', 'CPU_INTR_DOOR_BELL', 'PublicationStep::Doorbell')
    $cursor = -1
    foreach ($needle in $needles) {
        $next = $surface.IndexOf($needle, $cursor + 1, [StringComparison]::Ordinal)
        if ($next -le $cursor) { return $false }
        $cursor = $next
    }
    $firstEnable = $surface.IndexOf('enable_bme()', [StringComparison]::Ordinal)
    $firstDoorbell = $surface.IndexOf('CPU_INTR_DOOR_BELL', [StringComparison]::Ordinal)
    ($firstDoorbell -gt $firstEnable) -and [regex]::IsMatch($surface, 'program_flush_status == u32::MAX.*verified_quiesce_while_gated', [Text.RegularExpressions.RegexOptions]::Singleline)
}

Require (Test-TransactionalPublication $driver) "HostCmd publication is not one modeled off-to-live transaction"
$earlyDoorbell = $driver.Replace('let vendor_device = address.read_u32(0x00);', 'write_reg(mmio, PCIE_CPU_INT_EVENT, CPU_INTR_DOOR_BELL); let vendor_device = address.read_u32(0x00);')
Require (-not (Test-TransactionalPublication $earlyDoorbell)) "doorbell-before-checked-enable failure injection was accepted"

$dmaPlan = Slice-Between $driver "fn validate_runtime_dma_region" "fn cleanup_host_command_mailbox_after_verified_off"
Require-Match $dmaPlan 'virt_to_phys.*validate_contiguous_translation.*DmaSpan::new.*MARVELL_DMA_REGION_COUNT.*EventRingDmaBlock.*RxRingDmaBlock.*TxRingDmaBlock.*authoritative_foreign_dma_regions_while_gated.*ForeignRegionsUnavailable.*validate_non_overlapping_regions' "runtime DMA spans are not page-wise, aligned, complete, and overlap-checked"
Require-Match $dmaPlan 'Absence is deliberately not interpreted as an empty.*None' "missing foreign-span authority is not fail-closed"
Require-NoMatch $dmaPlan 'validate_non_overlapping_regions\(&regions,\s*&\[\]\)|Some\(&\[\]\)' "empty foreign-span set can masquerade as release evidence"
$foreignAdapter = Slice-Between $driver "fn authoritative_foreign_dma_regions_while_gated" "fn cleanup_host_command_mailbox_after_verified_off"
$fakeForeign = $foreignAdapter -replace '\bNone\b', 'Some(&[])'
Require-Match $fakeForeign 'Some\(&\[\]\)' "foreign-span failure injection did not apply"
Require-NoMatch $driver 'Some\(&\[\]\)' "foreign-span adapter currently fabricates an empty authoritative set"

$hwPoll = Slice-Between $driver "pub fn poll_hw_spec" "pub fn poll_scan_ext"
$scanPoll = Slice-Between $driver "pub fn poll_scan_ext" "fn ensure_pci_memory_bus_master"
$connectionArm = Slice-Between $driver "fn arm_connection_command" "pub fn poll_connection"
Require-Match $hwPoll 'begin_host_command_epoch_while_gated.*prepare_hw_spec_dma.*publish_host_command_while_gated.*terminal_quiesce_and_cleanup_while_gated.*parse_hw_spec_dma_response' "HWSPEC bypasses the common transaction or parses before quiesce"
Require-Match $scanPoll 'begin_host_command_epoch_while_gated.*prepare_scan_dma.*publish_host_command_while_gated.*terminal_quiesce_and_cleanup_while_gated.*parse_scan_dma_response' "scan bypasses the common transaction or parses before quiesce"
Require-Match $connectionArm 'publish_host_command_while_gated.*connection_mmio_liveness' "connection bypasses the common HostCmd publisher"

$firmwarePoll = Slice-Between $driver "pub fn poll()" "fn sync_job_phase_clock"
$firmwareFinish = Slice-Between $driver "fn finish_locked" "fn mark_stage"
Require-Match $firmwarePoll 'FwAction::WriteBlock.*terminal_quiesce_and_cleanup_while_gated.*copy_block_into_dma.*FwAction::Retry.*terminal_quiesce_and_cleanup_while_gated.*activate_validated_data_dma_while_gated' "firmware block/retry mutates or reactivates DMA without verified quiesce"
Require-Match $firmwareFinish 'FirmwareStage::Ready \| FirmwareStage::Failed.*terminal_quiesce_and_cleanup_while_gated.*DrvReadyQuarantined.*FirmwareStage::Failed' "firmware terminal transitions can publish Ready without verified quiesce"
$terminalWithoutOff = $firmwareFinish.Replace('terminal_quiesce_and_cleanup_while_gated', 'terminal_quiesce_removed')
Require-NoMatch $terminalWithoutOff 'terminal_quiesce_and_cleanup_while_gated' "terminal-quiesce failure injection was accepted"

foreach ($ringSurface in @(
    (Slice-Between $driver "pub fn receive_ethernet" "pub fn transmit_ethernet"),
    (Slice-Between $driver "pub fn transmit_ethernet" "pub fn start_bring_up_firmware"),
    (Slice-Between $driver "pub fn poll_event_ring" "fn handle_connection_event"),
    (Slice-Between $driver "fn arm_event_ring_while_gated" "fn arm_rx_ring_while_gated"),
    (Slice-Between $driver "fn arm_rx_ring_while_gated" "fn arm_tx_ring_while_gated")
)) {
    Require-Match $ringSurface 'drop\(runtime\).*quarantine_invalid_pointer_after_ring_unlock_while_gated' "invalid pointer can quiesce while its ring lock is held"
}

$pciPrimitive = Slice-Between $pci "fn pci_config_write_command_u16_checked" "fn pci_config_write_u32"
Require-Match $pciPrimitive 'PCI_LOCK\.lock.*vendor_device == u32::MAX.*command == u16::MAX.*command != expected_command.*outw\(CONFIG_DATA, value\).*readback' "command-only PCI write is not atomically rechecked and verified"
Require-NoMatch $pciPrimitive 'outl\(CONFIG_DATA,\s*(?:current|value)\)|mask\s*=|&\s*mask' "command-only PCI path performs forbidden Status dword RMW"

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
Require-Match $quarantine 'terminal_quiesce_and_cleanup_while_gated.*clear_connection_secret_dma.*CONNECTION_REBOOT_REQUIRED\.swap\(true' "response failure quarantine lacks verified-off cleanup, secret clearing, or reboot latch"
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
        & cargo test --locked -p raios-core marvell_dma_safety
        if ($LASTEXITCODE -ne 0) { throw "focused Marvell DMA safety tests failed" }
    }
    finally {
        $env:CARGO_HOME = $oldCargoHome
        $env:CARGO_TARGET_DIR = $oldTarget
    }
}

Write-Output "MARVELL_CONNECTION_TELEMETRY_FIXTURE status=pass seq_field=low8 bss=0 payload_logged=false"
