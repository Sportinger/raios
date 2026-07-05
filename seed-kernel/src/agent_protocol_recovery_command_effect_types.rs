use crate::{
    agent_protocol_recovery_runtime_types::CommandBindings, agent_protocol_support::SelftestCase,
};

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

pub(crate) type RecoveryMemoryWriteAuthoritySelfTestCase = SelftestCase;
pub(crate) type DurableAuditRollbackWriteAuthoritySelfTestCase = SelftestCase;
pub(crate) type RecoveryServiceInventorySideEffectBoundarySelfTestCase = SelftestCase;
pub(crate) type RecoveryLifelineCommandDispatchBehaviorSelfTestCase = SelftestCase;
pub(crate) type RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase = SelftestCase;
pub(crate) type RecoveryLifelineCommandSideEffectGateSelfTestCase = SelftestCase;
