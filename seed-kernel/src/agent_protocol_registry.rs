use alloc::{vec, vec::Vec};

use raios_core::{
    distribution_provenance::PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
    distribution_registry::{
        evaluate_distribution_registry_selection, ChunkedDeliveryError,
        ChunkedDistributionChunkInput, ChunkedDistributionDelivery, ChunkedDistributionTarget,
        DistributionRegistryEntry, RegistrySelectionDecision, RegistrySelectionReason,
        DISTRIBUTION_CHUNKED_DELIVERY_MAX_CHUNKS,
    },
    parse_sha256_ref,
    record::Value as V,
    sha256_bytes,
};

use crate::{
    agent_protocol_distribution::decode_distribution_signature_der_hex,
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, method_head_eq, record_bool as b,
        record_field as f, record_sha, record_sha_or_null, record_str as s,
    },
    distribution_candidate, module_candidate_channel, module_candidate_intake, wasm_runtime,
};

use super::distribution_registry;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use spin::Mutex;

const SERIAL_DISTRIBUTION_DELIVERY_CHANNEL: &str = "serial_console_distribution_chunks_v0";
const SERIAL_DISTRIBUTION_ENTRY_ID: &str = "serial.local.distribution";
const SERIAL_DISTRIBUTION_SOURCE_ID: &str = "serial.command";
const LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID: &str = "local.serial.catalog";
const LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID: &str = "local.catalog.distribution";
const DISTRIBUTION_BEGIN_ACCEPTED_REASON: &str = "accepted_distribution_delivery_target";
const DISTRIBUTION_CATALOG_ENTRY_ACCEPTED_REASON: &str =
    "accepted_local_distribution_catalog_entry";
const DISTRIBUTION_CATALOG_BEGIN_ACCEPTED_REASON: &str =
    "accepted_catalog_distribution_delivery_target";
const DISTRIBUTION_CATALOG_ENTRY_NOT_FOUND_REASON: &str =
    "local_distribution_catalog_entry_not_found";
const DISTRIBUTION_RECEIVER_IDENTITY_ACCEPTED_REASON: &str =
    "accepted_local_distribution_receiver_identity";
const DISTRIBUTION_RECEIVER_IDENTITY_INVALID_REASON: &str =
    "invalid_distribution_receiver_identity_metadata";
const DISTRIBUTION_RECEIVER_IDENTITY_CATALOG_NOT_FOUND_REASON: &str =
    "receiver_identity_catalog_entry_not_found";
const DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_ACCEPTED_REASON: &str =
    "accepted_local_distribution_receiver_identity_evidence";
const DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_FINALIZED_REASON: &str =
    "local_distribution_receiver_identity_evidence_verified_by_guest";
const DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON: &str =
    "invalid_distribution_receiver_identity_evidence";
const DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INCOMPLETE_REASON: &str =
    "distribution_receiver_identity_evidence_incomplete";
const DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_HASH_MISMATCH_REASON: &str =
    "distribution_receiver_identity_evidence_hash_mismatch";
const DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_TOO_LARGE_REASON: &str =
    "distribution_receiver_identity_evidence_too_large";
const DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_VERIFY_FAILED_REASON: &str =
    "distribution_receiver_identity_evidence_verify_failed";
const DISTRIBUTION_RECEIVER_IDENTITY_PREFLIGHT_MISSING_GATES_REASON: &str =
    "distribution_receiver_identity_load_preflight_missing_required_gates";
const DISTRIBUTION_RECEIVER_IDENTITY_PREFLIGHT_CANDIDATE_NOT_FINALIZED_REASON: &str =
    "distribution_receiver_identity_load_preflight_candidate_not_finalized";
const DISTRIBUTION_RECEIVER_IDENTITY_PREFLIGHT_INVALID_REASON: &str =
    "invalid_distribution_receiver_identity_load_preflight";
const DISTRIBUTION_CHUNK_ACCEPTED_REASON: &str = "accepted_distribution_chunk";
const DISTRIBUTION_DUPLICATE_CHUNK_ACCEPTED_REASON: &str = "accepted_duplicate_distribution_chunk";
const DISTRIBUTION_DELIVERY_NOT_STARTED_REASON: &str = "distribution_delivery_not_started";
const DISTRIBUTION_INVALID_ARGS_REASON: &str = "invalid_distribution_delivery_arguments";
const DISTRIBUTION_INVALID_LENGTH_REASON: &str = "invalid_distribution_total_length";
const DISTRIBUTION_INVALID_CHUNK_COUNT_REASON: &str = "invalid_distribution_chunk_count";
const DISTRIBUTION_INVALID_SIGNATURE_REASON: &str = "invalid_distribution_signature_hex";
const DISTRIBUTION_INVALID_CHUNK_INDEX_REASON: &str = "invalid_distribution_chunk_index";
const RECEIVER_IDENTITY_EVIDENCE_PART_COUNT: usize = 6;
const MAX_RECEIVER_DESCRIPTOR_BYTES: usize = 4096;
const MAX_RECEIVER_PUBLIC_KEY_BYTES: usize = 256;
const MAX_RECEIVER_SIGNATURE_BYTES: usize = 256;

static PENDING_SERIAL_DISTRIBUTION: Mutex<PendingSerialDistribution> =
    Mutex::new(PendingSerialDistribution::new());
static LOCAL_DISTRIBUTION_CATALOG: Mutex<LocalDistributionCatalog> =
    Mutex::new(LocalDistributionCatalog::new());

#[derive(Clone, Copy)]
struct RegistrySelectionRun<'a> {
    parse_ok: bool,
    requested_artifact_sha256: Option<[u8; 32]>,
    registry_entry_count: usize,
    registry_capacity: usize,
    selection: Option<RegistrySelectionDecision<'a>>,
    entry_reason: Option<RegistrySelectionReason>,
    staged_candidate: Option<module_candidate_intake::ExternalWasmCandidateOutcome>,
    retained_provenance: Option<distribution_candidate::DistributionCandidateOutcome>,
}

#[derive(Clone, Copy)]
struct SelftestCase {
    name: &'static str,
    passed: bool,
    status: &'static str,
    reason: &'static str,
    selected_for_candidate_intake: bool,
    staged: bool,
    retained_provenance_verified: bool,
    authorizes_load: bool,
    authorizes_execute: bool,
    authorizes_persist: bool,
}

struct PendingSerialDistribution {
    active: bool,
    source_id: &'static str,
    entry_id: &'static str,
    content_sha256: [u8; 32],
    total_length: usize,
    chunk_count: usize,
    provenance_signature_der: Vec<u8>,
    receiver_identity: Option<LocalReceiverIdentity>,
    chunks: Vec<PendingSerialDistributionChunk>,
}

struct LocalDistributionCatalog {
    active: bool,
    content_sha256: [u8; 32],
    total_length: usize,
    chunk_count: usize,
    provenance_signature_der: Vec<u8>,
    receiver_identity: Option<LocalReceiverIdentity>,
    receiver_identity_evidence: LocalReceiverIdentityEvidencePayloads,
    finalized_candidate_sha256: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct LocalReceiverIdentity {
    artifact_identity_descriptor_sha256: [u8; 32],
    artifact_identity_public_key_sha256: [u8; 32],
    artifact_identity_signature_sha256: [u8; 32],
    load_descriptor_sha256: [u8; 32],
    load_descriptor_public_key_sha256: [u8; 32],
    load_descriptor_signature_sha256: [u8; 32],
    artifact_identity_signature_verified_by_host_export: bool,
    load_descriptor_signature_verified_by_host_export: bool,
    artifact_hash_bound_by_identity: bool,
    artifact_hash_bound_by_load_descriptor: bool,
    load_descriptor_binds_artifact_identity: bool,
    load_descriptor_authorizes_current_boot_wasm_execution: bool,
    artifact_identity_signature_verified_by_guest: bool,
    load_descriptor_signature_verified_by_guest: bool,
    guest_signature_verification_performed: bool,
    receiver_identity_complete: bool,
}

struct LocalReceiverIdentityEvidencePayloads {
    artifact_identity_descriptor: Option<Vec<u8>>,
    artifact_identity_public_key: Option<Vec<u8>>,
    artifact_identity_signature: Option<Vec<u8>>,
    load_descriptor: Option<Vec<u8>>,
    load_descriptor_public_key: Option<Vec<u8>>,
    load_descriptor_signature: Option<Vec<u8>>,
}

struct PendingSerialDistributionChunk {
    index: usize,
    bytes: Vec<u8>,
    claimed_chunk_sha256: [u8; 32],
}

struct DistributionDeliveryTargetMetadata {
    content_sha256: [u8; 32],
    total_length: usize,
    chunk_count: usize,
    provenance_signature_der: Vec<u8>,
    receiver_identity: Option<LocalReceiverIdentity>,
}

#[derive(Clone, Copy)]
struct DistributionReceiverIdentityOutcome {
    content_sha256: Option<[u8; 32]>,
    receiver_identity: Option<LocalReceiverIdentity>,
    retained_in_catalog: bool,
    accepted: bool,
    rejected: bool,
    reason: &'static str,
}

#[derive(Clone, Copy)]
struct DistributionReceiverIdentityEvidenceOutcome {
    content_sha256: Option<[u8; 32]>,
    evidence_kind: &'static str,
    evidence_sha256: Option<[u8; 32]>,
    decoded_byte_len: usize,
    retained_part_count: usize,
    receiver_identity: Option<LocalReceiverIdentity>,
    accepted: bool,
    rejected: bool,
    reason: &'static str,
}

#[derive(Clone, Copy)]
struct DistributionReceiverIdentityLoadPreflightOutcome {
    content_sha256: Option<[u8; 32]>,
    retained_part_count: usize,
    receiver_identity: Option<LocalReceiverIdentity>,
    retained_candidate_sha256: Option<[u8; 32]>,
    retained_candidate_wasm_valid: bool,
    catalog_finalize_candidate_sha256: Option<[u8; 32]>,
    retained_candidate_matches_catalog_finalize: bool,
    status: &'static str,
    reason: &'static str,
    preflight_evaluated: bool,
    accepted: bool,
    rejected: bool,
    missing_gate_count: usize,
}

#[derive(Clone, Copy)]
struct DistributionCatalogEntryOutcome {
    content_sha256: Option<[u8; 32]>,
    total_length: usize,
    chunk_count: usize,
    signature_byte_len: usize,
    receiver_identity_retained: bool,
    retained_in_catalog: bool,
    accepted: bool,
    rejected: bool,
    reason: &'static str,
}

#[derive(Clone, Copy)]
struct DistributionBeginOutcome {
    source_id: &'static str,
    entry_id: &'static str,
    content_sha256: Option<[u8; 32]>,
    total_length: usize,
    chunk_count: usize,
    signature_byte_len: usize,
    receiver_identity: Option<LocalReceiverIdentity>,
    accepted: bool,
    rejected: bool,
    reason: &'static str,
}

#[derive(Clone, Copy)]
struct DistributionChunkOutcome {
    content_sha256: Option<[u8; 32]>,
    chunk_index: usize,
    chunk_sha256: Option<[u8; 32]>,
    decoded_byte_len: usize,
    pending_chunk_count: usize,
    accepted: bool,
    rejected: bool,
    discarded_pending_delivery: bool,
    reason: &'static str,
}

#[derive(Clone, Copy)]
struct DistributionTransportSelection {
    entry_id: &'static str,
    status: &'static str,
    reason: &'static str,
    artifact_sha256: [u8; 32],
    artifact_byte_len: usize,
    provenance_signature_verified: bool,
    selected_for_candidate_intake: bool,
    load_attempted: bool,
    execution_attempted: bool,
    durable_write_attempted: bool,
    authorizes_acquisition: bool,
    authorizes_install: bool,
    authorizes_load: bool,
    authorizes_execute: bool,
    authorizes_persist: bool,
    writes_persistent_state: bool,
    owner_sealed: bool,
}

#[derive(Clone, Copy)]
struct DistributionFinalizeOutcome {
    source_id: &'static str,
    entry_id: &'static str,
    content_sha256: Option<[u8; 32]>,
    total_length: usize,
    declared_chunk_count: usize,
    accepted_chunk_count: usize,
    delivered_byte_len: usize,
    selection: Option<DistributionTransportSelection>,
    staged_candidate: Option<module_candidate_intake::ExternalWasmCandidateOutcome>,
    retained_provenance: Option<distribution_candidate::DistributionCandidateOutcome>,
    receiver_identity: Option<LocalReceiverIdentity>,
    status: &'static str,
    reason: &'static str,
}

impl PendingSerialDistribution {
    const fn new() -> Self {
        Self {
            active: false,
            source_id: SERIAL_DISTRIBUTION_SOURCE_ID,
            entry_id: SERIAL_DISTRIBUTION_ENTRY_ID,
            content_sha256: [0; 32],
            total_length: 0,
            chunk_count: 0,
            provenance_signature_der: Vec::new(),
            receiver_identity: None,
            chunks: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.active = false;
        self.source_id = SERIAL_DISTRIBUTION_SOURCE_ID;
        self.entry_id = SERIAL_DISTRIBUTION_ENTRY_ID;
        self.content_sha256 = [0; 32];
        self.total_length = 0;
        self.chunk_count = 0;
        self.provenance_signature_der.clear();
        self.receiver_identity = None;
        self.chunks.clear();
    }
}

impl LocalDistributionCatalog {
    const fn new() -> Self {
        Self {
            active: false,
            content_sha256: [0; 32],
            total_length: 0,
            chunk_count: 0,
            provenance_signature_der: Vec::new(),
            receiver_identity: None,
            receiver_identity_evidence: LocalReceiverIdentityEvidencePayloads::new(),
            finalized_candidate_sha256: None,
        }
    }

    fn clear(&mut self) {
        self.active = false;
        self.content_sha256 = [0; 32];
        self.total_length = 0;
        self.chunk_count = 0;
        self.provenance_signature_der.clear();
        self.receiver_identity = None;
        self.receiver_identity_evidence.clear();
        self.finalized_candidate_sha256 = None;
    }
}

impl LocalReceiverIdentityEvidencePayloads {
    const fn new() -> Self {
        Self {
            artifact_identity_descriptor: None,
            artifact_identity_public_key: None,
            artifact_identity_signature: None,
            load_descriptor: None,
            load_descriptor_public_key: None,
            load_descriptor_signature: None,
        }
    }

    fn clear(&mut self) {
        self.artifact_identity_descriptor = None;
        self.artifact_identity_public_key = None;
        self.artifact_identity_signature = None;
        self.load_descriptor = None;
        self.load_descriptor_public_key = None;
        self.load_descriptor_signature = None;
    }

    fn retained_part_count(&self) -> usize {
        self.artifact_identity_descriptor.is_some() as usize
            + self.artifact_identity_public_key.is_some() as usize
            + self.artifact_identity_signature.is_some() as usize
            + self.load_descriptor.is_some() as usize
            + self.load_descriptor_public_key.is_some() as usize
            + self.load_descriptor_signature.is_some() as usize
    }

    fn is_complete(&self) -> bool {
        self.retained_part_count() == RECEIVER_IDENTITY_EVIDENCE_PART_COUNT
    }

    fn set(&mut self, kind: ReceiverIdentityEvidenceKind, bytes: Vec<u8>) {
        match kind {
            ReceiverIdentityEvidenceKind::ArtifactIdentityDescriptor => {
                self.artifact_identity_descriptor = Some(bytes)
            }
            ReceiverIdentityEvidenceKind::ArtifactIdentityPublicKey => {
                self.artifact_identity_public_key = Some(bytes)
            }
            ReceiverIdentityEvidenceKind::ArtifactIdentitySignature => {
                self.artifact_identity_signature = Some(bytes)
            }
            ReceiverIdentityEvidenceKind::LoadDescriptor => self.load_descriptor = Some(bytes),
            ReceiverIdentityEvidenceKind::LoadDescriptorPublicKey => {
                self.load_descriptor_public_key = Some(bytes)
            }
            ReceiverIdentityEvidenceKind::LoadDescriptorSignature => {
                self.load_descriptor_signature = Some(bytes)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ReceiverIdentityEvidenceKind {
    ArtifactIdentityDescriptor,
    ArtifactIdentityPublicKey,
    ArtifactIdentitySignature,
    LoadDescriptor,
    LoadDescriptorPublicKey,
    LoadDescriptorSignature,
}

impl ReceiverIdentityEvidenceKind {
    fn parse(token: &str) -> Option<Self> {
        match token {
            "artifact_identity_descriptor" => Some(Self::ArtifactIdentityDescriptor),
            "artifact_identity_public_key" => Some(Self::ArtifactIdentityPublicKey),
            "artifact_identity_signature" => Some(Self::ArtifactIdentitySignature),
            "load_descriptor" => Some(Self::LoadDescriptor),
            "load_descriptor_public_key" => Some(Self::LoadDescriptorPublicKey),
            "load_descriptor_signature" => Some(Self::LoadDescriptorSignature),
            _ => None,
        }
    }

    fn token(self) -> &'static str {
        match self {
            Self::ArtifactIdentityDescriptor => "artifact_identity_descriptor",
            Self::ArtifactIdentityPublicKey => "artifact_identity_public_key",
            Self::ArtifactIdentitySignature => "artifact_identity_signature",
            Self::LoadDescriptor => "load_descriptor",
            Self::LoadDescriptorPublicKey => "load_descriptor_public_key",
            Self::LoadDescriptorSignature => "load_descriptor_signature",
        }
    }

    fn max_bytes(self) -> usize {
        match self {
            Self::ArtifactIdentityDescriptor | Self::LoadDescriptor => {
                MAX_RECEIVER_DESCRIPTOR_BYTES
            }
            Self::ArtifactIdentityPublicKey | Self::LoadDescriptorPublicKey => {
                MAX_RECEIVER_PUBLIC_KEY_BYTES
            }
            Self::ArtifactIdentitySignature | Self::LoadDescriptorSignature => {
                MAX_RECEIVER_SIGNATURE_BYTES
            }
        }
    }

    fn expected_sha256(self, identity: &LocalReceiverIdentity) -> [u8; 32] {
        match self {
            Self::ArtifactIdentityDescriptor => identity.artifact_identity_descriptor_sha256,
            Self::ArtifactIdentityPublicKey => identity.artifact_identity_public_key_sha256,
            Self::ArtifactIdentitySignature => identity.artifact_identity_signature_sha256,
            Self::LoadDescriptor => identity.load_descriptor_sha256,
            Self::LoadDescriptorPublicKey => identity.load_descriptor_public_key_sha256,
            Self::LoadDescriptorSignature => identity.load_descriptor_signature_sha256,
        }
    }
}

pub(crate) fn emit_submit_distribution_catalog_entry(arg: &str) {
    let outcome = submit_distribution_catalog_entry(arg);

    begin_response("module.submit_distribution_catalog_entry");
    emit_record_fields(
        vec![
            f("method", s("module.submit_distribution_catalog_entry")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_id", s(LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID)),
            f("entry_id", s(LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID)),
            f("content_sha256", record_sha_or_null(outcome.content_sha256)),
            f("total_length", V::U64(outcome.total_length as u64)),
            f("chunk_count", V::U64(outcome.chunk_count as u64)),
            f(
                "max_chunk_count",
                V::U64(DISTRIBUTION_CHUNKED_DELIVERY_MAX_CHUNKS as u64),
            ),
            f(
                "signature_byte_len",
                V::U64(outcome.signature_byte_len as u64),
            ),
            f(
                "receiver_identity_retained",
                b(outcome.receiver_identity_retained),
            ),
            f("retained_in_catalog", b(outcome.retained_in_catalog)),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f("reason", s(outcome.reason)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("durable_write_attempted", b(false)),
            f("authorizes_acquisition", b(false)),
            f("authorizes_install", b(false)),
            f("authorizes_load", b(false)),
            f("authorizes_execute", b(false)),
            f("authorizes_persist", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_attempted", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response("module.submit_distribution_catalog_entry");
}

pub(crate) fn emit_submit_distribution_receiver_identity(arg: &str) {
    let outcome = submit_distribution_receiver_identity(arg);

    begin_response("module.submit_distribution_receiver_identity");
    emit_record_fields(
        vec![
            f("method", s("module.submit_distribution_receiver_identity")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_id", s(LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID)),
            f("entry_id", s(LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID)),
            f("content_sha256", record_sha_or_null(outcome.content_sha256)),
            f(
                "receiver_identity",
                outcome
                    .receiver_identity
                    .as_ref()
                    .map(record_receiver_identity)
                    .unwrap_or(V::Null),
            ),
            f("retained_in_catalog", b(outcome.retained_in_catalog)),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f("reason", s(outcome.reason)),
            f("metadata_is_non_authorizing", b(true)),
            f("guest_signature_verification_performed", b(false)),
            f("requires_m6_m7_reverify_for_load", b(true)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("durable_write_attempted", b(false)),
            f("authorizes_acquisition", b(false)),
            f("authorizes_install", b(false)),
            f("authorizes_load", b(false)),
            f("authorizes_execute", b(false)),
            f("authorizes_persist", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_attempted", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response("module.submit_distribution_receiver_identity");
}

pub(crate) fn emit_submit_distribution_receiver_identity_evidence(arg: &str) {
    let outcome = submit_distribution_receiver_identity_evidence(arg);

    emit_distribution_receiver_identity_evidence_response(
        "module.submit_distribution_receiver_identity_evidence",
        &outcome,
    );
}

pub(crate) fn emit_submit_distribution_receiver_identity_finalize(arg: &str) {
    let outcome = submit_distribution_receiver_identity_finalize(arg);

    emit_distribution_receiver_identity_evidence_response(
        "module.submit_distribution_receiver_identity_finalize",
        &outcome,
    );
}

pub(crate) fn emit_distribution_receiver_identity_load_preflight(arg: &str) {
    let outcome = distribution_receiver_identity_load_preflight(arg);
    let receiver_identity_complete = outcome
        .receiver_identity
        .map(|identity| identity.receiver_identity_complete)
        .unwrap_or(false);
    let guest_signature_verification_performed = outcome
        .receiver_identity
        .map(|identity| identity.guest_signature_verification_performed)
        .unwrap_or(false);

    begin_response("module.distribution_receiver_identity_load_preflight");
    emit_record_fields(
        vec![
            f(
                "method",
                s("module.distribution_receiver_identity_load_preflight"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_id", s(LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID)),
            f("entry_id", s(LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID)),
            f("status", s(outcome.status)),
            f("reason", s(outcome.reason)),
            f("content_sha256", record_sha_or_null(outcome.content_sha256)),
            f(
                "retained_part_count",
                V::U64(outcome.retained_part_count as u64),
            ),
            f(
                "receiver_identity",
                outcome
                    .receiver_identity
                    .as_ref()
                    .map(record_receiver_identity)
                    .unwrap_or(V::Null),
            ),
            f(
                "receiver_identity_retained",
                b(outcome.receiver_identity.is_some()),
            ),
            f("receiver_identity_complete", b(receiver_identity_complete)),
            f(
                "guest_signature_verification_performed",
                b(guest_signature_verification_performed),
            ),
            f(
                "retained_candidate_sha256",
                record_sha_or_null(outcome.retained_candidate_sha256),
            ),
            f(
                "retained_candidate_present",
                b(outcome.retained_candidate_sha256.is_some()),
            ),
            f(
                "retained_candidate_wasm_valid",
                b(outcome.retained_candidate_wasm_valid),
            ),
            f(
                "catalog_finalize_candidate_sha256",
                record_sha_or_null(outcome.catalog_finalize_candidate_sha256),
            ),
            f(
                "retained_candidate_matches_catalog_finalize",
                b(outcome.retained_candidate_matches_catalog_finalize),
            ),
            f("preflight_evaluated", b(outcome.preflight_evaluated)),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f(
                "missing_gate_count",
                V::U64(outcome.missing_gate_count as u64),
            ),
            f("m6_reverification_gate_satisfied", b(false)),
            f("m7_loader_policy_gate_satisfied", b(false)),
            f("provider_trust_gate_satisfied", b(false)),
            f("owner_seal_gate_satisfied", b(false)),
            f("requires_m6_m7_reverify_for_load", b(true)),
            f("requires_provider_trust_for_load", b(true)),
            f("requires_owner_seal_for_load", b(true)),
            f("can_load_now", b(false)),
            f("load_authorized", b(false)),
            f("install_authorized", b(false)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("durable_write_attempted", b(false)),
            f("authorizes_acquisition", b(false)),
            f("authorizes_install", b(false)),
            f("authorizes_load", b(false)),
            f("authorizes_execute", b(false)),
            f("authorizes_persist", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_attempted", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response("module.distribution_receiver_identity_load_preflight");
}

fn emit_distribution_receiver_identity_evidence_response(
    method: &'static str,
    outcome: &DistributionReceiverIdentityEvidenceOutcome,
) {
    let receiver_identity_complete = outcome
        .receiver_identity
        .map(|identity| identity.receiver_identity_complete)
        .unwrap_or(false);
    let guest_signature_verification_performed = outcome
        .receiver_identity
        .map(|identity| identity.guest_signature_verification_performed)
        .unwrap_or(false);

    begin_response(method);
    emit_record_fields(
        vec![
            f("method", s(method)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_id", s(LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID)),
            f("entry_id", s(LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID)),
            f("content_sha256", record_sha_or_null(outcome.content_sha256)),
            f("evidence_kind", s(outcome.evidence_kind)),
            f(
                "evidence_sha256",
                record_sha_or_null(outcome.evidence_sha256),
            ),
            f("decoded_byte_len", V::U64(outcome.decoded_byte_len as u64)),
            f(
                "retained_part_count",
                V::U64(outcome.retained_part_count as u64),
            ),
            f(
                "receiver_identity",
                outcome
                    .receiver_identity
                    .as_ref()
                    .map(record_receiver_identity)
                    .unwrap_or(V::Null),
            ),
            f("receiver_identity_complete", b(receiver_identity_complete)),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f("reason", s(outcome.reason)),
            f("metadata_is_non_authorizing", b(true)),
            f(
                "guest_signature_verification_performed",
                b(guest_signature_verification_performed),
            ),
            f("requires_m6_m7_reverify_for_load", b(true)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("durable_write_attempted", b(false)),
            f("authorizes_acquisition", b(false)),
            f("authorizes_install", b(false)),
            f("authorizes_load", b(false)),
            f("authorizes_execute", b(false)),
            f("authorizes_persist", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_attempted", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response(method);
}

pub(crate) fn emit_submit_distribution_begin(arg: &str) {
    let outcome = submit_distribution_begin(arg);

    emit_distribution_begin_response("module.submit_distribution_begin", &outcome);
}

pub(crate) fn emit_submit_distribution_begin_from_catalog(arg: &str) {
    let outcome = submit_distribution_begin_from_catalog(arg);

    emit_distribution_begin_response("module.submit_distribution_begin_from_catalog", &outcome);
}

fn emit_distribution_begin_response(method: &'static str, outcome: &DistributionBeginOutcome) {
    begin_response(method);
    emit_record_fields(
        vec![
            f("method", s(method)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("delivery_channel", s(SERIAL_DISTRIBUTION_DELIVERY_CHANNEL)),
            f("source_id", s(outcome.source_id)),
            f("entry_id", s(outcome.entry_id)),
            f("content_sha256", record_sha_or_null(outcome.content_sha256)),
            f("total_length", V::U64(outcome.total_length as u64)),
            f("chunk_count", V::U64(outcome.chunk_count as u64)),
            f(
                "max_chunk_count",
                V::U64(DISTRIBUTION_CHUNKED_DELIVERY_MAX_CHUNKS as u64),
            ),
            f(
                "signature_byte_len",
                V::U64(outcome.signature_byte_len as u64),
            ),
            f(
                "receiver_identity",
                outcome
                    .receiver_identity
                    .as_ref()
                    .map(record_receiver_identity)
                    .unwrap_or(V::Null),
            ),
            f(
                "receiver_identity_retained",
                b(outcome.receiver_identity.is_some()),
            ),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f("reason", s(outcome.reason)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("durable_write_attempted", b(false)),
            f("authorizes_acquisition", b(false)),
            f("authorizes_install", b(false)),
            f("authorizes_load", b(false)),
            f("authorizes_execute", b(false)),
            f("authorizes_persist", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_attempted", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response(method);
}

pub(crate) fn emit_submit_distribution_chunk(arg: &str) {
    let outcome = submit_distribution_chunk(arg);

    begin_response("module.submit_distribution_chunk");
    emit_record_fields(
        vec![
            f("method", s("module.submit_distribution_chunk")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("delivery_channel", s(SERIAL_DISTRIBUTION_DELIVERY_CHANNEL)),
            f("content_sha256", record_sha_or_null(outcome.content_sha256)),
            f("chunk_index", V::U64(outcome.chunk_index as u64)),
            f("chunk_sha256", record_sha_or_null(outcome.chunk_sha256)),
            f("decoded_byte_len", V::U64(outcome.decoded_byte_len as u64)),
            f(
                "pending_chunk_count",
                V::U64(outcome.pending_chunk_count as u64),
            ),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f(
                "discarded_pending_delivery",
                b(outcome.discarded_pending_delivery),
            ),
            f("reason", s(outcome.reason)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("durable_write_attempted", b(false)),
            f("authorizes_acquisition", b(false)),
            f("authorizes_install", b(false)),
            f("authorizes_load", b(false)),
            f("authorizes_execute", b(false)),
            f("authorizes_persist", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_attempted", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response("module.submit_distribution_chunk");
}

pub(crate) fn emit_submit_distribution_finalize() {
    let outcome = submit_distribution_finalize();

    begin_response("module.submit_distribution_finalize");
    emit_record_fields(
        vec![
            f("method", s("module.submit_distribution_finalize")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("delivery_channel", s(SERIAL_DISTRIBUTION_DELIVERY_CHANNEL)),
            f("source_id", s(outcome.source_id)),
            f("entry_id", s(outcome.entry_id)),
            f("status", s(outcome.status)),
            f("reason", s(outcome.reason)),
            f("content_sha256", record_sha_or_null(outcome.content_sha256)),
            f("total_length", V::U64(outcome.total_length as u64)),
            f(
                "declared_chunk_count",
                V::U64(outcome.declared_chunk_count as u64),
            ),
            f(
                "accepted_chunk_count",
                V::U64(outcome.accepted_chunk_count as u64),
            ),
            f(
                "delivered_byte_len",
                V::U64(outcome.delivered_byte_len as u64),
            ),
            f(
                "selection",
                outcome
                    .selection
                    .as_ref()
                    .map(record_distribution_transport_selection)
                    .unwrap_or(V::Null),
            ),
            f(
                "staged_candidate",
                outcome
                    .staged_candidate
                    .as_ref()
                    .map(record_candidate_outcome)
                    .unwrap_or(V::Null),
            ),
            f(
                "retained_provenance",
                outcome
                    .retained_provenance
                    .as_ref()
                    .map(record_retained_provenance)
                    .unwrap_or(V::Null),
            ),
            f(
                "receiver_identity",
                outcome
                    .receiver_identity
                    .as_ref()
                    .map(record_receiver_identity)
                    .unwrap_or(V::Null),
            ),
            f(
                "receiver_identity_retained",
                b(outcome.receiver_identity.is_some()),
            ),
            f(
                "staged_only_after_valid_selection",
                b(outcome.staged_candidate.is_some()
                    == outcome
                        .selection
                        .map(|selection| selection.selected_for_candidate_intake)
                        .unwrap_or(false)),
            ),
            f("provenance_is_origin_evidence_only", b(true)),
            f("requires_m6_reverify_for_load", b(true)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("durable_write_attempted", b(false)),
            f("authorizes_acquisition", b(false)),
            f("authorizes_install", b(false)),
            f("authorizes_load", b(false)),
            f("authorizes_execute", b(false)),
            f("authorizes_persist", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_attempted", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response("module.submit_distribution_finalize");
}

fn submit_distribution_begin(arg: &str) -> DistributionBeginOutcome {
    clear_pending_distribution();

    let payload = distribution_payload(arg, "module.submit_distribution_begin");
    let metadata = match parse_distribution_target_metadata(payload) {
        Ok(metadata) => metadata,
        Err(reason) => {
            return rejected_distribution_begin_for(
                SERIAL_DISTRIBUTION_SOURCE_ID,
                SERIAL_DISTRIBUTION_ENTRY_ID,
                reason,
            )
        }
    };

    accept_distribution_begin(
        SERIAL_DISTRIBUTION_SOURCE_ID,
        SERIAL_DISTRIBUTION_ENTRY_ID,
        metadata,
        DISTRIBUTION_BEGIN_ACCEPTED_REASON,
    )
}

fn submit_distribution_catalog_entry(arg: &str) -> DistributionCatalogEntryOutcome {
    let mut catalog = LOCAL_DISTRIBUTION_CATALOG.lock();
    catalog.clear();

    let payload = distribution_payload(arg, "module.submit_distribution_catalog_entry");
    let metadata = match parse_distribution_target_metadata(payload) {
        Ok(metadata) => metadata,
        Err(reason) => return rejected_distribution_catalog_entry(reason),
    };

    catalog.active = true;
    catalog.content_sha256 = metadata.content_sha256;
    catalog.total_length = metadata.total_length;
    catalog.chunk_count = metadata.chunk_count;
    catalog.provenance_signature_der = metadata.provenance_signature_der;
    catalog.receiver_identity = None;
    catalog.finalized_candidate_sha256 = None;

    DistributionCatalogEntryOutcome {
        content_sha256: Some(catalog.content_sha256),
        total_length: catalog.total_length,
        chunk_count: catalog.chunk_count,
        signature_byte_len: catalog.provenance_signature_der.len(),
        receiver_identity_retained: false,
        retained_in_catalog: true,
        accepted: true,
        rejected: false,
        reason: DISTRIBUTION_CATALOG_ENTRY_ACCEPTED_REASON,
    }
}

fn submit_distribution_receiver_identity(arg: &str) -> DistributionReceiverIdentityOutcome {
    let payload = distribution_payload(arg, "module.submit_distribution_receiver_identity");
    let (content_sha256, receiver_identity) = match parse_receiver_identity_metadata(payload) {
        Ok(metadata) => metadata,
        Err(reason) => return rejected_receiver_identity(reason),
    };

    let mut catalog = LOCAL_DISTRIBUTION_CATALOG.lock();
    if !catalog.active || catalog.content_sha256 != content_sha256 {
        return rejected_receiver_identity(DISTRIBUTION_RECEIVER_IDENTITY_CATALOG_NOT_FOUND_REASON);
    }
    catalog.receiver_identity = Some(receiver_identity);
    catalog.receiver_identity_evidence.clear();

    DistributionReceiverIdentityOutcome {
        content_sha256: Some(content_sha256),
        receiver_identity: Some(receiver_identity),
        retained_in_catalog: true,
        accepted: true,
        rejected: false,
        reason: DISTRIBUTION_RECEIVER_IDENTITY_ACCEPTED_REASON,
    }
}

fn submit_distribution_receiver_identity_evidence(
    arg: &str,
) -> DistributionReceiverIdentityEvidenceOutcome {
    let payload =
        distribution_payload(arg, "module.submit_distribution_receiver_identity_evidence");
    let mut words = payload.split_whitespace();
    let content_sha256 = match next_sha256_or(
        &mut words,
        DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
    ) {
        Ok(value) => value,
        Err(reason) => return rejected_receiver_identity_evidence(reason),
    };
    let Some(kind_token) = words.next() else {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
        );
    };
    let Some(kind) = ReceiverIdentityEvidenceKind::parse(kind_token) else {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
        );
    };
    let expected_sha256 = match next_sha256_or(
        &mut words,
        DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
    ) {
        Ok(value) => value,
        Err(reason) => return rejected_receiver_identity_evidence(reason),
    };
    let Some(payload_token) = words.next() else {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
        );
    };
    if words.next().is_some() {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
        );
    }
    if payload_token.len() > max_base64_encoded_len(kind.max_bytes()) {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_TOO_LARGE_REASON,
        );
    }

    let decoded = match module_candidate_channel::decode_base64_chunk(payload_token) {
        Ok(decoded) => decoded,
        Err(reason) => return rejected_receiver_identity_evidence(reason),
    };
    let decoded_byte_len = decoded.len();
    if decoded_byte_len == 0 || decoded_byte_len > kind.max_bytes() {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_TOO_LARGE_REASON,
        );
    }
    if sha256_bytes(&decoded) != expected_sha256 {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_HASH_MISMATCH_REASON,
        );
    }

    let mut catalog = LOCAL_DISTRIBUTION_CATALOG.lock();
    if !catalog.active || catalog.content_sha256 != content_sha256 {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_CATALOG_NOT_FOUND_REASON,
        );
    }
    let Some(identity) = catalog.receiver_identity else {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_CATALOG_NOT_FOUND_REASON,
        );
    };
    if kind.expected_sha256(&identity) != expected_sha256 {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_HASH_MISMATCH_REASON,
        );
    }
    catalog.receiver_identity_evidence.set(kind, decoded);
    let retained_part_count = catalog.receiver_identity_evidence.retained_part_count();

    DistributionReceiverIdentityEvidenceOutcome {
        content_sha256: Some(content_sha256),
        evidence_kind: kind.token(),
        evidence_sha256: Some(expected_sha256),
        decoded_byte_len,
        retained_part_count,
        receiver_identity: catalog.receiver_identity,
        accepted: true,
        rejected: false,
        reason: DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_ACCEPTED_REASON,
    }
}

fn submit_distribution_receiver_identity_finalize(
    arg: &str,
) -> DistributionReceiverIdentityEvidenceOutcome {
    let payload =
        distribution_payload(arg, "module.submit_distribution_receiver_identity_finalize");
    let mut words = payload.split_whitespace();
    let content_sha256 = match next_sha256_or(
        &mut words,
        DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
    ) {
        Ok(value) => value,
        Err(reason) => return rejected_receiver_identity_evidence(reason),
    };
    if words.next().is_some() {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INVALID_REASON,
        );
    }

    let mut catalog = LOCAL_DISTRIBUTION_CATALOG.lock();
    if !catalog.active || catalog.content_sha256 != content_sha256 {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_CATALOG_NOT_FOUND_REASON,
        );
    }
    let Some(mut identity) = catalog.receiver_identity else {
        return rejected_receiver_identity_evidence(
            DISTRIBUTION_RECEIVER_IDENTITY_CATALOG_NOT_FOUND_REASON,
        );
    };
    let retained_part_count = catalog.receiver_identity_evidence.retained_part_count();
    if !catalog.receiver_identity_evidence.is_complete() {
        return DistributionReceiverIdentityEvidenceOutcome {
            content_sha256: Some(content_sha256),
            evidence_kind: "finalize",
            evidence_sha256: None,
            decoded_byte_len: 0,
            retained_part_count,
            receiver_identity: catalog.receiver_identity,
            accepted: false,
            rejected: true,
            reason: DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INCOMPLETE_REASON,
        };
    }

    if !verify_receiver_identity_evidence(
        catalog.content_sha256,
        &identity,
        &catalog.receiver_identity_evidence,
    ) {
        catalog.receiver_identity_evidence.clear();
        return DistributionReceiverIdentityEvidenceOutcome {
            content_sha256: Some(content_sha256),
            evidence_kind: "finalize",
            evidence_sha256: None,
            decoded_byte_len: 0,
            retained_part_count: 0,
            receiver_identity: catalog.receiver_identity,
            accepted: false,
            rejected: true,
            reason: DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_VERIFY_FAILED_REASON,
        };
    }

    identity.artifact_identity_signature_verified_by_guest = true;
    identity.load_descriptor_signature_verified_by_guest = true;
    identity.guest_signature_verification_performed = true;
    identity.receiver_identity_complete = true;
    catalog.receiver_identity = Some(identity);

    DistributionReceiverIdentityEvidenceOutcome {
        content_sha256: Some(content_sha256),
        evidence_kind: "finalize",
        evidence_sha256: None,
        decoded_byte_len: 0,
        retained_part_count,
        receiver_identity: Some(identity),
        accepted: true,
        rejected: false,
        reason: DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_FINALIZED_REASON,
    }
}

fn distribution_receiver_identity_load_preflight(
    arg: &str,
) -> DistributionReceiverIdentityLoadPreflightOutcome {
    let payload = distribution_payload(arg, "module.distribution_receiver_identity_load_preflight");
    let mut words = payload.split_whitespace();
    let content_sha256 = match next_sha256_or(
        &mut words,
        DISTRIBUTION_RECEIVER_IDENTITY_PREFLIGHT_INVALID_REASON,
    ) {
        Ok(value) => value,
        Err(reason) => return rejected_receiver_identity_load_preflight(reason),
    };
    if words.next().is_some() {
        return rejected_receiver_identity_load_preflight(
            DISTRIBUTION_RECEIVER_IDENTITY_PREFLIGHT_INVALID_REASON,
        );
    }

    let catalog = LOCAL_DISTRIBUTION_CATALOG.lock();
    if !catalog.active || catalog.content_sha256 != content_sha256 {
        return rejected_receiver_identity_load_preflight(
            DISTRIBUTION_RECEIVER_IDENTITY_CATALOG_NOT_FOUND_REASON,
        );
    }
    let retained_part_count = catalog.receiver_identity_evidence.retained_part_count();
    let catalog_finalize_candidate_sha256 = catalog.finalized_candidate_sha256;
    let Some(identity) = catalog.receiver_identity else {
        return DistributionReceiverIdentityLoadPreflightOutcome {
            content_sha256: Some(content_sha256),
            retained_part_count,
            receiver_identity: None,
            retained_candidate_sha256: None,
            retained_candidate_wasm_valid: false,
            catalog_finalize_candidate_sha256,
            retained_candidate_matches_catalog_finalize: false,
            status: "denied",
            reason: DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INCOMPLETE_REASON,
            preflight_evaluated: false,
            accepted: false,
            rejected: true,
            missing_gate_count: 0,
        };
    };
    if !identity.receiver_identity_complete
        || !identity.guest_signature_verification_performed
        || !identity.artifact_identity_signature_verified_by_guest
        || !identity.load_descriptor_signature_verified_by_guest
    {
        return DistributionReceiverIdentityLoadPreflightOutcome {
            content_sha256: Some(content_sha256),
            retained_part_count,
            receiver_identity: Some(identity),
            retained_candidate_sha256: None,
            retained_candidate_wasm_valid: false,
            catalog_finalize_candidate_sha256,
            retained_candidate_matches_catalog_finalize: false,
            status: "denied",
            reason: DISTRIBUTION_RECEIVER_IDENTITY_EVIDENCE_INCOMPLETE_REASON,
            preflight_evaluated: false,
            accepted: false,
            rejected: true,
            missing_gate_count: 0,
        };
    }
    let retained_candidate = module_candidate_intake::retained();
    let retained_candidate_sha256 = retained_candidate
        .as_ref()
        .map(|candidate| candidate.sha256);
    let retained_candidate_wasm_valid = retained_candidate
        .as_ref()
        .map(|candidate| candidate.wasm_valid)
        .unwrap_or(false);
    let retained_candidate_matches_catalog_finalize = retained_candidate
        .as_ref()
        .map(|candidate| {
            candidate.wasm_valid
                && candidate.sha256 == content_sha256
                && catalog_finalize_candidate_sha256 == Some(content_sha256)
        })
        .unwrap_or(false);
    if !retained_candidate_matches_catalog_finalize {
        return DistributionReceiverIdentityLoadPreflightOutcome {
            content_sha256: Some(content_sha256),
            retained_part_count,
            receiver_identity: Some(identity),
            retained_candidate_sha256,
            retained_candidate_wasm_valid,
            catalog_finalize_candidate_sha256,
            retained_candidate_matches_catalog_finalize,
            status: "denied",
            reason: DISTRIBUTION_RECEIVER_IDENTITY_PREFLIGHT_CANDIDATE_NOT_FINALIZED_REASON,
            preflight_evaluated: false,
            accepted: false,
            rejected: true,
            missing_gate_count: 0,
        };
    }

    DistributionReceiverIdentityLoadPreflightOutcome {
        content_sha256: Some(content_sha256),
        retained_part_count,
        receiver_identity: Some(identity),
        retained_candidate_sha256,
        retained_candidate_wasm_valid,
        catalog_finalize_candidate_sha256,
        retained_candidate_matches_catalog_finalize,
        status: "denied",
        reason: DISTRIBUTION_RECEIVER_IDENTITY_PREFLIGHT_MISSING_GATES_REASON,
        preflight_evaluated: true,
        accepted: true,
        rejected: false,
        missing_gate_count: 4,
    }
}

fn submit_distribution_begin_from_catalog(arg: &str) -> DistributionBeginOutcome {
    clear_pending_distribution();

    let payload = distribution_payload(arg, "module.submit_distribution_begin_from_catalog");
    let mut words = payload.split_whitespace();
    let Some(content_sha_token) = words.next() else {
        return rejected_distribution_begin_for(
            LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID,
            LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID,
            DISTRIBUTION_INVALID_ARGS_REASON,
        );
    };
    if words.next().is_some() {
        return rejected_distribution_begin_for(
            LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID,
            LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID,
            DISTRIBUTION_INVALID_ARGS_REASON,
        );
    }
    let Some(content_sha256) = parse_sha256_ref(content_sha_token) else {
        return rejected_distribution_begin_for(
            LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID,
            LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID,
            DISTRIBUTION_INVALID_ARGS_REASON,
        );
    };

    let metadata = {
        let catalog = LOCAL_DISTRIBUTION_CATALOG.lock();
        if !catalog.active || catalog.content_sha256 != content_sha256 {
            return rejected_distribution_begin_for(
                LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID,
                LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID,
                DISTRIBUTION_CATALOG_ENTRY_NOT_FOUND_REASON,
            );
        }
        DistributionDeliveryTargetMetadata {
            content_sha256: catalog.content_sha256,
            total_length: catalog.total_length,
            chunk_count: catalog.chunk_count,
            provenance_signature_der: catalog.provenance_signature_der.clone(),
            receiver_identity: catalog.receiver_identity,
        }
    };

    accept_distribution_begin(
        LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID,
        LOCAL_DISTRIBUTION_CATALOG_ENTRY_ID,
        metadata,
        DISTRIBUTION_CATALOG_BEGIN_ACCEPTED_REASON,
    )
}

fn accept_distribution_begin(
    source_id: &'static str,
    entry_id: &'static str,
    metadata: DistributionDeliveryTargetMetadata,
    reason: &'static str,
) -> DistributionBeginOutcome {
    let mut pending = PENDING_SERIAL_DISTRIBUTION.lock();
    pending.clear();
    pending.active = true;
    pending.source_id = source_id;
    pending.entry_id = entry_id;
    pending.content_sha256 = metadata.content_sha256;
    pending.total_length = metadata.total_length;
    pending.chunk_count = metadata.chunk_count;
    pending.provenance_signature_der = metadata.provenance_signature_der;
    pending.receiver_identity = metadata.receiver_identity;

    DistributionBeginOutcome {
        source_id,
        entry_id,
        content_sha256: Some(pending.content_sha256),
        total_length: pending.total_length,
        chunk_count: pending.chunk_count,
        signature_byte_len: pending.provenance_signature_der.len(),
        receiver_identity: pending.receiver_identity,
        accepted: true,
        rejected: false,
        reason,
    }
}

fn submit_distribution_chunk(arg: &str) -> DistributionChunkOutcome {
    let mut words =
        distribution_payload(arg, "module.submit_distribution_chunk").split_whitespace();
    let Some(index_token) = words.next() else {
        return reject_distribution_chunk_with_clear(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(chunk_sha_token) = words.next() else {
        return reject_distribution_chunk_with_clear(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(payload_token) = words.next() else {
        return reject_distribution_chunk_with_clear(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    if words.next().is_some() {
        return reject_distribution_chunk_with_clear(DISTRIBUTION_INVALID_ARGS_REASON);
    }

    let Some(index) = parse_usize(index_token) else {
        return reject_distribution_chunk_with_clear(DISTRIBUTION_INVALID_CHUNK_INDEX_REASON);
    };
    let Some(claimed_chunk_sha256) = parse_sha256_ref(chunk_sha_token) else {
        return reject_distribution_chunk_with_clear(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let decoded = match module_candidate_channel::decode_base64_chunk(payload_token) {
        Ok(decoded) => decoded,
        Err(reason) => return reject_distribution_chunk_with_clear(reason),
    };
    let decoded_byte_len = decoded.len();
    let actual_chunk_sha256 = sha256_bytes(&decoded);
    if actual_chunk_sha256 != claimed_chunk_sha256 {
        return reject_distribution_chunk_with_clear(
            ChunkedDeliveryError::ChunkHashMismatch.as_str(),
        );
    }

    let mut pending = PENDING_SERIAL_DISTRIBUTION.lock();
    if !pending.active {
        return rejected_distribution_chunk(
            None,
            index,
            Some(claimed_chunk_sha256),
            decoded_byte_len,
            0,
            false,
            DISTRIBUTION_DELIVERY_NOT_STARTED_REASON,
        );
    }
    if index >= pending.chunk_count {
        pending.clear();
        return rejected_distribution_chunk(
            None,
            index,
            Some(claimed_chunk_sha256),
            decoded_byte_len,
            0,
            true,
            ChunkedDeliveryError::ChunkIndexOutOfRange.as_str(),
        );
    }
    if decoded_byte_len > pending.total_length {
        pending.clear();
        return rejected_distribution_chunk(
            None,
            index,
            Some(claimed_chunk_sha256),
            decoded_byte_len,
            0,
            true,
            ChunkedDeliveryError::TotalLengthOverflow.as_str(),
        );
    }
    if let Some(existing) = pending.chunks.iter().find(|chunk| chunk.index == index) {
        if existing.bytes == decoded && existing.claimed_chunk_sha256 == claimed_chunk_sha256 {
            return accepted_distribution_chunk(
                pending.content_sha256,
                index,
                claimed_chunk_sha256,
                decoded_byte_len,
                pending.chunks.len(),
                DISTRIBUTION_DUPLICATE_CHUNK_ACCEPTED_REASON,
            );
        }
        pending.clear();
        return rejected_distribution_chunk(
            None,
            index,
            Some(claimed_chunk_sha256),
            decoded_byte_len,
            0,
            true,
            ChunkedDeliveryError::DuplicateChunkBytesMismatch.as_str(),
        );
    }
    if pending.chunks.len() >= pending.chunk_count
        || pending.chunks.len() >= DISTRIBUTION_CHUNKED_DELIVERY_MAX_CHUNKS
    {
        pending.clear();
        return rejected_distribution_chunk(
            None,
            index,
            Some(claimed_chunk_sha256),
            decoded_byte_len,
            0,
            true,
            ChunkedDeliveryError::ChunkCapacityExceeded.as_str(),
        );
    }

    pending.chunks.push(PendingSerialDistributionChunk {
        index,
        bytes: decoded,
        claimed_chunk_sha256,
    });
    accepted_distribution_chunk(
        pending.content_sha256,
        index,
        claimed_chunk_sha256,
        decoded_byte_len,
        pending.chunks.len(),
        DISTRIBUTION_CHUNK_ACCEPTED_REASON,
    )
}

fn submit_distribution_finalize() -> DistributionFinalizeOutcome {
    let mut pending = PENDING_SERIAL_DISTRIBUTION.lock();
    if !pending.active {
        return DistributionFinalizeOutcome {
            source_id: SERIAL_DISTRIBUTION_SOURCE_ID,
            entry_id: SERIAL_DISTRIBUTION_ENTRY_ID,
            content_sha256: None,
            total_length: 0,
            declared_chunk_count: 0,
            accepted_chunk_count: 0,
            delivered_byte_len: 0,
            selection: None,
            staged_candidate: None,
            retained_provenance: None,
            receiver_identity: None,
            status: "denied",
            reason: DISTRIBUTION_DELIVERY_NOT_STARTED_REASON,
        };
    }

    let content_sha256 = pending.content_sha256;
    let source_id = pending.source_id;
    let entry_id = pending.entry_id;
    let total_length = pending.total_length;
    let declared_chunk_count = pending.chunk_count;
    let accepted_chunk_count = pending.chunks.len();
    if accepted_chunk_count != declared_chunk_count {
        pending.clear();
        return rejected_distribution_finalize(
            source_id,
            entry_id,
            Some(content_sha256),
            total_length,
            declared_chunk_count,
            accepted_chunk_count,
            ChunkedDeliveryError::ChunkSetNotComplete.as_str(),
        );
    }

    let finalized = finalize_pending_distribution(&pending);
    pending.clear();
    finalized
}

fn finalize_pending_distribution(
    pending: &PendingSerialDistribution,
) -> DistributionFinalizeOutcome {
    let mut delivery = ChunkedDistributionDelivery::new(ChunkedDistributionTarget {
        entry_id: pending.entry_id,
        content_sha256: pending.content_sha256,
        total_length: pending.total_length,
        chunk_count: pending.chunk_count,
        provenance_signature_der: Some(&pending.provenance_signature_der),
        publisher_key_sha256: PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
        classification: "local_only",
    });

    let mut idx = 0usize;
    while idx < pending.chunks.len() {
        let chunk = &pending.chunks[idx];
        if let Err(reason) = delivery.accept_chunk(ChunkedDistributionChunkInput {
            index: chunk.index,
            bytes: &chunk.bytes,
            claimed_chunk_sha256: chunk.claimed_chunk_sha256,
        }) {
            return rejected_distribution_finalize(
                pending.source_id,
                pending.entry_id,
                Some(pending.content_sha256),
                pending.total_length,
                pending.chunk_count,
                pending.chunks.len(),
                reason.as_str(),
            );
        }
        idx += 1;
    }

    let mut reassembled = vec![0u8; pending.total_length];
    let entry = match delivery.try_finalize(&mut reassembled) {
        Ok(entry) => entry,
        Err(reason) => {
            return rejected_distribution_finalize(
                pending.source_id,
                pending.entry_id,
                Some(pending.content_sha256),
                pending.total_length,
                pending.chunk_count,
                pending.chunks.len(),
                reason.as_str(),
            )
        }
    };
    let selection = evaluate_distribution_registry_selection(&entry, entry.artifact_sha256);
    let selection_summary = distribution_transport_selection(pending.entry_id, &selection);
    if !selection.selected_for_candidate_intake {
        return DistributionFinalizeOutcome {
            source_id: pending.source_id,
            entry_id: pending.entry_id,
            content_sha256: Some(pending.content_sha256),
            total_length: pending.total_length,
            declared_chunk_count: pending.chunk_count,
            accepted_chunk_count: pending.chunks.len(),
            delivered_byte_len: 0,
            selection: Some(selection_summary),
            staged_candidate: None,
            retained_provenance: None,
            receiver_identity: pending.receiver_identity,
            status: "denied",
            reason: selection.reason.as_str(),
        };
    }

    let staged_candidate = module_candidate_intake::intake_and_retain_external_wasm_candidate(
        Vec::from(entry.artifact_bytes),
    );
    let retained_provenance = distribution_candidate::verify_retained_candidate_provenance(
        entry.provenance_signature_der,
    );
    if pending.source_id == LOCAL_DISTRIBUTION_CATALOG_SOURCE_ID
        && staged_candidate.retained_in_ram
        && staged_candidate.wasm_valid
        && !staged_candidate.rejected
        && staged_candidate.artifact_sha256 == pending.content_sha256
    {
        LOCAL_DISTRIBUTION_CATALOG.lock().finalized_candidate_sha256 =
            Some(staged_candidate.artifact_sha256);
    }

    DistributionFinalizeOutcome {
        source_id: pending.source_id,
        entry_id: pending.entry_id,
        content_sha256: Some(pending.content_sha256),
        total_length: pending.total_length,
        declared_chunk_count: pending.chunk_count,
        accepted_chunk_count: pending.chunks.len(),
        delivered_byte_len: staged_candidate.byte_len,
        selection: Some(selection_summary),
        staged_candidate: Some(staged_candidate),
        retained_provenance: Some(retained_provenance),
        receiver_identity: pending.receiver_identity,
        status: selection.status,
        reason: selection.reason.as_str(),
    }
}

fn distribution_payload<'a>(arg: &'a str, method: &str) -> &'a str {
    let trimmed = arg.trim();
    if method_head_eq(trimmed, method) {
        trimmed.strip_prefix(method).unwrap_or(trimmed).trim()
    } else {
        trimmed
    }
}

fn decode_signature_token(token: &str) -> Option<Vec<u8>> {
    let hex = token.strip_prefix("sig:").unwrap_or(token).as_bytes();
    let (bytes, len) = decode_distribution_signature_der_hex(hex)?;
    Some(Vec::from(&bytes[..len]))
}

fn parse_distribution_target_metadata(
    payload: &str,
) -> Result<DistributionDeliveryTargetMetadata, &'static str> {
    let mut words = payload.split_whitespace();
    let Some(content_sha_token) = words.next() else {
        return Err(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(total_length_token) = words.next() else {
        return Err(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(chunk_count_token) = words.next() else {
        return Err(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(signature_token) = words.next() else {
        return Err(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    if words.next().is_some() {
        return Err(DISTRIBUTION_INVALID_ARGS_REASON);
    }

    let Some(content_sha256) = parse_sha256_ref(content_sha_token) else {
        return Err(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(total_length) = parse_positive_usize(total_length_token) else {
        return Err(DISTRIBUTION_INVALID_LENGTH_REASON);
    };
    if total_length > module_candidate_intake::MAX_EXTERNAL_WASM_CANDIDATE_BYTES {
        return Err(DISTRIBUTION_INVALID_LENGTH_REASON);
    }
    let Some(chunk_count) = parse_positive_usize(chunk_count_token) else {
        return Err(DISTRIBUTION_INVALID_CHUNK_COUNT_REASON);
    };
    if chunk_count > DISTRIBUTION_CHUNKED_DELIVERY_MAX_CHUNKS {
        return Err(DISTRIBUTION_INVALID_CHUNK_COUNT_REASON);
    }
    let Some(provenance_signature_der) = decode_signature_token(signature_token) else {
        return Err(DISTRIBUTION_INVALID_SIGNATURE_REASON);
    };

    Ok(DistributionDeliveryTargetMetadata {
        content_sha256,
        total_length,
        chunk_count,
        provenance_signature_der,
        receiver_identity: None,
    })
}

fn parse_receiver_identity_metadata(
    payload: &str,
) -> Result<([u8; 32], LocalReceiverIdentity), &'static str> {
    let mut words = payload.split_whitespace();
    let content_sha256 = next_sha256(&mut words)?;
    let artifact_identity_descriptor_sha256 = next_sha256(&mut words)?;
    let artifact_identity_public_key_sha256 = next_sha256(&mut words)?;
    let artifact_identity_signature_sha256 = next_sha256(&mut words)?;
    let load_descriptor_sha256 = next_sha256(&mut words)?;
    let load_descriptor_public_key_sha256 = next_sha256(&mut words)?;
    let load_descriptor_signature_sha256 = next_sha256(&mut words)?;
    expect_next_token(&mut words, "classification:local_only")?;
    expect_next_token(&mut words, "artifact_identity_signature_verified:true")?;
    expect_next_token(&mut words, "load_descriptor_signature_verified:true")?;
    expect_next_token(&mut words, "artifact_hash_bound_by_identity:true")?;
    expect_next_token(&mut words, "artifact_hash_bound_by_load_descriptor:true")?;
    expect_next_token(&mut words, "load_descriptor_binds_artifact_identity:true")?;
    expect_next_token(
        &mut words,
        "load_descriptor_authorizes_current_boot_wasm_execution:true",
    )?;
    expect_next_token(&mut words, "export_authorizes_load:false")?;
    expect_next_token(&mut words, "export_authorizes_install:false")?;
    expect_next_token(&mut words, "export_authorizes_execute:false")?;
    expect_next_token(&mut words, "export_writes_persistent_state:false")?;
    expect_next_token(&mut words, "requires_m6_m7_reverify_for_load:true")?;
    if words.next().is_some() {
        return Err(DISTRIBUTION_RECEIVER_IDENTITY_INVALID_REASON);
    }

    Ok((
        content_sha256,
        LocalReceiverIdentity {
            artifact_identity_descriptor_sha256,
            artifact_identity_public_key_sha256,
            artifact_identity_signature_sha256,
            load_descriptor_sha256,
            load_descriptor_public_key_sha256,
            load_descriptor_signature_sha256,
            artifact_identity_signature_verified_by_host_export: true,
            load_descriptor_signature_verified_by_host_export: true,
            artifact_hash_bound_by_identity: true,
            artifact_hash_bound_by_load_descriptor: true,
            load_descriptor_binds_artifact_identity: true,
            load_descriptor_authorizes_current_boot_wasm_execution: true,
            artifact_identity_signature_verified_by_guest: false,
            load_descriptor_signature_verified_by_guest: false,
            guest_signature_verification_performed: false,
            receiver_identity_complete: false,
        },
    ))
}

fn verify_receiver_identity_evidence(
    content_sha256: [u8; 32],
    identity: &LocalReceiverIdentity,
    evidence: &LocalReceiverIdentityEvidencePayloads,
) -> bool {
    let Some(artifact_identity_descriptor) = evidence.artifact_identity_descriptor.as_ref() else {
        return false;
    };
    let Some(artifact_identity_public_key) = evidence.artifact_identity_public_key.as_ref() else {
        return false;
    };
    let Some(artifact_identity_signature) = evidence.artifact_identity_signature.as_ref() else {
        return false;
    };
    let Some(load_descriptor) = evidence.load_descriptor.as_ref() else {
        return false;
    };
    let Some(load_descriptor_public_key) = evidence.load_descriptor_public_key.as_ref() else {
        return false;
    };
    let Some(load_descriptor_signature) = evidence.load_descriptor_signature.as_ref() else {
        return false;
    };

    if sha256_bytes(artifact_identity_descriptor) != identity.artifact_identity_descriptor_sha256
        || sha256_bytes(artifact_identity_public_key)
            != identity.artifact_identity_public_key_sha256
        || sha256_bytes(artifact_identity_signature) != identity.artifact_identity_signature_sha256
        || sha256_bytes(load_descriptor) != identity.load_descriptor_sha256
        || sha256_bytes(load_descriptor_public_key) != identity.load_descriptor_public_key_sha256
        || sha256_bytes(load_descriptor_signature) != identity.load_descriptor_signature_sha256
    {
        return false;
    }
    if !verify_p256_signature(
        artifact_identity_public_key,
        artifact_identity_signature,
        artifact_identity_descriptor,
    ) || !verify_p256_signature(
        load_descriptor_public_key,
        load_descriptor_signature,
        load_descriptor,
    ) {
        return false;
    }

    let Ok(artifact_identity_text) = core::str::from_utf8(artifact_identity_descriptor) else {
        return false;
    };
    let Ok(load_descriptor_text) = core::str::from_utf8(load_descriptor) else {
        return false;
    };
    if !descriptor_sha256_field_eq(
        artifact_identity_text,
        "artifact_reference_bytes_sha256",
        content_sha256,
    ) || !descriptor_sha256_field_eq(
        load_descriptor_text,
        "artifact_reference_bytes_sha256",
        content_sha256,
    ) || !descriptor_sha256_field_eq(
        load_descriptor_text,
        "artifact_identity_sha256",
        identity.artifact_identity_descriptor_sha256,
    ) {
        return false;
    }
    for (text, field, expected) in [
        (artifact_identity_text, "classification", "local_only"),
        (
            artifact_identity_text,
            "accepts_external_artifact_bytes",
            "false",
        ),
        (
            artifact_identity_text,
            "authorizes_external_artifact_load",
            "false",
        ),
        (
            artifact_identity_text,
            "authorizes_persistent_install",
            "false",
        ),
        (
            artifact_identity_text,
            "authorizes_rollback_install",
            "false",
        ),
        (artifact_identity_text, "writes_persistent_state", "false"),
        (load_descriptor_text, "classification", "local_only"),
        (
            load_descriptor_text,
            "accepts_external_artifact_bytes",
            "false",
        ),
        (load_descriptor_text, "loads_external_artifact", "false"),
        (load_descriptor_text, "maps_executable_pages", "false"),
        (load_descriptor_text, "writes_persistent_state", "false"),
        (
            load_descriptor_text,
            "authorizes_persistent_install",
            "false",
        ),
        (load_descriptor_text, "authorizes_rollback_install", "false"),
        (
            load_descriptor_text,
            "authorizes_current_boot_wasm_execution",
            "true",
        ),
    ] {
        if !descriptor_field_eq(text, field, expected) {
            return false;
        }
    }

    let Some(service_id) = descriptor_field(artifact_identity_text, "service_id") else {
        return false;
    };
    let Some(artifact_id) = descriptor_field(artifact_identity_text, "artifact_id") else {
        return false;
    };
    let Some(artifact_identity_id) = descriptor_field(artifact_identity_text, "id") else {
        return false;
    };
    descriptor_field_eq(load_descriptor_text, "service_id", service_id)
        && descriptor_field_eq(load_descriptor_text, "artifact_id", artifact_id)
        && descriptor_field_eq(
            load_descriptor_text,
            "artifact_identity_id",
            artifact_identity_id,
        )
}

fn verify_p256_signature(public_key_sec1: &[u8], signature_der: &[u8], payload: &[u8]) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_sec1_bytes(public_key_sec1) else {
        return false;
    };
    let Ok(signature) = Signature::from_der(signature_der) else {
        return false;
    };
    verifying_key.verify(payload, &signature).is_ok()
}

fn descriptor_sha256_field_eq(text: &str, key: &str, expected: [u8; 32]) -> bool {
    descriptor_field(text, key)
        .and_then(parse_sha256_ref)
        .map(|actual| actual == expected)
        .unwrap_or(false)
}

fn descriptor_field_eq(text: &str, key: &str, expected: &str) -> bool {
    descriptor_field(text, key)
        .map(|actual| actual == expected)
        .unwrap_or(false)
}

fn descriptor_field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let mut found = None;
    for line in text.lines() {
        let Some((field, value)) = line.split_once('=') else {
            continue;
        };
        if field == key {
            if found.is_some() {
                return None;
            }
            found = Some(value);
        }
    }
    found
}

fn max_base64_encoded_len(decoded_max: usize) -> usize {
    decoded_max.div_ceil(3) * 4
}

fn next_sha256<'a, I>(words: &mut I) -> Result<[u8; 32], &'static str>
where
    I: Iterator<Item = &'a str>,
{
    next_sha256_or(words, DISTRIBUTION_RECEIVER_IDENTITY_INVALID_REASON)
}

fn next_sha256_or<'a, I>(words: &mut I, reason: &'static str) -> Result<[u8; 32], &'static str>
where
    I: Iterator<Item = &'a str>,
{
    let Some(token) = words.next() else {
        return Err(reason);
    };
    parse_sha256_ref(token).ok_or(reason)
}

fn expect_next_token<'a, I>(words: &mut I, expected: &'static str) -> Result<(), &'static str>
where
    I: Iterator<Item = &'a str>,
{
    match words.next() {
        Some(token) if token == expected => Ok(()),
        _ => Err(DISTRIBUTION_RECEIVER_IDENTITY_INVALID_REASON),
    }
}

fn parse_positive_usize(token: &str) -> Option<usize> {
    let value = parse_usize(token)?;
    (value > 0).then_some(value)
}

fn parse_usize(token: &str) -> Option<usize> {
    token.parse::<usize>().ok()
}

fn clear_pending_distribution() {
    let mut pending = PENDING_SERIAL_DISTRIBUTION.lock();
    pending.clear();
}

fn rejected_distribution_catalog_entry(reason: &'static str) -> DistributionCatalogEntryOutcome {
    DistributionCatalogEntryOutcome {
        content_sha256: None,
        total_length: 0,
        chunk_count: 0,
        signature_byte_len: 0,
        receiver_identity_retained: false,
        retained_in_catalog: false,
        accepted: false,
        rejected: true,
        reason,
    }
}

fn rejected_receiver_identity(reason: &'static str) -> DistributionReceiverIdentityOutcome {
    DistributionReceiverIdentityOutcome {
        content_sha256: None,
        receiver_identity: None,
        retained_in_catalog: false,
        accepted: false,
        rejected: true,
        reason,
    }
}

fn rejected_receiver_identity_evidence(
    reason: &'static str,
) -> DistributionReceiverIdentityEvidenceOutcome {
    DistributionReceiverIdentityEvidenceOutcome {
        content_sha256: None,
        evidence_kind: "unknown",
        evidence_sha256: None,
        decoded_byte_len: 0,
        retained_part_count: 0,
        receiver_identity: None,
        accepted: false,
        rejected: true,
        reason,
    }
}

fn rejected_receiver_identity_load_preflight(
    reason: &'static str,
) -> DistributionReceiverIdentityLoadPreflightOutcome {
    DistributionReceiverIdentityLoadPreflightOutcome {
        content_sha256: None,
        retained_part_count: 0,
        receiver_identity: None,
        retained_candidate_sha256: None,
        retained_candidate_wasm_valid: false,
        catalog_finalize_candidate_sha256: None,
        retained_candidate_matches_catalog_finalize: false,
        status: "denied",
        reason,
        preflight_evaluated: false,
        accepted: false,
        rejected: true,
        missing_gate_count: 0,
    }
}

fn rejected_distribution_begin_for(
    source_id: &'static str,
    entry_id: &'static str,
    reason: &'static str,
) -> DistributionBeginOutcome {
    DistributionBeginOutcome {
        source_id,
        entry_id,
        content_sha256: None,
        total_length: 0,
        chunk_count: 0,
        signature_byte_len: 0,
        receiver_identity: None,
        accepted: false,
        rejected: true,
        reason,
    }
}

fn reject_distribution_chunk_with_clear(reason: &'static str) -> DistributionChunkOutcome {
    let mut pending = PENDING_SERIAL_DISTRIBUTION.lock();
    pending.clear();
    rejected_distribution_chunk(None, 0, None, 0, 0, true, reason)
}

fn rejected_distribution_chunk(
    content_sha256: Option<[u8; 32]>,
    chunk_index: usize,
    chunk_sha256: Option<[u8; 32]>,
    decoded_byte_len: usize,
    pending_chunk_count: usize,
    discarded_pending_delivery: bool,
    reason: &'static str,
) -> DistributionChunkOutcome {
    DistributionChunkOutcome {
        content_sha256,
        chunk_index,
        chunk_sha256,
        decoded_byte_len,
        pending_chunk_count,
        accepted: false,
        rejected: true,
        discarded_pending_delivery,
        reason,
    }
}

fn accepted_distribution_chunk(
    content_sha256: [u8; 32],
    chunk_index: usize,
    chunk_sha256: [u8; 32],
    decoded_byte_len: usize,
    pending_chunk_count: usize,
    reason: &'static str,
) -> DistributionChunkOutcome {
    DistributionChunkOutcome {
        content_sha256: Some(content_sha256),
        chunk_index,
        chunk_sha256: Some(chunk_sha256),
        decoded_byte_len,
        pending_chunk_count,
        accepted: true,
        rejected: false,
        discarded_pending_delivery: false,
        reason,
    }
}

fn rejected_distribution_finalize(
    source_id: &'static str,
    entry_id: &'static str,
    content_sha256: Option<[u8; 32]>,
    total_length: usize,
    declared_chunk_count: usize,
    accepted_chunk_count: usize,
    reason: &'static str,
) -> DistributionFinalizeOutcome {
    DistributionFinalizeOutcome {
        source_id,
        entry_id,
        content_sha256,
        total_length,
        declared_chunk_count,
        accepted_chunk_count,
        delivered_byte_len: 0,
        selection: None,
        staged_candidate: None,
        retained_provenance: None,
        receiver_identity: None,
        status: "denied",
        reason,
    }
}

fn distribution_transport_selection(
    entry_id: &'static str,
    selection: &RegistrySelectionDecision<'_>,
) -> DistributionTransportSelection {
    DistributionTransportSelection {
        entry_id,
        status: selection.status,
        reason: selection.reason.as_str(),
        artifact_sha256: selection.artifact_sha256,
        artifact_byte_len: selection.artifact_byte_len,
        provenance_signature_verified: selection.provenance_signature_verified,
        selected_for_candidate_intake: selection.selected_for_candidate_intake,
        load_attempted: selection.load_attempted,
        execution_attempted: selection.execution_attempted,
        durable_write_attempted: selection.durable_write_attempted,
        authorizes_acquisition: selection.authorizes_acquisition,
        authorizes_install: selection.authorizes_install,
        authorizes_load: selection.authorizes_load,
        authorizes_execute: selection.authorizes_execute,
        authorizes_persist: selection.authorizes_persist,
        writes_persistent_state: selection.writes_persistent_state,
        owner_sealed: selection.owner_sealed,
    }
}

fn record_distribution_transport_selection(
    selection: &DistributionTransportSelection,
) -> V<'static> {
    V::InlineObject(vec![
        f("entry_id", s(selection.entry_id)),
        f("status", s(selection.status)),
        f("reason", s(selection.reason)),
        f("artifact_sha256", record_sha(selection.artifact_sha256)),
        f(
            "artifact_byte_len",
            V::U64(selection.artifact_byte_len as u64),
        ),
        f(
            "provenance_signature_verified",
            b(selection.provenance_signature_verified),
        ),
        f(
            "selected_for_candidate_intake",
            b(selection.selected_for_candidate_intake),
        ),
        f("load_attempted", b(selection.load_attempted)),
        f("execution_attempted", b(selection.execution_attempted)),
        f(
            "durable_write_attempted",
            b(selection.durable_write_attempted),
        ),
        f(
            "authorizes_acquisition",
            b(selection.authorizes_acquisition),
        ),
        f("authorizes_install", b(selection.authorizes_install)),
        f("authorizes_load", b(selection.authorizes_load)),
        f("authorizes_execute", b(selection.authorizes_execute)),
        f("authorizes_persist", b(selection.authorizes_persist)),
        f(
            "writes_persistent_state",
            b(selection.writes_persistent_state),
        ),
        f("owner_sealed", b(selection.owner_sealed)),
    ])
}

fn record_receiver_identity(identity: &LocalReceiverIdentity) -> V<'static> {
    V::InlineObject(vec![
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f(
            "artifact_identity_descriptor_sha256",
            record_sha(identity.artifact_identity_descriptor_sha256),
        ),
        f(
            "artifact_identity_public_key_sha256",
            record_sha(identity.artifact_identity_public_key_sha256),
        ),
        f(
            "artifact_identity_signature_sha256",
            record_sha(identity.artifact_identity_signature_sha256),
        ),
        f(
            "load_descriptor_sha256",
            record_sha(identity.load_descriptor_sha256),
        ),
        f(
            "load_descriptor_public_key_sha256",
            record_sha(identity.load_descriptor_public_key_sha256),
        ),
        f(
            "load_descriptor_signature_sha256",
            record_sha(identity.load_descriptor_signature_sha256),
        ),
        f(
            "artifact_identity_signature_verified_by_host_export",
            b(identity.artifact_identity_signature_verified_by_host_export),
        ),
        f(
            "load_descriptor_signature_verified_by_host_export",
            b(identity.load_descriptor_signature_verified_by_host_export),
        ),
        f(
            "artifact_identity_signature_verified_by_guest",
            b(identity.artifact_identity_signature_verified_by_guest),
        ),
        f(
            "load_descriptor_signature_verified_by_guest",
            b(identity.load_descriptor_signature_verified_by_guest),
        ),
        f(
            "guest_signature_verification_performed",
            b(identity.guest_signature_verification_performed),
        ),
        f(
            "receiver_identity_complete",
            b(identity.receiver_identity_complete),
        ),
        f(
            "artifact_hash_bound_by_identity",
            b(identity.artifact_hash_bound_by_identity),
        ),
        f(
            "artifact_hash_bound_by_load_descriptor",
            b(identity.artifact_hash_bound_by_load_descriptor),
        ),
        f(
            "load_descriptor_binds_artifact_identity",
            b(identity.load_descriptor_binds_artifact_identity),
        ),
        f(
            "load_descriptor_authorizes_current_boot_wasm_execution",
            b(identity.load_descriptor_authorizes_current_boot_wasm_execution),
        ),
        f("metadata_is_non_authorizing", b(true)),
        f("export_authorizes_load", b(false)),
        f("export_authorizes_install", b(false)),
        f("export_authorizes_execute", b(false)),
        f("export_writes_persistent_state", b(false)),
        f("requires_m6_m7_reverify_for_load", b(true)),
    ])
}

pub(crate) fn emit_registry_selection_diagnostic(arg: &str) {
    let run = run_registry_selection(registry_selection_hash_arg(arg), true);

    begin_response("module.registry_selection_diagnostic");
    emit_record_fields(record_run("module.registry_selection_diagnostic", &run), 6);
    end_response("module.registry_selection_diagnostic");
}

pub(crate) fn emit_registry_selection_diagnostic_selftest() {
    let cases = registry_selection_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);

    begin_response("module.registry_selection_diagnostic_selftest");
    emit_record_fields(
        vec![
            f("method", s("module.registry_selection_diagnostic_selftest")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("passed", b(passed)),
            f("case_count", V::U64(cases.len() as u64)),
            f(
                "cases",
                V::Array(cases.iter().map(record_selftest_case).collect()),
            ),
            f("read_only", b(true)),
            f("durable_write", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
            f("install_authorized", b(false)),
            f("load_authorized", b(false)),
            f("execute_authorized", b(false)),
            f("persist_authorized", b(false)),
        ],
        6,
    );
    end_response("module.registry_selection_diagnostic_selftest");
}

fn run_registry_selection(selector: &str, stage_valid: bool) -> RegistrySelectionRun<'static> {
    let Some(requested_artifact_sha256) = parse_sha256_ref(selector) else {
        return RegistrySelectionRun {
            parse_ok: false,
            requested_artifact_sha256: None,
            registry_entry_count: 0,
            registry_capacity: 0,
            selection: None,
            entry_reason: None,
            staged_candidate: None,
            retained_provenance: None,
        };
    };

    let registry = match distribution_registry::builtin_registry() {
        Ok(registry) => registry,
        Err(reason) => {
            return RegistrySelectionRun {
                parse_ok: true,
                requested_artifact_sha256: Some(requested_artifact_sha256),
                registry_entry_count: 0,
                registry_capacity: 0,
                selection: None,
                entry_reason: Some(reason),
                staged_candidate: None,
                retained_provenance: None,
            }
        }
    };
    let registry_entry_count = registry.len();
    let registry_capacity = registry.capacity();

    let selection = match registry.select_by_hash(requested_artifact_sha256) {
        Ok(selection) => selection,
        Err(reason) => {
            return RegistrySelectionRun {
                parse_ok: true,
                requested_artifact_sha256: Some(requested_artifact_sha256),
                registry_entry_count,
                registry_capacity,
                selection: None,
                entry_reason: Some(reason),
                staged_candidate: None,
                retained_provenance: None,
            }
        }
    };
    if !stage_valid || !selection.selected_for_candidate_intake {
        return RegistrySelectionRun {
            parse_ok: true,
            requested_artifact_sha256: Some(requested_artifact_sha256),
            registry_entry_count,
            registry_capacity,
            selection: Some(selection),
            entry_reason: None,
            staged_candidate: None,
            retained_provenance: None,
        };
    }

    let Some(entry) = registry_entry_for_selection(&registry, &selection) else {
        return RegistrySelectionRun {
            parse_ok: true,
            requested_artifact_sha256: Some(requested_artifact_sha256),
            registry_entry_count,
            registry_capacity,
            selection: Some(selection),
            entry_reason: Some(RegistrySelectionReason::RegistryEntryNotFound),
            staged_candidate: None,
            retained_provenance: None,
        };
    };
    let staged_candidate = module_candidate_intake::intake_and_retain_external_wasm_candidate(
        Vec::from(entry.artifact_bytes),
    );
    let retained_provenance = distribution_candidate::verify_retained_candidate_provenance(
        entry.provenance_signature_der,
    );

    RegistrySelectionRun {
        parse_ok: true,
        requested_artifact_sha256: Some(requested_artifact_sha256),
        registry_entry_count,
        registry_capacity,
        selection: Some(selection),
        entry_reason: None,
        staged_candidate: Some(staged_candidate),
        retained_provenance: Some(retained_provenance),
    }
}

fn registry_entry_for_selection<'a>(
    registry: &'a raios_core::distribution_registry::DistributionRegistry<'a>,
    selection: &RegistrySelectionDecision<'_>,
) -> Option<DistributionRegistryEntry<'a>> {
    let mut idx = 0usize;
    while idx < registry.len() {
        if let Some(entry) = registry.get(idx) {
            if entry.artifact_sha256 == selection.artifact_sha256 {
                return Some(*entry);
            }
        }
        idx += 1;
    }
    None
}

fn registry_selection_selftest_cases() -> [SelftestCase; 5] {
    let valid_echo = run_registry_selection(
        "sha256:f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2",
        true,
    );
    let valid_bufecho = run_registry_selection(
        "sha256:1983797d9ecc6f3f85deedc0c82a8651062f01dc80710ee699e834a51c52e544",
        true,
    );
    let chunked_bufecho = chunked_bufecho_selftest_case();
    let wrong_hash = run_registry_selection(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        true,
    );
    let invalid = run_registry_selection("not-a-sha256", true);

    [
        selftest_case(
            "valid_echo_registry_selection_stages_inert_candidate",
            &valid_echo,
            "selected",
            "registry_entry_selected_for_inert_candidate_intake",
            true,
        ),
        selftest_case(
            "valid_bufecho_registry_selection_stages_inert_candidate",
            &valid_bufecho,
            "selected",
            "registry_entry_selected_for_inert_candidate_intake",
            true,
        ),
        chunked_bufecho,
        selftest_case(
            "wrong_hash_denied_without_staging",
            &wrong_hash,
            "denied",
            "registry_entry_not_found",
            false,
        ),
        selftest_case(
            "invalid_selector_denied_without_staging",
            &invalid,
            "denied",
            "invalid_sha256_selector",
            false,
        ),
    ]
}

fn chunked_bufecho_selftest_case() -> SelftestCase {
    match run_chunked_bufecho_selection() {
        Ok((selection, staged_candidate, retained_provenance)) => {
            let staged = staged_candidate.retained_in_ram
                && staged_candidate.wasm_valid
                && !staged_candidate.rejected;
            let retained_provenance_verified = retained_provenance.provenance_verified;
            SelftestCase {
                name: "chunked_bufecho_delivery_stages_inert_candidate",
                passed: selection.status == "selected"
                    && selection.reason.as_str()
                        == "registry_entry_selected_for_inert_candidate_intake"
                    && selection.selected_for_candidate_intake
                    && staged
                    && retained_provenance_verified
                    && !selection.authorizes_load
                    && !selection.authorizes_execute
                    && !selection.authorizes_persist,
                status: selection.status,
                reason: selection.reason.as_str(),
                selected_for_candidate_intake: selection.selected_for_candidate_intake,
                staged,
                retained_provenance_verified,
                authorizes_load: selection.authorizes_load,
                authorizes_execute: selection.authorizes_execute,
                authorizes_persist: selection.authorizes_persist,
            }
        }
        Err(reason) => SelftestCase {
            name: "chunked_bufecho_delivery_stages_inert_candidate",
            passed: false,
            status: "denied",
            reason,
            selected_for_candidate_intake: false,
            staged: false,
            retained_provenance_verified: false,
            authorizes_load: false,
            authorizes_execute: false,
            authorizes_persist: false,
        },
    }
}

fn run_chunked_bufecho_selection() -> Result<
    (
        RegistrySelectionDecision<'static>,
        module_candidate_intake::ExternalWasmCandidateOutcome,
        distribution_candidate::DistributionCandidateOutcome,
    ),
    &'static str,
> {
    let bytes = wasm_runtime::BUFECHO_WASM_ARTIFACT_BYTES;
    let first_end = bytes.len() / 3;
    let second_end = (bytes.len() * 2) / 3;
    let mut delivery = ChunkedDistributionDelivery::new(ChunkedDistributionTarget {
        entry_id: distribution_registry::BUILTIN_BUFECHO_REGISTRY_ENTRY_ID,
        content_sha256: wasm_runtime::BUFECHO_WASM_ARTIFACT_BYTES_HASH,
        total_length: bytes.len(),
        chunk_count: 3,
        provenance_signature_der: Some(
            distribution_registry::BUILTIN_BUFECHO_PROVENANCE_SIGNATURE_DER,
        ),
        publisher_key_sha256: PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
        classification: "local_only",
    });
    for (index, chunk) in [
        (2usize, &bytes[second_end..]),
        (0usize, &bytes[..first_end]),
        (1usize, &bytes[first_end..second_end]),
    ] {
        delivery
            .accept_chunk(ChunkedDistributionChunkInput {
                index,
                bytes: chunk,
                claimed_chunk_sha256: sha256_bytes(chunk),
            })
            .map_err(ChunkedDeliveryError::as_str)?;
    }

    let mut reassembled = vec![0u8; bytes.len()];
    let entry = delivery
        .try_finalize(&mut reassembled)
        .map_err(ChunkedDeliveryError::as_str)?;
    let selection = evaluate_distribution_registry_selection(&entry, entry.artifact_sha256);
    if !selection.selected_for_candidate_intake {
        return Err(selection.reason.as_str());
    }
    let staged_candidate = module_candidate_intake::intake_and_retain_external_wasm_candidate(
        Vec::from(entry.artifact_bytes),
    );
    let retained_provenance = distribution_candidate::verify_retained_candidate_provenance(
        entry.provenance_signature_der,
    );

    Ok((
        RegistrySelectionDecision {
            entry_id: distribution_registry::BUILTIN_BUFECHO_REGISTRY_ENTRY_ID,
            ..selection
        },
        staged_candidate,
        retained_provenance,
    ))
}

fn selftest_case(
    name: &'static str,
    run: &RegistrySelectionRun<'_>,
    expected_status: &'static str,
    expected_reason: &'static str,
    expect_staged: bool,
) -> SelftestCase {
    let status = run_status(run);
    let reason = run_reason(run);
    let selected = run
        .selection
        .map(|selection| selection.selected_for_candidate_intake)
        .unwrap_or(false);
    let staged = run
        .staged_candidate
        .map(|candidate| candidate.retained_in_ram && candidate.wasm_valid && !candidate.rejected)
        .unwrap_or(false);
    let retained_provenance_verified = run
        .retained_provenance
        .map(|provenance| provenance.provenance_verified)
        .unwrap_or(false);
    let authorizes_load = run
        .selection
        .map(|selection| selection.authorizes_load)
        .unwrap_or(false);
    let authorizes_execute = run
        .selection
        .map(|selection| selection.authorizes_execute)
        .unwrap_or(false);
    let authorizes_persist = run
        .selection
        .map(|selection| selection.authorizes_persist)
        .unwrap_or(false);

    SelftestCase {
        name,
        passed: status == expected_status
            && reason == expected_reason
            && staged == expect_staged
            && (!expect_staged || retained_provenance_verified)
            && !authorizes_load
            && !authorizes_execute
            && !authorizes_persist,
        status,
        reason,
        selected_for_candidate_intake: selected,
        staged,
        retained_provenance_verified,
        authorizes_load,
        authorizes_execute,
        authorizes_persist,
    }
}

fn record_run<'a>(
    method: &'static str,
    run: &'a RegistrySelectionRun<'a>,
) -> Vec<raios_core::record::Field<'a>> {
    let selection = run.selection;
    vec![
        f("method", s(method)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("parse_ok", b(run.parse_ok)),
        f(
            "requested_artifact_sha256",
            record_sha_or_null(run.requested_artifact_sha256),
        ),
        f(
            "registry_entry_count",
            V::U64(run.registry_entry_count as u64),
        ),
        f("registry_capacity", V::U64(run.registry_capacity as u64)),
        f("status", s(run_status(run))),
        f("reason", s(run_reason(run))),
        f(
            "selection",
            selection
                .map(|selection| selection.as_record_value())
                .unwrap_or(V::Null),
        ),
        f(
            "entry_id",
            s(selection.map(|s| s.entry_id).unwrap_or("none")),
        ),
        f(
            "staged_candidate",
            run.staged_candidate
                .as_ref()
                .map(record_candidate_outcome)
                .unwrap_or(V::Null),
        ),
        f(
            "retained_provenance",
            run.retained_provenance
                .as_ref()
                .map(record_retained_provenance)
                .unwrap_or(V::Null),
        ),
        f(
            "staged_only_after_valid_selection",
            b(run.staged_candidate.is_some()
                == selection
                    .map(|selection| selection.selected_for_candidate_intake)
                    .unwrap_or(false)),
        ),
        f(
            "recomputed_sha256_matches_selection",
            b(recomputed_sha256_matches_selection(run)),
        ),
        f("provenance_is_origin_evidence_only", b(true)),
        f("requires_m6_reverify_for_load", b(true)),
        f("load_attempted", b(false)),
        f("execution_attempted", b(false)),
        f("durable_write_attempted", b(false)),
        f("authorizes_acquisition", b(false)),
        f("authorizes_install", b(false)),
        f("authorizes_load", b(false)),
        f("authorizes_execute", b(false)),
        f("authorizes_persist", b(false)),
        f("writes_persistent_state", b(false)),
        f("owner_sealed", b(false)),
        f("trust_tier", s("dev_key_not_owner_sealed")),
        f("network_attempted", b(false)),
        f("host_import_added", b(false)),
        f("durable_write", b(false)),
    ]
}

fn record_candidate_outcome(
    candidate: &module_candidate_intake::ExternalWasmCandidateOutcome,
) -> V<'static> {
    V::InlineObject(vec![
        f("byte_len", V::U64(candidate.byte_len as u64)),
        f("artifact_sha256", record_sha(candidate.artifact_sha256)),
        f("wasm_valid", b(candidate.wasm_valid)),
        f("scope", s(candidate.scope)),
        f("retained_in_ram", b(candidate.retained_in_ram)),
        f("rejected", b(candidate.rejected)),
        f("reason", s(candidate.reason)),
        f("load_attempted", b(candidate.load_attempted)),
        f("execution_attempted", b(candidate.execution_attempted)),
        f("authorizes_load", b(candidate.authorizes_load)),
        f("authorizes_execution", b(candidate.authorizes_execution)),
        f(
            "writes_persistent_state",
            b(candidate.writes_persistent_state),
        ),
        f(
            "external_delivery_channel",
            s(candidate.external_delivery_channel),
        ),
    ])
}

fn record_retained_provenance(
    provenance: &distribution_candidate::DistributionCandidateOutcome,
) -> V<'static> {
    V::InlineObject(vec![
        f("source_kind", s(provenance.source_kind)),
        f("retained_present", b(provenance.retained_present)),
        f("retained_wasm_valid", b(provenance.retained_wasm_valid)),
        f("artifact_sha256", record_sha(provenance.artifact_sha256)),
        f(
            "provenance_signature_present",
            b(provenance.provenance_signature_present),
        ),
        f("provenance_verified", b(provenance.provenance_verified)),
        f(
            "publisher_key_sha256",
            record_sha(provenance.publisher_key_sha256),
        ),
        f("status", s(provenance.status)),
        f("reason", s(provenance.reason)),
        f("honest", b(provenance.honest)),
        f("load_authorized", b(provenance.load_authorized)),
        f("install_authorized", b(provenance.install_authorized)),
        f("owner_sealed", b(provenance.owner_sealed)),
        f(
            "requires_m6_reverify_for_load",
            b(provenance.requires_m6_reverify_for_load),
        ),
        f("trust_tier", s(provenance.trust_tier)),
        f("load_attempted", b(provenance.load_attempted)),
        f("execution_attempted", b(provenance.execution_attempted)),
        f("authorizes_load", b(provenance.authorizes_load)),
        f("authorizes_execution", b(provenance.authorizes_execution)),
        f(
            "writes_persistent_state",
            b(provenance.writes_persistent_state),
        ),
    ])
}

fn record_selftest_case(case: &SelftestCase) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("status", s(case.status)),
        f("reason", s(case.reason)),
        f(
            "selected_for_candidate_intake",
            b(case.selected_for_candidate_intake),
        ),
        f("staged", b(case.staged)),
        f(
            "retained_provenance_verified",
            b(case.retained_provenance_verified),
        ),
        f("authorizes_load", b(case.authorizes_load)),
        f("authorizes_execute", b(case.authorizes_execute)),
        f("authorizes_persist", b(case.authorizes_persist)),
        f("passed", b(case.passed)),
    ])
}

fn registry_selection_hash_arg(arg: &str) -> &str {
    let trimmed = arg.trim();
    let payload = if method_head_eq(trimmed, "module.registry_selection_diagnostic") {
        trimmed
            .strip_prefix("module.registry_selection_diagnostic")
            .unwrap_or(trimmed)
    } else {
        trimmed
    };
    payload.split_whitespace().next().unwrap_or("")
}

fn run_status(run: &RegistrySelectionRun<'_>) -> &'static str {
    if !run.parse_ok {
        return "denied";
    }
    if run.entry_reason.is_some() {
        return "denied";
    }
    run.selection
        .map(|selection| selection.status)
        .unwrap_or("denied")
}

fn run_reason(run: &RegistrySelectionRun<'_>) -> &'static str {
    if !run.parse_ok {
        return "invalid_sha256_selector";
    }
    if let Some(reason) = run.entry_reason {
        return reason.as_str();
    }
    run.selection
        .map(|selection| selection.reason.as_str())
        .unwrap_or("registry_entry_unavailable")
}

fn recomputed_sha256_matches_selection(run: &RegistrySelectionRun<'_>) -> bool {
    let Some(candidate) = run.staged_candidate else {
        return false;
    };
    let Some(selection) = run.selection else {
        return false;
    };
    candidate.artifact_sha256 == selection.artifact_sha256
}
