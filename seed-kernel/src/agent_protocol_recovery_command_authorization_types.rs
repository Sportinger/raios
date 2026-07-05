use crate::agent_protocol_recovery_runtime_types::CommandBindings;

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

pub(crate) struct RecoveryLifelineCommandHandlerBindingSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryLifelineStatusReadHandlerSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryRollbackPreviewAuthorizationSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryRollbackApplyAuthorizationSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryDisableModuleTargetBindingSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryRestartLastGoodTargetBindingSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) struct RecoveryLoadArtifactByHashTargetBindingSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}
