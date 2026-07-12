use alloc::{vec, vec::Vec};

use raios_core::{
    project_workspace::{
        blob_record_key, decode_blob_record, decode_revision_manifest, is_revision_record_key,
        revision_record_key, BuiltRevision, ProjectId, ProjectRevision,
    },
    structured_store::{plan_transaction, RecordOperation, RecordValue, TransactionRequest},
};

use crate::{
    structured_store::{append_with_readback, open_and_replay_with_history, PortDenied},
    structured_store_c1::{
        open_disposable_qemu_store_port, DisposableQemuStoreDenied, DisposableQemuStorePort,
    },
};

#[derive(Debug)]
pub(crate) enum ProjectStoreDenied {
    FixtureMissing,
    Open,
    Port,
    Core,
    Bounds,
    BlobCollision,
    RevisionCollision,
    BlobMissing,
    BlobInvalid,
    RevisionInvalid,
}

impl ProjectStoreDenied {
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::FixtureMissing => "project_qemu_store_missing",
            Self::Open => "project_qemu_store_open_denied",
            Self::Port => "project_qemu_store_io_denied",
            Self::Core => "project_qemu_store_core_denied",
            Self::Bounds => "project_qemu_store_bounds_denied",
            Self::BlobCollision => "project_blob_key_collision",
            Self::RevisionCollision => "project_revision_key_collision",
            Self::BlobMissing => "project_blob_missing",
            Self::BlobInvalid => "project_blob_invalid",
            Self::RevisionInvalid => "project_revision_invalid",
        }
    }
}

pub(crate) fn commit(built: &BuiltRevision) -> Result<(), ProjectStoreDenied> {
    let (mut port, identity, mut snapshot) = open()?;

    for blob in &built.blobs {
        let replay =
            open_and_replay_with_history(&mut port, identity, &mut snapshot).map_err(map_port)?;
        let key = blob_record_key(blob.sha256);
        if let Some(existing) = replay
            .state()
            .record(key)
            .map_err(|_| ProjectStoreDenied::Core)?
        {
            let RecordValue::Present(payload) = &existing.value else {
                return Err(ProjectStoreDenied::BlobCollision);
            };
            let decoded =
                decode_blob_record(payload).map_err(|_| ProjectStoreDenied::BlobCollision)?;
            if decoded.sha256 != blob.sha256 || payload.as_slice() != blob.payload.as_slice() {
                return Err(ProjectStoreDenied::BlobCollision);
            }
            continue;
        }
        append(&mut port, identity, &mut snapshot, key, None, &blob.payload)?;
    }

    // The immutable manifest is deliberately last. Committed blobs without it
    // are inert and cannot make an interrupted import visible as a revision.
    let replay =
        open_and_replay_with_history(&mut port, identity, &mut snapshot).map_err(map_port)?;
    let key = revision_record_key(built.revision.revision_sha256);
    if let Some(existing) = replay
        .state()
        .record(key)
        .map_err(|_| ProjectStoreDenied::Core)?
    {
        let RecordValue::Present(payload) = &existing.value else {
            return Err(ProjectStoreDenied::RevisionCollision);
        };
        let decoded =
            decode_revision_manifest(payload).map_err(|_| ProjectStoreDenied::RevisionCollision)?;
        if decoded != built.revision || payload.as_slice() != built.manifest_payload.as_slice() {
            return Err(ProjectStoreDenied::RevisionCollision);
        }
    } else {
        append(
            &mut port,
            identity,
            &mut snapshot,
            key,
            None,
            &built.manifest_payload,
        )?;
    }

    let loaded = inspect_from_open(
        &mut port,
        identity,
        &mut snapshot,
        built.revision.project_id,
    )?
    .ok_or(ProjectStoreDenied::RevisionInvalid)?;
    if loaded != built.revision {
        return Err(ProjectStoreDenied::RevisionInvalid);
    }
    Ok(())
}

pub(crate) fn inspect(
    project_id: ProjectId,
) -> Result<Option<ProjectRevision>, ProjectStoreDenied> {
    let (mut port, identity, mut snapshot) = open()?;
    inspect_from_open(&mut port, identity, &mut snapshot, project_id)
}

fn inspect_from_open(
    port: &mut DisposableQemuStorePort,
    identity: crate::structured_store::ValidatedRegionIdentity,
    snapshot: &mut [u8],
    project_id: ProjectId,
) -> Result<Option<ProjectRevision>, ProjectStoreDenied> {
    let replay = open_and_replay_with_history(port, identity, snapshot).map_err(map_port)?;
    let mut latest = None;
    for record in replay
        .committed_history()
        .map_err(|_| ProjectStoreDenied::Core)?
        .iter()
        .rev()
    {
        if !is_revision_record_key(record.key) {
            continue;
        }
        let RecordValue::Present(payload) = &record.value else {
            return Err(ProjectStoreDenied::RevisionInvalid);
        };
        let revision =
            decode_revision_manifest(payload).map_err(|_| ProjectStoreDenied::RevisionInvalid)?;
        if revision_record_key(revision.revision_sha256) != record.key {
            return Err(ProjectStoreDenied::RevisionCollision);
        }
        if revision.project_id == project_id {
            latest = Some(revision);
            break;
        }
    }
    let Some(revision) = latest else {
        return Ok(None);
    };

    for entry in &revision.entries {
        let record = replay
            .state()
            .record(blob_record_key(entry.blob_sha256))
            .map_err(|_| ProjectStoreDenied::Core)?
            .ok_or(ProjectStoreDenied::BlobMissing)?;
        let RecordValue::Present(payload) = &record.value else {
            return Err(ProjectStoreDenied::BlobMissing);
        };
        let decoded = decode_blob_record(payload).map_err(|_| ProjectStoreDenied::BlobInvalid)?;
        if decoded.sha256 != entry.blob_sha256
            || decoded.bytes.len() != entry.byte_len
            || blob_record_key(decoded.sha256) != record.key
        {
            return Err(ProjectStoreDenied::BlobInvalid);
        }
    }
    Ok(Some(revision))
}

fn open() -> Result<
    (
        DisposableQemuStorePort,
        crate::structured_store::ValidatedRegionIdentity,
        Vec<u8>,
    ),
    ProjectStoreDenied,
> {
    let port = open_disposable_qemu_store_port()
        .map_err(map_open)?
        .ok_or(ProjectStoreDenied::FixtureMissing)?;
    let identity = port.identity();
    let snapshot_len = usize::try_from(
        identity
            .geometry
            .log_byte_len()
            .map_err(|_| ProjectStoreDenied::Core)?,
    )
    .map_err(|_| ProjectStoreDenied::Bounds)?;
    Ok((port, identity, vec![0u8; snapshot_len]))
}

fn append(
    port: &mut DisposableQemuStorePort,
    identity: crate::structured_store::ValidatedRegionIdentity,
    snapshot: &mut [u8],
    key: raios_core::structured_store::RecordKey,
    expected_committed_version: Option<u64>,
    payload: &[u8],
) -> Result<(), ProjectStoreDenied> {
    let replay = open_and_replay_with_history(port, identity, snapshot).map_err(map_port)?;
    let plan = plan_transaction(
        replay.state(),
        TransactionRequest {
            identity: identity.store,
            key,
            operation: RecordOperation::Put,
            expected_committed_version,
        },
        payload,
        identity
            .geometry
            .log_byte_len()
            .map_err(|_| ProjectStoreDenied::Core)?,
    )
    .map_err(|_| ProjectStoreDenied::Core)?;
    append_with_readback(port, identity, &plan, snapshot).map_err(map_port)?;
    Ok(())
}

fn map_open(_: DisposableQemuStoreDenied) -> ProjectStoreDenied {
    ProjectStoreDenied::Open
}

fn map_port(_: PortDenied<&'static str>) -> ProjectStoreDenied {
    ProjectStoreDenied::Port
}
