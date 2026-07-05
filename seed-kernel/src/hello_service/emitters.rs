use alloc::{vec, vec::Vec};

use super::*;
use crate::agent_protocol_support::{
    begin_error, emit_record_fields, emit_record_fields_trailing_comma, emit_record_value_fragment,
    end_error,
};
use raios_core::record::{Field, Value as V};

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
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let descriptor = snapshot.load_descriptor;
    let activation_status = service_slot_activation_status(snapshot);
    let activation_active = service_slot_activation_active(snapshot);
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => s("raios.ram_only_hello_service.health.v0")),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("action" => s("health_probe")),
            j!("event_id" => event_opt(Some(event_id))),
            j!("audit_event_id" => event_opt(Some(event_id))),
            j!("service_slot_activation" => service_slot_activation_value(descriptor, activation_status, activation_active),
            ),
            j!("state" => hello_state_value(snapshot)),
            j!("state_migration" => hello_state_migration_value(snapshot.state_migration),
            ),
            j!("hot_swap_probation" => hello_hot_swap_probation_value(snapshot.hot_swap_probation),
            ),
            j!("service" => health_service_value(snapshot, descriptor, activation_status, activation_active),
            ),
            j!("load_descriptor" => health_load_descriptor_value(descriptor, activation_status, activation_active),
            ),
            j!("denied_surfaces" => hello_basic_denied_surfaces_value()),
        ],
        6,
    );
    end_response(method);
}

fn health_service_value(
    snapshot: Snapshot,
    descriptor: LoadDescriptor,
    activation_status: &'static str,
    activation_active: bool,
) -> V<'static> {
    object(vec![
        j!("id" => s(SERVICE_ID)),
        j!("kind" => s("service")),
        j!("version" => s(service_version(descriptor))),
        j!("loaded" => b(snapshot.loaded)),
        j!("running" => b(snapshot.running)),
        j!("health" => s(health_state(snapshot))),
        j!("generation" => u(snapshot.generation)),
        j!("state" => hello_state_value(snapshot)),
        j!("last_action" => s(snapshot.last_action)),
        j!("last_reason" => s(snapshot.last_reason)),
        j!("service_slot_activation_id" => s(SERVICE_SLOT_ACTIVATION_ID)),
        j!("service_slot_activation_hash" => sha(service_slot_activation_hash(descriptor))),
        j!("service_slot_activation_status" => s(activation_status)),
        j!("service_slot_activation_active" => b(activation_active)),
        j!("capabilities" => inline_str_array(CAPABILITIES)),
    ])
}

fn health_load_descriptor_value(
    descriptor: LoadDescriptor,
    activation_status: &'static str,
    activation_active: bool,
) -> V<'static> {
    object(vec![
        j!("schema" => s(descriptor.schema)),
        j!("id" => s(descriptor.id)),
        j!("source" => object(vec![
                j!("locator" => s(descriptor.source_locator)),
                j!("kind" => s(descriptor.source_kind)),
                j!("validated" => b(true)),
                j!("sha256" => sha(descriptor_source_hash(descriptor))),
                j!("binds_source_locator" => s_opt(descriptor.binds_source_locator),
                ),
                j!("binds_source_kind" => s_opt(descriptor.binds_source_kind)),
                j!("binds_source_hash" => sha_opt(descriptor.binds_source_hash)),
                j!("signature_envelope" => descriptor_source_signature_envelope_value(descriptor),
                ),
            ]),
        ),
        j!("artifact_identity" => artifact_identity_value(descriptor)),
        j!("artifact_load_plan_preflight" => artifact_load_plan_preflight_value(descriptor),
        ),
        j!("service_slot_activation" => service_slot_activation_value(descriptor, activation_status, activation_active),
        ),
    ])
}

fn hello_basic_denied_surfaces_value() -> V<'static> {
    object(vec![
        j!("external_artifact_load" => s("denied")),
        j!("persistent_install" => s("denied")),
        j!("durable_audit" => s("denied")),
        j!("rollback_install" => s("denied")),
        j!("broad_mutation" => s("denied")),
    ])
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
            j!("target" => s("svc.demo.hello.reset_state")),
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

pub(crate) fn emit_rollback_transaction_preflight(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    emit_inline_value(rollback_transaction_preflight_value(snapshot, probation));
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

pub(crate) fn emit_rollback_write_authority_gate(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    emit_inline_value(rollback_write_authority_gate_value(snapshot, probation));
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
        j!("gate_hash" => sha(hello_rollback_write_authority_gate_hash(
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

pub(crate) fn emit_rollback_append_intent_gate(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    emit_inline_value(rollback_append_intent_gate_value(snapshot, probation));
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
        j!("gate_hash" => sha(hello_rollback_append_intent_gate_hash(snapshot, probation)),
        ),
        j!("rollback_write_authority_gate_hash" => sha(hello_rollback_write_authority_gate_hash(
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

pub(crate) fn emit_rollback_payload_envelope_gate(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    emit_inline_value(rollback_payload_envelope_gate_value(snapshot, probation));
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
        j!("gate_hash" => sha(hello_rollback_payload_envelope_gate_hash(
                snapshot, probation,
            )),
        ),
        j!("rollback_append_intent_gate_hash" => sha(hello_rollback_append_intent_gate_hash(snapshot, probation)),
        ),
        j!("rollback_write_authority_gate_hash" => sha(hello_rollback_write_authority_gate_hash(
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

pub(crate) fn emit_rollback_transaction_writer_storage_authority_gate(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    if let Some(probation) = probation {
        let foundation = hello_rollback_writer_storage_foundation();
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
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_STATUS);
        raw(", \"service_id\": ");
        json_str(SERVICE_ID);
        raw(", \"requested_capability\": ");
        json_str(HELLO_ROLLBACK_APPLY_CAPABILITY);
        raw(", \"gate_hash\": ");
        json_sha256(
            hello_rollback_transaction_writer_storage_authority_gate_hash(snapshot, probation),
        );
        raw(", \"rollback_payload_envelope_gate_hash\": ");
        json_sha256(hello_rollback_payload_envelope_gate_hash(
            snapshot, probation,
        ));
        raw(", \"payload_hash\": ");
        json_sha256(hello_rollback_transaction_payload_hash(snapshot, probation));
        raw(", \"provenance_hash\": ");
        json_sha256(hello_rollback_transaction_payload_provenance_hash(
            snapshot, probation,
        ));
        raw(", \"rollback_append_intent_gate_hash\": ");
        json_sha256(hello_rollback_append_intent_gate_hash(snapshot, probation));
        raw(", \"rollback_write_authority_gate_hash\": ");
        json_sha256(hello_rollback_write_authority_gate_hash(
            snapshot, probation,
        ));
        raw(", \"rollback_transaction_preflight_hash\": ");
        json_sha256(hello_rollback_transaction_preflight_hash(
            snapshot, probation,
        ));
        raw(", \"rollback_apply_hash\": ");
        json_sha256(hello_rollback_apply_denial_hash(snapshot, probation));
        raw(", \"rollback_preview_hash\": ");
        json_sha256(hello_rollback_preview_hash(snapshot, probation));
        raw(", \"source_probation_hash\": ");
        json_sha256(probation.probation_hash);
        raw(", \"current_state_hash\": ");
        json_sha256(hello_state_hash(snapshot.state_counter));
        raw(", \"current_state_counter\": ");
        raw_fmt(format_args!("{}", snapshot.state_counter));
        raw(", \"rollback_target_descriptor_source_hash\": ");
        json_sha256(probation.previous_descriptor_source_hash);
        raw(", \"rollback_target_artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"current_candidate_descriptor_source_hash\": ");
        json_sha256(probation.new_descriptor_source_hash);
        raw(", \"current_candidate_artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"state_migration_hash\": ");
        json_sha256(probation.state_migration_hash);
        raw(", \"required_schemas\": {\"audit_record\": \"raios.audit_record.v0\", \"rollback_transaction\": \"raios.rollback_transaction.v0\"}");
        raw(", \"writer_storage_foundation\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_SCHEMA);
        raw(", \"owner_method\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_OWNER);
        raw(", \"recovery_visible_method\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_OWNER);
        raw(", \"storage_authority\": {\"schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_SCHEMA);
        raw(", \"id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
        raw(", \"owner_method\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER);
        raw(", \"source_method\": ");
        json_str(rollback_storage_layout::MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_METHOD);
        raw(", \"status\": ");
        json_str(foundation.storage_layout_status);
        raw(", \"reason\": ");
        json_str(foundation.storage_layout_reason);
        raw(", \"available\": ");
        raw_bool(foundation.storage_layout_available);
        raw(", \"authorizes_append\": false}");
        raw(", \"append_targets\": {\"audit_ledger\": {\"id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"available\": ");
        raw_bool(foundation.durable_audit_store_available);
        raw("}, \"rollback_store\": {\"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID);
        raw(", \"schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"available\": ");
        raw_bool(foundation.rollback_store_available);
        raw(", \"append_available\": ");
        raw_bool(foundation.rollback_transaction_append_available);
        raw("}}");
        raw(", \"target_region_discovery\": ");
        emit_audit_rollback_target_region_discovery_inline(foundation.target_region_discovery);
        raw(", \"append_target_owner\": {\"schema\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_SCHEMA);
        raw(", \"id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
        raw(", \"owner_method\": ");
        json_str(rollback_append_contract::MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_METHOD);
        raw(", \"storage_authority_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
        raw(", \"status\": ");
        json_str(foundation.append_target_owner_status);
        raw(", \"reason\": ");
        json_str(foundation.append_target_owner_reason);
        raw(", \"available\": ");
        raw_bool(foundation.append_target_owner_available);
        raw(", \"block_write_path_available\": ");
        raw_bool(foundation.block_write_path_available);
        raw(", \"block_write_path_reason\": ");
        json_str(foundation.block_write_path_reason);
        raw(", \"authorizes_append\": false}");
        raw(", \"transaction_writer_readiness\": {\"schema\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_SCHEMA);
        raw(", \"id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID);
        raw(", \"owner_method\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
        raw(", \"append_target_owner_id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
        raw(", \"status\": ");
        json_str(foundation.transaction_writer_status);
        raw(", \"reason\": ");
        json_str(foundation.transaction_writer_reason);
        raw(", \"ready\": ");
        raw_bool(foundation.transaction_writer_ready);
        raw(", \"block_write_path_available\": ");
        raw_bool(foundation.block_write_path_available);
        raw(", \"block_write_path_reason\": ");
        json_str(foundation.block_write_path_reason);
        raw(", \"scratch_only_writer_dry_run\": {\"schema\": ");
        json_str(
            rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_SCHEMA,
        );
        raw(", \"id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"status\": ");
        json_str(foundation.scratch_writer_dry_run_status);
        raw(", \"reason\": ");
        json_str(foundation.scratch_writer_dry_run_reason);
        raw(", \"source_authority_id\": ");
        json_str(foundation.scratch_block_write_authority_id);
        raw(", \"source_region_id\": ");
        json_str(foundation.scratch_region_id);
        raw(", \"target_region_start_lba\": ");
        raw_fmt(format_args!("{}", foundation.scratch_region_start_lba));
        raw(", \"target_region_lba_count\": ");
        raw_fmt(format_args!("{}", foundation.scratch_region_lba_count));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!("{}", foundation.scratch_region_byte_count));
        raw(", \"target_range_scratch_owned\": ");
        raw_bool(foundation.scratch_block_write_authority_available);
        raw(", \"target_range_within_device_bounds\": ");
        raw_bool(foundation.scratch_region_within_device_bounds);
        raw(", \"target_range_no_boot_or_partition_metadata_overlap\": ");
        raw_bool(foundation.scratch_region_no_boot_or_partition_metadata_overlap);
        raw(", \"target_range_ready\": ");
        raw_bool(foundation.scratch_writer_dry_run_ready);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"append_record_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"canonicalization\": ");
        json_str(HELLO_ROLLBACK_APPEND_RECORD_CANONICALIZATION);
        raw(", \"status\": ");
        json_str(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_REASON);
        raw(", \"dry_run_hash\": ");
        json_sha256(append_record.dry_run_hash);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"audit_record_image_hash\": ");
        json_sha256(append_record.audit_record_image_hash);
        raw(", \"audit_record_byte_length\": ");
        raw_fmt(format_args!("{}", append_record.audit_record_byte_length));
        raw(", \"rollback_store_target_id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_transaction_image_hash\": ");
        json_sha256(append_record.rollback_transaction_image_hash);
        raw(", \"rollback_transaction_byte_length\": ");
        raw_fmt(format_args!(
            "{}",
            append_record.rollback_transaction_byte_length
        ));
        raw(", \"total_record_byte_length\": ");
        raw_fmt(format_args!("{}", append_record.total_record_byte_length));
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!("{}", append_record.target_start_lba));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!("{}", append_record.target_lba_count));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!("{}", append_record.target_byte_count));
        raw(", \"target_range_ready\": ");
        raw_bool(append_record.target_range_ready);
        raw(", \"source_payload_hash\": ");
        json_sha256(hello_rollback_transaction_payload_hash(snapshot, probation));
        raw(", \"source_provenance_hash\": ");
        json_sha256(hello_rollback_transaction_payload_provenance_hash(
            snapshot, probation,
        ));
        raw(", \"sector_plan_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"canonicalization\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_CANONICALIZATION);
        raw(", \"status\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_REASON);
        raw(", \"plan_hash\": ");
        json_sha256(sector_plan.plan_hash);
        raw(", \"sector_image_hash\": ");
        json_sha256(sector_plan.sector_image_hash);
        raw(", \"sector_size_bytes\": ");
        raw_fmt(format_args!("{}", sector_plan.sector_size_bytes));
        raw(", \"audit_record_offset\": ");
        raw_fmt(format_args!("{}", sector_plan.audit_record_offset));
        raw(", \"audit_record_byte_length\": ");
        raw_fmt(format_args!("{}", sector_plan.audit_record_byte_length));
        raw(", \"rollback_transaction_offset\": ");
        raw_fmt(format_args!("{}", sector_plan.rollback_transaction_offset));
        raw(", \"rollback_transaction_byte_length\": ");
        raw_fmt(format_args!(
            "{}",
            sector_plan.rollback_transaction_byte_length
        ));
        raw(", \"padding_policy\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PADDING_POLICY);
        raw(", \"padding_offset\": ");
        raw_fmt(format_args!("{}", sector_plan.padding_offset));
        raw(", \"padding_byte_length\": ");
        raw_fmt(format_args!("{}", sector_plan.padding_byte_length));
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!("{}", sector_plan.target_start_lba));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!("{}", sector_plan.target_lba_count));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!("{}", sector_plan.target_byte_count));
        raw(", \"target_range_ready\": ");
        raw_bool(sector_plan.target_range_ready);
        raw(", \"source_append_record_hash\": ");
        json_sha256(append_record.dry_run_hash);
        raw(", \"scratch_sector_write_readback_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(sector_write.status);
        raw(", \"reason\": ");
        json_str(sector_write.reason);
        raw(", \"dry_run_hash\": ");
        json_sha256(sector_write.dry_run_hash);
        raw(", \"source_plan_hash\": ");
        json_sha256(sector_write.source_plan_hash);
        raw(", \"planned_sector_image_hash\": ");
        json_sha256(sector_write.planned_sector_image_hash);
        raw(", \"readback_sector_image_hash\": ");
        json_sha256(sector_write.readback_sector_image_hash);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!("{}", sector_write.target_start_lba));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!("{}", sector_write.target_lba_count));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!("{}", sector_write.target_byte_count));
        raw(", \"label_found\": ");
        raw_bool(sector_write.label_found);
        raw(", \"target_range_ready\": ");
        raw_bool(sector_write.target_range_ready);
        raw(", \"write_attempted\": ");
        raw_bool(sector_write.write_attempted);
        raw(", \"write_completed\": ");
        raw_bool(sector_write.write_completed);
        raw(", \"readback_completed\": ");
        raw_bool(sector_write.readback_completed);
        raw(", \"readback_matches_planned_image\": ");
        raw_bool(sector_write.readback_matches_planned_image);
        raw(", \"durable_append_authority_preflight\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_REASON);
        raw(", \"preflight_hash\": ");
        json_sha256(durable_append_preflight.preflight_hash);
        raw(", \"source_write_readback_hash\": ");
        json_sha256(durable_append_preflight.source_write_readback_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_append_preflight.source_target_region_write_readback_hash);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(durable_append_preflight.test_infrastructure_media_write_authority_available);
        raw(", \"remaining_denial_reason\": ");
        json_str(durable_append_preflight.remaining_denial_reason);
        let durable_writer_policy_preflight =
            durable_append_preflight.durable_writer_policy_preflight;
        raw(", \"durable_writer_policy_preflight\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_REASON);
        raw(", \"preflight_hash\": ");
        json_sha256(durable_writer_policy_preflight.preflight_hash);
        raw(", \"source_append_record_hash\": ");
        json_sha256(durable_writer_policy_preflight.source_append_record_hash);
        raw(", \"source_sector_plan_hash\": ");
        json_sha256(durable_writer_policy_preflight.source_sector_plan_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_writer_policy_preflight.source_target_region_write_readback_hash);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_writer_policy_preflight.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_writer_policy_preflight.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_writer_policy_preflight.target_byte_count
        ));
        raw(", \"target_range_ready\": ");
        raw_bool(durable_writer_policy_preflight.target_range_ready);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_writer_policy_preflight.test_infrastructure_media_write_authority_available,
        );
        raw(", \"durable_audit_writer_available\": ");
        raw_bool(durable_writer_policy_preflight.durable_audit_writer_available);
        raw(", \"rollback_store_writer_available\": ");
        raw_bool(durable_writer_policy_preflight.rollback_store_writer_available);
        raw(", \"transaction_append_writer_available\": ");
        raw_bool(durable_writer_policy_preflight.transaction_append_writer_available);
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_append_transaction_authorization_gate =
            hello_rollback_durable_append_transaction_authorization_gate(
                durable_writer_policy_preflight,
                append_record,
                sector_plan,
                target_region_write,
            );
        raw(", \"durable_append_transaction_authorization_gate\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_REASON);
        raw(", \"gate_hash\": ");
        json_sha256(durable_append_transaction_authorization_gate.gate_hash);
        raw(", \"source_writer_policy_preflight_hash\": ");
        json_sha256(
            durable_append_transaction_authorization_gate.source_writer_policy_preflight_hash,
        );
        raw(", \"source_append_record_hash\": ");
        json_sha256(durable_append_transaction_authorization_gate.source_append_record_hash);
        raw(", \"source_sector_plan_hash\": ");
        json_sha256(durable_append_transaction_authorization_gate.source_sector_plan_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            durable_append_transaction_authorization_gate.source_target_region_write_readback_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_transaction_authorization_gate.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_transaction_authorization_gate.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_transaction_authorization_gate.target_byte_count
        ));
        raw(", \"target_range_ready\": ");
        raw_bool(durable_append_transaction_authorization_gate.target_range_ready);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_append_transaction_authorization_gate
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"append_engine_available\": ");
        raw_bool(durable_append_transaction_authorization_gate.append_engine_available);
        raw(", \"durable_audit_writer_available\": ");
        raw_bool(durable_append_transaction_authorization_gate.durable_audit_writer_available);
        raw(", \"rollback_store_writer_available\": ");
        raw_bool(durable_append_transaction_authorization_gate.rollback_store_writer_available);
        raw(", \"transaction_append_writer_available\": ");
        raw_bool(durable_append_transaction_authorization_gate.transaction_append_writer_available);
        raw(", \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let append_engine_readiness_decision = hello_rollback_append_engine_readiness_decision(
            durable_append_transaction_authorization_gate,
        );
        raw(", \"append_engine_readiness_decision\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(append_engine_readiness_decision.status);
        raw(", \"reason\": ");
        json_str(append_engine_readiness_decision.reason);
        raw(", \"decision_hash\": ");
        json_sha256(append_engine_readiness_decision.decision_hash);
        raw(", \"source_authorization_gate_hash\": ");
        json_sha256(append_engine_readiness_decision.source_authorization_gate_hash);
        raw(", \"source_writer_policy_preflight_hash\": ");
        json_sha256(append_engine_readiness_decision.source_writer_policy_preflight_hash);
        raw(", \"source_append_record_hash\": ");
        json_sha256(append_engine_readiness_decision.source_append_record_hash);
        raw(", \"source_sector_plan_hash\": ");
        json_sha256(append_engine_readiness_decision.source_sector_plan_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(append_engine_readiness_decision.source_target_region_write_readback_hash);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            append_engine_readiness_decision.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            append_engine_readiness_decision.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            append_engine_readiness_decision.target_byte_count
        ));
        raw(", \"target_range_ready\": ");
        raw_bool(append_engine_readiness_decision.target_range_ready);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            append_engine_readiness_decision.test_infrastructure_media_write_authority_available,
        );
        raw(", \"append_engine_available\": ");
        raw_bool(append_engine_readiness_decision.append_engine_available);
        raw(", \"durable_audit_writer_available\": ");
        raw_bool(append_engine_readiness_decision.durable_audit_writer_available);
        raw(", \"rollback_store_writer_available\": ");
        raw_bool(append_engine_readiness_decision.rollback_store_writer_available);
        raw(", \"transaction_append_writer_available\": ");
        raw_bool(append_engine_readiness_decision.transaction_append_writer_available);
        raw(", \"ready\": ");
        raw_bool(append_engine_readiness_decision.ready);
        raw(", \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        raw(", \"target_region_discovery\": ");
        emit_audit_rollback_target_region_discovery_inline(
            durable_append_preflight.target_region_discovery,
        );
        let target_region_media_write_policy_preflight =
            durable_append_preflight.target_region_media_write_policy_preflight;
        raw(", \"target_region_media_write_policy_preflight\": {\"schema\": ");
        json_str(
            rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,
        );
        raw(", \"id\": ");
        json_str(
            rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID,
        );
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(foundation.target_region_media_write_policy_preflight_status);
        raw(", \"reason\": ");
        json_str(foundation.target_region_media_write_policy_preflight_reason);
        raw(", \"preflight_hash\": ");
        json_sha256(target_region_media_write_policy_preflight.preflight_hash);
        raw(", \"source_contract_schema\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA);
        raw(", \"source_contract_id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID);
        raw(", \"source_contract_status\": ");
        json_str(target_region_media_write_policy_preflight.source_contract_status);
        raw(", \"source_contract_reason\": ");
        json_str(target_region_media_write_policy_preflight.source_contract_reason);
        raw(", \"owner_method\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
        raw(", \"append_target_owner_id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
        raw(", \"storage_authority_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_region_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_media_write_policy_preflight.target_region_start_lba
        ));
        raw(", \"target_region_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_media_write_policy_preflight.target_region_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_media_write_policy_preflight.target_byte_count
        ));
        raw(", \"source_contract_target_range_ready\": ");
        raw_bool(target_region_media_write_policy_preflight.source_contract_target_range_ready);
        raw(", \"owner_ids_verified\": ");
        raw_bool(target_region_media_write_policy_preflight.owner_ids_verified);
        raw(", \"target_ids_verified\": ");
        raw_bool(target_region_media_write_policy_preflight.target_ids_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(target_region_media_write_policy_preflight.target_span_verified);
        raw(", \"schema_ids_verified\": ");
        raw_bool(target_region_media_write_policy_preflight.schema_ids_verified);
        raw(", \"media_write_authority_required\": true, \"media_write_authority_available\": ");
        raw_bool(target_region_media_write_policy_preflight.media_write_authority_available);
        raw(", \"media_write_authority_reason\": ");
        json_str(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_MISSING_REASON);
        raw(", \"durable_audit_policy_required\": true, \"durable_audit_policy_available\": ");
        raw_bool(target_region_media_write_policy_preflight.durable_audit_policy_available);
        raw(", \"durable_audit_policy_reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_MISSING_REASON);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let media_write_authority_gate = hello_rollback_media_write_authority_gate(
            durable_append_preflight,
            target_region_write,
        );
        raw(", \"media_write_authority_gate\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_REASON);
        raw(", \"gate_hash\": ");
        json_sha256(media_write_authority_gate.gate_hash);
        raw(", \"source_durable_append_authority_preflight_schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA);
        raw(", \"source_durable_append_authority_preflight_id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID);
        raw(", \"source_durable_append_authority_preflight_hash\": ");
        json_sha256(media_write_authority_gate.source_durable_append_authority_preflight_hash);
        raw(", \"source_policy_preflight_schema\": ");
        json_str(
            rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,
        );
        raw(", \"source_policy_preflight_id\": ");
        json_str(
            rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID,
        );
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(media_write_authority_gate.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_schema\": ");
        json_str(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA);
        raw(", \"source_target_region_write_readback_id\": ");
        json_str(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(media_write_authority_gate.source_target_region_write_readback_hash);
        raw(", \"source_contract_schema\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA);
        raw(", \"source_contract_id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID);
        raw(", \"source_contract_status\": ");
        json_str(media_write_authority_gate.source_contract_status);
        raw(", \"source_contract_reason\": ");
        json_str(media_write_authority_gate.source_contract_reason);
        raw(", \"target_region_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            media_write_authority_gate.target_region_start_lba
        ));
        raw(", \"target_region_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            media_write_authority_gate.target_region_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            media_write_authority_gate.target_byte_count
        ));
        raw(", \"source_contract_target_range_ready\": ");
        raw_bool(media_write_authority_gate.source_contract_target_range_ready);
        raw(", \"owner_ids_verified\": ");
        raw_bool(media_write_authority_gate.owner_ids_verified);
        raw(", \"target_ids_verified\": ");
        raw_bool(media_write_authority_gate.target_ids_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(media_write_authority_gate.target_span_verified);
        raw(", \"schema_ids_verified\": ");
        raw_bool(media_write_authority_gate.schema_ids_verified);
        raw(", \"media_write_authority_required\": true, \"media_write_authority_available\": ");
        raw_bool(media_write_authority_gate.media_write_authority_available);
        raw(", \"media_write_authority_reason\": ");
        json_str(HELLO_ROLLBACK_TEST_MEDIA_WRITE_AUTHORITY_REASON);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(media_write_authority_gate.test_infrastructure_media_write_authority_available);
        raw(", \"durable_audit_policy_required\": true, \"durable_audit_policy_available\": ");
        raw_bool(media_write_authority_gate.durable_audit_policy_available);
        raw(", \"durable_audit_policy_reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_MISSING_REASON);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"target_region_write_attempted\": ");
        raw_bool(media_write_authority_gate.target_region_write_attempted);
        raw(", \"write_attempted\": false}");
        let durable_append_authority_decision = hello_rollback_durable_append_authority_decision(
            durable_append_preflight,
            media_write_authority_gate,
            append_engine_readiness_decision,
        );
        raw(", \"durable_append_authority_decision\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_REASON);
        raw(", \"decision_hash\": ");
        json_sha256(durable_append_authority_decision.decision_hash);
        raw(", \"source_durable_append_authority_preflight_hash\": ");
        json_sha256(
            durable_append_authority_decision.source_durable_append_authority_preflight_hash,
        );
        raw(", \"source_writer_policy_preflight_hash\": ");
        json_sha256(durable_append_authority_decision.source_writer_policy_preflight_hash);
        raw(", \"source_append_engine_readiness_decision_hash\": ");
        json_sha256(durable_append_authority_decision.source_append_engine_readiness_decision_hash);
        raw(", \"source_media_write_authority_gate_hash\": ");
        json_sha256(durable_append_authority_decision.source_media_write_authority_gate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_append_authority_decision.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_append_authority_decision.source_target_region_write_readback_hash);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_decision.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_decision.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_decision.target_byte_count
        ));
        raw(", \"writer_policy_ready\": ");
        raw_bool(durable_append_authority_decision.writer_policy_ready);
        raw(", \"append_engine_ready\": ");
        raw_bool(durable_append_authority_decision.append_engine_ready);
        raw(", \"media_write_gate_ready\": ");
        raw_bool(durable_append_authority_decision.media_write_gate_ready);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_append_authority_decision.test_infrastructure_media_write_authority_available,
        );
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_append_authority_decision.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_append_authority_decision.durable_append_authority_available);
        raw(", \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_audit_policy_decision =
            hello_rollback_durable_audit_policy_decision(durable_append_authority_decision);
        raw(", \"durable_audit_policy_decision\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_REASON);
        raw(", \"decision_hash\": ");
        json_sha256(durable_audit_policy_decision.decision_hash);
        raw(", \"source_durable_append_authority_decision_hash\": ");
        json_sha256(durable_audit_policy_decision.source_durable_append_authority_decision_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_audit_policy_decision.source_policy_preflight_hash);
        raw(", \"source_media_write_authority_gate_hash\": ");
        json_sha256(durable_audit_policy_decision.source_media_write_authority_gate_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_audit_policy_decision.source_target_region_write_readback_hash);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_decision.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_decision.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_decision.target_byte_count
        ));
        raw(", \"append_engine_ready\": ");
        raw_bool(durable_audit_policy_decision.append_engine_ready);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_audit_policy_decision.media_write_policy_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(durable_audit_policy_decision.test_infrastructure_media_write_authority_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_audit_policy_decision.durable_append_authority_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_audit_policy_decision.durable_audit_policy_available);
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_audit_policy_candidate = hello_rollback_durable_audit_policy_candidate(
            durable_audit_policy_decision,
            append_record,
        );
        raw(", \"durable_audit_policy_candidate\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_REASON);
        raw(", \"candidate_hash\": ");
        json_sha256(durable_audit_policy_candidate.candidate_hash);
        raw(", \"source_durable_audit_policy_decision_hash\": ");
        json_sha256(durable_audit_policy_candidate.source_durable_audit_policy_decision_hash);
        raw(", \"source_audit_record_image_hash\": ");
        json_sha256(durable_audit_policy_candidate.source_audit_record_image_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_audit_policy_candidate.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_audit_policy_candidate.source_target_region_write_readback_hash);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_candidate.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_candidate.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_candidate.target_byte_count
        ));
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_audit_policy_candidate.media_write_policy_verified);
        raw(", \"durable_audit_policy_candidate_available\": ");
        raw_bool(durable_audit_policy_candidate.durable_audit_policy_candidate_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_audit_policy_candidate.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_audit_policy_candidate.durable_append_authority_available);
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_audit_policy_acceptance_gate =
            hello_rollback_durable_audit_policy_acceptance_gate(durable_audit_policy_candidate);
        raw(", \"durable_audit_policy_acceptance_gate\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_REASON);
        raw(", \"gate_hash\": ");
        json_sha256(durable_audit_policy_acceptance_gate.gate_hash);
        raw(", \"source_durable_audit_policy_candidate_hash\": ");
        json_sha256(
            durable_audit_policy_acceptance_gate.source_durable_audit_policy_candidate_hash,
        );
        raw(", \"source_durable_audit_policy_decision_hash\": ");
        json_sha256(durable_audit_policy_acceptance_gate.source_durable_audit_policy_decision_hash);
        raw(", \"source_audit_record_image_hash\": ");
        json_sha256(durable_audit_policy_acceptance_gate.source_audit_record_image_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_audit_policy_acceptance_gate.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_audit_policy_acceptance_gate.source_target_region_write_readback_hash);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_acceptance_gate.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_acceptance_gate.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_acceptance_gate.target_byte_count
        ));
        raw(", \"candidate_available\": ");
        raw_bool(durable_audit_policy_acceptance_gate.candidate_available);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_audit_policy_acceptance_gate.media_write_policy_verified);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_audit_policy_acceptance_gate.durable_policy_ledger_available);
        raw(", \"write_authority_available\": ");
        raw_bool(durable_audit_policy_acceptance_gate.write_authority_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_audit_policy_acceptance_gate.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_audit_policy_acceptance_gate.durable_append_authority_available);
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_audit_policy_ledger_candidate =
            hello_rollback_durable_audit_policy_ledger_candidate(
                durable_audit_policy_acceptance_gate,
            );
        raw(", \"durable_audit_policy_ledger_candidate\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"access\": \"read_only\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_REASON);
        raw(", \"ledger_candidate_hash\": ");
        json_sha256(durable_audit_policy_ledger_candidate.ledger_candidate_hash);
        raw(", \"source_acceptance_gate_hash\": ");
        json_sha256(durable_audit_policy_ledger_candidate.source_acceptance_gate_hash);
        raw(", \"source_durable_audit_policy_candidate_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_candidate.source_durable_audit_policy_candidate_hash,
        );
        raw(", \"source_durable_audit_policy_decision_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_candidate.source_durable_audit_policy_decision_hash,
        );
        raw(", \"source_audit_record_image_hash\": ");
        json_sha256(durable_audit_policy_ledger_candidate.source_audit_record_image_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_audit_policy_ledger_candidate.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_audit_policy_ledger_candidate.source_target_region_write_readback_hash);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_ledger_candidate.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_ledger_candidate.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_ledger_candidate.target_byte_count
        ));
        raw(", \"read_only_ledger_candidate_available\": ");
        raw_bool(durable_audit_policy_ledger_candidate.read_only_ledger_candidate_available);
        raw(", \"candidate_available\": ");
        raw_bool(durable_audit_policy_ledger_candidate.candidate_available);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_audit_policy_ledger_candidate.media_write_policy_verified);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_audit_policy_ledger_candidate.durable_policy_ledger_available);
        raw(", \"write_authority_available\": ");
        raw_bool(durable_audit_policy_ledger_candidate.write_authority_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_audit_policy_ledger_candidate.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_audit_policy_ledger_candidate.durable_append_authority_available);
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_audit_policy_ledger_aware_acceptance_result =
            hello_rollback_durable_audit_policy_ledger_aware_acceptance_result(
                durable_audit_policy_ledger_candidate,
            );
        raw(", \"durable_audit_policy_ledger_aware_acceptance_result\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_REASON);
        raw(", \"result_hash\": ");
        json_sha256(durable_audit_policy_ledger_aware_acceptance_result.result_hash);
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_aware_acceptance_result.source_ledger_candidate_hash,
        );
        raw(", \"source_acceptance_gate_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_aware_acceptance_result.source_acceptance_gate_hash,
        );
        raw(", \"source_durable_audit_policy_candidate_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_aware_acceptance_result
                .source_durable_audit_policy_candidate_hash,
        );
        raw(", \"source_durable_audit_policy_decision_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_aware_acceptance_result
                .source_durable_audit_policy_decision_hash,
        );
        raw(", \"source_audit_record_image_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_aware_acceptance_result.source_audit_record_image_hash,
        );
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_aware_acceptance_result.source_policy_preflight_hash,
        );
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            durable_audit_policy_ledger_aware_acceptance_result
                .source_target_region_write_readback_hash,
        );
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_ledger_aware_acceptance_result.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_ledger_aware_acceptance_result.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_ledger_aware_acceptance_result.target_byte_count
        ));
        raw(", \"read_only_ledger_candidate_available\": ");
        raw_bool(
            durable_audit_policy_ledger_aware_acceptance_result
                .read_only_ledger_candidate_available,
        );
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_audit_policy_ledger_aware_acceptance_result.ledger_evidence_verified);
        raw(", \"write_authority_available\": ");
        raw_bool(durable_audit_policy_ledger_aware_acceptance_result.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(
            durable_audit_policy_ledger_aware_acceptance_result.durable_policy_ledger_available,
        );
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(
            durable_audit_policy_ledger_aware_acceptance_result.durable_audit_policy_available,
        );
        raw(", \"durable_append_authority_available\": ");
        raw_bool(
            durable_audit_policy_ledger_aware_acceptance_result.durable_append_authority_available,
        );
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_audit_policy_write_authority_availability =
            hello_rollback_durable_audit_policy_write_authority_availability(
                durable_audit_policy_ledger_aware_acceptance_result,
                durable_audit_policy_ledger_candidate,
                target_region_media_write_policy_preflight,
                target_region_write,
            );
        raw(", \"durable_audit_policy_write_authority_availability\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_REASON);
        raw(", \"availability_hash\": ");
        json_sha256(durable_audit_policy_write_authority_availability.availability_hash);
        raw(", \"source_ledger_aware_acceptance_result_hash\": ");
        json_sha256(
            durable_audit_policy_write_authority_availability
                .source_ledger_aware_acceptance_result_hash,
        );
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(durable_audit_policy_write_authority_availability.source_ledger_candidate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_audit_policy_write_authority_availability.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            durable_audit_policy_write_authority_availability
                .source_target_region_write_readback_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_write_authority_availability.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_write_authority_availability.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_write_authority_availability.target_byte_count
        ));
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_audit_policy_write_authority_availability.ledger_evidence_verified);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_audit_policy_write_authority_availability.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(
            durable_audit_policy_write_authority_availability.target_region_write_readback_verified,
        );
        raw(", \"target_span_verified\": ");
        raw_bool(durable_audit_policy_write_authority_availability.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(
            durable_audit_policy_write_authority_availability.audit_rollback_target_ids_verified,
        );
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_audit_policy_write_authority_availability
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_audit_policy_write_authority_availability.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_audit_policy_write_authority_availability.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_audit_policy_write_authority_availability.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(
            durable_audit_policy_write_authority_availability.durable_append_authority_available,
        );
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_policy_ledger_availability = hello_rollback_durable_policy_ledger_availability(
            durable_audit_policy_write_authority_availability,
        );
        raw(", \"durable_policy_ledger_availability\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_REASON);
        raw(", \"availability_hash\": ");
        json_sha256(durable_policy_ledger_availability.availability_hash);
        raw(", \"source_write_authority_availability_hash\": ");
        json_sha256(durable_policy_ledger_availability.source_write_authority_availability_hash);
        raw(", \"source_ledger_aware_acceptance_result_hash\": ");
        json_sha256(durable_policy_ledger_availability.source_ledger_aware_acceptance_result_hash);
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(durable_policy_ledger_availability.source_ledger_candidate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_policy_ledger_availability.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_policy_ledger_availability.source_target_region_write_readback_hash);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_ledger_availability.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_ledger_availability.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_ledger_availability.target_byte_count
        ));
        raw(", \"write_authority_evidence_verified\": ");
        raw_bool(durable_policy_ledger_availability.write_authority_evidence_verified);
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_policy_ledger_availability.ledger_evidence_verified);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_policy_ledger_availability.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(durable_policy_ledger_availability.target_region_write_readback_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(durable_policy_ledger_availability.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(durable_policy_ledger_availability.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_policy_ledger_availability.test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_policy_ledger_availability.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_policy_ledger_availability.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_policy_ledger_availability.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_policy_ledger_availability.durable_append_authority_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_audit_policy_availability =
            hello_rollback_durable_audit_policy_availability(durable_policy_ledger_availability);
        raw(", \"durable_audit_policy_availability\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_REASON);
        raw(", \"availability_hash\": ");
        json_sha256(durable_audit_policy_availability.availability_hash);
        raw(", \"source_policy_ledger_availability_hash\": ");
        json_sha256(durable_audit_policy_availability.source_policy_ledger_availability_hash);
        raw(", \"source_write_authority_availability_hash\": ");
        json_sha256(durable_audit_policy_availability.source_write_authority_availability_hash);
        raw(", \"source_ledger_aware_acceptance_result_hash\": ");
        json_sha256(durable_audit_policy_availability.source_ledger_aware_acceptance_result_hash);
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(durable_audit_policy_availability.source_ledger_candidate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_audit_policy_availability.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_audit_policy_availability.source_target_region_write_readback_hash);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_availability.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_availability.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_availability.target_byte_count
        ));
        raw(", \"policy_ledger_availability_evidence_verified\": ");
        raw_bool(durable_audit_policy_availability.policy_ledger_availability_evidence_verified);
        raw(", \"write_authority_evidence_verified\": ");
        raw_bool(durable_audit_policy_availability.write_authority_evidence_verified);
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_audit_policy_availability.ledger_evidence_verified);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_audit_policy_availability.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(durable_audit_policy_availability.target_region_write_readback_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(durable_audit_policy_availability.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(durable_audit_policy_availability.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_audit_policy_availability.test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_audit_policy_availability.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_audit_policy_availability.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_audit_policy_availability.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_audit_policy_availability.durable_append_authority_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_append_authority_availability =
            hello_rollback_durable_append_authority_availability(durable_audit_policy_availability);
        raw(", \"durable_append_authority_availability\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_REASON);
        raw(", \"availability_hash\": ");
        json_sha256(durable_append_authority_availability.availability_hash);
        raw(", \"source_audit_policy_availability_hash\": ");
        json_sha256(durable_append_authority_availability.source_audit_policy_availability_hash);
        raw(", \"source_policy_ledger_availability_hash\": ");
        json_sha256(durable_append_authority_availability.source_policy_ledger_availability_hash);
        raw(", \"source_write_authority_availability_hash\": ");
        json_sha256(durable_append_authority_availability.source_write_authority_availability_hash);
        raw(", \"source_ledger_aware_acceptance_result_hash\": ");
        json_sha256(
            durable_append_authority_availability.source_ledger_aware_acceptance_result_hash,
        );
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(durable_append_authority_availability.source_ledger_candidate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_append_authority_availability.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(durable_append_authority_availability.source_target_region_write_readback_hash);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_availability.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_availability.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_availability.target_byte_count
        ));
        raw(", \"audit_policy_availability_evidence_verified\": ");
        raw_bool(durable_append_authority_availability.audit_policy_availability_evidence_verified);
        raw(", \"policy_ledger_availability_evidence_verified\": ");
        raw_bool(
            durable_append_authority_availability.policy_ledger_availability_evidence_verified,
        );
        raw(", \"write_authority_evidence_verified\": ");
        raw_bool(durable_append_authority_availability.write_authority_evidence_verified);
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_append_authority_availability.ledger_evidence_verified);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_append_authority_availability.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(durable_append_authority_availability.target_region_write_readback_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(durable_append_authority_availability.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(durable_append_authority_availability.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_append_authority_availability
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_append_authority_availability.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_append_authority_availability.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_append_authority_availability.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_append_authority_availability.durable_append_authority_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let transaction_append_availability_decision =
            hello_rollback_transaction_append_availability_decision(
                durable_append_authority_availability,
                append_engine_readiness_decision,
                durable_writer_policy_preflight,
            );
        raw(", \"transaction_append_availability_decision\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_REASON);
        raw(", \"decision_hash\": ");
        json_sha256(transaction_append_availability_decision.decision_hash);
        raw(", \"source_durable_append_authority_availability_hash\": ");
        json_sha256(
            transaction_append_availability_decision
                .source_durable_append_authority_availability_hash,
        );
        raw(", \"source_audit_policy_availability_hash\": ");
        json_sha256(transaction_append_availability_decision.source_audit_policy_availability_hash);
        raw(", \"source_append_engine_readiness_decision_hash\": ");
        json_sha256(
            transaction_append_availability_decision.source_append_engine_readiness_decision_hash,
        );
        raw(", \"source_writer_policy_preflight_hash\": ");
        json_sha256(transaction_append_availability_decision.source_writer_policy_preflight_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(transaction_append_availability_decision.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            transaction_append_availability_decision.source_target_region_write_readback_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_availability_decision.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_availability_decision.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_availability_decision.target_byte_count
        ));
        raw(", \"durable_append_authority_availability_evidence_verified\": ");
        raw_bool(
            transaction_append_availability_decision
                .durable_append_authority_availability_evidence_verified,
        );
        raw(", \"audit_policy_availability_evidence_verified\": ");
        raw_bool(
            transaction_append_availability_decision.audit_policy_availability_evidence_verified,
        );
        raw(", \"append_engine_ready\": ");
        raw_bool(transaction_append_availability_decision.append_engine_ready);
        raw(", \"writer_policy_ready\": ");
        raw_bool(transaction_append_availability_decision.writer_policy_ready);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(transaction_append_availability_decision.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(transaction_append_availability_decision.target_region_write_readback_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(transaction_append_availability_decision.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(transaction_append_availability_decision.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            transaction_append_availability_decision
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"durable_append_authority_available\": ");
        raw_bool(transaction_append_availability_decision.durable_append_authority_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(transaction_append_availability_decision.durable_audit_policy_available);
        raw(", \"transaction_append_available\": ");
        raw_bool(transaction_append_availability_decision.transaction_append_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let transaction_append_authority_denial_gate =
            hello_rollback_transaction_append_authority_denial_gate(
                transaction_append_availability_decision,
            );
        raw(", \"transaction_append_authority_denial_gate\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_REASON);
        raw(", \"gate_hash\": ");
        json_sha256(transaction_append_authority_denial_gate.gate_hash);
        raw(", \"source_transaction_append_availability_decision_hash\": ");
        json_sha256(
            transaction_append_authority_denial_gate
                .source_transaction_append_availability_decision_hash,
        );
        raw(", \"source_durable_append_authority_availability_hash\": ");
        json_sha256(
            transaction_append_authority_denial_gate
                .source_durable_append_authority_availability_hash,
        );
        raw(", \"source_audit_policy_availability_hash\": ");
        json_sha256(transaction_append_authority_denial_gate.source_audit_policy_availability_hash);
        raw(", \"source_append_engine_readiness_decision_hash\": ");
        json_sha256(
            transaction_append_authority_denial_gate.source_append_engine_readiness_decision_hash,
        );
        raw(", \"source_writer_policy_preflight_hash\": ");
        json_sha256(transaction_append_authority_denial_gate.source_writer_policy_preflight_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(transaction_append_authority_denial_gate.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            transaction_append_authority_denial_gate.source_target_region_write_readback_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_authority_denial_gate.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_authority_denial_gate.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_authority_denial_gate.target_byte_count
        ));
        raw(", \"availability_decision_evidence_verified\": ");
        raw_bool(transaction_append_authority_denial_gate.availability_decision_evidence_verified);
        raw(", \"append_engine_ready\": ");
        raw_bool(transaction_append_authority_denial_gate.append_engine_ready);
        raw(", \"writer_policy_ready\": ");
        raw_bool(transaction_append_authority_denial_gate.writer_policy_ready);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(transaction_append_authority_denial_gate.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(transaction_append_authority_denial_gate.target_region_write_readback_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(transaction_append_authority_denial_gate.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(transaction_append_authority_denial_gate.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            transaction_append_authority_denial_gate
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"durable_append_authority_available\": ");
        raw_bool(transaction_append_authority_denial_gate.durable_append_authority_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(transaction_append_authority_denial_gate.durable_audit_policy_available);
        raw(", \"transaction_append_available\": ");
        raw_bool(transaction_append_authority_denial_gate.transaction_append_available);
        raw(", \"missing_transaction_append_authority\": ");
        raw_bool(transaction_append_authority_denial_gate.missing_transaction_append_authority);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        let durable_policy_ledger_availability_dry_run =
            hello_rollback_durable_policy_ledger_availability_dry_run(
                durable_policy_ledger_availability,
                durable_audit_policy_write_authority_availability,
                transaction_append_authority_denial_gate,
                target_region_write,
            );
        raw(", \"durable_policy_ledger_availability_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_REASON);
        raw(", \"dry_run_hash\": ");
        json_sha256(durable_policy_ledger_availability_dry_run.dry_run_hash);
        raw(", \"source_policy_ledger_availability_hash\": ");
        json_sha256(
            durable_policy_ledger_availability_dry_run.source_policy_ledger_availability_hash,
        );
        raw(", \"source_write_authority_availability_hash\": ");
        json_sha256(
            durable_policy_ledger_availability_dry_run.source_write_authority_availability_hash,
        );
        raw(", \"source_ledger_aware_acceptance_result_hash\": ");
        json_sha256(
            durable_policy_ledger_availability_dry_run.source_ledger_aware_acceptance_result_hash,
        );
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(durable_policy_ledger_availability_dry_run.source_ledger_candidate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_policy_ledger_availability_dry_run.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            durable_policy_ledger_availability_dry_run.source_target_region_write_readback_hash,
        );
        raw(", \"source_authority_denial_gate_hash\": ");
        json_sha256(durable_policy_ledger_availability_dry_run.source_authority_denial_gate_hash);
        raw(", \"source_transaction_append_availability_decision_hash\": ");
        json_sha256(
            durable_policy_ledger_availability_dry_run
                .source_transaction_append_availability_decision_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_ledger_availability_dry_run.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_ledger_availability_dry_run.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_ledger_availability_dry_run.target_byte_count
        ));
        raw(", \"policy_ledger_availability_evidence_verified\": ");
        raw_bool(
            durable_policy_ledger_availability_dry_run.policy_ledger_availability_evidence_verified,
        );
        raw(", \"write_authority_evidence_verified\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.write_authority_evidence_verified);
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.ledger_evidence_verified);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.target_region_write_readback_verified);
        raw(", \"transaction_append_denial_gate_verified\": ");
        raw_bool(
            durable_policy_ledger_availability_dry_run.transaction_append_denial_gate_verified,
        );
        raw(", \"target_span_verified\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_policy_ledger_availability_dry_run
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.durable_append_authority_available);
        raw(", \"transaction_append_available\": ");
        raw_bool(durable_policy_ledger_availability_dry_run.transaction_append_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false, \"applies_rollback\": false, \"installs_rollback_state\": false}");
        let durable_audit_policy_availability_dry_run =
            hello_rollback_durable_audit_policy_availability_dry_run(
                durable_audit_policy_availability,
                durable_policy_ledger_availability_dry_run,
                transaction_append_authority_denial_gate,
                target_region_write,
            );
        raw(", \"durable_audit_policy_availability_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_REASON);
        raw(", \"dry_run_hash\": ");
        json_sha256(durable_audit_policy_availability_dry_run.dry_run_hash);
        raw(", \"source_audit_policy_availability_hash\": ");
        json_sha256(
            durable_audit_policy_availability_dry_run.source_audit_policy_availability_hash,
        );
        raw(", \"source_policy_ledger_availability_dry_run_hash\": ");
        json_sha256(
            durable_audit_policy_availability_dry_run
                .source_policy_ledger_availability_dry_run_hash,
        );
        raw(", \"source_policy_ledger_availability_hash\": ");
        json_sha256(
            durable_audit_policy_availability_dry_run.source_policy_ledger_availability_hash,
        );
        raw(", \"source_write_authority_availability_hash\": ");
        json_sha256(
            durable_audit_policy_availability_dry_run.source_write_authority_availability_hash,
        );
        raw(", \"source_ledger_aware_acceptance_result_hash\": ");
        json_sha256(
            durable_audit_policy_availability_dry_run.source_ledger_aware_acceptance_result_hash,
        );
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(durable_audit_policy_availability_dry_run.source_ledger_candidate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_audit_policy_availability_dry_run.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            durable_audit_policy_availability_dry_run.source_target_region_write_readback_hash,
        );
        raw(", \"source_authority_denial_gate_hash\": ");
        json_sha256(durable_audit_policy_availability_dry_run.source_authority_denial_gate_hash);
        raw(", \"source_transaction_append_availability_decision_hash\": ");
        json_sha256(
            durable_audit_policy_availability_dry_run
                .source_transaction_append_availability_decision_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_availability_dry_run.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_availability_dry_run.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_audit_policy_availability_dry_run.target_byte_count
        ));
        raw(", \"audit_policy_availability_evidence_verified\": ");
        raw_bool(
            durable_audit_policy_availability_dry_run.audit_policy_availability_evidence_verified,
        );
        raw(", \"policy_ledger_dry_run_evidence_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.policy_ledger_dry_run_evidence_verified);
        raw(", \"policy_ledger_availability_evidence_verified\": ");
        raw_bool(
            durable_audit_policy_availability_dry_run.policy_ledger_availability_evidence_verified,
        );
        raw(", \"write_authority_evidence_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.write_authority_evidence_verified);
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.ledger_evidence_verified);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.target_region_write_readback_verified);
        raw(", \"transaction_append_denial_gate_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.transaction_append_denial_gate_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(durable_audit_policy_availability_dry_run.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_audit_policy_availability_dry_run
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_audit_policy_availability_dry_run.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_audit_policy_availability_dry_run.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_audit_policy_availability_dry_run.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_audit_policy_availability_dry_run.durable_append_authority_available);
        raw(", \"transaction_append_available\": ");
        raw_bool(durable_audit_policy_availability_dry_run.transaction_append_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false, \"applies_rollback\": false, \"installs_rollback_state\": false}");
        let durable_append_authority_availability_dry_run =
            hello_rollback_durable_append_authority_availability_dry_run(
                durable_append_authority_availability,
                durable_audit_policy_availability_dry_run,
                transaction_append_authority_denial_gate,
                target_region_write,
            );
        raw(", \"durable_append_authority_availability_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_REASON);
        raw(", \"dry_run_hash\": ");
        json_sha256(durable_append_authority_availability_dry_run.dry_run_hash);
        raw(", \"source_append_authority_availability_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run.source_append_authority_availability_hash,
        );
        raw(", \"source_audit_policy_availability_dry_run_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run
                .source_audit_policy_availability_dry_run_hash,
        );
        raw(", \"source_audit_policy_availability_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run.source_audit_policy_availability_hash,
        );
        raw(", \"source_policy_ledger_availability_dry_run_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run
                .source_policy_ledger_availability_dry_run_hash,
        );
        raw(", \"source_policy_ledger_availability_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run.source_policy_ledger_availability_hash,
        );
        raw(", \"source_write_authority_availability_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run.source_write_authority_availability_hash,
        );
        raw(", \"source_ledger_aware_acceptance_result_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run
                .source_ledger_aware_acceptance_result_hash,
        );
        raw(", \"source_ledger_candidate_hash\": ");
        json_sha256(durable_append_authority_availability_dry_run.source_ledger_candidate_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(durable_append_authority_availability_dry_run.source_policy_preflight_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run.source_target_region_write_readback_hash,
        );
        raw(", \"source_authority_denial_gate_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run.source_authority_denial_gate_hash,
        );
        raw(", \"source_transaction_append_availability_decision_hash\": ");
        json_sha256(
            durable_append_authority_availability_dry_run
                .source_transaction_append_availability_decision_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_availability_dry_run.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_availability_dry_run.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_append_authority_availability_dry_run.target_byte_count
        ));
        raw(", \"append_authority_availability_evidence_verified\": ");
        raw_bool(
            durable_append_authority_availability_dry_run
                .append_authority_availability_evidence_verified,
        );
        raw(", \"audit_policy_dry_run_evidence_verified\": ");
        raw_bool(
            durable_append_authority_availability_dry_run.audit_policy_dry_run_evidence_verified,
        );
        raw(", \"audit_policy_availability_evidence_verified\": ");
        raw_bool(
            durable_append_authority_availability_dry_run
                .audit_policy_availability_evidence_verified,
        );
        raw(", \"policy_ledger_dry_run_evidence_verified\": ");
        raw_bool(
            durable_append_authority_availability_dry_run.policy_ledger_dry_run_evidence_verified,
        );
        raw(", \"policy_ledger_availability_evidence_verified\": ");
        raw_bool(
            durable_append_authority_availability_dry_run
                .policy_ledger_availability_evidence_verified,
        );
        raw(", \"write_authority_evidence_verified\": ");
        raw_bool(durable_append_authority_availability_dry_run.write_authority_evidence_verified);
        raw(", \"ledger_evidence_verified\": ");
        raw_bool(durable_append_authority_availability_dry_run.ledger_evidence_verified);
        raw(", \"media_write_policy_verified\": ");
        raw_bool(durable_append_authority_availability_dry_run.media_write_policy_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(
            durable_append_authority_availability_dry_run.target_region_write_readback_verified,
        );
        raw(", \"transaction_append_denial_gate_verified\": ");
        raw_bool(
            durable_append_authority_availability_dry_run.transaction_append_denial_gate_verified,
        );
        raw(", \"target_span_verified\": ");
        raw_bool(durable_append_authority_availability_dry_run.target_span_verified);
        raw(", \"audit_rollback_target_ids_verified\": ");
        raw_bool(durable_append_authority_availability_dry_run.audit_rollback_target_ids_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_append_authority_availability_dry_run
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_append_authority_availability_dry_run.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_append_authority_availability_dry_run.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_append_authority_availability_dry_run.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_append_authority_availability_dry_run.durable_append_authority_available);
        raw(", \"transaction_append_available\": ");
        raw_bool(durable_append_authority_availability_dry_run.transaction_append_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false, \"applies_rollback\": false, \"installs_rollback_state\": false}");
        let transaction_append_dry_run = hello_rollback_transaction_append_dry_run(
            transaction_append_authority_denial_gate,
            append_record,
            sector_plan,
            target_region_write,
        );
        raw(", \"transaction_append_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_REASON);
        raw(", \"dry_run_hash\": ");
        json_sha256(transaction_append_dry_run.dry_run_hash);
        raw(", \"source_authority_denial_gate_hash\": ");
        json_sha256(transaction_append_dry_run.source_authority_denial_gate_hash);
        raw(", \"source_transaction_append_availability_decision_hash\": ");
        json_sha256(
            transaction_append_dry_run.source_transaction_append_availability_decision_hash,
        );
        raw(", \"source_append_record_hash\": ");
        json_sha256(transaction_append_dry_run.source_append_record_hash);
        raw(", \"source_sector_plan_hash\": ");
        json_sha256(transaction_append_dry_run.source_sector_plan_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(transaction_append_dry_run.source_target_region_write_readback_hash);
        raw(", \"planned_sector_image_hash\": ");
        json_sha256(transaction_append_dry_run.planned_sector_image_hash);
        raw(", \"readback_sector_image_hash\": ");
        json_sha256(transaction_append_dry_run.readback_sector_image_hash);
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_dry_run.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_dry_run.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            transaction_append_dry_run.target_byte_count
        ));
        raw(", \"authority_denial_gate_verified\": ");
        raw_bool(transaction_append_dry_run.authority_denial_gate_verified);
        raw(", \"target_span_verified\": ");
        raw_bool(transaction_append_dry_run.target_span_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(transaction_append_dry_run.target_region_write_readback_verified);
        raw(", \"append_image_ready\": ");
        raw_bool(transaction_append_dry_run.append_image_ready);
        raw(", \"blocked_by_authority_denial_gate\": ");
        raw_bool(transaction_append_dry_run.blocked_by_authority_denial_gate);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(transaction_append_dry_run.test_infrastructure_media_write_authority_available);
        raw(", \"transaction_append_available\": ");
        raw_bool(transaction_append_dry_run.transaction_append_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"transaction_append_attempted\": false}");
        let target_region_sector_inspection =
            hello_rollback_target_region_sector_inspection_from_retained_inspect(
                append_record,
                sector_plan,
                target_region_write,
            );
        raw(", \"target_region_sector_inspection\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(target_region_sector_inspection.status);
        raw(", \"reason\": ");
        json_str(target_region_sector_inspection.reason);
        raw(", \"inspection_hash\": ");
        json_sha256(target_region_sector_inspection.inspection_hash);
        raw(", \"source_sector_plan_hash\": ");
        json_sha256(target_region_sector_inspection.source_sector_plan_hash);
        raw(", \"source_target_region_write_readback_hash\": ");
        json_sha256(target_region_sector_inspection.source_target_region_write_readback_hash);
        raw(", \"expected_sector_image_hash\": ");
        json_sha256(target_region_sector_inspection.expected_sector_image_hash);
        raw(", \"sector_image_hash\": ");
        json_sha256(target_region_sector_inspection.sector_image_hash);
        raw(", \"audit_record_image_hash\": ");
        json_sha256(target_region_sector_inspection.audit_record_image_hash);
        raw(", \"rollback_transaction_image_hash\": ");
        json_sha256(target_region_sector_inspection.rollback_transaction_image_hash);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.target_byte_count
        ));
        raw(", \"audit_record_offset\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.audit_record_offset
        ));
        raw(", \"audit_record_byte_length\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.audit_record_byte_length
        ));
        raw(", \"rollback_transaction_offset\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.rollback_transaction_offset
        ));
        raw(", \"rollback_transaction_byte_length\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.rollback_transaction_byte_length
        ));
        raw(", \"padding_offset\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.padding_offset
        ));
        raw(", \"padding_byte_length\": ");
        raw_fmt(format_args!(
            "{}",
            target_region_sector_inspection.padding_byte_length
        ));
        raw(", \"label_found\": ");
        raw_bool(target_region_sector_inspection.label_found);
        raw(", \"read_attempted\": ");
        raw_bool(target_region_sector_inspection.read_attempted);
        raw(", \"read_completed\": ");
        raw_bool(target_region_sector_inspection.read_completed);
        raw(", \"sector_hash_verified\": ");
        raw_bool(target_region_sector_inspection.sector_hash_verified);
        raw(", \"audit_record_hash_verified\": ");
        raw_bool(target_region_sector_inspection.audit_record_hash_verified);
        raw(", \"rollback_transaction_hash_verified\": ");
        raw_bool(target_region_sector_inspection.rollback_transaction_hash_verified);
        raw(", \"offsets_verified\": ");
        raw_bool(target_region_sector_inspection.offsets_verified);
        raw(", \"padding_zeroed\": ");
        raw_bool(target_region_sector_inspection.padding_zeroed);
        raw(", \"target_span_verified\": ");
        raw_bool(target_region_sector_inspection.target_span_verified);
        raw(", \"target_region_write_readback_verified\": ");
        raw_bool(target_region_sector_inspection.target_region_write_readback_verified);
        raw(", \"inspection_verified\": ");
        raw_bool(target_region_sector_inspection.inspection_verified);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_state\": false}");
        raw(", \"retained_recovery_rollback_inspect_source\": ");
        emit_recovery_rollback_inspect_source_reference_inline(target_region_sector_inspection);
        let durable_policy_write_authority_decision =
            hello_rollback_durable_policy_write_authority_decision(
                durable_append_authority_availability_dry_run,
                durable_audit_policy_write_authority_availability,
                durable_audit_policy_availability,
                durable_append_authority_availability,
                transaction_append_dry_run,
                target_region_sector_inspection,
            );
        raw(", \"durable_policy_write_authority_decision\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_STATUS);
        raw(", \"reason\": ");
        json_str(HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_REASON);
        raw(", \"decision_hash\": ");
        json_sha256(durable_policy_write_authority_decision.decision_hash);
        raw(", \"source_durable_append_authority_availability_dry_run_hash\": ");
        json_sha256(
            durable_policy_write_authority_decision
                .source_durable_append_authority_availability_dry_run_hash,
        );
        raw(", \"source_transaction_append_dry_run_hash\": ");
        json_sha256(durable_policy_write_authority_decision.source_transaction_append_dry_run_hash);
        raw(", \"source_target_region_sector_inspection_hash\": ");
        json_sha256(
            durable_policy_write_authority_decision.source_target_region_sector_inspection_hash,
        );
        raw(", \"source_write_authority_availability_hash\": ");
        json_sha256(
            durable_policy_write_authority_decision.source_write_authority_availability_hash,
        );
        raw(", \"source_audit_policy_availability_hash\": ");
        json_sha256(durable_policy_write_authority_decision.source_audit_policy_availability_hash);
        raw(", \"source_durable_append_authority_availability_hash\": ");
        json_sha256(
            durable_policy_write_authority_decision
                .source_durable_append_authority_availability_hash,
        );
        raw(", \"source_authority_denial_gate_hash\": ");
        json_sha256(durable_policy_write_authority_decision.source_authority_denial_gate_hash);
        raw(", \"source_transaction_append_availability_decision_hash\": ");
        json_sha256(
            durable_policy_write_authority_decision
                .source_transaction_append_availability_decision_hash,
        );
        raw(", \"audit_ledger_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"audit_record_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
        raw(", \"rollback_store_target_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"rollback_transaction_schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_write_authority_decision.target_start_lba
        ));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_write_authority_decision.target_lba_count
        ));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!(
            "{}",
            durable_policy_write_authority_decision.target_byte_count
        ));
        raw(", \"transaction_append_dry_run_verified\": ");
        raw_bool(durable_policy_write_authority_decision.transaction_append_dry_run_verified);
        raw(", \"target_region_sector_inspection_verified\": ");
        raw_bool(durable_policy_write_authority_decision.target_region_sector_inspection_verified);
        raw(", \"write_authority_evidence_verified\": ");
        raw_bool(durable_policy_write_authority_decision.write_authority_evidence_verified);
        raw(", \"audit_policy_availability_evidence_verified\": ");
        raw_bool(
            durable_policy_write_authority_decision.audit_policy_availability_evidence_verified,
        );
        raw(", \"durable_append_authority_availability_evidence_verified\": ");
        raw_bool(
            durable_policy_write_authority_decision
                .durable_append_authority_availability_evidence_verified,
        );
        raw(", \"target_span_verified\": ");
        raw_bool(durable_policy_write_authority_decision.target_span_verified);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(
            durable_policy_write_authority_decision
                .test_infrastructure_media_write_authority_available,
        );
        raw(", \"write_authority_available\": ");
        raw_bool(durable_policy_write_authority_decision.write_authority_available);
        raw(", \"durable_policy_ledger_available\": ");
        raw_bool(durable_policy_write_authority_decision.durable_policy_ledger_available);
        raw(", \"durable_audit_policy_available\": ");
        raw_bool(durable_policy_write_authority_decision.durable_audit_policy_available);
        raw(", \"durable_append_authority_available\": ");
        raw_bool(durable_policy_write_authority_decision.durable_append_authority_available);
        raw(", \"transaction_append_available\": ");
        raw_bool(durable_policy_write_authority_decision.transaction_append_available);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"authorizes_transaction_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false, \"applies_rollback\": false}");
        raw(", \"target_region_write_readback_dry_run\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(target_region_write.status);
        raw(", \"reason\": ");
        json_str(target_region_write.reason);
        raw(", \"dry_run_hash\": ");
        json_sha256(target_region_write.dry_run_hash);
        raw(", \"source_sector_plan_hash\": ");
        json_sha256(target_region_write.source_sector_plan_hash);
        raw(", \"source_policy_preflight_hash\": ");
        json_sha256(target_region_write.source_policy_preflight_hash);
        raw(", \"planned_sector_image_hash\": ");
        json_sha256(target_region_write.planned_sector_image_hash);
        raw(", \"readback_sector_image_hash\": ");
        json_sha256(target_region_write.readback_sector_image_hash);
        raw(", \"target_region_id\": ");
        json_str(ahci::AUDIT_ROLLBACK_TARGET_REGION_ID);
        raw(", \"target_start_lba\": ");
        raw_fmt(format_args!("{}", target_region_write.target_start_lba));
        raw(", \"target_lba_count\": ");
        raw_fmt(format_args!("{}", target_region_write.target_lba_count));
        raw(", \"target_byte_count\": ");
        raw_fmt(format_args!("{}", target_region_write.target_byte_count));
        raw(", \"label_found\": ");
        raw_bool(target_region_write.label_found);
        raw(", \"target_range_ready\": ");
        raw_bool(target_region_write.target_range_ready);
        raw(", \"test_infrastructure_media_write_authority_available\": ");
        raw_bool(target_region_write.test_infrastructure_media_write_authority_available);
        raw(", \"write_attempted\": ");
        raw_bool(target_region_write.write_attempted);
        raw(", \"write_completed\": ");
        raw_bool(target_region_write.write_completed);
        raw(", \"readback_completed\": ");
        raw_bool(target_region_write.readback_completed);
        raw(", \"readback_matches_planned_image\": ");
        raw_bool(target_region_write.readback_matches_planned_image);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_state\": false}");
        raw(", \"storage_authority_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
        raw(", \"append_target_owner_id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
        raw(", \"transaction_writer_readiness_id\": ");
        json_str(rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID);
        raw(", \"audit_ledger_writer_fact_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
        raw(", \"rollback_store_writer_fact_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
        raw(", \"scratch_write_readback_verified\": ");
        raw_bool(durable_append_preflight.scratch_write_readback_verified);
        raw(", \"scratch_used_as_durable_authority\": false, \"durable_audit_writer_available\": ");
        raw_bool(durable_append_preflight.durable_audit_writer_available);
        raw(", \"rollback_store_writer_available\": ");
        raw_bool(durable_append_preflight.rollback_store_writer_available);
        raw(", \"transaction_append_writer_available\": ");
        raw_bool(durable_append_preflight.transaction_append_writer_available);
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false}");
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false}");
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        raw(", \"dry_run_evaluated\": true, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"write_attempted\": false}");
        raw(", \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false}");
        raw(", \"block_write_path_authority_gate\": {\"schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_ID);
        raw(", \"storage_authority_id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
        raw(", \"status\": ");
        json_str(foundation.block_write_path_gate_status);
        raw(", \"reason\": ");
        json_str(foundation.block_write_path_reason);
        raw(", \"available\": ");
        raw_bool(foundation.block_write_path_available);
        raw(", \"read_only_block_driver_id\": ");
        json_str(foundation.read_only_block_driver_id);
        raw(", \"read_only_block_driver_available\": ");
        raw_bool(foundation.read_only_block_driver_available);
        raw(", \"partition_inventory_available\": ");
        raw_bool(foundation.partition_inventory_available);
        raw(", \"partition_inventory_scheme\": ");
        json_str(foundation.partition_inventory_scheme);
        raw(", \"scratch_block_write_authority_available\": ");
        raw_bool(foundation.scratch_block_write_authority_available);
        raw(", \"scratch_block_write_authority_id\": ");
        json_str(foundation.scratch_block_write_authority_id);
        raw(", \"scratch_region_within_device_bounds\": ");
        raw_bool(foundation.scratch_region_within_device_bounds);
        raw(", \"scratch_region_no_boot_or_partition_metadata_overlap\": ");
        raw_bool(foundation.scratch_region_no_boot_or_partition_metadata_overlap);
        raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_enabled\": false, \"write_attempted\": false}");
        raw(", \"append_target\": {\"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID);
        raw(", \"schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
        raw(", \"available\": ");
        raw_bool(foundation.rollback_transaction_append_available);
        raw("}, \"storage_layout\": {\"schema\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_LAYOUT_SCHEMA);
        raw(", \"id\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_LAYOUT_ID);
        raw(", \"status\": ");
        json_str(foundation.storage_layout_status);
        raw(", \"reason\": ");
        json_str(foundation.storage_layout_reason);
        raw(", \"available\": ");
        raw_bool(foundation.storage_layout_available);
        raw("}, \"append_engine\": {\"schema\": \"raios.rollback_store_transaction_engine.v0\", \"id\": \"append_engine.rollback_store.current_boot\", \"status\": ");
        json_str(foundation.append_engine_status);
        raw(", \"reason\": ");
        json_str(foundation.append_engine_reason);
        raw(", \"available\": ");
        raw_bool(foundation.append_engine_available);
        raw("}, \"append_contract\": {\"schema\": \"raios.rollback_store_transaction_envelope.v0\", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID);
        raw(", \"status\": ");
        json_str(foundation.rollback_transaction_envelope_status);
        raw(", \"reason\": ");
        json_str(foundation.rollback_transaction_envelope_reason);
        raw(", \"available\": ");
        raw_bool(foundation.rollback_transaction_envelope_available);
        raw("}, \"transaction_writer\": {\"owner\": ");
        json_str(rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
        raw(", \"status\": ");
        json_str(foundation.append_contract_status);
        raw(", \"reason\": ");
        json_str(foundation.append_contract_reason);
        raw(", \"available\": ");
        raw_bool(foundation.transaction_writer_available);
        raw("}}");
        raw(", \"unavailable_authorities\": {\"transaction_writer\": ");
        raw_bool(!foundation.transaction_writer_available);
        raw(", \"durable_audit_store\": ");
        raw_bool(!foundation.durable_audit_store_available);
        raw(", \"rollback_store\": ");
        raw_bool(!foundation.rollback_store_available);
        raw(", \"rollback_transaction_append\": ");
        raw_bool(!foundation.rollback_transaction_append_available);
        raw("}");
        raw(", \"side_effects\": {\"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_plan\": false, \"applies_rollback\": false}");
        raw("}");
    } else {
        raw("null");
    }
}

pub(crate) fn recovery_rollback_inspection_evidence(
    snapshot: Snapshot,
) -> Option<(
    RollbackTargetRegionWriteReadbackDryRun,
    RollbackTargetRegionSectorInspection,
)> {
    let probation = snapshot.hot_swap_probation?;
    let foundation = hello_rollback_writer_storage_foundation();
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

pub(crate) fn recovery_rollback_materialization_evidence(
    snapshot: Snapshot,
) -> Option<(
    RollbackAppendRecordDryRun,
    RollbackAppendSectorPlanDryRun,
    RollbackTargetRegionWriteReadbackDryRun,
)> {
    let probation = snapshot.hot_swap_probation?;
    let foundation = hello_rollback_writer_storage_foundation();
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

pub(crate) fn emit_target_region_write_readback_inline(
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) {
    emit_inline_value(target_region_write_readback_value(target_region_write));
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

pub(crate) fn emit_target_region_sector_inspection_inline(
    inspection: RollbackTargetRegionSectorInspection,
) {
    emit_inline_value(target_region_sector_inspection_value(inspection));
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
            j!("requested_capability" => s("cap.recovery.rollback_materialize_dry_run.current_boot"),
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
    let evidence = recovery_rollback_inspection_evidence(snapshot);
    let inspection_available = evidence
        .map(|(_, inspection)| inspection.inspection_verified)
        .unwrap_or(false);
    let materialized_sector_evidence_available = evidence
        .map(|(target_region_write, _)| {
            materialized_target_region_sector_available(target_region_write)
        })
        .unwrap_or(false);
    let status = if inspection_available {
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
    let reason = if inspection_available {
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
    if let Some((_, inspection)) = evidence {
        retain_recovery_rollback_inspect_source_reference(event_id, inspection);
    }

    begin_response(method);
    emit_record_fields(
        vec![
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
            j!("requested_capability" => s("cap.recovery.rollback_inspect.read"),
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
        ],
        6,
    );
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
    raw("    \"rollback_transaction_writer_storage_authority_gate\": ");
    emit_rollback_transaction_writer_storage_authority_gate(snapshot, probation);
    raw_line(",");
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
    action: &'static str,
    snapshot: Snapshot,
    descriptor: LoadDescriptor,
) {
    let activation_status = service_slot_activation_status(snapshot);
    let activation_active = service_slot_activation_active(snapshot);
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => s("raios.ram_only_hello_service.v0")),
            j!("scope" => s("current_boot")),
            j!("classification" => s("local_only")),
            j!("persistence" => s("none")),
            j!("action" => s(action)),
            j!("event_id" => event_opt(snapshot.last_event_id)),
            j!("audit_event_id" => event_opt(snapshot.last_event_id)),
            j!("load_request" => load_request_value(descriptor)),
            j!("load_descriptor" => load_descriptor_value(descriptor)),
            j!("artifact_load_plan_preflight" => artifact_load_plan_preflight_value(descriptor),
            ),
            j!("service_slot_activation" => service_slot_activation_value(descriptor, activation_status, activation_active),
            ),
            j!("state" => hello_state_value(snapshot)),
            j!("state_migration" => hello_state_migration_value(snapshot.state_migration),
            ),
            j!("hot_swap_probation" => hello_hot_swap_probation_value(snapshot.hot_swap_probation),
            ),
            j!("service" => hello_response_service_value(
                    snapshot,
                    descriptor,
                    activation_status,
                    activation_active,
                ),
            ),
            j!("lifecycle" => hello_response_lifecycle_value(snapshot)),
            j!("loader" => hello_response_loader_value(descriptor, activation_status, activation_active),
            ),
            j!("denied_surfaces" => hello_response_denied_surfaces_value()),
        ],
        6,
    );
    end_response(method);
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

fn hello_response_lifecycle_value(snapshot: Snapshot) -> V<'static> {
    object(vec![
        j!("last_action" => s(snapshot.last_action)),
        j!("reason" => s(snapshot.last_reason)),
        j!("service_inventory_change" => s(snapshot.last_inventory_change),
        ),
        j!("load_event_id" => event_opt(snapshot.load_event_id)),
        j!("start_event_id" => event_opt(snapshot.start_event_id)),
        j!("hot_swap_event_id" => event_opt(snapshot.hot_swap_event_id)),
        j!("stop_event_id" => event_opt(snapshot.stop_event_id)),
        j!("drop_event_id" => event_opt(snapshot.drop_event_id)),
    ])
}

fn hello_response_loader_value(
    descriptor: LoadDescriptor,
    activation_status: &'static str,
    activation_active: bool,
) -> V<'static> {
    object(vec![
        j!("kind" => s(descriptor.artifact_kind)),
        j!("descriptor_id" => s(descriptor.id)),
        j!("descriptor_source_locator" => s(descriptor.source_locator)),
        j!("descriptor_source_kind" => s(descriptor.source_kind)),
        j!("descriptor_source_validated" => b(true)),
        j!("descriptor_source_hash" => sha(descriptor_source_hash(descriptor)),
        ),
        j!("descriptor_source_signature_envelope" => descriptor_source_signature_envelope_value(descriptor),
        ),
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
        j!("service_slot_intent_id" => s(SERVICE_SLOT_INTENT_ID)),
        j!("ram_only_service_slot_id" => s(RAM_ONLY_SERVICE_SLOT_ID)),
        j!("binds_source_locator" => s_opt(descriptor.binds_source_locator),
        ),
        j!("binds_source_kind" => s_opt(descriptor.binds_source_kind)),
        j!("binds_source_hash" => sha_opt(descriptor.binds_source_hash)),
        j!("accepts_external_artifact_bytes" => b(false)),
        j!("loads_external_artifact" => b(false)),
        j!("maps_executable_pages" => b(false)),
        j!("writes_persistent_state" => b(false)),
        j!("writes_durable_audit_log" => b(false)),
        j!("installs_rollback_plan" => b(false)),
        j!("grants_broad_mutation" => b(false)),
    ])
}

fn hello_response_denied_surfaces_value() -> V<'static> {
    object(vec![
        j!("general_module_load" => s("unchanged_denied")),
        j!("external_artifact_load" => s("denied")),
        j!("persistent_install" => s("denied")),
        j!("durable_audit" => s("denied")),
        j!("rollback_install" => s("denied")),
        j!("broad_mutation" => s("denied")),
    ])
}

pub(crate) fn emit_load_request(descriptor: LoadDescriptor) {
    raw("      \"load_request\": ");
    emit_record_value_fragment(load_request_value(descriptor), 6);
}

fn load_request_value(descriptor: LoadDescriptor) -> V<'static> {
    object(vec![
        j!("schema" => s("raios.current_boot_load_request.v0")),
        j!("scope" => s("current_boot")),
        j!("classification" => s("local_only")),
        j!("descriptor_schema" => s(descriptor.schema)),
        j!("descriptor_id" => s(descriptor.id)),
        j!("descriptor_source_locator" => s(descriptor.source_locator)),
        j!("descriptor_source_kind" => s(descriptor.source_kind)),
        j!("descriptor_source_validated" => b(true)),
        j!("descriptor_source_hash" => sha(descriptor_source_hash(descriptor)),
        ),
        j!("descriptor_source_signature_envelope" => descriptor_source_signature_envelope_value(descriptor),
        ),
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
        j!("service_slot_intent_id" => s(SERVICE_SLOT_INTENT_ID)),
        j!("ram_only_service_slot_id" => s(RAM_ONLY_SERVICE_SLOT_ID)),
        j!("binds_source_locator" => s_opt(descriptor.binds_source_locator),
        ),
        j!("binds_source_kind" => s_opt(descriptor.binds_source_kind)),
        j!("binds_source_hash" => sha_opt(descriptor.binds_source_hash)),
        j!("service_id" => s(descriptor.service_id)),
        j!("accepted" => b(true)),
    ])
}

pub(crate) fn emit_load_descriptor(descriptor: LoadDescriptor) {
    raw("      \"load_descriptor\": ");
    emit_record_value_fragment(load_descriptor_value(descriptor), 6);
}

fn load_descriptor_value(descriptor: LoadDescriptor) -> V<'static> {
    object(vec![
        j!("schema" => s(descriptor.schema)),
        j!("id" => s(descriptor.id)),
        j!("source" => object(vec![
                j!("canonicalization" => s(descriptor.canonicalization)),
                j!("locator" => s(descriptor.source_locator)),
                j!("kind" => s(descriptor.source_kind)),
                j!("validated" => b(true)),
                j!("sha256" => sha(descriptor_source_hash(descriptor))),
                j!("binds_source_locator" => s_opt(descriptor.binds_source_locator),
                ),
                j!("binds_source_kind" => s_opt(descriptor.binds_source_kind)),
                j!("binds_source_hash" => sha_opt(descriptor.binds_source_hash)),
                j!("signature_envelope" => descriptor_source_signature_envelope_value(descriptor),
                ),
                j!("text" => s(descriptor.source_text)),
            ]),
        ),
        j!("service_id" => s(descriptor.service_id)),
        j!("artifact_id" => s(descriptor.artifact_id)),
        j!("artifact_kind" => s(descriptor.artifact_kind)),
        j!("artifact_identity" => artifact_identity_value(descriptor)),
        j!("artifact_load_plan_preflight" => artifact_load_plan_preflight_value(descriptor),
        ),
        j!("scope" => s(descriptor.scope)),
        j!("classification" => s(descriptor.classification)),
        j!("persistence" => s(descriptor.persistence)),
        j!("accepts_external_artifact_bytes" => b(false)),
        j!("loads_external_artifact" => b(false)),
        j!("maps_executable_pages" => b(false)),
        j!("writes_persistent_state" => b(false)),
    ])
}

pub(crate) fn emit_artifact_load_plan_preflight(descriptor: LoadDescriptor) {
    emit_inline_value(artifact_load_plan_preflight_value(descriptor));
}

fn artifact_load_plan_preflight_value(descriptor: LoadDescriptor) -> V<'static> {
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

pub(crate) fn emit_service_slot_activation(
    descriptor: LoadDescriptor,
    status: &'static str,
    active: bool,
) {
    emit_inline_value(service_slot_activation_value(descriptor, status, active));
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

pub(crate) fn emit_hello_state_migration_option(record: Option<HelloStateMigrationRecord>) {
    emit_inline_value(hello_state_migration_value(record));
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

fn descriptor_source_signature_envelope_value(descriptor: LoadDescriptor) -> V<'static> {
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

pub(crate) fn emit_artifact_identity(descriptor: LoadDescriptor) {
    emit_inline_value(artifact_identity_value(descriptor));
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

pub(crate) fn emit_artifact_content_binding(descriptor: LoadDescriptor) {
    emit_inline_value(artifact_content_binding_value(descriptor));
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

fn artifact_reference_value(descriptor: LoadDescriptor) -> V<'static> {
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

fn artifact_identity_signature_envelope_value(descriptor: LoadDescriptor) -> V<'static> {
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
