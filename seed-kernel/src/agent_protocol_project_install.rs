use alloc::{string::String, vec, vec::Vec};

use raios_core::{
    project_build::encode_receipt,
    project_install::{
        install_action_signature_payload_sha256, seal_install_action, seal_install_envelope,
        seal_install_intent, seal_state_schema, seal_ui_program_install_envelope,
        validate_ui_program_install_envelope, GrantedCandidateInstallEnvelope,
        ProjectAppStateSchema, ProjectInstallAction, ProjectInstallActionKind,
        ProjectInstallAuthority, ProjectInstallEnvelope, ProjectInstallIntent,
        StateMigrationPolicy, UiProgramInstallEnvelope, UI_PROGRAM_INSTALL_SUBJECT_KIND,
        PROJECT_INSTALL_TRUST_TIER,
    },
    project_runtime::{WORKSPACE_ENTRYPOINT, WORKSPACE_SERVICE_ID},
    promotion_attestation::{
        verify_promotion_authority_signature, PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
    },
    record::{Field, Value as V},
    scoped_wasm_import_grant::PERSONAL_SHELL_SERVICE_ID,
    sha256_bytes,
    ui_program::PROGRAM_ABI_VERSION,
};
use spin::Mutex;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, parse_sha256_ref, record_bool as b,
        record_field as f, record_sha_or_null, record_str as s,
    },
    console, granted_candidate_service, module_candidate_intake, pci, program_persistence,
    program_workspace, project_app_autoload, project_build, project_install_store, serial,
    workspace_candidate_service,
};

const INSTALL_APPROVAL_DOMAIN: &[u8] = b"raios.project_install_pointer_approval.v1";
const UNINSTALL_APPROVAL_DOMAIN: &[u8] = b"raios.project_uninstall_pointer_approval.v1";
const AUTHORITY_EVIDENCE_DOMAIN: &[u8] = b"raios.project_install_authority_evidence.v1";
const MAX_SIGNATURE_DER_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreviewKind {
    Install,
    Uninstall,
}

impl PreviewKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Uninstall => "uninstall",
        }
    }
}

#[derive(Clone)]
enum PendingAction {
    Install {
        intent: ProjectInstallIntent,
        candidate: Vec<u8>,
        encoded_receipt: Vec<u8>,
        binding: raios_core::project_runtime::WorkspaceRunBinding,
    },
    GrantedCandidateInstall {
        envelope: GrantedCandidateInstallEnvelope,
        action: ProjectInstallAction,
    },
    UiProgramInstall {
        envelope: UiProgramInstallEnvelope,
        action: ProjectInstallAction,
    },
    Uninstall {
        action: ProjectInstallAction,
    },
}

impl PendingAction {
    fn kind(&self) -> PreviewKind {
        match self {
            Self::Install { .. } => PreviewKind::Install,
            Self::GrantedCandidateInstall { .. } => PreviewKind::Install,
            Self::UiProgramInstall { .. } => PreviewKind::Install,
            Self::Uninstall { .. } => PreviewKind::Uninstall,
        }
    }

    fn action(&self) -> &ProjectInstallAction {
        match self {
            Self::Install { intent, .. } => &intent.action,
            Self::GrantedCandidateInstall { action, .. } => action,
            Self::UiProgramInstall { action, .. } => action,
            Self::Uninstall { action } => action,
        }
    }

    fn candidate_sha256(&self) -> Option<[u8; 32]> {
        match self {
            Self::Install { intent, .. } => Some(intent.envelope.candidate_sha256),
            Self::GrantedCandidateInstall { envelope, .. } => Some(envelope.candidate_sha256),
            Self::UiProgramInstall { envelope, .. } => {
                Some(envelope.canonical_program_sha256)
            }
            Self::Uninstall { .. } => None,
        }
    }

    fn receipt_sha256(&self) -> Option<[u8; 32]> {
        match self {
            Self::Install { intent, .. } => Some(intent.envelope.build_receipt_sha256),
            Self::GrantedCandidateInstall { envelope, .. } => Some(envelope.w7_receipt_sha256),
            Self::UiProgramInstall { envelope, .. } => {
                Some(envelope.canonical_program_sha256)
            }
            Self::Uninstall { .. } => None,
        }
    }
}

struct State {
    pending: Option<PendingAction>,
    signature_verified: bool,
    last_reason: &'static str,
    last_commit_sha256: Option<[u8; 32]>,
    candidate_blob_offset: Option<u64>,
    candidate_blob_frame_len: Option<u64>,
}

impl State {
    const fn new() -> Self {
        Self {
            pending: None,
            signature_verified: false,
            last_reason: "project_install_not_prepared",
            last_commit_sha256: None,
            candidate_blob_offset: None,
            candidate_blob_frame_len: None,
        }
    }

    fn snapshot(&self) -> PreviewSnapshot {
        let pending = self.pending.as_ref();
        let action = pending.map(PendingAction::action);
        PreviewSnapshot {
            phase: if pending.is_none() {
                "idle"
            } else if self.signature_verified {
                "pending_physical_pointer_approval"
            } else {
                "pending_owner_signature"
            },
            kind: pending.map(PendingAction::kind),
            signature_verified: self.signature_verified,
            action_signature_message_sha256: action
                .and_then(|action| install_action_signature_payload_sha256(action).ok()),
            physical_approval_sha256: action.and_then(|action| action.physical_approval_sha256),
            generation: action.map(|action| action.generation).unwrap_or(0),
            log_sequence: action.map(|action| action.log_sequence).unwrap_or(0),
            previous_commit_sha256: action.and_then(|action| action.previous_commit_sha256),
            candidate_sha256: pending.and_then(PendingAction::candidate_sha256),
            receipt_sha256: pending.and_then(PendingAction::receipt_sha256),
            last_reason: self.last_reason,
            last_commit_sha256: self.last_commit_sha256,
            candidate_blob_offset: self.candidate_blob_offset,
            candidate_blob_frame_len: self.candidate_blob_frame_len,
            install_source: pending.and_then(|pending| match pending {
                PendingAction::GrantedCandidateInstall { .. } => Some("granted_candidate"),
                PendingAction::UiProgramInstall { .. } => Some("ui_program"),
                _ => None,
            }),
            receipt_kind: pending.and_then(|pending| match pending {
                PendingAction::GrantedCandidateInstall { .. } => Some("w7_acquisition"),
                PendingAction::UiProgramInstall { .. } => Some("ruip_canonical"),
                _ => None,
            }),
            w4_project_receipt_present: false,
            activation_approval_sha256: pending.and_then(|pending| match pending {
                PendingAction::GrantedCandidateInstall { envelope, .. } => {
                    Some(envelope.activation_approval_sha256)
                }
                PendingAction::UiProgramInstall { envelope, .. } => {
                    Some(envelope.activation_approval_sha256)
                }
                _ => None,
            }),
            install_envelope_sha256: action.and_then(|action| action.install_envelope_sha256),
            service_id: pending.map_or(WORKSPACE_SERVICE_ID, |pending| match pending {
                PendingAction::UiProgramInstall { .. } => PERSONAL_SHELL_SERVICE_ID,
                _ => WORKSPACE_SERVICE_ID,
            }),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PreviewSnapshot {
    pub(crate) phase: &'static str,
    pub(crate) kind: Option<PreviewKind>,
    pub(crate) signature_verified: bool,
    pub(crate) action_signature_message_sha256: Option<[u8; 32]>,
    pub(crate) physical_approval_sha256: Option<[u8; 32]>,
    pub(crate) generation: u64,
    pub(crate) log_sequence: u64,
    pub(crate) previous_commit_sha256: Option<[u8; 32]>,
    pub(crate) candidate_sha256: Option<[u8; 32]>,
    pub(crate) receipt_sha256: Option<[u8; 32]>,
    pub(crate) last_reason: &'static str,
    pub(crate) last_commit_sha256: Option<[u8; 32]>,
    pub(crate) candidate_blob_offset: Option<u64>,
    pub(crate) candidate_blob_frame_len: Option<u64>,
    pub(crate) install_source: Option<&'static str>,
    pub(crate) receipt_kind: Option<&'static str>,
    pub(crate) w4_project_receipt_present: bool,
    pub(crate) activation_approval_sha256: Option<[u8; 32]>,
    pub(crate) install_envelope_sha256: Option<[u8; 32]>,
    pub(crate) service_id: &'static str,
}

static STATE: Mutex<State> = Mutex::new(State::new());

pub(crate) fn snapshot() -> PreviewSnapshot {
    STATE.lock().snapshot()
}

pub(crate) fn pending_physical_approval() -> bool {
    let state = STATE.lock();
    state.signature_verified && state.pending.is_some()
}

pub(crate) fn emit_install_prepare(input: &str) {
    let outcome = prepare_install(input);
    emit("project.install_prepare", outcome.0, outcome.1);
}

pub(crate) fn emit_install_signature(input: &str) {
    let outcome = accept_signature(input, "project.install_signature", PreviewKind::Install);
    emit("project.install_signature", outcome.0, outcome.1);
}

pub(crate) fn emit_install_status() {
    emit(
        "project.install_status",
        true,
        "project_install_status_read",
    );
}

pub(crate) fn emit_install_serial_approval_denied(_input: &str) {
    emit(
        "project.install_approve",
        false,
        "project_install_physical_pointer_approval_required",
    );
}

pub(crate) fn emit_uninstall_prepare(input: &str) {
    let outcome = prepare_uninstall(input);
    emit("project.uninstall_prepare", outcome.0, outcome.1);
}

pub(crate) fn emit_uninstall_signature(input: &str) {
    let outcome = accept_signature(input, "project.uninstall_signature", PreviewKind::Uninstall);
    emit("project.uninstall_signature", outcome.0, outcome.1);
}

pub(crate) fn emit_uninstall_serial_approval_denied(_input: &str) {
    emit(
        "project.uninstall_approve",
        false,
        "project_uninstall_physical_pointer_approval_required",
    );
}

pub(crate) fn emit_rollback_status() {
    let Some(controller) = pci::find_mass_storage_controller() else {
        emit(
            "project.rollback_status",
            false,
            "ahci_controller_not_observed",
        );
        return;
    };
    match project_install_store::replay_project_install(controller, WORKSPACE_SERVICE_ID) {
        Ok(_) => emit(
            "project.rollback_status",
            true,
            "project_rollback_status_read",
        ),
        Err(reason) => emit("project.rollback_status", false, reason),
    }
}

/// The only UI entry point that can cross from a signed preview into a durable
/// install/uninstall write. Storage replays the cursor and verifies the same
/// signature message again immediately before its first media write.
pub(crate) fn approve_from_pointer() -> bool {
    let pending = {
        let state = STATE.lock();
        if !state.signature_verified {
            return false;
        }
        let Some(pending) = state.pending.clone() else {
            return false;
        };
        pending
    };
    if let PendingAction::UiProgramInstall { envelope, action } = &pending {
        match validate_current_ui_program_install_preview(envelope, action)
            .and_then(|_| program_persistence::install_approved_from_pointer(envelope, action))
        {
            Ok(()) => {
                let mut state = STATE.lock();
                state.pending = None;
                state.signature_verified = false;
                state.last_reason = "program_installed";
                return true;
            }
            Err(reason) => {
                finish_ui_program_denied(reason);
                return true;
            }
        }
    }
    let Some(controller) = pci::find_mass_storage_controller() else {
        finish_denied("ahci_controller_not_observed");
        return true;
    };
    if let PendingAction::GrantedCandidateInstall { envelope, action } = &pending {
        match validate_current_granted_install_preview(&envelope, &action)
            .and_then(|_| {
                granted_candidate_service::install_approved_from_pointer(&envelope, &action)
            })
        {
            Ok(()) => {
                let mut state = STATE.lock();
                state.pending = None;
                state.signature_verified = false;
                state.last_reason = "granted_candidate_installed";
                return true;
            }
            Err(reason) => {
                finish_denied(reason);
                return true;
            }
        }
    }

    let result = match pending {
        PendingAction::Install {
            intent,
            candidate,
            encoded_receipt,
            binding,
        } => validate_current_install_preview(&binding, &candidate, &encoded_receipt).and_then(
            |_| {
                project_install_store::persist_physical_install(
                    controller,
                    &intent,
                    &candidate,
                    &encoded_receipt,
                    &binding,
                )
                .map(|commit| (PreviewKind::Install, commit))
            },
        ),
        PendingAction::Uninstall { action } => {
            project_install_store::append_physical_uninstall(controller, action)
                .map(|commit| (PreviewKind::Uninstall, commit))
        }
        PendingAction::GrantedCandidateInstall { .. } => {
            // Handled by the early granted-candidate branch above; reaching this
            // arm means the dispatch order changed, so fail closed.
            finish_denied("granted_candidate_install_dispatch_error");
            return true;
        }
        PendingAction::UiProgramInstall { .. } => {
            finish_ui_program_denied("ui_program_install_dispatch_error");
            return true;
        }
    };

    match result {
        Ok((kind, commit)) => {
            if kind == PreviewKind::Uninstall {
                if let Err(reason) =
                    project_app_autoload::note_physical_uninstall_tombstone(&commit)
                {
                    finish_denied(reason);
                    return true;
                }
            }
            let candidate_blob = commit.candidate_blob;
            {
                let mut state = STATE.lock();
                state.pending = None;
                state.signature_verified = false;
                state.last_reason = match kind {
                    PreviewKind::Install => "project_install_persisted",
                    PreviewKind::Uninstall => "project_uninstall_tombstoned",
                };
                state.last_commit_sha256 = Some(commit.commit_sha256);
                state.candidate_blob_offset = candidate_blob.map(|blob| blob.offset);
                state.candidate_blob_frame_len = candidate_blob.map(|blob| blob.frame_len);
            }
            if kind == PreviewKind::Uninstall {
                let _ = workspace_candidate_service::drop_service("project_install_uninstalled");
            }
            emit_commit_marker(kind, &commit);
            console::write_event(format_args!(
                "Project {} approved and persisted",
                kind.label()
            ));
        }
        Err(reason) => finish_denied(reason),
    }
    true
}

fn prepare_install(input: &str) -> (bool, &'static str) {
    let binding_hash = match one_hash_arg(input, "project.install_prepare") {
        Ok(hash) => hash,
        Err(reason) => return (false, reason),
    };
    if granted_candidate_service::approved_install_envelope()
        .map(|envelope| envelope.activation_approval_sha256 == binding_hash)
        .unwrap_or(false)
    {
        return prepare_granted_candidate_install(binding_hash);
    }
    if program_workspace::approved_program_install()
        .map(|approved| approved.activation_approval_sha256 == binding_hash)
        .unwrap_or(false)
    {
        return prepare_ui_program_install(binding_hash);
    }
    let runtime = workspace_candidate_service::snapshot();
    let Some(binding) = runtime.binding else {
        return (false, "project_install_workspace_binding_missing");
    };
    if runtime.phase != "running"
        || runtime.last_run_outcome != "success"
        || runtime.last_return_value.is_none()
    {
        return (false, "project_install_workspace_run_not_healthy");
    }
    if binding.approval_challenge_sha256 != binding_hash {
        return (false, "project_install_workspace_binding_mismatch");
    }
    let Some(candidate) = module_candidate_intake::retained() else {
        return (false, "project_install_candidate_missing");
    };
    if !candidate.wasm_valid || candidate.sha256 != binding.candidate_sha256 {
        return (false, "project_install_candidate_mismatch");
    }
    let receipt = match project_build::verified_receipt_exact(
        binding.project_id,
        binding.project_revision_sha256,
        binding.receipt_sha256,
    ) {
        Ok(Some(receipt)) => receipt,
        Ok(None) => return (false, "project_install_receipt_missing"),
        Err(reason) => return (false, reason),
    };
    let encoded_receipt = match encode_receipt(&receipt) {
        Ok(bytes) => bytes,
        Err(_) => return (false, "project_install_receipt_invalid"),
    };
    let Some(controller) = pci::find_mass_storage_controller() else {
        return (false, "ahci_controller_not_observed");
    };
    let cursor = match project_install_store::current_project_install_cursor(
        controller,
        WORKSPACE_SERVICE_ID,
    ) {
        Ok(cursor) => cursor,
        Err(reason) => return (false, reason),
    };
    let state_schema = match seal_state_schema(ProjectAppStateSchema {
        schema_id: String::from("raios.app_state.stateless.v1"),
        version: 1,
        max_state_bytes: 0,
        migration_policy: StateMigrationPolicy::Denied,
        schema_sha256: [0; 32],
    }) {
        Ok(schema) => schema,
        Err(error) => return (false, error.reason()),
    };
    let envelope = match seal_install_envelope(ProjectInstallEnvelope {
        service_id: String::from(WORKSPACE_SERVICE_ID),
        project_id: binding.project_id,
        project_revision_sha256: binding.project_revision_sha256,
        input_manifest_sha256: binding.input_manifest_sha256,
        build_receipt_sha256: binding.receipt_sha256,
        encoded_receipt_sha256: sha256_bytes(&encoded_receipt),
        workspace_run_binding_sha256: binding.approval_challenge_sha256,
        candidate_sha256: binding.candidate_sha256,
        candidate_byte_len: binding.candidate_byte_len as u64,
        import_list_sha256: binding.import_list_sha256,
        import_count: binding.import_count as u32,
        entrypoint: String::from(WORKSPACE_ENTRYPOINT),
        fuel_budget: binding.fuel_budget,
        memory_limit_bytes: binding.memory_limit_bytes as u64,
        instance_limit: binding.instance_limit as u32,
        memory_count_limit: binding.memory_count_limit as u32,
        table_limit: binding.table_limit as u32,
        state_schema,
        previous_install_commit_sha256: cursor.active_install_commit_sha256,
        previous_artifact_sha256: cursor.active_artifact_sha256,
        generation: cursor.generation,
        auto_start: true,
        trust_tier: String::from(PROJECT_INSTALL_TRUST_TIER),
        envelope_sha256: [0; 32],
    }) {
        Ok(envelope) => envelope,
        Err(error) => return (false, error.reason()),
    };
    let physical_approval_sha256 = approval_hash(
        INSTALL_APPROVAL_DOMAIN,
        cursor.generation,
        cursor.next_install_commit_seq,
        envelope.envelope_sha256,
        cursor.head_commit_sha256,
    );
    let authority_evidence_sha256 = authority_evidence_hash(
        PreviewKind::Install,
        binding.approval_challenge_sha256,
        envelope.envelope_sha256,
    );
    let action = ProjectInstallAction {
        kind: ProjectInstallActionKind::Install,
        authority: ProjectInstallAuthority::PhysicalOwner,
        service_id: String::from(WORKSPACE_SERVICE_ID),
        generation: cursor.generation,
        log_sequence: cursor.next_install_commit_seq,
        previous_commit_sha256: cursor.head_commit_sha256,
        install_envelope_sha256: Some(envelope.envelope_sha256),
        target_install_commit_sha256: None,
        authority_evidence_sha256,
        physical_approval_sha256: Some(physical_approval_sha256),
        authority_key_sha256: Some(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        authority_signature: Vec::new(),
        action_sha256: [0; 32],
    };
    if install_action_signature_payload_sha256(&action).is_err() {
        return (false, "project_install_action_shape_invalid");
    }
    let intent = ProjectInstallIntent {
        action,
        envelope,
        intent_sha256: [0; 32],
    };
    let mut state = STATE.lock();
    state.pending = Some(PendingAction::Install {
        intent,
        candidate: candidate.bytes,
        encoded_receipt,
        binding,
    });
    state.signature_verified = false;
    state.last_reason = "project_install_signature_required";
    state.last_commit_sha256 = None;
    state.candidate_blob_offset = None;
    state.candidate_blob_frame_len = None;
    drop(state);
    emit_preview_marker(PreviewKind::Install, false);
    (true, "project_install_signature_required")
}

fn prepare_granted_candidate_install(binding_hash: [u8; 32]) -> (bool, &'static str) {
    let envelope = match granted_candidate_service::approved_install_envelope() {
        Ok(envelope) if envelope.activation_approval_sha256 == binding_hash => envelope,
        Ok(_) => return (false, "granted_candidate_install_binding_stale"),
        Err(reason) => return (false, reason),
    };
    let Some(controller) = pci::find_mass_storage_controller() else { return (false, "ahci_controller_not_observed"); };
    let cursor = match current_granted_install_cursor(controller) { Ok(cursor) => cursor, Err(reason) => return (false, reason) };
    let physical_approval_sha256 = approval_hash(INSTALL_APPROVAL_DOMAIN, envelope.generation, cursor.next_install_commit_seq, envelope.envelope_sha256, cursor.head_commit_sha256);
    let action = ProjectInstallAction {
        kind: ProjectInstallActionKind::Install,
        authority: ProjectInstallAuthority::PhysicalOwner,
        service_id: String::from("svc.dev.granted_candidate"),
        generation: envelope.generation,
        log_sequence: cursor.next_install_commit_seq,
        previous_commit_sha256: cursor.head_commit_sha256,
        install_envelope_sha256: Some(envelope.envelope_sha256),
        target_install_commit_sha256: None,
        authority_evidence_sha256: authority_evidence_hash(PreviewKind::Install, binding_hash, envelope.envelope_sha256),
        physical_approval_sha256: Some(physical_approval_sha256),
        authority_key_sha256: Some(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        authority_signature: Vec::new(),
        action_sha256: [0; 32],
    };
    if install_action_signature_payload_sha256(&action).is_err() { return (false, "project_install_action_shape_invalid"); }
    let mut state = STATE.lock();
    state.pending = Some(PendingAction::GrantedCandidateInstall { envelope, action });
    state.signature_verified = false;
    state.last_reason = "project_install_signature_required";
    state.last_commit_sha256 = None;
    state.candidate_blob_offset = None;
    state.candidate_blob_frame_len = None;
    drop(state);
    emit_preview_marker(PreviewKind::Install, false);
    (true, "project_install_signature_required")
}

fn prepare_ui_program_install(binding_hash: [u8; 32]) -> (bool, &'static str) {
    let approved = match program_workspace::approved_program_install() {
        Ok(approved) if approved.activation_approval_sha256 == binding_hash => approved,
        Ok(_) => return (false, "ui_program_install_binding_stale"),
        Err(reason) => return (false, reason),
    };
    let Some(controller) = pci::find_mass_storage_controller() else {
        return (false, "ahci_controller_not_observed");
    };
    let cursor = match project_install_store::current_project_install_cursor(
        controller,
        PERSONAL_SHELL_SERVICE_ID,
    ) {
        Ok(cursor) => cursor,
        Err(reason) => return (false, reason),
    };
    let envelope = match seal_ui_program_install_envelope(UiProgramInstallEnvelope {
        subject_kind: String::from(UI_PROGRAM_INSTALL_SUBJECT_KIND),
        engine_service_id: String::from(PERSONAL_SHELL_SERVICE_ID),
        program_abi_version: PROGRAM_ABI_VERSION,
        canonical_program_sha256: approved.canonical_program_sha256,
        canonical_program_byte_len: approved.canonical_program_byte_len,
        activation_approval_sha256: approved.activation_approval_sha256,
        generation: cursor.generation,
        auto_load: true,
        trust_tier: String::from(PROJECT_INSTALL_TRUST_TIER),
        envelope_sha256: [0; 32],
    }) {
        Ok(envelope) => envelope,
        Err(error) => return (false, error.reason()),
    };
    let physical_approval_sha256 = approval_hash(
        INSTALL_APPROVAL_DOMAIN,
        envelope.generation,
        cursor.next_install_commit_seq,
        envelope.envelope_sha256,
        cursor.head_commit_sha256,
    );
    let action = ProjectInstallAction {
        kind: ProjectInstallActionKind::Install,
        authority: ProjectInstallAuthority::PhysicalOwner,
        service_id: String::from(PERSONAL_SHELL_SERVICE_ID),
        generation: envelope.generation,
        log_sequence: cursor.next_install_commit_seq,
        previous_commit_sha256: cursor.head_commit_sha256,
        install_envelope_sha256: Some(envelope.envelope_sha256),
        target_install_commit_sha256: None,
        authority_evidence_sha256: authority_evidence_hash(
            PreviewKind::Install,
            binding_hash,
            envelope.envelope_sha256,
        ),
        physical_approval_sha256: Some(physical_approval_sha256),
        authority_key_sha256: Some(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        authority_signature: Vec::new(),
        action_sha256: [0; 32],
    };
    if install_action_signature_payload_sha256(&action).is_err() {
        return (false, "project_install_action_shape_invalid");
    }
    let mut state = STATE.lock();
    state.pending = Some(PendingAction::UiProgramInstall { envelope, action });
    state.signature_verified = false;
    state.last_reason = "project_install_signature_required";
    state.last_commit_sha256 = None;
    state.candidate_blob_offset = None;
    state.candidate_blob_frame_len = None;
    drop(state);
    emit_preview_marker(PreviewKind::Install, false);
    (true, "project_install_signature_required")
}

fn current_granted_install_cursor(controller: pci::PciMassStorageController) -> Result<project_install_store::ProjectInstallCursor, &'static str> {
    project_install_store::current_project_install_cursor(controller, WORKSPACE_SERVICE_ID)
}

fn validate_current_granted_install_preview(
    envelope: &GrantedCandidateInstallEnvelope,
    action: &ProjectInstallAction,
) -> Result<(), &'static str> {
    if !STATE.lock().signature_verified || action.authority_signature.is_empty() {
        return Err("project_install_physical_signature_invalid");
    }
    if !verify_promotion_authority_signature(&action.authority_signature, &install_action_signature_payload_sha256(action).map_err(|error| error.reason())?) {
        return Err("project_install_physical_signature_invalid");
    }
    granted_candidate_service::validate_approved_install_envelope(envelope)
}

fn validate_current_ui_program_install_preview(
    envelope: &UiProgramInstallEnvelope,
    action: &ProjectInstallAction,
) -> Result<(), &'static str> {
    if !STATE.lock().signature_verified || action.authority_signature.is_empty() {
        return Err("project_install_physical_signature_invalid");
    }
    validate_ui_program_install_envelope(envelope).map_err(|error| error.reason())?;
    let approved = program_workspace::approved_program_install()?;
    if approved.canonical_program_sha256 != envelope.canonical_program_sha256
        || approved.canonical_program_byte_len != envelope.canonical_program_byte_len
        || approved.activation_approval_sha256 != envelope.activation_approval_sha256
    {
        return Err("ui_program_install_binding_stale");
    }
    let message = install_action_signature_payload_sha256(action)
        .map_err(|error| error.reason())?;
    if !verify_promotion_authority_signature(&action.authority_signature, &message) {
        return Err("project_install_physical_signature_invalid");
    }
    Ok(())
}


fn validate_current_install_preview(
    binding: &raios_core::project_runtime::WorkspaceRunBinding,
    candidate: &[u8],
    encoded_receipt: &[u8],
) -> Result<(), &'static str> {
    let runtime = workspace_candidate_service::snapshot();
    if runtime.phase != "running" || runtime.binding.as_ref() != Some(binding) {
        return Err("project_install_workspace_binding_stale");
    }
    let retained =
        module_candidate_intake::retained().ok_or("project_install_candidate_missing")?;
    if !retained.wasm_valid
        || retained.sha256 != binding.candidate_sha256
        || retained.bytes != candidate
    {
        return Err("project_install_candidate_mismatch");
    }
    let receipt = project_build::verified_receipt_exact(
        binding.project_id,
        binding.project_revision_sha256,
        binding.receipt_sha256,
    )?
    .ok_or("project_install_receipt_missing")?;
    let current = encode_receipt(&receipt).map_err(|_| "project_install_receipt_invalid")?;
    if current != encoded_receipt {
        return Err("project_install_receipt_stale");
    }
    Ok(())
}

fn prepare_uninstall(input: &str) -> (bool, &'static str) {
    if input
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .as_slice()
        != ["project.uninstall_prepare", WORKSPACE_SERVICE_ID]
    {
        return (false, "project_uninstall_argument_invalid");
    }
    let Some(controller) = pci::find_mass_storage_controller() else {
        return (false, "ahci_controller_not_observed");
    };
    let replay =
        match project_install_store::replay_project_install(controller, WORKSPACE_SERVICE_ID) {
            Ok(replay) => replay,
            Err(reason) => return (false, reason),
        };
    if !replay.resolved.installed || replay.resolved.head_commit_sha256.is_none() {
        return (false, "project_uninstall_not_installed");
    }
    let cursor = match project_install_store::current_project_install_cursor(
        controller,
        WORKSPACE_SERVICE_ID,
    ) {
        Ok(cursor) => cursor,
        Err(reason) => return (false, reason),
    };
    let generation = cursor.generation.saturating_sub(1);
    let active = cursor.active_artifact_sha256.unwrap_or([0; 32]);
    let head = cursor.head_commit_sha256.unwrap_or([0; 32]);
    let physical_approval_sha256 = approval_hash(
        UNINSTALL_APPROVAL_DOMAIN,
        generation,
        cursor.next_action_seq,
        active,
        Some(head),
    );
    let action = ProjectInstallAction {
        kind: ProjectInstallActionKind::Uninstall,
        authority: ProjectInstallAuthority::PhysicalOwner,
        service_id: String::from(WORKSPACE_SERVICE_ID),
        generation,
        log_sequence: cursor.next_action_seq,
        previous_commit_sha256: Some(head),
        install_envelope_sha256: None,
        target_install_commit_sha256: None,
        authority_evidence_sha256: authority_evidence_hash(PreviewKind::Uninstall, head, active),
        physical_approval_sha256: Some(physical_approval_sha256),
        authority_key_sha256: Some(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        authority_signature: Vec::new(),
        action_sha256: [0; 32],
    };
    if install_action_signature_payload_sha256(&action).is_err() {
        return (false, "project_install_action_shape_invalid");
    }
    let mut state = STATE.lock();
    state.pending = Some(PendingAction::Uninstall { action });
    state.signature_verified = false;
    state.last_reason = "project_uninstall_signature_required";
    state.last_commit_sha256 = None;
    drop(state);
    emit_preview_marker(PreviewKind::Uninstall, false);
    (true, "project_uninstall_signature_required")
}

fn accept_signature(input: &str, method: &str, expected_kind: PreviewKind) -> (bool, &'static str) {
    let signature = match signature_arg(input, method) {
        Ok(signature) => signature,
        Err(reason) => return (false, reason),
    };
    let mut state = STATE.lock();
    let Some(mut pending) = state.pending.clone() else {
        return (false, "project_install_preview_missing");
    };
    if pending.kind() != expected_kind {
        return (false, "project_install_preview_kind_mismatch");
    }
    let digest = match install_action_signature_payload_sha256(pending.action()) {
        Ok(digest) => digest,
        Err(error) => return (false, error.reason()),
    };
    if !verify_promotion_authority_signature(&signature, &digest) {
        return (false, "project_install_physical_signature_invalid");
    }
    match &mut pending {
        PendingAction::Install { intent, .. } => {
            intent.action.authority_signature = signature;
            intent.action.action_sha256 = [0; 32];
            intent.action = match seal_install_action(intent.action.clone()) {
                Ok(action) => action,
                Err(error) => return (false, error.reason()),
            };
            intent.intent_sha256 = [0; 32];
            *intent = match seal_install_intent(intent.clone()) {
                Ok(intent) => intent,
                Err(error) => return (false, error.reason()),
            };
        }
        PendingAction::GrantedCandidateInstall { action, .. } => {
            action.authority_signature = signature;
            action.action_sha256 = [0; 32];
            *action = match seal_install_action(action.clone()) {
                Ok(action) => action,
                Err(error) => return (false, error.reason()),
            };
        }
        PendingAction::UiProgramInstall { action, .. } => {
            action.authority_signature = signature;
            action.action_sha256 = [0; 32];
            *action = match seal_install_action(action.clone()) {
                Ok(action) => action,
                Err(error) => return (false, error.reason()),
            };
        }
        PendingAction::Uninstall { action } => {
            action.authority_signature = signature;
            action.action_sha256 = [0; 32];
            *action = match seal_install_action(action.clone()) {
                Ok(action) => action,
                Err(error) => return (false, error.reason()),
            };
        }
    }
    state.pending = Some(pending);
    state.signature_verified = true;
    state.last_reason = match expected_kind {
        PreviewKind::Install => "project_install_pending_physical_pointer_approval",
        PreviewKind::Uninstall => "project_uninstall_pending_physical_pointer_approval",
    };
    drop(state);
    emit_preview_marker(expected_kind, true);
    (
        true,
        match expected_kind {
            PreviewKind::Install => "project_install_pending_physical_pointer_approval",
            PreviewKind::Uninstall => "project_uninstall_pending_physical_pointer_approval",
        },
    )
}

fn emit(method: &'static str, accepted: bool, reason: &'static str) {
    let snapshot = STATE.lock().snapshot();
    begin_response(method);
    emit_record_fields(fields(method, accepted, reason, &snapshot), 6);
    end_response(method);
}

fn fields<'a>(
    method: &'static str,
    accepted: bool,
    reason: &'static str,
    snapshot: &'a PreviewSnapshot,
) -> Vec<Field<'a>> {
    vec![
        f("method", s(method)),
        f("scope", s("physical_owner_current_boot_preview")),
        f("classification", s("local_only")),
        f("persistence_posture", s("write_only_after_genesis_pointer")),
        f("status", s(if accepted { "accepted" } else { "denied" })),
        f("reason", s(reason)),
        f("accepted", b(accepted)),
        f("rejected", b(!accepted)),
        f("service_id", s(snapshot.service_id)),
        f("phase", s(snapshot.phase)),
        f(
            "action_kind",
            snapshot.kind.map(|kind| s(kind.label())).unwrap_or(V::Null),
        ),
        f("signature_verified", b(snapshot.signature_verified)),
        f(
            "action_signature_message_sha256",
            record_sha_or_null(snapshot.action_signature_message_sha256),
        ),
        f(
            "physical_approval_sha256",
            record_sha_or_null(snapshot.physical_approval_sha256),
        ),
        f(
            "authority_key_sha256",
            V::Sha256(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        ),
        f("trust_tier", s(PROJECT_INSTALL_TRUST_TIER)),
        f("generation", V::U64(snapshot.generation)),
        f("log_sequence", V::U64(snapshot.log_sequence)),
        f(
            "previous_commit_sha256",
            record_sha_or_null(snapshot.previous_commit_sha256),
        ),
        f(
            "candidate_sha256",
            record_sha_or_null(snapshot.candidate_sha256),
        ),
        f(
            "receipt_sha256",
            record_sha_or_null(snapshot.receipt_sha256),
        ),
        f(
            "last_commit_sha256",
            record_sha_or_null(snapshot.last_commit_sha256),
        ),
        f(
            "candidate_blob_offset",
            snapshot
                .candidate_blob_offset
                .map(V::U64)
                .unwrap_or(V::Null),
        ),
        f(
            "candidate_blob_frame_len",
            snapshot
                .candidate_blob_frame_len
                .map(V::U64)
                .unwrap_or(V::Null),
        ),
        f("install_source", snapshot.install_source.map(s).unwrap_or(V::Null)),
        f("receipt_kind", snapshot.receipt_kind.map(s).unwrap_or(V::Null)),
        f("w4_project_receipt_present", b(snapshot.w4_project_receipt_present)),
        f("activation_approval_sha256", record_sha_or_null(snapshot.activation_approval_sha256)),
        f("install_envelope_sha256", record_sha_or_null(snapshot.install_envelope_sha256)),
        f("promotion_transaction_sha256", V::Null),
        f("artifact_persist_frame_sha256", V::Null),
        f("last_reason", s(snapshot.last_reason)),
        f("serial_approval_authorized", b(false)),
        f("physical_pointer_required", b(true)),
        f("writes_persistent_state", b(false)),
    ]
}

fn one_hash_arg(input: &str, method: &str) -> Result<[u8; 32], &'static str> {
    let mut parts = input.split_ascii_whitespace();
    if parts.next() != Some(method) {
        return Err("project_install_method_invalid");
    }
    let hash = parse_sha256_ref(parts.next().ok_or("project_install_binding_missing")?)
        .ok_or("project_install_binding_invalid")?;
    if parts.next().is_some() {
        return Err("project_install_argument_count_invalid");
    }
    Ok(hash)
}

fn signature_arg(input: &str, method: &str) -> Result<Vec<u8>, &'static str> {
    let mut parts = input.split_ascii_whitespace();
    if parts.next() != Some(method) {
        return Err("project_install_method_invalid");
    }
    let token = parts.next().ok_or("project_install_signature_missing")?;
    if parts.next().is_some() || token.is_empty() || token.len() > MAX_SIGNATURE_DER_BYTES * 2 {
        return Err("project_install_signature_invalid");
    }
    let bytes = token.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err("project_install_signature_invalid");
    }
    let mut signature = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        signature.push(
            (hex(bytes[index]).ok_or("project_install_signature_invalid")? << 4)
                | hex(bytes[index + 1]).ok_or("project_install_signature_invalid")?,
        );
        index += 2;
    }
    Ok(signature)
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn approval_hash(
    domain: &[u8],
    generation: u64,
    sequence: u64,
    subject: [u8; 32],
    previous: Option<[u8; 32]>,
) -> [u8; 32] {
    let mut bytes = Vec::from(domain);
    bytes.extend_from_slice(&generation.to_le_bytes());
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&subject);
    bytes.extend_from_slice(&previous.unwrap_or([0; 32]));
    sha256_bytes(&bytes)
}

pub(crate) fn install_pointer_approval_hash(
    generation: u64,
    sequence: u64,
    subject: [u8; 32],
    previous: Option<[u8; 32]>,
) -> [u8; 32] {
    approval_hash(
        INSTALL_APPROVAL_DOMAIN,
        generation,
        sequence,
        subject,
        previous,
    )
}

fn authority_evidence_hash(kind: PreviewKind, first: [u8; 32], second: [u8; 32]) -> [u8; 32] {
    let mut bytes = Vec::from(AUTHORITY_EVIDENCE_DOMAIN);
    bytes.push(match kind {
        PreviewKind::Install => 1,
        PreviewKind::Uninstall => 2,
    });
    bytes.extend_from_slice(&first);
    bytes.extend_from_slice(&second);
    sha256_bytes(&bytes)
}

fn emit_preview_marker(kind: PreviewKind, signed: bool) {
    let snapshot = STATE.lock().snapshot();
    serial::write_raw_str("PROJECT_INSTALL_PREVIEW kind=");
    serial::write_raw_str(kind.label());
    serial::write_raw_str(" result=accepted signature_verified=");
    serial::write_raw_str(if signed { "true" } else { "false" });
    serial::write_raw_str(" action_signature_message_sha256=sha256:");
    write_hash(snapshot.action_signature_message_sha256.unwrap_or([0; 32]));
    serial::write_raw_str(" physical_approval_sha256=sha256:");
    write_hash(snapshot.physical_approval_sha256.unwrap_or([0; 32]));
    serial::write_raw_fmt(format_args!(
        " generation={} sequence={} approval={}\r\n",
        snapshot.generation,
        snapshot.log_sequence,
        if signed {
            "genesis_pointer_required"
        } else {
            "owner_signature_required"
        },
    ));
}

fn emit_commit_marker(
    kind: PreviewKind,
    commit: &raios_core::project_install::ProjectInstallCommit,
) {
    serial::write_raw_str("PROJECT_INSTALL_COMMIT kind=");
    serial::write_raw_str(kind.label());
    serial::write_raw_str(
        " physical_approval=genesis_pointer result=accepted commit_sha256=sha256:",
    );
    write_hash(commit.commit_sha256);
    serial::write_raw_str(" candidate_sha256=");
    if let Some(envelope) = commit.envelope.as_ref() {
        serial::write_raw_str("sha256:");
        write_hash(envelope.candidate_sha256);
    } else {
        serial::write_raw_str("none");
    }
    if let Some(blob) = commit.candidate_blob {
        serial::write_raw_fmt(format_args!(
            " candidate_blob_offset={} candidate_blob_frame_len={}",
            blob.offset, blob.frame_len
        ));
    }
    serial::write_raw_fmt(format_args!(
        " generation={} sequence={} trust_tier={}\r\n",
        commit.action.generation, commit.action.log_sequence, PROJECT_INSTALL_TRUST_TIER,
    ));
}

fn finish_denied(reason: &'static str) {
    let mut state = STATE.lock();
    state.pending = None;
    state.signature_verified = false;
    state.last_reason = reason;
    drop(state);
    serial::write_raw_str("PROJECT_INSTALL_COMMIT result=denied reason=");
    serial::write_line(reason);
}

fn finish_ui_program_denied(reason: &'static str) {
    let mut state = STATE.lock();
    state.pending = None;
    state.signature_verified = false;
    state.last_reason = reason;
}

fn write_hash(hash: [u8; 32]) {
    for byte in hash {
        serial::write_raw_fmt(format_args!("{byte:02x}"));
    }
}
