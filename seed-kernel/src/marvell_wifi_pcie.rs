//! Marvell 88W8897 PCIe firmware-download hardware shell.
//!
//! Hardware bring-up is owner-triggered only. It is honestly unaudited: the
//! proprietary firmware blob and DMA path are trusted until raiOS has IOMMU
//! enforcement.

use core::hint::spin_loop;
use core::ptr;
use core::slice;
use core::sync::atomic::{compiler_fence, AtomicBool, AtomicU64, Ordering};

use raios_core::boot_control::BootPosture;
use raios_core::dot11_scan::Dot11Security;
use raios_core::hw_failure_trace::{
    HwFailurePhase, HwFailureRegister, HwFailureStatus, HwFailureSubsystem, HwFailureTrace,
    HwFailureTraceFlushStatus, HwFailureTraceQueueResult, HwFailureTraceStep,
    SEED_KERNEL_BUILD_ID_V0_1_0,
};
use raios_core::marvell_dma_safety::{
    compose_rx_tx_pointer_register, decode_event_pointer, decode_rx_tx_pointer_register,
    update_rx_pointer_preserving_tx, update_tx_pointer_preserving_rx,
    validate_contiguous_translation, validate_non_overlapping_regions, DevicePointerError,
    DeviceRingPointer, DmaSpan, ForeignRegion, ForeignRegionKind, MarvellDmaRegion,
    MarvellDmaRegionKind, PageTranslation, PublicationModel, PublicationStep, RxTxDevicePointers,
};
use raios_core::marvell_wifi_cmd::{
    self, HwSpecCmdError, MarvellCmdError, SingleBssHostCmdSequenceAllocator,
};
use raios_core::marvell_wifi_fw::{
    decide_fw_ready_poll, plan_register_writes, FirmwareDownload, FwAction, FwError, FwPhase,
    FwReadyPollDecision, RegisterReads, CMDRSP_ADDR_HI, CMDRSP_ADDR_LO, CMD_ADDR_HI, CMD_ADDR_LO,
    CMD_SIZE, CPU_INTR_DOOR_BELL, DRV_READY, FW_DMA_STAGING_SIZE, FW_DUMP_CTRL,
    FW_READY_TIMEOUT_MS, FW_STATUS, HOST_INTR_CMD_DONE, HOST_INTR_EVENT_RDY, HOST_INTR_MASK,
    HOST_INTR_UPLD_RDY, MWIFIEX_UPLD_SIZE, PCIE_CPU_INT_EVENT, PCIE_CPU_INT_STATUS,
    PCIE_HOST_INT_MASK, PCIE_HOST_INT_STATUS, PCIE_HOST_INT_STATUS_MASK,
};
use raios_core::marvell_wifi_supplicant::{self, SupplicantError};
use spin::Mutex;

use crate::{ahci, heap, memory, net, pci, secret_vault, serial, time, usb, wifi};

// Linux resource index 2: PCI config offset 0x18. BAR0 is 64-bit on this part.
const MARVELL_REGISTER_BAR: u8 = 2;
const CMD_SIZE_TIMEOUT_MS: u64 = 5_000;
const DOORBELL_ACK_TIMEOUT_MS: u64 = 5_000;
const TOTAL_BRINGUP_TIMEOUT_MS: u64 = 300_000;
const SHORT_POLL_DELAY_US: u64 = 10;
const ACTIONS_PER_POLL: usize = 32;
const FW_DOWNLOAD_ACTIONS_PER_POLL: usize = 512;
const HWSPEC_CMD_BUFFER_SIZE: usize = 128;
const HWSPEC_TIMEOUT_MS: u64 = 3_000;
const SCAN_CMD_BUFFER_SIZE: usize = marvell_wifi_cmd::SCAN_24GHZ_CMD_TOTAL_LEN;
const SCAN_CMD_TIMEOUT_MS: u64 = 15_000;
const CONNECT_CMD_BUFFER_SIZE: usize = 512;
const CONNECT_CMD_TIMEOUT_MS: u64 = 15_000;
const CONNECTION_CMD_DONE_CLEAR_POLLS: usize = 64;
const CONNECTION_MMIO_LIVENESS_POLLS: usize = 64;
const INIT_HOST_CMD_PHASE_COUNT: u8 = 4;
const CONNECTION_HOST_CMD_PHASE_COUNT: u8 = 3;
const PORT_RELEASE_TIMEOUT_MS: u64 = 30_000;
const RX_RING_COUNT: usize = 32;
const RX_RING_MASK: u32 = 0x0000_03ff;
const RX_ROLLOVER_IND: u32 = 1 << 10;
const RX_BUFFER_SIZE: usize = 4096;
const TX_RING_COUNT: usize = 32;
const TX_RING_MASK: u32 = 0x03ff_0000;
const TX_RING_WRAP_MASK: u32 = 0x07ff_0000;
const TX_ROLLOVER_IND: u32 = 1 << 26;
const TX_RING_STEP: u32 = 1 << 16;
const TX_BUFFER_SIZE: usize = 4096;
const DATA_INTERFACE_HEADER_LEN: usize = 4;
const TX_PD_LEN: usize = 20;
const RX_PD_LEN: usize = 20;
pub const MAX_ETHERNET_FRAME_SIZE: usize = 1536;
const PCIE_RX_RD_TX_WR_PTR: u32 = 0xC05C;
const PCIE_RX_WR_PTR: u32 = 0xC08C;
const PCIE_TX_RD_PTR: u32 = 0xC08C;
const RX_DESC_FLAG_SOP: u16 = 1 << 0;
const RX_DESC_FLAG_EOP: u16 = 1 << 1;
const CPU_INTR_DNLD_RDY: u32 = 1 << 0;
const HOST_INTR_DNLD_DONE: u32 = 1 << 0;
const EVENT_RING_COUNT: usize = 8;
const EVENT_RING_MASK: u32 = 0x0f;
const EVENT_ROLLOVER_IND: u32 = 1 << 7;
const EVENT_BUFFER_SIZE: usize = 2048;
const EVENT_HEADER_LEN: usize = 4;
const DMA_PAGE_SIZE: u64 = 4096;
const DMA_ADDRESS_BITS: u8 = 64;
const DMA_REQUIRED_ALIGNMENT: u64 = 2;
const MAX_DMA_TRANSLATION_PAGES: usize = 40;
const MARVELL_DMA_REGION_COUNT: usize = 10;
const PCIE_EVT_RD_PTR: u32 = 0xCE8;
const PCIE_EVT_WR_PTR: u32 = 0xCEC;
const CPU_INTR_EVENT_DONE: u32 = 1 << 5;
const K2_PUBLICATION_STEP_DMA_VALIDATION: u32 = 1;
const K2_PUBLICATION_STEP_MODEL_ENABLE: u32 = 2;
const K2_PUBLICATION_STEP_DOORBELL: u32 = 4;
const K2_PUBLICATION_STEP_CHECKPOINT_FLUSH: u32 = 5;

// Stable MarvellPublicationStep payload for data-ring failures:
// 0xD1KK_DDDD, where KK is the failing ring/publication operation and DDDD is
// only a decoder class or a bounded descriptor index. Raw pointers, DMA
// contents, request metadata, and authority data are deliberately excluded.
const DATA_RING_DIAG_EVENT_WR_DECODE: u32 = 0xD101_0000;
const DATA_RING_DIAG_EVENT_DMA_TRANSLATION: u32 = 0xD102_0000;
const DATA_RING_DIAG_RX_WR_TX_RD_DECODE: u32 = 0xD103_0000;
const DATA_RING_DIAG_RX_DMA_TRANSLATION: u32 = 0xD104_0000;
const DATA_RING_DIAG_HOST_EVENT_RD_PUBLICATION: u32 = 0xD105_0000;
const DATA_RING_DIAG_SHARED_RX_TX_PUBLICATION: u32 = 0xD106_0000;
const DATA_RING_DIAG_EXISTING_EVENT_FAILURE: u32 = 0xD107_0000;
const DATA_RING_DIAG_EXISTING_RX_FAILURE: u32 = 0xD108_0000;
const DATA_RING_DIAG_DECODER_ALL_ONES: u32 = 1;
const DATA_RING_DIAG_DECODER_RESERVED_BITS: u32 = 2;
const DATA_RING_DIAG_DECODER_INDEX_OUT_OF_RANGE: u32 = 3;

fn fixed_hw_failure_trace(
    phase: HwFailurePhase,
    status: HwFailureStatus,
    register: HwFailureRegister,
    register_value: u32,
) -> Option<HwFailureTrace> {
    let mut trace = HwFailureTrace::new(
        SEED_KERNEL_BUILD_ID_V0_1_0,
        HwFailureSubsystem::MarvellWifiPcie,
    );
    let boot_ms = (time::rdtsc() / time::tsc_per_ms().max(1)).min(u64::from(u32::MAX)) as u32;
    trace
        .push_step(HwFailureTraceStep {
            boot_ms,
            phase,
            status,
            register,
            register_value,
        })
        .ok()
        .map(|()| trace)
}

fn queue_fixed_hw_failure_trace(
    phase: HwFailurePhase,
    status: HwFailureStatus,
    register: HwFailureRegister,
    register_value: u32,
) {
    if let Some(trace) = fixed_hw_failure_trace(phase, status, register, register_value) {
        let _ = usb::queue_hw_failure_trace(trace);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreBmeCheckpoint {
    Persisted,
    Pending,
    Failed,
}

fn pre_bme_checkpoint_while_gated(
    address: pci::PciAddress,
    phase: HwFailurePhase,
) -> PreBmeCheckpoint {
    match usb::publication_checkpoint_status() {
        HwFailureTraceFlushStatus::Persisted => PreBmeCheckpoint::Persisted,
        HwFailureTraceFlushStatus::Pending | HwFailureTraceFlushStatus::InFlight => {
            PreBmeCheckpoint::Pending
        }
        HwFailureTraceFlushStatus::Failed => PreBmeCheckpoint::Failed,
        HwFailureTraceFlushStatus::Empty => {
            let command = address.read_u16(0x04);
            let Some(trace) = fixed_hw_failure_trace(
                phase,
                HwFailureStatus::Started,
                HwFailureRegister::MarvellPciCommand,
                u32::from(command),
            ) else {
                return PreBmeCheckpoint::Failed;
            };
            match usb::queue_publication_checkpoint(trace) {
                HwFailureTraceQueueResult::Queued => PreBmeCheckpoint::Pending,
                HwFailureTraceQueueResult::AlreadyQueuedOrAttempted => {
                    match usb::publication_checkpoint_status() {
                        HwFailureTraceFlushStatus::Persisted => PreBmeCheckpoint::Persisted,
                        HwFailureTraceFlushStatus::Pending
                        | HwFailureTraceFlushStatus::InFlight => PreBmeCheckpoint::Pending,
                        HwFailureTraceFlushStatus::Empty | HwFailureTraceFlushStatus::Failed => {
                            PreBmeCheckpoint::Failed
                        }
                    }
                }
                HwFailureTraceQueueResult::InvalidTrace => PreBmeCheckpoint::Failed,
            }
        }
    }
}

fn queue_k2_publication_terminal(
    phase: HwFailurePhase,
    status: HwFailureStatus,
    register: HwFailureRegister,
    register_value: u32,
) {
    queue_fixed_hw_failure_trace(phase, status, register, register_value);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DataRingFailure {
    EventWriteDecode(DevicePointerError),
    EventDmaTranslation { index: usize },
    RxWriteTxReadDecode(DevicePointerError),
    RxDmaTranslation { index: usize },
    HostEventReadPublication(DevicePointerError),
    SharedRxTxPublication(DevicePointerError),
    ExistingEventFailure(Option<EventRingResult>),
    ExistingRxFailure(Option<RxRingResult>),
}

impl DataRingFailure {
    const fn diagnostic_code(self) -> u32 {
        match self {
            Self::EventWriteDecode(error) => {
                DATA_RING_DIAG_EVENT_WR_DECODE | data_ring_decoder_class(error)
            }
            Self::EventDmaTranslation { index } => {
                DATA_RING_DIAG_EVENT_DMA_TRANSLATION | index as u32
            }
            Self::RxWriteTxReadDecode(error) => {
                DATA_RING_DIAG_RX_WR_TX_RD_DECODE | data_ring_decoder_class(error)
            }
            Self::RxDmaTranslation { index } => DATA_RING_DIAG_RX_DMA_TRANSLATION | index as u32,
            Self::HostEventReadPublication(error) => {
                DATA_RING_DIAG_HOST_EVENT_RD_PUBLICATION | data_ring_decoder_class(error)
            }
            Self::SharedRxTxPublication(error) => {
                DATA_RING_DIAG_SHARED_RX_TX_PUBLICATION | data_ring_decoder_class(error)
            }
            Self::ExistingEventFailure(result) => {
                DATA_RING_DIAG_EXISTING_EVENT_FAILURE | event_ring_result_class(result)
            }
            Self::ExistingRxFailure(result) => {
                DATA_RING_DIAG_EXISTING_RX_FAILURE | rx_ring_result_class(result)
            }
        }
    }
}

const fn data_ring_decoder_class(error: DevicePointerError) -> u32 {
    match error {
        DevicePointerError::AllOnes => DATA_RING_DIAG_DECODER_ALL_ONES,
        DevicePointerError::ReservedBits => DATA_RING_DIAG_DECODER_RESERVED_BITS,
        DevicePointerError::IndexOutOfRange => DATA_RING_DIAG_DECODER_INDEX_OUT_OF_RANGE,
    }
}

const fn event_ring_result_class(result: Option<EventRingResult>) -> u32 {
    match result {
        None => 0,
        Some(EventRingResult::Armed) => 1,
        Some(EventRingResult::DmaAddressUnavailable) => 2,
        Some(EventRingResult::BadReadPointer) => 3,
        Some(EventRingResult::BadEventLength) => 4,
        Some(EventRingResult::PointerAdvancedEmptyBuffer) => 5,
        Some(EventRingResult::EventObserved) => 6,
    }
}

const fn rx_ring_result_class(result: Option<RxRingResult>) -> u32 {
    match result {
        None => 0,
        Some(RxRingResult::Armed) => 1,
        Some(RxRingResult::DmaAddressUnavailable) => 2,
        Some(RxRingResult::BadReadPointer) => 3,
        Some(RxRingResult::BadRxLength) => 4,
        Some(RxRingResult::PointerAdvancedEmptyBuffer) => 5,
        Some(RxRingResult::PacketObserved) => 6,
    }
}

fn queue_data_ring_failure_trace(phase: HwFailurePhase, failure: DataRingFailure) {
    queue_fixed_hw_failure_trace(
        phase,
        HwFailureStatus::TransportFault,
        HwFailureRegister::MarvellPublicationStep,
        failure.diagnostic_code(),
    );
}

#[repr(align(64))]
struct DmaBlock([u8; FW_DMA_STAGING_SIZE]);

#[repr(C, align(64))]
struct HwSpecDmaBlock {
    cmd: [u8; HWSPEC_CMD_BUFFER_SIZE],
    rsp: [u8; MWIFIEX_UPLD_SIZE],
}

static mut DMA_BLOCK: DmaBlock = DmaBlock([0; FW_DMA_STAGING_SIZE]);
static mut HWSPEC_DMA_BLOCK: HwSpecDmaBlock = HwSpecDmaBlock {
    cmd: [0; HWSPEC_CMD_BUFFER_SIZE],
    rsp: [0; MWIFIEX_UPLD_SIZE],
};
#[repr(C, align(64))]
struct ScanDmaBlock {
    cmd: [u8; SCAN_CMD_BUFFER_SIZE],
    rsp: [u8; MWIFIEX_UPLD_SIZE],
}

#[repr(C, align(64))]
struct ConnectDmaBlock {
    cmd: [u8; CONNECT_CMD_BUFFER_SIZE],
    rsp: [u8; MWIFIEX_UPLD_SIZE],
}

// Stable MarvellConnectionTimeoutFingerprint payload (v1):
// bits 31..25 = 0x53 format tag, 24..22 = ConnectionStage, 21..14 = exact
// expected Connection HostCmd ID, 13..4 = exact published command length,
// bit 3 = fixed 12-byte request header matched, bit 2 = terminal quiesce plus
// mailbox cleanup verified, and bits 1..0 = response class. Connection command
// IDs fit eight bits and the dedicated command buffer bounds lengths to 512,
// so neither field is truncated. No address, header word, payload, target, or
// credential is encoded.
const CONNECTION_TIMEOUT_FINGERPRINT_TAG: u32 = 0x53;
const CONNECTION_TIMEOUT_FINGERPRINT_TAG_SHIFT: u32 = 25;
const CONNECTION_TIMEOUT_STAGE_SHIFT: u32 = 22;
const CONNECTION_TIMEOUT_COMMAND_SHIFT: u32 = 14;
const CONNECTION_TIMEOUT_COMMAND_LEN_SHIFT: u32 = 4;
const CONNECTION_TIMEOUT_REQUEST_HEADER_MATCH: u32 = 1 << 3;
const CONNECTION_TIMEOUT_CLEANUP_VERIFIED: u32 = 1 << 2;

// Stable, secret-free H25 post-PMK GET_HW_SPEC canary result. The upper
// 24 bits are an exact format tag and the low byte is one terminal outcome.
// No response fields, hardware data, target metadata, DMA address, or secret
// material are persisted. Values outside the enum are reserved.
const POST_PMK_CANARY_RESULT_TAG: u32 = 0xD225_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum PostPmkCanaryOutcome {
    ExpectedCompletion = 0,
    FirmwareResult = 1,
    MalformedOrWrongCompletion = 2,
    TimeoutDoorbellCleared = 3,
    TimeoutDoorbellStillSet = 4,
    MmioOrDoorbellUnavailable = 5,
    StaleHighCompletion = 6,
    HostPublicationFailure = 7,
}

const fn post_pmk_canary_trace_value(outcome: PostPmkCanaryOutcome) -> u32 {
    POST_PMK_CANARY_RESULT_TAG | outcome as u32
}

fn queue_post_pmk_canary_result(outcome: PostPmkCanaryOutcome) {
    queue_fixed_hw_failure_trace(
        HwFailurePhase::HardwareSpec,
        HwFailureStatus::NetworkStateNotGranted,
        HwFailureRegister::MarvellCommandResponseStatus,
        post_pmk_canary_trace_value(outcome),
    );
}

// Stable, secret-free Associate doorbell acknowledgement values. These are
// publication diagnostics, not command completion or DMA-consumption proof.
const ASSOCIATE_DOORBELL_ACK_CLEARED: u32 = 0xD201_0000;
const ASSOCIATE_DOORBELL_ACK_STILL_SET: u32 = 0xD201_0001;
const ASSOCIATE_DOORBELL_ACK_UNAVAILABLE: u32 = 0xD201_0002;

const fn associate_doorbell_ack_trace_value(cpu_int_status: u32) -> u32 {
    if cpu_int_status == u32::MAX {
        ASSOCIATE_DOORBELL_ACK_UNAVAILABLE
    } else if cpu_int_status & CPU_INTR_DOOR_BELL == 0 {
        ASSOCIATE_DOORBELL_ACK_CLEARED
    } else {
        ASSOCIATE_DOORBELL_ACK_STILL_SET
    }
}

fn queue_connection_timeout_trace(
    job: &ConnectionJob,
    host_int_status: u32,
    cleanup_verified: bool,
    response_class: ConnectionTimeoutResponseClass,
    associate_doorbell_ack: Option<u32>,
) {
    let phase = if job.phase == ConnectionStage::Associate {
        HwFailurePhase::Associate
    } else {
        HwFailurePhase::Authenticate
    };
    let expected_command = connection_phase_command(job.phase).unwrap_or(0);
    let fingerprint = connection_timeout_fingerprint(
        job.phase,
        expected_command,
        job.published_command_len,
        job.request_header_matches_expected,
        cleanup_verified,
        response_class,
    );
    let boot_ms = (time::rdtsc() / time::tsc_per_ms().max(1)).min(u64::from(u32::MAX)) as u32;
    let mut trace = HwFailureTrace::new(
        SEED_KERNEL_BUILD_ID_V0_1_0,
        HwFailureSubsystem::MarvellWifiPcie,
    );
    if trace
        .push_step(HwFailureTraceStep {
            boot_ms,
            phase,
            status: HwFailureStatus::Timeout,
            register: HwFailureRegister::MarvellHostInterruptStatus,
            register_value: host_int_status,
        })
        .is_err()
    {
        return;
    }
    if trace
        .push_step(HwFailureTraceStep {
            boot_ms,
            phase,
            status: HwFailureStatus::Timeout,
            register: HwFailureRegister::MarvellConnectionTimeoutFingerprint,
            register_value: fingerprint,
        })
        .is_err()
    {
        return;
    }
    if let Some(register_value) = associate_doorbell_ack {
        if trace
            .push_step(HwFailureTraceStep {
                boot_ms,
                phase,
                status: HwFailureStatus::Timeout,
                register: HwFailureRegister::MarvellPublicationStep,
                register_value,
            })
            .is_err()
        {
            return;
        }
    }
    let _ = usb::queue_hw_failure_trace(trace);
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct EventBufDesc {
    paddr: u64,
    len: u16,
    flags: u16,
}

impl EventBufDesc {
    const EMPTY: Self = Self {
        paddr: 0,
        len: 0,
        flags: 0,
    };
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct RxPfuBufDesc {
    flags: u16,
    offset: u16,
    frag_len: u16,
    len: u16,
    paddr: u64,
    reserved: u32,
}

impl RxPfuBufDesc {
    const EMPTY: Self = Self {
        flags: 0,
        offset: 0,
        frag_len: 0,
        len: 0,
        paddr: 0,
        reserved: 0,
    };
}

#[repr(C, align(64))]
struct EventRingDmaBlock {
    desc: [EventBufDesc; EVENT_RING_COUNT],
    data: [[u8; EVENT_BUFFER_SIZE]; EVENT_RING_COUNT],
}

#[repr(C, align(64))]
struct RxRingDmaBlock {
    desc: [RxPfuBufDesc; RX_RING_COUNT],
    data: [[u8; RX_BUFFER_SIZE]; RX_RING_COUNT],
}

#[repr(C, align(64))]
struct TxRingDmaBlock {
    desc: [RxPfuBufDesc; TX_RING_COUNT],
    data: [[u8; TX_BUFFER_SIZE]; TX_RING_COUNT],
}

static mut SCAN_DMA_BLOCK: ScanDmaBlock = ScanDmaBlock {
    cmd: [0; SCAN_CMD_BUFFER_SIZE],
    rsp: [0; MWIFIEX_UPLD_SIZE],
};
static mut CONNECT_DMA_BLOCK: ConnectDmaBlock = ConnectDmaBlock {
    cmd: [0; CONNECT_CMD_BUFFER_SIZE],
    rsp: [0; MWIFIEX_UPLD_SIZE],
};
static mut EVENT_RING_DMA_BLOCK: EventRingDmaBlock = EventRingDmaBlock {
    desc: [EventBufDesc::EMPTY; EVENT_RING_COUNT],
    data: [[0; EVENT_BUFFER_SIZE]; EVENT_RING_COUNT],
};
static mut RX_RING_DMA_BLOCK: RxRingDmaBlock = RxRingDmaBlock {
    desc: [RxPfuBufDesc::EMPTY; RX_RING_COUNT],
    data: [[0; RX_BUFFER_SIZE]; RX_RING_COUNT],
};
static mut TX_RING_DMA_BLOCK: TxRingDmaBlock = TxRingDmaBlock {
    desc: [RxPfuBufDesc::EMPTY; TX_RING_COUNT],
    data: [[0; TX_BUFFER_SIZE]; TX_RING_COUNT],
};
static BRINGUP: Mutex<FirmwareBringupRuntime> = Mutex::new(FirmwareBringupRuntime::new());
static HWSPEC: Mutex<HwSpecRuntime> = Mutex::new(HwSpecRuntime::new());
static SCAN: Mutex<ScanCmdRuntime> = Mutex::new(ScanCmdRuntime::new());
static EVENT_RING: Mutex<EventRingRuntime> = Mutex::new(EventRingRuntime::new());
static RX_RING: Mutex<RxRingRuntime> = Mutex::new(RxRingRuntime::new());
static TX_RING: Mutex<TxRingRuntime> = Mutex::new(TxRingRuntime::new());
static CONNECTION: Mutex<ConnectionRuntime> = Mutex::new(ConnectionRuntime::new());
static MARVELL_DMA_GATE: Mutex<()> = Mutex::new(());
static RX_TX_POINTER_REGISTER: Mutex<u32> = Mutex::new(0);
static DATA_LINK_READY: AtomicBool = AtomicBool::new(false);
static CONNECTION_REBOOT_REQUIRED: AtomicBool = AtomicBool::new(false);
static HOST_COMMAND_EPOCH: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirmwareDownloadResult {
    Done,
    DrvReadyQuarantined,
    NotPresent,
    FirmwareImageAbsent,
    Bar2Missing,
    Bar2NotMmio,
    MmioMapFailed(&'static str),
    MmioProbeAllOnes,
    DmaAddressUnavailable,
    FirmwareSliceOutOfRange,
    BlockLenOutOfRange,
    CmdSizeTimeout,
    DoorbellAckTimeout,
    FirmwareReadyTimeout,
    TotalTimeout,
    Failed(FwError),
}

impl FirmwareDownloadResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Done => "ready",
            Self::DrvReadyQuarantined => "drv_ready_quarantined",
            Self::NotPresent => "target_not_detected",
            Self::FirmwareImageAbsent => "firmware_absent",
            Self::Bar2Missing => "bar2_missing",
            Self::Bar2NotMmio => "bar2_not_mmio",
            Self::MmioMapFailed(error) => error,
            Self::MmioProbeAllOnes => "mmio_probe_all_ones",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::FirmwareSliceOutOfRange => "firmware_slice_out_of_range",
            Self::BlockLenOutOfRange => "block_len_out_of_range",
            Self::CmdSizeTimeout => "cmd_size_timeout",
            Self::DoorbellAckTimeout => "doorbell_ack_timeout",
            Self::FirmwareReadyTimeout => "firmware_ready_timeout",
            Self::TotalTimeout => "total_timeout",
            Self::Failed(error) => error.reason(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirmwareBringupTriggerResult {
    Started,
    AlreadyRunning,
    AlreadyAttempted,
    Failed(FirmwareDownloadResult),
}

impl FirmwareBringupTriggerResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::AlreadyRunning => "already_running",
            Self::AlreadyAttempted => "already_attempted",
            Self::Failed(result) => result.label(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirmwareStage {
    Idle,
    Preflight,
    DetectOk,
    Bar2MapOk,
    Downloading,
    DoorbellAck,
    PollingReady,
    DrvReadyQuarantined,
    Ready,
    Failed,
}

impl FirmwareStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Preflight => "preflight",
            Self::DetectOk => "detect",
            Self::Bar2MapOk => "bar2_map",
            Self::Downloading => "download",
            Self::DoorbellAck => "doorbell_ack",
            Self::PollingReady => "fw_status",
            Self::DrvReadyQuarantined => "drv_ready_quarantined",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HwSpecStage {
    Idle,
    PcieDescDetails,
    FuncInit,
    GetHwSpec,
    MacControl,
    Ready,
    Failed,
}

impl HwSpecStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::PcieDescDetails => "pcie_desc_details",
            Self::FuncInit => "func_init",
            Self::GetHwSpec => "get_hw_spec",
            Self::MacControl => "mac_control",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanCmdStage {
    Idle,
    Arming,
    WaitCmdDone,
    Done,
    Failed,
}

impl ScanCmdStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Arming => "arming",
            Self::WaitCmdDone => "wait_cmd_done",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HwSpecResult {
    Done,
    DmaAddressUnavailable,
    DataRingUnavailable,
    PciFunctionUnavailable,
    PciCommandEnableFailed,
    FirmwareNotReady,
    CommandBuild(HwSpecCmdError),
    InitCommandBuild(MarvellCmdError),
    CmdDoneTimeout,
    EmptyResponseOnCommandDone,
    Response(HwSpecCmdError),
    InitResponse(MarvellCmdError),
}

impl HwSpecResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Done => "ready",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::DataRingUnavailable => "data_ring_unavailable",
            Self::PciFunctionUnavailable => "pci_function_unavailable",
            Self::PciCommandEnableFailed => "pci_command_enable_failed",
            Self::FirmwareNotReady => "firmware_not_ready",
            Self::CommandBuild(HwSpecCmdError::OutputBufferTooSmall) => "cmd_buffer_too_small",
            Self::CommandBuild(HwSpecCmdError::TooShort) => "cmd_build_too_short",
            Self::CommandBuild(HwSpecCmdError::BadLength) => "cmd_build_bad_length",
            Self::CommandBuild(HwSpecCmdError::BadCommand { .. }) => "cmd_build_bad_command",
            Self::CommandBuild(HwSpecCmdError::BadSequence { .. }) => "cmd_build_bad_sequence",
            Self::CommandBuild(HwSpecCmdError::FwResult { .. }) => "cmd_build_fw_result",
            Self::CommandBuild(HwSpecCmdError::InvalidSequenceContext { .. }) => {
                "cmd_build_invalid_sequence_context"
            }
            Self::InitCommandBuild(_) => "init_command_build",
            Self::CmdDoneTimeout => "cmd_done_timeout",
            Self::EmptyResponseOnCommandDone => "empty_response_on_cmd_done",
            Self::Response(HwSpecCmdError::TooShort) => "response_too_short",
            Self::Response(HwSpecCmdError::BadLength) => "response_bad_length",
            Self::Response(HwSpecCmdError::BadCommand { .. }) => "bad_command",
            Self::Response(HwSpecCmdError::BadSequence { .. }) => "bad_sequence",
            Self::Response(HwSpecCmdError::FwResult { .. }) => "fw_result",
            Self::Response(HwSpecCmdError::InvalidSequenceContext { .. }) => {
                "invalid_sequence_context"
            }
            Self::Response(HwSpecCmdError::OutputBufferTooSmall) => "response_buffer_too_small",
            Self::InitResponse(_) => "init_response",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanCmdResult {
    Done,
    RebootRequired,
    FirmwareNotReady,
    HwSpecNotReady,
    DmaAddressUnavailable,
    CommandBuild(HwSpecCmdError),
    CmdDoneTimeout,
    Response(HwSpecCmdError),
    LiveResultParseFailed,
}

impl ScanCmdResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Done => "live_results_ready",
            Self::RebootRequired => "reboot_required",
            Self::FirmwareNotReady => "firmware_not_ready",
            Self::HwSpecNotReady => "hw_spec_not_ready",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::CommandBuild(HwSpecCmdError::OutputBufferTooSmall) => "cmd_buffer_too_small",
            Self::CommandBuild(HwSpecCmdError::TooShort) => "cmd_build_too_short",
            Self::CommandBuild(HwSpecCmdError::BadLength) => "cmd_build_bad_length",
            Self::CommandBuild(HwSpecCmdError::BadCommand { .. }) => "cmd_build_bad_command",
            Self::CommandBuild(HwSpecCmdError::BadSequence { .. }) => "cmd_build_bad_sequence",
            Self::CommandBuild(HwSpecCmdError::FwResult { .. }) => "cmd_build_fw_result",
            Self::CommandBuild(HwSpecCmdError::InvalidSequenceContext { .. }) => {
                "cmd_build_invalid_sequence_context"
            }
            Self::CmdDoneTimeout => "cmd_done_timeout",
            Self::Response(HwSpecCmdError::TooShort) => "response_too_short",
            Self::Response(HwSpecCmdError::BadLength) => "response_bad_length",
            Self::Response(HwSpecCmdError::BadCommand { .. }) => "bad_command",
            Self::Response(HwSpecCmdError::BadSequence { .. }) => "bad_sequence",
            Self::Response(HwSpecCmdError::FwResult { .. }) => "fw_result",
            Self::Response(HwSpecCmdError::InvalidSequenceContext { .. }) => {
                "invalid_sequence_context"
            }
            Self::Response(HwSpecCmdError::OutputBufferTooSmall) => "response_buffer_too_small",
            Self::LiveResultParseFailed => "live_result_parse_failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventRingStage {
    Idle,
    Armed,
    EventReady,
    Failed,
}

impl EventRingStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Armed => "armed",
            Self::EventReady => "event_ready",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventRingResult {
    Armed,
    DmaAddressUnavailable,
    BadReadPointer,
    BadEventLength,
    PointerAdvancedEmptyBuffer,
    EventObserved,
}

impl EventRingResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Armed => "armed",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::BadReadPointer => "bad_read_pointer",
            Self::BadEventLength => "bad_event_length",
            Self::PointerAdvancedEmptyBuffer => "pointer_advanced_empty_buffer",
            Self::EventObserved => "event_observed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanCmdTriggerResult {
    Started,
    AlreadyRunning,
    Failed(ScanCmdResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionStage {
    Idle,
    SupplicantProfile,
    SupplicantPmk,
    PostPmkHwSpecCanary,
    Associate,
    WaitPortRelease,
    LinkReady,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum ConnectionTimeoutResponseClass {
    UntouchedZero = 0,
    ExpectedHeaderSeen = 1,
    NonemptyMismatch = 2,
    Unavailable = 3,
}

impl ConnectionStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::SupplicantProfile => "supplicant_profile",
            Self::SupplicantPmk => "supplicant_pmk",
            Self::PostPmkHwSpecCanary => "post_pmk_hw_spec_canary",
            Self::Associate => "associate",
            Self::WaitPortRelease => "wait_port_release",
            Self::LinkReady => "link_ready",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionResult {
    LinkReady,
    FirmwareNotReady,
    HwSpecNotReady,
    NoSelectedBss,
    UnsupportedSecurity,
    FirmwareSupplicantUnavailable,
    PassphraseUnavailable,
    SafeRecoveryActionMissing,
    BootPostureDenied,
    EphemeralAuthorityDenied,
    EphemeralAttemptNotFresh,
    EphemeralRevalidationFailed,
    DataRingUnavailable,
    DmaAddressUnavailable,
    Transport(ConnectionTransportError),
    RebootRequired,
    CommandBuild(MarvellCmdError),
    SupplicantBuild(SupplicantError),
    CommandTimeout,
    CommandResponse(MarvellCmdError),
    SupplicantResponse(SupplicantError),
    AssociationRejected(u16),
    PortReleaseTimeout,
    LinkLost(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionTransportError {
    PciFunctionUnavailable,
    PciCommandEnableFailed,
    FirmwareNotReadyAfterEnable,
    MmioUnavailable,
    StaleCommandDone,
    EmptyResponseOnCommandDone,
}

impl ConnectionTransportError {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PciFunctionUnavailable => "pci_function_unavailable",
            Self::PciCommandEnableFailed => "pci_memory_bus_master_not_enabled",
            Self::FirmwareNotReadyAfterEnable => "firmware_not_ready_after_pci_enable",
            Self::MmioUnavailable => "host_int_status_unavailable",
            Self::StaleCommandDone => "stale_cmd_done",
            Self::EmptyResponseOnCommandDone => "empty_response_on_cmd_done",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionResponseErrorClass {
    EmptyResponseOnCommandDone,
    TooShort,
    BadLength,
    BadInterfaceLength,
    BadInterfaceType,
    BadHostCommandLength,
    BadCommand,
    BadSequence,
    FirmwareResult,
    Unexpected,
}

impl ConnectionResponseErrorClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::EmptyResponseOnCommandDone => "empty_response_on_cmd_done",
            Self::TooShort => "too_short",
            Self::BadLength => "bad_length",
            Self::BadInterfaceLength => "bad_intf_len",
            Self::BadInterfaceType => "bad_intf_type",
            Self::BadHostCommandLength => "bad_host_size",
            Self::BadCommand => "bad_command",
            Self::BadSequence => "bad_sequence",
            Self::FirmwareResult => "fw_result",
            Self::Unexpected => "unexpected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionResponseDiagnostic {
    pub error_class: ConnectionResponseErrorClass,
    pub interface_len: u16,
    pub interface_type: u16,
    pub command: u16,
    pub host_command_size: u16,
    pub sequence: u16,
    pub result: u16,
    pub expected_command: u16,
    pub expected_sequence: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionTransportDiagnostic {
    pub pci_vendor_device: u32,
    pub pci_command_before: u16,
    pub pci_command_after: u16,
    pub pci_config_valid: bool,
    pub pre_enable_status: u32,
    pub pre_enable_status_valid: bool,
    pub firmware_status: u32,
    pub firmware_status_valid: bool,
    pub post_enable_status: u32,
    pub post_enable_status_valid: bool,
    pub pre_clear_status: u32,
    pub pre_clear_status_valid: bool,
    pub post_clear_status: u32,
    pub post_clear_status_valid: bool,
    pub program_flush_status: u32,
    pub program_flush_status_valid: bool,
    pub first_poll_status: u32,
    pub first_poll_status_valid: bool,
    pub poll_count: u16,
}

impl ConnectionTransportDiagnostic {
    const fn new() -> Self {
        Self {
            pci_vendor_device: 0,
            pci_command_before: 0,
            pci_command_after: 0,
            pci_config_valid: false,
            pre_enable_status: 0,
            pre_enable_status_valid: false,
            firmware_status: 0,
            firmware_status_valid: false,
            post_enable_status: 0,
            post_enable_status_valid: false,
            pre_clear_status: 0,
            pre_clear_status_valid: false,
            post_clear_status: 0,
            post_clear_status_valid: false,
            program_flush_status: 0,
            program_flush_status_valid: false,
            first_poll_status: 0,
            first_poll_status_valid: false,
            poll_count: 0,
        }
    }
}

impl ConnectionResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::LinkReady => "link_ready",
            Self::FirmwareNotReady => "firmware_not_ready",
            Self::HwSpecNotReady => "hw_spec_not_ready",
            Self::NoSelectedBss => "selected_bss_unavailable",
            Self::UnsupportedSecurity => "security_unsupported",
            Self::FirmwareSupplicantUnavailable => "firmware_supplicant_unavailable",
            Self::PassphraseUnavailable => "passphrase_unavailable",
            Self::SafeRecoveryActionMissing => "safe_recovery_action_missing",
            Self::BootPostureDenied => "boot_posture_denied",
            Self::EphemeralAuthorityDenied => "ephemeral_wifi_authority_denied",
            Self::EphemeralAttemptNotFresh => "ephemeral_wifi_attempt_not_fresh",
            Self::EphemeralRevalidationFailed => "ephemeral_wifi_revalidation_failed",
            Self::DataRingUnavailable => "data_ring_unavailable",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::Transport(error) => error.label(),
            Self::RebootRequired => "reboot_required",
            Self::CommandBuild(_) => "command_build_failed",
            Self::SupplicantBuild(_) => "supplicant_build_failed",
            Self::CommandTimeout => "command_timeout",
            Self::CommandResponse(_) => "command_response_failed",
            Self::SupplicantResponse(_) => "supplicant_response_failed",
            Self::AssociationRejected(_) => "association_rejected",
            Self::PortReleaseTimeout => "port_release_timeout",
            Self::LinkLost(_) => "link_lost",
        }
    }

    pub const fn requires_reboot(self) -> bool {
        matches!(self, Self::Transport(_) | Self::RebootRequired)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionSnapshot {
    pub attempted: bool,
    pub running: bool,
    pub stage: ConnectionStage,
    pub failed_stage: Option<ConnectionStage>,
    pub result: Option<ConnectionResult>,
    pub response_diagnostic: Option<ConnectionResponseDiagnostic>,
    pub transport_diagnostic: Option<ConnectionTransportDiagnostic>,
    pub association_status: Option<u16>,
    pub association_id: Option<u16>,
    pub host_int_status: u32,
}

impl ConnectionSnapshot {
    pub const fn new() -> Self {
        Self {
            attempted: false,
            running: false,
            stage: ConnectionStage::Idle,
            failed_stage: None,
            result: None,
            response_diagnostic: None,
            transport_diagnostic: None,
            association_status: None,
            association_id: None,
            host_int_status: 0,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.stage == ConnectionStage::LinkReady
    }

    pub fn is_failed(&self) -> bool {
        self.stage == ConnectionStage::Failed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionTriggerResult {
    Started,
    AlreadyRunning,
    AlreadyReady,
    Failed(ConnectionResult),
}

impl ConnectionTriggerResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::AlreadyRunning => "already_running",
            Self::AlreadyReady => "already_ready",
            Self::Failed(result) => result.label(),
        }
    }
}

impl ScanCmdTriggerResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::AlreadyRunning => "already_running",
            Self::Failed(result) => result.label(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmwareRegisterSnapshot {
    pub valid: bool,
    pub cmd_size: u32,
    pub fw_status: u32,
    pub drv_ready: u32,
    pub host_int_status: u32,
}

impl FirmwareRegisterSnapshot {
    pub const fn unavailable() -> Self {
        Self {
            valid: false,
            cmd_size: 0,
            fw_status: 0,
            drv_ready: 0,
            host_int_status: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RxRingStage {
    Idle,
    Armed,
    PacketReady,
    Failed,
}

impl RxRingStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Armed => "armed",
            Self::PacketReady => "packet_ready",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RxRingResult {
    Armed,
    DmaAddressUnavailable,
    BadReadPointer,
    BadRxLength,
    PointerAdvancedEmptyBuffer,
    PacketObserved,
}

impl RxRingResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Armed => "armed",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::BadReadPointer => "bad_read_pointer",
            Self::BadRxLength => "bad_rx_length",
            Self::PointerAdvancedEmptyBuffer => "pointer_advanced_empty_buffer",
            Self::PacketObserved => "packet_observed_not_parsed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmwareBringupSnapshot {
    pub attempted: bool,
    pub running: bool,
    pub stage: FirmwareStage,
    pub failed_stage: Option<FirmwareStage>,
    pub result: Option<FirmwareDownloadResult>,
    pub downloaded: usize,
    pub total: usize,
    pub registers: FirmwareRegisterSnapshot,
}

impl FirmwareBringupSnapshot {
    pub const fn new() -> Self {
        Self {
            attempted: false,
            running: false,
            stage: FirmwareStage::Idle,
            failed_stage: None,
            result: None,
            downloaded: 0,
            total: 0,
            registers: FirmwareRegisterSnapshot::unavailable(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.stage == FirmwareStage::Ready
    }

    pub fn is_failed(&self) -> bool {
        self.stage == FirmwareStage::Failed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HwSpecSnapshot {
    pub attempted: bool,
    pub running: bool,
    pub stage: HwSpecStage,
    pub failed_stage: Option<HwSpecStage>,
    pub result: Option<HwSpecResult>,
    pub last_response: Option<marvell_wifi_cmd::MarvellResponseHeader>,
    pub mac: Option<[u8; 6]>,
    pub fw_release: Option<u32>,
    pub fw_cap_info: u32,
    pub key_api_version: Option<(u8, u8)>,
    pub host_int_status: u32,
    pub pci_vendor_device: u32,
    pub pci_command_before: u16,
    pub pci_command_after: u16,
    pub firmware_status: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanCmdSnapshot {
    pub attempted: bool,
    pub running: bool,
    pub stage: ScanCmdStage,
    pub result: Option<ScanCmdResult>,
    pub host_int_status: u32,
    pub command_len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventRingSnapshot {
    pub attempted: bool,
    pub armed: bool,
    pub stage: EventRingStage,
    pub result: Option<EventRingResult>,
    pub host_int_status: u32,
    pub rdptr: u32,
    pub wrptr: u32,
    pub event_len: u16,
    pub event_type: u16,
    pub event_cause: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RxRingSnapshot {
    pub attempted: bool,
    pub armed: bool,
    pub stage: RxRingStage,
    pub result: Option<RxRingResult>,
    pub host_int_status: u32,
    pub rdptr: u32,
    pub wrptr: u32,
    pub rx_len: u16,
    pub rx_type: u16,
}

impl EventRingSnapshot {
    pub const fn new() -> Self {
        Self {
            attempted: false,
            armed: false,
            stage: EventRingStage::Idle,
            result: None,
            host_int_status: 0,
            rdptr: 0,
            wrptr: 0,
            event_len: 0,
            event_type: 0,
            event_cause: 0,
        }
    }

    pub fn is_failed(&self) -> bool {
        self.stage == EventRingStage::Failed
    }

    pub fn has_event(&self) -> bool {
        self.stage == EventRingStage::EventReady
    }
}

impl RxRingSnapshot {
    pub const fn new() -> Self {
        Self {
            attempted: false,
            armed: false,
            stage: RxRingStage::Idle,
            result: None,
            host_int_status: 0,
            rdptr: 0,
            wrptr: 0,
            rx_len: 0,
            rx_type: 0,
        }
    }

    pub fn is_failed(&self) -> bool {
        self.stage == RxRingStage::Failed
    }

    pub fn has_packet(&self) -> bool {
        self.stage == RxRingStage::PacketReady
    }
}

impl ScanCmdSnapshot {
    pub const fn new() -> Self {
        Self {
            attempted: false,
            running: false,
            stage: ScanCmdStage::Idle,
            result: None,
            host_int_status: 0,
            command_len: 0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.stage == ScanCmdStage::Done
    }

    pub fn is_failed(&self) -> bool {
        self.stage == ScanCmdStage::Failed
    }
}

impl HwSpecSnapshot {
    pub const fn new() -> Self {
        Self {
            attempted: false,
            running: false,
            stage: HwSpecStage::Idle,
            failed_stage: None,
            result: None,
            last_response: None,
            mac: None,
            fw_release: None,
            fw_cap_info: 0,
            key_api_version: None,
            host_int_status: 0,
            pci_vendor_device: u32::MAX,
            pci_command_before: u16::MAX,
            pci_command_after: u16::MAX,
            firmware_status: u32::MAX,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.stage == HwSpecStage::Ready
    }

    pub fn is_failed(&self) -> bool {
        self.stage == HwSpecStage::Failed
    }
}

struct FirmwareBringupRuntime {
    snapshot: FirmwareBringupSnapshot,
    job: Option<FirmwareJob>,
    mmio_base: Option<usize>,
}

impl FirmwareBringupRuntime {
    const fn new() -> Self {
        Self {
            snapshot: FirmwareBringupSnapshot::new(),
            job: None,
            mmio_base: None,
        }
    }
}

struct HwSpecRuntime {
    snapshot: HwSpecSnapshot,
    job: Option<HwSpecJob>,
}

impl HwSpecRuntime {
    const fn new() -> Self {
        Self {
            snapshot: HwSpecSnapshot::new(),
            job: None,
        }
    }
}

struct ScanCmdRuntime {
    snapshot: ScanCmdSnapshot,
    job: Option<ScanCmdJob>,
    next_seq: u16,
}

struct ConnectionRuntime {
    snapshot: ConnectionSnapshot,
    job: Option<ConnectionJob>,
    ready_target: Option<wifi::ScannedNetwork>,
    sequence_allocator: SingleBssHostCmdSequenceAllocator,
}

struct EventRingRuntime {
    snapshot: EventRingSnapshot,
    mmio_base: Option<usize>,
    rdptr: u32,
    arm_failure: Option<DataRingFailure>,
}

struct RxRingRuntime {
    snapshot: RxRingSnapshot,
    mmio_base: Option<usize>,
    rdptr: u32,
    arm_failure: Option<DataRingFailure>,
}

struct TxRingRuntime {
    armed: bool,
    mmio_base: Option<usize>,
    wrptr: u32,
    rdptr: u32,
}

#[derive(Clone, Copy)]
enum SharedRxTxPointerUpdate {
    Initialize { rx_rdptr: u32, tx_wrptr: u32 },
    RxRead(u32),
    TxWrite(u32),
}

#[derive(Clone, Copy)]
enum DeferredNetworkAction {
    None,
    Attach([u8; 6]),
    Detach,
}

impl EventRingRuntime {
    const fn new() -> Self {
        Self {
            snapshot: EventRingSnapshot::new(),
            mmio_base: None,
            rdptr: EVENT_ROLLOVER_IND,
            arm_failure: None,
        }
    }
}

impl RxRingRuntime {
    const fn new() -> Self {
        Self {
            snapshot: RxRingSnapshot::new(),
            mmio_base: None,
            rdptr: RX_ROLLOVER_IND,
            arm_failure: None,
        }
    }
}

impl TxRingRuntime {
    const fn new() -> Self {
        Self {
            armed: false,
            mmio_base: None,
            wrptr: 0,
            rdptr: 0,
        }
    }
}

pub struct RxPacket {
    len: usize,
    bytes: [u8; MAX_ETHERNET_FRAME_SIZE],
}

impl RxPacket {
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes[..self.len]
    }
}

impl ScanCmdRuntime {
    const fn new() -> Self {
        Self {
            snapshot: ScanCmdSnapshot::new(),
            job: None,
            next_seq: 1,
        }
    }
}

impl ConnectionRuntime {
    const fn new() -> Self {
        Self {
            snapshot: ConnectionSnapshot::new(),
            job: None,
            ready_target: None,
            sequence_allocator: SingleBssHostCmdSequenceAllocator::new(),
        }
    }
}

struct FirmwareJob {
    download: FirmwareDownload<'static>,
    pci_address: pci::PciAddress,
    mmio_base: usize,
    block_dma_phys: u64,
    phase: FwPhase,
    phase_started_tsc: u64,
    started_tsc: u64,
    firmware_len: usize,
    pre_bme_checkpoint_confirmed: bool,
}

struct HwSpecJob {
    pci_address: pci::PciAddress,
    mmio_base: usize,
    cmd_dma_phys: u64,
    rsp_dma_phys: u64,
    phase_started_tsc: u64,
    waiting: bool,
    seq: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PciCommandEnableError {
    FunctionUnavailable,
    EnableFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PciCommandEnableOutcome {
    vendor_device: u32,
    command_before: u16,
    command_after: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VerifiedQuiesceError {
    FunctionUnavailable,
    CommandChanged,
    BusMasterStillEnabled,
}

struct VerifiedOff(());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeDmaPlanError {
    AddressUnavailable,
    AddressOverflow,
    TooManyPages,
    TranslationRejected,
    PublishedAddressMismatch,
    SpanRejected,
    ForeignRegionsUnavailable,
    KernelOwnershipMismatch,
    RegionOverlap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostCommandPublishError {
    Quiesce(VerifiedQuiesceError),
    MmioUnavailable,
    StaleCommandDone,
    CheckpointNotPersisted,
    Pci(PciCommandEnableError),
    PublicationOrder,
}

struct HostCommandEpoch {
    verified_off: VerifiedOff,
    epoch: u64,
    pre_clear_status: u32,
    post_clear_status: u32,
}

#[derive(Clone, Copy)]
struct HostCommandPublication {
    pci: PciCommandEnableOutcome,
    program_flush_status: u32,
    first_poll_status: u32,
}

struct ScanCmdJob {
    pci_address: pci::PciAddress,
    mmio_base: usize,
    cmd_dma_phys: u64,
    rsp_dma_phys: u64,
    started_tsc: u64,
    seq: u16,
}

struct ConnectionJob {
    pci_address: pci::PciAddress,
    mmio_base: usize,
    cmd_dma_phys: u64,
    rsp_dma_phys: u64,
    phase: ConnectionStage,
    waiting: bool,
    cmd_done_low_baseline: bool,
    published_command_len: u16,
    request_header_matches_expected: bool,
    phase_started_tsc: u64,
    transport_diagnostic: ConnectionTransportDiagnostic,
    seq: u8,
    target: wifi::ScannedNetwork,
    secret_source: ConnectionSecretSource,
}

/// Mutually exclusive credential sources for one connection job. In
/// particular, an ephemeral job can never fall through to Vault or legacy RAM.
enum ConnectionSecretSource {
    Ordinary,
    SafeVault(Option<secret_vault::ExplicitSafeWifiReconnect>),
    EphemeralPhysical {
        pending: Option<secret_vault::EphemeralPhysicalWifiUse>,
        receipt: Option<secret_vault::EphemeralPhysicalWifiReceipt>,
    },
}

impl ConnectionSecretSource {
    const fn is_ephemeral(&self) -> bool {
        matches!(self, Self::EphemeralPhysical { .. })
    }
}

#[derive(Clone, Copy)]
struct ParsedEvent {
    len: u16,
    event_type: u16,
    cause: u32,
}

#[cfg(marvell_fw_present)]
pub fn firmware_image() -> &'static [u8] {
    include_bytes!(env!("MARVELL_FW_PATH"))
}

#[cfg(not(marvell_fw_present))]
pub fn firmware_image() -> &'static [u8] {
    &[]
}

pub fn snapshot() -> FirmwareBringupSnapshot {
    BRINGUP.lock().snapshot
}

pub fn hw_spec_snapshot() -> HwSpecSnapshot {
    HWSPEC.lock().snapshot
}

pub fn scan_cmd_snapshot() -> ScanCmdSnapshot {
    SCAN.lock().snapshot
}

pub fn event_ring_snapshot() -> EventRingSnapshot {
    EVENT_RING.lock().snapshot
}

pub fn rx_ring_snapshot() -> RxRingSnapshot {
    RX_RING.lock().snapshot
}

pub fn connection_snapshot() -> ConnectionSnapshot {
    CONNECTION.lock().snapshot
}

pub fn data_link_ready() -> bool {
    DATA_LINK_READY.load(Ordering::Acquire)
}

pub fn connection_reboot_required() -> bool {
    CONNECTION_REBOOT_REQUIRED.load(Ordering::Acquire)
}

fn latch_invalid_ring_pointer_while_gated(register: &'static str, raw: u32) {
    DATA_LINK_READY.store(false, Ordering::Release);
    let first_fault = !CONNECTION_REBOOT_REQUIRED.swap(true, Ordering::AcqRel);
    compiler_fence(Ordering::SeqCst);
    if first_fault {
        serial::write_fmt(format_args!(
            "marvell wifi: invalid {} pointer 0x{:08x}; reboot required\r\n",
            register, raw
        ));
    }
}

fn poison_dma_epoch_while_gated(reason: &'static str) {
    DATA_LINK_READY.store(false, Ordering::Release);
    let first_fault = !CONNECTION_REBOOT_REQUIRED.swap(true, Ordering::AcqRel);
    compiler_fence(Ordering::SeqCst);
    if first_fault {
        serial::write_fmt(format_args!(
            "marvell wifi: DMA epoch poisoned ({reason}); reboot required\r\n"
        ));
    }
}

fn verified_quiesce_while_gated(
    address: pci::PciAddress,
) -> Result<VerifiedOff, VerifiedQuiesceError> {
    DATA_LINK_READY.store(false, Ordering::Release);
    let vendor_device = address.read_u32(0x00);
    let command = address.read_u16(0x04);
    if vendor_device == u32::MAX || command == u16::MAX {
        poison_dma_epoch_while_gated("PCI function unavailable during BME-off verification");
        return Err(VerifiedQuiesceError::FunctionUnavailable);
    }
    let requested = (command & !0x0004) | (1 << 10);
    let readback = match address.write_command_u16_checked(vendor_device, command, requested) {
        pci::PciCommandWriteResult::Written { readback } => readback,
        pci::PciCommandWriteResult::DeviceUnavailable => {
            poison_dma_epoch_while_gated("PCI function vanished during BME-off verification");
            return Err(VerifiedQuiesceError::FunctionUnavailable);
        }
        pci::PciCommandWriteResult::CommandChanged { .. } => {
            poison_dma_epoch_while_gated("PCI command changed during BME-off verification");
            return Err(VerifiedQuiesceError::CommandChanged);
        }
    };
    if readback & 0x0004 != 0 {
        poison_dma_epoch_while_gated("PCI BME remained enabled after checked write");
        return Err(VerifiedQuiesceError::BusMasterStillEnabled);
    }
    compiler_fence(Ordering::SeqCst);
    Ok(VerifiedOff(()))
}

fn enable_memory_space_while_verified_off(
    address: pci::PciAddress,
    _verified_off: VerifiedOff,
) -> Result<VerifiedOff, VerifiedQuiesceError> {
    let vendor_device = address.read_u32(0x00);
    let command = address.read_u16(0x04);
    if vendor_device == u32::MAX || command == u16::MAX {
        poison_dma_epoch_while_gated("PCI function unavailable before MMIO mapping");
        return Err(VerifiedQuiesceError::FunctionUnavailable);
    }
    let requested = (command | 0x0002 | (1 << 10)) & !0x0004;
    let readback = match address.write_command_u16_checked(vendor_device, command, requested) {
        pci::PciCommandWriteResult::Written { readback } => readback,
        pci::PciCommandWriteResult::DeviceUnavailable => {
            poison_dma_epoch_while_gated("PCI function vanished before MMIO mapping");
            return Err(VerifiedQuiesceError::FunctionUnavailable);
        }
        pci::PciCommandWriteResult::CommandChanged { .. } => {
            poison_dma_epoch_while_gated("PCI command changed before MMIO mapping");
            return Err(VerifiedQuiesceError::CommandChanged);
        }
    };
    if readback & 0x0004 != 0 || readback & 0x0002 == 0 {
        poison_dma_epoch_while_gated("PCI memory enable lost verified BME-off state");
        return Err(VerifiedQuiesceError::BusMasterStillEnabled);
    }
    compiler_fence(Ordering::SeqCst);
    Ok(VerifiedOff(()))
}

fn validate_runtime_dma_region(
    virtual_start: *const u8,
    len: usize,
    published_start: Option<u64>,
    kind: MarvellDmaRegionKind,
) -> Result<MarvellDmaRegion, RuntimeDmaPlanError> {
    let virtual_ptr = virtual_start;
    let virtual_start = virtual_ptr as u64;
    let len = u64::try_from(len).map_err(|_| RuntimeDmaPlanError::AddressOverflow)?;
    let virtual_end = virtual_start
        .checked_add(len)
        .ok_or(RuntimeDmaPlanError::AddressOverflow)?;
    let first_page = virtual_start & !(DMA_PAGE_SIZE - 1);
    let last_page = (virtual_end - 1) & !(DMA_PAGE_SIZE - 1);
    let page_count = usize::try_from(((last_page - first_page) / DMA_PAGE_SIZE) + 1)
        .map_err(|_| RuntimeDmaPlanError::TooManyPages)?;
    if page_count > MAX_DMA_TRANSLATION_PAGES {
        return Err(RuntimeDmaPlanError::TooManyPages);
    }

    let empty = PageTranslation {
        virtual_page_start: 0,
        physical_page_start: 0,
    };
    let mut pages = [empty; MAX_DMA_TRANSLATION_PAGES];
    let mut index = 0usize;
    while index < page_count {
        let virtual_page_start = first_page
            .checked_add((index as u64) * DMA_PAGE_SIZE)
            .ok_or(RuntimeDmaPlanError::AddressOverflow)?;
        let physical_page_start = memory::virt_to_phys(virtual_page_start as *const u8)
            .ok_or(RuntimeDmaPlanError::AddressUnavailable)?;
        pages[index] = PageTranslation {
            virtual_page_start,
            physical_page_start,
        };
        index += 1;
    }
    let translated = validate_contiguous_translation(
        virtual_start,
        len,
        DMA_PAGE_SIZE,
        DMA_ADDRESS_BITS,
        &pages[..page_count],
    )
    .map_err(|_| RuntimeDmaPlanError::TranslationRejected)?;
    let authoritative = memory::physical_span_for_virtual_range(virtual_ptr, len)
        .ok_or(RuntimeDmaPlanError::TranslationRejected)?;
    if authoritative.start() != translated.physical_span.start()
        || authoritative.end_exclusive() != translated.physical_span.end_exclusive()
    {
        return Err(RuntimeDmaPlanError::TranslationRejected);
    }
    if published_start != Some(translated.physical_span.start()) {
        return Err(RuntimeDmaPlanError::PublishedAddressMismatch);
    }
    let span = DmaSpan::new(
        translated.physical_span.start(),
        len,
        DMA_REQUIRED_ALIGNMENT,
        DMA_ADDRESS_BITS,
    )
    .map_err(|_| RuntimeDmaPlanError::SpanRejected)?;
    Ok(MarvellDmaRegion { kind, span })
}

fn validate_runtime_dma_plan_while_gated() -> Result<(), RuntimeDmaPlanError> {
    let regions: [MarvellDmaRegion; MARVELL_DMA_REGION_COUNT] = [
        validate_runtime_dma_region(
            dma_block_ptr().cast_const(),
            core::mem::size_of::<DmaBlock>(),
            dma_block_phys(),
            MarvellDmaRegionKind::CommandBuffer,
        )?,
        validate_runtime_dma_region(
            hw_spec_cmd_ptr().cast_const(),
            HWSPEC_CMD_BUFFER_SIZE,
            hw_spec_cmd_phys(),
            MarvellDmaRegionKind::CommandBuffer,
        )?,
        validate_runtime_dma_region(
            hw_spec_rsp_ptr().cast_const(),
            MWIFIEX_UPLD_SIZE,
            hw_spec_rsp_phys(),
            MarvellDmaRegionKind::ResponseBuffer,
        )?,
        validate_runtime_dma_region(
            scan_cmd_ptr().cast_const(),
            SCAN_CMD_BUFFER_SIZE,
            scan_cmd_phys(),
            MarvellDmaRegionKind::CommandBuffer,
        )?,
        validate_runtime_dma_region(
            scan_rsp_ptr().cast_const(),
            MWIFIEX_UPLD_SIZE,
            scan_rsp_phys(),
            MarvellDmaRegionKind::ResponseBuffer,
        )?,
        validate_runtime_dma_region(
            connect_cmd_ptr().cast_const(),
            CONNECT_CMD_BUFFER_SIZE,
            connect_cmd_phys(),
            MarvellDmaRegionKind::CommandBuffer,
        )?,
        validate_runtime_dma_region(
            connect_rsp_ptr().cast_const(),
            MWIFIEX_UPLD_SIZE,
            connect_rsp_phys(),
            MarvellDmaRegionKind::ResponseBuffer,
        )?,
        validate_runtime_dma_region(
            event_desc_ptr(0).cast::<u8>().cast_const(),
            core::mem::size_of::<EventRingDmaBlock>(),
            event_desc_phys(),
            MarvellDmaRegionKind::EventRing,
        )?,
        validate_runtime_dma_region(
            rx_desc_ptr(0).cast::<u8>().cast_const(),
            core::mem::size_of::<RxRingDmaBlock>(),
            rx_desc_phys(),
            MarvellDmaRegionKind::RxRing,
        )?,
        validate_runtime_dma_region(
            tx_desc_ptr(0).cast::<u8>().cast_const(),
            core::mem::size_of::<TxRingDmaBlock>(),
            tx_desc_phys(),
            MarvellDmaRegionKind::TxRing,
        )?,
    ];

    authoritative_foreign_dma_regions_while_gated(&regions)
}

fn validate_foreign_physical_span(
    regions: &[MarvellDmaRegion; MARVELL_DMA_REGION_COUNT],
    kind: ForeignRegionKind,
    physical: memory::PhysicalSpan,
) -> Result<(), RuntimeDmaPlanError> {
    let span = DmaSpan::new(physical.start(), physical.len(), 1, DMA_ADDRESS_BITS)
        .map_err(|_| RuntimeDmaPlanError::SpanRejected)?;
    validate_non_overlapping_regions(regions, &[ForeignRegion { kind, span }])
        .map_err(|_| RuntimeDmaPlanError::RegionOverlap)
}

fn validate_kernel_remainder_after_marvell_owned(
    regions: &[MarvellDmaRegion; MARVELL_DMA_REGION_COUNT],
    kernel: memory::PhysicalSpan,
) -> Result<(), RuntimeDmaPlanError> {
    let mut ordered = *regions;
    ordered.sort_unstable_by_key(|region| region.span.start());
    let mut cursor = kernel.start();

    for region in ordered {
        if region.span.start() < cursor || region.span.end_exclusive() > kernel.end_exclusive() {
            return Err(RuntimeDmaPlanError::KernelOwnershipMismatch);
        }
        if region.span.start() > cursor {
            let remainder =
                memory::PhysicalSpan::from_start_len(cursor, region.span.start() - cursor)
                    .ok_or(RuntimeDmaPlanError::KernelOwnershipMismatch)?;
            validate_foreign_physical_span(regions, ForeignRegionKind::Kernel, remainder)?;
        }
        cursor = region.span.end_exclusive();
    }

    if cursor < kernel.end_exclusive() {
        let remainder =
            memory::PhysicalSpan::from_start_len(cursor, kernel.end_exclusive() - cursor)
                .ok_or(RuntimeDmaPlanError::KernelOwnershipMismatch)?;
        validate_foreign_physical_span(regions, ForeignRegionKind::Kernel, remainder)?;
    }
    Ok(())
}

fn authoritative_foreign_dma_regions_while_gated(
    regions: &[MarvellDmaRegion; MARVELL_DMA_REGION_COUNT],
) -> Result<(), RuntimeDmaPlanError> {
    let kernel = memory::kernel_image_physical_span()
        .ok_or(RuntimeDmaPlanError::ForeignRegionsUnavailable)?;
    validate_kernel_remainder_after_marvell_owned(regions, kernel)?;

    let heap_span = heap::physical_span().ok_or(RuntimeDmaPlanError::ForeignRegionsUnavailable)?;
    validate_foreign_physical_span(regions, ForeignRegionKind::Heap, heap_span)?;

    let xhci_spans =
        usb::xhci_dma_physical_spans().ok_or(RuntimeDmaPlanError::ForeignRegionsUnavailable)?;
    for span in xhci_spans {
        validate_foreign_physical_span(regions, ForeignRegionKind::Xhci, span)?;
    }

    let usb_reclog_spans = usb::usb_msc_reclog_staging_physical_spans()
        .ok_or(RuntimeDmaPlanError::ForeignRegionsUnavailable)?;
    for span in usb_reclog_spans {
        validate_foreign_physical_span(regions, ForeignRegionKind::Reclog, span)?;
    }

    let ahci_reclog_spans = ahci::reclog_staging_physical_spans()
        .ok_or(RuntimeDmaPlanError::ForeignRegionsUnavailable)?;
    for span in ahci_reclog_spans {
        validate_foreign_physical_span(regions, ForeignRegionKind::Reclog, span)?;
    }
    Ok(())
}

fn cleanup_host_command_mailbox_after_verified_off(
    mmio: *mut u8,
    _verified_off: &VerifiedOff,
) -> Result<u32, HostCommandPublishError> {
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio, CMD_SIZE, 0);
    write_reg(mmio, CMD_ADDR_LO, 0);
    write_reg(mmio, CMD_ADDR_HI, 0);
    write_reg(mmio, CMDRSP_ADDR_LO, 0);
    write_reg(mmio, CMDRSP_ADDR_HI, 0);
    compiler_fence(Ordering::SeqCst);
    let flush = read_reg(mmio, PCIE_HOST_INT_STATUS);
    if flush == u32::MAX {
        poison_dma_epoch_while_gated("mailbox cleanup flush unavailable");
        return Err(HostCommandPublishError::MmioUnavailable);
    }
    Ok(flush)
}

fn begin_host_command_epoch_while_gated(
    address: pci::PciAddress,
    mmio: *mut u8,
) -> Result<HostCommandEpoch, HostCommandPublishError> {
    let verified_off =
        verified_quiesce_while_gated(address).map_err(HostCommandPublishError::Quiesce)?;
    if validate_runtime_dma_plan_while_gated().is_err() {
        poison_dma_epoch_while_gated("runtime DMA plan validation failed");
        return Err(HostCommandPublishError::PublicationOrder);
    }
    write_reg(mmio, PCIE_HOST_INT_MASK, 0);
    write_reg(mmio, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
    let pre_clear_status = cleanup_host_command_mailbox_after_verified_off(mmio, &verified_off)?;
    if pre_clear_status != 0 {
        write_reg(mmio, PCIE_HOST_INT_STATUS, !pre_clear_status);
    }
    let mut post_clear_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    for _ in 1..CONNECTION_CMD_DONE_CLEAR_POLLS {
        if marvell_wifi_cmd::host_cmd_done_low_after_clear(post_clear_status, HOST_INTR_CMD_DONE) {
            break;
        }
        delay_us(SHORT_POLL_DELAY_US);
        post_clear_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    }
    if post_clear_status == u32::MAX {
        poison_dma_epoch_while_gated("HostCmd baseline status unavailable");
        return Err(HostCommandPublishError::MmioUnavailable);
    }
    if !marvell_wifi_cmd::host_cmd_done_low_after_clear(post_clear_status, HOST_INTR_CMD_DONE) {
        return Err(HostCommandPublishError::StaleCommandDone);
    }
    Ok(HostCommandEpoch {
        verified_off,
        epoch: HOST_COMMAND_EPOCH.fetch_add(1, Ordering::AcqRel),
        pre_clear_status,
        post_clear_status,
    })
}

fn publish_host_command_while_gated(
    address: pci::PciAddress,
    mmio: *mut u8,
    cmd_dma_phys: u64,
    rsp_dma_phys: u64,
    command_len: usize,
    epoch: HostCommandEpoch,
) -> Result<HostCommandPublication, HostCommandPublishError> {
    let HostCommandEpoch {
        verified_off: _verified_off,
        epoch,
        ..
    } = epoch;
    let mut model = PublicationModel::new();
    model
        .begin_epoch(epoch)
        .map_err(|_| HostCommandPublishError::PublicationOrder)?;
    write_reg(mmio, CMDRSP_ADDR_LO, (rsp_dma_phys & 0xffff_ffff) as u32);
    model
        .publish(epoch, PublicationStep::ResponseLow)
        .map_err(|_| HostCommandPublishError::PublicationOrder)?;
    write_reg(mmio, CMDRSP_ADDR_HI, (rsp_dma_phys >> 32) as u32);
    model
        .publish(epoch, PublicationStep::ResponseHigh)
        .map_err(|_| HostCommandPublishError::PublicationOrder)?;
    write_reg(mmio, CMD_ADDR_LO, (cmd_dma_phys & 0xffff_ffff) as u32);
    model
        .publish(epoch, PublicationStep::CommandLow)
        .map_err(|_| HostCommandPublishError::PublicationOrder)?;
    write_reg(mmio, CMD_ADDR_HI, (cmd_dma_phys >> 32) as u32);
    model
        .publish(epoch, PublicationStep::CommandHigh)
        .map_err(|_| HostCommandPublishError::PublicationOrder)?;
    write_reg(mmio, CMD_SIZE, command_len as u32);
    model
        .publish(epoch, PublicationStep::CommandSize)
        .and_then(|_| model.publish(epoch, PublicationStep::RingsPublished))
        .map_err(|_| HostCommandPublishError::PublicationOrder)?;
    compiler_fence(Ordering::SeqCst);
    let program_flush_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    if program_flush_status == u32::MAX {
        poison_dma_epoch_while_gated("HostCmd publication flush unavailable");
        return Err(HostCommandPublishError::MmioUnavailable);
    }
    if program_flush_status & HOST_INTR_CMD_DONE != 0 {
        return Err(HostCommandPublishError::StaleCommandDone);
    }

    if usb::publication_checkpoint_status() != HwFailureTraceFlushStatus::Persisted {
        queue_k2_publication_terminal(
            HwFailurePhase::HardwareSpec,
            HwFailureStatus::K2CheckpointPersistFailed,
            HwFailureRegister::MarvellPublicationStep,
            K2_PUBLICATION_STEP_CHECKPOINT_FLUSH,
        );
        return Err(HostCommandPublishError::CheckpointNotPersisted);
    }
    if model.enable_bme().is_err() {
        queue_k2_publication_terminal(
            HwFailurePhase::HardwareSpec,
            HwFailureStatus::K2PublicationRejected,
            HwFailureRegister::MarvellPublicationStep,
            K2_PUBLICATION_STEP_MODEL_ENABLE,
        );
        let _ = verified_quiesce_while_gated(address);
        return Err(HostCommandPublishError::PublicationOrder);
    }
    let vendor_device = address.read_u32(0x00);
    let command_before = address.read_u16(0x04);
    let pci = match ensure_pci_memory_bus_master(address, vendor_device, command_before) {
        Ok(pci) => pci,
        Err(error) => {
            let command_after = address.read_u16(0x04);
            queue_k2_publication_terminal(
                HwFailurePhase::HardwareSpec,
                HwFailureStatus::K2PciCommandRejected,
                HwFailureRegister::MarvellPciCommand,
                u32::from(command_before) | (u32::from(command_after) << 16),
            );
            let _ = verified_quiesce_while_gated(address);
            return Err(HostCommandPublishError::Pci(error));
        }
    };
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio, PCIE_CPU_INT_EVENT, CPU_INTR_DOOR_BELL);
    if model.publish(epoch, PublicationStep::Doorbell).is_err() {
        queue_k2_publication_terminal(
            HwFailurePhase::HardwareSpec,
            HwFailureStatus::K2PublicationRejected,
            HwFailureRegister::MarvellPublicationStep,
            K2_PUBLICATION_STEP_DOORBELL,
        );
        let _ = verified_quiesce_while_gated(address);
        return Err(HostCommandPublishError::PublicationOrder);
    }
    compiler_fence(Ordering::SeqCst);
    let first_poll_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    if first_poll_status == u32::MAX {
        let _ = verified_quiesce_while_gated(address);
        return Err(HostCommandPublishError::MmioUnavailable);
    }
    Ok(HostCommandPublication {
        pci,
        program_flush_status,
        first_poll_status,
    })
}

fn terminal_quiesce_and_cleanup_while_gated(address: pci::PciAddress, mmio: *mut u8) -> bool {
    let Ok(verified_off) = verified_quiesce_while_gated(address) else {
        return false;
    };
    cleanup_host_command_mailbox_after_verified_off(mmio, &verified_off).is_ok()
}

fn activate_validated_data_dma_while_gated(address: pci::PciAddress) -> bool {
    let Ok(_verified_off) = verified_quiesce_while_gated(address) else {
        poison_dma_epoch_while_gated("data DMA activation could not verify BME-off");
        return false;
    };
    if validate_runtime_dma_plan_while_gated().is_err() {
        poison_dma_epoch_while_gated("data DMA activation plan validation failed");
        return false;
    }
    if usb::publication_checkpoint_status() != HwFailureTraceFlushStatus::Persisted {
        queue_k2_publication_terminal(
            HwFailurePhase::FirmwareTransport,
            HwFailureStatus::K2CheckpointPersistFailed,
            HwFailureRegister::MarvellPublicationStep,
            K2_PUBLICATION_STEP_CHECKPOINT_FLUSH,
        );
        return false;
    }
    let vendor_device = address.read_u32(0x00);
    let command_before = address.read_u16(0x04);
    match ensure_pci_memory_bus_master(address, vendor_device, command_before) {
        Ok(_) => true,
        Err(_) => {
            let command_after = address.read_u16(0x04);
            queue_k2_publication_terminal(
                HwFailurePhase::FirmwareTransport,
                HwFailureStatus::K2PciCommandRejected,
                HwFailureRegister::MarvellPciCommand,
                u32::from(command_before) | (u32::from(command_after) << 16),
            );
            let _ = verified_quiesce_while_gated(address);
            poison_dma_epoch_while_gated("checked data DMA activation failed");
            false
        }
    }
}

fn quarantine_invalid_pointer_after_ring_unlock_while_gated(
    mmio: *mut u8,
    register: &'static str,
    raw: u32,
) {
    latch_invalid_ring_pointer_while_gated(register, raw);
    let Some(address) = wifi::snapshot().address else {
        poison_dma_epoch_while_gated("PCI address unavailable after pointer poison");
        return;
    };
    let _ = terminal_quiesce_and_cleanup_while_gated(address, mmio);
}

fn apply_deferred_network_action(action: DeferredNetworkAction) {
    match action {
        DeferredNetworkAction::None => {}
        DeferredNetworkAction::Detach => net::detach_wifi(),
        DeferredNetworkAction::Attach(mac) => {
            if connection_reboot_required() || !data_link_ready() {
                net::detach_wifi();
                return;
            }
            net::attach_wifi(mac);
            if connection_reboot_required() || !data_link_ready() {
                net::detach_wifi();
            }
        }
    }
}

pub fn start_association() -> ConnectionTriggerResult {
    match crate::agent_protocol::boot_control::current_boot_posture() {
        BootPosture::Normal | BootPosture::Probation => {}
        BootPosture::Safe => {
            return fail_connection_start(ConnectionResult::SafeRecoveryActionMissing)
        }
        BootPosture::PersistenceUnavailable => {
            return fail_connection_start(ConnectionResult::BootPostureDenied)
        }
    }
    let Some(target) = wifi::association_target() else {
        return fail_connection_start(ConnectionResult::NoSelectedBss);
    };
    start_association_inner(target, ConnectionSecretSource::Ordinary)
}

/// The only SAFE association entrypoint. Its caller is the trusted physical
/// Genesis flow; serial and legacy setup continue through `start_association`.
pub fn start_association_from_physical_genesis() -> ConnectionTriggerResult {
    match crate::agent_protocol::boot_control::current_boot_posture() {
        BootPosture::Normal | BootPosture::Probation => start_association(),
        BootPosture::Safe => {
            let Some(target) = wifi::association_target() else {
                return fail_connection_start(ConnectionResult::NoSelectedBss);
            };
            let safe_reconnect = match secret_vault::begin_explicit_safe_wifi_reconnect(
                target.ssid.as_bytes(),
                target.bssid,
            ) {
                Ok(authority) => authority,
                Err(denied) => {
                    serial::write_fmt(format_args!(
                        "VAULT_WIFI_SAFE_RECONNECT_REJECTED reason={}\r\n",
                        denied.wifi_use_reason()
                    ));
                    return fail_connection_start(ConnectionResult::SafeRecoveryActionMissing);
                }
            };
            start_association_inner(
                target,
                ConnectionSecretSource::SafeVault(Some(safe_reconnect)),
            )
        }
        BootPosture::PersistenceUnavailable => {
            fail_connection_start(ConnectionResult::BootPostureDenied)
        }
    }
}

/// The only persistence-unavailable association entrypoint. Its unforgeable
/// argument can originate only in a fresh physical SecureOverlay submission.
pub(crate) fn start_ephemeral_association_from_physical_genesis(
    authority: secret_vault::EphemeralPhysicalWifiUse,
) -> ConnectionTriggerResult {
    if crate::agent_protocol::boot_control::current_boot_posture()
        != BootPosture::PersistenceUnavailable
    {
        return ConnectionTriggerResult::Failed(ConnectionResult::EphemeralAuthorityDenied);
    }
    let Some(target) = wifi::association_target() else {
        return ConnectionTriggerResult::Failed(ConnectionResult::NoSelectedBss);
    };
    if secret_vault::revalidate_ephemeral_physical_wifi_use(&authority, target).is_err() {
        return ConnectionTriggerResult::Failed(ConnectionResult::EphemeralAuthorityDenied);
    }
    start_association_inner(
        target,
        ConnectionSecretSource::EphemeralPhysical {
            pending: Some(authority),
            receipt: None,
        },
    )
}

fn start_association_inner(
    target: wifi::ScannedNetwork,
    secret_source: ConnectionSecretSource,
) -> ConnectionTriggerResult {
    if connection_reboot_required() {
        return fail_connection_start(ConnectionResult::RebootRequired);
    }
    let ephemeral = secret_source.is_ephemeral();
    let replace_ready = {
        let runtime = CONNECTION.lock();
        if ephemeral && (runtime.snapshot.is_ready() || runtime.job.is_some()) {
            return ConnectionTriggerResult::Failed(ConnectionResult::EphemeralAttemptNotFresh);
        }
        if runtime.snapshot.is_ready()
            && data_link_ready()
            && runtime
                .ready_target
                .is_some_and(|ready| same_connection_target(ready, target))
        {
            return ConnectionTriggerResult::AlreadyReady;
        }
        if runtime.job.is_some() {
            return ConnectionTriggerResult::AlreadyRunning;
        }
        runtime.snapshot.is_ready()
    };
    if replace_ready {
        quarantine_retained_ready_connection();
    }
    if !snapshot().is_ready() {
        return fail_connection_start(ConnectionResult::FirmwareNotReady);
    }
    let hw_spec = hw_spec_snapshot();
    if !hw_spec.is_ready() {
        return fail_connection_start(ConnectionResult::HwSpecNotReady);
    }
    let secure = match target.security {
        Dot11Security::Open => false,
        Dot11Security::Wpa2 if target.supports_wpa2_psk_ccmp() => true,
        Dot11Security::Wpa2 => return fail_connection_start(ConnectionResult::UnsupportedSecurity),
        Dot11Security::Wep | Dot11Security::Wpa | Dot11Security::Wpa3 | Dot11Security::Unknown => {
            return fail_connection_start(ConnectionResult::UnsupportedSecurity)
        }
    };
    if secure && hw_spec.fw_cap_info & marvell_wifi_supplicant::FW_CAP_FIRMWARE_SUPPLICANT == 0 {
        return fail_connection_start(ConnectionResult::FirmwareSupplicantUnavailable);
    }

    let Some(pci_address) = wifi::snapshot().address else {
        return fail_connection_start(ConnectionResult::FirmwareNotReady);
    };
    let Some(mmio_base) = ready_mmio_base() else {
        return fail_connection_start(ConnectionResult::FirmwareNotReady);
    };

    enum GatedAssociationStart {
        Started { replacing_ready: bool },
        Return(ConnectionTriggerResult),
        Fail(ConnectionResult),
    }

    let gated_start = (move || {
        let _dma_gate = MARVELL_DMA_GATE.lock();
        if connection_reboot_required() {
            return GatedAssociationStart::Fail(ConnectionResult::RebootRequired);
        }
        if let Err(failure) = arm_data_rings_while_gated(mmio_base) {
            queue_data_ring_failure_trace(HwFailurePhase::LinkBringup, failure);
            return GatedAssociationStart::Fail(ConnectionResult::DataRingUnavailable);
        }
        let (Some(cmd_dma_phys), Some(rsp_dma_phys)) = (connect_cmd_phys(), connect_rsp_phys())
        else {
            return GatedAssociationStart::Fail(ConnectionResult::DmaAddressUnavailable);
        };

        let mut runtime = CONNECTION.lock();
        if ephemeral && (runtime.snapshot.is_ready() || runtime.job.is_some()) {
            return GatedAssociationStart::Return(ConnectionTriggerResult::Failed(
                ConnectionResult::EphemeralAttemptNotFresh,
            ));
        }
        if runtime.snapshot.is_ready()
            && data_link_ready()
            && runtime
                .ready_target
                .is_some_and(|ready| same_connection_target(ready, target))
        {
            return GatedAssociationStart::Return(ConnectionTriggerResult::AlreadyReady);
        }
        if runtime.job.is_some() {
            return GatedAssociationStart::Return(ConnectionTriggerResult::AlreadyRunning);
        }
        let seq = runtime
            .sequence_allocator
            .reserve_window(CONNECTION_HOST_CMD_PHASE_COUNT);
        let first_phase = if secure {
            ConnectionStage::SupplicantProfile
        } else {
            ConnectionStage::Associate
        };
        let replacing_ready = runtime.snapshot.is_ready();
        runtime.snapshot = ConnectionSnapshot {
            attempted: true,
            running: true,
            stage: first_phase,
            failed_stage: None,
            result: None,
            response_diagnostic: None,
            transport_diagnostic: None,
            association_status: None,
            association_id: None,
            host_int_status: 0,
        };
        runtime.job = Some(ConnectionJob {
            pci_address,
            mmio_base,
            cmd_dma_phys,
            rsp_dma_phys,
            phase: first_phase,
            waiting: false,
            cmd_done_low_baseline: false,
            published_command_len: 0,
            request_header_matches_expected: false,
            phase_started_tsc: time::rdtsc(),
            transport_diagnostic: ConnectionTransportDiagnostic::new(),
            seq,
            target,
            secret_source,
        });
        runtime.ready_target = None;
        DATA_LINK_READY.store(false, Ordering::Release);
        GatedAssociationStart::Started { replacing_ready }
    })();

    match gated_start {
        GatedAssociationStart::Started { replacing_ready } => {
            if replacing_ready {
                net::detach_wifi();
            }
            serial::write_line("marvell wifi: bounded association sequence started");
            ConnectionTriggerResult::Started
        }
        GatedAssociationStart::Return(result) => result,
        GatedAssociationStart::Fail(result) => fail_connection_start(result),
    }
}

fn same_connection_target(left: wifi::ScannedNetwork, right: wifi::ScannedNetwork) -> bool {
    left.bssid == right.bssid
        && left.ssid.as_bytes() == right.ssid.as_bytes()
        && left.security == right.security
}

fn revalidate_ephemeral_release(job: &ConnectionJob) -> Result<(), ConnectionResult> {
    let ConnectionSecretSource::EphemeralPhysical { pending, receipt } = &job.secret_source else {
        return Ok(());
    };
    if pending.is_some() {
        return Err(ConnectionResult::EphemeralRevalidationFailed);
    }
    let receipt = receipt
        .as_ref()
        .ok_or(ConnectionResult::EphemeralRevalidationFailed)?;
    let current_target =
        wifi::association_target().ok_or(ConnectionResult::EphemeralRevalidationFailed)?;
    secret_vault::revalidate_ephemeral_physical_wifi_receipt(receipt, current_target)
        .map_err(|_| ConnectionResult::EphemeralRevalidationFailed)
}

fn quarantine_retained_ready_connection() {
    {
        let _dma_gate = MARVELL_DMA_GATE.lock();
        if let Some(address) = wifi::snapshot().address {
            if let Some(mmio_base) = ready_mmio_base() {
                let _ = terminal_quiesce_and_cleanup_while_gated(address, mmio_base as *mut u8);
            } else {
                let _ = verified_quiesce_while_gated(address);
            }
        }
        DATA_LINK_READY.store(false, Ordering::Release);
        let mut runtime = CONNECTION.lock();
        if runtime.snapshot.is_ready() {
            runtime.snapshot = ConnectionSnapshot::new();
            runtime.ready_target = None;
        }
    }
    net::detach_wifi();
}

fn fail_connection_start(result: ConnectionResult) -> ConnectionTriggerResult {
    {
        let _dma_gate = MARVELL_DMA_GATE.lock();
        if let Some(address) = wifi::snapshot().address {
            if let Some(mmio_base) = ready_mmio_base() {
                let _ = terminal_quiesce_and_cleanup_while_gated(address, mmio_base as *mut u8);
            } else {
                let _ = verified_quiesce_while_gated(address);
            }
        }
        let mut runtime = CONNECTION.lock();
        runtime.snapshot = ConnectionSnapshot {
            attempted: true,
            running: false,
            stage: ConnectionStage::Failed,
            failed_stage: None,
            result: Some(result),
            response_diagnostic: None,
            transport_diagnostic: None,
            association_status: None,
            association_id: None,
            host_int_status: 0,
        };
        runtime.ready_target = None;
        DATA_LINK_READY.store(false, Ordering::Release);
    }
    net::detach_wifi();
    ConnectionTriggerResult::Failed(result)
}

pub fn receive_ethernet() -> Option<RxPacket> {
    let _dma_gate = MARVELL_DMA_GATE.lock();
    if connection_reboot_required() || !data_link_ready() {
        return None;
    }

    let mut runtime = RX_RING.lock();
    if !runtime.snapshot.armed {
        return None;
    }
    let mmio_base = runtime.mmio_base? as *mut u8;
    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
    let wrptr = read_reg(mmio_base, PCIE_RX_WR_PTR);
    let device_wrptr = match decode_rx_tx_pointer_register(wrptr) {
        Ok(pointers) => pointers.rx,
        Err(_) => {
            runtime.snapshot.stage = RxRingStage::Failed;
            runtime.snapshot.result = Some(RxRingResult::BadReadPointer);
            drop(runtime);
            quarantine_invalid_pointer_after_ring_unlock_while_gated(
                mmio_base,
                "RX-WR/TX-RD",
                wrptr,
            );
            return None;
        }
    };
    let host_rdptr = match decode_rx_tx_pointer_register(runtime.rdptr) {
        Ok(pointers) => pointers.rx,
        Err(_) => {
            let raw = runtime.rdptr;
            runtime.snapshot.stage = RxRingStage::Failed;
            runtime.snapshot.result = Some(RxRingResult::BadReadPointer);
            drop(runtime);
            quarantine_invalid_pointer_after_ring_unlock_while_gated(mmio_base, "host RX-RD", raw);
            return None;
        }
    };
    if !device_ring_has_entry(device_wrptr, host_rdptr) {
        if status & HOST_INTR_UPLD_RDY != 0 {
            write_reg(mmio_base, PCIE_HOST_INT_STATUS, !HOST_INTR_UPLD_RDY);
        }
        return None;
    }

    let index = usize::from(host_rdptr.index);
    let packet = parse_rx_ethernet(index);
    let next_rdptr = next_rx_rdptr(runtime.rdptr);
    arm_rx_desc(index);
    compiler_fence(Ordering::SeqCst);
    if !publish_shared_rx_tx_pointer_while_gated(
        mmio_base,
        SharedRxTxPointerUpdate::RxRead(next_rdptr),
    ) {
        runtime.snapshot.stage = RxRingStage::Failed;
        runtime.snapshot.result = Some(RxRingResult::BadReadPointer);
        drop(runtime);
        quarantine_invalid_pointer_after_ring_unlock_while_gated(
            mmio_base,
            "host RX-RD/TX-WR",
            next_rdptr,
        );
        return None;
    }
    runtime.rdptr = next_rdptr;
    if status & HOST_INTR_UPLD_RDY != 0 {
        write_reg(mmio_base, PCIE_HOST_INT_STATUS, !HOST_INTR_UPLD_RDY);
    }
    compiler_fence(Ordering::SeqCst);

    runtime.snapshot.host_int_status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
    runtime.snapshot.rdptr = next_rdptr;
    runtime.snapshot.wrptr = wrptr;
    match packet {
        Some(ref packet) => {
            runtime.snapshot.stage = RxRingStage::PacketReady;
            runtime.snapshot.result = Some(RxRingResult::PacketObserved);
            runtime.snapshot.rx_len = packet.len as u16;
        }
        None => {
            runtime.snapshot.stage = RxRingStage::Failed;
            runtime.snapshot.result = Some(RxRingResult::BadRxLength);
        }
    }
    packet
}

pub fn transmit_ethernet(frame: &[u8]) -> bool {
    if frame.is_empty() || frame.len() > MAX_ETHERNET_FRAME_SIZE {
        return false;
    }
    let _dma_gate = MARVELL_DMA_GATE.lock();
    if connection_reboot_required() || !data_link_ready() {
        return false;
    }

    let mut runtime = TX_RING.lock();
    if !runtime.armed {
        return false;
    }
    let Some(mmio_base) = runtime.mmio_base else {
        return false;
    };
    let mmio = mmio_base as *mut u8;
    let status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    let rdptr = read_reg(mmio, PCIE_TX_RD_PTR);
    let device_rdptr = match decode_rx_tx_pointer_register(rdptr) {
        Ok(pointers) => pointers.tx,
        Err(_) => {
            drop(runtime);
            quarantine_invalid_pointer_after_ring_unlock_while_gated(mmio, "RX-WR/TX-RD", rdptr);
            return false;
        }
    };
    let host_wrptr = match decode_rx_tx_pointer_register(runtime.wrptr) {
        Ok(pointers) => pointers.tx,
        Err(_) => {
            let raw = runtime.wrptr;
            drop(runtime);
            quarantine_invalid_pointer_after_ring_unlock_while_gated(mmio, "host TX-WR", raw);
            return false;
        }
    };
    if status & HOST_INTR_DNLD_DONE != 0 {
        write_reg(mmio, PCIE_HOST_INT_STATUS, !HOST_INTR_DNLD_DONE);
    }
    runtime.rdptr = rdptr & TX_RING_WRAP_MASK;
    if host_ring_is_full(host_wrptr, device_rdptr) {
        return false;
    }

    let index = usize::from(host_wrptr.index);
    let total_len = DATA_INTERFACE_HEADER_LEN + TX_PD_LEN + frame.len();
    if total_len > TX_BUFFER_SIZE || !prepare_tx_buffer(index, frame, total_len) {
        return false;
    }

    let next_wrptr = next_tx_wrptr(runtime.wrptr);
    compiler_fence(Ordering::SeqCst);
    if !publish_shared_rx_tx_pointer_while_gated(mmio, SharedRxTxPointerUpdate::TxWrite(next_wrptr))
    {
        drop(runtime);
        quarantine_invalid_pointer_after_ring_unlock_while_gated(
            mmio,
            "host RX-RD/TX-WR",
            next_wrptr,
        );
        return false;
    }
    runtime.wrptr = next_wrptr;
    let _ = read_reg(mmio, 0);
    write_reg(mmio, PCIE_CPU_INT_EVENT, CPU_INTR_DNLD_RDY);
    compiler_fence(Ordering::SeqCst);
    true
}

pub fn start_bring_up_firmware() -> FirmwareBringupTriggerResult {
    let _dma_gate = MARVELL_DMA_GATE.lock();
    if connection_reboot_required() {
        // The latch can only be raised after bring-up has already been
        // attempted in this boot, so restarting here would violate quarantine.
        return FirmwareBringupTriggerResult::AlreadyAttempted;
    }
    {
        let mut runtime = BRINGUP.lock();
        if runtime.job.is_some() {
            return FirmwareBringupTriggerResult::AlreadyRunning;
        }
        if runtime.snapshot.attempted {
            return FirmwareBringupTriggerResult::AlreadyAttempted;
        }
        runtime.snapshot = FirmwareBringupSnapshot {
            attempted: true,
            running: false,
            stage: FirmwareStage::Preflight,
            total: firmware_image().len(),
            ..FirmwareBringupSnapshot::new()
        };
        runtime.mmio_base = None;
    }

    let wifi_snapshot = wifi::probe();
    if wifi_snapshot.state != wifi::WifiState::Detected {
        finish_without_mmio(FirmwareDownloadResult::NotPresent, FirmwareStage::Preflight);
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::NotPresent);
    }
    mark_stage(
        FirmwareStage::DetectOk,
        FirmwareRegisterSnapshot::unavailable(),
    );

    let firmware = firmware_image();
    if firmware.is_empty() {
        serial::write_line("marvell wifi: firmware image absent; pcie8897_uapsta.bin not embedded");
        finish_without_mmio(
            FirmwareDownloadResult::FirmwareImageAbsent,
            FirmwareStage::DetectOk,
        );
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::FirmwareImageAbsent);
    }

    let Some(address) = wifi_snapshot.address else {
        finish_without_mmio(FirmwareDownloadResult::NotPresent, FirmwareStage::DetectOk);
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::NotPresent);
    };
    let Some(bar) = pci::read_bar_info(address, MARVELL_REGISTER_BAR) else {
        finish_without_mmio(FirmwareDownloadResult::Bar2Missing, FirmwareStage::DetectOk);
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::Bar2Missing);
    };
    if !bar.is_memory() {
        finish_without_mmio(FirmwareDownloadResult::Bar2NotMmio, FirmwareStage::DetectOk);
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::Bar2NotMmio);
    }

    let verified_off = match verified_quiesce_while_gated(address) {
        Ok(verified_off) => verified_off,
        Err(_) => {
            finish_without_mmio(
                FirmwareDownloadResult::DmaAddressUnavailable,
                FirmwareStage::DetectOk,
            );
            return FirmwareBringupTriggerResult::Failed(
                FirmwareDownloadResult::DmaAddressUnavailable,
            );
        }
    };
    if validate_runtime_dma_plan_while_gated().is_err() {
        poison_dma_epoch_while_gated("firmware preflight DMA plan validation failed");
        finish_without_mmio(
            FirmwareDownloadResult::DmaAddressUnavailable,
            FirmwareStage::DetectOk,
        );
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::DmaAddressUnavailable);
    }
    if enable_memory_space_while_verified_off(address, verified_off).is_err() {
        finish_without_mmio(
            FirmwareDownloadResult::DmaAddressUnavailable,
            FirmwareStage::DetectOk,
        );
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::DmaAddressUnavailable);
    }
    let mapping = match memory::map_mmio(bar.base, bar.size as usize) {
        Ok(mapping) => mapping,
        Err(error) => {
            let result = FirmwareDownloadResult::MmioMapFailed(error);
            finish_without_mmio(result, FirmwareStage::DetectOk);
            return FirmwareBringupTriggerResult::Failed(result);
        }
    };

    let mmio_base = mapping.as_ptr::<u8>();
    let registers = read_register_snapshot(mmio_base);
    if !probe_mmio(mmio_base) {
        finish_with_registers(
            FirmwareDownloadResult::MmioProbeAllOnes,
            FirmwareStage::Bar2MapOk,
            registers,
            0,
            firmware.len(),
        );
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::MmioProbeAllOnes);
    }

    let Some(block_dma_phys) = dma_block_phys() else {
        finish_with_registers(
            FirmwareDownloadResult::DmaAddressUnavailable,
            FirmwareStage::Bar2MapOk,
            registers,
            0,
            firmware.len(),
        );
        return FirmwareBringupTriggerResult::Failed(FirmwareDownloadResult::DmaAddressUnavailable);
    };

    let now = time::rdtsc();
    let download = FirmwareDownload::new(firmware);
    let mut runtime = BRINGUP.lock();
    runtime.snapshot = FirmwareBringupSnapshot {
        attempted: true,
        running: true,
        stage: FirmwareStage::Bar2MapOk,
        failed_stage: None,
        result: None,
        downloaded: 0,
        total: firmware.len(),
        registers,
    };
    runtime.mmio_base = Some(mmio_base as usize);
    runtime.job = Some(FirmwareJob {
        download,
        pci_address: address,
        mmio_base: mmio_base as usize,
        block_dma_phys,
        phase: FwPhase::Downloading,
        phase_started_tsc: now,
        started_tsc: now,
        firmware_len: firmware.len(),
        pre_bme_checkpoint_confirmed: false,
    });
    serial::write_line(
        "marvell wifi: firmware bring-up ATTEMPT started (unaudited blob; DMA not IOMMU-confined)",
    );
    FirmwareBringupTriggerResult::Started
}

pub fn start_scan_ext_24ghz() -> ScanCmdTriggerResult {
    let _dma_gate = MARVELL_DMA_GATE.lock();
    if connection_reboot_required() {
        finish_scan_without_job(ScanCmdResult::RebootRequired, ScanCmdStage::Failed, 0, 0);
        return ScanCmdTriggerResult::Failed(ScanCmdResult::RebootRequired);
    }
    if !snapshot().is_ready() {
        finish_scan_without_job(ScanCmdResult::FirmwareNotReady, ScanCmdStage::Failed, 0, 0);
        return ScanCmdTriggerResult::Failed(ScanCmdResult::FirmwareNotReady);
    }
    if !hw_spec_snapshot().is_ready() {
        finish_scan_without_job(ScanCmdResult::HwSpecNotReady, ScanCmdStage::Failed, 0, 0);
        return ScanCmdTriggerResult::Failed(ScanCmdResult::HwSpecNotReady);
    }
    let Some(pci_address) = wifi::snapshot().address else {
        finish_scan_without_job(ScanCmdResult::FirmwareNotReady, ScanCmdStage::Failed, 0, 0);
        return ScanCmdTriggerResult::Failed(ScanCmdResult::FirmwareNotReady);
    };
    let Some(mmio_base) = ready_mmio_base() else {
        finish_scan_without_job(ScanCmdResult::FirmwareNotReady, ScanCmdStage::Failed, 0, 0);
        return ScanCmdTriggerResult::Failed(ScanCmdResult::FirmwareNotReady);
    };
    let Some(cmd_dma_phys) = scan_cmd_phys() else {
        finish_scan_without_job(
            ScanCmdResult::DmaAddressUnavailable,
            ScanCmdStage::Failed,
            0,
            0,
        );
        return ScanCmdTriggerResult::Failed(ScanCmdResult::DmaAddressUnavailable);
    };
    let Some(rsp_dma_phys) = scan_rsp_phys() else {
        finish_scan_without_job(
            ScanCmdResult::DmaAddressUnavailable,
            ScanCmdStage::Failed,
            0,
            0,
        );
        return ScanCmdTriggerResult::Failed(ScanCmdResult::DmaAddressUnavailable);
    };

    let mut runtime = SCAN.lock();
    if runtime.job.is_some() {
        return ScanCmdTriggerResult::AlreadyRunning;
    }

    let seq = runtime.next_seq;
    runtime.next_seq = runtime.next_seq.wrapping_add(1);
    if runtime.next_seq == 0 {
        runtime.next_seq = 1;
    }
    runtime.snapshot = ScanCmdSnapshot {
        attempted: true,
        running: true,
        stage: ScanCmdStage::Arming,
        result: None,
        host_int_status: 0,
        command_len: 0,
    };
    runtime.job = Some(ScanCmdJob {
        pci_address,
        mmio_base,
        cmd_dma_phys,
        rsp_dma_phys,
        started_tsc: time::rdtsc(),
        seq,
    });
    drop(runtime);

    wifi::note_scan_command_started();
    serial::write_line("marvell wifi: legacy scan command armed (results in bounded response)");
    ScanCmdTriggerResult::Started
}

pub fn poll_hw_spec() -> bool {
    if !snapshot().is_ready() {
        return false;
    }
    let _dma_gate = MARVELL_DMA_GATE.lock();
    if connection_reboot_required() {
        return false;
    }

    let mut runtime = HWSPEC.lock();
    if runtime.snapshot.stage == HwSpecStage::Idle
        || runtime.snapshot.stage == HwSpecStage::Ready
        || runtime.snapshot.stage == HwSpecStage::Failed
    {
        return false;
    }

    let Some(mut job) = runtime.job.take() else {
        return false;
    };
    let mmio_base = job.mmio_base as *mut u8;
    let mut changed = false;
    let mut actions = 0usize;

    while actions < ACTIONS_PER_POLL {
        actions += 1;
        let phase = runtime.snapshot.stage;
        if elapsed_ms(job.phase_started_tsc) >= HWSPEC_TIMEOUT_MS {
            let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
            let _ = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
            finish_hw_spec_locked(
                &mut runtime,
                HwSpecResult::CmdDoneTimeout,
                HwSpecStage::Failed,
                status,
                None,
                None,
            );
            write_hw_spec_failure(phase, HwSpecResult::CmdDoneTimeout, status);
            return true;
        }

        if !job.waiting {
            let host_epoch = match begin_host_command_epoch_while_gated(job.pci_address, mmio_base)
            {
                Ok(epoch) => epoch,
                Err(_) => {
                    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                    finish_hw_spec_locked(
                        &mut runtime,
                        HwSpecResult::PciFunctionUnavailable,
                        HwSpecStage::Failed,
                        status,
                        None,
                        None,
                    );
                    write_hw_spec_failure(phase, HwSpecResult::PciFunctionUnavailable, status);
                    return true;
                }
            };
            let command_len = match prepare_hw_spec_dma(&job, phase) {
                Ok(command_len) => command_len,
                Err(result) => {
                    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                    let _ = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
                    finish_hw_spec_locked(
                        &mut runtime,
                        result,
                        HwSpecStage::Failed,
                        status,
                        None,
                        None,
                    );
                    write_hw_spec_failure(phase, result, status);
                    return true;
                }
            };

            let publication = match publish_host_command_while_gated(
                job.pci_address,
                mmio_base,
                job.cmd_dma_phys,
                job.rsp_dma_phys,
                command_len,
                host_epoch,
            ) {
                Ok(publication) => publication,
                Err(_) => {
                    let result = HwSpecResult::PciCommandEnableFailed;
                    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                    let _ = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
                    finish_hw_spec_locked(
                        &mut runtime,
                        result,
                        HwSpecStage::Failed,
                        status,
                        None,
                        None,
                    );
                    write_hw_spec_failure(phase, result, status);
                    return true;
                }
            };
            runtime.snapshot.pci_command_before = publication.pci.command_before;
            runtime.snapshot.pci_command_after = publication.pci.command_after;
            runtime.snapshot.pci_vendor_device = publication.pci.vendor_device;
            let mut liveness = marvell_wifi_cmd::ConnectionMmioLiveness::MmioUnavailable;
            let mut firmware_status = u32::MAX;
            let mut pending = u32::MAX;
            for _ in 0..CONNECTION_MMIO_LIVENESS_POLLS {
                firmware_status = read_reg(mmio_base, FW_STATUS);
                pending = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                liveness = marvell_wifi_cmd::connection_mmio_liveness(firmware_status, pending);
                if liveness == marvell_wifi_cmd::ConnectionMmioLiveness::Ready {
                    break;
                }
                delay_us(SHORT_POLL_DELAY_US);
            }
            runtime.snapshot.firmware_status = firmware_status;
            runtime.snapshot.host_int_status = pending;
            let liveness_failure = match liveness {
                marvell_wifi_cmd::ConnectionMmioLiveness::Ready => None,
                marvell_wifi_cmd::ConnectionMmioLiveness::FirmwareNotReady => {
                    Some(HwSpecResult::FirmwareNotReady)
                }
                marvell_wifi_cmd::ConnectionMmioLiveness::MmioUnavailable => {
                    Some(HwSpecResult::PciFunctionUnavailable)
                }
            };
            if let Some(result) = liveness_failure {
                let _ = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
                finish_hw_spec_locked(
                    &mut runtime,
                    result,
                    HwSpecStage::Failed,
                    pending,
                    None,
                    None,
                );
                write_hw_spec_failure(phase, result, pending);
                return true;
            }

            let status = publication.first_poll_status;
            runtime.snapshot.host_int_status = status;
            job.waiting = true;
            job.phase_started_tsc = time::rdtsc();
            serial::write_fmt(format_args!(
                    "marvell wifi: init stage={} armed seq={} pci={:04x}>{:04x} fw=0x{:08x} host=0x{:08x}\r\n",
                    phase.label(),
                    init_phase_seq(job.seq, phase),
                    publication.pci.command_before,
                    publication.pci.command_after,
                    firmware_status,
                    status,
                ));
            changed = true;
        } else {
            let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
            if runtime.snapshot.host_int_status != status {
                runtime.snapshot.host_int_status = status;
                changed = true;
            }
            if status == u32::MAX {
                let _ = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
                finish_hw_spec_locked(
                    &mut runtime,
                    HwSpecResult::PciFunctionUnavailable,
                    HwSpecStage::Failed,
                    status,
                    None,
                    None,
                );
                write_hw_spec_failure(phase, HwSpecResult::PciFunctionUnavailable, status);
                return true;
            }
            if status & HOST_INTR_CMD_DONE != 0 {
                if !terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base) {
                    finish_hw_spec_locked(
                        &mut runtime,
                        HwSpecResult::PciFunctionUnavailable,
                        HwSpecStage::Failed,
                        status,
                        None,
                        None,
                    );
                    return true;
                }
                compiler_fence(Ordering::SeqCst);
                let (parsed, response_header) =
                    parse_hw_spec_dma_response(phase, init_phase_seq(job.seq, phase));
                runtime.snapshot.last_response = response_header;
                write_reg(mmio_base, PCIE_HOST_INT_STATUS, !status);
                let _cleanup_flush_status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                compiler_fence(Ordering::SeqCst);

                match parsed {
                    Ok(InitCommandResponse::HwSpec(hw_spec)) => {
                        runtime.snapshot.mac = Some(hw_spec.mac);
                        runtime.snapshot.fw_release = Some(hw_spec.fw_release);
                        runtime.snapshot.fw_cap_info = hw_spec.fw_cap_info;
                        runtime.snapshot.key_api_version = hw_spec.key_api_version;
                        serial::write_fmt(format_args!(
                            "marvell wifi: init stage={} response=ok fw_release=0x{:08x}\r\n",
                            phase.label(),
                            hw_spec.fw_release
                        ));
                        job.waiting = false;
                        job.phase_started_tsc = time::rdtsc();
                        runtime.snapshot.stage = HwSpecStage::MacControl;
                    }
                    Ok(InitCommandResponse::Unit) => {
                        let next = next_init_phase(phase);
                        serial::write_fmt(format_args!(
                            "marvell wifi: init stage={} response=ok next={}\r\n",
                            phase.label(),
                            next.label(),
                        ));
                        job.waiting = false;
                        job.phase_started_tsc = time::rdtsc();
                        if next == HwSpecStage::Ready {
                            if let Err(failure) =
                                publish_data_ring_pointers_while_gated(job.mmio_base)
                            {
                                queue_data_ring_failure_trace(
                                    HwFailurePhase::HardwareSpec,
                                    failure,
                                );
                                finish_hw_spec_locked(
                                    &mut runtime,
                                    HwSpecResult::DataRingUnavailable,
                                    HwSpecStage::Failed,
                                    status,
                                    None,
                                    None,
                                );
                                write_hw_spec_failure(
                                    phase,
                                    HwSpecResult::DataRingUnavailable,
                                    status,
                                );
                                return true;
                            }
                            finish_hw_spec_locked(
                                &mut runtime,
                                HwSpecResult::Done,
                                HwSpecStage::Ready,
                                status,
                                None,
                                None,
                            );
                            serial::write_line(
                                "marvell wifi: firmware epoch init ready; scan unlocked",
                            );
                            return true;
                        }
                        runtime.snapshot.stage = next;
                    }
                    Err(result) => {
                        finish_hw_spec_locked(
                            &mut runtime,
                            result,
                            HwSpecStage::Failed,
                            status,
                            None,
                            None,
                        );
                        write_hw_spec_failure(phase, result, status);
                        return true;
                    }
                }
                changed = true;
                continue;
            }
            delay_us(SHORT_POLL_DELAY_US);
        }
    }

    runtime.job = Some(job);
    changed
}

pub fn poll_scan_ext() -> bool {
    if !snapshot().is_ready() {
        return false;
    }
    let _dma_gate = MARVELL_DMA_GATE.lock();
    if connection_reboot_required() {
        return false;
    }

    let mut runtime = SCAN.lock();
    if runtime.snapshot.stage == ScanCmdStage::Idle
        || runtime.snapshot.stage == ScanCmdStage::Done
        || runtime.snapshot.stage == ScanCmdStage::Failed
    {
        return false;
    }

    let Some(job) = runtime.job.take() else {
        return false;
    };
    let mmio_base = job.mmio_base as *mut u8;
    let mut changed = false;
    let mut actions = 0usize;

    while actions < ACTIONS_PER_POLL {
        actions += 1;
        if elapsed_ms(job.started_tsc) >= SCAN_CMD_TIMEOUT_MS {
            let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
            let _ = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
            let command_len = runtime.snapshot.command_len;
            finish_scan_locked(
                &mut runtime,
                ScanCmdResult::CmdDoneTimeout,
                ScanCmdStage::Failed,
                status,
                command_len,
            );
            write_scan_failure(ScanCmdResult::CmdDoneTimeout, status);
            wifi::note_scan_command_failed();
            return true;
        }

        match runtime.snapshot.stage {
            ScanCmdStage::Arming => {
                let host_epoch =
                    match begin_host_command_epoch_while_gated(job.pci_address, mmio_base) {
                        Ok(epoch) => epoch,
                        Err(_) => {
                            let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                            finish_scan_locked(
                                &mut runtime,
                                ScanCmdResult::RebootRequired,
                                ScanCmdStage::Failed,
                                status,
                                0,
                            );
                            wifi::note_scan_command_failed();
                            return true;
                        }
                    };
                let command_len = match prepare_scan_dma(job.seq) {
                    Ok(command_len) => command_len,
                    Err(error) => {
                        let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                        let _ =
                            terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
                        let result = ScanCmdResult::CommandBuild(error);
                        finish_scan_locked(&mut runtime, result, ScanCmdStage::Failed, status, 0);
                        write_scan_failure(result, status);
                        wifi::note_scan_command_failed();
                        return true;
                    }
                };

                let publication = match publish_host_command_while_gated(
                    job.pci_address,
                    mmio_base,
                    job.cmd_dma_phys,
                    job.rsp_dma_phys,
                    command_len,
                    host_epoch,
                ) {
                    Ok(publication) => publication,
                    Err(_) => {
                        let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                        let _ =
                            terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base);
                        finish_scan_locked(
                            &mut runtime,
                            ScanCmdResult::RebootRequired,
                            ScanCmdStage::Failed,
                            status,
                            command_len,
                        );
                        wifi::note_scan_command_failed();
                        return true;
                    }
                };

                let status = publication.first_poll_status;
                runtime.snapshot = ScanCmdSnapshot {
                    attempted: true,
                    running: true,
                    stage: ScanCmdStage::WaitCmdDone,
                    result: None,
                    host_int_status: status,
                    command_len,
                };
                changed = true;
            }
            ScanCmdStage::WaitCmdDone => {
                let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                if runtime.snapshot.host_int_status != status {
                    runtime.snapshot.host_int_status = status;
                    changed = true;
                }
                if status & HOST_INTR_CMD_DONE != 0 {
                    if !terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base) {
                        let command_len = runtime.snapshot.command_len;
                        finish_scan_locked(
                            &mut runtime,
                            ScanCmdResult::RebootRequired,
                            ScanCmdStage::Failed,
                            status,
                            command_len,
                        );
                        wifi::note_scan_command_failed();
                        return true;
                    }
                    compiler_fence(Ordering::SeqCst);
                    let parsed = parse_scan_dma_response();
                    write_reg(mmio_base, PCIE_HOST_INT_STATUS, !status);
                    let _cleanup_flush_status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                    compiler_fence(Ordering::SeqCst);

                    match parsed {
                        Ok((declared, ingested)) => {
                            let command_len = runtime.snapshot.command_len;
                            if declared == 0 || ingested != 0 {
                                finish_scan_locked(
                                    &mut runtime,
                                    ScanCmdResult::Done,
                                    ScanCmdStage::Done,
                                    status,
                                    command_len,
                                );
                                wifi::note_scan_results_available();
                                serial::write_fmt(format_args!(
                                    "marvell wifi: legacy scan response ready declared={} ingested={}\r\n",
                                    declared, ingested
                                ));
                            } else {
                                let result = ScanCmdResult::LiveResultParseFailed;
                                finish_scan_locked(
                                    &mut runtime,
                                    result,
                                    ScanCmdStage::Failed,
                                    status,
                                    command_len,
                                );
                                write_scan_failure(result, status);
                                wifi::note_scan_command_failed();
                                serial::write_fmt(format_args!(
                                    "marvell wifi: legacy scan declared {} networks but ingested none\r\n",
                                    declared
                                ));
                            }
                        }
                        Err(error) => {
                            let result = ScanCmdResult::Response(error);
                            let command_len = runtime.snapshot.command_len;
                            finish_scan_locked(
                                &mut runtime,
                                result,
                                ScanCmdStage::Failed,
                                status,
                                command_len,
                            );
                            write_scan_failure(result, status);
                            wifi::note_scan_command_failed();
                        }
                    }
                    return true;
                }
                delay_us(SHORT_POLL_DELAY_US);
            }
            ScanCmdStage::Idle | ScanCmdStage::Done | ScanCmdStage::Failed => {
                runtime.job = Some(job);
                return changed;
            }
        }
    }

    runtime.job = Some(job);
    changed
}

fn ensure_pci_memory_bus_master(
    address: pci::PciAddress,
    vendor_device: u32,
    command_before: u16,
) -> Result<PciCommandEnableOutcome, PciCommandEnableError> {
    let command =
        match marvell_wifi_cmd::plan_pci_memory_bus_master_enable(vendor_device, command_before) {
            marvell_wifi_cmd::PciCommandEnablePlan::Unavailable => {
                return Err(PciCommandEnableError::FunctionUnavailable);
            }
            marvell_wifi_cmd::PciCommandEnablePlan::AlreadyEnabled => command_before,
            marvell_wifi_cmd::PciCommandEnablePlan::Write(command) => command,
        };
    let readback = match address.write_command_u16_checked(vendor_device, command_before, command) {
        pci::PciCommandWriteResult::Written { readback } => readback,
        pci::PciCommandWriteResult::DeviceUnavailable => {
            return Err(PciCommandEnableError::FunctionUnavailable);
        }
        pci::PciCommandWriteResult::CommandChanged { .. } => {
            return Err(PciCommandEnableError::EnableFailed);
        }
    };
    if !marvell_wifi_cmd::pci_memory_bus_master_enabled(readback) {
        return Err(PciCommandEnableError::EnableFailed);
    }
    Ok(PciCommandEnableOutcome {
        vendor_device,
        command_before,
        command_after: readback,
    })
}

fn arm_connection_command(
    job: &mut ConnectionJob,
    command_len: usize,
    epoch: HostCommandEpoch,
) -> Result<(), ConnectionTransportError> {
    let mmio = job.mmio_base as *mut u8;
    job.cmd_done_low_baseline = false;
    job.transport_diagnostic = ConnectionTransportDiagnostic::new();
    job.transport_diagnostic.pre_enable_status = epoch.pre_clear_status;
    job.transport_diagnostic.pre_enable_status_valid = true;
    job.transport_diagnostic.pre_clear_status = epoch.pre_clear_status;
    job.transport_diagnostic.pre_clear_status_valid = true;
    job.transport_diagnostic.post_clear_status = epoch.post_clear_status;
    job.transport_diagnostic.post_clear_status_valid = true;
    job.cmd_done_low_baseline = true;

    debug_assert!(command_len <= CONNECT_CMD_BUFFER_SIZE);
    debug_assert!(command_len <= 0x03ff);
    job.request_header_matches_expected =
        connection_request_header_matches_expected(job, command_len);
    job.published_command_len = command_len as u16;

    let publication = publish_host_command_while_gated(
        job.pci_address,
        mmio,
        job.cmd_dma_phys,
        job.rsp_dma_phys,
        command_len,
        epoch,
    )
    .map_err(|error| match error {
        HostCommandPublishError::Quiesce(VerifiedQuiesceError::FunctionUnavailable)
        | HostCommandPublishError::Pci(PciCommandEnableError::FunctionUnavailable) => {
            ConnectionTransportError::PciFunctionUnavailable
        }
        HostCommandPublishError::Quiesce(_)
        | HostCommandPublishError::Pci(PciCommandEnableError::EnableFailed)
        | HostCommandPublishError::CheckpointNotPersisted
        | HostCommandPublishError::PublicationOrder => {
            ConnectionTransportError::PciCommandEnableFailed
        }
        HostCommandPublishError::MmioUnavailable => ConnectionTransportError::MmioUnavailable,
        HostCommandPublishError::StaleCommandDone => ConnectionTransportError::StaleCommandDone,
    })?;
    job.transport_diagnostic.pci_vendor_device = publication.pci.vendor_device;
    job.transport_diagnostic.pci_command_before = publication.pci.command_before;
    job.transport_diagnostic.pci_command_after = publication.pci.command_after;
    job.transport_diagnostic.pci_config_valid = true;
    job.transport_diagnostic.program_flush_status = publication.program_flush_status;
    job.transport_diagnostic.program_flush_status_valid = true;
    job.transport_diagnostic.first_poll_status = publication.first_poll_status;
    job.transport_diagnostic.first_poll_status_valid = true;

    let mut liveness = marvell_wifi_cmd::ConnectionMmioLiveness::MmioUnavailable;
    for _ in 0..CONNECTION_MMIO_LIVENESS_POLLS {
        let firmware_status = read_reg(mmio, FW_STATUS);
        let host_interrupt_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
        job.transport_diagnostic.firmware_status = firmware_status;
        job.transport_diagnostic.firmware_status_valid = true;
        job.transport_diagnostic.post_enable_status = host_interrupt_status;
        job.transport_diagnostic.post_enable_status_valid = true;
        liveness =
            marvell_wifi_cmd::connection_mmio_liveness(firmware_status, host_interrupt_status);
        if liveness == marvell_wifi_cmd::ConnectionMmioLiveness::Ready {
            break;
        }
        delay_us(SHORT_POLL_DELAY_US);
    }
    match liveness {
        marvell_wifi_cmd::ConnectionMmioLiveness::Ready => {}
        marvell_wifi_cmd::ConnectionMmioLiveness::FirmwareNotReady => {
            return Err(ConnectionTransportError::FirmwareNotReadyAfterEnable);
        }
        marvell_wifi_cmd::ConnectionMmioLiveness::MmioUnavailable => {
            return Err(ConnectionTransportError::MmioUnavailable);
        }
    }
    Ok(())
}

pub fn poll_connection() -> bool {
    let (changed, network_action) = (|| {
        let _dma_gate = MARVELL_DMA_GATE.lock();
        if connection_reboot_required() {
            return (false, DeferredNetworkAction::None);
        }
        let mut runtime = CONNECTION.lock();
        if matches!(
            runtime.snapshot.stage,
            ConnectionStage::Idle | ConnectionStage::LinkReady | ConnectionStage::Failed
        ) {
            return (false, DeferredNetworkAction::None);
        }
        let Some(mut job) = runtime.job.take() else {
            return (false, DeferredNetworkAction::None);
        };
        let mmio = job.mmio_base as *mut u8;

        if job.phase == ConnectionStage::WaitPortRelease {
            if elapsed_ms(job.phase_started_tsc) >= PORT_RELEASE_TIMEOUT_MS {
                let status = read_reg(mmio, PCIE_HOST_INT_STATUS);
                quarantine_connection_job(&job);
                finish_connection_locked(
                    &mut runtime,
                    ConnectionResult::PortReleaseTimeout,
                    ConnectionStage::Failed,
                    status,
                );
                DATA_LINK_READY.store(false, Ordering::Release);
                serial::write_line("marvell wifi: firmware supplicant port release timed out");
                return (true, DeferredNetworkAction::None);
            }
            runtime.job = Some(job);
            return (false, DeferredNetworkAction::None);
        }

        if elapsed_ms(job.phase_started_tsc) >= CONNECT_CMD_TIMEOUT_MS {
            let status = read_reg(mmio, PCIE_HOST_INT_STATUS);
            if job.phase == ConnectionStage::PostPmkHwSpecCanary {
                let doorbell_status = read_reg(mmio, PCIE_CPU_INT_STATUS);
                let cleanup_verified = quarantine_connection_job(&job);
                let outcome =
                    if status == u32::MAX || doorbell_status == u32::MAX || !cleanup_verified {
                        PostPmkCanaryOutcome::MmioOrDoorbellUnavailable
                    } else if doorbell_status & CPU_INTR_DOOR_BELL == 0 {
                        PostPmkCanaryOutcome::TimeoutDoorbellCleared
                    } else {
                        PostPmkCanaryOutcome::TimeoutDoorbellStillSet
                    };
                queue_post_pmk_canary_result(outcome);
                finish_connection_locked(
                    &mut runtime,
                    ConnectionResult::RebootRequired,
                    ConnectionStage::Failed,
                    status,
                );
                DATA_LINK_READY.store(false, Ordering::Release);
                return (true, DeferredNetworkAction::None);
            }
            let associate_doorbell_ack = if job.phase == ConnectionStage::Associate {
                Some(associate_doorbell_ack_trace_value(read_reg(
                    mmio,
                    PCIE_CPU_INT_STATUS,
                )))
            } else {
                None
            };
            let cleanup_verified = quarantine_connection_job(&job);
            let response_class = if cleanup_verified {
                compiler_fence(Ordering::SeqCst);
                connection_timeout_response_class_after_verified_cleanup(&job)
            } else {
                ConnectionTimeoutResponseClass::Unavailable
            };
            queue_connection_timeout_trace(
                &job,
                status,
                cleanup_verified,
                response_class,
                associate_doorbell_ack,
            );
            finish_connection_locked(
                &mut runtime,
                ConnectionResult::CommandTimeout,
                ConnectionStage::Failed,
                status,
            );
            DATA_LINK_READY.store(false, Ordering::Release);
            serial::write_fmt(format_args!(
                "marvell wifi: connection command timeout at {}\r\n",
                job.phase.label()
            ));
            return (true, DeferredNetworkAction::None);
        }

        if !job.waiting {
            let host_epoch = match begin_host_command_epoch_while_gated(job.pci_address, mmio) {
                Ok(epoch) => epoch,
                Err(error) => {
                    let transport = match error {
                        HostCommandPublishError::Quiesce(
                            VerifiedQuiesceError::FunctionUnavailable,
                        )
                        | HostCommandPublishError::Pci(
                            PciCommandEnableError::FunctionUnavailable,
                        ) => ConnectionTransportError::PciFunctionUnavailable,
                        HostCommandPublishError::MmioUnavailable => {
                            ConnectionTransportError::MmioUnavailable
                        }
                        HostCommandPublishError::StaleCommandDone => {
                            ConnectionTransportError::StaleCommandDone
                        }
                        _ => ConnectionTransportError::PciCommandEnableFailed,
                    };
                    if job.phase == ConnectionStage::PostPmkHwSpecCanary {
                        let outcome = if transport == ConnectionTransportError::StaleCommandDone {
                            PostPmkCanaryOutcome::StaleHighCompletion
                        } else if transport == ConnectionTransportError::MmioUnavailable {
                            PostPmkCanaryOutcome::MmioOrDoorbellUnavailable
                        } else {
                            PostPmkCanaryOutcome::HostPublicationFailure
                        };
                        quarantine_connection_job(&job);
                        queue_post_pmk_canary_result(outcome);
                    }
                    finish_connection_locked(
                        &mut runtime,
                        ConnectionResult::Transport(transport),
                        ConnectionStage::Failed,
                        read_reg(mmio, PCIE_HOST_INT_STATUS),
                    );
                    DATA_LINK_READY.store(false, Ordering::Release);
                    return (true, DeferredNetworkAction::None);
                }
            };
            let command_len = match prepare_connection_dma(&mut job) {
                Ok(len) => len,
                Err(result) => {
                    let _ = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio);
                    clear_connection_secret_dma(&job);
                    if job.phase == ConnectionStage::PostPmkHwSpecCanary {
                        quarantine_connection_job(&job);
                        queue_post_pmk_canary_result(PostPmkCanaryOutcome::HostPublicationFailure);
                    }
                    finish_connection_locked(
                        &mut runtime,
                        result,
                        ConnectionStage::Failed,
                        read_reg(mmio, PCIE_HOST_INT_STATUS),
                    );
                    DATA_LINK_READY.store(false, Ordering::Release);
                    return (true, DeferredNetworkAction::None);
                }
            };
            if let Err(error) = arm_connection_command(&mut job, command_len, host_epoch) {
                let status = job.transport_diagnostic.post_clear_status;
                runtime.snapshot.transport_diagnostic = Some(job.transport_diagnostic);
                quarantine_connection_job(&job);
                if job.phase == ConnectionStage::PostPmkHwSpecCanary {
                    let outcome = if error == ConnectionTransportError::MmioUnavailable {
                        PostPmkCanaryOutcome::MmioOrDoorbellUnavailable
                    } else {
                        PostPmkCanaryOutcome::HostPublicationFailure
                    };
                    queue_post_pmk_canary_result(outcome);
                }
                finish_connection_locked(
                    &mut runtime,
                    ConnectionResult::Transport(error),
                    ConnectionStage::Failed,
                    status,
                );
                serial::write_fmt(format_args!(
                    "marvell wifi: connection mailbox arm failed phase={} reason={} pre=0x{:08x} post=0x{:08x}\r\n",
                    job.phase.label(),
                    error.label(),
                    job.transport_diagnostic.pre_clear_status,
                    job.transport_diagnostic.post_clear_status,
                ));
                return (true, DeferredNetworkAction::None);
            }
            job.waiting = true;
            job.phase_started_tsc = time::rdtsc();
            runtime.snapshot.stage = job.phase;
            runtime.snapshot.host_int_status = job.transport_diagnostic.first_poll_status;
            runtime.snapshot.transport_diagnostic = Some(job.transport_diagnostic);
            runtime.job = Some(job);
            return (true, DeferredNetworkAction::None);
        }

        let status = read_reg(mmio, PCIE_HOST_INT_STATUS);
        runtime.snapshot.host_int_status = status;
        job.transport_diagnostic.poll_count = job.transport_diagnostic.poll_count.saturating_add(1);
        runtime.snapshot.transport_diagnostic = Some(job.transport_diagnostic);
        if status == u32::MAX {
            quarantine_connection_job(&job);
            if job.phase == ConnectionStage::PostPmkHwSpecCanary {
                queue_post_pmk_canary_result(PostPmkCanaryOutcome::MmioOrDoorbellUnavailable);
            }
            finish_connection_locked(
                &mut runtime,
                ConnectionResult::Transport(ConnectionTransportError::MmioUnavailable),
                ConnectionStage::Failed,
                status,
            );
            serial::write_line("marvell wifi: HOST_INT_STATUS unavailable while awaiting CMD_DONE");
            return (true, DeferredNetworkAction::None);
        }
        if !marvell_wifi_cmd::host_cmd_done_is_current(
            job.cmd_done_low_baseline,
            status,
            HOST_INTR_CMD_DONE,
        ) {
            if status & HOST_INTR_CMD_DONE != 0 {
                quarantine_connection_job(&job);
                if job.phase == ConnectionStage::PostPmkHwSpecCanary {
                    queue_post_pmk_canary_result(PostPmkCanaryOutcome::StaleHighCompletion);
                }
                finish_connection_locked(
                    &mut runtime,
                    ConnectionResult::Transport(ConnectionTransportError::StaleCommandDone),
                    ConnectionStage::Failed,
                    status,
                );
                serial::write_line(
                    "marvell wifi: CMD_DONE rejected without a proven current low baseline",
                );
                return (true, DeferredNetworkAction::None);
            }
            runtime.job = Some(job);
            return (false, DeferredNetworkAction::None);
        }

        if job.phase == ConnectionStage::PostPmkHwSpecCanary {
            let cleanup_verified = quarantine_connection_job(&job);
            let outcome = if !cleanup_verified {
                PostPmkCanaryOutcome::MmioOrDoorbellUnavailable
            } else {
                compiler_fence(Ordering::SeqCst);
                match parse_post_pmk_hw_spec_canary_response(&job) {
                    Ok(()) => PostPmkCanaryOutcome::ExpectedCompletion,
                    Err(HwSpecCmdError::FwResult { .. }) => PostPmkCanaryOutcome::FirmwareResult,
                    Err(_) => PostPmkCanaryOutcome::MalformedOrWrongCompletion,
                }
            };
            queue_post_pmk_canary_result(outcome);
            finish_connection_locked(
                &mut runtime,
                ConnectionResult::RebootRequired,
                ConnectionStage::Failed,
                status,
            );
            DATA_LINK_READY.store(false, Ordering::Release);
            return (true, DeferredNetworkAction::None);
        }

        if !terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio) {
            finish_connection_locked(
                &mut runtime,
                ConnectionResult::RebootRequired,
                ConnectionStage::Failed,
                status,
            );
            return (true, DeferredNetworkAction::None);
        }
        compiler_fence(Ordering::SeqCst);
        let (response, response_header) = parse_connection_dma_response(&job);
        write_reg(mmio, PCIE_HOST_INT_STATUS, !status);
        let _cleanup_flush_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
        compiler_fence(Ordering::SeqCst);
        let association = match response {
            Ok(association) => association,
            Err(result) => {
                let response_diagnostic =
                    connection_response_diagnostic(&job, result, response_header);
                runtime.snapshot.response_diagnostic = response_diagnostic;
                quarantine_connection_job(&job);
                finish_connection_locked(&mut runtime, result, ConnectionStage::Failed, status);
                DATA_LINK_READY.store(false, Ordering::Release);
                write_connection_response_failure(job.phase, result, response_diagnostic, status);
                return (true, DeferredNetworkAction::None);
            }
        };
        clear_connection_secret_dma(&job);

        if let Some(association) = association {
            runtime.snapshot.association_status = Some(association.status_code);
            runtime.snapshot.association_id = association.association_id;
            if association.status_code != 0 {
                let result = ConnectionResult::AssociationRejected(association.status_code);
                quarantine_connection_job(&job);
                finish_connection_locked(&mut runtime, result, ConnectionStage::Failed, status);
                DATA_LINK_READY.store(false, Ordering::Release);
                serial::write_fmt(format_args!(
                    "marvell wifi: association rejected status={}\r\n",
                    association.status_code
                ));
                return (true, DeferredNetworkAction::None);
            }
        }

        job.waiting = false;
        job.cmd_done_low_baseline = false;
        job.phase_started_tsc = time::rdtsc();
        job.phase = next_connection_phase(job.phase, job.target.security);
        if job.phase == ConnectionStage::WaitPortRelease {
            if let Err(result) = revalidate_ephemeral_release(&job) {
                quarantine_connection_job(&job);
                finish_connection_locked(&mut runtime, result, ConnectionStage::Failed, status);
                DATA_LINK_READY.store(false, Ordering::Release);
                serial::write_line("marvell wifi: ephemeral authority denied before port release");
                return (true, DeferredNetworkAction::Detach);
            }
            if !activate_validated_data_dma_while_gated(job.pci_address) {
                finish_connection_locked(
                    &mut runtime,
                    ConnectionResult::RebootRequired,
                    ConnectionStage::Failed,
                    status,
                );
                return (true, DeferredNetworkAction::Detach);
            }
            runtime.snapshot.stage = ConnectionStage::WaitPortRelease;
            runtime.job = Some(job);
            serial::write_line(
                "marvell wifi: association accepted; waiting for secure port release",
            );
            return (true, DeferredNetworkAction::None);
        }
        if job.phase == ConnectionStage::LinkReady {
            if job.secret_source.is_ephemeral() {
                quarantine_connection_job(&job);
                finish_connection_locked(
                    &mut runtime,
                    ConnectionResult::EphemeralRevalidationFailed,
                    ConnectionStage::Failed,
                    status,
                );
                DATA_LINK_READY.store(false, Ordering::Release);
                return (true, DeferredNetworkAction::Detach);
            }
            if !activate_validated_data_dma_while_gated(job.pci_address) {
                finish_connection_locked(
                    &mut runtime,
                    ConnectionResult::RebootRequired,
                    ConnectionStage::Failed,
                    status,
                );
                return (true, DeferredNetworkAction::Detach);
            }
            let mac = hw_spec_snapshot().mac;
            let ready_target = job.target;
            finish_connection_locked(
                &mut runtime,
                ConnectionResult::LinkReady,
                ConnectionStage::LinkReady,
                status,
            );
            runtime.ready_target = Some(ready_target);
            DATA_LINK_READY.store(true, Ordering::Release);
            serial::write_line("marvell wifi: open association accepted; data link released");
            return (
                true,
                mac.map_or(DeferredNetworkAction::None, DeferredNetworkAction::Attach),
            );
        }

        runtime.snapshot.stage = job.phase;
        runtime.job = Some(job);
        (true, DeferredNetworkAction::None)
    })();

    apply_deferred_network_action(network_action);
    changed
}

fn next_connection_phase(stage: ConnectionStage, security: Dot11Security) -> ConnectionStage {
    match stage {
        ConnectionStage::SupplicantProfile => ConnectionStage::SupplicantPmk,
        ConnectionStage::SupplicantPmk => ConnectionStage::PostPmkHwSpecCanary,
        ConnectionStage::PostPmkHwSpecCanary => ConnectionStage::Failed,
        ConnectionStage::Associate if security == Dot11Security::Wpa2 => {
            ConnectionStage::WaitPortRelease
        }
        ConnectionStage::Associate => ConnectionStage::LinkReady,
        _ => ConnectionStage::Failed,
    }
}

fn finish_connection_locked(
    runtime: &mut ConnectionRuntime,
    result: ConnectionResult,
    stage: ConnectionStage,
    host_int_status: u32,
) {
    let previous_stage = runtime.snapshot.stage;
    if stage == ConnectionStage::Failed {
        let phase = match previous_stage {
            ConnectionStage::SupplicantProfile | ConnectionStage::SupplicantPmk => {
                HwFailurePhase::Authenticate
            }
            ConnectionStage::PostPmkHwSpecCanary => HwFailurePhase::HardwareSpec,
            ConnectionStage::Associate => HwFailurePhase::Associate,
            ConnectionStage::WaitPortRelease => HwFailurePhase::KeyExchange,
            _ => HwFailurePhase::LinkBringup,
        };
        let status = match result {
            ConnectionResult::CommandTimeout | ConnectionResult::PortReleaseTimeout => {
                HwFailureStatus::Timeout
            }
            ConnectionResult::UnsupportedSecurity => HwFailureStatus::UnsupportedSecurity,
            ConnectionResult::AssociationRejected(_) => HwFailureStatus::AssociationRejected,
            ConnectionResult::LinkLost(_) => HwFailureStatus::LinkLost,
            ConnectionResult::BootPostureDenied => HwFailureStatus::BootPostureDenied,
            ConnectionResult::CommandResponse(_) | ConnectionResult::SupplicantResponse(_) => {
                HwFailureStatus::FirmwareRejected
            }
            _ => HwFailureStatus::TransportFault,
        };
        if previous_stage != ConnectionStage::PostPmkHwSpecCanary {
            queue_fixed_hw_failure_trace(
                phase,
                status,
                HwFailureRegister::MarvellHostInterruptStatus,
                host_int_status,
            );
        }
    }
    runtime.job = None;
    runtime.snapshot.running = false;
    runtime.snapshot.failed_stage = if stage == ConnectionStage::Failed {
        Some(previous_stage)
    } else {
        None
    };
    runtime.snapshot.stage = stage;
    runtime.snapshot.result = Some(result);
    runtime.snapshot.host_int_status = host_int_status;
    if stage != ConnectionStage::Failed {
        runtime.snapshot.response_diagnostic = None;
        runtime.snapshot.transport_diagnostic = None;
    }
    runtime.ready_target = None;
}

// Callers are structurally constrained to hold MARVELL_DMA_GATE before
// acquiring CONNECTION; see the permanent integration predicate.
fn quarantine_connection_job(job: &ConnectionJob) -> bool {
    DATA_LINK_READY.store(false, Ordering::Release);
    let mmio = job.mmio_base as *mut u8;
    let cleanup_verified = terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio);
    if cleanup_verified {
        clear_connection_secret_dma(job);
    }
    CONNECTION_REBOOT_REQUIRED.swap(true, Ordering::AcqRel);
    compiler_fence(Ordering::SeqCst);
    cleanup_verified
}

fn clear_connection_secret_dma(job: &ConnectionJob) {
    if job.phase != ConnectionStage::SupplicantPmk {
        return;
    }
    unsafe {
        // SAFETY: the connection mailbox is dedicated to CONNECTION, and the
        // caller invokes this only after DMA is complete or bus mastering is off.
        ptr::write_bytes(connect_cmd_ptr(), 0, CONNECT_CMD_BUFFER_SIZE);
    }
    compiler_fence(Ordering::SeqCst);
}

pub fn poll_event_ring() -> bool {
    let (changed, serial_event) = {
        let _dma_gate = MARVELL_DMA_GATE.lock();
        if connection_reboot_required() {
            return false;
        }
        let mut runtime = EVENT_RING.lock();
        if !runtime.snapshot.armed {
            return false;
        }
        let Some(mmio_base) = runtime.mmio_base else {
            return false;
        };
        let mmio_base = mmio_base as *mut u8;
        let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
        let wrptr = read_reg(mmio_base, PCIE_EVT_WR_PTR);
        let device_wrptr = match decode_event_pointer(wrptr) {
            Ok(pointer) => pointer,
            Err(_) => {
                runtime.snapshot.stage = EventRingStage::Failed;
                runtime.snapshot.result = Some(EventRingResult::BadReadPointer);
                drop(runtime);
                quarantine_invalid_pointer_after_ring_unlock_while_gated(
                    mmio_base, "EVENT-WR", wrptr,
                );
                return true;
            }
        };
        let host_rdptr = match decode_event_pointer(runtime.rdptr) {
            Ok(pointer) => pointer,
            Err(_) => {
                let raw = runtime.rdptr;
                runtime.snapshot.stage = EventRingStage::Failed;
                runtime.snapshot.result = Some(EventRingResult::BadReadPointer);
                drop(runtime);
                quarantine_invalid_pointer_after_ring_unlock_while_gated(
                    mmio_base,
                    "host EVENT-RD",
                    raw,
                );
                return true;
            }
        };
        let changed = runtime.snapshot.host_int_status != status
            || runtime.snapshot.wrptr != wrptr
            || runtime.snapshot.rdptr != runtime.rdptr;
        runtime.snapshot.host_int_status = status;
        runtime.snapshot.wrptr = wrptr;
        runtime.snapshot.rdptr = runtime.rdptr;

        let event_available = device_ring_has_entry(device_wrptr, host_rdptr);
        if status & HOST_INTR_EVENT_RDY == 0 && !event_available {
            return changed;
        }
        if !event_available {
            write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_EVENT_DONE);
            return true;
        }

        let rd_index = usize::from(host_rdptr.index);

        let parsed = match parse_event_buffer(rd_index) {
            Ok(parsed) => parsed,
            Err(result) => {
                let next_rdptr = next_event_rdptr(runtime.rdptr);
                arm_event_desc(rd_index);
                compiler_fence(Ordering::SeqCst);
                write_reg(mmio_base, PCIE_EVT_RD_PTR, next_rdptr);
                write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_EVENT_DONE);
                compiler_fence(Ordering::SeqCst);
                runtime.rdptr = next_rdptr;
                runtime.snapshot = EventRingSnapshot {
                    attempted: true,
                    armed: true,
                    stage: if result == EventRingResult::PointerAdvancedEmptyBuffer {
                        EventRingStage::Armed
                    } else {
                        EventRingStage::Failed
                    },
                    result: Some(result),
                    host_int_status: status,
                    rdptr: next_rdptr,
                    wrptr,
                    event_len: event_buffer_len(rd_index),
                    event_type: event_buffer_type(rd_index),
                    event_cause: 0,
                };
                return true;
            }
        };

        let next_rdptr = next_event_rdptr(runtime.rdptr);
        arm_event_desc(rd_index);
        compiler_fence(Ordering::SeqCst);
        write_reg(mmio_base, PCIE_EVT_RD_PTR, next_rdptr);
        write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_EVENT_DONE);
        compiler_fence(Ordering::SeqCst);

        runtime.rdptr = next_rdptr;
        runtime.snapshot = EventRingSnapshot {
            attempted: true,
            armed: true,
            stage: EventRingStage::EventReady,
            result: Some(EventRingResult::EventObserved),
            host_int_status: status,
            rdptr: next_rdptr,
            wrptr,
            event_len: parsed.len,
            event_type: parsed.event_type,
            event_cause: parsed.cause,
        };
        (true, Some(parsed))
    };

    if let Some(parsed) = serial_event {
        handle_connection_event(parsed.cause);
        serial::write_fmt(format_args!(
            "marvell wifi: event observed cause=0x{:08x} len={} type=0x{:04x}\r\n",
            parsed.cause, parsed.len, parsed.event_type
        ));
    }
    changed
}

fn handle_connection_event(cause: u32) {
    let network_action = (|| {
        let _dma_gate = MARVELL_DMA_GATE.lock();
        if connection_reboot_required() {
            return DeferredNetworkAction::None;
        }
        let event_id = cause & 0xffff;
        if matches!(event_id, 0x0003 | 0x0008 | 0x0009) {
            let mut runtime = CONNECTION.lock();
            if runtime.snapshot.attempted && !runtime.snapshot.is_failed() {
                if let Some(job) = runtime.job.take() {
                    quarantine_connection_job(&job);
                } else if let Some(address) = wifi::snapshot().address {
                    if let Some(mmio_base) = ready_mmio_base() {
                        let _ =
                            terminal_quiesce_and_cleanup_while_gated(address, mmio_base as *mut u8);
                    } else {
                        let _ = verified_quiesce_while_gated(address);
                    }
                }
                runtime.snapshot.running = false;
                runtime.snapshot.failed_stage = Some(runtime.snapshot.stage);
                runtime.snapshot.stage = ConnectionStage::Failed;
                runtime.snapshot.result = Some(ConnectionResult::LinkLost(event_id as u16));
                runtime.snapshot.response_diagnostic = None;
                runtime.ready_target = None;
                DATA_LINK_READY.store(false, Ordering::Release);
                serial::write_fmt(format_args!(
                    "marvell wifi: link loss event 0x{:04x}; DMA quarantined\r\n",
                    event_id
                ));
                return DeferredNetworkAction::Detach;
            }
            return DeferredNetworkAction::None;
        }
        if event_id != marvell_wifi_supplicant::EVENT_PORT_RELEASE {
            return DeferredNetworkAction::None;
        }

        let mut runtime = CONNECTION.lock();
        if runtime.snapshot.stage != ConnectionStage::WaitPortRelease {
            return DeferredNetworkAction::None;
        }
        let Some(job) = runtime.job.take() else {
            return DeferredNetworkAction::None;
        };
        if revalidate_ephemeral_release(&job).is_err() {
            quarantine_connection_job(&job);
            finish_connection_locked(
                &mut runtime,
                ConnectionResult::EphemeralRevalidationFailed,
                ConnectionStage::Failed,
                read_reg(job.mmio_base as *mut u8, PCIE_HOST_INT_STATUS),
            );
            DATA_LINK_READY.store(false, Ordering::Release);
            serial::write_line("marvell wifi: ephemeral authority denied before net attach");
            return DeferredNetworkAction::Detach;
        }
        let ready_target = job.target;
        runtime.snapshot.running = false;
        runtime.snapshot.failed_stage = None;
        runtime.snapshot.stage = ConnectionStage::LinkReady;
        runtime.snapshot.result = Some(ConnectionResult::LinkReady);
        runtime.snapshot.response_diagnostic = None;
        runtime.ready_target = Some(ready_target);
        DATA_LINK_READY.store(true, Ordering::Release);
        drop(job);
        serial::write_line("marvell wifi: secure port released; data link and DHCP enabled");
        hw_spec_snapshot()
            .mac
            .map_or(DeferredNetworkAction::None, DeferredNetworkAction::Attach)
    })();

    apply_deferred_network_action(network_action);
}

pub fn poll() -> bool {
    let _dma_gate = MARVELL_DMA_GATE.lock();
    if connection_reboot_required() {
        return false;
    }
    let mut runtime = BRINGUP.lock();
    let Some(mut job) = runtime.job.take() else {
        return false;
    };
    let mmio_base = job.mmio_base as *mut u8;

    if !job.pre_bme_checkpoint_confirmed {
        let Ok(_verified_off) = verified_quiesce_while_gated(job.pci_address) else {
            queue_k2_publication_terminal(
                HwFailurePhase::FirmwareTransport,
                HwFailureStatus::K2PciCommandRejected,
                HwFailureRegister::MarvellPciCommand,
                u32::from(job.pci_address.read_u16(0x04)),
            );
            let registers = read_register_snapshot(mmio_base);
            finish_locked(
                &mut runtime,
                FirmwareDownloadResult::DmaAddressUnavailable,
                FirmwareStage::Failed,
                Some(FirmwareStage::Downloading),
                registers,
                job.download.offset(),
                job.firmware_len,
            );
            return true;
        };
        if validate_runtime_dma_plan_while_gated().is_err() {
            queue_k2_publication_terminal(
                HwFailurePhase::FirmwareTransport,
                HwFailureStatus::K2PublicationRejected,
                HwFailureRegister::MarvellPublicationStep,
                K2_PUBLICATION_STEP_DMA_VALIDATION,
            );
            let registers = read_register_snapshot(mmio_base);
            finish_locked(
                &mut runtime,
                FirmwareDownloadResult::DmaAddressUnavailable,
                FirmwareStage::Failed,
                Some(FirmwareStage::Downloading),
                registers,
                job.download.offset(),
                job.firmware_len,
            );
            return true;
        }
        match pre_bme_checkpoint_while_gated(job.pci_address, HwFailurePhase::FirmwareTransport) {
            PreBmeCheckpoint::Persisted => job.pre_bme_checkpoint_confirmed = true,
            PreBmeCheckpoint::Pending => {
                runtime.job = Some(job);
                return true;
            }
            PreBmeCheckpoint::Failed => {
                queue_k2_publication_terminal(
                    HwFailurePhase::FirmwareTransport,
                    HwFailureStatus::K2CheckpointPersistFailed,
                    HwFailureRegister::MarvellPublicationStep,
                    K2_PUBLICATION_STEP_CHECKPOINT_FLUSH,
                );
                let registers = read_register_snapshot(mmio_base);
                finish_locked(
                    &mut runtime,
                    FirmwareDownloadResult::DmaAddressUnavailable,
                    FirmwareStage::Failed,
                    Some(FirmwareStage::Downloading),
                    registers,
                    job.download.offset(),
                    job.firmware_len,
                );
                return true;
            }
        }
    }

    let mut actions = 0usize;
    let action_limit = if matches!(
        job.phase,
        FwPhase::Downloading | FwPhase::BlockPrepared | FwPhase::WaitingDoorbellAck
    ) {
        FW_DOWNLOAD_ACTIONS_PER_POLL
    } else {
        ACTIONS_PER_POLL
    };
    while actions < action_limit {
        actions += 1;
        if elapsed_ms(job.started_tsc) >= TOTAL_BRINGUP_TIMEOUT_MS {
            let registers = read_register_snapshot(mmio_base);
            finish_locked(
                &mut runtime,
                FirmwareDownloadResult::TotalTimeout,
                FirmwareStage::Failed,
                Some(stage_for_phase(job.phase)),
                registers,
                job.download.offset(),
                job.firmware_len,
            );
            serial::write_line("marvell wifi: firmware bring-up failed: total_timeout");
            return true;
        }

        sync_job_phase_clock(&mut job);
        let observed = read_observed(mmio_base);
        let action = job.download.step(observed);
        sync_job_phase_clock(&mut job);
        let stage = stage_for_action(action, job.phase);
        let registers = read_register_snapshot(mmio_base);
        update_running_snapshot(&mut runtime, &job, stage, registers);

        match action {
            FwAction::Done => {
                finish_locked(
                    &mut runtime,
                    FirmwareDownloadResult::Done,
                    FirmwareStage::Ready,
                    None,
                    registers,
                    job.firmware_len,
                    job.firmware_len,
                );
                arm_hw_spec_after_firmware_ready_while_gated(job.mmio_base, job.pci_address);
                wifi::note_firmware_ready_scan_unavailable();
                serial::write_line(
                    "marvell wifi: firmware ready 0xfedcba00; bounded hw_spec probe armed",
                );
                return true;
            }
            FwAction::Fail(error) => {
                finish_locked(
                    &mut runtime,
                    FirmwareDownloadResult::Failed(error),
                    FirmwareStage::Failed,
                    Some(stage),
                    registers,
                    job.download.offset(),
                    job.firmware_len,
                );
                serial::write_fmt(format_args!(
                    "marvell wifi: firmware bring-up failed: {}\r\n",
                    error.reason()
                ));
                return true;
            }
            FwAction::WriteBlock {
                image_offset,
                payload_len,
                wire_len,
            } => {
                if !terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base) {
                    finish_locked(
                        &mut runtime,
                        FirmwareDownloadResult::DrvReadyQuarantined,
                        FirmwareStage::Failed,
                        Some(FirmwareStage::Downloading),
                        registers,
                        job.download.offset(),
                        job.firmware_len,
                    );
                    return true;
                }
                let Some(end) = image_offset.checked_add(payload_len) else {
                    finish_locked(
                        &mut runtime,
                        FirmwareDownloadResult::FirmwareSliceOutOfRange,
                        FirmwareStage::Failed,
                        Some(FirmwareStage::Downloading),
                        registers,
                        job.download.offset(),
                        job.firmware_len,
                    );
                    return true;
                };
                let firmware = firmware_image();
                let Some(src) = firmware.get(image_offset..end) else {
                    finish_locked(
                        &mut runtime,
                        FirmwareDownloadResult::FirmwareSliceOutOfRange,
                        FirmwareStage::Failed,
                        Some(FirmwareStage::Downloading),
                        registers,
                        job.download.offset(),
                        job.firmware_len,
                    );
                    return true;
                };
                if let Err(error) = copy_block_into_dma(src, wire_len) {
                    finish_locked(
                        &mut runtime,
                        error,
                        FirmwareStage::Failed,
                        Some(FirmwareStage::Downloading),
                        registers,
                        job.download.offset(),
                        job.firmware_len,
                    );
                    return true;
                }
            }
            FwAction::Retry { .. } => {
                if !terminal_quiesce_and_cleanup_while_gated(job.pci_address, mmio_base) {
                    finish_locked(
                        &mut runtime,
                        FirmwareDownloadResult::DrvReadyQuarantined,
                        FirmwareStage::Failed,
                        Some(FirmwareStage::Downloading),
                        registers,
                        job.download.offset(),
                        job.firmware_len,
                    );
                    return true;
                }
            }
            FwAction::RingDoorbell => {}
            FwAction::WriteDrvReady { value } => {
                if !write_drv_ready_pre_quarantined(job.pci_address, mmio_base, value) {
                    finish_locked(
                        &mut runtime,
                        FirmwareDownloadResult::DrvReadyQuarantined,
                        FirmwareStage::Failed,
                        Some(FirmwareStage::PollingReady),
                        registers,
                        job.download.offset(),
                        job.firmware_len,
                    );
                    return true;
                }
                serial::write_line(
                    "marvell wifi: DRV_READY written after DMA/INTx pre-quarantine; polling FW_STATUS",
                );
                continue;
            }
            FwAction::PollDoorbellAck => {
                if elapsed_ms(job.phase_started_tsc) >= DOORBELL_ACK_TIMEOUT_MS {
                    finish_locked(
                        &mut runtime,
                        FirmwareDownloadResult::DoorbellAckTimeout,
                        FirmwareStage::Failed,
                        Some(FirmwareStage::DoorbellAck),
                        registers,
                        job.download.offset(),
                        job.firmware_len,
                    );
                    serial::write_line(
                        "marvell wifi: firmware bring-up failed: doorbell_ack_timeout",
                    );
                    return true;
                }
                delay_us(SHORT_POLL_DELAY_US);
                continue;
            }
            FwAction::PollFwStatus => match job.phase {
                FwPhase::Downloading => {
                    if elapsed_ms(job.phase_started_tsc) >= CMD_SIZE_TIMEOUT_MS {
                        finish_locked(
                            &mut runtime,
                            FirmwareDownloadResult::CmdSizeTimeout,
                            FirmwareStage::Failed,
                            Some(FirmwareStage::Downloading),
                            registers,
                            job.download.offset(),
                            job.firmware_len,
                        );
                        serial::write_line(
                            "marvell wifi: firmware bring-up failed: cmd_size_timeout",
                        );
                        return true;
                    }
                    delay_us(SHORT_POLL_DELAY_US);
                    continue;
                }
                FwPhase::PollingReady => {
                    match decide_fw_ready_poll(
                        observed.fw_status_reg,
                        elapsed_ms(job.phase_started_tsc),
                        FW_READY_TIMEOUT_MS,
                    ) {
                        FwReadyPollDecision::Ready => {}
                        FwReadyPollDecision::StillDownloading => {
                            continue;
                        }
                        FwReadyPollDecision::Timeout => {
                            finish_locked(
                                &mut runtime,
                                FirmwareDownloadResult::FirmwareReadyTimeout,
                                FirmwareStage::Failed,
                                Some(FirmwareStage::PollingReady),
                                registers,
                                job.download.offset(),
                                job.firmware_len,
                            );
                            serial::write_line(
                                "marvell wifi: firmware bring-up failed: firmware_ready_timeout",
                            );
                            return true;
                        }
                    }
                }
                _ => {
                    delay_us(SHORT_POLL_DELAY_US);
                    continue;
                }
            },
        }

        let plan = plan_register_writes(action, job.block_dma_phys);
        if !plan.is_empty() {
            compiler_fence(Ordering::SeqCst);
        }
        let mut index = 0;
        while index < plan.len() {
            if let Some(write) = plan.get(index) {
                write_reg(mmio_base, write.offset, write.value);
            }
            index += 1;
        }
        if !plan.is_empty() {
            compiler_fence(Ordering::SeqCst);
        }
        if matches!(action, FwAction::WriteBlock { .. } | FwAction::Retry { .. }) {
            let flush = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
            if flush == u32::MAX || !activate_validated_data_dma_while_gated(job.pci_address) {
                finish_locked(
                    &mut runtime,
                    FirmwareDownloadResult::DmaAddressUnavailable,
                    FirmwareStage::Failed,
                    Some(FirmwareStage::Downloading),
                    registers,
                    job.download.offset(),
                    job.firmware_len,
                );
                return true;
            }
        }
    }

    runtime.job = Some(job);
    true
}

fn sync_job_phase_clock(job: &mut FirmwareJob) {
    let phase = job.download.phase();
    if phase != job.phase {
        job.phase = phase;
        job.phase_started_tsc = time::rdtsc();
    }
}

fn finish_without_mmio(result: FirmwareDownloadResult, failed_stage: FirmwareStage) {
    finish_with_registers(
        result,
        failed_stage,
        FirmwareRegisterSnapshot::unavailable(),
        0,
        firmware_image().len(),
    );
}

fn finish_with_registers(
    result: FirmwareDownloadResult,
    failed_stage: FirmwareStage,
    registers: FirmwareRegisterSnapshot,
    downloaded: usize,
    total: usize,
) {
    let mut runtime = BRINGUP.lock();
    finish_locked(
        &mut runtime,
        result,
        FirmwareStage::Failed,
        Some(failed_stage),
        registers,
        downloaded,
        total,
    );
}

fn finish_locked(
    runtime: &mut FirmwareBringupRuntime,
    mut result: FirmwareDownloadResult,
    mut stage: FirmwareStage,
    mut failed_stage: Option<FirmwareStage>,
    registers: FirmwareRegisterSnapshot,
    downloaded: usize,
    total: usize,
) {
    if matches!(stage, FirmwareStage::Ready | FirmwareStage::Failed) {
        if let (Some(address), Some(mmio_base)) = (wifi::snapshot().address, runtime.mmio_base) {
            if !terminal_quiesce_and_cleanup_while_gated(address, mmio_base as *mut u8)
                && stage == FirmwareStage::Ready
            {
                result = FirmwareDownloadResult::DrvReadyQuarantined;
                stage = FirmwareStage::Failed;
                failed_stage = Some(FirmwareStage::PollingReady);
            }
        }
    }
    if stage == FirmwareStage::Failed {
        let status = match result {
            FirmwareDownloadResult::CmdSizeTimeout
            | FirmwareDownloadResult::DoorbellAckTimeout
            | FirmwareDownloadResult::FirmwareReadyTimeout
            | FirmwareDownloadResult::TotalTimeout => HwFailureStatus::Timeout,
            _ => HwFailureStatus::TransportFault,
        };
        queue_fixed_hw_failure_trace(
            HwFailurePhase::FirmwareTransport,
            status,
            HwFailureRegister::MarvellFirmwareStatus,
            registers.fw_status,
        );
    }
    runtime.job = None;
    runtime.snapshot = FirmwareBringupSnapshot {
        attempted: true,
        running: false,
        stage,
        failed_stage,
        result: Some(result),
        downloaded,
        total,
        registers,
    };
}

fn mark_stage(stage: FirmwareStage, registers: FirmwareRegisterSnapshot) {
    let mut runtime = BRINGUP.lock();
    runtime.snapshot.stage = stage;
    runtime.snapshot.registers = registers;
}

fn update_running_snapshot(
    runtime: &mut FirmwareBringupRuntime,
    job: &FirmwareJob,
    stage: FirmwareStage,
    registers: FirmwareRegisterSnapshot,
) {
    runtime.snapshot = FirmwareBringupSnapshot {
        attempted: true,
        running: true,
        stage,
        failed_stage: None,
        result: None,
        downloaded: job.download.offset(),
        total: job.firmware_len,
        registers,
    };
}

fn arm_hw_spec_after_firmware_ready_while_gated(mmio_base: usize, pci_address: pci::PciAddress) {
    let mut runtime = HWSPEC.lock();
    if runtime.snapshot.attempted {
        return;
    }

    if let Err(failure) = arm_data_rings_while_gated(mmio_base) {
        queue_data_ring_failure_trace(HwFailurePhase::HardwareSpec, failure);
        finish_hw_spec_locked(
            &mut runtime,
            HwSpecResult::DataRingUnavailable,
            HwSpecStage::Failed,
            0,
            None,
            None,
        );
        write_hw_spec_failure(
            HwSpecStage::PcieDescDetails,
            HwSpecResult::DataRingUnavailable,
            0,
        );
        return;
    }

    let Some(cmd_dma_phys) = hw_spec_cmd_phys() else {
        finish_hw_spec_locked(
            &mut runtime,
            HwSpecResult::DmaAddressUnavailable,
            HwSpecStage::Failed,
            0,
            None,
            None,
        );
        write_hw_spec_failure(
            HwSpecStage::PcieDescDetails,
            HwSpecResult::DmaAddressUnavailable,
            0,
        );
        return;
    };
    let Some(rsp_dma_phys) = hw_spec_rsp_phys() else {
        finish_hw_spec_locked(
            &mut runtime,
            HwSpecResult::DmaAddressUnavailable,
            HwSpecStage::Failed,
            0,
            None,
            None,
        );
        write_hw_spec_failure(
            HwSpecStage::PcieDescDetails,
            HwSpecResult::DmaAddressUnavailable,
            0,
        );
        return;
    };

    runtime.snapshot = HwSpecSnapshot {
        attempted: true,
        running: true,
        stage: HwSpecStage::PcieDescDetails,
        failed_stage: None,
        result: None,
        last_response: None,
        mac: None,
        fw_release: None,
        fw_cap_info: 0,
        key_api_version: None,
        host_int_status: 0,
        pci_vendor_device: u32::MAX,
        pci_command_before: u16::MAX,
        pci_command_after: u16::MAX,
        firmware_status: u32::MAX,
    };
    runtime.job = Some(HwSpecJob {
        pci_address,
        mmio_base,
        cmd_dma_phys,
        rsp_dma_phys,
        phase_started_tsc: time::rdtsc(),
        waiting: false,
        seq: 1,
    });
}

fn arm_event_ring_while_gated(mmio_base: usize) -> Result<(), DataRingFailure> {
    let mut runtime = EVENT_RING.lock();
    if runtime.snapshot.attempted {
        if let Some(failure) = runtime.arm_failure {
            return Err(failure);
        }
        return if runtime.mmio_base.is_some() && !runtime.snapshot.is_failed() {
            Ok(())
        } else {
            Err(DataRingFailure::ExistingEventFailure(
                runtime.snapshot.result,
            ))
        };
    }
    let mmio = mmio_base as *mut u8;
    let host_int_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    let wrptr = read_reg(mmio, PCIE_EVT_WR_PTR);
    if let Err(error) = decode_event_pointer(wrptr) {
        let failure = DataRingFailure::EventWriteDecode(error);
        runtime.snapshot = EventRingSnapshot {
            attempted: true,
            armed: false,
            stage: EventRingStage::Failed,
            result: Some(EventRingResult::BadReadPointer),
            host_int_status,
            rdptr: runtime.rdptr,
            wrptr,
            event_len: 0,
            event_type: 0,
            event_cause: 0,
        };
        runtime.arm_failure = Some(failure);
        drop(runtime);
        quarantine_invalid_pointer_after_ring_unlock_while_gated(mmio, "EVENT-WR", wrptr);
        return Err(failure);
    }

    let mut index = 0usize;
    while index < EVENT_RING_COUNT {
        let Some(data_phys) = event_data_phys(index) else {
            let failure = DataRingFailure::EventDmaTranslation { index };
            runtime.snapshot = EventRingSnapshot {
                attempted: true,
                armed: false,
                stage: EventRingStage::Failed,
                result: Some(EventRingResult::DmaAddressUnavailable),
                ..EventRingSnapshot::new()
            };
            runtime.arm_failure = Some(failure);
            serial::write_line("marvell wifi: event ring arm failed: dma_address_unavailable");
            return Err(failure);
        };
        unsafe {
            // SAFETY: EVENT_RING_DMA_BLOCK is the driver's fixed event DMA
            // storage; EVENT_RING serializes descriptor setup and index bounds
            // are checked by the loop.
            ptr::write_bytes(event_data_ptr(index), 0, EVENT_BUFFER_SIZE);
            ptr::write(
                event_desc_ptr(index),
                EventBufDesc {
                    paddr: data_phys,
                    len: EVENT_BUFFER_SIZE as u16,
                    flags: 0,
                },
            );
        }
        index += 1;
    }

    runtime.rdptr = EVENT_ROLLOVER_IND;
    compiler_fence(Ordering::SeqCst);

    runtime.mmio_base = Some(mmio_base);
    runtime.snapshot = EventRingSnapshot {
        attempted: true,
        armed: false,
        stage: EventRingStage::Armed,
        result: Some(EventRingResult::Armed),
        host_int_status,
        rdptr: runtime.rdptr,
        wrptr,
        event_len: 0,
        event_type: 0,
        event_cause: 0,
    };
    serial::write_line("marvell wifi: event ring prepared for descriptor registration");
    Ok(())
}

fn arm_rx_ring_while_gated(mmio_base: usize) -> Result<(), DataRingFailure> {
    let mut runtime = RX_RING.lock();
    if runtime.snapshot.attempted {
        if let Some(failure) = runtime.arm_failure {
            return Err(failure);
        }
        return if runtime.snapshot.armed {
            Ok(())
        } else {
            Err(DataRingFailure::ExistingRxFailure(runtime.snapshot.result))
        };
    }
    let mmio = mmio_base as *mut u8;
    let host_int_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    // Firmware owns RX-WR/TX-RD; it is not observable until data-ring use.
    let wrptr = 0;
    let mut index = 0usize;
    while index < RX_RING_COUNT {
        let Some(data_phys) = rx_data_phys(index) else {
            let failure = DataRingFailure::RxDmaTranslation { index };
            runtime.snapshot = RxRingSnapshot {
                attempted: true,
                armed: false,
                stage: RxRingStage::Failed,
                result: Some(RxRingResult::DmaAddressUnavailable),
                ..RxRingSnapshot::new()
            };
            runtime.arm_failure = Some(failure);
            return Err(failure);
        };
        unsafe {
            // SAFETY: RX_RING_DMA_BLOCK is the driver's fixed RX DMA storage;
            // this setup runs before DRV_READY and indices are loop-bounded.
            ptr::write_bytes(rx_data_ptr(index), 0, RX_BUFFER_SIZE);
            ptr::write(
                rx_desc_ptr(index),
                RxPfuBufDesc {
                    flags: RX_DESC_FLAG_SOP | RX_DESC_FLAG_EOP,
                    offset: 0,
                    frag_len: RX_BUFFER_SIZE as u16,
                    len: RX_BUFFER_SIZE as u16,
                    paddr: data_phys,
                    reserved: 0,
                },
            );
        }
        index += 1;
    }

    runtime.rdptr = RX_ROLLOVER_IND;
    runtime.mmio_base = Some(mmio_base);
    runtime.snapshot = RxRingSnapshot {
        attempted: true,
        armed: true,
        stage: RxRingStage::Armed,
        result: Some(RxRingResult::Armed),
        host_int_status,
        rdptr: runtime.rdptr,
        wrptr,
        rx_len: 0,
        rx_type: 0,
    };
    Ok(())
}

fn arm_tx_ring_while_gated(mmio_base: usize) -> bool {
    let mut runtime = TX_RING.lock();
    if runtime.armed {
        return true;
    }
    let mut index = 0usize;
    while index < TX_RING_COUNT {
        unsafe {
            // SAFETY: the loop bounds every descriptor and buffer access to
            // the fixed TX DMA block before firmware receives its address.
            ptr::write_bytes(tx_data_ptr(index), 0, TX_BUFFER_SIZE);
            ptr::write(tx_desc_ptr(index), RxPfuBufDesc::EMPTY);
        }
        index += 1;
    }
    runtime.armed = true;
    runtime.mmio_base = Some(mmio_base);
    runtime.wrptr = 0;
    runtime.rdptr = 0;
    true
}

fn arm_data_rings_while_gated(mmio_base: usize) -> Result<(), DataRingFailure> {
    arm_event_ring_while_gated(mmio_base)?;
    arm_rx_ring_while_gated(mmio_base)?;
    let _ = arm_tx_ring_while_gated(mmio_base);
    Ok(())
}

fn plan_shared_rx_tx_pointer(
    current_raw: u32,
    update: SharedRxTxPointerUpdate,
) -> Result<u32, DevicePointerError> {
    match update {
        SharedRxTxPointerUpdate::Initialize { rx_rdptr, tx_wrptr } => {
            let rx = decode_rx_tx_pointer_register(rx_rdptr)?.rx;
            let tx = decode_rx_tx_pointer_register(tx_wrptr)?.tx;
            compose_rx_tx_pointer_register(RxTxDevicePointers { rx, tx })
        }
        SharedRxTxPointerUpdate::RxRead(rx_rdptr) => {
            let rx = decode_rx_tx_pointer_register(rx_rdptr)?.rx;
            update_rx_pointer_preserving_tx(current_raw, rx)
        }
        SharedRxTxPointerUpdate::TxWrite(tx_wrptr) => {
            let tx = decode_rx_tx_pointer_register(tx_wrptr)?.tx;
            update_tx_pointer_preserving_rx(current_raw, tx)
        }
    }
}

fn publish_shared_rx_tx_pointer_while_gated(
    mmio_base: *mut u8,
    update: SharedRxTxPointerUpdate,
) -> bool {
    let current_raw = *RX_TX_POINTER_REGISTER.lock();
    let next_raw = match plan_shared_rx_tx_pointer(current_raw, update) {
        Ok(raw) => raw,
        Err(_) => return false,
    };
    publish_prepared_shared_rx_tx_pointer_while_gated(mmio_base, next_raw);
    true
}

fn publish_prepared_shared_rx_tx_pointer_while_gated(mmio_base: *mut u8, next_raw: u32) {
    let mut shared_raw = RX_TX_POINTER_REGISTER.lock();
    *shared_raw = next_raw;
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio_base, PCIE_RX_RD_TX_WR_PTR, next_raw);
    compiler_fence(Ordering::SeqCst);
}

fn publish_data_ring_pointers_while_gated(mmio_base: usize) -> Result<(), DataRingFailure> {
    let event_rdptr = {
        let runtime = EVENT_RING.lock();
        runtime.rdptr
    };
    let rx_rdptr = RX_RING.lock().rdptr;
    let tx_wrptr = TX_RING.lock().wrptr;
    if let Err(error) = decode_event_pointer(event_rdptr) {
        quarantine_invalid_pointer_after_ring_unlock_while_gated(
            mmio_base as *mut u8,
            "host EVENT-RD",
            event_rdptr,
        );
        return Err(DataRingFailure::HostEventReadPublication(error));
    }
    let shared_raw = match plan_shared_rx_tx_pointer(
        0,
        SharedRxTxPointerUpdate::Initialize { rx_rdptr, tx_wrptr },
    ) {
        Ok(raw) => raw,
        Err(error) => {
            quarantine_invalid_pointer_after_ring_unlock_while_gated(
                mmio_base as *mut u8,
                "host RX-RD/TX-WR",
                rx_rdptr | tx_wrptr,
            );
            return Err(DataRingFailure::SharedRxTxPublication(error));
        }
    };
    EVENT_RING.lock().snapshot.armed = true;
    let mmio = mmio_base as *mut u8;
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
    write_reg(mmio, PCIE_EVT_RD_PTR, event_rdptr);
    publish_prepared_shared_rx_tx_pointer_while_gated(mmio, shared_raw);
    compiler_fence(Ordering::SeqCst);
    Ok(())
}

fn finish_hw_spec_locked(
    runtime: &mut HwSpecRuntime,
    result: HwSpecResult,
    stage: HwSpecStage,
    host_int_status: u32,
    mac: Option<[u8; 6]>,
    fw_release: Option<u32>,
) {
    let previous = runtime.snapshot;
    if stage == HwSpecStage::Failed && result != HwSpecResult::DataRingUnavailable {
        let status = match result {
            HwSpecResult::CmdDoneTimeout => HwFailureStatus::Timeout,
            HwSpecResult::Response(HwSpecCmdError::FwResult { .. })
            | HwSpecResult::InitResponse(MarvellCmdError::FwResult { .. }) => {
                HwFailureStatus::FirmwareRejected
            }
            _ => HwFailureStatus::TransportFault,
        };
        queue_fixed_hw_failure_trace(
            HwFailurePhase::HardwareSpec,
            status,
            HwFailureRegister::MarvellHostInterruptStatus,
            host_int_status,
        );
    }
    runtime.job = None;
    runtime.snapshot = HwSpecSnapshot {
        attempted: true,
        running: false,
        stage,
        failed_stage: if stage == HwSpecStage::Failed {
            Some(previous.stage)
        } else {
            None
        },
        result: Some(result),
        last_response: previous.last_response,
        mac: mac.or(previous.mac),
        fw_release: fw_release.or(previous.fw_release),
        fw_cap_info: previous.fw_cap_info,
        key_api_version: previous.key_api_version,
        host_int_status,
        pci_vendor_device: previous.pci_vendor_device,
        pci_command_before: previous.pci_command_before,
        pci_command_after: previous.pci_command_after,
        firmware_status: previous.firmware_status,
    };
}

fn finish_scan_without_job(
    result: ScanCmdResult,
    stage: ScanCmdStage,
    host_int_status: u32,
    command_len: usize,
) {
    let mut runtime = SCAN.lock();
    finish_scan_locked(&mut runtime, result, stage, host_int_status, command_len);
}

fn finish_scan_locked(
    runtime: &mut ScanCmdRuntime,
    result: ScanCmdResult,
    stage: ScanCmdStage,
    host_int_status: u32,
    command_len: usize,
) {
    if stage == ScanCmdStage::Failed
        && !matches!(
            result,
            ScanCmdResult::FirmwareNotReady | ScanCmdResult::HwSpecNotReady
        )
    {
        let status = match result {
            ScanCmdResult::CmdDoneTimeout => HwFailureStatus::Timeout,
            ScanCmdResult::Response(HwSpecCmdError::FwResult { .. }) => {
                HwFailureStatus::FirmwareRejected
            }
            _ => HwFailureStatus::TransportFault,
        };
        queue_fixed_hw_failure_trace(
            HwFailurePhase::Scan,
            status,
            HwFailureRegister::MarvellHostInterruptStatus,
            host_int_status,
        );
    }
    runtime.job = None;
    runtime.snapshot = ScanCmdSnapshot {
        attempted: true,
        running: false,
        stage,
        result: Some(result),
        host_int_status,
        command_len,
    };
}

fn prepare_hw_spec_dma(job: &HwSpecJob, phase: HwSpecStage) -> Result<usize, HwSpecResult> {
    unsafe {
        // SAFETY: HWSPEC_DMA_BLOCK is this driver's distinct command mailbox
        // DMA area. Access is serialized by HWSPEC and bounded by fixed sizes.
        ptr::write_bytes(hw_spec_rsp_ptr(), 0, MWIFIEX_UPLD_SIZE);
        let cmd = slice::from_raw_parts_mut(hw_spec_cmd_ptr(), HWSPEC_CMD_BUFFER_SIZE);
        let seq = init_phase_seq(job.seq, phase);
        match phase {
            HwSpecStage::PcieDescDetails => {
                let rings = marvell_wifi_cmd::PcieDescriptorRings {
                    tx_phys: tx_desc_phys().ok_or(HwSpecResult::DmaAddressUnavailable)?,
                    rx_phys: rx_desc_phys().ok_or(HwSpecResult::DmaAddressUnavailable)?,
                    event_phys: event_desc_phys().ok_or(HwSpecResult::DmaAddressUnavailable)?,
                };
                marvell_wifi_cmd::build_pcie_desc_details(seq, rings, cmd)
                    .map_err(HwSpecResult::InitCommandBuild)
            }
            HwSpecStage::FuncInit => {
                marvell_wifi_cmd::build_func_init(seq, cmd).map_err(HwSpecResult::InitCommandBuild)
            }
            HwSpecStage::GetHwSpec => {
                marvell_wifi_cmd::build_get_hw_spec(seq, cmd).map_err(HwSpecResult::CommandBuild)
            }
            HwSpecStage::MacControl => marvell_wifi_cmd::build_mac_control(seq, cmd)
                .map_err(HwSpecResult::InitCommandBuild),
            _ => Err(HwSpecResult::InitCommandBuild(MarvellCmdError::BadLength)),
        }
    }
}

fn prepare_scan_dma(seq: u16) -> Result<usize, HwSpecCmdError> {
    unsafe {
        // SAFETY: SCAN_DMA_BLOCK is this driver's distinct command mailbox DMA
        // area. Access is serialized by SCAN and bounded by fixed sizes.
        ptr::write_bytes(scan_rsp_ptr(), 0, MWIFIEX_UPLD_SIZE);
        let cmd = slice::from_raw_parts_mut(scan_cmd_ptr(), SCAN_CMD_BUFFER_SIZE);
        marvell_wifi_cmd::build_scan_24ghz(seq, cmd)
    }
}

fn prepare_connection_dma(job: &mut ConnectionJob) -> Result<usize, ConnectionResult> {
    unsafe {
        // SAFETY: CONNECTION serializes the dedicated command/response DMA
        // block and every pure builder is bounded by CONNECT_CMD_BUFFER_SIZE.
        ptr::write_bytes(connect_rsp_ptr(), 0, MWIFIEX_UPLD_SIZE);
        let out = slice::from_raw_parts_mut(connect_cmd_ptr(), CONNECT_CMD_BUFFER_SIZE);
        let seq = connection_phase_seq(job.seq, job.phase);
        match job.phase {
            ConnectionStage::SupplicantProfile => {
                marvell_wifi_supplicant::build_supplicant_profile_set(seq, out)
                    .map_err(ConnectionResult::SupplicantBuild)
            }
            ConnectionStage::SupplicantPmk => match &mut job.secret_source {
                ConnectionSecretSource::SafeVault(authority) => {
                    let Some(authority) = authority.take() else {
                        return Err(ConnectionResult::PassphraseUnavailable);
                    };
                    secret_vault::write_wifi_pmk_for_safe_association(
                        authority,
                        seq,
                        job.target.ssid.as_bytes(),
                        job.target.bssid,
                        out,
                    )
                    .map_err(|denied| {
                        serial::write_fmt(format_args!(
                            "VAULT_WIFI_SAFE_USE_REJECTED reason={}\r\n",
                            denied.wifi_use_reason()
                        ));
                        ConnectionResult::PassphraseUnavailable
                    })
                }
                ConnectionSecretSource::EphemeralPhysical { pending, receipt } => {
                    if receipt.is_some() {
                        return Err(ConnectionResult::EphemeralAttemptNotFresh);
                    }
                    let current_target = wifi::association_target()
                        .ok_or(ConnectionResult::EphemeralRevalidationFailed)?;
                    let authority = pending
                        .as_ref()
                        .ok_or(ConnectionResult::EphemeralAttemptNotFresh)?;
                    secret_vault::revalidate_ephemeral_physical_wifi_use(authority, current_target)
                        .map_err(|_| ConnectionResult::EphemeralRevalidationFailed)?;
                    let authority = pending
                        .take()
                        .ok_or(ConnectionResult::EphemeralAttemptNotFresh)?;
                    let (len, consumed) = secret_vault::write_ephemeral_physical_wifi_pmk(
                        authority, seq, job.target, out,
                    )
                    .map_err(|_| ConnectionResult::EphemeralRevalidationFailed)?;
                    *receipt = Some(consumed);
                    Ok(len)
                }
                ConnectionSecretSource::Ordinary => match secret_vault::wifi_status() {
                    secret_vault::VaultSecretStatus::Available { .. } => {
                        secret_vault::write_wifi_pmk_for_association(
                            seq,
                            job.target.ssid.as_bytes(),
                            job.target.bssid,
                            out,
                        )
                        .map_err(|denied| {
                            serial::write_fmt(format_args!(
                                "VAULT_WIFI_USE_REJECTED reason={}\r\n",
                                denied.wifi_use_reason()
                            ));
                            ConnectionResult::PassphraseUnavailable
                        })
                    }
                    secret_vault::VaultSecretStatus::Missing => {
                        wifi::format_legacy_supplicant_pmk_set(
                            seq,
                            job.target.bssid,
                            job.target.ssid.as_bytes(),
                            out,
                        )
                        .map_err(ConnectionResult::SupplicantBuild)
                    }
                    secret_vault::VaultSecretStatus::Forgotten { .. } => {
                        serial::write_line("VAULT_WIFI_USE_REJECTED reason=secret_forgotten");
                        Err(ConnectionResult::PassphraseUnavailable)
                    }
                },
            },
            ConnectionStage::PostPmkHwSpecCanary => marvell_wifi_cmd::build_get_hw_spec(seq, out)
                .map_err(|_| ConnectionResult::CommandBuild(MarvellCmdError::BadLength)),
            ConnectionStage::Associate => {
                let security_ie = if job.target.security_ie().is_empty() {
                    None
                } else {
                    Some(job.target.security_ie())
                };
                marvell_wifi_cmd::build_associate_24ghz(
                    seq,
                    marvell_wifi_cmd::AssociationBss {
                        bssid: job.target.bssid,
                        ssid: job.target.ssid.as_bytes(),
                        channel: job.target.channel,
                        beacon_period: job.target.beacon_period,
                        capability_info: job.target.capability_info,
                        rates: job.target.rates(),
                        rsn_or_wpa_ie: security_ie,
                    },
                    out,
                )
                .map_err(ConnectionResult::CommandBuild)
            }
            _ => Err(ConnectionResult::CommandBuild(MarvellCmdError::BadLength)),
        }
    }
}

/// Captures one secret-free request predicate before publication. The view is
/// exactly the six fixed protocol words (12 bytes); payload/TLV bytes are never
/// read, copied, compared, or retained by this diagnostic.
fn connection_request_header_matches_expected(job: &ConnectionJob, command_len: usize) -> bool {
    if command_len < marvell_wifi_cmd::HOST_CMD_MIN_RESPONSE_LEN
        || command_len > CONNECT_CMD_BUFFER_SIZE
        || command_len > u16::MAX as usize
    {
        return false;
    }
    let Some(expected_command) = connection_phase_command(job.phase) else {
        return false;
    };
    let Some(expected_host_size) = command_len.checked_sub(marvell_wifi_cmd::INTF_HEADER_LEN)
    else {
        return false;
    };
    let header = read_fixed_connection_header(connect_cmd_ptr());
    header.interface_len as usize == command_len
        && header.interface_type == marvell_wifi_cmd::MWIFIEX_TYPE_CMD
        && header.command == expected_command
        && header.host_command_size as usize == expected_host_size
        && header.sequence == connection_phase_seq(job.seq, job.phase)
        && header.result == 0
}

/// Reads only the fixed redacted response header. The sole caller invokes this
/// after terminal_quiesce_and_cleanup_while_gated returned true; this helper is
/// never used as completion evidence and cannot replace current-epoch CMD_DONE.
fn connection_timeout_response_class_after_verified_cleanup(
    job: &ConnectionJob,
) -> ConnectionTimeoutResponseClass {
    // Verified terminal cleanup proved BME-off and cleared the mailbox before
    // this exact 12-byte redacted response projection is read.
    let header = read_fixed_connection_header(connect_rsp_ptr());
    if header.is_empty() {
        return ConnectionTimeoutResponseClass::UntouchedZero;
    }
    let expected_command = connection_phase_command(job.phase);
    let interface_len = usize::from(header.interface_len);
    let host_command_size = usize::from(header.host_command_size);
    if expected_command
        .is_some_and(|command| header.command == command | marvell_wifi_cmd::HOST_CMD_RET_BIT)
        && header.interface_type == marvell_wifi_cmd::MWIFIEX_TYPE_CMD
        && header.sequence == connection_phase_seq(job.seq, job.phase)
        && interface_len >= marvell_wifi_cmd::HOST_CMD_MIN_RESPONSE_LEN
        && interface_len <= MWIFIEX_UPLD_SIZE
        && host_command_size >= marvell_wifi_cmd::S_DS_GEN
        && interface_len == marvell_wifi_cmd::INTF_HEADER_LEN + host_command_size
    {
        ConnectionTimeoutResponseClass::ExpectedHeaderSeen
    } else {
        ConnectionTimeoutResponseClass::NonemptyMismatch
    }
}

/// Projects exactly three aligned volatile u32 words into the fixed six-word
/// redacted HostCmd header. connect_cmd_ptr/connect_rsp_ptr are 64-byte aligned,
/// and no offset at or beyond byte 12 is reachable through this helper.
fn read_fixed_connection_header(buffer: *mut u8) -> marvell_wifi_cmd::MarvellResponseHeader {
    let interface = read_reg(buffer, 0);
    let command_size = read_reg(buffer, 4);
    let sequence_result = read_reg(buffer, 8);
    marvell_wifi_cmd::MarvellResponseHeader {
        interface_len: interface as u16,
        interface_type: (interface >> 16) as u16,
        command: command_size as u16,
        host_command_size: (command_size >> 16) as u16,
        sequence: sequence_result as u16,
        result: (sequence_result >> 16) as u16,
    }
}

const fn connection_stage_timeout_code(stage: ConnectionStage) -> u32 {
    match stage {
        ConnectionStage::Idle => 0,
        ConnectionStage::SupplicantProfile => 1,
        ConnectionStage::SupplicantPmk => 2,
        ConnectionStage::PostPmkHwSpecCanary => 7,
        ConnectionStage::Associate => 3,
        ConnectionStage::WaitPortRelease => 4,
        ConnectionStage::LinkReady => 5,
        ConnectionStage::Failed => 6,
    }
}

fn connection_timeout_fingerprint(
    stage: ConnectionStage,
    expected_command: u16,
    published_command_len: u16,
    request_header_matches_expected: bool,
    cleanup_verified: bool,
    response_class: ConnectionTimeoutResponseClass,
) -> u32 {
    debug_assert!(expected_command <= 0x00ff);
    debug_assert!(published_command_len <= 0x03ff);
    (CONNECTION_TIMEOUT_FINGERPRINT_TAG << CONNECTION_TIMEOUT_FINGERPRINT_TAG_SHIFT)
        | (connection_stage_timeout_code(stage) << CONNECTION_TIMEOUT_STAGE_SHIFT)
        | (u32::from(expected_command) << CONNECTION_TIMEOUT_COMMAND_SHIFT)
        | (u32::from(published_command_len) << CONNECTION_TIMEOUT_COMMAND_LEN_SHIFT)
        | if request_header_matches_expected {
            CONNECTION_TIMEOUT_REQUEST_HEADER_MATCH
        } else {
            0
        }
        | if cleanup_verified {
            CONNECTION_TIMEOUT_CLEANUP_VERIFIED
        } else {
            0
        }
        | response_class as u32
}

/// Parses the H25 canary only after a proven current-epoch CMD_DONE and
/// verified terminal cleanup. Returned hardware data is deliberately dropped.
fn parse_post_pmk_hw_spec_canary_response(job: &ConnectionJob) -> Result<(), HwSpecCmdError> {
    unsafe {
        // SAFETY: the caller proved current CMD_DONE and terminal cleanup for
        // the dedicated connection response mailbox before entering here.
        let response = slice::from_raw_parts(connect_rsp_ptr().cast_const(), MWIFIEX_UPLD_SIZE);
        marvell_wifi_cmd::parse_hw_spec_response(connection_phase_seq(job.seq, job.phase), response)
            .map(|_| ())
    }
}

fn parse_connection_dma_response(
    job: &ConnectionJob,
) -> (
    Result<Option<marvell_wifi_cmd::AssociationResponse>, ConnectionResult>,
    Option<marvell_wifi_cmd::MarvellResponseHeader>,
) {
    unsafe {
        // SAFETY: a low baseline followed by the current CMD_DONE is observed
        // before parsing the fixed response DMA buffer under CONNECTION.
        let response = slice::from_raw_parts(connect_rsp_ptr().cast_const(), MWIFIEX_UPLD_SIZE);
        let header = marvell_wifi_cmd::redacted_response_header(response);
        if header.is_some_and(marvell_wifi_cmd::MarvellResponseHeader::is_empty) {
            return (
                Err(ConnectionResult::Transport(
                    ConnectionTransportError::EmptyResponseOnCommandDone,
                )),
                header,
            );
        }
        let seq = connection_phase_seq(job.seq, job.phase);
        let parsed = match job.phase {
            ConnectionStage::SupplicantProfile => {
                marvell_wifi_supplicant::parse_supplicant_profile_response(seq, response)
                    .map(|()| None)
                    .map_err(ConnectionResult::SupplicantResponse)
            }
            ConnectionStage::SupplicantPmk => {
                marvell_wifi_supplicant::parse_supplicant_pmk_response(seq, response)
                    .map(|()| None)
                    .map_err(ConnectionResult::SupplicantResponse)
            }
            ConnectionStage::Associate => marvell_wifi_cmd::parse_associate_response(seq, response)
                .map(Some)
                .map_err(ConnectionResult::CommandResponse),
            _ => Err(ConnectionResult::CommandResponse(
                MarvellCmdError::BadLength,
            )),
        };
        (parsed, header)
    }
}

fn connection_phase_seq(base: u8, phase: ConnectionStage) -> u16 {
    let offset: u16 = match phase {
        ConnectionStage::SupplicantProfile => 0,
        ConnectionStage::SupplicantPmk => 1,
        ConnectionStage::PostPmkHwSpecCanary => 2,
        ConnectionStage::Associate => 2,
        _ => 7,
    };
    u16::from(base) + offset
}

fn connection_phase_command(phase: ConnectionStage) -> Option<u16> {
    match phase {
        ConnectionStage::SupplicantProfile => Some(marvell_wifi_supplicant::SUPPLICANT_PROFILE_CMD),
        ConnectionStage::SupplicantPmk => Some(marvell_wifi_supplicant::SUPPLICANT_PMK_CMD),
        ConnectionStage::PostPmkHwSpecCanary => Some(marvell_wifi_cmd::GET_HW_SPEC_CMD),
        ConnectionStage::Associate => Some(marvell_wifi_cmd::ASSOCIATE_CMD),
        _ => None,
    }
}

fn connection_response_error_class(
    result: ConnectionResult,
) -> Option<ConnectionResponseErrorClass> {
    match result {
        ConnectionResult::Transport(ConnectionTransportError::EmptyResponseOnCommandDone) => {
            Some(ConnectionResponseErrorClass::EmptyResponseOnCommandDone)
        }
        ConnectionResult::CommandResponse(error) => Some(match error {
            MarvellCmdError::TooShort => ConnectionResponseErrorClass::TooShort,
            MarvellCmdError::BadLength => ConnectionResponseErrorClass::BadLength,
            MarvellCmdError::BadCommand { .. } => ConnectionResponseErrorClass::BadCommand,
            MarvellCmdError::BadSequence { .. } => ConnectionResponseErrorClass::BadSequence,
            MarvellCmdError::FwResult { .. } => ConnectionResponseErrorClass::FirmwareResult,
            _ => ConnectionResponseErrorClass::Unexpected,
        }),
        ConnectionResult::SupplicantResponse(error) => Some(match error {
            SupplicantError::ResponseTooShort => ConnectionResponseErrorClass::TooShort,
            SupplicantError::BadInterfaceLength => ConnectionResponseErrorClass::BadInterfaceLength,
            SupplicantError::BadInterfaceType { .. } => {
                ConnectionResponseErrorClass::BadInterfaceType
            }
            SupplicantError::BadHostCommandLength => {
                ConnectionResponseErrorClass::BadHostCommandLength
            }
            SupplicantError::BadCommand { .. } => ConnectionResponseErrorClass::BadCommand,
            SupplicantError::BadSequence { .. } => ConnectionResponseErrorClass::BadSequence,
            SupplicantError::FirmwareResult { .. } => ConnectionResponseErrorClass::FirmwareResult,
            _ => ConnectionResponseErrorClass::Unexpected,
        }),
        _ => None,
    }
}

/// Captures fixed response-header words only. This is called exclusively after
/// HOST_INTR_CMD_DONE; request DMA and response payload bytes never enter the
/// snapshot or diagnostics path.
fn connection_response_diagnostic(
    job: &ConnectionJob,
    result: ConnectionResult,
    header: Option<marvell_wifi_cmd::MarvellResponseHeader>,
) -> Option<ConnectionResponseDiagnostic> {
    let error_class = connection_response_error_class(result)?;
    let expected_command = connection_phase_command(job.phase)?;
    let header = header?;
    Some(ConnectionResponseDiagnostic {
        error_class,
        interface_len: header.interface_len,
        interface_type: header.interface_type,
        command: header.command,
        host_command_size: header.host_command_size,
        sequence: header.sequence,
        result: header.result,
        expected_command,
        expected_sequence: connection_phase_seq(job.seq, job.phase),
    })
}

fn write_connection_response_failure(
    phase: ConnectionStage,
    result: ConnectionResult,
    diagnostic: Option<ConnectionResponseDiagnostic>,
    host_int_status: u32,
) {
    if let Some(diagnostic) = diagnostic {
        serial::write_fmt(format_args!(
            "marvell wifi: connection response failed phase={} class={} intf_len=0x{:04x} intf_type=0x{:04x} cmd=0x{:04x} host_size=0x{:04x} seq=0x{:04x} result=0x{:04x} expected_cmd=0x{:04x} expected_seq=0x{:04x} host_int=0x{:08x}\r\n",
            phase.label(),
            diagnostic.error_class.label(),
            diagnostic.interface_len,
            diagnostic.interface_type,
            diagnostic.command,
            diagnostic.host_command_size,
            diagnostic.sequence,
            diagnostic.result,
            diagnostic.expected_command,
            diagnostic.expected_sequence,
            host_int_status,
        ));
    } else {
        serial::write_fmt(format_args!(
            "marvell wifi: connection command failed phase={} result={} host_int=0x{:08x}\r\n",
            phase.label(),
            result.label(),
            host_int_status,
        ));
    }
}

enum InitCommandResponse {
    Unit,
    HwSpec(marvell_wifi_cmd::HwSpec),
}

fn init_phase_seq(base: u8, phase: HwSpecStage) -> u16 {
    let offset = match phase {
        HwSpecStage::PcieDescDetails => 0,
        HwSpecStage::FuncInit => 1,
        HwSpecStage::GetHwSpec => 2,
        HwSpecStage::MacControl => 3,
        _ => INIT_HOST_CMD_PHASE_COUNT,
    };
    u16::from(base) + u16::from(offset)
}

fn next_init_phase(phase: HwSpecStage) -> HwSpecStage {
    match phase {
        HwSpecStage::PcieDescDetails => HwSpecStage::FuncInit,
        HwSpecStage::FuncInit => HwSpecStage::GetHwSpec,
        HwSpecStage::GetHwSpec => HwSpecStage::MacControl,
        HwSpecStage::MacControl => HwSpecStage::Ready,
        _ => HwSpecStage::Failed,
    }
}

fn parse_hw_spec_dma_response(
    phase: HwSpecStage,
    expected_seq: u16,
) -> (
    Result<InitCommandResponse, HwSpecResult>,
    Option<marvell_wifi_cmd::MarvellResponseHeader>,
) {
    unsafe {
        // SAFETY: the firmware has raised CMD_DONE before this is called, and
        // the response slice covers only the fixed mailbox response buffer.
        let response = slice::from_raw_parts(hw_spec_rsp_ptr().cast_const(), MWIFIEX_UPLD_SIZE);
        let header = marvell_wifi_cmd::redacted_response_header(response);
        if header.is_some_and(marvell_wifi_cmd::MarvellResponseHeader::is_empty) {
            return (Err(HwSpecResult::EmptyResponseOnCommandDone), header);
        }
        let parsed = match phase {
            HwSpecStage::PcieDescDetails => {
                marvell_wifi_cmd::parse_pcie_desc_details_response(expected_seq, response)
                    .map(|()| InitCommandResponse::Unit)
                    .map_err(HwSpecResult::InitResponse)
            }
            HwSpecStage::FuncInit => {
                marvell_wifi_cmd::parse_func_init_response(expected_seq, response)
                    .map(|()| InitCommandResponse::Unit)
                    .map_err(HwSpecResult::InitResponse)
            }
            HwSpecStage::GetHwSpec => {
                marvell_wifi_cmd::parse_hw_spec_response(expected_seq, response)
                    .map(InitCommandResponse::HwSpec)
                    .map_err(HwSpecResult::Response)
            }
            HwSpecStage::MacControl => {
                marvell_wifi_cmd::parse_mac_control_response(expected_seq, response)
                    .map(|()| InitCommandResponse::Unit)
                    .map_err(HwSpecResult::InitResponse)
            }
            _ => Err(HwSpecResult::InitResponse(MarvellCmdError::BadLength)),
        };
        (parsed, header)
    }
}

fn parse_scan_dma_response() -> Result<(u8, usize), HwSpecCmdError> {
    unsafe {
        // SAFETY: the firmware has raised CMD_DONE before this is called, and
        // the response slice covers only the fixed mailbox response buffer.
        let response = slice::from_raw_parts(scan_rsp_ptr().cast_const(), MWIFIEX_UPLD_SIZE);
        let mut frame = [0u8; 24 + MWIFIEX_UPLD_SIZE];
        let mut ingested = 0usize;
        let declared = marvell_wifi_cmd::visit_scan_response(response, |bss| {
            let frame_len = 36usize.saturating_add(bss.ies.len());
            if frame_len > frame.len() {
                return;
            }
            frame[..frame_len].fill(0);
            frame[0] = 0x80;
            frame[16..22].copy_from_slice(&bss.bssid);
            frame[24..36].copy_from_slice(bss.fixed_beacon_params);
            frame[36..frame_len].copy_from_slice(bss.ies);
            let rssi = i8::try_from(bss.rssi_dbm).ok();
            if wifi::ingest_scan_frame(&frame[..frame_len], wifi::ScanSource::LiveRadio, rssi)
                .is_ok()
            {
                ingested += 1;
            }
        })?;
        Ok((declared, ingested))
    }
}

fn parse_event_buffer(index: usize) -> Result<ParsedEvent, EventRingResult> {
    unsafe {
        // SAFETY: index is checked by poll_event_ring before parsing, and each
        // event buffer has the fixed EVENT_BUFFER_SIZE.
        let bytes = slice::from_raw_parts(event_data_ptr(index).cast_const(), EVENT_BUFFER_SIZE);
        let len = u16::from_le_bytes([bytes[0], bytes[1]]);
        let event_type = u16::from_le_bytes([bytes[2], bytes[3]]);
        if len == 0 {
            return Err(EventRingResult::PointerAdvancedEmptyBuffer);
        }
        if len as usize <= EVENT_HEADER_LEN || len as usize > EVENT_BUFFER_SIZE {
            return Err(EventRingResult::BadEventLength);
        }
        let cause_offset = marvell_wifi_cmd::INTF_HEADER_LEN;
        if (len as usize) < cause_offset + 4 {
            return Err(EventRingResult::BadEventLength);
        }
        let cause = u32::from_le_bytes([
            bytes[cause_offset],
            bytes[cause_offset + 1],
            bytes[cause_offset + 2],
            bytes[cause_offset + 3],
        ]);
        Ok(ParsedEvent {
            len,
            event_type,
            cause,
        })
    }
}

fn event_buffer_len(index: usize) -> u16 {
    unsafe {
        // SAFETY: caller passes a bounded event-ring index.
        let ptr = event_data_ptr(index);
        u16::from_le_bytes([ptr::read_volatile(ptr), ptr::read_volatile(ptr.add(1))])
    }
}

fn event_buffer_type(index: usize) -> u16 {
    unsafe {
        // SAFETY: caller passes a bounded event-ring index.
        let ptr = event_data_ptr(index);
        u16::from_le_bytes([
            ptr::read_volatile(ptr.add(2)),
            ptr::read_volatile(ptr.add(3)),
        ])
    }
}

fn rx_buffer_len(index: usize) -> u16 {
    unsafe {
        // SAFETY: caller passes a bounded RX-ring index.
        let ptr = rx_data_ptr(index);
        u16::from_le_bytes([ptr::read_volatile(ptr), ptr::read_volatile(ptr.add(1))])
    }
}

fn rx_buffer_type(index: usize) -> u16 {
    unsafe {
        // SAFETY: caller passes a bounded RX-ring index.
        let ptr = rx_data_ptr(index);
        u16::from_le_bytes([
            ptr::read_volatile(ptr.add(2)),
            ptr::read_volatile(ptr.add(3)),
        ])
    }
}

fn parse_rx_ethernet(index: usize) -> Option<RxPacket> {
    unsafe {
        // SAFETY: callers validate the RX ring index; firmware DMA is complete
        // before the write pointer makes this buffer visible.
        let bytes = slice::from_raw_parts(rx_data_ptr(index).cast_const(), RX_BUFFER_SIZE);
        let interface_len = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
        let interface_type = u16::from_le_bytes([bytes[2], bytes[3]]);
        if interface_type != 0
            || interface_len < DATA_INTERFACE_HEADER_LEN + RX_PD_LEN
            || interface_len > RX_BUFFER_SIZE
        {
            return None;
        }

        let rx_pd = DATA_INTERFACE_HEADER_LEN;
        let packet_len = u16::from_le_bytes([bytes[rx_pd + 2], bytes[rx_pd + 3]]) as usize;
        let packet_offset = u16::from_le_bytes([bytes[rx_pd + 4], bytes[rx_pd + 5]]) as usize;
        let packet_type = u16::from_le_bytes([bytes[rx_pd + 6], bytes[rx_pd + 7]]);
        let start = rx_pd.checked_add(packet_offset)?;
        let end = start.checked_add(packet_len)?;
        if packet_type != 0 || packet_len < 14 || end > interface_len || end > bytes.len() {
            return None;
        }

        let source = &bytes[start..end];
        let mut packet = RxPacket {
            len: 0,
            bytes: [0; MAX_ETHERNET_FRAME_SIZE],
        };
        if source.len() >= 22
            && source[14..17] == [0xaa, 0xaa, 0x03]
            && (source[17..20] == [0x00, 0x00, 0x00] || source[17..20] == [0x00, 0x00, 0xf8])
        {
            let converted_len = source.len().checked_sub(8)?;
            if converted_len > packet.bytes.len() {
                return None;
            }
            packet.bytes[..12].copy_from_slice(&source[..12]);
            packet.bytes[12..14].copy_from_slice(&source[20..22]);
            packet.bytes[14..converted_len].copy_from_slice(&source[22..]);
            packet.len = converted_len;
        } else {
            if source.len() > packet.bytes.len() {
                return None;
            }
            packet.bytes[..source.len()].copy_from_slice(source);
            packet.len = source.len();
        }
        Some(packet)
    }
}

fn prepare_tx_buffer(index: usize, frame: &[u8], total_len: usize) -> bool {
    let Some(data_phys) = tx_data_phys(index) else {
        return false;
    };
    unsafe {
        // SAFETY: the caller bounds index and total_len against the fixed TX
        // DMA block and owns this descriptor until firmware advances TX-RD.
        let data = slice::from_raw_parts_mut(tx_data_ptr(index), TX_BUFFER_SIZE);
        data[..total_len].fill(0);
        data[0..2].copy_from_slice(&(total_len as u16).to_le_bytes());
        data[2..4].copy_from_slice(&0u16.to_le_bytes());
        let tx_pd = DATA_INTERFACE_HEADER_LEN;
        data[tx_pd + 2..tx_pd + 4].copy_from_slice(&(frame.len() as u16).to_le_bytes());
        data[tx_pd + 4..tx_pd + 6].copy_from_slice(&(TX_PD_LEN as u16).to_le_bytes());
        data[DATA_INTERFACE_HEADER_LEN + TX_PD_LEN..total_len].copy_from_slice(frame);
        ptr::write(
            tx_desc_ptr(index),
            RxPfuBufDesc {
                flags: RX_DESC_FLAG_SOP | RX_DESC_FLAG_EOP,
                offset: 0,
                frag_len: total_len as u16,
                len: total_len as u16,
                paddr: data_phys,
                reserved: 0,
            },
        );
    }
    true
}

fn host_ring_is_full(host_write: DeviceRingPointer, device_read: DeviceRingPointer) -> bool {
    host_write.index == device_read.index && host_write.rollover != device_read.rollover
}

fn next_tx_wrptr(wrptr: u32) -> u32 {
    let next = wrptr.wrapping_add(TX_RING_STEP);
    if (next & TX_RING_MASK) == (TX_RING_COUNT as u32) << 16 {
        (next & TX_ROLLOVER_IND) ^ TX_ROLLOVER_IND
    } else {
        next
    }
}

fn device_ring_has_entry(device_write: DeviceRingPointer, host_read: DeviceRingPointer) -> bool {
    device_write.index != host_read.index || device_write.rollover == host_read.rollover
}

fn next_event_rdptr(rdptr: u32) -> u32 {
    let next = rdptr.wrapping_add(1);
    if (next & EVENT_RING_MASK) == EVENT_RING_COUNT as u32 {
        (next & EVENT_ROLLOVER_IND) ^ EVENT_ROLLOVER_IND
    } else {
        next
    }
}

fn next_rx_rdptr(rdptr: u32) -> u32 {
    let next = rdptr.wrapping_add(1);
    if (next & RX_RING_MASK) == RX_RING_COUNT as u32 {
        (next & RX_ROLLOVER_IND) ^ RX_ROLLOVER_IND
    } else {
        next
    }
}

fn arm_event_desc(index: usize) {
    if let Some(data_phys) = event_data_phys(index) {
        unsafe {
            // SAFETY: caller only passes an event-ring index already validated
            // against EVENT_RING_COUNT; ptr::write avoids forming references to
            // the packed descriptor.
            ptr::write(
                event_desc_ptr(index),
                EventBufDesc {
                    paddr: data_phys,
                    len: EVENT_BUFFER_SIZE as u16,
                    flags: 0,
                },
            );
        }
    }
}

fn arm_rx_desc(index: usize) {
    if let Some(data_phys) = rx_data_phys(index) {
        unsafe {
            // SAFETY: caller only passes an RX-ring index already validated
            // against RX_RING_COUNT; ptr::write avoids forming references to
            // the packed descriptor.
            ptr::write_bytes(rx_data_ptr(index), 0, RX_BUFFER_SIZE);
            ptr::write(
                rx_desc_ptr(index),
                RxPfuBufDesc {
                    flags: RX_DESC_FLAG_SOP | RX_DESC_FLAG_EOP,
                    offset: 0,
                    frag_len: RX_BUFFER_SIZE as u16,
                    len: RX_BUFFER_SIZE as u16,
                    paddr: data_phys,
                    reserved: 0,
                },
            );
        }
    }
}

fn write_hw_spec_failure(phase: HwSpecStage, result: HwSpecResult, host_int_status: u32) {
    match result {
        HwSpecResult::Response(HwSpecCmdError::BadCommand { got })
        | HwSpecResult::CommandBuild(HwSpecCmdError::BadCommand { got }) => {
            serial::write_fmt(format_args!(
                "marvell wifi: init failed stage={} result={} got=0x{:04x} host_int=0x{:08x}\r\n",
                phase.label(),
                result.label(),
                got,
                host_int_status
            ));
        }
        HwSpecResult::Response(HwSpecCmdError::FwResult { code })
        | HwSpecResult::CommandBuild(HwSpecCmdError::FwResult { code }) => {
            serial::write_fmt(format_args!(
                "marvell wifi: init failed stage={} result={} code=0x{:04x} host_int=0x{:08x}\r\n",
                phase.label(),
                result.label(),
                code,
                host_int_status
            ));
        }
        _ => {
            serial::write_fmt(format_args!(
                "marvell wifi: init failed stage={} result={} host_int=0x{:08x}\r\n",
                phase.label(),
                result.label(),
                host_int_status
            ));
        }
    }
}

fn write_scan_failure(result: ScanCmdResult, host_int_status: u32) {
    match result {
        ScanCmdResult::Response(HwSpecCmdError::BadCommand { got })
        | ScanCmdResult::CommandBuild(HwSpecCmdError::BadCommand { got }) => {
            serial::write_fmt(format_args!(
                "marvell wifi: scan failed: {} got=0x{:04x} host_int=0x{:08x}\r\n",
                result.label(),
                got,
                host_int_status
            ));
        }
        ScanCmdResult::Response(HwSpecCmdError::FwResult { code })
        | ScanCmdResult::CommandBuild(HwSpecCmdError::FwResult { code }) => {
            serial::write_fmt(format_args!(
                "marvell wifi: scan failed: {} code=0x{:04x} host_int=0x{:08x}\r\n",
                result.label(),
                code,
                host_int_status
            ));
        }
        _ => {
            serial::write_fmt(format_args!(
                "marvell wifi: scan failed: {} host_int=0x{:08x}\r\n",
                result.label(),
                host_int_status
            ));
        }
    }
}

fn ready_mmio_base() -> Option<usize> {
    let runtime = BRINGUP.lock();
    if runtime.snapshot.is_ready() {
        runtime.mmio_base
    } else {
        None
    }
}

fn stage_for_action(action: FwAction, phase: FwPhase) -> FirmwareStage {
    match action {
        FwAction::WriteBlock { .. } | FwAction::Retry { .. } => FirmwareStage::Downloading,
        FwAction::RingDoorbell | FwAction::PollDoorbellAck => FirmwareStage::DoorbellAck,
        FwAction::WriteDrvReady { .. } | FwAction::PollFwStatus => stage_for_phase(phase),
        FwAction::Done => FirmwareStage::Ready,
        FwAction::Fail(_) => FirmwareStage::Failed,
    }
}

fn stage_for_phase(phase: FwPhase) -> FirmwareStage {
    match phase {
        FwPhase::Downloading | FwPhase::BlockPrepared => FirmwareStage::Downloading,
        FwPhase::WaitingDoorbellAck => FirmwareStage::DoorbellAck,
        FwPhase::PollingReady => FirmwareStage::PollingReady,
        FwPhase::Done => FirmwareStage::Ready,
        FwPhase::Failed(_) => FirmwareStage::Failed,
    }
}

fn read_observed(mmio_base: *mut u8) -> RegisterReads {
    RegisterReads {
        cmd_size_reg: read_reg(mmio_base, CMD_SIZE),
        fw_status_reg: read_reg(mmio_base, FW_STATUS),
        int_status_reg: read_reg(mmio_base, PCIE_CPU_INT_STATUS),
    }
}

fn read_register_snapshot(mmio_base: *mut u8) -> FirmwareRegisterSnapshot {
    FirmwareRegisterSnapshot {
        valid: true,
        cmd_size: read_reg(mmio_base, CMD_SIZE),
        fw_status: read_reg(mmio_base, FW_STATUS),
        drv_ready: read_reg(mmio_base, DRV_READY),
        host_int_status: read_reg(mmio_base, PCIE_HOST_INT_STATUS),
    }
}

fn write_drv_ready_pre_quarantined(
    pci_address: pci::PciAddress,
    mmio_base: *mut u8,
    value: u32,
) -> bool {
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio_base, PCIE_HOST_INT_MASK, 0);
    write_reg(mmio_base, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
    let pending = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
    if pending != 0 && pending != u32::MAX {
        write_reg(mmio_base, PCIE_HOST_INT_STATUS, !pending);
    }
    if !terminal_quiesce_and_cleanup_while_gated(pci_address, mmio_base) {
        return false;
    }
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio_base, DRV_READY, value);
    compiler_fence(Ordering::SeqCst);
    read_reg(mmio_base, PCIE_HOST_INT_STATUS) != u32::MAX
}

fn probe_mmio(mmio_base: *mut u8) -> bool {
    let fw_status = read_reg(mmio_base, FW_STATUS);
    let dump_ctrl = read_reg(mmio_base, FW_DUMP_CTRL);
    fw_status != u32::MAX || dump_ctrl != u32::MAX
}

fn read_reg(mmio_base: *mut u8, offset: u32) -> u32 {
    unsafe {
        // SAFETY: mmio_base is only produced by memory::map_mmio for BAR2 after
        // chip-present and firmware-present gates pass; offsets are fixed
        // 32-bit registers from the pure firmware module.
        ptr::read_volatile(mmio_base.add(offset as usize).cast::<u32>())
    }
}

fn write_reg(mmio_base: *mut u8, offset: u32, value: u32) {
    unsafe {
        // SAFETY: same BAR2 mapping invariant as read_reg; writes come only
        // from the firmware sequencer or its immediate quarantine boundary.
        ptr::write_volatile(mmio_base.add(offset as usize).cast::<u32>(), value);
    }
}

fn dma_block_phys() -> Option<u64> {
    memory::virt_to_phys(dma_block_ptr().cast_const())
}

fn hw_spec_cmd_phys() -> Option<u64> {
    memory::virt_to_phys(hw_spec_cmd_ptr().cast_const())
}

fn hw_spec_rsp_phys() -> Option<u64> {
    memory::virt_to_phys(hw_spec_rsp_ptr().cast_const())
}

fn scan_cmd_phys() -> Option<u64> {
    memory::virt_to_phys(scan_cmd_ptr().cast_const())
}

fn scan_rsp_phys() -> Option<u64> {
    memory::virt_to_phys(scan_rsp_ptr().cast_const())
}

fn connect_cmd_phys() -> Option<u64> {
    memory::virt_to_phys(connect_cmd_ptr().cast_const())
}

fn connect_rsp_phys() -> Option<u64> {
    memory::virt_to_phys(connect_rsp_ptr().cast_const())
}

fn event_data_phys(index: usize) -> Option<u64> {
    memory::virt_to_phys(event_data_ptr(index).cast_const())
}

fn event_desc_phys() -> Option<u64> {
    memory::virt_to_phys(event_desc_ptr(0).cast_const())
}

fn rx_data_phys(index: usize) -> Option<u64> {
    memory::virt_to_phys(rx_data_ptr(index).cast_const())
}

fn rx_desc_phys() -> Option<u64> {
    memory::virt_to_phys(rx_desc_ptr(0).cast_const())
}

fn tx_data_phys(index: usize) -> Option<u64> {
    memory::virt_to_phys(tx_data_ptr(index).cast_const())
}

fn tx_desc_phys() -> Option<u64> {
    memory::virt_to_phys(tx_desc_ptr(0).cast_const())
}

fn copy_block_into_dma(src: &[u8], wire_len: usize) -> Result<(), FirmwareDownloadResult> {
    if src.len() > wire_len || wire_len > FW_DMA_STAGING_SIZE {
        return Err(FirmwareDownloadResult::BlockLenOutOfRange);
    }

    let dst = dma_block_ptr();
    unsafe {
        // SAFETY: DMA_BLOCK is this driver's single firmware bounce buffer,
        // sized to FW_DMA_STAGING_SIZE; src/wire lengths are checked above.
        ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
        ptr::write_bytes(dst.add(src.len()), 0, wire_len - src.len());
    }
    compiler_fence(Ordering::SeqCst);
    Ok(())
}

fn elapsed_ms(start_tsc: u64) -> u64 {
    time::rdtsc().wrapping_sub(start_tsc) / time::tsc_per_ms().max(1)
}

fn delay_us(us: u64) {
    let ticks = time::tsc_per_ms().saturating_mul(us).saturating_add(999) / 1_000;
    delay_ticks(ticks.max(1));
}

fn delay_ticks(ticks: u64) {
    let start = time::rdtsc();
    while time::rdtsc().wrapping_sub(start) < ticks {
        spin_loop();
    }
}

fn dma_block_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access to this static bounce buffer stays inside this I/O shell.
        ptr::addr_of_mut!(DMA_BLOCK.0).cast::<u8>()
    }
}

fn hw_spec_cmd_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access is serialized by HWSPEC and bounded by HWSPEC_CMD_BUFFER_SIZE.
        ptr::addr_of_mut!(HWSPEC_DMA_BLOCK.cmd).cast::<u8>()
    }
}

fn hw_spec_rsp_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access is serialized by HWSPEC and bounded by MWIFIEX_UPLD_SIZE.
        ptr::addr_of_mut!(HWSPEC_DMA_BLOCK.rsp).cast::<u8>()
    }
}

fn scan_cmd_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access is serialized by SCAN and bounded by SCAN_CMD_BUFFER_SIZE.
        ptr::addr_of_mut!(SCAN_DMA_BLOCK.cmd).cast::<u8>()
    }
}

fn scan_rsp_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access is serialized by SCAN and bounded by MWIFIEX_UPLD_SIZE.
        ptr::addr_of_mut!(SCAN_DMA_BLOCK.rsp).cast::<u8>()
    }
}

fn connect_cmd_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer creates no reference; CONNECTION
        // serializes and bounds all access to this command buffer.
        ptr::addr_of_mut!(CONNECT_DMA_BLOCK.cmd).cast::<u8>()
    }
}

fn connect_rsp_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: same serialization and fixed-buffer invariant as above.
        ptr::addr_of_mut!(CONNECT_DMA_BLOCK.rsp).cast::<u8>()
    }
}

fn event_desc_ptr(index: usize) -> *mut EventBufDesc {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access is serialized by EVENT_RING and index callers are bounded.
        ptr::addr_of_mut!(EVENT_RING_DMA_BLOCK.desc)
            .cast::<EventBufDesc>()
            .add(index)
    }
}

fn event_data_ptr(index: usize) -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access is serialized by EVENT_RING and index callers are bounded.
        ptr::addr_of_mut!(EVENT_RING_DMA_BLOCK.data)
            .cast::<u8>()
            .add(index * EVENT_BUFFER_SIZE)
    }
}

fn rx_desc_ptr(index: usize) -> *mut RxPfuBufDesc {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; RX
        // setup bounds every index against RX_RING_COUNT.
        ptr::addr_of_mut!(RX_RING_DMA_BLOCK.desc)
            .cast::<RxPfuBufDesc>()
            .add(index)
    }
}

fn rx_data_ptr(index: usize) -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; RX
        // setup bounds every index against RX_RING_COUNT.
        ptr::addr_of_mut!(RX_RING_DMA_BLOCK.data)
            .cast::<u8>()
            .add(index * RX_BUFFER_SIZE)
    }
}

fn tx_desc_ptr(index: usize) -> *mut RxPfuBufDesc {
    unsafe {
        // SAFETY: returning a raw pointer creates no reference; callers bound
        // every index against TX_RING_COUNT while holding TX_RING.
        ptr::addr_of_mut!(TX_RING_DMA_BLOCK.desc)
            .cast::<RxPfuBufDesc>()
            .add(index)
    }
}

fn tx_data_ptr(index: usize) -> *mut u8 {
    unsafe {
        // SAFETY: same fixed-block and index invariant as tx_desc_ptr.
        ptr::addr_of_mut!(TX_RING_DMA_BLOCK.data)
            .cast::<u8>()
            .add(index * TX_BUFFER_SIZE)
    }
}
