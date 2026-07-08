//! Marvell 88W8897 PCIe firmware-download hardware shell.
//!
//! Hardware bring-up is owner-triggered only. It is honestly unaudited: the
//! proprietary firmware blob and DMA path are trusted until raiOS has IOMMU
//! enforcement.

use core::hint::spin_loop;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

use raios_core::marvell_wifi_fw::{
    decide_fw_ready_poll, plan_register_writes, FirmwareDownload, FwAction, FwError, FwPhase,
    FwReadyPollDecision, RegisterReads, CMD_SIZE, DRV_READY, FIRMWARE_READY_PCIE,
    FW_DMA_STAGING_SIZE, FW_DUMP_CTRL, FW_READY_TIMEOUT_MS, FW_STATUS, PCIE_CPU_INT_STATUS,
    PCIE_HOST_INT_STATUS,
};
use spin::Mutex;

use crate::{memory, pci, serial, time, wifi};

// Linux resource index 2: PCI config offset 0x18. BAR0 is 64-bit on this part.
const MARVELL_REGISTER_BAR: u8 = 2;
const CMD_SIZE_TIMEOUT_MS: u64 = 5_000;
const DOORBELL_ACK_TIMEOUT_MS: u64 = 5_000;
const TOTAL_BRINGUP_TIMEOUT_MS: u64 = 300_000;
const SHORT_POLL_DELAY_US: u64 = 20;
const ACTIONS_PER_POLL: usize = 128;

#[repr(align(64))]
struct DmaBlock([u8; FW_DMA_STAGING_SIZE]);

static mut DMA_BLOCK: DmaBlock = DmaBlock([0; FW_DMA_STAGING_SIZE]);
static BRINGUP: Mutex<FirmwareBringupRuntime> = Mutex::new(FirmwareBringupRuntime::new());

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

struct FirmwareBringupRuntime {
    snapshot: FirmwareBringupSnapshot,
    job: Option<FirmwareJob>,
}

impl FirmwareBringupRuntime {
    const fn new() -> Self {
        Self {
            snapshot: FirmwareBringupSnapshot::new(),
            job: None,
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
                    "marvell wifi: firmware ready 0xfedcba00; command mailbox not attempted",
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
            FwAction::RingDoorbell | FwAction::WriteDrvReady { .. } => {}
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
