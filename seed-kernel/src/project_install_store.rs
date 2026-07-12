use alloc::{string::String, vec::Vec};

use crate::{agent_protocol::artifact_store, ahci, pci};
use raios_core::{
    artifact_blob_frame::{
        parse_artifact_blob_frame, plan_artifact_blob_write, ARTIFACT_BLOB_FRAME_HEADER_LEN,
    },
    durable_record_frame::{
        parse_reclog_frame, plan_reclog_append, scan_reclog_payloads, PlannedAppend,
    },
    project_build::{decode_receipt, validate_build_receipt},
    project_install::{
        decode_install_commit, decode_install_intent, encode_install_commit, encode_install_intent,
        install_action_signature_payload_sha256, resolve_install_log, seal_install_action,
        seal_install_commit, validate_install_action, validate_install_envelope,
        validate_install_intent, ContentBlobRef, ProjectInstallAction, ProjectInstallActionKind,
        ProjectInstallAuthority, ProjectInstallCommit, ProjectInstallEnvelope,
        ProjectInstallIntent, ResolvedProjectInstall,
    },
    project_runtime::WorkspaceRunBinding,
    promotion_attestation::{
        verify_promotion_authority_signature, PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
    },
    sha256_bytes,
};

#[derive(Clone)]
pub(crate) struct ProjectInstallReplay {
    pub(crate) commits: Vec<ProjectInstallCommit>,
    pub(crate) resolved: ResolvedProjectInstall,
    pub(crate) reclog_tail_seq: u64,
}

#[derive(Clone)]
pub(crate) struct InstalledProjectPayloads {
    pub(crate) install_commit_sha256: [u8; 32],
    pub(crate) envelope: ProjectInstallEnvelope,
    pub(crate) candidate_blob: ContentBlobRef,
    pub(crate) receipt_blob: ContentBlobRef,
    pub(crate) candidate: Vec<u8>,
    pub(crate) encoded_receipt: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(crate) struct ProjectInstallCursor {
    pub(crate) reclog_tail_seq: u64,
    pub(crate) next_install_commit_seq: u64,
    pub(crate) next_action_seq: u64,
    pub(crate) generation: u64,
    pub(crate) head_commit_sha256: Option<[u8; 32]>,
    pub(crate) active_install_commit_sha256: Option<[u8; 32]>,
    pub(crate) active_artifact_sha256: Option<[u8; 32]>,
}

pub(crate) fn current_project_install_cursor(
    controller: pci::PciMassStorageController,
    service_id: &str,
) -> Result<ProjectInstallCursor, &'static str> {
    let replay = replay_project_install(controller, service_id)?;
    Ok(ProjectInstallCursor {
        reclog_tail_seq: replay.reclog_tail_seq,
        next_install_commit_seq: replay
            .reclog_tail_seq
            .checked_add(2)
            .ok_or("project_install_sequence_overflow")?,
        next_action_seq: replay
            .reclog_tail_seq
            .checked_add(1)
            .ok_or("project_install_sequence_overflow")?,
        generation: replay.resolved.generation.saturating_add(1),
        head_commit_sha256: replay.resolved.head_commit_sha256,
        active_install_commit_sha256: replay.resolved.active_install_commit_sha256,
        active_artifact_sha256: replay.resolved.candidate_sha256,
    })
}

pub(crate) fn replay_project_install(
    controller: pci::PciMassStorageController,
    service_id: &str,
) -> Result<ProjectInstallReplay, &'static str> {
    let scan = artifact_store::current_boot_reclog_scan(controller);
    if !scan.read_completed || !scan.scan.full_region_valid || !scan.scan.valid_prefix_chain {
        return Err(scan.reason);
    }
    let bytes = scan
        .bytes
        .as_deref()
        .ok_or("project_install_reclog_missing")?;
    let mut commits = Vec::new();
    let mut intents = Vec::new();
    for (seq, payload) in scan_reclog_payloads(bytes) {
        if let Ok(intent) = decode_install_intent(payload) {
            intents.push((seq, intent));
            continue;
        }
        let Ok(commit) = decode_install_commit(payload) else {
            continue;
        };
        if commit.action.log_sequence != seq {
            return Err("project_install_record_sequence_mismatch");
        }
        if commit.action.authority == ProjectInstallAuthority::PhysicalOwner {
            verify_physical_action(&commit.action)?;
        }
        if commit.action.kind == ProjectInstallActionKind::Install {
            let intent_sha256 = commit
                .install_intent_sha256
                .ok_or("project_install_intent_reference_missing")?;
            let (_, intent) = intents
                .iter()
                .find(|(intent_seq, intent)| {
                    intent.intent_sha256 == intent_sha256 && intent_seq.saturating_add(1) == seq
                })
                .ok_or("project_install_intent_not_observed")?;
            if intent.action != commit.action || Some(&intent.envelope) != commit.envelope.as_ref()
            {
                return Err("project_install_intent_commit_mismatch");
            }
        }
        if commit.action.service_id == service_id {
            commits.push(commit);
        }
    }
    let resolved = resolve_install_log(&commits).map_err(|error| error.reason())?;
    Ok(ProjectInstallReplay {
        commits,
        resolved,
        reclog_tail_seq: scan.scan.tail_seq,
    })
}

pub(crate) fn persist_physical_install(
    controller: pci::PciMassStorageController,
    intent: &ProjectInstallIntent,
    candidate: &[u8],
    encoded_receipt: &[u8],
    binding: &WorkspaceRunBinding,
) -> Result<ProjectInstallCommit, &'static str> {
    validate_install_intent(intent).map_err(|error| error.reason())?;
    verify_physical_action(&intent.action)?;
    verify_install_inputs(&intent.envelope, candidate, encoded_receipt, binding)?;

    let before = replay_project_install(controller, &intent.action.service_id)?;
    if intent.action.log_sequence != before.reclog_tail_seq.saturating_add(2)
        || intent.action.previous_commit_sha256 != before.resolved.head_commit_sha256
        || intent.action.generation != before.resolved.generation.saturating_add(1)
        || intent.envelope.previous_install_commit_sha256
            != before.resolved.active_install_commit_sha256
        || intent.envelope.previous_artifact_sha256 != before.resolved.candidate_sha256
    {
        return Err("project_install_preview_stale");
    }

    let encoded_intent = encode_install_intent(intent).map_err(|error| error.reason())?;
    let intent_append = append_reclog_payload(controller, &encoded_intent)?;
    if intent_append.seq.saturating_add(1) != intent.action.log_sequence {
        return Err("project_install_intent_sequence_mismatch");
    }

    let next_free = next_free_artstor_offset(controller)?;
    let candidate_blob = write_blob(controller, next_free, candidate)?;
    let receipt_offset = candidate_blob
        .offset
        .checked_add(candidate_blob.frame_len)
        .ok_or("project_install_artstor_offset_overflow")?;
    let receipt_blob = write_blob(controller, receipt_offset, encoded_receipt)?;

    let commit = seal_install_commit(ProjectInstallCommit {
        action: intent.action.clone(),
        install_intent_sha256: Some(intent.intent_sha256),
        envelope: Some(intent.envelope.clone()),
        candidate_blob: Some(candidate_blob),
        receipt_blob: Some(receipt_blob),
        commit_sha256: [0; 32],
    })
    .map_err(|error| error.reason())?;
    let encoded = encode_install_commit(&commit).map_err(|error| error.reason())?;
    let final_append = append_reclog_payload(controller, &encoded)?;
    if final_append.seq != commit.action.log_sequence {
        return Err("project_install_commit_sequence_mismatch");
    }
    let replay = replay_project_install(controller, &commit.action.service_id)?;
    if replay.resolved.head_commit_sha256 != Some(commit.commit_sha256) {
        return Err("project_install_commit_replay_mismatch");
    }
    Ok(commit)
}

pub(crate) fn append_physical_uninstall(
    controller: pci::PciMassStorageController,
    action: ProjectInstallAction,
) -> Result<ProjectInstallCommit, &'static str> {
    if action.kind != ProjectInstallActionKind::Uninstall {
        return Err("project_install_action_kind_denied");
    }
    verify_physical_action(&action)?;
    append_non_install_action(controller, action)
}

fn append_runtime_install_action(
    controller: pci::PciMassStorageController,
    action: ProjectInstallAction,
) -> Result<ProjectInstallCommit, &'static str> {
    if !matches!(
        (action.kind, action.authority),
        (
            ProjectInstallActionKind::ActivationAttempt
                | ProjectInstallActionKind::ActivationSuccess,
            ProjectInstallAuthority::RuntimeHealth
        ) | (
            ProjectInstallActionKind::Rollback,
            ProjectInstallAuthority::RecoveryFailure
        )
    ) {
        return Err("project_install_runtime_action_denied");
    }
    append_non_install_action(controller, action)
}

pub(crate) fn append_activation_attempt(
    controller: pci::PciMassStorageController,
    replay: &ProjectInstallReplay,
    runtime_health_evidence_sha256: [u8; 32],
) -> Result<ProjectInstallCommit, &'static str> {
    let target = replay
        .resolved
        .probation_install_commit_sha256
        .ok_or("project_install_probation_missing")?;
    append_runtime_marker(
        controller,
        replay,
        ProjectInstallActionKind::ActivationAttempt,
        ProjectInstallAuthority::RuntimeHealth,
        Some(target),
        runtime_health_evidence_sha256,
    )
}

pub(crate) fn append_activation_success(
    controller: pci::PciMassStorageController,
    replay: &ProjectInstallReplay,
    runtime_health_evidence_sha256: [u8; 32],
) -> Result<ProjectInstallCommit, &'static str> {
    let target = replay
        .resolved
        .probation_install_commit_sha256
        .ok_or("project_install_probation_missing")?;
    append_runtime_marker(
        controller,
        replay,
        ProjectInstallActionKind::ActivationSuccess,
        ProjectInstallAuthority::RuntimeHealth,
        Some(target),
        runtime_health_evidence_sha256,
    )
}

pub(crate) fn append_recovery_failure_rollback(
    controller: pci::PciMassStorageController,
    replay: &ProjectInstallReplay,
    recovery_evidence_sha256: [u8; 32],
) -> Result<ProjectInstallCommit, &'static str> {
    append_runtime_marker(
        controller,
        replay,
        ProjectInstallActionKind::Rollback,
        ProjectInstallAuthority::RecoveryFailure,
        replay.resolved.last_good_install_commit_sha256,
        recovery_evidence_sha256,
    )
}

fn append_runtime_marker(
    controller: pci::PciMassStorageController,
    replay: &ProjectInstallReplay,
    kind: ProjectInstallActionKind,
    authority: ProjectInstallAuthority,
    target_install_commit_sha256: Option<[u8; 32]>,
    authority_evidence_sha256: [u8; 32],
) -> Result<ProjectInstallCommit, &'static str> {
    if authority_evidence_sha256 == [0; 32] {
        return Err("project_install_runtime_evidence_missing");
    }
    let service_id = replay
        .resolved
        .service_id
        .as_ref()
        .ok_or("project_install_service_missing")?;
    let action = seal_install_action(ProjectInstallAction {
        kind,
        authority,
        service_id: String::from(service_id),
        generation: replay.resolved.generation,
        log_sequence: replay
            .reclog_tail_seq
            .checked_add(1)
            .ok_or("project_install_sequence_overflow")?,
        previous_commit_sha256: replay.resolved.head_commit_sha256,
        install_envelope_sha256: None,
        target_install_commit_sha256,
        authority_evidence_sha256,
        physical_approval_sha256: None,
        authority_key_sha256: None,
        authority_signature: Vec::new(),
        action_sha256: [0; 32],
    })
    .map_err(|error| error.reason())?;
    append_runtime_install_action(controller, action)
}

fn append_non_install_action(
    controller: pci::PciMassStorageController,
    action: ProjectInstallAction,
) -> Result<ProjectInstallCommit, &'static str> {
    validate_install_action(&action).map_err(|error| error.reason())?;
    let before = replay_project_install(controller, &action.service_id)?;
    if action.log_sequence != before.reclog_tail_seq.saturating_add(1)
        || action.previous_commit_sha256 != before.resolved.head_commit_sha256
        || action.generation != before.resolved.generation
    {
        return Err("project_install_action_preview_stale");
    }
    let commit = seal_install_commit(ProjectInstallCommit {
        action,
        install_intent_sha256: None,
        envelope: None,
        candidate_blob: None,
        receipt_blob: None,
        commit_sha256: [0; 32],
    })
    .map_err(|error| error.reason())?;
    let mut projected = before.commits;
    projected.push(commit.clone());
    resolve_install_log(&projected).map_err(|error| error.reason())?;
    let encoded = encode_install_commit(&commit).map_err(|error| error.reason())?;
    let appended = append_reclog_payload(controller, &encoded)?;
    if appended.seq != commit.action.log_sequence {
        return Err("project_install_commit_sequence_mismatch");
    }
    Ok(commit)
}

pub(crate) fn read_project_install_payloads(
    controller: pci::PciMassStorageController,
    replay: &ProjectInstallReplay,
    install_commit_sha256: [u8; 32],
) -> Result<InstalledProjectPayloads, &'static str> {
    let install = replay
        .commits
        .iter()
        .find(|commit| {
            commit.commit_sha256 == install_commit_sha256
                && commit.action.kind == ProjectInstallActionKind::Install
        })
        .ok_or("project_install_target_missing")?;
    let envelope = install
        .envelope
        .clone()
        .ok_or("project_install_envelope_missing")?;
    let candidate_blob = install
        .candidate_blob
        .ok_or("project_install_candidate_blob_missing")?;
    let receipt_blob = install
        .receipt_blob
        .ok_or("project_install_receipt_blob_missing")?;
    let candidate = read_blob(controller, candidate_blob)?;
    let encoded_receipt = read_blob(controller, receipt_blob)?;
    let receipt =
        decode_receipt(&encoded_receipt).map_err(|_| "project_install_receipt_invalid")?;
    validate_build_receipt(&receipt).map_err(|_| "project_install_receipt_invalid")?;
    if receipt.receipt_sha256 != envelope.build_receipt_sha256
        || receipt.candidate_sha256 != envelope.candidate_sha256
        || receipt.candidate_byte_len as u64 != envelope.candidate_byte_len
        || sha256_bytes(&candidate) != envelope.candidate_sha256
    {
        return Err("project_install_payload_binding_mismatch");
    }
    Ok(InstalledProjectPayloads {
        install_commit_sha256,
        envelope,
        candidate_blob,
        receipt_blob,
        candidate,
        encoded_receipt,
    })
}

fn verify_physical_action(action: &ProjectInstallAction) -> Result<(), &'static str> {
    validate_install_action(action).map_err(|error| error.reason())?;
    if action.authority != ProjectInstallAuthority::PhysicalOwner
        || action.authority_key_sha256 != Some(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256)
    {
        return Err("project_install_physical_authority_invalid");
    }
    let digest = install_action_signature_payload_sha256(action).map_err(|error| error.reason())?;
    if !verify_promotion_authority_signature(&action.authority_signature, &digest) {
        return Err("project_install_physical_signature_invalid");
    }
    Ok(())
}

fn verify_install_inputs(
    envelope: &ProjectInstallEnvelope,
    candidate: &[u8],
    encoded_receipt: &[u8],
    binding: &WorkspaceRunBinding,
) -> Result<(), &'static str> {
    validate_install_envelope(envelope).map_err(|error| error.reason())?;
    if candidate.len() as u64 != envelope.candidate_byte_len
        || sha256_bytes(candidate) != envelope.candidate_sha256
        || sha256_bytes(encoded_receipt) != envelope.encoded_receipt_sha256
    {
        return Err("project_install_input_hash_mismatch");
    }
    let receipt = decode_receipt(encoded_receipt).map_err(|_| "project_install_receipt_invalid")?;
    validate_build_receipt(&receipt).map_err(|_| "project_install_receipt_invalid")?;
    if receipt.receipt_sha256 != envelope.build_receipt_sha256
        || receipt.candidate_sha256 != envelope.candidate_sha256
        || receipt.candidate_byte_len as u64 != envelope.candidate_byte_len
        || binding.approval_challenge_sha256 != envelope.workspace_run_binding_sha256
        || binding.project_id != envelope.project_id
        || binding.project_revision_sha256 != envelope.project_revision_sha256
        || binding.input_manifest_sha256 != envelope.input_manifest_sha256
        || binding.receipt_sha256 != envelope.build_receipt_sha256
        || binding.candidate_sha256 != envelope.candidate_sha256
        || binding.candidate_byte_len as u64 != envelope.candidate_byte_len
        || binding.import_list_sha256 != envelope.import_list_sha256
        || binding.import_count as u32 != envelope.import_count
        || binding.fuel_budget != envelope.fuel_budget
        || binding.memory_limit_bytes as u64 != envelope.memory_limit_bytes
        || binding.instance_limit as u32 != envelope.instance_limit
        || binding.memory_count_limit as u32 != envelope.memory_count_limit
        || binding.table_limit as u32 != envelope.table_limit
    {
        return Err("project_install_workspace_binding_mismatch");
    }
    Ok(())
}

fn append_reclog_payload(
    controller: pci::PciMassStorageController,
    payload: &[u8],
) -> Result<PlannedAppend, &'static str> {
    let before = artifact_store::current_boot_reclog_scan(controller);
    if !before.read_completed || !before.scan.full_region_valid || !before.scan.valid_prefix_chain {
        return Err(before.reason);
    }
    let planned = plan_reclog_append(&before.scan, payload, before.reclog_byte_count)
        .map_err(|denied| denied.reason())?;
    let write = unsafe {
        ahci::write_readback_reclog_append(controller, planned.write_offset, &planned.frame)
    };
    let readback = write.readback.as_deref().ok_or(write.reason)?;
    if !write.write_completed
        || !write.readback_completed
        || !write.span_in_bounds
        || sha256_bytes(readback) != planned.frame_sha256
        || parse_reclog_frame(readback, 0, planned.seq, planned.prev_frame_sha256).is_err()
    {
        return Err("project_install_reclog_readback_invalid");
    }
    let after = artifact_store::current_boot_reclog_scan(controller);
    if after.scan.tail_seq != planned.seq
        || after.scan.tail_frame_sha256 != Some(planned.frame_sha256)
        || after.scan.count != before.scan.count.saturating_add(1)
    {
        return Err("project_install_reclog_rescan_mismatch");
    }
    Ok(planned)
}

fn next_free_artstor_offset(
    controller: pci::PciMassStorageController,
) -> Result<u64, &'static str> {
    artifact_store::next_free_artstor_offset_on_disk(controller)
}

fn write_blob(
    controller: pci::PciMassStorageController,
    offset: u64,
    payload: &[u8],
) -> Result<ContentBlobRef, &'static str> {
    let region = ahci::read_persist_artstor_region(controller, 0, 512);
    if !region.region_bounds_valid || region.byte_count == 0 {
        return Err(region.reason);
    }
    let planned = plan_artifact_blob_write(offset, payload, region.byte_count)
        .map_err(|denied| denied.reason())?;
    let write = unsafe {
        ahci::write_readback_artstor_blob(controller, planned.write_offset, &planned.frame)
    };
    let readback = write.readback.as_deref().ok_or(write.reason)?;
    let parsed = parse_artifact_blob_frame(readback, 0)
        .map_err(|_| "project_install_blob_reparse_failed")?;
    if !write.write_completed
        || !write.readback_completed
        || !write.span_in_bounds
        || sha256_bytes(readback) != planned.frame_sha256
        || parsed.frame_sha256 != planned.frame_sha256
        || parsed.payload_sha256 != planned.payload_sha256
    {
        return Err("project_install_blob_readback_invalid");
    }
    Ok(ContentBlobRef {
        payload_sha256: planned.payload_sha256,
        frame_sha256: planned.frame_sha256,
        offset: planned.write_offset,
        frame_len: planned.frame_len,
    })
}

fn read_blob(
    controller: pci::PciMassStorageController,
    blob: ContentBlobRef,
) -> Result<Vec<u8>, &'static str> {
    let read = ahci::read_persist_artstor_region(controller, blob.offset, blob.frame_len);
    let bytes = read.bytes.ok_or(read.reason)?;
    if sha256_bytes(&bytes) != blob.frame_sha256 {
        return Err("project_install_blob_frame_hash_mismatch");
    }
    let parsed =
        parse_artifact_blob_frame(&bytes, 0).map_err(|_| "project_install_blob_reparse_failed")?;
    if parsed.frame_sha256 != blob.frame_sha256
        || parsed.payload_sha256 != blob.payload_sha256
        || parsed.frame_len as u64 != blob.frame_len
    {
        return Err("project_install_blob_reference_mismatch");
    }
    let end = ARTIFACT_BLOB_FRAME_HEADER_LEN
        .checked_add(parsed.payload_len as usize)
        .ok_or("project_install_blob_payload_overflow")?;
    if end > bytes.len() {
        return Err("project_install_blob_payload_out_of_bounds");
    }
    Ok(bytes[ARTIFACT_BLOB_FRAME_HEADER_LEN..end].to_vec())
}
