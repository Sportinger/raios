//! Marvell 88W8897 PCIe firmware-download hardware shell.
//!
//! Compile-only and honestly HARDWARE-UNTESTED. The public entry is inert in
//! this slice: it returns before BAR mapping, bus-master enable, MMIO, or DMA
//! unless the target chip is present and firmware is explicitly made available.

use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

use raios_core::marvell_wifi_fw::{
    plan_register_writes, FirmwareDownload, FwAction, FwError, RegisterReads, CMD_SIZE,
    FW_BLOCK_SIZE, FW_STATUS, PCIE_CPU_INT_STATUS,
};

use crate::{memory, pci, wifi};

const MARVELL_REGISTER_BAR: u8 = 2;
const FIRMWARE_AVAILABLE: bool = false;
const FIRMWARE_BYTES: &[u8] = &[];

#[repr(align(64))]
struct DmaBlock([u8; FW_BLOCK_SIZE]);

static mut DMA_BLOCK: DmaBlock = DmaBlock([0; FW_BLOCK_SIZE]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirmwareDownloadResult {
    Done,
    NotPresent,
    FirmwareUnavailable,
    Bar2Missing,
    Bar2NotMmio,
    MmioMapFailed(&'static str),
    DmaAddressUnavailable,
    FirmwareSliceOutOfRange,
    BlockLenOutOfRange,
    Failed(FwError),
}

pub fn try_bring_up_firmware() -> FirmwareDownloadResult {
    let snapshot = wifi::probe();
    if snapshot.state != wifi::WifiState::Detected {
        return FirmwareDownloadResult::NotPresent;
    }

    if !FIRMWARE_AVAILABLE {
        return FirmwareDownloadResult::FirmwareUnavailable;
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

    run_firmware_download(mapping.as_ptr::<u8>(), FIRMWARE_BYTES)
}

fn run_firmware_download(mmio_base: *mut u8, firmware: &[u8]) -> FirmwareDownloadResult {
    let mut download = FirmwareDownload::new(firmware);
    let mut block_dma_phys = None;

    loop {
        let action = download.step(RegisterReads {
            cmd_size_reg: read_reg(mmio_base, CMD_SIZE),
            fw_status_reg: read_reg(mmio_base, FW_STATUS),
            int_status_reg: read_reg(mmio_base, PCIE_CPU_INT_STATUS),
        });

        match action {
            FwAction::Done => return FirmwareDownloadResult::Done,
            FwAction::Fail(error) => return FirmwareDownloadResult::Failed(error),
            FwAction::WriteBlock { image_offset, len } => {
                let Some(end) = image_offset.checked_add(len) else {
                    return FirmwareDownloadResult::FirmwareSliceOutOfRange;
                };
                let Some(src) = firmware.get(image_offset..end) else {
                    return FirmwareDownloadResult::FirmwareSliceOutOfRange;
                };
                if let Err(error) = copy_block_into_dma(src) {
                    return error;
                }
                if block_dma_phys.is_none() {
                    block_dma_phys = dma_block_phys();
                }
                if block_dma_phys.is_none() {
                    return FirmwareDownloadResult::DmaAddressUnavailable;
                }
            }
            FwAction::RingDoorbell | FwAction::WriteDrvReady { .. } => {}
            FwAction::PollFwStatus | FwAction::Retry { .. } => {}
        }

        let plan = plan_register_writes(action, block_dma_phys.unwrap_or(0));
        let mut index = 0;
        while index < plan.len() {
            if let Some(write) = plan.get(index) {
                write_reg(mmio_base, write.offset, write.value);
            }
            index += 1;
        }
    }
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

fn copy_block_into_dma(src: &[u8]) -> Result<(), FirmwareDownloadResult> {
    if src.len() > FW_BLOCK_SIZE {
        return Err(FirmwareDownloadResult::BlockLenOutOfRange);
    }

    let dst = dma_block_ptr();
    unsafe {
        // SAFETY: DMA_BLOCK is this driver's single firmware bounce buffer,
        // sized to FW_BLOCK_SIZE; src length is checked above, and the unused
        // tail is zeroed before the hardware is told the block address/size.
        ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
        ptr::write_bytes(dst.add(src.len()), 0, FW_BLOCK_SIZE - src.len());
    }
    compiler_fence(Ordering::SeqCst);
    Ok(())
}

fn dma_block_ptr() -> *mut u8 {
    unsafe {
        // SAFETY: returning a raw pointer does not create a Rust reference; all
        // access to this static bounce buffer stays inside this I/O shell.
        ptr::addr_of_mut!(DMA_BLOCK.0).cast::<u8>()
    }
}
