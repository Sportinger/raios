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
$memoryPath = Join-Path $RepoRoot "seed-kernel\src\memory.rs"
$heapPath = Join-Path $RepoRoot "seed-kernel\src\heap.rs"
$usbPath = Join-Path $RepoRoot "seed-kernel\src\usb.rs"
$ahciPath = Join-Path $RepoRoot "seed-kernel\src\ahci.rs"
$mainPath = Join-Path $RepoRoot "seed-kernel\src\main.rs"
$tracePath = Join-Path $RepoRoot "crates\raios-core\src\hw_failure_trace.rs"
$extractorPath = Join-Path $RepoRoot "scripts\extract-hw-failure-trace.ps1"
$core = Get-Content -LiteralPath $corePath -Raw
$supplicant = Get-Content -LiteralPath $supplicantPath -Raw
$driver = Get-Content -LiteralPath $driverPath -Raw
$pci = Get-Content -LiteralPath $pciPath -Raw
$ui = Get-Content -LiteralPath $uiPath -Raw
$memory = Get-Content -LiteralPath $memoryPath -Raw
$heap = Get-Content -LiteralPath $heapPath -Raw
$usb = Get-Content -LiteralPath $usbPath -Raw
$ahci = Get-Content -LiteralPath $ahciPath -Raw
$main = Get-Content -LiteralPath $mainPath -Raw
$trace = Get-Content -LiteralPath $tracePath -Raw
$extractor = Get-Content -LiteralPath $extractorPath -Raw

$physicalSpan = Slice-Between $memory "pub(crate) struct PhysicalSpan" "static KERNEL_PHYSICAL_BASE"
Require-Match $physicalSpan 'start:\s*u64.*end_exclusive:\s*u64.*from_start_len.*len == 0.*checked_add\(len\)' "physical half-interval lacks nonzero/overflow checks"
$kernelSpan = Slice-Between $memory "pub(crate) fn kernel_image_physical_span" "/// Resolves a stable virtual interval"
Require-Match $kernelSpan 'kernel_address_map\(\)\?.*kernel_image_virtual_end\(\).*checked_sub\(virtual_base\)\?.*PhysicalSpan::from_start_len' "kernel image span is not derived from real Limine bases and __bss_end"
Require-NoMatch $kernelSpan 'identity_phys|virt_to_phys' "kernel image span can use a non-authoritative address fallback"
$translation = Slice-Between $memory "fn checked_physical_span_from_virtual_range" "#[cfg(test)]"
Require-Match $translation 'len == 0.*virtual_start\.checked_add\(len\)\?.*translate\(virtual_start\)\?.*next_page.*last_virtual.*translate\(last_virtual\)' "physical span translation lacks zero/overflow/page-contiguity boundaries"
Require-Match $memory 'physical_half_interval_rejects_zero_and_overflow.*contiguous_virtual_translation_is_accepted.*untranslated_or_discontiguous_virtual_translation_is_rejected' "physical span positive/negative tests are incomplete"

$heapSpan = Slice-Between $heap "pub(crate) fn physical_span" "fn init_static_fallback"
Require-Match $heap 'publish_heap_span\(heap_base,\s*heap_len,\s*Some\(region\.base\)\).*matches!\(report\.source,\s*HeapSource::StaticFallback\).*publish_heap_span\(report\.base,\s*report\.size,\s*None\)' "heap does not publish both memmap and static-fallback arena sources"
Require-Match $heapSpan 'HEAP_SPAN_READY\.load\(Ordering::Acquire\).*physical_span_for_virtual_range.*HEAP_EXPECTED_PHYSICAL_READY.*HEAP_EXPECTED_PHYSICAL_BASE' "heap span is not an atomic, translated, source-checked snapshot"
Require-NoMatch $heapSpan 'ALLOCATOR\.lock|HEAP_REPORT|Mutex|Vec' "heap physical snapshot takes a lock or reads mutable diagnostic state"

$xhciSpans = Slice-Between $usb "pub(crate) fn xhci_dma_physical_spans" "/// Returns the fixed USB-MSC buffers"
$xhciSpanCalls = [regex]::Matches($xhciSpans, 'static_dma_span\(ptr::addr_of!\(').Count
Require ($xhciSpanCalls -eq 18) "xHCI physical inventory does not contain exactly 18 static DMA objects"
foreach ($name in @("COMMAND_RING", "EVENT_RING", "EP0_RINGS", "INTR_RINGS", "BULK_IN_RINGS", "BULK_OUT_RINGS", "ERST", "DCBAA", "SCRATCHPAD_ARRAY", "SCRATCHPAD_PAGES", "INPUT_CONTEXT", "DEVICE_CONTEXTS", "CONTROL_BUFFER", "KEYBOARD_REPORT", "MOUSE_REPORT", "MSC_CBW", "MSC_CSW", "MSC_BUFFER")) {
    Require-Match $xhciSpans ("addr_of!\(" + $name + "\)") "xHCI physical inventory missing $name"
}
Require-Match $xhciSpans 'kernel_image_physical_span\(\)\?.*!kernel\.contains\(span\)' "xHCI spans are not proven inside the kernel image"
Require-NoMatch $xhciSpans 'STATE\.lock|Mutex|Vec|read_reg|write_reg|ring_doorbell' "xHCI physical inventory takes locks, allocates, or performs device I/O"

$usbReclogSpans = Slice-Between $usb "pub(crate) fn usb_msc_reclog_staging_physical_spans" "fn static_dma_span"
Require-Match $usbReclogSpans 'addr_of!\(MSC_CBW\).*addr_of!\(MSC_BUFFER\).*addr_of!\(MSC_CSW\).*kernel\.contains\(span\)' "USB-MSC RECLOG staging inventory is incomplete or uncontained"
Require-NoMatch $usbReclogSpans 'STATE\.lock|Mutex|Vec|read_reg|write_reg|bulk_transfer' "USB-MSC RECLOG snapshot takes locks, allocates, or performs device I/O"
Require-Match $usb 'full RECLOG copy is deliberately absent.*temporary `Vec`.*heap::physical_span' "USB RECLOG API does not document heap coverage of the temporary Vec"

$ahciReclogSpans = Slice-Between $ahci "pub(crate) fn reclog_staging_physical_spans" "/// An AHCI port"
$ahciSpanCalls = [regex]::Matches($ahciReclogSpans, 'physical_span_for_virtual_range\(').Count
Require ($ahciSpanCalls -eq 6) "AHCI RECLOG physical inventory does not contain exactly six fallible span resolutions"
foreach ($name in @("COMMAND_LIST", "RECEIVED_FIS", "COMMAND_TABLE", "SCRATCH_BUFFER", "IDENTIFY_BUFFER", "SECTOR0_BUFFER")) {
    Require-Match $ahciReclogSpans ("addr_of!\(" + $name + "\)") "AHCI RECLOG physical inventory missing $name"
}
Require-Match $ahciReclogSpans 'kernel_image_physical_span\(\)\?.*!kernel\.contains\(span\)' "AHCI RECLOG spans are not proven inside the kernel image"
Require-NoMatch $ahciReclogSpans 'EXPLICIT_ATA_COMMAND_LOCK|\.lock\(|read32|write32|issue_|Vec' "AHCI RECLOG snapshot takes locks, allocates, or performs device I/O"
Require-Match $ahci 'complete RECLOG `Vec` is intentionally represented by the enclosing heap' "AHCI RECLOG API does not document heap coverage of the temporary Vec"

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

function Test-QuiesceBeforeRuntimeValidation([string]$Text) {
    $surfaces = @(
        (Slice-Between $Text "fn begin_host_command_epoch_while_gated" "fn publish_host_command_while_gated"),
        (Slice-Between $Text "fn activate_validated_data_dma_while_gated" "fn quarantine_invalid_pointer_after_ring_unlock_while_gated"),
        (Slice-Between $Text "pub fn start_bring_up_firmware" "pub fn start_scan_ext_24ghz")
    )
    foreach ($surface in $surfaces) {
        $quiesceAt = $surface.IndexOf('verified_quiesce_while_gated(address)', [StringComparison]::Ordinal)
        $validateAt = $surface.IndexOf('validate_runtime_dma_plan_while_gated()', [StringComparison]::Ordinal)
        if (($quiesceAt -lt 0) -or ($validateAt -le $quiesceAt)) { return $false }
    }
    $true
}

Require (Test-QuiesceBeforeRuntimeValidation $driver) "runtime DMA validation can fail while BME remains active"
$validationBeforeQuiesce = $driver.Replace('verified_quiesce_while_gated(address).map_err(HostCommandPublishError::Quiesce)?', 'validate_runtime_dma_plan_while_gated(); verified_quiesce_while_gated(address).map_err(HostCommandPublishError::Quiesce)?')
Require (-not (Test-QuiesceBeforeRuntimeValidation $validationBeforeQuiesce)) "validation-before-quiesce failure injection was accepted"

function Test-TransactionalPublication([string]$Text) {
    $surface = Slice-Between $Text "fn publish_host_command_while_gated" "fn terminal_quiesce_and_cleanup_while_gated"
    $needles = @('PublicationModel::new', 'PublicationStep::ResponseLow', 'PublicationStep::ResponseHigh', 'PublicationStep::CommandLow', 'PublicationStep::CommandHigh', 'PublicationStep::CommandSize', 'PublicationStep::RingsPublished', 'program_flush_status', 'enable_bme()', 'ensure_pci_memory_bus_master', 'CPU_INTR_DOOR_BELL', 'PublicationStep::Doorbell')
    $cursor = -1
    foreach ($needle in $needles) {
        $next = $surface.IndexOf($needle, $cursor + 1, [StringComparison]::Ordinal)
        if ($next -le $cursor) { return $false }
        $cursor = $next
    }
    $firstModelEnable = $surface.IndexOf('model.enable_bme()', [StringComparison]::Ordinal)
    $firstHardwareEnable = $surface.IndexOf('ensure_pci_memory_bus_master', [StringComparison]::Ordinal)
    $firstDoorbell = $surface.IndexOf('CPU_INTR_DOOR_BELL', [StringComparison]::Ordinal)
    ($firstHardwareEnable -gt $firstModelEnable) -and ($firstDoorbell -gt $firstHardwareEnable) -and [regex]::IsMatch($surface, 'program_flush_status == u32::MAX.*verified_quiesce_while_gated', [Text.RegularExpressions.RegexOptions]::Singleline)
}

Require (Test-TransactionalPublication $driver) "HostCmd publication is not one modeled off-to-live transaction"
$hardwareBeforeModel = $driver.Replace('if model.enable_bme().is_err() {', 'let _hardware_before_model = ensure_pci_memory_bus_master(address, 0, 0); if model.enable_bme().is_err() {')
Require (-not (Test-TransactionalPublication $hardwareBeforeModel)) "hardware-enable-before-model failure injection was accepted"
$earlyDoorbell = $driver.Replace('let vendor_device = address.read_u32(0x00);', 'write_reg(mmio, PCIE_CPU_INT_EVENT, CPU_INTR_DOOR_BELL); let vendor_device = address.read_u32(0x00);')
Require (-not (Test-TransactionalPublication $earlyDoorbell)) "doorbell-before-checked-enable failure injection was accepted"

$dmaPlan = Slice-Between $driver "fn validate_runtime_dma_region" "fn cleanup_host_command_mailbox_after_verified_off"
Require-Match $dmaPlan 'virt_to_phys.*validate_contiguous_translation.*physical_span_for_virtual_range.*DmaSpan::new.*MARVELL_DMA_REGION_COUNT.*EventRingDmaBlock.*RxRingDmaBlock.*TxRingDmaBlock.*authoritative_foreign_dma_regions_while_gated' "runtime DMA spans are not authoritatively translated, aligned, and complete"
Require-Match $dmaPlan 'validate_non_overlapping_regions.*ForeignRegionsUnavailable' "runtime DMA foreign spans are not fail-closed and overlap-checked"
Require-NoMatch $dmaPlan 'validate_non_overlapping_regions\(&regions,\s*&\[\]\)|Some\(&\[\]\)' "empty foreign-span set can masquerade as release evidence"

function Test-AuthoritativeForeignRegions([string]$Text) {
    $kernelRemainder = Slice-Between $Text "fn validate_kernel_remainder_after_marvell_owned" "fn authoritative_foreign_dma_regions_while_gated"
    $adapter = Slice-Between $Text "fn authoritative_foreign_dma_regions_while_gated" "fn cleanup_host_command_mailbox_after_verified_off"
    [regex]::IsMatch($kernelRemainder, 'sort_unstable_by_key.*kernel\.start\(\).*KernelOwnershipMismatch.*ForeignRegionKind::Kernel.*kernel\.end_exclusive', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($adapter, 'kernel_image_physical_span.*validate_kernel_remainder_after_marvell_owned.*heap::physical_span.*ForeignRegionKind::Heap.*usb::xhci_dma_physical_spans.*ForeignRegionKind::Xhci.*usb::usb_msc_reclog_staging_physical_spans.*ForeignRegionKind::Reclog.*ahci::reclog_staging_physical_spans.*ForeignRegionKind::Reclog', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        ([regex]::Matches($adapter, 'ForeignRegionsUnavailable').Count -ge 5)
}

Require (Test-AuthoritativeForeignRegions $driver) "kernel remainder, heap, xHCI/USB, and AHCI foreign spans are not all fail-closed"
$missingKernelRemainder = $driver.Replace('validate_kernel_remainder_after_marvell_owned(regions, kernel)?;', '/* kernel remainder omitted */')
Require (-not (Test-AuthoritativeForeignRegions $missingKernelRemainder)) "missing kernel-remainder failure injection was accepted"
$missingXhci = $driver.Replace('usb::xhci_dma_physical_spans()', 'usb::missing_xhci_dma_physical_spans()')
Require (-not (Test-AuthoritativeForeignRegions $missingXhci)) "missing xHCI foreign-span failure injection was accepted"
Require-NoMatch $driver 'Some\(&\[\]\)' "foreign-span adapter fabricates an empty authoritative set"

function Test-PersistedCheckpointBeforeBme([string]$Text) {
    $firmware = Slice-Between $Text "pub fn poll()" "fn sync_job_phase_clock"
    $publish = Slice-Between $Text "fn publish_host_command_while_gated" "fn terminal_quiesce_and_cleanup_while_gated"
    $activate = Slice-Between $Text "fn activate_validated_data_dma_while_gated" "fn quarantine_invalid_pointer_after_ring_unlock_while_gated"

    $firmwareOff = $firmware.IndexOf('verified_quiesce_while_gated(job.pci_address)', [StringComparison]::Ordinal)
    $firmwareValidate = $firmware.IndexOf('validate_runtime_dma_plan_while_gated()', [StringComparison]::Ordinal)
    $firmwareQueue = $firmware.IndexOf('pre_bme_checkpoint_while_gated', [StringComparison]::Ordinal)
    $firmwareReturn = $firmware.IndexOf('PreBmeCheckpoint::Pending', [StringComparison]::Ordinal)

    $publishAck = $publish.IndexOf('usb::publication_checkpoint_status()', [StringComparison]::Ordinal)
    $publishModel = $publish.IndexOf('model.enable_bme()', [StringComparison]::Ordinal)
    $publishHardware = $publish.IndexOf('ensure_pci_memory_bus_master', [StringComparison]::Ordinal)

    $activateOff = $activate.IndexOf('verified_quiesce_while_gated(address)', [StringComparison]::Ordinal)
    $activateValidate = $activate.IndexOf('validate_runtime_dma_plan_while_gated()', [StringComparison]::Ordinal)
    $activateAck = $activate.IndexOf('usb::publication_checkpoint_status()', [StringComparison]::Ordinal)
    $activateHardware = $activate.IndexOf('ensure_pci_memory_bus_master', [StringComparison]::Ordinal)

    ($firmwareOff -ge 0) -and ($firmwareValidate -gt $firmwareOff) -and ($firmwareQueue -gt $firmwareValidate) -and ($firmwareReturn -gt $firmwareQueue) -and
        [regex]::IsMatch($firmware, 'PreBmeCheckpoint::Pending.*runtime\.job = Some\(job\);.*return true', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        ($publishAck -ge 0) -and ($publishModel -gt $publishAck) -and ($publishHardware -gt $publishModel) -and
        ($activateOff -ge 0) -and ($activateValidate -gt $activateOff) -and ($activateAck -gt $activateValidate) -and ($activateHardware -gt $activateAck)
}

Require (Test-PersistedCheckpointBeforeBme $driver) "BME can be enabled before a persisted pre-BME checkpoint"
$hardwareBeforePersisted = $driver.Replace('if usb::publication_checkpoint_status() != HwFailureTraceFlushStatus::Persisted {', 'let _hardware_before_persisted = ensure_pci_memory_bus_master(address, 0, 0); if usb::publication_checkpoint_status() != HwFailureTraceFlushStatus::Persisted {')
Require (-not (Test-PersistedCheckpointBeforeBme $hardwareBeforePersisted)) "BME-before-persisted-checkpoint failure injection was accepted"

$traceLatch = Slice-Between $trace "pub enum HwFailureTraceFlushStatus" "pub fn validate_hw_failure_append_target"
Require-Match $traceLatch 'Empty.*Pending.*InFlight.*Persisted.*Failed.*take_for_flush.*InFlight.*finish_flush.*Persisted.*Failed' "checkpoint latch has no explicit durable acknowledgement state"
Require-Match $usb 'PUBLICATION_CHECKPOINT_LATCH.*queue_publication_checkpoint.*publication_checkpoint_status' "pre-BME checkpoint does not use a distinct bounded latch"
$usbFlush = Slice-Between $usb "pub fn flush_pending_hw_failure_trace" "fn report_hw_failure_trace_result"
Require-Match $usbFlush 'PUBLICATION_CHECKPOINT_LATCH.*HW_FAILURE_TRACE_LATCH.*take_for_flush.*STATE\.lock.*append_hw_failure_trace.*finish_flush' "main-loop USB flush does not acknowledge the durable write after unlocked Marvell return"
Require-Match $main 'marvell_wifi_pcie::poll\(\).*marvell_wifi_pcie::poll_hw_spec\(\).*marvell_wifi_pcie::poll_scan_ext\(\).*marvell_wifi_pcie::poll_connection\(\).*usb::flush_pending_hw_failure_trace\(\)' "checkpoint flush is not after all Marvell polls in the main loop"

$write10 = Slice-Between $usb "unsafe fn msc_write_sector_from_buffer" "unsafe fn msc_scsi_command"
Require-Match $usb 'const SCSI_WRITE10_FUA:\s*u8 = 1 << 3' "USB RECLOG write has no WRITE(10) FUA contract"
Require-Match $write10 '0x2A,\s*SCSI_WRITE10_FUA' "USB RECLOG writes do not set WRITE(10) FUA"
Require-Match $usb 'msc_write_sector_from_buffer\(point\.lba\)\?.*msc_read_sector_copy\(point\.lba\)\?.*verify_hw_failure_trace_readback' "FUA checkpoint is not read back and reparsed"

Require-Match $trace 'K2CheckpointPersistFailed = 110.*K2PublicationRejected = 111.*K2PciCommandRejected = 112.*MarvellPciCommand = 7.*MarvellPublicationStep = 8.*MarvellConnectionTimeoutFingerprint = 9' "dedicated terminal K2/connection-timeout status and register codes are missing"
Require-Match $driver 'HwFailureStatus::K2CheckpointPersistFailed.*HwFailureStatus::K2PublicationRejected.*HwFailureStatus::K2PciCommandRejected' "K2 terminal paths still collapse to generic TransportFault"

function Test-DataRingFailureDiagnostics([string]$Text) {
    $diagnostic = Slice-Between $Text "// Stable MarvellPublicationStep payload" "#[repr(align(64))]"
    $eventArm = Slice-Between $Text "fn arm_event_ring_while_gated" "fn arm_rx_ring_while_gated"
    $rxArm = Slice-Between $Text "fn arm_rx_ring_while_gated" "fn arm_tx_ring_while_gated"
    $publish = Slice-Between $Text "fn publish_data_ring_pointers_while_gated" "fn finish_hw_spec_locked"
    $hwStart = Slice-Between $Text "fn arm_hw_spec_after_firmware_ready_while_gated" "fn arm_event_ring_while_gated"
    $hwPoll = Slice-Between $Text "pub fn poll_hw_spec" "pub fn poll_scan_ext"
    $hwFinish = Slice-Between $Text "fn finish_hw_spec_locked" "fn finish_scan_without_job"
    $association = Slice-Between $Text "fn start_association_inner" "fn same_connection_target"
    $queueHelper = Slice-Between $Text "fn queue_data_ring_failure_trace" "#[repr(align(64))]"

    $codesAreFixed = [regex]::IsMatch($diagnostic, '0xD1KK_DDDD.*decoder class or a bounded descriptor index.*DATA_RING_DIAG_EVENT_WR_DECODE:\s*u32\s*=\s*0xD101_0000.*DATA_RING_DIAG_EVENT_DMA_TRANSLATION:\s*u32\s*=\s*0xD102_0000.*DATA_RING_DIAG_RX_WR_TX_RD_DECODE:\s*u32\s*=\s*0xD103_0000.*DATA_RING_DIAG_RX_DMA_TRANSLATION:\s*u32\s*=\s*0xD104_0000.*DATA_RING_DIAG_HOST_EVENT_RD_PUBLICATION:\s*u32\s*=\s*0xD105_0000.*DATA_RING_DIAG_SHARED_RX_TX_PUBLICATION:\s*u32\s*=\s*0xD106_0000.*DATA_RING_DIAG_DECODER_ALL_ONES:\s*u32\s*=\s*1.*DATA_RING_DIAG_DECODER_RESERVED_BITS:\s*u32\s*=\s*2.*DATA_RING_DIAG_DECODER_INDEX_OUT_OF_RANGE:\s*u32\s*=\s*3', [Text.RegularExpressions.RegexOptions]::Singleline)
    $decoderClassesAreComplete = [regex]::IsMatch($diagnostic, 'DevicePointerError::AllOnes\s*=>\s*DATA_RING_DIAG_DECODER_ALL_ONES.*DevicePointerError::ReservedBits\s*=>\s*DATA_RING_DIAG_DECODER_RESERVED_BITS.*DevicePointerError::IndexOutOfRange\s*=>\s*DATA_RING_DIAG_DECODER_INDEX_OUT_OF_RANGE', [Text.RegularExpressions.RegexOptions]::Singleline)
    $causeMapIsComplete = [regex]::IsMatch($diagnostic, 'Self::EventWriteDecode\(error\).*DATA_RING_DIAG_EVENT_WR_DECODE\s*\|\s*data_ring_decoder_class\(error\).*Self::EventDmaTranslation\s*\{\s*index\s*\}.*DATA_RING_DIAG_EVENT_DMA_TRANSLATION\s*\|\s*index as u32.*Self::RxWriteTxReadDecode\(error\).*DATA_RING_DIAG_RX_WR_TX_RD_DECODE\s*\|\s*data_ring_decoder_class\(error\).*Self::RxDmaTranslation\s*\{\s*index\s*\}.*DATA_RING_DIAG_RX_DMA_TRANSLATION\s*\|\s*index as u32.*Self::HostEventReadPublication\(error\).*DATA_RING_DIAG_HOST_EVENT_RD_PUBLICATION\s*\|\s*data_ring_decoder_class\(error\).*Self::SharedRxTxPublication\(error\).*DATA_RING_DIAG_SHARED_RX_TX_PUBLICATION\s*\|\s*data_ring_decoder_class\(error\)', [Text.RegularExpressions.RegexOptions]::Singleline)
    $eventCausePropagates = [regex]::IsMatch($eventArm, 'if let Err\(error\) = decode_event_pointer\(wrptr\).*EventWriteDecode\(error\).*arm_failure = Some\(failure\).*return Err\(failure\).*event_data_phys\(index\).*EventDmaTranslation \{ index \}.*arm_failure = Some\(failure\).*return Err\(failure\)', [Text.RegularExpressions.RegexOptions]::Singleline)
    $rxConstructionDefersFirmwarePointer = (-not [regex]::IsMatch($rxArm, 'PCIE_RX_WR_PTR|decode_rx_tx_pointer_register')) -and [regex]::IsMatch($rxArm, 'let wrptr = 0;.*rx_data_phys\(index\).*RxDmaTranslation \{ index \}.*arm_failure = Some\(failure\).*return Err\(failure\).*runtime\.rdptr = RX_ROLLOVER_IND.*wrptr,', [Text.RegularExpressions.RegexOptions]::Singleline)
    $publicationCausePropagates = [regex]::IsMatch($publish, 'if let Err\(error\) = decode_event_pointer\(event_rdptr\).*quarantine_invalid_pointer_after_ring_unlock_while_gated.*HostEventReadPublication\(error\).*Err\(error\).*quarantine_invalid_pointer_after_ring_unlock_while_gated.*SharedRxTxPublication\(error\)', [Text.RegularExpressions.RegexOptions]::Singleline)
    $oneQueueHelper = ([regex]::Matches($queueHelper, 'queue_fixed_hw_failure_trace\(').Count -eq 1) -and [regex]::IsMatch($queueHelper, 'HwFailureStatus::TransportFault.*HwFailureRegister::MarvellPublicationStep.*failure\.diagnostic_code\(\)', [Text.RegularExpressions.RegexOptions]::Singleline)
    $oneHardwareSpecTrace = ([regex]::Matches($hwStart, 'queue_data_ring_failure_trace\(').Count -eq 1) -and ([regex]::Matches($hwPoll, 'queue_data_ring_failure_trace\(').Count -eq 1) -and [regex]::IsMatch($hwStart, 'Err\(failure\).*queue_data_ring_failure_trace.*finish_hw_spec_locked.*DataRingUnavailable', [Text.RegularExpressions.RegexOptions]::Singleline) -and [regex]::IsMatch($hwPoll, 'Err\(failure\).*queue_data_ring_failure_trace.*finish_hw_spec_locked.*DataRingUnavailable', [Text.RegularExpressions.RegexOptions]::Singleline) -and [regex]::IsMatch($hwFinish, 'stage == HwSpecStage::Failed\s*&&\s*result != HwSpecResult::DataRingUnavailable.*queue_fixed_hw_failure_trace', [Text.RegularExpressions.RegexOptions]::Singleline)
    $associationHasOneTrace = ([regex]::Matches($association, 'queue_data_ring_failure_trace\(').Count -eq 1) -and [regex]::IsMatch($association, 'Err\(failure\).*queue_data_ring_failure_trace.*ConnectionResult::DataRingUnavailable', [Text.RegularExpressions.RegexOptions]::Singleline)
    $secretFree = -not [regex]::IsMatch($diagnostic, 'ssid|bssid|passphrase|pmk|security_ie|connect_cmd_ptr|connect_rsp_ptr|secret_source|target\.', [Text.RegularExpressions.RegexOptions]::IgnoreCase)

    $codesAreFixed -and $decoderClassesAreComplete -and $causeMapIsComplete -and
        $eventCausePropagates -and $rxConstructionDefersFirmwarePointer -and $publicationCausePropagates -and
        $oneQueueHelper -and $oneHardwareSpecTrace -and $associationHasOneTrace -and $secretFree
}

Require (Test-DataRingFailureDiagnostics $driver) "data-ring failure codes, propagation, redaction, or exactly-one trace behavior are incomplete"
$swappedRxDmaCause = $driver.Replace('DATA_RING_DIAG_RX_DMA_TRANSLATION | index as u32', 'DATA_RING_DIAG_EVENT_DMA_TRANSLATION | index as u32')
Require (-not (Test-DataRingFailureDiagnostics $swappedRxDmaCause)) "swapped RX-DMA cause mapping failure injection was accepted"
$rxWithEarlyPointerObservation = $driver.Replace(
    '    let wrptr = 0;',
    '    let wrptr = read_reg(mmio, PCIE_RX_WR_PTR);' + [Environment]::NewLine +
        '    let _ = decode_rx_tx_pointer_register(wrptr);'
)
Require (-not (Test-DataRingFailureDiagnostics $rxWithEarlyPointerObservation)) "RX arm-time pointer read/decode failure injection was accepted"

function Test-ConnectionTimeoutFingerprint([string]$DriverText, [string]$TraceText) {
    $layout = Slice-Between $DriverText "// Stable MarvellConnectionTimeoutFingerprint payload" "fn queue_connection_timeout_trace"
    $job = Slice-Between $DriverText "struct ConnectionJob" "/// Mutually exclusive credential sources"
    $arm = Slice-Between $DriverText "fn arm_connection_command" "pub fn poll_connection"
    $poll = Slice-Between $DriverText "pub fn poll_connection" "fn next_connection_phase"
    $timeoutAll = Slice-Between $poll "if elapsed_ms(job.phase_started_tsc) >= CONNECT_CMD_TIMEOUT_MS" "if !job.waiting"
    $associateTimeoutAt = $timeoutAll.IndexOf("let associate_doorbell_ack", [StringComparison]::Ordinal)
    Require ($associateTimeoutAt -ge 0) "missing Associate timeout fixture"
    $timeout = $timeoutAll.Substring($associateTimeoutAt)
    $portTimeout = Slice-Between $poll "if job.phase == ConnectionStage::WaitPortRelease" "if elapsed_ms(job.phase_started_tsc) >= CONNECT_CMD_TIMEOUT_MS"
    $waitingAt = $poll.IndexOf("if !job.waiting", [StringComparison]::Ordinal)
    $pollStatusAt = $poll.IndexOf("let status = read_reg(mmio, PCIE_HOST_INT_STATUS);", $waitingAt, [StringComparison]::Ordinal)
    $genericCleanupAt = $poll.IndexOf("if !terminal_quiesce_and_cleanup_while_gated", $pollStatusAt, [StringComparison]::Ordinal)
    Require (($pollStatusAt -gt $waitingAt) -and ($genericCleanupAt -gt $pollStatusAt)) "missing connection completion fixture"
    $completion = $poll.Substring($pollStatusAt, $genericCleanupAt - $pollStatusAt)
    $queue = Slice-Between $DriverText "fn queue_connection_timeout_trace" "#[repr(C, packed)]"
    $finish = Slice-Between $DriverText "fn finish_connection_locked" "fn quarantine_connection_job"
    $quarantine = Slice-Between $DriverText "fn quarantine_connection_job" "fn clear_connection_secret_dma"
    $request = Slice-Between $DriverText "fn connection_request_header_matches_expected" "/// Reads only the fixed redacted response header"
    $response = Slice-Between $DriverText "fn connection_timeout_response_class_after_verified_cleanup" "const fn connection_stage_timeout_code"
    $fixedHeader = Slice-Between $DriverText "fn read_fixed_connection_header" "const fn connection_stage_timeout_code"
    $packer = Slice-Between $DriverText "const fn connection_stage_timeout_code" "fn parse_connection_dma_response"

    $layoutStable = [regex]::IsMatch($layout, 'bits 31\.\.25 = 0x53.*24\.\.22 = ConnectionStage.*21\.\.14 = exact.*HostCmd ID.*13\.\.4 = exact published command length.*bit 3 = fixed 12-byte request header matched.*bit 2 = terminal quiesce.*bits 1\.\.0 = response class.*CONNECTION_TIMEOUT_FINGERPRINT_TAG:\s*u32\s*=\s*0x53.*TAG_SHIFT:\s*u32\s*=\s*25.*STAGE_SHIFT:\s*u32\s*=\s*22.*COMMAND_SHIFT:\s*u32\s*=\s*14.*COMMAND_LEN_SHIFT:\s*u32\s*=\s*4.*REQUEST_HEADER_MATCH:\s*u32\s*=\s*1 << 3.*CLEANUP_VERIFIED:\s*u32\s*=\s*1 << 2', [Text.RegularExpressions.RegexOptions]::Singleline)
    $responseClassesStable = [regex]::IsMatch($DriverText, '#\[repr\(u8\)\]\s*enum ConnectionTimeoutResponseClass.*UntouchedZero\s*=\s*0.*ExpectedHeaderSeen\s*=\s*1.*NonemptyMismatch\s*=\s*2.*Unavailable\s*=\s*3', [Text.RegularExpressions.RegexOptions]::Singleline)
    $jobStoresPublicationFacts = [regex]::IsMatch($job, 'published_command_len:\s*u16.*request_header_matches_expected:\s*bool', [Text.RegularExpressions.RegexOptions]::Singleline)

    $requestAt = $arm.IndexOf('connection_request_header_matches_expected', [StringComparison]::Ordinal)
    $lengthAt = $arm.IndexOf('job.published_command_len = command_len as u16', [StringComparison]::Ordinal)
    $publishAt = $arm.IndexOf('publish_host_command_while_gated', [StringComparison]::Ordinal)
    $requestCapturedBeforePublication = ($requestAt -ge 0) -and ($lengthAt -gt $requestAt) -and ($publishAt -gt $lengthAt)
    $requestIsFixedAndRedacted = [regex]::IsMatch($request, 'read_fixed_connection_header\(connect_cmd_ptr\(\)\).*interface_len as usize == command_len.*interface_type == marvell_wifi_cmd::MWIFIEX_TYPE_CMD.*command == expected_command.*host_command_size as usize == expected_host_size.*sequence == connection_phase_seq.*result == 0', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($request, 'target\.|ssid|bssid|passphrase|pmk|security_ie|secret_source|MWIFIEX_UPLD_SIZE', [Text.RegularExpressions.RegexOptions]::IgnoreCase)

    $statusAt = $timeoutAll.IndexOf('let status = read_reg(mmio, PCIE_HOST_INT_STATUS)', [StringComparison]::Ordinal)
    $quarantineAt = $timeout.IndexOf('quarantine_connection_job(&job)', [StringComparison]::Ordinal)
    $responseAt = $timeout.IndexOf('connection_timeout_response_class_after_verified_cleanup(&job)', [StringComparison]::Ordinal)
    $specializedQueueAt = $timeout.IndexOf('queue_connection_timeout_trace', [StringComparison]::Ordinal)
    $finishAt = $timeout.IndexOf('finish_connection_locked', [StringComparison]::Ordinal)
    $timeoutOrder = ($statusAt -ge 0) -and ($quarantineAt -gt $statusAt) -and ($responseAt -gt $quarantineAt) -and ($specializedQueueAt -gt $responseAt) -and ($finishAt -gt $specializedQueueAt) -and
        [regex]::IsMatch($timeout, 'let cleanup_verified = quarantine_connection_job.*if cleanup_verified.*connection_timeout_response_class_after_verified_cleanup.*else.*ConnectionTimeoutResponseClass::Unavailable.*queue_connection_timeout_trace.*ConnectionResult::CommandTimeout', [Text.RegularExpressions.RegexOptions]::Singleline)
    $responseIsFixedAndPostCleanupOnly = [regex]::IsMatch($response, 'read_fixed_connection_header\(connect_rsp_ptr\(\)\).*header\.is_empty\(\).*UntouchedZero.*HOST_CMD_RET_BIT.*interface_type.*sequence.*interface_len.*host_command_size.*ExpectedHeaderSeen.*NonemptyMismatch', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($response, 'slice::from_raw_parts|HOST_INTR_CMD_DONE|host_cmd_done_is_current|ConnectionResult::LinkReady|status\s*==\s*0', [Text.RegularExpressions.RegexOptions]::Singleline)
    $fixedHeaderIsTwelveBytes = ([regex]::Matches($fixedHeader, 'read_reg\(buffer,').Count -eq 3) -and
        [regex]::IsMatch($fixedHeader, 'read_reg\(buffer, 0\).*read_reg\(buffer, 4\).*read_reg\(buffer, 8\).*interface_len: interface as u16.*interface_type: \(interface >> 16\) as u16.*command: command_size as u16.*host_command_size: \(command_size >> 16\) as u16.*sequence: sequence_result as u16.*result: \(sequence_result >> 16\) as u16', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($fixedHeader, 'read_reg\(buffer, (?:1[2-9]|[2-9][0-9])\)|slice::from_raw_parts|MWIFIEX_UPLD_SIZE')

    $hostStepAt = $queue.IndexOf('register: HwFailureRegister::MarvellHostInterruptStatus', [StringComparison]::Ordinal)
    $fingerprintStepAt = $queue.IndexOf('register: HwFailureRegister::MarvellConnectionTimeoutFingerprint', [StringComparison]::Ordinal)
    $terminalPairIsOneShot = ($hostStepAt -ge 0) -and ($fingerprintStepAt -gt $hostStepAt) -and
        ([regex]::Matches($queue, 'usb::queue_hw_failure_trace\(').Count -eq 1) -and
        ([regex]::Matches($DriverText, 'queue_connection_timeout_trace\(').Count -eq 2) -and
        [regex]::IsMatch($queue, 'status:\s*HwFailureStatus::Timeout.*register_value:\s*host_int_status.*status:\s*HwFailureStatus::Timeout.*register_value:\s*fingerprint', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($queue, 'job\.target|job\.secret_source|connect_cmd_ptr|connect_rsp_ptr|cmd_dma_phys|rsp_dma_phys|\bheader\.|ssid|bssid|passphrase|pmk|security_ie|payload', [Text.RegularExpressions.RegexOptions]::IgnoreCase)
    $genericFallbackRemains = [regex]::IsMatch($finish, 'ConnectionResult::CommandTimeout \| ConnectionResult::PortReleaseTimeout.*HwFailureStatus::Timeout.*queue_fixed_hw_failure_trace.*HwFailureRegister::MarvellHostInterruptStatus.*host_int_status', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($portTimeout, 'queue_connection_timeout_trace')

    $quarantineIsFailClosed = [regex]::IsMatch($quarantine, 'DATA_LINK_READY\.store\(false.*let cleanup_verified = terminal_quiesce_and_cleanup_while_gated.*if cleanup_verified.*clear_connection_secret_dma.*CONNECTION_REBOOT_REQUIRED\.swap\(true.*cleanup_verified', [Text.RegularExpressions.RegexOptions]::Singleline)
    $completionStillRequiresCurrentDone = [regex]::IsMatch($completion, 'host_cmd_done_is_current\(\s*job\.cmd_done_low_baseline,\s*status,\s*HOST_INTR_CMD_DONE.*status & HOST_INTR_CMD_DONE != 0.*StaleCommandDone.*runtime\.job = Some\(job\).*return \(false', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($completion, 'status\s*==\s*0|ExpectedHeaderSeen|expected_header_seen')
    $timeoutAndMmioBoundariesRemain = [regex]::IsMatch($DriverText, 'const CONNECT_CMD_TIMEOUT_MS:\s*u64\s*=\s*15_000') -and
        [regex]::IsMatch($completion, 'status == u32::MAX.*MmioUnavailable', [Text.RegularExpressions.RegexOptions]::Singleline)

    $packingIsComplete = [regex]::IsMatch($packer, 'ConnectionStage::Idle\s*=>\s*0.*SupplicantProfile\s*=>\s*1.*SupplicantPmk\s*=>\s*2.*Associate\s*=>\s*3.*WaitPortRelease\s*=>\s*4.*LinkReady\s*=>\s*5.*Failed\s*=>\s*6.*expected_command:\s*u16.*published_command_len:\s*u16.*request_header_matches_expected:\s*bool.*cleanup_verified:\s*bool.*response_class:\s*ConnectionTimeoutResponseClass.*FINGERPRINT_TAG << CONNECTION_TIMEOUT_FINGERPRINT_TAG_SHIFT.*connection_stage_timeout_code\(stage\) << CONNECTION_TIMEOUT_STAGE_SHIFT.*u32::from\(expected_command\) << CONNECTION_TIMEOUT_COMMAND_SHIFT.*u32::from\(published_command_len\) << CONNECTION_TIMEOUT_COMMAND_LEN_SHIFT.*CONNECTION_TIMEOUT_REQUEST_HEADER_MATCH.*CONNECTION_TIMEOUT_CLEANUP_VERIFIED.*response_class as u32', [Text.RegularExpressions.RegexOptions]::Singleline)
    $traceRegisterIsStable = [regex]::IsMatch($TraceText, 'MarvellHostInterruptStatus = 1.*MarvellConnectionTimeoutFingerprint = 9', [Text.RegularExpressions.RegexOptions]::Singleline)

    $layoutStable -and $responseClassesStable -and $jobStoresPublicationFacts -and
        $requestCapturedBeforePublication -and $requestIsFixedAndRedacted -and $timeoutOrder -and
        $responseIsFixedAndPostCleanupOnly -and $fixedHeaderIsTwelveBytes -and $terminalPairIsOneShot -and $genericFallbackRemains -and
        $quarantineIsFailClosed -and $completionStillRequiresCurrentDone -and
        $timeoutAndMmioBoundariesRemain -and $packingIsComplete -and $traceRegisterIsStable
}

function Test-AssociateDoorbellAck([string]$DriverText) {
    $layout = Slice-Between $DriverText "// Stable, secret-free Associate doorbell acknowledgement values" "fn queue_connection_timeout_trace"
    $poll = Slice-Between $DriverText "pub fn poll_connection" "fn next_connection_phase"
    $timeoutAll = Slice-Between $poll "if elapsed_ms(job.phase_started_tsc) >= CONNECT_CMD_TIMEOUT_MS" "if !job.waiting"
    $associateTimeoutAt = $timeoutAll.IndexOf("let associate_doorbell_ack", [StringComparison]::Ordinal)
    Require ($associateTimeoutAt -ge 0) "missing Associate timeout fixture"
    $timeout = $timeoutAll.Substring($associateTimeoutAt)
    $portTimeout = Slice-Between $poll "if job.phase == ConnectionStage::WaitPortRelease" "if elapsed_ms(job.phase_started_tsc) >= CONNECT_CMD_TIMEOUT_MS"
    $waitingAt = $poll.IndexOf("if !job.waiting", [StringComparison]::Ordinal)
    $pollStatusAt = $poll.IndexOf("let status = read_reg(mmio, PCIE_HOST_INT_STATUS);", $waitingAt, [StringComparison]::Ordinal)
    $genericCleanupAt = $poll.IndexOf("if !terminal_quiesce_and_cleanup_while_gated", $pollStatusAt, [StringComparison]::Ordinal)
    Require (($pollStatusAt -gt $waitingAt) -and ($genericCleanupAt -gt $pollStatusAt)) "missing connection completion fixture"
    $completion = $poll.Substring($pollStatusAt, $genericCleanupAt - $pollStatusAt)
    $queue = Slice-Between $DriverText "fn queue_connection_timeout_trace" "#[repr(C, packed)]"

    $stableValuesAndMapping = [regex]::IsMatch(
        $layout,
        'ASSOCIATE_DOORBELL_ACK_CLEARED:\s*u32\s*=\s*0xD201_0000.*ASSOCIATE_DOORBELL_ACK_STILL_SET:\s*u32\s*=\s*0xD201_0001.*ASSOCIATE_DOORBELL_ACK_UNAVAILABLE:\s*u32\s*=\s*0xD201_0002.*fn associate_doorbell_ack_trace_value\(cpu_int_status:\s*u32\)\s*->\s*u32.*cpu_int_status == u32::MAX.*ASSOCIATE_DOORBELL_ACK_UNAVAILABLE.*cpu_int_status & CPU_INTR_DOOR_BELL == 0.*ASSOCIATE_DOORBELL_ACK_CLEARED.*ASSOCIATE_DOORBELL_ACK_STILL_SET',
        [Text.RegularExpressions.RegexOptions]::Singleline
    )

    $cpuStatusReadCount = [regex]::Matches($timeout, 'read_reg\(\s*mmio,\s*PCIE_CPU_INT_STATUS,?\s*\)').Count
    $cpuStatusTokenCount = [regex]::Matches($timeout, 'PCIE_CPU_INT_STATUS').Count
    $readAt = $timeout.IndexOf('PCIE_CPU_INT_STATUS', [StringComparison]::Ordinal)
    $quarantineAt = $timeout.IndexOf('quarantine_connection_job(&job)', [StringComparison]::Ordinal)
    $associateOnlySingleReadBeforeQuarantine = ($cpuStatusReadCount -eq 1) -and
        ($cpuStatusTokenCount -eq 1) -and ($readAt -ge 0) -and ($quarantineAt -gt $readAt) -and
        [regex]::IsMatch(
            $timeout,
            'let associate_doorbell_ack = if job\.phase == ConnectionStage::Associate\s*\{\s*Some\(associate_doorbell_ack_trace_value\(read_reg\(\s*mmio,\s*PCIE_CPU_INT_STATUS,?\s*\)\)\)\s*\}\s*else\s*\{\s*None\s*\};\s*let cleanup_verified = quarantine_connection_job',
            [Text.RegularExpressions.RegexOptions]::Singleline
        ) -and
        -not [regex]::IsMatch($portTimeout, 'PCIE_CPU_INT_STATUS|associate_doorbell_ack')

    $hostStepAt = $queue.IndexOf('register: HwFailureRegister::MarvellHostInterruptStatus', [StringComparison]::Ordinal)
    $fingerprintStepAt = $queue.IndexOf('register: HwFailureRegister::MarvellConnectionTimeoutFingerprint', [StringComparison]::Ordinal)
    $doorbellStepAt = $queue.IndexOf('register: HwFailureRegister::MarvellPublicationStep', [StringComparison]::Ordinal)
    $thirdStepIsBoundedAndOneShot = ($hostStepAt -ge 0) -and ($fingerprintStepAt -gt $hostStepAt) -and
        ($doorbellStepAt -gt $fingerprintStepAt) -and
        ([regex]::Matches($queue, 'register:\s*HwFailureRegister::MarvellPublicationStep').Count -eq 1) -and
        ([regex]::Matches($queue, 'usb::queue_hw_failure_trace\(').Count -eq 1) -and
        [regex]::IsMatch(
            $queue,
            'response_class:\s*ConnectionTimeoutResponseClass,\s*associate_doorbell_ack:\s*Option<u32>.*register_value:\s*fingerprint.*if let Some\(register_value\) = associate_doorbell_ack.*status:\s*HwFailureStatus::Timeout.*register:\s*HwFailureRegister::MarvellPublicationStep.*register_value',
            [Text.RegularExpressions.RegexOptions]::Singleline
        ) -and
        [regex]::IsMatch(
            $timeout,
            'queue_connection_timeout_trace\(\s*&job,\s*status,\s*cleanup_verified,\s*response_class,\s*associate_doorbell_ack,\s*\)',
            [Text.RegularExpressions.RegexOptions]::Singleline
        )

    $completionDoesNotObserveDoorbell = -not [regex]::IsMatch(
        $completion,
        'PCIE_CPU_INT_STATUS|CPU_INTR_DOOR_BELL|associate_doorbell_ack|ASSOCIATE_DOORBELL_ACK|0xD201',
        [Text.RegularExpressions.RegexOptions]::IgnoreCase
    )
    $classifierHasOneRuntimeUse = [regex]::Matches(
        $DriverText,
        'associate_doorbell_ack_trace_value\('
    ).Count -eq 2
    $secretFree = -not [regex]::IsMatch(
        ($layout + $queue),
        'connect_cmd_ptr|connect_rsp_ptr|cmd_dma_phys|rsp_dma_phys|job\.target|job\.secret_source|\bheader\.|payload|address|ssid|bssid|passphrase|pmk|security_ie',
        [Text.RegularExpressions.RegexOptions]::IgnoreCase
    )

    $stableValuesAndMapping -and $associateOnlySingleReadBeforeQuarantine -and
        $thirdStepIsBoundedAndOneShot -and $completionDoesNotObserveDoorbell -and
        $classifierHasOneRuntimeUse -and $secretFree
}

function Test-H25PostPmkHwSpecCanary([string]$DriverText) {
    $layout = Slice-Between $DriverText "// Stable, secret-free H25 post-PMK GET_HW_SPEC canary result" "// Stable, secret-free Associate doorbell acknowledgement values"
    $start = Slice-Between $DriverText "fn start_association_inner" "fn same_connection_target"
    $prepare = Slice-Between $DriverText "fn prepare_connection_dma" "/// Captures one secret-free request predicate"
    $canaryBuild = Slice-Between $prepare "ConnectionStage::PostPmkHwSpecCanary" "ConnectionStage::Associate"
    $phase = Slice-Between $DriverText "fn connection_phase_seq" "fn connection_phase_command"
    $command = Slice-Between $DriverText "fn connection_phase_command" "fn connection_response_error_class"
    $poll = Slice-Between $DriverText "pub fn poll_connection" "fn next_connection_phase"
    $timeout = Slice-Between $poll "if job.phase == ConnectionStage::PostPmkHwSpecCanary" "let associate_doorbell_ack"
    $doneAt = $poll.IndexOf('host_cmd_done_is_current', [StringComparison]::Ordinal)
    $staleCanaryAt = $poll.IndexOf('if job.phase == ConnectionStage::PostPmkHwSpecCanary', $doneAt, [StringComparison]::Ordinal)
    $canaryAt = $poll.IndexOf('if job.phase == ConnectionStage::PostPmkHwSpecCanary', $staleCanaryAt + 1, [StringComparison]::Ordinal)
    $genericCleanupAt = $poll.IndexOf('if !terminal_quiesce_and_cleanup_while_gated', $canaryAt, [StringComparison]::Ordinal)
    Require (($doneAt -ge 0) -and ($staleCanaryAt -gt $doneAt) -and ($canaryAt -gt $staleCanaryAt) -and ($genericCleanupAt -gt $canaryAt)) "H25 canary completion fixture is not ordered after current CMD_DONE"
    $currentEpochGate = $poll.Substring($doneAt, $canaryAt - $doneAt)
    $completion = $poll.Substring($canaryAt, $genericCleanupAt - $canaryAt)
    $parser = Slice-Between $DriverText "fn parse_post_pmk_hw_spec_canary_response" "fn parse_connection_dma_response"
    $next = Slice-Between $DriverText "fn next_connection_phase" "fn finish_connection_locked"
    $finish = Slice-Between $DriverText "fn finish_connection_locked" "fn quarantine_connection_job"
    $quarantine = Slice-Between $DriverText "fn quarantine_connection_job" "fn clear_connection_secret_dma"
    $queue = Slice-Between $DriverText "fn queue_post_pmk_canary_result" "// Stable, secret-free Associate doorbell acknowledgement values"

    $stableResult = [regex]::IsMatch($layout, 'POST_PMK_CANARY_RESULT_TAG:\s*u32\s*=\s*0xD225_0000.*#\[repr\(u8\)\].*ExpectedCompletion\s*=\s*0.*FirmwareResult\s*=\s*1.*MalformedOrWrongCompletion\s*=\s*2.*TimeoutDoorbellCleared\s*=\s*3.*TimeoutDoorbellStillSet\s*=\s*4.*MmioOrDoorbellUnavailable\s*=\s*5.*StaleHighCompletion\s*=\s*6.*HostPublicationFailure\s*=\s*7.*POST_PMK_CANARY_RESULT_TAG \| outcome as u32', [Text.RegularExpressions.RegexOptions]::Singleline)
    $oneSecretFreeRecord = [regex]::IsMatch($queue, 'fn queue_post_pmk_canary_result\(outcome: PostPmkCanaryOutcome\)\s*\{\s*queue_fixed_hw_failure_trace\(\s*HwFailurePhase::HardwareSpec,\s*HwFailureStatus::NetworkStateNotGranted,\s*HwFailureRegister::MarvellCommandResponseStatus,\s*post_pmk_canary_trace_value\(outcome\),\s*\);\s*\}', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($queue, 'connect_(?:cmd|rsp)_ptr|job\.|\bssid\b|\bbssid\b|\bpassphrase\b|secret_source|security_ie|header\.|dma_phys', [Text.RegularExpressions.RegexOptions]::IgnoreCase)

    $profilePmkCanaryOnly = [regex]::IsMatch($next, 'SupplicantProfile\s*=>\s*ConnectionStage::SupplicantPmk.*SupplicantPmk\s*=>\s*ConnectionStage::PostPmkHwSpecCanary.*PostPmkHwSpecCanary\s*=>\s*ConnectionStage::Failed', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($canaryBuild, 'build_get_hw_spec\(seq, out\)') -and
        -not [regex]::IsMatch($canaryBuild, 'build_associate|ASSOCIATE_CMD') -and
        [regex]::IsMatch($phase, 'PostPmkHwSpecCanary\s*=>\s*2') -and
        [regex]::IsMatch($command, 'PostPmkHwSpecCanary\s*=>\s*Some\(marvell_wifi_cmd::GET_HW_SPEC_CMD\)')

    $freshDoneThenCleanupThenParse = [regex]::IsMatch($poll, 'host_cmd_done_is_current\(\s*job\.cmd_done_low_baseline,\s*status,\s*HOST_INTR_CMD_DONE.*if job\.phase == ConnectionStage::PostPmkHwSpecCanary', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($completion, 'let cleanup_verified = quarantine_connection_job\(&job\).*if !cleanup_verified.*MmioOrDoorbellUnavailable.*else.*parse_post_pmk_hw_spec_canary_response\(&job\).*ExpectedCompletion.*FwResult.*FirmwareResult.*MalformedOrWrongCompletion', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($parser, 'slice::from_raw_parts\(connect_rsp_ptr\(\)\.cast_const\(\), MWIFIEX_UPLD_SIZE\).*parse_hw_spec_response\(\s*connection_phase_seq\(job.seq, job.phase\),\s*response.*\.map\(\|_\| \(\)\)', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($poll.Substring(0, $doneAt), 'parse_post_pmk_hw_spec_canary_response|connect_rsp_ptr')

    $timeoutIsBoundedAndObservational = [regex]::IsMatch($DriverText, 'const CONNECT_CMD_TIMEOUT_MS:\s*u64\s*=\s*15_000') -and
        ([regex]::Matches($timeout, 'read_reg\(mmio, PCIE_CPU_INT_STATUS\)').Count -eq 1) -and
        [regex]::IsMatch($timeout, 'doorbell_status = read_reg\(mmio, PCIE_CPU_INT_STATUS\).*cleanup_verified = quarantine_connection_job.*status == u32::MAX.*doorbell_status == u32::MAX.*!cleanup_verified.*MmioOrDoorbellUnavailable.*doorbell_status & CPU_INTR_DOOR_BELL == 0.*TimeoutDoorbellCleared.*TimeoutDoorbellStillSet', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch($timeout, 'parse_post_pmk_hw_spec_canary_response|connect_rsp_ptr|ExpectedCompletion')

    $allTerminalAndNoNetwork = [regex]::IsMatch($timeout, 'MmioOrDoorbellUnavailable.*TimeoutDoorbellCleared.*TimeoutDoorbellStillSet.*queue_post_pmk_canary_result\(outcome\).*ConnectionResult::RebootRequired.*ConnectionStage::Failed', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($currentEpochGate, 'if status & HOST_INTR_CMD_DONE != 0.*if job\.phase == ConnectionStage::PostPmkHwSpecCanary.*queue_post_pmk_canary_result\(PostPmkCanaryOutcome::StaleHighCompletion\).*ConnectionTransportError::StaleCommandDone.*ConnectionStage::Failed', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($completion, 'MmioOrDoorbellUnavailable.*ExpectedCompletion.*FirmwareResult.*MalformedOrWrongCompletion.*queue_post_pmk_canary_result\(outcome\).*ConnectionResult::RebootRequired.*ConnectionStage::Failed', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($poll, 'ConnectionTransportError::StaleCommandDone.*PostPmkCanaryOutcome::StaleHighCompletion.*ConnectionTransportError::MmioUnavailable.*PostPmkCanaryOutcome::MmioOrDoorbellUnavailable.*PostPmkCanaryOutcome::HostPublicationFailure', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch(($timeout + $completion), 'ConnectionResult::RebootRequired.*ConnectionStage::Failed.*DATA_LINK_READY\.store\(false.*DeferredNetworkAction::None', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        -not [regex]::IsMatch(($timeout + $completion), 'DATA_LINK_READY\.store\(true|activate_validated_data_dma|DeferredNetworkAction::Attach|ConnectionStage::LinkReady|WaitPortRelease|build_associate') -and
        [regex]::IsMatch($quarantine, 'DATA_LINK_READY\.store\(false.*terminal_quiesce_and_cleanup_while_gated.*CONNECTION_REBOOT_REQUIRED\.swap\(true', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($start, 'connection_reboot_required\(\).*ConnectionResult::RebootRequired', [Text.RegularExpressions.RegexOptions]::Singleline) -and
        [regex]::IsMatch($finish, 'previous_stage != ConnectionStage::PostPmkHwSpecCanary.*queue_fixed_hw_failure_trace', [Text.RegularExpressions.RegexOptions]::Singleline)

    foreach ($check in @(@("stable",$stableResult),@("record",$oneSecretFreeRecord),@("flow",$profilePmkCanaryOnly),@("done",$freshDoneThenCleanupThenParse),@("timeout",$timeoutIsBoundedAndObservational),@("terminal",$allTerminalAndNoNetwork))) { if (-not $check[1]) { [Console]::Error.WriteLine("H25_DEBUG " + $check[0]) } }
    $stableResult -and $oneSecretFreeRecord -and $profilePmkCanaryOnly -and
        $freshDoneThenCleanupThenParse -and $timeoutIsBoundedAndObservational -and
        $allTerminalAndNoNetwork
}

Require (Test-ConnectionTimeoutFingerprint $driver $trace) "connection-timeout fingerprint layout, ordering, redaction, one-shot trace, or fail-closed boundary is incomplete"
Require (Test-AssociateDoorbellAck $driver) "Associate doorbell ACK values, single-read placement, third-step ordering, redaction, or observational boundary is incomplete"
Require (Test-H25PostPmkHwSpecCanary $driver) "H25 post-PMK GET_HW_SPEC canary flow, completion boundary, result encoding, quarantine, or redaction is incomplete"
$canaryResponseAsCompletion = $driver.Replace(
    'if !marvell_wifi_cmd::host_cmd_done_is_current(',
    'if parse_post_pmk_hw_spec_canary_response(&job).is_err() || !marvell_wifi_cmd::host_cmd_done_is_current('
)
Require ($canaryResponseAsCompletion -cne $driver) "H25 response-as-completion mutation fixture did not apply"
Require (-not (Test-H25PostPmkHwSpecCanary $canaryResponseAsCompletion)) "H25 response-as-completion failure injection was accepted"
$canaryParseBeforeCleanup = $driver.Replace(
    'let cleanup_verified = quarantine_connection_job(&job);',
    'let _early_canary_response = parse_post_pmk_hw_spec_canary_response(&job); let cleanup_verified = quarantine_connection_job(&job);'
)
Require ($canaryParseBeforeCleanup -cne $driver) "H25 pre-cleanup parse mutation fixture did not apply"
Require (-not (Test-H25PostPmkHwSpecCanary $canaryParseBeforeCleanup)) "H25 pre-cleanup response parse failure injection was accepted"
$canaryStaleAsExpected = $driver.Replace(
    'queue_post_pmk_canary_result(PostPmkCanaryOutcome::StaleHighCompletion);',
    'queue_post_pmk_canary_result(PostPmkCanaryOutcome::ExpectedCompletion);'
)
Require ($canaryStaleAsExpected -cne $driver) "H25 stale-high mutation fixture did not apply"
Require (-not (Test-H25PostPmkHwSpecCanary $canaryStaleAsExpected)) "H25 stale-high completion failure injection was accepted"
$canaryAssociatePublication = $driver.Replace(
    'ConnectionStage::PostPmkHwSpecCanary => Some(marvell_wifi_cmd::GET_HW_SPEC_CMD),',
    'ConnectionStage::PostPmkHwSpecCanary => Some(marvell_wifi_cmd::ASSOCIATE_CMD),'
)
Require ($canaryAssociatePublication -cne $driver) "H25 Associate publication mutation fixture did not apply"
Require (-not (Test-H25PostPmkHwSpecCanary $canaryAssociatePublication)) "H25 accidental Associate publication failure injection was accepted"
$canarySecretLeak = $driver.Replace(
    'post_pmk_canary_trace_value(outcome),',
    'connect_rsp_ptr() as u32,'
)
Require ($canarySecretLeak -cne $driver) "H25 response payload/address mutation fixture did not apply"
Require (-not (Test-H25PostPmkHwSpecCanary $canarySecretLeak)) "H25 response payload/address persistence failure injection was accepted"
$responseBeforeQuiesce = $driver.Replace(
    'let cleanup_verified = quarantine_connection_job(&job);',
    'let _early_response = connection_timeout_response_class_after_verified_cleanup(&job); let cleanup_verified = quarantine_connection_job(&job);'
)
Require (-not (Test-ConnectionTimeoutFingerprint $responseBeforeQuiesce $trace)) "response probe before verified BME-off/cleanup failure injection was accepted"
$headerInsteadOfDone = $driver.Replace('marvell_wifi_cmd::host_cmd_done_is_current(', 'connection_timeout_expected_header_seen(')
Require (-not (Test-ConnectionTimeoutFingerprint $headerInsteadOfDone $trace)) "expected-header-seen as current-epoch CMD_DONE failure injection was accepted"
$statusZeroInsteadOfDone = $driver.Replace('if !marvell_wifi_cmd::host_cmd_done_is_current(', 'if status == 0 && !marvell_wifi_cmd::host_cmd_done_is_current(')
Require (-not (Test-ConnectionTimeoutFingerprint $statusZeroInsteadOfDone $trace)) "status zero as current-epoch CMD_DONE failure injection was accepted"
$changedTimeout = $driver.Replace('const CONNECT_CMD_TIMEOUT_MS: u64 = 15_000;', 'const CONNECT_CMD_TIMEOUT_MS: u64 = 15_001;')
Require ($changedTimeout -cne $driver) "timeout mutation fixture did not apply"
Require (-not (Test-ConnectionTimeoutFingerprint $changedTimeout $trace)) "connection timeout change failure injection was accepted"
$withoutOriginalHostInterruptStep = $driver.Replace('register: HwFailureRegister::MarvellHostInterruptStatus,', 'register: HwFailureRegister::None,')
Require (-not (Test-ConnectionTimeoutFingerprint $withoutOriginalHostInterruptStep $trace)) "removing the original host-interrupt timeout step was accepted"
$forbiddenTraceLeaks = @(
    @("raw_response_address", "connect_rsp_ptr() as u32"),
    @("raw_header_word", "header.command as u32"),
    @("payload", "payload.len() as u32"),
    @("ssid", "ssid.len() as u32"),
    @("bssid", "bssid[0] as u32"),
    @("passphrase", "passphrase.len() as u32"),
    @("pmk", "pmk[0] as u32")
)
foreach ($leak in $forbiddenTraceLeaks) {
    $mutatedTrace = $driver.Replace('register_value: fingerprint,', ("register_value: " + $leak[1] + ","))
    Require (-not (Test-ConnectionTimeoutFingerprint $mutatedTrace $trace)) ("forbidden " + $leak[0] + " in the timeout trace failure injection was accepted")

    $mutatedDoorbellTrace = $driver.Replace(
        '                register_value,',
        ("                register_value: " + $leak[1] + ",")
    )
    Require ($mutatedDoorbellTrace -cne $driver) ("forbidden doorbell " + $leak[0] + " mutation fixture did not apply")
    Require (-not (Test-AssociateDoorbellAck $mutatedDoorbellTrace)) ("forbidden " + $leak[0] + " in the doorbell ACK trace failure injection was accepted")
}

$readAfterQuarantine = $driver.Replace(
    'let cleanup_verified = quarantine_connection_job(&job);',
    'let cleanup_verified = quarantine_connection_job(&job); let _late_doorbell = read_reg(mmio, PCIE_CPU_INT_STATUS);'
)
Require ($readAfterQuarantine -cne $driver) "post-quarantine doorbell mutation fixture did not apply"
Require (-not (Test-AssociateDoorbellAck $readAfterQuarantine)) "doorbell read after quarantine failure injection was accepted"
$readOutsideAssociate = $driver.Replace(
    'let associate_doorbell_ack = if job.phase == ConnectionStage::Associate {',
    'let associate_doorbell_ack = if job.phase != ConnectionStage::Associate {'
)
Require ($readOutsideAssociate -cne $driver) "non-Associate doorbell mutation fixture did not apply"
Require (-not (Test-AssociateDoorbellAck $readOutsideAssociate)) "doorbell read in non-Associate phases failure injection was accepted"
$secondDoorbellRead = $driver.Replace(
    'Some(associate_doorbell_ack_trace_value(read_reg(',
    'Some(associate_doorbell_ack_trace_value(read_reg(mmio, PCIE_CPU_INT_STATUS) | read_reg('
)
Require ($secondDoorbellRead -cne $driver) "second doorbell read mutation fixture did not apply"
Require (-not (Test-AssociateDoorbellAck $secondDoorbellRead)) "second doorbell read failure injection was accepted"
$doorbellAsCompletion = $driver.Replace(
    'if !marvell_wifi_cmd::host_cmd_done_is_current(',
    'if associate_doorbell_ack_trace_value(read_reg(mmio, PCIE_CPU_INT_STATUS)) == ASSOCIATE_DOORBELL_ACK_CLEARED || !marvell_wifi_cmd::host_cmd_done_is_current('
)
Require ($doorbellAsCompletion -cne $driver) "doorbell completion mutation fixture did not apply"
Require (-not (Test-AssociateDoorbellAck $doorbellAsCompletion)) "doorbell acknowledgement as completion evidence failure injection was accepted"
$mutatedDoorbellValue = $driver.Replace('0xD201_0001', '0xD201_0011')
Require ($mutatedDoorbellValue -cne $driver) "doorbell value mutation fixture did not apply"
Require (-not (Test-AssociateDoorbellAck $mutatedDoorbellValue)) "doorbell stable-value mutation failure injection was accepted"
$mutatedUnavailableMapping = $driver.Replace('cpu_int_status == u32::MAX', 'cpu_int_status == 0')
Require ($mutatedUnavailableMapping -cne $driver) "doorbell unavailable mapping mutation fixture did not apply"
Require (-not (Test-AssociateDoorbellAck $mutatedUnavailableMapping)) "all-ones unavailable mapping failure injection was accepted"

Require-Match $extractor 'k2_publication' "Windows extractor does not identify K2 publication records"
Require-Match $extractor 'marvell_pci_command.*k2_publication_rejected' "Windows extractor does not decode K2 PCI/publication steps"
Require-Match $extractor 'raios\.usb_diag\.v0.*usb_diagnostics' "Windows extractor does not surface usb_diag records"
Require-Match $extractor 'function Convert-ConnectionTimeoutFingerprint.*connection_stage.*expected_command_id.*published_command_len.*request_header_matches_expected.*verified_quiesce_cleanup.*response_class' "Windows extractor does not decode every connection-timeout fingerprint field"
Require-Match $extractor 'marvell_connection_timeout_fingerprint.*connection_timeout_fingerprint_tag_invalid' "Windows extractor lacks the dedicated timeout register name or mutation boundary"
Require-Match $extractor 'function Convert-AssociateDoorbellAck.*d2010000.*cleared.*d2010001.*still_set.*d2010002.*unavailable.*associate_doorbell_ack_value_invalid' "Windows extractor does not strictly decode the three doorbell ACK values"
Require-Match $extractor 'phase -eq 5.*status -eq 100.*register -eq 8.*associate_doorbell_ack' "Windows extractor does not type only the Associate timeout publication step as doorbell ACK"
$extractorSelfTest = & powershell -NoProfile -ExecutionPolicy Bypass -File $extractorPath -SelfTest | ConvertFrom-Json
Require ($extractorSelfTest.status -eq "passed") "Windows trace extractor self-test failed"
Require (@($extractorSelfTest.checks) -contains "k2_decoded") "Windows extractor self-test does not cover K2 decoding"
Require (@($extractorSelfTest.checks) -contains "usb_diag_visible") "Windows extractor self-test does not cover usb_diag visibility"
Require (@($extractorSelfTest.checks) -contains "connection_timeout_fingerprint_decoded") "Windows extractor self-test does not cover the connection-timeout fingerprint"
Require (@($extractorSelfTest.checks) -contains "connection_timeout_fingerprint_bit_mutation_rejected") "Windows extractor self-test does not reject a fingerprint bit mutation"
Require (@($extractorSelfTest.checks) -contains "associate_doorbell_ack_decoded") "Windows extractor self-test does not decode the doorbell acknowledgement"
Require (@($extractorSelfTest.checks) -contains "associate_doorbell_ack_tag_mutation_rejected") "Windows extractor self-test does not reject a doorbell ACK tag mutation"
Require (@($extractorSelfTest.checks) -contains "associate_doorbell_ack_value_mutation_rejected") "Windows extractor self-test does not reject a doorbell ACK value mutation"

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
    (Slice-Between $driver "fn arm_event_ring_while_gated" "fn arm_rx_ring_while_gated")
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
        & powershell -NoProfile -ExecutionPolicy Bypass `
            -File (Join-Path $RepoRoot "scripts\build-seed-kernel.ps1") `
            -Profile release
        if ($LASTEXITCODE -ne 0) { throw "freestanding Seed kernel compilecheck failed" }
    }
    finally {
        $env:CARGO_HOME = $oldCargoHome
        $env:CARGO_TARGET_DIR = $oldTarget
    }
}

Write-Output "MARVELL_CONNECTION_TELEMETRY_FIXTURE status=pass seq_field=low8 bss=0 payload_logged=false"
