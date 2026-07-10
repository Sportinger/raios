//! C1's one disposable QEMU structured-store proof path.
//!
//! It never scans for a disk. The only admitted target is Q35 00:1f.2 port 4
//! with the frozen QEMU fixture GUIDs; all normal and physical media stay out
//! of this path before any write can occur.

use alloc::vec;

use raios_core::{
    gpt_layout::GPT_ENTRY_ARRAY_BYTES,
    sha256_bytes,
    structured_store::{
        plan_transaction, RecordKey, RecordOperation, StoreGeometry, StoreIdentity,
        TransactionRequest, STORE_BLOCK_LEN,
    },
    structured_store_partition::{
        validate_approved_structured_store_partition, ApprovedStructuredStorePartition,
        STRUCTURED_STORE_LABEL, STRUCTURED_STORE_TYPE_GUID_LE,
    },
};
use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{
    ahci::{AhciBlockDeviceIdentity, AhciExplicitAtaPort, AhciSectorBuffer, SECTOR_BYTES},
    pci::{self, PciAddress},
    serial,
    structured_store::{
        append_with_readback, format_empty_disposable_test_media, open_and_replay, PortDenied,
        ValidatedRegionIdentity, ValidatedStoreRegionPort,
    },
};

const QEMU_AHCI_BDF: PciAddress = PciAddress::new(0, 31, 2);
const QEMU_AHCI_PORT: u8 = 4;
const QEMU_DISK_GUID_LE: [u8; 16] = [
    0x7a, 0xda, 0xed, 0x5e, 0xde, 0xc0, 0x55, 0x4a, 0x9a, 0x15, 0x00, 0x00, 0x00, 0x00, 0x13, 0x00,
];
const QEMU_PARTITION_GUID_LE: [u8; 16] = [
    0x7a, 0xda, 0xed, 0x5e, 0xde, 0xc0, 0x55, 0x4a, 0x9a, 0x15, 0x00, 0x00, 0x00, 0x00, 0x13, 0x01,
];
const STORE: StoreIdentity = StoreIdentity {
    store_uuid: *b"c1-test-store-v1",
    generation: 1,
};
const PROOF_KEY: RecordKey = RecordKey {
    namespace: *b"c1-structured-v1",
    record_id: *b"c1-append-proof!",
};
const PROOF_PAYLOAD: &[u8] = b"C1 durable structured-store proof";

// DMA must use a kernel-static, aligned buffer; stack memory is not guaranteed
// to have a physical mapping in this higher-half kernel.
static C1_DMA_BUFFER: Mutex<AhciSectorBuffer> = Mutex::new(AhciSectorBuffer::zeroed());

#[derive(Debug)]
enum C1Error {
    Device(&'static str),
    Port(PortDenied<&'static str>),
    Core(raios_core::structured_store::StoreDenied),
    Gpt(raios_core::structured_store_partition::StructuredStorePartitionDenied),
    InvalidDeviceGeometry,
    Bounds,
}

impl From<PortDenied<&'static str>> for C1Error {
    fn from(value: PortDenied<&'static str>) -> Self {
        Self::Port(value)
    }
}

/// Runs only when the harness has attached the frozen test disk. An absent
/// controller or port is ordinary normal-boot state and emits nothing.
pub(crate) fn run_disposable_qemu_boot_probe() {
    let Ok(controller) = pci::exact_ahci_controller(QEMU_AHCI_BDF) else {
        return;
    };
    let Ok(inner) = AhciExplicitAtaPort::open(controller, QEMU_AHCI_PORT) else {
        return;
    };

    let mut port = match C1Port::open(inner) {
        Ok(port) => port,
        Err(error) => {
            serial::write_fmt(format_args!("C1_STRUCTURED_STORE_DENIED: {:?}\r\n", error));
            return;
        }
    };
    serial::write_line("C1_STRUCTURED_STORE_FIXTURE_ACCEPTED");

    if let Err(error) = run_fixture(&mut port) {
        serial::write_fmt(format_args!("C1_STRUCTURED_STORE_DENIED: {:?}\r\n", error));
    }
}

fn run_fixture(port: &mut C1Port) -> Result<(), C1Error> {
    let identity = port.identity;
    let snapshot_len = usize::try_from(identity.geometry.log_byte_len().map_err(C1Error::Core)?)
        .map_err(|_| C1Error::Bounds)?;
    let mut snapshot = vec![0u8; snapshot_len];

    let replay = match open_and_replay(port, identity, &mut snapshot) {
        Ok(replay) => replay,
        Err(PortDenied::Core(raios_core::structured_store::StoreDenied::NoValidSuperblock)) => {
            format_empty_disposable_test_media(port, identity)?;
            let replay = open_and_replay(port, identity, &mut snapshot)?;
            serial::write_line("C1_STRUCTURED_STORE_FORMAT_OPEN_OK");
            replay
        }
        Err(error) => return Err(error.into()),
    };

    if replay.record(PROOF_KEY).map_err(C1Error::Core)?.is_some() {
        serial::write_line("C1_STRUCTURED_STORE_REBOOT_REPLAY_OK");
        return Ok(());
    }

    let plan = plan_transaction(
        &replay,
        TransactionRequest {
            identity: STORE,
            key: PROOF_KEY,
            operation: RecordOperation::Put,
            expected_committed_version: None,
        },
        PROOF_PAYLOAD,
        identity.geometry.log_byte_len().map_err(C1Error::Core)?,
    )
    .map_err(C1Error::Core)?;
    let replay = append_with_readback(port, identity, &plan, &mut snapshot)?;
    if replay.record(PROOF_KEY).map_err(C1Error::Core)?.is_none() {
        return Err(C1Error::Bounds);
    }
    serial::write_line("C1_STRUCTURED_STORE_APPEND_FLUSH_READBACK_OK");
    Ok(())
}

struct C1Port {
    inner: AhciExplicitAtaPort,
    identity: ValidatedRegionIdentity,
}

impl C1Port {
    fn open(inner: AhciExplicitAtaPort) -> Result<Self, C1Error> {
        let identity = establish_identity(inner)?;
        Ok(Self { inner, identity })
    }

    fn lba(&self, relative: u64) -> Result<u64, &'static str> {
        if relative >= self.identity.geometry.partition_lba_count {
            return Err("c1_structured_store_relative_lba_out_of_bounds");
        }
        self.identity
            .geometry
            .partition_start_lba
            .checked_add(relative)
            .ok_or("c1_structured_store_absolute_lba_overflow")
    }

    fn read_block(
        &mut self,
        lba: u64,
        out: &mut [u8; STORE_BLOCK_LEN],
    ) -> Result<(), &'static str> {
        let mut sector = C1_DMA_BUFFER.lock();
        unsafe { self.inner.read_sector(lba, &mut sector)? };
        out.copy_from_slice(&sector.0);
        Ok(())
    }

    fn write_block(&mut self, lba: u64, block: &[u8; STORE_BLOCK_LEN]) -> Result<(), &'static str> {
        let mut sector = C1_DMA_BUFFER.lock();
        sector.0.copy_from_slice(block);
        unsafe { self.inner.write_sector(lba, &mut sector) }
    }
}

impl ValidatedStoreRegionPort for C1Port {
    type Error = &'static str;

    fn revalidate_identity(&mut self) -> Result<ValidatedRegionIdentity, Self::Error> {
        establish_identity(self.inner).map_err(c1_error_reason)
    }

    fn read_superblock_copy(
        &mut self,
        copy_index: u8,
        out: &mut [u8; STORE_BLOCK_LEN],
    ) -> Result<(), Self::Error> {
        if copy_index > 1 {
            return Err("c1_structured_store_superblock_copy_invalid");
        }
        self.read_block(self.lba(copy_index as u64)?, out)
    }

    fn write_superblock_copy(
        &mut self,
        copy_index: u8,
        block: &[u8; STORE_BLOCK_LEN],
    ) -> Result<(), Self::Error> {
        if copy_index > 1 {
            return Err("c1_structured_store_superblock_copy_invalid");
        }
        self.write_block(self.lba(copy_index as u64)?, block)
    }

    fn read_log_block(
        &mut self,
        relative_block: u64,
        out: &mut [u8; STORE_BLOCK_LEN],
    ) -> Result<(), Self::Error> {
        if relative_block >= self.identity.geometry.log_lba_count {
            return Err("c1_structured_store_log_read_out_of_bounds");
        }
        let relative = self
            .identity
            .geometry
            .log_start_lba
            .checked_add(relative_block)
            .ok_or("c1_structured_store_log_read_overflow")?;
        self.read_block(self.lba(relative)?, out)
    }

    fn write_log_block(
        &mut self,
        relative_block: u64,
        block: &[u8; STORE_BLOCK_LEN],
    ) -> Result<(), Self::Error> {
        if relative_block >= self.identity.geometry.log_lba_count {
            return Err("c1_structured_store_log_write_out_of_bounds");
        }
        let relative = self
            .identity
            .geometry
            .log_start_lba
            .checked_add(relative_block)
            .ok_or("c1_structured_store_log_write_overflow")?;
        self.write_block(self.lba(relative)?, block)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        unsafe { self.inner.flush_cache_ext() }
    }
}

fn establish_identity(inner: AhciExplicitAtaPort) -> Result<ValidatedRegionIdentity, C1Error> {
    if inner.port_index() != QEMU_AHCI_PORT {
        return Err(C1Error::Bounds);
    }
    let device = unsafe { inner.identify().map_err(C1Error::Device)? };
    let total_lbas = device_sector_count(device).ok_or(C1Error::InvalidDeviceGeometry)?;
    let mut mbr = [0u8; SECTOR_BYTES];
    read_sector_copy(inner, 0, &mut mbr)?;
    let mut primary_header = [0u8; SECTOR_BYTES];
    read_sector_copy(inner, 1, &mut primary_header)?;
    let mut primary_entries = [0u8; GPT_ENTRY_ARRAY_BYTES];
    read_entry_array(inner, 2, &mut primary_entries)?;
    let backup_header_lba = total_lbas
        .checked_sub(1)
        .ok_or(C1Error::InvalidDeviceGeometry)?;
    let backup_entries_lba = backup_header_lba
        .checked_sub((GPT_ENTRY_ARRAY_BYTES / SECTOR_BYTES) as u64)
        .ok_or(C1Error::InvalidDeviceGeometry)?;
    let mut backup_header = [0u8; SECTOR_BYTES];
    read_sector_copy(inner, backup_header_lba, &mut backup_header)?;
    let mut backup_entries = [0u8; GPT_ENTRY_ARRAY_BYTES];
    read_entry_array(inner, backup_entries_lba, &mut backup_entries)?;
    let partition = validate_approved_structured_store_partition(
        &mbr,
        &primary_header,
        &primary_entries,
        &backup_header,
        &backup_entries,
        total_lbas,
        ApprovedStructuredStorePartition {
            gpt_disk_guid: QEMU_DISK_GUID_LE,
            partition_guid: QEMU_PARTITION_GUID_LE,
        },
    )
    .map_err(C1Error::Gpt)?;
    let log_lba_count = partition.lba_count.checked_sub(2).ok_or(C1Error::Bounds)?;
    let geometry = StoreGeometry {
        partition_start_lba: partition.first_lba,
        partition_lba_count: partition.lba_count,
        log_start_lba: 2,
        log_lba_count,
        logical_sector_size: SECTOR_BYTES as u32,
    };
    geometry.validate().map_err(C1Error::Core)?;
    Ok(ValidatedRegionIdentity {
        pci_segment: 0,
        pci_bus: QEMU_AHCI_BDF.bus,
        pci_device: QEMU_AHCI_BDF.device,
        pci_function: QEMU_AHCI_BDF.function,
        controller_port: QEMU_AHCI_PORT,
        device_identity_sha256: device_identity_sha256(device),
        gpt_disk_guid: partition.gpt_disk_guid,
        partition_type_guid: STRUCTURED_STORE_TYPE_GUID_LE,
        partition_guid: partition.partition_guid,
        partition_label_sha256: sha256_bytes(STRUCTURED_STORE_LABEL.as_bytes()),
        store: STORE,
        geometry,
    })
}

fn read_sector_copy(
    inner: AhciExplicitAtaPort,
    lba: u64,
    out: &mut [u8; SECTOR_BYTES],
) -> Result<(), C1Error> {
    let mut sector = C1_DMA_BUFFER.lock();
    unsafe {
        inner
            .read_sector(lba, &mut sector)
            .map_err(C1Error::Device)?
    };
    out.copy_from_slice(&sector.0);
    Ok(())
}

fn read_entry_array(
    inner: AhciExplicitAtaPort,
    first_lba: u64,
    out: &mut [u8; GPT_ENTRY_ARRAY_BYTES],
) -> Result<(), C1Error> {
    for index in 0..(GPT_ENTRY_ARRAY_BYTES / SECTOR_BYTES) {
        let lba = first_lba.checked_add(index as u64).ok_or(C1Error::Bounds)?;
        let mut sector = C1_DMA_BUFFER.lock();
        unsafe {
            inner
                .read_sector(lba, &mut sector)
                .map_err(C1Error::Device)?
        };
        let start = index * SECTOR_BYTES;
        out[start..start + SECTOR_BYTES].copy_from_slice(&sector.0);
    }
    Ok(())
}

fn device_sector_count(device: AhciBlockDeviceIdentity) -> Option<u64> {
    if !device.available || device.logical_sector_size_bytes != SECTOR_BYTES as u32 {
        return None;
    }
    let count = if device.lba48_sector_count != 0 {
        device.lba48_sector_count
    } else {
        device.lba28_sector_count as u64
    };
    (count > 0).then_some(count)
}

fn device_identity_sha256(device: AhciBlockDeviceIdentity) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"raios.structured-store.device.v1");
    hash.update(device.logical_sector_size_bytes.to_le_bytes());
    hash.update(device.lba28_sector_count.to_le_bytes());
    hash.update(device.lba48_sector_count.to_le_bytes());
    hash.update(device.serial);
    hash.update(device.firmware);
    hash.update(device.model);
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn c1_error_reason(error: C1Error) -> &'static str {
    match error {
        C1Error::Device(reason) => reason,
        C1Error::Port(_) => "c1_structured_store_revalidation_denied",
        C1Error::Core(_) => "c1_structured_store_revalidation_core_denied",
        C1Error::Gpt(_) => "c1_structured_store_revalidation_gpt_denied",
        C1Error::InvalidDeviceGeometry => {
            "c1_structured_store_revalidation_device_geometry_invalid"
        }
        C1Error::Bounds => "c1_structured_store_revalidation_bounds_denied",
    }
}
