//! Marvell 88W8897 PCIe firmware-download hardware shell.
//!
//! Hardware bring-up is owner-triggered only. It is honestly unaudited: the
//! proprietary firmware blob and DMA path are trusted until raiOS has IOMMU
//! enforcement.

use core::hint::spin_loop;
use core::ptr;
use core::slice;
use core::sync::atomic::{compiler_fence, AtomicBool, AtomicU32, Ordering};

use raios_core::boot_control::BootPosture;
use raios_core::dot11_scan::Dot11Security;
use raios_core::hw_failure_trace::{
    HwFailurePhase, HwFailureRegister, HwFailureStatus, HwFailureSubsystem, HwFailureTrace,
    HwFailureTraceStep, SEED_KERNEL_BUILD_ID_V0_1_0,
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

use crate::{memory, net, pci, secret_vault, serial, time, usb, wifi};

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
const PCIE_RX_RD_PTR: u32 = 0xC05C;
const PCIE_RX_WR_PTR: u32 = 0xC08C;
const PCIE_TX_RD_PTR: u32 = 0xC08C;
const PCIE_TX_WR_PTR: u32 = 0xC05C;
const RX_DESC_FLAG_SOP: u16 = 1 << 0;
const RX_DESC_FLAG_EOP: u16 = 1 << 1;
const CPU_INTR_DNLD_RDY: u32 = 1 << 0;
const HOST_INTR_DNLD_DONE: u32 = 1 << 0;
const EVENT_RING_COUNT: usize = 8;
const EVENT_RING_MASK: u32 = 0x0f;
const EVENT_ROLLOVER_IND: u32 = 1 << 7;
const EVENT_BUFFER_SIZE: usize = 2048;
const EVENT_HEADER_LEN: usize = 4;
const PCIE_EVT_RD_PTR: u32 = 0xCE8;
const PCIE_EVT_WR_PTR: u32 = 0xCEC;
const CPU_INTR_EVENT_DONE: u32 = 1 << 5;

fn queue_fixed_hw_failure_trace(
    phase: HwFailurePhase,
    status: HwFailureStatus,
    register: HwFailureRegister,
    register_value: u32,
) {
    let mut trace = HwFailureTrace::new(
        SEED_KERNEL_BUILD_ID_V0_1_0,
        HwFailureSubsystem::MarvellWifiPcie,
    );
    let boot_ms = (time::rdtsc() / time::tsc_per_ms().max(1)).min(u64::from(u32::MAX)) as u32;
    if trace
        .push_step(HwFailureTraceStep {
            boot_ms,
            phase,
            status,
            register,
            register_value,
        })
        .is_ok()
    {
        let _ = usb::queue_hw_failure_trace(trace);
    }
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
static RX_RDPTR_SHARED: AtomicU32 = AtomicU32::new(RX_ROLLOVER_IND);
static TX_WRPTR_SHARED: AtomicU32 = AtomicU32::new(0);
static DATA_LINK_READY: AtomicBool = AtomicBool::new(false);
static CONNECTION_REBOOT_REQUIRED: AtomicBool = AtomicBool::new(false);

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
    Associate,
    WaitPortRelease,
    LinkReady,
    Failed,
}

impl ConnectionStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::SupplicantProfile => "supplicant_profile",
            Self::SupplicantPmk => "supplicant_pmk",
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
}

struct RxRingRuntime {
    snapshot: RxRingSnapshot,
    mmio_base: Option<usize>,
    rdptr: u32,
}

struct TxRingRuntime {
    armed: bool,
    mmio_base: Option<usize>,
    wrptr: u32,
    rdptr: u32,
}

impl EventRingRuntime {
    const fn new() -> Self {
        Self {
            snapshot: EventRingSnapshot::new(),
            mmio_base: None,
            rdptr: EVENT_ROLLOVER_IND,
        }
    }
}

impl RxRingRuntime {
    const fn new() -> Self {
        Self {
            snapshot: RxRingSnapshot::new(),
            mmio_base: None,
            rdptr: RX_ROLLOVER_IND,
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
    if !arm_data_rings(mmio_base) {
        return fail_connection_start(ConnectionResult::DataRingUnavailable);
    }
    let (Some(cmd_dma_phys), Some(rsp_dma_phys)) = (connect_cmd_phys(), connect_rsp_phys()) else {
        return fail_connection_start(ConnectionResult::DmaAddressUnavailable);
    };

    let mut runtime = CONNECTION.lock();
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
        phase_started_tsc: time::rdtsc(),
        transport_diagnostic: ConnectionTransportDiagnostic::new(),
        seq,
        target,
        secret_source,
    });
    runtime.ready_target = None;
    DATA_LINK_READY.store(false, Ordering::Release);
    drop(runtime);
    if replacing_ready {
        net::detach_wifi();
    }
    serial::write_line("marvell wifi: bounded association sequence started");
    ConnectionTriggerResult::Started
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
    if let Some(address) = wifi::snapshot().address {
        pci::disable_bus_master(address);
    }
    DATA_LINK_READY.store(false, Ordering::Release);
    net::detach_wifi();
    let mut runtime = CONNECTION.lock();
    if runtime.snapshot.is_ready() {
        runtime.snapshot = ConnectionSnapshot::new();
        runtime.ready_target = None;
    }
}

fn fail_connection_start(result: ConnectionResult) -> ConnectionTriggerResult {
    if let Some(address) = wifi::snapshot().address {
        pci::disable_bus_master(address);
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
    drop(runtime);
    net::detach_wifi();
    ConnectionTriggerResult::Failed(result)
}

pub fn receive_ethernet() -> Option<RxPacket> {
    if !data_link_ready() {
        return None;
    }

    let mut runtime = RX_RING.lock();
    if !runtime.snapshot.armed {
        return None;
    }
    let mmio_base = runtime.mmio_base? as *mut u8;
    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
    let wrptr = read_reg(mmio_base, PCIE_RX_WR_PTR);
    if !rx_ring_has_entry(wrptr, runtime.rdptr) {
        if status & HOST_INTR_UPLD_RDY != 0 {
            write_reg(mmio_base, PCIE_HOST_INT_STATUS, !HOST_INTR_UPLD_RDY);
        }
        return None;
    }

    let index = (runtime.rdptr & RX_RING_MASK) as usize;
    if index >= RX_RING_COUNT {
        runtime.snapshot.stage = RxRingStage::Failed;
        runtime.snapshot.result = Some(RxRingResult::BadReadPointer);
        DATA_LINK_READY.store(false, Ordering::Release);
        return None;
    }

    let packet = parse_rx_ethernet(index);
    let next_rdptr = next_rx_rdptr(runtime.rdptr);
    arm_rx_desc(index);
    compiler_fence(Ordering::SeqCst);
    runtime.rdptr = next_rdptr;
    RX_RDPTR_SHARED.store(next_rdptr, Ordering::Release);
    let tx_wrptr = TX_WRPTR_SHARED.load(Ordering::Acquire) & TX_RING_WRAP_MASK;
    write_reg(
        mmio_base,
        PCIE_RX_RD_PTR,
        tx_wrptr | (next_rdptr & 0x0000_07ff),
    );
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
    if !data_link_ready() || frame.is_empty() || frame.len() > MAX_ETHERNET_FRAME_SIZE {
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
    if status & HOST_INTR_DNLD_DONE != 0 {
        write_reg(mmio, PCIE_HOST_INT_STATUS, !HOST_INTR_DNLD_DONE);
    }
    runtime.rdptr = read_reg(mmio, PCIE_TX_RD_PTR) & TX_RING_WRAP_MASK;
    if tx_ring_full(runtime.wrptr, runtime.rdptr) {
        return false;
    }

    let index = ((runtime.wrptr & TX_RING_MASK) >> 16) as usize;
    if index >= TX_RING_COUNT {
        DATA_LINK_READY.store(false, Ordering::Release);
        return false;
    }
    let total_len = DATA_INTERFACE_HEADER_LEN + TX_PD_LEN + frame.len();
    if total_len > TX_BUFFER_SIZE || !prepare_tx_buffer(index, frame, total_len) {
        return false;
    }

    let next_wrptr = next_tx_wrptr(runtime.wrptr);
    runtime.wrptr = next_wrptr;
    TX_WRPTR_SHARED.store(next_wrptr, Ordering::Release);
    let rx_rdptr = RX_RDPTR_SHARED.load(Ordering::Acquire) & 0x0000_07ff;
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio, PCIE_TX_WR_PTR, next_wrptr | rx_rdptr);
    let _ = read_reg(mmio, 0);
    write_reg(mmio, PCIE_CPU_INT_EVENT, CPU_INTR_DNLD_RDY);
    compiler_fence(Ordering::SeqCst);
    true
}

pub fn start_bring_up_firmware() -> FirmwareBringupTriggerResult {
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

    pci::enable_bus_master(address);
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
    });
    serial::write_line(
        "marvell wifi: firmware bring-up ATTEMPT started (unaudited blob; DMA not IOMMU-confined)",
    );
    FirmwareBringupTriggerResult::Started
}

pub fn start_scan_ext_24ghz() -> ScanCmdTriggerResult {
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
            pci::disable_bus_master(job.pci_address);
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
            let command_len = match prepare_hw_spec_dma(&job, phase) {
                Ok(command_len) => command_len,
                Err(result) => {
                    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                    pci::disable_bus_master(job.pci_address);
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

            let vendor_device = job.pci_address.read_u32(0x00);
            let command_before = job.pci_address.read_u16(0x04);
            runtime.snapshot.pci_command_before = command_before;
            let pci = match ensure_pci_memory_bus_master(
                job.pci_address,
                vendor_device,
                command_before,
            ) {
                Ok(pci) => pci,
                Err(error) => {
                    let result = match error {
                        PciCommandEnableError::FunctionUnavailable => {
                            HwSpecResult::PciFunctionUnavailable
                        }
                        PciCommandEnableError::EnableFailed => HwSpecResult::PciCommandEnableFailed,
                    };
                    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                    pci::disable_bus_master(job.pci_address);
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
            runtime.snapshot.pci_command_before = pci.command_before;
            runtime.snapshot.pci_command_after = pci.command_after;

            runtime.snapshot.pci_vendor_device = pci.vendor_device;
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
                pci::disable_bus_master(job.pci_address);
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

            compiler_fence(Ordering::SeqCst);
            write_reg(mmio_base, PCIE_HOST_INT_MASK, 0);
            write_reg(mmio_base, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
            if pending != 0 && pending != u32::MAX {
                write_reg(mmio_base, PCIE_HOST_INT_STATUS, !pending);
            }
            write_reg(
                mmio_base,
                CMDRSP_ADDR_LO,
                (job.rsp_dma_phys & 0xffff_ffff) as u32,
            );
            write_reg(mmio_base, CMDRSP_ADDR_HI, (job.rsp_dma_phys >> 32) as u32);
            write_reg(
                mmio_base,
                CMD_ADDR_LO,
                (job.cmd_dma_phys & 0xffff_ffff) as u32,
            );
            write_reg(mmio_base, CMD_ADDR_HI, (job.cmd_dma_phys >> 32) as u32);
            write_reg(mmio_base, CMD_SIZE, command_len as u32);
            compiler_fence(Ordering::SeqCst);
            write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_DOOR_BELL);
            compiler_fence(Ordering::SeqCst);

            let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
            runtime.snapshot.host_int_status = status;
            job.waiting = true;
            job.phase_started_tsc = time::rdtsc();
            serial::write_fmt(format_args!(
                    "marvell wifi: init stage={} armed seq={} pci={:04x}>{:04x} fw=0x{:08x} host=0x{:08x}\r\n",
                    phase.label(),
                    init_phase_seq(job.seq, phase),
                    pci.command_before,
                    pci.command_after,
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
                pci::disable_bus_master(job.pci_address);
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
                compiler_fence(Ordering::SeqCst);
                let (parsed, response_header) =
                    parse_hw_spec_dma_response(phase, init_phase_seq(job.seq, phase));
                runtime.snapshot.last_response = response_header;
                write_reg(mmio_base, PCIE_HOST_INT_STATUS, !status);
                clear_connection_response_mailbox(mmio_base);
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
                            publish_data_ring_pointers(job.mmio_base);
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
                        pci::disable_bus_master(job.pci_address);
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
            pci::disable_bus_master(job.pci_address);
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
                let command_len = match prepare_scan_dma(job.seq) {
                    Ok(command_len) => command_len,
                    Err(error) => {
                        let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                        let result = ScanCmdResult::CommandBuild(error);
                        finish_scan_locked(&mut runtime, result, ScanCmdStage::Failed, status, 0);
                        write_scan_failure(result, status);
                        wifi::note_scan_command_failed();
                        return true;
                    }
                };

                compiler_fence(Ordering::SeqCst);
                write_reg(mmio_base, PCIE_HOST_INT_MASK, 0);
                write_reg(mmio_base, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
                let pending = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                if pending != 0 && pending != u32::MAX {
                    write_reg(mmio_base, PCIE_HOST_INT_STATUS, !pending);
                }
                write_reg(
                    mmio_base,
                    CMDRSP_ADDR_LO,
                    (job.rsp_dma_phys & 0xffff_ffff) as u32,
                );
                write_reg(mmio_base, CMDRSP_ADDR_HI, (job.rsp_dma_phys >> 32) as u32);
                write_reg(
                    mmio_base,
                    CMD_ADDR_LO,
                    (job.cmd_dma_phys & 0xffff_ffff) as u32,
                );
                write_reg(mmio_base, CMD_ADDR_HI, (job.cmd_dma_phys >> 32) as u32);
                write_reg(mmio_base, CMD_SIZE, command_len as u32);
                compiler_fence(Ordering::SeqCst);
                pci::enable_bus_master(job.pci_address);
                compiler_fence(Ordering::SeqCst);
                write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_DOOR_BELL);
                compiler_fence(Ordering::SeqCst);

                let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
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
                    compiler_fence(Ordering::SeqCst);
                    let parsed = parse_scan_dma_response();
                    write_reg(mmio_base, PCIE_HOST_INT_STATUS, !status);
                    clear_connection_response_mailbox(mmio_base);
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
                                pci::disable_bus_master(job.pci_address);
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
                            pci::disable_bus_master(job.pci_address);
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

fn clear_connection_response_mailbox(mmio: *mut u8) {
    write_reg(mmio, CMDRSP_ADDR_LO, 0);
    write_reg(mmio, CMDRSP_ADDR_HI, 0);
    compiler_fence(Ordering::SeqCst);
}

fn ensure_pci_memory_bus_master(
    address: pci::PciAddress,
    vendor_device: u32,
    command_before: u16,
) -> Result<PciCommandEnableOutcome, PciCommandEnableError> {
    match marvell_wifi_cmd::plan_pci_memory_bus_master_enable(vendor_device, command_before) {
        marvell_wifi_cmd::PciCommandEnablePlan::Unavailable => {
            Err(PciCommandEnableError::FunctionUnavailable)
        }
        marvell_wifi_cmd::PciCommandEnablePlan::AlreadyEnabled => Ok(PciCommandEnableOutcome {
            vendor_device,
            command_before,
            command_after: command_before,
        }),
        marvell_wifi_cmd::PciCommandEnablePlan::Write(command) => {
            let readback =
                match address.write_command_u16_checked(vendor_device, command_before, command) {
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
    }
}

fn arm_connection_command(
    job: &mut ConnectionJob,
    command_len: usize,
) -> Result<(), ConnectionTransportError> {
    let mmio = job.mmio_base as *mut u8;
    job.cmd_done_low_baseline = false;
    job.transport_diagnostic = ConnectionTransportDiagnostic::new();

    let vendor_device = job.pci_address.read_u32(0x00);
    let command_before = job.pci_address.read_u16(0x04);
    job.transport_diagnostic.pci_vendor_device = vendor_device;
    job.transport_diagnostic.pci_command_before = command_before;
    job.transport_diagnostic.pci_config_valid =
        vendor_device != u32::MAX && command_before != u16::MAX;
    job.transport_diagnostic.pre_enable_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    job.transport_diagnostic.pre_enable_status_valid = true;
    let pci = ensure_pci_memory_bus_master(job.pci_address, vendor_device, command_before)
        .map_err(|error| match error {
            PciCommandEnableError::FunctionUnavailable => {
                ConnectionTransportError::PciFunctionUnavailable
            }
            PciCommandEnableError::EnableFailed => ConnectionTransportError::PciCommandEnableFailed,
        })?;
    job.transport_diagnostic.pci_vendor_device = pci.vendor_device;
    job.transport_diagnostic.pci_command_before = pci.command_before;
    job.transport_diagnostic.pci_command_after = pci.command_after;
    job.transport_diagnostic.pci_config_valid = true;
    compiler_fence(Ordering::SeqCst);

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

    compiler_fence(Ordering::SeqCst);
    write_reg(mmio, PCIE_HOST_INT_MASK, 0);
    write_reg(mmio, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);

    // Runtime HostCmd scratch registers do not promise read-your-write. Clear
    // response ownership blindly and use the known-readable status register as
    // the same-device posted-write flush.
    clear_connection_response_mailbox(mmio);

    let pending = read_reg(mmio, PCIE_HOST_INT_STATUS);
    job.transport_diagnostic.pre_clear_status = pending;
    job.transport_diagnostic.pre_clear_status_valid = true;
    if pending == u32::MAX {
        return Err(ConnectionTransportError::MmioUnavailable);
    }
    if pending != 0 && pending != u32::MAX {
        // The status register is W0C on this device: invert the observed bits
        // so every currently pending source receives a zero acknowledgement.
        write_reg(mmio, PCIE_HOST_INT_STATUS, !pending);
    }
    let mut post_clear_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    for _ in 1..CONNECTION_CMD_DONE_CLEAR_POLLS {
        if marvell_wifi_cmd::host_cmd_done_low_after_clear(post_clear_status, HOST_INTR_CMD_DONE) {
            break;
        }
        delay_us(SHORT_POLL_DELAY_US);
        post_clear_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    }
    job.transport_diagnostic.post_clear_status = post_clear_status;
    job.transport_diagnostic.post_clear_status_valid = true;
    if post_clear_status == u32::MAX {
        return Err(ConnectionTransportError::MmioUnavailable);
    }
    if !marvell_wifi_cmd::host_cmd_done_low_after_clear(post_clear_status, HOST_INTR_CMD_DONE) {
        return Err(ConnectionTransportError::StaleCommandDone);
    }
    job.cmd_done_low_baseline = true;

    write_reg(
        mmio,
        CMDRSP_ADDR_LO,
        (job.rsp_dma_phys & 0xffff_ffff) as u32,
    );
    write_reg(mmio, CMDRSP_ADDR_HI, (job.rsp_dma_phys >> 32) as u32);
    write_reg(mmio, CMD_ADDR_LO, (job.cmd_dma_phys & 0xffff_ffff) as u32);
    write_reg(mmio, CMD_ADDR_HI, (job.cmd_dma_phys >> 32) as u32);
    write_reg(mmio, CMD_SIZE, command_len as u32);
    compiler_fence(Ordering::SeqCst);
    let program_flush_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    job.transport_diagnostic.program_flush_status = program_flush_status;
    job.transport_diagnostic.program_flush_status_valid = true;
    if program_flush_status == u32::MAX {
        return Err(ConnectionTransportError::MmioUnavailable);
    }
    if program_flush_status & HOST_INTR_CMD_DONE != 0 {
        return Err(ConnectionTransportError::StaleCommandDone);
    }

    write_reg(mmio, PCIE_CPU_INT_EVENT, CPU_INTR_DOOR_BELL);
    compiler_fence(Ordering::SeqCst);
    job.transport_diagnostic.first_poll_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    job.transport_diagnostic.first_poll_status_valid = true;
    Ok(())
}

pub fn poll_connection() -> bool {
    let mut runtime = CONNECTION.lock();
    if matches!(
        runtime.snapshot.stage,
        ConnectionStage::Idle | ConnectionStage::LinkReady | ConnectionStage::Failed
    ) {
        return false;
    }
    let Some(mut job) = runtime.job.take() else {
        return false;
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
            return true;
        }
        runtime.job = Some(job);
        return false;
    }

    if elapsed_ms(job.phase_started_tsc) >= CONNECT_CMD_TIMEOUT_MS {
        let status = read_reg(mmio, PCIE_HOST_INT_STATUS);
        quarantine_connection_job(&job);
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
        return true;
    }

    if !job.waiting {
        let command_len = match prepare_connection_dma(&mut job) {
            Ok(len) => len,
            Err(result) => {
                quarantine_connection_job(&job);
                finish_connection_locked(
                    &mut runtime,
                    result,
                    ConnectionStage::Failed,
                    read_reg(mmio, PCIE_HOST_INT_STATUS),
                );
                DATA_LINK_READY.store(false, Ordering::Release);
                return true;
            }
        };
        if let Err(error) = arm_connection_command(&mut job, command_len) {
            let status = job.transport_diagnostic.post_clear_status;
            runtime.snapshot.transport_diagnostic = Some(job.transport_diagnostic);
            quarantine_connection_job(&job);
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
            return true;
        }
        job.waiting = true;
        job.phase_started_tsc = time::rdtsc();
        runtime.snapshot.stage = job.phase;
        runtime.snapshot.host_int_status = job.transport_diagnostic.first_poll_status;
        runtime.snapshot.transport_diagnostic = Some(job.transport_diagnostic);
        runtime.job = Some(job);
        return true;
    }

    let status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    runtime.snapshot.host_int_status = status;
    job.transport_diagnostic.poll_count = job.transport_diagnostic.poll_count.saturating_add(1);
    runtime.snapshot.transport_diagnostic = Some(job.transport_diagnostic);
    if status == u32::MAX {
        quarantine_connection_job(&job);
        finish_connection_locked(
            &mut runtime,
            ConnectionResult::Transport(ConnectionTransportError::MmioUnavailable),
            ConnectionStage::Failed,
            status,
        );
        serial::write_line("marvell wifi: HOST_INT_STATUS unavailable while awaiting CMD_DONE");
        return true;
    }
    if !marvell_wifi_cmd::host_cmd_done_is_current(
        job.cmd_done_low_baseline,
        status,
        HOST_INTR_CMD_DONE,
    ) {
        if status & HOST_INTR_CMD_DONE != 0 {
            quarantine_connection_job(&job);
            finish_connection_locked(
                &mut runtime,
                ConnectionResult::Transport(ConnectionTransportError::StaleCommandDone),
                ConnectionStage::Failed,
                status,
            );
            serial::write_line(
                "marvell wifi: CMD_DONE rejected without a proven current low baseline",
            );
            return true;
        }
        runtime.job = Some(job);
        return false;
    }

    compiler_fence(Ordering::SeqCst);
    let (response, response_header) = parse_connection_dma_response(&job);
    write_reg(mmio, PCIE_HOST_INT_STATUS, !status);
    clear_connection_response_mailbox(mmio);
    let _cleanup_flush_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    compiler_fence(Ordering::SeqCst);
    let association = match response {
        Ok(association) => association,
        Err(result) => {
            let response_diagnostic = connection_response_diagnostic(&job, result, response_header);
            runtime.snapshot.response_diagnostic = response_diagnostic;
            quarantine_connection_job(&job);
            finish_connection_locked(&mut runtime, result, ConnectionStage::Failed, status);
            DATA_LINK_READY.store(false, Ordering::Release);
            write_connection_response_failure(job.phase, result, response_diagnostic, status);
            return true;
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
            return true;
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
            drop(runtime);
            net::detach_wifi();
            serial::write_line("marvell wifi: ephemeral authority denied before port release");
            return true;
        }
        runtime.snapshot.stage = ConnectionStage::WaitPortRelease;
        runtime.job = Some(job);
        serial::write_line("marvell wifi: association accepted; waiting for secure port release");
        return true;
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
            drop(runtime);
            net::detach_wifi();
            return true;
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
        if let Some(mac) = mac {
            net::attach_wifi(mac);
        }
        serial::write_line("marvell wifi: open association accepted; data link released");
        return true;
    }

    runtime.snapshot.stage = job.phase;
    runtime.job = Some(job);
    true
}

fn next_connection_phase(stage: ConnectionStage, security: Dot11Security) -> ConnectionStage {
    match stage {
        ConnectionStage::SupplicantProfile => ConnectionStage::SupplicantPmk,
        ConnectionStage::SupplicantPmk => ConnectionStage::Associate,
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
        queue_fixed_hw_failure_trace(
            phase,
            status,
            HwFailureRegister::MarvellHostInterruptStatus,
            host_int_status,
        );
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

fn quarantine_connection_job(job: &ConnectionJob) {
    DATA_LINK_READY.store(false, Ordering::Release);
    pci::disable_bus_master(job.pci_address);
    let mmio = job.mmio_base as *mut u8;
    clear_connection_response_mailbox(mmio);
    let _cleanup_flush_status = read_reg(mmio, PCIE_HOST_INT_STATUS);
    clear_connection_secret_dma(job);
    CONNECTION_REBOOT_REQUIRED.store(true, Ordering::Release);
    compiler_fence(Ordering::SeqCst);
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
        let changed = runtime.snapshot.host_int_status != status
            || runtime.snapshot.wrptr != wrptr
            || runtime.snapshot.rdptr != runtime.rdptr;
        runtime.snapshot.host_int_status = status;
        runtime.snapshot.wrptr = wrptr;
        runtime.snapshot.rdptr = runtime.rdptr;

        let event_available = event_ring_has_entry(wrptr, runtime.rdptr);
        if status & HOST_INTR_EVENT_RDY == 0 && !event_available {
            return changed;
        }
        if !event_available {
            write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_EVENT_DONE);
            return true;
        }

        let rd_index = (runtime.rdptr & EVENT_RING_MASK) as usize;
        if rd_index >= EVENT_RING_COUNT {
            runtime.snapshot.stage = EventRingStage::Failed;
            runtime.snapshot.result = Some(EventRingResult::BadReadPointer);
            write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_EVENT_DONE);
            return true;
        }

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
    let event_id = cause & 0xffff;
    if matches!(event_id, 0x0003 | 0x0008 | 0x0009) {
        let mut runtime = CONNECTION.lock();
        if runtime.snapshot.attempted && !runtime.snapshot.is_failed() {
            if let Some(job) = runtime.job.take() {
                quarantine_connection_job(&job);
            } else if let Some(address) = wifi::snapshot().address {
                pci::disable_bus_master(address);
            }
            runtime.snapshot.running = false;
            runtime.snapshot.failed_stage = Some(runtime.snapshot.stage);
            runtime.snapshot.stage = ConnectionStage::Failed;
            runtime.snapshot.result = Some(ConnectionResult::LinkLost(event_id as u16));
            runtime.snapshot.response_diagnostic = None;
            runtime.ready_target = None;
            DATA_LINK_READY.store(false, Ordering::Release);
            drop(runtime);
            net::detach_wifi();
            serial::write_fmt(format_args!(
                "marvell wifi: link loss event 0x{:04x}; DMA quarantined\r\n",
                event_id
            ));
        }
        return;
    }
    if event_id != marvell_wifi_supplicant::EVENT_PORT_RELEASE {
        return;
    }

    let mut runtime = CONNECTION.lock();
    if runtime.snapshot.stage != ConnectionStage::WaitPortRelease {
        return;
    }
    let Some(job) = runtime.job.take() else {
        return;
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
        drop(runtime);
        net::detach_wifi();
        serial::write_line("marvell wifi: ephemeral authority denied before net attach");
        return;
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
    drop(runtime);

    if let Some(mac) = hw_spec_snapshot().mac {
        net::attach_wifi(mac);
    }
    serial::write_line("marvell wifi: secure port released; data link and DHCP enabled");
}

pub fn poll_rx_ring() -> bool {
    let mut runtime = RX_RING.lock();
    if !runtime.snapshot.armed {
        return false;
    }
    let Some(mmio_base) = runtime.mmio_base else {
        return false;
    };
    let mmio_base = mmio_base as *mut u8;
    let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
    let wrptr = read_reg(mmio_base, PCIE_RX_WR_PTR);
    let changed = runtime.snapshot.host_int_status != status
        || runtime.snapshot.wrptr != wrptr
        || runtime.snapshot.rdptr != runtime.rdptr;
    runtime.snapshot.host_int_status = status;
    runtime.snapshot.wrptr = wrptr;
    runtime.snapshot.rdptr = runtime.rdptr;

    let rx_available = rx_ring_has_entry(wrptr, runtime.rdptr);
    if status & HOST_INTR_UPLD_RDY == 0 && !rx_available {
        return changed;
    }
    if !rx_available {
        return changed;
    }

    let rd_index = (runtime.rdptr & RX_RING_MASK) as usize;
    if rd_index >= RX_RING_COUNT {
        runtime.snapshot.stage = RxRingStage::Failed;
        runtime.snapshot.result = Some(RxRingResult::BadReadPointer);
        return true;
    }

    let len = rx_buffer_len(rd_index);
    let packet_type = rx_buffer_type(rd_index);
    let result = if len == 0 {
        RxRingResult::PointerAdvancedEmptyBuffer
    } else if len as usize <= EVENT_HEADER_LEN || len as usize > RX_BUFFER_SIZE {
        RxRingResult::BadRxLength
    } else {
        RxRingResult::PacketObserved
    };
    let next_rdptr = next_rx_rdptr(runtime.rdptr);
    arm_rx_desc(rd_index);
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio_base, PCIE_RX_RD_PTR, next_rdptr);
    compiler_fence(Ordering::SeqCst);

    runtime.rdptr = next_rdptr;
    runtime.snapshot = RxRingSnapshot {
        attempted: true,
        armed: true,
        stage: if result == RxRingResult::PacketObserved {
            RxRingStage::PacketReady
        } else if result == RxRingResult::PointerAdvancedEmptyBuffer {
            RxRingStage::Armed
        } else {
            RxRingStage::Failed
        },
        result: Some(result),
        host_int_status: status,
        rdptr: next_rdptr,
        wrptr,
        rx_len: len,
        rx_type: packet_type,
    };
    if result == RxRingResult::PacketObserved {
        serial::write_fmt(format_args!(
            "marvell wifi: rx packet observed len={} type=0x{:04x}; scan parsing still denied\r\n",
            len, packet_type
        ));
    }
    true
}

pub fn poll() -> bool {
    let mut runtime = BRINGUP.lock();
    let Some(mut job) = runtime.job.take() else {
        return false;
    };
    let mmio_base = job.mmio_base as *mut u8;

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
                arm_hw_spec_after_firmware_ready(job.mmio_base, job.pci_address);
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
            FwAction::Retry { .. } => {}
            FwAction::RingDoorbell => {}
            FwAction::WriteDrvReady { value } => {
                write_drv_ready_pre_quarantined(job.pci_address, mmio_base, value);
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
    result: FirmwareDownloadResult,
    stage: FirmwareStage,
    failed_stage: Option<FirmwareStage>,
    registers: FirmwareRegisterSnapshot,
    downloaded: usize,
    total: usize,
) {
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

fn arm_hw_spec_after_firmware_ready(mmio_base: usize, pci_address: pci::PciAddress) {
    let mut runtime = HWSPEC.lock();
    if runtime.snapshot.attempted {
        return;
    }

    if !arm_data_rings(mmio_base) {
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

fn arm_event_ring(mmio_base: usize) {
    let mut runtime = EVENT_RING.lock();
    if runtime.snapshot.attempted {
        return;
    }

    let mut index = 0usize;
    while index < EVENT_RING_COUNT {
        let Some(data_phys) = event_data_phys(index) else {
            runtime.snapshot = EventRingSnapshot {
                attempted: true,
                armed: false,
                stage: EventRingStage::Failed,
                result: Some(EventRingResult::DmaAddressUnavailable),
                ..EventRingSnapshot::new()
            };
            serial::write_line("marvell wifi: event ring arm failed: dma_address_unavailable");
            return;
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

    let mmio = mmio_base as *mut u8;
    runtime.rdptr = EVENT_ROLLOVER_IND;
    compiler_fence(Ordering::SeqCst);

    runtime.mmio_base = Some(mmio_base);
    runtime.snapshot = EventRingSnapshot {
        attempted: true,
        armed: false,
        stage: EventRingStage::Armed,
        result: Some(EventRingResult::Armed),
        host_int_status: read_reg(mmio, PCIE_HOST_INT_STATUS),
        rdptr: runtime.rdptr,
        wrptr: read_reg(mmio, PCIE_EVT_WR_PTR),
        event_len: 0,
        event_type: 0,
        event_cause: 0,
    };
    serial::write_line("marvell wifi: event ring prepared for descriptor registration");
}

fn arm_rx_ring(mmio_base: usize) -> bool {
    let mut runtime = RX_RING.lock();
    if runtime.snapshot.attempted {
        return runtime.snapshot.armed;
    }
    let mut index = 0usize;
    while index < RX_RING_COUNT {
        let Some(data_phys) = rx_data_phys(index) else {
            runtime.snapshot = RxRingSnapshot {
                attempted: true,
                armed: false,
                stage: RxRingStage::Failed,
                result: Some(RxRingResult::DmaAddressUnavailable),
                ..RxRingSnapshot::new()
            };
            return false;
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
    RX_RDPTR_SHARED.store(runtime.rdptr, Ordering::Release);
    runtime.mmio_base = Some(mmio_base);
    runtime.snapshot = RxRingSnapshot {
        attempted: true,
        armed: true,
        stage: RxRingStage::Armed,
        result: Some(RxRingResult::Armed),
        host_int_status: read_reg(mmio_base as *mut u8, PCIE_HOST_INT_STATUS),
        rdptr: runtime.rdptr,
        wrptr: read_reg(mmio_base as *mut u8, PCIE_RX_WR_PTR),
        rx_len: 0,
        rx_type: 0,
    };
    true
}

fn arm_tx_ring(mmio_base: usize) -> bool {
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
    TX_WRPTR_SHARED.store(0, Ordering::Release);
    true
}

fn arm_data_rings(mmio_base: usize) -> bool {
    arm_event_ring(mmio_base);
    let event_ready = {
        let runtime = EVENT_RING.lock();
        runtime.mmio_base.is_some() && !runtime.snapshot.is_failed()
    };
    event_ready && arm_rx_ring(mmio_base) && arm_tx_ring(mmio_base)
}

fn publish_data_ring_pointers(mmio_base: usize) {
    let event_rdptr = {
        let mut runtime = EVENT_RING.lock();
        runtime.snapshot.armed = true;
        runtime.rdptr
    };
    let rx_rdptr = RX_RDPTR_SHARED.load(Ordering::Acquire) & 0x0000_07ff;
    let tx_wrptr = TX_WRPTR_SHARED.load(Ordering::Acquire) & TX_RING_WRAP_MASK;
    let mmio = mmio_base as *mut u8;
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
    write_reg(mmio, PCIE_EVT_RD_PTR, event_rdptr);
    write_reg(mmio, PCIE_RX_RD_PTR, tx_wrptr | rx_rdptr);
    compiler_fence(Ordering::SeqCst);
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
    if stage == HwSpecStage::Failed {
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
        ConnectionStage::Associate => 2,
        _ => 7,
    };
    u16::from(base) + offset
}

fn connection_phase_command(phase: ConnectionStage) -> Option<u16> {
    match phase {
        ConnectionStage::SupplicantProfile => Some(marvell_wifi_supplicant::SUPPLICANT_PROFILE_CMD),
        ConnectionStage::SupplicantPmk => Some(marvell_wifi_supplicant::SUPPLICANT_PMK_CMD),
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

fn tx_ring_full(wrptr: u32, rdptr: u32) -> bool {
    (wrptr & TX_RING_MASK) == (rdptr & TX_RING_MASK)
        && (wrptr & TX_ROLLOVER_IND) != (rdptr & TX_ROLLOVER_IND)
}

fn next_tx_wrptr(wrptr: u32) -> u32 {
    let next = wrptr.wrapping_add(TX_RING_STEP);
    if (next & TX_RING_MASK) == (TX_RING_COUNT as u32) << 16 {
        (next & TX_ROLLOVER_IND) ^ TX_ROLLOVER_IND
    } else {
        next
    }
}

fn event_ring_has_entry(wrptr: u32, rdptr: u32) -> bool {
    ((wrptr & EVENT_RING_MASK) != (rdptr & EVENT_RING_MASK))
        || ((wrptr & EVENT_ROLLOVER_IND) == (rdptr & EVENT_ROLLOVER_IND))
}

fn rx_ring_has_entry(wrptr: u32, rdptr: u32) -> bool {
    ((wrptr & RX_RING_MASK) != (rdptr & RX_RING_MASK))
        || ((wrptr & RX_ROLLOVER_IND) == (rdptr & RX_ROLLOVER_IND))
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

fn write_drv_ready_pre_quarantined(pci_address: pci::PciAddress, mmio_base: *mut u8, value: u32) {
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio_base, PCIE_HOST_INT_MASK, 0);
    write_reg(mmio_base, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
    let pending = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
    if pending != 0 && pending != u32::MAX {
        write_reg(mmio_base, PCIE_HOST_INT_STATUS, !pending);
    }
    compiler_fence(Ordering::SeqCst);
    pci::disable_bus_master(pci_address);
    compiler_fence(Ordering::SeqCst);
    write_reg(mmio_base, DRV_READY, value);
    compiler_fence(Ordering::SeqCst);
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
