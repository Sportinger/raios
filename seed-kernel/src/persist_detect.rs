use alloc::vec;
use core::ptr;

use crate::{memory, pci, pci::PciMassStorageController};
use raios_core::{
    gpt_layout::{
        self, GptLayoutEvidence, LayoutStatus, GPT_ENTRY_ARRAY_BYTES, PRIMARY_GPT_ENTRIES_LBA,
        SECTOR_SIZE,
    },
    seed_data_layout::{self, DataLayoutEvidence},
};

use super::{
    issue_identify, issue_read_sector_into, read32, AhciBlockDeviceIdentity, AHCI_BAR,
    AHCI_SUBCLASS, MAX_PROBE_LEN, MIN_PROBE_LEN, PORT_BASE, PORT_CI, PORT_SIG, PORT_SSTS,
    PORT_STRIDE, REG_PI, SATA_SIG_ATA, SCRATCH_BUFFER, SECTOR_BYTES,
};

const PERSIST_PORT_PREFERRED: u8 = 3;
const MISSING_PORT: u8 = u8::MAX;

#[derive(Clone, Copy, Debug)]
pub(crate) struct PersistLayoutEvidence {
    pub(crate) source: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) port_index: u8,
    pub(crate) controller_present: bool,
    pub(crate) read_attempted: bool,
    pub(crate) read_completed: bool,
    pub(crate) write_attempted: bool,
    pub(crate) write_dma_ext_called: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) persistence_claimed: bool,
    pub(crate) gpt: GptLayoutEvidence,
    pub(crate) data: DataLayoutEvidence,
}

impl PersistLayoutEvidence {
    pub(crate) const fn absent(reason: &'static str) -> Self {
        Self {
            source: "ahci.persist_detect",
            reason,
            port_index: MISSING_PORT,
            controller_present: false,
            read_attempted: false,
            read_completed: false,
            write_attempted: false,
            write_dma_ext_called: false,
            writes_enabled: false,
            persistence_claimed: false,
            gpt: GptLayoutEvidence::absent(reason),
            data: DataLayoutEvidence::absent(reason),
        }
    }

    const fn from_layouts(
        reason: &'static str,
        port_index: u8,
        read_attempted: bool,
        read_completed: bool,
        gpt: GptLayoutEvidence,
        data: DataLayoutEvidence,
    ) -> Self {
        Self {
            source: "ahci.persist_detect",
            reason,
            port_index,
            controller_present: true,
            read_attempted,
            read_completed,
            write_attempted: false,
            write_dma_ext_called: false,
            writes_enabled: false,
            persistence_claimed: false,
            gpt,
            data,
        }
    }

    pub(crate) const fn status(self) -> &'static str {
        match (self.gpt.status, self.data.status) {
            (LayoutStatus::Invalid, _) | (_, LayoutStatus::Invalid) => "invalid",
            (LayoutStatus::Present, LayoutStatus::Present) => "present",
            _ => "absent",
        }
    }
}

pub(super) unsafe fn detect_uncached(
    controller: PciMassStorageController,
) -> PersistLayoutEvidence {
    if controller.subclass != AHCI_SUBCLASS {
        return PersistLayoutEvidence::absent("ahci_controller_not_observed");
    }

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return PersistLayoutEvidence::absent("ahci_abar_missing");
    };
    if !bar.is_memory() {
        return PersistLayoutEvidence::absent("ahci_abar_not_mmio");
    }

    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return PersistLayoutEvidence::absent("ahci_abar_probe_range_too_small");
    }

    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return PersistLayoutEvidence::absent("ahci_abar_mmio_map_failed");
    };
    let base = mapping.as_ptr::<u8>();
    let ports_mask = read32(base, REG_PI);
    if ports_mask == 0 {
        return PersistLayoutEvidence::absent("ahci_port_inventory_missing");
    }

    let mut last_absent = PersistLayoutEvidence::absent("gpt_persist_disk_absent");
    last_absent.controller_present = true;

    let mut ordinal = 0u8;
    while ordinal < 32 {
        let port = ordered_port(ordinal);
        ordinal += 1;
        if (ports_mask & (1u32 << port)) == 0 {
            continue;
        }

        let offset = PORT_BASE + port as usize * PORT_STRIDE;
        if offset + PORT_CI + core::mem::size_of::<u32>() > mapping.len() {
            return PersistLayoutEvidence::from_layouts(
                "ahci_port_register_range_unavailable",
                port,
                false,
                false,
                GptLayoutEvidence::absent("ahci_port_register_range_unavailable"),
                DataLayoutEvidence::absent("ahci_port_register_range_unavailable"),
            );
        }

        let ssts = read32(base, offset + PORT_SSTS);
        let device_present = (ssts & 0x0f) == 0x03;
        let interface_active = ((ssts >> 8) & 0x0f) == 0x01;
        let signature = read32(base, offset + PORT_SIG);
        if !device_present || !interface_active || signature != SATA_SIG_ATA {
            continue;
        }

        let Ok(identity) = issue_identify(controller, base, offset) else {
            continue;
        };
        if !identity_can_read_512(identity) {
            continue;
        }

        match detect_on_port(controller, base, offset, port) {
            Ok(evidence) => match evidence.gpt.status {
                LayoutStatus::Present | LayoutStatus::Invalid => return evidence,
                LayoutStatus::Absent => last_absent = evidence,
            },
            Err(reason) => {
                return PersistLayoutEvidence::from_layouts(
                    reason,
                    port,
                    true,
                    false,
                    GptLayoutEvidence::absent(reason),
                    DataLayoutEvidence::absent(reason),
                );
            }
        }
    }

    last_absent
}

unsafe fn detect_on_port(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    port_index: u8,
) -> Result<PersistLayoutEvidence, &'static str> {
    let mbr = read_sector(controller, base, port_offset, 0)?;
    let header = read_sector(controller, base, port_offset, 1)?;
    let mut entries = vec![0u8; GPT_ENTRY_ARRAY_BYTES];
    let mut sector_idx = 0usize;
    while sector_idx < GPT_ENTRY_ARRAY_BYTES / SECTOR_SIZE {
        let sector = read_sector(
            controller,
            base,
            port_offset,
            PRIMARY_GPT_ENTRIES_LBA + sector_idx as u64,
        )?;
        let out = sector_idx * SECTOR_SIZE;
        entries[out..out + SECTOR_SIZE].copy_from_slice(&sector);
        sector_idx += 1;
    }

    let gpt = gpt_layout::validate_gpt_layout(&mbr, &header, &entries);
    if gpt.status != LayoutStatus::Present {
        return Ok(PersistLayoutEvidence::from_layouts(
            gpt.reason,
            port_index,
            true,
            true,
            gpt,
            DataLayoutEvidence::absent(gpt.reason),
        ));
    }

    let sb0 = match read_sector(controller, base, port_offset, gpt.seed_data_first_lba) {
        Ok(sector) => sector,
        Err(reason) => {
            return Ok(PersistLayoutEvidence::from_layouts(
                reason,
                port_index,
                true,
                false,
                gpt,
                DataLayoutEvidence::invalid(reason, gpt.seed_data_lba_count),
            ));
        }
    };
    // checked_add: a maliciously-emulated device could present seed_data_first_lba
    // = u64::MAX; fail closed (Invalid) instead of overflow-panicking in debug builds.
    let Some(seed_data_lba1) = gpt.seed_data_first_lba.checked_add(1) else {
        return Ok(PersistLayoutEvidence::from_layouts(
            "seed_data_first_lba_overflow",
            port_index,
            true,
            false,
            gpt,
            DataLayoutEvidence::invalid("seed_data_first_lba_overflow", gpt.seed_data_lba_count),
        ));
    };
    let sb1 = match read_sector(controller, base, port_offset, seed_data_lba1) {
        Ok(sector) => sector,
        Err(reason) => {
            return Ok(PersistLayoutEvidence::from_layouts(
                reason,
                port_index,
                true,
                false,
                gpt,
                DataLayoutEvidence::invalid(reason, gpt.seed_data_lba_count),
            ));
        }
    };
    let data = seed_data_layout::validate_seed_data_layout(&sb0, &sb1, gpt.seed_data_lba_count);
    Ok(PersistLayoutEvidence::from_layouts(
        data.reason,
        port_index,
        true,
        data.status == LayoutStatus::Present,
        gpt,
        data,
    ))
}

unsafe fn read_sector(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    lba: u64,
) -> Result<[u8; SECTOR_BYTES], &'static str> {
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    issue_read_sector_into(
        controller,
        base,
        port_offset,
        lba,
        buffer,
        "persist_detect_sector_buffer_phys_missing",
        "persist_detect_sector_read_timeout",
        "persist_detect_sector_read_task_file_error",
        "persist_detect_sector_read_incomplete",
    )?;

    let mut sector = [0u8; SECTOR_BYTES];
    let mut idx = 0usize;
    while idx < SECTOR_BYTES {
        sector[idx] = ptr::read_volatile(buffer.add(idx));
        idx += 1;
    }
    Ok(sector)
}

const fn ordered_port(ordinal: u8) -> u8 {
    if ordinal == 0 {
        PERSIST_PORT_PREFERRED
    } else if ordinal <= PERSIST_PORT_PREFERRED {
        ordinal - 1
    } else {
        ordinal
    }
}

const fn identity_can_read_512(identity: AhciBlockDeviceIdentity) -> bool {
    identity.available && identity.logical_sector_size_bytes == SECTOR_BYTES as u32
}
