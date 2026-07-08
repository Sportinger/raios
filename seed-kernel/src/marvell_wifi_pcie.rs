//! Marvell 88W8897 PCIe firmware-download hardware shell.
//!
//! Compile-only and honestly HARDWARE-UNTESTED. The public entry is inert in
//! this slice: it returns before BAR mapping, bus-master enable, MMIO, or DMA
//! unless the target chip is present and firmware is explicitly made available.

use core::hint::spin_loop;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

use raios_core::marvell_wifi_fw::{
    decide_fw_ready_poll, plan_register_writes, FirmwareDownload, FwAction, FwError, FwPhase,
    FwReadyPollDecision, RegisterReads, CMD_SIZE, FW_DMA_STAGING_SIZE, FW_DUMP_CTRL,
    FW_READY_POLL_INTERVAL_MS, FW_READY_TIMEOUT_MS, FW_STATUS, PCIE_CPU_INT_STATUS,
};

use crate::{memory, pci, serial, time, wifi};

// Linux resource index 2: PCI config offset 0x18. BAR0 is 64-bit on this part.
const MARVELL_REGISTER_BAR: u8 = 2;
const CMD_SIZE_TIMEOUT_MS: u64 = 5_000;
const DOORBELL_ACK_TIMEOUT_MS: u64 = 5_000;
const SHORT_POLL_DELAY_US: u64 = 20;

#[repr(align(64))]
struct DmaBlock([u8; FW_DMA_STAGING_SIZE]);

static mut DMA_BLOCK: DmaBlock = DmaBlock([0; FW_DMA_STAGING_SIZE]);

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
    Failed(FwError),
}

pub fn firmware_image() -> &'static [u8] {
    // TODO: real pcie8897_uapsta.bin from linux-firmware must be provided here.
    &[]
}

pub fn try_bring_up_firmware() -> FirmwareDownloadResult {
    let snapshot = wifi::probe();
    if snapshot.state != wifi::WifiState::Detected {
        return FirmwareDownloadResult::NotPresent;
    }

    let firmware = firmware_image();
    if firmware.is_empty() {
        serial::write_line(
            "marvell wifi: firmware image absent; pcie8897_uapsta.bin not embedded",
        );
        return FirmwareDownloadResult::FirmwareImageAbsent;
    }

    let Some(address) = snapshot.address else {
        return FirmwareDownloadResult::NotPresent;
    };
    let Some(bar) = pci::read_bar_info(address, MARVELL_REGISTER_BAR) else {
        return FirmwareDownloadResult::Bar2Missing;
    };
    if !bar.is_memory() {
        return FirmwareDownloadResult::Bar2NotMmio;
    }

    pci::enable_bus_master(address);
    let mapping = match memory::map_mmio(bar.base, bar.size as usize) {
        Ok(mapping) => mapping,
        Err(error) => return FirmwareDownloadResult::MmioMapFailed(error),
    };

    let mmio_base = mapping.as_ptr::<u8>();
    if !probe_mmio(mmio_base) {
        return FirmwareDownloadResult::MmioProbeAllOnes;
    }

    run_firmware_download(mmio_base, firmware)
}

fn run_firmware_download(mmio_base: *mut u8, firmware: &[u8]) -> FirmwareDownloadResult {
    let mut download = FirmwareDownload::new(firmware);
    let Some(block_dma_phys) = dma_block_phys() else {
        return FirmwareDownloadResult::DmaAddressUnavailable;
    };
    let mut phase = download.phase();
    let mut phase_started_tsc = time::rdtsc();

    loop {
        let observed = read_observed(mmio_base);
        let action = download.step(observed);
        let current_phase = download.phase();
        if current_phase != phase {
            phase = current_phase;
            phase_started_tsc = time::rdtsc();
        }

        match action {
            FwAction::Done => return FirmwareDownloadResult::Done,
            FwAction::Fail(error) => return FirmwareDownloadResult::Failed(error),
            FwAction::WriteBlock {
                image_offset,
                payload_len,
                wire_len,
            } => {
                let Some(end) = image_offset.checked_add(payload_len) else {
                    return FirmwareDownloadResult::FirmwareSliceOutOfRange;
                };
                let Some(src) = firmware.get(image_offset..end) else {
                    return FirmwareDownloadResult::FirmwareSliceOutOfRange;
                };
                if let Err(error) = copy_block_into_dma(src, wire_len) {
                    return error;
                }
            }
            FwAction::Retry { .. } => {}
            FwAction::RingDoorbell | FwAction::WriteDrvReady { .. } => {}
            FwAction::PollDoorbellAck => {
                if elapsed_ms(phase_started_tsc) >= DOORBELL_ACK_TIMEOUT_MS {
                    return FirmwareDownloadResult::DoorbellAckTimeout;
                }
                delay_us(SHORT_POLL_DELAY_US);
                continue;
            }
            FwAction::PollFwStatus => match phase {
                FwPhase::Downloading => {
                    if elapsed_ms(phase_started_tsc) >= CMD_SIZE_TIMEOUT_MS {
                        return FirmwareDownloadResult::CmdSizeTimeout;
                    }
                    delay_us(SHORT_POLL_DELAY_US);
                    continue;
                }
                FwPhase::PollingReady => {
                    match decide_fw_ready_poll(
                        observed.fw_status_reg,
                        elapsed_ms(phase_started_tsc),
                        FW_READY_TIMEOUT_MS,
                    ) {
                        FwReadyPollDecision::Ready => {}
                        FwReadyPollDecision::StillDownloading => {
                            delay_ms(FW_READY_POLL_INTERVAL_MS);
                            continue;
                        }
                        FwReadyPollDecision::Timeout => {
                            return FirmwareDownloadResult::FirmwareReadyTimeout;
                        }
                    }
                }
                _ => {
                    delay_us(SHORT_POLL_DELAY_US);
                    continue;
                }
            },
        }

        let plan = plan_register_writes(action, block_dma_phys);
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
}

fn read_observed(mmio_base: *mut u8) -> RegisterReads {
    RegisterReads {
        cmd_size_reg: read_reg(mmio_base, CMD_SIZE),
        fw_status_reg: read_reg(mmio_base, FW_STATUS),
        int_status_reg: read_reg(mmio_base, PCIE_CPU_INT_STATUS),
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
        // chip-present and firmware-present gates pass; offsets are the fixed
        // 32-bit register offsets supplied by the pure firmware module.
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
        // sized to FW_DMA_STAGING_SIZE; src/wire lengths are checked above, and the unused
        // tail is zeroed before the hardware is told the block address/size.
        ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
        ptr::write_bytes(dst.add(src.len()), 0, wire_len - src.len());
    }
    compiler_fence(Ordering::SeqCst);
    Ok(())
}

fn elapsed_ms(start_tsc: u64) -> u64 {
    time::rdtsc().wrapping_sub(start_tsc) / time::tsc_per_ms().max(1)
}

fn delay_ms(ms: u64) {
    delay_ticks(time::tsc_per_ms().saturating_mul(ms));
}

fn delay_us(us: u64) {
    let ticks = time::tsc_per_ms()
        .saturating_mul(us)
        .saturating_add(999)
        / 1_000;
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
