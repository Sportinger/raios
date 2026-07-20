//! Typed evidence-v1 projection for the denial-only module load gate.

use alloc::vec::Vec;

use crate::{
    evidence_response::{denied, Blocked, Evidence},
    record::{Field, Value},
};

const REQUESTED_CAPABILITY: &str = "cap.module.load_ephemeral";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadGateEvidenceStatus {
    Present,
    Missing,
    Rejected,
    Verified,
    Unavailable,
    NotApplicable,
}

impl LoadGateEvidenceStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Missing => "missing",
            Self::Rejected => "rejected",
            Self::Verified => "verified",
            Self::Unavailable => "unavailable",
            Self::NotApplicable => "not_applicable",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadGateDisposition {
    Satisfied,
    Blocked,
}

pub struct LoadGatePrerequisite<'a> {
    pub status: LoadGateEvidenceStatus,
    pub status_detail: &'a str,
    pub reason: &'a str,
    pub source_event_sequence: Option<u64>,
    pub facts: Vec<Field<'a>>,
    pub disposition: LoadGateDisposition,
}

pub struct LoadGateProjectionInput<'a> {
    pub module_manifest: LoadGatePrerequisite<'a>,
    pub candidate_artifact: LoadGatePrerequisite<'a>,
    pub vm_test_report: LoadGatePrerequisite<'a>,
    pub local_attestation: LoadGatePrerequisite<'a>,
    pub local_approval: LoadGatePrerequisite<'a>,
    pub computed_capability_grant: LoadGatePrerequisite<'a>,
    pub durable_audit_record: LoadGatePrerequisite<'a>,
    pub rollback_plan: LoadGatePrerequisite<'a>,
    pub service_slot: LoadGatePrerequisite<'a>,
    pub service_slot_allocator: LoadGatePrerequisite<'a>,
    pub loader_runtime: LoadGatePrerequisite<'a>,
    pub loader: LoadGatePrerequisite<'a>,
}

pub struct LoadGateProjection<'a> {
    pub evidence: Vec<Evidence<'a>>,
    pub blocked_by: Vec<Blocked<'a>>,
    pub decision: Value<'a>,
}

struct EvidenceIdentity {
    id: &'static str,
    kind: &'static str,
}

const EVIDENCE_ORDER: [EvidenceIdentity; 12] = [
    EvidenceIdentity {
        id: "module_manifest",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "candidate_artifact",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "vm_test_report",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "local_attestation",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "local_approval",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "computed_capability_grant",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "durable_audit_record",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "rollback_plan",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "service_slot",
        kind: "retained_reference",
    },
    EvidenceIdentity {
        id: "service_slot_allocator",
        kind: "readiness",
    },
    EvidenceIdentity {
        id: "loader_runtime",
        kind: "readiness",
    },
    EvidenceIdentity {
        id: "loader",
        kind: "availability",
    },
];

pub fn project_load_gate_denial<'a>(input: LoadGateProjectionInput<'a>) -> LoadGateProjection<'a> {
    let prerequisites = [
        input.module_manifest,
        input.candidate_artifact,
        input.vm_test_report,
        input.local_attestation,
        input.local_approval,
        input.computed_capability_grant,
        input.durable_audit_record,
        input.rollback_plan,
        input.service_slot,
        input.service_slot_allocator,
        input.loader_runtime,
        input.loader,
    ];
    let mut evidence = Vec::with_capacity(EVIDENCE_ORDER.len());
    let mut blocked_by = Vec::with_capacity(EVIDENCE_ORDER.len());

    for (index, (identity, prerequisite)) in EVIDENCE_ORDER
        .iter()
        .zip(prerequisites.into_iter())
        .enumerate()
    {
        let status = prerequisite.status.as_str();
        let mut facts = Vec::with_capacity(prerequisite.facts.len() + 1);
        facts.push(Field::new(
            "status_detail",
            Value::Str(prerequisite.status_detail),
        ));
        facts.extend(prerequisite.facts);
        evidence.push(Evidence {
            id: identity.id,
            kind: identity.kind,
            status,
            reason: prerequisite.reason,
            source_event_sequence: prerequisite.source_event_sequence,
            classification: "local_only",
            facts: Value::InlineObject(facts),
        });
        if prerequisite.disposition == LoadGateDisposition::Blocked
            || index == EVIDENCE_ORDER.len() - 1
        {
            blocked_by.push(Blocked {
                evidence_id: identity.id,
                status,
                reason: prerequisite.reason,
            });
        }
    }

    let decision = denied(blocked_by[0].reason, REQUESTED_CAPABILITY, &blocked_by);
    LoadGateProjection {
        evidence,
        blocked_by,
        decision,
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::{
        evidence_response::{evidence_value, response_value, Envelope},
        record::write_json,
    };

    const IDS: [&str; 12] = [
        "module_manifest",
        "candidate_artifact",
        "vm_test_report",
        "local_attestation",
        "local_approval",
        "computed_capability_grant",
        "durable_audit_record",
        "rollback_plan",
        "service_slot",
        "service_slot_allocator",
        "loader_runtime",
        "loader",
    ];

    fn prerequisite(index: usize, blocked: bool) -> LoadGatePrerequisite<'static> {
        LoadGatePrerequisite {
            status: if blocked {
                LoadGateEvidenceStatus::Missing
            } else {
                LoadGateEvidenceStatus::Verified
            },
            status_detail: if blocked { "missing" } else { "ready" },
            reason: IDS[index],
            source_event_sequence: (index == 0).then_some(7),
            facts: vec![Field::new("index", Value::U64(index as u64))],
            disposition: if blocked {
                LoadGateDisposition::Blocked
            } else {
                LoadGateDisposition::Satisfied
            },
        }
    }

    fn input(first_blocked: usize) -> LoadGateProjectionInput<'static> {
        let mut p = (0..12).map(|index| prerequisite(index, index >= first_blocked));
        LoadGateProjectionInput {
            module_manifest: p.next().unwrap(),
            candidate_artifact: p.next().unwrap(),
            vm_test_report: p.next().unwrap(),
            local_attestation: p.next().unwrap(),
            local_approval: p.next().unwrap(),
            computed_capability_grant: p.next().unwrap(),
            durable_audit_record: p.next().unwrap(),
            rollback_plan: p.next().unwrap(),
            service_slot: p.next().unwrap(),
            service_slot_allocator: p.next().unwrap(),
            loader_runtime: p.next().unwrap(),
            loader: p.next().unwrap(),
        }
    }

    #[test]
    fn full_order_contains_all_twelve_evidence_units() {
        let projection = project_load_gate_denial(input(0));
        assert_eq!(
            projection
                .evidence
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            IDS
        );
        assert_eq!(
            projection
                .blocked_by
                .iter()
                .map(|item| item.evidence_id)
                .collect::<Vec<_>>(),
            IDS
        );
    }

    #[test]
    fn each_prerequisite_can_be_the_first_failure() {
        for first in 0..12 {
            let projection = project_load_gate_denial(input(first));
            assert_eq!(projection.blocked_by[0].evidence_id, IDS[first]);
            assert_eq!(projection.blocked_by[0].reason, IDS[first]);
        }
    }

    #[test]
    fn blocked_by_order_follows_approval_before_computed_grant() {
        let projection = project_load_gate_denial(input(0));
        assert_eq!(projection.evidence[4].id, "local_approval");
        assert_eq!(projection.evidence[5].id, "computed_capability_grant");
        assert_eq!(projection.blocked_by[4].evidence_id, "local_approval");
        assert_eq!(
            projection.blocked_by[5].evidence_id,
            "computed_capability_grant"
        );
    }

    #[test]
    fn reason_and_null_source_event_are_always_explicit() {
        let projection = project_load_gate_denial(input(0));
        let mut json = Vec::new();
        write_json(
            &evidence_value(projection.evidence.into_iter().nth(1).unwrap()),
            &mut json,
            0,
        );
        let json = core::str::from_utf8(&json).unwrap();
        assert!(json.contains("\"reason\": \"candidate_artifact\""));
        assert!(json.contains("\"source_event_id\": null"));
    }

    #[test]
    fn rendered_json_is_denied_without_grants_or_effects_in_blocker_order() {
        let projection = project_load_gate_denial(input(4));
        let envelope = Envelope {
            response_sequence: 1,
            family: "module.load_gate",
            scope: "current_boot",
            classification: "local_only",
            source_method: "module.load_ephemeral",
            event_sequence: None,
            facts: Value::InlineObject(vec![]),
            evidence: projection
                .evidence
                .into_iter()
                .map(evidence_value)
                .collect(),
            decision: projection.decision,
        };
        let mut json = Vec::new();
        write_json(&response_value(&envelope), &mut json, 0);
        let json = core::str::from_utf8(&json).unwrap();
        assert!(json.contains("\"outcome\": \"denied\""));
        assert!(json.contains("\"grants\": []"));
        assert!(json.contains("\"effects\": []"));
        assert!(json.contains("\"reason\": \"local_approval\""));
        assert!(
            json.find("\"evidence_id\": \"local_approval\"").unwrap()
                < json
                    .find("\"evidence_id\": \"computed_capability_grant\"")
                    .unwrap()
        );
    }

    #[test]
    fn loader_remains_blocking_on_an_all_satisfied_input() {
        let projection = project_load_gate_denial(input(12));
        assert_eq!(projection.blocked_by.len(), 1);
        assert_eq!(projection.blocked_by[0].evidence_id, "loader");
    }
}
