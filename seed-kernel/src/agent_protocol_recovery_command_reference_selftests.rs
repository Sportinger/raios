use crate::{
    agent_protocol_recovery_command_authorization_types::*,
    agent_protocol_recovery_command_effect_reference_eval::*,
    agent_protocol_recovery_command_effect_types::*,
    agent_protocol_recovery_command_reference_eval::*, agent_protocol_recovery_constants::*,
    agent_protocol_recovery_lifeline::RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    agent_protocol_support::method_eq, module_evidence,
};

pub(crate) fn recovery_lifeline_command_handler_binding_selftest_cases(
) -> [RecoveryLifelineCommandHandlerBindingSelfTestCase;
       RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_SELFTEST_CASES] {
    let valid_input = RecoveryLifelineCommandHandlerBindingInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        handler_binding_hash: None,
        retained_command_body_canonicalization_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x31; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x32; 32]),
        command_body_canonicalization_hash: Some([0x33; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        handler_id: Some(RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID),
        handler_input_binding_hash: Some([0x34; 32]),
    };
    let expected = module_evidence::computed_recovery_lifeline_command_handler_binding_hash(
        module_evidence::RecoveryLifelineCommandHandlerBindingHashInput {
            retained_command_body_canonicalization_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x31; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x32; 32],
            command_body_canonicalization_hash: [0x33; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            handler_id: RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID,
            handler_input_binding_hash: [0x34; 32],
        },
    );
    let mut valid = valid_input;
    valid.handler_binding_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut handler = valid;
    handler.handler_id = Some("boundary.recovery_lifeline_command_handler_binding.wrong");
    let mut hash = valid;
    hash.handler_binding_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_lifeline_command_handler_binding_selftest_case(
            "handler_binding_absent",
            "missing",
            "recovery_lifeline_command_handler_binding_absent",
            evaluate_recovery_lifeline_command_handler_binding_reference(missing, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "handler_binding_arity_invalid",
            "invalid_reference",
            "recovery_lifeline_command_handler_binding_arity_invalid",
            evaluate_recovery_lifeline_command_handler_binding_reference(arity, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "previous_boot_handler_binding",
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_handler_binding_scope_must_be_current_boot",
            evaluate_recovery_lifeline_command_handler_binding_reference(previous, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_lifeline_command_handler_binding_reference(unsupported, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_lifeline_command_handler_binding_reference(schema, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_lifeline_command_handler_binding_reference(boundary, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "handler_id_mismatch",
            "rejected",
            "recovery_lifeline_command_handler_id_mismatch",
            evaluate_recovery_lifeline_command_handler_binding_reference(handler, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "handler_binding_hash_mismatch",
            "mismatched_command_handler_binding_hash",
            "recovery_lifeline_command_handler_binding_hash_mismatch",
            evaluate_recovery_lifeline_command_handler_binding_reference(hash, false),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "retained_body_reference_missing",
            "rejected",
            "retained_recovery_lifeline_command_body_canonicalization_missing",
            evaluate_recovery_lifeline_command_handler_binding_reference(live_missing, true),
        ),
        recovery_lifeline_command_handler_binding_selftest_case(
            "all_inputs_present_handler_binding_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_lifeline_command_handler_binding_valid_but_command_dispatch_disabled",
            evaluate_recovery_lifeline_command_handler_binding_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_lifeline_command_handler_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandHandlerBindingReferenceCheck<'_>,
) -> RecoveryLifelineCommandHandlerBindingSelfTestCase {
    RecoveryLifelineCommandHandlerBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_lifeline_status_read_handler_selftest_cases(
) -> [RecoveryLifelineStatusReadHandlerSelfTestCase;
       RECOVERY_LIFELINE_STATUS_READ_HANDLER_SELFTEST_CASES] {
    let valid_input = RecoveryLifelineStatusReadHandlerInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        status_read_handler_hash: None,
        retained_command_handler_binding_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x41; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x42; 32]),
        command_body_canonicalization_hash: Some([0x43; 32]),
        handler_binding_hash: Some([0x44; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        status_handler_id: Some(RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID),
        status_read_projection_hash: Some([0x45; 32]),
    };
    let expected = module_evidence::computed_recovery_lifeline_status_read_handler_hash(
        module_evidence::RecoveryLifelineStatusReadHandlerHashInput {
            retained_command_handler_binding_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x41; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x42; 32],
            command_body_canonicalization_hash: [0x43; 32],
            handler_binding_hash: [0x44; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            status_handler_id: RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID,
            status_read_projection_hash: [0x45; 32],
        },
    );
    let mut valid = valid_input;
    valid.status_read_handler_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut handler = valid;
    handler.status_handler_id = Some("boundary.recovery_lifeline_status_read_handler.wrong");
    let mut hash = valid;
    hash.status_read_handler_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_lifeline_status_read_handler_selftest_case(
            "status_read_handler_absent",
            "missing",
            "recovery_lifeline_status_read_handler_absent",
            evaluate_recovery_lifeline_status_read_handler_reference(missing, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "status_read_handler_arity_invalid",
            "invalid_reference",
            "recovery_lifeline_status_read_handler_arity_invalid",
            evaluate_recovery_lifeline_status_read_handler_reference(arity, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "previous_boot_status_read_handler",
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_status_read_handler_scope_must_be_current_boot",
            evaluate_recovery_lifeline_status_read_handler_reference(previous, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_lifeline_status_read_handler_reference(unsupported, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_lifeline_status_read_handler_reference(schema, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_lifeline_status_read_handler_reference(boundary, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "status_handler_id_mismatch",
            "rejected",
            "recovery_lifeline_status_read_handler_id_mismatch",
            evaluate_recovery_lifeline_status_read_handler_reference(handler, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "status_read_handler_hash_mismatch",
            "mismatched_status_read_handler_hash",
            "recovery_lifeline_status_read_handler_hash_mismatch",
            evaluate_recovery_lifeline_status_read_handler_reference(hash, false),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "retained_handler_binding_reference_missing",
            "rejected",
            "retained_recovery_lifeline_command_handler_binding_missing",
            evaluate_recovery_lifeline_status_read_handler_reference(live_missing, true),
        ),
        recovery_lifeline_status_read_handler_selftest_case(
            "all_inputs_present_status_read_handler_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_lifeline_status_read_handler_valid_but_command_dispatch_disabled",
            evaluate_recovery_lifeline_status_read_handler_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_lifeline_status_read_handler_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineStatusReadHandlerReferenceCheck<'_>,
) -> RecoveryLifelineStatusReadHandlerSelfTestCase {
    RecoveryLifelineStatusReadHandlerSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_rollback_preview_authorization_selftest_cases(
) -> [RecoveryRollbackPreviewAuthorizationSelfTestCase;
       RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_SELFTEST_CASES] {
    let valid_input = RecoveryRollbackPreviewAuthorizationInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        rollback_preview_authorization_hash: None,
        retained_status_read_handler_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x51; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x52; 32]),
        command_body_canonicalization_hash: Some([0x53; 32]),
        handler_binding_hash: Some([0x54; 32]),
        status_read_handler_hash: Some([0x55; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        rollback_preview_authorization_id: Some(
            RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID,
        ),
        rollback_preview_projection_hash: Some([0x56; 32]),
    };
    let expected = module_evidence::computed_recovery_rollback_preview_authorization_hash(
        module_evidence::RecoveryRollbackPreviewAuthorizationHashInput {
            retained_status_read_handler_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x51; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x52; 32],
            command_body_canonicalization_hash: [0x53; 32],
            handler_binding_hash: [0x54; 32],
            status_read_handler_hash: [0x55; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            rollback_preview_authorization_id: RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID,
            rollback_preview_projection_hash: [0x56; 32],
        },
    );
    let mut valid = valid_input;
    valid.rollback_preview_authorization_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut authorization = valid;
    authorization.rollback_preview_authorization_id =
        Some("boundary.recovery_rollback_preview_authorization.wrong");
    let mut hash = valid;
    hash.rollback_preview_authorization_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_rollback_preview_authorization_selftest_case(
            "rollback_preview_authorization_absent",
            "missing",
            "recovery_rollback_preview_authorization_absent",
            evaluate_recovery_rollback_preview_authorization_reference(missing, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "rollback_preview_authorization_arity_invalid",
            "invalid_reference",
            "recovery_rollback_preview_authorization_arity_invalid",
            evaluate_recovery_rollback_preview_authorization_reference(arity, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "previous_boot_rollback_preview_authorization",
            "stale_or_non_current_boot_reference",
            "recovery_rollback_preview_authorization_scope_must_be_current_boot",
            evaluate_recovery_rollback_preview_authorization_reference(previous, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_rollback_preview_authorization_reference(unsupported, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_rollback_preview_authorization_reference(schema, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_rollback_preview_authorization_reference(boundary, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "rollback_preview_authorization_id_mismatch",
            "rejected",
            "recovery_rollback_preview_authorization_id_mismatch",
            evaluate_recovery_rollback_preview_authorization_reference(authorization, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "rollback_preview_authorization_hash_mismatch",
            "mismatched_rollback_preview_authorization_hash",
            "recovery_rollback_preview_authorization_hash_mismatch",
            evaluate_recovery_rollback_preview_authorization_reference(hash, false),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "retained_status_read_handler_reference_missing",
            "rejected",
            "retained_recovery_lifeline_status_read_handler_missing",
            evaluate_recovery_rollback_preview_authorization_reference(live_missing, true),
        ),
        recovery_rollback_preview_authorization_selftest_case(
            "all_inputs_present_rollback_preview_authorization_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_rollback_preview_authorization_valid_but_command_dispatch_disabled",
            evaluate_recovery_rollback_preview_authorization_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_rollback_preview_authorization_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryRollbackPreviewAuthorizationReferenceCheck<'_>,
) -> RecoveryRollbackPreviewAuthorizationSelfTestCase {
    RecoveryRollbackPreviewAuthorizationSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_rollback_apply_authorization_selftest_cases(
) -> [RecoveryRollbackApplyAuthorizationSelfTestCase;
       RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_SELFTEST_CASES] {
    let valid_input = RecoveryRollbackApplyAuthorizationInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        rollback_apply_authorization_hash: None,
        retained_rollback_preview_authorization_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x61; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x62; 32]),
        command_body_canonicalization_hash: Some([0x63; 32]),
        handler_binding_hash: Some([0x64; 32]),
        status_read_handler_hash: Some([0x65; 32]),
        rollback_preview_authorization_hash: Some([0x66; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        rollback_apply_authorization_id: Some(RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID),
        rollback_apply_projection_hash: Some([0x67; 32]),
    };
    let expected = module_evidence::computed_recovery_rollback_apply_authorization_hash(
        module_evidence::RecoveryRollbackApplyAuthorizationHashInput {
            retained_rollback_preview_authorization_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x61; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x62; 32],
            command_body_canonicalization_hash: [0x63; 32],
            handler_binding_hash: [0x64; 32],
            status_read_handler_hash: [0x65; 32],
            rollback_preview_authorization_hash: [0x66; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            rollback_apply_authorization_id: RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID,
            rollback_apply_projection_hash: [0x67; 32],
        },
    );
    let mut valid = valid_input;
    valid.rollback_apply_authorization_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut authorization = valid;
    authorization.rollback_apply_authorization_id =
        Some("boundary.recovery_rollback_apply_authorization.wrong");
    let mut hash = valid;
    hash.rollback_apply_authorization_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_rollback_apply_authorization_selftest_case(
            "rollback_apply_authorization_absent",
            "missing",
            "recovery_rollback_apply_authorization_absent",
            evaluate_recovery_rollback_apply_authorization_reference(missing, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "rollback_apply_authorization_arity_invalid",
            "invalid_reference",
            "recovery_rollback_apply_authorization_arity_invalid",
            evaluate_recovery_rollback_apply_authorization_reference(arity, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "previous_boot_rollback_apply_authorization",
            "stale_or_non_current_boot_reference",
            "recovery_rollback_apply_authorization_scope_must_be_current_boot",
            evaluate_recovery_rollback_apply_authorization_reference(previous, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_rollback_apply_authorization_reference(unsupported, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_rollback_apply_authorization_reference(schema, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_rollback_apply_authorization_reference(boundary, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "rollback_apply_authorization_id_mismatch",
            "rejected",
            "recovery_rollback_apply_authorization_id_mismatch",
            evaluate_recovery_rollback_apply_authorization_reference(authorization, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "rollback_apply_authorization_hash_mismatch",
            "mismatched_rollback_apply_authorization_hash",
            "recovery_rollback_apply_authorization_hash_mismatch",
            evaluate_recovery_rollback_apply_authorization_reference(hash, false),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "retained_rollback_preview_authorization_reference_missing",
            "rejected",
            "retained_recovery_rollback_preview_authorization_missing",
            evaluate_recovery_rollback_apply_authorization_reference(live_missing, true),
        ),
        recovery_rollback_apply_authorization_selftest_case(
            "all_inputs_present_rollback_apply_authorization_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_rollback_apply_authorization_valid_but_command_dispatch_disabled",
            evaluate_recovery_rollback_apply_authorization_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_rollback_apply_authorization_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryRollbackApplyAuthorizationReferenceCheck<'_>,
) -> RecoveryRollbackApplyAuthorizationSelfTestCase {
    RecoveryRollbackApplyAuthorizationSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_disable_module_target_binding_selftest_cases(
) -> [RecoveryDisableModuleTargetBindingSelfTestCase;
       RECOVERY_DISABLE_MODULE_TARGET_BINDING_SELFTEST_CASES] {
    let valid_input = RecoveryDisableModuleTargetBindingInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        disable_module_target_binding_hash: None,
        retained_rollback_apply_authorization_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x61; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x62; 32]),
        command_body_canonicalization_hash: Some([0x63; 32]),
        handler_binding_hash: Some([0x64; 32]),
        status_read_handler_hash: Some([0x65; 32]),
        rollback_preview_authorization_hash: Some([0x66; 32]),
        rollback_apply_authorization_hash: Some([0x67; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        disable_module_target_id: Some(RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID),
        disable_module_target_projection_hash: Some([0x68; 32]),
    };
    let expected = module_evidence::computed_recovery_disable_module_target_binding_hash(
        module_evidence::RecoveryDisableModuleTargetBindingHashInput {
            retained_rollback_apply_authorization_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x61; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x62; 32],
            command_body_canonicalization_hash: [0x63; 32],
            handler_binding_hash: [0x64; 32],
            status_read_handler_hash: [0x65; 32],
            rollback_preview_authorization_hash: [0x66; 32],
            rollback_apply_authorization_hash: [0x67; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            disable_module_target_id: RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID,
            disable_module_target_projection_hash: [0x68; 32],
        },
    );
    let mut valid = valid_input;
    valid.disable_module_target_binding_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut target = valid;
    target.disable_module_target_id = Some("boundary.recovery_disable_module_target_binding.wrong");
    let mut hash = valid;
    hash.disable_module_target_binding_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_disable_module_target_binding_selftest_case(
            "disable_module_target_binding_absent",
            "missing",
            "recovery_disable_module_target_binding_absent",
            evaluate_recovery_disable_module_target_binding_reference(missing, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "disable_module_target_binding_arity_invalid",
            "invalid_reference",
            "recovery_disable_module_target_binding_arity_invalid",
            evaluate_recovery_disable_module_target_binding_reference(arity, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "previous_boot_disable_module_target_binding",
            "stale_or_non_current_boot_reference",
            "recovery_disable_module_target_binding_scope_must_be_current_boot",
            evaluate_recovery_disable_module_target_binding_reference(previous, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_disable_module_target_binding_reference(unsupported, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_disable_module_target_binding_reference(schema, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_disable_module_target_binding_reference(boundary, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "disable_module_target_id_mismatch",
            "rejected",
            "recovery_disable_module_target_id_mismatch",
            evaluate_recovery_disable_module_target_binding_reference(target, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "disable_module_target_binding_hash_mismatch",
            "mismatched_disable_module_target_binding_hash",
            "recovery_disable_module_target_binding_hash_mismatch",
            evaluate_recovery_disable_module_target_binding_reference(hash, false),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "retained_rollback_apply_authorization_reference_missing",
            "rejected",
            "retained_recovery_rollback_apply_authorization_missing",
            evaluate_recovery_disable_module_target_binding_reference(live_missing, true),
        ),
        recovery_disable_module_target_binding_selftest_case(
            "all_inputs_present_disable_module_target_binding_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_disable_module_target_binding_valid_but_command_dispatch_disabled",
            evaluate_recovery_disable_module_target_binding_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_disable_module_target_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryDisableModuleTargetBindingReferenceCheck<'_>,
) -> RecoveryDisableModuleTargetBindingSelfTestCase {
    RecoveryDisableModuleTargetBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_restart_last_good_target_binding_selftest_cases(
) -> [RecoveryRestartLastGoodTargetBindingSelfTestCase;
       RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_SELFTEST_CASES] {
    let valid_input = RecoveryRestartLastGoodTargetBindingInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        restart_last_good_target_binding_hash: None,
        retained_disable_module_target_binding_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x61; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x62; 32]),
        command_body_canonicalization_hash: Some([0x63; 32]),
        handler_binding_hash: Some([0x64; 32]),
        status_read_handler_hash: Some([0x65; 32]),
        rollback_preview_authorization_hash: Some([0x66; 32]),
        rollback_apply_authorization_hash: Some([0x67; 32]),
        disable_module_target_binding_hash: Some([0x68; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        restart_last_good_target_id: Some(RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID),
        restart_last_good_target_projection_hash: Some([0x69; 32]),
    };
    let expected = module_evidence::computed_recovery_restart_last_good_target_binding_hash(
        module_evidence::RecoveryRestartLastGoodTargetBindingHashInput {
            retained_disable_module_target_binding_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x61; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x62; 32],
            command_body_canonicalization_hash: [0x63; 32],
            handler_binding_hash: [0x64; 32],
            status_read_handler_hash: [0x65; 32],
            rollback_preview_authorization_hash: [0x66; 32],
            rollback_apply_authorization_hash: [0x67; 32],
            disable_module_target_binding_hash: [0x68; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            restart_last_good_target_id: RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID,
            restart_last_good_target_projection_hash: [0x69; 32],
        },
    );
    let mut valid = valid_input;
    valid.restart_last_good_target_binding_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut target = valid;
    target.restart_last_good_target_id =
        Some("boundary.recovery_restart_last_good_target_binding.wrong");
    let mut hash = valid;
    hash.restart_last_good_target_binding_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_restart_last_good_target_binding_selftest_case(
            "restart_last_good_target_binding_absent",
            "missing",
            "recovery_restart_last_good_target_binding_absent",
            evaluate_recovery_restart_last_good_target_binding_reference(missing, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "restart_last_good_target_binding_arity_invalid",
            "invalid_reference",
            "recovery_restart_last_good_target_binding_arity_invalid",
            evaluate_recovery_restart_last_good_target_binding_reference(arity, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "previous_boot_restart_last_good_target_binding",
            "stale_or_non_current_boot_reference",
            "recovery_restart_last_good_target_binding_scope_must_be_current_boot",
            evaluate_recovery_restart_last_good_target_binding_reference(previous, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_restart_last_good_target_binding_reference(unsupported, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_restart_last_good_target_binding_reference(schema, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_restart_last_good_target_binding_reference(boundary, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "restart_last_good_target_id_mismatch",
            "rejected",
            "recovery_restart_last_good_target_id_mismatch",
            evaluate_recovery_restart_last_good_target_binding_reference(target, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "restart_last_good_target_binding_hash_mismatch",
            "mismatched_restart_last_good_target_binding_hash",
            "recovery_restart_last_good_target_binding_hash_mismatch",
            evaluate_recovery_restart_last_good_target_binding_reference(hash, false),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "retained_disable_module_target_binding_reference_missing",
            "rejected",
            "retained_recovery_disable_module_target_binding_missing",
            evaluate_recovery_restart_last_good_target_binding_reference(live_missing, true),
        ),
        recovery_restart_last_good_target_binding_selftest_case(
            "all_inputs_present_restart_last_good_target_binding_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_restart_last_good_target_binding_valid_but_command_dispatch_disabled",
            evaluate_recovery_restart_last_good_target_binding_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_restart_last_good_target_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryRestartLastGoodTargetBindingReferenceCheck<'_>,
) -> RecoveryRestartLastGoodTargetBindingSelfTestCase {
    RecoveryRestartLastGoodTargetBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_selftest_cases(
) -> [RecoveryLoadArtifactByHashTargetBindingSelfTestCase;
       RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_SELFTEST_CASES] {
    let valid_input = RecoveryLoadArtifactByHashTargetBindingInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        load_artifact_by_hash_target_binding_hash: None,
        retained_restart_last_good_target_binding_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x61; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x62; 32]),
        command_body_canonicalization_hash: Some([0x63; 32]),
        handler_binding_hash: Some([0x64; 32]),
        status_read_handler_hash: Some([0x65; 32]),
        rollback_preview_authorization_hash: Some([0x66; 32]),
        rollback_apply_authorization_hash: Some([0x67; 32]),
        disable_module_target_binding_hash: Some([0x68; 32]),
        restart_last_good_target_binding_hash: Some([0x69; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        load_artifact_by_hash_target_id: Some(
            RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID,
        ),
        load_artifact_by_hash_target_artifact_hash: Some([0x6a; 32]),
        load_artifact_by_hash_target_projection_hash: Some([0x6b; 32]),
    };
    let expected = module_evidence::computed_recovery_load_artifact_by_hash_target_binding_hash(
        module_evidence::RecoveryLoadArtifactByHashTargetBindingHashInput {
            retained_restart_last_good_target_binding_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x61; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x62; 32],
            command_body_canonicalization_hash: [0x63; 32],
            handler_binding_hash: [0x64; 32],
            status_read_handler_hash: [0x65; 32],
            rollback_preview_authorization_hash: [0x66; 32],
            rollback_apply_authorization_hash: [0x67; 32],
            disable_module_target_binding_hash: [0x68; 32],
            restart_last_good_target_binding_hash: [0x69; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            load_artifact_by_hash_target_id:
                RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID,
            load_artifact_by_hash_target_artifact_hash: [0x6a; 32],
            load_artifact_by_hash_target_projection_hash: [0x6b; 32],
        },
    );
    let mut valid = valid_input;
    valid.load_artifact_by_hash_target_binding_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut target = valid;
    target.load_artifact_by_hash_target_id =
        Some("boundary.recovery_load_artifact_by_hash_target_binding.wrong");
    let mut hash = valid;
    hash.load_artifact_by_hash_target_binding_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "load_artifact_by_hash_target_binding_absent",
            "missing",
            "recovery_load_artifact_by_hash_target_binding_absent",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(missing, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "load_artifact_by_hash_target_binding_arity_invalid",
            "invalid_reference",
            "recovery_load_artifact_by_hash_target_binding_arity_invalid",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(arity, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "previous_boot_load_artifact_by_hash_target_binding",
            "stale_or_non_current_boot_reference",
            "recovery_load_artifact_by_hash_target_binding_scope_must_be_current_boot",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(previous, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(unsupported, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(schema, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(boundary, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "load_artifact_by_hash_target_id_mismatch",
            "rejected",
            "recovery_load_artifact_by_hash_target_id_mismatch",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(target, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "load_artifact_by_hash_target_binding_hash_mismatch",
            "mismatched_load_artifact_by_hash_target_binding_hash",
            "recovery_load_artifact_by_hash_target_binding_hash_mismatch",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(hash, false),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "retained_restart_last_good_target_binding_reference_missing",
            "rejected",
            "retained_recovery_restart_last_good_target_binding_missing",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(live_missing, true),
        ),
        recovery_load_artifact_by_hash_target_binding_selftest_case(
            "all_inputs_present_load_artifact_by_hash_target_binding_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_load_artifact_by_hash_target_binding_valid_but_command_dispatch_disabled",
            evaluate_recovery_load_artifact_by_hash_target_binding_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_>,
) -> RecoveryLoadArtifactByHashTargetBindingSelfTestCase {
    RecoveryLoadArtifactByHashTargetBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_memory_write_authority_selftest_cases(
) -> [RecoveryMemoryWriteAuthoritySelfTestCase; RECOVERY_MEMORY_WRITE_AUTHORITY_SELFTEST_CASES] {
    let valid_input = RecoveryMemoryWriteAuthorityInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        recovery_memory_write_authority_hash: None,
        retained_load_artifact_by_hash_target_binding_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x61; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x62; 32]),
        command_body_canonicalization_hash: Some([0x63; 32]),
        handler_binding_hash: Some([0x64; 32]),
        status_read_handler_hash: Some([0x65; 32]),
        rollback_preview_authorization_hash: Some([0x66; 32]),
        rollback_apply_authorization_hash: Some([0x67; 32]),
        disable_module_target_binding_hash: Some([0x68; 32]),
        restart_last_good_target_binding_hash: Some([0x69; 32]),
        load_artifact_by_hash_target_binding_hash: Some([0x6a; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        recovery_memory_write_authority_id: Some(RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID),
        recovery_memory_projection_hash: Some([0x6b; 32]),
    };
    let expected = module_evidence::computed_recovery_memory_write_authority_hash(
        module_evidence::RecoveryMemoryWriteAuthorityHashInput {
            retained_load_artifact_by_hash_target_binding_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x61; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x62; 32],
            command_body_canonicalization_hash: [0x63; 32],
            handler_binding_hash: [0x64; 32],
            status_read_handler_hash: [0x65; 32],
            rollback_preview_authorization_hash: [0x66; 32],
            rollback_apply_authorization_hash: [0x67; 32],
            disable_module_target_binding_hash: [0x68; 32],
            restart_last_good_target_binding_hash: [0x69; 32],
            load_artifact_by_hash_target_binding_hash: [0x6a; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            recovery_memory_write_authority_id: RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID,
            recovery_memory_projection_hash: [0x6b; 32],
        },
    );
    let mut valid = valid_input;
    valid.recovery_memory_write_authority_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut authority = valid;
    authority.recovery_memory_write_authority_id =
        Some("boundary.recovery_memory_write_authority.wrong");
    let mut hash = valid;
    hash.recovery_memory_write_authority_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_memory_write_authority_selftest_case(
            "recovery_memory_write_authority_absent",
            "missing",
            "recovery_memory_write_authority_absent",
            evaluate_recovery_memory_write_authority_reference(missing, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "recovery_memory_write_authority_arity_invalid",
            "invalid_reference",
            "recovery_memory_write_authority_arity_invalid",
            evaluate_recovery_memory_write_authority_reference(arity, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "previous_boot_recovery_memory_write_authority",
            "stale_or_non_current_boot_reference",
            "recovery_memory_write_authority_scope_must_be_current_boot",
            evaluate_recovery_memory_write_authority_reference(previous, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_memory_write_authority_reference(unsupported, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_memory_write_authority_reference(schema, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_memory_write_authority_reference(boundary, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "recovery_memory_write_authority_id_mismatch",
            "rejected",
            "recovery_memory_write_authority_id_mismatch",
            evaluate_recovery_memory_write_authority_reference(authority, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "recovery_memory_write_authority_hash_mismatch",
            "mismatched_recovery_memory_write_authority_hash",
            "recovery_memory_write_authority_hash_mismatch",
            evaluate_recovery_memory_write_authority_reference(hash, false),
        ),
        recovery_memory_write_authority_selftest_case(
            "retained_load_artifact_by_hash_target_binding_reference_missing",
            "rejected",
            "retained_recovery_load_artifact_by_hash_target_binding_missing",
            evaluate_recovery_memory_write_authority_reference(live_missing, true),
        ),
        recovery_memory_write_authority_selftest_case(
            "all_inputs_present_recovery_memory_write_authority_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_memory_write_authority_valid_but_memory_writes_disabled",
            evaluate_recovery_memory_write_authority_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_memory_write_authority_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryMemoryWriteAuthorityReferenceCheck<'_>,
) -> RecoveryMemoryWriteAuthoritySelfTestCase {
    RecoveryMemoryWriteAuthoritySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn durable_audit_rollback_write_authority_selftest_cases(
) -> [DurableAuditRollbackWriteAuthoritySelfTestCase;
       DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_SELFTEST_CASES] {
    let valid_input = DurableAuditRollbackWriteAuthorityInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        durable_audit_rollback_write_authority_hash: None,
        retained_recovery_memory_write_authority_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x71; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x72; 32]),
        command_body_canonicalization_hash: Some([0x73; 32]),
        handler_binding_hash: Some([0x74; 32]),
        status_read_handler_hash: Some([0x75; 32]),
        rollback_preview_authorization_hash: Some([0x76; 32]),
        rollback_apply_authorization_hash: Some([0x77; 32]),
        disable_module_target_binding_hash: Some([0x78; 32]),
        restart_last_good_target_binding_hash: Some([0x79; 32]),
        load_artifact_by_hash_target_binding_hash: Some([0x7a; 32]),
        recovery_memory_write_authority_hash: Some([0x7b; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        durable_audit_rollback_write_authority_id: Some(
            DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID,
        ),
        durable_audit_rollback_projection_hash: Some([0x7c; 32]),
    };
    let expected = module_evidence::computed_durable_audit_rollback_write_authority_hash(
        module_evidence::DurableAuditRollbackWriteAuthorityHashInput {
            retained_recovery_memory_write_authority_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x71; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x72; 32],
            command_body_canonicalization_hash: [0x73; 32],
            handler_binding_hash: [0x74; 32],
            status_read_handler_hash: [0x75; 32],
            rollback_preview_authorization_hash: [0x76; 32],
            rollback_apply_authorization_hash: [0x77; 32],
            disable_module_target_binding_hash: [0x78; 32],
            restart_last_good_target_binding_hash: [0x79; 32],
            load_artifact_by_hash_target_binding_hash: [0x7a; 32],
            recovery_memory_write_authority_hash: [0x7b; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            durable_audit_rollback_write_authority_id:
                DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID,
            durable_audit_rollback_projection_hash: [0x7c; 32],
        },
    );
    let mut valid = valid_input;
    valid.durable_audit_rollback_write_authority_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut authority = valid;
    authority.durable_audit_rollback_write_authority_id =
        Some("boundary.durable_audit_rollback_write_authority.wrong");
    let mut hash = valid;
    hash.durable_audit_rollback_write_authority_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        durable_audit_rollback_write_authority_selftest_case(
            "durable_audit_rollback_write_authority_absent",
            "missing",
            "durable_audit_rollback_write_authority_absent",
            evaluate_durable_audit_rollback_write_authority_reference(missing, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "durable_audit_rollback_write_authority_arity_invalid",
            "invalid_reference",
            "durable_audit_rollback_write_authority_arity_invalid",
            evaluate_durable_audit_rollback_write_authority_reference(arity, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "previous_boot_durable_audit_rollback_write_authority",
            "stale_or_non_current_boot_reference",
            "durable_audit_rollback_write_authority_scope_must_be_current_boot",
            evaluate_durable_audit_rollback_write_authority_reference(previous, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_durable_audit_rollback_write_authority_reference(unsupported, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_durable_audit_rollback_write_authority_reference(schema, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_durable_audit_rollback_write_authority_reference(boundary, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "durable_audit_rollback_write_authority_id_mismatch",
            "rejected",
            "durable_audit_rollback_write_authority_id_mismatch",
            evaluate_durable_audit_rollback_write_authority_reference(authority, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "durable_audit_rollback_write_authority_hash_mismatch",
            "mismatched_durable_audit_rollback_write_authority_hash",
            "durable_audit_rollback_write_authority_hash_mismatch",
            evaluate_durable_audit_rollback_write_authority_reference(hash, false),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "retained_recovery_memory_write_authority_reference_missing",
            "rejected",
            "retained_recovery_memory_write_authority_missing",
            evaluate_durable_audit_rollback_write_authority_reference(live_missing, true),
        ),
        durable_audit_rollback_write_authority_selftest_case(
            "all_inputs_present_durable_audit_rollback_write_authority_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "durable_audit_rollback_write_authority_valid_but_durable_writes_disabled",
            evaluate_durable_audit_rollback_write_authority_reference(valid, false),
        ),
    ]
}

pub(crate) fn durable_audit_rollback_write_authority_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: DurableAuditRollbackWriteAuthorityReferenceCheck<'_>,
) -> DurableAuditRollbackWriteAuthoritySelfTestCase {
    DurableAuditRollbackWriteAuthoritySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_selftest_cases(
) -> [RecoveryServiceInventorySideEffectBoundarySelfTestCase;
       RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_SELFTEST_CASES] {
    let valid_input = RecoveryServiceInventorySideEffectBoundaryInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        service_inventory_side_effect_boundary_hash: None,
        retained_durable_audit_rollback_write_authority_event_id: Some(
            "event.current_boot.00000001",
        ),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x81; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x82; 32]),
        command_body_canonicalization_hash: Some([0x83; 32]),
        handler_binding_hash: Some([0x84; 32]),
        status_read_handler_hash: Some([0x85; 32]),
        rollback_preview_authorization_hash: Some([0x86; 32]),
        rollback_apply_authorization_hash: Some([0x87; 32]),
        disable_module_target_binding_hash: Some([0x88; 32]),
        restart_last_good_target_binding_hash: Some([0x89; 32]),
        load_artifact_by_hash_target_binding_hash: Some([0x8a; 32]),
        recovery_memory_write_authority_hash: Some([0x8b; 32]),
        durable_audit_rollback_write_authority_hash: Some([0x8c; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        service_inventory_side_effect_boundary_id: Some(
            RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID,
        ),
        service_inventory_projection_hash: Some([0x8d; 32]),
    };
    let expected = module_evidence::computed_recovery_service_inventory_side_effect_boundary_hash(
        module_evidence::RecoveryServiceInventorySideEffectBoundaryHashInput {
            retained_durable_audit_rollback_write_authority_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x81; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x82; 32],
            command_body_canonicalization_hash: [0x83; 32],
            handler_binding_hash: [0x84; 32],
            status_read_handler_hash: [0x85; 32],
            rollback_preview_authorization_hash: [0x86; 32],
            rollback_apply_authorization_hash: [0x87; 32],
            disable_module_target_binding_hash: [0x88; 32],
            restart_last_good_target_binding_hash: [0x89; 32],
            load_artifact_by_hash_target_binding_hash: [0x8a; 32],
            recovery_memory_write_authority_hash: [0x8b; 32],
            durable_audit_rollback_write_authority_hash: [0x8c; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            service_inventory_side_effect_boundary_id:
                RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID,
            service_inventory_projection_hash: [0x8d; 32],
        },
    );
    let mut valid = valid_input;
    valid.service_inventory_side_effect_boundary_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut side_effect_boundary = valid;
    side_effect_boundary.service_inventory_side_effect_boundary_id =
        Some("boundary.recovery_service_inventory_side_effect_boundary.wrong");
    let mut hash = valid;
    hash.service_inventory_side_effect_boundary_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "recovery_service_inventory_side_effect_boundary_absent",
            "missing",
            "recovery_service_inventory_side_effect_boundary_absent",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(missing, false),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "recovery_service_inventory_side_effect_boundary_arity_invalid",
            "invalid_reference",
            "recovery_service_inventory_side_effect_boundary_arity_invalid",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(arity, false),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "previous_boot_recovery_service_inventory_side_effect_boundary",
            "stale_or_non_current_boot_reference",
            "recovery_service_inventory_side_effect_boundary_scope_must_be_current_boot",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(previous, false),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(unsupported, false),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(schema, false),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(boundary, false),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "service_inventory_side_effect_boundary_id_mismatch",
            "rejected",
            "recovery_service_inventory_side_effect_boundary_id_mismatch",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(
                side_effect_boundary,
                false,
            ),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "service_inventory_side_effect_boundary_hash_mismatch",
            "mismatched_recovery_service_inventory_side_effect_boundary_hash",
            "recovery_service_inventory_side_effect_boundary_hash_mismatch",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(hash, false),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "retained_durable_audit_rollback_write_authority_reference_missing",
            "rejected",
            "retained_durable_audit_rollback_write_authority_missing",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(live_missing, true),
        ),
        recovery_service_inventory_side_effect_boundary_selftest_case(
            "all_inputs_present_service_inventory_side_effect_boundary_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_service_inventory_side_effect_boundary_valid_but_service_inventory_unchanged",
            evaluate_recovery_service_inventory_side_effect_boundary_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'_>,
) -> RecoveryServiceInventorySideEffectBoundarySelfTestCase {
    RecoveryServiceInventorySideEffectBoundarySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_selftest_cases(
) -> [RecoveryLifelineCommandDispatchBehaviorSelfTestCase;
       RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_SELFTEST_CASES] {
    let valid_input = RecoveryLifelineCommandDispatchBehaviorInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        command_dispatch_behavior_hash: None,
        retained_service_inventory_side_effect_boundary_event_id: Some(
            "event.current_boot.00000001",
        ),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0x91; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0x92; 32]),
        command_body_canonicalization_hash: Some([0x93; 32]),
        handler_binding_hash: Some([0x94; 32]),
        status_read_handler_hash: Some([0x95; 32]),
        rollback_preview_authorization_hash: Some([0x96; 32]),
        rollback_apply_authorization_hash: Some([0x97; 32]),
        disable_module_target_binding_hash: Some([0x98; 32]),
        restart_last_good_target_binding_hash: Some([0x99; 32]),
        load_artifact_by_hash_target_binding_hash: Some([0x9a; 32]),
        recovery_memory_write_authority_hash: Some([0x9b; 32]),
        durable_audit_rollback_write_authority_hash: Some([0x9c; 32]),
        service_inventory_side_effect_boundary_hash: Some([0x9d; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        command_dispatch_behavior_id: Some(RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID),
        command_dispatch_behavior_projection_hash: Some([0x9e; 32]),
    };
    let expected = module_evidence::computed_recovery_lifeline_command_dispatch_behavior_hash(
        module_evidence::RecoveryLifelineCommandDispatchBehaviorHashInput {
            retained_service_inventory_side_effect_boundary_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0x91; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0x92; 32],
            command_body_canonicalization_hash: [0x93; 32],
            handler_binding_hash: [0x94; 32],
            status_read_handler_hash: [0x95; 32],
            rollback_preview_authorization_hash: [0x96; 32],
            rollback_apply_authorization_hash: [0x97; 32],
            disable_module_target_binding_hash: [0x98; 32],
            restart_last_good_target_binding_hash: [0x99; 32],
            load_artifact_by_hash_target_binding_hash: [0x9a; 32],
            recovery_memory_write_authority_hash: [0x9b; 32],
            durable_audit_rollback_write_authority_hash: [0x9c; 32],
            service_inventory_side_effect_boundary_hash: [0x9d; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            command_dispatch_behavior_id: RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID,
            command_dispatch_behavior_projection_hash: [0x9e; 32],
        },
    );
    let mut valid = valid_input;
    valid.command_dispatch_behavior_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut behavior = valid;
    behavior.command_dispatch_behavior_id =
        Some("boundary.recovery_lifeline_command_dispatch_behavior.wrong");
    let mut hash = valid;
    hash.command_dispatch_behavior_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "recovery_lifeline_command_dispatch_behavior_absent",
            "missing",
            "recovery_lifeline_command_dispatch_behavior_absent",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(missing, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "recovery_lifeline_command_dispatch_behavior_arity_invalid",
            "invalid_reference",
            "recovery_lifeline_command_dispatch_behavior_arity_invalid",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(arity, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "previous_boot_recovery_lifeline_command_dispatch_behavior",
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_dispatch_behavior_scope_must_be_current_boot",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(previous, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(unsupported, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(schema, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(boundary, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "command_dispatch_behavior_id_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_behavior_id_mismatch",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(behavior, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "command_dispatch_behavior_hash_mismatch",
            "mismatched_recovery_lifeline_command_dispatch_behavior_hash",
            "recovery_lifeline_command_dispatch_behavior_hash_mismatch",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(hash, false),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "retained_recovery_service_inventory_side_effect_boundary_reference_missing",
            "rejected",
            "retained_recovery_service_inventory_side_effect_boundary_missing",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(live_missing, true),
        ),
        recovery_lifeline_command_dispatch_behavior_selftest_case(
            "all_inputs_present_recovery_lifeline_command_dispatch_behavior_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_lifeline_command_dispatch_behavior_valid_but_execution_disabled",
            evaluate_recovery_lifeline_command_dispatch_behavior_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_>,
) -> RecoveryLifelineCommandDispatchBehaviorSelfTestCase {
    RecoveryLifelineCommandDispatchBehaviorSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_selftest_cases(
) -> [RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase;
       RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_SELFTEST_CASES] {
    let valid_input = RecoveryLifelineCommandExecutorCapabilityTableInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        executor_capability_table_hash: None,
        retained_command_dispatch_behavior_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0xa1; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0xa2; 32]),
        command_body_canonicalization_hash: Some([0xa3; 32]),
        handler_binding_hash: Some([0xa4; 32]),
        status_read_handler_hash: Some([0xa5; 32]),
        rollback_preview_authorization_hash: Some([0xa6; 32]),
        rollback_apply_authorization_hash: Some([0xa7; 32]),
        disable_module_target_binding_hash: Some([0xa8; 32]),
        restart_last_good_target_binding_hash: Some([0xa9; 32]),
        load_artifact_by_hash_target_binding_hash: Some([0xaa; 32]),
        recovery_memory_write_authority_hash: Some([0xab; 32]),
        durable_audit_rollback_write_authority_hash: Some([0xac; 32]),
        service_inventory_side_effect_boundary_hash: Some([0xad; 32]),
        command_dispatch_behavior_hash: Some([0xae; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        executor_capability_table_id: Some(
            RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID,
        ),
        executor_capability_projection_hash: Some([0xaf; 32]),
    };
    let expected =
        module_evidence::computed_recovery_lifeline_command_executor_capability_table_hash(
            module_evidence::RecoveryLifelineCommandExecutorCapabilityTableHashInput {
                retained_command_dispatch_behavior_event_id: "event.current_boot.00000001",
                command_id: "recovery.lifeline.status",
                argument_schema: "raios.recovery_lifeline_command.status_args.v0",
                argument_hash: [0xa1; 32],
                target_locator: "recovery.lifeline.status.current_boot",
                command_envelope_reference_hash: [0xa2; 32],
                command_body_canonicalization_hash: [0xa3; 32],
                handler_binding_hash: [0xa4; 32],
                status_read_handler_hash: [0xa5; 32],
                rollback_preview_authorization_hash: [0xa6; 32],
                rollback_apply_authorization_hash: [0xa7; 32],
                disable_module_target_binding_hash: [0xa8; 32],
                restart_last_good_target_binding_hash: [0xa9; 32],
                load_artifact_by_hash_target_binding_hash: [0xaa; 32],
                recovery_memory_write_authority_hash: [0xab; 32],
                durable_audit_rollback_write_authority_hash: [0xac; 32],
                service_inventory_side_effect_boundary_hash: [0xad; 32],
                command_dispatch_behavior_hash: [0xae; 32],
                command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
                executor_capability_table_id:
                    RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID,
                executor_capability_projection_hash: [0xaf; 32],
            },
        );
    let mut valid = valid_input;
    valid.executor_capability_table_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut executor = valid;
    executor.executor_capability_table_id =
        Some("boundary.recovery_lifeline_command_executor_capability_table.wrong");
    let mut hash = valid;
    hash.executor_capability_table_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "recovery_lifeline_command_executor_capability_table_absent",
            "missing",
            "recovery_lifeline_command_executor_capability_table_absent",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(missing, false),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "recovery_lifeline_command_executor_capability_table_arity_invalid",
            "invalid_reference",
            "recovery_lifeline_command_executor_capability_table_arity_invalid",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(arity, false),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "previous_boot_recovery_lifeline_command_executor_capability_table",
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_executor_capability_table_scope_must_be_current_boot",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(
                previous,
                false,
            ),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(
                unsupported,
                false,
            ),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(schema, false),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(
                boundary,
                false,
            ),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "executor_capability_table_id_mismatch",
            "rejected",
            "recovery_lifeline_command_executor_capability_table_id_mismatch",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(
                executor,
                false,
            ),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "executor_capability_table_hash_mismatch",
            "mismatched_recovery_lifeline_command_executor_capability_table_hash",
            "recovery_lifeline_command_executor_capability_table_hash_mismatch",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(hash, false),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "retained_recovery_lifeline_command_dispatch_behavior_missing",
            "rejected",
            "retained_recovery_lifeline_command_dispatch_behavior_missing",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(
                live_missing,
                true,
            ),
        ),
        recovery_lifeline_command_executor_capability_table_selftest_case(
            "all_inputs_present_recovery_lifeline_command_executor_capability_table_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_lifeline_command_executor_capability_table_valid_but_execution_disabled",
            evaluate_recovery_lifeline_command_executor_capability_table_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_>,
) -> RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase {
    RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_selftest_cases(
) -> [RecoveryLifelineCommandSideEffectGateSelfTestCase;
       RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_SELFTEST_CASES] {
    let valid_input = RecoveryLifelineCommandSideEffectGateInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        side_effect_gate_hash: None,
        retained_executor_capability_table_event_id: Some("event.current_boot.00000001"),
        command_id: Some("recovery.lifeline.status"),
        argument_schema: Some("raios.recovery_lifeline_command.status_args.v0"),
        argument_hash: Some([0xb1; 32]),
        target_locator: Some("recovery.lifeline.status.current_boot"),
        command_envelope_reference_hash: Some([0xb2; 32]),
        command_body_canonicalization_hash: Some([0xb3; 32]),
        handler_binding_hash: Some([0xb4; 32]),
        status_read_handler_hash: Some([0xb5; 32]),
        rollback_preview_authorization_hash: Some([0xb6; 32]),
        rollback_apply_authorization_hash: Some([0xb7; 32]),
        disable_module_target_binding_hash: Some([0xb8; 32]),
        restart_last_good_target_binding_hash: Some([0xb9; 32]),
        load_artifact_by_hash_target_binding_hash: Some([0xba; 32]),
        recovery_memory_write_authority_hash: Some([0xbb; 32]),
        durable_audit_rollback_write_authority_hash: Some([0xbc; 32]),
        service_inventory_side_effect_boundary_hash: Some([0xbd; 32]),
        command_dispatch_behavior_hash: Some([0xbe; 32]),
        executor_capability_table_hash: Some([0xbf; 32]),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        side_effect_gate_id: Some(RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID),
        side_effect_projection_hash: Some([0xc0; 32]),
    };
    let expected = module_evidence::computed_recovery_lifeline_command_side_effect_gate_hash(
        module_evidence::RecoveryLifelineCommandSideEffectGateHashInput {
            retained_executor_capability_table_event_id: "event.current_boot.00000001",
            command_id: "recovery.lifeline.status",
            argument_schema: "raios.recovery_lifeline_command.status_args.v0",
            argument_hash: [0xb1; 32],
            target_locator: "recovery.lifeline.status.current_boot",
            command_envelope_reference_hash: [0xb2; 32],
            command_body_canonicalization_hash: [0xb3; 32],
            handler_binding_hash: [0xb4; 32],
            status_read_handler_hash: [0xb5; 32],
            rollback_preview_authorization_hash: [0xb6; 32],
            rollback_apply_authorization_hash: [0xb7; 32],
            disable_module_target_binding_hash: [0xb8; 32],
            restart_last_good_target_binding_hash: [0xb9; 32],
            load_artifact_by_hash_target_binding_hash: [0xba; 32],
            recovery_memory_write_authority_hash: [0xbb; 32],
            durable_audit_rollback_write_authority_hash: [0xbc; 32],
            service_inventory_side_effect_boundary_hash: [0xbd; 32],
            command_dispatch_behavior_hash: [0xbe; 32],
            executor_capability_table_hash: [0xbf; 32],
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            side_effect_gate_id: RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID,
            side_effect_projection_hash: [0xc0; 32],
        },
    );
    let mut valid = valid_input;
    valid.side_effect_gate_hash = Some(expected);
    let mut missing = valid;
    missing.has_reference = false;
    let mut arity = valid;
    arity.arity_valid = false;
    let mut previous = valid;
    previous.scope = "previous_boot";
    let mut unsupported = valid;
    unsupported.command_id = Some("recovery.lifeline.unsupported");
    let mut schema = valid;
    schema.argument_schema = Some("raios.recovery_lifeline_command.bad_args.v0");
    let mut boundary = valid;
    boundary.command_dispatch_boundary_id =
        Some("boundary.recovery_lifeline_command_dispatch.wrong");
    let mut side_effect = valid;
    side_effect.side_effect_gate_id =
        Some("boundary.recovery_lifeline_command_side_effect_gate.wrong");
    let mut hash = valid;
    hash.side_effect_gate_hash = Some([0xff; 32]);
    let live_missing = valid;

    [
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "recovery_lifeline_command_side_effect_gate_absent",
            "missing",
            "recovery_lifeline_command_side_effect_gate_absent",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(missing, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "recovery_lifeline_command_side_effect_gate_arity_invalid",
            "invalid_reference",
            "recovery_lifeline_command_side_effect_gate_arity_invalid",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(arity, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "previous_boot_recovery_lifeline_command_side_effect_gate",
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_side_effect_gate_scope_must_be_current_boot",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(previous, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "unsupported_command_id",
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(unsupported, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "argument_schema_mismatch",
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(schema, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "dispatch_boundary_mismatch",
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(boundary, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "side_effect_gate_id_mismatch",
            "rejected",
            "recovery_lifeline_command_side_effect_gate_id_mismatch",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(side_effect, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "side_effect_gate_hash_mismatch",
            "mismatched_recovery_lifeline_command_side_effect_gate_hash",
            "recovery_lifeline_command_side_effect_gate_hash_mismatch",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(hash, false),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "retained_recovery_lifeline_command_executor_capability_table_missing",
            "rejected",
            "retained_recovery_lifeline_command_executor_capability_table_missing",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(live_missing, true),
        ),
        recovery_lifeline_command_side_effect_gate_selftest_case(
            "all_inputs_present_recovery_lifeline_command_side_effect_gate_still_non_executable",
            "valid_hash_reference_command_still_denied",
            "recovery_lifeline_command_side_effect_gate_valid_but_execution_disabled",
            evaluate_recovery_lifeline_command_side_effect_gate_reference(valid, false),
        ),
    ]
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandSideEffectGateReferenceCheck<'_>,
) -> RecoveryLifelineCommandSideEffectGateSelfTestCase {
    RecoveryLifelineCommandSideEffectGateSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}
