use alloc::{string::String, vec::Vec};

use crate::sha256_bytes;

const STATE_SCHEMA_DOMAIN: &[u8] = b"raios.project_app_state_schema.v1";
const INSTALL_ENVELOPE_DOMAIN: &[u8] = b"raios.project_install_envelope.v1";
const GRANTED_CANDIDATE_INSTALL_ENVELOPE_DOMAIN: &[u8] =
    b"raios.granted_candidate_install_envelope.v1";
const ACTION_SIGNATURE_DOMAIN: &[u8] = b"raios.project_install_action_signature.v1";
const ACTION_DOMAIN: &[u8] = b"raios.project_install_action.v1";
const INTENT_DOMAIN: &[u8] = b"raios.project_install_intent.v1";
const COMMIT_DOMAIN: &[u8] = b"raios.project_install_commit.v1";
const INTENT_MAGIC: &[u8; 8] = b"RAIINT01";
const COMMIT_MAGIC: &[u8; 8] = b"RAIINS01";

const MAX_TEXT_BYTES: usize = 128;
const MAX_SIGNATURE_BYTES: usize = 256;
const MAX_CANDIDATE_BYTES: u64 = 262_144;
const MAX_STATE_BYTES: u64 = 1024 * 1024;
const MAX_IMPORTS: u32 = 16;

/// Durable project apps are signed by the current development authority, but
/// are not yet sealed to an owner hardware identity. This is intentionally
/// distinct from the W5 current-boot workstation-build trust tier.
pub const PROJECT_INSTALL_TRUST_TIER: &str = "dev_key_not_owner_sealed";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateMigrationPolicy {
    Denied,
    ExactSchemaOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectAppStateSchema {
    pub schema_id: String,
    pub version: u32,
    pub max_state_bytes: u64,
    pub migration_policy: StateMigrationPolicy,
    pub schema_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContentBlobRef {
    pub payload_sha256: [u8; 32],
    pub frame_sha256: [u8; 32],
    pub offset: u64,
    pub frame_len: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectInstallEnvelope {
    pub service_id: String,
    pub project_id: [u8; 16],
    pub project_revision_sha256: [u8; 32],
    pub input_manifest_sha256: [u8; 32],
    pub build_receipt_sha256: [u8; 32],
    pub encoded_receipt_sha256: [u8; 32],
    pub workspace_run_binding_sha256: [u8; 32],
    pub candidate_sha256: [u8; 32],
    pub candidate_byte_len: u64,
    pub import_list_sha256: [u8; 32],
    pub import_count: u32,
    pub entrypoint: String,
    pub fuel_budget: u64,
    pub memory_limit_bytes: u64,
    pub instance_limit: u32,
    pub memory_count_limit: u32,
    pub table_limit: u32,
    pub state_schema: ProjectAppStateSchema,
    pub previous_install_commit_sha256: Option<[u8; 32]>,
    pub previous_artifact_sha256: Option<[u8; 32]>,
    pub generation: u64,
    pub auto_start: bool,
    pub trust_tier: String,
    pub envelope_sha256: [u8; 32],
}

/// W7/M6 provenance carried into the existing W6 install-action ceremony.
/// This deliberately has no workspace or project-build fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantedCandidateInstallEnvelope {
    pub service_id: String,
    pub candidate_sha256: [u8; 32],
    pub candidate_byte_len: u64,
    pub activation_approval_sha256: [u8; 32],
    pub computed_grant_sha256: [u8; 32],
    pub attestation_reference_sha256: [u8; 32],
    pub w7_invocation_sha256: [u8; 32],
    pub w7_receipt_sha256: [u8; 32],
    pub receiver_content_sha256: [u8; 32],
    pub receiver_candidate_sha256: [u8; 32],
    pub catalog_candidate_sha256: [u8; 32],
    pub generation: u64,
    pub auto_start: bool,
    pub trust_tier: String,
    pub envelope_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectInstallActionKind {
    Install,
    ActivationAttempt,
    ActivationSuccess,
    Rollback,
    Uninstall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectInstallAuthority {
    PhysicalOwner,
    RuntimeHealth,
    RecoveryFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectInstallAction {
    pub kind: ProjectInstallActionKind,
    pub authority: ProjectInstallAuthority,
    pub service_id: String,
    pub generation: u64,
    pub log_sequence: u64,
    pub previous_commit_sha256: Option<[u8; 32]>,
    pub install_envelope_sha256: Option<[u8; 32]>,
    pub target_install_commit_sha256: Option<[u8; 32]>,
    pub authority_evidence_sha256: [u8; 32],
    pub physical_approval_sha256: Option<[u8; 32]>,
    pub authority_key_sha256: Option<[u8; 32]>,
    pub authority_signature: Vec<u8>,
    pub action_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectInstallIntent {
    pub action: ProjectInstallAction,
    pub envelope: ProjectInstallEnvelope,
    pub intent_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectInstallCommit {
    pub action: ProjectInstallAction,
    pub install_intent_sha256: Option<[u8; 32]>,
    pub envelope: Option<ProjectInstallEnvelope>,
    pub candidate_blob: Option<ContentBlobRef>,
    pub receipt_blob: Option<ContentBlobRef>,
    pub commit_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProjectInstall {
    pub service_id: Option<String>,
    pub head_commit_sha256: Option<[u8; 32]>,
    pub generation: u64,
    pub installed: bool,
    pub probation_install_commit_sha256: Option<[u8; 32]>,
    pub probation_activation_attempted: bool,
    pub active_install_commit_sha256: Option<[u8; 32]>,
    pub last_good_install_commit_sha256: Option<[u8; 32]>,
    pub candidate_sha256: Option<[u8; 32]>,
    pub candidate_blob: Option<ContentBlobRef>,
    pub receipt_blob: Option<ContentBlobRef>,
    pub state_schema: Option<ProjectAppStateSchema>,
    pub rollback_applied: bool,
    pub uninstall_tombstoned: bool,
}

impl ResolvedProjectInstall {
    fn empty() -> Self {
        Self {
            service_id: None,
            head_commit_sha256: None,
            generation: 0,
            installed: false,
            probation_install_commit_sha256: None,
            probation_activation_attempted: false,
            active_install_commit_sha256: None,
            last_good_install_commit_sha256: None,
            candidate_sha256: None,
            candidate_blob: None,
            receipt_blob: None,
            state_schema: None,
            rollback_applied: false,
            uninstall_tombstoned: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectInstallError {
    InvalidEncoding,
    InvalidHash,
    InvalidField,
    InvalidActionShape,
    ChainFork,
    SequenceNotIncreasing,
    ServiceMismatch,
    GenerationMismatch,
    PendingProbation,
    PreviousInstallMismatch,
    TargetMissing,
    TargetNotSuccessful,
    StaleProbation,
}

impl ProjectInstallError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::InvalidEncoding => "project_install_invalid_encoding",
            Self::InvalidHash => "project_install_hash_mismatch",
            Self::InvalidField => "project_install_field_invalid",
            Self::InvalidActionShape => "project_install_action_shape_invalid",
            Self::ChainFork => "project_install_commit_chain_fork",
            Self::SequenceNotIncreasing => "project_install_sequence_not_increasing",
            Self::ServiceMismatch => "project_install_service_mismatch",
            Self::GenerationMismatch => "project_install_generation_mismatch",
            Self::PendingProbation => "project_install_probation_pending",
            Self::PreviousInstallMismatch => "project_install_previous_binding_mismatch",
            Self::TargetMissing => "project_install_target_missing",
            Self::TargetNotSuccessful => "project_install_target_not_last_good",
            Self::StaleProbation => "project_install_probation_target_stale",
        }
    }
}

pub fn seal_state_schema(
    mut schema: ProjectAppStateSchema,
) -> Result<ProjectAppStateSchema, ProjectInstallError> {
    validate_state_schema_fields(&schema)?;
    schema.schema_sha256 = state_schema_hash(&schema)?;
    Ok(schema)
}

pub fn seal_install_envelope(
    mut envelope: ProjectInstallEnvelope,
) -> Result<ProjectInstallEnvelope, ProjectInstallError> {
    validate_envelope_fields(&envelope)?;
    validate_state_schema(&envelope.state_schema)?;
    envelope.envelope_sha256 = install_envelope_hash(&envelope)?;
    Ok(envelope)
}

pub fn seal_granted_candidate_install_envelope(
    mut envelope: GrantedCandidateInstallEnvelope,
) -> Result<GrantedCandidateInstallEnvelope, ProjectInstallError> {
    validate_granted_candidate_install_envelope_fields(&envelope)?;
    envelope.envelope_sha256 = granted_candidate_install_envelope_hash(&envelope)?;
    Ok(envelope)
}

pub fn granted_candidate_install_envelope_hash(
    envelope: &GrantedCandidateInstallEnvelope,
) -> Result<[u8; 32], ProjectInstallError> {
    let mut bytes = Vec::from(GRANTED_CANDIDATE_INSTALL_ENVELOPE_DOMAIN);
    encode_granted_candidate_install_envelope_fields(&mut bytes, envelope)?;
    Ok(sha256_bytes(&bytes))
}

pub fn seal_install_action(
    mut action: ProjectInstallAction,
) -> Result<ProjectInstallAction, ProjectInstallError> {
    validate_action_fields(&action)?;
    action.action_sha256 = install_action_hash(&action)?;
    Ok(action)
}

pub fn seal_install_commit(
    mut commit: ProjectInstallCommit,
) -> Result<ProjectInstallCommit, ProjectInstallError> {
    validate_commit_fields(&commit)?;
    validate_install_action(&commit.action)?;
    if let Some(envelope) = &commit.envelope {
        validate_install_envelope(envelope)?;
    }
    commit.commit_sha256 = install_commit_hash(&commit)?;
    Ok(commit)
}

pub fn seal_install_intent(
    mut intent: ProjectInstallIntent,
) -> Result<ProjectInstallIntent, ProjectInstallError> {
    validate_intent_fields(&intent)?;
    validate_install_action(&intent.action)?;
    validate_install_envelope(&intent.envelope)?;
    intent.intent_sha256 = install_intent_hash(&intent)?;
    Ok(intent)
}

pub fn state_schema_hash(schema: &ProjectAppStateSchema) -> Result<[u8; 32], ProjectInstallError> {
    let mut bytes = Vec::from(STATE_SCHEMA_DOMAIN);
    encode_state_schema_fields(&mut bytes, schema)?;
    Ok(sha256_bytes(&bytes))
}

pub fn install_envelope_hash(
    envelope: &ProjectInstallEnvelope,
) -> Result<[u8; 32], ProjectInstallError> {
    let mut bytes = Vec::from(INSTALL_ENVELOPE_DOMAIN);
    encode_envelope_fields(&mut bytes, envelope)?;
    Ok(sha256_bytes(&bytes))
}

pub fn install_action_signature_payload(
    action: &ProjectInstallAction,
) -> Result<Vec<u8>, ProjectInstallError> {
    validate_action_semantics(action, false)?;
    let mut bytes = Vec::from(ACTION_SIGNATURE_DOMAIN);
    encode_action_semantics(&mut bytes, action)?;
    Ok(bytes)
}

/// The physical-owner signer signs this digest, not an independently encoded
/// copy of the action. The storage adapter recomputes it before any write.
pub fn install_action_signature_payload_sha256(
    action: &ProjectInstallAction,
) -> Result<[u8; 32], ProjectInstallError> {
    Ok(sha256_bytes(&install_action_signature_payload(action)?))
}

pub fn install_action_hash(action: &ProjectInstallAction) -> Result<[u8; 32], ProjectInstallError> {
    let mut bytes = Vec::from(ACTION_DOMAIN);
    encode_action_semantics(&mut bytes, action)?;
    put_bytes(&mut bytes, &action.authority_signature)?;
    Ok(sha256_bytes(&bytes))
}

pub fn install_commit_hash(commit: &ProjectInstallCommit) -> Result<[u8; 32], ProjectInstallError> {
    let mut bytes = Vec::from(COMMIT_DOMAIN);
    encode_commit_body(&mut bytes, commit, false)?;
    Ok(sha256_bytes(&bytes))
}

pub fn install_intent_hash(intent: &ProjectInstallIntent) -> Result<[u8; 32], ProjectInstallError> {
    let mut bytes = Vec::from(INTENT_DOMAIN);
    encode_intent_body(&mut bytes, intent, false)?;
    Ok(sha256_bytes(&bytes))
}

pub fn validate_install_intent(intent: &ProjectInstallIntent) -> Result<(), ProjectInstallError> {
    validate_intent_fields(intent)?;
    validate_install_action(&intent.action)?;
    validate_install_envelope(&intent.envelope)?;
    if intent.intent_sha256 != install_intent_hash(intent)? {
        return Err(ProjectInstallError::InvalidHash);
    }
    Ok(())
}

pub fn validate_state_schema(schema: &ProjectAppStateSchema) -> Result<(), ProjectInstallError> {
    validate_state_schema_fields(schema)?;
    if schema.schema_sha256 != state_schema_hash(schema)? {
        return Err(ProjectInstallError::InvalidHash);
    }
    Ok(())
}

pub fn validate_install_envelope(
    envelope: &ProjectInstallEnvelope,
) -> Result<(), ProjectInstallError> {
    validate_envelope_fields(envelope)?;
    validate_state_schema(&envelope.state_schema)?;
    if envelope.envelope_sha256 != install_envelope_hash(envelope)? {
        return Err(ProjectInstallError::InvalidHash);
    }
    Ok(())
}

pub fn validate_granted_candidate_install_envelope(
    envelope: &GrantedCandidateInstallEnvelope,
) -> Result<(), ProjectInstallError> {
    validate_granted_candidate_install_envelope_fields(envelope)?;
    if envelope.envelope_sha256 != granted_candidate_install_envelope_hash(envelope)? {
        return Err(ProjectInstallError::InvalidHash);
    }
    Ok(())
}

pub fn validate_install_action(action: &ProjectInstallAction) -> Result<(), ProjectInstallError> {
    validate_action_fields(action)?;
    if action.action_sha256 != install_action_hash(action)? {
        return Err(ProjectInstallError::InvalidHash);
    }
    Ok(())
}

pub fn validate_install_commit(commit: &ProjectInstallCommit) -> Result<(), ProjectInstallError> {
    validate_commit_fields(commit)?;
    validate_install_action(&commit.action)?;
    if let Some(envelope) = &commit.envelope {
        validate_install_envelope(envelope)?;
    }
    if commit.commit_sha256 != install_commit_hash(commit)? {
        return Err(ProjectInstallError::InvalidHash);
    }
    Ok(())
}

pub fn encode_install_commit(
    commit: &ProjectInstallCommit,
) -> Result<Vec<u8>, ProjectInstallError> {
    validate_install_commit(commit)?;
    let mut out = Vec::from(COMMIT_MAGIC);
    encode_commit_body(&mut out, commit, true)?;
    Ok(out)
}

pub fn decode_install_commit(bytes: &[u8]) -> Result<ProjectInstallCommit, ProjectInstallError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(COMMIT_MAGIC.len())? != COMMIT_MAGIC {
        return Err(ProjectInstallError::InvalidEncoding);
    }
    let commit = decode_commit_body(&mut cursor)?;
    if !cursor.done() {
        return Err(ProjectInstallError::InvalidEncoding);
    }
    validate_install_commit(&commit)?;
    Ok(commit)
}

pub fn encode_install_intent(
    intent: &ProjectInstallIntent,
) -> Result<Vec<u8>, ProjectInstallError> {
    validate_install_intent(intent)?;
    let mut out = Vec::from(INTENT_MAGIC);
    encode_intent_body(&mut out, intent, true)?;
    Ok(out)
}

pub fn decode_install_intent(bytes: &[u8]) -> Result<ProjectInstallIntent, ProjectInstallError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(INTENT_MAGIC.len())? != INTENT_MAGIC {
        return Err(ProjectInstallError::InvalidEncoding);
    }
    let intent = decode_intent_body(&mut cursor)?;
    if !cursor.done() {
        return Err(ProjectInstallError::InvalidEncoding);
    }
    validate_install_intent(&intent)?;
    Ok(intent)
}

/// Resolves one service's ordered projection from the shared RECLOG.
///
/// The storage adapter remains responsible for frame-chain validation and for
/// cryptographically verifying each action's signature payload before calling
/// this pure state transition layer.
pub fn resolve_install_log(
    records: &[ProjectInstallCommit],
) -> Result<ResolvedProjectInstall, ProjectInstallError> {
    let mut resolved = ResolvedProjectInstall::empty();
    let mut installs: Vec<ProjectInstallCommit> = Vec::new();
    let mut successful: Vec<[u8; 32]> = Vec::new();
    let mut previous_sequence = None;

    for record in records {
        validate_install_commit(record)?;
        if let Some(sequence) = previous_sequence {
            if record.action.log_sequence <= sequence {
                return Err(ProjectInstallError::SequenceNotIncreasing);
            }
        }
        if record.action.previous_commit_sha256 != resolved.head_commit_sha256 {
            return Err(ProjectInstallError::ChainFork);
        }
        match resolved.service_id.as_deref() {
            Some(service_id) if service_id != record.action.service_id => {
                return Err(ProjectInstallError::ServiceMismatch)
            }
            None => resolved.service_id = Some(record.action.service_id.clone()),
            _ => {}
        }

        match record.action.kind {
            ProjectInstallActionKind::Install => {
                apply_install(&mut resolved, record, &mut installs)?
            }
            ProjectInstallActionKind::ActivationAttempt => apply_attempt(&mut resolved, record)?,
            ProjectInstallActionKind::ActivationSuccess => {
                apply_success(&mut resolved, record, &installs, &mut successful)?
            }
            ProjectInstallActionKind::Rollback => {
                apply_rollback(&mut resolved, record, &installs, &successful)?
            }
            ProjectInstallActionKind::Uninstall => apply_uninstall(&mut resolved, record)?,
        }
        resolved.head_commit_sha256 = Some(record.commit_sha256);
        previous_sequence = Some(record.action.log_sequence);
    }
    Ok(resolved)
}

fn apply_install(
    resolved: &mut ResolvedProjectInstall,
    record: &ProjectInstallCommit,
    installs: &mut Vec<ProjectInstallCommit>,
) -> Result<(), ProjectInstallError> {
    if resolved.probation_install_commit_sha256.is_some() {
        return Err(ProjectInstallError::PendingProbation);
    }
    if record.action.generation != resolved.generation.saturating_add(1) {
        return Err(ProjectInstallError::GenerationMismatch);
    }
    let envelope = record
        .envelope
        .as_ref()
        .ok_or(ProjectInstallError::InvalidActionShape)?;
    if envelope.previous_install_commit_sha256 != resolved.active_install_commit_sha256 {
        return Err(ProjectInstallError::PreviousInstallMismatch);
    }
    let previous_artifact = resolved
        .active_install_commit_sha256
        .and_then(|hash| install_by_hash(installs, hash))
        .and_then(|install| install.envelope.as_ref())
        .map(|item| item.candidate_sha256);
    if envelope.previous_artifact_sha256 != previous_artifact {
        return Err(ProjectInstallError::PreviousInstallMismatch);
    }
    resolved.generation = record.action.generation;
    resolved.installed = true;
    resolved.probation_install_commit_sha256 = Some(record.commit_sha256);
    resolved.probation_activation_attempted = false;
    resolved.rollback_applied = false;
    resolved.uninstall_tombstoned = false;
    installs.push(record.clone());
    project_active_from_record(resolved, record, false)?;
    Ok(())
}

fn apply_attempt(
    resolved: &mut ResolvedProjectInstall,
    record: &ProjectInstallCommit,
) -> Result<(), ProjectInstallError> {
    if record.action.generation != resolved.generation {
        return Err(ProjectInstallError::GenerationMismatch);
    }
    let target = record
        .action
        .target_install_commit_sha256
        .ok_or(ProjectInstallError::TargetMissing)?;
    if resolved.probation_install_commit_sha256 != Some(target)
        || resolved.probation_activation_attempted
    {
        return Err(ProjectInstallError::StaleProbation);
    }
    resolved.probation_activation_attempted = true;
    Ok(())
}

fn apply_success(
    resolved: &mut ResolvedProjectInstall,
    record: &ProjectInstallCommit,
    installs: &[ProjectInstallCommit],
    successful: &mut Vec<[u8; 32]>,
) -> Result<(), ProjectInstallError> {
    if record.action.generation != resolved.generation {
        return Err(ProjectInstallError::GenerationMismatch);
    }
    let target = record
        .action
        .target_install_commit_sha256
        .ok_or(ProjectInstallError::TargetMissing)?;
    if resolved.probation_install_commit_sha256 != Some(target)
        || !resolved.probation_activation_attempted
    {
        return Err(ProjectInstallError::StaleProbation);
    }
    let install = install_by_hash(installs, target).ok_or(ProjectInstallError::TargetMissing)?;
    resolved.probation_install_commit_sha256 = None;
    resolved.probation_activation_attempted = false;
    resolved.active_install_commit_sha256 = Some(target);
    resolved.last_good_install_commit_sha256 = Some(target);
    resolved.installed = true;
    resolved.rollback_applied = false;
    resolved.uninstall_tombstoned = false;
    if !successful.contains(&target) {
        successful.push(target);
    }
    project_active_from_record(resolved, install, true)
}

fn apply_rollback(
    resolved: &mut ResolvedProjectInstall,
    record: &ProjectInstallCommit,
    installs: &[ProjectInstallCommit],
    successful: &[[u8; 32]],
) -> Result<(), ProjectInstallError> {
    if record.action.generation != resolved.generation {
        return Err(ProjectInstallError::GenerationMismatch);
    }
    if resolved.uninstall_tombstoned {
        return Err(ProjectInstallError::TargetNotSuccessful);
    }
    if record.action.authority == ProjectInstallAuthority::RecoveryFailure
        && (resolved.probation_install_commit_sha256.is_none()
            || !resolved.probation_activation_attempted)
    {
        return Err(ProjectInstallError::StaleProbation);
    }
    match record.action.target_install_commit_sha256 {
        Some(target) => {
            if !successful.contains(&target) {
                return Err(ProjectInstallError::TargetNotSuccessful);
            }
            if record.action.authority == ProjectInstallAuthority::RecoveryFailure
                && resolved.last_good_install_commit_sha256 != Some(target)
            {
                return Err(ProjectInstallError::TargetNotSuccessful);
            }
            let install =
                install_by_hash(installs, target).ok_or(ProjectInstallError::TargetMissing)?;
            resolved.active_install_commit_sha256 = Some(target);
            resolved.last_good_install_commit_sha256 = Some(target);
            resolved.installed = true;
            project_active_from_record(resolved, install, true)?;
        }
        None if record.action.authority == ProjectInstallAuthority::RecoveryFailure
            && resolved.last_good_install_commit_sha256.is_none() =>
        {
            clear_active(resolved);
        }
        None => return Err(ProjectInstallError::TargetMissing),
    }
    resolved.probation_install_commit_sha256 = None;
    resolved.probation_activation_attempted = false;
    resolved.rollback_applied = true;
    resolved.uninstall_tombstoned = false;
    Ok(())
}

fn apply_uninstall(
    resolved: &mut ResolvedProjectInstall,
    record: &ProjectInstallCommit,
) -> Result<(), ProjectInstallError> {
    if record.action.generation != resolved.generation || !resolved.installed {
        return Err(ProjectInstallError::GenerationMismatch);
    }
    clear_active(resolved);
    resolved.probation_install_commit_sha256 = None;
    resolved.probation_activation_attempted = false;
    resolved.last_good_install_commit_sha256 = None;
    resolved.rollback_applied = false;
    resolved.uninstall_tombstoned = true;
    Ok(())
}

fn clear_active(resolved: &mut ResolvedProjectInstall) {
    resolved.installed = false;
    resolved.active_install_commit_sha256 = None;
    resolved.candidate_sha256 = None;
    resolved.candidate_blob = None;
    resolved.receipt_blob = None;
    resolved.state_schema = None;
}

fn project_active_from_record(
    resolved: &mut ResolvedProjectInstall,
    record: &ProjectInstallCommit,
    active: bool,
) -> Result<(), ProjectInstallError> {
    let envelope = record
        .envelope
        .as_ref()
        .ok_or(ProjectInstallError::TargetMissing)?;
    resolved.candidate_sha256 = Some(envelope.candidate_sha256);
    resolved.candidate_blob = record.candidate_blob;
    resolved.receipt_blob = record.receipt_blob;
    resolved.state_schema = Some(envelope.state_schema.clone());
    if active {
        resolved.active_install_commit_sha256 = Some(record.commit_sha256);
    }
    Ok(())
}

fn install_by_hash(
    installs: &[ProjectInstallCommit],
    hash: [u8; 32],
) -> Option<&ProjectInstallCommit> {
    installs.iter().find(|record| record.commit_sha256 == hash)
}

fn validate_state_schema_fields(schema: &ProjectAppStateSchema) -> Result<(), ProjectInstallError> {
    validate_text(&schema.schema_id)?;
    if schema.version == 0
        || schema.max_state_bytes > MAX_STATE_BYTES
        || (schema.max_state_bytes == 0 && schema.migration_policy != StateMigrationPolicy::Denied)
    {
        return Err(ProjectInstallError::InvalidField);
    }
    Ok(())
}

fn validate_envelope_fields(envelope: &ProjectInstallEnvelope) -> Result<(), ProjectInstallError> {
    validate_text(&envelope.service_id)?;
    validate_text(&envelope.entrypoint)?;
    validate_text(&envelope.trust_tier)?;
    validate_state_schema_fields(&envelope.state_schema)?;
    if envelope.project_id == [0; 16]
        || any_zero(&[
            envelope.project_revision_sha256,
            envelope.input_manifest_sha256,
            envelope.build_receipt_sha256,
            envelope.encoded_receipt_sha256,
            envelope.workspace_run_binding_sha256,
            envelope.candidate_sha256,
            envelope.import_list_sha256,
        ])
        || envelope.candidate_byte_len == 0
        || envelope.candidate_byte_len > MAX_CANDIDATE_BYTES
        || envelope.import_count > MAX_IMPORTS
        || envelope.generation == 0
        || envelope.fuel_budget == 0
        || envelope.memory_limit_bytes == 0
        || envelope.instance_limit == 0
        || envelope.memory_count_limit == 0
        || envelope.trust_tier != PROJECT_INSTALL_TRUST_TIER
    {
        return Err(ProjectInstallError::InvalidField);
    }
    if envelope.previous_install_commit_sha256.is_some()
        != envelope.previous_artifact_sha256.is_some()
        || !optional_hash_valid(envelope.previous_install_commit_sha256)
        || !optional_hash_valid(envelope.previous_artifact_sha256)
    {
        return Err(ProjectInstallError::InvalidField);
    }
    Ok(())
}

fn validate_granted_candidate_install_envelope_fields(
    envelope: &GrantedCandidateInstallEnvelope,
) -> Result<(), ProjectInstallError> {
    validate_text(&envelope.service_id)?;
    validate_text(&envelope.trust_tier)?;
    if envelope.service_id != "svc.dev.granted_candidate"
        || envelope.candidate_byte_len == 0
        || envelope.candidate_byte_len > MAX_CANDIDATE_BYTES
        || envelope.generation == 0
        || !envelope.auto_start
        || envelope.trust_tier != PROJECT_INSTALL_TRUST_TIER
        || any_zero(&[
            envelope.candidate_sha256,
            envelope.activation_approval_sha256,
            envelope.computed_grant_sha256,
            envelope.attestation_reference_sha256,
            envelope.w7_invocation_sha256,
            envelope.w7_receipt_sha256,
            envelope.receiver_content_sha256,
            envelope.receiver_candidate_sha256,
            envelope.catalog_candidate_sha256,
        ])
        || envelope.candidate_sha256 != envelope.receiver_content_sha256
        || envelope.candidate_sha256 != envelope.receiver_candidate_sha256
        || envelope.candidate_sha256 != envelope.catalog_candidate_sha256
    {
        return Err(ProjectInstallError::InvalidField);
    }
    Ok(())
}

fn validate_action_fields(action: &ProjectInstallAction) -> Result<(), ProjectInstallError> {
    validate_action_semantics(action, true)
}

fn validate_action_semantics(
    action: &ProjectInstallAction,
    require_signature: bool,
) -> Result<(), ProjectInstallError> {
    validate_text(&action.service_id)?;
    if action.generation == 0
        || action.log_sequence == 0
        || action.authority_evidence_sha256 == [0; 32]
        || action.authority_signature.len() > MAX_SIGNATURE_BYTES
    {
        return Err(ProjectInstallError::InvalidField);
    }
    let physical = action.authority == ProjectInstallAuthority::PhysicalOwner;
    if physical
        && (action.physical_approval_sha256.is_none()
            || action.authority_key_sha256.is_none()
            || (require_signature && action.authority_signature.is_empty()))
    {
        return Err(ProjectInstallError::InvalidActionShape);
    }
    if !physical {
        let signing_pair_invalid = if require_signature {
            action.authority_key_sha256.is_some() != !action.authority_signature.is_empty()
        } else {
            action.authority_key_sha256.is_none() && !action.authority_signature.is_empty()
        };
        if action.physical_approval_sha256.is_some() || signing_pair_invalid {
            return Err(ProjectInstallError::InvalidActionShape);
        }
    }
    if !optional_hash_valid(action.previous_commit_sha256)
        || !optional_hash_valid(action.install_envelope_sha256)
        || !optional_hash_valid(action.target_install_commit_sha256)
        || !optional_hash_valid(action.physical_approval_sha256)
        || !optional_hash_valid(action.authority_key_sha256)
    {
        return Err(ProjectInstallError::InvalidField);
    }
    let shape_ok = match action.kind {
        ProjectInstallActionKind::Install => {
            physical
                && action.install_envelope_sha256.is_some()
                && action.target_install_commit_sha256.is_none()
        }
        ProjectInstallActionKind::ActivationAttempt => {
            action.authority == ProjectInstallAuthority::RuntimeHealth
                && action.install_envelope_sha256.is_none()
                && action.target_install_commit_sha256.is_some()
        }
        ProjectInstallActionKind::ActivationSuccess => {
            action.authority == ProjectInstallAuthority::RuntimeHealth
                && action.install_envelope_sha256.is_none()
                && action.target_install_commit_sha256.is_some()
        }
        ProjectInstallActionKind::Rollback => {
            matches!(
                action.authority,
                ProjectInstallAuthority::PhysicalOwner | ProjectInstallAuthority::RecoveryFailure
            ) && action.install_envelope_sha256.is_none()
        }
        ProjectInstallActionKind::Uninstall => {
            physical
                && action.install_envelope_sha256.is_none()
                && action.target_install_commit_sha256.is_none()
        }
    };
    if !shape_ok {
        return Err(ProjectInstallError::InvalidActionShape);
    }
    Ok(())
}

fn validate_commit_fields(commit: &ProjectInstallCommit) -> Result<(), ProjectInstallError> {
    validate_action_fields(&commit.action)?;
    match commit.action.kind {
        ProjectInstallActionKind::Install => {
            if !optional_hash_valid(commit.install_intent_sha256)
                || commit.install_intent_sha256.is_none()
            {
                return Err(ProjectInstallError::InvalidActionShape);
            }
            let envelope = commit
                .envelope
                .as_ref()
                .ok_or(ProjectInstallError::InvalidActionShape)?;
            let candidate = commit
                .candidate_blob
                .ok_or(ProjectInstallError::InvalidActionShape)?;
            let receipt = commit
                .receipt_blob
                .ok_or(ProjectInstallError::InvalidActionShape)?;
            if envelope.service_id != commit.action.service_id
                || envelope.generation != commit.action.generation
                || commit.action.install_envelope_sha256 != Some(envelope.envelope_sha256)
                || candidate.payload_sha256 != envelope.candidate_sha256
                || receipt.payload_sha256 != envelope.encoded_receipt_sha256
            {
                return Err(ProjectInstallError::InvalidActionShape);
            }
            validate_blob(candidate)?;
            validate_blob(receipt)?;
            if blob_ranges_overlap(candidate, receipt) {
                return Err(ProjectInstallError::InvalidField);
            }
        }
        _ if commit.install_intent_sha256.is_some()
            || commit.envelope.is_some()
            || commit.candidate_blob.is_some()
            || commit.receipt_blob.is_some() =>
        {
            return Err(ProjectInstallError::InvalidActionShape)
        }
        _ => {}
    }
    Ok(())
}

fn validate_intent_fields(intent: &ProjectInstallIntent) -> Result<(), ProjectInstallError> {
    if intent.action.kind != ProjectInstallActionKind::Install
        || intent.action.service_id != intent.envelope.service_id
        || intent.action.generation != intent.envelope.generation
        || intent.action.install_envelope_sha256 != Some(intent.envelope.envelope_sha256)
    {
        return Err(ProjectInstallError::InvalidActionShape);
    }
    Ok(())
}

fn validate_blob(blob: ContentBlobRef) -> Result<(), ProjectInstallError> {
    if blob.payload_sha256 == [0; 32]
        || blob.frame_sha256 == [0; 32]
        || blob.frame_len == 0
        || blob.offset.checked_add(blob.frame_len).is_none()
    {
        return Err(ProjectInstallError::InvalidField);
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<(), ProjectInstallError> {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.as_bytes().contains(&0) {
        return Err(ProjectInstallError::InvalidField);
    }
    Ok(())
}

fn any_zero(hashes: &[[u8; 32]]) -> bool {
    hashes.iter().any(|hash| *hash == [0; 32])
}

fn optional_hash_valid(hash: Option<[u8; 32]>) -> bool {
    hash.is_none_or(|value| value != [0; 32])
}

fn blob_ranges_overlap(left: ContentBlobRef, right: ContentBlobRef) -> bool {
    let Some(left_end) = left.offset.checked_add(left.frame_len) else {
        return true;
    };
    let Some(right_end) = right.offset.checked_add(right.frame_len) else {
        return true;
    };
    left.offset < right_end && right.offset < left_end
}

fn encode_state_schema_fields(
    out: &mut Vec<u8>,
    schema: &ProjectAppStateSchema,
) -> Result<(), ProjectInstallError> {
    put_string(out, &schema.schema_id)?;
    put_u32(out, schema.version);
    put_u64(out, schema.max_state_bytes);
    put_u8(out, migration_tag(schema.migration_policy));
    Ok(())
}

fn encode_state_schema(
    out: &mut Vec<u8>,
    schema: &ProjectAppStateSchema,
) -> Result<(), ProjectInstallError> {
    encode_state_schema_fields(out, schema)?;
    put_hash(out, schema.schema_sha256);
    Ok(())
}

fn encode_envelope_fields(
    out: &mut Vec<u8>,
    envelope: &ProjectInstallEnvelope,
) -> Result<(), ProjectInstallError> {
    put_string(out, &envelope.service_id)?;
    out.extend_from_slice(&envelope.project_id);
    for hash in [
        envelope.project_revision_sha256,
        envelope.input_manifest_sha256,
        envelope.build_receipt_sha256,
        envelope.encoded_receipt_sha256,
        envelope.workspace_run_binding_sha256,
        envelope.candidate_sha256,
    ] {
        put_hash(out, hash);
    }
    put_u64(out, envelope.candidate_byte_len);
    put_hash(out, envelope.import_list_sha256);
    put_u32(out, envelope.import_count);
    put_string(out, &envelope.entrypoint)?;
    put_u64(out, envelope.fuel_budget);
    put_u64(out, envelope.memory_limit_bytes);
    put_u32(out, envelope.instance_limit);
    put_u32(out, envelope.memory_count_limit);
    put_u32(out, envelope.table_limit);
    encode_state_schema(out, &envelope.state_schema)?;
    put_optional_hash(out, envelope.previous_install_commit_sha256);
    put_optional_hash(out, envelope.previous_artifact_sha256);
    put_u64(out, envelope.generation);
    put_bool(out, envelope.auto_start);
    put_string(out, &envelope.trust_tier)?;
    Ok(())
}

fn encode_envelope(
    out: &mut Vec<u8>,
    envelope: &ProjectInstallEnvelope,
) -> Result<(), ProjectInstallError> {
    encode_envelope_fields(out, envelope)?;
    put_hash(out, envelope.envelope_sha256);
    Ok(())
}

fn encode_action_semantics(
    out: &mut Vec<u8>,
    action: &ProjectInstallAction,
) -> Result<(), ProjectInstallError> {
    put_u8(out, action_tag(action.kind));
    put_u8(out, authority_tag(action.authority));
    put_string(out, &action.service_id)?;
    put_u64(out, action.generation);
    put_u64(out, action.log_sequence);
    put_optional_hash(out, action.previous_commit_sha256);
    put_optional_hash(out, action.install_envelope_sha256);
    put_optional_hash(out, action.target_install_commit_sha256);
    put_hash(out, action.authority_evidence_sha256);
    put_optional_hash(out, action.physical_approval_sha256);
    put_optional_hash(out, action.authority_key_sha256);
    Ok(())
}

fn encode_granted_candidate_install_envelope_fields(
    out: &mut Vec<u8>,
    envelope: &GrantedCandidateInstallEnvelope,
) -> Result<(), ProjectInstallError> {
    put_string(out, &envelope.service_id)?;
    put_hash(out, envelope.candidate_sha256);
    put_u64(out, envelope.candidate_byte_len);
    put_hash(out, envelope.activation_approval_sha256);
    put_hash(out, envelope.computed_grant_sha256);
    put_hash(out, envelope.attestation_reference_sha256);
    put_hash(out, envelope.w7_invocation_sha256);
    put_hash(out, envelope.w7_receipt_sha256);
    put_hash(out, envelope.receiver_content_sha256);
    put_hash(out, envelope.receiver_candidate_sha256);
    put_hash(out, envelope.catalog_candidate_sha256);
    put_u64(out, envelope.generation);
    put_bool(out, envelope.auto_start);
    put_string(out, &envelope.trust_tier)
}

fn encode_action(
    out: &mut Vec<u8>,
    action: &ProjectInstallAction,
) -> Result<(), ProjectInstallError> {
    encode_action_semantics(out, action)?;
    put_bytes(out, &action.authority_signature)?;
    put_hash(out, action.action_sha256);
    Ok(())
}

fn encode_blob(out: &mut Vec<u8>, blob: ContentBlobRef) {
    put_hash(out, blob.payload_sha256);
    put_hash(out, blob.frame_sha256);
    put_u64(out, blob.offset);
    put_u64(out, blob.frame_len);
}

fn encode_commit_body(
    out: &mut Vec<u8>,
    commit: &ProjectInstallCommit,
    include_hash: bool,
) -> Result<(), ProjectInstallError> {
    encode_action(out, &commit.action)?;
    put_optional_hash(out, commit.install_intent_sha256);
    put_option(out, commit.envelope.as_ref(), encode_envelope)?;
    put_option_copy(out, commit.candidate_blob, encode_blob);
    put_option_copy(out, commit.receipt_blob, encode_blob);
    if include_hash {
        put_hash(out, commit.commit_sha256);
    }
    Ok(())
}

fn encode_intent_body(
    out: &mut Vec<u8>,
    intent: &ProjectInstallIntent,
    include_hash: bool,
) -> Result<(), ProjectInstallError> {
    encode_action(out, &intent.action)?;
    encode_envelope(out, &intent.envelope)?;
    if include_hash {
        put_hash(out, intent.intent_sha256);
    }
    Ok(())
}

fn decode_commit_body(
    cursor: &mut Cursor<'_>,
) -> Result<ProjectInstallCommit, ProjectInstallError> {
    let action = decode_action(cursor)?;
    let install_intent_sha256 = cursor.optional_hash()?;
    let envelope = cursor.option(decode_envelope)?;
    let candidate_blob = cursor.option(decode_blob)?;
    let receipt_blob = cursor.option(decode_blob)?;
    let commit_sha256 = cursor.hash()?;
    Ok(ProjectInstallCommit {
        action,
        install_intent_sha256,
        envelope,
        candidate_blob,
        receipt_blob,
        commit_sha256,
    })
}

fn decode_intent_body(
    cursor: &mut Cursor<'_>,
) -> Result<ProjectInstallIntent, ProjectInstallError> {
    Ok(ProjectInstallIntent {
        action: decode_action(cursor)?,
        envelope: decode_envelope(cursor)?,
        intent_sha256: cursor.hash()?,
    })
}

fn decode_state_schema(
    cursor: &mut Cursor<'_>,
) -> Result<ProjectAppStateSchema, ProjectInstallError> {
    Ok(ProjectAppStateSchema {
        schema_id: cursor.string()?,
        version: cursor.u32()?,
        max_state_bytes: cursor.u64()?,
        migration_policy: migration_from_tag(cursor.u8()?)?,
        schema_sha256: cursor.hash()?,
    })
}

fn decode_envelope(cursor: &mut Cursor<'_>) -> Result<ProjectInstallEnvelope, ProjectInstallError> {
    let service_id = cursor.string()?;
    let mut project_id = [0; 16];
    project_id.copy_from_slice(cursor.take(16)?);
    Ok(ProjectInstallEnvelope {
        service_id,
        project_id,
        project_revision_sha256: cursor.hash()?,
        input_manifest_sha256: cursor.hash()?,
        build_receipt_sha256: cursor.hash()?,
        encoded_receipt_sha256: cursor.hash()?,
        workspace_run_binding_sha256: cursor.hash()?,
        candidate_sha256: cursor.hash()?,
        candidate_byte_len: cursor.u64()?,
        import_list_sha256: cursor.hash()?,
        import_count: cursor.u32()?,
        entrypoint: cursor.string()?,
        fuel_budget: cursor.u64()?,
        memory_limit_bytes: cursor.u64()?,
        instance_limit: cursor.u32()?,
        memory_count_limit: cursor.u32()?,
        table_limit: cursor.u32()?,
        state_schema: decode_state_schema(cursor)?,
        previous_install_commit_sha256: cursor.optional_hash()?,
        previous_artifact_sha256: cursor.optional_hash()?,
        generation: cursor.u64()?,
        auto_start: cursor.boolean()?,
        trust_tier: cursor.string()?,
        envelope_sha256: cursor.hash()?,
    })
}

fn decode_action(cursor: &mut Cursor<'_>) -> Result<ProjectInstallAction, ProjectInstallError> {
    Ok(ProjectInstallAction {
        kind: action_from_tag(cursor.u8()?)?,
        authority: authority_from_tag(cursor.u8()?)?,
        service_id: cursor.string()?,
        generation: cursor.u64()?,
        log_sequence: cursor.u64()?,
        previous_commit_sha256: cursor.optional_hash()?,
        install_envelope_sha256: cursor.optional_hash()?,
        target_install_commit_sha256: cursor.optional_hash()?,
        authority_evidence_sha256: cursor.hash()?,
        physical_approval_sha256: cursor.optional_hash()?,
        authority_key_sha256: cursor.optional_hash()?,
        authority_signature: cursor.bytes()?,
        action_sha256: cursor.hash()?,
    })
}

fn decode_blob(cursor: &mut Cursor<'_>) -> Result<ContentBlobRef, ProjectInstallError> {
    Ok(ContentBlobRef {
        payload_sha256: cursor.hash()?,
        frame_sha256: cursor.hash()?,
        offset: cursor.u64()?,
        frame_len: cursor.u64()?,
    })
}

fn action_tag(value: ProjectInstallActionKind) -> u8 {
    match value {
        ProjectInstallActionKind::Install => 1,
        ProjectInstallActionKind::ActivationAttempt => 2,
        ProjectInstallActionKind::ActivationSuccess => 3,
        ProjectInstallActionKind::Rollback => 4,
        ProjectInstallActionKind::Uninstall => 5,
    }
}

fn action_from_tag(value: u8) -> Result<ProjectInstallActionKind, ProjectInstallError> {
    match value {
        1 => Ok(ProjectInstallActionKind::Install),
        2 => Ok(ProjectInstallActionKind::ActivationAttempt),
        3 => Ok(ProjectInstallActionKind::ActivationSuccess),
        4 => Ok(ProjectInstallActionKind::Rollback),
        5 => Ok(ProjectInstallActionKind::Uninstall),
        _ => Err(ProjectInstallError::InvalidEncoding),
    }
}

fn authority_tag(value: ProjectInstallAuthority) -> u8 {
    match value {
        ProjectInstallAuthority::PhysicalOwner => 1,
        ProjectInstallAuthority::RuntimeHealth => 2,
        ProjectInstallAuthority::RecoveryFailure => 3,
    }
}

fn authority_from_tag(value: u8) -> Result<ProjectInstallAuthority, ProjectInstallError> {
    match value {
        1 => Ok(ProjectInstallAuthority::PhysicalOwner),
        2 => Ok(ProjectInstallAuthority::RuntimeHealth),
        3 => Ok(ProjectInstallAuthority::RecoveryFailure),
        _ => Err(ProjectInstallError::InvalidEncoding),
    }
}

fn migration_tag(value: StateMigrationPolicy) -> u8 {
    match value {
        StateMigrationPolicy::Denied => 1,
        StateMigrationPolicy::ExactSchemaOnly => 2,
    }
}

fn migration_from_tag(value: u8) -> Result<StateMigrationPolicy, ProjectInstallError> {
    match value {
        1 => Ok(StateMigrationPolicy::Denied),
        2 => Ok(StateMigrationPolicy::ExactSchemaOnly),
        _ => Err(ProjectInstallError::InvalidEncoding),
    }
}

fn put_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_bool(out: &mut Vec<u8>, value: bool) {
    put_u8(out, u8::from(value));
}

fn put_hash(out: &mut Vec<u8>, value: [u8; 32]) {
    out.extend_from_slice(&value);
}

fn put_optional_hash(out: &mut Vec<u8>, value: Option<[u8; 32]>) {
    put_bool(out, value.is_some());
    put_hash(out, value.unwrap_or([0; 32]));
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), ProjectInstallError> {
    let len = u16::try_from(value.len()).map_err(|_| ProjectInstallError::InvalidField)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn put_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<(), ProjectInstallError> {
    let len = u16::try_from(value.len()).map_err(|_| ProjectInstallError::InvalidField)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value);
    Ok(())
}

fn put_option<T, F>(
    out: &mut Vec<u8>,
    value: Option<&T>,
    encode: F,
) -> Result<(), ProjectInstallError>
where
    F: Fn(&mut Vec<u8>, &T) -> Result<(), ProjectInstallError>,
{
    put_bool(out, value.is_some());
    if let Some(value) = value {
        encode(out, value)?;
    }
    Ok(())
}

fn put_option_copy<T: Copy>(out: &mut Vec<u8>, value: Option<T>, encode: fn(&mut Vec<u8>, T)) {
    put_bool(out, value.is_some());
    if let Some(value) = value {
        encode(out, value);
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], ProjectInstallError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(ProjectInstallError::InvalidEncoding)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(ProjectInstallError::InvalidEncoding)?;
        self.position = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, ProjectInstallError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ProjectInstallError> {
        let mut bytes = [0; 2];
        bytes.copy_from_slice(self.take(2)?);
        Ok(u16::from_le_bytes(bytes))
    }

    fn u32(&mut self) -> Result<u32, ProjectInstallError> {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(self.take(4)?);
        Ok(u32::from_le_bytes(bytes))
    }

    fn u64(&mut self) -> Result<u64, ProjectInstallError> {
        let mut bytes = [0; 8];
        bytes.copy_from_slice(self.take(8)?);
        Ok(u64::from_le_bytes(bytes))
    }

    fn boolean(&mut self) -> Result<bool, ProjectInstallError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ProjectInstallError::InvalidEncoding),
        }
    }

    fn hash(&mut self) -> Result<[u8; 32], ProjectInstallError> {
        let mut value = [0; 32];
        value.copy_from_slice(self.take(32)?);
        Ok(value)
    }

    fn optional_hash(&mut self) -> Result<Option<[u8; 32]>, ProjectInstallError> {
        let present = self.boolean()?;
        let value = self.hash()?;
        if present {
            Ok(Some(value))
        } else if value == [0; 32] {
            Ok(None)
        } else {
            Err(ProjectInstallError::InvalidEncoding)
        }
    }

    fn string(&mut self) -> Result<String, ProjectInstallError> {
        let len = self.u16()? as usize;
        let value = core::str::from_utf8(self.take(len)?)
            .map_err(|_| ProjectInstallError::InvalidEncoding)?;
        Ok(String::from(value))
    }

    fn bytes(&mut self) -> Result<Vec<u8>, ProjectInstallError> {
        let len = self.u16()? as usize;
        Ok(Vec::from(self.take(len)?))
    }

    fn option<T, F>(&mut self, decode: F) -> Result<Option<T>, ProjectInstallError>
    where
        F: Fn(&mut Cursor<'a>) -> Result<T, ProjectInstallError>,
    {
        if self.boolean()? {
            Ok(Some(decode(self)?))
        } else {
            Ok(None)
        }
    }

    fn done(&self) -> bool {
        self.position == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn schema() -> ProjectAppStateSchema {
        seal_state_schema(ProjectAppStateSchema {
            schema_id: String::from("raios.app_state.stateless.v1"),
            version: 1,
            max_state_bytes: 0,
            migration_policy: StateMigrationPolicy::Denied,
            schema_sha256: [0; 32],
        })
        .unwrap()
    }

    fn blob(payload: [u8; 32], offset: u64) -> ContentBlobRef {
        ContentBlobRef {
            payload_sha256: payload,
            frame_sha256: hash((offset as u8).wrapping_add(40)),
            offset,
            frame_len: 4096,
        }
    }

    fn envelope(
        generation: u64,
        previous: Option<&ProjectInstallCommit>,
        candidate: u8,
    ) -> ProjectInstallEnvelope {
        seal_install_envelope(ProjectInstallEnvelope {
            service_id: String::from("svc.workspace.installed"),
            project_id: [1; 16],
            project_revision_sha256: hash(2),
            input_manifest_sha256: hash(3),
            build_receipt_sha256: hash(candidate.wrapping_add(4)),
            encoded_receipt_sha256: hash(candidate.wrapping_add(5)),
            workspace_run_binding_sha256: hash(candidate.wrapping_add(6)),
            candidate_sha256: hash(candidate),
            candidate_byte_len: 64,
            import_list_sha256: hash(7),
            import_count: 0,
            entrypoint: String::from("raios_service_main"),
            fuel_budget: 250_000,
            memory_limit_bytes: 4 * 1024 * 1024,
            instance_limit: 1,
            memory_count_limit: 1,
            table_limit: 0,
            state_schema: schema(),
            previous_install_commit_sha256: previous.map(|item| item.commit_sha256),
            previous_artifact_sha256: previous
                .and_then(|item| item.envelope.as_ref())
                .map(|item| item.candidate_sha256),
            generation,
            auto_start: true,
            trust_tier: String::from(PROJECT_INSTALL_TRUST_TIER),
            envelope_sha256: [0; 32],
        })
        .unwrap()
    }

    fn action(
        kind: ProjectInstallActionKind,
        authority: ProjectInstallAuthority,
        generation: u64,
        sequence: u64,
        previous: Option<&ProjectInstallCommit>,
        envelope: Option<&ProjectInstallEnvelope>,
        target: Option<&ProjectInstallCommit>,
    ) -> ProjectInstallAction {
        let physical = authority == ProjectInstallAuthority::PhysicalOwner;
        seal_install_action(ProjectInstallAction {
            kind,
            authority,
            service_id: String::from("svc.workspace.installed"),
            generation,
            log_sequence: sequence,
            previous_commit_sha256: previous.map(|item| item.commit_sha256),
            install_envelope_sha256: envelope.map(|item| item.envelope_sha256),
            target_install_commit_sha256: target.map(|item| item.commit_sha256),
            authority_evidence_sha256: hash(20),
            physical_approval_sha256: physical.then(|| hash(21)),
            authority_key_sha256: physical.then(|| hash(22)),
            authority_signature: if physical {
                alloc::vec![1, 2, 3]
            } else {
                Vec::new()
            },
            action_sha256: [0; 32],
        })
        .unwrap()
    }

    fn install(
        generation: u64,
        sequence: u64,
        previous_head: Option<&ProjectInstallCommit>,
        previous_install: Option<&ProjectInstallCommit>,
        candidate: u8,
    ) -> ProjectInstallCommit {
        let envelope = envelope(generation, previous_install, candidate);
        let action = action(
            ProjectInstallActionKind::Install,
            ProjectInstallAuthority::PhysicalOwner,
            generation,
            sequence,
            previous_head,
            Some(&envelope),
            None,
        );
        let intent = seal_install_intent(ProjectInstallIntent {
            action: action.clone(),
            envelope: envelope.clone(),
            intent_sha256: [0; 32],
        })
        .unwrap();
        seal_install_commit(ProjectInstallCommit {
            action,
            install_intent_sha256: Some(intent.intent_sha256),
            candidate_blob: Some(blob(envelope.candidate_sha256, sequence * 8192)),
            receipt_blob: Some(blob(
                envelope.encoded_receipt_sha256,
                sequence * 8192 + 4096,
            )),
            envelope: Some(envelope),
            commit_sha256: [0; 32],
        })
        .unwrap()
    }

    fn non_install(
        kind: ProjectInstallActionKind,
        authority: ProjectInstallAuthority,
        generation: u64,
        sequence: u64,
        previous: &ProjectInstallCommit,
        target: Option<&ProjectInstallCommit>,
    ) -> ProjectInstallCommit {
        seal_install_commit(ProjectInstallCommit {
            action: action(
                kind,
                authority,
                generation,
                sequence,
                Some(previous),
                None,
                target,
            ),
            install_intent_sha256: None,
            envelope: None,
            candidate_blob: None,
            receipt_blob: None,
            commit_sha256: [0; 32],
        })
        .unwrap()
    }

    #[test]
    fn install_success_round_trips_and_resolves_active() {
        let install = install(1, 10, None, None, 30);
        let intent = seal_install_intent(ProjectInstallIntent {
            action: install.action.clone(),
            envelope: install.envelope.clone().unwrap(),
            intent_sha256: [0; 32],
        })
        .unwrap();
        let attempt = non_install(
            ProjectInstallActionKind::ActivationAttempt,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            11,
            &install,
            Some(&install),
        );
        let success = non_install(
            ProjectInstallActionKind::ActivationSuccess,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            12,
            &attempt,
            Some(&install),
        );
        let encoded = encode_install_commit(&install).unwrap();
        assert_eq!(decode_install_commit(&encoded).unwrap(), install);
        let encoded_intent = encode_install_intent(&intent).unwrap();
        assert_eq!(decode_install_intent(&encoded_intent).unwrap(), intent);

        let resolved = resolve_install_log(&[install.clone(), attempt, success.clone()]).unwrap();
        assert!(resolved.installed);
        assert_eq!(
            resolved.active_install_commit_sha256,
            Some(install.commit_sha256)
        );
        assert_eq!(
            resolved.last_good_install_commit_sha256,
            Some(install.commit_sha256)
        );
        assert_eq!(resolved.head_commit_sha256, Some(success.commit_sha256));
        assert_eq!(resolved.candidate_sha256, Some(hash(30)));
    }

    #[test]
    fn failed_probation_rolls_back_to_previous_last_good() {
        let first = install(1, 10, None, None, 30);
        let first_attempt = non_install(
            ProjectInstallActionKind::ActivationAttempt,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            11,
            &first,
            Some(&first),
        );
        let first_success = non_install(
            ProjectInstallActionKind::ActivationSuccess,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            12,
            &first_attempt,
            Some(&first),
        );
        let second = install(2, 20, Some(&first_success), Some(&first), 31);
        let second_attempt = non_install(
            ProjectInstallActionKind::ActivationAttempt,
            ProjectInstallAuthority::RuntimeHealth,
            2,
            21,
            &second,
            Some(&second),
        );
        let rollback = non_install(
            ProjectInstallActionKind::Rollback,
            ProjectInstallAuthority::RecoveryFailure,
            2,
            22,
            &second_attempt,
            Some(&first),
        );

        let resolved = resolve_install_log(&[
            first.clone(),
            first_attempt,
            first_success,
            second,
            second_attempt,
            rollback,
        ])
        .unwrap();
        assert!(resolved.installed);
        assert!(resolved.rollback_applied);
        assert_eq!(
            resolved.active_install_commit_sha256,
            Some(first.commit_sha256)
        );
        assert_eq!(resolved.candidate_sha256, Some(hash(30)));
        assert_eq!(resolved.probation_install_commit_sha256, None);
    }

    #[test]
    fn uninstall_is_a_durable_tombstone() {
        let install = install(1, 10, None, None, 30);
        let attempt = non_install(
            ProjectInstallActionKind::ActivationAttempt,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            11,
            &install,
            Some(&install),
        );
        let success = non_install(
            ProjectInstallActionKind::ActivationSuccess,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            12,
            &attempt,
            Some(&install),
        );
        let uninstall = non_install(
            ProjectInstallActionKind::Uninstall,
            ProjectInstallAuthority::PhysicalOwner,
            1,
            14,
            &success,
            None,
        );

        let resolved = resolve_install_log(&[install, attempt, success, uninstall]).unwrap();
        assert!(!resolved.installed);
        assert!(resolved.uninstall_tombstoned);
        assert_eq!(resolved.candidate_sha256, None);
        assert_eq!(resolved.active_install_commit_sha256, None);
    }

    #[test]
    fn fork_sequence_and_target_tampering_deny() {
        let install = install(1, 10, None, None, 30);
        let mut fork = non_install(
            ProjectInstallActionKind::ActivationSuccess,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            12,
            &install,
            Some(&install),
        );
        fork.action.previous_commit_sha256 = Some(hash(99));
        fork.action = seal_install_action(fork.action).unwrap();
        fork.commit_sha256 = [0; 32];
        fork = seal_install_commit(fork).unwrap();
        assert_eq!(
            resolve_install_log(&[install.clone(), fork]),
            Err(ProjectInstallError::ChainFork)
        );

        let attempt = non_install(
            ProjectInstallActionKind::ActivationAttempt,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            11,
            &install,
            Some(&install),
        );
        let stale_sequence = non_install(
            ProjectInstallActionKind::ActivationSuccess,
            ProjectInstallAuthority::RuntimeHealth,
            1,
            11,
            &attempt,
            Some(&install),
        );
        assert_eq!(
            resolve_install_log(&[install, attempt, stale_sequence]),
            Err(ProjectInstallError::SequenceNotIncreasing)
        );
    }

    #[test]
    fn envelope_action_commit_and_encoded_tampering_deny() {
        let install = install(1, 10, None, None, 30);

        let signed_payload = install_action_signature_payload(&install.action).unwrap();
        let mut unsigned_action = install.action.clone();
        unsigned_action.authority_signature.clear();
        unsigned_action.action_sha256 = [0; 32];
        assert_eq!(
            install_action_signature_payload(&unsigned_action).unwrap(),
            signed_payload
        );

        let mut envelope_tampered = install.clone();
        envelope_tampered
            .envelope
            .as_mut()
            .unwrap()
            .candidate_byte_len += 1;
        assert_eq!(
            validate_install_commit(&envelope_tampered),
            Err(ProjectInstallError::InvalidHash)
        );

        let mut action_tampered = install.clone();
        action_tampered.action.authority_evidence_sha256 = hash(90);
        assert_eq!(
            validate_install_commit(&action_tampered),
            Err(ProjectInstallError::InvalidHash)
        );

        let mut commit_tampered = install.clone();
        commit_tampered.candidate_blob.as_mut().unwrap().offset += 8192;
        assert_eq!(
            validate_install_commit(&commit_tampered),
            Err(ProjectInstallError::InvalidHash)
        );

        let mut overlapping = install.clone();
        overlapping.receipt_blob.as_mut().unwrap().offset =
            overlapping.candidate_blob.unwrap().offset;
        overlapping.commit_sha256 = [0; 32];
        assert_eq!(
            seal_install_commit(overlapping),
            Err(ProjectInstallError::InvalidField)
        );

        let mut bytes = encode_install_commit(&install).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert_eq!(
            decode_install_commit(&bytes),
            Err(ProjectInstallError::InvalidHash)
        );
    }
}
