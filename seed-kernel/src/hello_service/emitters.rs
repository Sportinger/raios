use super::*;

pub(crate) fn emit_health_response(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let descriptor = snapshot.load_descriptor;
    let activation_status = service_slot_activation_status(snapshot);
    let activation_active = service_slot_activation_active(snapshot);
    begin_response(method);
    raw_line("      \"schema\": \"raios.ram_only_hello_service.health.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"action\": \"health_probe\",");
    raw("      \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"service_slot_activation\": ");
    emit_service_slot_activation(descriptor, activation_status, activation_active);
    raw_line(",");
    raw("      \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("      \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw("      \"hot_swap_probation\": ");
    emit_hello_hot_swap_probation_option(snapshot.hot_swap_probation);
    raw_line(",");
    raw_line("      \"service\": {");
    raw("        \"id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw_line("        \"kind\": \"service\",");
    raw("        \"version\": ");
    json_str(service_version(descriptor));
    raw_line(",");
    raw("        \"loaded\": ");
    raw_bool(snapshot.loaded);
    raw_line(",");
    raw("        \"running\": ");
    raw_bool(snapshot.running);
    raw_line(",");
    raw("        \"health\": ");
    json_str(health_state(snapshot));
    raw_line(",");
    raw("        \"generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("        \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("        \"last_action\": ");
    json_str(snapshot.last_action);
    raw_line(",");
    raw("        \"last_reason\": ");
    json_str(snapshot.last_reason);
    raw_line(",");
    raw("        \"service_slot_activation_id\": ");
    json_str(SERVICE_SLOT_ACTIVATION_ID);
    raw_line(",");
    raw("        \"service_slot_activation_hash\": ");
    json_sha256(service_slot_activation_hash(descriptor));
    raw_line(",");
    raw("        \"service_slot_activation_status\": ");
    json_str(activation_status);
    raw_line(",");
    raw("        \"service_slot_activation_active\": ");
    raw_bool(activation_active);
    raw_line(",");
    raw("        \"capabilities\": [");
    emit_inline_string_array(CAPABILITIES);
    raw_line("]");
    raw_line("      },");
    raw_line("      \"load_descriptor\": {");
    raw("        \"schema\": ");
    json_str(descriptor.schema);
    raw_line(",");
    raw("        \"id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw_line("        \"source\": {");
    raw("          \"locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("          \"kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("          \"validated\": true,");
    raw("          \"sha256\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("          \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("          \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("          \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line(",");
    raw("          \"signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line("");
    raw_line("        },");
    raw("        \"artifact_identity\": ");
    emit_artifact_identity(descriptor);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(descriptor);
    raw_line(",");
    raw("        \"service_slot_activation\": ");
    emit_service_slot_activation(descriptor, activation_status, activation_active);
    raw_line("");
    raw_line("      },");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
}

pub(crate) fn emit_hot_swap_state_migration_denied(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
    migration: HelloStateMigrationRecord,
) {
    raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw("    \"reason\": ");
    json_str("state_migration_would_reset_state");
    raw_line(",");
    raw("    \"message\": ");
    json_str("hello hot-swap denied because the candidate would reset RAM-only service state before a rollback-capable migrator exists");
    raw_line(",");
    raw("    \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("    \"target\": ");
    json_str("svc.demo.hello.reset_state");
    raw_line(",");
    raw("    \"active_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("    \"active_descriptor_id\": ");
    json_str(snapshot.load_descriptor.id);
    raw_line(",");
    raw("    \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("    \"state_migration\": ");
    emit_hello_state_migration_option(Some(migration));
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"state_preserving_migrator\",");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"rollback_plan\"");
    raw_line("    ],");
    raw_line("    \"denied_surfaces\": {");
    raw_line("      \"descriptor_mutation\": \"not_attempted\",");
    raw_line("      \"state_reset\": \"denied\",");
    raw_line("      \"external_artifact_load\": \"denied\",");
    raw_line("      \"candidate_artifact_execution\": \"denied\",");
    raw_line("      \"executable_mapping\": \"denied\",");
    raw_line("      \"persistent_install\": \"denied\",");
    raw_line("      \"durable_audit\": \"denied\",");
    raw_line("      \"rollback_install\": \"denied\",");
    raw_line("      \"broad_mutation\": \"denied\"");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

pub(crate) fn emit_rollback_transaction_preflight(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    if let Some(probation) = probation {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS);
        raw(", \"service_id\": ");
        json_str(SERVICE_ID);
        raw(", \"requested_capability\": ");
        json_str(HELLO_ROLLBACK_APPLY_CAPABILITY);
        raw(", \"preflight_hash\": ");
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
        raw(", \"rollback_target_descriptor_id\": ");
        json_str(probation.previous_descriptor_id);
        raw(", \"rollback_target_descriptor_source_hash\": ");
        json_sha256(probation.previous_descriptor_source_hash);
        raw(", \"rollback_target_artifact_identity_id\": ");
        json_str(probation.previous_artifact_identity_id);
        raw(", \"rollback_target_artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"rollback_target_generation\": ");
        raw_fmt(format_args!("{}", probation.previous_generation));
        raw(", \"rollback_target_state_hash\": ");
        json_sha256(probation.previous_state_hash);
        raw(", \"rollback_target_state_counter\": ");
        raw_fmt(format_args!("{}", probation.previous_state_counter));
        raw(", \"current_candidate_descriptor_id\": ");
        json_str(probation.new_descriptor_id);
        raw(", \"current_candidate_descriptor_source_hash\": ");
        json_sha256(probation.new_descriptor_source_hash);
        raw(", \"current_candidate_artifact_identity_id\": ");
        json_str(probation.new_artifact_identity_id);
        raw(", \"current_candidate_artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"current_candidate_generation\": ");
        raw_fmt(format_args!("{}", probation.new_generation));
        raw(", \"current_candidate_state_hash\": ");
        json_sha256(probation.new_state_hash);
        raw(", \"current_candidate_state_counter\": ");
        raw_fmt(format_args!("{}", probation.new_state_counter));
        raw(", \"state_migration_hash\": ");
        json_sha256(probation.state_migration_hash);
        raw(", \"missing_authorities\": {\"rollback_apply_authority\": true, \"rollback_transaction_authority\": true, \"durable_audit_write_authority\": true, \"persistent_install_authority\": true}");
        raw(", \"side_effects\": {\"mutates_service_state\": false, \"applies_rollback\": false, \"writes_persistent_state\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"installs_rollback_plan\": false, \"accepts_external_artifact_bytes\": false, \"loads_candidate_bytes\": false, \"maps_executable_pages\": false, \"provider_auto_load\": false, \"grants_broad_mutation\": false}");
        raw("}");
    } else {
        raw("null");
    }
}

pub(crate) fn emit_rollback_write_authority_gate(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    if let Some(probation) = probation {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_STATUS);
        raw(", \"service_id\": ");
        json_str(SERVICE_ID);
        raw(", \"requested_capability\": ");
        json_str(HELLO_ROLLBACK_APPLY_CAPABILITY);
        raw(", \"gate_hash\": ");
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
        raw(", \"rollback_target_artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"current_candidate_artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"state_migration_hash\": ");
        json_sha256(probation.state_migration_hash);
        raw(", \"required_schemas\": {\"audit_record\": \"raios.audit_record.v0\", \"rollback_transaction\": \"raios.rollback_transaction.v0\"}");
        raw(", \"unavailable_authorities\": {\"durable_audit_write_authority\": true, \"rollback_store_write_authority\": true, \"rollback_transaction_append\": true}");
        raw(", \"side_effects\": {\"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_plan\": false, \"applies_rollback\": false}");
        raw("}");
    } else {
        raw("null");
    }
}

pub(crate) fn emit_rollback_append_intent_gate(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    if let Some(probation) = probation {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_APPEND_INTENT_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_APPEND_INTENT_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_APPEND_INTENT_GATE_STATUS);
        raw(", \"service_id\": ");
        json_str(SERVICE_ID);
        raw(", \"requested_capability\": ");
        json_str(HELLO_ROLLBACK_APPLY_CAPABILITY);
        raw(", \"gate_hash\": ");
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
        raw(", \"unavailable_authorities\": {\"append_intent\": true, \"rollback_transaction_append\": true, \"durable_audit_store\": true, \"rollback_store\": true}");
        raw(", \"side_effects\": {\"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_plan\": false, \"applies_rollback\": false}");
        raw("}");
    } else {
        raw("null");
    }
}

pub(crate) fn emit_rollback_payload_envelope_gate(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    if let Some(probation) = probation {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_STATUS);
        raw(", \"service_id\": ");
        json_str(SERVICE_ID);
        raw(", \"requested_capability\": ");
        json_str(HELLO_ROLLBACK_APPLY_CAPABILITY);
        raw(", \"gate_hash\": ");
        json_sha256(hello_rollback_payload_envelope_gate_hash(
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
        raw(", \"proposed_transaction\": {\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PAYLOAD_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PAYLOAD_STATUS);
        raw(", \"payload_hash\": ");
        json_sha256(hello_rollback_transaction_payload_hash(snapshot, probation));
        raw(", \"provenance_hash\": ");
        json_sha256(hello_rollback_transaction_payload_provenance_hash(
            snapshot, probation,
        ));
        raw(", \"appended_to_rollback_log\": false}");
        raw(", \"required_schemas\": {\"audit_record\": \"raios.audit_record.v0\", \"rollback_transaction\": \"raios.rollback_transaction.v0\"}");
        raw(", \"unavailable_authorities\": {\"transaction_writer\": true, \"rollback_transaction_append\": true, \"durable_audit_store\": true, \"rollback_store\": true}");
        raw(", \"side_effects\": {\"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_plan\": false, \"applies_rollback\": false}");
        raw("}");
    } else {
        raw("null");
    }
}

pub(crate) fn emit_audit_rollback_target_region_discovery_inline(
    discovery: rollback_storage_layout::AuditRollbackTargetRegionDiscovery,
) {
    raw("{\"schema\": ");
    json_str(discovery.schema);
    raw(", \"id\": ");
    json_str(discovery.id);
    raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
    json_str(discovery.status);
    raw(", \"reason\": ");
    json_str(discovery.reason);
    raw(", \"source\": ");
    json_str(discovery.source);
    raw(", \"storage_authority_id\": ");
    json_str(discovery.storage_authority_id);
    raw(", \"partition_inventory_available\": ");
    raw_bool(discovery.partition_inventory_available);
    raw(", \"partition_inventory_scheme\": ");
    json_str(discovery.partition_inventory_scheme);
    raw(", \"partition_inventory_source_lba\": ");
    raw_fmt(format_args!("{}", discovery.partition_inventory_source_lba));
    raw(", \"partition_entry_count\": ");
    raw_fmt(format_args!("{}", discovery.partition_entry_count));
    raw(", \"mbr_signature_valid\": ");
    raw_bool(discovery.mbr_signature_valid);
    raw(", \"boot_metadata_lba\": ");
    raw_fmt(format_args!("{}", discovery.boot_metadata_lba));
    raw(", \"candidate_region_present\": ");
    raw_bool(discovery.candidate_region_present);
    raw(", \"candidate_region_start_lba\": ");
    raw_fmt(format_args!("{}", discovery.candidate_region_start_lba));
    raw(", \"candidate_region_lba_count\": ");
    raw_fmt(format_args!("{}", discovery.candidate_region_lba_count));
    raw(", \"candidate_region_is_scratch\": ");
    raw_bool(discovery.candidate_region_is_scratch);
    raw(", \"candidate_overlaps_boot_metadata\": ");
    raw_bool(discovery.candidate_overlaps_boot_metadata);
    raw(", \"candidate_overlaps_scratch\": ");
    raw_bool(discovery.candidate_overlaps_scratch);
    raw(", \"scratch_region_id\": ");
    json_str(discovery.scratch_region_id);
    raw(", \"scratch_region_available\": ");
    raw_bool(discovery.scratch_region_available);
    raw(", \"scratch_region_start_lba\": ");
    raw_fmt(format_args!("{}", discovery.scratch_region_start_lba));
    raw(", \"scratch_region_lba_count\": ");
    raw_fmt(format_args!("{}", discovery.scratch_region_lba_count));
    raw(", \"scratch_rejected_as_durable_authority\": ");
    raw_bool(discovery.scratch_rejected_as_durable_authority);
    raw(", \"durable_region_available\": ");
    raw_bool(discovery.durable_region_available);
    raw(", \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"write_attempted\": false}");
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
    raw("{\"schema\": ");
    json_str(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA);
    raw(", \"id\": ");
    json_str(HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID);
    raw(", \"status\": ");
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
    raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_state\": false, \"applies_rollback\": false}");
}

pub(crate) fn emit_target_region_sector_inspection_inline(
    inspection: RollbackTargetRegionSectorInspection,
) {
    raw("{\"schema\": ");
    json_str(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_SCHEMA);
    raw(", \"id\": ");
    json_str(HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_ID);
    raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
    json_str(inspection.status);
    raw(", \"reason\": ");
    json_str(inspection.reason);
    raw(", \"inspection_hash\": ");
    json_sha256(inspection.inspection_hash);
    raw(", \"source_sector_plan_hash\": ");
    json_sha256(inspection.source_sector_plan_hash);
    raw(", \"source_target_region_write_readback_hash\": ");
    json_sha256(inspection.source_target_region_write_readback_hash);
    raw(", \"expected_sector_image_hash\": ");
    json_sha256(inspection.expected_sector_image_hash);
    raw(", \"sector_image_hash\": ");
    json_sha256(inspection.sector_image_hash);
    raw(", \"audit_record_schema\": ");
    json_str(rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
    raw(", \"audit_record_image_hash\": ");
    json_sha256(inspection.audit_record_image_hash);
    raw(", \"rollback_transaction_schema\": ");
    json_str(rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
    raw(", \"rollback_transaction_image_hash\": ");
    json_sha256(inspection.rollback_transaction_image_hash);
    raw(", \"target_start_lba\": ");
    raw_fmt(format_args!("{}", inspection.target_start_lba));
    raw(", \"target_lba_count\": ");
    raw_fmt(format_args!("{}", inspection.target_lba_count));
    raw(", \"target_byte_count\": ");
    raw_fmt(format_args!("{}", inspection.target_byte_count));
    raw(", \"audit_record_offset\": ");
    raw_fmt(format_args!("{}", inspection.audit_record_offset));
    raw(", \"audit_record_byte_length\": ");
    raw_fmt(format_args!("{}", inspection.audit_record_byte_length));
    raw(", \"rollback_transaction_offset\": ");
    raw_fmt(format_args!("{}", inspection.rollback_transaction_offset));
    raw(", \"rollback_transaction_byte_length\": ");
    raw_fmt(format_args!(
        "{}",
        inspection.rollback_transaction_byte_length
    ));
    raw(", \"padding_offset\": ");
    raw_fmt(format_args!("{}", inspection.padding_offset));
    raw(", \"padding_byte_length\": ");
    raw_fmt(format_args!("{}", inspection.padding_byte_length));
    raw(", \"label_found\": ");
    raw_bool(inspection.label_found);
    raw(", \"read_attempted\": ");
    raw_bool(inspection.read_attempted);
    raw(", \"read_completed\": ");
    raw_bool(inspection.read_completed);
    raw(", \"sector_hash_verified\": ");
    raw_bool(inspection.sector_hash_verified);
    raw(", \"audit_record_hash_verified\": ");
    raw_bool(inspection.audit_record_hash_verified);
    raw(", \"rollback_transaction_hash_verified\": ");
    raw_bool(inspection.rollback_transaction_hash_verified);
    raw(", \"offsets_verified\": ");
    raw_bool(inspection.offsets_verified);
    raw(", \"padding_zeroed\": ");
    raw_bool(inspection.padding_zeroed);
    raw(", \"target_span_verified\": ");
    raw_bool(inspection.target_span_verified);
    raw(", \"target_region_write_readback_verified\": ");
    raw_bool(inspection.target_region_write_readback_verified);
    raw(", \"inspection_verified\": ");
    raw_bool(inspection.inspection_verified);
    raw(", \"authorizes_media_write\": false, \"authorizes_append\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"appends_rollback_transaction\": false, \"installs_rollback_state\": false, \"applies_rollback\": false}");
}

pub(crate) fn emit_recovery_rollback_inspect_source_reference_inline(
    inspection: RollbackTargetRegionSectorInspection,
) {
    let state = recovery_rollback_inspect_source_reference_state(inspection);
    let reference = state.reference;
    let source_matches_sector_inspection = state.ram_audit_validated;
    raw("{\"schema\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SCHEMA);
    raw(", \"id\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_ID);
    raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
    json_str(state.status);
    raw(", \"reason\": ");
    json_str(state.reason);
    raw(", \"source_method\": \"recovery.rollback_inspect\", \"source_event_id\": ");
    json_event_id_option(reference.map(|reference| reference.event_id));
    raw(", \"source_audit_event_id\": ");
    json_event_id_option(reference.map(|reference| reference.audit_event_id));
    raw(", \"source_available\": ");
    raw_bool(source_matches_sector_inspection);
    raw(", \"source_matches_sector_inspection\": ");
    raw_bool(source_matches_sector_inspection);
    raw(", \"source_event_retained\": ");
    raw_bool(state.source_event_retained);
    raw(", \"source_audit_event_retained\": ");
    raw_bool(state.audit_event_retained);
    raw(", \"ram_audit_status\": ");
    json_str(state.ram_audit_status);
    raw(", \"ram_audit_reason\": ");
    json_str(state.ram_audit_reason);
    raw(", \"ram_audit_validated\": ");
    raw_bool(state.ram_audit_validated);
    raw(", \"reference_hash\": ");
    json_sha256_option(reference.map(|reference| reference.reference_hash));
    raw(", \"source_inspection_hash\": ");
    json_sha256_option(reference.map(|reference| reference.inspection_hash));
    raw(", \"target_region_sector_inspection_hash\": ");
    json_sha256(inspection.inspection_hash);
    raw(", \"source_sector_plan_hash\": ");
    json_sha256_option(reference.map(|reference| reference.source_sector_plan_hash));
    raw(", \"source_target_region_write_readback_hash\": ");
    json_sha256_option(
        reference.map(|reference| reference.source_target_region_write_readback_hash),
    );
    raw(", \"authorizes_rollback_apply\": false}");
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
    raw("      \"schema\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_SCHEMA);
    raw_line(",");
    raw("      \"id\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": false,");
    raw_line("      \"test_infrastructure\": true,");
    raw("      \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"status\": ");
    json_str(status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"requested_capability\": ");
    json_str("cap.recovery.rollback_materialize_dry_run.current_boot");
    raw_line(",");
    raw("      \"active_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("      \"source_probation\": ");
    emit_hello_hot_swap_probation_option(snapshot.hot_swap_probation);
    raw_line(",");
    raw("      \"append_record_dry_run\": ");
    if let Some((append_record, _, _)) = evidence {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_ID);
        raw(", \"dry_run_hash\": ");
        json_sha256(append_record.dry_run_hash);
        raw(", \"audit_record_image_hash\": ");
        json_sha256(append_record.audit_record_image_hash);
        raw(", \"rollback_transaction_image_hash\": ");
        json_sha256(append_record.rollback_transaction_image_hash);
        raw(", \"audit_record_byte_length\": ");
        raw_fmt(format_args!("{}", append_record.audit_record_byte_length));
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
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"sector_plan_dry_run\": ");
    if let Some((_, sector_plan, _)) = evidence {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_ID);
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
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"target_region_write_readback\": ");
    if let Some((_, _, target_region_write)) = evidence {
        emit_target_region_write_readback_inline(target_region_write);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"materialized_sector_evidence_available\": ");
    raw_bool(materialized_sector_evidence_available);
    raw_line(",");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"mutates_service_state\": false,");
    raw_line("        \"descriptor_mutation\": \"not_attempted\",");
    raw_line("        \"generation_mutation\": \"not_attempted\",");
    raw_line("        \"running_state_mutation\": \"not_attempted\",");
    raw_line("        \"ram_only_state_mutation\": \"not_attempted\",");
    raw_line("        \"authorizes_media_write\": false,");
    raw_line("        \"authorizes_append\": false,");
    raw_line("        \"authorizes_transaction_append\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"appends_rollback_transaction\": false,");
    raw_line("        \"applies_rollback\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"persistence\": \"denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"candidate_artifact_execution\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"provider_auto_load\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
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
    raw("      \"schema\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_INSPECT_SCHEMA);
    raw_line(",");
    raw("      \"id\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_INSPECT_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw("      \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"status\": ");
    json_str(status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"requested_capability\": ");
    json_str("cap.recovery.rollback_inspect.read");
    raw_line(",");
    raw("      \"active_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("      \"source_probation\": ");
    emit_hello_hot_swap_probation_option(snapshot.hot_swap_probation);
    raw_line(",");
    raw("      \"materialized_sector_evidence_available\": ");
    raw_bool(materialized_sector_evidence_available);
    raw_line(",");
    raw("      \"inspection_available\": ");
    raw_bool(inspection_available);
    raw_line(",");
    raw("      \"target_region_write_readback\": ");
    if let Some((target_region_write, _)) = evidence {
        emit_target_region_write_readback_inline(target_region_write);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"target_region_sector_inspection\": ");
    if let Some((_, inspection)) = evidence {
        emit_target_region_sector_inspection_inline(inspection);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"retained_recovery_rollback_inspect_source\": ");
    if let Some((_, inspection)) = evidence {
        emit_recovery_rollback_inspect_source_reference_inline(inspection);
    } else {
        raw("null");
    }
    raw_line(",");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"authorizes_media_write\": false,");
    raw_line("        \"authorizes_append\": false,");
    raw_line("        \"authorizes_transaction_append\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"appends_rollback_transaction\": false,");
    raw_line("        \"applies_rollback\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"persistence\": \"denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"candidate_artifact_execution\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"provider_auto_load\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
}

pub(crate) fn emit_rollback_apply_denied(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let probation = snapshot.hot_swap_probation;
    let retained_denial_sources = probation
        .map(|probation| hello_rollback_apply_retained_denial_sources(snapshot, probation));
    raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw("    \"schema\": ");
    json_str(HELLO_ROLLBACK_APPLY_SCHEMA);
    raw_line(",");
    raw("    \"id\": ");
    json_str(HELLO_ROLLBACK_APPLY_ID);
    raw_line(",");
    raw_line("    \"scope\": \"current_boot\",");
    raw_line("    \"classification\": \"local_only\",");
    raw_line("    \"persistence\": \"none\",");
    raw("    \"status\": ");
    json_str(HELLO_ROLLBACK_APPLY_STATUS);
    raw_line(",");
    raw("    \"reason\": ");
    json_str(if probation.is_some() {
        "rollback_apply_authority_missing"
    } else if snapshot.loaded {
        "rollback_preview_or_probation_missing"
    } else {
        "service_not_loaded"
    });
    raw_line(",");
    raw("    \"message\": ");
    json_str("hello rollback apply is denied until rollback apply authority, durable audit, and rollback transaction evidence exist");
    raw_line(",");
    raw("    \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("    \"active_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("    \"active_descriptor_id\": ");
    json_str(snapshot.load_descriptor.id);
    raw_line(",");
    raw("    \"current_state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("    \"source_probation\": ");
    emit_hello_hot_swap_probation_option(probation);
    raw_line(",");
    raw("    \"required_preview\": ");
    if let Some(probation) = probation {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_PREVIEW_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_PREVIEW_ID);
        raw(", \"status\": ");
        json_str(HELLO_ROLLBACK_PREVIEW_STATUS);
        raw(", \"preview_hash\": ");
        json_sha256(hello_rollback_preview_hash(snapshot, probation));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"rollback_apply_hash\": ");
    if let Some(probation) = probation {
        if let Some((durable_policy_write_authority_decision, recovery_inspect_source_state)) =
            retained_denial_sources
        {
            json_sha256(hello_rollback_apply_denial_hash_with_retained_sources(
                snapshot,
                probation,
                Some(durable_policy_write_authority_decision),
                Some(recovery_inspect_source_state),
            ));
        } else {
            json_sha256(hello_rollback_apply_denial_hash(snapshot, probation));
        }
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"source_durable_policy_write_authority_decision_hash\": ");
    if let Some((durable_policy_write_authority_decision, _)) = retained_denial_sources {
        json_sha256(durable_policy_write_authority_decision.decision_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"source_recovery_rollback_inspect_source_reference_hash\": ");
    if let Some((_, recovery_inspect_source_state)) = retained_denial_sources {
        json_sha256_option(
            recovery_inspect_source_state
                .reference
                .map(|reference| reference.reference_hash),
        );
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"retained_durable_policy_write_authority_decision_verified\": ");
    if let Some((durable_policy_write_authority_decision, _)) = retained_denial_sources {
        raw_bool(
            durable_policy_write_authority_decision.transaction_append_dry_run_verified
                && durable_policy_write_authority_decision.target_region_sector_inspection_verified
                && durable_policy_write_authority_decision.write_authority_evidence_verified
                && durable_policy_write_authority_decision
                    .audit_policy_availability_evidence_verified
                && durable_policy_write_authority_decision
                    .durable_append_authority_availability_evidence_verified
                && durable_policy_write_authority_decision.target_span_verified,
        );
    } else {
        raw_bool(false);
    }
    raw_line(",");
    raw("    \"retained_recovery_rollback_inspect_source_reference_validated\": ");
    if let Some((_, recovery_inspect_source_state)) = retained_denial_sources {
        raw_bool(recovery_inspect_source_state.ram_audit_validated);
    } else {
        raw_bool(false);
    }
    raw_line(",");
    raw("    \"rollback_transaction_preflight\": ");
    emit_rollback_transaction_preflight(snapshot, probation);
    raw_line(",");
    raw("    \"rollback_write_authority_gate\": ");
    emit_rollback_write_authority_gate(snapshot, probation);
    raw_line(",");
    raw("    \"rollback_append_intent_gate\": ");
    emit_rollback_append_intent_gate(snapshot, probation);
    raw_line(",");
    raw("    \"rollback_payload_envelope_gate\": ");
    emit_rollback_payload_envelope_gate(snapshot, probation);
    raw_line(",");
    raw("    \"rollback_transaction_writer_storage_authority_gate\": ");
    emit_rollback_transaction_writer_storage_authority_gate(snapshot, probation);
    raw_line(",");
    raw("    \"rollback_target\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.previous_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.previous_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.previous_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.previous_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.previous_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.previous_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.previous_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"current_candidate\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.new_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.new_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.new_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.new_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.new_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.new_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.new_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"rollback_apply_authority\",");
    raw_line("      \"rollback_transaction_authority\",");
    raw_line("      \"durable_audit_write_authority\",");
    raw_line("      \"persistent_install_authority\",");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"raios.rollback_transaction.v0\",");
    raw_line("      \"durable_audit_rollback_store\"");
    raw_line("    ],");
    raw_line("    \"denied_surfaces\": {");
    raw_line("      \"mutates_service_state\": false,");
    raw_line("      \"applies_rollback\": false,");
    raw_line("      \"descriptor_mutation\": \"not_attempted\",");
    raw_line("      \"generation_mutation\": \"not_attempted\",");
    raw_line("      \"running_state_mutation\": \"not_attempted\",");
    raw_line("      \"ram_only_state_mutation\": \"not_attempted\",");
    raw_line("      \"persistent_install\": \"denied\",");
    raw_line("      \"durable_audit_write\": \"denied\",");
    raw_line("      \"external_artifact_load\": \"denied\",");
    raw_line("      \"candidate_artifact_execution\": \"denied\",");
    raw_line("      \"executable_mapping\": \"denied\",");
    raw_line("      \"provider_auto_load\": \"denied\",");
    raw_line("      \"broad_mutation\": \"denied\"");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

pub(crate) fn emit_rollback_preview_response(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let probation = snapshot.hot_swap_probation;
    begin_response(method);
    raw_line("      \"schema\": \"raios.ram_only_hello_service_rollback_preview.v0\",");
    raw("      \"id\": ");
    json_str(HELLO_ROLLBACK_PREVIEW_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw("      \"status\": ");
    json_str(if probation.is_some() {
        HELLO_ROLLBACK_PREVIEW_STATUS
    } else {
        "missing_hot_swap_probation"
    });
    raw_line(",");
    raw("      \"preview_available\": ");
    raw_bool(probation.is_some());
    raw_line(",");
    raw("      \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"current_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("      \"current_state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("      \"source_probation\": ");
    emit_hello_hot_swap_probation_option(probation);
    raw_line(",");
    raw("      \"preview_hash\": ");
    if let Some(probation) = probation {
        json_sha256(hello_rollback_preview_hash(snapshot, probation));
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"rollback_target\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.previous_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.previous_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.previous_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.previous_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.previous_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.previous_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.previous_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"current_candidate\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.new_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.new_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.new_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.new_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.new_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.new_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.new_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"mutates_service_state\": false,");
    raw_line("        \"applies_rollback\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit_write\": \"denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"candidate_artifact_execution\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"provider_auto_load\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
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
    raw_line("      \"schema\": \"raios.ram_only_hello_service.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw("      \"action\": ");
    json_str(action);
    raw_line(",");
    raw("      \"event_id\": ");
    json_event_id_option(snapshot.last_event_id);
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id_option(snapshot.last_event_id);
    raw_line(",");
    emit_load_request(descriptor);
    raw_line(",");
    emit_load_descriptor(descriptor);
    raw_line(",");
    raw("      \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(descriptor);
    raw_line(",");
    raw("      \"service_slot_activation\": ");
    emit_service_slot_activation(descriptor, activation_status, activation_active);
    raw_line(",");
    raw("      \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("      \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw("      \"hot_swap_probation\": ");
    emit_hello_hot_swap_probation_option(snapshot.hot_swap_probation);
    raw_line(",");
    raw_line("      \"service\": {");
    raw("        \"id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw("        \"artifact_id\": ");
    json_str(descriptor.artifact_id);
    raw_line(",");
    raw("        \"version\": ");
    json_str(service_version(descriptor));
    raw_line(",");
    raw("        \"artifact_identity_id\": ");
    json_str(descriptor.artifact_identity.id);
    raw_line(",");
    raw("        \"artifact_identity_hash\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_content_binding_id\": ");
    json_str(descriptor.artifact_identity.artifact_content_binding_id);
    raw_line(",");
    raw("        \"artifact_content_binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_content_source_hash\": ");
    json_sha256(descriptor.artifact_identity.artifact_content_source_hash);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_reference_id\": ");
    json_str(descriptor.artifact_identity.artifact_reference_id);
    raw_line(",");
    raw("        \"artifact_reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_bytes_sha256\": ");
    json_sha256(artifact_reference_bytes_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_reference_content_binding_hash\": ");
    json_sha256(
        descriptor
            .artifact_identity
            .artifact_reference_content_binding_hash,
    );
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_hash\": ");
    json_sha256(artifact_load_plan_preflight_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_status\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    raw_line(",");
    raw("        \"service_slot_activation_id\": ");
    json_str(SERVICE_SLOT_ACTIVATION_ID);
    raw_line(",");
    raw("        \"service_slot_activation_hash\": ");
    json_sha256(service_slot_activation_hash(descriptor));
    raw_line(",");
    raw("        \"service_slot_activation_status\": ");
    json_str(activation_status);
    raw_line(",");
    raw("        \"service_slot_activation_active\": ");
    raw_bool(activation_active);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw_line(",");
    raw("        \"load_descriptor_id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw("        \"load_descriptor_source_kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("        \"load_descriptor_source_validated\": true,");
    raw("        \"load_descriptor_source_hash\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("        \"load_descriptor_source_signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line(",");
    raw_line("        \"kind\": \"service\",");
    raw("        \"loaded\": ");
    raw_bool(snapshot.loaded);
    raw_line(",");
    raw("        \"running\": ");
    raw_bool(snapshot.running);
    raw_line(",");
    raw("        \"generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("        \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("        \"health\": ");
    json_str(if snapshot.running {
        "healthy"
    } else if snapshot.loaded {
        "stopped"
    } else {
        "missing"
    });
    raw_line(",");
    raw("        \"capabilities\": [");
    emit_inline_string_array(CAPABILITIES);
    raw_line("]");
    raw_line("      },");
    raw_line("      \"lifecycle\": {");
    raw("        \"last_action\": ");
    json_str(snapshot.last_action);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(snapshot.last_reason);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(snapshot.last_inventory_change);
    raw_line(",");
    raw("        \"load_event_id\": ");
    json_event_id_option(snapshot.load_event_id);
    raw_line(",");
    raw("        \"start_event_id\": ");
    json_event_id_option(snapshot.start_event_id);
    raw_line(",");
    raw("        \"hot_swap_event_id\": ");
    json_event_id_option(snapshot.hot_swap_event_id);
    raw_line(",");
    raw("        \"stop_event_id\": ");
    json_event_id_option(snapshot.stop_event_id);
    raw_line(",");
    raw("        \"drop_event_id\": ");
    json_event_id_option(snapshot.drop_event_id);
    raw_line("");
    raw_line("      },");
    raw_line("      \"loader\": {");
    raw("        \"kind\": ");
    json_str(descriptor.artifact_kind);
    raw_line(",");
    raw("        \"descriptor_id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw("        \"descriptor_source_locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("        \"descriptor_source_kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("        \"descriptor_source_validated\": true,");
    raw("        \"descriptor_source_hash\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("        \"descriptor_source_signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_identity_id\": ");
    json_str(descriptor.artifact_identity.id);
    raw_line(",");
    raw("        \"artifact_identity_hash\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_content_binding_id\": ");
    json_str(descriptor.artifact_identity.artifact_content_binding_id);
    raw_line(",");
    raw("        \"artifact_content_binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_content_source_hash\": ");
    json_sha256(descriptor.artifact_identity.artifact_content_source_hash);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_reference_id\": ");
    json_str(descriptor.artifact_identity.artifact_reference_id);
    raw_line(",");
    raw("        \"artifact_reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_bytes_sha256\": ");
    json_sha256(artifact_reference_bytes_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_reference_content_binding_hash\": ");
    json_sha256(
        descriptor
            .artifact_identity
            .artifact_reference_content_binding_hash,
    );
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_hash\": ");
    json_sha256(artifact_load_plan_preflight_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_status\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    raw_line(",");
    raw("        \"service_slot_activation_id\": ");
    json_str(SERVICE_SLOT_ACTIVATION_ID);
    raw_line(",");
    raw("        \"service_slot_activation_hash\": ");
    json_sha256(service_slot_activation_hash(descriptor));
    raw_line(",");
    raw("        \"service_slot_activation_status\": ");
    json_str(activation_status);
    raw_line(",");
    raw("        \"service_slot_activation_active\": ");
    raw_bool(activation_active);
    raw_line(",");
    raw("        \"service_slot_intent_id\": ");
    json_str(SERVICE_SLOT_INTENT_ID);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw_line(",");
    raw("        \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("        \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("        \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line(",");
    raw_line("        \"accepts_external_artifact_bytes\": false,");
    raw_line("        \"loads_external_artifact\": false,");
    raw_line("        \"maps_executable_pages\": false,");
    raw_line("        \"writes_persistent_state\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"grants_broad_mutation\": false");
    raw_line("      },");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"general_module_load\": \"unchanged_denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
}

pub(crate) fn emit_load_request(descriptor: LoadDescriptor) {
    raw_line("      \"load_request\": {");
    raw_line("        \"schema\": \"raios.current_boot_load_request.v0\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"descriptor_schema\": ");
    json_str(descriptor.schema);
    raw_line(",");
    raw("        \"descriptor_id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw("        \"descriptor_source_locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("        \"descriptor_source_kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("        \"descriptor_source_validated\": true,");
    raw("        \"descriptor_source_hash\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("        \"descriptor_source_signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_identity_id\": ");
    json_str(descriptor.artifact_identity.id);
    raw_line(",");
    raw("        \"artifact_identity_hash\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_content_binding_id\": ");
    json_str(descriptor.artifact_identity.artifact_content_binding_id);
    raw_line(",");
    raw("        \"artifact_content_binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_content_source_hash\": ");
    json_sha256(descriptor.artifact_identity.artifact_content_source_hash);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_reference_id\": ");
    json_str(descriptor.artifact_identity.artifact_reference_id);
    raw_line(",");
    raw("        \"artifact_reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_bytes_sha256\": ");
    json_sha256(artifact_reference_bytes_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_reference_content_binding_hash\": ");
    json_sha256(
        descriptor
            .artifact_identity
            .artifact_reference_content_binding_hash,
    );
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_hash\": ");
    json_sha256(artifact_load_plan_preflight_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_status\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    raw_line(",");
    raw("        \"service_slot_intent_id\": ");
    json_str(SERVICE_SLOT_INTENT_ID);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw_line(",");
    raw("        \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("        \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("        \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line(",");
    raw("        \"service_id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw_line("        \"accepted\": true");
    raw("      }");
}

pub(crate) fn emit_load_descriptor(descriptor: LoadDescriptor) {
    raw_line("      \"load_descriptor\": {");
    raw("        \"schema\": ");
    json_str(descriptor.schema);
    raw_line(",");
    raw("        \"id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw_line("        \"source\": {");
    raw("          \"canonicalization\": ");
    json_str(descriptor.canonicalization);
    raw_line(",");
    raw("          \"locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("          \"kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("          \"validated\": true,");
    raw("          \"sha256\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("          \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("          \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("          \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line(",");
    raw("          \"signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line(",");
    raw("          \"text\": ");
    json_str(descriptor.source_text);
    raw_line("");
    raw_line("        },");
    raw("        \"service_id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw("        \"artifact_id\": ");
    json_str(descriptor.artifact_id);
    raw_line(",");
    raw("        \"artifact_kind\": ");
    json_str(descriptor.artifact_kind);
    raw_line(",");
    raw("        \"artifact_identity\": ");
    emit_artifact_identity(descriptor);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(descriptor);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(descriptor.scope);
    raw_line(",");
    raw("        \"classification\": ");
    json_str(descriptor.classification);
    raw_line(",");
    raw("        \"persistence\": ");
    json_str(descriptor.persistence);
    raw_line(",");
    raw_line("        \"accepts_external_artifact_bytes\": false,");
    raw_line("        \"loads_external_artifact\": false,");
    raw_line("        \"maps_executable_pages\": false,");
    raw_line("        \"writes_persistent_state\": false");
    raw("      }");
}

pub(crate) fn emit_artifact_load_plan_preflight(descriptor: LoadDescriptor) {
    let record = artifact_load_plan_preflight_record(descriptor);
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"status\": ");
    json_str(record.status);
    raw(", \"preflight_hash\": ");
    json_sha256(record.preflight_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"artifact_id\": ");
    json_str(record.artifact_id);
    raw(", \"load_descriptor_id\": ");
    json_str(record.load_descriptor_id);
    raw(", \"descriptor_source_locator\": ");
    json_str(record.descriptor_source_locator);
    raw(", \"descriptor_source_hash\": ");
    json_sha256(record.descriptor_source_hash);
    raw(", \"artifact_identity_id\": ");
    json_str(record.artifact_identity_id);
    raw(", \"artifact_identity_hash\": ");
    json_sha256(record.artifact_identity_hash);
    raw(", \"artifact_content_binding_hash\": ");
    json_sha256(record.artifact_content_binding_hash);
    raw(", \"artifact_reference_id\": ");
    json_str(record.artifact_reference_id);
    raw(", \"artifact_reference_hash\": ");
    json_sha256(record.artifact_reference_hash);
    raw(", \"artifact_bytes_sha256\": ");
    json_sha256(record.artifact_bytes_sha256);
    raw(", \"service_slot_intent_schema\": ");
    json_str(record.service_slot_intent_schema);
    raw(", \"service_slot_intent_id\": ");
    json_str(record.service_slot_intent_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"accepted\": ");
    raw_bool(record.accepted);
    raw(", \"authorizes_builtin_current_boot_start\": ");
    raw_bool(record.authorizes_builtin_current_boot_start);
    raw(", \"authorizes_candidate_artifact_execution\": ");
    raw_bool(record.authorizes_candidate_artifact_execution);
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(record.accepts_external_artifact_bytes);
    raw(", \"loads_candidate_bytes\": ");
    raw_bool(record.loads_candidate_bytes);
    raw(", \"maps_executable_pages\": ");
    raw_bool(record.maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw(", \"writes_durable_audit_log\": ");
    raw_bool(record.writes_durable_audit_log);
    raw(", \"installs_rollback_plan\": ");
    raw_bool(record.installs_rollback_plan);
    raw(", \"grants_broad_mutation\": ");
    raw_bool(record.grants_broad_mutation);
    raw("}");
}

pub(crate) fn emit_service_slot_activation(
    descriptor: LoadDescriptor,
    status: &'static str,
    active: bool,
) {
    let record = service_slot_activation_record(descriptor, status, active);
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"persistence\": ");
    json_str(record.persistence);
    raw(", \"status\": ");
    json_str(record.status);
    raw(", \"activation_hash\": ");
    json_sha256(record.activation_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"artifact_id\": ");
    json_str(record.artifact_id);
    raw(", \"load_descriptor_id\": ");
    json_str(record.load_descriptor_id);
    raw(", \"descriptor_source_hash\": ");
    json_sha256(record.descriptor_source_hash);
    raw(", \"artifact_load_plan_preflight_id\": ");
    json_str(record.artifact_load_plan_preflight_id);
    raw(", \"artifact_load_plan_preflight_hash\": ");
    json_sha256(record.artifact_load_plan_preflight_hash);
    raw(", \"artifact_load_plan_preflight_status\": ");
    json_str(record.artifact_load_plan_preflight_status);
    raw(", \"service_slot_intent_id\": ");
    json_str(record.service_slot_intent_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"active\": ");
    raw_bool(record.active);
    raw(", \"accepted_preflight\": ");
    raw_bool(record.accepted_preflight);
    raw(", \"authorizes_builtin_current_boot_start\": ");
    raw_bool(record.authorizes_builtin_current_boot_start);
    raw(", \"authorizes_candidate_artifact_execution\": ");
    raw_bool(record.authorizes_candidate_artifact_execution);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw("}");
}

pub(crate) fn emit_hello_state(snapshot: Snapshot) {
    raw("{");
    raw("\"schema\": ");
    json_str(HELLO_STATE_SCHEMA);
    raw(", \"id\": ");
    json_str(HELLO_STATE_ID);
    raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\"");
    raw(", \"service_id\": ");
    json_str(SERVICE_ID);
    raw(", \"version\": ");
    json_str(service_version(snapshot.load_descriptor));
    raw(", \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw(", \"state_counter\": ");
    raw_fmt(format_args!("{}", snapshot.state_counter));
    raw(", \"state_hash\": ");
    json_sha256(hello_state_hash(snapshot.state_counter));
    raw(", \"loaded\": ");
    raw_bool(snapshot.loaded);
    raw(", \"running\": ");
    raw_bool(snapshot.running);
    raw(", \"writes_persistent_state\": false");
    raw("}");
}

pub(crate) fn emit_hello_state_migration_option(record: Option<HelloStateMigrationRecord>) {
    let Some(record) = record else {
        raw("null");
        return;
    };
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"persistence\": ");
    json_str(record.persistence);
    raw(", \"migration_hash\": ");
    json_sha256(record.migration_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"from_version\": ");
    json_str(record.from_version);
    raw(", \"to_version\": ");
    json_str(record.to_version);
    raw(", \"pre_state_hash\": ");
    json_sha256(record.pre_state_hash);
    raw(", \"post_state_hash\": ");
    json_sha256(record.post_state_hash);
    raw(", \"pre_state_counter\": ");
    raw_fmt(format_args!("{}", record.pre_state_counter));
    raw(", \"post_state_counter\": ");
    raw_fmt(format_args!("{}", record.post_state_counter));
    raw(", \"state_preserved\": ");
    raw_bool(record.state_preserved);
    raw(", \"accepted\": ");
    raw_bool(record.accepted);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw(", \"writes_durable_audit_log\": ");
    raw_bool(record.writes_durable_audit_log);
    raw(", \"installs_rollback_plan\": ");
    raw_bool(record.installs_rollback_plan);
    raw("}");
}

pub(crate) fn emit_hello_hot_swap_probation_option(record: Option<HelloHotSwapProbationRecord>) {
    let Some(record) = record else {
        raw("null");
        return;
    };
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"persistence\": ");
    json_str(record.persistence);
    raw(", \"status\": ");
    json_str(record.status);
    raw(", \"probation_hash\": ");
    json_sha256(record.probation_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"previous_version\": ");
    json_str(record.previous_version);
    raw(", \"new_version\": ");
    json_str(record.new_version);
    raw(", \"previous_descriptor_id\": ");
    json_str(record.previous_descriptor_id);
    raw(", \"new_descriptor_id\": ");
    json_str(record.new_descriptor_id);
    raw(", \"previous_descriptor_source_hash\": ");
    json_sha256(record.previous_descriptor_source_hash);
    raw(", \"new_descriptor_source_hash\": ");
    json_sha256(record.new_descriptor_source_hash);
    raw(", \"previous_artifact_identity_id\": ");
    json_str(record.previous_artifact_identity_id);
    raw(", \"new_artifact_identity_id\": ");
    json_str(record.new_artifact_identity_id);
    raw(", \"previous_artifact_identity_hash\": ");
    json_sha256(record.previous_artifact_identity_hash);
    raw(", \"new_artifact_identity_hash\": ");
    json_sha256(record.new_artifact_identity_hash);
    raw(", \"previous_generation\": ");
    raw_fmt(format_args!("{}", record.previous_generation));
    raw(", \"new_generation\": ");
    raw_fmt(format_args!("{}", record.new_generation));
    raw(", \"previous_state_hash\": ");
    json_sha256(record.previous_state_hash);
    raw(", \"new_state_hash\": ");
    json_sha256(record.new_state_hash);
    raw(", \"previous_state_counter\": ");
    raw_fmt(format_args!("{}", record.previous_state_counter));
    raw(", \"new_state_counter\": ");
    raw_fmt(format_args!("{}", record.new_state_counter));
    raw(", \"state_migration_hash\": ");
    json_sha256(record.state_migration_hash);
    raw(", \"accepted\": ");
    raw_bool(record.accepted);
    raw(", \"loads_candidate_bytes\": ");
    raw_bool(record.loads_candidate_bytes);
    raw(", \"maps_executable_pages\": ");
    raw_bool(record.maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw(", \"writes_durable_audit_log\": ");
    raw_bool(record.writes_durable_audit_log);
    raw(", \"installs_rollback_plan\": ");
    raw_bool(record.installs_rollback_plan);
    raw(", \"applies_rollback\": ");
    raw_bool(record.applies_rollback);
    raw("}");
}

pub(crate) fn emit_descriptor_source_signature_envelope(descriptor: LoadDescriptor) {
    let Some(envelope) = descriptor.source_envelope else {
        raw("null");
        return;
    };
    raw("{");
    raw("\"schema\": ");
    json_str(envelope.schema);
    raw(", \"id\": ");
    json_str(envelope.id);
    raw(", \"algorithm\": ");
    json_str(envelope.algorithm);
    raw(", \"verification_phase\": ");
    json_str(envelope.verification_phase);
    raw(", \"trust_scope\": ");
    json_str(envelope.trust_scope);
    raw(", \"envelope_hash\": ");
    json_sha256(envelope.envelope_hash);
    raw(", \"payload_sha256\": ");
    json_sha256(envelope.payload_hash);
    raw(", \"public_key_sha256\": ");
    json_sha256(envelope.public_key_hash);
    raw(", \"signature_sha256\": ");
    json_sha256(envelope.signature_hash);
    raw(", \"signature_verified\": ");
    raw_bool(descriptor_source_signature_verified(descriptor));
    raw(", \"authorizes_external_artifact_load\": ");
    raw_bool(envelope.authorizes_external_artifact_load);
    raw(", \"authorizes_persistent_install\": ");
    raw_bool(envelope.authorizes_persistent_install);
    raw("}");
}

pub(crate) fn emit_artifact_identity(descriptor: LoadDescriptor) {
    let identity = descriptor.artifact_identity;
    raw("{");
    raw("\"schema\": ");
    json_str(identity.schema);
    raw(", \"id\": ");
    json_str(identity.id);
    raw(", \"canonicalization\": ");
    json_str(identity.canonicalization);
    raw(", \"sha256\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw(", \"service_id\": ");
    json_str(identity.service_id);
    raw(", \"artifact_id\": ");
    json_str(identity.artifact_id);
    raw(", \"artifact_kind\": ");
    json_str(identity.artifact_kind);
    raw(", \"load_descriptor_id\": ");
    json_str(identity.load_descriptor_id);
    raw(", \"scope\": ");
    json_str(identity.scope);
    raw(", \"classification\": ");
    json_str(identity.classification);
    raw(", \"persistence\": ");
    json_str(identity.persistence);
    raw(", \"content_binding\": ");
    emit_artifact_content_binding(descriptor);
    raw(", \"artifact_reference\": ");
    emit_artifact_reference(descriptor);
    raw(", \"signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw(", \"validated\": ");
    raw_bool(descriptor_sources::validate_builtin_hello_artifact_identity(identity));
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(identity.accepts_external_artifact_bytes);
    raw(", \"loads_external_artifact\": ");
    raw_bool(identity.loads_external_artifact);
    raw(", \"maps_executable_pages\": ");
    raw_bool(identity.maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(identity.writes_persistent_state);
    raw(", \"authorizes_external_artifact_load\": ");
    raw_bool(identity.authorizes_external_artifact_load);
    raw(", \"authorizes_persistent_install\": ");
    raw_bool(identity.authorizes_persistent_install);
    raw(", \"authorizes_rollback_install\": ");
    raw_bool(identity.authorizes_rollback_install);
    raw("}");
}

pub(crate) fn emit_artifact_content_binding(descriptor: LoadDescriptor) {
    let identity = descriptor.artifact_identity;
    raw("{");
    raw("\"schema\": ");
    json_str(identity.artifact_content_binding_schema);
    raw(", \"id\": ");
    json_str(identity.artifact_content_binding_id);
    raw(", \"artifact_id\": ");
    json_str(identity.artifact_id);
    raw(", \"content_kind\": ");
    json_str(identity.artifact_content_kind);
    raw(", \"source_locator\": ");
    json_str(identity.artifact_content_source_locator);
    raw(", \"source_sha256\": ");
    json_sha256(identity.artifact_content_source_hash);
    raw(", \"binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw(", \"trusted_by_envelope_id\": ");
    json_str(identity.signed_envelope.id);
    raw(", \"trusted_by_envelope_hash\": ");
    json_sha256(identity.signed_envelope.envelope_hash);
    raw(", \"trust_signature_verified\": ");
    raw_bool(artifact_identity_signature_verified(descriptor));
    raw(", \"validated\": ");
    raw_bool(descriptor_sources::validate_builtin_hello_artifact_identity(identity));
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(identity.artifact_content_accepts_external_artifact_bytes);
    raw(", \"loads_external_artifact\": ");
    raw_bool(identity.artifact_content_loads_external_artifact);
    raw(", \"maps_executable_pages\": ");
    raw_bool(identity.artifact_content_maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(identity.artifact_content_writes_persistent_state);
    raw("}");
}

pub(crate) fn emit_artifact_reference(descriptor: LoadDescriptor) {
    let identity = descriptor.artifact_identity;
    raw("{");
    raw("\"schema\": ");
    json_str(identity.artifact_reference_schema);
    raw(", \"id\": ");
    json_str(identity.artifact_reference_id);
    raw(", \"artifact_id\": ");
    json_str(identity.artifact_id);
    raw(", \"service_id\": ");
    json_str(identity.service_id);
    raw(", \"reference_kind\": ");
    json_str(identity.artifact_reference_kind);
    raw(", \"artifact_locator\": ");
    json_str(identity.artifact_reference_locator);
    raw(", \"artifact_bytes_sha256\": ");
    json_sha256(identity.artifact_reference_bytes_hash);
    raw(", \"content_binding_hash\": ");
    json_sha256(identity.artifact_reference_content_binding_hash);
    raw(", \"reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw(", \"trusted_by_envelope_id\": ");
    json_str(identity.signed_envelope.id);
    raw(", \"trusted_by_envelope_hash\": ");
    json_sha256(identity.signed_envelope.envelope_hash);
    raw(", \"trust_signature_verified\": ");
    raw_bool(artifact_identity_signature_verified(descriptor));
    raw(", \"validated\": ");
    raw_bool(descriptor_sources::validate_builtin_hello_artifact_identity(identity));
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(identity.artifact_reference_accepts_external_artifact_bytes);
    raw(", \"loads_artifact_as_code\": ");
    raw_bool(identity.artifact_reference_loads_artifact_as_code);
    raw(", \"maps_executable_pages\": ");
    raw_bool(identity.artifact_reference_maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(identity.artifact_reference_writes_persistent_state);
    raw("}");
}

pub(crate) fn emit_artifact_identity_signature_envelope(descriptor: LoadDescriptor) {
    let envelope = descriptor.artifact_identity.signed_envelope;
    raw("{");
    raw("\"schema\": ");
    json_str(envelope.schema);
    raw(", \"id\": ");
    json_str(envelope.id);
    raw(", \"algorithm\": ");
    json_str(envelope.algorithm);
    raw(", \"verification_phase\": ");
    json_str(envelope.verification_phase);
    raw(", \"trust_scope\": ");
    json_str(envelope.trust_scope);
    raw(", \"envelope_hash\": ");
    json_sha256(envelope.envelope_hash);
    raw(", \"payload_sha256\": ");
    json_sha256(envelope.payload_hash);
    raw(", \"public_key_sha256\": ");
    json_sha256(envelope.public_key_hash);
    raw(", \"signature_sha256\": ");
    json_sha256(envelope.signature_hash);
    raw(", \"signature_verified\": ");
    raw_bool(artifact_identity_signature_verified(descriptor));
    raw(", \"authorizes_external_artifact_load\": ");
    raw_bool(envelope.authorizes_external_artifact_load);
    raw(", \"authorizes_persistent_install\": ");
    raw_bool(envelope.authorizes_persistent_install);
    raw(", \"authorizes_rollback_install\": ");
    raw_bool(envelope.authorizes_rollback_install);
    raw("}");
}
