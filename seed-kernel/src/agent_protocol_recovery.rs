use crate::{
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, end_response, json_event_id,
        json_event_id_option, json_opt_str, json_sha256, json_sha256_option, json_str, method_eq,
        method_head_eq, parse_current_boot_event_id, parse_sha256_ref, raw, raw_bool, raw_fmt,
        raw_line,
    },
    event_log, module_evidence, serial,
};

pub(crate) const RECOVERY_ARTIFACT_LOAD_METHOD: &str = "recovery.load_artifact";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_METHOD: &str = "module.load_recovery_artifact";
pub(crate) const RECOVERY_ARTIFACT_LOAD_CAPABILITY: &str = "cap.recovery.load_artifact";
pub(crate) const RECOVERY_ARTIFACT_LOAD_READ_CAPABILITY: &str = "cap.recovery.load_artifact.read";
pub(crate) const RECOVERY_ARTIFACT_LOAD_BINDING_METHOD: &str = "recovery.load_binding";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_METHOD: &str =
    "module.recovery_load_binding";
pub(crate) const RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD: &str =
    "recovery.load_binding_selftest";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD: &str =
    "module.recovery_load_binding_selftest";
const RECOVERY_LOAD_BINDING_SELFTEST_CASES: usize = 14;
const RECOVERY_IDENTITY_SELFTEST_CASES: usize = 6;
const RECOVERY_TRUST_SELFTEST_CASES: usize = 8;
const RECOVERY_VM_TEST_SELFTEST_CASES: usize = 10;
const RECOVERY_LOCAL_APPROVAL_SELFTEST_CASES: usize = 11;
const RECOVERY_LOADER_SELFTEST_CASES: usize = 10;
const RECOVERY_ROLLBACK_EVIDENCE_SELFTEST_CASES: usize = 10;
const RECOVERY_LIFELINE_REQUEST_SELFTEST_CASES: usize = 11;
const RECOVERY_LIFELINE_PROTOCOL_SELFTEST_CASES: usize = 15;
const RECOVERY_LIFELINE_COMMAND_VOCABULARY_SELFTEST_CASES: usize = 16;
const RECOVERY_LOADER_RUNTIME_ISOLATION_SELFTEST_CASES: usize = 27;
const RECOVERY_ROLLBACK_TRANSACTION_ENGINE_SELFTEST_CASES: usize = 38;
const RECOVERY_DURABLE_AUDIT_ROLLBACK_PERSISTENCE_SELFTEST_CASES: usize = 51;
const RECOVERY_MEMORY_PROVENANCE_SELFTEST_CASES: usize = 65;

#[derive(Clone, Copy)]
struct RecoveryIdentityReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    identity_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    expected_identity_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryIdentitySelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryTrustReferenceInput<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    trust_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct RecoveryTrustReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    trust_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    expected_trust_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryTrustSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryVmTestReferenceInput<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    vm_test_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct RecoveryVmTestReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    vm_test_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    expected_vm_test_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryVmTestSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLocalApprovalReferenceInput<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    local_approval_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct RecoveryLocalApprovalReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    local_approval_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
    expected_local_approval_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryLocalApprovalSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLoaderReferenceInput<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    loader_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    retained_local_approval_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    local_approval_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
    loader_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct RecoveryLoaderReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    loader_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    retained_local_approval_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    local_approval_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
    loader_hash: Option<[u8; 32]>,
    expected_loader_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryLoaderSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryRollbackEvidenceReferenceInput<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    rollback_evidence_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    retained_local_approval_reference_event_id: Option<&'a str>,
    retained_loader_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    local_approval_reference_hash: Option<[u8; 32]>,
    loader_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
    loader_hash: Option<[u8; 32]>,
    rollback_evidence_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct RecoveryRollbackEvidenceReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    rollback_evidence_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    retained_local_approval_reference_event_id: Option<&'a str>,
    retained_loader_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    local_approval_reference_hash: Option<[u8; 32]>,
    loader_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
    loader_hash: Option<[u8; 32]>,
    rollback_evidence_hash: Option<[u8; 32]>,
    expected_rollback_evidence_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryRollbackEvidenceSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLifelineRequestReferenceInput<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    lifeline_request_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    retained_local_approval_reference_event_id: Option<&'a str>,
    retained_loader_reference_event_id: Option<&'a str>,
    retained_rollback_evidence_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    local_approval_reference_hash: Option<[u8; 32]>,
    loader_reference_hash: Option<[u8; 32]>,
    rollback_evidence_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
    loader_hash: Option<[u8; 32]>,
    rollback_evidence_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct RecoveryLifelineRequestReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    lifeline_request_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    retained_trust_reference_event_id: Option<&'a str>,
    retained_vm_test_reference_event_id: Option<&'a str>,
    retained_local_approval_reference_event_id: Option<&'a str>,
    retained_loader_reference_event_id: Option<&'a str>,
    retained_rollback_evidence_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    trust_reference_hash: Option<[u8; 32]>,
    vm_test_reference_hash: Option<[u8; 32]>,
    local_approval_reference_hash: Option<[u8; 32]>,
    loader_reference_hash: Option<[u8; 32]>,
    rollback_evidence_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    vm_test_hash: Option<[u8; 32]>,
    local_approval_hash: Option<[u8; 32]>,
    loader_hash: Option<[u8; 32]>,
    rollback_evidence_hash: Option<[u8; 32]>,
    expected_lifeline_request_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryLifelineRequestSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLifelineProtocolCandidate {
    request_retained: bool,
    request_current_boot: bool,
    request_schema_ok: bool,
    request_binding_ok: bool,
    request_binding_reason: &'static str,
    direct_openai_recovery_shortcut_used: bool,
    lifeline_protocol_state_present: bool,
    command_vocabulary_present: bool,
    loader_runtime_isolation_present: bool,
    rollback_transaction_engine_present: bool,
    durable_audit_rollback_persistence_present: bool,
    recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLifelineProtocolCheck {
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    can_report_protocol_gaps: bool,
    authorizes_recovery_load: bool,
    can_move_beyond_denial: bool,
    loads_recovery_loader: bool,
    loads_recovery_artifact: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    allocates_service_slot: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryLifelineProtocolSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLifelineCommandVocabularyCandidate {
    protocol_candidate: RecoveryLifelineProtocolCandidate,
    protocol_state_retained: bool,
    protocol_state_current_boot: bool,
    protocol_state_schema_ok: bool,
    protocol_state_binding_ok: bool,
    protocol_state_binding_reason: &'static str,
    direct_openai_recovery_shortcut_used: bool,
    loader_runtime_isolation_present: bool,
    rollback_transaction_engine_present: bool,
    durable_audit_rollback_persistence_present: bool,
    recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLifelineCommandVocabularyCheck {
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_exposed: bool,
    command_execution_enabled: bool,
    accepts_lifeline_command_envelope: bool,
    authorizes_recovery_load: bool,
    can_move_beyond_denial: bool,
    loads_recovery_loader: bool,
    loads_recovery_artifact: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    allocates_service_slot: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryLifelineCommandVocabularySelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLoaderRuntimeIsolationCandidate {
    command_candidate: RecoveryLifelineCommandVocabularyCandidate,
    command_vocabulary_available: bool,
    command_vocabulary_current_boot: bool,
    command_vocabulary_schema_ok: bool,
    command_vocabulary_binding_ok: bool,
    command_vocabulary_binding_reason: &'static str,
    direct_openai_recovery_shortcut_used: bool,
    loader_address_space_boundary_present: bool,
    loader_entrypoint_abi_present: bool,
    loader_memory_map_constraints_present: bool,
    loader_capability_import_table_present: bool,
    loader_artifact_hash_binding_present: bool,
    loader_provider_separation_present: bool,
    loader_normal_module_separation_present: bool,
    rollback_transaction_engine_present: bool,
    durable_audit_rollback_persistence_present: bool,
    recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLoaderRuntimeIsolationCheck {
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    isolation_requirements_exposed: bool,
    loader_runtime_isolation_ready: bool,
    command_execution_enabled: bool,
    accepts_lifeline_command_envelope: bool,
    authorizes_recovery_load: bool,
    can_move_beyond_denial: bool,
    loads_recovery_loader: bool,
    loads_recovery_artifact: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    allocates_service_slot: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryLoaderRuntimeIsolationSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryRollbackTransactionEngineCandidate {
    loader_candidate: RecoveryLoaderRuntimeIsolationCandidate,
    loader_runtime_isolation_available: bool,
    loader_runtime_isolation_current_boot: bool,
    loader_runtime_isolation_schema_ok: bool,
    loader_runtime_isolation_binding_ok: bool,
    loader_runtime_isolation_binding_reason: &'static str,
    direct_openai_recovery_shortcut_used: bool,
    rollback_target_selection_present: bool,
    rollback_transaction_id_provenance_present: bool,
    rollback_last_good_binding_present: bool,
    rollback_disabled_module_set_binding_present: bool,
    rollback_artifact_hash_binding_present: bool,
    rollback_replay_preconditions_present: bool,
    rollback_recovery_capability_import_present: bool,
    rollback_atomic_apply_abort_semantics_present: bool,
    durable_audit_rollback_persistence_present: bool,
    recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
struct RecoveryRollbackTransactionEngineCheck {
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    transaction_requirements_exposed: bool,
    rollback_transaction_engine_ready: bool,
    rollback_preview_enabled: bool,
    rollback_apply_enabled: bool,
    command_execution_enabled: bool,
    accepts_lifeline_command_envelope: bool,
    authorizes_recovery_load: bool,
    can_move_beyond_denial: bool,
    loads_recovery_loader: bool,
    loads_recovery_artifact: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    allocates_service_slot: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryRollbackTransactionEngineSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryDurableAuditRollbackPersistenceCandidate {
    transaction_candidate: RecoveryRollbackTransactionEngineCandidate,
    rollback_transaction_engine_available: bool,
    rollback_transaction_engine_current_boot: bool,
    rollback_transaction_engine_schema_ok: bool,
    rollback_transaction_engine_binding_ok: bool,
    rollback_transaction_engine_binding_reason: &'static str,
    direct_openai_recovery_shortcut_used: bool,
    persistence_device_inventory_present: bool,
    storage_layout_identity_present: bool,
    audit_append_log_identity_present: bool,
    rollback_store_identity_present: bool,
    transaction_replay_cursor_present: bool,
    last_good_checkpoint_binding_present: bool,
    write_ordering_present: bool,
    crash_consistency_present: bool,
    integrity_root_hash_chain_present: bool,
    recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
struct RecoveryDurableAuditRollbackPersistenceCheck {
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    rollback_transaction_engine_boundary_exposed: bool,
    rollback_transaction_engine_accepted: bool,
    persistence_requirements_exposed: bool,
    durable_audit_rollback_persistence_ready: bool,
    durable_writes_enabled: bool,
    rollback_replay_enabled: bool,
    recovery_memory_writes_enabled: bool,
    rollback_preview_enabled: bool,
    rollback_apply_enabled: bool,
    command_execution_enabled: bool,
    accepts_lifeline_command_envelope: bool,
    authorizes_recovery_load: bool,
    can_move_beyond_denial: bool,
    loads_recovery_loader: bool,
    loads_recovery_artifact: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    allocates_service_slot: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryDurableAuditRollbackPersistenceSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryMemoryProvenanceCandidate {
    persistence_candidate: RecoveryDurableAuditRollbackPersistenceCandidate,
    durable_audit_rollback_persistence_available: bool,
    durable_audit_rollback_persistence_current_boot: bool,
    durable_audit_rollback_persistence_schema_ok: bool,
    durable_audit_rollback_persistence_binding_ok: bool,
    durable_audit_rollback_persistence_binding_reason: &'static str,
    direct_openai_recovery_shortcut_used: bool,
    source_record_ids_present: bool,
    source_schema_hashes_present: bool,
    memory_classification_present: bool,
    memory_authority_level_present: bool,
    memory_rollback_transaction_binding_present: bool,
    memory_last_good_checkpoint_binding_present: bool,
    recovery_only_export_profile_present: bool,
    memory_redaction_state_present: bool,
    memory_replay_window_present: bool,
    memory_audit_linkage_present: bool,
}

#[derive(Clone, Copy)]
struct RecoveryMemoryProvenanceCheck {
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    rollback_transaction_engine_boundary_exposed: bool,
    rollback_transaction_engine_accepted: bool,
    durable_audit_rollback_persistence_boundary_exposed: bool,
    durable_audit_rollback_persistence_accepted: bool,
    memory_provenance_requirements_exposed: bool,
    recovery_memory_provenance_ready: bool,
    memory_writes_enabled: bool,
    provider_export_enabled: bool,
    durable_writes_enabled: bool,
    rollback_replay_enabled: bool,
    recovery_memory_writes_enabled: bool,
    rollback_preview_enabled: bool,
    rollback_apply_enabled: bool,
    command_execution_enabled: bool,
    accepts_lifeline_command_envelope: bool,
    authorizes_recovery_load: bool,
    can_move_beyond_denial: bool,
    loads_recovery_loader: bool,
    loads_recovery_artifact: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    allocates_service_slot: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryMemoryProvenanceSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryEvidenceCandidate {
    retained: bool,
    current_boot: bool,
    schema_ok: bool,
    binding_ok: bool,
    binding_reason: &'static str,
}

#[derive(Clone, Copy)]
struct RecoveryLoadBindingCandidate {
    requested_capability: &'static str,
    identity: RecoveryEvidenceCandidate,
    trust: RecoveryEvidenceCandidate,
    vm_test: RecoveryEvidenceCandidate,
    local_approval: RecoveryEvidenceCandidate,
    loader: RecoveryEvidenceCandidate,
    rollback_evidence: RecoveryEvidenceCandidate,
    normal_module_capability_substituted: bool,
    normal_module_append_intent_substituted: bool,
    append_payload_hash_claimed_authority: bool,
    normal_module_writer_facts_substituted: bool,
    normal_module_service_slot_substituted: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLoadBindingCheck {
    status: &'static str,
    reason: &'static str,
    recovery_only_capability_used: bool,
    accepts_normal_module_authority: bool,
    append_payload_hash_authority: bool,
    can_move_beyond_denial: bool,
    loads_recovery_artifact: bool,
    loads_normal_module: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryLoadBindingSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

pub(crate) fn recovery_artifact_load_method(method: &str) -> bool {
    crate::agent_protocol_support::method_eq(method, RECOVERY_ARTIFACT_LOAD_METHOD)
        || crate::agent_protocol_support::method_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_METHOD)
}

pub(crate) fn recovery_artifact_identity_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.identity_diagnostic")
}

pub(crate) fn recovery_artifact_identity_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.identity_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_trust_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.trust_diagnostic")
}

pub(crate) fn recovery_artifact_trust_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.trust_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_vm_test_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.vm_test_diagnostic")
}

pub(crate) fn recovery_artifact_vm_test_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.vm_test_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_local_approval_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.local_approval_diagnostic")
}

pub(crate) fn recovery_artifact_local_approval_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.local_approval_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_loader_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_diagnostic")
}

pub(crate) fn recovery_artifact_loader_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_rollback_evidence_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_evidence_diagnostic")
}

pub(crate) fn recovery_artifact_rollback_evidence_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_evidence_diagnostic_selftest")
}

pub(crate) fn recovery_lifeline_request_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_request_diagnostic")
}

pub(crate) fn recovery_lifeline_request_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_request_diagnostic_selftest")
}

pub(crate) fn recovery_lifeline_protocol_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_protocol_diagnostic")
}

pub(crate) fn recovery_lifeline_protocol_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_protocol_diagnostic_selftest")
}

pub(crate) fn recovery_lifeline_command_vocabulary_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_vocabulary")
        || method_head_eq(method, "recovery.lifeline_command_vocabulary_diagnostic")
}

pub(crate) fn recovery_lifeline_command_vocabulary_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_vocabulary_selftest")
        || method_head_eq(
            method,
            "recovery.lifeline_command_vocabulary_diagnostic_selftest",
        )
}

pub(crate) fn recovery_loader_runtime_isolation_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_runtime_isolation")
        || method_head_eq(method, "recovery.loader_runtime_isolation_diagnostic")
}

pub(crate) fn recovery_loader_runtime_isolation_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_runtime_isolation_selftest")
        || method_head_eq(
            method,
            "recovery.loader_runtime_isolation_diagnostic_selftest",
        )
}

pub(crate) fn recovery_rollback_transaction_engine_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_transaction_engine")
        || method_head_eq(method, "recovery.rollback_transaction_engine_diagnostic")
}

pub(crate) fn recovery_rollback_transaction_engine_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_transaction_engine_selftest")
        || method_head_eq(
            method,
            "recovery.rollback_transaction_engine_diagnostic_selftest",
        )
}

pub(crate) fn recovery_durable_audit_rollback_persistence_method(method: &str) -> bool {
    method_head_eq(method, "recovery.durable_audit_rollback_persistence")
        || method_head_eq(
            method,
            "recovery.durable_audit_rollback_persistence_diagnostic",
        )
}

pub(crate) fn recovery_durable_audit_rollback_persistence_selftest_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.durable_audit_rollback_persistence_selftest",
    ) || method_head_eq(
        method,
        "recovery.durable_audit_rollback_persistence_diagnostic_selftest",
    )
}

pub(crate) fn recovery_memory_provenance_method(method: &str) -> bool {
    method_head_eq(method, "recovery.memory_provenance")
        || method_head_eq(method, "recovery.memory_provenance_diagnostic")
}

pub(crate) fn recovery_memory_provenance_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.memory_provenance_selftest")
        || method_head_eq(method, "recovery.memory_provenance_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_load_binding_method(method: &str) -> bool {
    method_head_eq(method, RECOVERY_ARTIFACT_LOAD_BINDING_METHOD)
        || method_head_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_METHOD)
}

pub(crate) fn recovery_artifact_load_binding_selftest_method(method: &str) -> bool {
    method_head_eq(method, RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD)
        || method_head_eq(
            method,
            MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD,
        )
}

pub(crate) fn canonical_recovery_artifact_load_method(method: &str) -> &'static str {
    if crate::agent_protocol_support::method_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_METHOD) {
        MODULE_RECOVERY_ARTIFACT_LOAD_METHOD
    } else {
        RECOVERY_ARTIFACT_LOAD_METHOD
    }
}

pub(crate) fn emit_recovery_artifact_identity_diagnostic(method: &str) {
    let check = parse_recovery_identity_reference(recovery_identity_diagnostic_arg(method));
    let recorded_event_id = if check.valid {
        recovery_identity_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_identity_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_identity_reference();

    begin_response("recovery.identity_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_identity_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.identity_diagnostic <identity_reference_hash> <artifact_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"identity_schema\": \"raios.recovery_artifact_identity.v0\",");
    raw_line(
        "        \"identity_canonicalization\": \"raios.recovery_artifact_identity.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_identity_reference_object(&check);
    raw_line(",");
    emit_recovery_identity_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"identity_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.identity_diagnostic");
}

pub(crate) fn emit_recovery_artifact_identity_diagnostic_selftest() {
    let cases = recovery_identity_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.identity_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_identity_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_identity_records\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_identity_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.identity_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_trust_diagnostic(method: &str) {
    let check = parse_recovery_trust_reference(recovery_trust_diagnostic_arg(method), true);
    let recorded_event_id = if check.valid {
        recovery_trust_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_trust_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_trust_reference();

    begin_response("recovery.trust_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_trust_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.trust_diagnostic <trust_reference_hash> <retained_identity_event_id> <identity_reference_hash> <artifact_hash> <trust_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"trust_schema\": \"raios.recovery_artifact_trust.v0\",");
    raw_line("        \"trust_canonicalization\": \"raios.recovery_artifact_trust.canonical.v0\"");
    raw_line("      },");
    emit_recovery_trust_reference_object(&check);
    raw_line(",");
    emit_recovery_trust_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"trust_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.trust_diagnostic");
}

pub(crate) fn emit_recovery_artifact_trust_diagnostic_selftest() {
    let cases = recovery_trust_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.trust_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_trust_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_trust_records\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_trust_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.trust_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_vm_test_diagnostic(method: &str) {
    let check = parse_recovery_vm_test_reference(recovery_vm_test_diagnostic_arg(method), true);
    let recorded_event_id = if check.valid {
        recovery_vm_test_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_vm_test_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_vm_test_reference();

    begin_response("recovery.vm_test_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_vm_test_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_vm_test_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.vm_test_diagnostic <vm_test_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <identity_reference_hash> <trust_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"vm_test_schema\": \"raios.recovery_artifact_vm_test.v0\",");
    raw_line(
        "        \"vm_test_canonicalization\": \"raios.recovery_artifact_vm_test.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_vm_test_reference_object(&check);
    raw_line(",");
    emit_recovery_vm_test_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"vm_test_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.vm_test_diagnostic");
}

pub(crate) fn emit_recovery_artifact_vm_test_diagnostic_selftest() {
    let cases = recovery_vm_test_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.vm_test_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_vm_test_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_vm_test_records\": false,");
    raw_line("      \"accepts_vm_test_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_vm_test_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.vm_test_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_local_approval_diagnostic(method: &str) {
    let check = parse_recovery_local_approval_reference(
        recovery_local_approval_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_local_approval_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_local_approval_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_local_approval_reference();

    begin_response("recovery.local_approval_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_local_approval_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_local_approval_text\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.local_approval_diagnostic <local_approval_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"local_approval_schema\": \"raios.recovery_artifact_local_approval.v0\",");
    raw_line(
        "        \"local_approval_canonicalization\": \"raios.recovery_artifact_local_approval.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_local_approval_reference_object(&check);
    raw_line(",");
    emit_recovery_local_approval_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"local_approval_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.local_approval_diagnostic");
}

pub(crate) fn emit_recovery_artifact_local_approval_diagnostic_selftest() {
    let cases = recovery_local_approval_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.local_approval_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_artifact_local_approval_diagnostic_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_local_approval_records\": false,");
    raw_line("      \"accepts_local_approval_text\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_local_approval_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.local_approval_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_loader_diagnostic(method: &str) {
    let check = parse_recovery_loader_reference(recovery_loader_diagnostic_arg(method), true);
    let recorded_event_id = if check.valid {
        recovery_loader_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_loader_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_loader_reference();

    begin_response("recovery.loader_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_loader_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.loader_diagnostic <loader_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"loader_schema\": \"raios.recovery_artifact_loader.v0\",");
    raw_line(
        "        \"loader_canonicalization\": \"raios.recovery_artifact_loader.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_loader_reference_object(&check);
    raw_line(",");
    emit_recovery_loader_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"loader_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.loader_diagnostic");
}

pub(crate) fn emit_recovery_artifact_loader_diagnostic_selftest() {
    let cases = recovery_loader_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.loader_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_loader_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_loader_records\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_loader_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.loader_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_rollback_evidence_diagnostic(method: &str) {
    let check = parse_recovery_rollback_evidence_reference(
        recovery_rollback_evidence_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_rollback_evidence_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_rollback_evidence_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_rollback_evidence_reference();

    begin_response("recovery.rollback_evidence_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_rollback_evidence_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_rollback_evidence_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.rollback_evidence_diagnostic <rollback_evidence_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <retained_loader_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <loader_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> <rollback_evidence_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line(
        "        \"rollback_evidence_schema\": \"raios.recovery_artifact_rollback_evidence.v0\",",
    );
    raw_line(
        "        \"rollback_evidence_canonicalization\": \"raios.recovery_artifact_rollback_evidence.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_rollback_evidence_reference_object(&check);
    raw_line(",");
    emit_recovery_rollback_evidence_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"rollback_evidence_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.rollback_evidence_diagnostic");
}

pub(crate) fn emit_recovery_artifact_rollback_evidence_diagnostic_selftest() {
    let cases = recovery_rollback_evidence_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.rollback_evidence_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_artifact_rollback_evidence_diagnostic_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_evidence_records\": false,");
    raw_line("      \"accepts_rollback_evidence_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_rollback_evidence_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.rollback_evidence_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_request_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_request_reference(
        recovery_lifeline_request_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_request_binding_from_check(&check)
            .map(event_log::record_recovery_lifeline_request_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_lifeline_request_reference();

    begin_response("recovery.lifeline_request_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_lifeline_request_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_request_diagnostic <lifeline_request_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <retained_loader_event_id> <retained_rollback_evidence_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <loader_reference_hash> <rollback_evidence_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> <rollback_evidence_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_request_canonicalization\": \"raios.recovery_lifeline_request.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_request_reference_object(&check);
    raw_line(",");
    emit_recovery_lifeline_request_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"lifeline_request_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.lifeline_request_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_request_diagnostic_selftest() {
    let cases = recovery_lifeline_request_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_request_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_request_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_request_records\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_request_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_request_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_protocol_diagnostic() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let check =
        evaluate_recovery_lifeline_protocol(recovery_lifeline_protocol_candidate_from_retained(
            retained_request,
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained_loader,
            retained_rollback_evidence,
        ));

    begin_response("recovery.lifeline_protocol_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_protocol_state_records\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    raw_line("      \"required_protocol_facts\": {");
    emit_recovery_lifeline_protocol_missing_fact(
        "lifeline_protocol_state",
        "raios.recovery_lifeline_protocol_state.v0",
        "recovery_lifeline_protocol_state_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "lifeline_command_vocabulary",
        "raios.recovery_lifeline_command_vocabulary.v0",
        "recovery_lifeline_command_vocabulary_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "loader_runtime_isolation",
        "raios.recovery_loader_runtime_isolation.v0",
        "recovery_loader_runtime_isolation_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "rollback_transaction_engine",
        "raios.recovery_rollback_transaction_engine.v0",
        "recovery_rollback_transaction_engine_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_lifeline_protocol_check(&check);
    raw_line("      }");
    end_response("recovery.lifeline_protocol_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_protocol_diagnostic_selftest() {
    let cases = recovery_lifeline_protocol_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_protocol_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_protocol_state_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_protocol_state_records\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_protocol_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_protocol_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_vocabulary() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let check = evaluate_recovery_lifeline_command_vocabulary(
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate),
    );

    begin_response("recovery.lifeline_command_vocabulary");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_vocabulary_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    raw_line("      \"required_execution_facts\": {");
    emit_recovery_lifeline_protocol_missing_fact(
        "lifeline_protocol_state",
        "raios.recovery_lifeline_protocol_state.v0",
        "recovery_lifeline_protocol_state_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "loader_runtime_isolation",
        "raios.recovery_loader_runtime_isolation.v0",
        "recovery_loader_runtime_isolation_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "rollback_transaction_engine",
        "raios.recovery_rollback_transaction_engine.v0",
        "recovery_rollback_transaction_engine_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&check, true);
    raw_line("      \"boundary\": {");
    emit_recovery_lifeline_command_vocabulary_check(&check);
    raw_line("      }");
    end_response("recovery.lifeline_command_vocabulary");
}

pub(crate) fn emit_recovery_lifeline_command_vocabulary_selftest() {
    let cases = recovery_lifeline_command_vocabulary_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_vocabulary_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_vocabulary_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_vocabulary_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_vocabulary_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_vocabulary_selftest");
}

pub(crate) fn emit_recovery_loader_runtime_isolation() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);

    begin_response("recovery.loader_runtime_isolation");
    raw_line("      \"schema\": \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_loader_runtime_isolation_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    emit_recovery_loader_runtime_isolation_input_state(&isolation_candidate, &check, true);
    raw_line("      \"required_isolation_facts\": {");
    emit_recovery_loader_runtime_isolation_fact(
        "loader_address_space_boundary",
        "raios.recovery_loader_address_space_boundary.v0",
        isolation_candidate.loader_address_space_boundary_present,
        "recovery_loader_address_space_boundary_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_entrypoint_abi",
        "raios.recovery_loader_entrypoint_abi.v0",
        isolation_candidate.loader_entrypoint_abi_present,
        "recovery_loader_entrypoint_abi_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_memory_map_constraints",
        "raios.recovery_loader_memory_map_constraints.v0",
        isolation_candidate.loader_memory_map_constraints_present,
        "recovery_loader_memory_map_constraints_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_capability_import_table",
        "raios.recovery_loader_capability_import_table.v0",
        isolation_candidate.loader_capability_import_table_present,
        "recovery_loader_capability_import_table_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_artifact_hash_binding",
        "raios.recovery_loader_artifact_hash_binding.v0",
        isolation_candidate.loader_artifact_hash_binding_present,
        "recovery_loader_artifact_hash_binding_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_provider_separation",
        "raios.recovery_loader_provider_separation.v0",
        isolation_candidate.loader_provider_separation_present,
        "recovery_loader_provider_separation_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_normal_module_separation",
        "raios.recovery_loader_normal_module_separation.v0",
        isolation_candidate.loader_normal_module_separation_present,
        "recovery_loader_normal_module_separation_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"required_downstream_facts\": {");
    emit_recovery_loader_runtime_isolation_fact(
        "rollback_transaction_engine",
        "raios.recovery_rollback_transaction_engine.v0",
        isolation_candidate.rollback_transaction_engine_present,
        "recovery_rollback_transaction_engine_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        isolation_candidate.durable_audit_rollback_persistence_present,
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        isolation_candidate.recovery_memory_provenance_present,
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"isolation_boundary\": {");
    emit_recovery_loader_runtime_isolation_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_loader_runtime_isolation_check(&check);
    raw_line("      }");
    end_response("recovery.loader_runtime_isolation");
}

pub(crate) fn emit_recovery_loader_runtime_isolation_selftest() {
    let cases = recovery_loader_runtime_isolation_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.loader_runtime_isolation_selftest");
    raw_line("      \"schema\": \"raios.recovery_loader_runtime_isolation_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_loader_runtime_isolation_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_loader_runtime_isolation_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"loader_execution_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.loader_runtime_isolation_selftest");
}

pub(crate) fn emit_recovery_rollback_transaction_engine() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let isolation_check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);
    let transaction_candidate =
        recovery_rollback_transaction_engine_candidate_from_loader(isolation_candidate);
    let check = evaluate_recovery_rollback_transaction_engine(transaction_candidate);

    begin_response("recovery.rollback_transaction_engine");
    raw_line("      \"schema\": \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_transaction_engine_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"rollback_read_capability\": \"cap.recovery.rollback.read\",");
    raw_line("        \"rollback_apply_capability\": \"cap.recovery.rollback\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\",",
    );
    raw_line(
        "        \"rollback_transaction_engine_schema\": \"raios.recovery_rollback_transaction_engine.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    raw_line("      \"loader_runtime_isolation\": {");
    emit_recovery_loader_runtime_isolation_boundary(&isolation_check);
    raw_line("      },");
    emit_recovery_rollback_transaction_engine_input_state(
        &transaction_candidate,
        &isolation_check,
        &check,
        true,
    );
    raw_line("      \"required_transaction_facts\": {");
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_target_selection",
        "raios.recovery_rollback_target_selection.v0",
        transaction_candidate.rollback_target_selection_present,
        "recovery_rollback_target_selection_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_transaction_id_provenance",
        "raios.recovery_rollback_transaction_provenance.v0",
        transaction_candidate.rollback_transaction_id_provenance_present,
        "recovery_rollback_transaction_id_provenance_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_last_good_binding",
        "raios.recovery_rollback_last_good_binding.v0",
        transaction_candidate.rollback_last_good_binding_present,
        "recovery_rollback_last_good_binding_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_disabled_module_set_binding",
        "raios.recovery_rollback_disabled_module_set_binding.v0",
        transaction_candidate.rollback_disabled_module_set_binding_present,
        "recovery_rollback_disabled_module_set_binding_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_artifact_hash_binding",
        "raios.recovery_rollback_artifact_hash_binding.v0",
        transaction_candidate.rollback_artifact_hash_binding_present,
        "recovery_rollback_artifact_hash_binding_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_replay_preconditions",
        "raios.recovery_rollback_replay_preconditions.v0",
        transaction_candidate.rollback_replay_preconditions_present,
        "recovery_rollback_replay_preconditions_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_recovery_capability_import",
        "raios.recovery_rollback_recovery_capability_import.v0",
        transaction_candidate.rollback_recovery_capability_import_present,
        "recovery_rollback_recovery_capability_import_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_atomic_apply_abort_semantics",
        "raios.recovery_rollback_atomic_apply_abort_semantics.v0",
        transaction_candidate.rollback_atomic_apply_abort_semantics_present,
        "recovery_rollback_atomic_apply_abort_semantics_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"required_downstream_facts\": {");
    emit_recovery_rollback_transaction_engine_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        transaction_candidate.durable_audit_rollback_persistence_present,
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        transaction_candidate.recovery_memory_provenance_present,
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"transaction_boundary\": {");
    emit_recovery_rollback_transaction_engine_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_rollback_transaction_engine_check(&check);
    raw_line("      }");
    end_response("recovery.rollback_transaction_engine");
}

pub(crate) fn emit_recovery_rollback_transaction_engine_selftest() {
    let cases = recovery_rollback_transaction_engine_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.rollback_transaction_engine_selftest");
    raw_line("      \"schema\": \"raios.recovery_rollback_transaction_engine_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_transaction_engine_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_rollback_transaction_engine_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"rollback_execution_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.rollback_transaction_engine_selftest");
}

pub(crate) fn emit_recovery_durable_audit_rollback_persistence() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let isolation_check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);
    let transaction_candidate =
        recovery_rollback_transaction_engine_candidate_from_loader(isolation_candidate);
    let transaction_check = evaluate_recovery_rollback_transaction_engine(transaction_candidate);
    let persistence_candidate =
        recovery_durable_audit_rollback_persistence_candidate_from_transaction(
            transaction_candidate,
        );
    let check = evaluate_recovery_durable_audit_rollback_persistence(persistence_candidate);

    begin_response("recovery.durable_audit_rollback_persistence");
    raw_line("      \"schema\": \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_durable_audit_rollback_persistence_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_persistence_device_inventory_json\": false,");
    raw_line("      \"accepts_storage_layout_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"updates_last_good_checkpoint\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"persistence_read_capability\": \"cap.recovery.persistence.read\",");
    raw_line("        \"persistence_write_capability\": \"cap.recovery.persistence\",");
    raw_line("        \"rollback_read_capability\": \"cap.recovery.rollback.read\",");
    raw_line("        \"rollback_apply_capability\": \"cap.recovery.rollback\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\",",
    );
    raw_line(
        "        \"rollback_transaction_engine_schema\": \"raios.recovery_rollback_transaction_engine.v0\",",
    );
    raw_line(
        "        \"durable_audit_rollback_persistence_schema\": \"raios.durable_audit_rollback_persistence.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    raw_line("      \"loader_runtime_isolation\": {");
    emit_recovery_loader_runtime_isolation_boundary(&isolation_check);
    raw_line("      },");
    raw_line("      \"rollback_transaction_engine\": {");
    emit_recovery_rollback_transaction_engine_boundary(&transaction_check);
    raw_line("      },");
    emit_recovery_durable_audit_rollback_persistence_input_state(
        &persistence_candidate,
        &transaction_check,
        &check,
        true,
    );
    raw_line("      \"required_persistence_facts\": {");
    emit_recovery_durable_audit_rollback_persistence_fact(
        "persistence_device_inventory",
        "raios.persistence_device_inventory.v0",
        persistence_candidate.persistence_device_inventory_present,
        "persistence_device_inventory_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "storage_layout_identity",
        "raios.durable_audit_rollback_storage_layout_identity.v0",
        persistence_candidate.storage_layout_identity_present,
        "durable_storage_layout_identity_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "audit_append_log_identity",
        "raios.durable_audit_append_log_identity.v0",
        persistence_candidate.audit_append_log_identity_present,
        "durable_audit_append_log_identity_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "rollback_store_identity",
        "raios.rollback_store_identity.v0",
        persistence_candidate.rollback_store_identity_present,
        "rollback_store_identity_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "transaction_replay_cursor",
        "raios.rollback_transaction_replay_cursor.v0",
        persistence_candidate.transaction_replay_cursor_present,
        "rollback_transaction_replay_cursor_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "last_good_checkpoint_binding",
        "raios.recovery_last_good_checkpoint_binding.v0",
        persistence_candidate.last_good_checkpoint_binding_present,
        "recovery_last_good_checkpoint_binding_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "write_ordering",
        "raios.durable_write_ordering.v0",
        persistence_candidate.write_ordering_present,
        "durable_write_ordering_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "crash_consistency",
        "raios.durable_crash_consistency.v0",
        persistence_candidate.crash_consistency_present,
        "durable_crash_consistency_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "integrity_root_hash_chain",
        "raios.durable_integrity_root_hash_chain.v0",
        persistence_candidate.integrity_root_hash_chain_present,
        "durable_integrity_root_hash_chain_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        persistence_candidate.recovery_memory_provenance_present,
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"persistence_boundary\": {");
    emit_recovery_durable_audit_rollback_persistence_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_durable_audit_rollback_persistence_check(&check);
    raw_line("      }");
    end_response("recovery.durable_audit_rollback_persistence");
}

pub(crate) fn emit_recovery_durable_audit_rollback_persistence_selftest() {
    let cases = recovery_durable_audit_rollback_persistence_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.durable_audit_rollback_persistence_selftest");
    raw_line("      \"schema\": \"raios.durable_audit_rollback_persistence_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_durable_audit_rollback_persistence_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_persistence_device_inventory_json\": false,");
    raw_line("      \"accepts_storage_layout_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"recovery_memory_writes_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_durable_audit_rollback_persistence_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"durable_write_enabled\": false,");
    raw_line("      \"rollback_execution_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.durable_audit_rollback_persistence_selftest");
}

pub(crate) fn emit_recovery_memory_provenance() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let isolation_check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);
    let transaction_candidate =
        recovery_rollback_transaction_engine_candidate_from_loader(isolation_candidate);
    let transaction_check = evaluate_recovery_rollback_transaction_engine(transaction_candidate);
    let persistence_candidate =
        recovery_durable_audit_rollback_persistence_candidate_from_transaction(
            transaction_candidate,
        );
    let persistence_check =
        evaluate_recovery_durable_audit_rollback_persistence(persistence_candidate);
    let memory_candidate =
        recovery_memory_provenance_candidate_from_persistence(persistence_candidate);
    let check = evaluate_recovery_memory_provenance(memory_candidate);

    begin_response("recovery.memory_provenance");
    raw_line("      \"schema\": \"raios.recovery_memory_provenance.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_memory_provenance_records\": false,");
    raw_line("      \"accepts_memory_record_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"accepts_provider_context_export\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_persistence_device_inventory_json\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"memory_read_capability\": \"cap.recovery.memory.read\",");
    raw_line("        \"memory_write_capability\": \"cap.recovery.memory\",");
    raw_line("        \"persistence_read_capability\": \"cap.recovery.persistence.read\",");
    raw_line("        \"rollback_read_capability\": \"cap.recovery.rollback.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_memory\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\",",
    );
    raw_line(
        "        \"rollback_transaction_engine_schema\": \"raios.recovery_rollback_transaction_engine.v0\",",
    );
    raw_line(
        "        \"durable_audit_rollback_persistence_schema\": \"raios.durable_audit_rollback_persistence.v0\",",
    );
    raw_line("        \"memory_source_record_ids_schema\": \"raios.recovery_memory_source_record_ids.v0\",");
    raw_line("        \"memory_source_schema_hashes_schema\": \"raios.recovery_memory_source_schema_hashes.v0\",");
    raw_line(
        "        \"memory_classification_schema\": \"raios.recovery_memory_classification.v0\",",
    );
    raw_line(
        "        \"memory_authority_level_schema\": \"raios.recovery_memory_authority_level.v0\",",
    );
    raw_line("        \"memory_rollback_transaction_binding_schema\": \"raios.recovery_memory_rollback_transaction_binding.v0\",");
    raw_line("        \"memory_last_good_checkpoint_binding_schema\": \"raios.recovery_memory_last_good_checkpoint_binding.v0\",");
    raw_line(
        "        \"memory_export_profile_schema\": \"raios.recovery_memory_export_profile.v0\",",
    );
    raw_line(
        "        \"memory_redaction_state_schema\": \"raios.recovery_memory_redaction_state.v0\",",
    );
    raw_line(
        "        \"memory_replay_window_schema\": \"raios.recovery_memory_replay_window.v0\",",
    );
    raw_line("        \"memory_audit_linkage_schema\": \"raios.recovery_memory_audit_linkage.v0\"");
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    raw_line("      \"loader_runtime_isolation\": {");
    emit_recovery_loader_runtime_isolation_boundary(&isolation_check);
    raw_line("      },");
    raw_line("      \"rollback_transaction_engine\": {");
    emit_recovery_rollback_transaction_engine_boundary(&transaction_check);
    raw_line("      },");
    raw_line("      \"durable_audit_rollback_persistence\": {");
    emit_recovery_durable_audit_rollback_persistence_boundary(&persistence_check);
    raw_line("      },");
    emit_recovery_memory_provenance_input_state(
        &memory_candidate,
        &persistence_check,
        &check,
        true,
    );
    raw_line("      \"required_memory_provenance_facts\": {");
    emit_recovery_memory_provenance_fact(
        "source_record_ids",
        "raios.recovery_memory_source_record_ids.v0",
        memory_candidate.source_record_ids_present,
        "recovery_memory_source_record_ids_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "source_schema_hashes",
        "raios.recovery_memory_source_schema_hashes.v0",
        memory_candidate.source_schema_hashes_present,
        "recovery_memory_source_schema_hashes_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_classification",
        "raios.recovery_memory_classification.v0",
        memory_candidate.memory_classification_present,
        "recovery_memory_classification_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_authority_level",
        "raios.recovery_memory_authority_level.v0",
        memory_candidate.memory_authority_level_present,
        "recovery_memory_authority_level_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_rollback_transaction_binding",
        "raios.recovery_memory_rollback_transaction_binding.v0",
        memory_candidate.memory_rollback_transaction_binding_present,
        "recovery_memory_rollback_transaction_binding_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_last_good_checkpoint_binding",
        "raios.recovery_memory_last_good_checkpoint_binding.v0",
        memory_candidate.memory_last_good_checkpoint_binding_present,
        "recovery_memory_last_good_checkpoint_binding_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "recovery_only_export_profile",
        "raios.recovery_memory_export_profile.v0",
        memory_candidate.recovery_only_export_profile_present,
        "recovery_only_export_profile_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_redaction_state",
        "raios.recovery_memory_redaction_state.v0",
        memory_candidate.memory_redaction_state_present,
        "recovery_memory_redaction_state_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_replay_window",
        "raios.recovery_memory_replay_window.v0",
        memory_candidate.memory_replay_window_present,
        "recovery_memory_replay_window_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_audit_linkage",
        "raios.recovery_memory_audit_linkage.v0",
        memory_candidate.memory_audit_linkage_present,
        "recovery_memory_audit_linkage_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"memory_provenance_boundary\": {");
    emit_recovery_memory_provenance_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_memory_provenance_check(&check);
    raw_line("      }");
    end_response("recovery.memory_provenance");
}

pub(crate) fn emit_recovery_memory_provenance_selftest() {
    let cases = recovery_memory_provenance_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.memory_provenance_selftest");
    raw_line("      \"schema\": \"raios.recovery_memory_provenance_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_memory_provenance_records\": false,");
    raw_line("      \"accepts_memory_record_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"accepts_provider_context_export\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"recovery_memory_writes_enabled\": false,");
    raw_line("      \"memory_writes_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_memory_provenance_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"memory_write_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.memory_provenance_selftest");
}

pub(crate) fn emit_recovery_artifact_load_binding() {
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let live = evaluate_recovery_load_binding(recovery_load_binding_candidate_from_retained(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    ));

    begin_response(RECOVERY_ARTIFACT_LOAD_BINDING_METHOD);
    raw_line("      \"schema\": \"raios.recovery_artifact_load_binding.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"status\": \"denied_missing_recovery_binding\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_records\": false,");
    raw_line("      \"request\": {");
    raw("        \"requested_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_CAPABILITY);
    raw_line(",");
    raw("        \"read_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_READ_CAPABILITY);
    raw_line(",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"risk\": \"recovery_modify_ram\",");
    raw_line("        \"target\": \"recovery_lifeline\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"recovery_only_capability_used\": true,");
    raw_line("        \"normal_module_capability_used\": false,");
    raw_line("        \"normal_module_load_path_used\": false,");
    raw_line("        \"separate_from\": \"cap.module.load_ephemeral\"");
    raw_line("      },");
    raw_line("      \"required_retained_evidence_ids\": [");
    raw_line("        \"recovery_artifact_identity_event_id\",");
    raw_line("        \"recovery_artifact_trust_event_id\",");
    raw_line("        \"recovery_vm_test_event_id\",");
    raw_line("        \"recovery_local_approval_event_id\",");
    raw_line("        \"recovery_loader_event_id\",");
    raw_line("        \"recovery_rollback_evidence_event_id\"");
    raw_line("      ],");
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    raw_line("      \"normal_module_authority_rejected\": {");
    raw_line("        \"module_load_ephemeral_facts_used\": false,");
    raw_line("        \"module_append_intent_used\": false,");
    raw_line("        \"module_append_payload_hash_used_as_authority\": false,");
    raw_line("        \"module_writer_facts_used\": false,");
    raw_line("        \"module_service_slot_used\": false,");
    raw_line("        \"normal_module_capability_accepted\": false");
    raw_line("      },");
    raw_line("      \"append_payload_hash_envelopes\": {");
    raw_line("        \"schema\": \"raios.module_audit_rollback_append_payload_hash.v0\",");
    raw_line("        \"authority\": false,");
    raw_line("        \"non_authority_input_only\": true,");
    raw_line("        \"append_payload_hash_authority\": false");
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_load_binding_check(&live, 8, true);
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote_blocker = false;
    if retained_identity.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_identity_event_id",
            "missing",
            "recovery_artifact_identity_event_id_missing",
        );
    }
    if retained_trust.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_trust_event_id",
            "missing",
            "recovery_artifact_trust_event_id_missing",
        );
    } else if let Some(reason) =
        recovery_load_binding_retained_trust_mismatch(retained_identity, retained_trust)
    {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_trust_event_id",
            "rejected",
            reason,
        );
    }
    if retained_vm_test.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_vm_test_event_id",
            "missing",
            "recovery_vm_test_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_vm_test_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_vm_test_event_id",
            "rejected",
            reason,
        );
    }
    if retained_local_approval.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_local_approval_event_id",
            "missing",
            "recovery_local_approval_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_local_approval_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_local_approval_event_id",
            "rejected",
            reason,
        );
    }
    if retained_loader.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_loader_event_id",
            "missing",
            "recovery_loader_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_loader_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_loader_event_id",
            "rejected",
            reason,
        );
    }
    if retained_rollback_evidence.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_rollback_evidence_event_id",
            "missing",
            "recovery_rollback_evidence_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_rollback_evidence_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_rollback_evidence_event_id",
            "rejected",
            reason,
        );
    }
    crlf();
    raw_line("      ]");
    end_response(RECOVERY_ARTIFACT_LOAD_BINDING_METHOD);
}

pub(crate) fn emit_recovery_artifact_load_binding_selftest() {
    let cases = recovery_load_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response(RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD);
    raw_line("      \"schema\": \"raios.recovery_artifact_load_binding_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_records\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("      \"normal_module_capability_accepted\": false,");
    raw_line("      \"append_payload_hash_authority\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_retained_evidence_ids\": [");
    raw_line("        \"recovery_artifact_identity_event_id\",");
    raw_line("        \"recovery_artifact_trust_event_id\",");
    raw_line("        \"recovery_vm_test_event_id\",");
    raw_line("        \"recovery_local_approval_event_id\",");
    raw_line("        \"recovery_loader_event_id\",");
    raw_line("        \"recovery_rollback_evidence_event_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_load_binding_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response(RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD);
}

pub(crate) fn emit_recovery_artifact_load_denied(
    method: &'static str,
    event_id: event_log::EventId,
) {
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw_line("    \"schema\": \"raios.recovery_artifact_load_boundary.v0\",");
    raw("    \"message\": ");
    json_str("recovery artifact loading is denied until recovery-only identity, trust, VM-test, approval, loader, and rollback evidence exist");
    raw_line(",");
    raw_line("    \"request\": {");
    raw_line("      \"load_mode\": \"recovery_only\",");
    raw("      \"requested_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_CAPABILITY);
    raw_line(",");
    raw_line("      \"risk\": \"recovery_modify_ram\",");
    raw_line("      \"target\": \"recovery_lifeline\",");
    raw_line("      \"subject\": \"agent.session.serial\",");
    raw_line("      \"normal_module_load_path_used\": false,");
    raw_line("      \"normal_module_capability_used\": false,");
    raw_line("      \"separate_from\": \"cap.module.load_ephemeral\"");
    raw_line("    },");
    raw_line("    \"boundary\": {");
    raw_line("      \"schema\": \"raios.recovery_artifact_load_denial_evidence.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"status\": \"denied_missing_recovery_artifact_evidence\",");
    raw_line("      \"recovery_artifact_identity\": \"missing\",");
    raw_line("      \"recovery_artifact_trust\": \"missing\",");
    raw_line("      \"recovery_vm_test\": \"missing\",");
    raw_line("      \"recovery_local_approval\": \"missing\",");
    raw_line("      \"recovery_loader\": \"missing\",");
    raw_line("      \"recovery_rollback_evidence\": \"missing\",");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false");
    raw_line("    },");
    raw_line("    \"missing_facts\": {");
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_identity",
        "raios.recovery_artifact_identity.v0",
        "recovery_artifact_identity_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_trust",
        "raios.recovery_artifact_trust.v0",
        "recovery_artifact_trust_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_vm_test",
        "raios.recovery_artifact_vm_test.v0",
        "recovery_vm_test_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_local_approval",
        "raios.recovery_artifact_local_approval.v0",
        "recovery_local_approval_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_loader",
        "raios.recovery_artifact_loader.v0",
        "recovery_loader_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_rollback_evidence",
        "raios.recovery_artifact_rollback_evidence.v0",
        "recovery_rollback_evidence_missing",
        false,
    );
    raw_line("    },");
    raw_line("    \"blocked_by\": [");
    raw_line("      {\"gate\": \"recovery_artifact_identity\", \"state\": \"missing\", \"reason\": \"recovery_artifact_identity_missing\"},");
    raw_line("      {\"gate\": \"recovery_artifact_trust\", \"state\": \"missing\", \"reason\": \"recovery_artifact_trust_missing\"},");
    raw_line("      {\"gate\": \"recovery_vm_test\", \"state\": \"missing\", \"reason\": \"recovery_vm_test_missing\"},");
    raw_line("      {\"gate\": \"recovery_local_approval\", \"state\": \"missing\", \"reason\": \"recovery_local_approval_missing\"},");
    raw_line("      {\"gate\": \"recovery_loader\", \"state\": \"missing\", \"reason\": \"recovery_loader_missing\"},");
    raw_line("      {\"gate\": \"recovery_rollback_evidence\", \"state\": \"missing\", \"reason\": \"recovery_rollback_evidence_missing\"}");
    raw_line("    ],");
    raw_line("    \"required\": [");
    raw_line("      \"raios.recovery_artifact_identity.v0\",");
    raw_line("      \"raios.recovery_artifact_trust.v0\",");
    raw_line("      \"raios.recovery_artifact_vm_test.v0\",");
    raw_line("      \"raios.recovery_artifact_local_approval.v0\",");
    raw_line("      \"raios.recovery_artifact_loader.v0\",");
    raw_line("      \"raios.recovery_artifact_rollback_evidence.v0\"");
    raw_line("    ],");
    raw_line("    \"evidence\": {");
    raw("      \"denial_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"event_scope\": \"current_boot\",");
    raw_line("      \"recovery_only_capability_id\": \"cap.recovery.load_artifact\",");
    raw_line("      \"normal_module_capability_id\": \"cap.module.load_ephemeral\",");
    raw_line("      \"normal_module_append_intent_used\": false,");
    raw_line("      \"append_payload_hash_authority\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"load_attempted\": false");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

pub(crate) fn emit_recovery_artifact_load_denial_event_binding(
    binding: event_log::RecoveryArtifactLoadDenialBinding,
) {
    raw(", \"bindings\": {\"schema\": \"raios.recovery_artifact_load_denial_evidence.v0\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"status\": \"denied_missing_recovery_artifact_evidence\", \"requested_capability\": \"cap.recovery.load_artifact\", \"load_mode\": \"recovery_only\", \"separate_from\": \"cap.module.load_ephemeral\", \"normal_module_load_path_used\": false, \"normal_module_capability_used\": false, \"recovery_artifact_identity\": ");
    json_missing_state(binding.recovery_artifact_identity_missing);
    raw(", \"recovery_artifact_trust\": ");
    json_missing_state(binding.recovery_artifact_trust_missing);
    raw(", \"recovery_vm_test\": ");
    json_missing_state(binding.recovery_vm_test_missing);
    raw(", \"recovery_local_approval\": ");
    json_missing_state(binding.recovery_local_approval_missing);
    raw(", \"recovery_loader\": ");
    json_missing_state(binding.recovery_loader_missing);
    raw(", \"recovery_rollback_evidence\": ");
    json_missing_state(binding.recovery_rollback_evidence_missing);
    raw(", \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"missing_fact_schemas\": [\"raios.recovery_artifact_identity.v0\", \"raios.recovery_artifact_trust.v0\", \"raios.recovery_artifact_vm_test.v0\", \"raios.recovery_artifact_local_approval.v0\", \"raios.recovery_artifact_loader.v0\", \"raios.recovery_artifact_rollback_evidence.v0\"]}");
}

fn emit_recovery_identity_reference_object(check: &RecoveryIdentityReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_identity_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"identity_schema\": \"raios.recovery_artifact_identity.v0\",");
    raw("        \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("        \"expected_identity_reference_hash\": ");
    json_sha256_option(check.expected_identity_reference_hash);
    raw_line(",");
    raw("        \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    crlf();
    raw_line("      }");
}

fn emit_recovery_identity_retained_reference(
    check: &RecoveryIdentityReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_identity_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_identity_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_identity.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw_line("        \"hashes\": {");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_identity.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_identity_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_trust_reference_object(check: &RecoveryTrustReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_trust_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw_line("        \"trust_schema\": \"raios.recovery_artifact_trust.v0\",");
    raw_line("        \"hashes\": {");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"expected_trust_reference_hash\": ");
    json_sha256_option(check.expected_trust_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

fn emit_recovery_trust_retained_reference(
    check: &RecoveryTrustReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_trust_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_trust_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_trust.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_trust.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_trust_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_vm_test_reference_object(check: &RecoveryVmTestReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_vm_test_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw_line("        \"vm_test_schema\": \"raios.recovery_artifact_vm_test.v0\",");
    raw_line("        \"hashes\": {");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"expected_vm_test_reference_hash\": ");
    json_sha256_option(check.expected_vm_test_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

fn emit_recovery_vm_test_retained_reference(
    check: &RecoveryVmTestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_vm_test_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_vm_test_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_vm_test.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_vm_test_json\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_vm_test.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_vm_test_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_local_approval_reference_object(check: &RecoveryLocalApprovalReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_local_approval_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw_line("        \"local_approval_schema\": \"raios.recovery_artifact_local_approval.v0\",");
    raw_line("        \"hashes\": {");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"expected_local_approval_reference_hash\": ");
    json_sha256_option(check.expected_local_approval_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

fn emit_recovery_local_approval_retained_reference(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_local_approval_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_local_approval_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_local_approval.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_local_approval_text\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_local_approval.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line(
            "        \"reason\": \"no_valid_recovery_artifact_local_approval_reference_retained\",",
        );
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_loader_reference_object(check: &RecoveryLoaderReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_loader_reference\": {");
    raw_line("        \"schema\": \"raios.recovery_artifact_loader.v0\",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(check.has_reference);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"hash_reference_only\": true,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
    json_opt_str(check.retained_local_approval_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"loader_reference_hash\": ");
    json_sha256_option(check.loader_reference_hash);
    raw_line(",");
    raw("          \"expected_loader_reference_hash\": ");
    json_sha256_option(check.expected_loader_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    raw_line(",");
    raw("          \"loader_hash\": ");
    json_sha256_option(check.loader_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

fn emit_recovery_loader_retained_reference(
    check: &RecoveryLoaderReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_loader_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_loader_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_loader.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_loader_descriptor\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"loads_recovery_loader\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_loader.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_loader_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_rollback_evidence_reference_object(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
) {
    raw_line("      \"recovery_artifact_rollback_evidence_reference\": {");
    raw_line("        \"schema\": \"raios.recovery_artifact_rollback_evidence.v0\",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(check.has_reference);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"hash_reference_only\": true,");
    raw_line("        \"accepts_rollback_evidence_json\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
    json_opt_str(check.retained_local_approval_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_loader_event_id\": ");
    json_opt_str(check.retained_loader_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"rollback_evidence_reference_hash\": ");
    json_sha256_option(check.rollback_evidence_reference_hash);
    raw_line(",");
    raw("          \"expected_rollback_evidence_reference_hash\": ");
    json_sha256_option(check.expected_rollback_evidence_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"loader_reference_hash\": ");
    json_sha256_option(check.loader_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    raw_line(",");
    raw("          \"loader_hash\": ");
    json_sha256_option(check.loader_hash);
    raw_line(",");
    raw("          \"rollback_evidence_hash\": ");
    json_sha256_option(check.rollback_evidence_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

fn emit_recovery_rollback_evidence_retained_reference(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_rollback_evidence_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_rollback_evidence_reference_matches(
            check, reference,
        ));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_rollback_evidence.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_rollback_evidence_json\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"creates_durable_records\": false,");
        raw_line("        \"installs_rollback_plan\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw_line(",");
        raw("          \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_rollback_evidence.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line(
            "        \"reason\": \"no_valid_recovery_artifact_rollback_evidence_reference_retained\",",
        );
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_lifeline_request_reference_object(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
) {
    raw_line("      \"recovery_lifeline_request_reference\": {");
    raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(check.has_reference);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"hash_reference_only\": true,");
    raw_line("        \"accepts_lifeline_request_json\": false,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
    json_opt_str(check.retained_local_approval_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_loader_event_id\": ");
    json_opt_str(check.retained_loader_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_rollback_evidence_event_id\": ");
    json_opt_str(check.retained_rollback_evidence_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"lifeline_request_reference_hash\": ");
    json_sha256_option(check.lifeline_request_reference_hash);
    raw_line(",");
    raw("          \"expected_lifeline_request_reference_hash\": ");
    json_sha256_option(check.expected_lifeline_request_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"loader_reference_hash\": ");
    json_sha256_option(check.loader_reference_hash);
    raw_line(",");
    raw("          \"rollback_evidence_reference_hash\": ");
    json_sha256_option(check.rollback_evidence_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    raw_line(",");
    raw("          \"loader_hash\": ");
    json_sha256_option(check.loader_hash);
    raw_line(",");
    raw("          \"rollback_evidence_hash\": ");
    json_sha256_option(check.rollback_evidence_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

fn emit_recovery_lifeline_request_retained_reference(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
) {
    raw_line("      \"retained_recovery_lifeline_request_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_lifeline_request_reference_matches(
            check, reference,
        ));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_lifeline_request_json\": false,");
        raw_line("        \"accepts_loader_descriptor\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"loads_recovery_loader\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"creates_durable_records\": false,");
        raw_line("        \"installs_rollback_plan\": false,");
        raw_line("        \"allocates_service_slot\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_rollback_evidence_event_id\": ");
        json_event_id(reference.retained_rollback_evidence_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"lifeline_request_reference_hash\": ");
        json_sha256(reference.lifeline_request_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw_line(",");
        raw("          \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_lifeline_request_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_identity_selftest_case(case: &RecoveryIdentitySelfTestCase, comma: bool) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_trust_selftest_case(case: &RecoveryTrustSelfTestCase, comma: bool) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_vm_test_selftest_case(case: &RecoveryVmTestSelfTestCase, comma: bool) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_local_approval_selftest_case(
    case: &RecoveryLocalApprovalSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_loader_selftest_case(case: &RecoveryLoaderSelfTestCase, comma: bool) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_rollback_evidence_selftest_case(
    case: &RecoveryRollbackEvidenceSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_lifeline_request_selftest_case(
    case: &RecoveryLifelineRequestSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_lifeline_protocol_request_state(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    check: &RecoveryLifelineProtocolCheck,
    comma: bool,
) {
    raw_line("      \"retained_recovery_lifeline_request\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw("        \"status\": ");
        json_str(if check.request_chain_valid {
            "retained_current_boot_hash_reference_only"
        } else {
            "rejected"
        });
        raw_line(",");
        raw("        \"reason\": ");
        json_str(check.reason);
        raw_line(",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"hash_reference_only\": true,");
        raw("        \"request_chain_valid\": ");
        raw_bool(check.request_chain_valid);
        raw_line(",");
        raw("        \"can_report_protocol_gaps\": ");
        raw_bool(check.can_report_protocol_gaps);
        raw_line(",");
        raw_line("        \"accepts_lifeline_request_json\": false,");
        raw_line("        \"accepts_loader_descriptor\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"loads_recovery_loader\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"creates_durable_records\": false,");
        raw_line("        \"installs_rollback_plan\": false,");
        raw_line("        \"allocates_service_slot\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_rollback_evidence_event_id\": ");
        json_event_id(reference.retained_rollback_evidence_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"lifeline_request_reference_hash\": ");
        json_sha256(reference.lifeline_request_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw_line(",");
        raw("          \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"recovery_lifeline_request_event_id_missing\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"request_chain_valid\": false,");
        raw_line("        \"can_report_protocol_gaps\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_lifeline_protocol_missing_fact(
    field: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(reason);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_lifeline_protocol_check(check: &RecoveryLifelineProtocolCheck) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"can_report_protocol_gaps\": ");
    raw_bool(check.can_report_protocol_gaps);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_lifeline_protocol_selftest_case(
    case: &RecoveryLifelineProtocolSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_lifeline_command_vocabulary_object(
    check: &RecoveryLifelineCommandVocabularyCheck,
    comma: bool,
) {
    raw_line("      \"command_vocabulary\": {");
    raw_line("        \"schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"exposed\": ");
    raw_bool(check.command_vocabulary_exposed);
    raw_line(",");
    raw_line("        \"authority\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_commands\": false,");
    raw_line(
        "        \"argument_envelope_schema\": \"raios.recovery_lifeline_command_envelope.v0\",",
    );
    raw("        \"primary_denial_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"command_count\": ");
    raw_fmt(format_args!(
        "{}",
        if check.command_vocabulary_exposed {
            6usize
        } else {
            0usize
        }
    ));
    raw_line(",");
    raw_line("        \"commands\": [");
    if check.command_vocabulary_exposed {
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.status",
            "raios.recovery_lifeline_command.status_args.v0",
            "cap.recovery.load_artifact.read",
            "observe",
            "read recovery lifeline status",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.rollback_preview",
            "raios.recovery_lifeline_command.rollback_preview_args.v0",
            "cap.recovery.rollback.read",
            "observe",
            "preview rollback target and required evidence",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.rollback_apply",
            "raios.recovery_lifeline_command.rollback_apply_args.v0",
            "cap.recovery.rollback",
            "persist",
            "apply a rollback transaction",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.disable_module",
            "raios.recovery_lifeline_command.disable_module_args.v0",
            "cap.recovery.module.disable",
            "persist",
            "disable a bad retained module id",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.restart_last_good",
            "raios.recovery_lifeline_command.restart_last_good_args.v0",
            "cap.recovery.service.restart",
            "recovery_modify_ram",
            "restart the last-good service set",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.load_artifact_by_hash",
            "raios.recovery_lifeline_command.load_artifact_by_hash_args.v0",
            "cap.recovery.load_artifact",
            "recovery_modify_ram",
            "load a recovery artifact by retained hash evidence",
            check.reason,
            false,
        );
    }
    raw_line("        ],");
    raw_line("        \"required_before_execution\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_provenance.v0\"");
    raw_line("        ]");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_lifeline_command_definition(
    command_id: &'static str,
    args_schema: &'static str,
    required_capability: &'static str,
    risk: &'static str,
    summary: &'static str,
    denial_reason: &'static str,
    comma: bool,
) {
    raw_line("          {");
    raw("            \"id\": ");
    json_str(command_id);
    raw_line(",");
    raw("            \"argument_schema\": ");
    json_str(args_schema);
    raw_line(",");
    raw("            \"required_capability\": ");
    json_str(required_capability);
    raw_line(",");
    raw("            \"risk\": ");
    json_str(risk);
    raw_line(",");
    raw("            \"summary\": ");
    json_str(summary);
    raw_line(",");
    raw_line("            \"status\": \"defined_non_executable\",");
    raw_line("            \"accepts_envelope\": false,");
    raw_line("            \"dispatches_command\": false,");
    raw_line("            \"authorizes_recovery_load\": false,");
    raw_line("            \"creates_durable_records\": false,");
    raw_line("            \"installs_rollback_plan\": false,");
    raw_line("            \"allocates_service_slot\": false,");
    raw("            \"denial_reason\": ");
    json_str(denial_reason);
    raw_line("");
    raw("          }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_lifeline_command_vocabulary_check(check: &RecoveryLifelineCommandVocabularyCheck) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"command_vocabulary_exposed\": ");
    raw_bool(check.command_vocabulary_exposed);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_lifeline_command_vocabulary_selftest_case(
    case: &RecoveryLifelineCommandVocabularySelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_loader_runtime_isolation_input_state(
    candidate: &RecoveryLoaderRuntimeIsolationCandidate,
    check: &RecoveryLoaderRuntimeIsolationCheck,
    comma: bool,
) {
    raw_line("      \"loader_isolation_inputs\": {");
    raw_line("        \"lifeline_protocol_state\": {");
    raw_line("          \"schema\": \"raios.recovery_lifeline_protocol_state.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.command_candidate.protocol_state_retained {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"event_id\": null,");
    raw("          \"current_boot\": ");
    raw_bool(candidate.command_candidate.protocol_state_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.command_candidate.protocol_state_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.command_candidate.protocol_state_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.command_candidate.protocol_state_binding_reason);
    raw_line(",");
    raw_line("          \"authorizes_recovery_load\": false");
    raw_line("        },");
    raw_line("        \"lifeline_command_vocabulary\": {");
    raw_line("          \"schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.command_vocabulary_available {
        "defined_read_only_envelope"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_read_only_diagnostic\",");
    raw_line("          \"event_id\": null,");
    raw("          \"envelope_exposed\": ");
    raw_bool(check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("          \"accepted_for_loader_readiness\": ");
    raw_bool(check.command_vocabulary_accepted);
    raw_line(",");
    raw("          \"current_boot\": ");
    raw_bool(candidate.command_vocabulary_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.command_vocabulary_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.command_vocabulary_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.command_vocabulary_binding_reason);
    raw_line(",");
    raw_line("          \"accepts_lifeline_command_envelope\": false,");
    raw_line("          \"dispatches_commands\": false,");
    raw_line("          \"authorizes_recovery_load\": false");
    raw_line("        }");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_loader_runtime_isolation_fact(
    field: &'static str,
    schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    comma: bool,
) {
    raw("        \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(if present { "present" } else { "missing" });
    raw(", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(if present {
        "current_boot_fact_available"
    } else {
        missing_reason
    });
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_loader_runtime_isolation_boundary(check: &RecoveryLoaderRuntimeIsolationCheck) {
    raw_line("        \"schema\": \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw("        \"requirements_exposed\": ");
    raw_bool(check.isolation_requirements_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_ready\": ");
    raw_bool(check.loader_runtime_isolation_ready);
    raw_line(",");
    raw_line("        \"loader_execution_enabled\": false,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"normal_module_load_path_used\": false,");
    raw_line("        \"recovery_lifeline_command_dispatch_enabled\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw_line("        \"required_before_execution\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("          \"raios.recovery_loader_address_space_boundary.v0\",");
    raw_line("          \"raios.recovery_loader_entrypoint_abi.v0\",");
    raw_line("          \"raios.recovery_loader_memory_map_constraints.v0\",");
    raw_line("          \"raios.recovery_loader_capability_import_table.v0\",");
    raw_line("          \"raios.recovery_loader_artifact_hash_binding.v0\",");
    raw_line("          \"raios.recovery_loader_provider_separation.v0\",");
    raw_line("          \"raios.recovery_loader_normal_module_separation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_provenance.v0\"");
    raw_line("        ]");
}

fn emit_recovery_loader_runtime_isolation_check(check: &RecoveryLoaderRuntimeIsolationCheck) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"command_vocabulary_envelope_exposed\": ");
    raw_bool(check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("        \"command_vocabulary_accepted\": ");
    raw_bool(check.command_vocabulary_accepted);
    raw_line(",");
    raw("        \"isolation_requirements_exposed\": ");
    raw_bool(check.isolation_requirements_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_ready\": ");
    raw_bool(check.loader_runtime_isolation_ready);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_loader_runtime_isolation_selftest_case(
    case: &RecoveryLoaderRuntimeIsolationSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"loader_execution_enabled\": false, \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_rollback_transaction_engine_input_state(
    candidate: &RecoveryRollbackTransactionEngineCandidate,
    loader_check: &RecoveryLoaderRuntimeIsolationCheck,
    check: &RecoveryRollbackTransactionEngineCheck,
    comma: bool,
) {
    raw_line("      \"rollback_transaction_inputs\": {");
    raw_line("        \"loader_runtime_isolation\": {");
    raw_line("          \"schema\": \"raios.recovery_loader_runtime_isolation.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.loader_runtime_isolation_available {
        "defined_read_only_boundary"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_read_only_diagnostic\",");
    raw_line("          \"event_id\": null,");
    raw("          \"boundary_exposed\": ");
    raw_bool(check.loader_runtime_isolation_boundary_exposed);
    raw_line(",");
    raw("          \"accepted_for_rollback_readiness\": ");
    raw_bool(check.loader_runtime_isolation_accepted);
    raw_line(",");
    raw("          \"loader_runtime_isolation_ready\": ");
    raw_bool(loader_check.loader_runtime_isolation_ready);
    raw_line(",");
    raw("          \"current_boot\": ");
    raw_bool(candidate.loader_runtime_isolation_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.loader_runtime_isolation_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.loader_runtime_isolation_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.loader_runtime_isolation_binding_reason);
    raw_line(",");
    raw_line("          \"accepts_loader_descriptor\": false,");
    raw_line("          \"loads_recovery_loader\": false,");
    raw_line("          \"authorizes_recovery_load\": false");
    raw_line("        }");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_rollback_transaction_engine_fact(
    field: &'static str,
    schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    comma: bool,
) {
    raw("        \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(if present { "present" } else { "missing" });
    raw(", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(if present {
        "current_boot_fact_available"
    } else {
        missing_reason
    });
    raw(", \"authorizes_recovery_load\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_rollback_transaction_engine_boundary(
    check: &RecoveryRollbackTransactionEngineCheck,
) {
    raw_line("        \"schema\": \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw("        \"requirements_exposed\": ");
    raw_bool(check.transaction_requirements_exposed);
    raw_line(",");
    raw("        \"rollback_transaction_engine_ready\": ");
    raw_bool(check.rollback_transaction_engine_ready);
    raw_line(",");
    raw_line("        \"rollback_preview_enabled\": false,");
    raw_line("        \"rollback_apply_enabled\": false,");
    raw_line("        \"accepts_rollback_transaction_envelope\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"executes_rollback_transaction\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw_line("        \"required_before_execution\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("          \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("          \"raios.recovery_rollback_target_selection.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_provenance.v0\",");
    raw_line("          \"raios.recovery_rollback_last_good_binding.v0\",");
    raw_line("          \"raios.recovery_rollback_disabled_module_set_binding.v0\",");
    raw_line("          \"raios.recovery_rollback_artifact_hash_binding.v0\",");
    raw_line("          \"raios.recovery_rollback_replay_preconditions.v0\",");
    raw_line("          \"raios.recovery_rollback_recovery_capability_import.v0\",");
    raw_line("          \"raios.recovery_rollback_atomic_apply_abort_semantics.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_provenance.v0\"");
    raw_line("        ]");
}

fn emit_recovery_rollback_transaction_engine_check(check: &RecoveryRollbackTransactionEngineCheck) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"command_vocabulary_envelope_exposed\": ");
    raw_bool(check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("        \"command_vocabulary_accepted\": ");
    raw_bool(check.command_vocabulary_accepted);
    raw_line(",");
    raw("        \"loader_runtime_isolation_boundary_exposed\": ");
    raw_bool(check.loader_runtime_isolation_boundary_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_accepted\": ");
    raw_bool(check.loader_runtime_isolation_accepted);
    raw_line(",");
    raw("        \"transaction_requirements_exposed\": ");
    raw_bool(check.transaction_requirements_exposed);
    raw_line(",");
    raw("        \"rollback_transaction_engine_ready\": ");
    raw_bool(check.rollback_transaction_engine_ready);
    raw_line(",");
    raw("        \"rollback_preview_enabled\": ");
    raw_bool(check.rollback_preview_enabled);
    raw_line(",");
    raw("        \"rollback_apply_enabled\": ");
    raw_bool(check.rollback_apply_enabled);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_rollback_transaction_engine_selftest_case(
    case: &RecoveryRollbackTransactionEngineSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_durable_audit_rollback_persistence_input_state(
    candidate: &RecoveryDurableAuditRollbackPersistenceCandidate,
    transaction_check: &RecoveryRollbackTransactionEngineCheck,
    check: &RecoveryDurableAuditRollbackPersistenceCheck,
    comma: bool,
) {
    raw_line("      \"durable_persistence_inputs\": {");
    raw_line("        \"rollback_transaction_engine\": {");
    raw_line("          \"schema\": \"raios.recovery_rollback_transaction_engine.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.rollback_transaction_engine_available {
        "defined_read_only_boundary"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_read_only_diagnostic\",");
    raw_line("          \"event_id\": null,");
    raw("          \"boundary_exposed\": ");
    raw_bool(check.rollback_transaction_engine_boundary_exposed);
    raw_line(",");
    raw("          \"accepted_for_persistence_readiness\": ");
    raw_bool(check.rollback_transaction_engine_accepted);
    raw_line(",");
    raw("          \"rollback_transaction_engine_ready\": ");
    raw_bool(transaction_check.rollback_transaction_engine_ready);
    raw_line(",");
    raw("          \"current_boot\": ");
    raw_bool(candidate.rollback_transaction_engine_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.rollback_transaction_engine_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.rollback_transaction_engine_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.rollback_transaction_engine_binding_reason);
    raw_line(",");
    raw_line("          \"accepts_rollback_transaction_envelope\": false,");
    raw_line("          \"executes_rollback_preview\": false,");
    raw_line("          \"executes_rollback_apply\": false,");
    raw_line("          \"creates_durable_records\": false,");
    raw_line("          \"installs_rollback_plan\": false");
    raw_line("        }");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_durable_audit_rollback_persistence_fact(
    field: &'static str,
    schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    comma: bool,
) {
    raw("        \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(if present { "present" } else { "missing" });
    raw(", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(if present {
        "current_boot_fact_available"
    } else {
        missing_reason
    });
    raw(", \"authorizes_recovery_load\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"recovery_memory_writes_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_durable_audit_rollback_persistence_boundary(
    check: &RecoveryDurableAuditRollbackPersistenceCheck,
) {
    raw_line("        \"schema\": \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw("        \"requirements_exposed\": ");
    raw_bool(check.persistence_requirements_exposed);
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_ready\": ");
    raw_bool(check.durable_audit_rollback_persistence_ready);
    raw_line(",");
    raw_line("        \"durable_writes_enabled\": false,");
    raw_line("        \"rollback_replay_enabled\": false,");
    raw_line("        \"recovery_memory_writes_enabled\": false,");
    raw_line("        \"rollback_preview_enabled\": false,");
    raw_line("        \"rollback_apply_enabled\": false,");
    raw_line("        \"accepts_rollback_transaction_envelope\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"updates_last_good_checkpoint\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw_line("        \"required_before_durable_write\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("          \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.persistence_device_inventory.v0\",");
    raw_line("          \"raios.durable_audit_rollback_storage_layout_identity.v0\",");
    raw_line("          \"raios.durable_audit_append_log_identity.v0\",");
    raw_line("          \"raios.rollback_store_identity.v0\",");
    raw_line("          \"raios.rollback_transaction_replay_cursor.v0\",");
    raw_line("          \"raios.recovery_last_good_checkpoint_binding.v0\",");
    raw_line("          \"raios.durable_write_ordering.v0\",");
    raw_line("          \"raios.durable_crash_consistency.v0\",");
    raw_line("          \"raios.durable_integrity_root_hash_chain.v0\",");
    raw_line("          \"raios.recovery_memory_provenance.v0\"");
    raw_line("        ]");
}

fn emit_recovery_durable_audit_rollback_persistence_check(
    check: &RecoveryDurableAuditRollbackPersistenceCheck,
) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"command_vocabulary_envelope_exposed\": ");
    raw_bool(check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("        \"command_vocabulary_accepted\": ");
    raw_bool(check.command_vocabulary_accepted);
    raw_line(",");
    raw("        \"loader_runtime_isolation_boundary_exposed\": ");
    raw_bool(check.loader_runtime_isolation_boundary_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_accepted\": ");
    raw_bool(check.loader_runtime_isolation_accepted);
    raw_line(",");
    raw("        \"rollback_transaction_engine_boundary_exposed\": ");
    raw_bool(check.rollback_transaction_engine_boundary_exposed);
    raw_line(",");
    raw("        \"rollback_transaction_engine_accepted\": ");
    raw_bool(check.rollback_transaction_engine_accepted);
    raw_line(",");
    raw("        \"persistence_requirements_exposed\": ");
    raw_bool(check.persistence_requirements_exposed);
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_ready\": ");
    raw_bool(check.durable_audit_rollback_persistence_ready);
    raw_line(",");
    raw("        \"durable_writes_enabled\": ");
    raw_bool(check.durable_writes_enabled);
    raw_line(",");
    raw("        \"rollback_replay_enabled\": ");
    raw_bool(check.rollback_replay_enabled);
    raw_line(",");
    raw("        \"recovery_memory_writes_enabled\": ");
    raw_bool(check.recovery_memory_writes_enabled);
    raw_line(",");
    raw("        \"rollback_preview_enabled\": ");
    raw_bool(check.rollback_preview_enabled);
    raw_line(",");
    raw("        \"rollback_apply_enabled\": ");
    raw_bool(check.rollback_apply_enabled);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"rollback_replay_attempted\": false,");
    raw_line("        \"recovery_memory_write_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_durable_audit_rollback_persistence_selftest_case(
    case: &RecoveryDurableAuditRollbackPersistenceSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"recovery_memory_writes_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_memory_provenance_input_state(
    candidate: &RecoveryMemoryProvenanceCandidate,
    persistence_check: &RecoveryDurableAuditRollbackPersistenceCheck,
    check: &RecoveryMemoryProvenanceCheck,
    comma: bool,
) {
    raw_line("      \"memory_provenance_inputs\": {");
    raw_line("        \"durable_audit_rollback_persistence\": {");
    raw_line("          \"schema\": \"raios.durable_audit_rollback_persistence.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.durable_audit_rollback_persistence_available {
        "defined_read_only_boundary"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_read_only_diagnostic\",");
    raw_line("          \"event_id\": null,");
    raw("          \"boundary_exposed\": ");
    raw_bool(check.durable_audit_rollback_persistence_boundary_exposed);
    raw_line(",");
    raw("          \"accepted_for_memory_readiness\": ");
    raw_bool(check.durable_audit_rollback_persistence_accepted);
    raw_line(",");
    raw("          \"durable_audit_rollback_persistence_ready\": ");
    raw_bool(persistence_check.durable_audit_rollback_persistence_ready);
    raw_line(",");
    raw("          \"current_boot\": ");
    raw_bool(candidate.durable_audit_rollback_persistence_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.durable_audit_rollback_persistence_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.durable_audit_rollback_persistence_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.durable_audit_rollback_persistence_binding_reason);
    raw_line(",");
    raw_line("          \"writes_durable_audit_log\": false,");
    raw_line("          \"writes_rollback_store\": false,");
    raw_line("          \"replays_rollback_transactions\": false,");
    raw_line("          \"recovery_memory_writes_enabled\": false");
    raw_line("        }");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_memory_provenance_fact(
    field: &'static str,
    schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    comma: bool,
) {
    raw("        \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(if present { "present" } else { "missing" });
    raw(", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(if present {
        "current_boot_fact_available"
    } else {
        missing_reason
    });
    raw(", \"authorizes_recovery_load\": false, \"memory_writes_enabled\": false, \"provider_export_enabled\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"recovery_memory_writes_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_memory_provenance_boundary(check: &RecoveryMemoryProvenanceCheck) {
    raw_line("        \"schema\": \"raios.recovery_memory_provenance.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw("        \"requirements_exposed\": ");
    raw_bool(check.memory_provenance_requirements_exposed);
    raw_line(",");
    raw("        \"recovery_memory_provenance_ready\": ");
    raw_bool(check.recovery_memory_provenance_ready);
    raw_line(",");
    raw_line("        \"memory_writes_enabled\": false,");
    raw_line("        \"provider_export_enabled\": false,");
    raw_line("        \"durable_writes_enabled\": false,");
    raw_line("        \"rollback_replay_enabled\": false,");
    raw_line("        \"recovery_memory_writes_enabled\": false,");
    raw_line("        \"rollback_preview_enabled\": false,");
    raw_line("        \"rollback_apply_enabled\": false,");
    raw_line("        \"accepts_memory_record_json\": false,");
    raw_line("        \"accepts_provider_context_export\": false,");
    raw_line("        \"accepts_rollback_transaction_envelope\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"exports_provider_context\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw_line("        \"required_before_memory_write\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("          \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_source_record_ids.v0\",");
    raw_line("          \"raios.recovery_memory_source_schema_hashes.v0\",");
    raw_line("          \"raios.recovery_memory_classification.v0\",");
    raw_line("          \"raios.recovery_memory_authority_level.v0\",");
    raw_line("          \"raios.recovery_memory_rollback_transaction_binding.v0\",");
    raw_line("          \"raios.recovery_memory_last_good_checkpoint_binding.v0\",");
    raw_line("          \"raios.recovery_memory_export_profile.v0\",");
    raw_line("          \"raios.recovery_memory_redaction_state.v0\",");
    raw_line("          \"raios.recovery_memory_replay_window.v0\",");
    raw_line("          \"raios.recovery_memory_audit_linkage.v0\"");
    raw_line("        ]");
}

fn emit_recovery_memory_provenance_check(check: &RecoveryMemoryProvenanceCheck) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"command_vocabulary_envelope_exposed\": ");
    raw_bool(check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("        \"command_vocabulary_accepted\": ");
    raw_bool(check.command_vocabulary_accepted);
    raw_line(",");
    raw("        \"loader_runtime_isolation_boundary_exposed\": ");
    raw_bool(check.loader_runtime_isolation_boundary_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_accepted\": ");
    raw_bool(check.loader_runtime_isolation_accepted);
    raw_line(",");
    raw("        \"rollback_transaction_engine_boundary_exposed\": ");
    raw_bool(check.rollback_transaction_engine_boundary_exposed);
    raw_line(",");
    raw("        \"rollback_transaction_engine_accepted\": ");
    raw_bool(check.rollback_transaction_engine_accepted);
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_boundary_exposed\": ");
    raw_bool(check.durable_audit_rollback_persistence_boundary_exposed);
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_accepted\": ");
    raw_bool(check.durable_audit_rollback_persistence_accepted);
    raw_line(",");
    raw("        \"memory_provenance_requirements_exposed\": ");
    raw_bool(check.memory_provenance_requirements_exposed);
    raw_line(",");
    raw("        \"recovery_memory_provenance_ready\": ");
    raw_bool(check.recovery_memory_provenance_ready);
    raw_line(",");
    raw("        \"memory_writes_enabled\": ");
    raw_bool(check.memory_writes_enabled);
    raw_line(",");
    raw("        \"provider_export_enabled\": ");
    raw_bool(check.provider_export_enabled);
    raw_line(",");
    raw("        \"durable_writes_enabled\": ");
    raw_bool(check.durable_writes_enabled);
    raw_line(",");
    raw("        \"rollback_replay_enabled\": ");
    raw_bool(check.rollback_replay_enabled);
    raw_line(",");
    raw("        \"recovery_memory_writes_enabled\": ");
    raw_bool(check.recovery_memory_writes_enabled);
    raw_line(",");
    raw("        \"rollback_preview_enabled\": ");
    raw_bool(check.rollback_preview_enabled);
    raw_line(",");
    raw("        \"rollback_apply_enabled\": ");
    raw_bool(check.rollback_apply_enabled);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"rollback_replay_attempted\": false,");
    raw_line("        \"recovery_memory_write_attempted\": false,");
    raw_line("        \"provider_export_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_memory_provenance_selftest_case(
    case: &RecoveryMemoryProvenanceSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"memory_writes_enabled\": false, \"provider_export_enabled\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"recovery_memory_writes_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_artifact_load_missing_fact(
    field: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("      \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(reason);
    raw(", \"authorizes_recovery_load\": false, \"loads_recovery_artifact\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_identity_binding_fact(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_artifact_identity_event_id\": {\"schema\": \"raios.recovery_artifact_identity.v0\"");
    if let Some((event_id, reference)) = retained {
        raw(", \"status\": \"retained_hash_reference_only\", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"retained_recovery_artifact_identity_reference_not_authorizing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_artifact_identity_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_trust_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_artifact_trust_event_id\": {\"schema\": \"raios.recovery_artifact_trust.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_trust_mismatch(retained_identity, retained);
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(mismatch.unwrap_or("retained_recovery_artifact_trust_reference_not_authorizing"));
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"hashes\": {\"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_artifact_trust_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_vm_test_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_vm_test_event_id\": {\"schema\": \"raios.recovery_artifact_vm_test.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_vm_test_mismatch(
            retained_identity,
            retained_trust,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(
            mismatch.unwrap_or("retained_recovery_artifact_vm_test_reference_not_authorizing"),
        );
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"hashes\": {\"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_vm_test_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_local_approval_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_local_approval_event_id\": {\"schema\": \"raios.recovery_artifact_local_approval.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_local_approval_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(
            mismatch
                .unwrap_or("retained_recovery_artifact_local_approval_reference_not_authorizing"),
        );
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw(", \"hashes\": {\"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_local_approval_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_loader_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_loader_event_id\": {\"schema\": \"raios.recovery_artifact_loader.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_loader_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(mismatch.unwrap_or("retained_recovery_artifact_loader_reference_not_authorizing"));
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw(", \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw(", \"hashes\": {\"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_loader_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_rollback_evidence_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_rollback_evidence_event_id\": {\"schema\": \"raios.recovery_artifact_rollback_evidence.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_rollback_evidence_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained_loader,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(
            mismatch.unwrap_or(
                "retained_recovery_artifact_rollback_evidence_reference_not_authorizing",
            ),
        );
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw(", \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw(", \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw(", \"hashes\": {\"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw(", \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw(", \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_rollback_evidence_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_blocker(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if *wrote {
        raw_line(",");
    }
    raw("        {\"gate\": ");
    json_str(gate);
    raw(", \"state\": ");
    json_str(state);
    raw(", \"reason\": ");
    json_str(reason);
    raw("}");
    *wrote = true;
}

fn emit_recovery_load_binding_check(
    check: &RecoveryLoadBindingCheck,
    _spaces: usize,
    _include_status: bool,
) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"recovery_only_capability_used\": ");
    raw_bool(check.recovery_only_capability_used);
    raw_line(",");
    raw("        \"accepts_normal_module_authority\": ");
    raw_bool(check.accepts_normal_module_authority);
    raw_line(",");
    raw("        \"append_payload_hash_authority\": ");
    raw_bool(check.append_payload_hash_authority);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"loads_normal_module\": ");
    raw_bool(check.loads_normal_module);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_load_binding_selftest_case(case: &RecoveryLoadBindingSelfTestCase, comma: bool) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"load_attempted\": false, \"normal_module_capability_accepted\": false, \"append_payload_hash_authority\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn recovery_lifeline_protocol_missing_candidate() -> RecoveryLifelineProtocolCandidate {
    RecoveryLifelineProtocolCandidate {
        request_retained: false,
        request_current_boot: false,
        request_schema_ok: false,
        request_binding_ok: false,
        request_binding_reason: "recovery_lifeline_request_event_id_missing",
        direct_openai_recovery_shortcut_used: false,
        lifeline_protocol_state_present: false,
        command_vocabulary_present: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn recovery_lifeline_protocol_valid_request_candidate() -> RecoveryLifelineProtocolCandidate {
    RecoveryLifelineProtocolCandidate {
        request_retained: true,
        request_current_boot: true,
        request_schema_ok: true,
        request_binding_ok: true,
        request_binding_reason: "retained_recovery_lifeline_request_valid",
        direct_openai_recovery_shortcut_used: false,
        lifeline_protocol_state_present: false,
        command_vocabulary_present: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn evaluate_recovery_lifeline_protocol(
    candidate: RecoveryLifelineProtocolCandidate,
) -> RecoveryLifelineProtocolCheck {
    if !candidate.request_retained {
        return recovery_lifeline_protocol_check(
            "missing",
            "recovery_lifeline_request_event_id_missing",
            false,
            false,
        );
    }
    if !candidate.request_current_boot {
        return recovery_lifeline_protocol_check(
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            false,
            false,
        );
    }
    if !candidate.request_schema_ok {
        return recovery_lifeline_protocol_check(
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            false,
            false,
        );
    }
    if !candidate.request_binding_ok {
        return recovery_lifeline_protocol_check(
            "rejected",
            candidate.request_binding_reason,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used {
        return recovery_lifeline_protocol_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            false,
            false,
        );
    }
    if !candidate.lifeline_protocol_state_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            true,
        );
    }
    if !candidate.command_vocabulary_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            true,
        );
    }
    if !candidate.loader_runtime_isolation_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
        );
    }
    if !candidate.rollback_transaction_engine_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
        );
    }
    recovery_lifeline_protocol_check(
        "denied_lifeline_protocol_behavior_unimplemented",
        "recovery_lifeline_protocol_behavior_not_implemented",
        true,
        true,
    )
}

fn recovery_lifeline_protocol_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    can_report_protocol_gaps: bool,
) -> RecoveryLifelineProtocolCheck {
    RecoveryLifelineProtocolCheck {
        status,
        reason,
        request_chain_valid,
        can_report_protocol_gaps,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn recovery_lifeline_protocol_selftest_cases(
) -> [RecoveryLifelineProtocolSelfTestCase; RECOVERY_LIFELINE_PROTOCOL_SELFTEST_CASES] {
    let valid = recovery_lifeline_protocol_valid_request_candidate();

    let mut stale = valid;
    stale.request_binding_ok = false;
    stale.request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";

    let mut previous_boot = valid;
    previous_boot.request_current_boot = false;

    let mut wrong_schema = valid;
    wrong_schema.request_schema_ok = false;

    let mut substituted = valid;
    substituted.request_binding_ok = false;
    substituted.request_binding_reason = "recovery_lifeline_request_substituted_record";

    let mut request_hash_mismatch = valid;
    request_hash_mismatch.request_binding_ok = false;
    request_hash_mismatch.request_binding_reason =
        "recovery_lifeline_request_reference_hash_mismatch";

    let mut identity_event_mismatch = valid;
    identity_event_mismatch.request_binding_ok = false;
    identity_event_mismatch.request_binding_reason =
        "recovery_lifeline_request_identity_event_id_mismatch";

    let mut rollback_reference_mismatch = valid;
    rollback_reference_mismatch.request_binding_ok = false;
    rollback_reference_mismatch.request_binding_reason =
        "recovery_artifact_rollback_evidence_reference_hash_mismatch";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut command_vocabulary_missing = valid;
    command_vocabulary_missing.lifeline_protocol_state_present = true;

    let mut loader_isolation_missing = command_vocabulary_missing;
    loader_isolation_missing.command_vocabulary_present = true;

    let mut rollback_engine_missing = loader_isolation_missing;
    rollback_engine_missing.loader_runtime_isolation_present = true;

    let mut durable_persistence_missing = rollback_engine_missing;
    durable_persistence_missing.rollback_transaction_engine_present = true;

    let mut memory_provenance_missing = durable_persistence_missing;
    memory_provenance_missing.durable_audit_rollback_persistence_present = true;

    [
        recovery_lifeline_protocol_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_lifeline_protocol(recovery_lifeline_protocol_missing_candidate()),
        ),
        recovery_lifeline_protocol_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_lifeline_protocol(stale),
        ),
        recovery_lifeline_protocol_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_lifeline_protocol(previous_boot),
        ),
        recovery_lifeline_protocol_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_lifeline_protocol(wrong_schema),
        ),
        recovery_lifeline_protocol_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_lifeline_protocol(substituted),
        ),
        recovery_lifeline_protocol_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_lifeline_protocol(request_hash_mismatch),
        ),
        recovery_lifeline_protocol_selftest_case(
            "retained_identity_event_id_mismatch",
            "rejected",
            "recovery_lifeline_request_identity_event_id_mismatch",
            evaluate_recovery_lifeline_protocol(identity_event_mismatch),
        ),
        recovery_lifeline_protocol_selftest_case(
            "rollback_evidence_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            evaluate_recovery_lifeline_protocol(rollback_reference_mismatch),
        ),
        recovery_lifeline_protocol_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_lifeline_protocol(direct_provider_shortcut),
        ),
        recovery_lifeline_protocol_selftest_case(
            "accepted_current_boot_request_protocol_state_missing",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_lifeline_protocol(valid),
        ),
        recovery_lifeline_protocol_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_lifeline_protocol(command_vocabulary_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_lifeline_protocol(loader_isolation_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "rollback_transaction_engine_missing_after_isolation",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_lifeline_protocol(rollback_engine_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "durable_audit_rollback_persistence_missing_after_engine",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_lifeline_protocol(durable_persistence_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "recovery_memory_provenance_missing_after_persistence",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_lifeline_protocol(memory_provenance_missing),
        ),
    ]
}

fn recovery_lifeline_protocol_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineProtocolCheck,
) -> RecoveryLifelineProtocolSelfTestCase {
    RecoveryLifelineProtocolSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

fn recovery_lifeline_command_vocabulary_candidate_from_protocol(
    protocol_candidate: RecoveryLifelineProtocolCandidate,
) -> RecoveryLifelineCommandVocabularyCandidate {
    RecoveryLifelineCommandVocabularyCandidate {
        protocol_candidate,
        protocol_state_retained: false,
        protocol_state_current_boot: false,
        protocol_state_schema_ok: false,
        protocol_state_binding_ok: false,
        protocol_state_binding_reason: "recovery_lifeline_protocol_state_missing",
        direct_openai_recovery_shortcut_used: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn evaluate_recovery_lifeline_command_vocabulary(
    candidate: RecoveryLifelineCommandVocabularyCandidate,
) -> RecoveryLifelineCommandVocabularyCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(candidate.protocol_candidate);
    if !protocol_check.request_chain_valid {
        return recovery_lifeline_command_vocabulary_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            false,
            false,
        );
    }
    if !candidate.protocol_state_retained {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            true,
        );
    }
    if !candidate.protocol_state_current_boot {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            false,
            false,
        );
    }
    if !candidate.protocol_state_schema_ok {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            false,
            false,
        );
    }
    if !candidate.protocol_state_binding_ok {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            candidate.protocol_state_binding_reason,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
        );
    }
    if !candidate.rollback_transaction_engine_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
        );
    }
    recovery_lifeline_command_vocabulary_check(
        "defined_non_executable",
        "recovery_lifeline_command_behavior_not_implemented",
        true,
        true,
    )
}

fn recovery_lifeline_command_vocabulary_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_exposed: bool,
) -> RecoveryLifelineCommandVocabularyCheck {
    RecoveryLifelineCommandVocabularyCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_exposed,
        command_execution_enabled: false,
        accepts_lifeline_command_envelope: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn recovery_lifeline_command_vocabulary_selftest_cases(
) -> [RecoveryLifelineCommandVocabularySelfTestCase;
       RECOVERY_LIFELINE_COMMAND_VOCABULARY_SELFTEST_CASES] {
    let valid_protocol = recovery_lifeline_protocol_valid_request_candidate();
    let valid = RecoveryLifelineCommandVocabularyCandidate {
        protocol_candidate: valid_protocol,
        protocol_state_retained: true,
        protocol_state_current_boot: true,
        protocol_state_schema_ok: true,
        protocol_state_binding_ok: true,
        protocol_state_binding_reason: "retained_recovery_lifeline_protocol_state_valid",
        direct_openai_recovery_shortcut_used: false,
        loader_runtime_isolation_present: true,
        rollback_transaction_engine_present: true,
        durable_audit_rollback_persistence_present: true,
        recovery_memory_provenance_present: true,
    };

    let mut missing_request = valid;
    missing_request.protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request.protocol_candidate.request_binding_ok = false;
    stale_request.protocol_candidate.request_binding_reason =
        "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request.protocol_candidate.request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request.protocol_candidate.request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request.protocol_candidate.request_binding_ok = false;
    substituted_request
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch.protocol_candidate.request_binding_ok = false;
    request_hash_mismatch
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state.protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state.protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state.protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state.protocol_state_binding_ok = false;
    substituted_protocol_state.protocol_state_binding_reason =
        "recovery_lifeline_protocol_state_substituted_record";
    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;
    let mut isolation_missing = valid;
    isolation_missing.loader_runtime_isolation_present = false;
    let mut rollback_engine_missing = valid;
    rollback_engine_missing.rollback_transaction_engine_present = false;
    let mut durable_persistence_missing = valid;
    durable_persistence_missing.durable_audit_rollback_persistence_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_lifeline_command_vocabulary_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_lifeline_command_vocabulary(missing_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_lifeline_command_vocabulary(stale_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_vocabulary(previous_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_vocabulary(wrong_schema_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_lifeline_command_vocabulary(substituted_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_lifeline_command_vocabulary(request_hash_mismatch),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_lifeline_command_vocabulary(missing_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_vocabulary(previous_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_vocabulary(wrong_schema_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_lifeline_command_vocabulary(substituted_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_lifeline_command_vocabulary(direct_provider_shortcut),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "loader_runtime_isolation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_lifeline_command_vocabulary(isolation_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "rollback_transaction_engine_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_lifeline_command_vocabulary(rollback_engine_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_lifeline_command_vocabulary(durable_persistence_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_lifeline_command_vocabulary(memory_provenance_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "all_inputs_present_commands_still_non_executable",
            "defined_non_executable",
            "recovery_lifeline_command_behavior_not_implemented",
            evaluate_recovery_lifeline_command_vocabulary(valid),
        ),
    ]
}

fn recovery_lifeline_command_vocabulary_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandVocabularyCheck,
) -> RecoveryLifelineCommandVocabularySelfTestCase {
    RecoveryLifelineCommandVocabularySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

fn recovery_loader_runtime_isolation_candidate_from_command_vocabulary(
    command_candidate: RecoveryLifelineCommandVocabularyCandidate,
) -> RecoveryLoaderRuntimeIsolationCandidate {
    RecoveryLoaderRuntimeIsolationCandidate {
        command_candidate,
        command_vocabulary_available: true,
        command_vocabulary_current_boot: true,
        command_vocabulary_schema_ok: true,
        command_vocabulary_binding_ok: true,
        command_vocabulary_binding_reason: "recovery_lifeline_command_vocabulary_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        loader_address_space_boundary_present: false,
        loader_entrypoint_abi_present: false,
        loader_memory_map_constraints_present: false,
        loader_capability_import_table_present: false,
        loader_artifact_hash_binding_present: false,
        loader_provider_separation_present: false,
        loader_normal_module_separation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn evaluate_recovery_loader_runtime_isolation(
    candidate: RecoveryLoaderRuntimeIsolationCandidate,
) -> RecoveryLoaderRuntimeIsolationCheck {
    let protocol_check =
        evaluate_recovery_lifeline_protocol(candidate.command_candidate.protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(candidate.command_candidate);
    if !protocol_check.request_chain_valid {
        return recovery_loader_runtime_isolation_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            true,
            false,
            false,
            false,
            false,
        );
    }

    let vocabulary_envelope_exposed =
        command_check.command_vocabulary_exposed || candidate.command_vocabulary_available;
    if !candidate.command_candidate.protocol_state_retained {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            vocabulary_envelope_exposed,
            false,
            vocabulary_envelope_exposed,
            false,
        );
    }
    if !candidate.command_candidate.protocol_state_current_boot {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            vocabulary_envelope_exposed,
            false,
            false,
            false,
        );
    }
    if !candidate.command_candidate.protocol_state_schema_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            vocabulary_envelope_exposed,
            false,
            false,
            false,
        );
    }
    if !candidate.command_candidate.protocol_state_binding_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            candidate.command_candidate.protocol_state_binding_reason,
            true,
            vocabulary_envelope_exposed,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_available || !command_check.command_vocabulary_exposed {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_current_boot {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_schema_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_binding_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            candidate.command_vocabulary_binding_reason,
            true,
            true,
            false,
            false,
            false,
        );
    }

    if !candidate.loader_address_space_boundary_present
        && !candidate.loader_entrypoint_abi_present
        && !candidate.loader_memory_map_constraints_present
        && !candidate.loader_capability_import_table_present
        && !candidate.loader_artifact_hash_binding_present
        && !candidate.loader_provider_separation_present
        && !candidate.loader_normal_module_separation_present
    {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_address_space_boundary_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_entrypoint_abi_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_memory_map_constraints_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_capability_import_table_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_artifact_hash_binding_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_provider_separation_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_normal_module_separation_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_transaction_engine_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
            true,
            true,
            true,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
            true,
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
            true,
            true,
            true,
        );
    }
    recovery_loader_runtime_isolation_check(
        "defined_non_executable",
        "recovery_loader_runtime_behavior_not_implemented",
        true,
        true,
        true,
        true,
        true,
    )
}

fn recovery_loader_runtime_isolation_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    isolation_requirements_exposed: bool,
    loader_runtime_isolation_ready: bool,
) -> RecoveryLoaderRuntimeIsolationCheck {
    RecoveryLoaderRuntimeIsolationCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        isolation_requirements_exposed,
        loader_runtime_isolation_ready,
        command_execution_enabled: false,
        accepts_lifeline_command_envelope: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn recovery_loader_runtime_isolation_valid_candidate() -> RecoveryLoaderRuntimeIsolationCandidate {
    let valid_protocol = recovery_lifeline_protocol_valid_request_candidate();
    let valid_command = RecoveryLifelineCommandVocabularyCandidate {
        protocol_candidate: valid_protocol,
        protocol_state_retained: true,
        protocol_state_current_boot: true,
        protocol_state_schema_ok: true,
        protocol_state_binding_ok: true,
        protocol_state_binding_reason: "retained_recovery_lifeline_protocol_state_valid",
        direct_openai_recovery_shortcut_used: false,
        loader_runtime_isolation_present: true,
        rollback_transaction_engine_present: true,
        durable_audit_rollback_persistence_present: true,
        recovery_memory_provenance_present: true,
    };
    RecoveryLoaderRuntimeIsolationCandidate {
        command_candidate: valid_command,
        command_vocabulary_available: true,
        command_vocabulary_current_boot: true,
        command_vocabulary_schema_ok: true,
        command_vocabulary_binding_ok: true,
        command_vocabulary_binding_reason: "retained_recovery_lifeline_command_vocabulary_valid",
        direct_openai_recovery_shortcut_used: false,
        loader_address_space_boundary_present: true,
        loader_entrypoint_abi_present: true,
        loader_memory_map_constraints_present: true,
        loader_capability_import_table_present: true,
        loader_artifact_hash_binding_present: true,
        loader_provider_separation_present: true,
        loader_normal_module_separation_present: true,
        rollback_transaction_engine_present: true,
        durable_audit_rollback_persistence_present: true,
        recovery_memory_provenance_present: true,
    }
}

fn recovery_loader_runtime_isolation_selftest_cases(
) -> [RecoveryLoaderRuntimeIsolationSelfTestCase; RECOVERY_LOADER_RUNTIME_ISOLATION_SELFTEST_CASES]
{
    let valid = recovery_loader_runtime_isolation_valid_candidate();

    let mut missing_request = valid;
    missing_request.command_candidate.protocol_candidate =
        recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary.command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary.command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary.command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary.command_vocabulary_binding_ok = false;
    substituted_command_vocabulary.command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut all_isolation_missing = valid;
    all_isolation_missing.loader_address_space_boundary_present = false;
    all_isolation_missing.loader_entrypoint_abi_present = false;
    all_isolation_missing.loader_memory_map_constraints_present = false;
    all_isolation_missing.loader_capability_import_table_present = false;
    all_isolation_missing.loader_artifact_hash_binding_present = false;
    all_isolation_missing.loader_provider_separation_present = false;
    all_isolation_missing.loader_normal_module_separation_present = false;

    let mut address_space_missing = valid;
    address_space_missing.loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing.loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing.loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing.loader_capability_import_table_present = false;
    let mut artifact_hash_binding_missing = valid;
    artifact_hash_binding_missing.loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing.loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing.loader_normal_module_separation_present = false;

    let mut rollback_engine_missing = valid;
    rollback_engine_missing.rollback_transaction_engine_present = false;
    let mut durable_persistence_missing = valid;
    durable_persistence_missing.durable_audit_rollback_persistence_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_loader_runtime_isolation_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_loader_runtime_isolation(missing_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_loader_runtime_isolation(stale_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_loader_runtime_isolation(previous_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_loader_runtime_isolation(wrong_schema_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_loader_runtime_isolation(substituted_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_loader_runtime_isolation(request_hash_mismatch),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_loader_runtime_isolation(missing_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_loader_runtime_isolation(previous_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_loader_runtime_isolation(wrong_schema_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_loader_runtime_isolation(substituted_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_loader_runtime_isolation(missing_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_loader_runtime_isolation(previous_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_loader_runtime_isolation(wrong_schema_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_loader_runtime_isolation(substituted_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_loader_runtime_isolation(direct_provider_shortcut),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_runtime_isolation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_loader_runtime_isolation(all_isolation_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_loader_runtime_isolation(address_space_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_loader_runtime_isolation(entrypoint_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_loader_runtime_isolation(memory_map_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_loader_runtime_isolation(import_table_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_loader_runtime_isolation(artifact_hash_binding_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_loader_runtime_isolation(provider_separation_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_loader_runtime_isolation(normal_module_separation_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "rollback_transaction_engine_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_loader_runtime_isolation(rollback_engine_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_loader_runtime_isolation(durable_persistence_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_loader_runtime_isolation(memory_provenance_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "all_inputs_present_loader_still_non_executable",
            "defined_non_executable",
            "recovery_loader_runtime_behavior_not_implemented",
            evaluate_recovery_loader_runtime_isolation(valid),
        ),
    ]
}

fn recovery_loader_runtime_isolation_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLoaderRuntimeIsolationCheck,
) -> RecoveryLoaderRuntimeIsolationSelfTestCase {
    RecoveryLoaderRuntimeIsolationSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

fn recovery_rollback_transaction_engine_candidate_from_loader(
    loader_candidate: RecoveryLoaderRuntimeIsolationCandidate,
) -> RecoveryRollbackTransactionEngineCandidate {
    RecoveryRollbackTransactionEngineCandidate {
        loader_candidate,
        loader_runtime_isolation_available: true,
        loader_runtime_isolation_current_boot: true,
        loader_runtime_isolation_schema_ok: true,
        loader_runtime_isolation_binding_ok: true,
        loader_runtime_isolation_binding_reason:
            "recovery_loader_runtime_isolation_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        rollback_target_selection_present: false,
        rollback_transaction_id_provenance_present: false,
        rollback_last_good_binding_present: false,
        rollback_disabled_module_set_binding_present: false,
        rollback_artifact_hash_binding_present: false,
        rollback_replay_preconditions_present: false,
        rollback_recovery_capability_import_present: false,
        rollback_atomic_apply_abort_semantics_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn evaluate_recovery_rollback_transaction_engine(
    candidate: RecoveryRollbackTransactionEngineCandidate,
) -> RecoveryRollbackTransactionEngineCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(
        candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate,
    );
    let command_check =
        evaluate_recovery_lifeline_command_vocabulary(candidate.loader_candidate.command_candidate);
    let loader_check = evaluate_recovery_loader_runtime_isolation(candidate.loader_candidate);

    if !protocol_check.request_chain_valid {
        return recovery_rollback_transaction_engine_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .loader_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .loader_candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }

    let command_vocabulary_envelope_exposed = command_check.command_vocabulary_exposed
        || candidate.loader_candidate.command_vocabulary_available;
    let loader_runtime_isolation_boundary_exposed =
        loader_check.isolation_requirements_exposed || candidate.loader_runtime_isolation_available;
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained
    {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            command_vocabulary_envelope_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
        );
    }
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_reason,
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_available
        || !command_check.command_vocabulary_exposed
    {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_current_boot {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_schema_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_binding_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            candidate.loader_candidate.command_vocabulary_binding_reason,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }

    if !candidate.loader_runtime_isolation_available {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_current_boot {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_schema_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_binding_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            candidate.loader_runtime_isolation_binding_reason,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !loader_check.loader_runtime_isolation_ready {
        return recovery_rollback_transaction_engine_check(
            loader_check.status,
            loader_check.reason,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }

    if !candidate.rollback_target_selection_present
        && !candidate.rollback_transaction_id_provenance_present
        && !candidate.rollback_last_good_binding_present
        && !candidate.rollback_disabled_module_set_binding_present
        && !candidate.rollback_artifact_hash_binding_present
        && !candidate.rollback_replay_preconditions_present
        && !candidate.rollback_recovery_capability_import_present
        && !candidate.rollback_atomic_apply_abort_semantics_present
    {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_target_selection_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_transaction_id_provenance_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_last_good_binding_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_disabled_module_set_binding_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_artifact_hash_binding_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_replay_preconditions_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_recovery_capability_import_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_atomic_apply_abort_semantics_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        );
    }
    recovery_rollback_transaction_engine_check(
        "defined_non_executable",
        "recovery_rollback_transaction_behavior_not_implemented",
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

fn recovery_rollback_transaction_engine_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    transaction_requirements_exposed: bool,
    rollback_transaction_engine_ready: bool,
) -> RecoveryRollbackTransactionEngineCheck {
    RecoveryRollbackTransactionEngineCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        loader_runtime_isolation_boundary_exposed,
        loader_runtime_isolation_accepted,
        transaction_requirements_exposed,
        rollback_transaction_engine_ready,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
        command_execution_enabled: false,
        accepts_lifeline_command_envelope: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn recovery_rollback_transaction_engine_valid_candidate(
) -> RecoveryRollbackTransactionEngineCandidate {
    RecoveryRollbackTransactionEngineCandidate {
        loader_candidate: recovery_loader_runtime_isolation_valid_candidate(),
        loader_runtime_isolation_available: true,
        loader_runtime_isolation_current_boot: true,
        loader_runtime_isolation_schema_ok: true,
        loader_runtime_isolation_binding_ok: true,
        loader_runtime_isolation_binding_reason: "retained_recovery_loader_runtime_isolation_valid",
        direct_openai_recovery_shortcut_used: false,
        rollback_target_selection_present: true,
        rollback_transaction_id_provenance_present: true,
        rollback_last_good_binding_present: true,
        rollback_disabled_module_set_binding_present: true,
        rollback_artifact_hash_binding_present: true,
        rollback_replay_preconditions_present: true,
        rollback_recovery_capability_import_present: true,
        rollback_atomic_apply_abort_semantics_present: true,
        durable_audit_rollback_persistence_present: true,
        recovery_memory_provenance_present: true,
    }
}

fn recovery_rollback_transaction_engine_selftest_cases(
) -> [RecoveryRollbackTransactionEngineSelfTestCase;
       RECOVERY_ROLLBACK_TRANSACTION_ENGINE_SELFTEST_CASES] {
    let valid = recovery_rollback_transaction_engine_valid_candidate();

    let mut missing_request = valid;
    missing_request
        .loader_candidate
        .command_candidate
        .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary
        .loader_candidate
        .command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary
        .loader_candidate
        .command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary
        .loader_candidate
        .command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary
        .loader_candidate
        .command_vocabulary_binding_ok = false;
    substituted_command_vocabulary
        .loader_candidate
        .command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut missing_loader_runtime_isolation = valid;
    missing_loader_runtime_isolation.loader_runtime_isolation_available = false;
    let mut previous_loader_runtime_isolation = valid;
    previous_loader_runtime_isolation.loader_runtime_isolation_current_boot = false;
    let mut wrong_schema_loader_runtime_isolation = valid;
    wrong_schema_loader_runtime_isolation.loader_runtime_isolation_schema_ok = false;
    let mut substituted_loader_runtime_isolation = valid;
    substituted_loader_runtime_isolation.loader_runtime_isolation_binding_ok = false;
    substituted_loader_runtime_isolation.loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_substituted_record";

    let mut address_space_missing = valid;
    address_space_missing
        .loader_candidate
        .loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing
        .loader_candidate
        .loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing
        .loader_candidate
        .loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing
        .loader_candidate
        .loader_capability_import_table_present = false;
    let mut artifact_hash_isolation_missing = valid;
    artifact_hash_isolation_missing
        .loader_candidate
        .loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing
        .loader_candidate
        .loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing
        .loader_candidate
        .loader_normal_module_separation_present = false;

    let mut all_transaction_missing = valid;
    all_transaction_missing.rollback_target_selection_present = false;
    all_transaction_missing.rollback_transaction_id_provenance_present = false;
    all_transaction_missing.rollback_last_good_binding_present = false;
    all_transaction_missing.rollback_disabled_module_set_binding_present = false;
    all_transaction_missing.rollback_artifact_hash_binding_present = false;
    all_transaction_missing.rollback_replay_preconditions_present = false;
    all_transaction_missing.rollback_recovery_capability_import_present = false;
    all_transaction_missing.rollback_atomic_apply_abort_semantics_present = false;

    let mut target_selection_missing = valid;
    target_selection_missing.rollback_target_selection_present = false;
    let mut transaction_id_missing = valid;
    transaction_id_missing.rollback_transaction_id_provenance_present = false;
    let mut last_good_missing = valid;
    last_good_missing.rollback_last_good_binding_present = false;
    let mut disabled_set_missing = valid;
    disabled_set_missing.rollback_disabled_module_set_binding_present = false;
    let mut artifact_hash_missing = valid;
    artifact_hash_missing.rollback_artifact_hash_binding_present = false;
    let mut replay_missing = valid;
    replay_missing.rollback_replay_preconditions_present = false;
    let mut capability_import_missing = valid;
    capability_import_missing.rollback_recovery_capability_import_present = false;
    let mut atomic_semantics_missing = valid;
    atomic_semantics_missing.rollback_atomic_apply_abort_semantics_present = false;
    let mut durable_persistence_missing = valid;
    durable_persistence_missing.durable_audit_rollback_persistence_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_rollback_transaction_engine_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_rollback_transaction_engine(missing_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_rollback_transaction_engine(stale_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_rollback_transaction_engine(request_hash_mismatch),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_rollback_transaction_engine(missing_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_rollback_transaction_engine(missing_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_rollback_transaction_engine(direct_provider_shortcut),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_rollback_transaction_engine(missing_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_rollback_transaction_engine(address_space_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_rollback_transaction_engine(entrypoint_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_rollback_transaction_engine(memory_map_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_rollback_transaction_engine(import_table_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_rollback_transaction_engine(artifact_hash_isolation_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_rollback_transaction_engine(provider_separation_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_rollback_transaction_engine(normal_module_separation_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_transaction_engine_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_rollback_transaction_engine(all_transaction_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_target_selection_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            evaluate_recovery_rollback_transaction_engine(target_selection_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_transaction_id_provenance_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            evaluate_recovery_rollback_transaction_engine(transaction_id_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_last_good_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            evaluate_recovery_rollback_transaction_engine(last_good_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_disabled_module_set_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            evaluate_recovery_rollback_transaction_engine(disabled_set_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_artifact_hash_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            evaluate_recovery_rollback_transaction_engine(artifact_hash_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_replay_preconditions_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            evaluate_recovery_rollback_transaction_engine(replay_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_recovery_capability_import_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            evaluate_recovery_rollback_transaction_engine(capability_import_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_atomic_apply_abort_semantics_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            evaluate_recovery_rollback_transaction_engine(atomic_semantics_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_rollback_transaction_engine(durable_persistence_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_rollback_transaction_engine(memory_provenance_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "all_inputs_present_rollback_still_non_executable",
            "defined_non_executable",
            "recovery_rollback_transaction_behavior_not_implemented",
            evaluate_recovery_rollback_transaction_engine(valid),
        ),
    ]
}

fn recovery_rollback_transaction_engine_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryRollbackTransactionEngineCheck,
) -> RecoveryRollbackTransactionEngineSelfTestCase {
    RecoveryRollbackTransactionEngineSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

fn recovery_durable_audit_rollback_persistence_candidate_from_transaction(
    transaction_candidate: RecoveryRollbackTransactionEngineCandidate,
) -> RecoveryDurableAuditRollbackPersistenceCandidate {
    RecoveryDurableAuditRollbackPersistenceCandidate {
        transaction_candidate,
        rollback_transaction_engine_available: true,
        rollback_transaction_engine_current_boot: true,
        rollback_transaction_engine_schema_ok: true,
        rollback_transaction_engine_binding_ok: true,
        rollback_transaction_engine_binding_reason:
            "recovery_rollback_transaction_engine_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        persistence_device_inventory_present: false,
        storage_layout_identity_present: false,
        audit_append_log_identity_present: false,
        rollback_store_identity_present: false,
        transaction_replay_cursor_present: false,
        last_good_checkpoint_binding_present: false,
        write_ordering_present: false,
        crash_consistency_present: false,
        integrity_root_hash_chain_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn evaluate_recovery_durable_audit_rollback_persistence(
    candidate: RecoveryDurableAuditRollbackPersistenceCandidate,
) -> RecoveryDurableAuditRollbackPersistenceCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(
        candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate,
    );
    let command_check = evaluate_recovery_lifeline_command_vocabulary(
        candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate,
    );
    let loader_check = evaluate_recovery_loader_runtime_isolation(
        candidate.transaction_candidate.loader_candidate,
    );
    let transaction_check =
        evaluate_recovery_rollback_transaction_engine(candidate.transaction_candidate);

    if !protocol_check.request_chain_valid {
        return recovery_durable_audit_rollback_persistence_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .loader_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }

    let command_vocabulary_envelope_exposed = command_check.command_vocabulary_exposed
        || candidate
            .transaction_candidate
            .loader_candidate
            .command_vocabulary_available;
    let loader_runtime_isolation_boundary_exposed = loader_check.isolation_requirements_exposed
        || candidate
            .transaction_candidate
            .loader_runtime_isolation_available;
    let rollback_transaction_engine_boundary_exposed = transaction_check
        .transaction_requirements_exposed
        || candidate.rollback_transaction_engine_available;

    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            command_vocabulary_envelope_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_reason,
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available
        || !command_check.command_vocabulary_exposed
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_binding_reason,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }

    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_available
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_current_boot
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_schema_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_reason,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !loader_check.loader_runtime_isolation_ready {
        return recovery_durable_audit_rollback_persistence_check(
            loader_check.status,
            loader_check.reason,
            true,
            true,
            true,
            true,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            false,
            false,
        );
    }

    if !candidate.rollback_transaction_engine_available {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.rollback_transaction_engine_current_boot {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.rollback_transaction_engine_schema_ok {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.rollback_transaction_engine_binding_ok {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate.rollback_transaction_engine_binding_reason,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !transaction_check.rollback_transaction_engine_ready {
        return recovery_durable_audit_rollback_persistence_check(
            transaction_check.status,
            transaction_check.reason,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }

    if !candidate.persistence_device_inventory_present
        && !candidate.storage_layout_identity_present
        && !candidate.audit_append_log_identity_present
        && !candidate.rollback_store_identity_present
        && !candidate.transaction_replay_cursor_present
        && !candidate.last_good_checkpoint_binding_present
        && !candidate.write_ordering_present
        && !candidate.crash_consistency_present
        && !candidate.integrity_root_hash_chain_present
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.persistence_device_inventory_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "persistence_device_inventory_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.storage_layout_identity_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_storage_layout_identity_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.audit_append_log_identity_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_append_log_identity_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_store_identity_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_store_identity_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.transaction_replay_cursor_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_transaction_replay_cursor_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.last_good_checkpoint_binding_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "recovery_last_good_checkpoint_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.write_ordering_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_write_ordering_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.crash_consistency_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_crash_consistency_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.integrity_root_hash_chain_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_integrity_root_hash_chain_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        );
    }
    recovery_durable_audit_rollback_persistence_check(
        "defined_non_executable",
        "durable_audit_rollback_persistence_behavior_not_implemented",
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

fn recovery_durable_audit_rollback_persistence_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    rollback_transaction_engine_boundary_exposed: bool,
    rollback_transaction_engine_accepted: bool,
    persistence_requirements_exposed: bool,
    durable_audit_rollback_persistence_ready: bool,
) -> RecoveryDurableAuditRollbackPersistenceCheck {
    RecoveryDurableAuditRollbackPersistenceCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        loader_runtime_isolation_boundary_exposed,
        loader_runtime_isolation_accepted,
        rollback_transaction_engine_boundary_exposed,
        rollback_transaction_engine_accepted,
        persistence_requirements_exposed,
        durable_audit_rollback_persistence_ready,
        durable_writes_enabled: false,
        rollback_replay_enabled: false,
        recovery_memory_writes_enabled: false,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
        command_execution_enabled: false,
        accepts_lifeline_command_envelope: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn recovery_durable_audit_rollback_persistence_valid_candidate(
) -> RecoveryDurableAuditRollbackPersistenceCandidate {
    RecoveryDurableAuditRollbackPersistenceCandidate {
        transaction_candidate: recovery_rollback_transaction_engine_valid_candidate(),
        rollback_transaction_engine_available: true,
        rollback_transaction_engine_current_boot: true,
        rollback_transaction_engine_schema_ok: true,
        rollback_transaction_engine_binding_ok: true,
        rollback_transaction_engine_binding_reason:
            "retained_recovery_rollback_transaction_engine_valid",
        direct_openai_recovery_shortcut_used: false,
        persistence_device_inventory_present: true,
        storage_layout_identity_present: true,
        audit_append_log_identity_present: true,
        rollback_store_identity_present: true,
        transaction_replay_cursor_present: true,
        last_good_checkpoint_binding_present: true,
        write_ordering_present: true,
        crash_consistency_present: true,
        integrity_root_hash_chain_present: true,
        recovery_memory_provenance_present: true,
    }
}

fn recovery_durable_audit_rollback_persistence_selftest_cases(
) -> [RecoveryDurableAuditRollbackPersistenceSelfTestCase;
       RECOVERY_DURABLE_AUDIT_ROLLBACK_PERSISTENCE_SELFTEST_CASES] {
    let valid = recovery_durable_audit_rollback_persistence_valid_candidate();

    let mut missing_request = valid;
    missing_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok = false;
    substituted_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut missing_loader_runtime_isolation = valid;
    missing_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_available = false;
    let mut previous_loader_runtime_isolation = valid;
    previous_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_current_boot = false;
    let mut wrong_schema_loader_runtime_isolation = valid;
    wrong_schema_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_schema_ok = false;
    let mut substituted_loader_runtime_isolation = valid;
    substituted_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_binding_ok = false;
    substituted_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_substituted_record";

    let mut address_space_missing = valid;
    address_space_missing
        .transaction_candidate
        .loader_candidate
        .loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing
        .transaction_candidate
        .loader_candidate
        .loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing
        .transaction_candidate
        .loader_candidate
        .loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing
        .transaction_candidate
        .loader_candidate
        .loader_capability_import_table_present = false;
    let mut artifact_hash_isolation_missing = valid;
    artifact_hash_isolation_missing
        .transaction_candidate
        .loader_candidate
        .loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing
        .transaction_candidate
        .loader_candidate
        .loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing
        .transaction_candidate
        .loader_candidate
        .loader_normal_module_separation_present = false;

    let mut missing_rollback_engine_boundary = valid;
    missing_rollback_engine_boundary.rollback_transaction_engine_available = false;
    let mut previous_rollback_engine = valid;
    previous_rollback_engine.rollback_transaction_engine_current_boot = false;
    let mut wrong_schema_rollback_engine = valid;
    wrong_schema_rollback_engine.rollback_transaction_engine_schema_ok = false;
    let mut substituted_rollback_engine = valid;
    substituted_rollback_engine.rollback_transaction_engine_binding_ok = false;
    substituted_rollback_engine.rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_substituted_record";
    let mut mismatched_rollback_engine = valid;
    mismatched_rollback_engine.rollback_transaction_engine_binding_ok = false;
    mismatched_rollback_engine.rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_binding_mismatch";

    let mut target_selection_missing = valid;
    target_selection_missing
        .transaction_candidate
        .rollback_target_selection_present = false;
    let mut transaction_id_missing = valid;
    transaction_id_missing
        .transaction_candidate
        .rollback_transaction_id_provenance_present = false;
    let mut last_good_missing = valid;
    last_good_missing
        .transaction_candidate
        .rollback_last_good_binding_present = false;
    let mut disabled_set_missing = valid;
    disabled_set_missing
        .transaction_candidate
        .rollback_disabled_module_set_binding_present = false;
    let mut artifact_hash_missing = valid;
    artifact_hash_missing
        .transaction_candidate
        .rollback_artifact_hash_binding_present = false;
    let mut replay_missing = valid;
    replay_missing
        .transaction_candidate
        .rollback_replay_preconditions_present = false;
    let mut capability_import_missing = valid;
    capability_import_missing
        .transaction_candidate
        .rollback_recovery_capability_import_present = false;
    let mut atomic_semantics_missing = valid;
    atomic_semantics_missing
        .transaction_candidate
        .rollback_atomic_apply_abort_semantics_present = false;

    let mut all_persistence_missing = valid;
    all_persistence_missing.persistence_device_inventory_present = false;
    all_persistence_missing.storage_layout_identity_present = false;
    all_persistence_missing.audit_append_log_identity_present = false;
    all_persistence_missing.rollback_store_identity_present = false;
    all_persistence_missing.transaction_replay_cursor_present = false;
    all_persistence_missing.last_good_checkpoint_binding_present = false;
    all_persistence_missing.write_ordering_present = false;
    all_persistence_missing.crash_consistency_present = false;
    all_persistence_missing.integrity_root_hash_chain_present = false;
    let mut persistence_device_missing = valid;
    persistence_device_missing.persistence_device_inventory_present = false;
    let mut storage_layout_missing = valid;
    storage_layout_missing.storage_layout_identity_present = false;
    let mut audit_log_missing = valid;
    audit_log_missing.audit_append_log_identity_present = false;
    let mut rollback_store_missing = valid;
    rollback_store_missing.rollback_store_identity_present = false;
    let mut replay_cursor_missing = valid;
    replay_cursor_missing.transaction_replay_cursor_present = false;
    let mut checkpoint_missing = valid;
    checkpoint_missing.last_good_checkpoint_binding_present = false;
    let mut write_ordering_missing = valid;
    write_ordering_missing.write_ordering_present = false;
    let mut crash_consistency_missing = valid;
    crash_consistency_missing.crash_consistency_present = false;
    let mut integrity_root_missing = valid;
    integrity_root_missing.integrity_root_hash_chain_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_durable_audit_rollback_persistence_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_durable_audit_rollback_persistence(stale_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_durable_audit_rollback_persistence(request_hash_mismatch),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_durable_audit_rollback_persistence(direct_provider_shortcut),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_loader_runtime_isolation),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_loader_runtime_isolation),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(
                wrong_schema_loader_runtime_isolation,
            ),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(
                substituted_loader_runtime_isolation,
            ),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_durable_audit_rollback_persistence(address_space_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_durable_audit_rollback_persistence(entrypoint_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_durable_audit_rollback_persistence(memory_map_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_durable_audit_rollback_persistence(import_table_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(artifact_hash_isolation_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_durable_audit_rollback_persistence(provider_separation_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_durable_audit_rollback_persistence(normal_module_separation_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_transaction_engine_boundary_missing_after_loader",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_rollback_engine_boundary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "mismatched_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_binding_mismatch",
            evaluate_recovery_durable_audit_rollback_persistence(mismatched_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_target_selection_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            evaluate_recovery_durable_audit_rollback_persistence(target_selection_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_transaction_id_provenance_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            evaluate_recovery_durable_audit_rollback_persistence(transaction_id_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_last_good_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(last_good_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_disabled_module_set_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(disabled_set_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_artifact_hash_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(artifact_hash_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_replay_preconditions_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            evaluate_recovery_durable_audit_rollback_persistence(replay_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_recovery_capability_import_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            evaluate_recovery_durable_audit_rollback_persistence(capability_import_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_atomic_apply_abort_semantics_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            evaluate_recovery_durable_audit_rollback_persistence(atomic_semantics_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_durable_audit_rollback_persistence(all_persistence_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "persistence_device_inventory_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "persistence_device_inventory_missing",
            evaluate_recovery_durable_audit_rollback_persistence(persistence_device_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "storage_layout_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_storage_layout_identity_missing",
            evaluate_recovery_durable_audit_rollback_persistence(storage_layout_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "audit_append_log_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_append_log_identity_missing",
            evaluate_recovery_durable_audit_rollback_persistence(audit_log_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_store_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_store_identity_missing",
            evaluate_recovery_durable_audit_rollback_persistence(rollback_store_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "transaction_replay_cursor_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_transaction_replay_cursor_missing",
            evaluate_recovery_durable_audit_rollback_persistence(replay_cursor_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "last_good_checkpoint_binding_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "recovery_last_good_checkpoint_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(checkpoint_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "write_ordering_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_write_ordering_missing",
            evaluate_recovery_durable_audit_rollback_persistence(write_ordering_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "crash_consistency_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_crash_consistency_missing",
            evaluate_recovery_durable_audit_rollback_persistence(crash_consistency_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "integrity_root_hash_chain_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_integrity_root_hash_chain_missing",
            evaluate_recovery_durable_audit_rollback_persistence(integrity_root_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_durable_audit_rollback_persistence(memory_provenance_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "all_inputs_present_persistence_still_non_executable",
            "defined_non_executable",
            "durable_audit_rollback_persistence_behavior_not_implemented",
            evaluate_recovery_durable_audit_rollback_persistence(valid),
        ),
    ]
}

fn recovery_durable_audit_rollback_persistence_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryDurableAuditRollbackPersistenceCheck,
) -> RecoveryDurableAuditRollbackPersistenceSelfTestCase {
    RecoveryDurableAuditRollbackPersistenceSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

fn recovery_memory_provenance_candidate_from_persistence(
    persistence_candidate: RecoveryDurableAuditRollbackPersistenceCandidate,
) -> RecoveryMemoryProvenanceCandidate {
    RecoveryMemoryProvenanceCandidate {
        persistence_candidate,
        durable_audit_rollback_persistence_available: true,
        durable_audit_rollback_persistence_current_boot: true,
        durable_audit_rollback_persistence_schema_ok: true,
        durable_audit_rollback_persistence_binding_ok: true,
        durable_audit_rollback_persistence_binding_reason:
            "durable_audit_rollback_persistence_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        source_record_ids_present: false,
        source_schema_hashes_present: false,
        memory_classification_present: false,
        memory_authority_level_present: false,
        memory_rollback_transaction_binding_present: false,
        memory_last_good_checkpoint_binding_present: false,
        recovery_only_export_profile_present: false,
        memory_redaction_state_present: false,
        memory_replay_window_present: false,
        memory_audit_linkage_present: false,
    }
}

fn evaluate_recovery_memory_provenance(
    candidate: RecoveryMemoryProvenanceCandidate,
) -> RecoveryMemoryProvenanceCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(
        candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate,
    );
    let command_check = evaluate_recovery_lifeline_command_vocabulary(
        candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate,
    );
    let loader_check = evaluate_recovery_loader_runtime_isolation(
        candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate,
    );
    let transaction_check = evaluate_recovery_rollback_transaction_engine(
        candidate.persistence_candidate.transaction_candidate,
    );
    let persistence_check =
        evaluate_recovery_durable_audit_rollback_persistence(candidate.persistence_candidate);

    if !protocol_check.request_chain_valid {
        return recovery_memory_provenance_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_memory_provenance_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }

    let command_vocabulary_envelope_exposed = command_check.command_vocabulary_exposed
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_vocabulary_available;
    let loader_runtime_isolation_boundary_exposed = loader_check.isolation_requirements_exposed
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_runtime_isolation_available;
    let rollback_transaction_engine_boundary_exposed = transaction_check
        .transaction_requirements_exposed
        || candidate
            .persistence_candidate
            .rollback_transaction_engine_available;
    let durable_audit_rollback_persistence_boundary_exposed = persistence_check
        .persistence_requirements_exposed
        || candidate.durable_audit_rollback_persistence_available;

    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained
    {
        return recovery_memory_provenance_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            command_vocabulary_envelope_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_reason,
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available
        || !command_check.command_vocabulary_exposed
    {
        return recovery_memory_provenance_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_binding_reason,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }

    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_available
    {
        return recovery_memory_provenance_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_reason,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !loader_check.loader_runtime_isolation_ready {
        return recovery_memory_provenance_check(
            loader_check.status,
            loader_check.reason,
            true,
            true,
            true,
            true,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
            false,
            false,
        );
    }

    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_available
    {
        return recovery_memory_provenance_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .rollback_transaction_engine_binding_reason,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !transaction_check.rollback_transaction_engine_ready {
        return recovery_memory_provenance_check(
            transaction_check.status,
            transaction_check.reason,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
            false,
            false,
        );
    }

    if !candidate.durable_audit_rollback_persistence_available {
        return recovery_memory_provenance_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.durable_audit_rollback_persistence_current_boot {
        return recovery_memory_provenance_check(
            "rejected",
            "durable_audit_rollback_persistence_event_id_not_current_boot",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.durable_audit_rollback_persistence_schema_ok {
        return recovery_memory_provenance_check(
            "rejected",
            "durable_audit_rollback_persistence_wrong_schema_or_variant",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.durable_audit_rollback_persistence_binding_ok {
        return recovery_memory_provenance_check(
            "rejected",
            candidate.durable_audit_rollback_persistence_binding_reason,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !persistence_check.durable_audit_rollback_persistence_ready {
        return recovery_memory_provenance_check(
            persistence_check.status,
            persistence_check.reason,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            persistence_check.persistence_requirements_exposed,
            false,
        );
    }

    if !candidate.source_record_ids_present
        && !candidate.source_schema_hashes_present
        && !candidate.memory_classification_present
        && !candidate.memory_authority_level_present
        && !candidate.memory_rollback_transaction_binding_present
        && !candidate.memory_last_good_checkpoint_binding_present
        && !candidate.recovery_only_export_profile_present
        && !candidate.memory_redaction_state_present
        && !candidate.memory_replay_window_present
        && !candidate.memory_audit_linkage_present
    {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.source_record_ids_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_record_ids_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.source_schema_hashes_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_schema_hashes_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.memory_classification_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_classification_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.memory_authority_level_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_authority_level_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.memory_rollback_transaction_binding_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_rollback_transaction_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.memory_last_good_checkpoint_binding_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_last_good_checkpoint_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.recovery_only_export_profile_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_only_export_profile_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.memory_redaction_state_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_redaction_state_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.memory_replay_window_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_replay_window_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.memory_audit_linkage_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_audit_linkage_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }

    recovery_memory_provenance_check(
        "defined_non_executable",
        "recovery_memory_provenance_behavior_not_implemented",
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

fn recovery_memory_provenance_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    rollback_transaction_engine_boundary_exposed: bool,
    rollback_transaction_engine_accepted: bool,
    durable_audit_rollback_persistence_boundary_exposed: bool,
    durable_audit_rollback_persistence_accepted: bool,
    memory_provenance_requirements_exposed: bool,
    recovery_memory_provenance_ready: bool,
) -> RecoveryMemoryProvenanceCheck {
    RecoveryMemoryProvenanceCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        loader_runtime_isolation_boundary_exposed,
        loader_runtime_isolation_accepted,
        rollback_transaction_engine_boundary_exposed,
        rollback_transaction_engine_accepted,
        durable_audit_rollback_persistence_boundary_exposed,
        durable_audit_rollback_persistence_accepted,
        memory_provenance_requirements_exposed,
        recovery_memory_provenance_ready,
        memory_writes_enabled: false,
        provider_export_enabled: false,
        durable_writes_enabled: false,
        rollback_replay_enabled: false,
        recovery_memory_writes_enabled: false,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
        command_execution_enabled: false,
        accepts_lifeline_command_envelope: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn recovery_memory_provenance_valid_candidate() -> RecoveryMemoryProvenanceCandidate {
    RecoveryMemoryProvenanceCandidate {
        persistence_candidate: recovery_durable_audit_rollback_persistence_valid_candidate(),
        durable_audit_rollback_persistence_available: true,
        durable_audit_rollback_persistence_current_boot: true,
        durable_audit_rollback_persistence_schema_ok: true,
        durable_audit_rollback_persistence_binding_ok: true,
        durable_audit_rollback_persistence_binding_reason:
            "retained_durable_audit_rollback_persistence_valid",
        direct_openai_recovery_shortcut_used: false,
        source_record_ids_present: true,
        source_schema_hashes_present: true,
        memory_classification_present: true,
        memory_authority_level_present: true,
        memory_rollback_transaction_binding_present: true,
        memory_last_good_checkpoint_binding_present: true,
        recovery_only_export_profile_present: true,
        memory_redaction_state_present: true,
        memory_replay_window_present: true,
        memory_audit_linkage_present: true,
    }
}

fn recovery_memory_provenance_selftest_cases(
) -> [RecoveryMemoryProvenanceSelfTestCase; RECOVERY_MEMORY_PROVENANCE_SELFTEST_CASES] {
    let valid = recovery_memory_provenance_valid_candidate();

    let mut missing_request = valid;
    missing_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok = false;
    substituted_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut missing_loader_runtime_isolation = valid;
    missing_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_available = false;
    let mut previous_loader_runtime_isolation = valid;
    previous_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_current_boot = false;
    let mut wrong_schema_loader_runtime_isolation = valid;
    wrong_schema_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_schema_ok = false;
    let mut substituted_loader_runtime_isolation = valid;
    substituted_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok = false;
    substituted_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_substituted_record";

    let mut address_space_missing = valid;
    address_space_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_capability_import_table_present = false;
    let mut artifact_hash_isolation_missing = valid;
    artifact_hash_isolation_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_normal_module_separation_present = false;

    let mut missing_rollback_engine_boundary = valid;
    missing_rollback_engine_boundary
        .persistence_candidate
        .rollback_transaction_engine_available = false;
    let mut previous_rollback_engine = valid;
    previous_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_current_boot = false;
    let mut wrong_schema_rollback_engine = valid;
    wrong_schema_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_schema_ok = false;
    let mut substituted_rollback_engine = valid;
    substituted_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_ok = false;
    substituted_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_substituted_record";
    let mut mismatched_rollback_engine = valid;
    mismatched_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_ok = false;
    mismatched_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_binding_mismatch";

    let mut target_selection_missing = valid;
    target_selection_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_target_selection_present = false;
    let mut transaction_id_missing = valid;
    transaction_id_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_transaction_id_provenance_present = false;
    let mut last_good_missing = valid;
    last_good_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_last_good_binding_present = false;
    let mut disabled_set_missing = valid;
    disabled_set_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_disabled_module_set_binding_present = false;
    let mut artifact_hash_missing = valid;
    artifact_hash_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_artifact_hash_binding_present = false;
    let mut replay_missing = valid;
    replay_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_replay_preconditions_present = false;
    let mut capability_import_missing = valid;
    capability_import_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_recovery_capability_import_present = false;
    let mut atomic_semantics_missing = valid;
    atomic_semantics_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_atomic_apply_abort_semantics_present = false;

    let mut durable_boundary_missing = valid;
    durable_boundary_missing.durable_audit_rollback_persistence_available = false;
    let mut previous_durable_persistence = valid;
    previous_durable_persistence.durable_audit_rollback_persistence_current_boot = false;
    let mut wrong_schema_durable_persistence = valid;
    wrong_schema_durable_persistence.durable_audit_rollback_persistence_schema_ok = false;
    let mut substituted_durable_persistence = valid;
    substituted_durable_persistence.durable_audit_rollback_persistence_binding_ok = false;
    substituted_durable_persistence.durable_audit_rollback_persistence_binding_reason =
        "durable_audit_rollback_persistence_substituted_record";
    let mut mismatched_durable_persistence = valid;
    mismatched_durable_persistence.durable_audit_rollback_persistence_binding_ok = false;
    mismatched_durable_persistence.durable_audit_rollback_persistence_binding_reason =
        "durable_audit_rollback_persistence_binding_mismatch";

    let mut persistence_device_missing = valid;
    persistence_device_missing
        .persistence_candidate
        .persistence_device_inventory_present = false;
    let mut storage_layout_missing = valid;
    storage_layout_missing
        .persistence_candidate
        .storage_layout_identity_present = false;
    let mut audit_log_missing = valid;
    audit_log_missing
        .persistence_candidate
        .audit_append_log_identity_present = false;
    let mut rollback_store_missing = valid;
    rollback_store_missing
        .persistence_candidate
        .rollback_store_identity_present = false;
    let mut replay_cursor_missing = valid;
    replay_cursor_missing
        .persistence_candidate
        .transaction_replay_cursor_present = false;
    let mut checkpoint_missing = valid;
    checkpoint_missing
        .persistence_candidate
        .last_good_checkpoint_binding_present = false;
    let mut write_ordering_missing = valid;
    write_ordering_missing
        .persistence_candidate
        .write_ordering_present = false;
    let mut crash_consistency_missing = valid;
    crash_consistency_missing
        .persistence_candidate
        .crash_consistency_present = false;
    let mut integrity_root_missing = valid;
    integrity_root_missing
        .persistence_candidate
        .integrity_root_hash_chain_present = false;
    let mut all_memory_provenance_missing = valid;
    all_memory_provenance_missing.source_record_ids_present = false;
    all_memory_provenance_missing.source_schema_hashes_present = false;
    all_memory_provenance_missing.memory_classification_present = false;
    all_memory_provenance_missing.memory_authority_level_present = false;
    all_memory_provenance_missing.memory_rollback_transaction_binding_present = false;
    all_memory_provenance_missing.memory_last_good_checkpoint_binding_present = false;
    all_memory_provenance_missing.recovery_only_export_profile_present = false;
    all_memory_provenance_missing.memory_redaction_state_present = false;
    all_memory_provenance_missing.memory_replay_window_present = false;
    all_memory_provenance_missing.memory_audit_linkage_present = false;

    let mut source_record_ids_missing = valid;
    source_record_ids_missing.source_record_ids_present = false;
    let mut source_schema_hashes_missing = valid;
    source_schema_hashes_missing.source_schema_hashes_present = false;
    let mut memory_classification_missing = valid;
    memory_classification_missing.memory_classification_present = false;
    let mut memory_authority_missing = valid;
    memory_authority_missing.memory_authority_level_present = false;
    let mut memory_rollback_binding_missing = valid;
    memory_rollback_binding_missing.memory_rollback_transaction_binding_present = false;
    let mut memory_checkpoint_binding_missing = valid;
    memory_checkpoint_binding_missing.memory_last_good_checkpoint_binding_present = false;
    let mut export_profile_missing = valid;
    export_profile_missing.recovery_only_export_profile_present = false;
    let mut redaction_state_missing = valid;
    redaction_state_missing.memory_redaction_state_present = false;
    let mut replay_window_missing = valid;
    replay_window_missing.memory_replay_window_present = false;
    let mut audit_linkage_missing = valid;
    audit_linkage_missing.memory_audit_linkage_present = false;

    [
        recovery_memory_provenance_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_memory_provenance(missing_request),
        ),
        recovery_memory_provenance_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_memory_provenance(stale_request),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_request),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_request),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_memory_provenance(substituted_request),
        ),
        recovery_memory_provenance_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_memory_provenance(request_hash_mismatch),
        ),
        recovery_memory_provenance_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_memory_provenance(missing_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_memory_provenance(substituted_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_memory_provenance(missing_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_memory_provenance(substituted_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_memory_provenance(direct_provider_shortcut),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_memory_provenance(missing_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_substituted_record",
            evaluate_recovery_memory_provenance(substituted_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_memory_provenance(address_space_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_memory_provenance(entrypoint_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_memory_provenance(memory_map_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_memory_provenance(import_table_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_memory_provenance(artifact_hash_isolation_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_memory_provenance(provider_separation_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_memory_provenance(normal_module_separation_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_transaction_engine_boundary_missing_after_loader",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_memory_provenance(missing_rollback_engine_boundary),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_substituted_record",
            evaluate_recovery_memory_provenance(substituted_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "mismatched_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_binding_mismatch",
            evaluate_recovery_memory_provenance(mismatched_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_target_selection_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            evaluate_recovery_memory_provenance(target_selection_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_transaction_id_provenance_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            evaluate_recovery_memory_provenance(transaction_id_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_last_good_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            evaluate_recovery_memory_provenance(last_good_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_disabled_module_set_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            evaluate_recovery_memory_provenance(disabled_set_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_artifact_hash_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            evaluate_recovery_memory_provenance(artifact_hash_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_replay_preconditions_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            evaluate_recovery_memory_provenance(replay_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_recovery_capability_import_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            evaluate_recovery_memory_provenance(capability_import_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_atomic_apply_abort_semantics_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            evaluate_recovery_memory_provenance(atomic_semantics_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "durable_persistence_boundary_missing_after_rollback_engine",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_memory_provenance(durable_boundary_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_substituted_record",
            evaluate_recovery_memory_provenance(substituted_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "mismatched_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_binding_mismatch",
            evaluate_recovery_memory_provenance(mismatched_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "persistence_device_inventory_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "persistence_device_inventory_missing",
            evaluate_recovery_memory_provenance(persistence_device_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "storage_layout_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_storage_layout_identity_missing",
            evaluate_recovery_memory_provenance(storage_layout_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "audit_append_log_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_append_log_identity_missing",
            evaluate_recovery_memory_provenance(audit_log_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_store_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_store_identity_missing",
            evaluate_recovery_memory_provenance(rollback_store_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "transaction_replay_cursor_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_transaction_replay_cursor_missing",
            evaluate_recovery_memory_provenance(replay_cursor_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "last_good_checkpoint_binding_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "recovery_last_good_checkpoint_binding_missing",
            evaluate_recovery_memory_provenance(checkpoint_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "write_ordering_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_write_ordering_missing",
            evaluate_recovery_memory_provenance(write_ordering_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "crash_consistency_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_crash_consistency_missing",
            evaluate_recovery_memory_provenance(crash_consistency_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "integrity_root_hash_chain_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_integrity_root_hash_chain_missing",
            evaluate_recovery_memory_provenance(integrity_root_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_memory_provenance(all_memory_provenance_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "source_record_ids_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_record_ids_missing",
            evaluate_recovery_memory_provenance(source_record_ids_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "source_schema_hashes_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_schema_hashes_missing",
            evaluate_recovery_memory_provenance(source_schema_hashes_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_classification_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_classification_missing",
            evaluate_recovery_memory_provenance(memory_classification_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_authority_level_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_authority_level_missing",
            evaluate_recovery_memory_provenance(memory_authority_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_rollback_transaction_binding_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_rollback_transaction_binding_missing",
            evaluate_recovery_memory_provenance(memory_rollback_binding_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_last_good_checkpoint_binding_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_last_good_checkpoint_binding_missing",
            evaluate_recovery_memory_provenance(memory_checkpoint_binding_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "recovery_only_export_profile_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_only_export_profile_missing",
            evaluate_recovery_memory_provenance(export_profile_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_redaction_state_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_redaction_state_missing",
            evaluate_recovery_memory_provenance(redaction_state_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_replay_window_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_replay_window_missing",
            evaluate_recovery_memory_provenance(replay_window_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_audit_linkage_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_audit_linkage_missing",
            evaluate_recovery_memory_provenance(audit_linkage_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "all_inputs_present_memory_still_non_executable",
            "defined_non_executable",
            "recovery_memory_provenance_behavior_not_implemented",
            evaluate_recovery_memory_provenance(valid),
        ),
    ]
}

fn recovery_memory_provenance_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryMemoryProvenanceCheck,
) -> RecoveryMemoryProvenanceSelfTestCase {
    RecoveryMemoryProvenanceSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

fn recovery_load_binding_selftest_cases(
) -> [RecoveryLoadBindingSelfTestCase; RECOVERY_LOAD_BINDING_SELFTEST_CASES] {
    let valid = recovery_load_binding_available_candidate();

    let mut identity_missing = valid;
    identity_missing.identity = recovery_evidence_missing();
    let mut identity_previous_boot = valid;
    identity_previous_boot.identity.current_boot = false;
    let mut identity_wrong_schema = valid;
    identity_wrong_schema.identity.schema_ok = false;

    let mut trust_missing = valid;
    trust_missing.trust = recovery_evidence_missing();
    let mut vm_test_missing = valid;
    vm_test_missing.vm_test = recovery_evidence_missing();
    let mut local_approval_missing = valid;
    local_approval_missing.local_approval = recovery_evidence_missing();
    let mut loader_missing = valid;
    loader_missing.loader = recovery_evidence_missing();
    let mut rollback_missing = valid;
    rollback_missing.rollback_evidence = recovery_evidence_missing();

    let mut module_capability = valid;
    module_capability.requested_capability = "cap.module.load_ephemeral";
    module_capability.normal_module_capability_substituted = true;
    let mut module_append_intent = valid;
    module_append_intent.normal_module_append_intent_substituted = true;
    let mut append_payload_hash = valid;
    append_payload_hash.append_payload_hash_claimed_authority = true;
    let mut module_writer_facts = valid;
    module_writer_facts.normal_module_writer_facts_substituted = true;
    let mut module_service_slot = valid;
    module_service_slot.normal_module_service_slot_substituted = true;

    [
        recovery_load_binding_selftest_case(
            "missing_recovery_artifact_identity_event_id",
            "missing",
            "recovery_artifact_identity_event_id_missing",
            identity_missing,
        ),
        recovery_load_binding_selftest_case(
            "previous_boot_recovery_artifact_identity_event_id",
            "rejected",
            "recovery_artifact_identity_event_id_not_current_boot",
            identity_previous_boot,
        ),
        recovery_load_binding_selftest_case(
            "wrong_schema_recovery_artifact_identity_event_id",
            "rejected",
            "recovery_artifact_identity_schema_mismatch",
            identity_wrong_schema,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_artifact_trust_event_id",
            "missing",
            "recovery_artifact_trust_event_id_missing",
            trust_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_vm_test_event_id",
            "missing",
            "recovery_vm_test_event_id_missing",
            vm_test_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_local_approval_event_id",
            "missing",
            "recovery_local_approval_event_id_missing",
            local_approval_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_loader_event_id",
            "missing",
            "recovery_loader_event_id_missing",
            loader_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_rollback_evidence_event_id",
            "missing",
            "recovery_rollback_evidence_event_id_missing",
            rollback_missing,
        ),
        recovery_load_binding_selftest_case(
            "module_load_ephemeral_capability_substituted",
            "rejected",
            "recovery_load_requires_cap_recovery_load_artifact",
            module_capability,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_append_intent_substituted",
            "rejected",
            "normal_module_append_intent_not_recovery_authority",
            module_append_intent,
        ),
        recovery_load_binding_selftest_case(
            "append_payload_hash_claimed_as_authority",
            "rejected",
            "append_payload_hash_not_recovery_authority",
            append_payload_hash,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_writer_facts_substituted",
            "rejected",
            "normal_module_writer_facts_not_recovery_authority",
            module_writer_facts,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_service_slot_substituted",
            "rejected",
            "normal_module_service_slot_not_recovery_authority",
            module_service_slot,
        ),
        recovery_load_binding_selftest_case(
            "available_recovery_binding_still_denied",
            "available_non_authorizing",
            "recovery_lifeline_protocol_missing",
            valid,
        ),
    ]
}

fn recovery_load_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: RecoveryLoadBindingCandidate,
) -> RecoveryLoadBindingSelfTestCase {
    let actual = evaluate_recovery_load_binding(candidate);
    RecoveryLoadBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_move_beyond_denial
            && !actual.accepts_normal_module_authority
            && !actual.append_payload_hash_authority
            && !actual.loads_recovery_artifact
            && !actual.loads_normal_module
            && !actual.creates_durable_records
            && !actual.installs_rollback_plan
            && method_eq(actual.service_inventory_change, "none")
            && !actual.load_attempted,
    }
}

fn evaluate_recovery_load_binding(
    candidate: RecoveryLoadBindingCandidate,
) -> RecoveryLoadBindingCheck {
    let (status, reason) = if !method_eq(
        candidate.requested_capability,
        RECOVERY_ARTIFACT_LOAD_CAPABILITY,
    ) || candidate.normal_module_capability_substituted
    {
        (
            "rejected",
            "recovery_load_requires_cap_recovery_load_artifact",
        )
    } else if candidate.normal_module_append_intent_substituted {
        (
            "rejected",
            "normal_module_append_intent_not_recovery_authority",
        )
    } else if candidate.append_payload_hash_claimed_authority {
        ("rejected", "append_payload_hash_not_recovery_authority")
    } else if candidate.normal_module_writer_facts_substituted {
        (
            "rejected",
            "normal_module_writer_facts_not_recovery_authority",
        )
    } else if candidate.normal_module_service_slot_substituted {
        (
            "rejected",
            "normal_module_service_slot_not_recovery_authority",
        )
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.identity,
        "missing",
        "recovery_artifact_identity_event_id_missing",
        "rejected",
        "recovery_artifact_identity_event_id_not_current_boot",
        "recovery_artifact_identity_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.trust,
        "missing",
        "recovery_artifact_trust_event_id_missing",
        "rejected",
        "recovery_artifact_trust_event_id_not_current_boot",
        "recovery_artifact_trust_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.vm_test,
        "missing",
        "recovery_vm_test_event_id_missing",
        "rejected",
        "recovery_vm_test_event_id_not_current_boot",
        "recovery_vm_test_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.local_approval,
        "missing",
        "recovery_local_approval_event_id_missing",
        "rejected",
        "recovery_local_approval_event_id_not_current_boot",
        "recovery_local_approval_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.loader,
        "missing",
        "recovery_loader_event_id_missing",
        "rejected",
        "recovery_loader_event_id_not_current_boot",
        "recovery_loader_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.rollback_evidence,
        "missing",
        "recovery_rollback_evidence_event_id_missing",
        "rejected",
        "recovery_rollback_evidence_event_id_not_current_boot",
        "recovery_rollback_evidence_schema_mismatch",
    ) {
        result
    } else {
        (
            "available_non_authorizing",
            "recovery_lifeline_protocol_missing",
        )
    };

    RecoveryLoadBindingCheck {
        status,
        reason,
        recovery_only_capability_used: method_eq(
            candidate.requested_capability,
            RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        ) && !candidate.normal_module_capability_substituted,
        accepts_normal_module_authority: false,
        append_payload_hash_authority: false,
        can_move_beyond_denial: false,
        loads_recovery_artifact: false,
        loads_normal_module: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn evaluate_recovery_evidence(
    evidence: RecoveryEvidenceCandidate,
    missing_status: &'static str,
    missing_reason: &'static str,
    rejected_status: &'static str,
    stale_reason: &'static str,
    schema_reason: &'static str,
) -> Option<(&'static str, &'static str)> {
    if !evidence.retained {
        Some((missing_status, missing_reason))
    } else if !evidence.current_boot {
        Some((rejected_status, stale_reason))
    } else if !evidence.schema_ok {
        Some((rejected_status, schema_reason))
    } else if !evidence.binding_ok {
        Some((rejected_status, evidence.binding_reason))
    } else {
        None
    }
}

fn recovery_load_binding_candidate_from_retained(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> RecoveryLoadBindingCandidate {
    let mut candidate = recovery_load_binding_missing_candidate();
    if retained_identity.is_some() {
        candidate.identity = recovery_evidence_available();
    }
    if retained_trust.is_some() {
        candidate.trust = if let Some(reason) =
            recovery_load_binding_retained_trust_mismatch(retained_identity, retained_trust)
        {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_vm_test.is_some() {
        candidate.vm_test = if let Some(reason) = recovery_load_binding_retained_vm_test_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
        ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_local_approval.is_some() {
        candidate.local_approval = if let Some(reason) =
            recovery_load_binding_retained_local_approval_mismatch(
                retained_identity,
                retained_trust,
                retained_vm_test,
                retained_local_approval,
            ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_loader.is_some() {
        candidate.loader = if let Some(reason) = recovery_load_binding_retained_loader_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained_loader,
        ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_rollback_evidence.is_some() {
        candidate.rollback_evidence = if let Some(reason) =
            recovery_load_binding_retained_rollback_evidence_mismatch(
                retained_identity,
                retained_trust,
                retained_vm_test,
                retained_local_approval,
                retained_loader,
                retained_rollback_evidence,
            ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    candidate
}

fn recovery_load_binding_retained_trust_mismatch(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((_trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    if trust_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_trust_identity_event_id_mismatch");
    }
    if trust_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_trust_identity_reference_hash_mismatch");
    }
    if trust_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_trust_artifact_hash_mismatch");
    }
    None
}

fn recovery_load_binding_retained_vm_test_mismatch(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((_vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    if vm_test_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_vm_test_identity_event_id_mismatch");
    }
    if vm_test_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_vm_test_trust_event_id_mismatch");
    }
    if vm_test_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_vm_test_identity_reference_hash_mismatch");
    }
    if vm_test_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_vm_test_trust_reference_hash_mismatch");
    }
    if vm_test_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_vm_test_artifact_hash_mismatch");
    }
    if vm_test_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_vm_test_trust_artifact_hash_mismatch");
    }
    if vm_test_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_vm_test_trust_hash_mismatch");
    }
    None
}

fn recovery_load_binding_retained_local_approval_mismatch(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    let Some((_approval_event_id, approval_reference)) = retained_local_approval else {
        return None;
    };
    if approval_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_local_approval_identity_event_id_mismatch");
    }
    if approval_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_local_approval_trust_event_id_mismatch");
    }
    if approval_reference.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_artifact_local_approval_vm_test_event_id_mismatch");
    }
    if approval_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_local_approval_identity_reference_hash_mismatch");
    }
    if approval_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_local_approval_trust_reference_hash_mismatch");
    }
    if approval_reference.vm_test_reference_hash != vm_test_reference.vm_test_reference_hash {
        return Some("recovery_artifact_local_approval_vm_test_reference_hash_mismatch");
    }
    if approval_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_local_approval_artifact_hash_mismatch");
    }
    if approval_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_local_approval_trust_artifact_hash_mismatch");
    }
    if approval_reference.artifact_hash != vm_test_reference.artifact_hash {
        return Some("recovery_artifact_local_approval_vm_test_artifact_hash_mismatch");
    }
    if approval_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_local_approval_trust_hash_mismatch");
    }
    if approval_reference.trust_hash != vm_test_reference.trust_hash {
        return Some("recovery_artifact_local_approval_vm_test_trust_hash_mismatch");
    }
    if approval_reference.vm_test_hash != vm_test_reference.vm_test_hash {
        return Some("recovery_artifact_local_approval_vm_test_hash_mismatch");
    }
    None
}

fn recovery_load_binding_retained_loader_mismatch(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    let Some((local_approval_event_id, approval_reference)) = retained_local_approval else {
        return None;
    };
    let Some((_loader_event_id, loader_reference)) = retained_loader else {
        return None;
    };
    if loader_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_loader_identity_event_id_mismatch");
    }
    if loader_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_loader_trust_event_id_mismatch");
    }
    if loader_reference.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_artifact_loader_vm_test_event_id_mismatch");
    }
    if loader_reference.retained_local_approval_reference_event_id != local_approval_event_id {
        return Some("recovery_artifact_loader_local_approval_event_id_mismatch");
    }
    if loader_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_loader_identity_reference_hash_mismatch");
    }
    if loader_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_loader_trust_reference_hash_mismatch");
    }
    if loader_reference.vm_test_reference_hash != vm_test_reference.vm_test_reference_hash {
        return Some("recovery_artifact_loader_vm_test_reference_hash_mismatch");
    }
    if loader_reference.local_approval_reference_hash
        != approval_reference.local_approval_reference_hash
    {
        return Some("recovery_artifact_loader_local_approval_reference_hash_mismatch");
    }
    if loader_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_loader_artifact_hash_mismatch");
    }
    if loader_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_loader_trust_artifact_hash_mismatch");
    }
    if loader_reference.artifact_hash != vm_test_reference.artifact_hash {
        return Some("recovery_artifact_loader_vm_test_artifact_hash_mismatch");
    }
    if loader_reference.artifact_hash != approval_reference.artifact_hash {
        return Some("recovery_artifact_loader_local_approval_artifact_hash_mismatch");
    }
    if loader_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_loader_trust_hash_mismatch");
    }
    if loader_reference.trust_hash != vm_test_reference.trust_hash {
        return Some("recovery_artifact_loader_vm_test_trust_hash_mismatch");
    }
    if loader_reference.trust_hash != approval_reference.trust_hash {
        return Some("recovery_artifact_loader_local_approval_trust_hash_mismatch");
    }
    if loader_reference.vm_test_hash != vm_test_reference.vm_test_hash {
        return Some("recovery_artifact_loader_vm_test_hash_mismatch");
    }
    if loader_reference.vm_test_hash != approval_reference.vm_test_hash {
        return Some("recovery_artifact_loader_local_approval_vm_test_hash_mismatch");
    }
    if loader_reference.local_approval_hash != approval_reference.local_approval_hash {
        return Some("recovery_artifact_loader_local_approval_hash_mismatch");
    }
    None
}

fn recovery_load_binding_retained_rollback_evidence_mismatch(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    let Some((local_approval_event_id, approval_reference)) = retained_local_approval else {
        return None;
    };
    let Some((loader_event_id, loader_reference)) = retained_loader else {
        return None;
    };
    let Some((_rollback_event_id, rollback_reference)) = retained_rollback_evidence else {
        return None;
    };
    if rollback_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_rollback_evidence_identity_event_id_mismatch");
    }
    if rollback_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_rollback_evidence_trust_event_id_mismatch");
    }
    if rollback_reference.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_artifact_rollback_evidence_vm_test_event_id_mismatch");
    }
    if rollback_reference.retained_local_approval_reference_event_id != local_approval_event_id {
        return Some("recovery_artifact_rollback_evidence_local_approval_event_id_mismatch");
    }
    if rollback_reference.retained_loader_reference_event_id != loader_event_id {
        return Some("recovery_artifact_rollback_evidence_loader_event_id_mismatch");
    }
    if rollback_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_rollback_evidence_identity_reference_hash_mismatch");
    }
    if rollback_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_rollback_evidence_trust_reference_hash_mismatch");
    }
    if rollback_reference.vm_test_reference_hash != vm_test_reference.vm_test_reference_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_reference_hash_mismatch");
    }
    if rollback_reference.local_approval_reference_hash
        != approval_reference.local_approval_reference_hash
    {
        return Some("recovery_artifact_rollback_evidence_local_approval_reference_hash_mismatch");
    }
    if rollback_reference.loader_reference_hash != loader_reference.loader_reference_hash {
        return Some("recovery_artifact_rollback_evidence_loader_reference_hash_mismatch");
    }
    if rollback_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_trust_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != vm_test_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != approval_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != loader_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_loader_artifact_hash_mismatch");
    }
    if rollback_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_trust_hash_mismatch");
    }
    if rollback_reference.trust_hash != vm_test_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_trust_hash_mismatch");
    }
    if rollback_reference.trust_hash != approval_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_trust_hash_mismatch");
    }
    if rollback_reference.trust_hash != loader_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_loader_trust_hash_mismatch");
    }
    if rollback_reference.vm_test_hash != vm_test_reference.vm_test_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_hash_mismatch");
    }
    if rollback_reference.vm_test_hash != approval_reference.vm_test_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_vm_test_hash_mismatch");
    }
    if rollback_reference.vm_test_hash != loader_reference.vm_test_hash {
        return Some("recovery_artifact_rollback_evidence_loader_vm_test_hash_mismatch");
    }
    if rollback_reference.local_approval_hash != approval_reference.local_approval_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_hash_mismatch");
    }
    if rollback_reference.local_approval_hash != loader_reference.local_approval_hash {
        return Some("recovery_artifact_rollback_evidence_loader_local_approval_hash_mismatch");
    }
    if rollback_reference.loader_hash != loader_reference.loader_hash {
        return Some("recovery_artifact_rollback_evidence_loader_hash_mismatch");
    }
    None
}

fn recovery_lifeline_protocol_candidate_from_retained(
    retained_request: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> RecoveryLifelineProtocolCandidate {
    let mismatch = recovery_lifeline_protocol_retained_request_mismatch(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    RecoveryLifelineProtocolCandidate {
        request_retained: retained_request.is_some(),
        request_current_boot: true,
        request_schema_ok: true,
        request_binding_ok: mismatch.is_none(),
        request_binding_reason: mismatch.unwrap_or("retained_recovery_lifeline_request_valid"),
        direct_openai_recovery_shortcut_used: false,
        lifeline_protocol_state_present: false,
        command_vocabulary_present: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn recovery_lifeline_protocol_retained_request_mismatch(
    retained_request: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> Option<&'static str> {
    let Some((_request_event_id, request)) = retained_request else {
        return Some("recovery_lifeline_request_event_id_missing");
    };
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((trust_event_id, _trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((vm_test_event_id, _vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((local_approval_event_id, _approval_reference)) = retained_local_approval else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    let Some((loader_event_id, _loader_reference)) = retained_loader else {
        return Some("recovery_artifact_loader_reference_missing");
    };
    let Some((rollback_event_id, rollback_reference)) = retained_rollback_evidence else {
        return Some("recovery_artifact_rollback_evidence_reference_missing");
    };
    if request.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_lifeline_request_identity_event_id_mismatch");
    }
    if request.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_lifeline_request_trust_event_id_mismatch");
    }
    if request.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_lifeline_request_vm_test_event_id_mismatch");
    }
    if request.retained_local_approval_reference_event_id != local_approval_event_id {
        return Some("recovery_lifeline_request_local_approval_event_id_mismatch");
    }
    if request.retained_loader_reference_event_id != loader_event_id {
        return Some("recovery_lifeline_request_loader_event_id_mismatch");
    }
    if request.retained_rollback_evidence_reference_event_id != rollback_event_id {
        return Some("recovery_lifeline_request_rollback_evidence_event_id_mismatch");
    }
    if let Some(reason) = recovery_load_binding_retained_rollback_evidence_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    ) {
        return Some(reason);
    }
    if request.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_lifeline_request_identity_reference_hash_mismatch");
    }
    if request.identity_reference_hash != rollback_reference.identity_reference_hash {
        return Some("recovery_lifeline_request_rollback_identity_reference_hash_mismatch");
    }
    if request.trust_reference_hash != rollback_reference.trust_reference_hash {
        return Some("recovery_lifeline_request_trust_reference_hash_mismatch");
    }
    if request.vm_test_reference_hash != rollback_reference.vm_test_reference_hash {
        return Some("recovery_lifeline_request_vm_test_reference_hash_mismatch");
    }
    if request.local_approval_reference_hash != rollback_reference.local_approval_reference_hash {
        return Some("recovery_lifeline_request_local_approval_reference_hash_mismatch");
    }
    if request.loader_reference_hash != rollback_reference.loader_reference_hash {
        return Some("recovery_lifeline_request_loader_reference_hash_mismatch");
    }
    if request.rollback_evidence_reference_hash
        != rollback_reference.rollback_evidence_reference_hash
    {
        return Some("recovery_artifact_rollback_evidence_reference_hash_mismatch");
    }
    if request.artifact_hash != rollback_reference.artifact_hash {
        return Some("recovery_lifeline_request_artifact_hash_mismatch");
    }
    if request.trust_hash != rollback_reference.trust_hash {
        return Some("recovery_lifeline_request_trust_hash_mismatch");
    }
    if request.vm_test_hash != rollback_reference.vm_test_hash {
        return Some("recovery_lifeline_request_vm_test_hash_mismatch");
    }
    if request.local_approval_hash != rollback_reference.local_approval_hash {
        return Some("recovery_lifeline_request_local_approval_hash_mismatch");
    }
    if request.loader_hash != rollback_reference.loader_hash {
        return Some("recovery_lifeline_request_loader_hash_mismatch");
    }
    if request.rollback_evidence_hash != rollback_reference.rollback_evidence_hash {
        return Some("recovery_artifact_rollback_evidence_hash_mismatch");
    }

    let mut identity_event_id_text = [0u8; 27];
    let mut trust_event_id_text = [0u8; 27];
    let mut vm_test_event_id_text = [0u8; 27];
    let mut local_approval_event_id_text = [0u8; 27];
    let mut loader_event_id_text = [0u8; 27];
    let mut rollback_event_id_text = [0u8; 27];
    let expected = module_evidence::computed_recovery_lifeline_request_reference_hash(
        module_evidence::RecoveryLifelineRequestReferenceHashInput {
            retained_identity_reference_event_id: current_boot_event_id_text(
                request.retained_identity_reference_event_id,
                &mut identity_event_id_text,
            ),
            retained_trust_reference_event_id: current_boot_event_id_text(
                request.retained_trust_reference_event_id,
                &mut trust_event_id_text,
            ),
            retained_vm_test_reference_event_id: current_boot_event_id_text(
                request.retained_vm_test_reference_event_id,
                &mut vm_test_event_id_text,
            ),
            retained_local_approval_reference_event_id: current_boot_event_id_text(
                request.retained_local_approval_reference_event_id,
                &mut local_approval_event_id_text,
            ),
            retained_loader_reference_event_id: current_boot_event_id_text(
                request.retained_loader_reference_event_id,
                &mut loader_event_id_text,
            ),
            retained_rollback_evidence_reference_event_id: current_boot_event_id_text(
                request.retained_rollback_evidence_reference_event_id,
                &mut rollback_event_id_text,
            ),
            identity_reference_hash: request.identity_reference_hash,
            trust_reference_hash: request.trust_reference_hash,
            vm_test_reference_hash: request.vm_test_reference_hash,
            local_approval_reference_hash: request.local_approval_reference_hash,
            loader_reference_hash: request.loader_reference_hash,
            rollback_evidence_reference_hash: request.rollback_evidence_reference_hash,
            artifact_hash: request.artifact_hash,
            trust_hash: request.trust_hash,
            vm_test_hash: request.vm_test_hash,
            local_approval_hash: request.local_approval_hash,
            loader_hash: request.loader_hash,
            rollback_evidence_hash: request.rollback_evidence_hash,
        },
    );
    if request.lifeline_request_reference_hash != expected {
        return Some("recovery_lifeline_request_reference_hash_mismatch");
    }
    None
}

fn current_boot_event_id_text<'a>(event_id: event_log::EventId, out: &'a mut [u8; 27]) -> &'a str {
    let prefix = b"event.current_boot.";
    let mut idx = 0usize;
    while idx < prefix.len() {
        out[idx] = prefix[idx];
        idx += 1;
    }
    let mut sequence = event_id.sequence();
    let mut digit = 0usize;
    while digit < 8 {
        out[prefix.len() + 7 - digit] = b'0' + (sequence % 10) as u8;
        sequence /= 10;
        digit += 1;
    }
    unsafe { core::str::from_utf8_unchecked(out) }
}

fn recovery_load_binding_missing_candidate() -> RecoveryLoadBindingCandidate {
    RecoveryLoadBindingCandidate {
        requested_capability: RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        identity: recovery_evidence_missing(),
        trust: recovery_evidence_missing(),
        vm_test: recovery_evidence_missing(),
        local_approval: recovery_evidence_missing(),
        loader: recovery_evidence_missing(),
        rollback_evidence: recovery_evidence_missing(),
        normal_module_capability_substituted: false,
        normal_module_append_intent_substituted: false,
        append_payload_hash_claimed_authority: false,
        normal_module_writer_facts_substituted: false,
        normal_module_service_slot_substituted: false,
    }
}

fn recovery_load_binding_available_candidate() -> RecoveryLoadBindingCandidate {
    RecoveryLoadBindingCandidate {
        requested_capability: RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        identity: recovery_evidence_available(),
        trust: recovery_evidence_available(),
        vm_test: recovery_evidence_available(),
        local_approval: recovery_evidence_available(),
        loader: recovery_evidence_available(),
        rollback_evidence: recovery_evidence_available(),
        normal_module_capability_substituted: false,
        normal_module_append_intent_substituted: false,
        append_payload_hash_claimed_authority: false,
        normal_module_writer_facts_substituted: false,
        normal_module_service_slot_substituted: false,
    }
}

fn recovery_evidence_available() -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: true,
        current_boot: true,
        schema_ok: true,
        binding_ok: true,
        binding_reason: "",
    }
}

fn recovery_evidence_missing() -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: false,
        current_boot: true,
        schema_ok: true,
        binding_ok: true,
        binding_reason: "",
    }
}

fn recovery_evidence_rejected(reason: &'static str) -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: true,
        current_boot: true,
        schema_ok: true,
        binding_ok: false,
        binding_reason: reason,
    }
}

fn parse_recovery_identity_reference(arg: &str) -> RecoveryIdentityReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let identity_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryIdentityReferenceCheck {
        has_reference: identity_reference_hash.is_some(),
        arity_valid: identity_reference_hash.is_some()
            && artifact_hash.is_some()
            && extra.is_none(),
        scope,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        expected_identity_reference_hash: None,
        status: "missing",
        reason: "recovery_artifact_identity_reference_absent",
        valid: false,
    };
    evaluate_recovery_identity_reference(input)
}

fn evaluate_recovery_identity_reference(
    input: RecoveryIdentityReferenceCheck<'_>,
) -> RecoveryIdentityReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_identity_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_identity_reference_absent",
            false,
        );
    }
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_identity_reference_check(
            input,
            None,
            if input.has_reference {
                "invalid_reference"
            } else {
                "missing"
            },
            if input.has_reference {
                "recovery_artifact_identity_reference_invalid_hash"
            } else {
                "recovery_artifact_identity_reference_absent"
            },
            false,
        );
    };
    if !input.arity_valid {
        return recovery_identity_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_identity_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_identity_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_identity_reference_scope_must_be_current_boot",
            false,
        );
    }
    let expected =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    if input.identity_reference_hash != Some(expected) {
        return recovery_identity_reference_check(
            input,
            Some(expected),
            "mismatched_identity_reference_hash",
            "recovery_artifact_identity_reference_hash_mismatch",
            false,
        );
    }
    recovery_identity_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing",
        true,
    )
}

fn recovery_identity_reference_check<'a>(
    input: RecoveryIdentityReferenceCheck<'a>,
    expected_identity_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryIdentityReferenceCheck<'a> {
    RecoveryIdentityReferenceCheck {
        expected_identity_reference_hash,
        status,
        reason,
        valid,
        ..input
    }
}

fn parse_recovery_trust_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryTrustReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let trust_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryTrustReferenceInput {
        has_reference: trust_reference_hash.is_some(),
        arity_valid: trust_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && extra.is_none(),
        scope,
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_trust_reference(input, require_live_retained)
}

fn evaluate_recovery_trust_reference(
    input: RecoveryTrustReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryTrustReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_trust_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_trust_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_trust_reference_check(
            input,
            None,
            if input.has_reference {
                "invalid_reference"
            } else {
                "missing"
            },
            if input.has_reference {
                "recovery_artifact_trust_reference_invalid_hash"
            } else {
                "recovery_artifact_trust_reference_absent"
            },
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_trust_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_trust_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_trust_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    if input.trust_reference_hash != Some(expected) {
        return recovery_trust_reference_check(
            input,
            Some(expected),
            "mismatched_trust_reference_hash",
            "recovery_artifact_trust_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_trust_live_identity_mismatch(&input) {
            return recovery_trust_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_trust_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing",
        true,
    )
}

fn recovery_trust_reference_check<'a>(
    input: RecoveryTrustReferenceInput<'a>,
    expected_trust_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryTrustReferenceCheck<'a> {
    RecoveryTrustReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        trust_reference_hash: input.trust_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        expected_trust_reference_hash,
        status,
        reason,
        valid,
    }
}

fn recovery_trust_live_identity_mismatch(
    input: &RecoveryTrustReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let Some((latest_event_id, identity_reference)) =
        event_log::latest_recovery_artifact_identity_reference()
    else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    if latest_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_identity_reference_hash_mismatch");
    }
    if Some(identity_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_identity_artifact_hash_mismatch");
    }
    None
}

fn parse_recovery_vm_test_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryVmTestReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let vm_test_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryVmTestReferenceInput {
        has_reference: vm_test_reference_hash.is_some(),
        arity_valid: vm_test_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && extra.is_none(),
        scope,
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_vm_test_reference(input, require_live_retained)
}

fn evaluate_recovery_vm_test_reference(
    input: RecoveryVmTestReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryVmTestReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_vm_test_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_vm_test_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_vm_test_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_vm_test_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_vm_test_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_vm_test_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    if input.vm_test_reference_hash != Some(expected) {
        return recovery_vm_test_reference_check(
            input,
            Some(expected),
            "mismatched_vm_test_reference_hash",
            "recovery_artifact_vm_test_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_vm_test_live_chain_mismatch(&input) {
            return recovery_vm_test_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_vm_test_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing",
        true,
    )
}

fn recovery_vm_test_reference_check<'a>(
    input: RecoveryVmTestReferenceInput<'a>,
    expected_vm_test_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryVmTestReferenceCheck<'a> {
    RecoveryVmTestReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        vm_test_reference_hash: input.vm_test_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        expected_vm_test_reference_hash,
        status,
        reason,
        valid,
    }
}

fn recovery_vm_test_live_chain_mismatch(
    input: &RecoveryVmTestReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let Some((latest_identity_event_id, identity_reference)) =
        event_log::latest_recovery_artifact_identity_reference()
    else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, trust_reference)) =
        event_log::latest_recovery_artifact_trust_reference()
    else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if trust_reference.retained_identity_reference_event_id != latest_identity_event_id {
        return Some("recovery_artifact_trust_identity_event_id_mismatch");
    }
    if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_identity_reference_hash_mismatch");
    }
    if Some(identity_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_identity_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_reference_hash) != input.trust_reference_hash {
        return Some("recovery_artifact_trust_reference_hash_mismatch");
    }
    if Some(trust_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_trust_identity_reference_hash_mismatch");
    }
    if Some(trust_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_trust_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_hash) != input.trust_hash {
        return Some("recovery_artifact_trust_hash_mismatch");
    }
    None
}

fn parse_recovery_local_approval_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLocalApprovalReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let local_approval_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLocalApprovalReferenceInput {
        has_reference: local_approval_reference_hash.is_some(),
        arity_valid: local_approval_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && extra.is_none(),
        scope,
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_local_approval_reference(input, require_live_retained)
}

fn evaluate_recovery_local_approval_reference(
    input: RecoveryLocalApprovalReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryLocalApprovalReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_local_approval_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_local_approval_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_local_approval_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_local_approval_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_local_approval_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_local_approval_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_local_approval_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_local_approval_reference_hash(
        module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
        },
    );
    if input.local_approval_reference_hash != Some(expected) {
        return recovery_local_approval_reference_check(
            input,
            Some(expected),
            "mismatched_local_approval_reference_hash",
            "recovery_artifact_local_approval_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_local_approval_live_chain_mismatch(&input) {
            return recovery_local_approval_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_local_approval_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_local_approval_reference_valid_but_loader_missing",
        true,
    )
}

fn recovery_local_approval_reference_check<'a>(
    input: RecoveryLocalApprovalReferenceInput<'a>,
    expected_local_approval_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLocalApprovalReferenceCheck<'a> {
    RecoveryLocalApprovalReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        local_approval_reference_hash: input.local_approval_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        expected_local_approval_reference_hash,
        status,
        reason,
        valid,
    }
}

fn recovery_local_approval_live_chain_mismatch(
    input: &RecoveryLocalApprovalReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let Some((latest_identity_event_id, identity_reference)) =
        event_log::latest_recovery_artifact_identity_reference()
    else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, trust_reference)) =
        event_log::latest_recovery_artifact_trust_reference()
    else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, vm_test_reference)) =
        event_log::latest_recovery_artifact_vm_test_reference()
    else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if trust_reference.retained_identity_reference_event_id != latest_identity_event_id {
        return Some("recovery_artifact_trust_identity_event_id_mismatch");
    }
    if vm_test_reference.retained_identity_reference_event_id != latest_identity_event_id {
        return Some("recovery_artifact_vm_test_identity_event_id_mismatch");
    }
    if vm_test_reference.retained_trust_reference_event_id != latest_trust_event_id {
        return Some("recovery_artifact_vm_test_trust_event_id_mismatch");
    }
    if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_identity_reference_hash_mismatch");
    }
    if Some(identity_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_identity_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_reference_hash) != input.trust_reference_hash {
        return Some("recovery_artifact_trust_reference_hash_mismatch");
    }
    if Some(trust_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_trust_identity_reference_hash_mismatch");
    }
    if Some(trust_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_trust_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_hash) != input.trust_hash {
        return Some("recovery_artifact_trust_hash_mismatch");
    }
    if Some(vm_test_reference.vm_test_reference_hash) != input.vm_test_reference_hash {
        return Some("recovery_artifact_vm_test_reference_hash_mismatch");
    }
    if Some(vm_test_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_vm_test_identity_reference_hash_mismatch");
    }
    if Some(vm_test_reference.trust_reference_hash) != input.trust_reference_hash {
        return Some("recovery_artifact_vm_test_trust_reference_hash_mismatch");
    }
    if Some(vm_test_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_vm_test_artifact_hash_mismatch");
    }
    if Some(vm_test_reference.trust_hash) != input.trust_hash {
        return Some("recovery_artifact_vm_test_trust_hash_mismatch");
    }
    if Some(vm_test_reference.vm_test_hash) != input.vm_test_hash {
        return Some("recovery_artifact_vm_test_hash_mismatch");
    }
    None
}

fn parse_recovery_loader_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLoaderReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let loader_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let retained_local_approval_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let local_approval_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let loader_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLoaderReferenceInput {
        has_reference: loader_reference_hash.is_some(),
        arity_valid: loader_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && retained_local_approval_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && local_approval_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && loader_hash.is_some()
            && extra.is_none(),
        scope,
        loader_reference_hash: loader_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
        loader_hash: loader_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_loader_reference(input, require_live_retained)
}

fn evaluate_recovery_loader_reference(
    input: RecoveryLoaderReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryLoaderReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_loader_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_loader_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_event_id) = input.retained_local_approval_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_reference_hash) = input.local_approval_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(loader_hash) = input.loader_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_loader_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_loader_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(local_approval_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    if input.loader_reference_hash != Some(expected) {
        return recovery_loader_reference_check(
            input,
            Some(expected),
            "mismatched_loader_reference_hash",
            "recovery_artifact_loader_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_loader_live_chain_mismatch(&input) {
            return recovery_loader_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_loader_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing",
        true,
    )
}

fn recovery_loader_reference_check<'a>(
    input: RecoveryLoaderReferenceInput<'a>,
    expected_loader_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLoaderReferenceCheck<'a> {
    RecoveryLoaderReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        loader_reference_hash: input.loader_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id: input
            .retained_local_approval_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        local_approval_reference_hash: input.local_approval_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        loader_hash: input.loader_hash,
        expected_loader_reference_hash,
        status,
        reason,
        valid,
    }
}

fn recovery_loader_live_chain_mismatch(
    input: &RecoveryLoaderReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let retained_local_approval_reference_event_id =
        parse_current_boot_event_id(input.retained_local_approval_reference_event_id?)?;
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let Some((latest_identity_event_id, identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((latest_local_approval_event_id, approval_reference)) = retained_local_approval else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if latest_local_approval_event_id != retained_local_approval_reference_event_id {
        return Some("recovery_artifact_local_approval_reference_event_id_mismatch");
    }
    recovery_load_binding_retained_local_approval_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
    )
    .or_else(|| {
        if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
            Some("recovery_artifact_identity_reference_hash_mismatch")
        } else if Some(trust_reference.trust_reference_hash) != input.trust_reference_hash {
            Some("recovery_artifact_trust_reference_hash_mismatch")
        } else if Some(vm_test_reference.vm_test_reference_hash) != input.vm_test_reference_hash {
            Some("recovery_artifact_vm_test_reference_hash_mismatch")
        } else if Some(approval_reference.local_approval_reference_hash)
            != input.local_approval_reference_hash
        {
            Some("recovery_artifact_local_approval_reference_hash_mismatch")
        } else if Some(approval_reference.local_approval_hash) != input.local_approval_hash {
            Some("recovery_artifact_local_approval_hash_mismatch")
        } else {
            None
        }
    })
}

fn parse_recovery_rollback_evidence_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryRollbackEvidenceReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let rollback_evidence_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let retained_local_approval_reference_event_id = parts.next();
    let retained_loader_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let local_approval_reference_hash = parts.next();
    let loader_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let loader_hash = parts.next();
    let rollback_evidence_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryRollbackEvidenceReferenceInput {
        has_reference: rollback_evidence_reference_hash.is_some(),
        arity_valid: rollback_evidence_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && retained_local_approval_reference_event_id.is_some()
            && retained_loader_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && local_approval_reference_hash.is_some()
            && loader_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && loader_hash.is_some()
            && rollback_evidence_hash.is_some()
            && extra.is_none(),
        scope,
        rollback_evidence_reference_hash: rollback_evidence_reference_hash
            .and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id,
        retained_loader_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        loader_reference_hash: loader_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
        loader_hash: loader_hash.and_then(parse_sha256_ref),
        rollback_evidence_hash: rollback_evidence_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_rollback_evidence_reference(input, require_live_retained)
}

fn evaluate_recovery_rollback_evidence_reference(
    input: RecoveryRollbackEvidenceReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryRollbackEvidenceReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_rollback_evidence_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(local_approval_event_id) = input.retained_local_approval_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(loader_event_id) = input.retained_loader_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(local_approval_reference_hash) = input.local_approval_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(loader_reference_hash) = input.loader_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(loader_hash) = input.loader_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(rollback_evidence_hash) = input.rollback_evidence_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    if !input.arity_valid {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_rollback_evidence_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_rollback_evidence_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(local_approval_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(loader_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_loader_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_rollback_evidence_reference_hash(
        module_evidence::RecoveryArtifactRollbackEvidenceReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    if input.rollback_evidence_reference_hash != Some(expected) {
        return recovery_rollback_evidence_reference_check(
            input,
            Some(expected),
            "mismatched_rollback_evidence_reference_hash",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_rollback_evidence_live_chain_mismatch(&input) {
            return recovery_rollback_evidence_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_rollback_evidence_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing",
        true,
    )
}

fn recovery_rollback_evidence_invalid(
    input: RecoveryRollbackEvidenceReferenceInput<'_>,
) -> RecoveryRollbackEvidenceReferenceCheck<'_> {
    recovery_rollback_evidence_reference_check(
        input,
        None,
        "invalid_reference",
        "recovery_artifact_rollback_evidence_reference_invalid_hash",
        false,
    )
}

fn recovery_rollback_evidence_reference_check<'a>(
    input: RecoveryRollbackEvidenceReferenceInput<'a>,
    expected_rollback_evidence_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryRollbackEvidenceReferenceCheck<'a> {
    RecoveryRollbackEvidenceReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        rollback_evidence_reference_hash: input.rollback_evidence_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id: input
            .retained_local_approval_reference_event_id,
        retained_loader_reference_event_id: input.retained_loader_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        local_approval_reference_hash: input.local_approval_reference_hash,
        loader_reference_hash: input.loader_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        loader_hash: input.loader_hash,
        rollback_evidence_hash: input.rollback_evidence_hash,
        expected_rollback_evidence_reference_hash,
        status,
        reason,
        valid,
    }
}

fn recovery_rollback_evidence_live_chain_mismatch(
    input: &RecoveryRollbackEvidenceReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let retained_local_approval_reference_event_id =
        parse_current_boot_event_id(input.retained_local_approval_reference_event_id?)?;
    let retained_loader_reference_event_id =
        parse_current_boot_event_id(input.retained_loader_reference_event_id?)?;
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let Some((latest_identity_event_id, _identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, _trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, _vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((latest_local_approval_event_id, _approval_reference)) = retained_local_approval
    else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    let Some((latest_loader_event_id, _loader_reference)) = retained_loader else {
        return Some("recovery_artifact_loader_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if latest_local_approval_event_id != retained_local_approval_reference_event_id {
        return Some("recovery_artifact_local_approval_reference_event_id_mismatch");
    }
    if latest_loader_event_id != retained_loader_reference_event_id {
        return Some("recovery_artifact_loader_reference_event_id_mismatch");
    }
    recovery_load_binding_retained_loader_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
    )
    .or_else(|| {
        if let Some((_loader_event_id, loader_reference)) = retained_loader {
            if Some(loader_reference.loader_reference_hash) != input.loader_reference_hash {
                Some("recovery_artifact_loader_reference_hash_mismatch")
            } else if Some(loader_reference.loader_hash) != input.loader_hash {
                Some("recovery_artifact_loader_hash_mismatch")
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn parse_recovery_lifeline_request_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineRequestReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let lifeline_request_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let retained_local_approval_reference_event_id = parts.next();
    let retained_loader_reference_event_id = parts.next();
    let retained_rollback_evidence_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let local_approval_reference_hash = parts.next();
    let loader_reference_hash = parts.next();
    let rollback_evidence_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let loader_hash = parts.next();
    let rollback_evidence_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineRequestReferenceInput {
        has_reference: lifeline_request_reference_hash.is_some(),
        arity_valid: lifeline_request_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && retained_local_approval_reference_event_id.is_some()
            && retained_loader_reference_event_id.is_some()
            && retained_rollback_evidence_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && local_approval_reference_hash.is_some()
            && loader_reference_hash.is_some()
            && rollback_evidence_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && loader_hash.is_some()
            && rollback_evidence_hash.is_some()
            && extra.is_none(),
        scope,
        lifeline_request_reference_hash: lifeline_request_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id,
        retained_loader_reference_event_id,
        retained_rollback_evidence_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        loader_reference_hash: loader_reference_hash.and_then(parse_sha256_ref),
        rollback_evidence_reference_hash: rollback_evidence_reference_hash
            .and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
        loader_hash: loader_hash.and_then(parse_sha256_ref),
        rollback_evidence_hash: rollback_evidence_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_request_reference(input, require_live_retained)
}

fn evaluate_recovery_lifeline_request_reference(
    input: RecoveryLifelineRequestReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineRequestReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "missing",
            "recovery_lifeline_request_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(local_approval_event_id) = input.retained_local_approval_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(loader_event_id) = input.retained_loader_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(rollback_evidence_event_id) = input.retained_rollback_evidence_reference_event_id
    else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(local_approval_reference_hash) = input.local_approval_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(loader_reference_hash) = input.loader_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(rollback_evidence_reference_hash) = input.rollback_evidence_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(loader_hash) = input.loader_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(rollback_evidence_hash) = input.rollback_evidence_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_lifeline_request_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_request_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(local_approval_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(loader_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_loader_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(rollback_evidence_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_rollback_evidence_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_lifeline_request_reference_hash(
        module_evidence::RecoveryLifelineRequestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            retained_rollback_evidence_reference_event_id: rollback_evidence_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            rollback_evidence_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    if input.lifeline_request_reference_hash != Some(expected) {
        return recovery_lifeline_request_reference_check(
            input,
            Some(expected),
            "mismatched_lifeline_request_reference_hash",
            "recovery_lifeline_request_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_lifeline_request_live_chain_mismatch(&input) {
            return recovery_lifeline_request_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_request_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_lifeline_request_reference_valid_but_lifeline_protocol_missing",
        true,
    )
}

fn recovery_lifeline_request_invalid(
    input: RecoveryLifelineRequestReferenceInput<'_>,
) -> RecoveryLifelineRequestReferenceCheck<'_> {
    recovery_lifeline_request_reference_check(
        input,
        None,
        "invalid_reference",
        "recovery_lifeline_request_reference_invalid_hash",
        false,
    )
}

fn recovery_lifeline_request_reference_check<'a>(
    input: RecoveryLifelineRequestReferenceInput<'a>,
    expected_lifeline_request_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineRequestReferenceCheck<'a> {
    RecoveryLifelineRequestReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        lifeline_request_reference_hash: input.lifeline_request_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id: input
            .retained_local_approval_reference_event_id,
        retained_loader_reference_event_id: input.retained_loader_reference_event_id,
        retained_rollback_evidence_reference_event_id: input
            .retained_rollback_evidence_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        local_approval_reference_hash: input.local_approval_reference_hash,
        loader_reference_hash: input.loader_reference_hash,
        rollback_evidence_reference_hash: input.rollback_evidence_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        loader_hash: input.loader_hash,
        rollback_evidence_hash: input.rollback_evidence_hash,
        expected_lifeline_request_reference_hash,
        status,
        reason,
        valid,
    }
}

fn recovery_lifeline_request_live_chain_mismatch(
    input: &RecoveryLifelineRequestReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let retained_local_approval_reference_event_id =
        parse_current_boot_event_id(input.retained_local_approval_reference_event_id?)?;
    let retained_loader_reference_event_id =
        parse_current_boot_event_id(input.retained_loader_reference_event_id?)?;
    let retained_rollback_evidence_reference_event_id =
        parse_current_boot_event_id(input.retained_rollback_evidence_reference_event_id?)?;
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let Some((latest_identity_event_id, _identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, _trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, _vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((latest_local_approval_event_id, _approval_reference)) = retained_local_approval
    else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    let Some((latest_loader_event_id, _loader_reference)) = retained_loader else {
        return Some("recovery_artifact_loader_reference_missing");
    };
    let Some((latest_rollback_event_id, _rollback_reference)) = retained_rollback_evidence else {
        return Some("recovery_artifact_rollback_evidence_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if latest_local_approval_event_id != retained_local_approval_reference_event_id {
        return Some("recovery_artifact_local_approval_reference_event_id_mismatch");
    }
    if latest_loader_event_id != retained_loader_reference_event_id {
        return Some("recovery_artifact_loader_reference_event_id_mismatch");
    }
    if latest_rollback_event_id != retained_rollback_evidence_reference_event_id {
        return Some("recovery_artifact_rollback_evidence_reference_event_id_mismatch");
    }
    recovery_load_binding_retained_rollback_evidence_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    )
    .or_else(|| {
        if let Some((_rollback_event_id, rollback_reference)) = retained_rollback_evidence {
            if Some(rollback_reference.rollback_evidence_reference_hash)
                != input.rollback_evidence_reference_hash
            {
                Some("recovery_artifact_rollback_evidence_reference_hash_mismatch")
            } else if Some(rollback_reference.rollback_evidence_hash)
                != input.rollback_evidence_hash
            {
                Some("recovery_artifact_rollback_evidence_hash_mismatch")
            } else if Some(rollback_reference.loader_reference_hash) != input.loader_reference_hash
            {
                Some("recovery_artifact_loader_reference_hash_mismatch")
            } else if Some(rollback_reference.loader_hash) != input.loader_hash {
                Some("recovery_artifact_loader_hash_mismatch")
            } else if Some(rollback_reference.local_approval_reference_hash)
                != input.local_approval_reference_hash
            {
                Some("recovery_artifact_local_approval_reference_hash_mismatch")
            } else if Some(rollback_reference.local_approval_hash) != input.local_approval_hash {
                Some("recovery_artifact_local_approval_hash_mismatch")
            } else if Some(rollback_reference.vm_test_reference_hash)
                != input.vm_test_reference_hash
            {
                Some("recovery_artifact_vm_test_reference_hash_mismatch")
            } else if Some(rollback_reference.vm_test_hash) != input.vm_test_hash {
                Some("recovery_artifact_vm_test_hash_mismatch")
            } else if Some(rollback_reference.trust_reference_hash) != input.trust_reference_hash {
                Some("recovery_artifact_trust_reference_hash_mismatch")
            } else if Some(rollback_reference.trust_hash) != input.trust_hash {
                Some("recovery_artifact_trust_hash_mismatch")
            } else if Some(rollback_reference.identity_reference_hash)
                != input.identity_reference_hash
            {
                Some("recovery_artifact_identity_reference_hash_mismatch")
            } else if Some(rollback_reference.artifact_hash) != input.artifact_hash {
                Some("recovery_artifact_identity_artifact_hash_mismatch")
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn recovery_identity_selftest_cases(
) -> [RecoveryIdentitySelfTestCase; RECOVERY_IDENTITY_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let valid_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let valid = RecoveryIdentityReferenceCheck {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        identity_reference_hash: Some(valid_hash),
        artifact_hash: Some(artifact_hash),
        expected_identity_reference_hash: None,
        status: "missing",
        reason: "missing",
        valid: false,
    };
    [
        recovery_identity_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_identity_reference_absent",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                has_reference: false,
                ..valid
            }),
        ),
        recovery_identity_selftest_case(
            "accepted_current_boot_identity_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing",
            evaluate_recovery_identity_reference(valid),
        ),
        recovery_identity_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_identity_reference_scope_must_be_current_boot",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                scope: "previous_boot",
                ..valid
            }),
        ),
        recovery_identity_selftest_case(
            "wrong_schema_identity_reference",
            "rejected",
            "recovery_artifact_identity_reference_wrong_schema_or_variant",
            RecoveryIdentityReferenceCheck {
                status: "rejected",
                reason: "recovery_artifact_identity_reference_wrong_schema_or_variant",
                valid: false,
                ..valid
            },
        ),
        recovery_identity_selftest_case(
            "substituted_identity_reference_record",
            "rejected",
            "recovery_artifact_identity_reference_substituted_record",
            RecoveryIdentityReferenceCheck {
                status: "rejected",
                reason: "recovery_artifact_identity_reference_substituted_record",
                valid: false,
                ..valid
            },
        ),
        recovery_identity_selftest_case(
            "identity_reference_hash_mismatch",
            "mismatched_identity_reference_hash",
            "recovery_artifact_identity_reference_hash_mismatch",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                identity_reference_hash: Some([0x92; 32]),
                ..valid
            }),
        ),
    ]
}

fn recovery_identity_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryIdentityReferenceCheck<'_>,
) -> RecoveryIdentitySelfTestCase {
    RecoveryIdentitySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_trust_selftest_cases() -> [RecoveryTrustSelfTestCase; RECOVERY_TRUST_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let identity_event_id = "event.current_boot.00000031";
    let valid_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let valid = RecoveryTrustReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        trust_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
    };
    [
        recovery_trust_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_trust_reference_absent",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "accepted_current_boot_trust_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing",
            evaluate_recovery_trust_reference(valid, false),
        ),
        recovery_trust_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_trust_reference_scope_must_be_current_boot",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    retained_identity_reference_event_id: Some("event.previous_boot.00000031"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_missing",
            "rejected",
            "recovery_artifact_identity_reference_missing",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_missing",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_identity_reference_wrong_schema_or_variant",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "substituted_identity_reference_record",
            "rejected",
            "recovery_artifact_identity_reference_substituted_record",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_substituted_record",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "trust_reference_hash_mismatch",
            "mismatched_trust_reference_hash",
            "recovery_artifact_trust_reference_hash_mismatch",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    trust_reference_hash: Some([0x94; 32]),
                    ..valid
                },
                false,
            ),
        ),
    ]
}

fn recovery_trust_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryTrustReferenceCheck<'_>,
) -> RecoveryTrustSelfTestCase {
    RecoveryTrustSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_vm_test_selftest_cases() -> [RecoveryVmTestSelfTestCase; RECOVERY_VM_TEST_SELFTEST_CASES]
{
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let valid_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let valid = RecoveryVmTestReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        vm_test_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
    };
    [
        recovery_vm_test_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_vm_test_reference_absent",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "accepted_current_boot_vm_test_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing",
            evaluate_recovery_vm_test_reference(valid, false),
        ),
        recovery_vm_test_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_vm_test_reference_scope_must_be_current_boot",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_identity_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    retained_identity_reference_event_id: Some("event.previous_boot.00000031"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_trust_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    retained_trust_reference_event_id: Some("event.previous_boot.00000032"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_identity_missing",
            "rejected",
            "recovery_artifact_identity_reference_missing",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_missing",
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_trust_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_trust_reference_wrong_schema_or_variant",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "substituted_trust_reference_record",
            "rejected",
            "recovery_artifact_trust_reference_substituted_record",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_reference_substituted_record",
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "vm_test_reference_hash_mismatch",
            "mismatched_vm_test_reference_hash",
            "recovery_artifact_vm_test_reference_hash_mismatch",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    vm_test_reference_hash: Some([0x96; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "trust_binding_mismatch",
            "rejected",
            "recovery_artifact_trust_identity_event_id_mismatch",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_identity_event_id_mismatch",
                false,
            ),
        ),
    ]
}

fn recovery_vm_test_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryVmTestReferenceCheck<'_>,
) -> RecoveryVmTestSelfTestCase {
    RecoveryVmTestSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_local_approval_selftest_cases(
) -> [RecoveryLocalApprovalSelfTestCase; RECOVERY_LOCAL_APPROVAL_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let valid_hash = module_evidence::computed_recovery_artifact_local_approval_reference_hash(
        module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
        },
    );
    let valid = RecoveryLocalApprovalReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        local_approval_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
    };
    [
        recovery_local_approval_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_local_approval_reference_absent",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "accepted_current_boot_local_approval_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_local_approval_reference_valid_but_loader_missing",
            evaluate_recovery_local_approval_reference(valid, false),
        ),
        recovery_local_approval_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_local_approval_reference_scope_must_be_current_boot",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_vm_test_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    retained_vm_test_reference_event_id: Some("event.previous_boot.00000033"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_vm_test_missing",
            "rejected",
            "recovery_artifact_vm_test_reference_missing",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_missing",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_vm_test_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_vm_test_reference_wrong_schema_or_variant",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "substituted_vm_test_reference_record",
            "rejected",
            "recovery_artifact_vm_test_reference_substituted_record",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_substituted_record",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "local_approval_reference_hash_mismatch",
            "mismatched_local_approval_reference_hash",
            "recovery_artifact_local_approval_reference_hash_mismatch",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    local_approval_reference_hash: Some([0x98; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "vm_test_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_vm_test_reference_hash_mismatch",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "trust_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_trust_reference_hash_mismatch",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_local_approval_vm_test_event_id_mismatch",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_vm_test_event_id_mismatch",
                false,
            ),
        ),
    ]
}

fn recovery_local_approval_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLocalApprovalReferenceCheck<'_>,
) -> RecoveryLocalApprovalSelfTestCase {
    RecoveryLocalApprovalSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_loader_selftest_cases() -> [RecoveryLoaderSelfTestCase; RECOVERY_LOADER_SELFTEST_CASES]
{
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let loader_hash = [0x99; 32];
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let local_approval_event_id = "event.current_boot.00000034";
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let local_approval_reference_hash =
        module_evidence::computed_recovery_artifact_local_approval_reference_hash(
            module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
            },
        );
    let valid_hash = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    let valid = RecoveryLoaderReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        loader_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        retained_local_approval_reference_event_id: Some(local_approval_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        local_approval_reference_hash: Some(local_approval_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
        loader_hash: Some(loader_hash),
    };
    [
        recovery_loader_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_loader_reference_absent",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "accepted_current_boot_loader_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing",
            evaluate_recovery_loader_reference(valid, false),
        ),
        recovery_loader_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_loader_reference_scope_must_be_current_boot",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_local_approval_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    retained_local_approval_reference_event_id: Some(
                        "event.previous_boot.00000034",
                    ),
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_local_approval_missing",
            "rejected",
            "recovery_artifact_local_approval_reference_missing",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_missing",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_local_approval_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_local_approval_reference_wrong_schema_or_variant",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "substituted_local_approval_reference_record",
            "rejected",
            "recovery_artifact_local_approval_reference_substituted_record",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_substituted_record",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "loader_reference_hash_mismatch",
            "mismatched_loader_reference_hash",
            "recovery_artifact_loader_reference_hash_mismatch",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    loader_reference_hash: Some([0x9a; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "local_approval_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_local_approval_reference_hash_mismatch",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_loader_local_approval_event_id_mismatch",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_local_approval_event_id_mismatch",
                false,
            ),
        ),
    ]
}

fn recovery_loader_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLoaderReferenceCheck<'_>,
) -> RecoveryLoaderSelfTestCase {
    RecoveryLoaderSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_rollback_evidence_selftest_cases(
) -> [RecoveryRollbackEvidenceSelfTestCase; RECOVERY_ROLLBACK_EVIDENCE_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let loader_hash = [0x99; 32];
    let rollback_evidence_hash = [0x9b; 32];
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let local_approval_event_id = "event.current_boot.00000034";
    let loader_event_id = "event.current_boot.00000035";
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let local_approval_reference_hash =
        module_evidence::computed_recovery_artifact_local_approval_reference_hash(
            module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
            },
        );
    let loader_reference_hash = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    let valid_hash = module_evidence::computed_recovery_artifact_rollback_evidence_reference_hash(
        module_evidence::RecoveryArtifactRollbackEvidenceReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    let valid = RecoveryRollbackEvidenceReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        rollback_evidence_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        retained_local_approval_reference_event_id: Some(local_approval_event_id),
        retained_loader_reference_event_id: Some(loader_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        local_approval_reference_hash: Some(local_approval_reference_hash),
        loader_reference_hash: Some(loader_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
        loader_hash: Some(loader_hash),
        rollback_evidence_hash: Some(rollback_evidence_hash),
    };
    [
        recovery_rollback_evidence_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_rollback_evidence_reference_absent",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "accepted_current_boot_rollback_evidence_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing",
            evaluate_recovery_rollback_evidence_reference(valid, false),
        ),
        recovery_rollback_evidence_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_rollback_evidence_reference_scope_must_be_current_boot",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_loader_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_loader_event_id_not_current_boot",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    retained_loader_reference_event_id: Some("event.previous_boot.00000035"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_loader_missing",
            "rejected",
            "recovery_artifact_loader_reference_missing",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_missing",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_loader_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_loader_reference_wrong_schema_or_variant",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "substituted_loader_reference_record",
            "rejected",
            "recovery_artifact_loader_reference_substituted_record",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_substituted_record",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "rollback_evidence_reference_hash_mismatch",
            "mismatched_rollback_evidence_reference_hash",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    rollback_evidence_reference_hash: Some([0x9c; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "loader_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_loader_reference_hash_mismatch",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
                false,
            ),
        ),
    ]
}

fn recovery_rollback_evidence_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryRollbackEvidenceReferenceCheck<'_>,
) -> RecoveryRollbackEvidenceSelfTestCase {
    RecoveryRollbackEvidenceSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_lifeline_request_selftest_cases(
) -> [RecoveryLifelineRequestSelfTestCase; RECOVERY_LIFELINE_REQUEST_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let loader_hash = [0x99; 32];
    let rollback_evidence_hash = [0x9b; 32];
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let local_approval_event_id = "event.current_boot.00000034";
    let loader_event_id = "event.current_boot.00000035";
    let rollback_evidence_event_id = "event.current_boot.00000036";
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let local_approval_reference_hash =
        module_evidence::computed_recovery_artifact_local_approval_reference_hash(
            module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
            },
        );
    let loader_reference_hash = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    let rollback_evidence_reference_hash =
        module_evidence::computed_recovery_artifact_rollback_evidence_reference_hash(
            module_evidence::RecoveryArtifactRollbackEvidenceReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                retained_local_approval_reference_event_id: local_approval_event_id,
                retained_loader_reference_event_id: loader_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                local_approval_reference_hash,
                loader_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
                loader_hash,
                rollback_evidence_hash,
            },
        );
    let valid_hash = module_evidence::computed_recovery_lifeline_request_reference_hash(
        module_evidence::RecoveryLifelineRequestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            retained_rollback_evidence_reference_event_id: rollback_evidence_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            rollback_evidence_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    let valid = RecoveryLifelineRequestReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        lifeline_request_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        retained_local_approval_reference_event_id: Some(local_approval_event_id),
        retained_loader_reference_event_id: Some(loader_event_id),
        retained_rollback_evidence_reference_event_id: Some(rollback_evidence_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        local_approval_reference_hash: Some(local_approval_reference_hash),
        loader_reference_hash: Some(loader_reference_hash),
        rollback_evidence_reference_hash: Some(rollback_evidence_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
        loader_hash: Some(loader_hash),
        rollback_evidence_hash: Some(rollback_evidence_hash),
    };
    [
        recovery_lifeline_request_selftest_case(
            "absent_reference",
            "missing",
            "recovery_lifeline_request_reference_absent",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "accepted_current_boot_lifeline_request_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_lifeline_request_reference_valid_but_lifeline_protocol_missing",
            evaluate_recovery_lifeline_request_reference(valid, false),
        ),
        recovery_lifeline_request_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_request_reference_scope_must_be_current_boot",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_identity_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    retained_identity_reference_event_id: Some("event.previous_boot.00000031"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_rollback_evidence_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_rollback_evidence_event_id_not_current_boot",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    retained_rollback_evidence_reference_event_id: Some(
                        "event.previous_boot.00000036",
                    ),
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_identity_missing",
            "rejected",
            "recovery_artifact_identity_reference_missing",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_missing",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_rollback_evidence_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_wrong_schema_or_variant",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "substituted_rollback_evidence_reference_record",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_substituted_record",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_reference_substituted_record",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "mismatched_lifeline_request_reference_hash",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    lifeline_request_reference_hash: Some([0x9d; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "rollback_evidence_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
                false,
            ),
        ),
    ]
}

fn recovery_lifeline_request_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineRequestReferenceCheck<'_>,
) -> RecoveryLifelineRequestSelfTestCase {
    RecoveryLifelineRequestSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_identity_binding_from_check(
    check: &RecoveryIdentityReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactIdentityReference> {
    Some(event_log::RecoveryArtifactIdentityReference {
        identity_reference_hash: check.identity_reference_hash?,
        artifact_hash: check.artifact_hash?,
    })
}

fn recovery_trust_binding_from_check(
    check: &RecoveryTrustReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactTrustReference> {
    Some(event_log::RecoveryArtifactTrustReference {
        trust_reference_hash: check.trust_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
    })
}

fn recovery_vm_test_binding_from_check(
    check: &RecoveryVmTestReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactVmTestReference> {
    Some(event_log::RecoveryArtifactVmTestReference {
        vm_test_reference_hash: check.vm_test_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
    })
}

fn recovery_local_approval_binding_from_check(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactLocalApprovalReference> {
    Some(event_log::RecoveryArtifactLocalApprovalReference {
        local_approval_reference_hash: check.local_approval_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
    })
}

fn recovery_loader_binding_from_check(
    check: &RecoveryLoaderReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactLoaderReference> {
    Some(event_log::RecoveryArtifactLoaderReference {
        loader_reference_hash: check.loader_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        retained_local_approval_reference_event_id: parse_current_boot_event_id(
            check.retained_local_approval_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        local_approval_reference_hash: check.local_approval_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
        loader_hash: check.loader_hash?,
    })
}

fn recovery_rollback_evidence_binding_from_check(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactRollbackEvidenceReference> {
    Some(event_log::RecoveryArtifactRollbackEvidenceReference {
        rollback_evidence_reference_hash: check.rollback_evidence_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        retained_local_approval_reference_event_id: parse_current_boot_event_id(
            check.retained_local_approval_reference_event_id?,
        )?,
        retained_loader_reference_event_id: parse_current_boot_event_id(
            check.retained_loader_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        local_approval_reference_hash: check.local_approval_reference_hash?,
        loader_reference_hash: check.loader_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
        loader_hash: check.loader_hash?,
        rollback_evidence_hash: check.rollback_evidence_hash?,
    })
}

fn recovery_lifeline_request_binding_from_check(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineRequestReference> {
    Some(event_log::RecoveryLifelineRequestReference {
        lifeline_request_reference_hash: check.lifeline_request_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        retained_local_approval_reference_event_id: parse_current_boot_event_id(
            check.retained_local_approval_reference_event_id?,
        )?,
        retained_loader_reference_event_id: parse_current_boot_event_id(
            check.retained_loader_reference_event_id?,
        )?,
        retained_rollback_evidence_reference_event_id: parse_current_boot_event_id(
            check.retained_rollback_evidence_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        local_approval_reference_hash: check.local_approval_reference_hash?,
        loader_reference_hash: check.loader_reference_hash?,
        rollback_evidence_reference_hash: check.rollback_evidence_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
        loader_hash: check.loader_hash?,
        rollback_evidence_hash: check.rollback_evidence_hash?,
    })
}

fn recovery_identity_reference_matches(
    check: &RecoveryIdentityReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactIdentityReference,
) -> bool {
    check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
}

fn recovery_trust_reference_matches(
    check: &RecoveryTrustReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactTrustReference,
) -> bool {
    check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
}

fn recovery_vm_test_reference_matches(
    check: &RecoveryVmTestReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactVmTestReference,
) -> bool {
    check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
}

fn recovery_local_approval_reference_matches(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactLocalApprovalReference,
) -> bool {
    check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
}

fn recovery_loader_reference_matches(
    check: &RecoveryLoaderReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactLoaderReference,
) -> bool {
    check.loader_reference_hash == Some(reference.loader_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check
            .retained_local_approval_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_local_approval_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
        && check.loader_hash == Some(reference.loader_hash)
}

fn recovery_rollback_evidence_reference_matches(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactRollbackEvidenceReference,
) -> bool {
    check.rollback_evidence_reference_hash == Some(reference.rollback_evidence_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check
            .retained_local_approval_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_local_approval_reference_event_id)
        && check
            .retained_loader_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_loader_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check.loader_reference_hash == Some(reference.loader_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
        && check.loader_hash == Some(reference.loader_hash)
        && check.rollback_evidence_hash == Some(reference.rollback_evidence_hash)
}

fn recovery_lifeline_request_reference_matches(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
    reference: event_log::RecoveryLifelineRequestReference,
) -> bool {
    check.lifeline_request_reference_hash == Some(reference.lifeline_request_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check
            .retained_local_approval_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_local_approval_reference_event_id)
        && check
            .retained_loader_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_loader_reference_event_id)
        && check
            .retained_rollback_evidence_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_rollback_evidence_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check.loader_reference_hash == Some(reference.loader_reference_hash)
        && check.rollback_evidence_reference_hash
            == Some(reference.rollback_evidence_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
        && check.loader_hash == Some(reference.loader_hash)
        && check.rollback_evidence_hash == Some(reference.rollback_evidence_hash)
}

fn recovery_identity_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.identity_diagnostic") {
        "recovery.identity_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn recovery_trust_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.trust_diagnostic") {
        "recovery.trust_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn recovery_vm_test_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.vm_test_diagnostic") {
        "recovery.vm_test_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn recovery_local_approval_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.local_approval_diagnostic") {
        "recovery.local_approval_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn recovery_loader_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.loader_diagnostic") {
        "recovery.loader_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn recovery_rollback_evidence_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.rollback_evidence_diagnostic") {
        "recovery.rollback_evidence_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn recovery_lifeline_request_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.lifeline_request_diagnostic") {
        "recovery.lifeline_request_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn json_missing_state(missing: bool) {
    json_str(if missing { "missing" } else { "available" });
}
