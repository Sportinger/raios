use super::*;

pub(crate) fn rollback_transaction_writer_storage_authority_gate_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
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
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_STATUS,
    );
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_str(
        &mut hash,
        b"requested_capability",
        HELLO_ROLLBACK_APPLY_CAPABILITY,
    );
    hash_line_str(&mut hash, b"required_audit_schema", "raios.audit_record.v0");
    hash_line_str(
        &mut hash,
        b"required_rollback_schema",
        "raios.rollback_transaction.v0",
    );
    hash_line_str(
        &mut hash,
        b"writer_storage_foundation_schema",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"writer_storage_foundation_owner",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_OWNER,
    );
    hash_line_str(
        &mut hash,
        b"storage_authority_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"storage_authority_id",
        rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID,
    );
    hash_line_str(
        &mut hash,
        b"storage_authority_owner",
        rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER,
    );
    hash_line_str(
        &mut hash,
        b"audit_append_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"audit_append_target_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"append_target_id",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"append_target_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"transaction_writer_owner",
        rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER,
    );
    hash_line_str(
        &mut hash,
        b"append_target_owner_schema",
        rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"append_target_owner_id",
        rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID,
    );
    hash_line_str(
        &mut hash,
        b"append_target_owner_status",
        foundation.append_target_owner_status,
    );
    hash_line_str(
        &mut hash,
        b"append_target_owner_reason",
        foundation.append_target_owner_reason,
    );
    hash_line_bool(
        &mut hash,
        b"append_target_owner_available",
        foundation.append_target_owner_available,
    );
    hash_line_str(
        &mut hash,
        b"transaction_writer_readiness_schema",
        rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"transaction_writer_readiness_id",
        rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_writer_status",
        foundation.transaction_writer_status,
    );
    hash_line_str(
        &mut hash,
        b"transaction_writer_reason",
        foundation.transaction_writer_reason,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_writer_ready",
        foundation.transaction_writer_ready,
    );
    hash_line_bool(
        &mut hash,
        b"block_write_path_available",
        foundation.block_write_path_available,
    );
    hash_line_str(
        &mut hash,
        b"block_write_path_reason",
        foundation.block_write_path_reason,
    );
    hash_line_str(
        &mut hash,
        b"block_write_path_gate_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"block_write_path_gate_id",
        rollback_storage_layout::AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_ID,
    );
    hash_line_str(
        &mut hash,
        b"block_write_path_gate_status",
        foundation.block_write_path_gate_status,
    );
    hash_line_str(
        &mut hash,
        b"block_write_path_gate_reason",
        foundation.block_write_path_reason,
    );
    hash_line_str(
        &mut hash,
        b"read_only_block_driver_id",
        foundation.read_only_block_driver_id,
    );
    hash_line_bool(
        &mut hash,
        b"read_only_block_driver_available",
        foundation.read_only_block_driver_available,
    );
    hash_line_bool(
        &mut hash,
        b"partition_inventory_available",
        foundation.partition_inventory_available,
    );
    hash_line_str(
        &mut hash,
        b"partition_inventory_scheme",
        foundation.partition_inventory_scheme,
    );
    hash_line_str(
        &mut hash,
        b"scratch_block_write_authority_id",
        foundation.scratch_block_write_authority_id,
    );
    hash_line_bool(
        &mut hash,
        b"scratch_block_write_authority_available",
        foundation.scratch_block_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"scratch_region_within_device_bounds",
        foundation.scratch_region_within_device_bounds,
    );
    hash_line_bool(
        &mut hash,
        b"scratch_region_no_boot_or_partition_metadata_overlap",
        foundation.scratch_region_no_boot_or_partition_metadata_overlap,
    );
    hash_line_str(
        &mut hash,
        b"scratch_region_id",
        foundation.scratch_region_id,
    );
    hash_line_u64(
        &mut hash,
        b"scratch_region_start_lba",
        foundation.scratch_region_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"scratch_region_lba_count",
        foundation.scratch_region_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"scratch_region_byte_count",
        foundation.scratch_region_byte_count as u64,
    );
    hash_line_str(
        &mut hash,
        b"scratch_writer_dry_run_schema",
        rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"scratch_writer_dry_run_id",
        rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"scratch_writer_dry_run_status",
        foundation.scratch_writer_dry_run_status,
    );
    hash_line_str(
        &mut hash,
        b"scratch_writer_dry_run_reason",
        foundation.scratch_writer_dry_run_reason,
    );
    hash_line_bool(
        &mut hash,
        b"scratch_writer_dry_run_ready",
        foundation.scratch_writer_dry_run_ready,
    );
    hash_line_str(
        &mut hash,
        b"append_record_dry_run_schema",
        HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"append_record_dry_run_id",
        HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"append_record_dry_run_status",
        HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"append_record_dry_run_reason",
        HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"append_record_dry_run_sha256",
        append_record.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"audit_record_image_sha256",
        append_record.audit_record_image_hash,
    );
    hash_line_u64(
        &mut hash,
        b"audit_record_byte_length",
        append_record.audit_record_byte_length,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_image_sha256",
        append_record.rollback_transaction_image_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_transaction_byte_length",
        append_record.rollback_transaction_byte_length,
    );
    hash_line_u64(
        &mut hash,
        b"total_record_byte_length",
        append_record.total_record_byte_length,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        append_record.target_lba_count,
    );
    hash_line_bool(
        &mut hash,
        b"append_record_target_range_ready",
        append_record.target_range_ready,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_plan_dry_run_schema",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_plan_dry_run_id",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_plan_dry_run_status",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_plan_dry_run_reason",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"append_sector_plan_sha256",
        sector_plan.plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"append_sector_image_sha256",
        sector_plan.sector_image_hash,
    );
    hash_line_u64(
        &mut hash,
        b"append_sector_size_bytes",
        sector_plan.sector_size_bytes,
    );
    hash_line_u64(
        &mut hash,
        b"append_sector_audit_record_offset",
        sector_plan.audit_record_offset,
    );
    hash_line_u64(
        &mut hash,
        b"append_sector_rollback_transaction_offset",
        sector_plan.rollback_transaction_offset,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_padding_policy",
        HELLO_ROLLBACK_APPEND_SECTOR_PADDING_POLICY,
    );
    hash_line_u64(
        &mut hash,
        b"append_sector_padding_offset",
        sector_plan.padding_offset,
    );
    hash_line_u64(
        &mut hash,
        b"append_sector_padding_byte_length",
        sector_plan.padding_byte_length,
    );
    hash_line_bool(
        &mut hash,
        b"append_sector_target_range_ready",
        sector_plan.target_range_ready,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_write_readback_dry_run_schema",
        HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_write_readback_dry_run_id",
        HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_write_readback_dry_run_status",
        sector_write.status,
    );
    hash_line_str(
        &mut hash,
        b"append_sector_write_readback_dry_run_reason",
        sector_write.reason,
    );
    hash_line_hash(
        &mut hash,
        b"append_sector_write_readback_dry_run_sha256",
        sector_write.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"append_sector_readback_sha256",
        sector_write.readback_sector_image_hash,
    );
    hash_line_bool(
        &mut hash,
        b"append_sector_readback_matches_planned_image",
        sector_write.readback_matches_planned_image,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_preflight_schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_preflight_id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_preflight_status",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_preflight_reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_preflight_sha256",
        durable_append_preflight.preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_preflight_source_target_region_write_readback_sha256",
        durable_append_preflight.source_target_region_write_readback_hash,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_preflight_test_media_write_authority_available",
        durable_append_preflight.test_infrastructure_media_write_authority_available,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_preflight_remaining_denial_reason",
        durable_append_preflight.remaining_denial_reason,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_schema",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_id",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_status",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_reason",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_writer_policy_preflight_sha256",
        durable_append_preflight
            .durable_writer_policy_preflight
            .preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_writer_policy_preflight_source_append_record_sha256",
        durable_append_preflight
            .durable_writer_policy_preflight
            .source_append_record_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_writer_policy_preflight_source_sector_plan_sha256",
        durable_append_preflight
            .durable_writer_policy_preflight
            .source_sector_plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_writer_policy_preflight_source_target_region_write_readback_sha256",
        durable_append_preflight
            .durable_writer_policy_preflight
            .source_target_region_write_readback_hash,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_target_range_ready",
        durable_append_preflight
            .durable_writer_policy_preflight
            .target_range_ready,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_test_media_write_authority_available",
        durable_append_preflight
            .durable_writer_policy_preflight
            .test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_durable_audit_writer_available",
        durable_append_preflight
            .durable_writer_policy_preflight
            .durable_audit_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_rollback_store_writer_available",
        durable_append_preflight
            .durable_writer_policy_preflight
            .rollback_store_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_transaction_append_writer_available",
        durable_append_preflight
            .durable_writer_policy_preflight
            .transaction_append_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_transaction_authorization_gate_schema",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_transaction_authorization_gate_id",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_transaction_authorization_gate_status",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_transaction_authorization_gate_reason",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_transaction_authorization_gate_sha256",
        durable_append_transaction_authorization_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_transaction_authorization_gate_source_writer_policy_preflight_sha256",
        durable_append_transaction_authorization_gate.source_writer_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_transaction_authorization_gate_source_append_record_sha256",
        durable_append_transaction_authorization_gate.source_append_record_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_transaction_authorization_gate_source_sector_plan_sha256",
        durable_append_transaction_authorization_gate.source_sector_plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_transaction_authorization_gate_source_target_region_write_readback_sha256",
        durable_append_transaction_authorization_gate.source_target_region_write_readback_hash,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_target_range_ready",
        durable_append_transaction_authorization_gate.target_range_ready,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_test_media_write_authority_available",
        durable_append_transaction_authorization_gate
            .test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_append_engine_available",
        durable_append_transaction_authorization_gate.append_engine_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_durable_audit_writer_available",
        durable_append_transaction_authorization_gate.durable_audit_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_rollback_store_writer_available",
        durable_append_transaction_authorization_gate.rollback_store_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_transaction_append_writer_available",
        durable_append_transaction_authorization_gate.transaction_append_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_transaction_authorization_gate_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"append_engine_readiness_decision_schema",
        HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"append_engine_readiness_decision_id",
        HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_ID,
    );
    hash_line_str(
        &mut hash,
        b"append_engine_readiness_decision_status",
        append_engine_readiness_decision.status,
    );
    hash_line_str(
        &mut hash,
        b"append_engine_readiness_decision_reason",
        append_engine_readiness_decision.reason,
    );
    hash_line_hash(
        &mut hash,
        b"append_engine_readiness_decision_sha256",
        append_engine_readiness_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"append_engine_readiness_decision_source_authorization_gate_sha256",
        append_engine_readiness_decision.source_authorization_gate_hash,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_readiness_decision_target_range_ready",
        append_engine_readiness_decision.target_range_ready,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_readiness_decision_test_media_write_authority_available",
        append_engine_readiness_decision.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_readiness_decision_append_engine_available",
        append_engine_readiness_decision.append_engine_available,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_readiness_decision_ready",
        append_engine_readiness_decision.ready,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_readiness_decision_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_readiness_decision_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_preflight_target_region_media_write_policy_preflight_schema",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_preflight_target_region_media_write_policy_preflight_status",
        foundation.target_region_media_write_policy_preflight_status,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_preflight_target_region_media_write_policy_preflight_sha256",
        durable_append_preflight
            .target_region_media_write_policy_preflight
            .preflight_hash,
    );
    hash_line_str(
        &mut hash,
        b"media_write_authority_gate_schema",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"media_write_authority_gate_id",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_ID,
    );
    hash_line_str(
        &mut hash,
        b"media_write_authority_gate_status",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"media_write_authority_gate_reason",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"media_write_authority_gate_sha256",
        media_write_authority_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"media_write_authority_gate_source_policy_preflight_sha256",
        media_write_authority_gate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"media_write_authority_gate_source_target_region_write_readback_sha256",
        media_write_authority_gate.source_target_region_write_readback_hash,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_authority_gate_test_media_write_authority_available",
        media_write_authority_gate.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_authority_gate_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_authority_gate_target_region_write_attempted",
        media_write_authority_gate.target_region_write_attempted,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_decision_schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_decision_id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_decision_status",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_decision_reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_decision_sha256",
        durable_append_authority_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_decision_source_append_engine_readiness_decision_sha256",
        durable_append_authority_decision.source_append_engine_readiness_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_decision_source_media_write_authority_gate_sha256",
        durable_append_authority_decision.source_media_write_authority_gate_hash,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_decision_append_engine_ready",
        durable_append_authority_decision.append_engine_ready,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_decision_durable_audit_policy_available",
        durable_append_authority_decision.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_decision_authorizes_append",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_decision_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_decision_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_decision_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_decision_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_decision_sha256",
        durable_audit_policy_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_decision_source_durable_append_authority_decision_sha256",
        durable_audit_policy_decision.source_durable_append_authority_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_decision_source_policy_preflight_sha256",
        durable_audit_policy_decision.source_policy_preflight_hash,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_decision_media_write_policy_verified",
        durable_audit_policy_decision.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_decision_durable_audit_policy_available",
        durable_audit_policy_decision.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_decision_authorizes_append",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_candidate_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_candidate_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_candidate_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_candidate_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_candidate_sha256",
        durable_audit_policy_candidate.candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_candidate_source_durable_audit_policy_decision_sha256",
        durable_audit_policy_candidate.source_durable_audit_policy_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_candidate_source_audit_record_image_sha256",
        durable_audit_policy_candidate.source_audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_candidate_source_policy_preflight_sha256",
        durable_audit_policy_candidate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_candidate_source_target_region_write_readback_sha256",
        durable_audit_policy_candidate.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_candidate_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_candidate_target_start_lba",
        durable_audit_policy_candidate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_candidate_target_lba_count",
        durable_audit_policy_candidate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_candidate_target_byte_count",
        durable_audit_policy_candidate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_media_write_policy_verified",
        durable_audit_policy_candidate.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_available",
        durable_audit_policy_candidate.durable_audit_policy_candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_durable_audit_policy_available",
        durable_audit_policy_candidate.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_durable_append_authority_available",
        durable_audit_policy_candidate.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_sha256",
        durable_audit_policy_acceptance_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_source_candidate_sha256",
        durable_audit_policy_acceptance_gate.source_durable_audit_policy_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_source_decision_sha256",
        durable_audit_policy_acceptance_gate.source_durable_audit_policy_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_source_audit_record_image_sha256",
        durable_audit_policy_acceptance_gate.source_audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_source_policy_preflight_sha256",
        durable_audit_policy_acceptance_gate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_source_target_region_write_readback_sha256",
        durable_audit_policy_acceptance_gate.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_target_start_lba",
        durable_audit_policy_acceptance_gate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_target_lba_count",
        durable_audit_policy_acceptance_gate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_target_byte_count",
        durable_audit_policy_acceptance_gate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_candidate_available",
        durable_audit_policy_acceptance_gate.candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_media_write_policy_verified",
        durable_audit_policy_acceptance_gate.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_durable_policy_ledger_available",
        durable_audit_policy_acceptance_gate.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_write_authority_available",
        durable_audit_policy_acceptance_gate.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_durable_audit_policy_available",
        durable_audit_policy_acceptance_gate.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_durable_append_authority_available",
        durable_audit_policy_acceptance_gate.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_acceptance_gate_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_sha256",
        durable_audit_policy_ledger_candidate.ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_source_acceptance_gate_sha256",
        durable_audit_policy_ledger_candidate.source_acceptance_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_source_candidate_sha256",
        durable_audit_policy_ledger_candidate.source_durable_audit_policy_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_source_decision_sha256",
        durable_audit_policy_ledger_candidate.source_durable_audit_policy_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_source_audit_record_image_sha256",
        durable_audit_policy_ledger_candidate.source_audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_source_policy_preflight_sha256",
        durable_audit_policy_ledger_candidate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_source_target_region_write_readback_sha256",
        durable_audit_policy_ledger_candidate.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_target_start_lba",
        durable_audit_policy_ledger_candidate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_target_lba_count",
        durable_audit_policy_ledger_candidate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_target_byte_count",
        durable_audit_policy_ledger_candidate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_read_only_available",
        durable_audit_policy_ledger_candidate.read_only_ledger_candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_candidate_available",
        durable_audit_policy_ledger_candidate.candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_media_write_policy_verified",
        durable_audit_policy_ledger_candidate.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_durable_policy_ledger_available",
        durable_audit_policy_ledger_candidate.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_write_authority_available",
        durable_audit_policy_ledger_candidate.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_durable_audit_policy_available",
        durable_audit_policy_ledger_candidate.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_durable_append_authority_available",
        durable_audit_policy_ledger_candidate.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_candidate_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_sha256",
        durable_audit_policy_ledger_aware_acceptance_result.result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_source_ledger_candidate_sha256",
        durable_audit_policy_ledger_aware_acceptance_result.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_source_acceptance_gate_sha256",
        durable_audit_policy_ledger_aware_acceptance_result.source_acceptance_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_source_candidate_sha256",
        durable_audit_policy_ledger_aware_acceptance_result
            .source_durable_audit_policy_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_source_decision_sha256",
        durable_audit_policy_ledger_aware_acceptance_result
            .source_durable_audit_policy_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_source_audit_record_image_sha256",
        durable_audit_policy_ledger_aware_acceptance_result.source_audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_source_policy_preflight_sha256",
        durable_audit_policy_ledger_aware_acceptance_result.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_source_target_region_write_readback_sha256",
        durable_audit_policy_ledger_aware_acceptance_result
            .source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_target_start_lba",
        durable_audit_policy_ledger_aware_acceptance_result.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_target_lba_count",
        durable_audit_policy_ledger_aware_acceptance_result.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_target_byte_count",
        durable_audit_policy_ledger_aware_acceptance_result.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_read_only_available",
        durable_audit_policy_ledger_aware_acceptance_result.read_only_ledger_candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_ledger_evidence_verified",
        durable_audit_policy_ledger_aware_acceptance_result.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_write_authority_available",
        durable_audit_policy_ledger_aware_acceptance_result.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_durable_policy_ledger_available",
        durable_audit_policy_ledger_aware_acceptance_result.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_durable_audit_policy_available",
        durable_audit_policy_ledger_aware_acceptance_result.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_durable_append_authority_available",
        durable_audit_policy_ledger_aware_acceptance_result.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_ledger_aware_acceptance_result_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_sha256",
        durable_audit_policy_write_authority_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_source_result_sha256",
        durable_audit_policy_write_authority_availability
            .source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_source_ledger_candidate_sha256",
        durable_audit_policy_write_authority_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_source_policy_preflight_sha256",
        durable_audit_policy_write_authority_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_source_target_region_write_readback_sha256",
        durable_audit_policy_write_authority_availability
            .source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_target_start_lba",
        durable_audit_policy_write_authority_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_target_lba_count",
        durable_audit_policy_write_authority_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_target_byte_count",
        durable_audit_policy_write_authority_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_ledger_evidence_verified",
        durable_audit_policy_write_authority_availability.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_media_write_policy_verified",
        durable_audit_policy_write_authority_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_target_region_write_readback_verified",
        durable_audit_policy_write_authority_availability.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_target_span_verified",
        durable_audit_policy_write_authority_availability.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_audit_rollback_target_ids_verified",
        durable_audit_policy_write_authority_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_test_media_write_authority_available",
        durable_audit_policy_write_authority_availability
            .test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_write_authority_available",
        durable_audit_policy_write_authority_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_durable_policy_ledger_available",
        durable_audit_policy_write_authority_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_durable_audit_policy_available",
        durable_audit_policy_write_authority_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_durable_append_authority_available",
        durable_audit_policy_write_authority_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_write_authority_availability_write_attempted",
        false,
    );
    let durable_policy_ledger_availability = hello_rollback_durable_policy_ledger_availability(
        durable_audit_policy_write_authority_availability,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_schema",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_id",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_status",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_reason",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_sha256",
        durable_policy_ledger_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_source_write_authority_availability_sha256",
        durable_policy_ledger_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_source_result_sha256",
        durable_policy_ledger_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_source_ledger_candidate_sha256",
        durable_policy_ledger_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_source_policy_preflight_sha256",
        durable_policy_ledger_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_source_target_region_write_readback_sha256",
        durable_policy_ledger_availability.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_ledger_availability_target_start_lba",
        durable_policy_ledger_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_ledger_availability_target_lba_count",
        durable_policy_ledger_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_ledger_availability_target_byte_count",
        durable_policy_ledger_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_write_authority_evidence_verified",
        durable_policy_ledger_availability.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_ledger_evidence_verified",
        durable_policy_ledger_availability.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_media_write_policy_verified",
        durable_policy_ledger_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_target_region_write_readback_verified",
        durable_policy_ledger_availability.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_target_span_verified",
        durable_policy_ledger_availability.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_audit_rollback_target_ids_verified",
        durable_policy_ledger_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_test_media_write_authority_available",
        durable_policy_ledger_availability.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_write_authority_available",
        durable_policy_ledger_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_durable_policy_ledger_available",
        durable_policy_ledger_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_durable_audit_policy_available",
        durable_policy_ledger_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_durable_append_authority_available",
        durable_policy_ledger_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_write_attempted",
        false,
    );
    let durable_audit_policy_availability =
        hello_rollback_durable_audit_policy_availability(durable_policy_ledger_availability);
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_sha256",
        durable_audit_policy_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_source_policy_ledger_availability_sha256",
        durable_audit_policy_availability.source_policy_ledger_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_source_write_authority_availability_sha256",
        durable_audit_policy_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_source_result_sha256",
        durable_audit_policy_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_source_ledger_candidate_sha256",
        durable_audit_policy_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_source_policy_preflight_sha256",
        durable_audit_policy_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_source_target_region_write_readback_sha256",
        durable_audit_policy_availability.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_availability_target_start_lba",
        durable_audit_policy_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_availability_target_lba_count",
        durable_audit_policy_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_availability_target_byte_count",
        durable_audit_policy_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_policy_ledger_availability_evidence_verified",
        durable_audit_policy_availability.policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_write_authority_evidence_verified",
        durable_audit_policy_availability.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_ledger_evidence_verified",
        durable_audit_policy_availability.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_media_write_policy_verified",
        durable_audit_policy_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_target_region_write_readback_verified",
        durable_audit_policy_availability.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_target_span_verified",
        durable_audit_policy_availability.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_audit_rollback_target_ids_verified",
        durable_audit_policy_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_test_media_write_authority_available",
        durable_audit_policy_availability.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_write_authority_available",
        durable_audit_policy_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_durable_policy_ledger_available",
        durable_audit_policy_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_durable_audit_policy_available",
        durable_audit_policy_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_durable_append_authority_available",
        durable_audit_policy_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_write_attempted",
        false,
    );
    let durable_append_authority_availability =
        hello_rollback_durable_append_authority_availability(durable_audit_policy_availability);
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_status",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_sha256",
        durable_append_authority_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_source_audit_policy_availability_sha256",
        durable_append_authority_availability.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_source_policy_ledger_availability_sha256",
        durable_append_authority_availability.source_policy_ledger_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_source_write_authority_availability_sha256",
        durable_append_authority_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_source_result_sha256",
        durable_append_authority_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_source_ledger_candidate_sha256",
        durable_append_authority_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_source_policy_preflight_sha256",
        durable_append_authority_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_append_authority_availability_source_target_region_write_readback_sha256",
        durable_append_authority_availability.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_append_authority_availability_rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"durable_append_authority_availability_target_start_lba",
        durable_append_authority_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_append_authority_availability_target_lba_count",
        durable_append_authority_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_append_authority_availability_target_byte_count",
        durable_append_authority_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_audit_policy_availability_evidence_verified",
        durable_append_authority_availability.audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_policy_ledger_availability_evidence_verified",
        durable_append_authority_availability.policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_write_authority_evidence_verified",
        durable_append_authority_availability.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_ledger_evidence_verified",
        durable_append_authority_availability.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_media_write_policy_verified",
        durable_append_authority_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_target_region_write_readback_verified",
        durable_append_authority_availability.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_target_span_verified",
        durable_append_authority_availability.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_audit_rollback_target_ids_verified",
        durable_append_authority_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_test_media_write_authority_available",
        durable_append_authority_availability.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_write_authority_available",
        durable_append_authority_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_durable_policy_ledger_available",
        durable_append_authority_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_durable_audit_policy_available",
        durable_append_authority_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_durable_append_authority_available",
        durable_append_authority_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_write_attempted",
        false,
    );
    let transaction_append_availability_decision =
        hello_rollback_transaction_append_availability_decision(
            durable_append_authority_availability,
            append_engine_readiness_decision,
            durable_writer_policy_preflight,
        );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_schema",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_id",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_status",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_reason",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_availability_decision_sha256",
        transaction_append_availability_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_availability_decision_source_durable_append_authority_availability_sha256",
        transaction_append_availability_decision.source_durable_append_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_availability_decision_source_audit_policy_availability_sha256",
        transaction_append_availability_decision.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_availability_decision_source_append_engine_readiness_decision_sha256",
        transaction_append_availability_decision.source_append_engine_readiness_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_availability_decision_source_writer_policy_preflight_sha256",
        transaction_append_availability_decision.source_writer_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_availability_decision_source_policy_preflight_sha256",
        transaction_append_availability_decision.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_availability_decision_source_target_region_write_readback_sha256",
        transaction_append_availability_decision.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_availability_decision_rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_availability_decision_target_start_lba",
        transaction_append_availability_decision.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_availability_decision_target_lba_count",
        transaction_append_availability_decision.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_availability_decision_target_byte_count",
        transaction_append_availability_decision.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_durable_append_authority_availability_evidence_verified",
        transaction_append_availability_decision
            .durable_append_authority_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_audit_policy_availability_evidence_verified",
        transaction_append_availability_decision.audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_append_engine_ready",
        transaction_append_availability_decision.append_engine_ready,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_writer_policy_ready",
        transaction_append_availability_decision.writer_policy_ready,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_media_write_policy_verified",
        transaction_append_availability_decision.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_target_region_write_readback_verified",
        transaction_append_availability_decision.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_target_span_verified",
        transaction_append_availability_decision.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_audit_rollback_target_ids_verified",
        transaction_append_availability_decision.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_test_media_write_authority_available",
        transaction_append_availability_decision
            .test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_durable_append_authority_available",
        transaction_append_availability_decision.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_durable_audit_policy_available",
        transaction_append_availability_decision.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_transaction_append_available",
        transaction_append_availability_decision.transaction_append_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_authorizes_transaction_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_availability_decision_write_attempted",
        false,
    );
    let transaction_append_authority_denial_gate =
        hello_rollback_transaction_append_authority_denial_gate(
            transaction_append_availability_decision,
        );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_schema",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_id",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_status",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_reason",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_sha256",
        transaction_append_authority_denial_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_source_transaction_append_availability_decision_sha256",
        transaction_append_authority_denial_gate
            .source_transaction_append_availability_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_source_durable_append_authority_availability_sha256",
        transaction_append_authority_denial_gate
            .source_durable_append_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_source_audit_policy_availability_sha256",
        transaction_append_authority_denial_gate.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_source_append_engine_readiness_decision_sha256",
        transaction_append_authority_denial_gate.source_append_engine_readiness_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_source_writer_policy_preflight_sha256",
        transaction_append_authority_denial_gate.source_writer_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_source_policy_preflight_sha256",
        transaction_append_authority_denial_gate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_authority_denial_gate_source_target_region_write_readback_sha256",
        transaction_append_authority_denial_gate.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_authority_denial_gate_rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_authority_denial_gate_target_start_lba",
        transaction_append_authority_denial_gate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_authority_denial_gate_target_lba_count",
        transaction_append_authority_denial_gate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_authority_denial_gate_target_byte_count",
        transaction_append_authority_denial_gate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_availability_decision_evidence_verified",
        transaction_append_authority_denial_gate.availability_decision_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_append_engine_ready",
        transaction_append_authority_denial_gate.append_engine_ready,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_writer_policy_ready",
        transaction_append_authority_denial_gate.writer_policy_ready,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_media_write_policy_verified",
        transaction_append_authority_denial_gate.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_target_region_write_readback_verified",
        transaction_append_authority_denial_gate.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_target_span_verified",
        transaction_append_authority_denial_gate.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_audit_rollback_target_ids_verified",
        transaction_append_authority_denial_gate.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_test_media_write_authority_available",
        transaction_append_authority_denial_gate
            .test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_durable_append_authority_available",
        transaction_append_authority_denial_gate.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_durable_audit_policy_available",
        transaction_append_authority_denial_gate.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_transaction_append_available",
        transaction_append_authority_denial_gate.transaction_append_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_missing_transaction_append_authority",
        transaction_append_authority_denial_gate.missing_transaction_append_authority,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_authorizes_transaction_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_authority_denial_gate_write_attempted",
        false,
    );
    let durable_policy_ledger_availability_dry_run =
        hello_rollback_durable_policy_ledger_availability_dry_run(
            durable_policy_ledger_availability,
            durable_audit_policy_write_authority_availability,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_schema",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_id",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_status",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_reason",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_sha256",
        durable_policy_ledger_availability_dry_run.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_policy_ledger_availability_sha256",
        durable_policy_ledger_availability_dry_run.source_policy_ledger_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_write_authority_availability_sha256",
        durable_policy_ledger_availability_dry_run.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_ledger_aware_acceptance_result_sha256",
        durable_policy_ledger_availability_dry_run.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_ledger_candidate_sha256",
        durable_policy_ledger_availability_dry_run.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_policy_preflight_sha256",
        durable_policy_ledger_availability_dry_run.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_target_region_write_readback_sha256",
        durable_policy_ledger_availability_dry_run.source_target_region_write_readback_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_authority_denial_gate_sha256",
        durable_policy_ledger_availability_dry_run.source_authority_denial_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_source_transaction_append_availability_decision_sha256",
        durable_policy_ledger_availability_dry_run
            .source_transaction_append_availability_decision_hash,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_target_start_lba",
        durable_policy_ledger_availability_dry_run.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_target_lba_count",
        durable_policy_ledger_availability_dry_run.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_target_byte_count",
        durable_policy_ledger_availability_dry_run.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_policy_ledger_availability_evidence_verified",
        durable_policy_ledger_availability_dry_run.policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_write_authority_evidence_verified",
        durable_policy_ledger_availability_dry_run.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_ledger_evidence_verified",
        durable_policy_ledger_availability_dry_run.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_media_write_policy_verified",
        durable_policy_ledger_availability_dry_run.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_target_region_write_readback_verified",
        durable_policy_ledger_availability_dry_run.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_transaction_append_denial_gate_verified",
        durable_policy_ledger_availability_dry_run.transaction_append_denial_gate_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_target_span_verified",
        durable_policy_ledger_availability_dry_run.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_audit_rollback_target_ids_verified",
        durable_policy_ledger_availability_dry_run.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_test_media_write_authority_available",
        durable_policy_ledger_availability_dry_run
            .test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_write_authority_available",
        durable_policy_ledger_availability_dry_run.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_durable_policy_ledger_available",
        durable_policy_ledger_availability_dry_run.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_durable_audit_policy_available",
        durable_policy_ledger_availability_dry_run.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_durable_append_authority_available",
        durable_policy_ledger_availability_dry_run.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_transaction_append_available",
        durable_policy_ledger_availability_dry_run.transaction_append_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_authorizes_transaction_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_write_attempted",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_applies_rollback",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_availability_dry_run_installs_rollback_state",
        false,
    );
    let durable_audit_policy_availability_dry_run =
        hello_rollback_durable_audit_policy_availability_dry_run(
            durable_audit_policy_availability,
            durable_policy_ledger_availability_dry_run,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_sha256",
        durable_audit_policy_availability_dry_run.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_audit_policy_availability_sha256",
        durable_audit_policy_availability_dry_run.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_policy_ledger_availability_dry_run_sha256",
        durable_audit_policy_availability_dry_run.source_policy_ledger_availability_dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_policy_ledger_availability_sha256",
        durable_audit_policy_availability_dry_run.source_policy_ledger_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_write_authority_availability_sha256",
        durable_audit_policy_availability_dry_run.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_ledger_aware_acceptance_result_sha256",
        durable_audit_policy_availability_dry_run.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_ledger_candidate_sha256",
        durable_audit_policy_availability_dry_run.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_policy_preflight_sha256",
        durable_audit_policy_availability_dry_run.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_target_region_write_readback_sha256",
        durable_audit_policy_availability_dry_run.source_target_region_write_readback_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_authority_denial_gate_sha256",
        durable_audit_policy_availability_dry_run.source_authority_denial_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_source_transaction_append_availability_decision_sha256",
        durable_audit_policy_availability_dry_run
            .source_transaction_append_availability_decision_hash,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_target_start_lba",
        durable_audit_policy_availability_dry_run.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_target_lba_count",
        durable_audit_policy_availability_dry_run.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_target_byte_count",
        durable_audit_policy_availability_dry_run.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_audit_policy_availability_evidence_verified",
        durable_audit_policy_availability_dry_run.audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_policy_ledger_dry_run_evidence_verified",
        durable_audit_policy_availability_dry_run.policy_ledger_dry_run_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_policy_ledger_availability_evidence_verified",
        durable_audit_policy_availability_dry_run.policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_write_authority_evidence_verified",
        durable_audit_policy_availability_dry_run.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_ledger_evidence_verified",
        durable_audit_policy_availability_dry_run.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_media_write_policy_verified",
        durable_audit_policy_availability_dry_run.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_target_region_write_readback_verified",
        durable_audit_policy_availability_dry_run.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_transaction_append_denial_gate_verified",
        durable_audit_policy_availability_dry_run.transaction_append_denial_gate_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_target_span_verified",
        durable_audit_policy_availability_dry_run.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_audit_rollback_target_ids_verified",
        durable_audit_policy_availability_dry_run.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_test_media_write_authority_available",
        durable_audit_policy_availability_dry_run
            .test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_write_authority_available",
        durable_audit_policy_availability_dry_run.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_durable_policy_ledger_available",
        durable_audit_policy_availability_dry_run.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_durable_audit_policy_available",
        durable_audit_policy_availability_dry_run.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_durable_append_authority_available",
        durable_audit_policy_availability_dry_run.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_transaction_append_available",
        durable_audit_policy_availability_dry_run.transaction_append_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_authorizes_transaction_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_write_attempted",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_applies_rollback",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_availability_dry_run_installs_rollback_state",
        false,
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
    hash_line_str(
        &mut hash,
        b"transaction_append_dry_run_schema",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_dry_run_id",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_dry_run_status",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"transaction_append_dry_run_reason",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_sha256",
        transaction_append_dry_run.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_source_authority_denial_gate_sha256",
        transaction_append_dry_run.source_authority_denial_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_source_transaction_append_availability_decision_sha256",
        transaction_append_dry_run.source_transaction_append_availability_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_source_append_record_sha256",
        transaction_append_dry_run.source_append_record_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_source_sector_plan_sha256",
        transaction_append_dry_run.source_sector_plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_source_target_region_write_readback_sha256",
        transaction_append_dry_run.source_target_region_write_readback_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_planned_sector_image_sha256",
        transaction_append_dry_run.planned_sector_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"transaction_append_dry_run_readback_sector_image_sha256",
        transaction_append_dry_run.readback_sector_image_hash,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_dry_run_target_start_lba",
        transaction_append_dry_run.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_dry_run_target_lba_count",
        transaction_append_dry_run.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"transaction_append_dry_run_target_byte_count",
        transaction_append_dry_run.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_authority_denial_gate_verified",
        transaction_append_dry_run.authority_denial_gate_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_target_span_verified",
        transaction_append_dry_run.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_target_region_write_readback_verified",
        transaction_append_dry_run.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_append_image_ready",
        transaction_append_dry_run.append_image_ready,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_blocked_by_authority_denial_gate",
        transaction_append_dry_run.blocked_by_authority_denial_gate,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_test_media_write_authority_available",
        transaction_append_dry_run.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_transaction_append_available",
        transaction_append_dry_run.transaction_append_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_authorizes_transaction_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_transaction_append_attempted",
        false,
    );
    let target_region_sector_inspection =
        hello_rollback_target_region_sector_inspection_from_retained_inspect(
            append_record,
            sector_plan,
            target_region_write,
        );
    hash_line_str(
        &mut hash,
        b"target_region_sector_inspection_schema",
        HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"target_region_sector_inspection_id",
        HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_ID,
    );
    hash_line_str(
        &mut hash,
        b"target_region_sector_inspection_status",
        target_region_sector_inspection.status,
    );
    hash_line_str(
        &mut hash,
        b"target_region_sector_inspection_reason",
        target_region_sector_inspection.reason,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_sector_inspection_sha256",
        target_region_sector_inspection.inspection_hash,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_sector_inspection_source_sector_plan_sha256",
        target_region_sector_inspection.source_sector_plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_sector_inspection_source_target_region_write_readback_sha256",
        target_region_sector_inspection.source_target_region_write_readback_hash,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_sector_inspection_sector_image_sha256",
        target_region_sector_inspection.sector_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_sector_inspection_audit_record_image_sha256",
        target_region_sector_inspection.audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_sector_inspection_rollback_transaction_image_sha256",
        target_region_sector_inspection.rollback_transaction_image_hash,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_sector_inspection_verified",
        target_region_sector_inspection.inspection_verified,
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
    hash_line_str(
        &mut hash,
        b"durable_policy_write_authority_decision_schema",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_write_authority_decision_id",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_write_authority_decision_status",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_policy_write_authority_decision_reason",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_sha256",
        durable_policy_write_authority_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_durable_append_authority_availability_dry_run_sha256",
        durable_policy_write_authority_decision
            .source_durable_append_authority_availability_dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_transaction_append_dry_run_sha256",
        durable_policy_write_authority_decision.source_transaction_append_dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_target_region_sector_inspection_sha256",
        durable_policy_write_authority_decision.source_target_region_sector_inspection_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_write_authority_availability_sha256",
        durable_policy_write_authority_decision.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_audit_policy_availability_sha256",
        durable_policy_write_authority_decision.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_durable_append_authority_availability_sha256",
        durable_policy_write_authority_decision
            .source_durable_append_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_authority_denial_gate_sha256",
        durable_policy_write_authority_decision.source_authority_denial_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"durable_policy_write_authority_decision_source_transaction_append_availability_decision_sha256",
        durable_policy_write_authority_decision
            .source_transaction_append_availability_decision_hash,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_write_authority_decision_target_start_lba",
        durable_policy_write_authority_decision.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_write_authority_decision_target_lba_count",
        durable_policy_write_authority_decision.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"durable_policy_write_authority_decision_target_byte_count",
        durable_policy_write_authority_decision.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_transaction_append_dry_run_verified",
        durable_policy_write_authority_decision.transaction_append_dry_run_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_target_region_sector_inspection_verified",
        durable_policy_write_authority_decision.target_region_sector_inspection_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_write_authority_evidence_verified",
        durable_policy_write_authority_decision.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_audit_policy_availability_evidence_verified",
        durable_policy_write_authority_decision.audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_durable_append_authority_availability_evidence_verified",
        durable_policy_write_authority_decision
            .durable_append_authority_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_target_span_verified",
        durable_policy_write_authority_decision.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_test_media_write_authority_available",
        durable_policy_write_authority_decision.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_write_authority_available",
        durable_policy_write_authority_decision.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_durable_policy_ledger_available",
        durable_policy_write_authority_decision.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_durable_audit_policy_available",
        durable_policy_write_authority_decision.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_durable_append_authority_available",
        durable_policy_write_authority_decision.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_transaction_append_available",
        durable_policy_write_authority_decision.transaction_append_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_authorizes_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_authorizes_transaction_append",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_writes_durable_audit_log",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_writes_rollback_store",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_appends_rollback_transaction",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_write_attempted",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_write_authority_decision_applies_rollback",
        false,
    );
    hash_line_str(
        &mut hash,
        b"target_region_write_readback_dry_run_schema",
        HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"target_region_write_readback_dry_run_id",
        HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"target_region_write_readback_dry_run_status",
        target_region_write.status,
    );
    hash_line_str(
        &mut hash,
        b"target_region_write_readback_dry_run_reason",
        target_region_write.reason,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_write_readback_dry_run_sha256",
        target_region_write.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_write_readback_source_policy_preflight_sha256",
        target_region_write.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_write_readback_readback_sha256",
        target_region_write.readback_sector_image_hash,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_matches_planned_image",
        target_region_write.readback_matches_planned_image,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_test_media_write_authority_available",
        target_region_write.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_attempted",
        target_region_write.write_attempted,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_completed",
        target_region_write.write_completed,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_readback_completed",
        target_region_write.readback_completed,
    );
    hash_line_bool(&mut hash, b"target_region_authorizes_media_write", false);
    hash_line_bool(&mut hash, b"target_region_authorizes_append", false);
    hash_line_bool(&mut hash, b"target_region_writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"target_region_writes_rollback_store", false);
    hash_line_bool(
        &mut hash,
        b"target_region_appends_rollback_transaction",
        false,
    );
    hash_line_bool(&mut hash, b"target_region_installs_rollback_state", false);
    hash_line_str(
        &mut hash,
        b"storage_layout_status",
        foundation.storage_layout_status,
    );
    hash_line_str(
        &mut hash,
        b"storage_layout_reason",
        foundation.storage_layout_reason,
    );
    hash_line_bool(
        &mut hash,
        b"storage_layout_available",
        foundation.storage_layout_available,
    );
    hash_line_str(
        &mut hash,
        b"append_engine_status",
        foundation.append_engine_status,
    );
    hash_line_str(
        &mut hash,
        b"append_engine_reason",
        foundation.append_engine_reason,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_available",
        foundation.append_engine_available,
    );
    hash_line_str(
        &mut hash,
        b"append_contract_status",
        foundation.append_contract_status,
    );
    hash_line_str(
        &mut hash,
        b"append_contract_reason",
        foundation.append_contract_reason,
    );
    hash_line_str(
        &mut hash,
        b"rollback_transaction_envelope_status",
        foundation.rollback_transaction_envelope_status,
    );
    hash_line_str(
        &mut hash,
        b"rollback_transaction_envelope_reason",
        foundation.rollback_transaction_envelope_reason,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_transaction_envelope_available",
        foundation.rollback_transaction_envelope_available,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_payload_envelope_gate_sha256",
        rollback_payload_envelope_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"payload_sha256",
        hello_rollback_transaction_payload_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"provenance_sha256",
        hello_rollback_transaction_payload_provenance_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_append_intent_gate_sha256",
        rollback_append_intent_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_write_authority_gate_sha256",
        rollback_write_authority_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_preflight_sha256",
        hello_rollback_transaction_preflight_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_writer_available",
        foundation.transaction_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_store_available",
        foundation.durable_audit_store_available,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_store_available",
        foundation.rollback_store_available,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_transaction_append_available",
        foundation.rollback_transaction_append_available,
    );
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"installs_rollback_plan", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    finalize_sha256(hash)
}

#[allow(dead_code)]
pub(crate) fn hello_rollback_transaction_writer_storage_authority_gate_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    rollback_transaction_writer_storage_authority_gate_hash(snapshot, probation)
}
