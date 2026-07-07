//! M9A-2b: the ONE system-authored durable-memory write driver.
//!
//! Builds exactly one fixed `raios.memory_record.v0` `capability_denial` record --
//! an honest audit of the permanently-denied generic durable module-load gate,
//! mirroring the REAL denial reasons `agent_protocol_module_grant.rs` renders
//! (durable audit missing, rollback plan missing, loader unavailable, service slot
//! unallocated) -- through the fail-closed `MemoryRecord::new`, then appends it via
//! `durable_store::append_memory_record`, authorized ONLY by
//! `evaluate_scoped_memory_record_append`. This GRANTS NOTHING: it is a passive
//! durable record of an already-denied capability, not a capability grant. There is
//! no agent-authored write path yet (a later slice); this file's only callers are the
//! two `memory.record_log_append*` Read0 methods wired in `agent_protocol.rs`.
//!
//! `boot_id` is the fixed `"current_boot"` marker. `sequence`/`created_at_ticks` fall
//! back to `0`: there is no RAM-only, side-effect-free way to peek the current
//! event-log sequence without recording a new event, so `0` is the honest,
//! deterministic-within-boot fallback (stated in the M9A-2b report).

use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol::durable_store,
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
};
use raios_core::{
    memory_record::{MemoryRecord, MemoryRecordError, MemoryRecordInput, MemorySource},
    record::Value as V,
    scoped_memory_record_append::{
        evaluate_scoped_memory_record_append, ScopedMemoryRecordAppendInput, EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA, EXPECTED_REGION_MARKER, EXPECTED_TARGET_ID, EXPECTED_TRUST_TIER,
    },
};

const METHOD: &str = "memory.record_log_append";
const RESPONSE_SCHEMA: &str = "raios.memory_record_append.v0";
const SELFTEST_METHOD: &str = "memory.record_log_append_selftest";
const SELFTEST_SCHEMA: &str = "raios.memory_record_append_selftest.v0";

/// The ONE fixed system-authored record this slice ever builds.
fn module_load_gate_denial_record() -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: "mem.capability_denial.module_load_ephemeral_durable.current_boot.v0",
        kind: "capability_denial",
        entity: "cap.module.load_ephemeral.durable",
        predicate: "capability_denied",
        value: module_load_gate_denial_value(),
        classification: "local_only",
        authority: "core_ledger",
        boot_id: "current_boot",
        // No RAM-only peek at the current event-log sequence exists without
        // recording a new event, so 0 is the honest, deterministic fallback for a
        // fixed single-record-per-boot driver (see module doc comment).
        sequence: 0,
        source: MemorySource::new("memory.record_log_append", "raios.module_load_gate.v0"),
        evidence: vec![],
        tags: vec!["capability", "module_load", "gate"],
        supersedes: vec![],
        created_at_ticks: 0,
    })
}

/// The record's `value`: the REAL current denial reasons rendered by
/// `agent_protocol_module_grant.rs::emit_module_grant_diagnostic` (durable audit
/// missing, rollback plan missing, loader unavailable, service slot unallocated) --
/// not invented reasons.
fn module_load_gate_denial_value() -> V<'static> {
    V::Object(vec![
        f("requested_capability", s("cap.module.load_ephemeral")),
        f("can_load_now", b(false)),
        f("durable_audit_record_status", s("missing")),
        f(
            "durable_audit_record_reason",
            s("durable_audit_write_missing"),
        ),
        f("rollback_plan_status", s("missing")),
        f("rollback_plan_reason", s("rollback_install_missing")),
        f("loader_status", s("unavailable")),
        f("loader_reason", s("module_loader_unimplemented")),
        f("service_slot_status", s("unallocated")),
        f(
            "service_slot_reason",
            s("ram_only_service_slot_unallocated"),
        ),
    ])
}

pub(crate) fn emit_memory_record_log_append() {
    let record = match module_load_gate_denial_record() {
        Ok(record) => record,
        Err(err) => {
            // Fail-closed construction error: a RAM-only capability_denied response,
            // NEVER a durable append. Unreachable for this fixed valid record, but
            // coded fail-closed regardless.
            emit_memory_record_append_denied(err.reason());
            return;
        }
    };

    let evidence = durable_store::append_memory_record(&record);
    emit_memory_record_append_response(&evidence);
}

fn emit_memory_record_append_denied(reason: &'static str) {
    begin_response(METHOD);
    emit_record_fields(
        vec![
            f("schema", s(RESPONSE_SCHEMA)),
            f("query_method", s(METHOD)),
            f("durable_append", s("capability_denied")),
            f("performed", b(false)),
            f("reason", s(reason)),
            f("authority", s("evidence_only")),
            f("record_schema", s(EXPECTED_RECORD_SCHEMA)),
            f("region_marker", s(EXPECTED_REGION_MARKER)),
            f("target_id", s(EXPECTED_TARGET_ID)),
            f("trust_tier", s(EXPECTED_TRUST_TIER)),
            f("owner_sealed", b(false)),
            f("persistence_claimed", b(false)),
        ],
        6,
    );
    end_response(METHOD);
}

fn emit_memory_record_append_response(evidence: &durable_store::MemoryRecordAppendEvidence) {
    begin_response(METHOD);
    emit_record_fields(
        vec![
            f("schema", s(RESPONSE_SCHEMA)),
            f("query_method", s(METHOD)),
            f("durable_append", s(evidence.durable_append)),
            f("performed", b(evidence.performed)),
            f("reason", s(evidence.reason)),
            f("authority", s(evidence.authority)),
            f("record_id", s(evidence.record_id)),
            f("kind", s(evidence.record_kind)),
            f("classification", s(evidence.record_classification)),
            f("record_authority", s(evidence.record_authority)),
            f("record_schema", s(evidence.record_schema)),
            f("region_marker", s(evidence.region_marker)),
            f("target_id", s(evidence.target_id)),
            f("seq", optional_u64(evidence.seq)),
            f("write_offset", optional_u64(evidence.write_offset)),
            f("frame_len", optional_u64(evidence.frame_len)),
            f(
                "payload_sha256",
                record_sha_or_null(evidence.payload_sha256),
            ),
            f("frame_sha256", record_sha_or_null(evidence.frame_sha256)),
            f(
                "readback_sha256",
                record_sha_or_null(evidence.readback_sha256),
            ),
            f("reparse_valid", b(evidence.reparse_valid)),
            f("tail_seq_before", optional_u64(evidence.tail_seq_before)),
            f("count_before", optional_u64(evidence.count_before)),
            f("tail_seq_after", optional_u64(evidence.tail_seq_after)),
            f("count_after", optional_u64(evidence.count_after)),
            f("owner_sealed", b(evidence.owner_sealed)),
            f("persistence_claimed", b(evidence.persistence_claimed)),
            f("trust_tier", s(evidence.trust_tier)),
        ],
        6,
    );
    end_response(METHOD);
}

fn optional_u64(value: Option<u64>) -> V<'static> {
    match value {
        Some(value) => V::U64(value),
        None => V::Null,
    }
}

// --- synthetic fail-closed selftest (NO disk write, RAM-only) ---------------------

#[derive(Clone, Copy)]
struct MemoryRecordAppendSelftestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

pub(crate) fn emit_memory_record_log_append_selftest() {
    // Drive the REAL per-boot RAM quota to exhaustion and back (no durable write),
    // so the proof shows the live gate genuinely fires + refunds, not just the
    // synthetic evaluator `quota_ok=false` path.
    let (quota_reservations, quota_restored) = durable_store::memory_write_quota_probe_exhaustion();
    let cases = selftest_cases(quota_reservations, quota_restored);
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases.iter().map(record_selftest_case).collect();

    begin_response(SELFTEST_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(SELFTEST_SCHEMA)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("writes_persistent_state", b(false)),
            f(
                "quota_budget_records",
                V::U64(durable_store::MEMORY_WRITE_QUOTA_BUDGET_RECORDS),
            ),
            f(
                "quota_budget_bytes",
                V::U64(durable_store::MEMORY_WRITE_QUOTA_BUDGET_BYTES),
            ),
            f(
                "live_quota_reservations_until_exhausted",
                V::U64(quota_reservations as u64),
            ),
            f("live_quota_restored", b(quota_restored)),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    end_response(SELFTEST_METHOD);
}

fn record_selftest_case(case: &MemoryRecordAppendSelftestCase) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("passed", b(case.passed)),
    ])
}

fn selftest_cases(
    quota_reservations: u32,
    quota_restored: bool,
) -> Vec<MemoryRecordAppendSelftestCase> {
    vec![
        constructor_case(
            "secret_classification_constructor_denied",
            "secret",
            "capability_denial",
        ),
        constructor_case(
            "unknown_kind_constructor_denied",
            "local_only",
            "chat_history",
        ),
        scoped_case(
            "scoped_classification_secret_denied",
            "classification_secret_never_durable",
            ScopedMutation::Classification,
        ),
        scoped_case(
            "scoped_kind_out_of_scope_denied",
            "memory_record_kind_out_of_scope",
            ScopedMutation::Kind,
        ),
        scoped_case(
            "scoped_quota_exhausted_denied",
            "memory_write_quota_exhausted",
            ScopedMutation::Quota,
        ),
        live_quota_case(quota_reservations, quota_restored),
    ]
}

/// M1: the REAL RAM quota, driven live to exhaustion and back by
/// `durable_store::memory_write_quota_probe_exhaustion`. Passes iff at least one
/// reservation succeeded before the live gate fired (finite budget genuinely
/// enforced) AND the quota fully refunded afterwards (a transient denial never
/// permanently burns the boot budget).
fn live_quota_case(reservations: u32, restored: bool) -> MemoryRecordAppendSelftestCase {
    let exhausted_then_restored = reservations >= 1 && restored;
    MemoryRecordAppendSelftestCase {
        name: "live_quota_exhausted_and_restored",
        expected_status: "exhausted_and_restored",
        expected_reason: "memory_write_quota_exhausted",
        actual_status: if exhausted_then_restored {
            "exhausted_and_restored"
        } else if reservations == 0 {
            "never_reserved"
        } else {
            "not_restored"
        },
        actual_reason: if exhausted_then_restored {
            "memory_write_quota_exhausted"
        } else {
            "live_quota_probe_failed"
        },
        passed: exhausted_then_restored,
    }
}

/// Exercises `MemoryRecord::new`'s own fail-closed constructor checks directly
/// (secret classification / unknown kind), NEVER touching the disk and NEVER
/// calling `durable_store::append_memory_record`.
fn constructor_case(
    name: &'static str,
    classification: &'static str,
    kind: &'static str,
) -> MemoryRecordAppendSelftestCase {
    let expected_reason = if classification == "secret" {
        MemoryRecordError::SecretNeverDurable.reason()
    } else {
        MemoryRecordError::UnknownKind.reason()
    };
    let result = MemoryRecord::new(MemoryRecordInput {
        id: "mem.selftest.constructor.v0",
        kind,
        entity: "cap.selftest.constructor",
        predicate: "selftest_probe",
        value: V::Null,
        classification,
        authority: "core_ledger",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new("memory.record_log_append_selftest", "selftest.synthetic.v0"),
        evidence: vec![],
        tags: vec![],
        supersedes: vec![],
        created_at_ticks: 0,
    });
    let (actual_status, actual_reason) = match result {
        Ok(_) => ("constructed", "constructed_unexpectedly"),
        Err(err) => ("denied", err.reason()),
    };
    MemoryRecordAppendSelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status,
        actual_reason,
        passed: actual_status == "denied" && actual_reason == expected_reason,
    }
}

#[derive(Clone, Copy)]
enum ScopedMutation {
    Classification,
    Kind,
    Quota,
}

/// A fully-valid synthetic `ScopedMemoryRecordAppendInput` (an empty-log first
/// append, matching the raios-core evaluator's own `valid_input()` shape), so each
/// case below mutates EXACTLY one pin and isolates its distinct denial reason.
fn scoped_baseline_input() -> ScopedMemoryRecordAppendInput<'static> {
    let payload_hash = [0xAAu8; 32];
    let frame_hash = [0xBBu8; 32];
    ScopedMemoryRecordAppendInput {
        method: Some(EXPECTED_METHOD),
        target_id: Some(EXPECTED_TARGET_ID),
        record_schema: Some(EXPECTED_RECORD_SCHEMA),
        region_marker: Some(EXPECTED_REGION_MARKER),
        frame_len: Some(512),
        write_offset: Some(0),
        reclog_byte_count: Some(4096),
        absolute_start_lba: Some(100),
        reclog_lba_count: Some(8),
        seq: Some(1),
        tail_seq: None,
        count: Some(0),
        prev_frame_sha256: Some([0u8; 32]),
        tail_frame_sha256: None,
        payload_sha256: Some(payload_hash),
        planned_payload_sha256: Some(payload_hash),
        planned_frame_sha256: Some(frame_hash),
        readback_frame_sha256: Some(frame_hash),
        write_attempted: true,
        write_completed: true,
        readback_completed: true,
        readback_matches_planned: true,
        reparse_valid: true,
        span_in_bounds: true,
        classification: Some("local_only"),
        kind: Some("observation"),
        trust_tier: Some(EXPECTED_TRUST_TIER),
        owner_sealed: false,
        persistence_claimed: false,
        quota_ok: true,
    }
}

fn scoped_case(
    name: &'static str,
    expected_reason: &'static str,
    mutation: ScopedMutation,
) -> MemoryRecordAppendSelftestCase {
    let mut input = scoped_baseline_input();
    match mutation {
        ScopedMutation::Classification => input.classification = Some("secret"),
        ScopedMutation::Kind => input.kind = Some("chat_history"),
        ScopedMutation::Quota => input.quota_ok = false,
    }
    let decision = evaluate_scoped_memory_record_append(&input);
    MemoryRecordAppendSelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status: decision.status,
        actual_reason: decision.reason,
        passed: decision.status == "denied" && decision.reason == expected_reason,
    }
}
