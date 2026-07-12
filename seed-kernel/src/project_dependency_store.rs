use alloc::{vec, vec::Vec};

use raios_core::{
    project_dependency::{
        decode_dependency_chunk, decode_dependency_manifest, dependency_chunk_record_key,
        dependency_manifest_record_key, is_dependency_manifest_record_key, BuiltDependencyBundle,
        DependencyBundle,
    },
    project_workspace::ProjectId,
    structured_store::{plan_transaction, RecordOperation, RecordValue, TransactionRequest},
};
use sha2::{Digest, Sha256};

use crate::{
    structured_store::{append_with_readback, open_and_replay_with_history, PortDenied},
    structured_store_c1::{
        open_disposable_qemu_store_port, DisposableQemuStoreDenied, DisposableQemuStorePort,
    },
};

pub(crate) struct ChunkPersisted {
    pub(crate) storage_write_attempted: bool,
    pub(crate) persisted: bool,
}

#[derive(Debug)]
pub(crate) enum DependencyStoreDenied {
    FixtureMissing,
    Open,
    Port,
    Core,
    Bounds,
    ChunkCollision,
    ChunkMissing,
    ChunkInvalid,
    FileHashMismatch,
    ManifestCollision,
    ManifestInvalid,
    AmbiguousPackage,
}

impl DependencyStoreDenied {
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::FixtureMissing => "dependency_qemu_store_missing",
            Self::Open => "dependency_qemu_store_open_denied",
            Self::Port => "dependency_qemu_store_io_denied",
            Self::Core => "dependency_qemu_store_core_denied",
            Self::Bounds => "dependency_qemu_store_bounds_denied",
            Self::ChunkCollision => "dependency_chunk_key_collision",
            Self::ChunkMissing => "dependency_chunk_missing",
            Self::ChunkInvalid => "dependency_chunk_invalid",
            Self::FileHashMismatch => "dependency_file_hash_mismatch",
            Self::ManifestCollision => "dependency_manifest_key_collision",
            Self::ManifestInvalid => "dependency_manifest_invalid",
            Self::AmbiguousPackage => "dependency_package_ambiguous",
        }
    }
}

pub(crate) fn persist_chunk(
    bytes: &[u8],
) -> Result<([u8; 32], ChunkPersisted), DependencyStoreDenied> {
    let payload = raios_core::project_dependency::encode_dependency_chunk(bytes)
        .map_err(|_| DependencyStoreDenied::ChunkInvalid)?;
    let decoded =
        decode_dependency_chunk(&payload).map_err(|_| DependencyStoreDenied::ChunkInvalid)?;
    let key = dependency_chunk_record_key(decoded.sha256);
    let (mut port, identity, mut snapshot) = open()?;
    let replay =
        open_and_replay_with_history(&mut port, identity, &mut snapshot).map_err(map_port)?;
    if let Some(record) = replay
        .state()
        .record(key)
        .map_err(|_| DependencyStoreDenied::Core)?
    {
        let RecordValue::Present(existing) = &record.value else {
            return Err(DependencyStoreDenied::ChunkCollision);
        };
        let verified =
            decode_dependency_chunk(existing).map_err(|_| DependencyStoreDenied::ChunkCollision)?;
        if verified.sha256 != decoded.sha256 || existing.as_slice() != payload.as_slice() {
            return Err(DependencyStoreDenied::ChunkCollision);
        }
        return Ok((
            decoded.sha256,
            ChunkPersisted {
                storage_write_attempted: false,
                persisted: true,
            },
        ));
    }
    append(&mut port, identity, &mut snapshot, key, &payload)?;
    Ok((
        decoded.sha256,
        ChunkPersisted {
            storage_write_attempted: true,
            persisted: true,
        },
    ))
}

pub(crate) fn commit_bundle(built: &BuiltDependencyBundle) -> Result<bool, DependencyStoreDenied> {
    let (mut port, identity, mut snapshot) = open()?;
    let replay =
        open_and_replay_with_history(&mut port, identity, &mut snapshot).map_err(map_port)?;
    for record in replay
        .committed_history()
        .map_err(|_| DependencyStoreDenied::Core)?
    {
        if !is_dependency_manifest_record_key(record.key) {
            continue;
        }
        let RecordValue::Present(payload) = &record.value else {
            return Err(DependencyStoreDenied::ManifestInvalid);
        };
        let existing = decode_dependency_manifest(payload)
            .map_err(|_| DependencyStoreDenied::ManifestInvalid)?;
        if existing.project_id == built.bundle.project_id
            && existing.project_revision_sha256 == built.bundle.project_revision_sha256
            && existing.name == built.bundle.name
            && existing.version == built.bundle.version
            && existing.bundle_sha256 != built.bundle.bundle_sha256
        {
            return Err(DependencyStoreDenied::AmbiguousPackage);
        }
    }
    verify_chunks(&replay, &built.bundle)?;
    let key = dependency_manifest_record_key(built.bundle.bundle_sha256);
    let manifest_written = if let Some(record) = replay
        .state()
        .record(key)
        .map_err(|_| DependencyStoreDenied::Core)?
    {
        let RecordValue::Present(payload) = &record.value else {
            return Err(DependencyStoreDenied::ManifestCollision);
        };
        if payload.as_slice() != built.manifest_payload.as_slice()
            || decode_dependency_manifest(payload)
                .map_err(|_| DependencyStoreDenied::ManifestCollision)?
                != built.bundle
        {
            return Err(DependencyStoreDenied::ManifestCollision);
        }
        false
    } else {
        append(
            &mut port,
            identity,
            &mut snapshot,
            key,
            &built.manifest_payload,
        )?;
        true
    };
    let loaded = load_from_open(
        &mut port,
        identity,
        &mut snapshot,
        built.bundle.project_id,
        built.bundle.project_revision_sha256,
    )?;
    if !loaded.iter().any(|bundle| bundle == &built.bundle) {
        return Err(DependencyStoreDenied::ManifestInvalid);
    }
    Ok(manifest_written)
}

pub(crate) fn load_bundles(
    project_id: ProjectId,
    revision_sha256: [u8; 32],
) -> Result<Vec<DependencyBundle>, DependencyStoreDenied> {
    let (mut port, identity, mut snapshot) = open()?;
    load_from_open(
        &mut port,
        identity,
        &mut snapshot,
        project_id,
        revision_sha256,
    )
}

fn load_from_open(
    port: &mut DisposableQemuStorePort,
    identity: crate::structured_store::ValidatedRegionIdentity,
    snapshot: &mut [u8],
    project_id: ProjectId,
    revision_sha256: [u8; 32],
) -> Result<Vec<DependencyBundle>, DependencyStoreDenied> {
    let replay = open_and_replay_with_history(port, identity, snapshot).map_err(map_port)?;
    let mut bundles = Vec::new();
    for record in replay
        .committed_history()
        .map_err(|_| DependencyStoreDenied::Core)?
    {
        if !is_dependency_manifest_record_key(record.key) {
            continue;
        }
        let RecordValue::Present(payload) = &record.value else {
            return Err(DependencyStoreDenied::ManifestInvalid);
        };
        let bundle = decode_dependency_manifest(payload)
            .map_err(|_| DependencyStoreDenied::ManifestInvalid)?;
        if dependency_manifest_record_key(bundle.bundle_sha256) != record.key {
            return Err(DependencyStoreDenied::ManifestCollision);
        }
        if bundle.project_id == project_id && bundle.project_revision_sha256 == revision_sha256 {
            verify_chunks(&replay, &bundle)?;
            if bundles.iter().any(|existing: &DependencyBundle| {
                existing.name == bundle.name
                    && existing.version == bundle.version
                    && existing.bundle_sha256 != bundle.bundle_sha256
            }) {
                return Err(DependencyStoreDenied::AmbiguousPackage);
            }
            bundles.push(bundle);
        }
    }
    bundles.sort_by(|a, b| {
        a.name
            .as_bytes()
            .cmp(b.name.as_bytes())
            .then(a.version.as_bytes().cmp(b.version.as_bytes()))
    });
    Ok(bundles)
}

fn verify_chunks(
    replay: &crate::structured_store::ValidatedReplayWithHistory,
    bundle: &DependencyBundle,
) -> Result<(), DependencyStoreDenied> {
    for file in &bundle.files {
        let mut hash = Sha256::new();
        let mut length = 0usize;
        for chunk in &file.chunks {
            let record = replay
                .state()
                .record(dependency_chunk_record_key(chunk.sha256))
                .map_err(|_| DependencyStoreDenied::Core)?
                .ok_or(DependencyStoreDenied::ChunkMissing)?;
            let RecordValue::Present(payload) = &record.value else {
                return Err(DependencyStoreDenied::ChunkMissing);
            };
            let decoded = decode_dependency_chunk(payload)
                .map_err(|_| DependencyStoreDenied::ChunkInvalid)?;
            if decoded.sha256 != chunk.sha256 || decoded.bytes.len() != chunk.byte_len {
                return Err(DependencyStoreDenied::ChunkInvalid);
            }
            hash.update(&decoded.bytes);
            length += decoded.bytes.len();
        }
        let digest: [u8; 32] = hash.finalize().into();
        if length != file.whole_byte_len || digest != file.whole_sha256 {
            return Err(DependencyStoreDenied::FileHashMismatch);
        }
    }
    Ok(())
}

fn open() -> Result<
    (
        DisposableQemuStorePort,
        crate::structured_store::ValidatedRegionIdentity,
        Vec<u8>,
    ),
    DependencyStoreDenied,
> {
    let port = open_disposable_qemu_store_port()
        .map_err(map_open)?
        .ok_or(DependencyStoreDenied::FixtureMissing)?;
    let identity = port.identity();
    let len = usize::try_from(
        identity
            .geometry
            .log_byte_len()
            .map_err(|_| DependencyStoreDenied::Core)?,
    )
    .map_err(|_| DependencyStoreDenied::Bounds)?;
    Ok((port, identity, vec![0; len]))
}
fn append(
    port: &mut DisposableQemuStorePort,
    identity: crate::structured_store::ValidatedRegionIdentity,
    snapshot: &mut [u8],
    key: raios_core::structured_store::RecordKey,
    payload: &[u8],
) -> Result<(), DependencyStoreDenied> {
    let replay = open_and_replay_with_history(port, identity, snapshot).map_err(map_port)?;
    let plan = plan_transaction(
        replay.state(),
        TransactionRequest {
            identity: identity.store,
            key,
            operation: RecordOperation::Put,
            expected_committed_version: None,
        },
        payload,
        identity
            .geometry
            .log_byte_len()
            .map_err(|_| DependencyStoreDenied::Core)?,
    )
    .map_err(|_| DependencyStoreDenied::Core)?;
    append_with_readback(port, identity, &plan, snapshot).map_err(map_port)?;
    Ok(())
}
fn map_open(_: DisposableQemuStoreDenied) -> DependencyStoreDenied {
    DependencyStoreDenied::Open
}
fn map_port(_: PortDenied<&'static str>) -> DependencyStoreDenied {
    DependencyStoreDenied::Port
}
