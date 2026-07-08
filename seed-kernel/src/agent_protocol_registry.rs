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
use spin::Mutex;

const SERIAL_DISTRIBUTION_DELIVERY_CHANNEL: &str = "serial_console_distribution_chunks_v0";
const SERIAL_DISTRIBUTION_ENTRY_ID: &str = "serial.local.distribution";
const DISTRIBUTION_BEGIN_ACCEPTED_REASON: &str = "accepted_distribution_delivery_target";
const DISTRIBUTION_CHUNK_ACCEPTED_REASON: &str = "accepted_distribution_chunk";
const DISTRIBUTION_DUPLICATE_CHUNK_ACCEPTED_REASON: &str = "accepted_duplicate_distribution_chunk";
const DISTRIBUTION_DELIVERY_NOT_STARTED_REASON: &str = "distribution_delivery_not_started";
const DISTRIBUTION_INVALID_ARGS_REASON: &str = "invalid_distribution_delivery_arguments";
const DISTRIBUTION_INVALID_LENGTH_REASON: &str = "invalid_distribution_total_length";
const DISTRIBUTION_INVALID_CHUNK_COUNT_REASON: &str = "invalid_distribution_chunk_count";
const DISTRIBUTION_INVALID_SIGNATURE_REASON: &str = "invalid_distribution_signature_hex";
const DISTRIBUTION_INVALID_CHUNK_INDEX_REASON: &str = "invalid_distribution_chunk_index";

static PENDING_SERIAL_DISTRIBUTION: Mutex<PendingSerialDistribution> =
    Mutex::new(PendingSerialDistribution::new());

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
    content_sha256: [u8; 32],
    total_length: usize,
    chunk_count: usize,
    provenance_signature_der: Vec<u8>,
    chunks: Vec<PendingSerialDistributionChunk>,
}

struct PendingSerialDistributionChunk {
    index: usize,
    bytes: Vec<u8>,
    claimed_chunk_sha256: [u8; 32],
}

#[derive(Clone, Copy)]
struct DistributionBeginOutcome {
    content_sha256: Option<[u8; 32]>,
    total_length: usize,
    chunk_count: usize,
    signature_byte_len: usize,
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
    content_sha256: Option<[u8; 32]>,
    total_length: usize,
    declared_chunk_count: usize,
    accepted_chunk_count: usize,
    delivered_byte_len: usize,
    selection: Option<DistributionTransportSelection>,
    staged_candidate: Option<module_candidate_intake::ExternalWasmCandidateOutcome>,
    retained_provenance: Option<distribution_candidate::DistributionCandidateOutcome>,
    status: &'static str,
    reason: &'static str,
}

impl PendingSerialDistribution {
    const fn new() -> Self {
        Self {
            active: false,
            content_sha256: [0; 32],
            total_length: 0,
            chunk_count: 0,
            provenance_signature_der: Vec::new(),
            chunks: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.active = false;
        self.content_sha256 = [0; 32];
        self.total_length = 0;
        self.chunk_count = 0;
        self.provenance_signature_der.clear();
        self.chunks.clear();
    }
}

pub(crate) fn emit_submit_distribution_begin(arg: &str) {
    let outcome = submit_distribution_begin(arg);

    begin_response("module.submit_distribution_begin");
    emit_record_fields(
        vec![
            f("method", s("module.submit_distribution_begin")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("delivery_channel", s(SERIAL_DISTRIBUTION_DELIVERY_CHANNEL)),
            f("entry_id", s(SERIAL_DISTRIBUTION_ENTRY_ID)),
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
    end_response("module.submit_distribution_begin");
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
            f("entry_id", s(SERIAL_DISTRIBUTION_ENTRY_ID)),
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
    let mut pending = PENDING_SERIAL_DISTRIBUTION.lock();
    pending.clear();

    let mut words =
        distribution_payload(arg, "module.submit_distribution_begin").split_whitespace();
    let Some(content_sha_token) = words.next() else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(total_length_token) = words.next() else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(chunk_count_token) = words.next() else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(signature_token) = words.next() else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    if words.next().is_some() {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_ARGS_REASON);
    }

    let Some(content_sha256) = parse_sha256_ref(content_sha_token) else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_ARGS_REASON);
    };
    let Some(total_length) = parse_positive_usize(total_length_token) else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_LENGTH_REASON);
    };
    if total_length > module_candidate_intake::MAX_EXTERNAL_WASM_CANDIDATE_BYTES {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_LENGTH_REASON);
    }
    let Some(chunk_count) = parse_positive_usize(chunk_count_token) else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_CHUNK_COUNT_REASON);
    };
    if chunk_count > DISTRIBUTION_CHUNKED_DELIVERY_MAX_CHUNKS {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_CHUNK_COUNT_REASON);
    }
    let Some(signature_der) = decode_signature_token(signature_token) else {
        return rejected_distribution_begin(DISTRIBUTION_INVALID_SIGNATURE_REASON);
    };

    pending.active = true;
    pending.content_sha256 = content_sha256;
    pending.total_length = total_length;
    pending.chunk_count = chunk_count;
    pending.provenance_signature_der = signature_der;

    DistributionBeginOutcome {
        content_sha256: Some(content_sha256),
        total_length,
        chunk_count,
        signature_byte_len: pending.provenance_signature_der.len(),
        accepted: true,
        rejected: false,
        reason: DISTRIBUTION_BEGIN_ACCEPTED_REASON,
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
            content_sha256: None,
            total_length: 0,
            declared_chunk_count: 0,
            accepted_chunk_count: 0,
            delivered_byte_len: 0,
            selection: None,
            staged_candidate: None,
            retained_provenance: None,
            status: "denied",
            reason: DISTRIBUTION_DELIVERY_NOT_STARTED_REASON,
        };
    }

    let content_sha256 = pending.content_sha256;
    let total_length = pending.total_length;
    let declared_chunk_count = pending.chunk_count;
    let accepted_chunk_count = pending.chunks.len();
    if accepted_chunk_count != declared_chunk_count {
        pending.clear();
        return rejected_distribution_finalize(
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
        entry_id: SERIAL_DISTRIBUTION_ENTRY_ID,
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
                Some(pending.content_sha256),
                pending.total_length,
                pending.chunk_count,
                pending.chunks.len(),
                reason.as_str(),
            )
        }
    };
    let selection = evaluate_distribution_registry_selection(&entry, entry.artifact_sha256);
    let selection_summary = distribution_transport_selection(&selection);
    if !selection.selected_for_candidate_intake {
        return DistributionFinalizeOutcome {
            content_sha256: Some(pending.content_sha256),
            total_length: pending.total_length,
            declared_chunk_count: pending.chunk_count,
            accepted_chunk_count: pending.chunks.len(),
            delivered_byte_len: 0,
            selection: Some(selection_summary),
            staged_candidate: None,
            retained_provenance: None,
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

    DistributionFinalizeOutcome {
        content_sha256: Some(pending.content_sha256),
        total_length: pending.total_length,
        declared_chunk_count: pending.chunk_count,
        accepted_chunk_count: pending.chunks.len(),
        delivered_byte_len: staged_candidate.byte_len,
        selection: Some(selection_summary),
        staged_candidate: Some(staged_candidate),
        retained_provenance: Some(retained_provenance),
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

fn parse_positive_usize(token: &str) -> Option<usize> {
    let value = parse_usize(token)?;
    (value > 0).then_some(value)
}

fn parse_usize(token: &str) -> Option<usize> {
    token.parse::<usize>().ok()
}

fn rejected_distribution_begin(reason: &'static str) -> DistributionBeginOutcome {
    DistributionBeginOutcome {
        content_sha256: None,
        total_length: 0,
        chunk_count: 0,
        signature_byte_len: 0,
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
    content_sha256: Option<[u8; 32]>,
    total_length: usize,
    declared_chunk_count: usize,
    accepted_chunk_count: usize,
    reason: &'static str,
) -> DistributionFinalizeOutcome {
    DistributionFinalizeOutcome {
        content_sha256,
        total_length,
        declared_chunk_count,
        accepted_chunk_count,
        delivered_byte_len: 0,
        selection: None,
        staged_candidate: None,
        retained_provenance: None,
        status: "denied",
        reason,
    }
}

fn distribution_transport_selection(
    selection: &RegistrySelectionDecision<'_>,
) -> DistributionTransportSelection {
    DistributionTransportSelection {
        entry_id: SERIAL_DISTRIBUTION_ENTRY_ID,
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
