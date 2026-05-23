#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineProtocolCandidate {
    pub(crate) request_retained: bool,
    pub(crate) request_current_boot: bool,
    pub(crate) request_schema_ok: bool,
    pub(crate) request_binding_ok: bool,
    pub(crate) request_binding_reason: &'static str,
    pub(crate) direct_openai_recovery_shortcut_used: bool,
    pub(crate) lifeline_protocol_state_present: bool,
    pub(crate) command_vocabulary_present: bool,
    pub(crate) loader_runtime_isolation_present: bool,
    pub(crate) rollback_transaction_engine_present: bool,
    pub(crate) durable_audit_rollback_persistence_present: bool,
    pub(crate) recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineProtocolCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) request_chain_valid: bool,
    pub(crate) can_report_protocol_gaps: bool,
    pub(crate) authorizes_recovery_load: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_loader: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryLifelineProtocolSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandVocabularyCandidate {
    pub(crate) protocol_candidate: RecoveryLifelineProtocolCandidate,
    pub(crate) protocol_state_retained: bool,
    pub(crate) protocol_state_current_boot: bool,
    pub(crate) protocol_state_schema_ok: bool,
    pub(crate) protocol_state_binding_ok: bool,
    pub(crate) protocol_state_binding_reason: &'static str,
    pub(crate) direct_openai_recovery_shortcut_used: bool,
    pub(crate) loader_runtime_isolation_present: bool,
    pub(crate) rollback_transaction_engine_present: bool,
    pub(crate) durable_audit_rollback_persistence_present: bool,
    pub(crate) recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandVocabularyCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) request_chain_valid: bool,
    pub(crate) command_vocabulary_exposed: bool,
    pub(crate) command_execution_enabled: bool,
    pub(crate) accepts_lifeline_command_envelope: bool,
    pub(crate) authorizes_recovery_load: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_loader: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryLifelineCommandVocabularySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}
