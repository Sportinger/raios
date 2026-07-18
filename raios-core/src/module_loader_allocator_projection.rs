//! Typed evidence-v1 projections for loader/allocator response families.

use alloc::{vec, vec::Vec};

use crate::{
    evidence_response::{denied, observed, Blocked, Evidence},
    record::{Field, Value},
};

const REQUESTED_CAPABILITY: &str = "cap.module.load_ephemeral";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoaderAllocatorEvidenceStatus {
    Present,
    Missing,
    Rejected,
    Verified,
    Unavailable,
    NotApplicable,
}

impl LoaderAllocatorEvidenceStatus {
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
pub enum LoaderAllocatorDisposition {
    Satisfied,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoaderAllocatorEvidenceInput<'a> {
    pub status: LoaderAllocatorEvidenceStatus,
    pub status_detail: &'a str,
    pub reason: &'a str,
    pub source_event_sequence: Option<u64>,
    pub facts: Vec<Field<'a>>,
    pub disposition: LoaderAllocatorDisposition,
}

pub struct LoaderAllocatorProjection<'a> {
    pub evidence: Vec<Evidence<'a>>,
    pub blocked_by: Vec<Blocked<'a>>,
    pub decision: Value<'a>,
}

#[derive(Clone, Copy)]
struct EvidenceIdentity {
    id: &'static str,
    kind: &'static str,
}

macro_rules! named_input {
    ($name:ident { $($field:ident),+ $(,)? }) => {
        pub struct $name<'a> {
            $(pub $field: LoaderAllocatorEvidenceInput<'a>,)+
        }

        impl<'a> $name<'a> {
            fn into_items(self) -> Vec<LoaderAllocatorEvidenceInput<'a>> {
                vec![$(self.$field),+]
            }

            #[cfg(test)]
            fn from_items<I: Iterator<Item = LoaderAllocatorEvidenceInput<'a>>>(
                items: &mut I,
            ) -> Self {
                Self {
                    $($field: items.next().unwrap(),)+
                }
            }
        }
    };
}

named_input!(AllocatorProjectionInput {
    service_slot_reservation,
    service_slot_allocator_runtime,
    service_slot_registry_binding,
    service_health_state_model,
    service_unload_cleanup_plan,
    durable_audit_write,
    rollback_plan_install,
    module_loader,
    service_slot_allocator_authority,
    service_slot_allocation_intent,
    policy_decision,
    registry_write_authority,
    loader_runtime_contract,
    health_monitor_binding,
    unload_cleanup_authority,
    service_slot_allocator_authority_decision,
    service_slot_registry_write_commit_gate,
});

named_input!(LoaderIdentityProjectionInput {
    retained_module_evidence,
    service_slot_allocator_readiness,
    service_slot_allocator_runtime,
    audit_rollback_write_boundary,
    loader_identity,
});

named_input!(LoaderArtifactHashBindingProjectionInput {
    retained_module_evidence,
    service_slot_allocator_readiness,
    service_slot_allocator_runtime,
    audit_rollback_write_boundary,
    loader_identity,
    artifact_hash_binding,
});

named_input!(LoaderFactProjectionInput {
    retained_module_evidence,
    service_slot_allocator_readiness,
    service_slot_allocator_runtime,
    audit_rollback_write_boundary,
    dependency,
    fact,
});

named_input!(LoaderRuntimeProjectionInput {
    manifest_reference,
    artifact_reference,
    vm_report_reference,
    local_attestation_reference,
    local_approval_reference,
    computed_grant_reference,
    audit_rollback_reference,
    service_slot_reservation,
    service_slot_allocator_readiness,
    loader_identity,
    artifact_hash_binding,
    entrypoint_abi,
    address_space_boundary,
    memory_map_constraints,
    capability_import_table,
    service_slot_binding,
    health_state_hooks,
    rollback_hooks,
    audit_rollback_write_boundary_binding,
    execution_commit_gate,
    descriptor_intake_boundary,
    artifact_byte_intake_boundary,
    execution_authorization_boundary,
    service_registry_mutation_boundary,
    load_attempt_boundary,
    artifact_load_boundary,
    executable_mapping_boundary,
    entrypoint_transfer_boundary,
    service_start_boundary,
    service_health_binding_boundary,
    service_running_state_boundary,
    service_start_audit_boundary,
    service_unload_cleanup_boundary,
    live_load_commit_boundary,
    commit_audit_boundary,
    commit_rollback_boundary,
    commit_result_boundary,
    descriptor_acceptance_authority_boundary,
    descriptor_parser_contract_boundary,
    descriptor_parser_result_boundary,
    descriptor_schema_validation_boundary,
    descriptor_capability_validation_boundary,
    descriptor_load_plan_boundary,
    executable_load_plan_authority_boundary,
    executable_load_plan_result_boundary,
    executable_image_layout_boundary,
    executable_page_mapping_plan_boundary,
    executable_page_mapping_boundary,
    descriptor_executable_page_binding_boundary,
    executable_entrypoint_binding_boundary,
    executable_entrypoint_transfer_authorization_boundary,
    executable_entrypoint_transfer_boundary,
    executable_entrypoint_handoff_boundary,
    executable_entrypoint_invocation_boundary,
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoaderFactFamily {
    EntrypointAbi,
    AddressSpaceBoundary,
    MemoryMapConstraints,
    CapabilityImportTable,
    ServiceSlotBinding,
    HealthStateHooks,
    RollbackHooks,
    AuditRollbackWriteBoundaryBinding,
}

impl LoaderFactFamily {
    const fn dependency_id(self) -> &'static str {
        match self {
            Self::EntrypointAbi => "artifact_hash_binding",
            Self::AddressSpaceBoundary => "entrypoint_abi",
            Self::MemoryMapConstraints => "address_space_boundary",
            Self::CapabilityImportTable => "memory_map_constraints",
            Self::ServiceSlotBinding => "capability_import_table",
            Self::HealthStateHooks => "service_slot_binding",
            Self::RollbackHooks => "health_state_hooks",
            Self::AuditRollbackWriteBoundaryBinding => "rollback_hooks",
        }
    }

    const fn fact_id(self) -> &'static str {
        match self {
            Self::EntrypointAbi => "entrypoint_abi",
            Self::AddressSpaceBoundary => "address_space_boundary",
            Self::MemoryMapConstraints => "memory_map_constraints",
            Self::CapabilityImportTable => "capability_import_table",
            Self::ServiceSlotBinding => "service_slot_binding",
            Self::HealthStateHooks => "health_state_hooks",
            Self::RollbackHooks => "rollback_hooks",
            Self::AuditRollbackWriteBoundaryBinding => "audit_rollback_write_boundary_binding",
        }
    }
}

macro_rules! identities {
    ($($id:ident => $kind:literal),+ $(,)?) => {
        [$(EvidenceIdentity { id: stringify!($id), kind: $kind }),+]
    };
}

const ALLOCATOR_ORDER: [EvidenceIdentity; 17] = identities! {
    service_slot_reservation => "retained_reference",
    service_slot_allocator_runtime => "readiness",
    service_slot_registry_binding => "readiness",
    service_health_state_model => "readiness",
    service_unload_cleanup_plan => "readiness",
    durable_audit_write => "prerequisite",
    rollback_plan_install => "prerequisite",
    module_loader => "prerequisite",
    service_slot_allocator_authority => "authority_boundary",
    service_slot_allocation_intent => "intent_boundary",
    policy_decision => "authority_input",
    registry_write_authority => "authority_input",
    loader_runtime_contract => "authority_input",
    health_monitor_binding => "authority_input",
    unload_cleanup_authority => "authority_input",
    service_slot_allocator_authority_decision => "authority_decision",
    service_slot_registry_write_commit_gate => "commit_gate",
};

const LOADER_IDENTITY_ORDER: [EvidenceIdentity; 5] = identities! {
    retained_module_evidence => "retained_reference",
    service_slot_allocator_readiness => "readiness",
    service_slot_allocator_runtime => "readiness",
    audit_rollback_write_boundary => "prerequisite",
    loader_identity => "loader_fact",
};

const LOADER_ARTIFACT_HASH_BINDING_ORDER: [EvidenceIdentity; 6] = identities! {
    retained_module_evidence => "retained_reference",
    service_slot_allocator_readiness => "readiness",
    service_slot_allocator_runtime => "readiness",
    audit_rollback_write_boundary => "prerequisite",
    loader_identity => "loader_fact",
    artifact_hash_binding => "loader_fact",
};

const LOADER_RUNTIME_ORDER: [EvidenceIdentity; 54] = identities! {
    manifest_reference => "retained_reference",
    artifact_reference => "retained_reference",
    vm_report_reference => "retained_reference",
    local_attestation_reference => "retained_reference",
    local_approval_reference => "retained_reference",
    computed_grant_reference => "retained_reference",
    audit_rollback_reference => "retained_reference",
    service_slot_reservation => "retained_reference",
    service_slot_allocator_readiness => "readiness",
    loader_identity => "loader_fact",
    artifact_hash_binding => "loader_fact",
    entrypoint_abi => "loader_fact",
    address_space_boundary => "loader_fact",
    memory_map_constraints => "loader_fact",
    capability_import_table => "loader_fact",
    service_slot_binding => "loader_fact",
    health_state_hooks => "loader_fact",
    rollback_hooks => "loader_fact",
    audit_rollback_write_boundary_binding => "loader_fact",
    execution_commit_gate => "execution_boundary",
    descriptor_intake_boundary => "execution_boundary",
    artifact_byte_intake_boundary => "execution_boundary",
    execution_authorization_boundary => "execution_boundary",
    service_registry_mutation_boundary => "execution_boundary",
    load_attempt_boundary => "execution_boundary",
    artifact_load_boundary => "execution_boundary",
    executable_mapping_boundary => "execution_boundary",
    entrypoint_transfer_boundary => "execution_boundary",
    service_start_boundary => "execution_boundary",
    service_health_binding_boundary => "execution_boundary",
    service_running_state_boundary => "execution_boundary",
    service_start_audit_boundary => "execution_boundary",
    service_unload_cleanup_boundary => "execution_boundary",
    live_load_commit_boundary => "execution_boundary",
    commit_audit_boundary => "execution_boundary",
    commit_rollback_boundary => "execution_boundary",
    commit_result_boundary => "execution_boundary",
    descriptor_acceptance_authority_boundary => "execution_boundary",
    descriptor_parser_contract_boundary => "execution_boundary",
    descriptor_parser_result_boundary => "execution_boundary",
    descriptor_schema_validation_boundary => "execution_boundary",
    descriptor_capability_validation_boundary => "execution_boundary",
    descriptor_load_plan_boundary => "execution_boundary",
    executable_load_plan_authority_boundary => "execution_boundary",
    executable_load_plan_result_boundary => "execution_boundary",
    executable_image_layout_boundary => "execution_boundary",
    executable_page_mapping_plan_boundary => "execution_boundary",
    executable_page_mapping_boundary => "execution_boundary",
    descriptor_executable_page_binding_boundary => "execution_boundary",
    executable_entrypoint_binding_boundary => "execution_boundary",
    executable_entrypoint_transfer_authorization_boundary => "execution_boundary",
    executable_entrypoint_transfer_boundary => "execution_boundary",
    executable_entrypoint_handoff_boundary => "execution_boundary",
    executable_entrypoint_invocation_boundary => "execution_boundary",
};

enum ProjectionDecision<'a> {
    Denied,
    Observed(&'a str),
}

fn project<'a>(
    order: &[EvidenceIdentity],
    items: Vec<LoaderAllocatorEvidenceInput<'a>>,
    decision: ProjectionDecision<'a>,
) -> LoaderAllocatorProjection<'a> {
    assert_eq!(
        order.len(),
        items.len(),
        "projection evidence count mismatch"
    );
    let mut evidence = Vec::with_capacity(order.len());
    let mut blocked_by = Vec::with_capacity(order.len());

    for (identity, item) in order.iter().zip(items) {
        assert_fact_keys(&item.facts);
        let status = item.status.as_str();
        let blocked = item.disposition == LoaderAllocatorDisposition::Blocked;
        let mut facts = Vec::with_capacity(item.facts.len() + 1);
        facts.push(Field::new("status_detail", Value::Str(item.status_detail)));
        facts.extend(item.facts);
        evidence.push(Evidence {
            id: identity.id,
            kind: identity.kind,
            status,
            reason: item.reason,
            source_event_sequence: item.source_event_sequence,
            classification: "local_only",
            facts: Value::InlineObject(facts),
        });
        if blocked {
            blocked_by.push(Blocked {
                evidence_id: identity.id,
                status,
                reason: item.reason,
            });
        }
    }

    let decision = match decision {
        ProjectionDecision::Observed(reason) => {
            blocked_by.clear();
            observed(reason)
        }
        ProjectionDecision::Denied => {
            if blocked_by.is_empty() {
                let last = evidence.last().expect("denial projection has evidence");
                blocked_by.push(Blocked {
                    evidence_id: last.id,
                    status: last.status,
                    reason: last.reason,
                });
            }
            denied(blocked_by[0].reason, REQUESTED_CAPABILITY, &blocked_by)
        }
    };

    LoaderAllocatorProjection {
        evidence,
        blocked_by,
        decision,
    }
}

fn assert_fact_keys(fields: &[Field<'_>]) {
    for field in fields {
        assert!(
            !matches!(field.key, "outcome" | "grants" | "effects" | "blocked_by")
                && !field.key.starts_with("authorizes_"),
            "decision key is reserved: {}",
            field.key
        );
        assert_value_keys(&field.value);
    }
}

fn assert_value_keys(value: &Value<'_>) {
    match value {
        Value::InlineObject(fields) | Value::Object(fields) => assert_fact_keys(fields),
        Value::InlineArray(values) | Value::Array(values) => {
            for value in values {
                assert_value_keys(value);
            }
        }
        _ => {}
    }
}

pub fn project_allocator_denial<'a>(
    input: AllocatorProjectionInput<'a>,
) -> LoaderAllocatorProjection<'a> {
    project(
        &ALLOCATOR_ORDER,
        input.into_items(),
        ProjectionDecision::Denied,
    )
}

/// Observational view for consumers that inspect allocator readiness without
/// making the direct allocator-family load decision.
pub fn project_allocator_readiness_observation<'a>(
    input: AllocatorProjectionInput<'a>,
    reason: &'a str,
) -> LoaderAllocatorProjection<'a> {
    project(
        &ALLOCATOR_ORDER,
        input.into_items(),
        ProjectionDecision::Observed(reason),
    )
}

pub fn project_loader_identity_denial<'a>(
    input: LoaderIdentityProjectionInput<'a>,
) -> LoaderAllocatorProjection<'a> {
    project(
        &LOADER_IDENTITY_ORDER,
        input.into_items(),
        ProjectionDecision::Denied,
    )
}

pub fn project_loader_artifact_hash_binding_denial<'a>(
    input: LoaderArtifactHashBindingProjectionInput<'a>,
) -> LoaderAllocatorProjection<'a> {
    project(
        &LOADER_ARTIFACT_HASH_BINDING_ORDER,
        input.into_items(),
        ProjectionDecision::Denied,
    )
}

pub fn project_loader_fact_denial<'a>(
    family: LoaderFactFamily,
    input: LoaderFactProjectionInput<'a>,
) -> LoaderAllocatorProjection<'a> {
    let order = [
        LOADER_IDENTITY_ORDER[0],
        LOADER_IDENTITY_ORDER[1],
        LOADER_IDENTITY_ORDER[2],
        LOADER_IDENTITY_ORDER[3],
        EvidenceIdentity {
            id: family.dependency_id(),
            kind: "loader_fact",
        },
        EvidenceIdentity {
            id: family.fact_id(),
            kind: "loader_fact",
        },
    ];
    project(&order, input.into_items(), ProjectionDecision::Denied)
}

pub fn project_loader_runtime_denial<'a>(
    input: LoaderRuntimeProjectionInput<'a>,
) -> LoaderAllocatorProjection<'a> {
    project(
        &LOADER_RUNTIME_ORDER,
        input.into_items(),
        ProjectionDecision::Denied,
    )
}

/// Observational view for the runtime prerequisite chain. The separately owned
/// live granted-candidate projection is intentionally absent.
pub fn project_loader_runtime_readiness_observation<'a>(
    input: LoaderRuntimeProjectionInput<'a>,
    reason: &'a str,
) -> LoaderAllocatorProjection<'a> {
    project(
        &LOADER_RUNTIME_ORDER,
        input.into_items(),
        ProjectionDecision::Observed(reason),
    )
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec};

    use super::*;
    use crate::{
        evidence_response::{evidence_value, response_value, Envelope},
        record::write_json,
    };

    const FACT_FAMILIES: [LoaderFactFamily; 8] = [
        LoaderFactFamily::EntrypointAbi,
        LoaderFactFamily::AddressSpaceBoundary,
        LoaderFactFamily::MemoryMapConstraints,
        LoaderFactFamily::CapabilityImportTable,
        LoaderFactFamily::ServiceSlotBinding,
        LoaderFactFamily::HealthStateHooks,
        LoaderFactFamily::RollbackHooks,
        LoaderFactFamily::AuditRollbackWriteBoundaryBinding,
    ];

    fn item(index: usize, blocked: bool) -> LoaderAllocatorEvidenceInput<'static> {
        LoaderAllocatorEvidenceInput {
            status: if blocked {
                LoaderAllocatorEvidenceStatus::Missing
            } else {
                LoaderAllocatorEvidenceStatus::Verified
            },
            status_detail: if blocked { "missing" } else { "ready" },
            reason: Box::leak(index.to_string().into_boxed_str()),
            source_event_sequence: (index == 0).then_some(7),
            facts: vec![Field::new("index", Value::U64(index as u64))],
            disposition: if blocked {
                LoaderAllocatorDisposition::Blocked
            } else {
                LoaderAllocatorDisposition::Satisfied
            },
        }
    }

    fn items(
        count: usize,
        first_blocked: usize,
    ) -> impl Iterator<Item = LoaderAllocatorEvidenceInput<'static>> {
        (0..count).map(move |index| item(index, index >= first_blocked))
    }

    fn ids<'a>(projection: &'a LoaderAllocatorProjection<'a>) -> Vec<&'a str> {
        projection.evidence.iter().map(|item| item.id).collect()
    }

    fn render(
        family: &'static str,
        source_method: &'static str,
        projection: LoaderAllocatorProjection<'static>,
    ) -> String {
        let envelope = Envelope {
            response_sequence: 1,
            family,
            scope: "current_boot",
            classification: "local_only",
            source_method,
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
        String::from_utf8(json).unwrap()
    }

    #[test]
    fn allocator_full_order_matches_manifest_and_kernel_evaluator() {
        // Manifest:715-731; allocator.rs:3586-3643.
        let mut source = items(17, 0);
        let projection =
            project_allocator_denial(AllocatorProjectionInput::from_items(&mut source));
        assert_eq!(ids(&projection), ALLOCATOR_ORDER.map(|item| item.id));
    }

    #[test]
    fn loader_family_full_orders_match_manifest_and_kernel_evaluators() {
        // Manifest:740-752; identity.rs:550-578; artifact_hash_binding.rs:533-566;
        // loader_fact.rs:731-772,870-900.
        let mut source = items(5, 0);
        assert_eq!(
            ids(&project_loader_identity_denial(
                LoaderIdentityProjectionInput::from_items(&mut source)
            )),
            LOADER_IDENTITY_ORDER.map(|item| item.id)
        );
        let mut source = items(6, 0);
        assert_eq!(
            ids(&project_loader_artifact_hash_binding_denial(
                LoaderArtifactHashBindingProjectionInput::from_items(&mut source)
            )),
            LOADER_ARTIFACT_HASH_BINDING_ORDER.map(|item| item.id)
        );
        for family in FACT_FAMILIES {
            let mut source = items(6, 0);
            assert_eq!(
                ids(&project_loader_fact_denial(
                    family,
                    LoaderFactProjectionInput::from_items(&mut source)
                )),
                [
                    "retained_module_evidence",
                    "service_slot_allocator_readiness",
                    "service_slot_allocator_runtime",
                    "audit_rollback_write_boundary",
                    family.dependency_id(),
                    family.fact_id(),
                ]
            );
        }
    }

    #[test]
    fn runtime_full_order_matches_manifest_and_kernel_evaluator() {
        // Manifest:759-810; loader_runtime/eval.rs:686-1096.
        let mut source = items(54, 0);
        let projection =
            project_loader_runtime_denial(LoaderRuntimeProjectionInput::from_items(&mut source));
        assert_eq!(ids(&projection), LOADER_RUNTIME_ORDER.map(|item| item.id));
        assert!(!ids(&projection).contains(&"live_granted_load_projection"));
    }

    fn assert_each_first_failure(
        count: usize,
        expected: &[EvidenceIdentity],
        mut project_at: impl FnMut(usize) -> LoaderAllocatorProjection<'static>,
    ) {
        for first in 0..count {
            let projection = project_at(first);
            assert_eq!(projection.blocked_by[0].evidence_id, expected[first].id);
            assert_eq!(projection.blocked_by[0].reason, first.to_string());
        }
    }

    #[test]
    fn every_allocator_prerequisite_can_be_first_failure() {
        assert_each_first_failure(17, &ALLOCATOR_ORDER, |first| {
            let mut source = items(17, first);
            project_allocator_denial(AllocatorProjectionInput::from_items(&mut source))
        });
    }

    #[test]
    fn every_loader_prerequisite_can_be_first_failure() {
        assert_each_first_failure(5, &LOADER_IDENTITY_ORDER, |first| {
            let mut source = items(5, first);
            project_loader_identity_denial(LoaderIdentityProjectionInput::from_items(&mut source))
        });
        assert_each_first_failure(6, &LOADER_ARTIFACT_HASH_BINDING_ORDER, |first| {
            let mut source = items(6, first);
            project_loader_artifact_hash_binding_denial(
                LoaderArtifactHashBindingProjectionInput::from_items(&mut source),
            )
        });
        for family in FACT_FAMILIES {
            let expected = [
                LOADER_IDENTITY_ORDER[0],
                LOADER_IDENTITY_ORDER[1],
                LOADER_IDENTITY_ORDER[2],
                LOADER_IDENTITY_ORDER[3],
                EvidenceIdentity {
                    id: family.dependency_id(),
                    kind: "loader_fact",
                },
                EvidenceIdentity {
                    id: family.fact_id(),
                    kind: "loader_fact",
                },
            ];
            assert_each_first_failure(6, &expected, |first| {
                let mut source = items(6, first);
                project_loader_fact_denial(
                    family,
                    LoaderFactProjectionInput::from_items(&mut source),
                )
            });
        }
    }

    #[test]
    fn every_runtime_prerequisite_can_be_first_failure() {
        assert_each_first_failure(54, &LOADER_RUNTIME_ORDER, |first| {
            let mut source = items(54, first);
            project_loader_runtime_denial(LoaderRuntimeProjectionInput::from_items(&mut source))
        });
    }

    #[test]
    fn reason_and_null_source_event_are_always_explicit() {
        let mut source = items(5, 0);
        let projection =
            project_loader_identity_denial(LoaderIdentityProjectionInput::from_items(&mut source));
        let mut json = Vec::new();
        write_json(
            &evidence_value(projection.evidence.into_iter().nth(1).unwrap()),
            &mut json,
            0,
        );
        let json = core::str::from_utf8(&json).unwrap();
        assert!(json.contains("\"reason\": \"1\""));
        assert!(json.contains("\"source_event_id\": null"));
    }

    #[test]
    fn denied_json_has_empty_grants_effects_and_allocator_blocker_order() {
        let mut source = items(17, 10);
        let json = render(
            "module.service_slot_allocator",
            "module.service_slot_allocator",
            project_allocator_denial(AllocatorProjectionInput::from_items(&mut source)),
        );
        assert!(json.contains("\"event_id\": null"));
        assert!(json.contains("\"outcome\": \"denied\""));
        assert!(json.contains("\"grants\": []"));
        assert!(json.contains("\"effects\": []"));
        assert!(
            json.find("\"evidence_id\": \"policy_decision\"").unwrap()
                < json
                    .find("\"evidence_id\": \"registry_write_authority\"")
                    .unwrap()
        );
    }

    #[test]
    fn denied_runtime_json_preserves_blocker_order() {
        let mut source = items(54, 49);
        let json = render(
            "module.loader_runtime",
            "module.loader_runtime",
            project_loader_runtime_denial(LoaderRuntimeProjectionInput::from_items(&mut source)),
        );
        assert!(json.contains("\"outcome\": \"denied\""));
        assert!(json.contains("\"grants\": []"));
        assert!(json.contains("\"effects\": []"));
        assert!(json.find("\"evidence_id\": \"executable_entrypoint_binding_boundary\"").unwrap()
            < json.find("\"evidence_id\": \"executable_entrypoint_transfer_authorization_boundary\"").unwrap());
    }

    #[test]
    fn observed_readiness_paths_have_null_event_and_no_grants_or_effects_keys() {
        let mut allocator = items(17, 17);
        let json = render(
            "module.service_slot_allocator.readiness",
            "module.service_slot_allocator",
            project_allocator_readiness_observation(
                AllocatorProjectionInput::from_items(&mut allocator),
                "allocator_readiness_observed",
            ),
        );
        assert!(json.contains("\"event_id\": null"));
        assert!(json.contains("\"outcome\": \"observed\""));
        assert!(!json.contains("\"grants\""));
        assert!(!json.contains("\"effects\""));

        let mut runtime = items(54, 54);
        let json = render(
            "module.loader_runtime.readiness",
            "module.loader_runtime",
            project_loader_runtime_readiness_observation(
                LoaderRuntimeProjectionInput::from_items(&mut runtime),
                "loader_runtime_readiness_observed",
            ),
        );
        assert!(json.contains("\"event_id\": null"));
        assert!(!json.contains("\"grants\""));
        assert!(!json.contains("\"effects\""));
    }

    #[test]
    #[should_panic(expected = "decision key is reserved: authorizes_load")]
    fn field_tables_reject_reserved_decision_keys() {
        let mut source = items(5, 0).collect::<Vec<_>>();
        source[0]
            .facts
            .push(Field::new("authorizes_load", Value::Bool(false)));
        project_loader_identity_denial(LoaderIdentityProjectionInput::from_items(
            &mut source.into_iter(),
        ));
    }
}
