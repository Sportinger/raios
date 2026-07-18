//! Typed evidence-v1 projections for memory reads and mutation decisions.

use alloc::{vec, vec::Vec};

use crate::{
    evidence_response::{denied, observed, Blocked, Evidence},
    memory_record::{Classification as DurableClassification, MemoryKind},
    record::{Field, Value},
    record_table::{decision_value, EvaluatedDecision, GrantDecision, GrantProof},
    scoped_rollback_apply::ScopedRollbackAuthorizedAppendProof,
};

const MUTATION_CAPABILITY: &str = "cap.memory.mutate";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryClassification {
    Public,
    LocalOnly,
    Secret,
    Unknown,
}

impl MemoryClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::LocalOnly => "local_only",
            Self::Secret => "secret",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRecordKind {
    Identity,
    SystemSnapshot,
    CapabilityInventory,
    ServiceInventory,
    ProblemList,
    Summary,
    ArchitectureDecision,
    ProviderProjection,
    Durable(MemoryKind),
}

impl MemoryRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::SystemSnapshot => "system_snapshot",
            Self::CapabilityInventory => "capability_inventory",
            Self::ServiceInventory => "service_inventory",
            Self::ProblemList => "problem_list",
            Self::Summary => "summary",
            Self::ArchitectureDecision => "architecture_decision",
            Self::ProviderProjection => "provider_projection",
            Self::Durable(kind) => kind.as_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryProfile {
    Diagnostic,
    Planning,
    ProviderMinimal,
}

impl MemoryProfile {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Diagnostic => "diagnostic",
            Self::Planning => "planning",
            Self::ProviderMinimal => "provider_minimal",
        }
    }
}

pub struct MemoryProfileEntry<'a> {
    pub profile: MemoryProfile,
    pub available: bool,
    pub local_projection: Option<&'a str>,
    pub target_tokens: u64,
    pub provider_export: &'a str,
    pub summary: &'a str,
}

pub struct MemoryProfileProjectionInput<'a> {
    pub profiles: Vec<MemoryProfileEntry<'a>>,
    pub read_methods: &'a [&'a str],
    pub mutation_policy: &'a str,
    pub provider_export_omission: Option<&'a str>,
}

pub struct MemoryProjection<'a> {
    pub facts: Value<'a>,
    pub evidence: Vec<Evidence<'a>>,
    pub decision: Value<'a>,
    pub classification: MemoryClassification,
}

pub fn project_memory_profile<'a>(input: MemoryProfileProjectionInput<'a>) -> MemoryProjection<'a> {
    let profiles = input
        .profiles
        .into_iter()
        .map(|profile| {
            Value::InlineObject(vec![
                Field::new("id", Value::Str(profile.profile.as_str())),
                Field::new("available", Value::Bool(profile.available)),
                Field::new(
                    "local_projection",
                    profile
                        .local_projection
                        .map(Value::Str)
                        .unwrap_or(Value::Null),
                ),
                Field::new("target_tokens", Value::U64(profile.target_tokens)),
                Field::new("provider_export", Value::Str(profile.provider_export)),
                Field::new("summary", Value::Str(profile.summary)),
            ])
        })
        .collect();
    let evidence = input
        .provider_export_omission
        .map(|reason| {
            vec![Evidence {
                id: "profile.provider_export_omission",
                kind: "omission",
                status: "not_applicable",
                reason,
                source_event_sequence: None,
                classification: "local_only",
                facts: Value::InlineObject(vec![Field::new(
                    "profile",
                    Value::Str("provider_minimal"),
                )]),
            }]
        })
        .unwrap_or_default();
    MemoryProjection {
        facts: Value::InlineObject(vec![
            Field::new("profiles", Value::InlineArray(profiles)),
            Field::new(
                "read_methods",
                Value::InlineArray(input.read_methods.iter().copied().map(Value::Str).collect()),
            ),
            Field::new("mutation_policy", Value::Str(input.mutation_policy)),
        ]),
        evidence,
        decision: observed("memory_profile_returned"),
        classification: MemoryClassification::LocalOnly,
    }
}

pub struct CurrentStatus<'a> {
    pub framebuffer: &'a str,
    pub entropy: &'a str,
    pub usb_xhci: &'a str,
    pub wifi: &'a str,
    pub network: &'a str,
    pub input: &'a str,
}

pub struct CurrentMemorySnapshot<'a> {
    pub snapshot_id: &'a str,
    pub status: CurrentStatus<'a>,
    pub provider_trust_state: &'a str,
    pub provider_api_key_state: &'a str,
    pub capability_posture: Value<'a>,
    pub services: Value<'a>,
    pub problems: Value<'a>,
}

pub struct IncludedSelection<'a> {
    pub identity: &'a str,
    pub policy: &'a str,
    pub current: &'a str,
    pub summaries: &'a [&'a str],
}

pub struct SelectedRecord<'a> {
    pub evidence_id: &'a str,
    pub record_id: &'a str,
    pub kind: MemoryRecordKind,
    pub authority: &'a str,
    pub classification: MemoryClassification,
    pub summary: &'a str,
    pub source_locator: &'a str,
    pub source_event_sequence: Option<u64>,
}

pub struct DurableRecordView<'a> {
    pub evidence_id: &'a str,
    pub record_id: &'a str,
    pub kind: MemoryRecordKind,
    pub entity: &'a str,
    pub predicate: &'a str,
    pub classification: MemoryClassification,
    pub authority: &'a str,
    pub scope: &'a str,
    pub exportable: bool,
    pub source_event_sequence: Option<u64>,
}

pub struct SupersedeLink<'a> {
    pub source_id: &'a str,
    pub target_id: &'a str,
}

pub enum MemoryOmission<'a> {
    Static {
        evidence_id: &'a str,
        kind: &'a str,
        reason: &'a str,
    },
    Superseded {
        evidence_id: &'a str,
        reason: &'a str,
        ids: &'a [&'a str],
    },
    IgnoredSupersede {
        evidence_id: &'a str,
        reason: &'a str,
        links: &'a [SupersedeLink<'a>],
    },
    DanglingSupersede {
        evidence_id: &'a str,
        reason: &'a str,
        ids: &'a [&'a str],
    },
    AuditIdReused {
        evidence_id: &'a str,
        reason: &'a str,
        ids: &'a [&'a str],
    },
    OverBudget {
        evidence_id: &'a str,
        reason: &'a str,
    },
    FrameDropped {
        evidence_id: &'a str,
        reason: &'a str,
        count: u64,
    },
    LocalOnlyValue {
        evidence_id: &'a str,
        reason: &'a str,
    },
}

pub struct MemoryContextInput<'a> {
    pub purpose: &'a str,
    pub profile: MemoryProfile,
    pub provider_export: &'a str,
    pub source_schemas: &'a [&'a str],
    pub target_tokens: u64,
    pub estimated_tokens: u64,
    pub authority_order: &'a [&'a str],
    pub included: IncludedSelection<'a>,
    pub current: CurrentMemorySnapshot<'a>,
    pub provider_projection: Option<EmbeddedProviderProjection<'a>>,
    pub records: Vec<SelectedRecord<'a>>,
    pub durable_records: Vec<DurableRecordView<'a>>,
    pub omissions: Vec<MemoryOmission<'a>>,
}

pub struct EmbeddedProviderProjection<'a> {
    pub classification: MemoryClassification,
    pub value: Value<'a>,
}

pub fn project_memory_context<'a>(input: MemoryContextInput<'a>) -> MemoryProjection<'a> {
    let mut classification = MemoryClassification::Public;
    let mut evidence = Vec::with_capacity(
        input.records.len() + input.durable_records.len() + input.omissions.len(),
    );
    for record in input.records {
        classification = classification.max(record.classification);
        let rejected = record.classification == MemoryClassification::Unknown;
        evidence.push(Evidence {
            id: record.evidence_id,
            kind: record.kind.as_str(),
            status: if rejected { "rejected" } else { "present" },
            reason: if rejected {
                "unknown_classification"
            } else {
                "selected_for_context"
            },
            source_event_sequence: record.source_event_sequence,
            classification: record.classification.as_str(),
            facts: checked_facts(vec![
                Field::new("record_id", Value::Str(record.record_id)),
                Field::new("authority", Value::Str(record.authority)),
                Field::new("summary", Value::Str(record.summary)),
                Field::new("source_locator", Value::Str(record.source_locator)),
            ]),
        });
    }
    for record in input.durable_records {
        classification = classification.max(record.classification);
        let rejected = matches!(
            record.classification,
            MemoryClassification::Secret | MemoryClassification::Unknown
        );
        evidence.push(Evidence {
            id: record.evidence_id,
            kind: record.kind.as_str(),
            status: if rejected { "rejected" } else { "present" },
            reason: if record.classification == MemoryClassification::Secret {
                "secret_never_durable"
            } else if rejected {
                "unknown_classification"
            } else {
                "selected_for_context"
            },
            source_event_sequence: record.source_event_sequence,
            classification: record.classification.as_str(),
            facts: checked_facts(vec![
                Field::new("record_id", Value::Str(record.record_id)),
                Field::new("entity", Value::Str(record.entity)),
                Field::new("predicate", Value::Str(record.predicate)),
                Field::new("authority", Value::Str(record.authority)),
                Field::new("scope", Value::Str(record.scope)),
                Field::new("exportable", Value::Bool(record.exportable)),
            ]),
        });
    }
    evidence.extend(input.omissions.into_iter().map(project_omission));

    let current = input.current;
    let status = current.status;
    let facts = vec![
        Field::new("purpose", Value::Str(input.purpose)),
        Field::new("profile", Value::Str(input.profile.as_str())),
        Field::new("provider_export", Value::Str(input.provider_export)),
        Field::new("source_schemas", strings(input.source_schemas)),
        Field::new(
            "budget",
            Value::InlineObject(vec![
                Field::new("target_tokens", Value::U64(input.target_tokens)),
                Field::new("estimated_tokens", Value::U64(input.estimated_tokens)),
            ]),
        ),
        Field::new("authority_order", strings(input.authority_order)),
        Field::new(
            "included",
            Value::InlineObject(vec![
                Field::new("identity", Value::Str(input.included.identity)),
                Field::new("policy", Value::Str(input.included.policy)),
                Field::new("current", Value::Str(input.included.current)),
                Field::new("summaries", strings(input.included.summaries)),
            ]),
        ),
        Field::new(
            "current",
            Value::InlineObject(vec![
                Field::new("snapshot_id", Value::Str(current.snapshot_id)),
                Field::new(
                    "status",
                    Value::InlineObject(vec![
                        Field::new("framebuffer", Value::Str(status.framebuffer)),
                        Field::new("entropy", Value::Str(status.entropy)),
                        Field::new("usb_xhci", Value::Str(status.usb_xhci)),
                        Field::new("wifi", Value::Str(status.wifi)),
                        Field::new("network", Value::Str(status.network)),
                        Field::new("input", Value::Str(status.input)),
                    ]),
                ),
                Field::new(
                    "provider_trust_state",
                    Value::Str(current.provider_trust_state),
                ),
                Field::new(
                    "provider_api_key_state",
                    Value::Str(current.provider_api_key_state),
                ),
                Field::new("capability_posture", current.capability_posture),
                Field::new("services", current.services),
                Field::new("problems", current.problems),
            ]),
        ),
    ];
    // Check OUR fields, then attach the provider projection verbatim. Its internals are
    // P4-8's (M1: embed unchanged, never re-derive), and one of its FACT keys is legitimately
    // named `blocked_by` — a description of what blocks the export, not a decision. Recursing
    // the reserved-key check into it made memory police provider's field names, and panicked
    // the kernel over a name memory does not own. The reserved-key rule guards THIS response's
    // decision keys; it is not a global namespace claim over every value we carry.
    let mut facts = match checked_facts(facts) {
        Value::InlineObject(fields) => fields,
        _ => unreachable!("checked_facts returns an InlineObject"),
    };
    if let Some(provider_projection) = input.provider_projection {
        classification = classification.max(provider_projection.classification);
        facts.push(Field::new("provider_projection", provider_projection.value));
    }
    MemoryProjection {
        facts: Value::InlineObject(facts),
        evidence,
        decision: observed("memory_context_returned"),
        classification,
    }
}

fn project_omission(omission: MemoryOmission<'_>) -> Evidence<'_> {
    let (id, kind, status, reason, facts) = match omission {
        MemoryOmission::Static {
            evidence_id,
            kind,
            reason,
        } => (
            evidence_id,
            kind,
            "not_applicable",
            reason,
            vec![Field::new("kind", Value::Str(kind))],
        ),
        MemoryOmission::Superseded {
            evidence_id,
            reason,
            ids,
        } => (
            evidence_id,
            "durable_superseded",
            "not_applicable",
            reason,
            vec![Field::new("ids", strings(ids))],
        ),
        MemoryOmission::IgnoredSupersede {
            evidence_id,
            reason,
            links,
        } => (
            evidence_id,
            "durable_r1_ignored_supersede",
            "rejected",
            reason,
            vec![Field::new(
                "links",
                Value::InlineArray(
                    links
                        .iter()
                        .map(|link| {
                            Value::InlineObject(vec![
                                Field::new("source_id", Value::Str(link.source_id)),
                                Field::new("target_id", Value::Str(link.target_id)),
                            ])
                        })
                        .collect(),
                ),
            )],
        ),
        MemoryOmission::DanglingSupersede {
            evidence_id,
            reason,
            ids,
        } => (
            evidence_id,
            "durable_dangling_supersede",
            "rejected",
            reason,
            vec![Field::new("ids", strings(ids))],
        ),
        MemoryOmission::AuditIdReused {
            evidence_id,
            reason,
            ids,
        } => (
            evidence_id,
            "durable_audit_id_reused",
            "rejected",
            reason,
            vec![Field::new("ids", strings(ids))],
        ),
        MemoryOmission::OverBudget {
            evidence_id,
            reason,
        } => (
            evidence_id,
            "durable_records_over_budget",
            "not_applicable",
            reason,
            vec![],
        ),
        MemoryOmission::FrameDropped {
            evidence_id,
            reason,
            count,
        } => (
            evidence_id,
            "durable_frame_dropped",
            "rejected",
            reason,
            vec![Field::new("count", Value::U64(count))],
        ),
        MemoryOmission::LocalOnlyValue {
            evidence_id,
            reason,
        } => (
            evidence_id,
            "durable_local_only_value",
            "not_applicable",
            reason,
            vec![],
        ),
    };
    Evidence {
        id,
        kind,
        status,
        reason,
        source_event_sequence: None,
        classification: "local_only",
        facts: checked_facts(facts),
    }
}

pub struct QueryCandidate<'a> {
    pub evidence_id: &'a str,
    pub record_id: &'a str,
    pub kind: MemoryRecordKind,
    pub classification: MemoryClassification,
    pub summary: &'a str,
}

pub struct MemoryQueryInput<'a> {
    pub query: &'a str,
    pub candidates: Vec<QueryCandidate<'a>>,
    pub semantic_index: &'a str,
}

pub fn project_memory_query<'a>(input: MemoryQueryInput<'a>) -> MemoryProjection<'a> {
    let mut classification = MemoryClassification::Public;
    let evidence = input
        .candidates
        .into_iter()
        .map(|record| {
            classification = classification.max(record.classification);
            let rejected = record.classification == MemoryClassification::Unknown;
            Evidence {
                id: record.evidence_id,
                kind: "locator",
                status: if rejected { "rejected" } else { "located" },
                reason: if rejected {
                    "unknown_classification"
                } else {
                    "query_candidate_located"
                },
                source_event_sequence: None,
                classification: record.classification.as_str(),
                facts: checked_facts(vec![
                    Field::new("record_id", Value::Str(record.record_id)),
                    Field::new("kind", Value::Str(record.kind.as_str())),
                    Field::new("summary", Value::Str(record.summary)),
                ]),
            }
        })
        .collect();
    MemoryProjection {
        facts: checked_facts(vec![
            Field::new("query", Value::Str(input.query)),
            Field::new("semantic_index", Value::Str(input.semantic_index)),
            Field::new("locator_only", Value::Bool(true)),
        ]),
        evidence,
        decision: observed("memory_query_returned"),
        classification,
    }
}

pub struct TraceHit<'a> {
    pub evidence_id: &'a str,
    pub record_id: &'a str,
    pub found: bool,
    pub classification: MemoryClassification,
    pub source_method: Option<&'a str>,
    pub source_locator: Option<&'a str>,
    pub reason: &'a str,
}

pub struct MemoryTraceInput<'a> {
    pub requested_id: Option<&'a str>,
    pub hits: Vec<TraceHit<'a>>,
}

pub fn project_memory_trace<'a>(input: MemoryTraceInput<'a>) -> MemoryProjection<'a> {
    let mut classification = MemoryClassification::Public;
    let evidence = input
        .hits
        .into_iter()
        .map(|hit| {
            classification = classification.max(hit.classification);
            let unknown = hit.classification == MemoryClassification::Unknown;
            Evidence {
                id: hit.evidence_id,
                kind: "locator",
                status: if unknown {
                    "rejected"
                } else if hit.found {
                    "located"
                } else {
                    "missing"
                },
                reason: if unknown {
                    "unknown_classification"
                } else {
                    hit.reason
                },
                source_event_sequence: None,
                classification: hit.classification.as_str(),
                facts: checked_facts(vec![
                    Field::new("record_id", Value::Str(hit.record_id)),
                    Field::new(
                        "source_method",
                        hit.source_method.map(Value::Str).unwrap_or(Value::Null),
                    ),
                    Field::new(
                        "source_locator",
                        hit.source_locator.map(Value::Str).unwrap_or(Value::Null),
                    ),
                ]),
            }
        })
        .collect();
    MemoryProjection {
        facts: checked_facts(vec![Field::new(
            "requested_id",
            input.requested_id.map(Value::Str).unwrap_or(Value::Null),
        )]),
        evidence,
        decision: observed("memory_trace_returned"),
        classification,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MutationPrerequisiteStatus {
    Present,
    Missing,
    Rejected,
    Unavailable,
}

impl MutationPrerequisiteStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Missing => "missing",
            Self::Rejected => "rejected",
            Self::Unavailable => "unavailable",
        }
    }
    const fn blocked(self) -> bool {
        !matches!(self, Self::Present)
    }
}

#[derive(Clone, Copy)]
pub struct MutationPrerequisite<'a> {
    pub status: MutationPrerequisiteStatus,
    pub reason: &'a str,
}

pub struct MemoryMutationInput<'a> {
    pub audit_record: MutationPrerequisite<'a>,
    pub policy_ledger: MutationPrerequisite<'a>,
    pub source_retention: MutationPrerequisite<'a>,
    pub redaction_transaction: MutationPrerequisite<'a>,
    pub memory_persistence: MutationPrerequisite<'a>,
    pub rollback_plan: MutationPrerequisite<'a>,
}

pub struct MemoryMutationDenial<'a> {
    pub evidence: Vec<Evidence<'a>>,
    pub blocked_by: Vec<Blocked<'a>>,
    pub decision: Value<'a>,
}

pub fn evaluate_memory_mutation<'a>(input: MemoryMutationInput<'a>) -> MemoryMutationDenial<'a> {
    const IDS: [&str; 6] = [
        "required.00000000",
        "required.00000001",
        "required.00000002",
        "required.00000003",
        "required.00000004",
        "required.00000005",
    ];
    const NAMES: [&str; 6] = [
        "raios.audit_record.v0",
        "policy_ledger",
        "source_retention",
        "redaction_transaction",
        "raios.memory_persistence.v0",
        "rollback_plan",
    ];
    let prerequisites = [
        input.audit_record,
        input.policy_ledger,
        input.source_retention,
        input.redaction_transaction,
        input.memory_persistence,
        input.rollback_plan,
    ];
    let mut evidence = Vec::with_capacity(6);
    let mut blocked_by = Vec::with_capacity(6);
    for ((id, name), prerequisite) in IDS.into_iter().zip(NAMES).zip(prerequisites) {
        let status = prerequisite.status.as_str();
        evidence.push(Evidence {
            id,
            kind: "prerequisite",
            status,
            reason: prerequisite.reason,
            source_event_sequence: None,
            classification: "local_only",
            facts: checked_facts(vec![Field::new("required", Value::Str(name))]),
        });
        if prerequisite.status.blocked() {
            blocked_by.push(Blocked {
                evidence_id: id,
                status,
                reason: prerequisite.reason,
            });
        }
    }
    if blocked_by.is_empty() {
        blocked_by.push(Blocked {
            evidence_id: IDS[5],
            status: "rejected",
            reason: "memory_mutation_not_authorized",
        });
    }
    let decision = denied(blocked_by[0].reason, MUTATION_CAPABILITY, &blocked_by);
    MemoryMutationDenial {
        evidence,
        blocked_by,
        decision,
    }
}

pub struct AuthorizedAppendProjectionInput<'a> {
    pub status_detail: &'a str,
    pub authority: &'a str,
    pub record_id: &'a str,
    pub kind: MemoryRecordKind,
    pub classification: DurableClassification,
    pub record_authority: &'a str,
    pub record_schema: &'a str,
    pub region_marker: &'a str,
    pub target_id: &'a str,
    pub sequence: u64,
    pub write_offset: u64,
    pub frame_len: u64,
    pub payload_sha256: [u8; 32],
    pub supersedes: &'a [&'a str],
    pub frame_sha256: [u8; 32],
    pub readback_sha256: [u8; 32],
    pub reparse_valid: bool,
    pub tail_seq_before: u64,
    pub count_before: u64,
    pub tail_seq_after: u64,
    pub count_after: u64,
    pub owner_sealed: bool,
    pub persistence_claimed: bool,
    pub trust_tier: &'a str,
}

pub fn project_authorized_append<'a>(
    proof: &'a ScopedRollbackAuthorizedAppendProof,
    input: AuthorizedAppendProjectionInput<'a>,
) -> MemoryProjection<'a> {
    let classification = match input.classification {
        DurableClassification::Public => MemoryClassification::Public,
        DurableClassification::LocalOnly => MemoryClassification::LocalOnly,
    };
    let append = Evidence {
        id: "durable_append",
        kind: "write_evidence",
        status: "verified",
        reason: "authorized_memory_record_append_readback_reparse_verified",
        source_event_sequence: None,
        classification: input.classification.as_str(),
        facts: checked_facts(vec![
            Field::new("status_detail", Value::Str(input.status_detail)),
            Field::new("performed", Value::Bool(true)),
            Field::new("authority", Value::Str(input.authority)),
            Field::new("region_marker", Value::Str(input.region_marker)),
            Field::new("target_id", Value::Str(input.target_id)),
            Field::new("seq", Value::U64(input.sequence)),
            Field::new("write_offset", Value::U64(input.write_offset)),
            Field::new("frame_len", Value::U64(input.frame_len)),
            Field::new("frame_sha256", Value::Sha256(input.frame_sha256)),
            Field::new("readback_sha256", Value::Sha256(input.readback_sha256)),
            Field::new("reparse_valid", Value::Bool(input.reparse_valid)),
            Field::new("trust_tier", Value::Str(input.trust_tier)),
        ]),
    };
    let record = Evidence {
        id: "memory_record",
        kind: input.kind.as_str(),
        status: "verified",
        reason: "canonical_record_payload_verified",
        source_event_sequence: None,
        classification: input.classification.as_str(),
        facts: checked_facts(vec![
            Field::new("record_id", Value::Str(input.record_id)),
            Field::new("authority", Value::Str(input.record_authority)),
            Field::new("record_schema", Value::Str(input.record_schema)),
            Field::new("payload_sha256", Value::Sha256(input.payload_sha256)),
            Field::new("supersedes", strings(input.supersedes)),
        ]),
    };
    let decision = decision_value(&EvaluatedDecision::Granted(GrantDecision::new(
        GrantProof::new(),
        "authorized_lba1_transaction_append_readback_and_inspection_verified",
        proof.requested_capability(),
        proof.grants(),
        proof.effects(),
    )));
    MemoryProjection {
        facts: checked_facts(vec![
            Field::new(
                "store_before",
                Value::InlineObject(vec![
                    Field::new("tail_seq", Value::U64(input.tail_seq_before)),
                    Field::new("count", Value::U64(input.count_before)),
                ]),
            ),
            Field::new(
                "store_after",
                Value::InlineObject(vec![
                    Field::new("tail_seq", Value::U64(input.tail_seq_after)),
                    Field::new("count", Value::U64(input.count_after)),
                ]),
            ),
            Field::new(
                "posture",
                Value::InlineObject(vec![
                    Field::new("owner_sealed", Value::Bool(input.owner_sealed)),
                    Field::new(
                        "persistence_claimed",
                        Value::Bool(input.persistence_claimed),
                    ),
                ]),
            ),
        ]),
        evidence: vec![append, record],
        decision,
        classification,
    }
}

fn strings<'a>(values: &'a [&'a str]) -> Value<'a> {
    Value::InlineArray(values.iter().copied().map(Value::Str).collect())
}

fn checked_facts(fields: Vec<Field<'_>>) -> Value<'_> {
    fn check(value: &Value<'_>) {
        match value {
            Value::InlineObject(fields) | Value::Object(fields) => {
                for field in fields {
                    assert!(
                        !matches!(field.key, "outcome" | "grants" | "effects" | "blocked_by")
                            && !field.key.starts_with("authorizes_"),
                        "decision key is reserved: {}",
                        field.key
                    );
                    check(&field.value);
                }
            }
            Value::InlineArray(values) | Value::Array(values) => {
                for value in values {
                    check(value);
                }
            }
            _ => {}
        }
    }
    let value = Value::InlineObject(fields);
    check(&value);
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        record::write_json,
        scoped_rollback_apply::{
            evaluate_scoped_rollback_authorized_append, ScopedRollbackAuthorizedAppendInput,
        },
    };
    use alloc::{format, string::String, vec};

    fn render(value: &Value<'_>) -> String {
        let mut bytes = Vec::new();
        write_json(value, &mut bytes, 0);
        String::from_utf8(bytes).unwrap()
    }
    fn prerequisite(
        status: MutationPrerequisiteStatus,
        reason: &'static str,
    ) -> MutationPrerequisite<'static> {
        MutationPrerequisite { status, reason }
    }
    fn mutation(first: usize) -> MemoryMutationInput<'static> {
        let mut p = [prerequisite(MutationPrerequisiteStatus::Present, "present"); 6];
        for item in &mut p[first..] {
            *item = prerequisite(MutationPrerequisiteStatus::Missing, "missing");
        }
        MemoryMutationInput {
            audit_record: p[0],
            policy_ledger: p[1],
            source_retention: p[2],
            redaction_transaction: p[3],
            memory_persistence: p[4],
            rollback_plan: p[5],
        }
    }

    #[test]
    fn mutation_order_and_first_failure_are_evaluator_owned() {
        let expected = [
            "required.00000000",
            "required.00000001",
            "required.00000002",
            "required.00000003",
            "required.00000004",
            "required.00000005",
        ];
        for first in 0..6 {
            let result = evaluate_memory_mutation(mutation(first));
            assert_eq!(
                result.evidence.iter().map(|e| e.id).collect::<Vec<_>>(),
                expected
            );
            assert_eq!(result.blocked_by[0].evidence_id, expected[first]);
            assert!(render(&result.decision).contains("\"reason\": \"missing\""));
        }
    }

    #[test]
    fn mutation_evaluator_cannot_authorize_even_when_all_are_present() {
        let result = evaluate_memory_mutation(mutation(6));
        let json = render(&result.decision);
        assert!(json.contains("\"outcome\": \"denied\""));
        assert!(json.contains("\"grants\": []"));
        assert!(json.contains("\"effects\": []"));
    }

    fn valid_gate() -> ScopedRollbackAuthorizedAppendInput {
        let h = |n| Some([n; 32]);
        ScopedRollbackAuthorizedAppendInput {
            requested_capability: "cap.rollback.apply",
            effects: &["durable_transaction_append"],
            scope_decision_authorized: true,
            scope_decision_hash: h(1),
            target_start_lba: Some(1),
            target_lba_count: Some(1),
            target_byte_count: Some(512),
            append_record_hash: h(2),
            sector_plan_hash: h(3),
            write_readback_hash: h(4),
            inspection_hash: h(5),
            write_readback_source_sector_plan_hash: h(3),
            inspection_source_sector_plan_hash: h(3),
            inspection_source_write_readback_hash: h(4),
            sector_plan_sector_image_hash: h(6),
            planned_sector_image_hash: h(6),
            readback_sector_image_hash: h(6),
            expected_sector_image_hash: h(6),
            inspected_sector_image_hash: h(6),
            append_record_audit_record_image_hash: h(7),
            inspected_audit_record_image_hash: h(7),
            append_record_rollback_transaction_image_hash: h(8),
            inspected_rollback_transaction_image_hash: h(8),
            audit_record_offset: Some(0),
            audit_record_byte_length: Some(200),
            rollback_transaction_offset: Some(200),
            rollback_transaction_byte_length: Some(280),
            padding_offset: Some(480),
            padding_byte_length: Some(32),
            write_attempted: true,
            write_completed: true,
            readback_completed: true,
            readback_matches_planned_image: true,
            inspection_read_attempted: true,
            inspection_read_completed: true,
            sector_hash_verified: true,
            audit_record_hash_verified: true,
            rollback_transaction_hash_verified: true,
            offsets_verified: true,
            padding_zeroed: true,
            target_span_verified: true,
            inspection_verified: true,
        }
    }
    fn append_input() -> AuthorizedAppendProjectionInput<'static> {
        AuthorizedAppendProjectionInput {
            status_detail: "performed",
            authority: "local_policy",
            record_id: "mem.1",
            kind: MemoryRecordKind::Durable(MemoryKind::Observation),
            classification: DurableClassification::LocalOnly,
            record_authority: "system",
            record_schema: "raios.memory_record.v0",
            region_marker: "RAIOS_AUDITRB_V0",
            target_id: "lba1",
            sequence: 1,
            write_offset: 16,
            frame_len: 128,
            payload_sha256: [0x5a; 32],
            supersedes: &["old.2", "old.1"],
            frame_sha256: [0x6b; 32],
            readback_sha256: [0x6b; 32],
            reparse_valid: true,
            tail_seq_before: 0,
            count_before: 0,
            tail_seq_after: 1,
            count_after: 1,
            owner_sealed: true,
            persistence_claimed: true,
            trust_tier: "local_durable",
        }
    }

    #[test]
    fn real_append_is_granted_from_proof_with_certified_names() {
        let proof = evaluate_scoped_rollback_authorized_append(&valid_gate())
            .proof()
            .unwrap();
        let json = render(&project_authorized_append(&proof, append_input()).decision);
        assert!(json.contains("\"outcome\": \"granted\""));
        assert!(!json.contains("\"outcome\": \"observed\""));
        assert!(json.contains("\"requested_capability\": \"cap.rollback.apply\""));
        assert!(json.contains("\"effects\": [\"durable_transaction_append\"]"));
    }

    #[test]
    fn denied_gate_has_no_proof_for_positive_projection() {
        let mut input = valid_gate();
        input.padding_zeroed = false;
        let decision = evaluate_scoped_rollback_authorized_append(&input);
        assert!(!decision.performed);
        assert!(decision.proof().is_none());
    }

    #[test]
    fn passed_gate_without_caller_authority_labels_has_no_proof() {
        let mut input = valid_gate();
        input.requested_capability = "";
        input.effects = &[];
        let decision = evaluate_scoped_rollback_authorized_append(&input);
        assert!(decision.performed);
        assert!(decision.proof().is_none());
    }

    #[test]
    fn each_chain_element_independently_prevents_proof() {
        let mutations: &[fn(&mut ScopedRollbackAuthorizedAppendInput)] = &[
            |i| i.scope_decision_authorized = false,
            |i| i.scope_decision_hash = None,
            |i| i.append_record_hash = None,
            |i| i.sector_plan_hash = None,
            |i| i.write_readback_hash = None,
            |i| i.inspection_hash = None,
            |i| i.target_start_lba = None,
            |i| i.target_lba_count = None,
            |i| i.target_byte_count = None,
            |i| i.write_readback_source_sector_plan_hash = None,
            |i| i.inspection_source_sector_plan_hash = None,
            |i| i.inspection_source_write_readback_hash = None,
            |i| i.sector_plan_sector_image_hash = None,
            |i| i.planned_sector_image_hash = None,
            |i| i.readback_sector_image_hash = None,
            |i| i.expected_sector_image_hash = None,
            |i| i.inspected_sector_image_hash = None,
            |i| i.append_record_audit_record_image_hash = None,
            |i| i.inspected_audit_record_image_hash = None,
            |i| i.append_record_rollback_transaction_image_hash = None,
            |i| i.inspected_rollback_transaction_image_hash = None,
            |i| i.audit_record_offset = None,
            |i| i.audit_record_byte_length = None,
            |i| i.rollback_transaction_offset = None,
            |i| i.rollback_transaction_byte_length = None,
            |i| i.padding_offset = None,
            |i| i.padding_byte_length = None,
            |i| i.write_attempted = false,
            |i| i.write_completed = false,
            |i| i.readback_completed = false,
            |i| i.readback_matches_planned_image = false,
            |i| i.inspection_read_attempted = false,
            |i| i.inspection_read_completed = false,
            |i| i.sector_hash_verified = false,
            |i| i.audit_record_hash_verified = false,
            |i| i.rollback_transaction_hash_verified = false,
            |i| i.offsets_verified = false,
            |i| i.padding_zeroed = false,
            |i| i.target_span_verified = false,
            |i| i.inspection_verified = false,
        ];
        for mutate in mutations {
            let mut input = valid_gate();
            mutate(&mut input);
            assert!(evaluate_scoped_rollback_authorized_append(&input)
                .proof()
                .is_none());
        }
    }

    fn selected(classification: MemoryClassification) -> SelectedRecord<'static> {
        SelectedRecord {
            evidence_id: "record.snapshot.current",
            record_id: "snapshot.current",
            kind: MemoryRecordKind::SystemSnapshot,
            authority: "system",
            classification,
            summary: "locator",
            source_locator: "system.snapshot",
            source_event_sequence: None,
        }
    }
    fn context(
        records: Vec<SelectedRecord<'static>>,
        provider_projection: Option<EmbeddedProviderProjection<'static>>,
    ) -> MemoryContextInput<'static> {
        MemoryContextInput {
            purpose: "diagnostic",
            profile: MemoryProfile::Diagnostic,
            provider_export: "denied",
            source_schemas: &["system.snapshot.v0"],
            target_tokens: 1000,
            estimated_tokens: 100,
            authority_order: &["facts"],
            included: IncludedSelection {
                identity: "yes",
                policy: "yes",
                current: "yes",
                summaries: &["summary"],
            },
            current: CurrentMemorySnapshot {
                snapshot_id: "snapshot.current",
                status: CurrentStatus {
                    framebuffer: "ready",
                    entropy: "ready",
                    usb_xhci: "ready",
                    wifi: "ready",
                    network: "ready",
                    input: "ready",
                },
                provider_trust_state: "local",
                provider_api_key_state: "absent",
                capability_posture: Value::Null,
                services: Value::InlineArray(vec![]),
                problems: Value::InlineArray(vec![]),
            },
            provider_projection,
            records,
            durable_records: vec![],
            omissions: vec![],
        }
    }

    #[test]
    fn unknown_classification_is_rejected_and_response_uses_maximum() {
        let projection = project_memory_context(context(
            vec![
                selected(MemoryClassification::Public),
                selected(MemoryClassification::Unknown),
            ],
            None,
        ));
        assert_eq!(projection.evidence[1].status, "rejected");
        assert_eq!(projection.evidence[1].classification, "unknown");
        assert_eq!(projection.classification, MemoryClassification::Unknown);
    }
    #[test]
    fn mixed_record_classification_is_not_flattened() {
        let projection = project_memory_context(context(
            vec![
                selected(MemoryClassification::Public),
                selected(MemoryClassification::Secret),
            ],
            None,
        ));
        assert_eq!(projection.evidence[0].classification, "public");
        assert_eq!(projection.evidence[1].classification, "secret");
        assert_eq!(projection.classification, MemoryClassification::Secret);
    }
    #[test]
    fn secret_durable_record_is_explicitly_rejected() {
        let mut input = context(vec![], None);
        input.durable_records.push(DurableRecordView {
            evidence_id: "durable_record.secret",
            record_id: "secret",
            kind: MemoryRecordKind::Durable(MemoryKind::Observation),
            entity: "svc.test",
            predicate: "state",
            classification: MemoryClassification::Secret,
            authority: "system",
            scope: "durable",
            exportable: false,
            source_event_sequence: None,
        });
        let projection = project_memory_context(input);
        assert_eq!(projection.evidence[0].status, "rejected");
        assert_eq!(projection.evidence[0].reason, "secret_never_durable");
    }
    #[test]
    fn provider_projection_value_is_embedded_unchanged() {
        let provider = Value::InlineObject(vec![Field::new("owned_hash", Value::Sha256([9; 32]))]);
        let projection = project_memory_context(context(
            vec![],
            Some(EmbeddedProviderProjection {
                classification: MemoryClassification::Public,
                value: provider.clone(),
            }),
        ));
        let Value::InlineObject(fields) = projection.facts else {
            panic!()
        };
        assert_eq!(fields.last().unwrap().value, provider);
    }
    #[test]
    fn profile_only_advertises_p4_4_reads_without_projecting_event_content() {
        let projection = project_memory_profile(MemoryProfileProjectionInput {
            profiles: vec![],
            read_methods: &["memory.recent_events", "audit.events"],
            mutation_policy: "denied",
            provider_export_omission: None,
        });
        let json = render(&projection.facts);
        assert!(json.contains("memory.recent_events"));
        assert!(json.contains("audit.events"));
        assert!(projection.evidence.is_empty());
    }
    #[test]
    fn query_candidates_and_trace_hits_are_located_never_verified() {
        let query = project_memory_query(MemoryQueryInput {
            query: "snap",
            candidates: vec![QueryCandidate {
                evidence_id: "query_locator.snapshot.current",
                record_id: "snapshot.current",
                kind: MemoryRecordKind::SystemSnapshot,
                classification: MemoryClassification::Public,
                summary: "locator",
            }],
            semantic_index: "not_implemented_locator_only",
        });
        let trace = project_memory_trace(MemoryTraceInput {
            requested_id: Some("snapshot.current"),
            hits: vec![TraceHit {
                evidence_id: "trace.snapshot.current",
                record_id: "snapshot.current",
                found: true,
                classification: MemoryClassification::LocalOnly,
                source_method: Some("system.snapshot"),
                source_locator: Some("system.snapshot"),
                reason: "trace_locator_found",
            }],
        });
        assert_eq!(query.evidence[0].status, "located");
        assert_eq!(trace.evidence[0].status, "located");
        assert!(!render(&query.decision).contains("granted"));
    }
    #[test]
    fn append_hashes_pass_through_as_sha256_values() {
        let proof = evaluate_scoped_rollback_authorized_append(&valid_gate())
            .proof()
            .unwrap();
        let projection = project_authorized_append(&proof, append_input());
        let json = render(&Value::InlineArray(
            projection
                .evidence
                .into_iter()
                .map(crate::evidence_response::evidence_value)
                .collect(),
        ));
        for digest in [[0x5a; 32], [0x6b; 32]] {
            let hex = crate::sha256_hex(&digest);
            let expected = format!("sha256:{}", core::str::from_utf8(&hex).unwrap());
            assert!(json.contains(&expected));
        }
        assert!(json.contains("\"supersedes\": [\"old.2\", \"old.1\"]"));
    }
    /// The reserved-key check guards THIS response's own facts. It does NOT reach into the
    /// embedded provider projection, and that is deliberate: the provider value is still on the
    /// PRE-v1 vocabulary (P4-8 converts it), so it legitimately still carries `outcome`,
    /// `blocked_by` and `authorizes_provider_*` as descriptive keys. An earlier version of this
    /// module recursed into it and PANICKED THE KERNEL over field names memory does not own —
    /// caught live at shadow-20260713-174237-10020. M1 is the rule: embed unchanged, never
    /// re-derive, never police. When P4-8 converts the provider internals those keys disappear
    /// on their own, because its facts stop claiming authority — which is the actual fix.
    #[test]
    fn embedded_provider_value_is_carried_verbatim_and_not_policed() {
        let pre_v1_provider = Value::InlineObject(vec![
            Field::new("outcome", Value::Str("denied")),
            Field::new("blocked_by", Value::InlineArray(vec![Value::Str("trust")])),
            Field::new("authorizes_provider_export", Value::Bool(false)),
        ]);
        let projection = project_memory_context(context(
            vec![],
            Some(EmbeddedProviderProjection {
                classification: MemoryClassification::Public,
                value: pre_v1_provider.clone(),
            }),
        ));
        let Value::InlineObject(fields) = projection.facts else {
            panic!("facts must be an inline object")
        };
        let embedded = fields.last().expect("provider projection is the last fact");
        assert_eq!(embedded.key, "provider_projection");
        assert_eq!(embedded.value, pre_v1_provider);
    }

    /// ...but memory's OWN facts are still policed. A reserved key we author is a bug.
    #[test]
    #[should_panic(expected = "decision key is reserved")]
    fn reserved_decision_keys_are_rejected_from_our_own_facts() {
        evaluate_memory_mutation(MemoryMutationInput {
            audit_record: MutationPrerequisite {
                status: MutationPrerequisiteStatus::Missing,
                reason: "missing",
            },
            ..mutation(0)
        });
        checked_facts(vec![Field::new("authorizes_export", Value::Bool(true))]);
    }
}
