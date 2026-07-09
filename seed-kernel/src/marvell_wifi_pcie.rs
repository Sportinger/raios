//! Marvell 88W8897 PCIe firmware-download hardware shell.
//!
//! Hardware bring-up is owner-triggered only. It is honestly unaudited: the
//! proprietary firmware blob and DMA path are trusted until raiOS has IOMMU
//! enforcement.

use core::hint::spin_loop;
use core::ptr;
use core::slice;
use core::sync::atomic::{compiler_fence, Ordering};

use raios_core::marvell_wifi_cmd::{self, HwSpecCmdError};
use raios_core::marvell_wifi_fw::{
    decide_fw_ready_poll, plan_register_writes, FirmwareDownload, FwAction, FwError, FwPhase,
    FwReadyPollDecision, RegisterReads, CMDRSP_ADDR_HI, CMDRSP_ADDR_LO, CMD_ADDR_HI, CMD_ADDR_LO,
    CMD_SIZE, CPU_INTR_DOOR_BELL, DRV_READY, FW_DMA_STAGING_SIZE, FW_DUMP_CTRL,
    FW_READY_TIMEOUT_MS, FW_STATUS, HOST_INTR_CMD_DONE, HOST_INTR_EVENT_RDY, HOST_INTR_MASK,
    MWIFIEX_UPLD_SIZE, PCIE_CPU_INT_EVENT, PCIE_CPU_INT_STATUS, PCIE_HOST_INT_STATUS,
    PCIE_HOST_INT_STATUS_MASK,
};
use spin::Mutex;

use crate::{memory, pci, serial, time, wifi};

// Linux resource index 2: PCI config offset 0x18. BAR0 is 64-bit on this part.
const MARVELL_REGISTER_BAR: u8 = 2;
const CMD_SIZE_TIMEOUT_MS: u64 = 5_000;
const DOORBELL_ACK_TIMEOUT_MS: u64 = 5_000;
const TOTAL_BRINGUP_TIMEOUT_MS: u64 = 300_000;
const SHORT_POLL_DELAY_US: u64 = 10;
const ACTIONS_PER_POLL: usize = 32;
const HWSPEC_CMD_BUFFER_SIZE: usize = 128;
const HWSPEC_TIMEOUT_MS: u64 = 3_000;
const SCAN_CMD_BUFFER_SIZE: usize = marvell_wifi_cmd::SCAN_EXT_24GHZ_CMD_TOTAL_LEN;
const SCAN_CMD_TIMEOUT_MS: u64 = 15_000;
const EVENT_RING_COUNT: usize = 8;
const EVENT_RING_MASK: u32 = 0x0f;
const EVENT_ROLLOVER_IND: u32 = 1 << 7;
const EVENT_BUFFER_SIZE: usize = 2048;
const EVENT_HEADER_LEN: usize = 4;
const PCIE_EVT_RD_PTR: u32 = 0xCE8;
const PCIE_EVT_WR_PTR: u32 = 0xCEC;
const CPU_INTR_EVENT_DONE: u32 = 1 << 5;

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

#[repr(C, align(64))]
struct EventRingDmaBlock {
    desc: [EventBufDesc; EVENT_RING_COUNT],
    data: [[u8; EVENT_BUFFER_SIZE]; EVENT_RING_COUNT],
}

static mut SCAN_DMA_BLOCK: ScanDmaBlock = ScanDmaBlock {
    cmd: [0; SCAN_CMD_BUFFER_SIZE],
    rsp: [0; MWIFIEX_UPLD_SIZE],
};
static mut EVENT_RING_DMA_BLOCK: EventRingDmaBlock = EventRingDmaBlock {
    desc: [EventBufDesc::EMPTY; EVENT_RING_COUNT],
    data: [[0; EVENT_BUFFER_SIZE]; EVENT_RING_COUNT],
};
static BRINGUP: Mutex<FirmwareBringupRuntime> = Mutex::new(FirmwareBringupRuntime::new());
static HWSPEC: Mutex<HwSpecRuntime> = Mutex::new(HwSpecRuntime::new());
static SCAN: Mutex<ScanCmdRuntime> = Mutex::new(ScanCmdRuntime::new());
static EVENT_RING: Mutex<EventRingRuntime> = Mutex::new(EventRingRuntime::new());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirmwareDownloadResult {
    Done,
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
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HwSpecStage {
    Idle,
    Arming,
    WaitCmdDone,
    Ready,
    Failed,
}

impl HwSpecStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Arming => "arming",
            Self::WaitCmdDone => "wait_cmd_done",
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
    CommandBuild(HwSpecCmdError),
    CmdDoneTimeout,
    Response(HwSpecCmdError),
}

impl HwSpecResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Done => "ready",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::CommandBuild(HwSpecCmdError::OutputBufferTooSmall) => "cmd_buffer_too_small",
            Self::CommandBuild(HwSpecCmdError::TooShort) => "cmd_build_too_short",
            Self::CommandBuild(HwSpecCmdError::BadCommand { .. }) => "cmd_build_bad_command",
            Self::CommandBuild(HwSpecCmdError::FwResult { .. }) => "cmd_build_fw_result",
            Self::CmdDoneTimeout => "cmd_done_timeout",
            Self::Response(HwSpecCmdError::TooShort) => "response_too_short",
            Self::Response(HwSpecCmdError::BadCommand { .. }) => "bad_command",
            Self::Response(HwSpecCmdError::FwResult { .. }) => "fw_result",
            Self::Response(HwSpecCmdError::OutputBufferTooSmall) => "response_buffer_too_small",
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
}

impl ScanCmdResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Done => "command_done_event_ring_unavailable",
            Self::FirmwareNotReady => "firmware_not_ready",
            Self::HwSpecNotReady => "hw_spec_not_ready",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::CommandBuild(HwSpecCmdError::OutputBufferTooSmall) => "cmd_buffer_too_small",
            Self::CommandBuild(HwSpecCmdError::TooShort) => "cmd_build_too_short",
            Self::CommandBuild(HwSpecCmdError::BadCommand { .. }) => "cmd_build_bad_command",
            Self::CommandBuild(HwSpecCmdError::FwResult { .. }) => "cmd_build_fw_result",
            Self::CmdDoneTimeout => "cmd_done_timeout",
            Self::Response(HwSpecCmdError::TooShort) => "response_too_short",
            Self::Response(HwSpecCmdError::BadCommand { .. }) => "bad_command",
            Self::Response(HwSpecCmdError::FwResult { .. }) => "fw_result",
            Self::Response(HwSpecCmdError::OutputBufferTooSmall) => "response_buffer_too_small",
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
    EventObservedRxRingUnavailable,
}

impl EventRingResult {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Armed => "armed",
            Self::DmaAddressUnavailable => "dma_address_unavailable",
            Self::BadReadPointer => "bad_read_pointer",
            Self::BadEventLength => "bad_event_length",
            Self::EventObservedRxRingUnavailable => "event_observed_rx_ring_unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanCmdTriggerResult {
    Started,
    AlreadyRunning,
    Failed(ScanCmdResult),
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
    pub result: Option<HwSpecResult>,
    pub mac: Option<[u8; 6]>,
    pub fw_release: Option<u32>,
    pub host_int_status: u32,
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
            result: None,
            mac: None,
            fw_release: None,
            host_int_status: 0,
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

struct EventRingRuntime {
    snapshot: EventRingSnapshot,
    mmio_base: Option<usize>,
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

impl ScanCmdRuntime {
    const fn new() -> Self {
        Self {
            snapshot: ScanCmdSnapshot::new(),
            job: None,
            next_seq: 1,
        }
    }
}

struct FirmwareJob {
    download: FirmwareDownload<'static>,
    mmio_base: usize,
    block_dma_phys: u64,
    phase: FwPhase,
    phase_started_tsc: u64,
    started_tsc: u64,
    firmware_len: usize,
}

struct HwSpecJob {
    mmio_base: usize,
    cmd_dma_phys: u64,
    rsp_dma_phys: u64,
    started_tsc: u64,
    seq: u16,
}

struct ScanCmdJob {
    mmio_base: usize,
    cmd_dma_phys: u64,
    rsp_dma_phys: u64,
    started_tsc: u64,
    seq: u16,
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
        mmio_base,
        cmd_dma_phys,
        rsp_dma_phys,
        started_tsc: time::rdtsc(),
        seq,
    });
    drop(runtime);

    wifi::note_scan_command_started();
    serial::write_line("marvell wifi: scan_ext command armed (results wait on event ring)");
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

    let Some(job) = runtime.job.take() else {
        return false;
    };
    let mmio_base = job.mmio_base as *mut u8;
    let mut changed = false;
    let mut actions = 0usize;

    while actions < ACTIONS_PER_POLL {
        actions += 1;
        if elapsed_ms(job.started_tsc) >= HWSPEC_TIMEOUT_MS {
            let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
            finish_hw_spec_locked(
                &mut runtime,
                HwSpecResult::CmdDoneTimeout,
                HwSpecStage::Failed,
                status,
                None,
                None,
            );
            write_hw_spec_failure(HwSpecResult::CmdDoneTimeout, status);
            return true;
        }

        match runtime.snapshot.stage {
            HwSpecStage::Arming => {
                let command_len = match prepare_hw_spec_dma(job.seq) {
                    Ok(command_len) => command_len,
                    Err(error) => {
                        let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                        let result = HwSpecResult::CommandBuild(error);
                        finish_hw_spec_locked(
                            &mut runtime,
                            result,
                            HwSpecStage::Failed,
                            status,
                            None,
                            None,
                        );
                        write_hw_spec_failure(result, status);
                        return true;
                    }
                };

                compiler_fence(Ordering::SeqCst);
                write_reg(mmio_base, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
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
                runtime.snapshot = HwSpecSnapshot {
                    attempted: true,
                    running: true,
                    stage: HwSpecStage::WaitCmdDone,
                    result: None,
                    mac: None,
                    fw_release: None,
                    host_int_status: status,
                };
                changed = true;
            }
            HwSpecStage::WaitCmdDone => {
                let status = read_reg(mmio_base, PCIE_HOST_INT_STATUS);
                if runtime.snapshot.host_int_status != status {
                    runtime.snapshot.host_int_status = status;
                    changed = true;
                }
                if status & HOST_INTR_CMD_DONE != 0 {
                    compiler_fence(Ordering::SeqCst);
                    let parsed = parse_hw_spec_dma_response();
                    write_reg(mmio_base, PCIE_HOST_INT_STATUS, !status);
                    compiler_fence(Ordering::SeqCst);

                    match parsed {
                        Ok(hw_spec) => {
                            finish_hw_spec_locked(
                                &mut runtime,
                                HwSpecResult::Done,
                                HwSpecStage::Ready,
                                status,
                                Some(hw_spec.mac),
                                Some(hw_spec.fw_release),
                            );
                            serial::write_fmt(format_args!(
                                "marvell wifi: hw_spec MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} fw_release 0x{:08x}\r\n",
                                hw_spec.mac[0],
                                hw_spec.mac[1],
                                hw_spec.mac[2],
                                hw_spec.mac[3],
                                hw_spec.mac[4],
                                hw_spec.mac[5],
                                hw_spec.fw_release
                            ));
                        }
                        Err(error) => {
                            let result = HwSpecResult::Response(error);
                            finish_hw_spec_locked(
                                &mut runtime,
                                result,
                                HwSpecStage::Failed,
                                status,
                                None,
                                None,
                            );
                            write_hw_spec_failure(result, status);
                        }
                    }
                    return true;
                }
                delay_us(SHORT_POLL_DELAY_US);
            }
            HwSpecStage::Idle | HwSpecStage::Ready | HwSpecStage::Failed => {
                runtime.job = Some(job);
                return changed;
            }
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
            let command_len = runtime.snapshot.command_len;
            finish_scan_locked(
                &mut runtime,
                ScanCmdResult::CmdDoneTimeout,
                ScanCmdStage::Failed,
                status,
                command_len,
            );
            write_scan_failure(ScanCmdResult::CmdDoneTimeout, status);
            wifi::note_scan_command_failed_event_ring_unavailable();
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
                        wifi::note_scan_command_failed_event_ring_unavailable();
                        return true;
                    }
                };

                compiler_fence(Ordering::SeqCst);
                write_reg(mmio_base, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
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
                    compiler_fence(Ordering::SeqCst);

                    match parsed {
                        Ok(()) => {
                            let command_len = runtime.snapshot.command_len;
                            finish_scan_locked(
                                &mut runtime,
                                ScanCmdResult::Done,
                                ScanCmdStage::Done,
                                status,
                                command_len,
                            );
                            wifi::note_scan_command_done_event_ring_unavailable();
                            serial::write_line(
                                "marvell wifi: scan_ext command done; event ring not implemented",
                            );
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
                            wifi::note_scan_command_failed_event_ring_unavailable();
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

pub fn poll_event_ring() -> bool {
    let (changed, scan_event_observed, serial_event) = {
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
                runtime.snapshot.stage = EventRingStage::Failed;
                runtime.snapshot.result = Some(result);
                write_reg(mmio_base, PCIE_CPU_INT_EVENT, CPU_INTR_EVENT_DONE);
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
            result: Some(EventRingResult::EventObservedRxRingUnavailable),
            host_int_status: status,
            rdptr: next_rdptr,
            wrptr,
            event_len: parsed.len,
            event_type: parsed.event_type,
            event_cause: parsed.cause,
        };
        (true, scan_cmd_snapshot().is_done(), Some(parsed))
    };

    if scan_event_observed {
        wifi::note_scan_event_observed_rx_ring_unavailable();
    }
    if let Some(parsed) = serial_event {
        serial::write_fmt(format_args!(
            "marvell wifi: event observed cause=0x{:08x} len={} type=0x{:04x}; rx ring not implemented\r\n",
            parsed.cause, parsed.len, parsed.event_type
        ));
    }
    changed
}

pub fn poll() -> bool {
    let mut runtime = BRINGUP.lock();
    let Some(mut job) = runtime.job.take() else {
        return false;
    };
    let mmio_base = job.mmio_base as *mut u8;

    let mut actions = 0usize;
    while actions < ACTIONS_PER_POLL {
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
                arm_hw_spec_after_firmware_ready(job.mmio_base);
                finish_locked(
                    &mut runtime,
                    FirmwareDownloadResult::Done,
                    FirmwareStage::Ready,
                    None,
                    registers,
                    job.firmware_len,
                    job.firmware_len,
                );
                wifi::note_firmware_ready_scan_unavailable();
                serial::write_line(
                    "marvell wifi: firmware ready 0xfedcba00; GET_HW_SPEC probe armed",
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
            FwAction::WriteDrvReady { .. } => {
                arm_event_ring(job.mmio_base);
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

fn arm_hw_spec_after_firmware_ready(mmio_base: usize) {
    let mut runtime = HWSPEC.lock();
    if runtime.snapshot.attempted {
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
        write_hw_spec_failure(HwSpecResult::DmaAddressUnavailable, 0);
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
        write_hw_spec_failure(HwSpecResult::DmaAddressUnavailable, 0);
        return;
    };

    runtime.snapshot = HwSpecSnapshot {
        attempted: true,
        running: true,
        stage: HwSpecStage::Arming,
        result: None,
        mac: None,
        fw_release: None,
        host_int_status: 0,
    };
    runtime.job = Some(HwSpecJob {
        mmio_base,
        cmd_dma_phys,
        rsp_dma_phys,
        started_tsc: time::rdtsc(),
        seq: 0,
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
    write_reg(mmio, PCIE_HOST_INT_STATUS_MASK, HOST_INTR_MASK);
    write_reg(mmio, PCIE_EVT_RD_PTR, runtime.rdptr);
    compiler_fence(Ordering::SeqCst);

    runtime.mmio_base = Some(mmio_base);
    runtime.snapshot = EventRingSnapshot {
        attempted: true,
        armed: true,
        stage: EventRingStage::Armed,
        result: Some(EventRingResult::Armed),
        host_int_status: read_reg(mmio, PCIE_HOST_INT_STATUS),
        rdptr: runtime.rdptr,
        wrptr: read_reg(mmio, PCIE_EVT_WR_PTR),
        event_len: 0,
        event_type: 0,
        event_cause: 0,
    };
    serial::write_line("marvell wifi: event ring armed before DRV_READY");
}

fn finish_hw_spec_locked(
    runtime: &mut HwSpecRuntime,
    result: HwSpecResult,
    stage: HwSpecStage,
    host_int_status: u32,
    mac: Option<[u8; 6]>,
    fw_release: Option<u32>,
) {
    runtime.job = None;
    runtime.snapshot = HwSpecSnapshot {
        attempted: true,
        running: false,
        stage,
        result: Some(result),
        mac,
        fw_release,
        host_int_status,
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

fn prepare_hw_spec_dma(seq: u16) -> Result<usize, HwSpecCmdError> {
    unsafe {
        // SAFETY: HWSPEC_DMA_BLOCK is this driver's distinct command mailbox
        // DMA area. Access is serialized by HWSPEC and bounded by fixed sizes.
        ptr::write_bytes(hw_spec_rsp_ptr(), 0, MWIFIEX_UPLD_SIZE);
        let cmd = slice::from_raw_parts_mut(hw_spec_cmd_ptr(), HWSPEC_CMD_BUFFER_SIZE);
        marvell_wifi_cmd::build_get_hw_spec(seq, cmd)
    }
}

fn prepare_scan_dma(seq: u16) -> Result<usize, HwSpecCmdError> {
    unsafe {
        // SAFETY: SCAN_DMA_BLOCK is this driver's distinct command mailbox DMA
        // area. Access is serialized by SCAN and bounded by fixed sizes.
        ptr::write_bytes(scan_rsp_ptr(), 0, MWIFIEX_UPLD_SIZE);
        let cmd = slice::from_raw_parts_mut(scan_cmd_ptr(), SCAN_CMD_BUFFER_SIZE);
        marvell_wifi_cmd::build_scan_ext_24ghz_wildcard(seq, cmd)
    }
}

fn parse_hw_spec_dma_response() -> Result<marvell_wifi_cmd::HwSpec, HwSpecCmdError> {
    unsafe {
        // SAFETY: the firmware has raised CMD_DONE before this is called, and
        // the response slice covers only the fixed mailbox response buffer.
        let response = slice::from_raw_parts(hw_spec_rsp_ptr().cast_const(), MWIFIEX_UPLD_SIZE);
        marvell_wifi_cmd::parse_hw_spec_response(response)
    }
}

fn parse_scan_dma_response() -> Result<(), HwSpecCmdError> {
    unsafe {
        // SAFETY: the firmware has raised CMD_DONE before this is called, and
        // the response slice covers only the fixed mailbox response buffer.
        let response = slice::from_raw_parts(scan_rsp_ptr().cast_const(), MWIFIEX_UPLD_SIZE);
        marvell_wifi_cmd::parse_scan_ext_response(response)
    }
}

fn parse_event_buffer(index: usize) -> Result<ParsedEvent, EventRingResult> {
    unsafe {
        // SAFETY: index is checked by poll_event_ring before parsing, and each
        // event buffer has the fixed EVENT_BUFFER_SIZE.
        let bytes = slice::from_raw_parts(event_data_ptr(index).cast_const(), EVENT_BUFFER_SIZE);
        let len = u16::from_le_bytes([bytes[0], bytes[1]]);
        let event_type = u16::from_le_bytes([bytes[2], bytes[3]]);
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

fn event_ring_has_entry(wrptr: u32, rdptr: u32) -> bool {
    ((wrptr & EVENT_RING_MASK) != (rdptr & EVENT_RING_MASK))
        || ((wrptr & EVENT_ROLLOVER_IND) == (rdptr & EVENT_ROLLOVER_IND))
}

fn next_event_rdptr(rdptr: u32) -> u32 {
    let next = rdptr.wrapping_add(1);
    if (next & EVENT_RING_MASK) == EVENT_RING_COUNT as u32 {
        (next & EVENT_ROLLOVER_IND) ^ EVENT_ROLLOVER_IND
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

fn write_hw_spec_failure(result: HwSpecResult, host_int_status: u32) {
    match result {
        HwSpecResult::Response(HwSpecCmdError::BadCommand { got })
        | HwSpecResult::CommandBuild(HwSpecCmdError::BadCommand { got }) => {
            serial::write_fmt(format_args!(
                "marvell wifi: hw_spec failed: {} got=0x{:04x} host_int=0x{:08x}\r\n",
                result.label(),
                got,
                host_int_status
            ));
        }
        HwSpecResult::Response(HwSpecCmdError::FwResult { code })
        | HwSpecResult::CommandBuild(HwSpecCmdError::FwResult { code }) => {
            serial::write_fmt(format_args!(
                "marvell wifi: hw_spec failed: {} code=0x{:04x} host_int=0x{:08x}\r\n",
                result.label(),
                code,
                host_int_status
            ));
        }
        _ => {
            serial::write_fmt(format_args!(
                "marvell wifi: hw_spec failed: {} host_int=0x{:08x}\r\n",
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
                "marvell wifi: scan_ext failed: {} got=0x{:04x} host_int=0x{:08x}\r\n",
                result.label(),
                got,
                host_int_status
            ));
        }
        ScanCmdResult::Response(HwSpecCmdError::FwResult { code })
        | ScanCmdResult::CommandBuild(HwSpecCmdError::FwResult { code }) => {
            serial::write_fmt(format_args!(
                "marvell wifi: scan_ext failed: {} code=0x{:04x} host_int=0x{:08x}\r\n",
                result.label(),
                code,
                host_int_status
            ));
        }
        _ => {
            serial::write_fmt(format_args!(
                "marvell wifi: scan_ext failed: {} host_int=0x{:08x}\r\n",
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
        // from plan_register_writes over the real sequencer action.
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

fn event_data_phys(index: usize) -> Option<u64> {
    memory::virt_to_phys(event_data_ptr(index).cast_const())
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
