//! M9A-2b/M9A-3b: the system-authored durable-memory write drivers.
//!
//! M9A-2b builds exactly one fixed `raios.memory_record.v0` `capability_denial`
//! record -- an honest audit of the permanently-denied generic durable module-load
//! gate, mirroring the REAL denial reasons `agent_protocol_module_grant.rs` renders
//! (durable audit missing, rollback plan missing, loader unavailable, service slot
//! unallocated) -- through the fail-closed `MemoryRecord::new`, then appends it via
//! `durable_store::append_memory_record`, authorized ONLY by
//! `evaluate_scoped_memory_record_append`.
//!
//! M9A-3b (below) builds THREE more fixed records through the SAME gauntlet: a
//! general standing `decision` (A), an honest `problem` naming the current
//! memory-mutation-policy limitation (P), and a refined `decision` (B) that
//! SUPERSEDES A -- the write-side proof of supersede-not-overwrite.
//!
//! Both drivers GRANT NOTHING: they are passive durable records of already-true
//! system facts, never a capability grant. This file's system-authored callers are
//! the `memory.record_log_append*` and `memory.decision_problem_log_append` Read0
//! methods wired in `agent_protocol.rs`.
//!
//! M9B-1b (near the bottom) adds the FIRST agent-authored write path:
//! `memory.observation_log_append` lets an AGENT durably record ONE `observation`
//! -- entity/predicate/value/source_record_id, base64-encoded, newline-separated --
//! while the kernel FORCES every authority-bearing field (id, kind, classification,
//! authority, boot_id, supersedes, tags) and appends through the SAME shared
//! gauntlet via `durable_store::append_agent_observation_record`, authorized ONLY by
//! `evaluate_scoped_memory_record_append` with `agent_authored=true` (M9B-1a,
//! already merged). The broad `memory.record_observation` and the entire
//! `MEMORY_MUTATION_METHODS` set are untouched by this file and stay denied.
//!
//! `boot_id` is the fixed `"current_boot"` marker. The system drivers'
//! `sequence`/`created_at_ticks` fall back to `0`: there is no RAM-only,
//! side-effect-free way to peek the current event-log sequence without recording a
//! new event, so `0` is the honest, deterministic-within-boot fallback (stated in
//! the M9A-2b report). The agent-observation driver instead has its OWN per-boot
//! RAM-only counter (`next_agent_observation_seq`), since its id must be unique per
//! write attempt.
//!
//! M9C-2b/M9C-2c-2 (bottom section) records durable provider-export audits
//! through the SAME system-authored append gauntlet: denials are deduped per
//! boot. The original authorized-audit driver remains fixed selftest
//! infrastructure; the live scoped-feedback path below appends a unique
//! per-request `export_audit` before any provider write.

use alloc::{format, vec, vec::Vec};

use spin::Mutex;

use crate::{
    agent_protocol::durable_store,
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, method_head_eq, record_bool as b,
        record_field as f, record_sha_or_null, record_str as s,
    },
    module_candidate_channel, wasm_runtime,
};
use raios_core::{
    memory_record::{
        Classification, MemoryKind, MemoryRecord, MemoryRecordError, MemoryRecordInput,
        MemoryRecordView, MemorySource,
    },
    memory_record_resolve::resolve_durable_memory,
    record::{Field, Value as V},
    scoped_memory_record_append::{
        evaluate_scoped_memory_record_append, ScopedMemoryRecordAppendInput, EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA, EXPECTED_REGION_MARKER, EXPECTED_TARGET_ID, EXPECTED_TRUST_TIER,
    },
    scoped_provider_export::{
        export_denial_dedupe_key, SCOPED_PROVIDER_EXPORT_DECISION_ID,
        SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA,
    },
    scoped_wasm_import_grant::{
        authorized_import_list_sha256, SCOPED_WASM_IMPORT_GRANT_DECISION_ID,
        SCOPED_WASM_IMPORT_GRANT_DECISION_SCHEMA,
    },
};

const METHOD: &str = "memory.record_log_append";
const RESPONSE_SCHEMA: &str = "raios.memory_record_append.v0";
const SELFTEST_METHOD: &str = "memory.record_log_append_selftest";
const SELFTEST_SCHEMA: &str = "raios.memory_record_append_selftest.v0";
const DECISION_PROBLEM_METHOD: &str = "memory.decision_problem_log_append";
const DECISION_PROBLEM_RESPONSE_SCHEMA: &str = "raios.memory_decision_problem_append.v0";
const BROKER_RESOLVE_SELFTEST_METHOD: &str = "memory.broker_resolve_selftest";
const BROKER_RESOLVE_SELFTEST_SCHEMA: &str = "raios.memory_broker_resolve_selftest.v0";
const PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD: &str = "memory.provider_export_public_fixture_append";

/// Record A source: the owner-answers standing decision that module sharing is
/// confirmed vision (commit b7241b2 / the M12+ direction review), NOT
/// speculation. Superseded by Record B below (refined, not overwritten).
const DECISION_PROBLEM_SOURCE_RECORD_ID: &str =
    "docs/plan-reviews/m12-plus-direction-2026-07-06.md";

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

fn emit_memory_record_append_response(evidence: &durable_store::MemoryRecordAppendEvidence<'_>) {
    let mut fields = vec![
        f("schema", s(RESPONSE_SCHEMA)),
        f("query_method", s(METHOD)),
    ];
    fields.extend(memory_record_evidence_fields(evidence));
    begin_response(METHOD);
    emit_record_fields(fields, 6);
    end_response(METHOD);
}

/// The evidence-field rendering shared by the single-record `memory.record_log_append`
/// response and each entry of the multi-record `memory.decision_problem_log_append`
/// response array (M9A-3b). Field names/order are IDENTICAL to the original M9A-2b
/// single-record body (everything after `schema`/`query_method`), so refactoring this
/// out changes nothing about the existing method's wire shape.
fn memory_record_evidence_fields<'a>(
    evidence: &durable_store::MemoryRecordAppendEvidence<'a>,
) -> Vec<Field<'a>> {
    vec![
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
    ]
}

fn optional_u64(value: Option<u64>) -> V<'static> {
    match value {
        Some(value) => V::U64(value),
        None => V::Null,
    }
}

pub(crate) fn durable_memory_context() -> durable_store::DurableMemoryContext {
    durable_store::current_boot_durable_memory_context()
}

pub(crate) fn durable_records_array<'a>(context: &'a durable_store::DurableMemoryContext) -> V<'a> {
    let mut records = Vec::with_capacity(context.records.len());
    let mut idx = 0usize;
    while idx < context.records.len() {
        let record = &context.records[idx];
        records.push(V::InlineObject(vec![
            f("id", s(record.id.as_str())),
            f("kind", s(record.kind)),
            f("entity", s(record.entity.as_str())),
            f("predicate", s(record.predicate.as_str())),
            f("classification", s(record.classification)),
            f("authority", s(record.authority.as_str())),
            f("scope", s("durable")),
            f("exportable", b(record.exportable)),
        ]));
        idx += 1;
    }
    V::Array(records)
}

pub(crate) fn durable_omitted_folds<'a>(
    context: &'a durable_store::DurableMemoryContext,
) -> Vec<V<'a>> {
    let mut folds = Vec::new();
    if !context.superseded_hidden.is_empty() {
        folds.push(V::InlineObject(vec![
            f("kind", s("durable_superseded")),
            f("ids", string_array(&context.superseded_hidden)),
        ]));
    }
    if !context.r1_ignored_supersedes.is_empty() {
        folds.push(V::InlineObject(vec![
            f("kind", s("durable_r1_ignored_supersede")),
            f(
                "reason",
                s("supersede target is an audit kind; audit stays visible"),
            ),
            f(
                "links",
                supersede_link_array(&context.r1_ignored_supersedes),
            ),
        ]));
    }
    if !context.dangling_supersedes.is_empty() {
        folds.push(V::InlineObject(vec![
            f("kind", s("durable_dangling_supersede")),
            f("ids", string_array(&context.dangling_supersedes)),
        ]));
    }
    if !context.audit_id_reused.is_empty() {
        folds.push(V::InlineObject(vec![
            f("kind", s("durable_audit_id_reused")),
            f("ids", string_array(&context.audit_id_reused)),
        ]));
    }
    if context.over_budget {
        folds.push(V::InlineObject(vec![f(
            "kind",
            s("durable_records_over_budget"),
        )]));
    }
    if context.frame_dropped_count > 0 {
        folds.push(V::InlineObject(vec![
            f("kind", s("durable_frame_dropped")),
            f("count", V::U64(context.frame_dropped_count)),
        ]));
    }
    if context.local_only_visible {
        folds.push(V::InlineObject(vec![f(
            "kind",
            s("durable_local_only_value"),
        )]));
    }
    folds
}

fn string_array<'a>(ids: &'a [alloc::string::String]) -> V<'a> {
    let mut values = Vec::with_capacity(ids.len());
    let mut idx = 0usize;
    while idx < ids.len() {
        values.push(s(ids[idx].as_str()));
        idx += 1;
    }
    V::Array(values)
}

fn supersede_link_array<'a>(links: &'a [durable_store::DurableMemorySupersedeLink]) -> V<'a> {
    let mut values = Vec::with_capacity(links.len());
    let mut idx = 0usize;
    while idx < links.len() {
        values.push(V::InlineArray(vec![
            s(links[idx].superseder.as_str()),
            s(links[idx].target.as_str()),
        ]));
        idx += 1;
    }
    V::Array(values)
}

// --- M9A-3b: decision/problem/supersede trio -------------------------------------
//
// Builds and durably appends THREE truthful system-authored records, in order A, P,
// B: a general standing `decision` (A), an honest `problem` naming the current
// memory-mutation-policy limitation (P), and a refined `decision` (B) that
// SUPERSEDES A (`supersedes: [A.id]`). This is the write-side proof of
// supersede-not-overwrite: constructing B never mutates A, it only carries the link.
// GRANTS NOTHING new -- system-authored only, through the SAME
// `durable_store::append_memory_record` gauntlet authorized ONLY by
// `evaluate_scoped_memory_record_append`.

/// Record A: the general owner-confirmed-vision standing decision (source: owner
/// answers commit b7241b2 / the M12+ direction review). Superseded by Record B.
fn decision_module_sharing_confirmed_vision_record(
) -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: "mem.decision.module_sharing_confirmed_vision.current_boot.v0",
        kind: "decision",
        entity: "raios.module_sharing",
        predicate: "standing_decision",
        value: V::Object(vec![
            f(
                "statement",
                s("module sharing between raiOS users is owner-confirmed vision, not speculation"),
            ),
            f("audience", s("everyone who likes the philosophy")),
            f("precision", s("general")),
        ]),
        classification: "local_only",
        authority: "decision",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new(DECISION_PROBLEM_METHOD, DECISION_PROBLEM_SOURCE_RECORD_ID),
        evidence: vec![],
        tags: vec!["decision", "vision", "module_sharing"],
        supersedes: vec![],
        created_at_ticks: 0,
    })
}

/// Record P: the honest current memory-mutation-policy limitation (source:
/// `agent_protocol_memory.rs`'s `mutation_policy` field, mirrored verbatim).
fn problem_memory_mutation_denied_record() -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: "mem.problem.memory_mutation_denied.current_boot.v0",
        kind: "problem",
        entity: "memory.mutation_policy",
        predicate: "open",
        value: V::Object(vec![
            f("severity", s("info")),
            f("status", s("open")),
            f(
                "policy",
                s("denied_until_event_log_audit_policy_persistence_and_rollback_exist"),
            ),
            f(
                "summary",
                s(
                    "memory.* mutations remain denied until durable audit, policy, source retention, persistence, and rollback evidence exist",
                ),
            ),
            f("source_field", s("agent_protocol_memory.mutation_policy")),
        ]),
        classification: "local_only",
        authority: "event",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new(
            DECISION_PROBLEM_METHOD,
            "agent_protocol_memory.mutation_policy",
        ),
        evidence: vec![],
        tags: vec!["problem", "memory", "policy"],
        supersedes: vec![],
        created_at_ticks: 0,
    })
}

/// Record B: the refined decision that SUPERSEDES A (`supersedes: [A.id]`). Same
/// source as A -- download/share is candidate intake, NEVER an install.
fn decision_module_sharing_evidence_gated_record(
) -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: "mem.decision.module_sharing_evidence_gated.current_boot.v0",
        kind: "decision",
        entity: "raios.module_sharing",
        predicate: "standing_decision",
        value: V::Object(vec![
            f(
                "statement",
                s(
                    "module sharing is evidence-gated: a shared or downloaded module is candidate intake, NEVER an install",
                ),
            ),
            f("precision", s("refined")),
            f(
                "supersede_reason",
                s("tightened_from_general_to_evidence_gated"),
            ),
        ]),
        classification: "local_only",
        authority: "decision",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new(DECISION_PROBLEM_METHOD, DECISION_PROBLEM_SOURCE_RECORD_ID),
        evidence: vec![],
        tags: vec!["decision", "vision", "module_sharing"],
        supersedes: vec!["mem.decision.module_sharing_confirmed_vision.current_boot.v0"],
        created_at_ticks: 0,
    })
}

pub(crate) fn emit_memory_decision_problem_log_append() {
    // Construct ALL THREE records BEFORE any append. Construction is side-effect-free
    // and fail-closed, so a construction Err can never hide an already-committed
    // append (there is no after-append error window) -- honestly "NEVER a partial
    // durable append". Unreachable for these fixed valid records, but coded
    // fail-closed regardless: a construction error is a RAM-only capability_denied
    // response, never a durable append.
    let record_a = match decision_module_sharing_confirmed_vision_record() {
        Ok(record) => record,
        Err(err) => {
            emit_memory_decision_problem_append_denied(err.reason());
            return;
        }
    };
    let record_p = match problem_memory_mutation_denied_record() {
        Ok(record) => record,
        Err(err) => {
            emit_memory_decision_problem_append_denied(err.reason());
            return;
        }
    };
    let record_b = match decision_module_sharing_evidence_gated_record() {
        Ok(record) => record,
        Err(err) => {
            emit_memory_decision_problem_append_denied(err.reason());
            return;
        }
    };

    // Append in order A, P, B -- each authorized ONLY by evaluate_scoped_memory_record_append.
    let evidence_a = durable_store::append_memory_record(&record_a);
    let evidence_p = durable_store::append_memory_record(&record_p);
    let evidence_b = durable_store::append_memory_record(&record_b);

    emit_memory_decision_problem_append_response(&[
        (&record_a, &evidence_a),
        (&record_p, &evidence_p),
        (&record_b, &evidence_b),
    ]);
}

fn emit_memory_decision_problem_append_denied(reason: &'static str) {
    begin_response(DECISION_PROBLEM_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(DECISION_PROBLEM_RESPONSE_SCHEMA)),
            f("query_method", s(DECISION_PROBLEM_METHOD)),
            f("durable_append", s("capability_denied")),
            f("performed", b(false)),
            f("reason", s(reason)),
            f("authority", s("evidence_only")),
            f("records", V::Array(vec![])),
            f("owner_sealed", b(false)),
            f("persistence_claimed", b(false)),
        ],
        6,
    );
    end_response(DECISION_PROBLEM_METHOD);
}

fn emit_memory_decision_problem_append_response(
    entries: &[(
        &MemoryRecord<'static>,
        &durable_store::MemoryRecordAppendEvidence<'static>,
    ); 3],
) {
    let mut records = Vec::with_capacity(entries.len());
    let mut all_performed = true;
    let mut idx = 0usize;
    while idx < entries.len() {
        let (record, evidence) = entries[idx];
        if !evidence.performed {
            all_performed = false;
        }
        records.push(record_decision_problem_entry(record, evidence));
        idx += 1;
    }

    // Top-level performed/durable_append are DERIVED from the real per-record evidence,
    // never hardcoded: if any of the three appends was denied (SAFE posture, quota
    // exhausted, no persist disk, scoped denial) this honestly reports capability_denied
    // / performed=false while the records[] array shows exactly which landed.
    begin_response(DECISION_PROBLEM_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(DECISION_PROBLEM_RESPONSE_SCHEMA)),
            f("query_method", s(DECISION_PROBLEM_METHOD)),
            f(
                "durable_append",
                s(if all_performed {
                    "appended"
                } else {
                    "capability_denied"
                }),
            ),
            f("performed", b(all_performed)),
            f(
                "reason",
                s(if all_performed {
                    "decision_problem_trio_appended"
                } else {
                    "one_or_more_appends_denied"
                }),
            ),
            f("records", V::Array(records)),
        ],
        6,
    );
    end_response(DECISION_PROBLEM_METHOD);
}

/// One record's evidence object inside the `records` array: the shared evidence
/// fields PLUS an echo of the record's OWN `supersedes` list (empty for A/P, `[A.id]`
/// for B) -- the on-disk proof that the supersede link landed, not just an in-memory
/// claim.
fn record_decision_problem_entry(
    record: &MemoryRecord<'static>,
    evidence: &durable_store::MemoryRecordAppendEvidence<'static>,
) -> V<'static> {
    let mut fields = memory_record_evidence_fields(evidence);
    fields.push(f("supersedes", record_supersedes_array(record)));
    V::InlineObject(fields)
}

fn record_supersedes_array(record: &MemoryRecord<'static>) -> V<'static> {
    let mut values = Vec::with_capacity(record.supersedes.len());
    let mut idx = 0usize;
    while idx < record.supersedes.len() {
        values.push(V::Str(record.supersedes[idx]));
        idx += 1;
    }
    V::Array(values)
}

// --- M9C-2c-1: fixed public provider-export packet fixture -----------------------

fn provider_export_public_fixture_record() -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: "mem.decision.provider_export_public_fixture.current_boot.v0",
        kind: "decision",
        entity: "provider.export.public_fixture",
        predicate: "public_export_fixture_ready",
        value: V::Object(vec![
            f("fixture", b(true)),
            f("purpose", s("provider_export_public_filter_selftest")),
            f("content", s("public test fixture only")),
            f("grants_authority", b(false)),
        ]),
        classification: "public",
        authority: "decision",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new(
            PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD,
            "docs/architecture-decisions/0004-system-memory-and-agent-context.md",
        ),
        evidence: vec![],
        tags: vec!["decision", "provider_export", "test_fixture"],
        supersedes: vec![],
        created_at_ticks: 0,
    })
}

pub(crate) fn record_provider_export_public_fixture(
) -> Result<durable_store::MemoryRecordAppendEvidence<'static>, MemoryRecordError> {
    let record = provider_export_public_fixture_record()?;
    Ok(durable_store::append_memory_record(&record))
}

pub(crate) fn emit_provider_export_public_fixture_append() {
    let evidence = match record_provider_export_public_fixture() {
        Ok(evidence) => evidence,
        Err(err) => {
            begin_response(PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD);
            emit_record_fields(
                vec![
                    f("schema", s(RESPONSE_SCHEMA)),
                    f("query_method", s(PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD)),
                    f("durable_append", s("capability_denied")),
                    f("performed", b(false)),
                    f("reason", s(err.reason())),
                    f("authority", s("evidence_only")),
                    f("record_schema", s(EXPECTED_RECORD_SCHEMA)),
                    f("owner_sealed", b(false)),
                    f("persistence_claimed", b(false)),
                ],
                6,
            );
            end_response(PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD);
            return;
        }
    };

    let mut fields = vec![
        f("schema", s(RESPONSE_SCHEMA)),
        f("query_method", s(PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD)),
    ];
    fields.extend(memory_record_evidence_fields(&evidence));
    begin_response(PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD);
    emit_record_fields(fields, 6);
    end_response(PROVIDER_EXPORT_PUBLIC_FIXTURE_METHOD);
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
        audit_kind_supersede_denied_case(),
        supersedes_list_too_long_denied_case(),
        self_supersede_denied_case(),
        decision_missing_source_denied_case(),
        problem_missing_status_denied_case(),
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

/// Exercises `MemoryRecord::new` directly on a fully custom (M9A-3b) input, NEVER
/// touching the disk and NEVER calling `durable_store::append_memory_record`.
fn direct_constructor_case(
    name: &'static str,
    input: MemoryRecordInput<'static>,
    expected_reason: &'static str,
) -> MemoryRecordAppendSelftestCase {
    let result = MemoryRecord::new(input);
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

/// An audit kind (`capability_denial`) may never be authored as a superseding
/// record -- `MemoryRecord::new`'s own supersede-hazard-closure check.
fn audit_kind_supersede_denied_case() -> MemoryRecordAppendSelftestCase {
    direct_constructor_case(
        "audit_kind_supersede_denied",
        MemoryRecordInput {
            id: "mem.selftest.audit_kind_supersede.v0",
            kind: "capability_denial",
            entity: "cap.selftest.audit_kind_supersede",
            predicate: "selftest_probe",
            value: V::Null,
            classification: "local_only",
            authority: "core_ledger",
            boot_id: "current_boot",
            sequence: 0,
            source: MemorySource::new(SELFTEST_METHOD, "selftest.synthetic.v0"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec!["mem.selftest.audit_kind_supersede.prior.v0"],
            created_at_ticks: 0,
        },
        MemoryRecordError::AuditKindMayNotSupersede.reason(),
    )
}

/// A `supersedes` list past `MAX_SUPERSEDES_PER_RECORD` (8) is rejected. Uses a
/// non-audit kind (`observation`) so this isolates the length check from the
/// audit-kind check above.
fn supersedes_list_too_long_denied_case() -> MemoryRecordAppendSelftestCase {
    direct_constructor_case(
        "supersedes_list_too_long_denied",
        MemoryRecordInput {
            id: "mem.selftest.supersedes_too_long.v0",
            kind: "observation",
            entity: "cap.selftest.supersedes_too_long",
            predicate: "selftest_probe",
            value: V::Null,
            classification: "local_only",
            authority: "event",
            boot_id: "current_boot",
            sequence: 0,
            source: MemorySource::new(SELFTEST_METHOD, "selftest.synthetic.v0"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![
                "id.1", "id.2", "id.3", "id.4", "id.5", "id.6", "id.7", "id.8", "id.9",
            ],
            created_at_ticks: 0,
        },
        MemoryRecordError::SupersedesListTooLong.reason(),
    )
}

/// A record may never name its own id in `supersedes`.
fn self_supersede_denied_case() -> MemoryRecordAppendSelftestCase {
    direct_constructor_case(
        "self_supersede_denied",
        MemoryRecordInput {
            id: "mem.selftest.self_supersede.v0",
            kind: "decision",
            entity: "cap.selftest.self_supersede",
            predicate: "selftest_probe",
            value: V::Null,
            classification: "local_only",
            authority: "decision",
            boot_id: "current_boot",
            sequence: 0,
            source: MemorySource::new(SELFTEST_METHOD, "selftest.synthetic.v0"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec!["mem.selftest.self_supersede.v0"],
            created_at_ticks: 0,
        },
        MemoryRecordError::SelfSupersede.reason(),
    )
}

/// A `decision` with an empty source (`MemorySource::new("", "")`) is rejected --
/// a decision must name the method/record that produced it.
fn decision_missing_source_denied_case() -> MemoryRecordAppendSelftestCase {
    direct_constructor_case(
        "decision_missing_source_denied",
        MemoryRecordInput {
            id: "mem.selftest.decision_missing_source.v0",
            kind: "decision",
            entity: "cap.selftest.decision_missing_source",
            predicate: "selftest_probe",
            value: V::Null,
            classification: "local_only",
            authority: "decision",
            boot_id: "current_boot",
            sequence: 0,
            source: MemorySource::new("", ""),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 0,
        },
        MemoryRecordError::DecisionMissingSource.reason(),
    )
}

/// A `problem` with an empty predicate (status) is rejected.
fn problem_missing_status_denied_case() -> MemoryRecordAppendSelftestCase {
    direct_constructor_case(
        "problem_missing_status_denied",
        MemoryRecordInput {
            id: "mem.selftest.problem_missing_status.v0",
            kind: "problem",
            entity: "cap.selftest.problem_missing_status",
            predicate: "",
            value: V::Null,
            classification: "local_only",
            authority: "event",
            boot_id: "current_boot",
            sequence: 0,
            source: MemorySource::new(SELFTEST_METHOD, "selftest.synthetic.v0"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 0,
        },
        MemoryRecordError::ProblemMissingStatus.reason(),
    )
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
        supersedes_len: Some(0),
        supersede_self_reference: false,
        trust_tier: Some(EXPECTED_TRUST_TIER),
        owner_sealed: false,
        persistence_claimed: false,
        quota_ok: true,
        agent_authored: false,
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

#[derive(Clone, Copy)]
struct MemoryBrokerResolveSelftestCase {
    name: &'static str,
    expected: &'static str,
    actual: &'static str,
    passed: bool,
}

pub(crate) fn emit_memory_broker_resolve_selftest() {
    let cases = broker_resolve_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases.iter().map(record_broker_resolve_case).collect();

    begin_response(BROKER_RESOLVE_SELFTEST_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(BROKER_RESOLVE_SELFTEST_SCHEMA)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("writes_persistent_state", b(false)),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    end_response(BROKER_RESOLVE_SELFTEST_METHOD);
}

fn record_broker_resolve_case(case: &MemoryBrokerResolveSelftestCase) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected", s(case.expected)),
        f("actual", s(case.actual)),
        f("passed", b(case.passed)),
    ])
}

fn broker_resolve_selftest_cases() -> Vec<MemoryBrokerResolveSelftestCase> {
    vec![
        broker_r1_case(),
        broker_low1_case(),
        broker_audit_shadow_case(),
        broker_low3_case(),
    ]
}

fn broker_view(
    id: &'static str,
    kind: MemoryKind,
    entity: &'static str,
    predicate: &'static str,
    authority: &'static str,
    sequence: u64,
    source_record_id: &'static str,
    supersedes: Vec<&'static str>,
) -> MemoryRecordView<'static> {
    MemoryRecordView {
        id,
        kind,
        entity,
        predicate,
        classification: Classification::LocalOnly,
        authority,
        boot_id: "current_boot",
        sequence,
        source: MemorySource::new(BROKER_RESOLVE_SELFTEST_METHOD, source_record_id),
        evidence: vec![],
        tags: vec![],
        supersedes,
        created_at_ticks: 0,
    }
}

fn visible_id(resolved: &raios_core::memory_record_resolve::ResolvedMemory<'_>, id: &str) -> bool {
    resolved.visible.iter().any(|(_, view)| view.id == id)
}

fn broker_r1_case() -> MemoryBrokerResolveSelftestCase {
    let audit_id = "mem.selftest.r1.capability_denial.v0";
    let superseder_id = "mem.selftest.r1.decision.v0";
    let views = vec![
        (
            1,
            broker_view(
                audit_id,
                MemoryKind::CapabilityDenial,
                "cap.module.load",
                "capability_denied",
                "core_ledger",
                0,
                "selftest.audit",
                vec![],
            ),
        ),
        (
            2,
            broker_view(
                superseder_id,
                MemoryKind::Decision,
                "cap.module.load",
                "standing_decision",
                "decision",
                0,
                "selftest.decision",
                vec![audit_id],
            ),
        ),
    ];
    let resolved = resolve_durable_memory(&views);
    let passed = visible_id(&resolved, audit_id)
        && visible_id(&resolved, superseder_id)
        && resolved.superseded_hidden.is_empty()
        && resolved.r1_ignored_supersedes == vec![(superseder_id, audit_id)];
    MemoryBrokerResolveSelftestCase {
        name: "R1_audit_supersede_ignored",
        expected: "audit target stays visible and ignored link is recorded",
        actual: if passed { "matched" } else { "mismatch" },
        passed,
    }
}

fn broker_low1_case() -> MemoryBrokerResolveSelftestCase {
    let id = "mem.selftest.low1.same_id.v0";
    let old = broker_view(
        id,
        MemoryKind::Decision,
        "ordering",
        "old_payload_sequence_high",
        "decision",
        999,
        "selftest.old",
        vec![],
    );
    let new = broker_view(
        id,
        MemoryKind::Decision,
        "ordering",
        "new_frame_seq_wins",
        "decision",
        1,
        "selftest.new",
        vec![],
    );
    let views = vec![(1, old), (2, new)];
    let resolved = resolve_durable_memory(&views);
    let passed = resolved.visible.len() == 1
        && resolved.visible[0].0 == 2
        && resolved.visible[0].1.predicate == "new_frame_seq_wins";
    MemoryBrokerResolveSelftestCase {
        name: "LOW_1_frame_seq_beats_payload_sequence",
        expected: "latest frame seq wins even when payload sequence is lower",
        actual: if passed { "matched" } else { "mismatch" },
        passed,
    }
}

fn broker_audit_shadow_case() -> MemoryBrokerResolveSelftestCase {
    let id = "mem.selftest.audit_shadow.same_id.v0";
    let views = vec![
        (
            1,
            broker_view(
                id,
                MemoryKind::CapabilityDenial,
                "cap.audit",
                "capability_denied",
                "core_ledger",
                0,
                "selftest.audit",
                vec![],
            ),
        ),
        (
            2,
            broker_view(
                id,
                MemoryKind::Decision,
                "cap.audit",
                "same_id_shadow_attempt",
                "decision",
                0,
                "selftest.shadow",
                vec![],
            ),
        ),
    ];
    let resolved = resolve_durable_memory(&views);
    let passed = resolved.visible.len() == 1
        && resolved.visible[0].1.kind == MemoryKind::CapabilityDenial
        && resolved.audit_id_reused == vec![id];
    MemoryBrokerResolveSelftestCase {
        name: "audit_id_shadow_keeps_audit_visible",
        expected: "later non-audit same-id record cannot displace an audit record",
        actual: if passed { "matched" } else { "mismatch" },
        passed,
    }
}

fn broker_low3_case() -> MemoryBrokerResolveSelftestCase {
    let real_id = "mem.selftest.low3.real.v0";
    let decoy_id = "mem.selftest.low3.decoy.v0";
    let superseder_id = "mem.selftest.low3.superseder.v0";
    let real = broker_view(
        real_id,
        MemoryKind::Decision,
        "source_spoof",
        "real_record",
        "decision",
        0,
        decoy_id,
        vec![],
    );
    let views = vec![
        (1, real),
        (
            2,
            broker_view(
                decoy_id,
                MemoryKind::Problem,
                "source_spoof",
                "open",
                "event",
                0,
                "selftest.decoy",
                vec![],
            ),
        ),
        (
            3,
            broker_view(
                superseder_id,
                MemoryKind::Decision,
                "source_spoof",
                "supersede_decoy",
                "decision",
                0,
                "selftest.superseder",
                vec![decoy_id],
            ),
        ),
    ];
    let resolved = resolve_durable_memory(&views);
    let passed = visible_id(&resolved, real_id)
        && visible_id(&resolved, superseder_id)
        && !visible_id(&resolved, decoy_id)
        && resolved.superseded_hidden == vec![decoy_id];
    MemoryBrokerResolveSelftestCase {
        name: "LOW_3_source_record_id_spoof_is_not_identity",
        expected: "source.record_id never changes the record id used for supersession",
        actual: if passed { "matched" } else { "mismatch" },
        passed,
    }
}

// --- M9B-1b: the ONE agent-authored durable observation write path ---------------
//
// `memory.observation_log_append` lets an AGENT durably record ONE `observation`
// `raios.memory_record.v0`. The agent supplies exactly 4 fields -- entity,
// predicate, value, source_record_id -- as a SINGLE base64-encoded, newline-
// separated blob (decoded via the SAME hardened `decode_base64_chunk` the module
// candidate channel uses; no second decoder). Every other authority-bearing field
// (id, kind, classification, authority, boot_id, supersedes, tags) is FORCED by the
// kernel below, never taken from the agent. The record is appended through the
// SAME shared reclog gauntlet as the system-authored drivers above, via
// `durable_store::append_agent_observation_record`, authorized ONLY by
// `evaluate_scoped_memory_record_append` with `agent_authored=true` (M9B-1a,
// already merged: confines an agent record to `observation`/no-supersede/
// `local_only`). Every malformed or oversized input is a distinct, RAM-only
// `capability_denied` -- nothing is ever appended on a denial path.

const OBSERVATION_METHOD: &str = "memory.observation_log_append";
const OBSERVATION_RESPONSE_SCHEMA: &str = "raios.memory_observation_append.v0";

/// Field byte caps (agent-controlled fields only): entity/predicate/value/
/// source_record_id, in that order -- matching the 4-field blob layout.
const OBSERVATION_ENTITY_MAX: usize = 64;
const OBSERVATION_PREDICATE_MAX: usize = 32;
const OBSERVATION_VALUE_MAX: usize = 96;
const OBSERVATION_SOURCE_MAX: usize = 64;

/// Per-boot, RAM-only monotonic counter for agent-authored observation ids AND the
/// record's `sequence` field (the SAME counter drives both). A fresh static reset to
/// `0` every boot by construction -- never read from or written to disk. Only advanced
/// once a write ATTEMPT has passed every pre-append validation (so a MALFORMED input
/// never burns a number): `next_agent_observation_seq()` returns `1` on the first such
/// attempt of a boot, `2` next, etc.
///
/// IMPORTANT (M9B-1b review LOW-1): this is the AGENT's own within-boot attempt counter
/// for id uniqueness. It is NOT the RECLOG frame sequence (the durable-log ordering lives
/// in the frame envelope, not the payload). A well-formed input denied by the durable
/// gauntlet (quota/safe-mode/rescan) still advances it, so agent ids may have gaps and
/// `record.sequence` may differ from the frame's reclog seq. A future context broker MUST
/// order agent records by the reclog frame seq / boot_id, NEVER by this `sequence` field,
/// and MUST treat the agent-supplied `source.record_id` as an untrusted locator (trust
/// only the forced `authority="agent"` + `source.method`).
static AGENT_OBSERVATION_SEQ: Mutex<u64> = Mutex::new(0);

fn next_agent_observation_seq() -> u64 {
    let mut seq = AGENT_OBSERVATION_SEQ.lock();
    *seq = seq.saturating_add(1);
    *seq
}

/// Strips the `memory.observation_log_append` method token (any alias-free, exact
/// Head match) from the full dispatch input line, returning the trimmed base64 arg
/// that follows. Mirrors `agent_protocol_memory.rs::memory_method_arg`'s idiom.
fn observation_log_append_payload(input: &str) -> &str {
    let input = input.trim();
    if method_head_eq(input, OBSERVATION_METHOD) {
        input[OBSERVATION_METHOD.len()..].trim()
    } else {
        input
    }
}

/// The locator-safe charset every one of the 4 agent-supplied fields must be
/// entirely composed of: `[A-Za-z0-9 ._:/-]`. Excludes control bytes, non-ASCII,
/// quotes, and backslashes -- nothing that could break out of the record's rendered
/// JSON string or smuggle a control character into a durable fact.
fn is_observation_locator_safe_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'.' | b'_' | b':' | b'/' | b'-')
}

pub(crate) fn emit_memory_observation_log_append(input: &str) {
    let payload = observation_log_append_payload(input);
    if payload.is_empty() {
        emit_observation_append_denied("agent_observation_missing_payload");
        return;
    }

    let decoded = match module_candidate_channel::decode_base64_chunk(payload) {
        Ok(decoded) => decoded,
        // The decoder's OWN reason (rejected_malformed_base64_chunk /
        // rejected_empty_base64_chunk / rejected_invalid_base64_length /
        // rejected_invalid_base64_padding) -- distinct and already fail-closed;
        // reused verbatim rather than re-labelled.
        Err(reason) => {
            emit_observation_append_denied(reason);
            return;
        }
    };

    let fields: Vec<&[u8]> = decoded.split(|&byte| byte == b'\n').collect();
    if fields.len() != 4 {
        emit_observation_append_denied("agent_observation_field_count");
        return;
    }

    let caps = [
        OBSERVATION_ENTITY_MAX,
        OBSERVATION_PREDICATE_MAX,
        OBSERVATION_VALUE_MAX,
        OBSERVATION_SOURCE_MAX,
    ];
    let mut idx = 0usize;
    while idx < fields.len() {
        if fields[idx].len() > caps[idx] {
            emit_observation_append_denied("agent_observation_field_too_long");
            return;
        }
        idx += 1;
    }

    idx = 0;
    while idx < fields.len() {
        if !fields[idx]
            .iter()
            .all(|&byte| is_observation_locator_safe_byte(byte))
        {
            emit_observation_append_denied("agent_observation_field_charset");
            return;
        }
        idx += 1;
    }

    // The charset check above guarantees every field is pure ASCII (a strict UTF-8
    // subset), so `from_utf8` cannot fail here -- handled fail-closed regardless.
    let entity = match core::str::from_utf8(fields[0]) {
        Ok(value) => value,
        Err(_) => {
            emit_observation_append_denied("agent_observation_field_not_utf8");
            return;
        }
    };
    let predicate = match core::str::from_utf8(fields[1]) {
        Ok(value) => value,
        Err(_) => {
            emit_observation_append_denied("agent_observation_field_not_utf8");
            return;
        }
    };
    let value_scalar = match core::str::from_utf8(fields[2]) {
        Ok(value) => value,
        Err(_) => {
            emit_observation_append_denied("agent_observation_field_not_utf8");
            return;
        }
    };
    let source_record_id = match core::str::from_utf8(fields[3]) {
        Ok(value) => value,
        Err(_) => {
            emit_observation_append_denied("agent_observation_field_not_utf8");
            return;
        }
    };

    // Pre-checked here for a clear, distinct reason; `MemoryRecord::new`'s own
    // `ObservationMissingEntity`/`ObservationMissingSource` checks would otherwise
    // catch these (deny-in-depth), but never fire given these pre-checks.
    if entity.is_empty() {
        emit_observation_append_denied("agent_observation_entity_empty");
        return;
    }
    if source_record_id.is_empty() {
        emit_observation_append_denied("agent_observation_source_empty");
        return;
    }

    // Only NOW -- after every pre-append validation has passed -- does this attempt
    // consume a sequence number, so a malformed input never burns one.
    let seq = next_agent_observation_seq();
    let id = format!("mem.observation.agent.current_boot.{:08}.v0", seq);

    let record = match MemoryRecord::new(MemoryRecordInput {
        id: id.as_str(),
        kind: "observation",
        entity,
        predicate,
        value: V::Str(value_scalar),
        classification: "local_only",
        authority: "agent",
        boot_id: "current_boot",
        sequence: seq,
        source: MemorySource::new(OBSERVATION_METHOD, source_record_id),
        evidence: vec![],
        tags: vec!["agent", "observation"],
        supersedes: vec![],
        created_at_ticks: 0,
    }) {
        Ok(record) => record,
        Err(err) => {
            emit_observation_append_denied(err.reason());
            return;
        }
    };

    // The record (and everything it borrows: `id`, `entity`/`predicate`/
    // `value_scalar`/`source_record_id` slices of `decoded`) must stay alive through
    // rendering -- none of it is `'static`. `append_agent_observation_record` and
    // `emit_observation_append_response` both run BEFORE `record`/`decoded`/`id`
    // drop at the end of this function.
    let evidence = durable_store::append_agent_observation_record(&record);
    emit_observation_append_response(&evidence);
}

fn emit_observation_append_denied(reason: &'static str) {
    begin_response(OBSERVATION_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(OBSERVATION_RESPONSE_SCHEMA)),
            f("query_method", s(OBSERVATION_METHOD)),
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
    end_response(OBSERVATION_METHOD);
}

fn emit_observation_append_response(evidence: &durable_store::MemoryRecordAppendEvidence<'_>) {
    let mut fields = vec![
        f("schema", s(OBSERVATION_RESPONSE_SCHEMA)),
        f("query_method", s(OBSERVATION_METHOD)),
    ];
    fields.extend(memory_record_evidence_fields(evidence));
    begin_response(OBSERVATION_METHOD);
    emit_record_fields(fields, 6);
    end_response(OBSERVATION_METHOD);
}

// --- M9C-2b: durable provider-export denial audits -------------------------------

const PROVIDER_EXPORT_DENIAL_DEDUPE_CAP: usize = 16;

struct ProviderExportDenialDedupeEntry {
    gate: &'static str,
    reason: &'static str,
    payload_sha256: Option<[u8; 32]>,
    seq: Option<u64>,
}

static PROVIDER_EXPORT_DENIAL_DEDUPE: Mutex<Vec<ProviderExportDenialDedupeEntry>> =
    Mutex::new(Vec::new());

pub(crate) struct ProviderExportDenialAuditOutcome {
    pub(crate) dedupe: &'static str,
    pub(crate) record_id: &'static str,
    pub(crate) evidence: Option<durable_store::MemoryRecordAppendEvidence<'static>>,
    pub(crate) cited_payload_sha256: Option<[u8; 32]>,
    pub(crate) cited_seq: Option<u64>,
}

fn provider_export_denial_record_id(reason: &str) -> Option<&'static str> {
    match reason {
        "missing_method" => Some(
            "mem.capability_denial.provider_context_export.missing_method.current_boot.v0",
        ),
        "method_not_provider_context_export" => Some(
            "mem.capability_denial.provider_context_export.method_not_provider_context_export.current_boot.v0",
        ),
        "missing_profile" => Some(
            "mem.capability_denial.provider_context_export.missing_profile.current_boot.v0",
        ),
        "profile_not_provider_minimal" => Some(
            "mem.capability_denial.provider_context_export.profile_not_provider_minimal.current_boot.v0",
        ),
        "missing_trust_state" => Some(
            "mem.capability_denial.provider_context_export.missing_trust_state.current_boot.v0",
        ),
        "trust_dev_bypass_not_positive" => Some(
            "mem.capability_denial.provider_context_export.trust_dev_bypass_not_positive.current_boot.v0",
        ),
        "trust_state_not_positive" => Some(
            "mem.capability_denial.provider_context_export.trust_state_not_positive.current_boot.v0",
        ),
        "packet_contains_non_public_record" => Some(
            "mem.capability_denial.provider_context_export.packet_contains_non_public_record.current_boot.v0",
        ),
        "missing_packet_record_count" => Some(
            "mem.capability_denial.provider_context_export.missing_packet_record_count.current_boot.v0",
        ),
        "missing_budget_tokens" => Some(
            "mem.capability_denial.provider_context_export.missing_budget_tokens.current_boot.v0",
        ),
        "missing_packet_estimated_tokens" => Some(
            "mem.capability_denial.provider_context_export.missing_packet_estimated_tokens.current_boot.v0",
        ),
        "packet_exceeds_token_budget" => Some(
            "mem.capability_denial.provider_context_export.packet_exceeds_token_budget.current_boot.v0",
        ),
        "missing_audit_packet_hash" => Some(
            "mem.capability_denial.provider_context_export.missing_audit_packet_hash.current_boot.v0",
        ),
        "missing_audit_destination" => Some(
            "mem.capability_denial.provider_context_export.missing_audit_destination.current_boot.v0",
        ),
        "missing_audit_trust_snapshot" => Some(
            "mem.capability_denial.provider_context_export.missing_audit_trust_snapshot.current_boot.v0",
        ),
        "missing_audit_binding_hash" => Some(
            "mem.capability_denial.provider_context_export.missing_audit_binding_hash.current_boot.v0",
        ),
        _ => None,
    }
}

pub(crate) fn record_provider_export_denial_audit(
    reason: &'static str,
    profile: &'static str,
    trust_state: &'static str,
) -> ProviderExportDenialAuditOutcome {
    let Some(record_id) = provider_export_denial_record_id(reason) else {
        return ProviderExportDenialAuditOutcome {
            dedupe: "reason_unmapped_ram_only",
            record_id: "",
            evidence: None,
            cited_payload_sha256: None,
            cited_seq: None,
        };
    };

    {
        let table = PROVIDER_EXPORT_DENIAL_DEDUPE.lock();
        let wanted_key = export_denial_dedupe_key(SCOPED_PROVIDER_EXPORT_DECISION_ID, reason);
        let mut idx = 0usize;
        while idx < table.len() {
            let entry = &table[idx];
            if wanted_key == export_denial_dedupe_key(entry.gate, entry.reason) {
                return ProviderExportDenialAuditOutcome {
                    dedupe: "duplicate_ram_only",
                    record_id,
                    evidence: None,
                    cited_payload_sha256: entry.payload_sha256,
                    cited_seq: entry.seq,
                };
            }
            idx += 1;
        }
        if table.len() >= PROVIDER_EXPORT_DENIAL_DEDUPE_CAP {
            return ProviderExportDenialAuditOutcome {
                dedupe: "dedupe_table_full_ram_only",
                record_id,
                evidence: None,
                cited_payload_sha256: None,
                cited_seq: None,
            };
        }
    }

    let record = match provider_export_denial_record(record_id, reason, profile, trust_state) {
        Ok(record) => record,
        Err(_) => {
            return ProviderExportDenialAuditOutcome {
                dedupe: "record_construction_denied_ram_only",
                record_id,
                evidence: None,
                cited_payload_sha256: None,
                cited_seq: None,
            };
        }
    };

    // Single-threaded dispatch, but keep the RAM dedupe lock out of the disk gauntlet.
    let evidence = durable_store::append_memory_record(&record);
    if !evidence.performed {
        return ProviderExportDenialAuditOutcome {
            dedupe: "append_denied_ram_only",
            record_id,
            evidence: Some(evidence),
            cited_payload_sha256: None,
            cited_seq: None,
        };
    }

    {
        let mut table = PROVIDER_EXPORT_DENIAL_DEDUPE.lock();
        if table.len() < PROVIDER_EXPORT_DENIAL_DEDUPE_CAP {
            table.push(ProviderExportDenialDedupeEntry {
                gate: SCOPED_PROVIDER_EXPORT_DECISION_ID,
                reason,
                payload_sha256: evidence.payload_sha256,
                seq: evidence.seq,
            });
        }
    }

    ProviderExportDenialAuditOutcome {
        dedupe: "first_denial_appended",
        record_id,
        evidence: Some(evidence),
        cited_payload_sha256: evidence.payload_sha256,
        cited_seq: evidence.seq,
    }
}

fn provider_export_denial_record(
    record_id: &'static str,
    reason: &'static str,
    profile: &'static str,
    trust_state: &'static str,
) -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: record_id,
        kind: "capability_denial",
        entity: "cap.provider.context_export",
        predicate: "capability_denied",
        value: provider_export_denial_value(reason, profile, trust_state),
        classification: "local_only",
        authority: "core_ledger",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new(
            "provider.context_export",
            SCOPED_PROVIDER_EXPORT_DECISION_ID,
        ),
        evidence: vec![],
        tags: vec!["capability", "provider_export", "gate"],
        supersedes: vec![],
        created_at_ticks: 0,
    })
}

fn provider_export_denial_value(
    reason: &'static str,
    profile: &'static str,
    trust_state: &'static str,
) -> V<'static> {
    V::Object(vec![
        f("gate", s(SCOPED_PROVIDER_EXPORT_DECISION_ID)),
        f("gate_schema", s(SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA)),
        f("reason", s(reason)),
        f("method", s("provider.context_export")),
        f("profile", s(profile)),
        f("trust_state", s(trust_state)),
        f("packet_assembled", b(false)),
        f("export_performed", b(false)),
        f("provider_write", s("not_attempted")),
    ])
}

// --- M9C-2c-2: selftest-only authorized provider-export audit --------------------

const PROVIDER_EXPORT_AUTHORIZED_AUDIT_RECORD_ID: &str =
    "mem.export_audit.provider_context_export_selftest.current_boot.v0";

#[derive(Clone, Copy)]
struct ProviderExportAuthorizedDedupeEntry {
    payload_sha256: Option<[u8; 32]>,
    seq: Option<u64>,
}

static PROVIDER_EXPORT_AUTHORIZED_DEDUPE: Mutex<Option<ProviderExportAuthorizedDedupeEntry>> =
    Mutex::new(None);

pub(crate) struct ProviderExportAuthorizedAuditOutcome {
    pub(crate) dedupe: &'static str,
    pub(crate) record_id: &'static str,
    pub(crate) evidence: Option<durable_store::MemoryRecordAppendEvidence<'static>>,
    pub(crate) cited_payload_sha256: Option<[u8; 32]>,
    pub(crate) cited_seq: Option<u64>,
}

pub(crate) fn record_provider_export_authorized_audit(
    packet_hash: [u8; 32],
    packet_record_count: u64,
    destination: &'static str,
    trust_state: &'static str,
    budget_tokens: u64,
    audit_binding_hash: [u8; 32],
) -> ProviderExportAuthorizedAuditOutcome {
    {
        let dedupe = PROVIDER_EXPORT_AUTHORIZED_DEDUPE.lock();
        if let Some(entry) = *dedupe {
            return ProviderExportAuthorizedAuditOutcome {
                dedupe: "duplicate_ram_only",
                record_id: PROVIDER_EXPORT_AUTHORIZED_AUDIT_RECORD_ID,
                evidence: None,
                cited_payload_sha256: entry.payload_sha256,
                cited_seq: entry.seq,
            };
        }
    }

    let record = match provider_export_authorized_audit_record(
        packet_hash,
        packet_record_count,
        destination,
        trust_state,
        budget_tokens,
        audit_binding_hash,
    ) {
        Ok(record) => record,
        Err(_) => {
            return ProviderExportAuthorizedAuditOutcome {
                dedupe: "record_construction_denied_ram_only",
                record_id: PROVIDER_EXPORT_AUTHORIZED_AUDIT_RECORD_ID,
                evidence: None,
                cited_payload_sha256: None,
                cited_seq: None,
            };
        }
    };

    let evidence = durable_store::append_memory_record(&record);
    if !evidence.performed {
        return ProviderExportAuthorizedAuditOutcome {
            dedupe: "append_denied_ram_only",
            record_id: PROVIDER_EXPORT_AUTHORIZED_AUDIT_RECORD_ID,
            evidence: Some(evidence),
            cited_payload_sha256: None,
            cited_seq: None,
        };
    }

    let cited_payload_sha256 = evidence.payload_sha256;
    let cited_seq = evidence.seq;
    {
        let mut dedupe = PROVIDER_EXPORT_AUTHORIZED_DEDUPE.lock();
        if dedupe.is_none() {
            *dedupe = Some(ProviderExportAuthorizedDedupeEntry {
                payload_sha256: cited_payload_sha256,
                seq: cited_seq,
            });
        }
    }

    ProviderExportAuthorizedAuditOutcome {
        dedupe: "first_authorized_appended",
        record_id: PROVIDER_EXPORT_AUTHORIZED_AUDIT_RECORD_ID,
        evidence: Some(evidence),
        cited_payload_sha256,
        cited_seq,
    }
}

fn provider_export_authorized_audit_record(
    packet_hash: [u8; 32],
    packet_record_count: u64,
    destination: &'static str,
    trust_state: &'static str,
    budget_tokens: u64,
    audit_binding_hash: [u8; 32],
) -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: PROVIDER_EXPORT_AUTHORIZED_AUDIT_RECORD_ID,
        kind: "export_audit",
        entity: "cap.provider.context_export",
        predicate: "provider_export_authorized_selftest",
        value: provider_export_authorized_audit_value(
            packet_hash,
            packet_record_count,
            destination,
            trust_state,
            budget_tokens,
            audit_binding_hash,
        ),
        classification: "local_only",
        authority: "core_ledger",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new(
            "provider.context_export",
            SCOPED_PROVIDER_EXPORT_DECISION_ID,
        ),
        evidence: vec![],
        tags: vec!["provider_export", "export_audit", "selftest"],
        supersedes: vec![],
        created_at_ticks: 0,
    })
}

fn provider_export_authorized_audit_value(
    packet_hash: [u8; 32],
    packet_record_count: u64,
    destination: &'static str,
    trust_state: &'static str,
    budget_tokens: u64,
    audit_binding_hash: [u8; 32],
) -> V<'static> {
    V::Object(vec![
        f("gate", s(SCOPED_PROVIDER_EXPORT_DECISION_ID)),
        f("gate_schema", s(SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA)),
        f(
            "gate_reason",
            s("authorized_provider_export_public_only_audited"),
        ),
        f("method", s("provider.context_export")),
        f("profile", s("provider_minimal")),
        f("trust_state", s(trust_state)),
        f("destination", s(destination)),
        f("budget_tokens", V::U64(budget_tokens)),
        f("packet_hash", V::Sha256(packet_hash)),
        f("packet_record_count", V::U64(packet_record_count)),
        f("audit_binding_hash", V::Sha256(audit_binding_hash)),
        f("packet_assembled", b(true)),
        f("transmission_performed", b(false)),
        f("export_performed", b(false)),
        f("provider_write", s("selftest_no_transmission")),
        f("owner_sealed", b(false)),
        f("trust_tier", s("dev_key_not_owner_sealed")),
        f("test_infrastructure", b(true)),
    ])
}

#[derive(Clone, Copy)]
pub(crate) struct LiveProviderExportAuditOutcome {
    pub(crate) performed: bool,
    pub(crate) reason: &'static str,
    pub(crate) seq: Option<u64>,
    pub(crate) payload_sha256: Option<[u8; 32]>,
    pub(crate) frame_sha256: Option<[u8; 32]>,
    pub(crate) readback_sha256: Option<[u8; 32]>,
    pub(crate) reparse_valid: bool,
}

pub(crate) fn record_live_provider_export_audit(
    request_id: u32,
    packet_hash: [u8; 32],
    packet_record_count: u64,
    destination: &'static str,
    trust_state: &'static str,
    budget_tokens: u64,
    packet_estimated_tokens: u64,
    request_body_sha256: [u8; 32],
    request_envelope_sha256: [u8; 32],
    audit_binding_hash: [u8; 32],
    provider_trust_evidence_hash: [u8; 32],
) -> LiveProviderExportAuditOutcome {
    let record_id = format!(
        "mem.export_audit.provider_context_export.request.{request_id:08}.{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}.v0",
        request_envelope_sha256[0],
        request_envelope_sha256[1],
        request_envelope_sha256[2],
        request_envelope_sha256[3],
        request_envelope_sha256[4],
        request_envelope_sha256[5],
        request_envelope_sha256[6],
        request_envelope_sha256[7],
    );
    let record = match MemoryRecord::new(MemoryRecordInput {
        id: record_id.as_str(),
        kind: "export_audit",
        entity: "cap.provider.context_export",
        predicate: "provider_export_authorized_prewrite",
        value: V::Object(vec![
            f("gate", s(SCOPED_PROVIDER_EXPORT_DECISION_ID)),
            f(
                "gate_schema",
                s(SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA),
            ),
            f(
                "gate_reason",
                s("authorized_provider_export_public_only_audited"),
            ),
            f("method", s("provider.context_export")),
            f("profile", s("provider_minimal")),
            f("request_id", V::U64(u64::from(request_id))),
            f("destination", s(destination)),
            f("trust_state", s(trust_state)),
            f("budget_tokens", V::U64(budget_tokens)),
            f("packet_estimated_tokens", V::U64(packet_estimated_tokens)),
            f("packet_hash", V::Sha256(packet_hash)),
            f("packet_record_count", V::U64(packet_record_count)),
            f("request_body_sha256", V::Sha256(request_body_sha256)),
            f(
                "request_envelope_sha256",
                V::Sha256(request_envelope_sha256),
            ),
            f("audit_binding_hash", V::Sha256(audit_binding_hash)),
            f(
                "provider_trust_evidence_hash",
                V::Sha256(provider_trust_evidence_hash),
            ),
            f("packet_all_records_public", b(true)),
            f("context_attached_to_provider_body", b(true)),
            f(
                "authorization_consumption",
                s("pending_until_durable_append_verified"),
            ),
            f("transmission_performed", b(false)),
            f("provider_write", s("authorized_not_attempted")),
            f("test_infrastructure", b(false)),
        ]),
        classification: "local_only",
        authority: "core_ledger",
        boot_id: "current_boot",
        sequence: u64::from(request_id),
        source: MemorySource::new(
            "provider.context_export",
            SCOPED_PROVIDER_EXPORT_DECISION_ID,
        ),
        evidence: vec![],
        tags: vec!["provider_export", "export_audit", "live"],
        supersedes: vec![],
        created_at_ticks: 0,
    }) {
        Ok(record) => record,
        Err(_) => {
            return LiveProviderExportAuditOutcome {
                performed: false,
                reason: "live_export_audit_record_construction_denied",
                seq: None,
                payload_sha256: None,
                frame_sha256: None,
                readback_sha256: None,
                reparse_valid: false,
            }
        }
    };
    let evidence = durable_store::append_memory_record(&record);
    LiveProviderExportAuditOutcome {
        performed: evidence.performed,
        reason: evidence.reason,
        seq: evidence.seq,
        payload_sha256: evidence.payload_sha256,
        frame_sha256: evidence.frame_sha256,
        readback_sha256: evidence.readback_sha256,
        reparse_valid: evidence.reparse_valid,
    }
}

// --- M11-3a: durable per-service Wasm import-grant audits -----------------------

const WASM_IMPORT_GRANT_DEDUPE_CAP: usize = 4;

struct WasmImportGrantDedupeEntry {
    service_id: &'static str,
    authorized_import_list_sha256: [u8; 32],
    payload_sha256: Option<[u8; 32]>,
    seq: Option<u64>,
}

static WASM_IMPORT_GRANT_DEDUPE: Mutex<Vec<WasmImportGrantDedupeEntry>> = Mutex::new(Vec::new());

pub(crate) struct WasmImportGrantAuditOutcome {
    pub(crate) dedupe: &'static str,
    pub(crate) record_id: &'static str,
    pub(crate) authorized_import_list_sha256: [u8; 32],
    pub(crate) linked_host_import_count: u64,
    pub(crate) module_imports_within_authorized_list: bool,
    pub(crate) import_grant_performed: bool,
    pub(crate) import_grant_status: &'static str,
    pub(crate) evidence: Option<durable_store::MemoryRecordAppendEvidence<'static>>,
    pub(crate) cited_payload_sha256: Option<[u8; 32]>,
    pub(crate) cited_seq: Option<u64>,
}

pub(crate) fn record_wasm_import_grant_audit(
    service_id: &'static str,
    imports: &'static [(&'static str, &'static str)],
    evidence: &wasm_runtime::EchoRunEvidence,
) -> WasmImportGrantAuditOutcome {
    let expected_hash = authorized_import_list_sha256(service_id, imports);
    let Some(record_id) = wasm_import_grant_record_id(service_id) else {
        return wasm_import_grant_ram_only_outcome(
            "service_unmapped_ram_only",
            "",
            expected_hash,
            evidence,
        );
    };

    if !wasm_import_grant_evidence_matches(evidence, imports, expected_hash) {
        return wasm_import_grant_ram_only_outcome(
            "evidence_mismatch_ram_only",
            record_id,
            expected_hash,
            evidence,
        );
    }

    {
        let table = WASM_IMPORT_GRANT_DEDUPE.lock();
        let mut idx = 0usize;
        while idx < table.len() {
            let entry = &table[idx];
            if entry.service_id == service_id
                && entry.authorized_import_list_sha256 == expected_hash
            {
                return WasmImportGrantAuditOutcome {
                    dedupe: "duplicate_ram_only",
                    record_id,
                    authorized_import_list_sha256: expected_hash,
                    linked_host_import_count: evidence.linked_host_import_count,
                    module_imports_within_authorized_list: evidence
                        .module_imports_within_authorized_list,
                    import_grant_performed: evidence.import_grant_performed,
                    import_grant_status: evidence.import_grant_status,
                    evidence: None,
                    cited_payload_sha256: entry.payload_sha256,
                    cited_seq: entry.seq,
                };
            }
            idx += 1;
        }
        if table.len() >= WASM_IMPORT_GRANT_DEDUPE_CAP {
            return wasm_import_grant_ram_only_outcome(
                "dedupe_table_full_ram_only",
                record_id,
                expected_hash,
                evidence,
            );
        }
    }

    let record = match wasm_import_grant_record(record_id, service_id, imports, evidence) {
        Ok(record) => record,
        Err(_) => {
            return wasm_import_grant_ram_only_outcome(
                "record_construction_denied_ram_only",
                record_id,
                expected_hash,
                evidence,
            );
        }
    };

    let append_evidence = durable_store::append_memory_record(&record);
    if !append_evidence.performed {
        return WasmImportGrantAuditOutcome {
            dedupe: "append_denied_ram_only",
            record_id,
            authorized_import_list_sha256: expected_hash,
            linked_host_import_count: evidence.linked_host_import_count,
            module_imports_within_authorized_list: evidence.module_imports_within_authorized_list,
            import_grant_performed: evidence.import_grant_performed,
            import_grant_status: evidence.import_grant_status,
            evidence: Some(append_evidence),
            cited_payload_sha256: None,
            cited_seq: None,
        };
    }

    let cited_payload_sha256 = append_evidence.payload_sha256;
    let cited_seq = append_evidence.seq;
    {
        let mut table = WASM_IMPORT_GRANT_DEDUPE.lock();
        if table.len() < WASM_IMPORT_GRANT_DEDUPE_CAP {
            table.push(WasmImportGrantDedupeEntry {
                service_id,
                authorized_import_list_sha256: expected_hash,
                payload_sha256: cited_payload_sha256,
                seq: cited_seq,
            });
        }
    }

    WasmImportGrantAuditOutcome {
        dedupe: "first_grant_appended",
        record_id,
        authorized_import_list_sha256: expected_hash,
        linked_host_import_count: evidence.linked_host_import_count,
        module_imports_within_authorized_list: evidence.module_imports_within_authorized_list,
        import_grant_performed: evidence.import_grant_performed,
        import_grant_status: evidence.import_grant_status,
        evidence: Some(append_evidence),
        cited_payload_sha256,
        cited_seq,
    }
}

fn wasm_import_grant_record_id(service_id: &str) -> Option<&'static str> {
    match service_id {
        "svc.demo.echo" => {
            Some("mem.capability_grant.wasm_import_surface.svc_demo_echo.current_boot.v0")
        }
        "svc.demo.bufecho" => {
            Some("mem.capability_grant.wasm_import_surface.svc_demo_bufecho.current_boot.v0")
        }
        "svc.dev.granted_candidate" => Some(
            "mem.capability_grant.wasm_import_surface.svc_dev_granted_candidate.current_boot.v0",
        ),
        _ => None,
    }
}

fn wasm_import_grant_evidence_matches(
    evidence: &wasm_runtime::EchoRunEvidence,
    imports: &[(&str, &str)],
    expected_hash: [u8; 32],
) -> bool {
    evidence.import_grant_performed
        && evidence.import_grant_status == "import_grant_authorized"
        && evidence.module_imports_within_authorized_list
        && evidence.authorized_import_count == imports.len() as u64
        && evidence.authorized_import_list_sha256 == expected_hash
}

fn wasm_import_grant_ram_only_outcome(
    dedupe: &'static str,
    record_id: &'static str,
    authorized_import_list_sha256: [u8; 32],
    evidence: &wasm_runtime::EchoRunEvidence,
) -> WasmImportGrantAuditOutcome {
    WasmImportGrantAuditOutcome {
        dedupe,
        record_id,
        authorized_import_list_sha256,
        linked_host_import_count: evidence.linked_host_import_count,
        module_imports_within_authorized_list: evidence.module_imports_within_authorized_list,
        import_grant_performed: evidence.import_grant_performed,
        import_grant_status: evidence.import_grant_status,
        evidence: None,
        cited_payload_sha256: None,
        cited_seq: None,
    }
}

fn wasm_import_grant_record(
    record_id: &'static str,
    service_id: &'static str,
    imports: &'static [(&'static str, &'static str)],
    evidence: &wasm_runtime::EchoRunEvidence,
) -> Result<MemoryRecord<'static>, MemoryRecordError> {
    MemoryRecord::new(MemoryRecordInput {
        id: record_id,
        kind: "capability_grant",
        entity: service_id,
        predicate: "wasm_import_surface_granted",
        value: wasm_import_grant_value(service_id, imports, evidence),
        classification: "local_only",
        authority: "core_ledger",
        boot_id: "current_boot",
        sequence: 0,
        source: MemorySource::new("wasm.import_grant", SCOPED_WASM_IMPORT_GRANT_DECISION_ID),
        evidence: vec![],
        tags: vec!["capability", "wasm", "import_grant"],
        supersedes: vec![],
        created_at_ticks: 0,
    })
}

fn wasm_import_grant_value(
    service_id: &'static str,
    imports: &'static [(&'static str, &'static str)],
    evidence: &wasm_runtime::EchoRunEvidence,
) -> V<'static> {
    V::Object(vec![
        f("gate", s(SCOPED_WASM_IMPORT_GRANT_DECISION_ID)),
        f("gate_schema", s(SCOPED_WASM_IMPORT_GRANT_DECISION_SCHEMA)),
        f("gate_reason", s("authorized_exact_declared_import_surface")),
        f("service_id", s(service_id)),
        f("capability_envelope", s("wasmi_linker_import_surface")),
        f("authorized_imports", wasm_imports_value(imports)),
        f("authorized_import_count", V::U64(imports.len() as u64)),
        f(
            "authorized_import_list_sha256",
            V::Sha256(evidence.authorized_import_list_sha256),
        ),
        f(
            "linked_host_import_count",
            V::U64(evidence.linked_host_import_count),
        ),
        f("module_imports_within_authorized_list", b(true)),
        f("import_grant_performed", b(true)),
        f("import_grant_status", s(evidence.import_grant_status)),
        f("transmission_performed", b(false)),
        f("provider_write", s("not_attempted")),
        f("owner_sealed", b(false)),
        f("trust_tier", s("dev_key_not_owner_sealed")),
    ])
}

fn wasm_imports_value(imports: &'static [(&'static str, &'static str)]) -> V<'static> {
    let mut values = Vec::with_capacity(imports.len());
    let mut idx = 0usize;
    while idx < imports.len() {
        values.push(V::InlineObject(vec![
            f("module", s(imports[idx].0)),
            f("name", s(imports[idx].1)),
        ]));
        idx += 1;
    }
    V::Array(values)
}

pub(crate) fn wasm_import_grant_audit_value(audit: &WasmImportGrantAuditOutcome) -> V<'static> {
    let evidence = audit.evidence.as_ref();
    let duplicate = audit.dedupe == "duplicate_ram_only";
    let durable_append = match evidence {
        Some(evidence) => evidence.durable_append,
        None if duplicate => "not_attempted_deduplicated",
        None => "not_attempted",
    };
    let performed = evidence.map(|evidence| evidence.performed).unwrap_or(false);
    let append_reason = match evidence {
        Some(evidence) => evidence.reason,
        None if duplicate => "deduplicated_first_audit_cited",
        None => audit.dedupe,
    };
    let payload_sha256 = match evidence {
        Some(evidence) => evidence.payload_sha256,
        None => audit.cited_payload_sha256,
    };
    let seq = match evidence {
        Some(evidence) => evidence.seq,
        None => audit.cited_seq,
    };

    V::InlineObject(vec![
        f("gate", s(SCOPED_WASM_IMPORT_GRANT_DECISION_ID)),
        f("gate_schema", s(SCOPED_WASM_IMPORT_GRANT_DECISION_SCHEMA)),
        f("gate_reason", s("authorized_exact_declared_import_surface")),
        f("dedupe", s(audit.dedupe)),
        f("record_id", s(audit.record_id)),
        f(
            "record_schema",
            s(evidence
                .map(|evidence| evidence.record_schema)
                .unwrap_or("raios.memory_record.v0")),
        ),
        f("kind", s("capability_grant")),
        f("classification", s("local_only")),
        f(
            "record_authority",
            s(evidence
                .map(|evidence| evidence.record_authority)
                .unwrap_or("core_ledger")),
        ),
        f("supersedes", V::Array(vec![])),
        f("durable_append", s(durable_append)),
        f("performed", b(performed)),
        f("append_reason", s(append_reason)),
        f("payload_sha256", record_sha_or_null(payload_sha256)),
        f(
            "frame_sha256",
            record_sha_or_null(evidence.and_then(|evidence| evidence.frame_sha256)),
        ),
        f(
            "readback_sha256",
            record_sha_or_null(evidence.and_then(|evidence| evidence.readback_sha256)),
        ),
        f(
            "reparse_valid",
            b(evidence
                .map(|evidence| evidence.reparse_valid)
                .unwrap_or(false)),
        ),
        f("seq", optional_u64(seq)),
        f(
            "tail_seq_before",
            optional_u64(evidence.and_then(|evidence| evidence.tail_seq_before)),
        ),
        f(
            "count_before",
            optional_u64(evidence.and_then(|evidence| evidence.count_before)),
        ),
        f(
            "tail_seq_after",
            optional_u64(evidence.and_then(|evidence| evidence.tail_seq_after)),
        ),
        f(
            "count_after",
            optional_u64(evidence.and_then(|evidence| evidence.count_after)),
        ),
        f(
            "authorized_import_list_sha256",
            V::Sha256(audit.authorized_import_list_sha256),
        ),
        f(
            "linked_host_import_count",
            V::U64(audit.linked_host_import_count),
        ),
        f(
            "module_imports_within_authorized_list",
            b(audit.module_imports_within_authorized_list),
        ),
        f("import_grant_performed", b(audit.import_grant_performed)),
        f("import_grant_status", s(audit.import_grant_status)),
        f(
            "owner_sealed",
            b(evidence
                .map(|evidence| evidence.owner_sealed)
                .unwrap_or(false)),
        ),
        f(
            "persistence_claimed",
            b(evidence
                .map(|evidence| evidence.persistence_claimed)
                .unwrap_or(false)),
        ),
        f(
            "trust_tier",
            s(evidence
                .map(|evidence| evidence.trust_tier)
                .unwrap_or("dev_key_not_owner_sealed")),
        ),
        f("transmission_performed", b(false)),
        f("provider_write", s("not_attempted")),
    ])
}
