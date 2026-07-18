pub const SCOPED_ROLLBACK_APPLY_DECISION_SCHEMA: &str =
    "raios.scoped_rollback_apply_authorization_decision.v0";
pub const SCOPED_ROLLBACK_APPLY_DECISION_ID: &str =
    "scoped_rollback_apply_authorization.current_boot.svc.demo.hello.v0";
pub const SCOPED_ROLLBACK_APPLY_DECISION_MARKER: &str = "RAIOS_ROLLBACK_APPLY_SCOPE_DECISION";
pub const SCOPED_ROLLBACK_AUTHORIZED_APPEND_SCHEMA: &str =
    "raios.scoped_rollback_authorized_append.v0";
pub const SCOPED_ROLLBACK_AUTHORIZED_APPEND_ID: &str =
    "scoped_rollback_authorized_append.current_boot.svc.demo.hello.v0";
pub const SCOPED_ROLLBACK_AUTHORIZED_APPEND_MARKER: &str = "RAIOS_ROLLBACK_AUTHORIZED_APPEND";
pub const ROLLBACK_INSPECT_APPLIED_TRANSACTION_STATUS: &str =
    "rollback_applied_transaction_inspected";
pub const ROLLBACK_INSPECT_APPLIED_TRANSACTION_REASON: &str =
    "applied_rollback_transaction_evidence_retained_current_boot";

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
    pub requested_capability: &'static str,
    pub effects: &'static [&'static str],
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
            requested_capability: "",
            effects: &[],
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
pub struct ScopedRollbackApplyProof {
    requested_capability: &'static str,
    grants: [&'static str; 1],
    effects: &'static [&'static str],
}

impl ScopedRollbackApplyProof {
    pub const fn requested_capability(self) -> &'static str {
        self.requested_capability
    }

    pub const fn effects(self) -> &'static [&'static str] {
        self.effects
    }

    pub const fn grants(&self) -> &[&'static str] {
        &self.grants
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedRollbackApplyDecision {
    pub authorized: bool,
    pub status: &'static str,
    pub reason: &'static str,
    proof: Option<ScopedRollbackApplyProof>,
}

impl ScopedRollbackApplyDecision {
    pub const fn proof(self) -> Option<ScopedRollbackApplyProof> {
        self.proof
    }
}

#[derive(Clone, Copy)]
pub struct ScopedRollbackAuthorizedAppendInput {
    pub requested_capability: &'static str,
    pub effects: &'static [&'static str],
    pub scope_decision_authorized: bool,
    pub scope_decision_hash: Option<[u8; 32]>,
    pub target_start_lba: Option<u64>,
    pub target_lba_count: Option<u64>,
    pub target_byte_count: Option<u64>,
    pub append_record_hash: Option<[u8; 32]>,
    pub sector_plan_hash: Option<[u8; 32]>,
    pub write_readback_hash: Option<[u8; 32]>,
    pub inspection_hash: Option<[u8; 32]>,
    pub write_readback_source_sector_plan_hash: Option<[u8; 32]>,
    pub inspection_source_sector_plan_hash: Option<[u8; 32]>,
    pub inspection_source_write_readback_hash: Option<[u8; 32]>,
    pub sector_plan_sector_image_hash: Option<[u8; 32]>,
    pub planned_sector_image_hash: Option<[u8; 32]>,
    pub readback_sector_image_hash: Option<[u8; 32]>,
    pub expected_sector_image_hash: Option<[u8; 32]>,
    pub inspected_sector_image_hash: Option<[u8; 32]>,
    pub append_record_audit_record_image_hash: Option<[u8; 32]>,
    pub inspected_audit_record_image_hash: Option<[u8; 32]>,
    pub append_record_rollback_transaction_image_hash: Option<[u8; 32]>,
    pub inspected_rollback_transaction_image_hash: Option<[u8; 32]>,
    pub audit_record_offset: Option<u64>,
    pub audit_record_byte_length: Option<u64>,
    pub rollback_transaction_offset: Option<u64>,
    pub rollback_transaction_byte_length: Option<u64>,
    pub padding_offset: Option<u64>,
    pub padding_byte_length: Option<u64>,
    pub write_attempted: bool,
    pub write_completed: bool,
    pub readback_completed: bool,
    pub readback_matches_planned_image: bool,
    pub inspection_read_attempted: bool,
    pub inspection_read_completed: bool,
    pub sector_hash_verified: bool,
    pub audit_record_hash_verified: bool,
    pub rollback_transaction_hash_verified: bool,
    pub offsets_verified: bool,
    pub padding_zeroed: bool,
    pub target_span_verified: bool,
    pub inspection_verified: bool,
}

impl ScopedRollbackAuthorizedAppendInput {
    pub const fn empty() -> Self {
        Self {
            requested_capability: "",
            effects: &[],
            scope_decision_authorized: false,
            scope_decision_hash: None,
            target_start_lba: None,
            target_lba_count: None,
            target_byte_count: None,
            append_record_hash: None,
            sector_plan_hash: None,
            write_readback_hash: None,
            inspection_hash: None,
            write_readback_source_sector_plan_hash: None,
            inspection_source_sector_plan_hash: None,
            inspection_source_write_readback_hash: None,
            sector_plan_sector_image_hash: None,
            planned_sector_image_hash: None,
            readback_sector_image_hash: None,
            expected_sector_image_hash: None,
            inspected_sector_image_hash: None,
            append_record_audit_record_image_hash: None,
            inspected_audit_record_image_hash: None,
            append_record_rollback_transaction_image_hash: None,
            inspected_rollback_transaction_image_hash: None,
            audit_record_offset: None,
            audit_record_byte_length: None,
            rollback_transaction_offset: None,
            rollback_transaction_byte_length: None,
            padding_offset: None,
            padding_byte_length: None,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            readback_matches_planned_image: false,
            inspection_read_attempted: false,
            inspection_read_completed: false,
            sector_hash_verified: false,
            audit_record_hash_verified: false,
            rollback_transaction_hash_verified: false,
            offsets_verified: false,
            padding_zeroed: false,
            target_span_verified: false,
            inspection_verified: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedRollbackAuthorizedAppendProof {
    requested_capability: &'static str,
    grants: [&'static str; 1],
    effects: &'static [&'static str],
}

impl ScopedRollbackAuthorizedAppendProof {
    pub const fn requested_capability(self) -> &'static str {
        self.requested_capability
    }

    pub const fn effects(self) -> &'static [&'static str] {
        self.effects
    }

    pub const fn grants(&self) -> &[&'static str] {
        &self.grants
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedRollbackAuthorizedAppendDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    proof: Option<ScopedRollbackAuthorizedAppendProof>,
}

impl ScopedRollbackAuthorizedAppendDecision {
    pub const fn proof(self) -> Option<ScopedRollbackAuthorizedAppendProof> {
        self.proof
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedRollbackVerifiedApplyProof {
    requested_capability: &'static str,
    grants: [&'static str; 1],
    effects: &'static [&'static str],
}

impl ScopedRollbackVerifiedApplyProof {
    pub const fn requested_capability(self) -> &'static str {
        self.requested_capability
    }

    pub const fn effects(self) -> &'static [&'static str] {
        self.effects
    }

    pub const fn grants(&self) -> &[&'static str] {
        &self.grants
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedRollbackVerifiedApplyDecision {
    pub applied: bool,
    pub status: &'static str,
    pub reason: &'static str,
    proof: Option<ScopedRollbackVerifiedApplyProof>,
}

impl ScopedRollbackVerifiedApplyDecision {
    pub const fn proof(self) -> Option<ScopedRollbackVerifiedApplyProof> {
        self.proof
    }
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
        proof: (!input.requested_capability.is_empty() && !input.effects.is_empty()).then_some(
            ScopedRollbackApplyProof {
                requested_capability: input.requested_capability,
                grants: [input.requested_capability],
                effects: input.effects,
            },
        ),
    }
}

pub fn evaluate_scoped_rollback_authorized_append(
    input: &ScopedRollbackAuthorizedAppendInput,
) -> ScopedRollbackAuthorizedAppendDecision {
    macro_rules! require_hash {
        ($field:ident, $reason:literal) => {
            match input.$field {
                Some(value) => value,
                None => return append_denied($reason),
            }
        };
    }
    macro_rules! require_u64 {
        ($field:ident, $expected:expr, $missing:literal, $mismatch:literal) => {
            match input.$field {
                Some(value) if value == $expected => {}
                Some(_) => return append_denied($mismatch),
                None => return append_denied($missing),
            }
        };
    }

    if !input.scope_decision_authorized {
        return append_denied("scope_decision_not_authorized");
    }
    require_hash!(scope_decision_hash, "missing_scope_decision_hash");
    require_hash!(append_record_hash, "missing_append_record_hash");
    let sector_plan_hash = require_hash!(sector_plan_hash, "missing_sector_plan_hash");
    let write_readback_hash = require_hash!(write_readback_hash, "missing_write_readback_hash");
    require_hash!(inspection_hash, "missing_inspection_hash");
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

    if require_hash!(
        write_readback_source_sector_plan_hash,
        "missing_write_readback_source_sector_plan_hash"
    ) != sector_plan_hash
    {
        return append_denied("write_readback_sector_plan_hash_mismatch");
    }
    if require_hash!(
        inspection_source_sector_plan_hash,
        "missing_inspection_source_sector_plan_hash"
    ) != sector_plan_hash
    {
        return append_denied("inspection_sector_plan_hash_mismatch");
    }
    if require_hash!(
        inspection_source_write_readback_hash,
        "missing_inspection_source_write_readback_hash"
    ) != write_readback_hash
    {
        return append_denied("inspection_write_readback_hash_mismatch");
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
        return append_denied("sector_image_hash_mismatch");
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
        return append_denied("audit_record_image_hash_mismatch");
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
        return append_denied("rollback_transaction_image_hash_mismatch");
    }

    let audit_record_byte_length = match input.audit_record_byte_length {
        Some(value) => value,
        None => return append_denied("missing_audit_record_byte_length"),
    };
    let rollback_transaction_byte_length = match input.rollback_transaction_byte_length {
        Some(value) => value,
        None => return append_denied("missing_rollback_transaction_byte_length"),
    };
    match input.audit_record_offset {
        Some(0) => {}
        Some(_) => return append_denied("audit_record_offset_mismatch"),
        None => return append_denied("missing_audit_record_offset"),
    }
    match input.rollback_transaction_offset {
        Some(value) if value == audit_record_byte_length => {}
        Some(_) => return append_denied("rollback_transaction_offset_mismatch"),
        None => return append_denied("missing_rollback_transaction_offset"),
    }
    let expected_padding_offset =
        audit_record_byte_length.saturating_add(rollback_transaction_byte_length);
    match input.padding_offset {
        Some(value) if value == expected_padding_offset => {}
        Some(_) => return append_denied("padding_offset_mismatch"),
        None => return append_denied("missing_padding_offset"),
    }
    match input.padding_byte_length {
        Some(value)
            if value == EXPECTED_TARGET_BYTE_COUNT.saturating_sub(expected_padding_offset) => {}
        Some(_) => return append_denied("padding_byte_length_mismatch"),
        None => return append_denied("missing_padding_byte_length"),
    }

    if !input.write_attempted {
        return append_denied("write_not_attempted");
    }
    if !input.write_completed {
        return append_denied("write_not_completed");
    }
    if !input.readback_completed {
        return append_denied("readback_not_completed");
    }
    if !input.readback_matches_planned_image {
        return append_denied("readback_hash_mismatch");
    }
    if !input.inspection_read_attempted || !input.inspection_read_completed {
        return append_denied("inspection_read_missing");
    }
    if !input.sector_hash_verified {
        return append_denied("sector_hash_not_verified");
    }
    if !input.audit_record_hash_verified {
        return append_denied("audit_record_hash_not_verified");
    }
    if !input.rollback_transaction_hash_verified {
        return append_denied("rollback_transaction_hash_not_verified");
    }
    if !input.offsets_verified {
        return append_denied("offsets_not_verified");
    }
    if !input.padding_zeroed {
        return append_denied("padding_not_zeroed");
    }
    if !input.target_span_verified {
        return append_denied("target_span_not_verified");
    }
    if !input.inspection_verified {
        return append_denied("inspection_not_verified");
    }

    ScopedRollbackAuthorizedAppendDecision {
        performed: true,
        status: "performed",
        reason: "authorized_lba1_transaction_append_readback_and_inspection_verified",
        proof: (!input.requested_capability.is_empty() && !input.effects.is_empty()).then_some(
            ScopedRollbackAuthorizedAppendProof {
                requested_capability: input.requested_capability,
                grants: [input.requested_capability],
                effects: input.effects,
            },
        ),
    }
}

pub fn evaluate_scoped_rollback_verified_apply(
    append_input: &ScopedRollbackAuthorizedAppendInput,
    authorized_append_hash: Option<[u8; 32]>,
    requested_capability: &'static str,
    effects: &'static [&'static str],
) -> ScopedRollbackVerifiedApplyDecision {
    macro_rules! require_hash {
        ($field:ident, $reason:literal) => {
            match append_input.$field {
                Some(value) => value,
                None => return apply_denied($reason),
            }
        };
    }

    if !append_input.scope_decision_authorized {
        return apply_denied("scope_decision_not_authorized");
    }
    if authorized_append_hash.is_none() {
        return apply_denied("missing_authorized_append_hash");
    }

    let append_decision = evaluate_scoped_rollback_authorized_append(append_input);
    if !append_decision.performed {
        return apply_denied(append_decision.reason);
    }

    require_hash!(write_readback_hash, "missing_write_readback_hash");
    require_hash!(inspection_hash, "missing_inspection_hash");
    let transaction_hash = require_hash!(
        append_record_rollback_transaction_image_hash,
        "missing_append_rollback_transaction_image_hash"
    );
    if require_hash!(
        inspected_rollback_transaction_image_hash,
        "missing_inspected_rollback_transaction_image_hash"
    ) != transaction_hash
    {
        return apply_denied("rollback_transaction_image_hash_mismatch");
    }

    ScopedRollbackVerifiedApplyDecision {
        applied: true,
        status: "current_boot_rollback_applied",
        reason: "verified_authorized_append_readback_and_inspection",
        proof: (!requested_capability.is_empty() && !effects.is_empty()).then_some(
            ScopedRollbackVerifiedApplyProof {
                requested_capability,
                grants: [requested_capability],
                effects,
            },
        ),
    }
}

pub fn applied_rollback_inspect_evidence_retained(
    rollback_transaction_hash: Option<[u8; 32]>,
    write_readback_hash: Option<[u8; 32]>,
    inspection_hash: Option<[u8; 32]>,
    inspected_rollback_transaction_hash: [u8; 32],
    source_target_region_write_readback_hash: [u8; 32],
    retained_inspection_hash: [u8; 32],
) -> bool {
    rollback_transaction_hash == Some(inspected_rollback_transaction_hash)
        && write_readback_hash == Some(source_target_region_write_readback_hash)
        && inspection_hash == Some(retained_inspection_hash)
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
        proof: None,
    }
}

fn append_denied(reason: &'static str) -> ScopedRollbackAuthorizedAppendDecision {
    ScopedRollbackAuthorizedAppendDecision {
        performed: false,
        status: "denied",
        reason,
        proof: None,
    }
}

fn apply_denied(reason: &'static str) -> ScopedRollbackVerifiedApplyDecision {
    ScopedRollbackVerifiedApplyDecision {
        applied: false,
        status: "denied",
        reason,
        proof: None,
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
            requested_capability: "cap.rollback.apply",
            effects: &["current_boot_target_region_lba1_write"],
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

    fn valid_append_input() -> ScopedRollbackAuthorizedAppendInput {
        ScopedRollbackAuthorizedAppendInput {
            requested_capability: "cap.rollback.apply",
            effects: &["durable_transaction_append"],
            scope_decision_authorized: true,
            scope_decision_hash: Some(h(40)),
            target_start_lba: Some(EXPECTED_TARGET_START_LBA),
            target_lba_count: Some(EXPECTED_TARGET_LBA_COUNT),
            target_byte_count: Some(EXPECTED_TARGET_BYTE_COUNT),
            append_record_hash: Some(h(41)),
            sector_plan_hash: Some(h(42)),
            write_readback_hash: Some(h(43)),
            inspection_hash: Some(h(44)),
            write_readback_source_sector_plan_hash: Some(h(42)),
            inspection_source_sector_plan_hash: Some(h(42)),
            inspection_source_write_readback_hash: Some(h(43)),
            sector_plan_sector_image_hash: Some(h(45)),
            planned_sector_image_hash: Some(h(45)),
            readback_sector_image_hash: Some(h(45)),
            expected_sector_image_hash: Some(h(45)),
            inspected_sector_image_hash: Some(h(45)),
            append_record_audit_record_image_hash: Some(h(46)),
            inspected_audit_record_image_hash: Some(h(46)),
            append_record_rollback_transaction_image_hash: Some(h(47)),
            inspected_rollback_transaction_image_hash: Some(h(47)),
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

    fn evaluate_scoped_rollback_verified_apply(
        input: &ScopedRollbackAuthorizedAppendInput,
        authorized_append_hash: Option<[u8; 32]>,
    ) -> ScopedRollbackVerifiedApplyDecision {
        super::evaluate_scoped_rollback_verified_apply(
            input,
            authorized_append_hash,
            "cap.rollback.apply",
            &["applies_rollback", "mutates_service_state"],
        )
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
                reason: "authorized_exact_scoped_rollback_apply_evidence",
                proof: Some(ScopedRollbackApplyProof {
                    requested_capability: "cap.rollback.apply",
                    grants: ["cap.rollback.apply"],
                    effects: &["current_boot_target_region_lba1_write"],
                }),
            }
        );
    }

    #[test]
    fn scoped_apply_proof_certifies_exact_caller_authority_labels() {
        let proof = evaluate_scoped_rollback_apply(&valid_input())
            .proof()
            .unwrap();
        assert_eq!(proof.requested_capability(), "cap.rollback.apply");
        assert_eq!(proof.grants(), &["cap.rollback.apply"]);
        assert_eq!(proof.effects(), &["current_boot_target_region_lba1_write"]);
    }

    #[test]
    fn passed_scope_gate_without_caller_authority_labels_has_no_proof() {
        let mut input = valid_input();
        input.requested_capability = "";
        input.effects = &[];
        let decision = evaluate_scoped_rollback_apply(&input);
        assert!(decision.authorized);
        assert!(decision.proof().is_none());
    }

    #[test]
    fn each_scope_chain_element_independently_prevents_proof() {
        let mutations: &[fn(&mut ScopedRollbackApplyInput<'static>)] = &[
            |i| i.method = None,
            |i| i.service_id = None,
            |i| i.target_region_id = None,
            |i| i.target_region_marker = None,
            |i| i.audit_ledger_target_id = None,
            |i| i.audit_record_schema = None,
            |i| i.rollback_store_target_id = None,
            |i| i.rollback_transaction_schema = None,
            |i| i.target_start_lba = None,
            |i| i.target_lba_count = None,
            |i| i.target_byte_count = None,
            |i| i.probation_status = None,
            |i| i.probation_hash = None,
            |i| i.probation_accepted = false,
            |i| i.rollback_preview_status = None,
            |i| i.rollback_preview_hash = None,
            |i| i.current_state_hash = None,
            |i| i.current_state_counter = None,
            |i| i.probation_new_state_hash = None,
            |i| i.probation_new_state_counter = None,
            |i| i.state_migration_hash = None,
            |i| i.scratch_readiness_verified = false,
            |i| i.append_record_ready = false,
            |i| i.sector_plan_ready = false,
            |i| i.target_region_write_readback_verified = false,
            |i| i.transaction_append_dry_run_verified = false,
            |i| i.target_region_sector_inspection_verified = false,
            |i| i.durable_policy_write_authority_decision_verified = false,
            |i| i.retained_inspect_source_reference_validated = false,
            |i| i.append_record_hash = None,
            |i| i.sector_plan_hash = None,
            |i| i.target_region_write_readback_hash = None,
            |i| i.transaction_append_dry_run_hash = None,
            |i| i.transaction_append_source_sector_plan_hash = None,
            |i| i.transaction_append_source_target_region_write_readback_hash = None,
            |i| i.durable_policy_write_authority_decision_hash = None,
            |i| i.policy_source_transaction_append_dry_run_hash = None,
            |i| i.policy_source_target_region_sector_inspection_hash = None,
            |i| i.target_region_sector_inspection_hash = None,
            |i| i.inspection_source_sector_plan_hash = None,
            |i| i.inspection_source_target_region_write_readback_hash = None,
            |i| i.sector_plan_sector_image_hash = None,
            |i| i.planned_sector_image_hash = None,
            |i| i.readback_sector_image_hash = None,
            |i| i.expected_sector_image_hash = None,
            |i| i.inspected_sector_image_hash = None,
            |i| i.append_record_audit_record_image_hash = None,
            |i| i.inspected_audit_record_image_hash = None,
            |i| i.append_record_rollback_transaction_image_hash = None,
            |i| i.inspected_rollback_transaction_image_hash = None,
            |i| i.retained_inspect_source_reference_hash = None,
            |i| i.retained_inspection_hash = None,
            |i| i.retained_source_sector_plan_hash = None,
            |i| i.retained_source_target_region_write_readback_hash = None,
        ];
        for mutate in mutations {
            let mut input = valid_input();
            mutate(&mut input);
            assert!(evaluate_scoped_rollback_apply(&input).proof().is_none());
        }
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

    #[test]
    fn authorized_append_requires_decision_write_readback_and_inspection() {
        let decision = evaluate_scoped_rollback_authorized_append(&valid_append_input());
        assert_eq!(
            decision,
            ScopedRollbackAuthorizedAppendDecision {
                performed: true,
                status: "performed",
                reason: "authorized_lba1_transaction_append_readback_and_inspection_verified",
                proof: Some(ScopedRollbackAuthorizedAppendProof {
                    requested_capability: "cap.rollback.apply",
                    grants: ["cap.rollback.apply"],
                    effects: &["durable_transaction_append"],
                }),
            }
        );
    }

    #[test]
    fn authorized_append_denies_mismatched_or_missing_evidence() {
        let cases: [(&str, fn(&mut ScopedRollbackAuthorizedAppendInput)); 9] = [
            (
                "scope_decision_not_authorized",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.scope_decision_authorized = false;
                },
            ),
            (
                "target_start_lba_out_of_scope",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.target_start_lba = Some(2);
                },
            ),
            (
                "inspection_write_readback_hash_mismatch",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.inspection_source_write_readback_hash = Some(h(99));
                },
            ),
            (
                "sector_image_hash_mismatch",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.readback_sector_image_hash = Some(h(98));
                },
            ),
            (
                "audit_record_image_hash_mismatch",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.inspected_audit_record_image_hash = Some(h(97));
                },
            ),
            (
                "rollback_transaction_image_hash_mismatch",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.inspected_rollback_transaction_image_hash = Some(h(96));
                },
            ),
            (
                "padding_byte_length_mismatch",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.padding_byte_length = Some(31);
                },
            ),
            (
                "write_not_completed",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.write_completed = false;
                },
            ),
            (
                "inspection_not_verified",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.inspection_verified = false;
                },
            ),
        ];

        for (reason, mutate) in cases {
            let mut input = valid_append_input();
            mutate(&mut input);
            let decision = evaluate_scoped_rollback_authorized_append(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, reason);
            assert!(!decision.performed);
        }
    }

    #[test]
    fn verified_apply_requires_authorized_append_record() {
        let append_input = valid_append_input();
        let decision = evaluate_scoped_rollback_verified_apply(&append_input, Some(h(48)));
        assert_eq!(
            decision,
            ScopedRollbackVerifiedApplyDecision {
                applied: true,
                status: "current_boot_rollback_applied",
                reason: "verified_authorized_append_readback_and_inspection",
                proof: Some(ScopedRollbackVerifiedApplyProof {
                    requested_capability: "cap.rollback.apply",
                    grants: ["cap.rollback.apply"],
                    effects: &["applies_rollback", "mutates_service_state"],
                }),
            }
        );
    }

    #[test]
    fn verified_apply_proof_certifies_exact_caller_authority_labels() {
        let proof = super::evaluate_scoped_rollback_verified_apply(
            &valid_append_input(),
            Some(h(48)),
            "cap.rollback.apply",
            &["applies_rollback", "mutates_service_state"],
        )
        .proof()
        .unwrap();
        assert_eq!(proof.requested_capability(), "cap.rollback.apply");
        assert_eq!(proof.grants(), &["cap.rollback.apply"]);
        assert_eq!(
            proof.effects(),
            &["applies_rollback", "mutates_service_state"]
        );
    }

    #[test]
    fn passed_verified_apply_without_caller_authority_labels_has_no_proof() {
        let decision = super::evaluate_scoped_rollback_verified_apply(
            &valid_append_input(),
            Some(h(48)),
            "",
            &[],
        );
        assert!(decision.applied);
        assert!(decision.proof().is_none());
    }

    #[test]
    fn each_verified_apply_chain_element_independently_prevents_proof() {
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
            let mut input = valid_append_input();
            mutate(&mut input);
            assert!(evaluate_scoped_rollback_verified_apply(&input, Some(h(48)))
                .proof()
                .is_none());
        }
        assert!(super::evaluate_scoped_rollback_verified_apply(
            &valid_append_input(),
            None,
            "cap.rollback.apply",
            &["applies_rollback", "mutates_service_state"],
        )
        .proof()
        .is_none());
    }

    #[test]
    fn scope_and_verified_apply_proofs_are_distinct_types() {
        use core::any::TypeId;

        assert_ne!(
            TypeId::of::<ScopedRollbackApplyProof>(),
            TypeId::of::<ScopedRollbackVerifiedApplyProof>()
        );
    }

    #[test]
    fn verified_apply_denies_missing_or_mismatched_append_chain() {
        let cases: [(
            &str,
            fn(&mut ScopedRollbackAuthorizedAppendInput),
            Option<[u8; 32]>,
        ); 5] = [
            (
                "missing_authorized_append_hash",
                |_input: &mut ScopedRollbackAuthorizedAppendInput| {},
                None,
            ),
            (
                "scope_decision_not_authorized",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.scope_decision_authorized = false;
                },
                Some(h(48)),
            ),
            (
                "readback_hash_mismatch",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.readback_matches_planned_image = false;
                },
                Some(h(48)),
            ),
            (
                "inspection_not_verified",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.inspection_verified = false;
                },
                Some(h(48)),
            ),
            (
                "rollback_transaction_image_hash_mismatch",
                |input: &mut ScopedRollbackAuthorizedAppendInput| {
                    input.inspected_rollback_transaction_image_hash = Some(h(96));
                },
                Some(h(48)),
            ),
        ];

        for (reason, mutate, append_hash) in cases {
            let mut input = valid_append_input();
            mutate(&mut input);
            let decision = evaluate_scoped_rollback_verified_apply(&input, append_hash);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, reason);
            assert!(!decision.applied);
        }
    }

    #[test]
    fn applied_inspect_retention_requires_same_transaction_hashes() {
        assert!(applied_rollback_inspect_evidence_retained(
            Some(h(47)),
            Some(h(43)),
            Some(h(44)),
            h(47),
            h(43),
            h(44)
        ));
        assert!(!applied_rollback_inspect_evidence_retained(
            Some(h(96)),
            Some(h(43)),
            Some(h(44)),
            h(47),
            h(43),
            h(44)
        ));
        assert!(!applied_rollback_inspect_evidence_retained(
            Some(h(47)),
            Some(h(96)),
            Some(h(44)),
            h(47),
            h(43),
            h(44)
        ));
    }
}
