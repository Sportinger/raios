use raios_core::structured_store::{
    parse_superblock, replay_log, select_superblock, validate_transaction_plan,
    verify_committed_plan, verify_frame_readback, ReplayState, SelectedSuperblock, StoreDenied,
    StoreGeometry, StoreIdentity, TransactionPlan, STORE_BLOCK_LEN, SUPERBLOCK_LEN,
};

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
    SnapshotLengthMismatch,
    BoundsDenied,
    Core(StoreDenied),
}

impl<E> From<StoreDenied> for PortDenied<E> {
    fn from(value: StoreDenied) -> Self {
        Self::Core(value)
    }
}

pub(crate) fn open_and_replay<P: ValidatedStoreRegionPort>(
    port: &mut P,
    expected: ValidatedRegionIdentity,
    snapshot: &mut [u8],
) -> Result<ReplayState, PortDenied<P::Error>> {
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
    replay_log(snapshot, expected.store).map_err(PortDenied::Core)
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
    }

    impl MemoryPort {
        fn new() -> Self {
            let superblocks = [
                raios_core::structured_store::encode_superblock(0, 0, STORE, 1, GEOMETRY).unwrap(),
                raios_core::structured_store::encode_superblock(1, 1, STORE, 2, GEOMETRY).unwrap(),
            ];
            Self {
                identity: IDENTITY,
                superblocks,
                bytes: vec![0; GEOMETRY.log_lba_count as usize * STORE_BLOCK_LEN],
                fail_flush: false,
                mutate_identity_after_write: false,
                writes: 0,
            }
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
            out.copy_from_slice(&self.superblocks[copy_index as usize]);
            Ok(())
        }

        fn write_superblock_copy(
            &mut self,
            copy_index: u8,
            block: &[u8; SUPERBLOCK_LEN],
        ) -> Result<(), Self::Error> {
            self.superblocks[copy_index as usize].copy_from_slice(block);
            self.identity.store = parse_superblock(block).unwrap().identity;
            Ok(())
        }

        fn read_log_block(
            &mut self,
            relative_block: u64,
            out: &mut [u8; STORE_BLOCK_LEN],
        ) -> Result<(), Self::Error> {
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
}
