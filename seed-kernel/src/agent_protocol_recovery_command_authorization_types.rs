use crate::{
    agent_protocol_recovery_runtime_types::CommandBindings, agent_protocol_support::SelftestCase,
};

pub(crate) type RecoveryLifelineCommandHandlerBindingInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineCommandHandlerBindingReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineStatusReadHandlerInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLifelineStatusReadHandlerReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryRollbackPreviewAuthorizationInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryRollbackPreviewAuthorizationReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryRollbackApplyAuthorizationInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryRollbackApplyAuthorizationReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryDisableModuleTargetBindingInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryDisableModuleTargetBindingReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryRestartLastGoodTargetBindingInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryRestartLastGoodTargetBindingReferenceCheck<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLoadArtifactByHashTargetBindingInput<'a> = CommandBindings<'a>;
pub(crate) type RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'a> = CommandBindings<'a>;

pub(crate) type RecoveryLifelineCommandHandlerBindingSelfTestCase = SelftestCase;
pub(crate) type RecoveryLifelineStatusReadHandlerSelfTestCase = SelftestCase;
pub(crate) type RecoveryRollbackPreviewAuthorizationSelfTestCase = SelftestCase;
pub(crate) type RecoveryRollbackApplyAuthorizationSelfTestCase = SelftestCase;
pub(crate) type RecoveryDisableModuleTargetBindingSelfTestCase = SelftestCase;
pub(crate) type RecoveryRestartLastGoodTargetBindingSelfTestCase = SelftestCase;
pub(crate) type RecoveryLoadArtifactByHashTargetBindingSelfTestCase = SelftestCase;
