pub const SCOPED_ROLLBACK_APPLY_DECISION_SCHEMA: &str =
    "raios.scoped_rollback_apply_authorization_decision.v0";
pub const SCOPED_ROLLBACK_APPLY_DECISION_ID: &str =
    "scoped_rollback_apply_authorization.current_boot.svc.demo.hello.v0";
pub const SCOPED_ROLLBACK_APPLY_DECISION_MARKER: &str = "RAIOS_ROLLBACK_APPLY_SCOPE_DECISION";

pub const EXPECTED_METHOD: &str = "service.rollback_apply";
pub const EXPECTED_SERVICE_ID: &str = "svc.demo.hello";
pub const EXPECTED_TARGET_REGION_ID: &str = "target_region.audit_rollback.current_boot";
pub const EXPECTED_TARGET_REGION_MARKER: &str = "RAIOS_AUDITRB_V0";
pub const EXPECTED_AUDIT_LEDGER_TARGET_ID: &str = "append.audit_ledger.current_boot";
pub const EXPECTED_AUDIT_RECORD_SCHEMA: &str = "raios.audit_record.v0";
pub const EXPECTED_ROLLBACK_STORE_TARGET_ID: &str = "append.rollback_store.current_boot";
pub const EXPECTED_ROLLBACK_TRANSACTION_SCHEMA: &str = "raios.rollback_transaction.v0";
pub const EXPECTED_TARGET_START_LBA: u64 = 1;
pub const EXPECTED_TARGET_LBA_COUNT: u64 = 1;
pub const EXPECTED_TARGET_BYTE_COUNT: u64 = 512;
pub const EXPECTED_PROBATION_STATUS: &str = "active_current_boot_probation";
pub const EXPECTED_ROLLBACK_PREVIEW_STATUS: &str = "preview_only_current_boot";

#[derive(Clone, Copy)]
pub struct ScopedRollbackApplyInput<'a> {
    pub method: Option<&'a str>,
    pub service_id: Option<&'a str>,
    pub target_region_id: Option<&'a str>,
    pub target_region_marker: Option<&'a str>,
    pub audit_ledger_target_id: Option<&'a str>,
    pub audit_record_schema: Option<&'a str>,
    pub rollback_store_target_id: Option<&'a str>,
    pub rollback_transaction_schema: Option<&'a str>,
    pub target_start_lba: Option<u64>,
    pub target_lba_count: Option<u64>,
    pub target_byte_count: Option<u64>,
    pub probation_status: Option<&'a str>,
    pub probation_hash: Option<[u8; 32]>,
    pub probation_accepted: bool,
    pub rollback_preview_status: Option<&'a str>,
    pub rollback_preview_hash: Option<[u8; 32]>,
    pub current_state_hash: Option<[u8; 32]>,
    pub current_state_counter: Option<u64>,
    pub probation_new_state_hash: Option<[u8; 32]>,
    pub probation_new_state_counter: Option<u64>,
    pub state_migration_hash: Option<[u8; 32]>,
    pub scratch_readiness_verified: bool,
    pub append_record_ready: bool,
    pub sector_plan_ready: bool,
    pub target_region_write_readback_verified: bool,
    pub transaction_append_dry_run_verified: bool,
    pub target_region_sector_inspection_verified: bool,
    pub durable_policy_write_authority_decision_verified: bool,
    pub retained_inspect_source_reference_validated: bool,
    pub append_record_hash: Option<[u8; 32]>,
    pub sector_plan_hash: Option<[u8; 32]>,
    pub target_region_write_readback_hash: Option<[u8; 32]>,
    pub transaction_append_dry_run_hash: Option<[u8; 32]>,
    pub transaction_append_source_sector_plan_hash: Option<[u8; 32]>,
    pub transaction_append_source_target_region_write_readback_hash: Option<[u8; 32]>,
    pub durable_policy_write_authority_decision_hash: Option<[u8; 32]>,
    pub policy_source_transaction_append_dry_run_hash: Option<[u8; 32]>,
    pub policy_source_target_region_sector_inspection_hash: Option<[u8; 32]>,
    pub target_region_sector_inspection_hash: Option<[u8; 32]>,
    pub inspection_source_sector_plan_hash: Option<[u8; 32]>,
    pub inspection_source_target_region_write_readback_hash: Option<[u8; 32]>,
    pub sector_plan_sector_image_hash: Option<[u8; 32]>,
    pub planned_sector_image_hash: Option<[u8; 32]>,
    pub readback_sector_image_hash: Option<[u8; 32]>,
    pub expected_sector_image_hash: Option<[u8; 32]>,
    pub inspected_sector_image_hash: Option<[u8; 32]>,
    pub append_record_audit_record_image_hash: Option<[u8; 32]>,
    pub inspected_audit_record_image_hash: Option<[u8; 32]>,
    pub append_record_rollback_transaction_image_hash: Option<[u8; 32]>,
    pub inspected_rollback_transaction_image_hash: Option<[u8; 32]>,
    pub retained_inspect_source_reference_hash: Option<[u8; 32]>,
    pub retained_inspection_hash: Option<[u8; 32]>,
    pub retained_source_sector_plan_hash: Option<[u8; 32]>,
    pub retained_source_target_region_write_readback_hash: Option<[u8; 32]>,
}

impl<'a> ScopedRollbackApplyInput<'a> {
    pub const fn empty() -> Self {
        Self {
            method: None,
            service_id: None,
            target_region_id: None,
            target_region_marker: None,
            audit_ledger_target_id: None,
            audit_record_schema: None,
            rollback_store_target_id: None,
            rollback_transaction_schema: None,
            target_start_lba: None,
            target_lba_count: None,
            target_byte_count: None,
            probation_status: None,
            probation_hash: None,
            probation_accepted: false,
            rollback_preview_status: None,
            rollback_preview_hash: None,
            current_state_hash: None,
            current_state_counter: None,
            probation_new_state_hash: None,
            probation_new_state_counter: None,
            state_migration_hash: None,
            scratch_readiness_verified: false,
            append_record_ready: false,
            sector_plan_ready: false,
            target_region_write_readback_verified: false,
            transaction_append_dry_run_verified: false,
            target_region_sector_inspection_verified: false,
            durable_policy_write_authority_decision_verified: false,
            retained_inspect_source_reference_validated: false,
            append_record_hash: None,
            sector_plan_hash: None,
            target_region_write_readback_hash: None,
            transaction_append_dry_run_hash: None,
            transaction_append_source_sector_plan_hash: None,
            transaction_append_source_target_region_write_readback_hash: None,
            durable_policy_write_authority_decision_hash: None,
            policy_source_transaction_append_dry_run_hash: None,
            policy_source_target_region_sector_inspection_hash: None,
            target_region_sector_inspection_hash: None,
            inspection_source_sector_plan_hash: None,
            inspection_source_target_region_write_readback_hash: None,
            sector_plan_sector_image_hash: None,
            planned_sector_image_hash: None,
            readback_sector_image_hash: None,
            expected_sector_image_hash: None,
            inspected_sector_image_hash: None,
            append_record_audit_record_image_hash: None,
            inspected_audit_record_image_hash: None,
            append_record_rollback_transaction_image_hash: None,
            inspected_rollback_transaction_image_hash: None,
            retained_inspect_source_reference_hash: None,
            retained_inspection_hash: None,
            retained_source_sector_plan_hash: None,
            retained_source_target_region_write_readback_hash: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedRollbackApplyDecision {
    pub authorized: bool,
    pub status: &'static str,
    pub reason: &'static str,
}

pub fn evaluate_scoped_rollback_apply(
    input: &ScopedRollbackApplyInput<'_>,
) -> ScopedRollbackApplyDecision {
    macro_rules! require_hash {
        ($field:ident, $reason:literal) => {
            match input.$field {
                Some(value) => value,
                None => return denied($reason),
            }
        };
    }
    macro_rules! require_u64 {
        ($field:ident, $expected:expr, $missing:literal, $mismatch:literal) => {
            match input.$field {
                Some(value) if value == $expected => {}
                Some(_) => return denied($mismatch),
                None => return denied($missing),
            }
        };
    }

    if let Err(decision) = require_str(
        input.method,
        EXPECTED_METHOD,
        "missing_method",
        "method_out_of_scope",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.service_id,
        EXPECTED_SERVICE_ID,
        "missing_service_id",
        "service_out_of_scope",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.target_region_id,
        EXPECTED_TARGET_REGION_ID,
        "missing_target_region_id",
        "target_region_out_of_scope",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.target_region_marker,
        EXPECTED_TARGET_REGION_MARKER,
        "missing_target_region_marker",
        "target_region_marker_mismatch",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.audit_ledger_target_id,
        EXPECTED_AUDIT_LEDGER_TARGET_ID,
        "missing_audit_ledger_target_id",
        "audit_ledger_target_out_of_scope",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.audit_record_schema,
        EXPECTED_AUDIT_RECORD_SCHEMA,
        "missing_audit_record_schema",
        "audit_record_schema_mismatch",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.rollback_store_target_id,
        EXPECTED_ROLLBACK_STORE_TARGET_ID,
        "missing_rollback_store_target_id",
        "rollback_store_target_out_of_scope",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.rollback_transaction_schema,
        EXPECTED_ROLLBACK_TRANSACTION_SCHEMA,
        "missing_rollback_transaction_schema",
        "rollback_transaction_schema_mismatch",
    ) {
        return decision;
    }
    require_u64!(
        target_start_lba,
        EXPECTED_TARGET_START_LBA,
        "missing_target_start_lba",
        "target_start_lba_out_of_scope"
    );
    require_u64!(
        target_lba_count,
        EXPECTED_TARGET_LBA_COUNT,
        "missing_target_lba_count",
        "target_lba_count_out_of_scope"
    );
    require_u64!(
        target_byte_count,
        EXPECTED_TARGET_BYTE_COUNT,
        "missing_target_byte_count",
        "target_byte_count_out_of_scope"
    );

    if let Err(decision) = require_str(
        input.probation_status,
        EXPECTED_PROBATION_STATUS,
        "missing_hot_swap_probation",
        "hot_swap_probation_status_mismatch",
    ) {
        return decision;
    }
    if !input.probation_accepted {
        return denied("hot_swap_probation_not_accepted");
    }
    if let Err(decision) = require_str(
        input.rollback_preview_status,
        EXPECTED_ROLLBACK_PREVIEW_STATUS,
        "missing_rollback_preview",
        "rollback_preview_status_mismatch",
    ) {
        return decision;
    }
    require_hash!(probation_hash, "missing_hot_swap_probation_hash");
    require_hash!(rollback_preview_hash, "missing_rollback_preview_hash");
    require_hash!(state_migration_hash, "missing_state_migration_hash");
    let current_state_hash = require_hash!(current_state_hash, "missing_current_state_hash");
    let probation_new_state_hash =
        require_hash!(probation_new_state_hash, "missing_probation_new_state_hash");
    if current_state_hash != probation_new_state_hash {
        return denied("current_state_hash_mismatch");
    }
    let current_state_counter = match input.current_state_counter {
        Some(value) => value,
        None => return denied("missing_current_state_counter"),
    };
    match input.probation_new_state_counter {
        Some(value) if value == current_state_counter => {}
        Some(_) => return denied("current_state_counter_mismatch"),
        None => return denied("missing_probation_new_state_counter"),
    }

    if !input.scratch_readiness_verified {
        return denied("scratch_readiness_missing");
    }
    if !input.append_record_ready {
        return denied("append_record_not_ready");
    }
    if !input.sector_plan_ready {
        return denied("sector_plan_not_ready");
    }
    if !input.target_region_write_readback_verified {
        return denied("target_region_write_readback_missing");
    }
    if !input.transaction_append_dry_run_verified {
        return denied("transaction_append_dry_run_not_verified");
    }
    if !input.target_region_sector_inspection_verified {
        return denied("target_region_sector_inspection_missing");
    }
    if !input.durable_policy_write_authority_decision_verified {
        return denied("durable_policy_write_authority_decision_not_verified");
    }
    if !input.retained_inspect_source_reference_validated {
        return denied("retained_inspect_source_reference_missing");
    }

    let sector_plan_hash = require_hash!(sector_plan_hash, "missing_sector_plan_hash");
    let target_region_write_readback_hash = require_hash!(
        target_region_write_readback_hash,
        "missing_target_region_write_readback_hash"
    );
    let transaction_append_dry_run_hash = require_hash!(
        transaction_append_dry_run_hash,
        "missing_transaction_append_dry_run_hash"
    );
    let target_region_sector_inspection_hash = require_hash!(
        target_region_sector_inspection_hash,
        "missing_target_region_sector_inspection_hash"
    );
    require_hash!(append_record_hash, "missing_append_record_hash");
    require_hash!(
        durable_policy_write_authority_decision_hash,
        "missing_durable_policy_write_authority_decision_hash"
    );
    if require_hash!(
        transaction_append_source_sector_plan_hash,
        "missing_transaction_append_source_sector_plan_hash"
    ) != sector_plan_hash
    {
        return denied("transaction_append_sector_plan_hash_mismatch");
    }
    if require_hash!(
        transaction_append_source_target_region_write_readback_hash,
        "missing_transaction_append_source_target_region_write_readback_hash"
    ) != target_region_write_readback_hash
    {
        return denied("transaction_append_target_region_write_readback_hash_mismatch");
    }
    if require_hash!(
        policy_source_transaction_append_dry_run_hash,
        "missing_policy_source_transaction_append_dry_run_hash"
    ) != transaction_append_dry_run_hash
    {
        return denied("policy_transaction_append_dry_run_hash_mismatch");
    }
    if require_hash!(
        policy_source_target_region_sector_inspection_hash,
        "missing_policy_source_target_region_sector_inspection_hash"
    ) != target_region_sector_inspection_hash
    {
        return denied("policy_target_region_sector_inspection_hash_mismatch");
    }
    if require_hash!(
        inspection_source_sector_plan_hash,
        "missing_inspection_source_sector_plan_hash"
    ) != sector_plan_hash
    {
        return denied("inspection_sector_plan_hash_mismatch");
    }
    if require_hash!(
        inspection_source_target_region_write_readback_hash,
        "missing_inspection_source_target_region_write_readback_hash"
    ) != target_region_write_readback_hash
    {
        return denied("inspection_target_region_write_readback_hash_mismatch");
    }

    let sector_hash = require_hash!(sector_plan_sector_image_hash, "missing_sector_image_hash");
    if require_hash!(
        planned_sector_image_hash,
        "missing_planned_sector_image_hash"
    ) != sector_hash
        || require_hash!(
            readback_sector_image_hash,
            "missing_readback_sector_image_hash"
        ) != sector_hash
        || require_hash!(
            expected_sector_image_hash,
            "missing_expected_sector_image_hash"
        ) != sector_hash
        || require_hash!(
            inspected_sector_image_hash,
            "missing_inspected_sector_image_hash"
        ) != sector_hash
    {
        return denied("sector_image_hash_mismatch");
    }
    let audit_hash = require_hash!(
        append_record_audit_record_image_hash,
        "missing_append_audit_record_image_hash"
    );
    if require_hash!(
        inspected_audit_record_image_hash,
        "missing_inspected_audit_record_image_hash"
    ) != audit_hash
    {
        return denied("audit_record_image_hash_mismatch");
    }
    let rollback_hash = require_hash!(
        append_record_rollback_transaction_image_hash,
        "missing_append_rollback_transaction_image_hash"
    );
    if require_hash!(
        inspected_rollback_transaction_image_hash,
        "missing_inspected_rollback_transaction_image_hash"
    ) != rollback_hash
    {
        return denied("rollback_transaction_image_hash_mismatch");
    }

    if require_hash!(retained_inspection_hash, "missing_retained_inspection_hash")
        != target_region_sector_inspection_hash
    {
        return denied("retained_inspection_hash_mismatch");
    }
    if require_hash!(
        retained_source_sector_plan_hash,
        "missing_retained_source_sector_plan_hash"
    ) != sector_plan_hash
    {
        return denied("retained_sector_plan_hash_mismatch");
    }
    if require_hash!(
        retained_source_target_region_write_readback_hash,
        "missing_retained_source_target_region_write_readback_hash"
    ) != target_region_write_readback_hash
    {
        return denied("retained_target_region_write_readback_hash_mismatch");
    }
    require_hash!(
        retained_inspect_source_reference_hash,
        "missing_retained_inspect_source_reference_hash"
    );

    ScopedRollbackApplyDecision {
        authorized: true,
        status: "authorized",
        reason: "authorized_exact_scoped_rollback_apply_evidence",
    }
}

fn require_str(
    actual: Option<&str>,
    expected: &str,
    missing: &'static str,
    mismatch: &'static str,
) -> Result<(), ScopedRollbackApplyDecision> {
    match actual {
        Some(actual) if actual == expected => Ok(()),
        Some(_) => Err(denied(mismatch)),
        None => Err(denied(missing)),
    }
}

fn denied(reason: &'static str) -> ScopedRollbackApplyDecision {
    ScopedRollbackApplyDecision {
        authorized: false,
        status: "denied",
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn valid_input() -> ScopedRollbackApplyInput<'static> {
        ScopedRollbackApplyInput {
            method: Some(EXPECTED_METHOD),
            service_id: Some(EXPECTED_SERVICE_ID),
            target_region_id: Some(EXPECTED_TARGET_REGION_ID),
            target_region_marker: Some(EXPECTED_TARGET_REGION_MARKER),
            audit_ledger_target_id: Some(EXPECTED_AUDIT_LEDGER_TARGET_ID),
            audit_record_schema: Some(EXPECTED_AUDIT_RECORD_SCHEMA),
            rollback_store_target_id: Some(EXPECTED_ROLLBACK_STORE_TARGET_ID),
            rollback_transaction_schema: Some(EXPECTED_ROLLBACK_TRANSACTION_SCHEMA),
            target_start_lba: Some(EXPECTED_TARGET_START_LBA),
            target_lba_count: Some(EXPECTED_TARGET_LBA_COUNT),
            target_byte_count: Some(EXPECTED_TARGET_BYTE_COUNT),
            probation_status: Some(EXPECTED_PROBATION_STATUS),
            probation_hash: Some(h(1)),
            probation_accepted: true,
            rollback_preview_status: Some(EXPECTED_ROLLBACK_PREVIEW_STATUS),
            rollback_preview_hash: Some(h(2)),
            current_state_hash: Some(h(3)),
            current_state_counter: Some(3),
            probation_new_state_hash: Some(h(3)),
            probation_new_state_counter: Some(3),
            state_migration_hash: Some(h(4)),
            scratch_readiness_verified: true,
            append_record_ready: true,
            sector_plan_ready: true,
            target_region_write_readback_verified: true,
            transaction_append_dry_run_verified: true,
            target_region_sector_inspection_verified: true,
            durable_policy_write_authority_decision_verified: true,
            retained_inspect_source_reference_validated: true,
            append_record_hash: Some(h(5)),
            sector_plan_hash: Some(h(6)),
            target_region_write_readback_hash: Some(h(7)),
            transaction_append_dry_run_hash: Some(h(8)),
            transaction_append_source_sector_plan_hash: Some(h(6)),
            transaction_append_source_target_region_write_readback_hash: Some(h(7)),
            durable_policy_write_authority_decision_hash: Some(h(9)),
            policy_source_transaction_append_dry_run_hash: Some(h(8)),
            policy_source_target_region_sector_inspection_hash: Some(h(10)),
            target_region_sector_inspection_hash: Some(h(10)),
            inspection_source_sector_plan_hash: Some(h(6)),
            inspection_source_target_region_write_readback_hash: Some(h(7)),
            sector_plan_sector_image_hash: Some(h(11)),
            planned_sector_image_hash: Some(h(11)),
            readback_sector_image_hash: Some(h(11)),
            expected_sector_image_hash: Some(h(11)),
            inspected_sector_image_hash: Some(h(11)),
            append_record_audit_record_image_hash: Some(h(12)),
            inspected_audit_record_image_hash: Some(h(12)),
            append_record_rollback_transaction_image_hash: Some(h(13)),
            inspected_rollback_transaction_image_hash: Some(h(13)),
            retained_inspect_source_reference_hash: Some(h(14)),
            retained_inspection_hash: Some(h(10)),
            retained_source_sector_plan_hash: Some(h(6)),
            retained_source_target_region_write_readback_hash: Some(h(7)),
        }
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingMethod,
        WrongMethod,
        WrongTargetLba,
        MissingPreview,
        BadStateHash,
        MissingTargetReadback,
        MissingRetainedInspectReference,
        BadSectorHash,
        BadAuditHash,
        BadRollbackHash,
        BadRetainedInspectionHash,
    }

    fn apply(input: &mut ScopedRollbackApplyInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingMethod => input.method = None,
            Mutation::WrongMethod => input.method = Some("service.start"),
            Mutation::WrongTargetLba => input.target_start_lba = Some(2),
            Mutation::MissingPreview => input.rollback_preview_status = None,
            Mutation::BadStateHash => input.probation_new_state_hash = Some(h(30)),
            Mutation::MissingTargetReadback => input.target_region_write_readback_verified = false,
            Mutation::MissingRetainedInspectReference => {
                input.retained_inspect_source_reference_validated = false
            }
            Mutation::BadSectorHash => input.readback_sector_image_hash = Some(h(31)),
            Mutation::BadAuditHash => input.inspected_audit_record_image_hash = Some(h(32)),
            Mutation::BadRollbackHash => {
                input.inspected_rollback_transaction_image_hash = Some(h(33))
            }
            Mutation::BadRetainedInspectionHash => input.retained_inspection_hash = Some(h(34)),
        }
    }

    #[test]
    fn authorizes_only_exact_scoped_evidence() {
        let decision = evaluate_scoped_rollback_apply(&valid_input());
        assert_eq!(
            decision,
            ScopedRollbackApplyDecision {
                authorized: true,
                status: "authorized",
                reason: "authorized_exact_scoped_rollback_apply_evidence"
            }
        );
    }

    #[test]
    fn denial_truth_table_names_first_failed_input() {
        let cases = [
            (Mutation::MissingMethod, "missing_method"),
            (Mutation::WrongMethod, "method_out_of_scope"),
            (Mutation::WrongTargetLba, "target_start_lba_out_of_scope"),
            (Mutation::MissingPreview, "missing_rollback_preview"),
            (Mutation::BadStateHash, "current_state_hash_mismatch"),
            (
                Mutation::MissingTargetReadback,
                "target_region_write_readback_missing",
            ),
            (
                Mutation::MissingRetainedInspectReference,
                "retained_inspect_source_reference_missing",
            ),
            (Mutation::BadSectorHash, "sector_image_hash_mismatch"),
            (Mutation::BadAuditHash, "audit_record_image_hash_mismatch"),
            (
                Mutation::BadRollbackHash,
                "rollback_transaction_image_hash_mismatch",
            ),
            (
                Mutation::BadRetainedInspectionHash,
                "retained_inspection_hash_mismatch",
            ),
        ];

        for (mutation, reason) in cases {
            let mut input = valid_input();
            apply(&mut input, mutation);
            let decision = evaluate_scoped_rollback_apply(&input);
            assert!(!decision.authorized, "{reason}");
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, reason);
        }
    }
}
