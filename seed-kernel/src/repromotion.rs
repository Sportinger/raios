use alloc::{vec, vec::Vec};
use core::{
    str,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    ahci, event_log, granted_candidate_service, module_candidate_intake,
    module_evidence::{self, ModuleLocalAttestationReferenceHashInput},
    pci, serial, ui, wasm_runtime,
};
use raios_core::{
    boot_control::BootPosture,
    durable_record_frame::{
        parse_reclog_frame, plan_reclog_append, scan_reclog, PlannedAppend, RECLOG_FRAME_HEADER_LEN,
    },
    promotion_attestation::{
        PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256, PROMOTION_AUTHORITY_IS_PLACEHOLDER,
    },
    record::{write_json, Field, Value as V},
    repromotion_reverify::{
        reverify_persisted_artifact, RepromotionReverifyInput, RepromotionTransactionFields,
    },
    scoped_repromotion_append::{
        evaluate_scoped_repromotion_append, ScopedRepromotionAppendInput,
        EXPECTED_METHOD as REPROMOTION_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as REPROMOTION_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as REPROMOTION_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as REPROMOTION_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as REPROMOTION_EXPECTED_TRUST_TIER,
    },
    sha256_bytes, ByteSink,
};

use super::{artifact_store, boot_control, durable_store};

const METHOD: &str = "repromotion.run";
const RESPONSE_SCHEMA: &str = "raios.repromotion.v0";
const RESPONSE_ID: &str = "repromotion.current_boot.seed_data.v0";
const DECISION_ID: &str = "repromotion.current_boot.svc.dev.granted_candidate.v0";
const SCOPE: &str = "current_boot";
const CLASSIFICATION: &str = "local_only";
const TRUST_TIER: &str = "dev_key_not_owner_sealed";
const GRANTED_CANDIDATE_SERVICE_ID: &str = "svc.dev.granted_candidate";
const MAX_PROMOTION_SIGNATURE_DER_LEN: usize = 80;
static PROVIDER_RECOVERY_CONSUMED: AtomicBool = AtomicBool::new(false);

struct VecSink(Vec<u8>);

impl ByteSink for VecSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

#[derive(Clone, Copy)]
struct EventIdText {
    bytes: [u8; 27],
}

impl EventIdText {
    fn parse(value: &str) -> Option<Self> {
        raios_core::parse_current_boot_event_sequence(value)?;
        if value.len() != 27 {
            return None;
        }
        let mut bytes = [0u8; 27];
        bytes.copy_from_slice(value.as_bytes());
        Some(Self { bytes })
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes) }
    }

    fn event_id(&self) -> Option<event_log::EventId> {
        event_log::EventId::from_sequence(raios_core::parse_current_boot_event_sequence(
            self.as_str(),
        )?)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PromotionTransactionReadback {
    transaction_kind: &'static str,
    computed_grant_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
    retained_manifest_reference_event_id: EventIdText,
    retained_artifact_reference_event_id: EventIdText,
    retained_vm_report_reference_event_id: EventIdText,
    retained_computed_grant_reference_event_id: EventIdText,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    vm_report_reference_hash: [u8; 32],
    attestation_reference_hash: [u8; 32],
    promotion_signature_der: [u8; MAX_PROMOTION_SIGNATURE_DER_LEN],
    promotion_signature_len: usize,
    promotion_authority_key_sha256: [u8; 32],
    signature_verified: bool,
    grant_binds_capability: bool,
    install_authorization_present: bool,
    install_envelope_binds_activation: bool,
    install_action_signature_verified: bool,
    physical_install_approval_consumed: bool,
    install_authorization_frame_sha256: [u8; 32],
    install_envelope_version: u16,
    grant_target_schema: &'static str,
    grant_target_count: u64,
    grant_target_snapshot_sha256: [u8; 32],
    rollback_plan_hash: [u8; 32],
    pre_load_inventory_hash: [u8; 32],
    ram_only_service_slot_id: &'static str,
    generation: u64,
    load_event_id: EventIdText,
    install_authorization: Option<granted_candidate_service::SignedInstallAuthorization>,
    frame_sha256: [u8; 32],
}

#[derive(Clone, Copy)]
struct InstallAuthorizationReadback {
    authorization: granted_candidate_service::SignedInstallAuthorization,
    frame_sha256: [u8; 32],
}

#[derive(Clone, Copy)]
pub(crate) enum GrantedCandidateInstallResolution {
    None,
    Incomplete,
    Active {
        transaction: PromotionTransactionReadback,
        artifact: artifact_store::ArtifactPersistRecord,
        artifact_persist_frame_sha256: [u8; 32],
    },
    RolledBack {
        transaction: PromotionTransactionReadback,
    },
}

#[derive(Clone, Copy)]
enum GrantedCandidateInstallFold {
    None,
    Incomplete,
    Promote {
        transaction: PromotionTransactionReadback,
        artifact: Option<(artifact_store::ArtifactPersistRecord, [u8; 32])>,
    },
    Unpromote {
        transaction: PromotionTransactionReadback,
    },
}

#[derive(Clone, Copy)]
struct AuditAppendEvidence {
    performed: bool,
    status: &'static str,
    reason: &'static str,
    authority: &'static str,
    seq: Option<u64>,
    write_offset: Option<u64>,
    frame_len: Option<u64>,
    payload_sha256: Option<[u8; 32]>,
    frame_sha256: Option<[u8; 32]>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
}

impl AuditAppendEvidence {
    fn denied(reason: &'static str) -> Self {
        Self {
            performed: false,
            status: "capability_denied",
            reason,
            authority: "evidence_only",
            seq: None,
            write_offset: None,
            frame_len: None,
            payload_sha256: None,
            frame_sha256: None,
            readback_sha256: None,
            reparse_valid: false,
        }
    }
}

#[derive(Clone, Copy)]
struct RepromotionEvidence {
    artifact_persist_seq: u64,
    artifact_persist_reclog_offset: u64,
    artstor_blob_offset: u64,
    artstor_blob_len: u64,
    artstor_blob_frame_sha256: [u8; 32],
    artifact_sha256: [u8; 32],
    manifest_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    grant_hash: [u8; 32],
    promotion_transaction_sha256: [u8; 32],
    transaction_frame_sha256: Option<[u8; 32]>,
    candidate_payload_sha256: Option<[u8; 32]>,
    candidate_payload_len: Option<u64>,
    recomputed_attestation_reference_hash: Option<[u8; 32]>,
    recorded_attestation_reference_hash: Option<[u8; 32]>,
    promotion_authority_key_sha256: Option<[u8; 32]>,
    promotion_signature_len: Option<u64>,
    recorded_signature_verified: Option<bool>,
    grant_binds_capability: Option<bool>,
    reconstructed_wasm_valid: Option<bool>,
    retained_candidate: bool,
    references_repopulated: bool,
    load_attempted: bool,
    load_granted: bool,
    service_loaded: bool,
    load_reason: Option<&'static str>,
    start_attempted: bool,
    service_started: bool,
    start_reason: Option<&'static str>,
    start_run_outcome: Option<&'static str>,
    status: &'static str,
    reason: &'static str,
    performed: bool,
    would_repromote: bool,
    authorizes_load: bool,
    trust_tier: &'static str,
    owner_sealed: bool,
    cross_reboot_proven: bool,
    persistence_claimed: bool,
    audit: Option<AuditAppendEvidence>,
}

impl RepromotionEvidence {
    fn deny(&mut self, reason: &'static str) {
        self.status = "repromotion_denied";
        self.reason = reason;
        self.performed = false;
        self.would_repromote = false;
        self.authorizes_load = false;
        self.cross_reboot_proven = false;
    }
}

pub(crate) fn emit_repromotion_run(_runtime: ui::RuntimeStatus) {
    match boot_control::current_boot_posture() {
        BootPosture::Safe | BootPosture::PersistenceUnavailable => {
            emit_skipped("repromotion_skipped_in_safe");
            return;
        }
        BootPosture::Normal | BootPosture::Probation => {}
    }

    let Some(controller) = pci::find_mass_storage_controller() else {
        emit_no_record("ahci_controller_not_observed", "capability_denied", false);
        return;
    };
    let scan = artifact_store::current_boot_reclog_scan(controller);
    let Some(bytes) = scan.bytes.as_deref() else {
        emit_no_record(scan.reason, "capability_denied", false);
        return;
    };
    let artifact = match resolve_granted_candidate_install(bytes) {
        GrantedCandidateInstallResolution::None => {
            emit_no_record("no_w6_authorized_install", "no_artifacts", true);
            return;
        }
        GrantedCandidateInstallResolution::Incomplete => {
            emit_no_record("install_history_incomplete", "capability_denied", false);
            return;
        }
        GrantedCandidateInstallResolution::RolledBack { .. } => {
            emit_no_record(
                "granted_candidate_install_rolled_back",
                "no_artifacts",
                true,
            );
            return;
        }
        GrantedCandidateInstallResolution::Active { artifact, .. } => artifact,
    };

    let ReverifyOnly { mut evidence, .. } = reverify_record_only(controller, &artifact);
    let audit = append_repromotion_audit(controller, &evidence);
    evidence.audit = Some(audit);
    let mut decisions = Vec::new();
    decisions.push(evidence);
    emit_run_record("completed", "repromotion_scan_completed", true, decisions);
}

pub(crate) fn resolve_granted_candidate_install(bytes: &[u8]) -> GrantedCandidateInstallResolution {
    let scan = scan_reclog(bytes);
    if !scan.valid_prefix_chain || !scan.full_region_valid {
        return GrantedCandidateInstallResolution::Incomplete;
    }
    let artifacts = artifact_store::artifact_persist_records_from_reclog(bytes);
    let mut install_authorizations = Vec::new();
    let mut fold = GrantedCandidateInstallFold::None;
    let mut offset = 0usize;
    let mut seq = 1u64;
    let mut prev = [0u8; 32];
    let mut count = 0u64;
    while count < scan.count && offset < bytes.len() {
        let Ok(frame) = parse_reclog_frame(bytes, offset, seq, prev) else {
            return GrantedCandidateInstallResolution::Incomplete;
        };
        let payload_start = offset + RECLOG_FRAME_HEADER_LEN;
        let payload_end = payload_start + frame.payload_len as usize;
        let Ok(payload) = str::from_utf8(&bytes[payload_start..payload_end]) else {
            return GrantedCandidateInstallResolution::Incomplete;
        };
        let target_service =
            extract_str(payload, b"\"service_id\": \"") == Some(GRANTED_CANDIDATE_SERVICE_ID);
        let target_record = target_service
            || contains_bytes(payload.as_bytes(), GRANTED_CANDIDATE_SERVICE_ID.as_bytes());

        if target_record
            && contains_bytes(
                payload.as_bytes(),
                b"\"schema\": \"raios.install_authorization.v0\"",
            )
        {
            fold = GrantedCandidateInstallFold::Incomplete;
            if let Ok(mut authorization) = parse_install_authorization_payload(payload) {
                authorization.frame_sha256 = frame.frame_sha256;
                install_authorizations.push(authorization);
            }
        } else if target_record
            && contains_bytes(
                payload.as_bytes(),
                b"\"schema\": \"raios.promotion_transaction.v0\"",
            )
        {
            fold = match parse_promotion_transaction_payload(payload) {
                Ok(mut transaction) => {
                    if link_install_authorization(&mut transaction, &install_authorizations)
                        && complete_install_authorization(&transaction)
                    {
                        transaction.frame_sha256 = frame.frame_sha256;
                        if transaction.transaction_kind == "promote" {
                            GrantedCandidateInstallFold::Promote {
                                transaction,
                                artifact: None,
                            }
                        } else {
                            GrantedCandidateInstallFold::Unpromote { transaction }
                        }
                    } else {
                        GrantedCandidateInstallFold::Incomplete
                    }
                }
                _ => GrantedCandidateInstallFold::Incomplete,
            };
        } else if target_record
            && contains_bytes(
                payload.as_bytes(),
                b"\"schema\": \"raios.artifact_persist.v0\"",
            )
        {
            let artifact = artifacts
                .iter()
                .find(|record| record.seq == frame.seq)
                .copied();
            fold = match (fold, artifact) {
                (
                    GrantedCandidateInstallFold::Promote {
                        transaction,
                        artifact: None,
                    },
                    Some(artifact),
                ) if exact_artifact_link(&transaction, &artifact) => {
                    GrantedCandidateInstallFold::Promote {
                        transaction,
                        artifact: Some((artifact, frame.frame_sha256)),
                    }
                }
                _ => GrantedCandidateInstallFold::Incomplete,
            };
        }

        offset += frame.frame_len as usize;
        seq = seq.saturating_add(1);
        prev = frame.frame_sha256;
        count += 1;
    }

    match fold {
        GrantedCandidateInstallFold::Promote {
            transaction,
            artifact: Some((artifact, artifact_persist_frame_sha256)),
        } => GrantedCandidateInstallResolution::Active {
            transaction,
            artifact,
            artifact_persist_frame_sha256,
        },
        GrantedCandidateInstallFold::Unpromote { transaction } => {
            GrantedCandidateInstallResolution::RolledBack { transaction }
        }
        GrantedCandidateInstallFold::None => GrantedCandidateInstallResolution::None,
        GrantedCandidateInstallFold::Incomplete
        | GrantedCandidateInstallFold::Promote { artifact: None, .. } => {
            GrantedCandidateInstallResolution::Incomplete
        }
    }
}

fn complete_install_authorization(transaction: &PromotionTransactionReadback) -> bool {
    transaction.install_authorization_present
        && transaction.install_envelope_binds_activation
        && transaction.install_action_signature_verified
        && transaction.physical_install_approval_consumed
        && transaction.install_authorization_frame_sha256 != [0; 32]
        && transaction.install_envelope_version
            == raios_core::project_install::GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION
        && transaction.grant_target_schema
            == raios_core::wasm_import_grant_event::GRANT_TARGET_SNAPSHOT_SCHEMA
        && transaction.grant_target_count
            <= raios_core::wasm_import_grant_event::MAX_PROJECTION_SLOTS as u64
        && transaction.grant_target_snapshot_sha256 != [0; 32]
        && transaction
            .install_authorization
            .is_some_and(|authorization| {
                authorization.candidate_sha256 == transaction.artifact_hash
                    && authorization.computed_grant_sha256 == transaction.computed_grant_hash
                    && authorization.attestation_reference_sha256
                        == transaction.attestation_reference_hash
                    && authorization.install_envelope_version
                        == transaction.install_envelope_version
                    && authorization.grant_target_schema == transaction.grant_target_schema
                    && authorization.grant_target_count == transaction.grant_target_count
                    && authorization.grant_target_snapshot_sha256
                        == transaction.grant_target_snapshot_sha256
                    && authorization.generation == transaction.generation
            })
}

fn link_install_authorization(
    transaction: &mut PromotionTransactionReadback,
    records: &[InstallAuthorizationReadback],
) -> bool {
    let Some(readback) = records
        .iter()
        .find(|record| record.frame_sha256 == transaction.install_authorization_frame_sha256)
        .copied()
    else {
        return false;
    };
    let record = durable_store::InstallAuthorizationRecord {
        service_id: GRANTED_CANDIDATE_SERVICE_ID,
        trust_tier: TRUST_TIER,
        authorization: readback.authorization,
    };
    if record.authorization.candidate_sha256 != transaction.artifact_hash
        || record.authorization.computed_grant_sha256 != transaction.computed_grant_hash
        || record.authorization.attestation_reference_sha256
            != transaction.attestation_reference_hash
        || durable_store::validate_signed_install_authorization(&record).is_err()
    {
        return false;
    }
    transaction.install_authorization = Some(record.authorization);
    true
}

fn exact_artifact_link(
    transaction: &PromotionTransactionReadback,
    artifact: &artifact_store::ArtifactPersistRecord,
) -> bool {
    signed_snapshot_context_matches_artifact(transaction, artifact)
        && artifact.promotion_transaction_sha256 == transaction.frame_sha256
        && artifact.artifact_sha256 == transaction.artifact_hash
        && artifact.manifest_hash == transaction.manifest_hash
        && artifact.vm_report_hash == transaction.vm_report_hash
        && artifact.grant_hash == transaction.computed_grant_hash
        && !artifact.authorizes_load
}

fn signed_snapshot_context_matches_artifact(
    transaction: &PromotionTransactionReadback,
    artifact: &artifact_store::ArtifactPersistRecord,
) -> bool {
    transaction.install_envelope_version
        == raios_core::project_install::GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION
        && transaction.grant_target_schema
            == raios_core::wasm_import_grant_event::GRANT_TARGET_SNAPSHOT_SCHEMA
        && artifact.grant_target_schema == transaction.grant_target_schema
        && artifact.service_id == GRANTED_CANDIDATE_SERVICE_ID
        && artifact.artifact_sha256 == transaction.artifact_hash
        && artifact.grant_target_count == transaction.grant_target_count
        && artifact.grant_target_snapshot_sha256 == transaction.grant_target_snapshot_sha256
        && transaction
            .install_authorization
            .is_some_and(|authorization| {
                authorization.install_envelope_version == transaction.install_envelope_version
                    && authorization.grant_target_schema == transaction.grant_target_schema
                    && authorization.grant_target_count == transaction.grant_target_count
                    && authorization.grant_target_snapshot_sha256
                        == transaction.grant_target_snapshot_sha256
                    && authorization.candidate_sha256 == transaction.artifact_hash
                    && authorization.computed_grant_sha256 == transaction.computed_grant_hash
                    && authorization.attestation_reference_sha256
                        == transaction.attestation_reference_hash
            })
}

/// Reverify-only outcome shared by the full re-promotion path and the recovery lifeline
/// (M8D-1). `evidence` carries the decision (`reverified` == `performed`) and every
/// reverify-evidence hash; `verified_payload`/`transaction` are `Some` ONLY when the chain
/// re-verified, and are consumed by the full path to continue to the M6 gate. Grants
/// nothing by itself: it retains no candidate, repopulates no references, loads/starts
/// nothing — the caller decides what (if anything) to do next.
pub(crate) struct ReverifyOnly {
    evidence: RepromotionEvidence,
    verified_payload: Option<artifact_store::VerifiedArtifactPayload>,
    transaction: Option<PromotionTransactionReadback>,
}

impl ReverifyOnly {
    fn denied(evidence: RepromotionEvidence) -> Self {
        Self {
            evidence,
            verified_payload: None,
            transaction: None,
        }
    }
}

/// STEP 0..decision of a re-promotion: read the verified payload, find the promotion
/// transaction, recompute the attestation reference hash, and run the pure
/// `reverify_persisted_artifact` decision. This is the ONE reverify implementation both
/// the full re-promotion path (`reverify_record`, which continues to the M6 gate) and the
/// recovery lifeline (M8D-1 `recovery.load_artifact_by_hash`, which stops here and reports)
/// share, so neither can drift from the other's notion of "reverified". It grants nothing.
pub(crate) fn reverify_record_only(
    controller: pci::PciMassStorageController,
    record: &artifact_store::ArtifactPersistRecord,
) -> ReverifyOnly {
    let verified_payload = match artifact_store::read_verified_artstor_payload(controller, record) {
        Ok(verified) => verified,
        Err(reason) => return ReverifyOnly::denied(denied_from_record(record, reason)),
    };
    let transaction_scan = artifact_store::current_boot_reclog_scan(controller);
    let transaction = match transaction_scan
        .bytes
        .as_deref()
        .ok_or(transaction_scan.reason)
        .and_then(|bytes| find_promotion_transaction(bytes, record.promotion_transaction_sha256))
    {
        Ok(transaction) => transaction,
        Err(reason) => {
            let mut evidence = denied_from_record(record, reason);
            evidence.candidate_payload_sha256 = Some(sha256_bytes(&verified_payload.payload));
            evidence.candidate_payload_len = Some(verified_payload.payload.len() as u64);
            return ReverifyOnly::denied(evidence);
        }
    };
    if !signed_snapshot_context_matches_artifact(&transaction, record) {
        let mut evidence = denied_from_record(record, "signed_snapshot_context_mismatch");
        evidence.candidate_payload_sha256 = Some(sha256_bytes(&verified_payload.payload));
        evidence.candidate_payload_len = Some(verified_payload.payload.len() as u64);
        return ReverifyOnly::denied(evidence);
    }

    let recomputed_attestation_reference_hash =
        module_evidence::computed_module_local_attestation_reference_hash(
            ModuleLocalAttestationReferenceHashInput {
                retained_manifest_reference_event_id: transaction
                    .retained_manifest_reference_event_id
                    .as_str(),
                retained_artifact_reference_event_id: transaction
                    .retained_artifact_reference_event_id
                    .as_str(),
                retained_vm_report_reference_event_id: transaction
                    .retained_vm_report_reference_event_id
                    .as_str(),
                retained_reference_event_id: transaction
                    .retained_computed_grant_reference_event_id
                    .as_str(),
                manifest_reference_hash: transaction.manifest_reference_hash,
                artifact_reference_hash: transaction.artifact_reference_hash,
                vm_report_reference_hash: transaction.vm_report_reference_hash,
                manifest_hash: transaction.manifest_hash,
                artifact_hash: transaction.artifact_hash,
                computed_grant_hash: transaction.computed_grant_hash,
                vm_report_hash: transaction.vm_report_hash,
                local_attestation_hash: transaction.local_attestation_hash,
            },
        );
    let signature = &transaction.promotion_signature_der[..transaction.promotion_signature_len];
    let install_signature = transaction
        .install_authorization
        .as_ref()
        .map(|authorization| &authorization.signature_der[..authorization.signature_len])
        .unwrap_or(&[]);
    let decision = reverify_persisted_artifact(&RepromotionReverifyInput {
        artstor_blob_bytes: &verified_payload.blob_bytes,
        record_artstor_blob_frame_sha256: record.artstor_blob_frame_sha256,
        record_artifact_sha256: record.artifact_sha256,
        record_manifest_hash: record.manifest_hash,
        record_vm_report_hash: record.vm_report_hash,
        record_grant_hash: record.grant_hash,
        transaction: Some(RepromotionTransactionFields {
            computed_grant_hash: transaction.computed_grant_hash,
            manifest_hash: transaction.manifest_hash,
            artifact_hash: transaction.artifact_hash,
            vm_report_hash: transaction.vm_report_hash,
            local_attestation_hash: transaction.local_attestation_hash,
            attestation_reference_hash: transaction.attestation_reference_hash,
            promotion_signature_der: signature,
            promotion_authority_key_sha256: transaction.promotion_authority_key_sha256,
            signature_verified: transaction.signature_verified,
            grant_binds_capability: transaction.grant_binds_capability,
            install_authorization_present: transaction.install_authorization_present,
            install_envelope_binds_activation: transaction.install_envelope_binds_activation,
            install_action_signature_message_sha256: transaction
                .install_authorization
                .map(|a| a.install_action_message_sha256)
                .unwrap_or([0; 32]),
            install_action_signature_der: install_signature,
            install_authority_key_sha256: transaction
                .install_authorization
                .map(|a| a.authority_key_sha256)
                .unwrap_or([0; 32]),
            install_action_signature_verified: transaction.install_action_signature_verified,
            physical_install_approval_consumed: transaction.physical_install_approval_consumed,
        }),
        recomputed_attestation_reference_hash,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
    });

    let evidence = RepromotionEvidence {
        artifact_persist_seq: record.seq,
        artifact_persist_reclog_offset: record.reclog_offset,
        artstor_blob_offset: record.artstor_blob_offset,
        artstor_blob_len: record.artstor_blob_len,
        artstor_blob_frame_sha256: record.artstor_blob_frame_sha256,
        artifact_sha256: record.artifact_sha256,
        manifest_hash: record.manifest_hash,
        vm_report_hash: record.vm_report_hash,
        grant_hash: record.grant_hash,
        promotion_transaction_sha256: record.promotion_transaction_sha256,
        transaction_frame_sha256: Some(transaction.frame_sha256),
        candidate_payload_sha256: decision.candidate_payload_sha256,
        candidate_payload_len: Some(verified_payload.payload.len() as u64),
        recomputed_attestation_reference_hash: Some(recomputed_attestation_reference_hash),
        recorded_attestation_reference_hash: Some(transaction.attestation_reference_hash),
        promotion_authority_key_sha256: Some(transaction.promotion_authority_key_sha256),
        promotion_signature_len: Some(transaction.promotion_signature_len as u64),
        recorded_signature_verified: Some(transaction.signature_verified),
        grant_binds_capability: Some(transaction.grant_binds_capability),
        reconstructed_wasm_valid: None,
        retained_candidate: false,
        references_repopulated: false,
        load_attempted: false,
        load_granted: false,
        service_loaded: false,
        load_reason: None,
        start_attempted: false,
        service_started: false,
        start_reason: None,
        start_run_outcome: None,
        status: decision.status,
        reason: decision.reason,
        performed: decision.reverified,
        would_repromote: decision.reverified,
        authorizes_load: false,
        trust_tier: TRUST_TIER,
        owner_sealed: false,
        cross_reboot_proven: false,
        persistence_claimed: false,
        audit: None,
    };
    if !decision.reverified {
        return ReverifyOnly {
            evidence,
            verified_payload: None,
            transaction: None,
        };
    }
    ReverifyOnly {
        evidence,
        verified_payload: Some(verified_payload),
        transaction: Some(transaction),
    }
}

#[derive(Clone)]
pub(crate) struct RollbackTargetPreflight {
    pub(crate) authorized: bool,
    pub(crate) reason: &'static str,
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) promotion_transaction_sha256: [u8; 32],
    pub(crate) install_action_sha256: [u8; 32],
    pub(crate) generation: u64,
    pub(crate) log_sequence: u64,
    pub(crate) envelope: Option<raios_core::project_install::GrantedCandidateInstallEnvelope>,
}

/// Read-only projection of the existing re-promotion verifier for rollback.
/// It neither retains bytes nor repopulates references nor invokes the M6 gate.
pub(crate) fn preflight_rollback_target(
    controller: pci::PciMassStorageController,
    record: &artifact_store::ArtifactPersistRecord,
) -> RollbackTargetPreflight {
    let verified = reverify_record_only(controller, record);
    let denied = |reason| RollbackTargetPreflight {
        authorized: false,
        reason,
        artifact_sha256: verified.evidence.artifact_sha256,
        promotion_transaction_sha256: verified.evidence.promotion_transaction_sha256,
        install_action_sha256: [0; 32],
        generation: 0,
        log_sequence: 0,
        envelope: None,
    };
    if !verified.evidence.performed {
        return denied(verified.evidence.reason);
    }
    let (Some(payload), Some(transaction)) = (
        verified.verified_payload.as_ref(),
        verified.transaction.as_ref(),
    ) else {
        return denied("rollback_target_reverification_incomplete");
    };
    if !signed_snapshot_context_matches_artifact(transaction, record) {
        return denied("signed_snapshot_context_mismatch");
    }
    let Ok(snapshot) = record.grant_target_snapshot() else {
        return denied("grant_target_snapshot_tampered");
    };
    if !wasm_runtime::validate_module_bytes(&payload.payload) {
        return denied("rollback_target_wasm_invalid");
    }
    let observed_import_mask = match inspect_recovery_import_mask(&payload.payload) {
        Ok(mask) => mask,
        Err(reason) => return denied(reason),
    };
    if !artifact_store::grant_target_inventory_matches(
        record.grant_target_surface_mask,
        record.import_set_hash,
        snapshot.entry_count,
        observed_import_mask,
    ) {
        return denied("rollback_target_import_inventory_mismatch");
    }
    let Some(authorization) = transaction.install_authorization else {
        return denied("rollback_target_reverification_incomplete");
    };
    let authorization_record = durable_store::InstallAuthorizationRecord {
        service_id: GRANTED_CANDIDATE_SERVICE_ID,
        trust_tier: TRUST_TIER,
        authorization,
    };
    let envelope =
        match durable_store::verified_granted_candidate_install_envelope(&authorization_record) {
            Ok(envelope) => envelope,
            Err(reason) => return denied(reason),
        };
    if envelope.candidate_byte_len != payload.payload.len() as u64
        || raios_core::project_install::validate_granted_candidate_install_target(
            &envelope, &snapshot,
        )
        .is_err()
    {
        return denied("signed_snapshot_context_mismatch");
    }
    RollbackTargetPreflight {
        authorized: true,
        reason: verified.evidence.reason,
        artifact_sha256: verified.evidence.artifact_sha256,
        promotion_transaction_sha256: verified.evidence.promotion_transaction_sha256,
        install_action_sha256: authorization.install_action_sha256,
        generation: authorization.generation,
        log_sequence: authorization.log_sequence,
        envelope: Some(envelope),
    }
}

pub(crate) fn inspect_recovery_import_mask(bytes: &[u8]) -> Result<u8, &'static str> {
    match wasm_runtime::inspect_granted_candidate_import_mask(bytes) {
        Ok(mask) => Ok(mask),
        Err("rollback_target_import_inventory_empty") => {
            let inspection = wasm_runtime::inspect_workspace_imports(bytes);
            if inspection.validation_ok
                && inspection.import_list_observed
                && inspection.import_count == 0
            {
                Ok(0)
            } else {
                Err("rollback_target_import_inventory_mismatch")
            }
        }
        Err(reason) => Err(reason),
    }
}

/// Full re-promotion of one persisted record: the shared reverify-only chain FIRST, then —
/// ONLY if it re-verified — validate the reconstructed wasm, retain the candidate,
/// repopulate references, and drive the shared M6 load/start core through its internal
/// recovery-only wrappers. The serial protocol routes cannot reach these wrappers; this
/// record already crossed physical install authority before it became recoverable.
fn claim_recovery_attempt(guard: &AtomicBool) -> bool {
    guard
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

pub(crate) fn recovery_guard_selftest() -> bool {
    let first_boot = AtomicBool::new(false);
    let first_allowed = claim_recovery_attempt(&first_boot);
    let second_denied = !claim_recovery_attempt(&first_boot);
    let fresh_boot = AtomicBool::new(false);
    first_allowed && second_denied && claim_recovery_attempt(&fresh_boot)
}

fn reverify_recovery_record(
    controller: pci::PciMassStorageController,
    record: &artifact_store::ArtifactPersistRecord,
) -> RepromotionEvidence {
    if !claim_recovery_attempt(&PROVIDER_RECOVERY_CONSUMED) {
        return denied_from_record(record, "provider_recovery_already_consumed");
    }
    let evidence = reverify_record_ram_only(controller, record);
    if evidence.status != "repromoted"
        && (evidence.retained_candidate
            || evidence.references_repopulated
            || evidence.load_attempted
            || evidence.start_attempted)
    {
        granted_candidate_service::clear_reverified_install_state();
    }
    evidence
}

fn reverify_record_ram_only(
    controller: pci::PciMassStorageController,
    record: &artifact_store::ArtifactPersistRecord,
) -> RepromotionEvidence {
    let ReverifyOnly {
        mut evidence,
        verified_payload,
        transaction,
    } = reverify_record_only(controller, record);
    let (Some(verified_payload), Some(transaction)) = (verified_payload, transaction) else {
        return evidence;
    };

    let payload_sha256 = sha256_bytes(&verified_payload.payload);
    if payload_sha256 != record.artifact_sha256 {
        evidence.deny("reconstructed_artifact_sha_mismatch");
        return evidence;
    }
    let wasm_valid = wasm_runtime::validate_module_bytes(&verified_payload.payload);
    evidence.reconstructed_wasm_valid = Some(wasm_valid);
    if !wasm_valid {
        evidence.deny("reconstructed_wasm_invalid");
        return evidence;
    }
    let snapshot = match record.grant_target_snapshot() {
        Ok(snapshot) => snapshot,
        Err(reason) => {
            evidence.deny(reason);
            return evidence;
        }
    };
    let observed_import_mask = match inspect_recovery_import_mask(&verified_payload.payload) {
        Ok(mask) => mask,
        Err(reason) => {
            evidence.deny(reason);
            return evidence;
        }
    };
    if !artifact_store::grant_target_inventory_matches(
        record.grant_target_surface_mask,
        record.import_set_hash,
        snapshot.entry_count,
        observed_import_mask,
    ) {
        evidence.deny("rollback_target_import_inventory_mismatch");
        return evidence;
    }
    let Some(authorization) = transaction.install_authorization else {
        evidence.deny("install_authorization_missing");
        return evidence;
    };
    let Some(load_event_id) = transaction.load_event_id.event_id() else {
        evidence.deny("promotion_transaction_event_id_invalid");
        return evidence;
    };
    let promotion = granted_candidate_service::PromotionRecord {
        plan_hash: transaction.rollback_plan_hash,
        pre_load_inventory_hash: transaction.pre_load_inventory_hash,
        ram_only_service_slot_id: transaction.ram_only_service_slot_id,
        generation: transaction.generation,
        load_event_id,
        artifact_hash: transaction.artifact_hash,
    };
    if let Err(reason) = granted_candidate_service::validate_reverified_install_state(
        authorization,
        transaction.install_authorization_frame_sha256,
        promotion,
        observed_import_mask,
    ) {
        evidence.deny(reason);
        return evidence;
    }
    if !granted_candidate_service::slot_allocatable() {
        evidence.deny("provider_recovery_service_slot_not_empty");
        return evidence;
    }

    module_candidate_intake::retain(verified_payload.payload, payload_sha256, wasm_valid);
    evidence.retained_candidate = true;
    if !repopulate_reverified_references(&transaction) {
        // Orphaned retain (no load happened): clear the just-retained candidate for hygiene.
        module_candidate_intake::clear();
        evidence.deny("reverified_reference_repopulation_failed");
        return evidence;
    }
    evidence.references_repopulated = true;

    if let Err(reason) = granted_candidate_service::restore_reverified_install_state(
        authorization,
        transaction.install_authorization_frame_sha256,
        promotion,
        observed_import_mask,
    ) {
        module_candidate_intake::clear();
        evidence.deny(reason);
        return evidence;
    }
    granted_candidate_service::emit_recovery_ram_only_load();
    evidence.load_attempted = true;
    let load_snapshot = granted_candidate_service::loaded_snapshot();
    if let Some(snapshot) = load_snapshot {
        evidence.service_loaded = snapshot.loaded;
        evidence.load_reason = Some(snapshot.last_reason);
        evidence.load_granted = snapshot.loaded
            && snapshot.trust_tier == TRUST_TIER
            && snapshot.last_action == "load"
            && snapshot.last_reason == "loaded_dev_key_granted_external_wasm_current_boot";
    }
    if !evidence.load_granted {
        // The M6 gate did NOT load this candidate: clear the orphaned retain so a denied
        // re-promotion never leaves the denied record's bytes as the live retained candidate.
        granted_candidate_service::clear_reverified_install_state();
        evidence.deny(
            load_snapshot
                .map(|snapshot| snapshot.last_reason)
                .unwrap_or("load_gate_denied_no_snapshot"),
        );
        return evidence;
    }

    granted_candidate_service::emit_recovery_ram_only_start();
    evidence.start_attempted = true;
    let start_snapshot = granted_candidate_service::loaded_snapshot();
    if let Some(snapshot) = start_snapshot {
        evidence.service_started = snapshot.loaded
            && snapshot.running
            && snapshot.last_action == "start"
            && snapshot.last_reason == "wasm_run_success"
            && snapshot.last_run_outcome == "success";
        evidence.start_reason = Some(snapshot.last_reason);
        evidence.start_run_outcome = Some(snapshot.last_run_outcome);
    }
    if !evidence.service_started {
        granted_candidate_service::clear_reverified_install_state();
        evidence.deny(
            start_snapshot
                .map(|snapshot| snapshot.last_reason)
                .unwrap_or("start_gate_denied_no_snapshot"),
        );
        return evidence;
    }

    evidence.status = "repromoted";
    evidence.reason = "repromotion_reverified_m6_gate_loaded_and_started";
    evidence.performed = true;
    evidence.would_repromote = true;
    evidence.authorizes_load = true;
    evidence.cross_reboot_proven = true;
    evidence
}

/// Honest, read-only projection of the FULL re-promotion outcome for the recovery
/// lifeline (M8D-2 `recovery.load_artifact_by_hash`). It carries the identity of the
/// re-verified record plus the REAL M6 gate load/start result, so the lifeline can both
/// audit the reinstatement and report it truthfully. `reinstated` is genuine only when
/// the gate actually re-verified, re-loaded AND re-started the service; `reverified`
/// reports whether the pure re-verification decision passed (the reconstructed-wasm
/// validation is only reached once it has), so a reverify failure can never masquerade
/// as loadable.
#[derive(Clone, Copy)]
pub(crate) struct ReinstateOutcome {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) reinstated: bool,
    pub(crate) reverified: bool,
    pub(crate) reconstructed_wasm_valid: Option<bool>,
    pub(crate) load_attempted: bool,
    pub(crate) load_granted: bool,
    pub(crate) service_loaded: bool,
    pub(crate) service_started: bool,
    pub(crate) start_run_outcome: Option<&'static str>,
    pub(crate) authorizes_load: bool,
    pub(crate) cross_reboot_proven: bool,
    pub(crate) candidate_payload_sha256: Option<[u8; 32]>,
    pub(crate) recomputed_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) recorded_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_persist_seq: u64,
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) manifest_hash: [u8; 32],
    pub(crate) vm_report_hash: [u8; 32],
    pub(crate) grant_hash: [u8; 32],
    pub(crate) promotion_transaction_sha256: [u8; 32],
    pub(crate) artstor_blob_offset: u64,
    pub(crate) artstor_blob_len: u64,
    pub(crate) artstor_blob_frame_sha256: [u8; 32],
}

/// M8D-2 load entry: run the EXISTING full `reverify_record` (reverify + reconstructed-wasm
/// validity + retain + repopulate references + the UNMODIFIED `granted_candidate_service`
/// M6 load/start gate) and project its honest outcome. This is the ONLY load path the
/// recovery lifeline uses — it re-instates the RAM-only current-boot service ONLY when the
/// full gate genuinely loads AND starts it. No second loader, no reverify_record_only
/// shortcut, no logic change to `reverify_record`.
pub(crate) fn reverify_and_load_record(
    controller: pci::PciMassStorageController,
    record: &artifact_store::ArtifactPersistRecord,
) -> ReinstateOutcome {
    let evidence = reverify_recovery_record(controller, record);
    ReinstateOutcome {
        status: evidence.status,
        reason: evidence.reason,
        // "repromoted" is set by reverify_record ONLY after the M6 gate loaded AND
        // started the service; performed/authorizes_load/cross_reboot_proven are set in
        // the same final step, so this is a genuine reinstatement, never a partial one.
        reinstated: evidence.status == "repromoted"
            && evidence.performed
            && evidence.service_loaded
            && evidence.service_started,
        // The reconstructed-wasm validation is reached ONLY after the pure re-verify
        // decision passed, so its presence is an honest "reverify succeeded" signal.
        reverified: evidence.reconstructed_wasm_valid.is_some(),
        reconstructed_wasm_valid: evidence.reconstructed_wasm_valid,
        load_attempted: evidence.load_attempted,
        load_granted: evidence.load_granted,
        service_loaded: evidence.service_loaded,
        service_started: evidence.service_started,
        start_run_outcome: evidence.start_run_outcome,
        authorizes_load: evidence.authorizes_load,
        cross_reboot_proven: evidence.cross_reboot_proven,
        candidate_payload_sha256: evidence.candidate_payload_sha256,
        recomputed_attestation_reference_hash: evidence.recomputed_attestation_reference_hash,
        recorded_attestation_reference_hash: evidence.recorded_attestation_reference_hash,
        artifact_persist_seq: evidence.artifact_persist_seq,
        artifact_sha256: evidence.artifact_sha256,
        manifest_hash: evidence.manifest_hash,
        vm_report_hash: evidence.vm_report_hash,
        grant_hash: evidence.grant_hash,
        promotion_transaction_sha256: evidence.promotion_transaction_sha256,
        artstor_blob_offset: evidence.artstor_blob_offset,
        artstor_blob_len: evidence.artstor_blob_len,
        artstor_blob_frame_sha256: evidence.artstor_blob_frame_sha256,
    }
}

pub(crate) fn reverify_and_load_rollback_record(
    controller: pci::PciMassStorageController,
    record: &artifact_store::ArtifactPersistRecord,
) -> ReinstateOutcome {
    let evidence = reverify_record_ram_only(controller, record);
    ReinstateOutcome {
        status: evidence.status,
        reason: evidence.reason,
        reinstated: evidence.status == "repromoted"
            && evidence.performed
            && evidence.service_loaded
            && evidence.service_started,
        reverified: evidence.reconstructed_wasm_valid.is_some(),
        reconstructed_wasm_valid: evidence.reconstructed_wasm_valid,
        load_attempted: evidence.load_attempted,
        load_granted: evidence.load_granted,
        service_loaded: evidence.service_loaded,
        service_started: evidence.service_started,
        start_run_outcome: evidence.start_run_outcome,
        authorizes_load: evidence.authorizes_load,
        cross_reboot_proven: evidence.cross_reboot_proven,
        candidate_payload_sha256: evidence.candidate_payload_sha256,
        recomputed_attestation_reference_hash: evidence.recomputed_attestation_reference_hash,
        recorded_attestation_reference_hash: evidence.recorded_attestation_reference_hash,
        artifact_persist_seq: evidence.artifact_persist_seq,
        artifact_sha256: evidence.artifact_sha256,
        manifest_hash: evidence.manifest_hash,
        vm_report_hash: evidence.vm_report_hash,
        grant_hash: evidence.grant_hash,
        promotion_transaction_sha256: evidence.promotion_transaction_sha256,
        artstor_blob_offset: evidence.artstor_blob_offset,
        artstor_blob_len: evidence.artstor_blob_len,
        artstor_blob_frame_sha256: evidence.artstor_blob_frame_sha256,
    }
}

pub(crate) fn repopulate_reverified_references(transaction: &PromotionTransactionReadback) -> bool {
    let (
        Some(retained_manifest_reference_event_id),
        Some(retained_artifact_reference_event_id),
        Some(retained_vm_report_reference_event_id),
        Some(retained_computed_grant_reference_event_id),
    ) = (
        transaction.retained_manifest_reference_event_id.event_id(),
        transaction.retained_artifact_reference_event_id.event_id(),
        transaction.retained_vm_report_reference_event_id.event_id(),
        transaction
            .retained_computed_grant_reference_event_id
            .event_id(),
    )
    else {
        return false;
    };

    event_log::record_module_computed_grant_reference(event_log::ModuleComputedGrantReference {
        computed_grant_hash: transaction.computed_grant_hash,
        manifest_hash: transaction.manifest_hash,
        artifact_hash: transaction.artifact_hash,
        vm_report_hash: transaction.vm_report_hash,
        local_attestation_hash: transaction.local_attestation_hash,
    });
    event_log::record_module_local_attestation_reference(
        event_log::ModuleLocalAttestationReference {
            attestation_reference_hash: transaction.attestation_reference_hash,
            retained_manifest_reference_event_id,
            retained_artifact_reference_event_id,
            retained_vm_report_reference_event_id,
            retained_reference_event_id: retained_computed_grant_reference_event_id,
            manifest_reference_hash: transaction.manifest_reference_hash,
            artifact_reference_hash: transaction.artifact_reference_hash,
            vm_report_reference_hash: transaction.vm_report_reference_hash,
            manifest_hash: transaction.manifest_hash,
            artifact_hash: transaction.artifact_hash,
            computed_grant_hash: transaction.computed_grant_hash,
            vm_report_hash: transaction.vm_report_hash,
            local_attestation_hash: transaction.local_attestation_hash,
            signature_verified: true,
        },
    );
    event_log::record_module_promotion_signature_reference(
        event_log::ModulePromotionSignatureReference {
            signature_der: transaction.promotion_signature_der,
            signature_len: transaction.promotion_signature_len,
            attestation_reference_hash: transaction.attestation_reference_hash,
            promotion_authority_key_sha256: transaction.promotion_authority_key_sha256,
            signature_verified: true,
        },
    );
    true
}

fn denied_from_record(
    record: &artifact_store::ArtifactPersistRecord,
    reason: &'static str,
) -> RepromotionEvidence {
    RepromotionEvidence {
        artifact_persist_seq: record.seq,
        artifact_persist_reclog_offset: record.reclog_offset,
        artstor_blob_offset: record.artstor_blob_offset,
        artstor_blob_len: record.artstor_blob_len,
        artstor_blob_frame_sha256: record.artstor_blob_frame_sha256,
        artifact_sha256: record.artifact_sha256,
        manifest_hash: record.manifest_hash,
        vm_report_hash: record.vm_report_hash,
        grant_hash: record.grant_hash,
        promotion_transaction_sha256: record.promotion_transaction_sha256,
        transaction_frame_sha256: None,
        candidate_payload_sha256: None,
        candidate_payload_len: None,
        recomputed_attestation_reference_hash: None,
        recorded_attestation_reference_hash: None,
        promotion_authority_key_sha256: None,
        promotion_signature_len: None,
        recorded_signature_verified: None,
        grant_binds_capability: None,
        reconstructed_wasm_valid: None,
        retained_candidate: false,
        references_repopulated: false,
        load_attempted: false,
        load_granted: false,
        service_loaded: false,
        load_reason: None,
        start_attempted: false,
        service_started: false,
        start_reason: None,
        start_run_outcome: None,
        status: "repromotion_denied",
        reason,
        performed: false,
        would_repromote: false,
        authorizes_load: false,
        trust_tier: TRUST_TIER,
        owner_sealed: false,
        cross_reboot_proven: false,
        persistence_claimed: false,
        audit: None,
    }
}

pub(crate) fn find_promotion_transaction(
    bytes: &[u8],
    promotion_transaction_sha256: [u8; 32],
) -> Result<PromotionTransactionReadback, &'static str> {
    let scan = scan_reclog(bytes);
    let mut offset = 0usize;
    let mut seq = 1u64;
    let mut prev = [0u8; 32];
    let mut count = 0u64;
    while count < scan.count && offset < bytes.len() {
        let frame = parse_reclog_frame(bytes, offset, seq, prev)
            .map_err(|_| "promotion_transaction_not_found")?;
        let payload_start = offset + RECLOG_FRAME_HEADER_LEN;
        let payload_end = payload_start + frame.payload_len as usize;
        if frame.frame_sha256 == promotion_transaction_sha256 {
            let payload = str::from_utf8(&bytes[payload_start..payload_end])
                .map_err(|_| "promotion_transaction_payload_invalid_utf8")?;
            let mut parsed = parse_promotion_transaction_payload(payload)?;
            parsed.frame_sha256 = frame.frame_sha256;
            let authorization =
                find_install_authorization(bytes, parsed.install_authorization_frame_sha256)?;
            if !link_install_authorization(&mut parsed, &[authorization])
                || !complete_install_authorization(&parsed)
            {
                return Err("install_authorization_missing");
            }
            return Ok(parsed);
        }
        offset += frame.frame_len as usize;
        seq = seq.saturating_add(1);
        prev = frame.frame_sha256;
        count += 1;
    }
    Err("promotion_transaction_not_found")
}

fn parse_promotion_transaction_payload(
    payload: &str,
) -> Result<PromotionTransactionReadback, &'static str> {
    if !contains_bytes(
        payload.as_bytes(),
        b"\"schema\": \"raios.promotion_transaction.v0\"",
    ) {
        return Err("promotion_transaction_schema_mismatch");
    }
    let transaction_kind = extract_str(payload, b"\"transaction_kind\": \"")
        .ok_or("promotion_transaction_field_missing")?;
    if transaction_kind != "promote" && transaction_kind != "unpromote" {
        return Err("promotion_transaction_kind_mismatch");
    }
    if extract_str(payload, b"\"scope\": \"") != Some("origin_boot")
        || extract_str(payload, b"\"classification\": \"") != Some("local_only")
        || extract_str(payload, b"\"record_kind\": \"") != Some("promotion_transaction")
        || extract_str(payload, b"\"service_id\": \"") != Some(GRANTED_CANDIDATE_SERVICE_ID)
        || extract_str(payload, b"\"trust_tier\": \"") != Some(TRUST_TIER)
    {
        return Err("promotion_transaction_scope_mismatch");
    }
    let install_envelope_version =
        artifact_store::extract_u64(payload, b"\"install_envelope_version\": ")
            .ok_or("signed_snapshot_context_mismatch")?;
    let grant_target_count = artifact_store::extract_u64(payload, b"\"grant_target_count\": ")
        .ok_or("signed_snapshot_context_mismatch")?;
    if install_envelope_version
        != raios_core::project_install::GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION as u64
        || extract_str(payload, b"\"grant_target_schema\": \"")
            != Some(raios_core::wasm_import_grant_event::GRANT_TARGET_SNAPSHOT_SCHEMA)
        || grant_target_count > raios_core::wasm_import_grant_event::MAX_PROJECTION_SLOTS as u64
    {
        return Err("signed_snapshot_context_mismatch");
    }
    let grant_target_snapshot_sha256 =
        artifact_store::extract_sha256(payload, b"\"grant_target_snapshot_sha256\": \"")
            .ok_or("signed_snapshot_context_mismatch")?;
    if grant_target_snapshot_sha256 == [0; 32] {
        return Err("signed_snapshot_context_mismatch");
    }
    let (promotion_signature_der, promotion_signature_len) =
        extract_hex_bytes(payload, b"\"promotion_signature_der\": \"")
            .ok_or("promotion_signature_der_invalid")?;
    let recorded_signature_len =
        artifact_store::extract_u64(payload, b"\"promotion_signature_len\": ")
            .ok_or("promotion_signature_len_missing")?;
    if recorded_signature_len == 0
        || recorded_signature_len as usize != promotion_signature_len
        || promotion_signature_len > MAX_PROMOTION_SIGNATURE_DER_LEN
    {
        return Err("promotion_signature_len_mismatch");
    }
    let generation = artifact_store::extract_u64(payload, b"\"generation\": ")
        .ok_or("promotion_transaction_field_missing")?;
    let load_event_id = extract_event_id(payload, b"\"load_event_id\": \"")?;
    if generation == 0
        || extract_str(payload, b"\"ram_only_service_slot_id\": \"")
            != Some(granted_candidate_service::ram_only_service_slot_id())
    {
        return Err("promotion_transaction_field_mismatch");
    }

    Ok(PromotionTransactionReadback {
        transaction_kind: if transaction_kind == "promote" {
            "promote"
        } else {
            "unpromote"
        },
        computed_grant_hash: artifact_store::extract_sha256(
            payload,
            b"\"computed_grant_hash\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        manifest_hash: artifact_store::extract_sha256(payload, b"\"manifest_hash\": \"")
            .ok_or("promotion_transaction_field_missing")?,
        artifact_hash: artifact_store::extract_sha256(payload, b"\"artifact_hash\": \"")
            .ok_or("promotion_transaction_field_missing")?,
        vm_report_hash: artifact_store::extract_sha256(payload, b"\"vm_report_hash\": \"")
            .ok_or("promotion_transaction_field_missing")?,
        local_attestation_hash: artifact_store::extract_sha256(
            payload,
            b"\"local_attestation_hash\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        retained_manifest_reference_event_id: extract_event_id(
            payload,
            b"\"retained_manifest_reference_event_id\": \"",
        )?,
        retained_artifact_reference_event_id: extract_event_id(
            payload,
            b"\"retained_artifact_reference_event_id\": \"",
        )?,
        retained_vm_report_reference_event_id: extract_event_id(
            payload,
            b"\"retained_vm_report_reference_event_id\": \"",
        )?,
        retained_computed_grant_reference_event_id: extract_event_id(
            payload,
            b"\"retained_computed_grant_reference_event_id\": \"",
        )?,
        manifest_reference_hash: artifact_store::extract_sha256(
            payload,
            b"\"manifest_reference_hash\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        artifact_reference_hash: artifact_store::extract_sha256(
            payload,
            b"\"artifact_reference_hash\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        vm_report_reference_hash: artifact_store::extract_sha256(
            payload,
            b"\"vm_report_reference_hash\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        attestation_reference_hash: artifact_store::extract_sha256(
            payload,
            b"\"attestation_reference_hash\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        promotion_signature_der,
        promotion_signature_len,
        promotion_authority_key_sha256: artifact_store::extract_sha256(
            payload,
            b"\"promotion_authority_key_sha256\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        signature_verified: artifact_store::extract_bool(payload, b"\"signature_verified\": ")
            .ok_or("promotion_transaction_field_missing")?,
        grant_binds_capability: artifact_store::extract_bool(
            payload,
            b"\"grant_binds_capability\": ",
        )
        .ok_or("promotion_transaction_field_missing")?,
        install_authorization_present: artifact_store::extract_bool(
            payload,
            b"\"install_authorization_present\": ",
        )
        .unwrap_or(false),
        install_envelope_binds_activation: artifact_store::extract_bool(
            payload,
            b"\"install_envelope_binds_activation\": ",
        )
        .unwrap_or(false),
        install_action_signature_verified: artifact_store::extract_bool(
            payload,
            b"\"install_action_signature_verified\": ",
        )
        .unwrap_or(false),
        physical_install_approval_consumed: artifact_store::extract_bool(
            payload,
            b"\"physical_install_approval_consumed\": ",
        )
        .unwrap_or(false),
        install_authorization_frame_sha256: artifact_store::extract_sha256(
            payload,
            b"\"install_authorization_frame_sha256\": \"",
        )
        .unwrap_or([0; 32]),
        install_envelope_version: install_envelope_version as u16,
        grant_target_schema: raios_core::wasm_import_grant_event::GRANT_TARGET_SNAPSHOT_SCHEMA,
        grant_target_count,
        grant_target_snapshot_sha256,
        rollback_plan_hash: artifact_store::extract_sha256(payload, b"\"rollback_plan_hash\": \"")
            .ok_or("promotion_transaction_field_missing")?,
        pre_load_inventory_hash: artifact_store::extract_sha256(
            payload,
            b"\"pre_load_inventory_hash\": \"",
        )
        .ok_or("promotion_transaction_field_missing")?,
        ram_only_service_slot_id: granted_candidate_service::ram_only_service_slot_id(),
        generation,
        load_event_id,
        install_authorization: None,
        frame_sha256: [0u8; 32],
    })
}

fn find_install_authorization(
    bytes: &[u8],
    frame_sha256: [u8; 32],
) -> Result<InstallAuthorizationReadback, &'static str> {
    if frame_sha256 == [0; 32] {
        return Err("install_authorization_missing");
    }
    let scan = scan_reclog(bytes);
    let mut offset = 0usize;
    let mut seq = 1u64;
    let mut prev = [0u8; 32];
    let mut count = 0u64;
    while count < scan.count && offset < bytes.len() {
        let frame = parse_reclog_frame(bytes, offset, seq, prev)
            .map_err(|_| "install_authorization_not_found")?;
        if frame.frame_sha256 == frame_sha256 {
            let payload_start = offset + RECLOG_FRAME_HEADER_LEN;
            let payload_end = payload_start + frame.payload_len as usize;
            let payload = str::from_utf8(&bytes[payload_start..payload_end])
                .map_err(|_| "install_authorization_payload_invalid_utf8")?;
            let mut parsed = parse_install_authorization_payload(payload)?;
            parsed.frame_sha256 = frame.frame_sha256;
            return Ok(parsed);
        }
        offset += frame.frame_len as usize;
        seq = seq.saturating_add(1);
        prev = frame.frame_sha256;
        count += 1;
    }
    Err("install_authorization_not_found")
}

fn parse_install_authorization_payload(
    payload: &str,
) -> Result<InstallAuthorizationReadback, &'static str> {
    if !contains_bytes(
        payload.as_bytes(),
        b"\"schema\": \"raios.install_authorization.v0\"",
    ) {
        return Err("install_authorization_schema_mismatch");
    }
    if extract_str(payload, b"\"id\": \"")
        != Some("install_authorization.origin_boot.svc.dev.granted_candidate.v0")
        || extract_str(payload, b"\"scope\": \"") != Some("origin_boot")
        || extract_str(payload, b"\"classification\": \"") != Some("local_only")
        || extract_str(payload, b"\"record_kind\": \"") != Some("install_authorization")
        || extract_str(payload, b"\"service_id\": \"") != Some(GRANTED_CANDIDATE_SERVICE_ID)
        || extract_str(payload, b"\"trust_tier\": \"") != Some(TRUST_TIER)
    {
        return Err("install_authorization_scope_mismatch");
    }
    let install_envelope_version =
        artifact_store::extract_u64(payload, b"\"install_envelope_version\": ")
            .ok_or("signed_snapshot_context_mismatch")?;
    let grant_target_count = artifact_store::extract_u64(payload, b"\"grant_target_count\": ")
        .ok_or("signed_snapshot_context_mismatch")?;
    if install_envelope_version
        != raios_core::project_install::GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION as u64
        || extract_str(payload, b"\"grant_target_schema\": \"")
            != Some(raios_core::wasm_import_grant_event::GRANT_TARGET_SNAPSHOT_SCHEMA)
        || grant_target_count > raios_core::wasm_import_grant_event::MAX_PROJECTION_SLOTS as u64
    {
        return Err("signed_snapshot_context_mismatch");
    }
    let (signature_der, signature_len) =
        extract_install_hex_bytes(payload, b"\"install_signature_der\": \"")
            .ok_or("install_action_signature_not_verified")?;
    if artifact_store::extract_u64(payload, b"\"install_signature_len\": ").map(|v| v as usize)
        != Some(signature_len)
    {
        return Err("install_action_signature_not_verified");
    }
    let sha =
        |key| artifact_store::extract_sha256(payload, key).ok_or("install_authorization_missing");
    let grant_target_snapshot_sha256 = sha(b"\"grant_target_snapshot_sha256\": \"")?;
    if grant_target_snapshot_sha256 == [0; 32] {
        return Err("signed_snapshot_context_mismatch");
    }
    Ok(InstallAuthorizationReadback {
        authorization: granted_candidate_service::SignedInstallAuthorization {
            install_envelope_version: install_envelope_version as u16,
            activation_approval_sha256: sha(b"\"activation_approval_sha256\": \"")?,
            computed_grant_sha256: sha(b"\"computed_grant_sha256\": \"")?,
            attestation_reference_sha256: sha(b"\"attestation_reference_sha256\": \"")?,
            grant_target_schema: raios_core::wasm_import_grant_event::GRANT_TARGET_SNAPSHOT_SCHEMA,
            grant_target_count,
            grant_target_snapshot_sha256,
            w7_invocation_sha256: sha(b"\"w7_invocation_sha256\": \"")?,
            w7_receipt_sha256: sha(b"\"w7_receipt_sha256\": \"")?,
            receiver_content_sha256: sha(b"\"receiver_content_sha256\": \"")?,
            receiver_candidate_sha256: sha(b"\"receiver_candidate_sha256\": \"")?,
            catalog_candidate_sha256: sha(b"\"catalog_candidate_sha256\": \"")?,
            install_envelope_sha256: sha(b"\"install_envelope_sha256\": \"")?,
            install_action_sha256: sha(b"\"install_action_sha256\": \"")?,
            install_action_message_sha256: sha(b"\"install_action_signature_message_sha256\": \"")?,
            authority_evidence_sha256: sha(b"\"authority_evidence_sha256\": \"")?,
            physical_approval_sha256: sha(b"\"physical_approval_sha256\": \"")?,
            authority_key_sha256: sha(b"\"install_authority_key_sha256\": \"")?,
            candidate_sha256: sha(b"\"install_candidate_sha256\": \"")?,
            candidate_byte_len: artifact_store::extract_u64(
                payload,
                b"\"install_candidate_byte_len\": ",
            )
            .ok_or("install_authorization_missing")?,
            generation: artifact_store::extract_u64(payload, b"\"install_generation\": ")
                .ok_or("install_authorization_missing")?,
            log_sequence: artifact_store::extract_u64(payload, b"\"install_log_sequence\": ")
                .ok_or("install_authorization_missing")?,
            signature_len,
            signature_der,
        },
        frame_sha256: [0; 32],
    })
}

fn extract_install_hex_bytes(payload: &str, key: &[u8]) -> Option<([u8; 256], usize)> {
    let value = extract_str(payload, key)?;
    if value.is_empty() || value.len() % 2 != 0 || value.len() / 2 > 256 {
        return None;
    }
    let mut out = [0u8; 256];
    let mut pos = 0usize;
    while pos < value.len() / 2 {
        out[pos] = (hex_nibble(value.as_bytes()[pos * 2])? << 4)
            | hex_nibble(value.as_bytes()[pos * 2 + 1])?;
        pos += 1;
    }
    Some((out, pos))
}

fn extract_event_id(payload: &str, needle: &[u8]) -> Result<EventIdText, &'static str> {
    let value = extract_str(payload, needle).ok_or("promotion_transaction_field_missing")?;
    EventIdText::parse(value).ok_or("promotion_transaction_event_id_invalid")
}

fn append_repromotion_audit(
    controller: pci::PciMassStorageController,
    evidence: &RepromotionEvidence,
) -> AuditAppendEvidence {
    let scan = artifact_store::current_boot_reclog_scan(controller);
    let payload = repromotion_payload_bytes(evidence);
    let planned = match plan_reclog_append(&scan.scan, &payload, scan.reclog_byte_count) {
        Ok(planned) => planned,
        Err(denied) => return AuditAppendEvidence::denied(denied.reason()),
    };
    let write = unsafe {
        ahci::write_readback_reclog_append(controller, planned.write_offset, &planned.frame)
    };
    let readback_sha256 = write.readback.as_deref().map(sha256_bytes);
    let reparse_valid = write
        .readback
        .as_deref()
        .map(|bytes| parse_reclog_frame(bytes, 0, planned.seq, planned.prev_frame_sha256).is_ok())
        .unwrap_or(false);
    let decision = evaluate_scoped_repromotion_append(&ScopedRepromotionAppendInput {
        method: Some(REPROMOTION_EXPECTED_METHOD),
        target_id: Some(REPROMOTION_EXPECTED_TARGET_ID),
        record_schema: Some(REPROMOTION_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(REPROMOTION_EXPECTED_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(scan.reclog_byte_count),
        absolute_start_lba: Some(scan.reclog_absolute_start_lba),
        reclog_lba_count: Some(scan.reclog_lba_count),
        seq: Some(planned.seq),
        tail_seq: Some(scan.scan.tail_seq),
        count: Some(scan.scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: scan.scan.tail_frame_sha256,
        payload_sha256: Some(planned.payload_sha256),
        planned_payload_sha256: Some(planned.payload_sha256),
        planned_frame_sha256: Some(planned.frame_sha256),
        readback_frame_sha256: readback_sha256,
        write_attempted: write.write_attempted,
        write_completed: write.write_completed,
        readback_completed: write.readback_completed,
        readback_matches_planned: readback_sha256 == Some(planned.frame_sha256),
        reparse_valid,
        span_in_bounds: write.span_in_bounds,
        decision_status: Some(evidence.status),
        would_repromote: evidence.would_repromote,
        authorizes_load: evidence.authorizes_load,
        trust_tier: Some(REPROMOTION_EXPECTED_TRUST_TIER),
        owner_sealed: evidence.owner_sealed,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        cross_reboot_proven: evidence.cross_reboot_proven,
        persistence_claimed: evidence.persistence_claimed,
    });
    if !decision.performed {
        return audit_from_write(
            "capability_denied",
            decision.reason,
            "evidence_only",
            &planned,
            readback_sha256,
            reparse_valid,
        );
    }

    let after = artifact_store::current_boot_reclog_scan(controller);
    let rescan_ok = scan
        .scan
        .count
        .checked_add(1)
        .map(|count| after.scan.count == count)
        .unwrap_or(false)
        && after.scan.tail_seq == planned.seq
        && after.scan.tail_frame_sha256 == Some(planned.frame_sha256);
    if !rescan_ok {
        return audit_from_write(
            "capability_denied",
            "post_append_rescan_mismatch",
            "evidence_only",
            &planned,
            readback_sha256,
            reparse_valid,
        );
    }

    audit_from_write(
        "appended",
        decision.reason,
        "scoped_repromotion_append_authorized",
        &planned,
        readback_sha256,
        reparse_valid,
    )
}

fn audit_from_write(
    status: &'static str,
    reason: &'static str,
    authority: &'static str,
    planned: &PlannedAppend,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
) -> AuditAppendEvidence {
    AuditAppendEvidence {
        performed: status == "appended",
        status,
        reason,
        authority,
        seq: Some(planned.seq),
        write_offset: Some(planned.write_offset),
        frame_len: Some(planned.frame_len),
        payload_sha256: Some(planned.payload_sha256),
        frame_sha256: Some(planned.frame_sha256),
        readback_sha256,
        reparse_valid,
    }
}

fn repromotion_payload_bytes(evidence: &RepromotionEvidence) -> Vec<u8> {
    let record = V::Object(repromotion_fields(evidence, false));
    let mut sink = VecSink(Vec::new());
    write_json(&record, &mut sink, 0);
    sink.0
}

fn emit_skipped(reason: &'static str) {
    begin_response(METHOD);
    emit_record_fields(
        vec![
            f("schema", s(RESPONSE_SCHEMA)),
            f("id", s(RESPONSE_ID)),
            f("method", s(METHOD)),
            f("scope", s(SCOPE)),
            f("classification", s(CLASSIFICATION)),
            f("status", s("repromotion_denied")),
            f("performed", b(false)),
            f("reason", s(reason)),
            f("artifact_count", V::U64(0)),
            f("would_repromote", b(false)),
            f("authorizes_load", b(false)),
            f("cross_reboot_proven", b(false)),
            f("owner_sealed", b(false)),
            f("persistence_claimed", b(false)),
            f("audit_write_attempted", b(false)),
        ],
        6,
    );
    end_response(METHOD);
}

pub(crate) fn run_provider_autoload() {
    let (posture, _, authoritative) = boot_control::current_boot_last_good_view();
    if posture != BootPosture::Normal {
        emit_provider_autoload_marker(
            "denied",
            "denied",
            "provider_autoload_posture_not_normal",
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
            0,
            false,
        );
        return;
    }
    if !authoritative.is_some_and(|record| record.boot_attempt.success_marked) {
        emit_provider_autoload_marker(
            "denied",
            "denied",
            "provider_autoload_requires_boot_success_mark",
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
            0,
            false,
        );
        return;
    }
    let Some(controller) = pci::find_mass_storage_controller() else {
        emit_provider_autoload_marker(
            "denied",
            "denied",
            "ahci_controller_not_observed",
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
            0,
            false,
        );
        return;
    };
    let scan = artifact_store::current_boot_reclog_scan(controller);
    let Some(bytes) = scan.bytes.as_deref() else {
        emit_provider_autoload_marker(
            "denied",
            "denied",
            scan.reason,
            posture,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
            0,
            false,
        );
        return;
    };

    if let Err(reason) =
        artifact_store::validate_unique_committed_install_history(controller, bytes)
    {
        emit_provider_autoload_marker(
            "denied", "denied", reason, posture, None, None, None, false, false, false, false, 0,
            false,
        );
        return;
    }

    match durable_store::wasm_rollback_recovery_state() {
        durable_store::WasmRollbackRecoveryState::Denied(reason) => {
            emit_provider_autoload_marker(
                "denied", "denied", reason, posture, None, None, None, false, false, false, false,
                0, false,
            );
            return;
        }
        durable_store::WasmRollbackRecoveryState::Pending(_) => {
            emit_provider_autoload_marker(
                "denied",
                "denied",
                "rollback_recovery_pending_quarantined",
                posture,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
                0,
                false,
            );
            return;
        }
        durable_store::WasmRollbackRecoveryState::None
        | durable_store::WasmRollbackRecoveryState::Committed(_) => {}
    }

    match durable_store::wasm_rollback_recovery_state() {
        durable_store::WasmRollbackRecoveryState::Denied(reason) => {
            emit_provider_autoload_marker(
                "denied", "denied", reason, posture, None, None, None, false, false, false, false,
                0, false,
            );
            return;
        }
        durable_store::WasmRollbackRecoveryState::Committed(commit) => {
            if commit.service_id != GRANTED_CANDIDATE_SERVICE_ID || commit.domain_instance != 1 {
                emit_provider_autoload_marker(
                    "denied",
                    "denied",
                    "rollback_commit_domain_mismatch",
                    posture,
                    None,
                    None,
                    None,
                    false,
                    false,
                    false,
                    false,
                    0,
                    false,
                );
                return;
            }
            let target = match artifact_store::load_verified_committed_rollback_target(
                commit.target_snapshot_sha256,
            ) {
                Ok(target) => target,
                Err(reason) => {
                    emit_provider_autoload_marker(
                        "denied", "denied", reason, posture, None, None, None, false, false, false,
                        false, 0, false,
                    );
                    return;
                }
            };
            let current_projection = durable_store::load_durable_wasm_grant_projection();
            if !current_projection.valid
                || current_projection.sha256 != commit.result_projection_sha256
            {
                emit_provider_autoload_marker(
                    "denied",
                    "denied",
                    "rollback_commit_projection_mismatch",
                    posture,
                    Some(target.record.artifact_sha256),
                    Some(target.record.promotion_transaction_sha256),
                    None,
                    false,
                    false,
                    false,
                    false,
                    0,
                    false,
                );
                return;
            }
            let evidence = reverify_recovery_record(controller, &target.record);
            let snapshot = granted_candidate_service::loaded_snapshot();
            let loaded = snapshot.is_some_and(|snapshot| snapshot.loaded);
            let running = snapshot.is_some_and(|snapshot| snapshot.running);
            let run_count = snapshot.map(|snapshot| snapshot.run_count).unwrap_or(0);
            let autoloaded = evidence.status == "repromoted"
                && loaded
                && running
                && run_count == 1
                && evidence.cross_reboot_proven;
            emit_provider_autoload_marker(
                if autoloaded { "accepted" } else { "denied" },
                if autoloaded { "autoloaded" } else { "denied" },
                if autoloaded {
                    "rollback_commit_target_autoloaded"
                } else {
                    evidence.reason
                },
                posture,
                Some(target.record.artifact_sha256),
                Some(target.record.promotion_transaction_sha256),
                None,
                evidence.performed,
                evidence.performed,
                loaded,
                running,
                run_count,
                autoloaded,
            );
            return;
        }
        durable_store::WasmRollbackRecoveryState::Pending(_) => {
            emit_provider_autoload_marker(
                "denied",
                "denied",
                "rollback_recovery_pending_quarantined",
                posture,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
                0,
                false,
            );
            return;
        }
        durable_store::WasmRollbackRecoveryState::None => {}
    }

    let (transaction, artifact, artifact_persist_frame_sha256) =
        match resolve_granted_candidate_install(bytes) {
            GrantedCandidateInstallResolution::None => {
                emit_provider_autoload_marker(
                    "accepted",
                    "not_installed",
                    "no_w6_authorized_install",
                    posture,
                    None,
                    None,
                    None,
                    false,
                    false,
                    false,
                    false,
                    0,
                    false,
                );
                return;
            }
            GrantedCandidateInstallResolution::Incomplete => {
                emit_provider_autoload_marker(
                    "denied",
                    "denied",
                    "install_history_incomplete",
                    posture,
                    None,
                    None,
                    None,
                    false,
                    false,
                    false,
                    false,
                    0,
                    false,
                );
                return;
            }
            GrantedCandidateInstallResolution::RolledBack { transaction } => {
                emit_provider_autoload_marker(
                    "accepted",
                    "rolled_back",
                    "granted_candidate_install_rolled_back",
                    posture,
                    Some(transaction.artifact_hash),
                    Some(transaction.frame_sha256),
                    None,
                    false,
                    false,
                    false,
                    false,
                    0,
                    false,
                );
                return;
            }
            GrantedCandidateInstallResolution::Active {
                transaction,
                artifact,
                artifact_persist_frame_sha256,
            } => (transaction, artifact, artifact_persist_frame_sha256),
        };

    let evidence = reverify_recovery_record(controller, &artifact);
    let signatures_reverified = evidence.reconstructed_wasm_valid.is_some();
    let snapshot = granted_candidate_service::loaded_snapshot();
    let loaded = snapshot.is_some_and(|snapshot| snapshot.loaded);
    let running = snapshot.is_some_and(|snapshot| snapshot.running);
    let run_count = snapshot.map(|snapshot| snapshot.run_count).unwrap_or(0);
    let autoloaded = evidence.status == "repromoted"
        && signatures_reverified
        && loaded
        && running
        && run_count == 1
        && evidence.cross_reboot_proven;
    emit_provider_autoload_marker(
        if autoloaded { "accepted" } else { "denied" },
        if autoloaded { "autoloaded" } else { "denied" },
        if evidence.status == "repromoted" && !autoloaded {
            "provider_autoload_completion_invariant_failed"
        } else {
            evidence.reason
        },
        posture,
        Some(transaction.artifact_hash),
        Some(transaction.frame_sha256),
        Some(artifact_persist_frame_sha256),
        signatures_reverified,
        signatures_reverified,
        loaded,
        running,
        run_count,
        autoloaded,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_provider_autoload_marker(
    result: &'static str,
    phase: &'static str,
    reason: &'static str,
    posture: BootPosture,
    candidate_sha256: Option<[u8; 32]>,
    promotion_transaction_sha256: Option<[u8; 32]>,
    artifact_persist_frame_sha256: Option<[u8; 32]>,
    m6_reverified: bool,
    w6_signature_verified: bool,
    loaded: bool,
    running: bool,
    run_count: u64,
    cross_reboot_proven: bool,
) {
    serial::write_raw_str("GRANTED_CANDIDATE_PROVIDER_AUTOLOAD result=");
    serial::write_raw_str(result);
    serial::write_raw_str(" phase=");
    serial::write_raw_str(phase);
    serial::write_raw_str(" reason=");
    serial::write_raw_str(reason);
    serial::write_raw_str(" posture=");
    serial::write_raw_str(provider_autoload_posture(posture));
    serial::write_raw_str(" candidate_sha256=");
    write_optional_hash(candidate_sha256);
    serial::write_raw_str(" promotion_transaction_sha256=");
    write_optional_hash(promotion_transaction_sha256);
    serial::write_raw_str(" artifact_persist_frame_sha256=");
    write_optional_hash(artifact_persist_frame_sha256);
    serial::write_raw_fmt(format_args!(
        " m6_reverified={m6_reverified} w6_signature_verified={w6_signature_verified} loaded={loaded} running={running} run_count={run_count} cross_reboot_proven={cross_reboot_proven}\r\n"
    ));
}

fn provider_autoload_posture(posture: BootPosture) -> &'static str {
    match posture {
        BootPosture::Normal => "Normal",
        BootPosture::Probation => "Probation",
        BootPosture::Safe => "Safe",
        BootPosture::PersistenceUnavailable => "PersistenceUnavailable",
    }
}

fn write_optional_hash(hash: Option<[u8; 32]>) {
    let Some(hash) = hash else {
        serial::write_raw_str("none");
        return;
    };
    serial::write_raw_str("sha256:");
    for byte in hash {
        serial::write_raw_fmt(format_args!("{byte:02x}"));
    }
}

fn emit_no_record(reason: &'static str, status: &'static str, performed: bool) {
    emit_run_record(status, reason, performed, Vec::new());
}

fn emit_run_record(
    status: &'static str,
    reason: &'static str,
    performed: bool,
    decisions: Vec<RepromotionEvidence>,
) {
    let decision_count = decisions.len() as u64;
    let decision_values = decisions
        .iter()
        .map(|evidence| V::Object(repromotion_fields(evidence, true)))
        .collect();
    begin_response(METHOD);
    emit_record_fields(
        vec![
            f("schema", s(RESPONSE_SCHEMA)),
            f("id", s(RESPONSE_ID)),
            f("method", s(METHOD)),
            f("scope", s(SCOPE)),
            f("classification", s(CLASSIFICATION)),
            f("status", s(status)),
            f("performed", b(performed)),
            f("reason", s(reason)),
            f("artifact_count", V::U64(decision_count)),
            f("decisions", V::Array(decision_values)),
            f("would_repromote", b(false)),
            f("authorizes_load", b(false)),
            f("cross_reboot_proven", b(false)),
            f("owner_sealed", b(false)),
            f("persistence_claimed", b(false)),
        ],
        6,
    );
    end_response(METHOD);
}

fn repromotion_fields(evidence: &RepromotionEvidence, include_audit: bool) -> Vec<Field<'_>> {
    let mut fields = vec![
        f("schema", s(RESPONSE_SCHEMA)),
        f("id", s(DECISION_ID)),
        f("method", s(METHOD)),
        f("scope", s(SCOPE)),
        f("classification", s(CLASSIFICATION)),
        f("status", s(evidence.status)),
        f("performed", b(evidence.performed)),
        f("reason", s(evidence.reason)),
        f(
            "artifact_persist_seq",
            V::U64(evidence.artifact_persist_seq),
        ),
        f(
            "artifact_persist_reclog_offset",
            V::U64(evidence.artifact_persist_reclog_offset),
        ),
        f("artstor_blob_offset", V::U64(evidence.artstor_blob_offset)),
        f("artstor_blob_len", V::U64(evidence.artstor_blob_len)),
        f(
            "artstor_blob_frame_sha256",
            V::Sha256(evidence.artstor_blob_frame_sha256),
        ),
        f("artifact_sha256", V::Sha256(evidence.artifact_sha256)),
        f("manifest_hash", V::Sha256(evidence.manifest_hash)),
        f("vm_report_hash", V::Sha256(evidence.vm_report_hash)),
        f("grant_hash", V::Sha256(evidence.grant_hash)),
        f(
            "promotion_transaction_sha256",
            V::Sha256(evidence.promotion_transaction_sha256),
        ),
        f(
            "transaction_frame_sha256",
            record_sha_or_null(evidence.transaction_frame_sha256),
        ),
        f(
            "candidate_payload_sha256",
            record_sha_or_null(evidence.candidate_payload_sha256),
        ),
        f(
            "candidate_payload_len",
            optional_u64(evidence.candidate_payload_len),
        ),
        f(
            "recomputed_attestation_reference_hash",
            record_sha_or_null(evidence.recomputed_attestation_reference_hash),
        ),
        f(
            "recorded_attestation_reference_hash",
            record_sha_or_null(evidence.recorded_attestation_reference_hash),
        ),
        f(
            "promotion_authority_key_sha256",
            record_sha_or_null(evidence.promotion_authority_key_sha256),
        ),
        f(
            "promotion_authority_key_expected_sha256",
            V::Sha256(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        ),
        f(
            "promotion_signature_len",
            optional_u64(evidence.promotion_signature_len),
        ),
        f(
            "recorded_signature_verified",
            optional_bool(evidence.recorded_signature_verified),
        ),
        f(
            "grant_binds_capability",
            optional_bool(evidence.grant_binds_capability),
        ),
        f(
            "reconstructed_wasm_valid",
            optional_bool(evidence.reconstructed_wasm_valid),
        ),
        f("retained_candidate", b(evidence.retained_candidate)),
        f("references_repopulated", b(evidence.references_repopulated)),
        f("load_attempted", b(evidence.load_attempted)),
        f("load_granted", b(evidence.load_granted)),
        f("service_loaded", b(evidence.service_loaded)),
        f("load_reason", optional_str(evidence.load_reason)),
        f("start_attempted", b(evidence.start_attempted)),
        f("service_started", b(evidence.service_started)),
        f("start_reason", optional_str(evidence.start_reason)),
        f(
            "start_run_outcome",
            optional_str(evidence.start_run_outcome),
        ),
        f("would_repromote", b(evidence.would_repromote)),
        f("authorizes_load", b(evidence.authorizes_load)),
        f("trust_tier", s(evidence.trust_tier)),
        f(
            "promotion_authority_is_placeholder",
            b(PROMOTION_AUTHORITY_IS_PLACEHOLDER),
        ),
        f("owner_sealed", b(evidence.owner_sealed)),
        f("cross_reboot_proven", b(evidence.cross_reboot_proven)),
        f("persistence_claimed", b(evidence.persistence_claimed)),
    ];
    if include_audit {
        let audit = evidence
            .audit
            .unwrap_or_else(|| AuditAppendEvidence::denied("audit_missing"));
        fields.extend_from_slice(&[
            f("audit_append_status", s(audit.status)),
            f("audit_append_performed", b(audit.performed)),
            f("audit_append_reason", s(audit.reason)),
            f("audit_append_authority", s(audit.authority)),
            f("audit_append_seq", optional_u64(audit.seq)),
            f(
                "audit_append_write_offset",
                optional_u64(audit.write_offset),
            ),
            f("audit_append_frame_len", optional_u64(audit.frame_len)),
            f(
                "audit_append_payload_sha256",
                record_sha_or_null(audit.payload_sha256),
            ),
            f(
                "audit_append_frame_sha256",
                record_sha_or_null(audit.frame_sha256),
            ),
            f(
                "audit_append_readback_sha256",
                record_sha_or_null(audit.readback_sha256),
            ),
            f("audit_append_reparse_valid", b(audit.reparse_valid)),
        ]);
    }
    fields
}

fn optional_u64(value: Option<u64>) -> V<'static> {
    match value {
        Some(value) => V::U64(value),
        None => V::Null,
    }
}

fn optional_bool(value: Option<bool>) -> V<'static> {
    match value {
        Some(value) => V::Bool(value),
        None => V::Null,
    }
}

fn optional_str(value: Option<&'static str>) -> V<'static> {
    match value {
        Some(value) => s(value),
        None => V::Null,
    }
}

fn extract_str<'a>(payload: &'a str, needle: &[u8]) -> Option<&'a str> {
    let bytes = payload.as_bytes();
    let start = find_bytes(bytes, needle)? + needle.len();
    let mut end = start;
    while end < bytes.len() && bytes[end] != b'"' {
        end += 1;
    }
    if end == bytes.len() {
        return None;
    }
    str::from_utf8(&bytes[start..end]).ok()
}

fn extract_hex_bytes(
    payload: &str,
    key: &[u8],
) -> Option<([u8; MAX_PROMOTION_SIGNATURE_DER_LEN], usize)> {
    let value = extract_str(payload, key)?;
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() % 2 != 0 || bytes.len() / 2 > MAX_PROMOTION_SIGNATURE_DER_LEN
    {
        return None;
    }
    let mut out = [0u8; MAX_PROMOTION_SIGNATURE_DER_LEN];
    let mut pos = 0usize;
    while pos < bytes.len() / 2 {
        let hi = hex_nibble(bytes[pos * 2])?;
        let lo = hex_nibble(bytes[pos * 2 + 1])?;
        out[pos] = (hi << 4) | lo;
        pos += 1;
    }
    Some((out, bytes.len() / 2))
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    find_bytes(haystack, needle).is_some()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    let mut idx = 0usize;
    while idx + needle.len() <= haystack.len() {
        if &haystack[idx..idx + needle.len()] == needle {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
