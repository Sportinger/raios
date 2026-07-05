use crate::agent_protocol_recovery_runtime_types::CommandBindings;

pub(crate) type RecoveryMemoryWriteAuthorityInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryMemoryWriteAuthorityReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type DurableAuditRollbackWriteAuthorityInput<'a> = CommandBindings<'a>;
pub(crate) type DurableAuditRollbackWriteAuthorityReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryServiceInventorySideEffectBoundaryInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineCommandDispatchBehaviorInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineCommandExecutorCapabilityTableInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'a> =
    CommandBindings<'a>;
pub(crate) type RecoveryLifelineCommandSideEffectGateInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineCommandSideEffectGateReferenceCheck<'a> = CommandBindings<'a>;

pub(crate) struct RecoveryMemoryWriteAuthoritySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct DurableAuditRollbackWriteAuthoritySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryServiceInventorySideEffectBoundarySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryLifelineCommandDispatchBehaviorSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryLifelineCommandSideEffectGateSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}
