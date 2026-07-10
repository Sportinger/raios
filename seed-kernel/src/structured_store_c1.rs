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
        plan_transaction, RecordKey, RecordOperation, RecordValue, ReplayTail, StoreGeometry,
        StoreIdentity, TransactionRequest, STORE_BLOCK_LEN,
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
    secret_vault, serial,
    structured_store::{
        append_with_readback, format_empty_disposable_test_media, open_and_replay_with_history,
        PortDenied, ValidatedRegionIdentity, ValidatedReplayWithHistory, ValidatedStoreRegionPort,
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
pub(crate) enum DisposableQemuStoreDenied {
    Device(&'static str),
    Port(PortDenied<&'static str>),
    Core(raios_core::structured_store::StoreDenied),
    Gpt(raios_core::structured_store_partition::StructuredStorePartitionDenied),
    Vault(secret_vault::VaultFacadeDenied),
    InvalidDeviceGeometry,
    Bounds,
}

impl From<PortDenied<&'static str>> for DisposableQemuStoreDenied {
    fn from(value: PortDenied<&'static str>) -> Self {
        Self::Port(value)
    }
}

#[derive(Debug)]
pub(crate) enum RecoveryWrapperStoreDenied {
    FixtureMissing,
    Open(&'static str),
    Port(PortDenied<&'static str>),
    Core(raios_core::structured_store::StoreDenied),
    IdentityChanged,
    VaultHistoryNotEmpty,
    ReadbackMismatch,
    Bounds,
}

#[derive(Debug)]
pub(crate) enum ProviderSecretStoreDenied {
    FixtureMissing,
    Open(&'static str),
    Port(PortDenied<&'static str>),
    Core(raios_core::structured_store::StoreDenied),
    IdentityChanged,
    CurrentVersionChanged,
    ReadbackMismatch,
    Bounds,
}

#[derive(Debug)]
pub(crate) enum WifiSecretStoreDenied {
    FixtureMissing,
    Open(&'static str),
    Port(PortDenied<&'static str>),
    Core(raios_core::structured_store::StoreDenied),
    IdentityChanged,
    CurrentVersionChanged,
    ReadbackMismatch,
    Bounds,
}

impl From<PortDenied<&'static str>> for ProviderSecretStoreDenied {
    fn from(value: PortDenied<&'static str>) -> Self {
        Self::Port(value)
    }
}

impl From<PortDenied<&'static str>> for WifiSecretStoreDenied {
    fn from(value: PortDenied<&'static str>) -> Self {
        Self::Port(value)
    }
}

impl From<PortDenied<&'static str>> for RecoveryWrapperStoreDenied {
    fn from(value: PortDenied<&'static str>) -> Self {
        Self::Port(value)
    }
}

/// Runs only when the harness has attached the frozen test disk. An absent
/// controller or port is ordinary normal-boot state and emits nothing.
pub(crate) fn run_disposable_qemu_boot_probe() {
    let mut port = match open_disposable_qemu_store_port() {
        Ok(Some(port)) => port,
        Ok(None) => return,
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

fn run_fixture(port: &mut DisposableQemuStorePort) -> Result<(), DisposableQemuStoreDenied> {
    let identity = port.identity;
    let snapshot_len = usize::try_from(
        identity
            .geometry
            .log_byte_len()
            .map_err(DisposableQemuStoreDenied::Core)?,
    )
    .map_err(|_| DisposableQemuStoreDenied::Bounds)?;
    let mut snapshot = vec![0u8; snapshot_len];

    let replay = match open_and_replay_with_history(port, identity, &mut snapshot) {
        Ok(replay) => replay,
        Err(PortDenied::Core(raios_core::structured_store::StoreDenied::NoValidSuperblock)) => {
            format_empty_disposable_test_media(port, identity)?;
            let replay = open_and_replay_with_history(port, identity, &mut snapshot)?;
            serial::write_line("C1_STRUCTURED_STORE_FORMAT_OPEN_OK");
            replay
        }
        Err(error) => return Err(error.into()),
    };

    if replay
        .state()
        .record(PROOF_KEY)
        .map_err(DisposableQemuStoreDenied::Core)?
        .is_some()
    {
        bind_vault_runtime_inputs(&replay)?;
        serial::write_line("C1_STRUCTURED_STORE_REBOOT_REPLAY_OK");
        return Ok(());
    }

    let plan = plan_transaction(
        replay.state(),
        TransactionRequest {
            identity: STORE,
            key: PROOF_KEY,
            operation: RecordOperation::Put,
            expected_committed_version: None,
        },
        PROOF_PAYLOAD,
        identity
            .geometry
            .log_byte_len()
            .map_err(DisposableQemuStoreDenied::Core)?,
    )
    .map_err(DisposableQemuStoreDenied::Core)?;
    let _ = append_with_readback(port, identity, &plan, &mut snapshot)?;
    let replay = open_and_replay_with_history(port, identity, &mut snapshot)?;
    if replay
        .state()
        .record(PROOF_KEY)
        .map_err(DisposableQemuStoreDenied::Core)?
        .is_none()
    {
        return Err(DisposableQemuStoreDenied::Bounds);
    }
    bind_vault_runtime_inputs(&replay)?;
    serial::write_line("C1_STRUCTURED_STORE_APPEND_FLUSH_READBACK_OK");
    Ok(())
}

fn bind_vault_runtime_inputs(
    replay: &ValidatedReplayWithHistory,
) -> Result<(), DisposableQemuStoreDenied> {
    secret_vault::load_verified_replay(replay).map_err(DisposableQemuStoreDenied::Vault)?;
    serial::write_line("C1_VAULT_COMPLETE_REPLAY_BOUND");
    secret_vault::bind_verified_core_policy().map_err(DisposableQemuStoreDenied::Vault)?;
    serial::write_line("C1_VAULT_CORE_POLICY_BOUND");
    if let Some(generation) = secret_vault::recovery_wrapper_generation() {
        serial::write_fmt(format_args!(
            "C1_VAULT_RECOVERY_WRAPPER_REPLAYED generation={}\r\n",
            generation
        ));
        serial::write_line("VAULT_RECOVERY_UNLOCK_READY");
    }
    if let secret_vault::VaultSecretStatus::Available { record_version, .. } =
        secret_vault::provider_status()
    {
        serial::write_fmt(format_args!(
            "C1_VAULT_PROVIDER_REPLAYED version={}\r\n",
            record_version
        ));
    }
    if let secret_vault::VaultSecretStatus::Available { record_version, .. } =
        secret_vault::wifi_status()
    {
        serial::write_fmt(format_args!(
            "C1_VAULT_WIFI_REPLAYED version={}\r\n",
            record_version
        ));
    }
    Ok(())
}

/// Appends only the initial recovery-wrapper record to the already admitted
/// disposable QEMU Vault fixture. The port is reopened after physical input,
/// and its complete prior identity is compared before any media operation.
pub(crate) fn commit_initial_recovery_wrapper(
    expected: ValidatedRegionIdentity,
    payload: &[u8; secret_vault::RECOVERY_WRAPPER_PAYLOAD_LEN],
) -> Result<ValidatedReplayWithHistory, RecoveryWrapperStoreDenied> {
    let mut port = open_disposable_qemu_store_port()
        .map_err(|error| RecoveryWrapperStoreDenied::Open(c1_error_reason(error)))?
        .ok_or(RecoveryWrapperStoreDenied::FixtureMissing)?;
    if port.identity() != expected {
        return Err(RecoveryWrapperStoreDenied::IdentityChanged);
    }

    let snapshot_len = usize::try_from(
        expected
            .geometry
            .log_byte_len()
            .map_err(RecoveryWrapperStoreDenied::Core)?,
    )
    .map_err(|_| RecoveryWrapperStoreDenied::Bounds)?;
    let mut snapshot = vec![0u8; snapshot_len];
    let replay = open_and_replay_with_history(&mut port, expected, &mut snapshot)?;
    if replay.state().tail != ReplayTail::ZeroFilled
        || replay
            .committed_history()
            .map_err(RecoveryWrapperStoreDenied::Core)?
            .iter()
            .any(|record| secret_vault::is_vault_namespace(record.key.namespace))
    {
        return Err(RecoveryWrapperStoreDenied::VaultHistoryNotEmpty);
    }

    let key = secret_vault::recovery_wrapper_record_key();
    if replay
        .state()
        .record(key)
        .map_err(RecoveryWrapperStoreDenied::Core)?
        .is_some()
    {
        return Err(RecoveryWrapperStoreDenied::VaultHistoryNotEmpty);
    }
    let plan = plan_transaction(
        replay.state(),
        TransactionRequest {
            identity: expected.store,
            key,
            operation: RecordOperation::Put,
            expected_committed_version: None,
        },
        payload,
        expected
            .geometry
            .log_byte_len()
            .map_err(RecoveryWrapperStoreDenied::Core)?,
    )
    .map_err(RecoveryWrapperStoreDenied::Core)?;
    let _ = append_with_readback(&mut port, expected, &plan, &mut snapshot)?;
    let replay = open_and_replay_with_history(&mut port, expected, &mut snapshot)?;
    let record = replay
        .state()
        .record(key)
        .map_err(RecoveryWrapperStoreDenied::Core)?
        .ok_or(RecoveryWrapperStoreDenied::ReadbackMismatch)?;
    if !matches!(&record.value, RecordValue::Present(bytes) if bytes.as_slice() == payload) {
        return Err(RecoveryWrapperStoreDenied::ReadbackMismatch);
    }
    serial::write_line("C1_VAULT_RECOVERY_WRAPPER_COMMITTED generation=1 readback=verified");
    Ok(replay)
}

/// Appends exactly one encrypted OpenAI credential to the already admitted
/// disposable QEMU fixture. This is test-media-only composition: it has no
/// device discovery, physical-media fallback, or generic record API.
pub(crate) fn commit_provider_secret(
    expected: ValidatedRegionIdentity,
    expected_committed_version: Option<u64>,
    proposed_version: u64,
    payload: &[u8; secret_vault::PROVIDER_SECRET_PAYLOAD_LEN],
) -> Result<ValidatedReplayWithHistory, ProviderSecretStoreDenied> {
    let mut port = open_disposable_qemu_store_port()
        .map_err(|error| ProviderSecretStoreDenied::Open(c1_error_reason(error)))?
        .ok_or(ProviderSecretStoreDenied::FixtureMissing)?;
    if port.identity() != expected {
        return Err(ProviderSecretStoreDenied::IdentityChanged);
    }

    let snapshot_len = usize::try_from(
        expected
            .geometry
            .log_byte_len()
            .map_err(ProviderSecretStoreDenied::Core)?,
    )
    .map_err(|_| ProviderSecretStoreDenied::Bounds)?;
    let mut snapshot = vec![0u8; snapshot_len];
    let replay = open_and_replay_with_history(&mut port, expected, &mut snapshot)?;
    let key = secret_vault::provider_record_key();
    let current_version = replay
        .state()
        .record(key)
        .map_err(ProviderSecretStoreDenied::Core)?
        .map(|record| record.record_version);
    if current_version != expected_committed_version
        || proposed_version
            != expected_committed_version
                .and_then(|version| version.checked_add(1))
                .unwrap_or(1)
    {
        return Err(ProviderSecretStoreDenied::CurrentVersionChanged);
    }

    let plan = plan_transaction(
        replay.state(),
        TransactionRequest {
            identity: expected.store,
            key,
            operation: RecordOperation::Put,
            expected_committed_version,
        },
        payload,
        expected
            .geometry
            .log_byte_len()
            .map_err(ProviderSecretStoreDenied::Core)?,
    )
    .map_err(ProviderSecretStoreDenied::Core)?;
    let _ = append_with_readback(&mut port, expected, &plan, &mut snapshot)?;
    let replay = open_and_replay_with_history(&mut port, expected, &mut snapshot)?;
    let record = replay
        .state()
        .record(key)
        .map_err(ProviderSecretStoreDenied::Core)?
        .ok_or(ProviderSecretStoreDenied::ReadbackMismatch)?;
    if record.record_version != proposed_version
        || !matches!(&record.value, RecordValue::Present(bytes) if bytes.as_slice() == payload)
    {
        return Err(ProviderSecretStoreDenied::ReadbackMismatch);
    }
    serial::write_fmt(format_args!(
        "C1_VAULT_PROVIDER_COMMITTED version={} readback=verified\r\n",
        proposed_version
    ));
    Ok(replay)
}

/// Appends exactly one encrypted bound-BSS credential to the already admitted
/// disposable QEMU fixture. It has no device discovery, physical-media
/// fallback, or generic record API.
pub(crate) fn commit_wifi_secret(
    expected: ValidatedRegionIdentity,
    expected_committed_version: Option<u64>,
    proposed_version: u64,
    payload: &[u8; secret_vault::WIFI_SECRET_PAYLOAD_LEN],
) -> Result<ValidatedReplayWithHistory, WifiSecretStoreDenied> {
    let mut port = open_disposable_qemu_store_port()
        .map_err(|error| WifiSecretStoreDenied::Open(c1_error_reason(error)))?
        .ok_or(WifiSecretStoreDenied::FixtureMissing)?;
    if port.identity() != expected {
        return Err(WifiSecretStoreDenied::IdentityChanged);
    }

    let snapshot_len = usize::try_from(
        expected
            .geometry
            .log_byte_len()
            .map_err(WifiSecretStoreDenied::Core)?,
    )
    .map_err(|_| WifiSecretStoreDenied::Bounds)?;
    let mut snapshot = vec![0u8; snapshot_len];
    let replay = open_and_replay_with_history(&mut port, expected, &mut snapshot)?;
    let key = secret_vault::wifi_record_key();
    let current_version = replay
        .state()
        .record(key)
        .map_err(WifiSecretStoreDenied::Core)?
        .map(|record| record.record_version);
    if current_version != expected_committed_version
        || proposed_version
            != expected_committed_version
                .and_then(|version| version.checked_add(1))
                .unwrap_or(1)
    {
        return Err(WifiSecretStoreDenied::CurrentVersionChanged);
    }

    let plan = plan_transaction(
        replay.state(),
        TransactionRequest {
            identity: expected.store,
            key,
            operation: RecordOperation::Put,
            expected_committed_version,
        },
        payload,
        expected
            .geometry
            .log_byte_len()
            .map_err(WifiSecretStoreDenied::Core)?,
    )
    .map_err(WifiSecretStoreDenied::Core)?;
    let _ = append_with_readback(&mut port, expected, &plan, &mut snapshot)?;
    let replay = open_and_replay_with_history(&mut port, expected, &mut snapshot)?;
    let record = replay
        .state()
        .record(key)
        .map_err(WifiSecretStoreDenied::Core)?
        .ok_or(WifiSecretStoreDenied::ReadbackMismatch)?;
    if record.record_version != proposed_version
        || !matches!(&record.value, RecordValue::Present(bytes) if bytes.as_slice() == payload)
    {
        return Err(WifiSecretStoreDenied::ReadbackMismatch);
    }
    serial::write_fmt(format_args!(
        "C1_VAULT_WIFI_COMMITTED version={} readback=verified\r\n",
        proposed_version
    ));
    Ok(replay)
}

/// Reopens and re-identifies the exact disposable C1 fixture before the
/// contained provider-consumer proof. This grants no write or secret access.
pub(crate) fn revalidate_qemu_store_identity(
    expected: ValidatedRegionIdentity,
) -> Result<(), ProviderSecretStoreDenied> {
    let port = open_disposable_qemu_store_port()
        .map_err(|error| ProviderSecretStoreDenied::Open(c1_error_reason(error)))?
        .ok_or(ProviderSecretStoreDenied::FixtureMissing)?;
    if port.identity() != expected {
        return Err(ProviderSecretStoreDenied::IdentityChanged);
    }
    Ok(())
}

/// WiFi-typed counterpart to the same exact C1 identity check. Keeping the
/// error type distinct prevents the driver path from reporting provider
/// custody errors while granting no additional device-selection authority.
pub(crate) fn revalidate_qemu_wifi_store_identity(
    expected: ValidatedRegionIdentity,
) -> Result<(), WifiSecretStoreDenied> {
    let port = open_disposable_qemu_store_port()
        .map_err(|error| WifiSecretStoreDenied::Open(c1_error_reason(error)))?
        .ok_or(WifiSecretStoreDenied::FixtureMissing)?;
    if port.identity() != expected {
        return Err(WifiSecretStoreDenied::IdentityChanged);
    }
    Ok(())
}

/// A bounded I/O handle for the one disposable QEMU C1 fixture. It can be
/// opened only at the exact Q35 controller/port/GPT identity; it has no path
/// to enumerate, select, or fall back to physical media.
pub(crate) struct DisposableQemuStorePort {
    inner: AhciExplicitAtaPort,
    identity: ValidatedRegionIdentity,
}

impl DisposableQemuStorePort {
    fn open(inner: AhciExplicitAtaPort) -> Result<Self, DisposableQemuStoreDenied> {
        let identity = establish_identity(inner)?;
        Ok(Self { inner, identity })
    }

    pub(crate) const fn identity(&self) -> ValidatedRegionIdentity {
        self.identity
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

impl ValidatedStoreRegionPort for DisposableQemuStorePort {
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

/// Returns `Ok(None)` only when the explicitly-addressed disposable fixture is
/// absent. A present but foreign/malformed fixture is denied; physical media
/// is never selected or used as a fallback.
pub(crate) fn open_disposable_qemu_store_port(
) -> Result<Option<DisposableQemuStorePort>, DisposableQemuStoreDenied> {
    let Ok(controller) = pci::exact_ahci_controller(QEMU_AHCI_BDF) else {
        return Ok(None);
    };
    let Ok(inner) = AhciExplicitAtaPort::open(controller, QEMU_AHCI_PORT) else {
        return Ok(None);
    };
    DisposableQemuStorePort::open(inner).map(Some)
}

fn establish_identity(
    inner: AhciExplicitAtaPort,
) -> Result<ValidatedRegionIdentity, DisposableQemuStoreDenied> {
    if inner.port_index() != QEMU_AHCI_PORT {
        return Err(DisposableQemuStoreDenied::Bounds);
    }
    let device = unsafe {
        inner
            .identify()
            .map_err(DisposableQemuStoreDenied::Device)?
    };
    let total_lbas =
        device_sector_count(device).ok_or(DisposableQemuStoreDenied::InvalidDeviceGeometry)?;
    let mut mbr = [0u8; SECTOR_BYTES];
    read_sector_copy(inner, 0, &mut mbr)?;
    let mut primary_header = [0u8; SECTOR_BYTES];
    read_sector_copy(inner, 1, &mut primary_header)?;
    let mut primary_entries = [0u8; GPT_ENTRY_ARRAY_BYTES];
    read_entry_array(inner, 2, &mut primary_entries)?;
    let backup_header_lba = total_lbas
        .checked_sub(1)
        .ok_or(DisposableQemuStoreDenied::InvalidDeviceGeometry)?;
    let backup_entries_lba = backup_header_lba
        .checked_sub((GPT_ENTRY_ARRAY_BYTES / SECTOR_BYTES) as u64)
        .ok_or(DisposableQemuStoreDenied::InvalidDeviceGeometry)?;
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
    .map_err(DisposableQemuStoreDenied::Gpt)?;
    let log_lba_count = partition
        .lba_count
        .checked_sub(2)
        .ok_or(DisposableQemuStoreDenied::Bounds)?;
    let geometry = StoreGeometry {
        partition_start_lba: partition.first_lba,
        partition_lba_count: partition.lba_count,
        log_start_lba: 2,
        log_lba_count,
        logical_sector_size: SECTOR_BYTES as u32,
    };
    geometry
        .validate()
        .map_err(DisposableQemuStoreDenied::Core)?;
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
) -> Result<(), DisposableQemuStoreDenied> {
    let mut sector = C1_DMA_BUFFER.lock();
    unsafe {
        inner
            .read_sector(lba, &mut sector)
            .map_err(DisposableQemuStoreDenied::Device)?
    };
    out.copy_from_slice(&sector.0);
    Ok(())
}

fn read_entry_array(
    inner: AhciExplicitAtaPort,
    first_lba: u64,
    out: &mut [u8; GPT_ENTRY_ARRAY_BYTES],
) -> Result<(), DisposableQemuStoreDenied> {
    for index in 0..(GPT_ENTRY_ARRAY_BYTES / SECTOR_BYTES) {
        let lba = first_lba
            .checked_add(index as u64)
            .ok_or(DisposableQemuStoreDenied::Bounds)?;
        let mut sector = C1_DMA_BUFFER.lock();
        unsafe {
            inner
                .read_sector(lba, &mut sector)
                .map_err(DisposableQemuStoreDenied::Device)?
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

fn c1_error_reason(error: DisposableQemuStoreDenied) -> &'static str {
    match error {
        DisposableQemuStoreDenied::Device(reason) => reason,
        DisposableQemuStoreDenied::Port(_) => "c1_structured_store_revalidation_denied",
        DisposableQemuStoreDenied::Core(_) => "c1_structured_store_revalidation_core_denied",
        DisposableQemuStoreDenied::Gpt(_) => "c1_structured_store_revalidation_gpt_denied",
        DisposableQemuStoreDenied::Vault(_) => "c1_vault_complete_replay_denied",
        DisposableQemuStoreDenied::InvalidDeviceGeometry => {
            "c1_structured_store_revalidation_device_geometry_invalid"
        }
        DisposableQemuStoreDenied::Bounds => "c1_structured_store_revalidation_bounds_denied",
    }
}
