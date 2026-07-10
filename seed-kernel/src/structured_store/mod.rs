use raios_core::structured_store::{
    encode_superblock, parse_superblock, replay_log, replay_log_with_history, select_superblock,
    validate_transaction_plan, verify_committed_plan, verify_frame_readback, ReplayState,
    ReplayWithHistory, SelectedSuperblock, StoreDenied, StoreGeometry, StoreIdentity,
    TransactionPlan, STORE_BLOCK_LEN, SUPERBLOCK_LEN,
};
use raios_core::{
    sha256_bytes,
    structured_store_partition::{STRUCTURED_STORE_LABEL, STRUCTURED_STORE_TYPE_GUID_LE},
};

// This is the only format target. `run-stage0-qemu.ps1` must attach the
// disposable structured-store image to Q35's ICH9 AHCI controller at `ide.4`.
// The device digest remains an input because it is established by the bounded
// AHCI identify path and rechecked before the zero scan, the first write, and
// every subsequent write/readback pair.
const QEMU_TEST_PCI_SEGMENT: u16 = 0;
const QEMU_TEST_PCI_BUS: u8 = 0;
const QEMU_TEST_PCI_DEVICE: u8 = 31;
const QEMU_TEST_PCI_FUNCTION: u8 = 2;
const QEMU_TEST_AHCI_PORT: u8 = 4;
const QEMU_TEST_DISK_GUID_LE: [u8; 16] = [
    0x7a, 0xda, 0xed, 0x5e, 0xde, 0xc0, 0x55, 0x4a, 0x9a, 0x15, 0x00, 0x00, 0x00, 0x00, 0x13, 0x00,
];
const QEMU_TEST_PARTITION_GUID_LE: [u8; 16] = [
    0x7a, 0xda, 0xed, 0x5e, 0xde, 0xc0, 0x55, 0x4a, 0x9a, 0x15, 0x00, 0x00, 0x00, 0x00, 0x13, 0x01,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedRegionIdentity {
    pub(crate) pci_segment: u16,
    pub(crate) pci_bus: u8,
    pub(crate) pci_device: u8,
    pub(crate) pci_function: u8,
    pub(crate) controller_port: u8,
    pub(crate) device_identity_sha256: [u8; 32],
    pub(crate) gpt_disk_guid: [u8; 16],
    pub(crate) partition_type_guid: [u8; 16],
    pub(crate) partition_guid: [u8; 16],
    pub(crate) partition_label_sha256: [u8; 32],
    pub(crate) store: StoreIdentity,
    pub(crate) geometry: StoreGeometry,
}

impl ValidatedRegionIdentity {
    fn log_block_count(self) -> Result<u64, StoreDenied> {
        self.geometry.validate()?;
        self.geometry.log_byte_len()?;
        Ok(self.geometry.log_lba_count)
    }
}

/// A port is created only after the orchestrator-owned device/GPT validator has
/// bound an exact controller, port, device and partition. Offsets are relative
/// to that validated store log; this trait intentionally cannot enumerate,
/// select, format, relabel or address another disk region.
pub(crate) trait ValidatedStoreRegionPort {
    type Error;

    fn revalidate_identity(&mut self) -> Result<ValidatedRegionIdentity, Self::Error>;

    fn read_superblock_copy(
        &mut self,
        copy_index: u8,
        out: &mut [u8; SUPERBLOCK_LEN],
    ) -> Result<(), Self::Error>;

    fn write_superblock_copy(
        &mut self,
        copy_index: u8,
        block: &[u8; SUPERBLOCK_LEN],
    ) -> Result<(), Self::Error>;

    fn read_log_block(
        &mut self,
        relative_block: u64,
        out: &mut [u8; STORE_BLOCK_LEN],
    ) -> Result<(), Self::Error>;

    fn write_log_block(
        &mut self,
        relative_block: u64,
        block: &[u8; STORE_BLOCK_LEN],
    ) -> Result<(), Self::Error>;

    fn flush(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PortDenied<E> {
    Device(E),
    IdentityChanged,
    FormatTestMediaDenied,
    FormatRequiresEmptyRegion,
    SnapshotLengthMismatch,
    BoundsDenied,
    Core(StoreDenied),
}

impl<E> From<StoreDenied> for PortDenied<E> {
    fn from(value: StoreDenied) -> Self {
        Self::Core(value)
    }
}

/// Formats only the frozen disposable QEMU store image after proving that every
/// exposed store block is zero. This is intentionally not a general format API:
/// a physical or differently attached region is denied before any media write.
pub(crate) fn format_empty_disposable_test_media<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
) -> Result<(), PortDenied<P::Error>> {
    if !is_frozen_qemu_test_media(expected) {
        return Err(PortDenied::FormatTestMediaDenied);
    }
    revalidate(port, expected)?;

    let mut block = [0u8; STORE_BLOCK_LEN];
    for copy_index in 0..2 {
        port.read_superblock_copy(copy_index, &mut block)
            .map_err(PortDenied::Device)?;
        if block != [0; STORE_BLOCK_LEN] {
            return Err(PortDenied::FormatRequiresEmptyRegion);
        }
    }
    for relative_block in 0..expected.log_block_count().map_err(PortDenied::Core)? {
        port.read_log_block(relative_block, &mut block)
            .map_err(PortDenied::Device)?;
        if block != [0; STORE_BLOCK_LEN] {
            return Err(PortDenied::FormatRequiresEmptyRegion);
        }
    }
    revalidate(port, expected)?;

    let copies = [
        encode_superblock(0, 0, expected.store, 1, expected.geometry).map_err(PortDenied::Core)?,
        encode_superblock(1, 1, expected.store, 2, expected.geometry).map_err(PortDenied::Core)?,
    ];
    for (copy_index, formatted) in copies.iter().enumerate() {
        let copy_index = copy_index as u8;
        revalidate(port, expected)?;
        port.write_superblock_copy(copy_index, formatted)
            .map_err(PortDenied::Device)?;
        port.flush().map_err(PortDenied::Device)?;

        revalidate(port, expected)?;
        port.read_superblock_copy(copy_index, &mut block)
            .map_err(PortDenied::Device)?;
        if block != *formatted || parse_superblock(&block).is_err() {
            return Err(PortDenied::Core(StoreDenied::ReadbackMismatch));
        }
    }
    revalidate_store(port, expected)
}

fn is_frozen_qemu_test_media(identity: ValidatedRegionIdentity) -> bool {
    identity.pci_segment == QEMU_TEST_PCI_SEGMENT
        && identity.pci_bus == QEMU_TEST_PCI_BUS
        && identity.pci_device == QEMU_TEST_PCI_DEVICE
        && identity.pci_function == QEMU_TEST_PCI_FUNCTION
        && identity.controller_port == QEMU_TEST_AHCI_PORT
        && identity.gpt_disk_guid == QEMU_TEST_DISK_GUID_LE
        && identity.partition_type_guid == STRUCTURED_STORE_TYPE_GUID_LE
        && identity.partition_guid == QEMU_TEST_PARTITION_GUID_LE
        && identity.partition_label_sha256 == sha256_bytes(STRUCTURED_STORE_LABEL.as_bytes())
}

pub(crate) fn open_and_replay<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
    snapshot: &mut [u8],
) -> Result<ReplayState, PortDenied<P::Error>> {
    read_validated_snapshot(port, expected, snapshot)?;
    replay_log(snapshot, expected.store).map_err(PortDenied::Core)
}

pub(crate) fn open_and_replay_with_history<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
    snapshot: &mut [u8],
) -> Result<ReplayWithHistory, PortDenied<P::Error>> {
    read_validated_snapshot(port, expected, snapshot)?;
    replay_log_with_history(snapshot, expected.store).map_err(PortDenied::Core)
}

fn read_validated_snapshot<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
    snapshot: &mut [u8],
) -> Result<(), PortDenied<P::Error>> {
    revalidate_store(port, expected)?;
    let block_count = expected.log_block_count().map_err(PortDenied::Core)?;
    let expected_len = block_count
        .checked_mul(STORE_BLOCK_LEN as u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(PortDenied::BoundsDenied)?;
    if snapshot.len() != expected_len {
        return Err(PortDenied::SnapshotLengthMismatch);
    }

    for relative_block in 0..block_count {
        let start = usize::try_from(relative_block)
            .ok()
            .and_then(|value| value.checked_mul(STORE_BLOCK_LEN))
            .ok_or(PortDenied::BoundsDenied)?;
        let block = &mut snapshot[start..start + STORE_BLOCK_LEN];
        let out: &mut [u8; STORE_BLOCK_LEN] =
            block.try_into().map_err(|_| PortDenied::BoundsDenied)?;
        port.read_log_block(relative_block, out)
            .map_err(PortDenied::Device)?;
    }
    revalidate_store(port, expected)?;
    Ok(())
}

pub(crate) fn append_with_readback<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
    plan: &TransactionPlan,
    full_log_snapshot: &mut [u8],
) -> Result<ReplayState, PortDenied<P::Error>> {
    if plan.identity != expected.store {
        return Err(PortDenied::IdentityChanged);
    }
    let block_count = expected.log_block_count().map_err(PortDenied::Core)?;
    let log_byte_len = block_count
        .checked_mul(STORE_BLOCK_LEN as u64)
        .ok_or(PortDenied::BoundsDenied)?;
    validate_transaction_plan(plan, log_byte_len).map_err(PortDenied::Core)?;

    let mut readback = [0u8; STORE_BLOCK_LEN];
    for frame in &plan.frames {
        let end = frame
            .offset
            .checked_add(STORE_BLOCK_LEN as u64)
            .ok_or(PortDenied::BoundsDenied)?;
        if frame.offset % STORE_BLOCK_LEN as u64 != 0 || end > log_byte_len {
            return Err(PortDenied::BoundsDenied);
        }
        let relative_block = frame.offset / STORE_BLOCK_LEN as u64;

        // Identity is checked immediately before every individual write.
        revalidate_store(port, expected)?;
        port.write_log_block(relative_block, &frame.bytes)
            .map_err(PortDenied::Device)?;
        port.flush().map_err(PortDenied::Device)?;

        // A frame is not accepted until the same identity was re-established
        // and the exact block was read back, reparsed and compared.
        revalidate_store(port, expected)?;
        port.read_log_block(relative_block, &mut readback)
            .map_err(PortDenied::Device)?;
        verify_frame_readback(frame, &readback).map_err(PortDenied::Core)?;
    }

    let replay = open_and_replay(port, expected, full_log_snapshot)?;
    verify_committed_plan(&replay, plan).map_err(PortDenied::Core)?;
    Ok(replay)
}

pub(crate) fn replace_superblock_with_readback<P: ValidatedStoreRegionPort>(
    port: &mut P,
    current: ValidatedRegionIdentity,
    next: ValidatedRegionIdentity,
    replacement: &SelectedSuperblock,
) -> Result<(), PortDenied<P::Error>> {
    if !same_physical_region(current, next)
        || replacement.superblock.identity != next.store
        || replacement.superblock.geometry != next.geometry
    {
        return Err(PortDenied::IdentityChanged);
    }
    let parsed_replacement = parse_superblock(&replacement.bytes).map_err(PortDenied::Core)?;
    if parsed_replacement != replacement.superblock
        || next.store.store_uuid != current.store.store_uuid
        || next.store.generation <= current.store.generation
    {
        return Err(PortDenied::Core(StoreDenied::StaleStoreGeneration));
    }

    revalidate_store(port, current)?;
    let mut current_copy0 = [0u8; SUPERBLOCK_LEN];
    let mut current_copy1 = [0u8; SUPERBLOCK_LEN];
    port.read_superblock_copy(0, &mut current_copy0)
        .map_err(PortDenied::Device)?;
    port.read_superblock_copy(1, &mut current_copy1)
        .map_err(PortDenied::Device)?;
    let selected = select_superblock(
        &current_copy0,
        &current_copy1,
        current.store,
        current.geometry,
    )
    .map_err(PortDenied::Core)?;
    if replacement.superblock.copy_index == selected.superblock.copy_index
        || replacement.superblock.selection_epoch <= selected.superblock.selection_epoch
    {
        return Err(PortDenied::Core(StoreDenied::AmbiguousSuperblockSelection));
    }

    revalidate_store(port, current)?;
    port.write_superblock_copy(replacement.superblock.copy_index, &replacement.bytes)
        .map_err(PortDenied::Device)?;
    port.flush().map_err(PortDenied::Device)?;

    let mut readback = [0u8; SUPERBLOCK_LEN];
    port.read_superblock_copy(replacement.superblock.copy_index, &mut readback)
        .map_err(PortDenied::Device)?;
    if readback != replacement.bytes || parse_superblock(&readback) != Ok(replacement.superblock) {
        return Err(PortDenied::Core(StoreDenied::ReadbackMismatch));
    }
    revalidate_store(port, next)?;
    Ok(())
}

fn revalidate<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
) -> Result<(), PortDenied<P::Error>> {
    if !identity_well_formed(expected) {
        return Err(PortDenied::IdentityChanged);
    }
    let observed = port.revalidate_identity().map_err(PortDenied::Device)?;
    if observed != expected {
        return Err(PortDenied::IdentityChanged);
    }
    Ok(())
}

fn identity_well_formed(identity: ValidatedRegionIdentity) -> bool {
    identity.controller_port < 32
        && identity.device_identity_sha256 != [0; 32]
        && identity.gpt_disk_guid != [0; 16]
        && identity.partition_type_guid != [0; 16]
        && identity.partition_guid != [0; 16]
        && identity.partition_label_sha256 != [0; 32]
        && identity.store.store_uuid != [0; 16]
        && identity.store.generation != 0
        && identity.geometry.validate().is_ok()
}

fn revalidate_store<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
) -> Result<(), PortDenied<P::Error>> {
    revalidate(port, expected)?;
    let mut copy0 = [0u8; SUPERBLOCK_LEN];
    let mut copy1 = [0u8; SUPERBLOCK_LEN];
    port.read_superblock_copy(0, &mut copy0)
        .map_err(PortDenied::Device)?;
    port.read_superblock_copy(1, &mut copy1)
        .map_err(PortDenied::Device)?;
    select_superblock(&copy0, &copy1, expected.store, expected.geometry)
        .map_err(PortDenied::Core)?;
    Ok(())
}

fn same_physical_region(current: ValidatedRegionIdentity, next: ValidatedRegionIdentity) -> bool {
    current.pci_segment == next.pci_segment
        && current.pci_bus == next.pci_bus
        && current.pci_device == next.pci_device
        && current.pci_function == next.pci_function
        && current.controller_port == next.controller_port
        && current.device_identity_sha256 == next.device_identity_sha256
        && current.gpt_disk_guid == next.gpt_disk_guid
        && current.partition_type_guid == next.partition_type_guid
        && current.partition_guid == next.partition_guid
        && current.partition_label_sha256 == next.partition_label_sha256
        && current.geometry == next.geometry
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};
    use raios_core::structured_store::{
        plan_superblock_replacement, plan_transaction, select_superblock, RecordKey,
        RecordOperation, TransactionRequest,
    };

    use super::*;

    const STORE: StoreIdentity = StoreIdentity {
        store_uuid: *b"raios-port-test-",
        generation: 3,
    };
    const GEOMETRY: StoreGeometry = StoreGeometry {
        partition_start_lba: 4096,
        partition_lba_count: 18,
        log_start_lba: 2,
        log_lba_count: 16,
        logical_sector_size: 512,
    };
    const IDENTITY: ValidatedRegionIdentity = ValidatedRegionIdentity {
        pci_segment: 0,
        pci_bus: 0,
        pci_device: 31,
        pci_function: 2,
        controller_port: 1,
        device_identity_sha256: [0x11; 32],
        gpt_disk_guid: [0x22; 16],
        partition_type_guid: [0x33; 16],
        partition_guid: [0x44; 16],
        partition_label_sha256: [0x55; 32],
        store: STORE,
        geometry: GEOMETRY,
    };
    const KEY: RecordKey = RecordKey {
        namespace: *b"vault-----------",
        record_id: *b"provider-key----",
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum DeviceError {
        Io,
    }

    struct MemoryPort {
        identity: ValidatedRegionIdentity,
        superblocks: [[u8; SUPERBLOCK_LEN]; 2],
        bytes: Vec<u8>,
        fail_flush: bool,
        mutate_identity_after_write: bool,
        writes: u64,
        superblock_writes: u8,
        flushes: u8,
        log_reads: u64,
        superblock_reads: u64,
        first_superblock_write_after_reads: Option<(u64, u64)>,
    }

    impl MemoryPort {
        fn empty(identity: ValidatedRegionIdentity) -> Self {
            Self {
                identity,
                superblocks: [[0; SUPERBLOCK_LEN]; 2],
                bytes: vec![0; identity.geometry.log_lba_count as usize * STORE_BLOCK_LEN],
                fail_flush: false,
                mutate_identity_after_write: false,
                writes: 0,
                superblock_writes: 0,
                flushes: 0,
                log_reads: 0,
                superblock_reads: 0,
                first_superblock_write_after_reads: None,
            }
        }

        fn new() -> Self {
            let mut port = Self::empty(IDENTITY);
            port.superblocks = [
                encode_superblock(0, 0, STORE, 1, GEOMETRY).unwrap(),
                encode_superblock(1, 1, STORE, 2, GEOMETRY).unwrap(),
            ];
            port
        }
    }

    fn qemu_test_identity() -> ValidatedRegionIdentity {
        ValidatedRegionIdentity {
            pci_segment: QEMU_TEST_PCI_SEGMENT,
            pci_bus: QEMU_TEST_PCI_BUS,
            pci_device: QEMU_TEST_PCI_DEVICE,
            pci_function: QEMU_TEST_PCI_FUNCTION,
            controller_port: QEMU_TEST_AHCI_PORT,
            device_identity_sha256: [0x66; 32],
            gpt_disk_guid: QEMU_TEST_DISK_GUID_LE,
            partition_type_guid: STRUCTURED_STORE_TYPE_GUID_LE,
            partition_guid: QEMU_TEST_PARTITION_GUID_LE,
            partition_label_sha256: sha256_bytes(STRUCTURED_STORE_LABEL.as_bytes()),
            store: StoreIdentity {
                store_uuid: *b"raios-qemu-test-",
                generation: 1,
            },
            geometry: GEOMETRY,
        }
    }

    impl ValidatedStoreRegionPort for MemoryPort {
        type Error = DeviceError;

        fn revalidate_identity(&mut self) -> Result<ValidatedRegionIdentity, Self::Error> {
            Ok(self.identity)
        }

        fn read_superblock_copy(
            &mut self,
            copy_index: u8,
            out: &mut [u8; SUPERBLOCK_LEN],
        ) -> Result<(), Self::Error> {
            self.superblock_reads += 1;
            out.copy_from_slice(&self.superblocks[copy_index as usize]);
            Ok(())
        }

        fn write_superblock_copy(
            &mut self,
            copy_index: u8,
            block: &[u8; SUPERBLOCK_LEN],
        ) -> Result<(), Self::Error> {
            if self.superblock_writes == 0 {
                self.first_superblock_write_after_reads =
                    Some((self.superblock_reads, self.log_reads));
            }
            self.superblock_writes += 1;
            self.superblocks[copy_index as usize].copy_from_slice(block);
            self.identity.store = parse_superblock(block).unwrap().identity;
            Ok(())
        }

        fn read_log_block(
            &mut self,
            relative_block: u64,
            out: &mut [u8; STORE_BLOCK_LEN],
        ) -> Result<(), Self::Error> {
            self.log_reads += 1;
            let start = relative_block as usize * STORE_BLOCK_LEN;
            out.copy_from_slice(&self.bytes[start..start + STORE_BLOCK_LEN]);
            Ok(())
        }

        fn write_log_block(
            &mut self,
            relative_block: u64,
            block: &[u8; STORE_BLOCK_LEN],
        ) -> Result<(), Self::Error> {
            let start = relative_block as usize * STORE_BLOCK_LEN;
            self.bytes[start..start + STORE_BLOCK_LEN].copy_from_slice(block);
            self.writes += 1;
            if self.mutate_identity_after_write {
                self.identity.partition_guid[0] ^= 1;
            }
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            if self.fail_flush {
                Err(DeviceError::Io)
            } else {
                self.flushes += 1;
                Ok(())
            }
        }
    }

    fn build_plan(port: &mut MemoryPort) -> TransactionPlan {
        let mut snapshot = vec![0; port.bytes.len()];
        let replay = open_and_replay(port, IDENTITY, &mut snapshot).unwrap();
        plan_transaction(
            &replay,
            TransactionRequest {
                identity: STORE,
                key: KEY,
                operation: RecordOperation::Put,
                expected_committed_version: None,
            },
            b"ciphertext-envelope",
            port.bytes.len() as u64,
        )
        .unwrap()
    }

    #[test]
    fn append_flushes_reads_back_and_replays_commit() {
        let mut port = MemoryPort::new();
        let plan = build_plan(&mut port);
        let mut snapshot = vec![0; port.bytes.len()];
        let replay = append_with_readback(&mut port, IDENTITY, &plan, &mut snapshot).unwrap();

        assert_eq!(port.writes, plan.frames.len() as u64);
        assert_eq!(replay.record(KEY).unwrap().unwrap().record_version, 1);
    }

    #[test]
    fn history_open_reuses_validated_readback_and_keeps_verified_commits() {
        let mut port = MemoryPort::new();
        let plan = build_plan(&mut port);
        let mut snapshot = vec![0; port.bytes.len()];
        append_with_readback(&mut port, IDENTITY, &plan, &mut snapshot).unwrap();

        let replay = open_and_replay_with_history(&mut port, IDENTITY, &mut snapshot).unwrap();
        let history = replay.committed_history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].transaction_id, plan.transaction_id);
        assert_eq!(replay.state().record(KEY).unwrap(), Some(&history[0]));

        port.identity.partition_guid[0] ^= 1;
        assert_eq!(
            open_and_replay_with_history(&mut port, IDENTITY, &mut snapshot),
            Err(PortDenied::IdentityChanged)
        );
    }

    #[test]
    fn identity_change_and_flush_failure_stop_before_positive_replay() {
        let mut port = MemoryPort::new();
        let plan = build_plan(&mut port);
        port.mutate_identity_after_write = true;
        let mut snapshot = vec![0; port.bytes.len()];
        assert_eq!(
            append_with_readback(&mut port, IDENTITY, &plan, &mut snapshot),
            Err(PortDenied::IdentityChanged)
        );

        let mut port = MemoryPort::new();
        let plan = build_plan(&mut port);
        port.fail_flush = true;
        let mut snapshot = vec![0; port.bytes.len()];
        assert_eq!(
            append_with_readback(&mut port, IDENTITY, &plan, &mut snapshot),
            Err(PortDenied::Device(DeviceError::Io))
        );
    }

    #[test]
    fn superblock_replacement_writes_only_other_copy_then_reads_back() {
        let mut port = MemoryPort::new();
        let selected =
            select_superblock(&port.superblocks[0], &port.superblocks[1], STORE, GEOMETRY).unwrap();
        let replacement = plan_superblock_replacement(&selected.superblock, 4).unwrap();
        let next = ValidatedRegionIdentity {
            store: replacement.superblock.identity,
            ..IDENTITY
        };
        let preserved = port.superblocks[selected.superblock.copy_index as usize];

        replace_superblock_with_readback(&mut port, IDENTITY, next, &replacement).unwrap();

        assert_eq!(
            port.superblocks[selected.superblock.copy_index as usize],
            preserved
        );
        assert_eq!(port.identity, next);
    }

    #[test]
    fn qemu_only_format_reads_every_zero_block_then_dual_writes_and_readbacks() {
        let identity = qemu_test_identity();
        let mut port = MemoryPort::empty(identity);

        format_empty_disposable_test_media(&mut port, identity).unwrap();

        assert_eq!(
            port.first_superblock_write_after_reads,
            Some((2, identity.geometry.log_lba_count))
        );
        assert_eq!(port.superblock_writes, 2);
        assert_eq!(port.flushes, 2);
        let selected = select_superblock(
            &port.superblocks[0],
            &port.superblocks[1],
            identity.store,
            identity.geometry,
        )
        .unwrap();
        assert_eq!(selected.superblock.copy_index, 1);
    }

    #[test]
    fn qemu_only_format_denies_foreign_or_nonempty_media_before_writing() {
        let mut foreign = qemu_test_identity();
        foreign.controller_port -= 1;
        let mut port = MemoryPort::empty(foreign);
        assert_eq!(
            format_empty_disposable_test_media(&mut port, foreign),
            Err(PortDenied::FormatTestMediaDenied)
        );
        assert_eq!(port.superblock_writes, 0);

        let identity = qemu_test_identity();
        let mut port = MemoryPort::empty(identity);
        port.bytes[0] = 1;
        assert_eq!(
            format_empty_disposable_test_media(&mut port, identity),
            Err(PortDenied::FormatRequiresEmptyRegion)
        );
        assert_eq!(port.superblock_writes, 0);
    }
}
