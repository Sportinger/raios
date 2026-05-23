#[derive(Clone, Copy)]
pub(crate) struct RecoveryEvidenceCandidate {
    pub(crate) retained: bool,
    pub(crate) current_boot: bool,
    pub(crate) schema_ok: bool,
    pub(crate) binding_ok: bool,
    pub(crate) binding_reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoadBindingCandidate {
    pub(crate) requested_capability: &'static str,
    pub(crate) identity: RecoveryEvidenceCandidate,
    pub(crate) trust: RecoveryEvidenceCandidate,
    pub(crate) vm_test: RecoveryEvidenceCandidate,
    pub(crate) local_approval: RecoveryEvidenceCandidate,
    pub(crate) loader: RecoveryEvidenceCandidate,
    pub(crate) rollback_evidence: RecoveryEvidenceCandidate,
    pub(crate) normal_module_capability_substituted: bool,
    pub(crate) normal_module_append_intent_substituted: bool,
    pub(crate) append_payload_hash_claimed_authority: bool,
    pub(crate) normal_module_writer_facts_substituted: bool,
    pub(crate) normal_module_service_slot_substituted: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoadBindingCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) recovery_only_capability_used: bool,
    pub(crate) accepts_normal_module_authority: bool,
    pub(crate) append_payload_hash_authority: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) loads_normal_module: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryLoadBindingSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}
