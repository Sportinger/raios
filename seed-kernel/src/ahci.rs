use alloc::vec::Vec;
use core::{ptr, sync::atomic};

use raios_core::{
    boot_control::{BOOTCTL_REGION_BYTE_COUNT, BOOTCTL_SLOT_SIZE, BOOTCTL_STORAGE_SLOT_SECTORS},
    gpt_layout::LayoutStatus,
};
use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{memory, pci, pci::PciMassStorageController};

#[path = "persist_detect.rs"]
mod persist_detect;

pub(crate) use persist_detect::PersistLayoutEvidence;

pub(crate) struct PersistReclogRegionRead {
    pub(crate) layout: PersistLayoutEvidence,
    pub(crate) reason: &'static str,
    pub(crate) read_attempted: bool,
    pub(crate) read_completed: bool,
    pub(crate) region_bounds_valid: bool,
    pub(crate) absolute_start_lba: u64,
    pub(crate) lba_count: u64,
    pub(crate) byte_count: u64,
    pub(crate) bytes: Option<Vec<u8>>,
}

pub(crate) struct PersistBootctlRegionRead {
    pub(crate) layout: PersistLayoutEvidence,
    pub(crate) reason: &'static str,
    pub(crate) read_attempted: bool,
    pub(crate) read_completed: bool,
    pub(crate) region_bounds_valid: bool,
    pub(crate) absolute_start_lba: u64,
    pub(crate) lba_count: u64,
    pub(crate) byte_count: u64,
    pub(crate) bytes: Option<Vec<u8>>,
}

pub(crate) struct ReclogAppendWriteEvidence {
    pub(crate) reason: &'static str,
    pub(crate) write_attempted: bool,
    pub(crate) write_completed: bool,
    pub(crate) readback_completed: bool,
    pub(crate) span_in_bounds: bool,
    pub(crate) absolute_start_lba: u64,
    pub(crate) reclog_lba_count: u64,
    pub(crate) byte_count: u64,
    pub(crate) write_offset: u64,
    pub(crate) readback: Option<Vec<u8>>,
}

pub(crate) struct BootctlSlotWriteEvidence {
    pub(crate) reason: &'static str,
    pub(crate) write_attempted: bool,
    pub(crate) write_completed: bool,
    pub(crate) readback_completed: bool,
    pub(crate) span_in_bounds: bool,
    pub(crate) absolute_start_lba: u64,
    pub(crate) bootctl_lba_count: u64,
    pub(crate) byte_count: u64,
    pub(crate) write_offset: u64,
    pub(crate) readback: Option<Vec<u8>>,
}

impl ReclogAppendWriteEvidence {
    fn missing(reason: &'static str, write_offset: u64) -> Self {
        Self {
            reason,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            span_in_bounds: false,
            absolute_start_lba: 0,
            reclog_lba_count: 0,
            byte_count: 0,
            write_offset,
            readback: None,
        }
    }
}

impl BootctlSlotWriteEvidence {
    fn missing(reason: &'static str, write_offset: u64) -> Self {
        Self {
            reason,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            span_in_bounds: false,
            absolute_start_lba: 0,
            bootctl_lba_count: 0,
            byte_count: 0,
            write_offset,
            readback: None,
        }
    }
}

impl PersistReclogRegionRead {
    fn missing(layout: PersistLayoutEvidence, reason: &'static str) -> Self {
        Self {
            layout,
            reason,
            read_attempted: false,
            read_completed: false,
            region_bounds_valid: false,
            absolute_start_lba: 0,
            lba_count: 0,
            byte_count: 0,
            bytes: None,
        }
    }
}

impl PersistBootctlRegionRead {
    fn missing(layout: PersistLayoutEvidence, reason: &'static str) -> Self {
        Self {
            layout,
            reason,
            read_attempted: false,
            read_completed: false,
            region_bounds_valid: false,
            absolute_start_lba: 0,
            lba_count: 0,
            byte_count: 0,
            bytes: None,
        }
    }
}

const AHCI_SUBCLASS: u8 = 0x06;
const AHCI_BAR: u8 = 5;
const MIN_PROBE_LEN: usize = 0x180;
const MAX_PROBE_LEN: usize = 0x2000;

const REG_CAP: usize = 0x00;
const REG_GHC: usize = 0x04;
const REG_PI: usize = 0x0c;
const REG_VS: usize = 0x10;

const GHC_AE: u32 = 1 << 31;

const PORT_BASE: usize = 0x100;
const PORT_STRIDE: usize = 0x80;
const PORT_CLB: usize = 0x00;
const PORT_CLBU: usize = 0x04;
const PORT_FB: usize = 0x08;
const PORT_FBU: usize = 0x0c;
const PORT_IS: usize = 0x10;
const PORT_CMD: usize = 0x18;
const PORT_TFD: usize = 0x20;
const PORT_SIG: usize = 0x24;
const PORT_SSTS: usize = 0x28;
const PORT_SERR: usize = 0x30;
const PORT_CI: usize = 0x38;

const PORT_CMD_ST: u32 = 1 << 0;
const PORT_CMD_FRE: u32 = 1 << 4;
const PORT_CMD_FR: u32 = 1 << 14;
const PORT_CMD_CR: u32 = 1 << 15;
const PORT_TFD_BSY: u32 = 1 << 7;
const PORT_TFD_DRQ: u32 = 1 << 3;
const PORT_IS_TFES: u32 = 1 << 30;

const SATA_SIG_ATA: u32 = 0x0000_0101;
const IDENTIFY_DEVICE: u8 = 0xec;
const READ_DMA_EXT: u8 = 0x25;
const WRITE_DMA_EXT: u8 = 0x35;
const IDENTIFY_BYTES: usize = 512;
pub(crate) const SECTOR_BYTES: usize = 512;
const MBR_SIGNATURE: u16 = 0xaa55;
const MBR_PARTITION_TABLE_OFFSET: usize = 446;
const MBR_PARTITION_ENTRY_BYTES: usize = 16;
const MBR_PARTITION_ENTRY_COUNT: usize = 4;
const SCRATCH_MARKER: [u8; 16] = *b"RAIOS_SCRATCH_V0";
const SCRATCH_MARKER_LBA: u64 = 0;
const SCRATCH_TEST_LBA: u64 = 1;
const AUDIT_ROLLBACK_TARGET_MARKER: [u8; 16] = *b"RAIOS_AUDITRB_V0";
const AUDIT_ROLLBACK_TARGET_MARKER_LBA: u64 = 0;
const AUDIT_ROLLBACK_TARGET_REGION_START_LBA: u64 = 1;
const AUDIT_ROLLBACK_TARGET_REGION_LBA_COUNT: u64 = 1;
const WAIT_ITERS: usize = 1_000_000;

static PROBE: Mutex<Option<AhciReadOnlyProbe>> = Mutex::new(None);
static PERSIST_LAYOUT: Mutex<Option<PersistLayoutEvidence>> = Mutex::new(None);
static SCRATCH_SECTOR_WRITE_READBACK: Mutex<Option<AhciScratchSectorWriteReadbackEvidence>> =
    Mutex::new(None);
static AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK: Mutex<
    Option<AhciAuditRollbackTargetSectorWriteReadbackEvidence>,
> = Mutex::new(None);
static AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION: Mutex<
    Option<AhciAuditRollbackTargetSectorInspectionEvidence>,
> = Mutex::new(None);

pub(crate) const SCRATCH_BLOCK_REGION_SCHEMA: &str = "raios.scratch_block_region_write_readback.v0";
pub(crate) const SCRATCH_BLOCK_REGION_ID: &str = "scratch.block_region.current_boot.v0";
pub(crate) const SCRATCH_BLOCK_WRITE_AUTHORITY_SCHEMA: &str =
    "raios.scratch_block_write_authority.v0";
pub(crate) const SCRATCH_BLOCK_WRITE_AUTHORITY_ID: &str =
    "scratch.block_write_authority.current_boot.v0";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_ID: &str =
    "target_region.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_MARKER: &str = "RAIOS_AUDITRB_V0";
pub(crate) const AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK_ID: &str =
    "target_region_sector_write_readback.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION_ID: &str =
    "target_region_sector_inspection.audit_rollback.current_boot";

#[repr(C, align(1024))]
struct AhciCommandList([u8; 1024]);

#[repr(C, align(256))]
struct AhciReceivedFis([u8; 256]);

#[repr(C, align(128))]
struct AhciCommandTable([u8; 256]);

#[repr(C, align(512))]
struct AhciIdentifyBuffer([u8; IDENTIFY_BYTES]);

#[repr(C, align(512))]
struct AhciSectorBuffer([u8; SECTOR_BYTES]);

static mut COMMAND_LIST: AhciCommandList = AhciCommandList([0; 1024]);
static mut RECEIVED_FIS: AhciReceivedFis = AhciReceivedFis([0; 256]);
static mut COMMAND_TABLE: AhciCommandTable = AhciCommandTable([0; 256]);
static mut IDENTIFY_BUFFER: AhciIdentifyBuffer = AhciIdentifyBuffer([0; IDENTIFY_BYTES]);
static mut SECTOR0_BUFFER: AhciSectorBuffer = AhciSectorBuffer([0; SECTOR_BYTES]);
static mut SCRATCH_BUFFER: AhciSectorBuffer = AhciSectorBuffer([0; SECTOR_BYTES]);

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciBlockDeviceIdentity {
    pub(crate) available: bool,
    pub(crate) logical_sector_size_bytes: u32,
    pub(crate) lba28_sector_count: u32,
    pub(crate) lba48_sector_count: u64,
    pub(crate) serial: [u8; 20],
    pub(crate) firmware: [u8; 8],
    pub(crate) model: [u8; 40],
}

impl AhciBlockDeviceIdentity {
    pub(crate) const fn missing() -> Self {
        Self {
            available: false,
            logical_sector_size_bytes: 0,
            lba28_sector_count: 0,
            lba48_sector_count: 0,
            serial: [0; 20],
            firmware: [0; 8],
            model: [0; 40],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciSectorReadEvidence {
    pub(crate) available: bool,
    pub(crate) lba: u64,
    pub(crate) byte_count: u32,
    pub(crate) mbr_signature: u16,
    pub(crate) first16: [u8; 16],
}

impl AhciSectorReadEvidence {
    pub(crate) const fn missing() -> Self {
        Self {
            available: false,
            lba: 0,
            byte_count: 0,
            mbr_signature: 0,
            first16: [0; 16],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciPartitionEntryEvidence {
    pub(crate) available: bool,
    pub(crate) boot_indicator: u8,
    pub(crate) partition_type: u8,
    pub(crate) first_lba: u32,
    pub(crate) sector_count: u32,
}

impl AhciPartitionEntryEvidence {
    pub(crate) const fn missing() -> Self {
        Self {
            available: false,
            boot_indicator: 0,
            partition_type: 0,
            first_lba: 0,
            sector_count: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciPartitionInventoryEvidence {
    pub(crate) available: bool,
    pub(crate) source_lba: u64,
    pub(crate) scheme: &'static str,
    pub(crate) mbr_signature_valid: bool,
    pub(crate) entry_count: u8,
    pub(crate) bootable_entry_count: u8,
    pub(crate) entries: [AhciPartitionEntryEvidence; MBR_PARTITION_ENTRY_COUNT],
}

impl AhciPartitionInventoryEvidence {
    pub(crate) const fn missing() -> Self {
        Self {
            available: false,
            source_lba: 0,
            scheme: "missing",
            mbr_signature_valid: false,
            entry_count: 0,
            bootable_entry_count: 0,
            entries: [AhciPartitionEntryEvidence::missing(); MBR_PARTITION_ENTRY_COUNT],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciReadOnlyBlockDriverEvidence {
    pub(crate) available: bool,
    pub(crate) driver_id: &'static str,
    pub(crate) source: &'static str,
    pub(crate) logical_sector_size_bytes: u32,
    pub(crate) lba28_sector_count: u32,
    pub(crate) lba48_sector_count: u64,
    pub(crate) verified_read_lba: u64,
    pub(crate) verified_read_byte_count: u32,
    pub(crate) binds_partition_inventory: bool,
    pub(crate) supports_read: bool,
    pub(crate) supports_write: bool,
}

impl AhciReadOnlyBlockDriverEvidence {
    pub(crate) const fn missing() -> Self {
        Self {
            available: false,
            driver_id: "block_driver.ahci.read_only.current_boot",
            source: "ahci_dma_read_verified_sector0",
            logical_sector_size_bytes: 0,
            lba28_sector_count: 0,
            lba48_sector_count: 0,
            verified_read_lba: 0,
            verified_read_byte_count: 0,
            binds_partition_inventory: false,
            supports_read: false,
            supports_write: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciScratchWriteReadbackEvidence {
    pub(crate) available: bool,
    pub(crate) region_id: &'static str,
    pub(crate) device_identity: AhciBlockDeviceIdentity,
    pub(crate) marker_lba: u64,
    pub(crate) test_lba: u64,
    pub(crate) region_start_lba: u64,
    pub(crate) region_lba_count: u64,
    pub(crate) byte_count: u32,
    pub(crate) port_index: u8,
    pub(crate) label_found: bool,
    pub(crate) region_within_device_bounds: bool,
    pub(crate) boot_port_overlap: bool,
    pub(crate) metadata_lba_overlap: bool,
    pub(crate) no_boot_or_partition_metadata_overlap: bool,
    pub(crate) block_write_authority_available: bool,
    pub(crate) write_attempted: bool,
    pub(crate) write_completed: bool,
    pub(crate) readback_completed: bool,
    pub(crate) readback_matches: bool,
    pub(crate) readback_first16: [u8; 16],
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

impl AhciScratchWriteReadbackEvidence {
    pub(crate) const fn missing(reason: &'static str) -> Self {
        Self {
            available: false,
            region_id: SCRATCH_BLOCK_REGION_ID,
            device_identity: AhciBlockDeviceIdentity::missing(),
            marker_lba: SCRATCH_MARKER_LBA,
            test_lba: SCRATCH_TEST_LBA,
            region_start_lba: SCRATCH_TEST_LBA,
            region_lba_count: 1,
            byte_count: SECTOR_BYTES as u32,
            port_index: 0,
            label_found: false,
            region_within_device_bounds: false,
            boot_port_overlap: false,
            metadata_lba_overlap: false,
            no_boot_or_partition_metadata_overlap: false,
            block_write_authority_available: false,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            readback_matches: false,
            readback_first16: [0; 16],
            status: "missing",
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciAuditRollbackTargetRegionEvidence {
    pub(crate) available: bool,
    pub(crate) region_id: &'static str,
    pub(crate) device_identity: AhciBlockDeviceIdentity,
    pub(crate) marker_lba: u64,
    pub(crate) region_start_lba: u64,
    pub(crate) region_lba_count: u64,
    pub(crate) byte_count: u32,
    pub(crate) port_index: u8,
    pub(crate) label_found: bool,
    pub(crate) read_attempted: bool,
    pub(crate) read_completed: bool,
    pub(crate) read_first16: [u8; 16],
    pub(crate) region_within_device_bounds: bool,
    pub(crate) boot_port_overlap: bool,
    pub(crate) metadata_lba_overlap: bool,
    pub(crate) scratch_region_overlap: bool,
    pub(crate) no_boot_or_partition_metadata_overlap: bool,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

impl AhciAuditRollbackTargetRegionEvidence {
    pub(crate) const fn missing(reason: &'static str) -> Self {
        Self {
            available: false,
            region_id: AUDIT_ROLLBACK_TARGET_REGION_ID,
            device_identity: AhciBlockDeviceIdentity::missing(),
            marker_lba: AUDIT_ROLLBACK_TARGET_MARKER_LBA,
            region_start_lba: AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
            region_lba_count: AUDIT_ROLLBACK_TARGET_REGION_LBA_COUNT,
            byte_count: SECTOR_BYTES as u32,
            port_index: 0,
            label_found: false,
            read_attempted: false,
            read_completed: false,
            read_first16: [0; 16],
            region_within_device_bounds: false,
            boot_port_overlap: false,
            metadata_lba_overlap: false,
            scratch_region_overlap: false,
            no_boot_or_partition_metadata_overlap: false,
            status: "missing",
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciScratchSectorWriteReadbackEvidence {
    pub(crate) available: bool,
    pub(crate) region_id: &'static str,
    pub(crate) planned_image_hash: [u8; 32],
    pub(crate) readback_image_hash: [u8; 32],
    pub(crate) target_lba: u64,
    pub(crate) byte_count: u32,
    pub(crate) port_index: u8,
    pub(crate) label_found: bool,
    pub(crate) region_within_device_bounds: bool,
    pub(crate) boot_port_overlap: bool,
    pub(crate) metadata_lba_overlap: bool,
    pub(crate) no_boot_or_partition_metadata_overlap: bool,
    pub(crate) write_attempted: bool,
    pub(crate) write_completed: bool,
    pub(crate) readback_completed: bool,
    pub(crate) readback_matches_planned_image: bool,
    pub(crate) readback_first16: [u8; 16],
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

impl AhciScratchSectorWriteReadbackEvidence {
    fn missing(reason: &'static str, planned_image_hash: [u8; 32]) -> Self {
        Self {
            available: false,
            region_id: SCRATCH_BLOCK_REGION_ID,
            planned_image_hash,
            readback_image_hash: [0; 32],
            target_lba: SCRATCH_TEST_LBA,
            byte_count: SECTOR_BYTES as u32,
            port_index: 0,
            label_found: false,
            region_within_device_bounds: false,
            boot_port_overlap: false,
            metadata_lba_overlap: false,
            no_boot_or_partition_metadata_overlap: false,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            readback_matches_planned_image: false,
            readback_first16: [0; 16],
            status: "missing",
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciAuditRollbackTargetSectorWriteReadbackEvidence {
    pub(crate) available: bool,
    pub(crate) region_id: &'static str,
    pub(crate) write_readback_id: &'static str,
    pub(crate) planned_image_hash: [u8; 32],
    pub(crate) readback_image_hash: [u8; 32],
    pub(crate) target_lba: u64,
    pub(crate) byte_count: u32,
    pub(crate) port_index: u8,
    pub(crate) label_found: bool,
    pub(crate) region_within_device_bounds: bool,
    pub(crate) boot_port_overlap: bool,
    pub(crate) metadata_lba_overlap: bool,
    pub(crate) scratch_region_overlap: bool,
    pub(crate) no_boot_or_partition_metadata_overlap: bool,
    pub(crate) write_attempted: bool,
    pub(crate) write_completed: bool,
    pub(crate) readback_completed: bool,
    pub(crate) readback_matches_planned_image: bool,
    pub(crate) readback_first16: [u8; 16],
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

impl AhciAuditRollbackTargetSectorWriteReadbackEvidence {
    fn missing(reason: &'static str, planned_image_hash: [u8; 32]) -> Self {
        Self {
            available: false,
            region_id: AUDIT_ROLLBACK_TARGET_REGION_ID,
            write_readback_id: AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK_ID,
            planned_image_hash,
            readback_image_hash: [0; 32],
            target_lba: AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
            byte_count: SECTOR_BYTES as u32,
            port_index: 0,
            label_found: false,
            region_within_device_bounds: false,
            boot_port_overlap: false,
            metadata_lba_overlap: false,
            scratch_region_overlap: false,
            no_boot_or_partition_metadata_overlap: false,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            readback_matches_planned_image: false,
            readback_first16: [0; 16],
            status: "missing",
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciAuditRollbackTargetSectorInspectionEvidence {
    pub(crate) available: bool,
    pub(crate) region_id: &'static str,
    pub(crate) inspection_id: &'static str,
    pub(crate) expected_sector_image_hash: [u8; 32],
    pub(crate) sector_image_hash: [u8; 32],
    pub(crate) audit_record_image_hash: [u8; 32],
    pub(crate) rollback_transaction_image_hash: [u8; 32],
    pub(crate) target_lba: u64,
    pub(crate) byte_count: u32,
    pub(crate) port_index: u8,
    pub(crate) label_found: bool,
    pub(crate) region_within_device_bounds: bool,
    pub(crate) boot_port_overlap: bool,
    pub(crate) metadata_lba_overlap: bool,
    pub(crate) scratch_region_overlap: bool,
    pub(crate) no_boot_or_partition_metadata_overlap: bool,
    pub(crate) read_attempted: bool,
    pub(crate) read_completed: bool,
    pub(crate) read_matches_expected_image: bool,
    pub(crate) offsets_within_sector: bool,
    pub(crate) padding_zeroed: bool,
    pub(crate) first16: [u8; 16],
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

impl AhciAuditRollbackTargetSectorInspectionEvidence {
    fn missing(reason: &'static str, expected_sector_image_hash: [u8; 32]) -> Self {
        Self {
            available: false,
            region_id: AUDIT_ROLLBACK_TARGET_REGION_ID,
            inspection_id: AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION_ID,
            expected_sector_image_hash,
            sector_image_hash: [0; 32],
            audit_record_image_hash: [0; 32],
            rollback_transaction_image_hash: [0; 32],
            target_lba: AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
            byte_count: SECTOR_BYTES as u32,
            port_index: 0,
            label_found: false,
            region_within_device_bounds: false,
            boot_port_overlap: false,
            metadata_lba_overlap: false,
            scratch_region_overlap: false,
            no_boot_or_partition_metadata_overlap: false,
            read_attempted: false,
            read_completed: false,
            read_matches_expected_image: false,
            offsets_within_sector: false,
            padding_zeroed: false,
            first16: [0; 16],
            status: "missing",
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AhciReadOnlyProbe {
    pub(crate) observed: bool,
    pub(crate) controller_is_ahci: bool,
    pub(crate) abar_available: bool,
    pub(crate) abar_base: u64,
    pub(crate) abar_size: u64,
    pub(crate) registers_mapped: bool,
    pub(crate) version: u32,
    pub(crate) capabilities: u32,
    pub(crate) ports_implemented_mask: u32,
    pub(crate) implemented_port_count: u8,
    pub(crate) command_slots: u8,
    pub(crate) supports_64bit: bool,
    pub(crate) supports_ncq: bool,
    pub(crate) first_port_index: u8,
    pub(crate) first_port_implemented: bool,
    pub(crate) first_port_device_present: bool,
    pub(crate) first_port_interface_active: bool,
    pub(crate) first_port_signature: u32,
    pub(crate) first_port_ssts: u32,
    pub(crate) first_port_tfd: u32,
    pub(crate) first_port_cmd: u32,
    pub(crate) identify_command_issued: bool,
    pub(crate) controller_register_write_attempted: bool,
    pub(crate) dma_started: bool,
    pub(crate) identify_completed: bool,
    pub(crate) identify_status: &'static str,
    pub(crate) sector_read_attempted: bool,
    pub(crate) sector_read_completed: bool,
    pub(crate) sector_read_status: &'static str,
    pub(crate) media_write_attempted: bool,
    pub(crate) scratch_write_path_available: bool,
    pub(crate) scratch_write_readback: AhciScratchWriteReadbackEvidence,
    pub(crate) audit_rollback_target_region: AhciAuditRollbackTargetRegionEvidence,
    pub(crate) identity: AhciBlockDeviceIdentity,
    pub(crate) sector0: AhciSectorReadEvidence,
    pub(crate) partition_inventory: AhciPartitionInventoryEvidence,
    pub(crate) block_driver: AhciReadOnlyBlockDriverEvidence,
    pub(crate) block_device_identity_available: bool,
    pub(crate) sector_read_available: bool,
    pub(crate) partition_inventory_available: bool,
    pub(crate) block_driver_available: bool,
    pub(crate) write_path_available: bool,
    pub(crate) reason: &'static str,
}

impl AhciReadOnlyProbe {
    pub(crate) const fn missing(reason: &'static str) -> Self {
        Self {
            observed: false,
            controller_is_ahci: false,
            abar_available: false,
            abar_base: 0,
            abar_size: 0,
            registers_mapped: false,
            version: 0,
            capabilities: 0,
            ports_implemented_mask: 0,
            implemented_port_count: 0,
            command_slots: 0,
            supports_64bit: false,
            supports_ncq: false,
            first_port_index: 0,
            first_port_implemented: false,
            first_port_device_present: false,
            first_port_interface_active: false,
            first_port_signature: 0,
            first_port_ssts: 0,
            first_port_tfd: 0,
            first_port_cmd: 0,
            identify_command_issued: false,
            controller_register_write_attempted: false,
            dma_started: false,
            identify_completed: false,
            identify_status: "not_attempted",
            sector_read_attempted: false,
            sector_read_completed: false,
            sector_read_status: "not_attempted",
            media_write_attempted: false,
            scratch_write_path_available: false,
            scratch_write_readback: AhciScratchWriteReadbackEvidence::missing(
                "scratch_block_region_device_missing",
            ),
            audit_rollback_target_region: AhciAuditRollbackTargetRegionEvidence::missing(
                "audit_rollback_target_region_device_missing",
            ),
            identity: AhciBlockDeviceIdentity::missing(),
            sector0: AhciSectorReadEvidence::missing(),
            partition_inventory: AhciPartitionInventoryEvidence::missing(),
            block_driver: AhciReadOnlyBlockDriverEvidence::missing(),
            block_device_identity_available: false,
            sector_read_available: false,
            partition_inventory_available: false,
            block_driver_available: false,
            write_path_available: false,
            reason,
        }
    }

    pub(crate) const fn available_for_selftest() -> Self {
        Self {
            observed: true,
            controller_is_ahci: true,
            abar_available: true,
            abar_base: 0x1000,
            abar_size: 0x1000,
            registers_mapped: true,
            version: 0x0001_0301,
            capabilities: 0,
            ports_implemented_mask: 1,
            implemented_port_count: 1,
            command_slots: 32,
            supports_64bit: true,
            supports_ncq: true,
            first_port_index: 0,
            first_port_implemented: true,
            first_port_device_present: true,
            first_port_interface_active: true,
            first_port_signature: SATA_SIG_ATA,
            first_port_ssts: 0x123,
            first_port_tfd: 0x50,
            first_port_cmd: 0,
            identify_command_issued: true,
            controller_register_write_attempted: true,
            dma_started: true,
            identify_completed: true,
            identify_status: "identify_complete",
            sector_read_attempted: true,
            sector_read_completed: true,
            sector_read_status: "sector0_read_complete",
            media_write_attempted: true,
            scratch_write_path_available: true,
            scratch_write_readback: AhciScratchWriteReadbackEvidence {
                available: true,
                region_id: SCRATCH_BLOCK_REGION_ID,
                device_identity: AhciBlockDeviceIdentity {
                    available: true,
                    logical_sector_size_bytes: 512,
                    lba28_sector_count: 1024,
                    lba48_sector_count: 1024,
                    serial: [b' '; 20],
                    firmware: [b' '; 8],
                    model: [b' '; 40],
                },
                marker_lba: SCRATCH_MARKER_LBA,
                test_lba: SCRATCH_TEST_LBA,
                region_start_lba: SCRATCH_TEST_LBA,
                region_lba_count: 1,
                byte_count: SECTOR_BYTES as u32,
                port_index: 1,
                label_found: true,
                region_within_device_bounds: true,
                boot_port_overlap: false,
                metadata_lba_overlap: false,
                no_boot_or_partition_metadata_overlap: true,
                block_write_authority_available: true,
                write_attempted: true,
                write_completed: true,
                readback_completed: true,
                readback_matches: true,
                readback_first16: *b"RAIOS_WRITEBACK!",
                status: "available",
                reason: "scratch_write_readback_verified",
            },
            audit_rollback_target_region: AhciAuditRollbackTargetRegionEvidence {
                available: true,
                region_id: AUDIT_ROLLBACK_TARGET_REGION_ID,
                device_identity: AhciBlockDeviceIdentity {
                    available: true,
                    logical_sector_size_bytes: 512,
                    lba28_sector_count: 1024,
                    lba48_sector_count: 1024,
                    serial: [b' '; 20],
                    firmware: [b' '; 8],
                    model: [b' '; 40],
                },
                marker_lba: AUDIT_ROLLBACK_TARGET_MARKER_LBA,
                region_start_lba: AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
                region_lba_count: AUDIT_ROLLBACK_TARGET_REGION_LBA_COUNT,
                byte_count: SECTOR_BYTES as u32,
                port_index: 2,
                label_found: true,
                read_attempted: true,
                read_completed: true,
                read_first16: [0; 16],
                region_within_device_bounds: true,
                boot_port_overlap: false,
                metadata_lba_overlap: false,
                scratch_region_overlap: false,
                no_boot_or_partition_metadata_overlap: true,
                status: "available",
                reason: "dedicated_audit_rollback_region_discovered_read_only",
            },
            identity: AhciBlockDeviceIdentity {
                available: true,
                logical_sector_size_bytes: 512,
                lba28_sector_count: 1024,
                lba48_sector_count: 1024,
                serial: [b' '; 20],
                firmware: [b' '; 8],
                model: [b' '; 40],
            },
            sector0: AhciSectorReadEvidence {
                available: true,
                lba: 0,
                byte_count: 512,
                mbr_signature: MBR_SIGNATURE,
                first16: [0; 16],
            },
            partition_inventory: AhciPartitionInventoryEvidence {
                available: true,
                source_lba: 0,
                scheme: "mbr_empty",
                mbr_signature_valid: true,
                entry_count: 0,
                bootable_entry_count: 0,
                entries: [AhciPartitionEntryEvidence::missing(); MBR_PARTITION_ENTRY_COUNT],
            },
            block_driver: AhciReadOnlyBlockDriverEvidence {
                available: true,
                driver_id: "block_driver.ahci.read_only.current_boot",
                source: "ahci_dma_read_verified_sector0",
                logical_sector_size_bytes: 512,
                lba28_sector_count: 1024,
                lba48_sector_count: 1024,
                verified_read_lba: 0,
                verified_read_byte_count: 512,
                binds_partition_inventory: true,
                supports_read: true,
                supports_write: false,
            },
            block_device_identity_available: true,
            sector_read_available: true,
            partition_inventory_available: true,
            block_driver_available: true,
            write_path_available: true,
            reason: "block_write_path_available",
        }
    }
}

pub(crate) fn probe(controller: PciMassStorageController) -> AhciReadOnlyProbe {
    let mut cached = PROBE.lock();
    if let Some(probe) = *cached {
        return probe;
    }

    let probe = probe_uncached(controller);
    *cached = Some(probe);
    probe
}

pub(crate) fn detect_persist_layout(controller: PciMassStorageController) -> PersistLayoutEvidence {
    let mut cached = PERSIST_LAYOUT.lock();
    if let Some(evidence) = *cached {
        return evidence;
    }

    let evidence = unsafe { persist_detect::detect_uncached(controller) };
    *cached = Some(evidence);
    evidence
}

pub(crate) fn read_persist_reclog_region(
    controller: PciMassStorageController,
) -> PersistReclogRegionRead {
    let layout = detect_persist_layout(controller);
    unsafe { read_persist_reclog_region_uncached(controller, layout) }
}

pub(crate) fn read_persist_bootctl_region(
    controller: PciMassStorageController,
) -> PersistBootctlRegionRead {
    let layout = detect_persist_layout(controller);
    unsafe { read_persist_bootctl_region_uncached(controller, layout) }
}

pub(crate) unsafe fn write_readback_reclog_append(
    controller: PciMassStorageController,
    write_offset: u64,
    frame: &[u8],
) -> ReclogAppendWriteEvidence {
    let layout = detect_persist_layout(controller);
    if layout.gpt.status != LayoutStatus::Present || layout.data.status != LayoutStatus::Present {
        return ReclogAppendWriteEvidence::missing(layout.reason, write_offset);
    }
    if layout.port_index == u8::MAX {
        return ReclogAppendWriteEvidence::missing("persist_reclog_port_missing", write_offset);
    }
    if layout.data.reclog_lba_count == 0
        || layout.data.reclog_start_lba > layout.gpt.seed_data_lba_count
        || layout.data.reclog_lba_count
            > layout
                .gpt
                .seed_data_lba_count
                .saturating_sub(layout.data.reclog_start_lba)
    {
        return ReclogAppendWriteEvidence::missing(
            "persist_reclog_region_bounds_invalid",
            write_offset,
        );
    }

    let Some(absolute_start_lba) = layout
        .gpt
        .seed_data_first_lba
        .checked_add(layout.data.reclog_start_lba)
    else {
        return ReclogAppendWriteEvidence::missing(
            "persist_reclog_start_lba_overflow",
            write_offset,
        );
    };
    let Some(byte_count) = layout
        .data
        .reclog_lba_count
        .checked_mul(SECTOR_BYTES as u64)
    else {
        return ReclogAppendWriteEvidence::missing(
            "persist_reclog_byte_count_overflow",
            write_offset,
        );
    };

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return ReclogAppendWriteEvidence::missing("ahci_abar_missing", write_offset);
    };
    if !bar.is_memory() {
        return ReclogAppendWriteEvidence::missing("ahci_abar_not_mmio", write_offset);
    }
    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return ReclogAppendWriteEvidence::missing("ahci_abar_probe_range_too_small", write_offset);
    }
    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return ReclogAppendWriteEvidence::missing("ahci_abar_mmio_map_failed", write_offset);
    };
    let base = mapping.as_ptr::<u8>();
    let port_offset = PORT_BASE + layout.port_index as usize * PORT_STRIDE;
    if port_offset + PORT_CI + core::mem::size_of::<u32>() > mapping.len() {
        return ReclogAppendWriteEvidence::missing(
            "persist_reclog_port_range_unavailable",
            write_offset,
        );
    }
    let ssts = read32(base, port_offset + PORT_SSTS);
    let signature = read32(base, port_offset + PORT_SIG);
    if (ssts & 0x0f) != 0x03 || ((ssts >> 8) & 0x0f) != 0x01 || signature != SATA_SIG_ATA {
        return ReclogAppendWriteEvidence::missing("persist_reclog_port_not_ready", write_offset);
    }

    let mut evidence = ReclogAppendWriteEvidence {
        reason: "persist_reclog_append_span_validated",
        write_attempted: false,
        write_completed: false,
        readback_completed: false,
        span_in_bounds: false,
        absolute_start_lba,
        reclog_lba_count: layout.data.reclog_lba_count,
        byte_count,
        write_offset,
        readback: None,
    };

    if frame.is_empty() || frame.len() % SECTOR_BYTES != 0 {
        evidence.reason = "reclog_append_frame_len_out_of_scope";
        return evidence;
    }
    if write_offset % SECTOR_BYTES as u64 != 0 {
        evidence.reason = "reclog_append_write_offset_misaligned";
        return evidence;
    }
    if write_offset
        .checked_add(frame.len() as u64)
        .map(|end| end > byte_count)
        .unwrap_or(true)
    {
        evidence.reason = "durable_store_full";
        return evidence;
    }
    let Some(reclog_end_lba) = absolute_start_lba.checked_add(layout.data.reclog_lba_count) else {
        evidence.reason = "persist_reclog_end_lba_overflow";
        return evidence;
    };

    let sector_count = frame.len() / SECTOR_BYTES;
    let start_sector = write_offset / SECTOR_BYTES as u64;
    let mut sector = 0usize;
    while sector < sector_count {
        let Some(lba) = absolute_start_lba
            .checked_add(start_sector)
            .and_then(|base_lba| base_lba.checked_add(sector as u64))
        else {
            evidence.reason = "persist_reclog_append_lba_overflow";
            return evidence;
        };
        if lba < absolute_start_lba || lba >= reclog_end_lba {
            evidence.reason = "write_span_out_of_reclog";
            return evidence;
        }
        sector += 1;
    }
    evidence.span_in_bounds = true;

    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    sector = 0;
    while sector < sector_count {
        let lba = absolute_start_lba + start_sector + sector as u64;
        let frame_offset = sector * SECTOR_BYTES;
        let mut idx = 0usize;
        while idx < SECTOR_BYTES {
            ptr::write_volatile(buffer.add(idx), frame[frame_offset + idx]);
            idx += 1;
        }
        evidence.write_attempted = true;
        if let Err(reason) = issue_write_sector(controller, base, port_offset, lba, buffer) {
            evidence.reason = reason;
            return evidence;
        }
        sector += 1;
    }
    evidence.write_completed = true;

    let mut readback = Vec::new();
    readback.resize(frame.len(), 0);
    sector = 0;
    while sector < sector_count {
        let lba = absolute_start_lba + start_sector + sector as u64;
        ptr::write_bytes(buffer, 0, SECTOR_BYTES);
        if let Err(reason) = issue_read_sector_into(
            controller,
            base,
            port_offset,
            lba,
            buffer,
            "persist_reclog_append_readback_buffer_phys_missing",
            "persist_reclog_append_readback_timeout",
            "persist_reclog_append_readback_task_file_error",
            "persist_reclog_append_readback_incomplete",
        ) {
            evidence.reason = reason;
            return evidence;
        }

        let out = sector * SECTOR_BYTES;
        let mut idx = 0usize;
        while idx < SECTOR_BYTES {
            readback[out + idx] = ptr::read_volatile(buffer.add(idx));
            idx += 1;
        }
        sector += 1;
    }

    evidence.readback_completed = true;
    evidence.reason = "persist_reclog_append_write_readback_completed";
    evidence.readback = Some(readback);
    evidence
}

pub(crate) unsafe fn write_readback_bootctl_slot(
    controller: PciMassStorageController,
    write_offset: u64,
    slot_bytes: &[u8; BOOTCTL_SLOT_SIZE],
) -> BootctlSlotWriteEvidence {
    let layout = detect_persist_layout(controller);
    if layout.gpt.status != LayoutStatus::Present || layout.data.status != LayoutStatus::Present {
        return BootctlSlotWriteEvidence::missing(layout.reason, write_offset);
    }
    if layout.port_index == u8::MAX {
        return BootctlSlotWriteEvidence::missing("persist_bootctl_port_missing", write_offset);
    }
    if layout.data.bootctl_lba_count == 0
        || layout.data.bootctl_start_lba > layout.gpt.seed_data_lba_count
        || layout.data.bootctl_lba_count
            > layout
                .gpt
                .seed_data_lba_count
                .saturating_sub(layout.data.bootctl_start_lba)
    {
        return BootctlSlotWriteEvidence::missing(
            "persist_bootctl_region_bounds_invalid",
            write_offset,
        );
    }

    let Some(absolute_start_lba) = layout
        .gpt
        .seed_data_first_lba
        .checked_add(layout.data.bootctl_start_lba)
    else {
        return BootctlSlotWriteEvidence::missing(
            "persist_bootctl_start_lba_overflow",
            write_offset,
        );
    };
    let Some(byte_count) = layout
        .data
        .bootctl_lba_count
        .checked_mul(SECTOR_BYTES as u64)
    else {
        return BootctlSlotWriteEvidence::missing(
            "persist_bootctl_byte_count_overflow",
            write_offset,
        );
    };

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return BootctlSlotWriteEvidence::missing("ahci_abar_missing", write_offset);
    };
    if !bar.is_memory() {
        return BootctlSlotWriteEvidence::missing("ahci_abar_not_mmio", write_offset);
    }
    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return BootctlSlotWriteEvidence::missing("ahci_abar_probe_range_too_small", write_offset);
    }
    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return BootctlSlotWriteEvidence::missing("ahci_abar_mmio_map_failed", write_offset);
    };
    let base = mapping.as_ptr::<u8>();
    let port_offset = PORT_BASE + layout.port_index as usize * PORT_STRIDE;
    if port_offset + PORT_CI + core::mem::size_of::<u32>() > mapping.len() {
        return BootctlSlotWriteEvidence::missing(
            "persist_bootctl_port_range_unavailable",
            write_offset,
        );
    }
    let ssts = read32(base, port_offset + PORT_SSTS);
    let signature = read32(base, port_offset + PORT_SIG);
    if (ssts & 0x0f) != 0x03 || ((ssts >> 8) & 0x0f) != 0x01 || signature != SATA_SIG_ATA {
        return BootctlSlotWriteEvidence::missing("persist_bootctl_port_not_ready", write_offset);
    }

    let mut evidence = BootctlSlotWriteEvidence {
        reason: "persist_bootctl_slot_span_validated",
        write_attempted: false,
        write_completed: false,
        readback_completed: false,
        span_in_bounds: false,
        absolute_start_lba,
        bootctl_lba_count: layout.data.bootctl_lba_count,
        byte_count,
        write_offset,
        readback: None,
    };

    if slot_bytes.len() != BOOTCTL_SLOT_SIZE {
        evidence.reason = "bootctl_slot_len_out_of_scope";
        return evidence;
    }
    if write_offset != 0 && write_offset != BOOTCTL_SLOT_SIZE as u64 {
        evidence.reason = "bootctl_write_offset_not_slot_aligned";
        return evidence;
    }
    if byte_count != BOOTCTL_REGION_BYTE_COUNT as u64 {
        evidence.reason = "bootctl_byte_count_out_of_scope";
        return evidence;
    }
    if layout.data.bootctl_lba_count != BOOTCTL_STORAGE_SLOT_SECTORS * 2 {
        evidence.reason = "bootctl_lba_count_out_of_scope";
        return evidence;
    }
    if write_offset
        .checked_add(BOOTCTL_SLOT_SIZE as u64)
        .map(|end| end > byte_count)
        .unwrap_or(true)
    {
        evidence.reason = "bootctl_region_full";
        return evidence;
    }
    let Some(bootctl_end_lba) = absolute_start_lba.checked_add(layout.data.bootctl_lba_count)
    else {
        evidence.reason = "persist_bootctl_end_lba_overflow";
        return evidence;
    };

    let sector_count = BOOTCTL_STORAGE_SLOT_SECTORS as usize;
    let start_sector = write_offset / SECTOR_BYTES as u64;
    let mut sector = 0usize;
    while sector < sector_count {
        let Some(lba) = absolute_start_lba
            .checked_add(start_sector)
            .and_then(|base_lba| base_lba.checked_add(sector as u64))
        else {
            evidence.reason = "persist_bootctl_slot_lba_overflow";
            return evidence;
        };
        if lba < absolute_start_lba || lba >= bootctl_end_lba {
            evidence.reason = "write_span_out_of_bootctl";
            return evidence;
        }
        if start_sector != 0 && start_sector != BOOTCTL_STORAGE_SLOT_SECTORS {
            evidence.reason = "write_span_out_of_bootctl";
            return evidence;
        }
        sector += 1;
    }
    evidence.span_in_bounds = true;

    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    sector = 0;
    while sector < sector_count {
        let lba = absolute_start_lba + start_sector + sector as u64;
        let slot_offset = sector * SECTOR_BYTES;
        let mut idx = 0usize;
        while idx < SECTOR_BYTES {
            ptr::write_volatile(buffer.add(idx), slot_bytes[slot_offset + idx]);
            idx += 1;
        }
        evidence.write_attempted = true;
        if let Err(reason) = issue_write_sector(controller, base, port_offset, lba, buffer) {
            evidence.reason = reason;
            return evidence;
        }
        sector += 1;
    }
    evidence.write_completed = true;

    let mut readback = Vec::new();
    readback.resize(BOOTCTL_SLOT_SIZE, 0);
    sector = 0;
    while sector < sector_count {
        let lba = absolute_start_lba + start_sector + sector as u64;
        ptr::write_bytes(buffer, 0, SECTOR_BYTES);
        if let Err(reason) = issue_read_sector_into(
            controller,
            base,
            port_offset,
            lba,
            buffer,
            "persist_bootctl_slot_readback_buffer_phys_missing",
            "persist_bootctl_slot_readback_timeout",
            "persist_bootctl_slot_readback_task_file_error",
            "persist_bootctl_slot_readback_incomplete",
        ) {
            evidence.reason = reason;
            return evidence;
        }

        let out = sector * SECTOR_BYTES;
        let mut idx = 0usize;
        while idx < SECTOR_BYTES {
            readback[out + idx] = ptr::read_volatile(buffer.add(idx));
            idx += 1;
        }
        sector += 1;
    }

    evidence.readback_completed = true;
    evidence.reason = "persist_bootctl_slot_write_readback_completed";
    evidence.readback = Some(readback);
    evidence
}

pub(crate) fn write_readback_scratch_sector_image(
    controller: PciMassStorageController,
    planned_image: &[u8; SECTOR_BYTES],
    planned_image_hash: [u8; 32],
) -> AhciScratchSectorWriteReadbackEvidence {
    {
        let cached = SCRATCH_SECTOR_WRITE_READBACK.lock();
        if let Some(evidence) = *cached {
            if evidence.planned_image_hash == planned_image_hash {
                return evidence;
            }
        }
    }

    let evidence = unsafe {
        write_readback_scratch_sector_image_uncached(controller, planned_image, planned_image_hash)
    };
    *SCRATCH_SECTOR_WRITE_READBACK.lock() = Some(evidence);
    evidence
}

pub(crate) fn write_readback_audit_rollback_target_sector_image(
    controller: PciMassStorageController,
    planned_image: &[u8; SECTOR_BYTES],
    planned_image_hash: [u8; 32],
) -> AhciAuditRollbackTargetSectorWriteReadbackEvidence {
    {
        let cached = AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK.lock();
        if let Some(evidence) = *cached {
            if evidence.planned_image_hash == planned_image_hash {
                return evidence;
            }
        }
    }

    let scratch = probe(controller).scratch_write_readback;
    let evidence = unsafe {
        write_readback_audit_rollback_target_sector_image_uncached(
            controller,
            scratch,
            planned_image,
            planned_image_hash,
        )
    };
    *AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK.lock() = Some(evidence);
    evidence
}

pub(crate) fn write_readback_audit_rollback_target_sector_image_for_authorized_append(
    controller: PciMassStorageController,
    planned_image: &[u8; SECTOR_BYTES],
    planned_image_hash: [u8; 32],
) -> AhciAuditRollbackTargetSectorWriteReadbackEvidence {
    let scratch = probe(controller).scratch_write_readback;
    let evidence = unsafe {
        write_readback_audit_rollback_target_sector_image_uncached(
            controller,
            scratch,
            planned_image,
            planned_image_hash,
        )
    };
    *AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK.lock() = Some(evidence);
    evidence
}

pub(crate) fn cached_audit_rollback_target_sector_write_readback(
    planned_image_hash: [u8; 32],
) -> Option<AhciAuditRollbackTargetSectorWriteReadbackEvidence> {
    let cached = AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK.lock();
    let evidence = (*cached)?;
    if evidence.planned_image_hash == planned_image_hash {
        Some(evidence)
    } else {
        None
    }
}

pub(crate) fn inspect_audit_rollback_target_sector_image(
    controller: PciMassStorageController,
    expected_sector_image_hash: [u8; 32],
    audit_record_offset: usize,
    audit_record_byte_length: usize,
    rollback_transaction_offset: usize,
    rollback_transaction_byte_length: usize,
    padding_offset: usize,
    padding_byte_length: usize,
) -> AhciAuditRollbackTargetSectorInspectionEvidence {
    {
        let cached = AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION.lock();
        if let Some(evidence) = *cached {
            if evidence.available
                && evidence.expected_sector_image_hash == expected_sector_image_hash
            {
                return evidence;
            }
        }
    }

    let scratch = probe(controller).scratch_write_readback;
    let evidence = unsafe {
        inspect_audit_rollback_target_sector_image_uncached(
            controller,
            scratch,
            expected_sector_image_hash,
            audit_record_offset,
            audit_record_byte_length,
            rollback_transaction_offset,
            rollback_transaction_byte_length,
            padding_offset,
            padding_byte_length,
        )
    };
    if evidence.available {
        *AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION.lock() = Some(evidence);
    }
    evidence
}

pub(crate) fn inspect_audit_rollback_target_sector_image_for_authorized_append(
    controller: PciMassStorageController,
    expected_sector_image_hash: [u8; 32],
    audit_record_offset: usize,
    audit_record_byte_length: usize,
    rollback_transaction_offset: usize,
    rollback_transaction_byte_length: usize,
    padding_offset: usize,
    padding_byte_length: usize,
) -> AhciAuditRollbackTargetSectorInspectionEvidence {
    let scratch = probe(controller).scratch_write_readback;
    let evidence = unsafe {
        inspect_audit_rollback_target_sector_image_uncached(
            controller,
            scratch,
            expected_sector_image_hash,
            audit_record_offset,
            audit_record_byte_length,
            rollback_transaction_offset,
            rollback_transaction_byte_length,
            padding_offset,
            padding_byte_length,
        )
    };
    if evidence.available {
        *AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION.lock() = Some(evidence);
    }
    evidence
}

pub(crate) fn cached_audit_rollback_target_sector_inspection(
    expected_sector_image_hash: [u8; 32],
) -> Option<AhciAuditRollbackTargetSectorInspectionEvidence> {
    let cached = AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION.lock();
    let evidence = (*cached)?;
    if evidence.available && evidence.expected_sector_image_hash == expected_sector_image_hash {
        Some(evidence)
    } else {
        None
    }
}

fn probe_uncached(controller: PciMassStorageController) -> AhciReadOnlyProbe {
    if controller.subclass != AHCI_SUBCLASS {
        return AhciReadOnlyProbe::missing("ahci_controller_not_observed");
    }

    let mut probe = AhciReadOnlyProbe {
        observed: true,
        controller_is_ahci: true,
        reason: "ahci_abar_missing",
        ..AhciReadOnlyProbe::missing("ahci_abar_missing")
    };

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return probe;
    };

    probe.abar_available = bar.is_memory();
    probe.abar_base = bar.base;
    probe.abar_size = bar.size;
    if !bar.is_memory() {
        probe.reason = "ahci_abar_not_mmio";
        return probe;
    }

    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        probe.reason = "ahci_abar_probe_range_too_small";
        return probe;
    }

    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        probe.reason = "ahci_abar_mmio_map_failed";
        return probe;
    };

    let base = mapping.as_ptr::<u8>();
    probe.registers_mapped = true;
    probe.capabilities = unsafe { read32(base, REG_CAP) };
    probe.version = unsafe { read32(base, REG_VS) };
    probe.ports_implemented_mask = unsafe { read32(base, REG_PI) };
    probe.implemented_port_count = probe.ports_implemented_mask.count_ones() as u8;
    probe.command_slots = (((probe.capabilities >> 8) & 0x1f) as u8).saturating_add(1);
    probe.supports_64bit = (probe.capabilities & (1 << 31)) != 0;
    probe.supports_ncq = (probe.capabilities & (1 << 30)) != 0;

    let Some(port) = first_port(probe.ports_implemented_mask) else {
        probe.reason = "ahci_port_inventory_missing";
        return probe;
    };

    let offset = PORT_BASE + port as usize * PORT_STRIDE;
    if offset + PORT_CI + core::mem::size_of::<u32>() > mapping.len() {
        probe.reason = "ahci_port_register_range_unavailable";
        return probe;
    }

    probe.first_port_index = port;
    probe.first_port_implemented = true;
    probe.first_port_cmd = unsafe { read32(base, offset + PORT_CMD) };
    probe.first_port_tfd = unsafe { read32(base, offset + PORT_TFD) };
    probe.first_port_signature = unsafe { read32(base, offset + PORT_SIG) };
    probe.first_port_ssts = unsafe { read32(base, offset + PORT_SSTS) };
    probe.first_port_device_present = (probe.first_port_ssts & 0x0f) == 0x03;
    probe.first_port_interface_active = ((probe.first_port_ssts >> 8) & 0x0f) == 0x01;
    if !probe.first_port_device_present {
        probe.reason = "ahci_port_device_not_present";
        return probe;
    }
    if !probe.first_port_interface_active {
        probe.reason = "ahci_port_interface_inactive";
        return probe;
    }
    if probe.first_port_signature != SATA_SIG_ATA {
        probe.reason = "ahci_first_port_not_ata";
        return probe;
    }

    match unsafe { issue_identify(controller, base, offset) } {
        Ok(identity) => {
            probe.identify_command_issued = true;
            probe.controller_register_write_attempted = true;
            probe.dma_started = true;
            probe.identify_completed = true;
            probe.identify_status = "identify_complete";
            probe.identity = identity;
            probe.block_device_identity_available = identity.available;
            match unsafe { issue_read_sector0(controller, base, offset) } {
                Ok(sector0) => {
                    probe.sector_read_attempted = true;
                    probe.sector_read_completed = true;
                    probe.sector_read_status = "sector0_read_complete";
                    probe.sector0 = sector0;
                    probe.sector_read_available = sector0.available;
                    let partition_inventory = unsafe {
                        parse_partition_inventory(
                            ptr::addr_of!(SECTOR0_BUFFER.0).cast::<u8>(),
                            sector0,
                        )
                    };
                    probe.partition_inventory = partition_inventory;
                    probe.partition_inventory_available = partition_inventory.available;
                    let block_driver =
                        build_read_only_block_driver(identity, sector0, partition_inventory);
                    probe.block_driver = block_driver;
                    probe.block_driver_available = block_driver.available;
                    let scratch = unsafe {
                        probe_scratch_write_readback(
                            controller,
                            base,
                            mapping.len(),
                            probe.ports_implemented_mask,
                            port,
                        )
                    };
                    probe.media_write_attempted = scratch.write_attempted;
                    probe.scratch_write_path_available = scratch.available;
                    probe.scratch_write_readback = scratch;
                    probe.audit_rollback_target_region = unsafe {
                        probe_audit_rollback_target_region(
                            controller,
                            base,
                            mapping.len(),
                            probe.ports_implemented_mask,
                            port,
                            scratch,
                        )
                    };
                    probe.reason = if !partition_inventory.available {
                        "persistence_partition_inventory_missing"
                    } else if block_driver.available {
                        "persistence_device_write_path_missing"
                    } else {
                        "persistence_block_driver_missing"
                    };
                }
                Err(status) => {
                    probe.sector_read_attempted = true;
                    probe.sector_read_status = status;
                    probe.reason = "persistence_sector_read_path_missing";
                }
            }
        }
        Err(status) => {
            probe.identify_command_issued = true;
            probe.controller_register_write_attempted = true;
            probe.dma_started = !method_is_pre_dma(status);
            probe.identify_status = status;
            probe.reason = "block_device_identity_missing";
        }
    }

    probe.first_port_cmd = unsafe { read32(base, offset + PORT_CMD) };
    probe.first_port_tfd = unsafe { read32(base, offset + PORT_TFD) };
    probe
}

unsafe fn write_readback_scratch_sector_image_uncached(
    controller: PciMassStorageController,
    planned_image: &[u8; SECTOR_BYTES],
    planned_image_hash: [u8; 32],
) -> AhciScratchSectorWriteReadbackEvidence {
    if controller.subclass != AHCI_SUBCLASS {
        return AhciScratchSectorWriteReadbackEvidence::missing(
            "ahci_controller_not_observed",
            planned_image_hash,
        );
    }

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return AhciScratchSectorWriteReadbackEvidence::missing(
            "ahci_abar_missing",
            planned_image_hash,
        );
    };
    if !bar.is_memory() {
        return AhciScratchSectorWriteReadbackEvidence::missing(
            "ahci_abar_not_mmio",
            planned_image_hash,
        );
    }

    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return AhciScratchSectorWriteReadbackEvidence::missing(
            "ahci_abar_probe_range_too_small",
            planned_image_hash,
        );
    }

    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return AhciScratchSectorWriteReadbackEvidence::missing(
            "ahci_abar_mmio_map_failed",
            planned_image_hash,
        );
    };

    let base = mapping.as_ptr::<u8>();
    let ports_mask = read32(base, REG_PI);
    let Some(boot_port) = first_port(ports_mask) else {
        return AhciScratchSectorWriteReadbackEvidence::missing(
            "ahci_port_inventory_missing",
            planned_image_hash,
        );
    };

    let mut saw_candidate = false;
    let mut port = 0u8;
    while port < 32 {
        if port != boot_port && (ports_mask & (1u32 << port)) != 0 {
            let offset = PORT_BASE + port as usize * PORT_STRIDE;
            if offset + PORT_CI + core::mem::size_of::<u32>() <= mapping.len() {
                let ssts = read32(base, offset + PORT_SSTS);
                let signature = read32(base, offset + PORT_SIG);
                if (ssts & 0x0f) == 0x03
                    && ((ssts >> 8) & 0x0f) == 0x01
                    && signature == SATA_SIG_ATA
                {
                    saw_candidate = true;
                    let evidence = write_readback_labeled_scratch_sector_image(
                        controller,
                        base,
                        offset,
                        port,
                        boot_port,
                        planned_image,
                        planned_image_hash,
                    );
                    if evidence.label_found || evidence.available {
                        return evidence;
                    }
                }
            }
        }
        port += 1;
    }

    if saw_candidate {
        AhciScratchSectorWriteReadbackEvidence::missing(
            "scratch_block_region_label_missing",
            planned_image_hash,
        )
    } else {
        AhciScratchSectorWriteReadbackEvidence::missing(
            "scratch_block_region_device_missing",
            planned_image_hash,
        )
    }
}

unsafe fn write_readback_audit_rollback_target_sector_image_uncached(
    controller: PciMassStorageController,
    scratch: AhciScratchWriteReadbackEvidence,
    planned_image: &[u8; SECTOR_BYTES],
    planned_image_hash: [u8; 32],
) -> AhciAuditRollbackTargetSectorWriteReadbackEvidence {
    if controller.subclass != AHCI_SUBCLASS {
        return AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "ahci_controller_not_observed",
            planned_image_hash,
        );
    }

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "ahci_abar_missing",
            planned_image_hash,
        );
    };
    if !bar.is_memory() {
        return AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "ahci_abar_not_mmio",
            planned_image_hash,
        );
    }

    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "ahci_abar_probe_range_too_small",
            planned_image_hash,
        );
    }

    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "ahci_abar_mmio_map_failed",
            planned_image_hash,
        );
    };

    let base = mapping.as_ptr::<u8>();
    let ports_mask = read32(base, REG_PI);
    let Some(boot_port) = first_port(ports_mask) else {
        return AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "ahci_port_inventory_missing",
            planned_image_hash,
        );
    };

    let mut saw_candidate = false;
    let mut port = 0u8;
    while port < 32 {
        if port != boot_port && (ports_mask & (1u32 << port)) != 0 {
            let offset = PORT_BASE + port as usize * PORT_STRIDE;
            if offset + PORT_CI + core::mem::size_of::<u32>() <= mapping.len() {
                let ssts = read32(base, offset + PORT_SSTS);
                let signature = read32(base, offset + PORT_SIG);
                if (ssts & 0x0f) == 0x03
                    && ((ssts >> 8) & 0x0f) == 0x01
                    && signature == SATA_SIG_ATA
                {
                    saw_candidate = true;
                    let evidence = write_readback_labeled_audit_rollback_target_sector_image(
                        controller,
                        base,
                        offset,
                        port,
                        boot_port,
                        scratch,
                        planned_image,
                        planned_image_hash,
                    );
                    if evidence.label_found || evidence.available {
                        return evidence;
                    }
                }
            }
        }
        port += 1;
    }

    if saw_candidate {
        AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "audit_rollback_target_region_label_missing",
            planned_image_hash,
        )
    } else {
        AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "audit_rollback_target_region_device_missing",
            planned_image_hash,
        )
    }
}

unsafe fn inspect_audit_rollback_target_sector_image_uncached(
    controller: PciMassStorageController,
    scratch: AhciScratchWriteReadbackEvidence,
    expected_sector_image_hash: [u8; 32],
    audit_record_offset: usize,
    audit_record_byte_length: usize,
    rollback_transaction_offset: usize,
    rollback_transaction_byte_length: usize,
    padding_offset: usize,
    padding_byte_length: usize,
) -> AhciAuditRollbackTargetSectorInspectionEvidence {
    if controller.subclass != AHCI_SUBCLASS {
        return AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "ahci_controller_not_observed",
            expected_sector_image_hash,
        );
    }

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "ahci_abar_missing",
            expected_sector_image_hash,
        );
    };
    if !bar.is_memory() {
        return AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "ahci_abar_not_mmio",
            expected_sector_image_hash,
        );
    }

    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "ahci_abar_probe_range_too_small",
            expected_sector_image_hash,
        );
    }

    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "ahci_abar_mmio_map_failed",
            expected_sector_image_hash,
        );
    };

    let base = mapping.as_ptr::<u8>();
    let ports_mask = read32(base, REG_PI);
    let Some(boot_port) = first_port(ports_mask) else {
        return AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "ahci_port_inventory_missing",
            expected_sector_image_hash,
        );
    };

    let mut saw_candidate = false;
    let mut port = 0u8;
    while port < 32 {
        if port != boot_port && (ports_mask & (1u32 << port)) != 0 {
            let offset = PORT_BASE + port as usize * PORT_STRIDE;
            if offset + PORT_CI + core::mem::size_of::<u32>() <= mapping.len() {
                let ssts = read32(base, offset + PORT_SSTS);
                let signature = read32(base, offset + PORT_SIG);
                if (ssts & 0x0f) == 0x03
                    && ((ssts >> 8) & 0x0f) == 0x01
                    && signature == SATA_SIG_ATA
                {
                    saw_candidate = true;
                    let evidence = inspect_labeled_audit_rollback_target_sector_image(
                        controller,
                        base,
                        offset,
                        port,
                        boot_port,
                        scratch,
                        expected_sector_image_hash,
                        audit_record_offset,
                        audit_record_byte_length,
                        rollback_transaction_offset,
                        rollback_transaction_byte_length,
                        padding_offset,
                        padding_byte_length,
                    );
                    if evidence.label_found || evidence.available {
                        return evidence;
                    }
                }
            }
        }
        port += 1;
    }

    if saw_candidate {
        AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "audit_rollback_target_region_label_missing",
            expected_sector_image_hash,
        )
    } else {
        AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "audit_rollback_target_region_device_missing",
            expected_sector_image_hash,
        )
    }
}

fn build_read_only_block_driver(
    identity: AhciBlockDeviceIdentity,
    sector0: AhciSectorReadEvidence,
    partition_inventory: AhciPartitionInventoryEvidence,
) -> AhciReadOnlyBlockDriverEvidence {
    let binds_partition_inventory = partition_inventory.available
        && sector0.available
        && partition_inventory.source_lba == sector0.lba;
    if !identity.available
        || identity.logical_sector_size_bytes != SECTOR_BYTES as u32
        || !sector0.available
        || sector0.byte_count != identity.logical_sector_size_bytes
        || !binds_partition_inventory
    {
        return AhciReadOnlyBlockDriverEvidence::missing();
    }

    AhciReadOnlyBlockDriverEvidence {
        available: true,
        driver_id: "block_driver.ahci.read_only.current_boot",
        source: "ahci_dma_read_verified_sector0",
        logical_sector_size_bytes: identity.logical_sector_size_bytes,
        lba28_sector_count: identity.lba28_sector_count,
        lba48_sector_count: identity.lba48_sector_count,
        verified_read_lba: sector0.lba,
        verified_read_byte_count: sector0.byte_count,
        binds_partition_inventory,
        supports_read: true,
        supports_write: false,
    }
}

unsafe fn read_persist_reclog_region_uncached(
    controller: PciMassStorageController,
    layout: PersistLayoutEvidence,
) -> PersistReclogRegionRead {
    if layout.gpt.status != LayoutStatus::Present || layout.data.status != LayoutStatus::Present {
        return PersistReclogRegionRead::missing(layout, layout.reason);
    }
    if layout.port_index == u8::MAX {
        return PersistReclogRegionRead::missing(layout, "persist_reclog_port_missing");
    }
    if layout.data.reclog_lba_count == 0
        || layout.data.reclog_start_lba > layout.gpt.seed_data_lba_count
        || layout.data.reclog_lba_count
            > layout
                .gpt
                .seed_data_lba_count
                .saturating_sub(layout.data.reclog_start_lba)
    {
        let mut read =
            PersistReclogRegionRead::missing(layout, "persist_reclog_region_bounds_invalid");
        read.region_bounds_valid = false;
        return read;
    }

    let Some(absolute_start_lba) = layout
        .gpt
        .seed_data_first_lba
        .checked_add(layout.data.reclog_start_lba)
    else {
        return PersistReclogRegionRead::missing(layout, "persist_reclog_start_lba_overflow");
    };
    let Some(byte_count) = layout
        .data
        .reclog_lba_count
        .checked_mul(SECTOR_BYTES as u64)
    else {
        return PersistReclogRegionRead::missing(layout, "persist_reclog_byte_count_overflow");
    };
    let Ok(byte_count_usize) = usize::try_from(byte_count) else {
        return PersistReclogRegionRead::missing(layout, "persist_reclog_byte_count_too_large");
    };

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return PersistReclogRegionRead::missing(layout, "ahci_abar_missing");
    };
    if !bar.is_memory() {
        return PersistReclogRegionRead::missing(layout, "ahci_abar_not_mmio");
    }
    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return PersistReclogRegionRead::missing(layout, "ahci_abar_probe_range_too_small");
    }
    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return PersistReclogRegionRead::missing(layout, "ahci_abar_mmio_map_failed");
    };
    let base = mapping.as_ptr::<u8>();
    let port_offset = PORT_BASE + layout.port_index as usize * PORT_STRIDE;
    if port_offset + PORT_CI + core::mem::size_of::<u32>() > mapping.len() {
        return PersistReclogRegionRead::missing(layout, "persist_reclog_port_range_unavailable");
    }
    let ssts = read32(base, port_offset + PORT_SSTS);
    let signature = read32(base, port_offset + PORT_SIG);
    if (ssts & 0x0f) != 0x03 || ((ssts >> 8) & 0x0f) != 0x01 || signature != SATA_SIG_ATA {
        return PersistReclogRegionRead::missing(layout, "persist_reclog_port_not_ready");
    }

    let mut bytes = Vec::new();
    bytes.resize(byte_count_usize, 0);
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    let mut sector = 0u64;
    while sector < layout.data.reclog_lba_count {
        ptr::write_bytes(buffer, 0, SECTOR_BYTES);
        if let Err(reason) = issue_read_sector_into(
            controller,
            base,
            port_offset,
            absolute_start_lba + sector,
            buffer,
            "persist_reclog_sector_buffer_phys_missing",
            "persist_reclog_sector_read_timeout",
            "persist_reclog_sector_read_task_file_error",
            "persist_reclog_sector_read_incomplete",
        ) {
            let mut read = PersistReclogRegionRead::missing(layout, reason);
            read.read_attempted = true;
            read.region_bounds_valid = true;
            read.absolute_start_lba = absolute_start_lba;
            read.lba_count = layout.data.reclog_lba_count;
            read.byte_count = byte_count;
            return read;
        }

        let out = sector as usize * SECTOR_BYTES;
        let mut idx = 0usize;
        while idx < SECTOR_BYTES {
            bytes[out + idx] = ptr::read_volatile(buffer.add(idx));
            idx += 1;
        }
        sector += 1;
    }

    PersistReclogRegionRead {
        layout,
        reason: "persist_reclog_region_read_completed",
        read_attempted: true,
        read_completed: true,
        region_bounds_valid: true,
        absolute_start_lba,
        lba_count: layout.data.reclog_lba_count,
        byte_count,
        bytes: Some(bytes),
    }
}

unsafe fn read_persist_bootctl_region_uncached(
    controller: PciMassStorageController,
    layout: PersistLayoutEvidence,
) -> PersistBootctlRegionRead {
    if layout.gpt.status != LayoutStatus::Present || layout.data.status != LayoutStatus::Present {
        return PersistBootctlRegionRead::missing(layout, layout.reason);
    }
    if layout.port_index == u8::MAX {
        return PersistBootctlRegionRead::missing(layout, "persist_bootctl_port_missing");
    }
    if layout.data.bootctl_lba_count == 0
        || layout.data.bootctl_start_lba > layout.gpt.seed_data_lba_count
        || layout.data.bootctl_lba_count
            > layout
                .gpt
                .seed_data_lba_count
                .saturating_sub(layout.data.bootctl_start_lba)
    {
        let mut read =
            PersistBootctlRegionRead::missing(layout, "persist_bootctl_region_bounds_invalid");
        read.region_bounds_valid = false;
        return read;
    }

    let Some(absolute_start_lba) = layout
        .gpt
        .seed_data_first_lba
        .checked_add(layout.data.bootctl_start_lba)
    else {
        return PersistBootctlRegionRead::missing(layout, "persist_bootctl_start_lba_overflow");
    };
    let Some(byte_count) = layout
        .data
        .bootctl_lba_count
        .checked_mul(SECTOR_BYTES as u64)
    else {
        return PersistBootctlRegionRead::missing(layout, "persist_bootctl_byte_count_overflow");
    };
    let Ok(byte_count_usize) = usize::try_from(byte_count) else {
        return PersistBootctlRegionRead::missing(layout, "persist_bootctl_byte_count_too_large");
    };

    let Some(bar) = pci::read_bar_info(controller.address, AHCI_BAR) else {
        return PersistBootctlRegionRead::missing(layout, "ahci_abar_missing");
    };
    if !bar.is_memory() {
        return PersistBootctlRegionRead::missing(layout, "ahci_abar_not_mmio");
    }
    let map_len = usize::min(bar.size as usize, MAX_PROBE_LEN);
    if map_len < MIN_PROBE_LEN {
        return PersistBootctlRegionRead::missing(layout, "ahci_abar_probe_range_too_small");
    }
    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return PersistBootctlRegionRead::missing(layout, "ahci_abar_mmio_map_failed");
    };
    let base = mapping.as_ptr::<u8>();
    let port_offset = PORT_BASE + layout.port_index as usize * PORT_STRIDE;
    if port_offset + PORT_CI + core::mem::size_of::<u32>() > mapping.len() {
        return PersistBootctlRegionRead::missing(layout, "persist_bootctl_port_range_unavailable");
    }
    let ssts = read32(base, port_offset + PORT_SSTS);
    let signature = read32(base, port_offset + PORT_SIG);
    if (ssts & 0x0f) != 0x03 || ((ssts >> 8) & 0x0f) != 0x01 || signature != SATA_SIG_ATA {
        return PersistBootctlRegionRead::missing(layout, "persist_bootctl_port_not_ready");
    }

    let mut bytes = Vec::new();
    bytes.resize(byte_count_usize, 0);
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    let mut sector = 0u64;
    while sector < layout.data.bootctl_lba_count {
        ptr::write_bytes(buffer, 0, SECTOR_BYTES);
        if let Err(reason) = issue_read_sector_into(
            controller,
            base,
            port_offset,
            absolute_start_lba + sector,
            buffer,
            "persist_bootctl_sector_buffer_phys_missing",
            "persist_bootctl_sector_read_timeout",
            "persist_bootctl_sector_read_task_file_error",
            "persist_bootctl_sector_read_incomplete",
        ) {
            let mut read = PersistBootctlRegionRead::missing(layout, reason);
            read.read_attempted = true;
            read.region_bounds_valid = true;
            read.absolute_start_lba = absolute_start_lba;
            read.lba_count = layout.data.bootctl_lba_count;
            read.byte_count = byte_count;
            return read;
        }

        let out = sector as usize * SECTOR_BYTES;
        let mut idx = 0usize;
        while idx < SECTOR_BYTES {
            bytes[out + idx] = ptr::read_volatile(buffer.add(idx));
            idx += 1;
        }
        sector += 1;
    }

    PersistBootctlRegionRead {
        layout,
        reason: "persist_bootctl_region_read_completed",
        read_attempted: true,
        read_completed: true,
        region_bounds_valid: true,
        absolute_start_lba,
        lba_count: layout.data.bootctl_lba_count,
        byte_count,
        bytes: Some(bytes),
    }
}

unsafe fn issue_read_sector0(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
) -> Result<AhciSectorReadEvidence, &'static str> {
    ptr::write_bytes(
        ptr::addr_of_mut!(SECTOR0_BUFFER.0).cast::<u8>(),
        0,
        SECTOR_BYTES,
    );
    issue_dma_read_command(
        controller,
        base,
        port_offset,
        READ_DMA_EXT,
        0,
        ptr::addr_of_mut!(SECTOR0_BUFFER.0).cast::<u8>(),
        SECTOR_BYTES,
        "ahci_sector0_buffer_phys_missing",
        "ahci_sector0_read_timeout",
        "ahci_sector0_read_task_file_error",
        "ahci_sector0_read_incomplete",
    )?;
    Ok(parse_sector0(ptr::addr_of!(SECTOR0_BUFFER.0).cast::<u8>()))
}

unsafe fn issue_write_sector(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    lba: u64,
    data_buffer: *mut u8,
) -> Result<(), &'static str> {
    issue_write_sector_with_reasons(
        controller,
        base,
        port_offset,
        lba,
        data_buffer,
        "ahci_scratch_write_buffer_phys_missing",
        "ahci_scratch_write_timeout",
        "ahci_scratch_write_task_file_error",
        "ahci_scratch_write_incomplete",
    )
}

unsafe fn issue_write_sector_with_reasons(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    lba: u64,
    data_buffer: *mut u8,
    data_phys_reason: &'static str,
    timeout_reason: &'static str,
    task_file_reason: &'static str,
    incomplete_reason: &'static str,
) -> Result<(), &'static str> {
    issue_dma_command(
        controller,
        base,
        port_offset,
        WRITE_DMA_EXT,
        lba,
        true,
        data_buffer,
        SECTOR_BYTES,
        data_phys_reason,
        timeout_reason,
        task_file_reason,
        incomplete_reason,
    )
}

unsafe fn issue_read_sector_into(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    lba: u64,
    data_buffer: *mut u8,
    data_phys_reason: &'static str,
    timeout_reason: &'static str,
    task_file_reason: &'static str,
    incomplete_reason: &'static str,
) -> Result<(), &'static str> {
    issue_dma_read_command(
        controller,
        base,
        port_offset,
        READ_DMA_EXT,
        lba,
        data_buffer,
        SECTOR_BYTES,
        data_phys_reason,
        timeout_reason,
        task_file_reason,
        incomplete_reason,
    )
}

unsafe fn issue_identify(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
) -> Result<AhciBlockDeviceIdentity, &'static str> {
    ptr::write_bytes(
        ptr::addr_of_mut!(IDENTIFY_BUFFER.0).cast::<u8>(),
        0,
        IDENTIFY_BYTES,
    );
    issue_dma_read_command(
        controller,
        base,
        port_offset,
        IDENTIFY_DEVICE,
        0,
        ptr::addr_of_mut!(IDENTIFY_BUFFER.0).cast::<u8>(),
        IDENTIFY_BYTES,
        "ahci_identify_buffer_phys_missing",
        "ahci_identify_timeout",
        "ahci_identify_task_file_error",
        "ahci_identify_incomplete",
    )?;

    Ok(parse_identify(
        ptr::addr_of!(IDENTIFY_BUFFER.0).cast::<u8>(),
    ))
}

unsafe fn probe_scratch_write_readback(
    controller: PciMassStorageController,
    base: *mut u8,
    map_len: usize,
    ports_mask: u32,
    boot_port: u8,
) -> AhciScratchWriteReadbackEvidence {
    let mut saw_candidate = false;
    let mut port = 0u8;
    while port < 32 {
        if port != boot_port && (ports_mask & (1u32 << port)) != 0 {
            let offset = PORT_BASE + port as usize * PORT_STRIDE;
            if offset + PORT_CI + core::mem::size_of::<u32>() <= map_len {
                let ssts = read32(base, offset + PORT_SSTS);
                let signature = read32(base, offset + PORT_SIG);
                if (ssts & 0x0f) == 0x03
                    && ((ssts >> 8) & 0x0f) == 0x01
                    && signature == SATA_SIG_ATA
                {
                    saw_candidate = true;
                    let evidence =
                        probe_labeled_scratch_port(controller, base, offset, port, boot_port);
                    if evidence.label_found || evidence.available {
                        return evidence;
                    }
                }
            }
        }
        port += 1;
    }

    if saw_candidate {
        AhciScratchWriteReadbackEvidence::missing("scratch_block_region_label_missing")
    } else {
        AhciScratchWriteReadbackEvidence::missing("scratch_block_region_device_missing")
    }
}

unsafe fn probe_labeled_scratch_port(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    port_index: u8,
    boot_port: u8,
) -> AhciScratchWriteReadbackEvidence {
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        SCRATCH_MARKER_LBA,
        buffer,
        "ahci_scratch_label_buffer_phys_missing",
        "ahci_scratch_label_read_timeout",
        "ahci_scratch_label_read_task_file_error",
        "ahci_scratch_label_read_incomplete",
    ) {
        let mut evidence = AhciScratchWriteReadbackEvidence::missing(reason);
        evidence.port_index = port_index;
        return evidence;
    }

    if !scratch_marker_matches(buffer) {
        let mut evidence =
            AhciScratchWriteReadbackEvidence::missing("scratch_block_region_label_missing");
        evidence.port_index = port_index;
        return evidence;
    }

    let mut evidence = AhciScratchWriteReadbackEvidence {
        port_index,
        label_found: true,
        status: "label_found",
        reason: "scratch_block_region_label_found",
        ..AhciScratchWriteReadbackEvidence::missing("scratch_block_region_label_found")
    };

    match issue_identify(controller, base, port_offset) {
        Ok(identity) => evidence.device_identity = identity,
        Err(reason) => {
            evidence.status = "missing";
            evidence.reason = reason;
            return evidence;
        }
    }

    let sector_count = identity_sector_count(evidence.device_identity);
    evidence.region_within_device_bounds = evidence.device_identity.available
        && evidence.device_identity.logical_sector_size_bytes == SECTOR_BYTES as u32
        && evidence.region_lba_count > 0
        && evidence.region_start_lba <= sector_count
        && evidence.region_lba_count <= sector_count.saturating_sub(evidence.region_start_lba);
    evidence.boot_port_overlap = port_index == boot_port;

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        SCRATCH_TEST_LBA,
        buffer,
        "ahci_scratch_prewrite_buffer_phys_missing",
        "ahci_scratch_prewrite_read_timeout",
        "ahci_scratch_prewrite_read_task_file_error",
        "ahci_scratch_prewrite_read_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }

    evidence.metadata_lba_overlap = evidence.region_start_lba == 0 || sector_has_gpt_header(buffer);
    evidence.no_boot_or_partition_metadata_overlap =
        !evidence.boot_port_overlap && !evidence.metadata_lba_overlap;
    if !evidence.region_within_device_bounds {
        evidence.status = "missing";
        evidence.reason = "scratch_block_region_bounds_invalid";
        return evidence;
    }
    if !evidence.no_boot_or_partition_metadata_overlap {
        evidence.status = "missing";
        evidence.reason = "scratch_block_region_metadata_overlap";
        return evidence;
    }

    fill_scratch_pattern(buffer);
    evidence.write_attempted = true;
    if let Err(reason) = issue_write_sector(controller, base, port_offset, SCRATCH_TEST_LBA, buffer)
    {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }
    evidence.write_completed = true;

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        SCRATCH_TEST_LBA,
        buffer,
        "ahci_scratch_readback_buffer_phys_missing",
        "ahci_scratch_readback_timeout",
        "ahci_scratch_readback_task_file_error",
        "ahci_scratch_readback_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }
    evidence.readback_completed = true;
    evidence.readback_matches = scratch_pattern_matches(buffer);
    evidence.readback_first16 = first16_from_buffer(buffer);
    if evidence.readback_matches {
        evidence.available = true;
        evidence.block_write_authority_available = true;
        evidence.status = "available";
        evidence.reason = "scratch_write_readback_verified";
    } else {
        evidence.status = "missing";
        evidence.reason = "scratch_readback_mismatch";
    }
    evidence
}

unsafe fn probe_audit_rollback_target_region(
    controller: PciMassStorageController,
    base: *mut u8,
    map_len: usize,
    ports_mask: u32,
    boot_port: u8,
    scratch: AhciScratchWriteReadbackEvidence,
) -> AhciAuditRollbackTargetRegionEvidence {
    let mut saw_candidate = false;
    let mut port = 0u8;
    while port < 32 {
        if port != boot_port && (ports_mask & (1u32 << port)) != 0 {
            let offset = PORT_BASE + port as usize * PORT_STRIDE;
            if offset + PORT_CI + core::mem::size_of::<u32>() <= map_len {
                let ssts = read32(base, offset + PORT_SSTS);
                let signature = read32(base, offset + PORT_SIG);
                if (ssts & 0x0f) == 0x03
                    && ((ssts >> 8) & 0x0f) == 0x01
                    && signature == SATA_SIG_ATA
                {
                    saw_candidate = true;
                    let evidence = probe_labeled_audit_rollback_target_port(
                        controller, base, offset, port, boot_port, scratch,
                    );
                    if evidence.label_found || evidence.available {
                        return evidence;
                    }
                }
            }
        }
        port += 1;
    }

    if saw_candidate {
        AhciAuditRollbackTargetRegionEvidence::missing("audit_rollback_target_region_label_missing")
    } else {
        AhciAuditRollbackTargetRegionEvidence::missing(
            "audit_rollback_target_region_device_missing",
        )
    }
}

unsafe fn probe_labeled_audit_rollback_target_port(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    port_index: u8,
    boot_port: u8,
    scratch: AhciScratchWriteReadbackEvidence,
) -> AhciAuditRollbackTargetRegionEvidence {
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_MARKER_LBA,
        buffer,
        "ahci_audit_rollback_target_label_buffer_phys_missing",
        "ahci_audit_rollback_target_label_read_timeout",
        "ahci_audit_rollback_target_label_read_task_file_error",
        "ahci_audit_rollback_target_label_read_incomplete",
    ) {
        let mut evidence = AhciAuditRollbackTargetRegionEvidence::missing(reason);
        evidence.port_index = port_index;
        return evidence;
    }

    if !audit_rollback_target_marker_matches(buffer) {
        let mut evidence = AhciAuditRollbackTargetRegionEvidence::missing(
            "audit_rollback_target_region_label_missing",
        );
        evidence.port_index = port_index;
        return evidence;
    }

    let mut evidence = AhciAuditRollbackTargetRegionEvidence {
        port_index,
        label_found: true,
        status: "label_found",
        reason: "audit_rollback_target_region_label_found",
        ..AhciAuditRollbackTargetRegionEvidence::missing("audit_rollback_target_region_label_found")
    };

    match issue_identify(controller, base, port_offset) {
        Ok(identity) => evidence.device_identity = identity,
        Err(reason) => {
            evidence.status = "missing";
            evidence.reason = reason;
            return evidence;
        }
    }

    let sector_count = identity_sector_count(evidence.device_identity);
    evidence.region_within_device_bounds = evidence.device_identity.available
        && evidence.device_identity.logical_sector_size_bytes == SECTOR_BYTES as u32
        && evidence.region_lba_count > 0
        && evidence.region_start_lba <= sector_count
        && evidence.region_lba_count <= sector_count.saturating_sub(evidence.region_start_lba);
    evidence.boot_port_overlap = port_index == boot_port;
    evidence.scratch_region_overlap = scratch.available
        && scratch.port_index == port_index
        && ranges_overlap(
            evidence.region_start_lba,
            evidence.region_lba_count,
            scratch.region_start_lba,
            scratch.region_lba_count,
        );

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    evidence.read_attempted = true;
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
        buffer,
        "ahci_audit_rollback_target_region_buffer_phys_missing",
        "ahci_audit_rollback_target_region_read_timeout",
        "ahci_audit_rollback_target_region_read_task_file_error",
        "ahci_audit_rollback_target_region_read_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }
    evidence.read_completed = true;
    evidence.read_first16 = first16_from_buffer(buffer);
    evidence.metadata_lba_overlap = evidence.region_start_lba == 0 || sector_has_gpt_header(buffer);
    evidence.no_boot_or_partition_metadata_overlap = !evidence.boot_port_overlap
        && !evidence.metadata_lba_overlap
        && !evidence.scratch_region_overlap;
    if !evidence.region_within_device_bounds {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_region_bounds_invalid";
        return evidence;
    }
    if !evidence.no_boot_or_partition_metadata_overlap {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_region_overlap";
        return evidence;
    }

    evidence.available = true;
    evidence.status = "available";
    evidence.reason = "dedicated_audit_rollback_region_discovered_read_only";
    evidence
}

unsafe fn write_readback_labeled_scratch_sector_image(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    port_index: u8,
    boot_port: u8,
    planned_image: &[u8; SECTOR_BYTES],
    planned_image_hash: [u8; 32],
) -> AhciScratchSectorWriteReadbackEvidence {
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        SCRATCH_MARKER_LBA,
        buffer,
        "ahci_scratch_label_buffer_phys_missing",
        "ahci_scratch_label_read_timeout",
        "ahci_scratch_label_read_task_file_error",
        "ahci_scratch_label_read_incomplete",
    ) {
        let mut evidence =
            AhciScratchSectorWriteReadbackEvidence::missing(reason, planned_image_hash);
        evidence.port_index = port_index;
        return evidence;
    }

    if !scratch_marker_matches(buffer) {
        let mut evidence = AhciScratchSectorWriteReadbackEvidence::missing(
            "scratch_block_region_label_missing",
            planned_image_hash,
        );
        evidence.port_index = port_index;
        return evidence;
    }

    let mut evidence = AhciScratchSectorWriteReadbackEvidence {
        port_index,
        label_found: true,
        status: "label_found",
        reason: "scratch_block_region_label_found",
        ..AhciScratchSectorWriteReadbackEvidence::missing(
            "scratch_block_region_label_found",
            planned_image_hash,
        )
    };

    let identity = match issue_identify(controller, base, port_offset) {
        Ok(identity) => identity,
        Err(reason) => {
            evidence.status = "missing";
            evidence.reason = reason;
            return evidence;
        }
    };

    let sector_count = identity_sector_count(identity);
    evidence.region_within_device_bounds = identity.available
        && identity.logical_sector_size_bytes == SECTOR_BYTES as u32
        && SCRATCH_TEST_LBA < sector_count;
    evidence.boot_port_overlap = port_index == boot_port;

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        SCRATCH_TEST_LBA,
        buffer,
        "ahci_scratch_prewrite_buffer_phys_missing",
        "ahci_scratch_prewrite_read_timeout",
        "ahci_scratch_prewrite_read_task_file_error",
        "ahci_scratch_prewrite_read_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }

    evidence.metadata_lba_overlap = SCRATCH_TEST_LBA == 0 || sector_has_gpt_header(buffer);
    evidence.no_boot_or_partition_metadata_overlap =
        !evidence.boot_port_overlap && !evidence.metadata_lba_overlap;
    if !evidence.region_within_device_bounds {
        evidence.status = "missing";
        evidence.reason = "scratch_block_region_bounds_invalid";
        return evidence;
    }
    if !evidence.no_boot_or_partition_metadata_overlap {
        evidence.status = "missing";
        evidence.reason = "scratch_block_region_metadata_overlap";
        return evidence;
    }

    copy_sector_image_to_buffer(buffer, planned_image);
    evidence.write_attempted = true;
    if let Err(reason) = issue_write_sector(controller, base, port_offset, SCRATCH_TEST_LBA, buffer)
    {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }
    evidence.write_completed = true;

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        SCRATCH_TEST_LBA,
        buffer,
        "ahci_scratch_readback_buffer_phys_missing",
        "ahci_scratch_readback_timeout",
        "ahci_scratch_readback_task_file_error",
        "ahci_scratch_readback_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }

    evidence.readback_completed = true;
    evidence.readback_image_hash = sha256_sector_buffer(buffer);
    evidence.readback_matches_planned_image = evidence.readback_image_hash == planned_image_hash;
    evidence.readback_first16 = first16_from_buffer(buffer);
    if evidence.readback_matches_planned_image {
        evidence.available = true;
        evidence.status = "available";
        evidence.reason = "scratch_sector_image_readback_verified";
    } else {
        evidence.status = "missing";
        evidence.reason = "scratch_sector_image_readback_mismatch";
    }
    evidence
}

unsafe fn write_readback_labeled_audit_rollback_target_sector_image(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    port_index: u8,
    boot_port: u8,
    scratch: AhciScratchWriteReadbackEvidence,
    planned_image: &[u8; SECTOR_BYTES],
    planned_image_hash: [u8; 32],
) -> AhciAuditRollbackTargetSectorWriteReadbackEvidence {
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_MARKER_LBA,
        buffer,
        "ahci_audit_rollback_target_label_buffer_phys_missing",
        "ahci_audit_rollback_target_label_read_timeout",
        "ahci_audit_rollback_target_label_read_task_file_error",
        "ahci_audit_rollback_target_label_read_incomplete",
    ) {
        let mut evidence =
            AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(reason, planned_image_hash);
        evidence.port_index = port_index;
        return evidence;
    }

    if !audit_rollback_target_marker_matches(buffer) {
        let mut evidence = AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "audit_rollback_target_region_label_missing",
            planned_image_hash,
        );
        evidence.port_index = port_index;
        return evidence;
    }

    let mut evidence = AhciAuditRollbackTargetSectorWriteReadbackEvidence {
        port_index,
        label_found: true,
        status: "label_found",
        reason: "audit_rollback_target_region_label_found",
        ..AhciAuditRollbackTargetSectorWriteReadbackEvidence::missing(
            "audit_rollback_target_region_label_found",
            planned_image_hash,
        )
    };

    let identity = match issue_identify(controller, base, port_offset) {
        Ok(identity) => identity,
        Err(reason) => {
            evidence.status = "missing";
            evidence.reason = reason;
            return evidence;
        }
    };

    let sector_count = identity_sector_count(identity);
    evidence.region_within_device_bounds = identity.available
        && identity.logical_sector_size_bytes == SECTOR_BYTES as u32
        && AUDIT_ROLLBACK_TARGET_REGION_START_LBA < sector_count
        && AUDIT_ROLLBACK_TARGET_REGION_LBA_COUNT
            <= sector_count.saturating_sub(AUDIT_ROLLBACK_TARGET_REGION_START_LBA);
    evidence.boot_port_overlap = port_index == boot_port;
    evidence.scratch_region_overlap = scratch.available
        && scratch.port_index == port_index
        && ranges_overlap(
            AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
            AUDIT_ROLLBACK_TARGET_REGION_LBA_COUNT,
            scratch.region_start_lba,
            scratch.region_lba_count,
        );

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
        buffer,
        "ahci_audit_rollback_target_prewrite_buffer_phys_missing",
        "ahci_audit_rollback_target_prewrite_read_timeout",
        "ahci_audit_rollback_target_prewrite_read_task_file_error",
        "ahci_audit_rollback_target_prewrite_read_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }

    evidence.metadata_lba_overlap =
        AUDIT_ROLLBACK_TARGET_REGION_START_LBA == 0 || sector_has_gpt_header(buffer);
    evidence.no_boot_or_partition_metadata_overlap = !evidence.boot_port_overlap
        && !evidence.metadata_lba_overlap
        && !evidence.scratch_region_overlap;
    if !evidence.region_within_device_bounds {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_region_bounds_invalid";
        return evidence;
    }
    if !evidence.no_boot_or_partition_metadata_overlap {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_region_overlap";
        return evidence;
    }

    copy_sector_image_to_buffer(buffer, planned_image);
    evidence.write_attempted = true;
    if let Err(reason) = issue_write_sector_with_reasons(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
        buffer,
        "ahci_audit_rollback_target_write_buffer_phys_missing",
        "ahci_audit_rollback_target_write_timeout",
        "ahci_audit_rollback_target_write_task_file_error",
        "ahci_audit_rollback_target_write_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }
    evidence.write_completed = true;

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
        buffer,
        "ahci_audit_rollback_target_readback_buffer_phys_missing",
        "ahci_audit_rollback_target_readback_timeout",
        "ahci_audit_rollback_target_readback_task_file_error",
        "ahci_audit_rollback_target_readback_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }

    evidence.readback_completed = true;
    evidence.readback_image_hash = sha256_sector_buffer(buffer);
    evidence.readback_matches_planned_image = evidence.readback_image_hash == planned_image_hash;
    evidence.readback_first16 = first16_from_buffer(buffer);
    if evidence.readback_matches_planned_image {
        evidence.available = true;
        evidence.status = "available";
        evidence.reason = "audit_rollback_target_sector_image_readback_verified";
    } else {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_sector_image_readback_mismatch";
    }
    evidence
}

unsafe fn inspect_labeled_audit_rollback_target_sector_image(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    port_index: u8,
    boot_port: u8,
    scratch: AhciScratchWriteReadbackEvidence,
    expected_sector_image_hash: [u8; 32],
    audit_record_offset: usize,
    audit_record_byte_length: usize,
    rollback_transaction_offset: usize,
    rollback_transaction_byte_length: usize,
    padding_offset: usize,
    padding_byte_length: usize,
) -> AhciAuditRollbackTargetSectorInspectionEvidence {
    let buffer = ptr::addr_of_mut!(SCRATCH_BUFFER.0).cast::<u8>();
    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_MARKER_LBA,
        buffer,
        "ahci_audit_rollback_target_inspection_label_buffer_phys_missing",
        "ahci_audit_rollback_target_inspection_label_read_timeout",
        "ahci_audit_rollback_target_inspection_label_read_task_file_error",
        "ahci_audit_rollback_target_inspection_label_read_incomplete",
    ) {
        let mut evidence = AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            reason,
            expected_sector_image_hash,
        );
        evidence.port_index = port_index;
        return evidence;
    }

    if !audit_rollback_target_marker_matches(buffer) {
        let mut evidence = AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "audit_rollback_target_region_label_missing",
            expected_sector_image_hash,
        );
        evidence.port_index = port_index;
        return evidence;
    }

    let mut evidence = AhciAuditRollbackTargetSectorInspectionEvidence {
        port_index,
        label_found: true,
        status: "label_found",
        reason: "audit_rollback_target_region_label_found",
        ..AhciAuditRollbackTargetSectorInspectionEvidence::missing(
            "audit_rollback_target_region_label_found",
            expected_sector_image_hash,
        )
    };

    let identity = match issue_identify(controller, base, port_offset) {
        Ok(identity) => identity,
        Err(reason) => {
            evidence.status = "missing";
            evidence.reason = reason;
            return evidence;
        }
    };

    let sector_count = identity_sector_count(identity);
    evidence.region_within_device_bounds = identity.available
        && identity.logical_sector_size_bytes == SECTOR_BYTES as u32
        && AUDIT_ROLLBACK_TARGET_REGION_START_LBA < sector_count
        && AUDIT_ROLLBACK_TARGET_REGION_LBA_COUNT
            <= sector_count.saturating_sub(AUDIT_ROLLBACK_TARGET_REGION_START_LBA);
    evidence.boot_port_overlap = port_index == boot_port;
    evidence.scratch_region_overlap = scratch.available
        && scratch.port_index == port_index
        && ranges_overlap(
            AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
            AUDIT_ROLLBACK_TARGET_REGION_LBA_COUNT,
            scratch.region_start_lba,
            scratch.region_lba_count,
        );
    if !evidence.region_within_device_bounds {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_region_bounds_invalid";
        return evidence;
    }

    ptr::write_bytes(buffer, 0, SECTOR_BYTES);
    evidence.read_attempted = true;
    if let Err(reason) = issue_read_sector_into(
        controller,
        base,
        port_offset,
        AUDIT_ROLLBACK_TARGET_REGION_START_LBA,
        buffer,
        "ahci_audit_rollback_target_inspection_buffer_phys_missing",
        "ahci_audit_rollback_target_inspection_read_timeout",
        "ahci_audit_rollback_target_inspection_read_task_file_error",
        "ahci_audit_rollback_target_inspection_read_incomplete",
    ) {
        evidence.status = "missing";
        evidence.reason = reason;
        return evidence;
    }

    evidence.read_completed = true;
    evidence.metadata_lba_overlap =
        AUDIT_ROLLBACK_TARGET_REGION_START_LBA == 0 || sector_has_gpt_header(buffer);
    evidence.no_boot_or_partition_metadata_overlap = !evidence.boot_port_overlap
        && !evidence.metadata_lba_overlap
        && !evidence.scratch_region_overlap;
    if !evidence.no_boot_or_partition_metadata_overlap {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_region_overlap";
        return evidence;
    }

    evidence.first16 = first16_from_buffer(buffer);
    evidence.sector_image_hash = sha256_sector_buffer(buffer);
    evidence.read_matches_expected_image = evidence.sector_image_hash == expected_sector_image_hash;
    evidence.offsets_within_sector =
        sector_range_within(audit_record_offset, audit_record_byte_length)
            && sector_range_within(
                rollback_transaction_offset,
                rollback_transaction_byte_length,
            )
            && sector_range_within(padding_offset, padding_byte_length);
    if !evidence.offsets_within_sector {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_sector_inspection_offsets_invalid";
        return evidence;
    }

    evidence.audit_record_image_hash =
        sha256_sector_buffer_range(buffer, audit_record_offset, audit_record_byte_length);
    evidence.rollback_transaction_image_hash = sha256_sector_buffer_range(
        buffer,
        rollback_transaction_offset,
        rollback_transaction_byte_length,
    );
    evidence.padding_zeroed = sector_range_zeroed(buffer, padding_offset, padding_byte_length);
    if !evidence.padding_zeroed {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_sector_padding_not_zero";
        return evidence;
    }
    if evidence.read_matches_expected_image {
        evidence.available = true;
        evidence.status = "available";
        evidence.reason = "audit_rollback_target_sector_image_inspected";
    } else {
        evidence.status = "missing";
        evidence.reason = "audit_rollback_target_sector_image_inspection_mismatch";
    }
    evidence
}

unsafe fn issue_dma_read_command(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    command: u8,
    lba: u64,
    data_buffer: *mut u8,
    data_len: usize,
    data_phys_reason: &'static str,
    timeout_reason: &'static str,
    task_file_reason: &'static str,
    incomplete_reason: &'static str,
) -> Result<(), &'static str> {
    issue_dma_command(
        controller,
        base,
        port_offset,
        command,
        lba,
        false,
        data_buffer,
        data_len,
        data_phys_reason,
        timeout_reason,
        task_file_reason,
        incomplete_reason,
    )
}

unsafe fn issue_dma_command(
    controller: PciMassStorageController,
    base: *mut u8,
    port_offset: usize,
    command: u8,
    lba: u64,
    is_write: bool,
    data_buffer: *mut u8,
    data_len: usize,
    data_phys_reason: &'static str,
    timeout_reason: &'static str,
    task_file_reason: &'static str,
    incomplete_reason: &'static str,
) -> Result<(), &'static str> {
    pci::enable_bus_master(controller.address);

    let ghc = read32(base, REG_GHC);
    if ghc & GHC_AE == 0 {
        write32(base, REG_GHC, ghc | GHC_AE);
    }

    stop_command_engine(base, port_offset)?;
    clear_command_buffers();

    let command_list = ptr::addr_of_mut!(COMMAND_LIST.0).cast::<u8>();
    let received_fis = ptr::addr_of_mut!(RECEIVED_FIS.0).cast::<u8>();
    let command_table = ptr::addr_of_mut!(COMMAND_TABLE.0).cast::<u8>();

    let command_list_phys = phys_of(command_list, "ahci_command_list_phys_missing")?;
    let received_fis_phys = phys_of(received_fis, "ahci_received_fis_phys_missing")?;
    let command_table_phys = phys_of(command_table, "ahci_command_table_phys_missing")?;
    let data_buffer_phys = phys_of(data_buffer, data_phys_reason)?;

    write32(base, port_offset + PORT_CLB, command_list_phys as u32);
    write32(
        base,
        port_offset + PORT_CLBU,
        (command_list_phys >> 32) as u32,
    );
    write32(base, port_offset + PORT_FB, received_fis_phys as u32);
    write32(
        base,
        port_offset + PORT_FBU,
        (received_fis_phys >> 32) as u32,
    );

    let write_bit = if is_write { 1 << 6 } else { 0 };
    write_mem32(command_list, 0x00, 5 | write_bit | (1 << 16));
    write_mem32(command_list, 0x04, 0);
    write_mem32(command_list, 0x08, command_table_phys as u32);
    write_mem32(command_list, 0x0c, (command_table_phys >> 32) as u32);

    ptr::write_volatile(command_table.add(0), 0x27);
    ptr::write_volatile(command_table.add(1), 1 << 7);
    ptr::write_volatile(command_table.add(2), command);
    if command == READ_DMA_EXT || command == WRITE_DMA_EXT {
        ptr::write_volatile(command_table.add(4), lba as u8);
        ptr::write_volatile(command_table.add(5), (lba >> 8) as u8);
        ptr::write_volatile(command_table.add(6), (lba >> 16) as u8);
        ptr::write_volatile(command_table.add(7), 1 << 6);
        ptr::write_volatile(command_table.add(8), (lba >> 24) as u8);
        ptr::write_volatile(command_table.add(9), (lba >> 32) as u8);
        ptr::write_volatile(command_table.add(10), (lba >> 40) as u8);
        ptr::write_volatile(command_table.add(12), 1);
    }
    let prdt = command_table.add(0x80);
    write_mem32(prdt, 0x00, data_buffer_phys as u32);
    write_mem32(prdt, 0x04, (data_buffer_phys >> 32) as u32);
    write_mem32(prdt, 0x08, 0);
    write_mem32(prdt, 0x0c, (data_len as u32) - 1);

    write32(base, port_offset + PORT_IS, u32::MAX);
    write32(base, port_offset + PORT_SERR, u32::MAX);

    wait_until("ahci_port_task_file_busy", || {
        read32(base, port_offset + PORT_TFD) & (PORT_TFD_BSY | PORT_TFD_DRQ) == 0
    })?;

    start_command_engine(base, port_offset);
    atomic::compiler_fence(atomic::Ordering::SeqCst);
    write32(base, port_offset + PORT_CI, 1);

    let completion = wait_until(timeout_reason, || {
        let is = read32(base, port_offset + PORT_IS);
        if is & PORT_IS_TFES != 0 {
            return true;
        }
        read32(base, port_offset + PORT_CI) & 1 == 0
    });
    atomic::compiler_fence(atomic::Ordering::SeqCst);
    let is = read32(base, port_offset + PORT_IS);
    let ci = read32(base, port_offset + PORT_CI);
    let _ = stop_command_engine(base, port_offset);

    completion?;
    if is & PORT_IS_TFES != 0 {
        return Err(task_file_reason);
    }
    if ci & 1 != 0 {
        return Err(incomplete_reason);
    }

    Ok(())
}

unsafe fn stop_command_engine(base: *mut u8, port_offset: usize) -> Result<(), &'static str> {
    let cmd = read32(base, port_offset + PORT_CMD) & !(PORT_CMD_ST | PORT_CMD_FRE);
    write32(base, port_offset + PORT_CMD, cmd);
    wait_until("ahci_command_engine_stop_timeout", || {
        read32(base, port_offset + PORT_CMD) & (PORT_CMD_CR | PORT_CMD_FR) == 0
    })
}

unsafe fn start_command_engine(base: *mut u8, port_offset: usize) {
    let cmd = read32(base, port_offset + PORT_CMD) | PORT_CMD_FRE | PORT_CMD_ST;
    write32(base, port_offset + PORT_CMD, cmd);
}

unsafe fn clear_command_buffers() {
    ptr::write_bytes(ptr::addr_of_mut!(COMMAND_LIST.0).cast::<u8>(), 0, 1024);
    ptr::write_bytes(ptr::addr_of_mut!(RECEIVED_FIS.0).cast::<u8>(), 0, 256);
    ptr::write_bytes(ptr::addr_of_mut!(COMMAND_TABLE.0).cast::<u8>(), 0, 256);
}

unsafe fn parse_identify(buffer: *const u8) -> AhciBlockDeviceIdentity {
    let mut identity = AhciBlockDeviceIdentity {
        available: true,
        logical_sector_size_bytes: 512,
        lba28_sector_count: read_word(buffer, 60) as u32 | ((read_word(buffer, 61) as u32) << 16),
        lba48_sector_count: read_word(buffer, 100) as u64
            | ((read_word(buffer, 101) as u64) << 16)
            | ((read_word(buffer, 102) as u64) << 32)
            | ((read_word(buffer, 103) as u64) << 48),
        serial: [0; 20],
        firmware: [0; 8],
        model: [0; 40],
    };
    copy_ata_string(buffer, 10, &mut identity.serial);
    copy_ata_string(buffer, 23, &mut identity.firmware);
    copy_ata_string(buffer, 27, &mut identity.model);
    identity
}

unsafe fn parse_sector0(buffer: *const u8) -> AhciSectorReadEvidence {
    let first16 = first16_from_buffer(buffer);
    let low = ptr::read_volatile(buffer.add(510)) as u16;
    let high = ptr::read_volatile(buffer.add(511)) as u16;
    AhciSectorReadEvidence {
        available: true,
        lba: 0,
        byte_count: SECTOR_BYTES as u32,
        mbr_signature: low | (high << 8),
        first16,
    }
}

unsafe fn first16_from_buffer(buffer: *const u8) -> [u8; 16] {
    let mut first16 = [0u8; 16];
    let mut idx = 0usize;
    while idx < first16.len() {
        first16[idx] = ptr::read_volatile(buffer.add(idx));
        idx += 1;
    }
    first16
}

unsafe fn copy_sector_image_to_buffer(buffer: *mut u8, planned_image: &[u8; SECTOR_BYTES]) {
    let mut idx = 0usize;
    while idx < SECTOR_BYTES {
        ptr::write_volatile(buffer.add(idx), planned_image[idx]);
        idx += 1;
    }
}

unsafe fn sha256_sector_buffer(buffer: *const u8) -> [u8; 32] {
    let bytes = core::slice::from_raw_parts(buffer, SECTOR_BYTES);
    let mut hash = Sha256::new();
    hash.update(bytes);
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn sector_range_within(offset: usize, byte_length: usize) -> bool {
    offset <= SECTOR_BYTES && byte_length <= SECTOR_BYTES - offset
}

unsafe fn sha256_sector_buffer_range(
    buffer: *const u8,
    offset: usize,
    byte_length: usize,
) -> [u8; 32] {
    let bytes = core::slice::from_raw_parts(buffer.add(offset), byte_length);
    let mut hash = Sha256::new();
    hash.update(bytes);
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

unsafe fn sector_range_zeroed(buffer: *const u8, offset: usize, byte_length: usize) -> bool {
    let mut idx = 0usize;
    while idx < byte_length {
        if ptr::read_volatile(buffer.add(offset + idx)) != 0 {
            return false;
        }
        idx += 1;
    }
    true
}

unsafe fn scratch_marker_matches(buffer: *const u8) -> bool {
    let mut idx = 0usize;
    while idx < SCRATCH_MARKER.len() {
        if ptr::read_volatile(buffer.add(idx)) != SCRATCH_MARKER[idx] {
            return false;
        }
        idx += 1;
    }
    true
}

unsafe fn audit_rollback_target_marker_matches(buffer: *const u8) -> bool {
    let mut idx = 0usize;
    while idx < AUDIT_ROLLBACK_TARGET_MARKER.len() {
        if ptr::read_volatile(buffer.add(idx)) != AUDIT_ROLLBACK_TARGET_MARKER[idx] {
            return false;
        }
        idx += 1;
    }
    true
}

unsafe fn sector_has_gpt_header(buffer: *const u8) -> bool {
    let signature = b"EFI PART";
    let mut idx = 0usize;
    while idx < signature.len() {
        if ptr::read_volatile(buffer.add(idx)) != signature[idx] {
            return false;
        }
        idx += 1;
    }
    true
}

fn ranges_overlap(left_start: u64, left_count: u64, right_start: u64, right_count: u64) -> bool {
    let left_end = left_start.saturating_add(left_count);
    let right_end = right_start.saturating_add(right_count);
    left_start < right_end && right_start < left_end
}

fn identity_sector_count(identity: AhciBlockDeviceIdentity) -> u64 {
    if identity.lba48_sector_count > 0 {
        identity.lba48_sector_count
    } else {
        identity.lba28_sector_count as u64
    }
}

unsafe fn fill_scratch_pattern(buffer: *mut u8) {
    let mut idx = 0usize;
    while idx < SECTOR_BYTES {
        ptr::write_volatile(buffer.add(idx), scratch_pattern_byte(idx));
        idx += 1;
    }
    let prefix = b"RAIOS_WRITEBACK!";
    idx = 0;
    while idx < prefix.len() {
        ptr::write_volatile(buffer.add(idx), prefix[idx]);
        idx += 1;
    }
}

unsafe fn scratch_pattern_matches(buffer: *const u8) -> bool {
    let prefix = b"RAIOS_WRITEBACK!";
    let mut idx = 0usize;
    while idx < SECTOR_BYTES {
        let expected = if idx < prefix.len() {
            prefix[idx]
        } else {
            scratch_pattern_byte(idx)
        };
        if ptr::read_volatile(buffer.add(idx)) != expected {
            return false;
        }
        idx += 1;
    }
    true
}

fn scratch_pattern_byte(idx: usize) -> u8 {
    (idx as u8).wrapping_mul(31).wrapping_add(0xa5)
}

unsafe fn parse_partition_inventory(
    buffer: *const u8,
    sector0: AhciSectorReadEvidence,
) -> AhciPartitionInventoryEvidence {
    if !sector0.available || sector0.mbr_signature != MBR_SIGNATURE {
        return AhciPartitionInventoryEvidence::missing();
    }

    let mut entries = [AhciPartitionEntryEvidence::missing(); MBR_PARTITION_ENTRY_COUNT];
    let mut entry_count = 0u8;
    let mut bootable_entry_count = 0u8;
    let mut idx = 0usize;
    while idx < MBR_PARTITION_ENTRY_COUNT {
        let offset = MBR_PARTITION_TABLE_OFFSET + idx * MBR_PARTITION_ENTRY_BYTES;
        let partition_type = ptr::read_volatile(buffer.add(offset + 4));
        let sector_count = read_le_u32(buffer.add(offset + 12));
        let entry = AhciPartitionEntryEvidence {
            available: partition_type != 0 && sector_count != 0,
            boot_indicator: ptr::read_volatile(buffer.add(offset)),
            partition_type,
            first_lba: read_le_u32(buffer.add(offset + 8)),
            sector_count,
        };
        if entry.available {
            entry_count = entry_count.saturating_add(1);
            if entry.boot_indicator == 0x80 {
                bootable_entry_count = bootable_entry_count.saturating_add(1);
            }
        }
        entries[idx] = entry;
        idx += 1;
    }

    AhciPartitionInventoryEvidence {
        available: true,
        source_lba: sector0.lba,
        scheme: if entry_count == 0 { "mbr_empty" } else { "mbr" },
        mbr_signature_valid: true,
        entry_count,
        bootable_entry_count,
        entries,
    }
}

unsafe fn copy_ata_string(buffer: *const u8, start_word: usize, out: &mut [u8]) {
    let mut idx = 0usize;
    while idx < out.len() / 2 {
        let word = buffer.add((start_word + idx) * 2);
        out[idx * 2] = ptr::read_volatile(word.add(1));
        out[idx * 2 + 1] = ptr::read_volatile(word);
        idx += 1;
    }
}

unsafe fn read_le_u32(buffer: *const u8) -> u32 {
    ptr::read_volatile(buffer) as u32
        | ((ptr::read_volatile(buffer.add(1)) as u32) << 8)
        | ((ptr::read_volatile(buffer.add(2)) as u32) << 16)
        | ((ptr::read_volatile(buffer.add(3)) as u32) << 24)
}

unsafe fn read_word(buffer: *const u8, word: usize) -> u16 {
    let low = ptr::read_volatile(buffer.add(word * 2)) as u16;
    let high = ptr::read_volatile(buffer.add(word * 2 + 1)) as u16;
    low | (high << 8)
}

fn first_port(mask: u32) -> Option<u8> {
    let mut port = 0u8;
    while port < 32 {
        if (mask & (1u32 << port)) != 0 {
            return Some(port);
        }
        port += 1;
    }
    None
}

fn method_is_pre_dma(status: &str) -> bool {
    matches!(
        status,
        "ahci_command_engine_stop_timeout"
            | "ahci_command_list_phys_missing"
            | "ahci_received_fis_phys_missing"
            | "ahci_command_table_phys_missing"
            | "ahci_identify_buffer_phys_missing"
            | "ahci_sector0_buffer_phys_missing"
            | "ahci_port_task_file_busy"
    )
}

fn phys_of<T>(ptr: *const T, reason: &'static str) -> Result<u64, &'static str> {
    memory::virt_to_phys(ptr).ok_or(reason)
}

fn wait_until<F>(reason: &'static str, mut predicate: F) -> Result<(), &'static str>
where
    F: FnMut() -> bool,
{
    let mut idx = 0usize;
    while idx < WAIT_ITERS {
        if predicate() {
            return Ok(());
        }
        core::hint::spin_loop();
        idx += 1;
    }
    Err(reason)
}

unsafe fn read32(base: *mut u8, offset: usize) -> u32 {
    ptr::read_volatile(base.add(offset).cast::<u32>())
}

unsafe fn write32(base: *mut u8, offset: usize, value: u32) {
    ptr::write_volatile(base.add(offset).cast::<u32>(), value);
}

unsafe fn write_mem32(base: *mut u8, offset: usize, value: u32) {
    ptr::write_volatile(base.add(offset).cast::<u32>(), value);
}
