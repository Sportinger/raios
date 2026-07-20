use alloc::{vec, vec::Vec};

use super::*;
use crate::agent_protocol_support::{
    begin_error, emit_record_fields, emit_record_fields_trailing_comma, emit_record_value_fragment,
    end_error,
};
use raios_core::{
    evidence_response::evidence_value,
    hello_lifecycle_projection::{
        project_hello_lifecycle, ArtifactSnapshot, DescriptorSnapshot, EvidenceClassification,
        EvidenceMeta, EvidenceStatus, HealthSnapshot, HelloLifecycleSnapshot, InventorySnapshot,
        LifecycleAction, ServiceSlotSnapshot, ServiceStateSnapshot, StateMigrationSnapshot,
        StateTransitionSnapshot,
    },
    record::{sha256_of_json, Field, Value as V},
    scoped_rollback_apply as scoped_apply,
};

macro_rules! j {
    ($key:literal => $value:expr,) => {
        Field::new($key, $value)
    };
    ($key:literal => $value:expr) => {
        Field::new($key, $value)
    };
}

fn s(value: &'static str) -> V<'static> {
    V::Str(value)
}

fn s_opt(value: Option<&'static str>) -> V<'static> {
    match value {
        Some(value) => s(value),
        None => V::Null,
    }
}

fn b(value: bool) -> V<'static> {
    V::Bool(value)
}

fn u(value: u64) -> V<'static> {
    V::U64(value)
}

fn u_opt(value: Option<u64>) -> V<'static> {
    match value {
        Some(value) => u(value),
        None => V::Null,
    }
}

fn sha(value: [u8; 32]) -> V<'static> {
    V::Sha256(value)
}

fn sha_opt(value: Option<[u8; 32]>) -> V<'static> {
    match value {
        Some(value) => sha(value),
        None => V::Null,
    }
}

fn event_opt(value: Option<event_log::EventId>) -> V<'static> {
    match value {
        Some(value) => V::EventSequence(value.sequence()),
        None => V::Null,
    }
}

fn inline(fields: Vec<Field<'static>>) -> V<'static> {
    V::InlineObject(fields)
}

fn object(fields: Vec<Field<'static>>) -> V<'static> {
    V::Object(fields)
}

fn inline_str_array(values: &[&'static str]) -> V<'static> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < values.len() {
        out.push(s(values[idx]));
        idx += 1;
    }
    V::InlineArray(out)
}

fn emit_inline_value(value: V<'static>) {
    emit_record_value_fragment(value, 0);
}

pub(crate) fn emit_health_response(
    method: &'static str,
    capture: LifecycleCapture,
    event_id: event_log::EventId,
) {
    emit_hello_lifecycle_v1(
        method,
        "hello.health",
        LifecycleAction::Health,
        capture,
        capture.after.load_descriptor,
        event_id,
    );
}

pub(crate) fn emit_hot_swap_state_migration_denied(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
    migration: HelloStateMigrationRecord,
) {
    begin_error(method);
    emit_record_fields(
        vec![
            j!("method" => s(method)),
            j!("event_id" => event_opt(Some(event_id))),
            j!("audit_event_id" => event_opt(Some(event_id))),
            j!("code" => s("capability_denied")),
            j!("reason" => s("state_migration_would_reset_state")),
            j!("message" => s("hello hot-swap denied because the candidate would reset RAM-only service state before a rollback-capable migrator exists")),
            j!("service_id" => s(SERVICE_ID)),
            j!("target" => s(HELLO_SERVICE_DESCRIPTOR.reset_state_service_id)),
            j!("active_generation" => u(snapshot.generation)),
            j!("active_descriptor_id" => s(snapshot.load_descriptor.id)),
            j!("state" => hello_state_value(snapshot)),
            j!("state_migration" => hello_state_migration_value(Some(migration))),
            j!("required" => V::Array(vec![
                    s("state_preserving_migrator"),
                    s("raios.audit_record.v0"),
                    s("rollback_plan"),
                ]),
            ),
            j!("denied_surfaces" => object(vec![
                    j!("descriptor_mutation" => s("not_attempted")),
                    j!("state_reset" => s("denied")),
                    j!("external_artifact_load" => s("denied")),
                    j!("candidate_artifact_execution" => s("denied")),
                    j!("executable_mapping" => s("denied")),
                    j!("persistent_install" => s("denied")),
                    j!("durable_audit" => s("denied")),
                    j!("rollback_install" => s("denied")),
                    j!("broad_mutation" => s("denied")),
                ]),
            ),
        ],
        4,
    );
    end_error(method);
}

fn rollback_transaction_preflight_value(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) -> V<'static> {
    let Some(probation) = probation else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA)),
        j!("id" => s(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("status" => s(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS)),
        j!("service_id" => s(SERVICE_ID)),
        j!("requested_capability" => s(HELLO_ROLLBACK_APPLY_CAPABILITY)),
        j!("preflight_hash" => sha(hello_rollback_transaction_preflight_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_apply_hash" => sha(hello_rollback_apply_denial_hash(snapshot, probation)),
        ),
        j!("rollback_preview_hash" => sha(hello_rollback_preview_hash(snapshot, probation)),
        ),
        j!("source_probation_hash" => sha(probation.probation_hash)),
        j!("current_state_hash" => sha(hello_state_hash(snapshot.state_counter)),
        ),
        j!("current_state_counter" => u(snapshot.state_counter)),
        j!("rollback_target_descriptor_id" => s(probation.previous_descriptor_id),
        ),
        j!("rollback_target_descriptor_source_hash" => sha(probation.previous_descriptor_source_hash),
        ),
        j!("rollback_target_artifact_identity_id" => s(probation.previous_artifact_identity_id),
        ),
        j!("rollback_target_artifact_identity_hash" => sha(probation.previous_artifact_identity_hash),
        ),
        j!("rollback_target_generation" => u(probation.previous_generation),
        ),
        j!("rollback_target_state_hash" => sha(probation.previous_state_hash),
        ),
        j!("rollback_target_state_counter" => u(probation.previous_state_counter),
        ),
        j!("current_candidate_descriptor_id" => s(probation.new_descriptor_id),
        ),
        j!("current_candidate_descriptor_source_hash" => sha(probation.new_descriptor_source_hash),
        ),
        j!("current_candidate_artifact_identity_id" => s(probation.new_artifact_identity_id),
        ),
        j!("current_candidate_artifact_identity_hash" => sha(probation.new_artifact_identity_hash),
        ),
        j!("current_candidate_generation" => u(probation.new_generation)),
        j!("current_candidate_state_hash" => sha(probation.new_state_hash),
        ),
        j!("current_candidate_state_counter" => u(probation.new_state_counter),
        ),
        j!("state_migration_hash" => sha(probation.state_migration_hash)),
        j!("missing_authorities" => inline(vec![
                j!("rollback_apply_authority" => b(true)),
                j!("rollback_transaction_authority" => b(true)),
                j!("durable_audit_write_authority" => b(true)),
                j!("persistent_install_authority" => b(true)),
            ]),
        ),
        j!("side_effects" => inline(vec![
                j!("mutates_service_state" => b(false)),
                j!("applies_rollback" => b(false)),
                j!("writes_persistent_state" => b(false)),
                j!("writes_durable_audit_log" => b(false)),
                j!("writes_rollback_store" => b(false)),
                j!("installs_rollback_plan" => b(false)),
                j!("accepts_external_artifact_bytes" => b(false)),
                j!("loads_candidate_bytes" => b(false)),
                j!("maps_executable_pages" => b(false)),
                j!("provider_auto_load" => b(false)),
                j!("grants_broad_mutation" => b(false)),
            ]),
        ),
    ])
}

fn rollback_write_authority_gate_value(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) -> V<'static> {
    let Some(probation) = probation else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_SCHEMA)),
        j!("id" => s(HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("status" => s(HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_STATUS)),
        j!("service_id" => s(SERVICE_ID)),
        j!("requested_capability" => s(HELLO_ROLLBACK_APPLY_CAPABILITY)),
        j!("gate_hash" => sha(rollback_write_authority_gate_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_transaction_preflight_hash" => sha(hello_rollback_transaction_preflight_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_apply_hash" => sha(hello_rollback_apply_denial_hash(snapshot, probation)),
        ),
        j!("rollback_preview_hash" => sha(hello_rollback_preview_hash(snapshot, probation)),
        ),
        j!("source_probation_hash" => sha(probation.probation_hash)),
        j!("current_state_hash" => sha(hello_state_hash(snapshot.state_counter)),
        ),
        j!("current_state_counter" => u(snapshot.state_counter)),
        j!("rollback_target_artifact_identity_hash" => sha(probation.previous_artifact_identity_hash),
        ),
        j!("current_candidate_artifact_identity_hash" => sha(probation.new_artifact_identity_hash),
        ),
        j!("state_migration_hash" => sha(probation.state_migration_hash)),
        j!("required_schemas" => inline(vec![
                j!("audit_record" => s("raios.audit_record.v0")),
                j!("rollback_transaction" => s("raios.rollback_transaction.v0")),
            ]),
        ),
        j!("unavailable_authorities" => inline(vec![
                j!("durable_audit_write_authority" => b(true)),
                j!("rollback_store_write_authority" => b(true)),
                j!("rollback_transaction_append" => b(true)),
            ]),
        ),
        j!("side_effects" => inline(vec![
                j!("writes_durable_audit_log" => b(false)),
                j!("writes_rollback_store" => b(false)),
                j!("appends_rollback_transaction" => b(false)),
                j!("installs_rollback_plan" => b(false)),
                j!("applies_rollback" => b(false)),
            ]),
        ),
    ])
}

fn rollback_append_intent_gate_value(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) -> V<'static> {
    let Some(probation) = probation else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_APPEND_INTENT_GATE_SCHEMA)),
        j!("id" => s(HELLO_ROLLBACK_APPEND_INTENT_GATE_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("status" => s(HELLO_ROLLBACK_APPEND_INTENT_GATE_STATUS)),
        j!("service_id" => s(SERVICE_ID)),
        j!("requested_capability" => s(HELLO_ROLLBACK_APPLY_CAPABILITY)),
        j!("gate_hash" => sha(rollback_append_intent_gate_hash(snapshot, probation)),
        ),
        j!("rollback_write_authority_gate_hash" => sha(rollback_write_authority_gate_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_transaction_preflight_hash" => sha(hello_rollback_transaction_preflight_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_apply_hash" => sha(hello_rollback_apply_denial_hash(snapshot, probation)),
        ),
        j!("rollback_preview_hash" => sha(hello_rollback_preview_hash(snapshot, probation)),
        ),
        j!("source_probation_hash" => sha(probation.probation_hash)),
        j!("current_state_hash" => sha(hello_state_hash(snapshot.state_counter)),
        ),
        j!("current_state_counter" => u(snapshot.state_counter)),
        j!("rollback_target_descriptor_source_hash" => sha(probation.previous_descriptor_source_hash),
        ),
        j!("rollback_target_artifact_identity_hash" => sha(probation.previous_artifact_identity_hash),
        ),
        j!("current_candidate_descriptor_source_hash" => sha(probation.new_descriptor_source_hash),
        ),
        j!("current_candidate_artifact_identity_hash" => sha(probation.new_artifact_identity_hash),
        ),
        j!("state_migration_hash" => sha(probation.state_migration_hash)),
        j!("required_schemas" => inline(vec![
                j!("audit_record" => s("raios.audit_record.v0")),
                j!("rollback_transaction" => s("raios.rollback_transaction.v0")),
            ]),
        ),
        j!("unavailable_authorities" => inline(vec![
                j!("append_intent" => b(true)),
                j!("rollback_transaction_append" => b(true)),
                j!("durable_audit_store" => b(true)),
                j!("rollback_store" => b(true)),
            ]),
        ),
        j!("side_effects" => inline(vec![
                j!("writes_durable_audit_log" => b(false)),
                j!("writes_rollback_store" => b(false)),
                j!("appends_rollback_transaction" => b(false)),
                j!("installs_rollback_plan" => b(false)),
                j!("applies_rollback" => b(false)),
            ]),
        ),
    ])
}

fn rollback_payload_envelope_gate_value(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) -> V<'static> {
    let Some(probation) = probation else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_SCHEMA)),
        j!("id" => s(HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("status" => s(HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_STATUS)),
        j!("service_id" => s(SERVICE_ID)),
        j!("requested_capability" => s(HELLO_ROLLBACK_APPLY_CAPABILITY)),
        j!("gate_hash" => sha(rollback_payload_envelope_gate_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_append_intent_gate_hash" => sha(rollback_append_intent_gate_hash(snapshot, probation)),
        ),
        j!("rollback_write_authority_gate_hash" => sha(rollback_write_authority_gate_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_transaction_preflight_hash" => sha(hello_rollback_transaction_preflight_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_apply_hash" => sha(hello_rollback_apply_denial_hash(snapshot, probation)),
        ),
        j!("rollback_preview_hash" => sha(hello_rollback_preview_hash(snapshot, probation)),
        ),
        j!("source_probation_hash" => sha(probation.probation_hash)),
        j!("current_state_hash" => sha(hello_state_hash(snapshot.state_counter)),
        ),
        j!("current_state_counter" => u(snapshot.state_counter)),
        j!("rollback_target_descriptor_source_hash" => sha(probation.previous_descriptor_source_hash),
        ),
        j!("rollback_target_artifact_identity_hash" => sha(probation.previous_artifact_identity_hash),
        ),
        j!("current_candidate_descriptor_source_hash" => sha(probation.new_descriptor_source_hash),
        ),
        j!("current_candidate_artifact_identity_hash" => sha(probation.new_artifact_identity_hash),
        ),
        j!("state_migration_hash" => sha(probation.state_migration_hash)),
        j!("proposed_transaction" => inline(vec![
                j!("schema" => s(HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA)),
                j!("id" => s(HELLO_ROLLBACK_TRANSACTION_PAYLOAD_ID)),
                j!("scope" => s("current_boot")),
                j!("classification" => s("local_only")),
                j!("persistence" => s("none")),
                j!("status" => s(HELLO_ROLLBACK_TRANSACTION_PAYLOAD_STATUS)),
                j!("payload_hash" => sha(hello_rollback_transaction_payload_hash(snapshot, probation)),
                ),
                j!("provenance_hash" => sha(hello_rollback_transaction_payload_provenance_hash(
                        snapshot, probation,
                    )),
                ),
                j!("appended_to_rollback_log" => b(false)),
            ]),
        ),
        j!("required_schemas" => inline(vec![
                j!("audit_record" => s("raios.audit_record.v0")),
                j!("rollback_transaction" => s("raios.rollback_transaction.v0")),
            ]),
        ),
        j!("unavailable_authorities" => inline(vec![
                j!("transaction_writer" => b(true)),
                j!("rollback_transaction_append" => b(true)),
                j!("durable_audit_store" => b(true)),
                j!("rollback_store" => b(true)),
            ]),
        ),
        j!("side_effects" => inline(vec![
                j!("writes_durable_audit_log" => b(false)),
                j!("writes_rollback_store" => b(false)),
                j!("appends_rollback_transaction" => b(false)),
                j!("installs_rollback_plan" => b(false)),
                j!("applies_rollback" => b(false)),
            ]),
        ),
    ])
}

pub(crate) fn emit_audit_rollback_target_region_discovery_inline(
    discovery: rollback_storage_layout::AuditRollbackTargetRegionDiscovery,
) {
    emit_inline_value(audit_rollback_target_region_discovery_value(discovery));
}

fn audit_rollback_target_region_discovery_value(
    discovery: rollback_storage_layout::AuditRollbackTargetRegionDiscovery,
) -> V<'static> {
    inline(vec![
        j!("schema" => s(discovery.schema)),
        j!("id" => s(discovery.id)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("status" => s(discovery.status)),
        j!("reason" => s(discovery.reason)),
        j!("source" => s(discovery.source)),
        j!("storage_authority_id" => s(discovery.storage_authority_id)),
        j!("partition_inventory_available" => b(discovery.partition_inventory_available),
        ),
        j!("partition_inventory_scheme" => s(discovery.partition_inventory_scheme),
        ),
        j!("partition_inventory_source_lba" => u(discovery.partition_inventory_source_lba),
        ),
        j!("partition_entry_count" => u(discovery.partition_entry_count.into())),
        j!("mbr_signature_valid" => b(discovery.mbr_signature_valid)),
        j!("boot_metadata_lba" => u(discovery.boot_metadata_lba)),
        j!("candidate_region_present" => b(discovery.candidate_region_present),
        ),
        j!("candidate_region_start_lba" => u(discovery.candidate_region_start_lba),
        ),
        j!("candidate_region_lba_count" => u(discovery.candidate_region_lba_count),
        ),
        j!("candidate_region_is_scratch" => b(discovery.candidate_region_is_scratch),
        ),
        j!("candidate_overlaps_boot_metadata" => b(discovery.candidate_overlaps_boot_metadata),
        ),
        j!("candidate_overlaps_scratch" => b(discovery.candidate_overlaps_scratch),
        ),
        j!("scratch_region_id" => s(discovery.scratch_region_id)),
        j!("scratch_region_available" => b(discovery.scratch_region_available),
        ),
        j!("scratch_region_start_lba" => u(discovery.scratch_region_start_lba),
        ),
        j!("scratch_region_lba_count" => u(discovery.scratch_region_lba_count),
        ),
        j!("scratch_rejected_as_durable_authority" => b(discovery.scratch_rejected_as_durable_authority),
        ),
        j!("durable_region_available" => b(discovery.durable_region_available),
        ),
        j!("authorizes_append" => b(false)),
        j!("writes_durable_audit_log" => b(false)),
        j!("writes_rollback_store" => b(false)),
        j!("write_attempted" => b(false)),
    ])
}

#[rustfmt::skip]
fn rollback_transaction_writer_storage_authority_gate_value(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) -> V<'static> {
    if let Some(probation) = probation {
        let foundation = rollback_writer_storage_foundation();
        let append_record = hello_rollback_append_record_dry_run(snapshot, probation, foundation);
        let sector_plan =
            hello_rollback_append_sector_plan_dry_run(snapshot, probation, append_record);
        let sector_write =
            hello_rollback_append_sector_write_readback_dry_run(snapshot, probation, sector_plan);
        let target_region_media_write_policy_preflight =
            hello_target_region_media_write_policy_preflight(foundation);
        let target_region_write =
            hello_rollback_target_region_write_readback_dry_run_from_materializer(
            sector_plan,
            foundation,
            target_region_media_write_policy_preflight,
            );
        let durable_append_preflight = hello_rollback_durable_append_authority_preflight(
            foundation,
            append_record,
            sector_plan,
            sector_write,
            target_region_media_write_policy_preflight,
            target_region_write,
            hello_rollback_durable_writer_policy_preflight(
            foundation,
            append_record,
            sector_plan,
            target_region_write,
            ),
        );
        let durable_writer_policy_preflight =
            durable_append_preflight.durable_writer_policy_preflight;
        let durable_append_transaction_authorization_gate =
            hello_rollback_durable_append_transaction_authorization_gate(
            durable_writer_policy_preflight,
            append_record,
            sector_plan,
            target_region_write,
            );
        let append_engine_readiness_decision = hello_rollback_append_engine_readiness_decision(
            durable_append_transaction_authorization_gate,
        );
        let durable_target_region_media_write_policy_preflight =
            durable_append_preflight.target_region_media_write_policy_preflight;
        let media_write_authority_gate = hello_rollback_media_write_authority_gate(
            durable_append_preflight,
            target_region_write,
        );
        let durable_append_authority_decision = hello_rollback_durable_append_authority_decision(
            durable_append_preflight,
            media_write_authority_gate,
            append_engine_readiness_decision,
        );
        let durable_audit_policy_decision =
            hello_rollback_durable_audit_policy_decision(durable_append_authority_decision);
        let durable_audit_policy_candidate = hello_rollback_durable_audit_policy_candidate(
            durable_audit_policy_decision,
            append_record,
        );
        let durable_audit_policy_acceptance_gate =
            hello_rollback_durable_audit_policy_acceptance_gate(durable_audit_policy_candidate);
        let durable_audit_policy_ledger_candidate =
            hello_rollback_durable_audit_policy_ledger_candidate(
            durable_audit_policy_acceptance_gate,
            );
        let durable_audit_policy_ledger_aware_acceptance_result =
            hello_rollback_durable_audit_policy_ledger_aware_acceptance_result(
            durable_audit_policy_ledger_candidate,
            );
        let durable_audit_policy_write_authority_availability =
            hello_rollback_durable_audit_policy_write_authority_availability(
            durable_audit_policy_ledger_aware_acceptance_result,
            durable_audit_policy_ledger_candidate,
            durable_target_region_media_write_policy_preflight,
            target_region_write,
            );
        let durable_policy_ledger_availability = hello_rollback_durable_policy_ledger_availability(
            durable_audit_policy_write_authority_availability,
        );
        let durable_audit_policy_availability =
            hello_rollback_durable_audit_policy_availability(durable_policy_ledger_availability);
        let durable_append_authority_availability =
            hello_rollback_durable_append_authority_availability(durable_audit_policy_availability);
        let transaction_append_availability_decision =
            hello_rollback_transaction_append_availability_decision(
            durable_append_authority_availability,
            append_engine_readiness_decision,
            durable_writer_policy_preflight,
            );
        let transaction_append_authority_denial_gate =
            hello_rollback_transaction_append_authority_denial_gate(
            transaction_append_availability_decision,
            );
        let durable_policy_ledger_availability_dry_run =
            hello_rollback_durable_policy_ledger_availability_dry_run(
            durable_policy_ledger_availability,
            durable_audit_policy_write_authority_availability,
            transaction_append_authority_denial_gate,
            target_region_write,
            );
        let durable_audit_policy_availability_dry_run =
            hello_rollback_durable_audit_policy_availability_dry_run(
            durable_audit_policy_availability,
            durable_policy_ledger_availability_dry_run,
            transaction_append_authority_denial_gate,
            target_region_write,
            );
        let durable_append_authority_availability_dry_run =
            hello_rollback_durable_append_authority_availability_dry_run(
            durable_append_authority_availability,
            durable_audit_policy_availability_dry_run,
            transaction_append_authority_denial_gate,
            target_region_write,
            );
        let transaction_append_dry_run = hello_rollback_transaction_append_dry_run(
            transaction_append_authority_denial_gate,
            append_record,
            sector_plan,
            target_region_write,
        );
        let target_region_sector_inspection =
            hello_rollback_target_region_sector_inspection_from_retained_inspect(
            append_record,
            sector_plan,
            target_region_write,
            );
        let durable_policy_write_authority_decision =
            hello_rollback_durable_policy_write_authority_decision(
            durable_append_authority_availability_dry_run,
            durable_audit_policy_write_authority_availability,
            durable_audit_policy_availability,
            durable_append_authority_availability,
            transaction_append_dry_run,
            target_region_sector_inspection,
            );
        inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_STATUS)),
            j!("service_id" => s(SERVICE_ID)),
            j!("requested_capability" => s(HELLO_ROLLBACK_APPLY_CAPABILITY)),
            j!("gate_hash" => sha(rollback_transaction_writer_storage_authority_gate_hash(
            snapshot, probation,
            ))),
            j!("rollback_payload_envelope_gate_hash" => sha(rollback_payload_envelope_gate_hash(snapshot, probation))),
            j!("payload_hash" => sha(hello_rollback_transaction_payload_hash(snapshot, probation))),
            j!("provenance_hash" => sha(hello_rollback_transaction_payload_provenance_hash(
            snapshot, probation,
            ))),
            j!("rollback_append_intent_gate_hash" => sha(rollback_append_intent_gate_hash(snapshot, probation))),
            j!("rollback_write_authority_gate_hash" => sha(rollback_write_authority_gate_hash(snapshot, probation))),
            j!("rollback_transaction_preflight_hash" => sha(hello_rollback_transaction_preflight_hash(
            snapshot, probation,
            ))),
            j!("rollback_apply_hash" => sha(hello_rollback_apply_denial_hash(snapshot, probation))),
            j!("rollback_preview_hash" => sha(hello_rollback_preview_hash(snapshot, probation))),
            j!("source_probation_hash" => sha(probation.probation_hash)),
            j!("current_state_hash" => sha(hello_state_hash(snapshot.state_counter))),
            j!("current_state_counter" => u(snapshot.state_counter)),
            j!("rollback_target_descriptor_source_hash" => sha(probation.previous_descriptor_source_hash)),
            j!("rollback_target_artifact_identity_hash" => sha(probation.previous_artifact_identity_hash)),
            j!("current_candidate_descriptor_source_hash" => sha(probation.new_descriptor_source_hash)),
            j!("current_candidate_artifact_identity_hash" => sha(probation.new_artifact_identity_hash)),
            j!("state_migration_hash" => sha(probation.state_migration_hash)),
            j!("required_schemas" => inline(vec![
            j!("audit_record" => s("raios.audit_record.v0")),
            j!("rollback_transaction" => s("raios.rollback_transaction.v0")),
            ])),
            j!("writer_storage_foundation" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_SCHEMA)),
            j!("owner_method" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_OWNER)),
            j!("recovery_visible_method" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_OWNER)),
            j!("storage_authority" => inline(vec![
            j!("schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_SCHEMA)),
            j!("id" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID)),
            j!("owner_method" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER)),
            j!("source_method" => s(rollback_storage_layout::MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_METHOD)),
            j!("status" => s(foundation.storage_layout_status)),
            j!("reason" => s(foundation.storage_layout_reason)),
            j!("available" => b(foundation.storage_layout_available)),
            j!("authorizes_append" => b(false)),
            ])),
            j!("append_targets" => inline(vec![
            j!("audit_ledger" => inline(vec![
            j!("id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("available" => b(foundation.durable_audit_store_available)),
            ])),
            j!("rollback_store" => inline(vec![
            j!("id" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID)),
            j!("schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("available" => b(foundation.rollback_store_available)),
            j!("append_available" => b(foundation.rollback_transaction_append_available)),
            ])),
            ])),
            j!("target_region_discovery" => audit_rollback_target_region_discovery_value(foundation.target_region_discovery)),
            j!("append_target_owner" => inline(vec![
            j!("schema" => s(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_SCHEMA)),
            j!("id" => s(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID)),
            j!("owner_method" => s(rollback_append_contract::MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_METHOD)),
            j!("storage_authority_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID)),
            j!("status" => s(foundation.append_target_owner_status)),
            j!("reason" => s(foundation.append_target_owner_reason)),
            j!("available" => b(foundation.append_target_owner_available)),
            j!("block_write_path_available" => b(foundation.block_write_path_available)),
            j!("block_write_path_reason" => s(foundation.block_write_path_reason)),
            j!("authorizes_append" => b(false)),
            ])),
            j!("transaction_writer_readiness" => inline(vec![
            j!("schema" => s(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_SCHEMA)),
            j!("id" => s(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID)),
            j!("owner_method" => s(rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER)),
            j!("append_target_owner_id" => s(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID)),
            j!("status" => s(foundation.transaction_writer_status)),
            j!("reason" => s(foundation.transaction_writer_reason)),
            j!("ready" => b(foundation.transaction_writer_ready)),
            j!("block_write_path_available" => b(foundation.block_write_path_available)),
            j!("block_write_path_reason" => s(foundation.block_write_path_reason)),
            j!("scratch_only_writer_dry_run" => inline(vec![
            j!("schema" => s(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_SCHEMA,)),
            j!("id" => s(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("status" => s(foundation.scratch_writer_dry_run_status)),
            j!("reason" => s(foundation.scratch_writer_dry_run_reason)),
            j!("source_authority_id" => s(foundation.scratch_block_write_authority_id)),
            j!("source_region_id" => s(foundation.scratch_region_id)),
            j!("target_region_start_lba" => u(foundation.scratch_region_start_lba)),
            j!("target_region_lba_count" => u(foundation.scratch_region_lba_count)),
            j!("target_byte_count" => u(foundation.scratch_region_byte_count.into())),
            j!("target_range_scratch_owned" => b(foundation.scratch_block_write_authority_available)),
            j!("target_range_within_device_bounds" => b(foundation.scratch_region_within_device_bounds)),
            j!("target_range_no_boot_or_partition_metadata_overlap" => b(foundation.scratch_region_no_boot_or_partition_metadata_overlap)),
            j!("target_range_ready" => b(foundation.scratch_writer_dry_run_ready)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("append_record_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("canonicalization" => s(HELLO_ROLLBACK_APPEND_RECORD_CANONICALIZATION)),
            j!("status" => s(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_REASON)),
            j!("dry_run_hash" => sha(append_record.dry_run_hash)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("audit_record_image_hash" => sha(append_record.audit_record_image_hash)),
            j!("audit_record_byte_length" => u(append_record.audit_record_byte_length)),
            j!("rollback_store_target_id" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("rollback_transaction_image_hash" => sha(append_record.rollback_transaction_image_hash)),
            j!("rollback_transaction_byte_length" => u(append_record.rollback_transaction_byte_length)),
            j!("total_record_byte_length" => u(append_record.total_record_byte_length)),
            j!("target_start_lba" => u(append_record.target_start_lba)),
            j!("target_lba_count" => u(append_record.target_lba_count)),
            j!("target_byte_count" => u(append_record.target_byte_count)),
            j!("target_range_ready" => b(append_record.target_range_ready)),
            j!("source_payload_hash" => sha(hello_rollback_transaction_payload_hash(snapshot, probation))),
            j!("source_provenance_hash" => sha(hello_rollback_transaction_payload_provenance_hash(
            snapshot, probation,
            ))),
            j!("sector_plan_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("canonicalization" => s(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_CANONICALIZATION)),
            j!("status" => s(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_REASON)),
            j!("plan_hash" => sha(sector_plan.plan_hash)),
            j!("sector_image_hash" => sha(sector_plan.sector_image_hash)),
            j!("sector_size_bytes" => u(sector_plan.sector_size_bytes)),
            j!("audit_record_offset" => u(sector_plan.audit_record_offset)),
            j!("audit_record_byte_length" => u(sector_plan.audit_record_byte_length)),
            j!("rollback_transaction_offset" => u(sector_plan.rollback_transaction_offset)),
            j!("rollback_transaction_byte_length" => u(sector_plan.rollback_transaction_byte_length)),
            j!("padding_policy" => s(HELLO_ROLLBACK_APPEND_SECTOR_PADDING_POLICY)),
            j!("padding_offset" => u(sector_plan.padding_offset)),
            j!("padding_byte_length" => u(sector_plan.padding_byte_length)),
            j!("target_start_lba" => u(sector_plan.target_start_lba)),
            j!("target_lba_count" => u(sector_plan.target_lba_count)),
            j!("target_byte_count" => u(sector_plan.target_byte_count)),
            j!("target_range_ready" => b(sector_plan.target_range_ready)),
            j!("source_append_record_hash" => sha(append_record.dry_run_hash)),
            j!("scratch_sector_write_readback_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(sector_write.status)),
            j!("reason" => s(sector_write.reason)),
            j!("dry_run_hash" => sha(sector_write.dry_run_hash)),
            j!("source_plan_hash" => sha(sector_write.source_plan_hash)),
            j!("planned_sector_image_hash" => sha(sector_write.planned_sector_image_hash)),
            j!("readback_sector_image_hash" => sha(sector_write.readback_sector_image_hash)),
            j!("target_start_lba" => u(sector_write.target_start_lba)),
            j!("target_lba_count" => u(sector_write.target_lba_count)),
            j!("target_byte_count" => u(sector_write.target_byte_count)),
            j!("label_found" => b(sector_write.label_found)),
            j!("target_range_ready" => b(sector_write.target_range_ready)),
            j!("write_attempted" => b(sector_write.write_attempted)),
            j!("write_completed" => b(sector_write.write_completed)),
            j!("readback_completed" => b(sector_write.readback_completed)),
            j!("readback_matches_planned_image" => b(sector_write.readback_matches_planned_image)),
            j!("durable_append_authority_preflight" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_REASON)),
            j!("preflight_hash" => sha(durable_append_preflight.preflight_hash)),
            j!("source_write_readback_hash" => sha(durable_append_preflight.source_write_readback_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_append_preflight.source_target_region_write_readback_hash)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_append_preflight.test_infrastructure_media_write_authority_available)),
            j!("remaining_denial_reason" => s(durable_append_preflight.remaining_denial_reason)),
            j!("durable_writer_policy_preflight" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_REASON)),
            j!("preflight_hash" => sha(durable_writer_policy_preflight.preflight_hash)),
            j!("source_append_record_hash" => sha(durable_writer_policy_preflight.source_append_record_hash)),
            j!("source_sector_plan_hash" => sha(durable_writer_policy_preflight.source_sector_plan_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_writer_policy_preflight.source_target_region_write_readback_hash)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_writer_policy_preflight.target_start_lba)),
            j!("target_lba_count" => u(durable_writer_policy_preflight.target_lba_count)),
            j!("target_byte_count" => u(durable_writer_policy_preflight.target_byte_count)),
            j!("target_range_ready" => b(durable_writer_policy_preflight.target_range_ready)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_writer_policy_preflight.test_infrastructure_media_write_authority_available,)),
            j!("durable_audit_writer_available" => b(durable_writer_policy_preflight.durable_audit_writer_available)),
            j!("rollback_store_writer_available" => b(durable_writer_policy_preflight.rollback_store_writer_available)),
            j!("transaction_append_writer_available" => b(durable_writer_policy_preflight.transaction_append_writer_available)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_append_transaction_authorization_gate" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_REASON)),
            j!("gate_hash" => sha(durable_append_transaction_authorization_gate.gate_hash)),
            j!("source_writer_policy_preflight_hash" => sha(durable_append_transaction_authorization_gate.source_writer_policy_preflight_hash,)),
            j!("source_append_record_hash" => sha(durable_append_transaction_authorization_gate.source_append_record_hash)),
            j!("source_sector_plan_hash" => sha(durable_append_transaction_authorization_gate.source_sector_plan_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_append_transaction_authorization_gate.source_target_region_write_readback_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_append_transaction_authorization_gate.target_start_lba)),
            j!("target_lba_count" => u(durable_append_transaction_authorization_gate.target_lba_count)),
            j!("target_byte_count" => u(durable_append_transaction_authorization_gate.target_byte_count)),
            j!("target_range_ready" => b(durable_append_transaction_authorization_gate.target_range_ready)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_append_transaction_authorization_gate
            .test_infrastructure_media_write_authority_available,)),
            j!("append_engine_available" => b(durable_append_transaction_authorization_gate.append_engine_available)),
            j!("durable_audit_writer_available" => b(durable_append_transaction_authorization_gate.durable_audit_writer_available)),
            j!("rollback_store_writer_available" => b(durable_append_transaction_authorization_gate.rollback_store_writer_available)),
            j!("transaction_append_writer_available" => b(durable_append_transaction_authorization_gate.transaction_append_writer_available)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("append_engine_readiness_decision" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(append_engine_readiness_decision.status)),
            j!("reason" => s(append_engine_readiness_decision.reason)),
            j!("decision_hash" => sha(append_engine_readiness_decision.decision_hash)),
            j!("source_authorization_gate_hash" => sha(append_engine_readiness_decision.source_authorization_gate_hash)),
            j!("source_writer_policy_preflight_hash" => sha(append_engine_readiness_decision.source_writer_policy_preflight_hash)),
            j!("source_append_record_hash" => sha(append_engine_readiness_decision.source_append_record_hash)),
            j!("source_sector_plan_hash" => sha(append_engine_readiness_decision.source_sector_plan_hash)),
            j!("source_target_region_write_readback_hash" => sha(append_engine_readiness_decision.source_target_region_write_readback_hash)),
            j!("target_start_lba" => u(append_engine_readiness_decision.target_start_lba)),
            j!("target_lba_count" => u(append_engine_readiness_decision.target_lba_count)),
            j!("target_byte_count" => u(append_engine_readiness_decision.target_byte_count)),
            j!("target_range_ready" => b(append_engine_readiness_decision.target_range_ready)),
            j!("test_infrastructure_media_write_authority_available" => b(append_engine_readiness_decision.test_infrastructure_media_write_authority_available,)),
            j!("append_engine_available" => b(append_engine_readiness_decision.append_engine_available)),
            j!("durable_audit_writer_available" => b(append_engine_readiness_decision.durable_audit_writer_available)),
            j!("rollback_store_writer_available" => b(append_engine_readiness_decision.rollback_store_writer_available)),
            j!("transaction_append_writer_available" => b(append_engine_readiness_decision.transaction_append_writer_available)),
            j!("ready" => b(append_engine_readiness_decision.ready)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("target_region_discovery" => audit_rollback_target_region_discovery_value(durable_append_preflight.target_region_discovery,)),
            j!("target_region_media_write_policy_preflight" => inline(vec![
            j!("schema" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,)),
            j!("id" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID,)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(foundation.target_region_media_write_policy_preflight_status)),
            j!("reason" => s(foundation.target_region_media_write_policy_preflight_reason)),
            j!("preflight_hash" => sha(durable_target_region_media_write_policy_preflight.preflight_hash)),
            j!("source_contract_schema" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA)),
            j!("source_contract_id" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID)),
            j!("source_contract_status" => s(durable_target_region_media_write_policy_preflight.source_contract_status)),
            j!("source_contract_reason" => s(durable_target_region_media_write_policy_preflight.source_contract_reason)),
            j!("owner_method" => s(rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER)),
            j!("append_target_owner_id" => s(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID)),
            j!("storage_authority_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_region_start_lba" => u(durable_target_region_media_write_policy_preflight.target_region_start_lba)),
            j!("target_region_lba_count" => u(durable_target_region_media_write_policy_preflight.target_region_lba_count)),
            j!("target_byte_count" => u(durable_target_region_media_write_policy_preflight.target_byte_count)),
            j!("source_contract_target_range_ready" => b(durable_target_region_media_write_policy_preflight.source_contract_target_range_ready)),
            j!("owner_ids_verified" => b(durable_target_region_media_write_policy_preflight.owner_ids_verified)),
            j!("target_ids_verified" => b(durable_target_region_media_write_policy_preflight.target_ids_verified)),
            j!("target_span_verified" => b(durable_target_region_media_write_policy_preflight.target_span_verified)),
            j!("schema_ids_verified" => b(durable_target_region_media_write_policy_preflight.schema_ids_verified)),
            j!("media_write_authority_required" => b(true)),
            j!("media_write_authority_available" => b(durable_target_region_media_write_policy_preflight.media_write_authority_available)),
            j!("media_write_authority_reason" => s(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_MISSING_REASON)),
            j!("durable_audit_policy_required" => b(true)),
            j!("durable_audit_policy_available" => b(durable_target_region_media_write_policy_preflight.durable_audit_policy_available)),
            j!("durable_audit_policy_reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_MISSING_REASON)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("media_write_authority_gate" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_REASON)),
            j!("gate_hash" => sha(media_write_authority_gate.gate_hash)),
            j!("source_durable_append_authority_preflight_schema" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA)),
            j!("source_durable_append_authority_preflight_id" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID)),
            j!("source_durable_append_authority_preflight_hash" => sha(media_write_authority_gate.source_durable_append_authority_preflight_hash)),
            j!("source_policy_preflight_schema" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,)),
            j!("source_policy_preflight_id" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID,)),
            j!("source_policy_preflight_hash" => sha(media_write_authority_gate.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_schema" => s(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA)),
            j!("source_target_region_write_readback_id" => s(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID)),
            j!("source_target_region_write_readback_hash" => sha(media_write_authority_gate.source_target_region_write_readback_hash)),
            j!("source_contract_schema" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA)),
            j!("source_contract_id" => s(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID)),
            j!("source_contract_status" => s(media_write_authority_gate.source_contract_status)),
            j!("source_contract_reason" => s(media_write_authority_gate.source_contract_reason)),
            j!("target_region_start_lba" => u(media_write_authority_gate.target_region_start_lba)),
            j!("target_region_lba_count" => u(media_write_authority_gate.target_region_lba_count)),
            j!("target_byte_count" => u(media_write_authority_gate.target_byte_count)),
            j!("source_contract_target_range_ready" => b(media_write_authority_gate.source_contract_target_range_ready)),
            j!("owner_ids_verified" => b(media_write_authority_gate.owner_ids_verified)),
            j!("target_ids_verified" => b(media_write_authority_gate.target_ids_verified)),
            j!("target_span_verified" => b(media_write_authority_gate.target_span_verified)),
            j!("schema_ids_verified" => b(media_write_authority_gate.schema_ids_verified)),
            j!("media_write_authority_required" => b(true)),
            j!("media_write_authority_available" => b(media_write_authority_gate.media_write_authority_available)),
            j!("media_write_authority_reason" => s(HELLO_ROLLBACK_TEST_MEDIA_WRITE_AUTHORITY_REASON)),
            j!("test_infrastructure_media_write_authority_available" => b(media_write_authority_gate.test_infrastructure_media_write_authority_available)),
            j!("durable_audit_policy_required" => b(true)),
            j!("durable_audit_policy_available" => b(media_write_authority_gate.durable_audit_policy_available)),
            j!("durable_audit_policy_reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_MISSING_REASON)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("target_region_write_attempted" => b(media_write_authority_gate.target_region_write_attempted)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_append_authority_decision" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_REASON)),
            j!("decision_hash" => sha(durable_append_authority_decision.decision_hash)),
            j!("source_durable_append_authority_preflight_hash" => sha(durable_append_authority_decision.source_durable_append_authority_preflight_hash,)),
            j!("source_writer_policy_preflight_hash" => sha(durable_append_authority_decision.source_writer_policy_preflight_hash)),
            j!("source_append_engine_readiness_decision_hash" => sha(durable_append_authority_decision.source_append_engine_readiness_decision_hash)),
            j!("source_media_write_authority_gate_hash" => sha(durable_append_authority_decision.source_media_write_authority_gate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_append_authority_decision.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_append_authority_decision.source_target_region_write_readback_hash)),
            j!("target_start_lba" => u(durable_append_authority_decision.target_start_lba)),
            j!("target_lba_count" => u(durable_append_authority_decision.target_lba_count)),
            j!("target_byte_count" => u(durable_append_authority_decision.target_byte_count)),
            j!("writer_policy_ready" => b(durable_append_authority_decision.writer_policy_ready)),
            j!("append_engine_ready" => b(durable_append_authority_decision.append_engine_ready)),
            j!("media_write_gate_ready" => b(durable_append_authority_decision.media_write_gate_ready)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_append_authority_decision.test_infrastructure_media_write_authority_available,)),
            j!("durable_audit_policy_available" => b(durable_append_authority_decision.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_append_authority_decision.durable_append_authority_available)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_audit_policy_decision" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_REASON)),
            j!("decision_hash" => sha(durable_audit_policy_decision.decision_hash)),
            j!("source_durable_append_authority_decision_hash" => sha(durable_audit_policy_decision.source_durable_append_authority_decision_hash)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_decision.source_policy_preflight_hash)),
            j!("source_media_write_authority_gate_hash" => sha(durable_audit_policy_decision.source_media_write_authority_gate_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_decision.source_target_region_write_readback_hash)),
            j!("target_start_lba" => u(durable_audit_policy_decision.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_decision.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_decision.target_byte_count)),
            j!("append_engine_ready" => b(durable_audit_policy_decision.append_engine_ready)),
            j!("media_write_policy_verified" => b(durable_audit_policy_decision.media_write_policy_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_audit_policy_decision.test_infrastructure_media_write_authority_available)),
            j!("durable_append_authority_available" => b(durable_audit_policy_decision.durable_append_authority_available)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_decision.durable_audit_policy_available)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_audit_policy_candidate" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_REASON)),
            j!("candidate_hash" => sha(durable_audit_policy_candidate.candidate_hash)),
            j!("source_durable_audit_policy_decision_hash" => sha(durable_audit_policy_candidate.source_durable_audit_policy_decision_hash)),
            j!("source_audit_record_image_hash" => sha(durable_audit_policy_candidate.source_audit_record_image_hash)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_candidate.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_candidate.source_target_region_write_readback_hash)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_audit_policy_candidate.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_candidate.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_candidate.target_byte_count)),
            j!("media_write_policy_verified" => b(durable_audit_policy_candidate.media_write_policy_verified)),
            j!("durable_audit_policy_candidate_available" => b(durable_audit_policy_candidate.durable_audit_policy_candidate_available)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_candidate.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_audit_policy_candidate.durable_append_authority_available)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_audit_policy_acceptance_gate" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_REASON)),
            j!("gate_hash" => sha(durable_audit_policy_acceptance_gate.gate_hash)),
            j!("source_durable_audit_policy_candidate_hash" => sha(durable_audit_policy_acceptance_gate.source_durable_audit_policy_candidate_hash,)),
            j!("source_durable_audit_policy_decision_hash" => sha(durable_audit_policy_acceptance_gate.source_durable_audit_policy_decision_hash)),
            j!("source_audit_record_image_hash" => sha(durable_audit_policy_acceptance_gate.source_audit_record_image_hash)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_acceptance_gate.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_acceptance_gate.source_target_region_write_readback_hash)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_audit_policy_acceptance_gate.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_acceptance_gate.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_acceptance_gate.target_byte_count)),
            j!("candidate_available" => b(durable_audit_policy_acceptance_gate.candidate_available)),
            j!("media_write_policy_verified" => b(durable_audit_policy_acceptance_gate.media_write_policy_verified)),
            j!("durable_policy_ledger_available" => b(durable_audit_policy_acceptance_gate.durable_policy_ledger_available)),
            j!("write_authority_available" => b(durable_audit_policy_acceptance_gate.write_authority_available)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_acceptance_gate.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_audit_policy_acceptance_gate.durable_append_authority_available)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_audit_policy_ledger_candidate" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("access" => s("read_only")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_REASON)),
            j!("ledger_candidate_hash" => sha(durable_audit_policy_ledger_candidate.ledger_candidate_hash)),
            j!("source_acceptance_gate_hash" => sha(durable_audit_policy_ledger_candidate.source_acceptance_gate_hash)),
            j!("source_durable_audit_policy_candidate_hash" => sha(durable_audit_policy_ledger_candidate.source_durable_audit_policy_candidate_hash,)),
            j!("source_durable_audit_policy_decision_hash" => sha(durable_audit_policy_ledger_candidate.source_durable_audit_policy_decision_hash,)),
            j!("source_audit_record_image_hash" => sha(durable_audit_policy_ledger_candidate.source_audit_record_image_hash)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_ledger_candidate.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_ledger_candidate.source_target_region_write_readback_hash)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_audit_policy_ledger_candidate.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_ledger_candidate.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_ledger_candidate.target_byte_count)),
            j!("read_only_ledger_candidate_available" => b(durable_audit_policy_ledger_candidate.read_only_ledger_candidate_available)),
            j!("candidate_available" => b(durable_audit_policy_ledger_candidate.candidate_available)),
            j!("media_write_policy_verified" => b(durable_audit_policy_ledger_candidate.media_write_policy_verified)),
            j!("durable_policy_ledger_available" => b(durable_audit_policy_ledger_candidate.durable_policy_ledger_available)),
            j!("write_authority_available" => b(durable_audit_policy_ledger_candidate.write_authority_available)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_ledger_candidate.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_audit_policy_ledger_candidate.durable_append_authority_available)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_audit_policy_ledger_aware_acceptance_result" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_REASON)),
            j!("result_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result.result_hash)),
            j!("source_ledger_candidate_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result.source_ledger_candidate_hash,)),
            j!("source_acceptance_gate_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result.source_acceptance_gate_hash,)),
            j!("source_durable_audit_policy_candidate_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result
            .source_durable_audit_policy_candidate_hash,)),
            j!("source_durable_audit_policy_decision_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result
            .source_durable_audit_policy_decision_hash,)),
            j!("source_audit_record_image_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result.source_audit_record_image_hash,)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result.source_policy_preflight_hash,)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_ledger_aware_acceptance_result
            .source_target_region_write_readback_hash,)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_audit_policy_ledger_aware_acceptance_result.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_ledger_aware_acceptance_result.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_ledger_aware_acceptance_result.target_byte_count)),
            j!("read_only_ledger_candidate_available" => b(durable_audit_policy_ledger_aware_acceptance_result
            .read_only_ledger_candidate_available,)),
            j!("ledger_evidence_verified" => b(durable_audit_policy_ledger_aware_acceptance_result.ledger_evidence_verified)),
            j!("write_authority_available" => b(durable_audit_policy_ledger_aware_acceptance_result.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_audit_policy_ledger_aware_acceptance_result.durable_policy_ledger_available,)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_ledger_aware_acceptance_result.durable_audit_policy_available,)),
            j!("durable_append_authority_available" => b(durable_audit_policy_ledger_aware_acceptance_result.durable_append_authority_available,)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_audit_policy_write_authority_availability" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_REASON)),
            j!("availability_hash" => sha(durable_audit_policy_write_authority_availability.availability_hash)),
            j!("source_ledger_aware_acceptance_result_hash" => sha(durable_audit_policy_write_authority_availability
            .source_ledger_aware_acceptance_result_hash,)),
            j!("source_ledger_candidate_hash" => sha(durable_audit_policy_write_authority_availability.source_ledger_candidate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_write_authority_availability.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_write_authority_availability
            .source_target_region_write_readback_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_audit_policy_write_authority_availability.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_write_authority_availability.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_write_authority_availability.target_byte_count)),
            j!("ledger_evidence_verified" => b(durable_audit_policy_write_authority_availability.ledger_evidence_verified)),
            j!("media_write_policy_verified" => b(durable_audit_policy_write_authority_availability.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(durable_audit_policy_write_authority_availability.target_region_write_readback_verified,)),
            j!("target_span_verified" => b(durable_audit_policy_write_authority_availability.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(durable_audit_policy_write_authority_availability.audit_rollback_target_ids_verified,)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_audit_policy_write_authority_availability
            .test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_audit_policy_write_authority_availability.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_audit_policy_write_authority_availability.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_write_authority_availability.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_audit_policy_write_authority_availability.durable_append_authority_available,)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_policy_ledger_availability" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_REASON)),
            j!("availability_hash" => sha(durable_policy_ledger_availability.availability_hash)),
            j!("source_write_authority_availability_hash" => sha(durable_policy_ledger_availability.source_write_authority_availability_hash)),
            j!("source_ledger_aware_acceptance_result_hash" => sha(durable_policy_ledger_availability.source_ledger_aware_acceptance_result_hash)),
            j!("source_ledger_candidate_hash" => sha(durable_policy_ledger_availability.source_ledger_candidate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_policy_ledger_availability.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_policy_ledger_availability.source_target_region_write_readback_hash)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_policy_ledger_availability.target_start_lba)),
            j!("target_lba_count" => u(durable_policy_ledger_availability.target_lba_count)),
            j!("target_byte_count" => u(durable_policy_ledger_availability.target_byte_count)),
            j!("write_authority_evidence_verified" => b(durable_policy_ledger_availability.write_authority_evidence_verified)),
            j!("ledger_evidence_verified" => b(durable_policy_ledger_availability.ledger_evidence_verified)),
            j!("media_write_policy_verified" => b(durable_policy_ledger_availability.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(durable_policy_ledger_availability.target_region_write_readback_verified)),
            j!("target_span_verified" => b(durable_policy_ledger_availability.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(durable_policy_ledger_availability.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_policy_ledger_availability.test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_policy_ledger_availability.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_policy_ledger_availability.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_policy_ledger_availability.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_policy_ledger_availability.durable_append_authority_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_audit_policy_availability" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_REASON)),
            j!("availability_hash" => sha(durable_audit_policy_availability.availability_hash)),
            j!("source_policy_ledger_availability_hash" => sha(durable_audit_policy_availability.source_policy_ledger_availability_hash)),
            j!("source_write_authority_availability_hash" => sha(durable_audit_policy_availability.source_write_authority_availability_hash)),
            j!("source_ledger_aware_acceptance_result_hash" => sha(durable_audit_policy_availability.source_ledger_aware_acceptance_result_hash)),
            j!("source_ledger_candidate_hash" => sha(durable_audit_policy_availability.source_ledger_candidate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_availability.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_availability.source_target_region_write_readback_hash)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_audit_policy_availability.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_availability.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_availability.target_byte_count)),
            j!("policy_ledger_availability_evidence_verified" => b(durable_audit_policy_availability.policy_ledger_availability_evidence_verified)),
            j!("write_authority_evidence_verified" => b(durable_audit_policy_availability.write_authority_evidence_verified)),
            j!("ledger_evidence_verified" => b(durable_audit_policy_availability.ledger_evidence_verified)),
            j!("media_write_policy_verified" => b(durable_audit_policy_availability.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(durable_audit_policy_availability.target_region_write_readback_verified)),
            j!("target_span_verified" => b(durable_audit_policy_availability.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(durable_audit_policy_availability.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_audit_policy_availability.test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_audit_policy_availability.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_audit_policy_availability.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_availability.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_audit_policy_availability.durable_append_authority_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_append_authority_availability" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_REASON)),
            j!("availability_hash" => sha(durable_append_authority_availability.availability_hash)),
            j!("source_audit_policy_availability_hash" => sha(durable_append_authority_availability.source_audit_policy_availability_hash)),
            j!("source_policy_ledger_availability_hash" => sha(durable_append_authority_availability.source_policy_ledger_availability_hash)),
            j!("source_write_authority_availability_hash" => sha(durable_append_authority_availability.source_write_authority_availability_hash)),
            j!("source_ledger_aware_acceptance_result_hash" => sha(durable_append_authority_availability.source_ledger_aware_acceptance_result_hash,)),
            j!("source_ledger_candidate_hash" => sha(durable_append_authority_availability.source_ledger_candidate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_append_authority_availability.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_append_authority_availability.source_target_region_write_readback_hash)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_append_authority_availability.target_start_lba)),
            j!("target_lba_count" => u(durable_append_authority_availability.target_lba_count)),
            j!("target_byte_count" => u(durable_append_authority_availability.target_byte_count)),
            j!("audit_policy_availability_evidence_verified" => b(durable_append_authority_availability.audit_policy_availability_evidence_verified)),
            j!("policy_ledger_availability_evidence_verified" => b(durable_append_authority_availability.policy_ledger_availability_evidence_verified,)),
            j!("write_authority_evidence_verified" => b(durable_append_authority_availability.write_authority_evidence_verified)),
            j!("ledger_evidence_verified" => b(durable_append_authority_availability.ledger_evidence_verified)),
            j!("media_write_policy_verified" => b(durable_append_authority_availability.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(durable_append_authority_availability.target_region_write_readback_verified)),
            j!("target_span_verified" => b(durable_append_authority_availability.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(durable_append_authority_availability.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_append_authority_availability
            .test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_append_authority_availability.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_append_authority_availability.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_append_authority_availability.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_append_authority_availability.durable_append_authority_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("transaction_append_availability_decision" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_REASON)),
            j!("decision_hash" => sha(transaction_append_availability_decision.decision_hash)),
            j!("source_durable_append_authority_availability_hash" => sha(transaction_append_availability_decision
            .source_durable_append_authority_availability_hash,)),
            j!("source_audit_policy_availability_hash" => sha(transaction_append_availability_decision.source_audit_policy_availability_hash)),
            j!("source_append_engine_readiness_decision_hash" => sha(transaction_append_availability_decision.source_append_engine_readiness_decision_hash,)),
            j!("source_writer_policy_preflight_hash" => sha(transaction_append_availability_decision.source_writer_policy_preflight_hash)),
            j!("source_policy_preflight_hash" => sha(transaction_append_availability_decision.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(transaction_append_availability_decision.source_target_region_write_readback_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(transaction_append_availability_decision.target_start_lba)),
            j!("target_lba_count" => u(transaction_append_availability_decision.target_lba_count)),
            j!("target_byte_count" => u(transaction_append_availability_decision.target_byte_count)),
            j!("durable_append_authority_availability_evidence_verified" => b(transaction_append_availability_decision
            .durable_append_authority_availability_evidence_verified,)),
            j!("audit_policy_availability_evidence_verified" => b(transaction_append_availability_decision.audit_policy_availability_evidence_verified,)),
            j!("append_engine_ready" => b(transaction_append_availability_decision.append_engine_ready)),
            j!("writer_policy_ready" => b(transaction_append_availability_decision.writer_policy_ready)),
            j!("media_write_policy_verified" => b(transaction_append_availability_decision.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(transaction_append_availability_decision.target_region_write_readback_verified)),
            j!("target_span_verified" => b(transaction_append_availability_decision.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(transaction_append_availability_decision.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(transaction_append_availability_decision
            .test_infrastructure_media_write_authority_available,)),
            j!("durable_append_authority_available" => b(transaction_append_availability_decision.durable_append_authority_available)),
            j!("durable_audit_policy_available" => b(transaction_append_availability_decision.durable_audit_policy_available)),
            j!("transaction_append_available" => b(transaction_append_availability_decision.transaction_append_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("transaction_append_authority_denial_gate" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_REASON)),
            j!("gate_hash" => sha(transaction_append_authority_denial_gate.gate_hash)),
            j!("source_transaction_append_availability_decision_hash" => sha(transaction_append_authority_denial_gate
            .source_transaction_append_availability_decision_hash,)),
            j!("source_durable_append_authority_availability_hash" => sha(transaction_append_authority_denial_gate
            .source_durable_append_authority_availability_hash,)),
            j!("source_audit_policy_availability_hash" => sha(transaction_append_authority_denial_gate.source_audit_policy_availability_hash)),
            j!("source_append_engine_readiness_decision_hash" => sha(transaction_append_authority_denial_gate.source_append_engine_readiness_decision_hash,)),
            j!("source_writer_policy_preflight_hash" => sha(transaction_append_authority_denial_gate.source_writer_policy_preflight_hash)),
            j!("source_policy_preflight_hash" => sha(transaction_append_authority_denial_gate.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(transaction_append_authority_denial_gate.source_target_region_write_readback_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(transaction_append_authority_denial_gate.target_start_lba)),
            j!("target_lba_count" => u(transaction_append_authority_denial_gate.target_lba_count)),
            j!("target_byte_count" => u(transaction_append_authority_denial_gate.target_byte_count)),
            j!("availability_decision_evidence_verified" => b(transaction_append_authority_denial_gate.availability_decision_evidence_verified)),
            j!("append_engine_ready" => b(transaction_append_authority_denial_gate.append_engine_ready)),
            j!("writer_policy_ready" => b(transaction_append_authority_denial_gate.writer_policy_ready)),
            j!("media_write_policy_verified" => b(transaction_append_authority_denial_gate.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(transaction_append_authority_denial_gate.target_region_write_readback_verified)),
            j!("target_span_verified" => b(transaction_append_authority_denial_gate.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(transaction_append_authority_denial_gate.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(transaction_append_authority_denial_gate
            .test_infrastructure_media_write_authority_available,)),
            j!("durable_append_authority_available" => b(transaction_append_authority_denial_gate.durable_append_authority_available)),
            j!("durable_audit_policy_available" => b(transaction_append_authority_denial_gate.durable_audit_policy_available)),
            j!("transaction_append_available" => b(transaction_append_authority_denial_gate.transaction_append_available)),
            j!("missing_transaction_append_authority" => b(transaction_append_authority_denial_gate.missing_transaction_append_authority)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("durable_policy_ledger_availability_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_REASON)),
            j!("dry_run_hash" => sha(durable_policy_ledger_availability_dry_run.dry_run_hash)),
            j!("source_policy_ledger_availability_hash" => sha(durable_policy_ledger_availability_dry_run.source_policy_ledger_availability_hash,)),
            j!("source_write_authority_availability_hash" => sha(durable_policy_ledger_availability_dry_run.source_write_authority_availability_hash,)),
            j!("source_ledger_aware_acceptance_result_hash" => sha(durable_policy_ledger_availability_dry_run.source_ledger_aware_acceptance_result_hash,)),
            j!("source_ledger_candidate_hash" => sha(durable_policy_ledger_availability_dry_run.source_ledger_candidate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_policy_ledger_availability_dry_run.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_policy_ledger_availability_dry_run.source_target_region_write_readback_hash,)),
            j!("source_authority_denial_gate_hash" => sha(durable_policy_ledger_availability_dry_run.source_authority_denial_gate_hash)),
            j!("source_transaction_append_availability_decision_hash" => sha(durable_policy_ledger_availability_dry_run
            .source_transaction_append_availability_decision_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_policy_ledger_availability_dry_run.target_start_lba)),
            j!("target_lba_count" => u(durable_policy_ledger_availability_dry_run.target_lba_count)),
            j!("target_byte_count" => u(durable_policy_ledger_availability_dry_run.target_byte_count)),
            j!("policy_ledger_availability_evidence_verified" => b(durable_policy_ledger_availability_dry_run.policy_ledger_availability_evidence_verified,)),
            j!("write_authority_evidence_verified" => b(durable_policy_ledger_availability_dry_run.write_authority_evidence_verified)),
            j!("ledger_evidence_verified" => b(durable_policy_ledger_availability_dry_run.ledger_evidence_verified)),
            j!("media_write_policy_verified" => b(durable_policy_ledger_availability_dry_run.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(durable_policy_ledger_availability_dry_run.target_region_write_readback_verified)),
            j!("transaction_append_denial_gate_verified" => b(durable_policy_ledger_availability_dry_run.transaction_append_denial_gate_verified,)),
            j!("target_span_verified" => b(durable_policy_ledger_availability_dry_run.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(durable_policy_ledger_availability_dry_run.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_policy_ledger_availability_dry_run
            .test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_policy_ledger_availability_dry_run.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_policy_ledger_availability_dry_run.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_policy_ledger_availability_dry_run.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_policy_ledger_availability_dry_run.durable_append_authority_available)),
            j!("transaction_append_available" => b(durable_policy_ledger_availability_dry_run.transaction_append_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            j!("applies_rollback" => b(false)),
            j!("installs_rollback_state" => b(false)),
            ])),
            j!("durable_audit_policy_availability_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_REASON)),
            j!("dry_run_hash" => sha(durable_audit_policy_availability_dry_run.dry_run_hash)),
            j!("source_audit_policy_availability_hash" => sha(durable_audit_policy_availability_dry_run.source_audit_policy_availability_hash,)),
            j!("source_policy_ledger_availability_dry_run_hash" => sha(durable_audit_policy_availability_dry_run
            .source_policy_ledger_availability_dry_run_hash,)),
            j!("source_policy_ledger_availability_hash" => sha(durable_audit_policy_availability_dry_run.source_policy_ledger_availability_hash,)),
            j!("source_write_authority_availability_hash" => sha(durable_audit_policy_availability_dry_run.source_write_authority_availability_hash,)),
            j!("source_ledger_aware_acceptance_result_hash" => sha(durable_audit_policy_availability_dry_run.source_ledger_aware_acceptance_result_hash,)),
            j!("source_ledger_candidate_hash" => sha(durable_audit_policy_availability_dry_run.source_ledger_candidate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_audit_policy_availability_dry_run.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_audit_policy_availability_dry_run.source_target_region_write_readback_hash,)),
            j!("source_authority_denial_gate_hash" => sha(durable_audit_policy_availability_dry_run.source_authority_denial_gate_hash)),
            j!("source_transaction_append_availability_decision_hash" => sha(durable_audit_policy_availability_dry_run
            .source_transaction_append_availability_decision_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_audit_policy_availability_dry_run.target_start_lba)),
            j!("target_lba_count" => u(durable_audit_policy_availability_dry_run.target_lba_count)),
            j!("target_byte_count" => u(durable_audit_policy_availability_dry_run.target_byte_count)),
            j!("audit_policy_availability_evidence_verified" => b(durable_audit_policy_availability_dry_run.audit_policy_availability_evidence_verified,)),
            j!("policy_ledger_dry_run_evidence_verified" => b(durable_audit_policy_availability_dry_run.policy_ledger_dry_run_evidence_verified)),
            j!("policy_ledger_availability_evidence_verified" => b(durable_audit_policy_availability_dry_run.policy_ledger_availability_evidence_verified,)),
            j!("write_authority_evidence_verified" => b(durable_audit_policy_availability_dry_run.write_authority_evidence_verified)),
            j!("ledger_evidence_verified" => b(durable_audit_policy_availability_dry_run.ledger_evidence_verified)),
            j!("media_write_policy_verified" => b(durable_audit_policy_availability_dry_run.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(durable_audit_policy_availability_dry_run.target_region_write_readback_verified)),
            j!("transaction_append_denial_gate_verified" => b(durable_audit_policy_availability_dry_run.transaction_append_denial_gate_verified)),
            j!("target_span_verified" => b(durable_audit_policy_availability_dry_run.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(durable_audit_policy_availability_dry_run.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_audit_policy_availability_dry_run
            .test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_audit_policy_availability_dry_run.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_audit_policy_availability_dry_run.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_audit_policy_availability_dry_run.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_audit_policy_availability_dry_run.durable_append_authority_available)),
            j!("transaction_append_available" => b(durable_audit_policy_availability_dry_run.transaction_append_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            j!("applies_rollback" => b(false)),
            j!("installs_rollback_state" => b(false)),
            ])),
            j!("durable_append_authority_availability_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_REASON)),
            j!("dry_run_hash" => sha(durable_append_authority_availability_dry_run.dry_run_hash)),
            j!("source_append_authority_availability_hash" => sha(durable_append_authority_availability_dry_run.source_append_authority_availability_hash,)),
            j!("source_audit_policy_availability_dry_run_hash" => sha(durable_append_authority_availability_dry_run
            .source_audit_policy_availability_dry_run_hash,)),
            j!("source_audit_policy_availability_hash" => sha(durable_append_authority_availability_dry_run.source_audit_policy_availability_hash,)),
            j!("source_policy_ledger_availability_dry_run_hash" => sha(durable_append_authority_availability_dry_run
            .source_policy_ledger_availability_dry_run_hash,)),
            j!("source_policy_ledger_availability_hash" => sha(durable_append_authority_availability_dry_run.source_policy_ledger_availability_hash,)),
            j!("source_write_authority_availability_hash" => sha(durable_append_authority_availability_dry_run.source_write_authority_availability_hash,)),
            j!("source_ledger_aware_acceptance_result_hash" => sha(durable_append_authority_availability_dry_run
            .source_ledger_aware_acceptance_result_hash,)),
            j!("source_ledger_candidate_hash" => sha(durable_append_authority_availability_dry_run.source_ledger_candidate_hash)),
            j!("source_policy_preflight_hash" => sha(durable_append_authority_availability_dry_run.source_policy_preflight_hash)),
            j!("source_target_region_write_readback_hash" => sha(durable_append_authority_availability_dry_run.source_target_region_write_readback_hash,)),
            j!("source_authority_denial_gate_hash" => sha(durable_append_authority_availability_dry_run.source_authority_denial_gate_hash,)),
            j!("source_transaction_append_availability_decision_hash" => sha(durable_append_authority_availability_dry_run
            .source_transaction_append_availability_decision_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_append_authority_availability_dry_run.target_start_lba)),
            j!("target_lba_count" => u(durable_append_authority_availability_dry_run.target_lba_count)),
            j!("target_byte_count" => u(durable_append_authority_availability_dry_run.target_byte_count)),
            j!("append_authority_availability_evidence_verified" => b(durable_append_authority_availability_dry_run
            .append_authority_availability_evidence_verified,)),
            j!("audit_policy_dry_run_evidence_verified" => b(durable_append_authority_availability_dry_run.audit_policy_dry_run_evidence_verified,)),
            j!("audit_policy_availability_evidence_verified" => b(durable_append_authority_availability_dry_run
            .audit_policy_availability_evidence_verified,)),
            j!("policy_ledger_dry_run_evidence_verified" => b(durable_append_authority_availability_dry_run.policy_ledger_dry_run_evidence_verified,)),
            j!("policy_ledger_availability_evidence_verified" => b(durable_append_authority_availability_dry_run
            .policy_ledger_availability_evidence_verified,)),
            j!("write_authority_evidence_verified" => b(durable_append_authority_availability_dry_run.write_authority_evidence_verified)),
            j!("ledger_evidence_verified" => b(durable_append_authority_availability_dry_run.ledger_evidence_verified)),
            j!("media_write_policy_verified" => b(durable_append_authority_availability_dry_run.media_write_policy_verified)),
            j!("target_region_write_readback_verified" => b(durable_append_authority_availability_dry_run.target_region_write_readback_verified,)),
            j!("transaction_append_denial_gate_verified" => b(durable_append_authority_availability_dry_run.transaction_append_denial_gate_verified,)),
            j!("target_span_verified" => b(durable_append_authority_availability_dry_run.target_span_verified)),
            j!("audit_rollback_target_ids_verified" => b(durable_append_authority_availability_dry_run.audit_rollback_target_ids_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_append_authority_availability_dry_run
            .test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_append_authority_availability_dry_run.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_append_authority_availability_dry_run.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_append_authority_availability_dry_run.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_append_authority_availability_dry_run.durable_append_authority_available)),
            j!("transaction_append_available" => b(durable_append_authority_availability_dry_run.transaction_append_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            j!("applies_rollback" => b(false)),
            j!("installs_rollback_state" => b(false)),
            ])),
            j!("transaction_append_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_REASON)),
            j!("dry_run_hash" => sha(transaction_append_dry_run.dry_run_hash)),
            j!("source_authority_denial_gate_hash" => sha(transaction_append_dry_run.source_authority_denial_gate_hash)),
            j!("source_transaction_append_availability_decision_hash" => sha(transaction_append_dry_run.source_transaction_append_availability_decision_hash,)),
            j!("source_append_record_hash" => sha(transaction_append_dry_run.source_append_record_hash)),
            j!("source_sector_plan_hash" => sha(transaction_append_dry_run.source_sector_plan_hash)),
            j!("source_target_region_write_readback_hash" => sha(transaction_append_dry_run.source_target_region_write_readback_hash)),
            j!("planned_sector_image_hash" => sha(transaction_append_dry_run.planned_sector_image_hash)),
            j!("readback_sector_image_hash" => sha(transaction_append_dry_run.readback_sector_image_hash)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(transaction_append_dry_run.target_start_lba)),
            j!("target_lba_count" => u(transaction_append_dry_run.target_lba_count)),
            j!("target_byte_count" => u(transaction_append_dry_run.target_byte_count)),
            j!("authority_denial_gate_verified" => b(transaction_append_dry_run.authority_denial_gate_verified)),
            j!("target_span_verified" => b(transaction_append_dry_run.target_span_verified)),
            j!("target_region_write_readback_verified" => b(transaction_append_dry_run.target_region_write_readback_verified)),
            j!("append_image_ready" => b(transaction_append_dry_run.append_image_ready)),
            j!("blocked_by_authority_denial_gate" => b(transaction_append_dry_run.blocked_by_authority_denial_gate)),
            j!("test_infrastructure_media_write_authority_available" => b(transaction_append_dry_run.test_infrastructure_media_write_authority_available)),
            j!("transaction_append_available" => b(transaction_append_dry_run.transaction_append_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("transaction_append_attempted" => b(false)),
            ])),
            j!("target_region_sector_inspection" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(target_region_sector_inspection.status)),
            j!("reason" => s(target_region_sector_inspection.reason)),
            j!("inspection_hash" => sha(target_region_sector_inspection.inspection_hash)),
            j!("source_sector_plan_hash" => sha(target_region_sector_inspection.source_sector_plan_hash)),
            j!("source_target_region_write_readback_hash" => sha(target_region_sector_inspection.source_target_region_write_readback_hash)),
            j!("expected_sector_image_hash" => sha(target_region_sector_inspection.expected_sector_image_hash)),
            j!("sector_image_hash" => sha(target_region_sector_inspection.sector_image_hash)),
            j!("audit_record_image_hash" => sha(target_region_sector_inspection.audit_record_image_hash)),
            j!("rollback_transaction_image_hash" => sha(target_region_sector_inspection.rollback_transaction_image_hash)),
            j!("target_start_lba" => u(target_region_sector_inspection.target_start_lba)),
            j!("target_lba_count" => u(target_region_sector_inspection.target_lba_count)),
            j!("target_byte_count" => u(target_region_sector_inspection.target_byte_count)),
            j!("audit_record_offset" => u(target_region_sector_inspection.audit_record_offset)),
            j!("audit_record_byte_length" => u(target_region_sector_inspection.audit_record_byte_length)),
            j!("rollback_transaction_offset" => u(target_region_sector_inspection.rollback_transaction_offset)),
            j!("rollback_transaction_byte_length" => u(target_region_sector_inspection.rollback_transaction_byte_length)),
            j!("padding_offset" => u(target_region_sector_inspection.padding_offset)),
            j!("padding_byte_length" => u(target_region_sector_inspection.padding_byte_length)),
            j!("label_found" => b(target_region_sector_inspection.label_found)),
            j!("read_attempted" => b(target_region_sector_inspection.read_attempted)),
            j!("read_completed" => b(target_region_sector_inspection.read_completed)),
            j!("sector_hash_verified" => b(target_region_sector_inspection.sector_hash_verified)),
            j!("audit_record_hash_verified" => b(target_region_sector_inspection.audit_record_hash_verified)),
            j!("rollback_transaction_hash_verified" => b(target_region_sector_inspection.rollback_transaction_hash_verified)),
            j!("offsets_verified" => b(target_region_sector_inspection.offsets_verified)),
            j!("padding_zeroed" => b(target_region_sector_inspection.padding_zeroed)),
            j!("target_span_verified" => b(target_region_sector_inspection.target_span_verified)),
            j!("target_region_write_readback_verified" => b(target_region_sector_inspection.target_region_write_readback_verified)),
            j!("inspection_verified" => b(target_region_sector_inspection.inspection_verified)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("installs_rollback_state" => b(false)),
            ])),
            j!("retained_recovery_rollback_inspect_source" => recovery_rollback_inspect_source_reference_value(target_region_sector_inspection)),
            j!("durable_policy_write_authority_decision" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_STATUS)),
            j!("reason" => s(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_REASON)),
            j!("decision_hash" => sha(durable_policy_write_authority_decision.decision_hash)),
            j!("source_durable_append_authority_availability_dry_run_hash" => sha(durable_policy_write_authority_decision
            .source_durable_append_authority_availability_dry_run_hash,)),
            j!("source_transaction_append_dry_run_hash" => sha(durable_policy_write_authority_decision.source_transaction_append_dry_run_hash)),
            j!("source_target_region_sector_inspection_hash" => sha(durable_policy_write_authority_decision.source_target_region_sector_inspection_hash,)),
            j!("source_write_authority_availability_hash" => sha(durable_policy_write_authority_decision.source_write_authority_availability_hash,)),
            j!("source_audit_policy_availability_hash" => sha(durable_policy_write_authority_decision.source_audit_policy_availability_hash)),
            j!("source_durable_append_authority_availability_hash" => sha(durable_policy_write_authority_decision
            .source_durable_append_authority_availability_hash,)),
            j!("source_authority_denial_gate_hash" => sha(durable_policy_write_authority_decision.source_authority_denial_gate_hash)),
            j!("source_transaction_append_availability_decision_hash" => sha(durable_policy_write_authority_decision
            .source_transaction_append_availability_decision_hash,)),
            j!("audit_ledger_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
            j!("rollback_store_target_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("target_start_lba" => u(durable_policy_write_authority_decision.target_start_lba)),
            j!("target_lba_count" => u(durable_policy_write_authority_decision.target_lba_count)),
            j!("target_byte_count" => u(durable_policy_write_authority_decision.target_byte_count)),
            j!("transaction_append_dry_run_verified" => b(durable_policy_write_authority_decision.transaction_append_dry_run_verified)),
            j!("target_region_sector_inspection_verified" => b(durable_policy_write_authority_decision.target_region_sector_inspection_verified)),
            j!("write_authority_evidence_verified" => b(durable_policy_write_authority_decision.write_authority_evidence_verified)),
            j!("audit_policy_availability_evidence_verified" => b(durable_policy_write_authority_decision.audit_policy_availability_evidence_verified,)),
            j!("durable_append_authority_availability_evidence_verified" => b(durable_policy_write_authority_decision
            .durable_append_authority_availability_evidence_verified,)),
            j!("target_span_verified" => b(durable_policy_write_authority_decision.target_span_verified)),
            j!("test_infrastructure_media_write_authority_available" => b(durable_policy_write_authority_decision
            .test_infrastructure_media_write_authority_available,)),
            j!("write_authority_available" => b(durable_policy_write_authority_decision.write_authority_available)),
            j!("durable_policy_ledger_available" => b(durable_policy_write_authority_decision.durable_policy_ledger_available)),
            j!("durable_audit_policy_available" => b(durable_policy_write_authority_decision.durable_audit_policy_available)),
            j!("durable_append_authority_available" => b(durable_policy_write_authority_decision.durable_append_authority_available)),
            j!("transaction_append_available" => b(durable_policy_write_authority_decision.transaction_append_available)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("authorizes_transaction_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            j!("applies_rollback" => b(false)),
            ])),
            j!("target_region_write_readback_dry_run" => inline(vec![
            j!("schema" => s(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(target_region_write.status)),
            j!("reason" => s(target_region_write.reason)),
            j!("dry_run_hash" => sha(target_region_write.dry_run_hash)),
            j!("source_sector_plan_hash" => sha(target_region_write.source_sector_plan_hash)),
            j!("source_policy_preflight_hash" => sha(target_region_write.source_policy_preflight_hash)),
            j!("planned_sector_image_hash" => sha(target_region_write.planned_sector_image_hash)),
            j!("readback_sector_image_hash" => sha(target_region_write.readback_sector_image_hash)),
            j!("target_region_id" => s(ahci::AUDIT_ROLLBACK_TARGET_REGION_ID)),
            j!("target_start_lba" => u(target_region_write.target_start_lba)),
            j!("target_lba_count" => u(target_region_write.target_lba_count)),
            j!("target_byte_count" => u(target_region_write.target_byte_count)),
            j!("label_found" => b(target_region_write.label_found)),
            j!("target_range_ready" => b(target_region_write.target_range_ready)),
            j!("test_infrastructure_media_write_authority_available" => b(target_region_write.test_infrastructure_media_write_authority_available)),
            j!("write_attempted" => b(target_region_write.write_attempted)),
            j!("write_completed" => b(target_region_write.write_completed)),
            j!("readback_completed" => b(target_region_write.readback_completed)),
            j!("readback_matches_planned_image" => b(target_region_write.readback_matches_planned_image)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("installs_rollback_state" => b(false)),
            ])),
            j!("storage_authority_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID)),
            j!("append_target_owner_id" => s(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID)),
            j!("transaction_writer_readiness_id" => s(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID)),
            j!("audit_ledger_writer_fact_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
            j!("rollback_store_writer_fact_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
            j!("scratch_write_readback_verified" => b(durable_append_preflight.scratch_write_readback_verified)),
            j!("scratch_used_as_durable_authority" => b(false)),
            j!("durable_audit_writer_available" => b(durable_append_preflight.durable_audit_writer_available)),
            j!("rollback_store_writer_available" => b(durable_append_preflight.rollback_store_writer_available)),
            j!("transaction_append_writer_available" => b(durable_append_preflight.transaction_append_writer_available)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            ])),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            ])),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("dry_run_evaluated" => b(true)),
            j!("authorizes_append" => b(false)),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            ])),
            j!("block_write_path_authority_gate" => inline(vec![
            j!("schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_SCHEMA)),
            j!("id" => s(rollback_storage_layout::AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_ID)),
            j!("storage_authority_id" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID)),
            j!("status" => s(foundation.block_write_path_gate_status)),
            j!("reason" => s(foundation.block_write_path_reason)),
            j!("available" => b(foundation.block_write_path_available)),
            j!("read_only_block_driver_id" => s(foundation.read_only_block_driver_id)),
            j!("read_only_block_driver_available" => b(foundation.read_only_block_driver_available)),
            j!("partition_inventory_available" => b(foundation.partition_inventory_available)),
            j!("partition_inventory_scheme" => s(foundation.partition_inventory_scheme)),
            j!("scratch_block_write_authority_available" => b(foundation.scratch_block_write_authority_available)),
            j!("scratch_block_write_authority_id" => s(foundation.scratch_block_write_authority_id)),
            j!("scratch_region_within_device_bounds" => b(foundation.scratch_region_within_device_bounds)),
            j!("scratch_region_no_boot_or_partition_metadata_overlap" => b(foundation.scratch_region_no_boot_or_partition_metadata_overlap)),
            j!("authorizes_media_write" => b(false)),
            j!("authorizes_append" => b(false)),
            j!("writes_enabled" => b(false)),
            j!("write_attempted" => b(false)),
            ])),
            j!("append_target" => inline(vec![
            j!("id" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID)),
            j!("schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
            j!("available" => b(foundation.rollback_transaction_append_available)),
            ])),
            j!("storage_layout" => inline(vec![
            j!("schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_LAYOUT_SCHEMA)),
            j!("id" => s(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_LAYOUT_ID)),
            j!("status" => s(foundation.storage_layout_status)),
            j!("reason" => s(foundation.storage_layout_reason)),
            j!("available" => b(foundation.storage_layout_available)),
            ])),
            j!("append_engine" => inline(vec![
            j!("schema" => s("raios.rollback_store_transaction_engine.v0")),
            j!("id" => s("append_engine.rollback_store.current_boot")),
            j!("status" => s(foundation.append_engine_status)),
            j!("reason" => s(foundation.append_engine_reason)),
            j!("available" => b(foundation.append_engine_available)),
            ])),
            j!("append_contract" => inline(vec![
            j!("schema" => s("raios.rollback_store_transaction_envelope.v0")),
            j!("id" => s(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID)),
            j!("status" => s(foundation.rollback_transaction_envelope_status)),
            j!("reason" => s(foundation.rollback_transaction_envelope_reason)),
            j!("available" => b(foundation.rollback_transaction_envelope_available)),
            ])),
            j!("transaction_writer" => inline(vec![
            j!("owner" => s(rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER)),
            j!("status" => s(foundation.append_contract_status)),
            j!("reason" => s(foundation.append_contract_reason)),
            j!("available" => b(foundation.transaction_writer_available)),
            ])),
            ])),
            j!("unavailable_authorities" => inline(vec![
            j!("transaction_writer" => b(!foundation.transaction_writer_available)),
            j!("durable_audit_store" => b(!foundation.durable_audit_store_available)),
            j!("rollback_store" => b(!foundation.rollback_store_available)),
            j!("rollback_transaction_append" => b(!foundation.rollback_transaction_append_available)),
            ])),
            j!("side_effects" => inline(vec![
            j!("writes_durable_audit_log" => b(false)),
            j!("writes_rollback_store" => b(false)),
            j!("appends_rollback_transaction" => b(false)),
            j!("installs_rollback_plan" => b(false)),
            j!("applies_rollback" => b(false)),
            ])),
        ])
    } else {
        V::Null
    }
}

pub(crate) fn recovery_rollback_inspection_evidence(
    snapshot: Snapshot,
) -> Option<(
    RollbackTargetRegionWriteReadbackDryRun,
    RollbackTargetRegionSectorInspection,
)> {
    let probation = snapshot.hot_swap_probation?;
    let foundation = rollback_writer_storage_foundation();
    let append_record = hello_rollback_append_record_dry_run(snapshot, probation, foundation);
    let sector_plan = hello_rollback_append_sector_plan_dry_run(snapshot, probation, append_record);
    let target_region_media_write_policy_preflight =
        hello_target_region_media_write_policy_preflight(foundation);
    let target_region_write = hello_rollback_target_region_write_readback_dry_run_from_materializer(
        sector_plan,
        foundation,
        target_region_media_write_policy_preflight,
    );
    let sector_inspection = hello_rollback_target_region_sector_inspection(
        append_record,
        sector_plan,
        target_region_write,
    );
    Some((target_region_write, sector_inspection))
}

fn applied_rollback_inspection_evidence(
    snapshot: Snapshot,
) -> Option<(
    AppliedRollbackRecord,
    RollbackTargetRegionWriteReadbackDryRun,
    RollbackTargetRegionSectorInspection,
)> {
    let record = snapshot.applied_rollback?;
    let inspection = record.target_region_sector_inspection;
    if !raios_core::scoped_rollback_apply::applied_rollback_inspect_evidence_retained(
        Some(record.rollback_transaction_hash),
        Some(record.write_readback_hash),
        Some(record.inspection_hash),
        inspection.rollback_transaction_image_hash,
        inspection.source_target_region_write_readback_hash,
        inspection.inspection_hash,
    ) {
        return None;
    }
    if record.inspected_rollback_transaction_hash != inspection.rollback_transaction_image_hash
        || record.audit_record_hash != inspection.audit_record_image_hash
    {
        return None;
    }
    Some((
        record,
        record.target_region_write_readback,
        record.target_region_sector_inspection,
    ))
}

pub(crate) fn recovery_rollback_materialization_evidence(
    snapshot: Snapshot,
) -> Option<(
    RollbackAppendRecordDryRun,
    RollbackAppendSectorPlanDryRun,
    RollbackTargetRegionWriteReadbackDryRun,
)> {
    let probation = snapshot.hot_swap_probation?;
    let foundation = rollback_writer_storage_foundation();
    let append_record = hello_rollback_append_record_dry_run(snapshot, probation, foundation);
    let sector_plan = hello_rollback_append_sector_plan_dry_run(snapshot, probation, append_record);
    let target_region_media_write_policy_preflight =
        hello_target_region_media_write_policy_preflight(foundation);
    let target_region_write = hello_rollback_target_region_write_readback_dry_run(
        snapshot,
        probation,
        sector_plan,
        foundation,
        target_region_media_write_policy_preflight,
    );
    Some((append_record, sector_plan, target_region_write))
}

pub(crate) fn materialized_target_region_sector_available(
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> bool {
    target_region_write.test_infrastructure_media_write_authority_available
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image
}

fn target_region_write_readback_value(
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> V<'static> {
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA),
        ),
        j!("id" => s(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID),
        ),
        j!("status" => s(target_region_write.status)),
        j!("reason" => s(target_region_write.reason)),
        j!("dry_run_hash" => sha(target_region_write.dry_run_hash)),
        j!("source_sector_plan_hash" => sha(target_region_write.source_sector_plan_hash),
        ),
        j!("source_policy_preflight_hash" => sha(target_region_write.source_policy_preflight_hash),
        ),
        j!("planned_sector_image_hash" => sha(target_region_write.planned_sector_image_hash),
        ),
        j!("readback_sector_image_hash" => sha(target_region_write.readback_sector_image_hash),
        ),
        j!("target_region_id" => s(ahci::AUDIT_ROLLBACK_TARGET_REGION_ID)),
        j!("target_start_lba" => u(target_region_write.target_start_lba)),
        j!("target_lba_count" => u(target_region_write.target_lba_count)),
        j!("target_byte_count" => u(target_region_write.target_byte_count),
        ),
        j!("label_found" => b(target_region_write.label_found)),
        j!("target_range_ready" => b(target_region_write.target_range_ready),
        ),
        j!("test_infrastructure_media_write_authority_available" => b(target_region_write.test_infrastructure_media_write_authority_available),
        ),
        j!("write_attempted" => b(target_region_write.write_attempted)),
        j!("write_completed" => b(target_region_write.write_completed)),
        j!("readback_completed" => b(target_region_write.readback_completed),
        ),
        j!("readback_matches_planned_image" => b(target_region_write.readback_matches_planned_image),
        ),
        j!("authorizes_media_write" => b(false)),
        j!("authorizes_append" => b(false)),
        j!("writes_durable_audit_log" => b(false)),
        j!("writes_rollback_store" => b(false)),
        j!("appends_rollback_transaction" => b(false)),
        j!("installs_rollback_state" => b(false)),
        j!("applies_rollback" => b(false)),
    ])
}

fn target_region_sector_inspection_value(
    inspection: RollbackTargetRegionSectorInspection,
) -> V<'static> {
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_SCHEMA),
        ),
        j!("id" => s(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("status" => s(inspection.status)),
        j!("reason" => s(inspection.reason)),
        j!("inspection_hash" => sha(inspection.inspection_hash)),
        j!("source_sector_plan_hash" => sha(inspection.source_sector_plan_hash),
        ),
        j!("source_target_region_write_readback_hash" => sha(inspection.source_target_region_write_readback_hash),
        ),
        j!("expected_sector_image_hash" => sha(inspection.expected_sector_image_hash),
        ),
        j!("sector_image_hash" => sha(inspection.sector_image_hash)),
        j!("audit_record_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA),
        ),
        j!("audit_record_image_hash" => sha(inspection.audit_record_image_hash),
        ),
        j!("rollback_transaction_schema" => s(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA),
        ),
        j!("rollback_transaction_image_hash" => sha(inspection.rollback_transaction_image_hash),
        ),
        j!("target_start_lba" => u(inspection.target_start_lba)),
        j!("target_lba_count" => u(inspection.target_lba_count)),
        j!("target_byte_count" => u(inspection.target_byte_count)),
        j!("audit_record_offset" => u(inspection.audit_record_offset)),
        j!("audit_record_byte_length" => u(inspection.audit_record_byte_length),
        ),
        j!("rollback_transaction_offset" => u(inspection.rollback_transaction_offset),
        ),
        j!("rollback_transaction_byte_length" => u(inspection.rollback_transaction_byte_length),
        ),
        j!("padding_offset" => u(inspection.padding_offset)),
        j!("padding_byte_length" => u(inspection.padding_byte_length)),
        j!("label_found" => b(inspection.label_found)),
        j!("read_attempted" => b(inspection.read_attempted)),
        j!("read_completed" => b(inspection.read_completed)),
        j!("sector_hash_verified" => b(inspection.sector_hash_verified)),
        j!("audit_record_hash_verified" => b(inspection.audit_record_hash_verified),
        ),
        j!("rollback_transaction_hash_verified" => b(inspection.rollback_transaction_hash_verified),
        ),
        j!("offsets_verified" => b(inspection.offsets_verified)),
        j!("padding_zeroed" => b(inspection.padding_zeroed)),
        j!("target_span_verified" => b(inspection.target_span_verified)),
        j!("target_region_write_readback_verified" => b(inspection.target_region_write_readback_verified),
        ),
        j!("inspection_verified" => b(inspection.inspection_verified)),
        j!("authorizes_media_write" => b(false)),
        j!("authorizes_append" => b(false)),
        j!("writes_durable_audit_log" => b(false)),
        j!("writes_rollback_store" => b(false)),
        j!("appends_rollback_transaction" => b(false)),
        j!("installs_rollback_state" => b(false)),
        j!("applies_rollback" => b(false)),
    ])
}

pub(crate) fn emit_recovery_rollback_inspect_source_reference_inline(
    inspection: RollbackTargetRegionSectorInspection,
) {
    emit_inline_value(recovery_rollback_inspect_source_reference_value(inspection));
}

fn recovery_rollback_inspect_source_reference_value(
    inspection: RollbackTargetRegionSectorInspection,
) -> V<'static> {
    let state = recovery_rollback_inspect_source_reference_state(inspection);
    let reference = state.reference;
    let source_matches_sector_inspection = state.ram_audit_validated;
    inline(vec![
        j!("schema" => s(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SCHEMA),
        ),
        j!("id" => s(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("status" => s(state.status)),
        j!("reason" => s(state.reason)),
        j!("source_method" => s("recovery.rollback_inspect")),
        j!("source_event_id" => event_opt(reference.map(|reference| reference.event_id)),
        ),
        j!("source_audit_event_id" => event_opt(reference.map(|reference| reference.audit_event_id)),
        ),
        j!("source_available" => b(source_matches_sector_inspection)),
        j!("source_matches_sector_inspection" => b(source_matches_sector_inspection),
        ),
        j!("source_event_retained" => b(state.source_event_retained)),
        j!("source_audit_event_retained" => b(state.audit_event_retained)),
        j!("ram_audit_status" => s(state.ram_audit_status)),
        j!("ram_audit_reason" => s(state.ram_audit_reason)),
        j!("ram_audit_validated" => b(state.ram_audit_validated)),
        j!("reference_hash" => sha_opt(reference.map(|reference| reference.reference_hash)),
        ),
        j!("source_inspection_hash" => sha_opt(reference.map(|reference| reference.inspection_hash)),
        ),
        j!("target_region_sector_inspection_hash" => sha(inspection.inspection_hash),
        ),
        j!("source_sector_plan_hash" => sha_opt(reference.map(|reference| reference.source_sector_plan_hash)),
        ),
        j!("source_target_region_write_readback_hash" => sha_opt(reference.map(|reference| reference.source_target_region_write_readback_hash)),
        ),
        j!("authorizes_rollback_apply" => b(false)),
    ])
}

fn append_record_dry_run_summary_value(record: Option<RollbackAppendRecordDryRun>) -> V<'static> {
    let Some(record) = record else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_SCHEMA)),
        j!("id" => s(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_ID)),
        j!("dry_run_hash" => sha(record.dry_run_hash)),
        j!("audit_record_image_hash" => sha(record.audit_record_image_hash),
        ),
        j!("rollback_transaction_image_hash" => sha(record.rollback_transaction_image_hash),
        ),
        j!("audit_record_byte_length" => u(record.audit_record_byte_length),
        ),
        j!("rollback_transaction_byte_length" => u(record.rollback_transaction_byte_length),
        ),
        j!("total_record_byte_length" => u(record.total_record_byte_length),
        ),
        j!("target_start_lba" => u(record.target_start_lba)),
        j!("target_lba_count" => u(record.target_lba_count)),
        j!("target_byte_count" => u(record.target_byte_count)),
        j!("target_range_ready" => b(record.target_range_ready)),
    ])
}

fn sector_plan_dry_run_summary_value(record: Option<RollbackAppendSectorPlanDryRun>) -> V<'static> {
    let Some(record) = record else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_SCHEMA),
        ),
        j!("id" => s(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_ID)),
        j!("plan_hash" => sha(record.plan_hash)),
        j!("sector_image_hash" => sha(record.sector_image_hash)),
        j!("sector_size_bytes" => u(record.sector_size_bytes)),
        j!("audit_record_offset" => u(record.audit_record_offset)),
        j!("audit_record_byte_length" => u(record.audit_record_byte_length),
        ),
        j!("rollback_transaction_offset" => u(record.rollback_transaction_offset),
        ),
        j!("rollback_transaction_byte_length" => u(record.rollback_transaction_byte_length),
        ),
        j!("padding_offset" => u(record.padding_offset)),
        j!("padding_byte_length" => u(record.padding_byte_length)),
        j!("target_start_lba" => u(record.target_start_lba)),
        j!("target_lba_count" => u(record.target_lba_count)),
        j!("target_byte_count" => u(record.target_byte_count)),
        j!("target_range_ready" => b(record.target_range_ready)),
    ])
}

fn applied_rollback_authority_record_value(record: AppliedRollbackRecord) -> V<'static> {
    inline(vec![
        j!("schema" => s("raios.rollback_transaction.v0")),
        j!("source_method" => s("service.rollback_apply")),
        j!("source_event_id" => event_opt(Some(record.event_id))),
        j!("source_audit_event_id" => event_opt(Some(record.audit_event_id))),
        j!("status" => s(HELLO_ROLLBACK_APPLY_APPLIED_STATUS)),
        j!("rollback_transaction_hash" => sha(record.rollback_transaction_hash)),
        j!("write_readback_hash" => sha(record.write_readback_hash)),
        j!("inspection_hash" => sha(record.inspection_hash)),
        j!("audit_record_hash" => sha(record.audit_record_hash)),
        j!("inspected_rollback_transaction_hash" => sha(record.inspected_rollback_transaction_hash)),
        j!("authorized_append_hash" => sha(record.authorized_append_hash)),
        j!("scope_decision_hash" => sha(record.scope_decision_hash)),
        j!("target_region_write_readback_hash" => sha(record.target_region_write_readback.dry_run_hash)),
        j!("target_region_sector_inspection_hash" => sha(record.target_region_sector_inspection.inspection_hash)),
    ])
}

fn recovery_materialize_denied_surfaces_value() -> V<'static> {
    object(vec![
        j!("mutates_service_state" => b(false)),
        j!("descriptor_mutation" => s("not_attempted")),
        j!("generation_mutation" => s("not_attempted")),
        j!("running_state_mutation" => s("not_attempted")),
        j!("ram_only_state_mutation" => s("not_attempted")),
        j!("authorizes_media_write" => b(false)),
        j!("authorizes_append" => b(false)),
        j!("authorizes_transaction_append" => b(false)),
        j!("writes_durable_audit_log" => b(false)),
        j!("writes_rollback_store" => b(false)),
        j!("appends_rollback_transaction" => b(false)),
        j!("applies_rollback" => b(false)),
        j!("installs_rollback_state" => b(false)),
        j!("persistence" => s("denied")),
        j!("external_artifact_load" => s("denied")),
        j!("candidate_artifact_execution" => s("denied")),
        j!("executable_mapping" => s("denied")),
        j!("provider_auto_load" => s("denied")),
        j!("broad_mutation" => s("denied")),
    ])
}

fn recovery_inspect_denied_surfaces_value() -> V<'static> {
    object(vec![
        j!("authorizes_media_write" => b(false)),
        j!("authorizes_append" => b(false)),
        j!("authorizes_transaction_append" => b(false)),
        j!("writes_durable_audit_log" => b(false)),
        j!("writes_rollback_store" => b(false)),
        j!("appends_rollback_transaction" => b(false)),
        j!("applies_rollback" => b(false)),
        j!("installs_rollback_state" => b(false)),
        j!("persistence" => s("denied")),
        j!("external_artifact_load" => s("denied")),
        j!("candidate_artifact_execution" => s("denied")),
        j!("executable_mapping" => s("denied")),
        j!("provider_auto_load" => s("denied")),
        j!("broad_mutation" => s("denied")),
    ])
}

fn probation_target_value(record: Option<HelloHotSwapProbationRecord>) -> V<'static> {
    let Some(record) = record else {
        return V::Null;
    };
    inline(vec![
        j!("version" => s(record.previous_version)),
        j!("descriptor_id" => s(record.previous_descriptor_id)),
        j!("descriptor_source_hash" => sha(record.previous_descriptor_source_hash),
        ),
        j!("artifact_identity_id" => s(record.previous_artifact_identity_id),
        ),
        j!("artifact_identity_hash" => sha(record.previous_artifact_identity_hash),
        ),
        j!("generation" => u(record.previous_generation)),
        j!("state_hash" => sha(record.previous_state_hash)),
        j!("state_counter" => u(record.previous_state_counter)),
    ])
}

fn probation_candidate_value(record: Option<HelloHotSwapProbationRecord>) -> V<'static> {
    let Some(record) = record else {
        return V::Null;
    };
    inline(vec![
        j!("version" => s(record.new_version)),
        j!("descriptor_id" => s(record.new_descriptor_id)),
        j!("descriptor_source_hash" => sha(record.new_descriptor_source_hash),
        ),
        j!("artifact_identity_id" => s(record.new_artifact_identity_id)),
        j!("artifact_identity_hash" => sha(record.new_artifact_identity_hash),
        ),
        j!("generation" => u(record.new_generation)),
        j!("state_hash" => sha(record.new_state_hash)),
        j!("state_counter" => u(record.new_state_counter)),
    ])
}

fn rollback_preview_denied_surfaces_value() -> V<'static> {
    object(vec![
        j!("mutates_service_state" => b(false)),
        j!("applies_rollback" => b(false)),
        j!("installs_rollback_plan" => b(false)),
        j!("persistent_install" => s("denied")),
        j!("durable_audit_write" => s("denied")),
        j!("external_artifact_load" => s("denied")),
        j!("candidate_artifact_execution" => s("denied")),
        j!("executable_mapping" => s("denied")),
        j!("provider_auto_load" => s("denied")),
        j!("broad_mutation" => s("denied")),
    ])
}

pub(crate) fn emit_recovery_rollback_materialize_dry_run_response(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let evidence = recovery_rollback_materialization_evidence(snapshot);
    let materialized_sector_evidence_available = evidence
        .map(|(_, _, target_region_write)| {
            materialized_target_region_sector_available(target_region_write)
        })
        .unwrap_or(false);
    let status = if materialized_sector_evidence_available {
        HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_STATUS
    } else if !snapshot.loaded {
        "service_not_loaded"
    } else if snapshot.hot_swap_probation.is_none() {
        "hot_swap_probation_missing"
    } else {
        "target_region_sector_materialization_failed"
    };
    let reason = if materialized_sector_evidence_available {
        "target_region_test_sector_written_and_read_back_current_boot"
    } else if !snapshot.loaded {
        "service_not_loaded"
    } else if snapshot.hot_swap_probation.is_none() {
        "hot_swap_probation_missing"
    } else {
        "target_region_write_readback_unavailable"
    };

    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => s(HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_SCHEMA),
            ),
            j!("id" => s(HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("read_only" => b(false)),
            j!("test_infrastructure" => b(true)),
            j!("event_id" => event_opt(Some(event_id))),
            j!("audit_event_id" => event_opt(Some(event_id))),
            j!("status" => s(status)),
            j!("reason" => s(reason)),
            j!("service_id" => s(SERVICE_ID)),
            j!("requested_capability" => s(HELLO_SERVICE_DESCRIPTOR.rollback_materialize_capability),
            ),
            j!("active_generation" => u(snapshot.generation)),
            j!("source_probation" => hello_hot_swap_probation_value(snapshot.hot_swap_probation),
            ),
            j!("append_record_dry_run" => append_record_dry_run_summary_value(
                    evidence.map(|(append_record, _, _)| append_record),
                ),
            ),
            j!("sector_plan_dry_run" => sector_plan_dry_run_summary_value(evidence.map(|(_, sector_plan, _)| sector_plan)),
            ),
            j!("target_region_write_readback" => evidence
                    .map(|(_, _, target_region_write)| {
                        target_region_write_readback_value(target_region_write)
                    })
                    .unwrap_or(V::Null),
            ),
            j!("materialized_sector_evidence_available" => b(materialized_sector_evidence_available),
            ),
            j!("denied_surfaces" => recovery_materialize_denied_surfaces_value(),
            ),
        ],
        6,
    );
    end_response(method);
}

pub(crate) fn emit_recovery_rollback_inspect_response(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let live_evidence = recovery_rollback_inspection_evidence(snapshot);
    let applied_evidence = if live_evidence.is_none() {
        applied_rollback_inspection_evidence(snapshot)
    } else {
        None
    };
    let applied_authority_record = applied_evidence.map(|(record, _, _)| record);
    let evidence = live_evidence.or_else(|| {
        applied_evidence
            .map(|(_, target_region_write, inspection)| (target_region_write, inspection))
    });
    let inspection_available = evidence
        .map(|(_, inspection)| inspection.inspection_verified)
        .unwrap_or(false);
    let materialized_sector_evidence_available = evidence
        .map(|(target_region_write, _)| {
            materialized_target_region_sector_available(target_region_write)
        })
        .unwrap_or(false);
    let applied_transaction_inspected = applied_authority_record.is_some()
        && inspection_available
        && materialized_sector_evidence_available;
    let status = if applied_transaction_inspected {
        raios_core::scoped_rollback_apply::ROLLBACK_INSPECT_APPLIED_TRANSACTION_STATUS
    } else if inspection_available {
        HELLO_RECOVERY_ROLLBACK_INSPECT_STATUS
    } else if !snapshot.loaded {
        "service_not_loaded"
    } else if snapshot.hot_swap_probation.is_none() {
        "hot_swap_probation_missing"
    } else if !materialized_sector_evidence_available {
        "materialized_target_region_sector_missing"
    } else {
        "target_region_sector_inspection_missing"
    };
    let reason = if applied_transaction_inspected {
        raios_core::scoped_rollback_apply::ROLLBACK_INSPECT_APPLIED_TRANSACTION_REASON
    } else if inspection_available {
        "target_region_sector_read_parsed_current_boot"
    } else if !snapshot.loaded {
        "service_not_loaded"
    } else if snapshot.hot_swap_probation.is_none() {
        "hot_swap_probation_missing"
    } else if !materialized_sector_evidence_available {
        "run_recovery_rollback_materialize_dry_run_before_read_only_inspection"
    } else {
        "target_region_sector_read_or_parse_failed"
    };
    if let Some((_, inspection)) = live_evidence {
        retain_recovery_rollback_inspect_source_reference(event_id, inspection);
    }

    begin_response(method);
    let mut fields = vec![
        j!("schema" => s(HELLO_RECOVERY_ROLLBACK_INSPECT_SCHEMA)),
        j!("id" => s(HELLO_RECOVERY_ROLLBACK_INSPECT_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("read_only" => b(true)),
        j!("event_id" => event_opt(Some(event_id))),
        j!("status" => s(status)),
        j!("reason" => s(reason)),
        j!("service_id" => s(SERVICE_ID)),
        j!("requested_capability" => s(HELLO_SERVICE_DESCRIPTOR.rollback_inspect_capability),
        ),
        j!("active_generation" => u(snapshot.generation)),
        j!("source_probation" => hello_hot_swap_probation_value(snapshot.hot_swap_probation),
        ),
        j!("materialized_sector_evidence_available" => b(materialized_sector_evidence_available),
        ),
        j!("inspection_available" => b(inspection_available)),
        j!("target_region_write_readback" => evidence
                .map(|(target_region_write, _)| {
                    target_region_write_readback_value(target_region_write)
                })
                .unwrap_or(V::Null),
        ),
        j!("target_region_sector_inspection" => evidence
                .map(|(_, inspection)| target_region_sector_inspection_value(inspection))
                .unwrap_or(V::Null),
        ),
        j!("retained_recovery_rollback_inspect_source" => evidence
                .map(|(_, inspection)| {
                    recovery_rollback_inspect_source_reference_value(inspection)
                })
                .unwrap_or(V::Null),
        ),
        j!("denied_surfaces" => recovery_inspect_denied_surfaces_value()),
    ];
    if let Some(record) = applied_authority_record {
        fields.push(
            j!("applied_authority_record" => applied_rollback_authority_record_value(record)),
        );
    }
    emit_record_fields(fields, 6);
    end_response(method);
}

fn rollback_apply_required_preview_value(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) -> V<'static> {
    let Some(probation) = probation else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(HELLO_ROLLBACK_PREVIEW_SCHEMA)),
        j!("id" => s(HELLO_ROLLBACK_PREVIEW_ID)),
        j!("status" => s(HELLO_ROLLBACK_PREVIEW_STATUS)),
        j!("preview_hash" => sha(hello_rollback_preview_hash(snapshot, probation)),
        ),
    ])
}

fn rollback_apply_denial_hash_value(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
    retained_sources: Option<(
        RollbackDurablePolicyWriteAuthorityDecision,
        RecoveryRollbackInspectSourceReferenceState,
    )>,
) -> V<'static> {
    let Some(probation) = probation else {
        return V::Null;
    };
    let hash = if let Some((decision, source)) = retained_sources {
        hello_rollback_apply_denial_hash_with_retained_sources(
            snapshot,
            probation,
            Some(decision),
            Some(source),
        )
    } else {
        hello_rollback_apply_denial_hash(snapshot, probation)
    };
    sha(hash)
}

fn rollback_apply_denied_surfaces_value() -> V<'static> {
    object(vec![
        j!("mutates_service_state" => b(false)),
        j!("applies_rollback" => b(false)),
        j!("descriptor_mutation" => s("not_attempted")),
        j!("generation_mutation" => s("not_attempted")),
        j!("running_state_mutation" => s("not_attempted")),
        j!("ram_only_state_mutation" => s("not_attempted")),
        j!("persistent_install" => s("denied")),
        j!("durable_audit_write" => s("denied")),
        j!("external_artifact_load" => s("denied")),
        j!("candidate_artifact_execution" => s("denied")),
        j!("executable_mapping" => s("denied")),
        j!("provider_auto_load" => s("denied")),
        j!("broad_mutation" => s("denied")),
    ])
}

pub(crate) fn emit_rollback_apply_denied(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let probation = snapshot.hot_swap_probation;
    let retained_denial_sources = probation
        .map(|probation| hello_rollback_apply_retained_denial_sources(snapshot, probation));
    begin_error(method);
    emit_record_fields_trailing_comma(
        vec![
            j!("method" => s(method)),
            j!("event_id" => event_opt(Some(event_id))),
            j!("audit_event_id" => event_opt(Some(event_id))),
            j!("code" => s("capability_denied")),
            j!("schema" => s(HELLO_ROLLBACK_APPLY_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_APPLY_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("status" => s(HELLO_ROLLBACK_APPLY_STATUS)),
            j!("reason" => s(if probation.is_some() {
                    "rollback_apply_authority_missing"
                } else if snapshot.loaded {
                    "rollback_preview_or_probation_missing"
                } else {
                    "service_not_loaded"
                }),
            ),
            j!("message" => s("hello rollback apply is denied until rollback apply authority, durable audit, and rollback transaction evidence exist")),
            j!("service_id" => s(SERVICE_ID)),
            j!("active_generation" => u(snapshot.generation)),
            j!("active_descriptor_id" => s(snapshot.load_descriptor.id)),
            j!("current_state" => hello_state_value(snapshot)),
            j!("source_probation" => hello_hot_swap_probation_value(probation)),
            j!("required_preview" => rollback_apply_required_preview_value(snapshot, probation)),
            j!("rollback_apply_hash" => rollback_apply_denial_hash_value(snapshot, probation, retained_denial_sources),
            ),
            j!("source_durable_policy_write_authority_decision_hash" => retained_denial_sources
                    .map(|(decision, _)| sha(decision.decision_hash))
                    .unwrap_or(V::Null),
            ),
            j!("source_recovery_rollback_inspect_source_reference_hash" => retained_denial_sources
                    .map(|(_, source)| sha_opt(source.reference.map(|reference| reference.reference_hash)))
                    .unwrap_or(V::Null),
            ),
            j!("retained_durable_policy_write_authority_decision_verified" => b(retained_denial_sources
                    .map(|(decision, _)| {
                        decision.transaction_append_dry_run_verified
                            && decision.target_region_sector_inspection_verified
                            && decision.write_authority_evidence_verified
                            && decision.audit_policy_availability_evidence_verified
                            && decision.durable_append_authority_availability_evidence_verified
                            && decision.target_span_verified
                    })
                    .unwrap_or(false)),
            ),
            j!("retained_recovery_rollback_inspect_source_reference_validated" => b(retained_denial_sources
                    .map(|(_, source)| source.ram_audit_validated)
                    .unwrap_or(false)),
            ),
            j!("rollback_transaction_preflight" => rollback_transaction_preflight_value(snapshot, probation),
            ),
            j!("rollback_write_authority_gate" => rollback_write_authority_gate_value(snapshot, probation),
            ),
            j!("rollback_append_intent_gate" => rollback_append_intent_gate_value(snapshot, probation),
            ),
            j!("rollback_payload_envelope_gate" => rollback_payload_envelope_gate_value(snapshot, probation),
            ),
        ],
        4,
    );
    emit_record_fields_trailing_comma(
        vec![j!("rollback_transaction_writer_storage_authority_gate" =>
            rollback_transaction_writer_storage_authority_gate_value(snapshot, probation)
        )],
        4,
    );
    emit_record_fields(
        vec![
            j!("rollback_target" => probation_target_value(probation)),
            j!("current_candidate" => probation_candidate_value(probation)),
            j!("state_migration" => hello_state_migration_value(snapshot.state_migration),
            ),
            j!("required" => V::Array(vec![
                    s("rollback_apply_authority"),
                    s("rollback_transaction_authority"),
                    s("durable_audit_write_authority"),
                    s("persistent_install_authority"),
                    s("raios.audit_record.v0"),
                    s("raios.rollback_transaction.v0"),
                    s("durable_audit_rollback_store"),
                ]),
            ),
            j!("denied_surfaces" => rollback_apply_denied_surfaces_value()),
        ],
        4,
    );
    end_error(method);
    emit_scoped_rollback_apply_decision_marker(method, snapshot);
    emit_scoped_rollback_authorized_append_marker(method, snapshot);
}

pub(crate) fn emit_rollback_apply_applied(
    method: &'static str,
    pre_apply_snapshot: Snapshot,
    snapshot: Snapshot,
    event_id: event_log::EventId,
    proof: ScopedRollbackApplyEvidence,
) {
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => s(HELLO_ROLLBACK_APPLY_SCHEMA)),
            j!("id" => s(HELLO_ROLLBACK_APPLY_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("current_boot_target_region_lba1")),
            j!("status" => s(proof.apply_decision.status)),
            j!("reason" => s(proof.apply_decision.reason)),
            j!("event_id" => event_opt(Some(event_id))),
            j!("audit_event_id" => event_opt(Some(event_id))),
            j!("service_id" => s(SERVICE_ID)),
            j!("rollback_apply_hash" => sha_opt(proof.append_input.append_record_rollback_transaction_image_hash)),
            j!("authority_record" => inline(vec![
                    j!("schema" => s("raios.rollback_transaction.v0")),
                    j!("rollback_transaction_hash" => sha_opt(proof.append_input.append_record_rollback_transaction_image_hash)),
                    j!("authorized_append_hash" => sha(proof.append_hash)),
                    j!("scope_decision_hash" => sha(proof.scope_decision_hash)),
                    j!("write_readback_hash" => sha_opt(proof.append_input.write_readback_hash)),
                    j!("inspection_hash" => sha_opt(proof.append_input.inspection_hash)),
                    j!("audit_record_hash" => sha_opt(proof.append_input.append_record_audit_record_image_hash)),
                    j!("inspected_rollback_transaction_hash" => sha_opt(proof.append_input.inspected_rollback_transaction_image_hash)),
                ]),
            ),
            j!("state_transition" => inline(vec![
                    j!("from_generation" => u(pre_apply_snapshot.generation)),
                    j!("to_generation" => u(snapshot.generation)),
                    j!("from_version" => s(service_version(pre_apply_snapshot.load_descriptor))),
                    j!("to_version" => s(service_version(snapshot.load_descriptor))),
                    j!("from_descriptor_id" => s(pre_apply_snapshot.load_descriptor.id)),
                    j!("to_descriptor_id" => s(snapshot.load_descriptor.id)),
                    j!("state_counter_preserved" => b(pre_apply_snapshot.state_counter == snapshot.state_counter)),
                    j!("current_boot_service_state_mutated" => b(true)),
                ]),
            ),
            j!("previous_state" => hello_state_value(pre_apply_snapshot)),
            j!("applied_state" => hello_state_value(snapshot)),
            j!("source_probation" => hello_hot_swap_probation_value(pre_apply_snapshot.hot_swap_probation)),
            j!("rollback_target" => probation_target_value(pre_apply_snapshot.hot_swap_probation)),
            j!("current_candidate" => probation_candidate_value(pre_apply_snapshot.hot_swap_probation)),
            j!("state_migration" => hello_state_migration_value(pre_apply_snapshot.state_migration)),
            j!("side_effects" => inline(vec![
                    j!("authorizes_media_write" => b(proof.scope_decision.authorized)),
                    j!("authorizes_append" => b(proof.scope_decision.authorized)),
                    j!("authorizes_transaction_append" => b(proof.scope_decision.authorized)),
                    j!("writes_durable_audit_log" => b(proof.append_decision.performed)),
                    j!("writes_rollback_store" => b(proof.append_decision.performed)),
                    j!("appends_rollback_transaction" => b(proof.append_decision.performed)),
                    j!("write_attempted" => b(proof.append_input.write_attempted)),
                    j!("applies_rollback" => b(proof.apply_decision.applied)),
                    j!("mutates_service_state" => b(proof.apply_decision.applied)),
                    j!("installs_rollback_state" => b(false)),
                    j!("persistent_install" => s("denied")),
                    j!("external_artifact_load" => s("denied")),
                    j!("candidate_artifact_execution" => s("denied")),
                    j!("executable_mapping" => s("denied")),
                    j!("provider_auto_load" => s("denied")),
                    j!("broad_mutation" => s("denied")),
                ]),
            ),
            j!("service" => hello_response_service_value(
                    snapshot,
                    snapshot.load_descriptor,
                    service_slot_activation_status(snapshot),
                    service_slot_activation_active(snapshot),
                ),
            ),
        ],
        6,
    );
    end_response(method);
    emit_scoped_rollback_apply_markers_from_proof(proof);
}

fn emit_scoped_rollback_apply_decision_marker(method: &'static str, snapshot: Snapshot) {
    raw(scoped_apply::SCOPED_ROLLBACK_APPLY_DECISION_MARKER);
    raw(" ");
    emit_record_value_fragment(scoped_rollback_apply_decision_value(method, snapshot), 0);
    raw_line("");
}

fn emit_scoped_rollback_authorized_append_marker(method: &'static str, snapshot: Snapshot) {
    raw(scoped_apply::SCOPED_ROLLBACK_AUTHORIZED_APPEND_MARKER);
    raw(" ");
    emit_record_value_fragment(scoped_rollback_authorized_append_value(method, snapshot), 0);
    raw_line("");
}

pub(crate) fn perform_scoped_rollback_apply_proof(
    method: &'static str,
    snapshot: Snapshot,
) -> ScopedRollbackApplyEvidence {
    let (scope_input, scope_decision, scope_decision_hash) =
        scoped_rollback_apply_decision_parts(method, snapshot);
    let append_input = scoped_rollback_authorized_append_input(
        snapshot,
        scope_input,
        scope_decision,
        scope_decision_hash,
    );
    let append_decision = scoped_apply::evaluate_scoped_rollback_authorized_append(&append_input);
    let append_hash = sha256_of_json(&inline(scoped_rollback_authorized_append_fields(
        scope_input,
        scope_decision,
        scope_decision_hash,
        append_input,
        append_decision,
        None,
    )));
    // P4-7b1: the apply evaluator no longer accepts a caller that has not said what it is
    // asking for. The capability is the one this family already declares (see the
    // requested_capability fields above); the effects are what the apply actually does.
    // Pass nothing and you get no proof — which is the whole point.
    let apply_decision = scoped_apply::evaluate_scoped_rollback_verified_apply(
        &append_input,
        Some(append_hash),
        HELLO_ROLLBACK_APPLY_CAPABILITY,
        &["applies_rollback", "mutates_service_state"],
    );
    ScopedRollbackApplyEvidence {
        scope_proof: scope_decision.proof(),
        append_proof: append_decision.proof(),
        verified_apply_proof: apply_decision.proof(),
        scope_input,
        scope_decision,
        scope_decision_hash,
        append_input,
        append_decision,
        append_hash,
        apply_decision,
    }
}

pub(crate) fn emit_scoped_rollback_apply_markers_from_proof(proof: ScopedRollbackApplyEvidence) {
    raw(scoped_apply::SCOPED_ROLLBACK_APPLY_DECISION_MARKER);
    raw(" ");
    emit_record_value_fragment(scoped_rollback_apply_decision_value_from_proof(proof), 0);
    raw_line("");
    raw(scoped_apply::SCOPED_ROLLBACK_AUTHORIZED_APPEND_MARKER);
    raw(" ");
    emit_record_value_fragment(scoped_rollback_authorized_append_value_from_proof(proof), 0);
    raw_line("");
}

fn scoped_rollback_apply_decision_value(method: &'static str, snapshot: Snapshot) -> V<'static> {
    let (input, decision, decision_hash) = scoped_rollback_apply_decision_parts(method, snapshot);
    inline(scoped_rollback_apply_decision_fields(
        input,
        decision,
        Some(decision_hash),
    ))
}

fn scoped_rollback_apply_decision_parts(
    method: &'static str,
    snapshot: Snapshot,
) -> (
    scoped_apply::ScopedRollbackApplyInput<'static>,
    scoped_apply::ScopedRollbackApplyDecision,
    [u8; 32],
) {
    let input = scoped_rollback_apply_input(method, snapshot);
    let decision = scoped_apply::evaluate_scoped_rollback_apply(&input);
    let decision_hash = sha256_of_json(&inline(scoped_rollback_apply_decision_fields(
        input, decision, None,
    )));
    (input, decision, decision_hash)
}

fn scoped_rollback_apply_decision_value_from_proof(
    proof: ScopedRollbackApplyEvidence,
) -> V<'static> {
    inline(scoped_rollback_apply_decision_fields(
        proof.scope_input,
        proof.scope_decision,
        Some(proof.scope_decision_hash),
    ))
}

fn scoped_rollback_apply_decision_fields(
    input: scoped_apply::ScopedRollbackApplyInput<'static>,
    decision: scoped_apply::ScopedRollbackApplyDecision,
    decision_hash: Option<[u8; 32]>,
) -> Vec<Field<'static>> {
    vec![
        j!("schema" => s(scoped_apply::SCOPED_ROLLBACK_APPLY_DECISION_SCHEMA)),
        j!("id" => s(scoped_apply::SCOPED_ROLLBACK_APPLY_DECISION_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("decision_hash" => sha_opt(decision_hash)),
        j!("method" => s_opt(input.method)),
        j!("service_id" => s_opt(input.service_id)),
        j!("authorized" => b(decision.authorized)),
        j!("status" => s(decision.status)),
        j!("reason" => s(decision.reason)),
        j!("target_scope" => inline(vec![
                j!("target_region_id" => s_opt(input.target_region_id)),
                j!("target_region_marker" => s_opt(input.target_region_marker)),
                j!("target_start_lba" => u_opt(input.target_start_lba)),
                j!("target_lba_count" => u_opt(input.target_lba_count)),
                j!("target_byte_count" => u_opt(input.target_byte_count)),
                j!("audit_ledger_target_id" => s_opt(input.audit_ledger_target_id)),
                j!("audit_record_schema" => s_opt(input.audit_record_schema)),
                j!("rollback_store_target_id" => s_opt(input.rollback_store_target_id)),
                j!("rollback_transaction_schema" => s_opt(input.rollback_transaction_schema)),
            ]),
        ),
        j!("state_scope" => inline(vec![
                j!("probation_status" => s_opt(input.probation_status)),
                j!("probation_hash" => sha_opt(input.probation_hash)),
                j!("probation_accepted" => b(input.probation_accepted)),
                j!("rollback_preview_status" => s_opt(input.rollback_preview_status)),
                j!("rollback_preview_hash" => sha_opt(input.rollback_preview_hash)),
                j!("current_state_hash" => sha_opt(input.current_state_hash)),
                j!("current_state_counter" => u_opt(input.current_state_counter)),
                j!("probation_new_state_hash" => sha_opt(input.probation_new_state_hash)),
                j!("probation_new_state_counter" => u_opt(input.probation_new_state_counter)),
                j!("state_migration_hash" => sha_opt(input.state_migration_hash)),
            ]),
        ),
        j!("evidence_readiness" => inline(vec![
                j!("scratch_readiness_verified" => b(input.scratch_readiness_verified)),
                j!("append_record_ready" => b(input.append_record_ready)),
                j!("sector_plan_ready" => b(input.sector_plan_ready)),
                j!("target_region_write_readback_verified" => b(input.target_region_write_readback_verified)),
                j!("transaction_append_dry_run_verified" => b(input.transaction_append_dry_run_verified)),
                j!("target_region_sector_inspection_verified" => b(input.target_region_sector_inspection_verified)),
                j!("durable_policy_write_authority_decision_verified" => b(input.durable_policy_write_authority_decision_verified)),
                j!("retained_inspect_source_reference_validated" => b(input.retained_inspect_source_reference_validated)),
            ]),
        ),
        j!("source_hashes" => inline(vec![
                j!("append_record_hash" => sha_opt(input.append_record_hash)),
                j!("sector_plan_hash" => sha_opt(input.sector_plan_hash)),
                j!("target_region_write_readback_hash" => sha_opt(input.target_region_write_readback_hash)),
                j!("transaction_append_dry_run_hash" => sha_opt(input.transaction_append_dry_run_hash)),
                j!("transaction_append_source_sector_plan_hash" => sha_opt(input.transaction_append_source_sector_plan_hash)),
                j!("transaction_append_source_target_region_write_readback_hash" => sha_opt(input.transaction_append_source_target_region_write_readback_hash)),
                j!("durable_policy_write_authority_decision_hash" => sha_opt(input.durable_policy_write_authority_decision_hash)),
                j!("policy_source_transaction_append_dry_run_hash" => sha_opt(input.policy_source_transaction_append_dry_run_hash)),
                j!("policy_source_target_region_sector_inspection_hash" => sha_opt(input.policy_source_target_region_sector_inspection_hash)),
                j!("target_region_sector_inspection_hash" => sha_opt(input.target_region_sector_inspection_hash)),
                j!("inspection_source_sector_plan_hash" => sha_opt(input.inspection_source_sector_plan_hash)),
                j!("inspection_source_target_region_write_readback_hash" => sha_opt(input.inspection_source_target_region_write_readback_hash)),
            ]),
        ),
        j!("sector_hashes" => inline(vec![
                j!("sector_plan_sector_image_hash" => sha_opt(input.sector_plan_sector_image_hash)),
                j!("planned_sector_image_hash" => sha_opt(input.planned_sector_image_hash)),
                j!("readback_sector_image_hash" => sha_opt(input.readback_sector_image_hash)),
                j!("expected_sector_image_hash" => sha_opt(input.expected_sector_image_hash)),
                j!("inspected_sector_image_hash" => sha_opt(input.inspected_sector_image_hash)),
            ]),
        ),
        j!("record_hashes" => inline(vec![
                j!("append_record_audit_record_image_hash" => sha_opt(input.append_record_audit_record_image_hash)),
                j!("inspected_audit_record_image_hash" => sha_opt(input.inspected_audit_record_image_hash)),
                j!("append_record_rollback_transaction_image_hash" => sha_opt(input.append_record_rollback_transaction_image_hash)),
                j!("inspected_rollback_transaction_image_hash" => sha_opt(input.inspected_rollback_transaction_image_hash)),
            ]),
        ),
        j!("retained_inspection" => inline(vec![
                j!("retained_inspect_source_reference_hash" => sha_opt(input.retained_inspect_source_reference_hash)),
                j!("retained_inspection_hash" => sha_opt(input.retained_inspection_hash)),
                j!("retained_source_sector_plan_hash" => sha_opt(input.retained_source_sector_plan_hash)),
                j!("retained_source_target_region_write_readback_hash" => sha_opt(input.retained_source_target_region_write_readback_hash)),
            ]),
        ),
        j!("side_effects" => inline(vec![
                j!("write_attempted" => b(false)),
                j!("writes_durable_audit_log" => b(false)),
                j!("writes_rollback_store" => b(false)),
                j!("appends_rollback_transaction" => b(false)),
                j!("applies_rollback" => b(false)),
                j!("mutates_service_state" => b(false)),
            ]),
        ),
    ]
}

fn scoped_rollback_authorized_append_value(method: &'static str, snapshot: Snapshot) -> V<'static> {
    let (scope_input, scope_decision, scope_decision_hash) =
        scoped_rollback_apply_decision_parts(method, snapshot);
    let append_input = scoped_rollback_authorized_append_input(
        snapshot,
        scope_input,
        scope_decision,
        scope_decision_hash,
    );
    let append_decision = scoped_apply::evaluate_scoped_rollback_authorized_append(&append_input);
    let append_hash = sha256_of_json(&inline(scoped_rollback_authorized_append_fields(
        scope_input,
        scope_decision,
        scope_decision_hash,
        append_input,
        append_decision,
        None,
    )));
    inline(scoped_rollback_authorized_append_fields(
        scope_input,
        scope_decision,
        scope_decision_hash,
        append_input,
        append_decision,
        Some(append_hash),
    ))
}

fn scoped_rollback_authorized_append_value_from_proof(
    proof: ScopedRollbackApplyEvidence,
) -> V<'static> {
    inline(scoped_rollback_authorized_append_fields(
        proof.scope_input,
        proof.scope_decision,
        proof.scope_decision_hash,
        proof.append_input,
        proof.append_decision,
        Some(proof.append_hash),
    ))
}

fn scoped_rollback_authorized_append_fields(
    scope_input: scoped_apply::ScopedRollbackApplyInput<'static>,
    scope_decision: scoped_apply::ScopedRollbackApplyDecision,
    scope_decision_hash: [u8; 32],
    append_input: scoped_apply::ScopedRollbackAuthorizedAppendInput,
    append_decision: scoped_apply::ScopedRollbackAuthorizedAppendDecision,
    append_hash: Option<[u8; 32]>,
) -> Vec<Field<'static>> {
    vec![
        j!("schema" => s(scoped_apply::SCOPED_ROLLBACK_AUTHORIZED_APPEND_SCHEMA)),
        j!("id" => s(scoped_apply::SCOPED_ROLLBACK_AUTHORIZED_APPEND_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("current_boot_target_region_lba1")),
        j!("append_hash" => sha_opt(append_hash)),
        j!("method" => s_opt(scope_input.method)),
        j!("service_id" => s_opt(scope_input.service_id)),
        j!("status" => s(append_decision.status)),
        j!("reason" => s(append_decision.reason)),
        j!("transaction_append_performed" => b(append_decision.performed)),
        j!("scope_decision" => inline(vec![
                j!("schema" => s(scoped_apply::SCOPED_ROLLBACK_APPLY_DECISION_SCHEMA)),
                j!("id" => s(scoped_apply::SCOPED_ROLLBACK_APPLY_DECISION_ID)),
                j!("decision_hash" => sha(scope_decision_hash)),
                j!("authorized" => b(scope_decision.authorized)),
                j!("status" => s(scope_decision.status)),
                j!("reason" => s(scope_decision.reason)),
            ]),
        ),
        j!("target_scope" => inline(vec![
                j!("target_region_id" => s_opt(scope_input.target_region_id)),
                j!("target_region_marker" => s_opt(scope_input.target_region_marker)),
                j!("target_start_lba" => u_opt(append_input.target_start_lba)),
                j!("target_lba_count" => u_opt(append_input.target_lba_count)),
                j!("target_byte_count" => u_opt(append_input.target_byte_count)),
                j!("audit_ledger_target_id" => s_opt(scope_input.audit_ledger_target_id)),
                j!("audit_record_schema" => s_opt(scope_input.audit_record_schema)),
                j!("rollback_store_target_id" => s_opt(scope_input.rollback_store_target_id)),
                j!("rollback_transaction_schema" => s_opt(scope_input.rollback_transaction_schema)),
            ]),
        ),
        j!("source_hashes" => inline(vec![
                j!("scope_decision_hash" => sha(append_input.scope_decision_hash.unwrap_or(scope_decision_hash))),
                j!("append_record_hash" => sha_opt(append_input.append_record_hash)),
                j!("sector_plan_hash" => sha_opt(append_input.sector_plan_hash)),
                j!("write_readback_hash" => sha_opt(append_input.write_readback_hash)),
                j!("inspection_hash" => sha_opt(append_input.inspection_hash)),
                j!("write_readback_source_sector_plan_hash" => sha_opt(append_input.write_readback_source_sector_plan_hash)),
                j!("inspection_source_sector_plan_hash" => sha_opt(append_input.inspection_source_sector_plan_hash)),
                j!("inspection_source_write_readback_hash" => sha_opt(append_input.inspection_source_write_readback_hash)),
            ]),
        ),
        j!("sector_hashes" => inline(vec![
                j!("sector_plan_sector_image_hash" => sha_opt(append_input.sector_plan_sector_image_hash)),
                j!("planned_sector_image_hash" => sha_opt(append_input.planned_sector_image_hash)),
                j!("readback_sector_image_hash" => sha_opt(append_input.readback_sector_image_hash)),
                j!("expected_sector_image_hash" => sha_opt(append_input.expected_sector_image_hash)),
                j!("inspected_sector_image_hash" => sha_opt(append_input.inspected_sector_image_hash)),
            ]),
        ),
        j!("transaction_hashes" => inline(vec![
                j!("audit_record_image_hash" => sha_opt(append_input.append_record_audit_record_image_hash)),
                j!("inspected_audit_record_image_hash" => sha_opt(append_input.inspected_audit_record_image_hash)),
                j!("rollback_transaction_image_hash" => sha_opt(append_input.append_record_rollback_transaction_image_hash)),
                j!("inspected_rollback_transaction_image_hash" => sha_opt(append_input.inspected_rollback_transaction_image_hash)),
            ]),
        ),
        j!("sector_layout" => inline(vec![
                j!("audit_record_offset" => u_opt(append_input.audit_record_offset)),
                j!("audit_record_byte_length" => u_opt(append_input.audit_record_byte_length)),
                j!("rollback_transaction_offset" => u_opt(append_input.rollback_transaction_offset)),
                j!("rollback_transaction_byte_length" => u_opt(append_input.rollback_transaction_byte_length)),
                j!("padding_offset" => u_opt(append_input.padding_offset)),
                j!("padding_byte_length" => u_opt(append_input.padding_byte_length)),
            ]),
        ),
        j!("write_readback" => inline(vec![
                j!("write_attempted" => b(append_input.write_attempted)),
                j!("write_completed" => b(append_input.write_completed)),
                j!("readback_completed" => b(append_input.readback_completed)),
                j!("readback_matches_planned_image" => b(append_input.readback_matches_planned_image)),
            ]),
        ),
        j!("inspection" => inline(vec![
                j!("read_attempted" => b(append_input.inspection_read_attempted)),
                j!("read_completed" => b(append_input.inspection_read_completed)),
                j!("sector_hash_verified" => b(append_input.sector_hash_verified)),
                j!("audit_record_hash_verified" => b(append_input.audit_record_hash_verified)),
                j!("rollback_transaction_hash_verified" => b(append_input.rollback_transaction_hash_verified)),
                j!("offsets_verified" => b(append_input.offsets_verified)),
                j!("padding_zeroed" => b(append_input.padding_zeroed)),
                j!("target_span_verified" => b(append_input.target_span_verified)),
                j!("inspection_verified" => b(append_input.inspection_verified)),
            ]),
        ),
        j!("side_effects" => inline(vec![
                j!("authorizes_media_write" => b(scope_decision.authorized)),
                j!("authorizes_append" => b(scope_decision.authorized)),
                j!("authorizes_transaction_append" => b(scope_decision.authorized)),
                j!("writes_durable_audit_log" => b(append_decision.performed)),
                j!("writes_rollback_store" => b(append_decision.performed)),
                j!("appends_rollback_transaction" => b(append_decision.performed)),
                j!("write_attempted" => b(append_input.write_attempted)),
                j!("applies_rollback" => b(false)),
                j!("mutates_service_state" => b(false)),
                j!("installs_rollback_state" => b(false)),
            ]),
        ),
    ]
}

fn scoped_rollback_authorized_append_input(
    snapshot: Snapshot,
    scope_input: scoped_apply::ScopedRollbackApplyInput<'static>,
    scope_decision: scoped_apply::ScopedRollbackApplyDecision,
    scope_decision_hash: [u8; 32],
) -> scoped_apply::ScopedRollbackAuthorizedAppendInput {
    let mut input = scoped_apply::ScopedRollbackAuthorizedAppendInput::empty();
    input.requested_capability = HELLO_ROLLBACK_APPLY_CAPABILITY;
    input.effects = &["durable_transaction_append"];
    input.scope_decision_authorized = scope_decision.authorized;
    input.scope_decision_hash = Some(scope_decision_hash);
    input.target_start_lba = scope_input.target_start_lba;
    input.target_lba_count = scope_input.target_lba_count;
    input.target_byte_count = scope_input.target_byte_count;

    let Some(scope_proof) = scope_decision.proof() else {
        return input;
    };

    let Some(probation) = snapshot.hot_swap_probation else {
        return input;
    };
    let foundation = rollback_writer_storage_foundation();
    let append_record = hello_rollback_append_record_dry_run(snapshot, probation, foundation);
    let sector_plan = hello_rollback_append_sector_plan_dry_run(snapshot, probation, append_record);
    let target_region_media_write_policy_preflight =
        hello_target_region_media_write_policy_preflight(foundation);
    let target_region_write = hello_rollback_target_region_authorized_append_write_readback(
        &scope_proof,
        snapshot,
        probation,
        sector_plan,
        foundation,
        target_region_media_write_policy_preflight,
    );
    let inspection = hello_rollback_target_region_authorized_append_sector_inspection(
        append_record,
        sector_plan,
        target_region_write,
    );

    input.append_record_hash = Some(append_record.dry_run_hash);
    input.sector_plan_hash = Some(sector_plan.plan_hash);
    input.write_readback_hash = Some(target_region_write.dry_run_hash);
    input.inspection_hash = Some(inspection.inspection_hash);
    input.write_readback_source_sector_plan_hash =
        Some(target_region_write.source_sector_plan_hash);
    input.inspection_source_sector_plan_hash = Some(inspection.source_sector_plan_hash);
    input.inspection_source_write_readback_hash =
        Some(inspection.source_target_region_write_readback_hash);
    input.sector_plan_sector_image_hash = Some(sector_plan.sector_image_hash);
    input.planned_sector_image_hash = Some(target_region_write.planned_sector_image_hash);
    input.readback_sector_image_hash = Some(target_region_write.readback_sector_image_hash);
    input.expected_sector_image_hash = Some(inspection.expected_sector_image_hash);
    input.inspected_sector_image_hash = Some(inspection.sector_image_hash);
    input.append_record_audit_record_image_hash = Some(append_record.audit_record_image_hash);
    input.inspected_audit_record_image_hash = Some(inspection.audit_record_image_hash);
    input.append_record_rollback_transaction_image_hash =
        Some(append_record.rollback_transaction_image_hash);
    input.inspected_rollback_transaction_image_hash =
        Some(inspection.rollback_transaction_image_hash);
    input.audit_record_offset = Some(inspection.audit_record_offset);
    input.audit_record_byte_length = Some(inspection.audit_record_byte_length);
    input.rollback_transaction_offset = Some(inspection.rollback_transaction_offset);
    input.rollback_transaction_byte_length = Some(inspection.rollback_transaction_byte_length);
    input.padding_offset = Some(inspection.padding_offset);
    input.padding_byte_length = Some(inspection.padding_byte_length);
    input.write_attempted = target_region_write.write_attempted;
    input.write_completed = target_region_write.write_completed;
    input.readback_completed = target_region_write.readback_completed;
    input.readback_matches_planned_image = target_region_write.readback_matches_planned_image;
    input.inspection_read_attempted = inspection.read_attempted;
    input.inspection_read_completed = inspection.read_completed;
    input.sector_hash_verified = inspection.sector_hash_verified;
    input.audit_record_hash_verified = inspection.audit_record_hash_verified;
    input.rollback_transaction_hash_verified = inspection.rollback_transaction_hash_verified;
    input.offsets_verified = inspection.offsets_verified;
    input.padding_zeroed = inspection.padding_zeroed;
    input.target_span_verified = inspection.target_span_verified;
    input.inspection_verified = inspection.inspection_verified;
    input
}

fn scoped_rollback_apply_input(
    method: &'static str,
    snapshot: Snapshot,
) -> scoped_apply::ScopedRollbackApplyInput<'static> {
    let mut input = scoped_apply::ScopedRollbackApplyInput::empty();
    input.requested_capability = HELLO_ROLLBACK_APPLY_CAPABILITY;
    input.effects = &["durable_transaction_append"];
    input.method = Some(method);
    input.service_id = Some(SERVICE_ID);
    input.target_region_id =
        Some(rollback_storage_layout::AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_ID);
    input.target_region_marker = Some(ahci::AUDIT_ROLLBACK_TARGET_REGION_MARKER);
    input.audit_ledger_target_id =
        Some(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
    input.audit_record_schema =
        Some(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
    input.rollback_store_target_id =
        Some(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
    input.rollback_transaction_schema =
        Some(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
    input.target_start_lba = Some(scoped_apply::EXPECTED_TARGET_START_LBA);
    input.target_lba_count = Some(scoped_apply::EXPECTED_TARGET_LBA_COUNT);
    input.target_byte_count = Some(scoped_apply::EXPECTED_TARGET_BYTE_COUNT);

    let Some(probation) = snapshot.hot_swap_probation else {
        return input;
    };

    let foundation = rollback_writer_storage_foundation();
    let append_record = hello_rollback_append_record_dry_run(snapshot, probation, foundation);
    let sector_plan = hello_rollback_append_sector_plan_dry_run(snapshot, probation, append_record);
    let sector_write =
        hello_rollback_append_sector_write_readback_dry_run(snapshot, probation, sector_plan);
    let target_region_media_write_policy_preflight =
        hello_target_region_media_write_policy_preflight(foundation);
    let target_region_write = hello_rollback_target_region_write_readback_dry_run_from_materializer(
        sector_plan,
        foundation,
        target_region_media_write_policy_preflight,
    );
    let durable_writer_policy_preflight = hello_rollback_durable_writer_policy_preflight(
        foundation,
        append_record,
        sector_plan,
        target_region_write,
    );
    let durable_append_preflight = hello_rollback_durable_append_authority_preflight(
        foundation,
        append_record,
        sector_plan,
        sector_write,
        target_region_media_write_policy_preflight,
        target_region_write,
        durable_writer_policy_preflight,
    );
    let media_write_authority_gate =
        hello_rollback_media_write_authority_gate(durable_append_preflight, target_region_write);
    let durable_append_transaction_authorization_gate =
        hello_rollback_durable_append_transaction_authorization_gate(
            durable_writer_policy_preflight,
            append_record,
            sector_plan,
            target_region_write,
        );
    let append_engine_readiness_decision = hello_rollback_append_engine_readiness_decision(
        durable_append_transaction_authorization_gate,
    );
    let durable_append_authority_decision = hello_rollback_durable_append_authority_decision(
        durable_append_preflight,
        media_write_authority_gate,
        append_engine_readiness_decision,
    );
    let durable_audit_policy_decision =
        hello_rollback_durable_audit_policy_decision(durable_append_authority_decision);
    let durable_audit_policy_candidate =
        hello_rollback_durable_audit_policy_candidate(durable_audit_policy_decision, append_record);
    let durable_audit_policy_acceptance_gate =
        hello_rollback_durable_audit_policy_acceptance_gate(durable_audit_policy_candidate);
    let durable_audit_policy_ledger_candidate =
        hello_rollback_durable_audit_policy_ledger_candidate(durable_audit_policy_acceptance_gate);
    let durable_audit_policy_ledger_aware_acceptance_result =
        hello_rollback_durable_audit_policy_ledger_aware_acceptance_result(
            durable_audit_policy_ledger_candidate,
        );
    let durable_audit_policy_write_authority_availability =
        hello_rollback_durable_audit_policy_write_authority_availability(
            durable_audit_policy_ledger_aware_acceptance_result,
            durable_audit_policy_ledger_candidate,
            target_region_media_write_policy_preflight,
            target_region_write,
        );
    let durable_policy_ledger_availability = hello_rollback_durable_policy_ledger_availability(
        durable_audit_policy_write_authority_availability,
    );
    let durable_audit_policy_availability =
        hello_rollback_durable_audit_policy_availability(durable_policy_ledger_availability);
    let durable_append_authority_availability =
        hello_rollback_durable_append_authority_availability(durable_audit_policy_availability);
    let transaction_append_availability_decision =
        hello_rollback_transaction_append_availability_decision(
            durable_append_authority_availability,
            append_engine_readiness_decision,
            durable_writer_policy_preflight,
        );
    let transaction_append_authority_denial_gate =
        hello_rollback_transaction_append_authority_denial_gate(
            transaction_append_availability_decision,
        );
    let durable_policy_ledger_availability_dry_run =
        hello_rollback_durable_policy_ledger_availability_dry_run(
            durable_policy_ledger_availability,
            durable_audit_policy_write_authority_availability,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    let durable_audit_policy_availability_dry_run =
        hello_rollback_durable_audit_policy_availability_dry_run(
            durable_audit_policy_availability,
            durable_policy_ledger_availability_dry_run,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    let durable_append_authority_availability_dry_run =
        hello_rollback_durable_append_authority_availability_dry_run(
            durable_append_authority_availability,
            durable_audit_policy_availability_dry_run,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    let transaction_append_dry_run = hello_rollback_transaction_append_dry_run(
        transaction_append_authority_denial_gate,
        append_record,
        sector_plan,
        target_region_write,
    );
    let target_region_sector_inspection =
        hello_rollback_target_region_sector_inspection_from_retained_inspect(
            append_record,
            sector_plan,
            target_region_write,
        );
    let durable_policy_write_authority_decision =
        hello_rollback_durable_policy_write_authority_decision(
            durable_append_authority_availability_dry_run,
            durable_audit_policy_write_authority_availability,
            durable_audit_policy_availability,
            durable_append_authority_availability,
            transaction_append_dry_run,
            target_region_sector_inspection,
        );
    let retained_source =
        recovery_rollback_inspect_source_reference_state(target_region_sector_inspection);

    input.probation_status = Some(probation.status);
    input.probation_hash = Some(probation.probation_hash);
    input.probation_accepted = probation.accepted;
    input.rollback_preview_status = Some(HELLO_ROLLBACK_PREVIEW_STATUS);
    input.rollback_preview_hash = Some(hello_rollback_preview_hash(snapshot, probation));
    input.current_state_hash = Some(hello_state_hash(snapshot.state_counter));
    input.current_state_counter = Some(snapshot.state_counter);
    input.probation_new_state_hash = Some(probation.new_state_hash);
    input.probation_new_state_counter = Some(probation.new_state_counter);
    input.state_migration_hash = Some(probation.state_migration_hash);
    input.scratch_readiness_verified = foundation.scratch_writer_dry_run_ready
        && sector_write.write_completed
        && sector_write.readback_completed
        && sector_write.readback_matches_planned_image;
    input.append_record_ready = append_record.target_range_ready
        && append_record.target_start_lba == scoped_apply::EXPECTED_TARGET_START_LBA
        && append_record.target_lba_count == scoped_apply::EXPECTED_TARGET_LBA_COUNT
        && append_record.target_byte_count == scoped_apply::EXPECTED_TARGET_BYTE_COUNT
        && append_record.total_record_byte_length <= append_record.target_byte_count;
    input.sector_plan_ready = sector_plan.target_range_ready
        && sector_plan.target_start_lba == scoped_apply::EXPECTED_TARGET_START_LBA
        && sector_plan.target_lba_count == scoped_apply::EXPECTED_TARGET_LBA_COUNT
        && sector_plan.target_byte_count == scoped_apply::EXPECTED_TARGET_BYTE_COUNT
        && sector_plan.audit_record_offset == 0
        && sector_plan.rollback_transaction_offset == append_record.audit_record_byte_length
        && sector_plan.padding_offset == append_record.total_record_byte_length;
    input.target_region_write_readback_verified = target_region_write.label_found
        && target_region_write.target_range_ready
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image;
    input.transaction_append_dry_run_verified = transaction_append_dry_run
        .authority_denial_gate_verified
        && transaction_append_dry_run.target_span_verified
        && transaction_append_dry_run.target_region_write_readback_verified
        && transaction_append_dry_run.append_image_ready;
    input.target_region_sector_inspection_verified =
        target_region_sector_inspection.inspection_verified;
    input.durable_policy_write_authority_decision_verified = durable_policy_write_authority_decision
        .transaction_append_dry_run_verified
        && durable_policy_write_authority_decision.target_region_sector_inspection_verified
        && durable_policy_write_authority_decision.write_authority_evidence_verified
        && durable_policy_write_authority_decision.audit_policy_availability_evidence_verified
        && durable_policy_write_authority_decision
            .durable_append_authority_availability_evidence_verified
        && durable_policy_write_authority_decision.target_span_verified;
    input.retained_inspect_source_reference_validated = retained_source.ram_audit_validated;
    input.append_record_hash = Some(append_record.dry_run_hash);
    input.sector_plan_hash = Some(sector_plan.plan_hash);
    input.target_region_write_readback_hash = Some(target_region_write.dry_run_hash);
    input.transaction_append_dry_run_hash = Some(transaction_append_dry_run.dry_run_hash);
    input.transaction_append_source_sector_plan_hash =
        Some(transaction_append_dry_run.source_sector_plan_hash);
    input.transaction_append_source_target_region_write_readback_hash =
        Some(transaction_append_dry_run.source_target_region_write_readback_hash);
    input.durable_policy_write_authority_decision_hash =
        Some(durable_policy_write_authority_decision.decision_hash);
    input.policy_source_transaction_append_dry_run_hash =
        Some(durable_policy_write_authority_decision.source_transaction_append_dry_run_hash);
    input.policy_source_target_region_sector_inspection_hash =
        Some(durable_policy_write_authority_decision.source_target_region_sector_inspection_hash);
    input.target_region_sector_inspection_hash =
        Some(target_region_sector_inspection.inspection_hash);
    input.inspection_source_sector_plan_hash =
        Some(target_region_sector_inspection.source_sector_plan_hash);
    input.inspection_source_target_region_write_readback_hash =
        Some(target_region_sector_inspection.source_target_region_write_readback_hash);
    input.sector_plan_sector_image_hash = Some(sector_plan.sector_image_hash);
    input.planned_sector_image_hash = Some(target_region_write.planned_sector_image_hash);
    input.readback_sector_image_hash = Some(target_region_write.readback_sector_image_hash);
    input.expected_sector_image_hash =
        Some(target_region_sector_inspection.expected_sector_image_hash);
    input.inspected_sector_image_hash = Some(target_region_sector_inspection.sector_image_hash);
    input.append_record_audit_record_image_hash = Some(append_record.audit_record_image_hash);
    input.inspected_audit_record_image_hash =
        Some(target_region_sector_inspection.audit_record_image_hash);
    input.append_record_rollback_transaction_image_hash =
        Some(append_record.rollback_transaction_image_hash);
    input.inspected_rollback_transaction_image_hash =
        Some(target_region_sector_inspection.rollback_transaction_image_hash);
    input.retained_inspect_source_reference_hash = retained_source
        .reference
        .map(|reference| reference.reference_hash);
    input.retained_inspection_hash = retained_source
        .reference
        .map(|reference| reference.inspection_hash);
    input.retained_source_sector_plan_hash = retained_source
        .reference
        .map(|reference| reference.source_sector_plan_hash);
    input.retained_source_target_region_write_readback_hash = retained_source
        .reference
        .map(|reference| reference.source_target_region_write_readback_hash);
    input
}

pub(crate) fn emit_rollback_preview_response(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let probation = snapshot.hot_swap_probation;
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => s("raios.ram_only_hello_service_rollback_preview.v0"),
            ),
            j!("id" => s(HELLO_ROLLBACK_PREVIEW_ID)),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("read_only" => b(true)),
            j!("status" => s(if probation.is_some() {
                    HELLO_ROLLBACK_PREVIEW_STATUS
                } else {
                    "missing_hot_swap_probation"
                }),
            ),
            j!("preview_available" => b(probation.is_some())),
            j!("event_id" => event_opt(Some(event_id))),
            j!("audit_event_id" => event_opt(Some(event_id))),
            j!("service_id" => s(SERVICE_ID)),
            j!("current_generation" => u(snapshot.generation)),
            j!("current_state" => hello_state_value(snapshot)),
            j!("source_probation" => hello_hot_swap_probation_value(probation),
            ),
            j!("preview_hash" => probation
                    .map(|probation| sha(hello_rollback_preview_hash(snapshot, probation)))
                    .unwrap_or(V::Null),
            ),
            j!("rollback_target" => probation_target_value(probation)),
            j!("current_candidate" => probation_candidate_value(probation)),
            j!("state_migration" => hello_state_migration_value(snapshot.state_migration),
            ),
            j!("denied_surfaces" => rollback_preview_denied_surfaces_value()),
        ],
        6,
    );
    end_response(method);
}

pub(crate) fn emit_response(
    method: &'static str,
    action: LifecycleAction,
    capture: LifecycleCapture,
    descriptor: LoadDescriptor,
) {
    let event_id = capture
        .after
        .last_event_id
        .expect("lifecycle mutation records one event");
    emit_hello_lifecycle_v1(
        method,
        "hello.lifecycle",
        action,
        capture,
        descriptor,
        event_id,
    );
}

fn emit_hello_lifecycle_v1(
    method: &'static str,
    family: &'static str,
    action: LifecycleAction,
    capture: LifecycleCapture,
    descriptor: LoadDescriptor,
    event_id: event_log::EventId,
) {
    let projection = project_hello_lifecycle(hello_lifecycle_snapshot(
        action, capture, descriptor, event_id,
    ));
    // decision() borrows the projection, so take it before the evidence is moved out.
    let decision = projection.decision();
    crate::agent_protocol_module_reference::emit_evidence_v1_response(
        method,
        family,
        Some(event_id),
        projection.facts,
        projection
            .evidence
            .into_iter()
            .map(evidence_value)
            .collect(),
        decision,
    );
}

fn hello_lifecycle_snapshot(
    action: LifecycleAction,
    capture: LifecycleCapture,
    descriptor: LoadDescriptor,
    event_id: event_log::EventId,
) -> HelloLifecycleSnapshot<'static> {
    let after = capture.after;
    let descriptor_hash = descriptor_source_hash(descriptor);
    let descriptor_verified = descriptor_source_signature_verified(descriptor);
    let artifact_verified = artifact_identity_signature_verified(descriptor)
        && descriptor_sources::validate_builtin_hello_artifact_identity(
            descriptor.artifact_identity,
        )
        && validate_artifact_load_plan_preflight_record(
            artifact_load_plan_preflight_record(descriptor),
            descriptor,
        );
    let classification = hello_evidence_classification(descriptor.classification);
    let descriptor_meta = hello_evidence_meta(
        descriptor_verified,
        "descriptor_signature_verified",
        "descriptor_signature_rejected",
        classification,
    );
    let artifact_meta = hello_evidence_meta(
        artifact_verified,
        "artifact_identity_and_preflight_verified",
        "artifact_identity_or_preflight_rejected",
        classification,
    );
    let transition_meta = if !capture.before.loaded && action == LifecycleAction::Load {
        EvidenceMeta {
            status: EvidenceStatus::Missing,
            reason: "pre_state_not_available_service_not_loaded",
            classification,
        }
    } else if action == LifecycleAction::Health {
        EvidenceMeta {
            status: EvidenceStatus::NotApplicable,
            reason: "health_observation",
            classification,
        }
    } else {
        EvidenceMeta {
            status: EvidenceStatus::Verified,
            reason: after.last_reason,
            classification,
        }
    };
    HelloLifecycleSnapshot {
        action,
        source_event_sequence: event_id.sequence(),
        service_id: SERVICE_ID,
        descriptor: DescriptorSnapshot {
            meta: descriptor_meta,
            descriptor_id: descriptor.id,
            source_locator: descriptor.source_locator,
            source_kind: descriptor.source_kind,
            descriptor_hash,
            bound_source_hash: descriptor.binds_source_hash.unwrap_or(descriptor_hash),
            signature_hash: descriptor
                .source_envelope
                .map(|envelope| envelope.signature_hash)
                .unwrap_or([0; 32]),
            attestation_hash: descriptor.artifact_identity.artifact_content_source_hash,
        },
        artifact: ArtifactSnapshot {
            meta: artifact_meta,
            artifact_id: descriptor.artifact_identity.id,
            artifact_hash: artifact_identity_hash(descriptor),
            content_binding_hash: artifact_content_binding_hash(descriptor),
            reference_hash: artifact_reference_hash(descriptor),
            bytes_hash: artifact_reference_bytes_hash(descriptor),
            signature_hash: descriptor.artifact_identity.signed_envelope.signature_hash,
            preflight_status: ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS,
        },
        service_slot: ServiceSlotSnapshot {
            meta: EvidenceMeta {
                status: EvidenceStatus::Verified,
                reason: after.last_reason,
                classification,
            },
            slot_id: RAM_ONLY_SERVICE_SLOT_ID,
            before_status: service_slot_activation_status(capture.before),
            after_status: service_slot_activation_status(after),
            active: service_slot_activation_active(after),
        },
        inventory: InventorySnapshot {
            meta: EvidenceMeta {
                status: EvidenceStatus::Verified,
                reason: after.last_reason,
                classification,
            },
            change: if action == LifecycleAction::Health {
                "none"
            } else {
                after.last_inventory_change
            },
            present_before: capture.before.loaded,
            present_after: after.loaded,
        },
        state_transition: StateTransitionSnapshot {
            meta: transition_meta,
            before: hello_service_state_snapshot(capture.before),
            after: hello_service_state_snapshot(after),
        },
        state_migration: after
            .state_migration
            .map(|migration| StateMigrationSnapshot {
                meta: hello_evidence_meta(
                    migration.accepted,
                    "state_migration_verified",
                    "state_migration_rejected",
                    classification,
                ),
                from_version: migration.from_version,
                to_version: migration.to_version,
                state_preserved: migration.state_preserved,
                migration_hash: migration.migration_hash,
            }),
        health: (action == LifecycleAction::Health).then_some(HealthSnapshot {
            meta: EvidenceMeta {
                status: EvidenceStatus::NotApplicable,
                reason: "health_observation",
                classification,
            },
            status_detail: health_state(after),
            loaded: after.loaded,
            running: after.running,
        }),
    }
}

fn hello_service_state_snapshot(snapshot: Snapshot) -> ServiceStateSnapshot<'static> {
    ServiceStateSnapshot {
        version: if snapshot.loaded {
            service_version(snapshot.load_descriptor)
        } else {
            "missing"
        },
        generation: snapshot.generation,
        loaded: snapshot.loaded,
        running: snapshot.running,
        state_counter: snapshot.state_counter,
        state_hash: hello_state_hash(snapshot.state_counter),
    }
}

fn hello_evidence_meta(
    verified: bool,
    verified_reason: &'static str,
    rejected_reason: &'static str,
    classification: EvidenceClassification,
) -> EvidenceMeta<'static> {
    EvidenceMeta {
        status: if verified {
            EvidenceStatus::Verified
        } else {
            EvidenceStatus::Rejected
        },
        reason: if verified {
            verified_reason
        } else {
            rejected_reason
        },
        classification,
    }
}

fn hello_evidence_classification(value: &str) -> EvidenceClassification {
    match value {
        "public" => EvidenceClassification::Public,
        "local_only" => EvidenceClassification::LocalOnly,
        "secret" => EvidenceClassification::Secret,
        _ => EvidenceClassification::Unknown,
    }
}

fn hello_response_service_value(
    snapshot: Snapshot,
    descriptor: LoadDescriptor,
    activation_status: &'static str,
    activation_active: bool,
) -> V<'static> {
    object(vec![
        j!("id" => s(descriptor.service_id)),
        j!("artifact_id" => s(descriptor.artifact_id)),
        j!("version" => s(service_version(descriptor))),
        j!("artifact_identity_id" => s(descriptor.artifact_identity.id)),
        j!("artifact_identity_hash" => sha(artifact_identity_hash(descriptor)),
        ),
        j!("artifact_identity_signature_envelope" => artifact_identity_signature_envelope_value(descriptor),
        ),
        j!("artifact_content_binding_id" => s(descriptor.artifact_identity.artifact_content_binding_id),
        ),
        j!("artifact_content_binding_hash" => sha(artifact_content_binding_hash(descriptor)),
        ),
        j!("artifact_content_source_hash" => sha(descriptor.artifact_identity.artifact_content_source_hash),
        ),
        j!("artifact_content_trust_envelope_id" => s(descriptor.artifact_identity.signed_envelope.id),
        ),
        j!("artifact_content_trust_envelope_hash" => sha(descriptor.artifact_identity.signed_envelope.envelope_hash),
        ),
        j!("artifact_reference_id" => s(descriptor.artifact_identity.artifact_reference_id),
        ),
        j!("artifact_reference_hash" => sha(artifact_reference_hash(descriptor)),
        ),
        j!("artifact_bytes_sha256" => sha(artifact_reference_bytes_hash(descriptor)),
        ),
        j!("artifact_reference_content_binding_hash" => sha(descriptor
                .artifact_identity
                .artifact_reference_content_binding_hash),
        ),
        j!("artifact_reference_trust_envelope_id" => s(descriptor.artifact_identity.signed_envelope.id),
        ),
        j!("artifact_reference_trust_envelope_hash" => sha(descriptor.artifact_identity.signed_envelope.envelope_hash),
        ),
        j!("artifact_load_plan_preflight_id" => s(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID),
        ),
        j!("artifact_load_plan_preflight_hash" => sha(artifact_load_plan_preflight_hash(descriptor)),
        ),
        j!("artifact_load_plan_preflight_status" => s(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS),
        ),
        j!("service_slot_activation_id" => s(SERVICE_SLOT_ACTIVATION_ID)),
        j!("service_slot_activation_hash" => sha(service_slot_activation_hash(descriptor)),
        ),
        j!("service_slot_activation_status" => s(activation_status)),
        j!("service_slot_activation_active" => b(activation_active)),
        j!("ram_only_service_slot_id" => s(RAM_ONLY_SERVICE_SLOT_ID)),
        j!("load_descriptor_id" => s(descriptor.id)),
        j!("load_descriptor_source_kind" => s(descriptor.source_kind)),
        j!("load_descriptor_source_validated" => b(true)),
        j!("load_descriptor_source_hash" => sha(descriptor_source_hash(descriptor)),
        ),
        j!("load_descriptor_source_signature_envelope" => descriptor_source_signature_envelope_value(descriptor),
        ),
        j!("kind" => s("service")),
        j!("loaded" => b(snapshot.loaded)),
        j!("running" => b(snapshot.running)),
        j!("generation" => u(snapshot.generation)),
        j!("state" => hello_state_value(snapshot)),
        j!("health" => s(if snapshot.running {
                "healthy"
            } else if snapshot.loaded {
                "stopped"
            } else {
                "missing"
            }),
        ),
        j!("capabilities" => inline_str_array(CAPABILITIES)),
    ])
}

pub(crate) fn emit_artifact_load_plan_preflight(descriptor: LoadDescriptor) {
    emit_inline_value(artifact_load_plan_preflight_value(descriptor));
}

pub(crate) fn artifact_load_plan_preflight_value(descriptor: LoadDescriptor) -> V<'static> {
    let record = artifact_load_plan_preflight_record(descriptor);
    inline(vec![
        j!("schema" => s(record.schema)),
        j!("id" => s(record.id)),
        j!("scope" => s(record.scope)),
        j!("classification" => s(record.classification)),
        j!("status" => s(record.status)),
        j!("preflight_hash" => sha(record.preflight_hash)),
        j!("service_id" => s(record.service_id)),
        j!("artifact_id" => s(record.artifact_id)),
        j!("load_descriptor_id" => s(record.load_descriptor_id)),
        j!("descriptor_source_locator" => s(record.descriptor_source_locator),
        ),
        j!("descriptor_source_hash" => sha(record.descriptor_source_hash)),
        j!("artifact_identity_id" => s(record.artifact_identity_id)),
        j!("artifact_identity_hash" => sha(record.artifact_identity_hash)),
        j!("artifact_content_binding_hash" => sha(record.artifact_content_binding_hash),
        ),
        j!("artifact_reference_id" => s(record.artifact_reference_id)),
        j!("artifact_reference_hash" => sha(record.artifact_reference_hash),
        ),
        j!("artifact_bytes_sha256" => sha(record.artifact_bytes_sha256)),
        j!("service_slot_intent_schema" => s(record.service_slot_intent_schema),
        ),
        j!("service_slot_intent_id" => s(record.service_slot_intent_id)),
        j!("ram_only_service_slot_id" => s(record.ram_only_service_slot_id),
        ),
        j!("accepted" => b(record.accepted)),
        j!("authorizes_builtin_current_boot_start" => b(record.authorizes_builtin_current_boot_start),
        ),
        j!("authorizes_candidate_artifact_execution" => b(record.authorizes_candidate_artifact_execution),
        ),
        j!("accepts_external_artifact_bytes" => b(record.accepts_external_artifact_bytes),
        ),
        j!("loads_candidate_bytes" => b(record.loads_candidate_bytes)),
        j!("maps_executable_pages" => b(record.maps_executable_pages)),
        j!("writes_persistent_state" => b(record.writes_persistent_state)),
        j!("writes_durable_audit_log" => b(record.writes_durable_audit_log),
        ),
        j!("installs_rollback_plan" => b(record.installs_rollback_plan)),
        j!("grants_broad_mutation" => b(record.grants_broad_mutation)),
    ])
}

fn service_slot_activation_value(
    descriptor: LoadDescriptor,
    status: &'static str,
    active: bool,
) -> V<'static> {
    let record = service_slot_activation_record(descriptor, status, active);
    inline(vec![
        j!("schema" => s(record.schema)),
        j!("id" => s(record.id)),
        j!("scope" => s(record.scope)),
        j!("classification" => s(record.classification)),
        j!("persistence" => s(record.persistence)),
        j!("status" => s(record.status)),
        j!("activation_hash" => sha(record.activation_hash)),
        j!("service_id" => s(record.service_id)),
        j!("artifact_id" => s(record.artifact_id)),
        j!("load_descriptor_id" => s(record.load_descriptor_id)),
        j!("descriptor_source_hash" => sha(record.descriptor_source_hash)),
        j!("artifact_load_plan_preflight_id" => s(record.artifact_load_plan_preflight_id),
        ),
        j!("artifact_load_plan_preflight_hash" => sha(record.artifact_load_plan_preflight_hash),
        ),
        j!("artifact_load_plan_preflight_status" => s(record.artifact_load_plan_preflight_status),
        ),
        j!("service_slot_intent_id" => s(record.service_slot_intent_id)),
        j!("ram_only_service_slot_id" => s(record.ram_only_service_slot_id),
        ),
        j!("active" => b(record.active)),
        j!("accepted_preflight" => b(record.accepted_preflight)),
        j!("authorizes_builtin_current_boot_start" => b(record.authorizes_builtin_current_boot_start),
        ),
        j!("authorizes_candidate_artifact_execution" => b(record.authorizes_candidate_artifact_execution),
        ),
        j!("writes_persistent_state" => b(record.writes_persistent_state)),
    ])
}

pub(crate) fn emit_hello_state(snapshot: Snapshot) {
    emit_inline_value(hello_state_value(snapshot));
}

fn hello_state_value(snapshot: Snapshot) -> V<'static> {
    inline(vec![
        j!("schema" => s(HELLO_STATE_SCHEMA)),
        j!("id" => s(HELLO_STATE_ID)),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("persistence" => s("none")),
        j!("service_id" => s(SERVICE_ID)),
        j!("version" => s(service_version(snapshot.load_descriptor))),
        j!("ram_only_service_slot_id" => s(RAM_ONLY_SERVICE_SLOT_ID)),
        j!("state_counter" => u(snapshot.state_counter)),
        j!("state_hash" => sha(hello_state_hash(snapshot.state_counter))),
        j!("loaded" => b(snapshot.loaded)),
        j!("running" => b(snapshot.running)),
        j!("writes_persistent_state" => b(false)),
    ])
}

fn hello_state_migration_value(record: Option<HelloStateMigrationRecord>) -> V<'static> {
    let Some(record) = record else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(record.schema)),
        j!("id" => s(record.id)),
        j!("scope" => s(record.scope)),
        j!("classification" => s(record.classification)),
        j!("persistence" => s(record.persistence)),
        j!("migration_hash" => sha(record.migration_hash)),
        j!("service_id" => s(record.service_id)),
        j!("ram_only_service_slot_id" => s(record.ram_only_service_slot_id),
        ),
        j!("from_version" => s(record.from_version)),
        j!("to_version" => s(record.to_version)),
        j!("pre_state_hash" => sha(record.pre_state_hash)),
        j!("post_state_hash" => sha(record.post_state_hash)),
        j!("pre_state_counter" => u(record.pre_state_counter)),
        j!("post_state_counter" => u(record.post_state_counter)),
        j!("state_preserved" => b(record.state_preserved)),
        j!("accepted" => b(record.accepted)),
        j!("writes_persistent_state" => b(record.writes_persistent_state)),
        j!("writes_durable_audit_log" => b(record.writes_durable_audit_log),
        ),
        j!("installs_rollback_plan" => b(record.installs_rollback_plan)),
    ])
}

pub(crate) fn emit_hello_hot_swap_probation_option(record: Option<HelloHotSwapProbationRecord>) {
    emit_inline_value(hello_hot_swap_probation_value(record));
}

fn hello_hot_swap_probation_value(record: Option<HelloHotSwapProbationRecord>) -> V<'static> {
    let Some(record) = record else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(record.schema)),
        j!("id" => s(record.id)),
        j!("scope" => s(record.scope)),
        j!("classification" => s(record.classification)),
        j!("persistence" => s(record.persistence)),
        j!("status" => s(record.status)),
        j!("probation_hash" => sha(record.probation_hash)),
        j!("service_id" => s(record.service_id)),
        j!("ram_only_service_slot_id" => s(record.ram_only_service_slot_id),
        ),
        j!("previous_version" => s(record.previous_version)),
        j!("new_version" => s(record.new_version)),
        j!("previous_descriptor_id" => s(record.previous_descriptor_id)),
        j!("new_descriptor_id" => s(record.new_descriptor_id)),
        j!("previous_descriptor_source_hash" => sha(record.previous_descriptor_source_hash),
        ),
        j!("new_descriptor_source_hash" => sha(record.new_descriptor_source_hash),
        ),
        j!("previous_artifact_identity_id" => s(record.previous_artifact_identity_id),
        ),
        j!("new_artifact_identity_id" => s(record.new_artifact_identity_id),
        ),
        j!("previous_artifact_identity_hash" => sha(record.previous_artifact_identity_hash),
        ),
        j!("new_artifact_identity_hash" => sha(record.new_artifact_identity_hash),
        ),
        j!("previous_generation" => u(record.previous_generation)),
        j!("new_generation" => u(record.new_generation)),
        j!("previous_state_hash" => sha(record.previous_state_hash)),
        j!("new_state_hash" => sha(record.new_state_hash)),
        j!("previous_state_counter" => u(record.previous_state_counter)),
        j!("new_state_counter" => u(record.new_state_counter)),
        j!("state_migration_hash" => sha(record.state_migration_hash)),
        j!("accepted" => b(record.accepted)),
        j!("loads_candidate_bytes" => b(record.loads_candidate_bytes)),
        j!("maps_executable_pages" => b(record.maps_executable_pages)),
        j!("writes_persistent_state" => b(record.writes_persistent_state)),
        j!("writes_durable_audit_log" => b(record.writes_durable_audit_log),
        ),
        j!("installs_rollback_plan" => b(record.installs_rollback_plan)),
        j!("applies_rollback" => b(record.applies_rollback)),
    ])
}

pub(crate) fn emit_descriptor_source_signature_envelope(descriptor: LoadDescriptor) {
    emit_inline_value(descriptor_source_signature_envelope_value(descriptor));
}

pub(crate) fn descriptor_source_signature_envelope_value(descriptor: LoadDescriptor) -> V<'static> {
    let Some(envelope) = descriptor.source_envelope else {
        return V::Null;
    };
    inline(vec![
        j!("schema" => s(envelope.schema)),
        j!("id" => s(envelope.id)),
        j!("algorithm" => s(envelope.algorithm)),
        j!("verification_phase" => s(envelope.verification_phase)),
        j!("trust_scope" => s(envelope.trust_scope)),
        j!("envelope_hash" => sha(envelope.envelope_hash)),
        j!("payload_sha256" => sha(envelope.payload_hash)),
        j!("public_key_sha256" => sha(envelope.public_key_hash)),
        j!("signature_sha256" => sha(envelope.signature_hash)),
        j!("signature_verified" => b(descriptor_source_signature_verified(descriptor)),
        ),
        j!("authorizes_external_artifact_load" => b(envelope.authorizes_external_artifact_load),
        ),
        j!("authorizes_persistent_install" => b(envelope.authorizes_persistent_install),
        ),
    ])
}

fn artifact_identity_value(descriptor: LoadDescriptor) -> V<'static> {
    let identity = descriptor.artifact_identity;
    inline(vec![
        j!("schema" => s(identity.schema)),
        j!("id" => s(identity.id)),
        j!("canonicalization" => s(identity.canonicalization)),
        j!("sha256" => sha(artifact_identity_hash(descriptor))),
        j!("service_id" => s(identity.service_id)),
        j!("artifact_id" => s(identity.artifact_id)),
        j!("artifact_kind" => s(identity.artifact_kind)),
        j!("load_descriptor_id" => s(identity.load_descriptor_id)),
        j!("scope" => s(identity.scope)),
        j!("classification" => s(identity.classification)),
        j!("persistence" => s(identity.persistence)),
        j!("content_binding" => artifact_content_binding_value(descriptor),
        ),
        j!("artifact_reference" => artifact_reference_value(descriptor)),
        j!("signature_envelope" => artifact_identity_signature_envelope_value(descriptor),
        ),
        j!("validated" => b(descriptor_sources::validate_builtin_hello_artifact_identity(identity)),
        ),
        j!("accepts_external_artifact_bytes" => b(identity.accepts_external_artifact_bytes),
        ),
        j!("loads_external_artifact" => b(identity.loads_external_artifact),
        ),
        j!("maps_executable_pages" => b(identity.maps_executable_pages)),
        j!("writes_persistent_state" => b(identity.writes_persistent_state),
        ),
        j!("authorizes_external_artifact_load" => b(identity.authorizes_external_artifact_load),
        ),
        j!("authorizes_persistent_install" => b(identity.authorizes_persistent_install),
        ),
        j!("authorizes_rollback_install" => b(identity.authorizes_rollback_install),
        ),
    ])
}

fn artifact_content_binding_value(descriptor: LoadDescriptor) -> V<'static> {
    let identity = descriptor.artifact_identity;
    inline(vec![
        j!("schema" => s(identity.artifact_content_binding_schema)),
        j!("id" => s(identity.artifact_content_binding_id)),
        j!("artifact_id" => s(identity.artifact_id)),
        j!("content_kind" => s(identity.artifact_content_kind)),
        j!("source_locator" => s(identity.artifact_content_source_locator),
        ),
        j!("source_sha256" => sha(identity.artifact_content_source_hash)),
        j!("binding_hash" => sha(artifact_content_binding_hash(descriptor)),
        ),
        j!("trusted_by_envelope_id" => s(identity.signed_envelope.id)),
        j!("trusted_by_envelope_hash" => sha(identity.signed_envelope.envelope_hash),
        ),
        j!("trust_signature_verified" => b(artifact_identity_signature_verified(descriptor)),
        ),
        j!("validated" => b(descriptor_sources::validate_builtin_hello_artifact_identity(identity)),
        ),
        j!("accepts_external_artifact_bytes" => b(identity.artifact_content_accepts_external_artifact_bytes),
        ),
        j!("loads_external_artifact" => b(identity.artifact_content_loads_external_artifact),
        ),
        j!("maps_executable_pages" => b(identity.artifact_content_maps_executable_pages),
        ),
        j!("writes_persistent_state" => b(identity.artifact_content_writes_persistent_state),
        ),
    ])
}

pub(crate) fn emit_artifact_reference(descriptor: LoadDescriptor) {
    emit_inline_value(artifact_reference_value(descriptor));
}

pub(crate) fn artifact_reference_value(descriptor: LoadDescriptor) -> V<'static> {
    let identity = descriptor.artifact_identity;
    inline(vec![
        j!("schema" => s(identity.artifact_reference_schema)),
        j!("id" => s(identity.artifact_reference_id)),
        j!("artifact_id" => s(identity.artifact_id)),
        j!("service_id" => s(identity.service_id)),
        j!("reference_kind" => s(identity.artifact_reference_kind)),
        j!("artifact_locator" => s(identity.artifact_reference_locator)),
        j!("artifact_bytes_sha256" => sha(identity.artifact_reference_bytes_hash),
        ),
        j!("content_binding_hash" => sha(identity.artifact_reference_content_binding_hash),
        ),
        j!("reference_hash" => sha(artifact_reference_hash(descriptor))),
        j!("trusted_by_envelope_id" => s(identity.signed_envelope.id)),
        j!("trusted_by_envelope_hash" => sha(identity.signed_envelope.envelope_hash),
        ),
        j!("trust_signature_verified" => b(artifact_identity_signature_verified(descriptor)),
        ),
        j!("validated" => b(descriptor_sources::validate_builtin_hello_artifact_identity(identity)),
        ),
        j!("accepts_external_artifact_bytes" => b(identity.artifact_reference_accepts_external_artifact_bytes),
        ),
        j!("loads_artifact_as_code" => b(identity.artifact_reference_loads_artifact_as_code),
        ),
        j!("maps_executable_pages" => b(identity.artifact_reference_maps_executable_pages),
        ),
        j!("writes_persistent_state" => b(identity.artifact_reference_writes_persistent_state),
        ),
    ])
}

pub(crate) fn emit_artifact_identity_signature_envelope(descriptor: LoadDescriptor) {
    emit_inline_value(artifact_identity_signature_envelope_value(descriptor));
}

pub(crate) fn artifact_identity_signature_envelope_value(descriptor: LoadDescriptor) -> V<'static> {
    let envelope = descriptor.artifact_identity.signed_envelope;
    inline(vec![
        j!("schema" => s(envelope.schema)),
        j!("id" => s(envelope.id)),
        j!("algorithm" => s(envelope.algorithm)),
        j!("verification_phase" => s(envelope.verification_phase)),
        j!("trust_scope" => s(envelope.trust_scope)),
        j!("envelope_hash" => sha(envelope.envelope_hash)),
        j!("payload_sha256" => sha(envelope.payload_hash)),
        j!("public_key_sha256" => sha(envelope.public_key_hash)),
        j!("signature_sha256" => sha(envelope.signature_hash)),
        j!("signature_verified" => b(artifact_identity_signature_verified(descriptor)),
        ),
        j!("authorizes_external_artifact_load" => b(envelope.authorizes_external_artifact_load),
        ),
        j!("authorizes_persistent_install" => b(envelope.authorizes_persistent_install),
        ),
        j!("authorizes_rollback_install" => b(envelope.authorizes_rollback_install),
        ),
    ])
}
