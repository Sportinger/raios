use crate::{
    agent_protocol_recovery_command_authorization_types::*,
    agent_protocol_recovery_command_effect_reference_eval::*,
    agent_protocol_recovery_command_effect_types::*,
    agent_protocol_recovery_command_reference_eval::*,
    agent_protocol_recovery_constants::*,
    agent_protocol_recovery_lifeline::RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    agent_protocol_recovery_runtime_types::CommandBindings,
    agent_protocol_support::{run_selftest_cases, CaseSpec},
};

const EVENT_ID: &str = "event.current_boot.00000001";
const CURRENT_BOOT: &str = "current_boot";
const PREVIOUS_BOOT: &str = "previous_boot";
const STATUS_COMMAND_ID: &str = "recovery.lifeline.status";
const UNSUPPORTED_COMMAND_ID: &str = "recovery.lifeline.unsupported";
const STATUS_ARGUMENT_SCHEMA: &str = "raios.recovery_lifeline_command.status_args.v0";
const BAD_ARGUMENT_SCHEMA: &str = "raios.recovery_lifeline_command.bad_args.v0";
const STATUS_TARGET_LOCATOR: &str = "recovery.lifeline.status.current_boot";
const WRONG_DISPATCH_BOUNDARY_ID: &str = "boundary.recovery_lifeline_command_dispatch.wrong";
const BAD_HASH: [u8; 32] = [0xff; 32];

#[derive(Clone, Copy)]
enum CommandReferenceMutation {
    None,
    Missing,
    ArityInvalid,
    PreviousBoot,
    UnsupportedCommand,
    ArgumentSchemaMismatch,
    DispatchBoundaryMismatch,
    BoundaryIdMismatch,
    HashMismatch,
}

const fn case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: CommandReferenceMutation,
    require_live_retained: bool,
) -> CaseSpec<CommandReferenceMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained,
    }
}

fn h(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn base_input(seed: u8) -> CommandBindings<'static> {
    CommandBindings {
        has_reference: true,
        arity_valid: true,
        scope: CURRENT_BOOT,
        command_id: Some(STATUS_COMMAND_ID),
        argument_schema: Some(STATUS_ARGUMENT_SCHEMA),
        argument_hash: Some(h(seed)),
        target_locator: Some(STATUS_TARGET_LOCATOR),
        command_envelope_reference_hash: Some(h(seed + 1)),
        command_body_canonicalization_hash: Some(h(seed + 2)),
        command_dispatch_boundary_id: Some(RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID),
        ..CommandBindings::empty()
    }
}

fn status(check: &CommandBindings<'_>) -> &'static str {
    check.status
}

fn reason(check: &CommandBindings<'_>) -> &'static str {
    check.reason
}

fn with_expected_hash(
    mut input: CommandBindings<'static>,
    eval: fn(CommandBindings<'static>, bool) -> CommandBindings<'static>,
    set_hash: fn(&mut CommandBindings<'static>, [u8; 32]),
) -> CommandBindings<'static> {
    let check = eval(input, false);
    if let Some(expected) = check.stage.expected_hash {
        set_hash(&mut input, expected);
    }
    input
}

fn apply_common(input: &mut CommandBindings<'static>, mutation: CommandReferenceMutation) -> bool {
    match mutation {
        CommandReferenceMutation::Missing => input.has_reference = false,
        CommandReferenceMutation::ArityInvalid => input.arity_valid = false,
        CommandReferenceMutation::PreviousBoot => input.scope = PREVIOUS_BOOT,
        CommandReferenceMutation::UnsupportedCommand => {
            input.command_id = Some(UNSUPPORTED_COMMAND_ID)
        }
        CommandReferenceMutation::ArgumentSchemaMismatch => {
            input.argument_schema = Some(BAD_ARGUMENT_SCHEMA)
        }
        CommandReferenceMutation::DispatchBoundaryMismatch => {
            input.command_dispatch_boundary_id = Some(WRONG_DISPATCH_BOUNDARY_ID)
        }
        CommandReferenceMutation::None
        | CommandReferenceMutation::BoundaryIdMismatch
        | CommandReferenceMutation::HashMismatch => return false,
    }
    true
}

macro_rules! selftest_cases {
    ($fn_name:ident, $case_ty:ty, $count:ident, $base:expr, $cases:ident, $apply:ident, $eval:path) => {
        pub(crate) fn $fn_name() -> [$case_ty; $count] {
            run_selftest_cases($base, &$cases, $apply, $eval, status, reason)
        }
    };
}

selftest_cases!(
    recovery_lifeline_command_handler_binding_selftest_cases,
    RecoveryLifelineCommandHandlerBindingSelfTestCase,
    RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_SELFTEST_CASES,
    valid_handler_binding_input(),
    HANDLER_BINDING_CASES,
    apply_handler_binding_case,
    evaluate_recovery_lifeline_command_handler_binding_reference
);

fn valid_handler_binding_input() -> CommandBindings<'static> {
    let mut input = base_input(0x31);
    input.retained_command_body_canonicalization_event_id = Some(EVENT_ID);
    input.handler_id = Some(RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID);
    input.handler_input_binding_hash = Some(h(0x34));
    with_expected_hash(
        input,
        evaluate_recovery_lifeline_command_handler_binding_reference,
        set_handler_binding_hash,
    )
}

fn set_handler_binding_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.handler_binding_hash = Some(hash);
}

fn apply_handler_binding_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.handler_id = Some("boundary.recovery_lifeline_command_handler_binding.wrong")
        }
        CommandReferenceMutation::HashMismatch => input.handler_binding_hash = Some(BAD_HASH),
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const HANDLER_BINDING_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_SELFTEST_CASES] = [
    case(
        "handler_binding_absent",
        "missing",
        "recovery_lifeline_command_handler_binding_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "handler_binding_arity_invalid",
        "invalid_reference",
        "recovery_lifeline_command_handler_binding_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_handler_binding",
        "stale_or_non_current_boot_reference",
        "recovery_lifeline_command_handler_binding_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "handler_id_mismatch",
        "rejected",
        "recovery_lifeline_command_handler_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "handler_binding_hash_mismatch",
        "mismatched_command_handler_binding_hash",
        "recovery_lifeline_command_handler_binding_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_body_reference_missing",
        "rejected",
        "retained_recovery_lifeline_command_body_canonicalization_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_handler_binding_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_handler_binding_valid_but_command_dispatch_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_lifeline_status_read_handler_selftest_cases,
    RecoveryLifelineStatusReadHandlerSelfTestCase,
    RECOVERY_LIFELINE_STATUS_READ_HANDLER_SELFTEST_CASES,
    valid_status_read_handler_input(),
    STATUS_READ_HANDLER_CASES,
    apply_status_read_handler_case,
    evaluate_recovery_lifeline_status_read_handler_reference
);

fn valid_status_read_handler_input() -> CommandBindings<'static> {
    let mut input = base_input(0x41);
    input.retained_command_handler_binding_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x44));
    input.status_handler_id = Some(RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID);
    input.status_read_projection_hash = Some(h(0x45));
    with_expected_hash(
        input,
        evaluate_recovery_lifeline_status_read_handler_reference,
        set_status_read_handler_hash,
    )
}

fn set_status_read_handler_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.status_read_handler_hash = Some(hash);
}

fn apply_status_read_handler_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.status_handler_id = Some("boundary.recovery_lifeline_status_read_handler.wrong")
        }
        CommandReferenceMutation::HashMismatch => input.status_read_handler_hash = Some(BAD_HASH),
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const STATUS_READ_HANDLER_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_LIFELINE_STATUS_READ_HANDLER_SELFTEST_CASES] = [
    case(
        "status_read_handler_absent",
        "missing",
        "recovery_lifeline_status_read_handler_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "status_read_handler_arity_invalid",
        "invalid_reference",
        "recovery_lifeline_status_read_handler_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_status_read_handler",
        "stale_or_non_current_boot_reference",
        "recovery_lifeline_status_read_handler_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "status_handler_id_mismatch",
        "rejected",
        "recovery_lifeline_status_read_handler_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "status_read_handler_hash_mismatch",
        "mismatched_status_read_handler_hash",
        "recovery_lifeline_status_read_handler_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_handler_binding_reference_missing",
        "rejected",
        "retained_recovery_lifeline_command_handler_binding_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_status_read_handler_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_status_read_handler_valid_but_command_dispatch_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_rollback_preview_authorization_selftest_cases,
    RecoveryRollbackPreviewAuthorizationSelfTestCase,
    RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_SELFTEST_CASES,
    valid_rollback_preview_authorization_input(),
    ROLLBACK_PREVIEW_AUTHORIZATION_CASES,
    apply_rollback_preview_authorization_case,
    evaluate_recovery_rollback_preview_authorization_reference
);

fn valid_rollback_preview_authorization_input() -> CommandBindings<'static> {
    let mut input = base_input(0x51);
    input.retained_status_read_handler_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x54));
    input.status_read_handler_hash = Some(h(0x55));
    input.rollback_preview_authorization_id =
        Some(RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID);
    input.rollback_preview_projection_hash = Some(h(0x56));
    with_expected_hash(
        input,
        evaluate_recovery_rollback_preview_authorization_reference,
        set_rollback_preview_authorization_hash,
    )
}

fn set_rollback_preview_authorization_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.rollback_preview_authorization_hash = Some(hash);
}

fn apply_rollback_preview_authorization_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.rollback_preview_authorization_id =
                Some("boundary.recovery_rollback_preview_authorization.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.rollback_preview_authorization_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const ROLLBACK_PREVIEW_AUTHORIZATION_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_SELFTEST_CASES] = [
    case(
        "rollback_preview_authorization_absent",
        "missing",
        "recovery_rollback_preview_authorization_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "rollback_preview_authorization_arity_invalid",
        "invalid_reference",
        "recovery_rollback_preview_authorization_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_rollback_preview_authorization",
        "stale_or_non_current_boot_reference",
        "recovery_rollback_preview_authorization_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "rollback_preview_authorization_id_mismatch",
        "rejected",
        "recovery_rollback_preview_authorization_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "rollback_preview_authorization_hash_mismatch",
        "mismatched_rollback_preview_authorization_hash",
        "recovery_rollback_preview_authorization_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_status_read_handler_reference_missing",
        "rejected",
        "retained_recovery_lifeline_status_read_handler_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_rollback_preview_authorization_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_rollback_preview_authorization_valid_but_command_dispatch_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_rollback_apply_authorization_selftest_cases,
    RecoveryRollbackApplyAuthorizationSelfTestCase,
    RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_SELFTEST_CASES,
    valid_rollback_apply_authorization_input(),
    ROLLBACK_APPLY_AUTHORIZATION_CASES,
    apply_rollback_apply_authorization_case,
    evaluate_recovery_rollback_apply_authorization_reference
);

fn valid_rollback_apply_authorization_input() -> CommandBindings<'static> {
    let mut input = base_input(0x61);
    input.retained_rollback_preview_authorization_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x64));
    input.status_read_handler_hash = Some(h(0x65));
    input.rollback_preview_authorization_hash = Some(h(0x66));
    input.rollback_apply_authorization_id = Some(RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID);
    input.rollback_apply_projection_hash = Some(h(0x67));
    input.source_rollback_apply_denial_hash = Some(h(0x68));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x69));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0x6a));
    with_expected_hash(
        input,
        evaluate_recovery_rollback_apply_authorization_reference,
        set_rollback_apply_authorization_hash,
    )
}

fn set_rollback_apply_authorization_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.rollback_apply_authorization_hash = Some(hash);
}

fn apply_rollback_apply_authorization_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.rollback_apply_authorization_id =
                Some("boundary.recovery_rollback_apply_authorization.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.rollback_apply_authorization_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const ROLLBACK_APPLY_AUTHORIZATION_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_SELFTEST_CASES] = [
    case(
        "rollback_apply_authorization_absent",
        "missing",
        "recovery_rollback_apply_authorization_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "rollback_apply_authorization_arity_invalid",
        "invalid_reference",
        "recovery_rollback_apply_authorization_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_rollback_apply_authorization",
        "stale_or_non_current_boot_reference",
        "recovery_rollback_apply_authorization_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "rollback_apply_authorization_id_mismatch",
        "rejected",
        "recovery_rollback_apply_authorization_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "rollback_apply_authorization_hash_mismatch",
        "mismatched_rollback_apply_authorization_hash",
        "recovery_rollback_apply_authorization_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_rollback_preview_authorization_reference_missing",
        "rejected",
        "retained_recovery_rollback_preview_authorization_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_rollback_apply_authorization_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_rollback_apply_authorization_valid_but_command_dispatch_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_disable_module_target_binding_selftest_cases,
    RecoveryDisableModuleTargetBindingSelfTestCase,
    RECOVERY_DISABLE_MODULE_TARGET_BINDING_SELFTEST_CASES,
    valid_disable_module_target_binding_input(),
    DISABLE_MODULE_TARGET_BINDING_CASES,
    apply_disable_module_target_binding_case,
    evaluate_recovery_disable_module_target_binding_reference
);

fn valid_disable_module_target_binding_input() -> CommandBindings<'static> {
    let mut input = base_input(0x61);
    input.retained_rollback_apply_authorization_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x64));
    input.status_read_handler_hash = Some(h(0x65));
    input.rollback_preview_authorization_hash = Some(h(0x66));
    input.rollback_apply_authorization_hash = Some(h(0x67));
    input.source_rollback_apply_denial_hash = Some(h(0x68));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x69));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0x6a));
    input.disable_module_target_id = Some(RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID);
    input.disable_module_target_projection_hash = Some(h(0x6b));
    with_expected_hash(
        input,
        evaluate_recovery_disable_module_target_binding_reference,
        set_disable_module_target_binding_hash,
    )
}

fn set_disable_module_target_binding_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.disable_module_target_binding_hash = Some(hash);
}

fn apply_disable_module_target_binding_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.disable_module_target_id =
                Some("boundary.recovery_disable_module_target_binding.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.disable_module_target_binding_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const DISABLE_MODULE_TARGET_BINDING_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_DISABLE_MODULE_TARGET_BINDING_SELFTEST_CASES] = [
    case(
        "disable_module_target_binding_absent",
        "missing",
        "recovery_disable_module_target_binding_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "disable_module_target_binding_arity_invalid",
        "invalid_reference",
        "recovery_disable_module_target_binding_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_disable_module_target_binding",
        "stale_or_non_current_boot_reference",
        "recovery_disable_module_target_binding_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "disable_module_target_id_mismatch",
        "rejected",
        "recovery_disable_module_target_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "disable_module_target_binding_hash_mismatch",
        "mismatched_disable_module_target_binding_hash",
        "recovery_disable_module_target_binding_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_rollback_apply_authorization_reference_missing",
        "rejected",
        "retained_recovery_rollback_apply_authorization_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_disable_module_target_binding_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_disable_module_target_binding_valid_but_command_dispatch_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_restart_last_good_target_binding_selftest_cases,
    RecoveryRestartLastGoodTargetBindingSelfTestCase,
    RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_SELFTEST_CASES,
    valid_restart_last_good_target_binding_input(),
    RESTART_LAST_GOOD_TARGET_BINDING_CASES,
    apply_restart_last_good_target_binding_case,
    evaluate_recovery_restart_last_good_target_binding_reference
);

fn valid_restart_last_good_target_binding_input() -> CommandBindings<'static> {
    let mut input = base_input(0x61);
    input.retained_disable_module_target_binding_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x64));
    input.status_read_handler_hash = Some(h(0x65));
    input.rollback_preview_authorization_hash = Some(h(0x66));
    input.rollback_apply_authorization_hash = Some(h(0x67));
    input.disable_module_target_binding_hash = Some(h(0x68));
    input.source_rollback_apply_denial_hash = Some(h(0x69));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x6a));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0x6b));
    input.restart_last_good_target_id = Some(RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID);
    input.restart_last_good_target_projection_hash = Some(h(0x6c));
    with_expected_hash(
        input,
        evaluate_recovery_restart_last_good_target_binding_reference,
        set_restart_last_good_target_binding_hash,
    )
}

fn set_restart_last_good_target_binding_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.restart_last_good_target_binding_hash = Some(hash);
}

fn apply_restart_last_good_target_binding_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.restart_last_good_target_id =
                Some("boundary.recovery_restart_last_good_target_binding.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.restart_last_good_target_binding_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const RESTART_LAST_GOOD_TARGET_BINDING_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_SELFTEST_CASES] = [
    case(
        "restart_last_good_target_binding_absent",
        "missing",
        "recovery_restart_last_good_target_binding_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "restart_last_good_target_binding_arity_invalid",
        "invalid_reference",
        "recovery_restart_last_good_target_binding_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_restart_last_good_target_binding",
        "stale_or_non_current_boot_reference",
        "recovery_restart_last_good_target_binding_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "restart_last_good_target_id_mismatch",
        "rejected",
        "recovery_restart_last_good_target_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "restart_last_good_target_binding_hash_mismatch",
        "mismatched_restart_last_good_target_binding_hash",
        "recovery_restart_last_good_target_binding_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_disable_module_target_binding_reference_missing",
        "rejected",
        "retained_recovery_disable_module_target_binding_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_restart_last_good_target_binding_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_restart_last_good_target_binding_valid_but_command_dispatch_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_load_artifact_by_hash_target_binding_selftest_cases,
    RecoveryLoadArtifactByHashTargetBindingSelfTestCase,
    RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_SELFTEST_CASES,
    valid_load_artifact_by_hash_target_binding_input(),
    LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_CASES,
    apply_load_artifact_by_hash_target_binding_case,
    evaluate_recovery_load_artifact_by_hash_target_binding_reference
);

fn valid_load_artifact_by_hash_target_binding_input() -> CommandBindings<'static> {
    let mut input = base_input(0x61);
    input.retained_restart_last_good_target_binding_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x64));
    input.status_read_handler_hash = Some(h(0x65));
    input.rollback_preview_authorization_hash = Some(h(0x66));
    input.rollback_apply_authorization_hash = Some(h(0x67));
    input.disable_module_target_binding_hash = Some(h(0x68));
    input.restart_last_good_target_binding_hash = Some(h(0x69));
    input.source_rollback_apply_denial_hash = Some(h(0x6a));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x6b));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0x6c));
    input.load_artifact_by_hash_target_id =
        Some(RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID);
    input.load_artifact_by_hash_target_artifact_hash = Some(h(0x6d));
    input.load_artifact_by_hash_target_projection_hash = Some(h(0x6e));
    with_expected_hash(
        input,
        evaluate_recovery_load_artifact_by_hash_target_binding_reference,
        set_load_artifact_by_hash_target_binding_hash,
    )
}

fn set_load_artifact_by_hash_target_binding_hash(
    input: &mut CommandBindings<'static>,
    hash: [u8; 32],
) {
    input.load_artifact_by_hash_target_binding_hash = Some(hash);
}

fn apply_load_artifact_by_hash_target_binding_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.load_artifact_by_hash_target_id =
                Some("boundary.recovery_load_artifact_by_hash_target_binding.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.load_artifact_by_hash_target_binding_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_SELFTEST_CASES] = [
    case(
        "load_artifact_by_hash_target_binding_absent",
        "missing",
        "recovery_load_artifact_by_hash_target_binding_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "load_artifact_by_hash_target_binding_arity_invalid",
        "invalid_reference",
        "recovery_load_artifact_by_hash_target_binding_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_load_artifact_by_hash_target_binding",
        "stale_or_non_current_boot_reference",
        "recovery_load_artifact_by_hash_target_binding_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "load_artifact_by_hash_target_id_mismatch",
        "rejected",
        "recovery_load_artifact_by_hash_target_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "load_artifact_by_hash_target_binding_hash_mismatch",
        "mismatched_load_artifact_by_hash_target_binding_hash",
        "recovery_load_artifact_by_hash_target_binding_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_restart_last_good_target_binding_reference_missing",
        "rejected",
        "retained_recovery_restart_last_good_target_binding_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_load_artifact_by_hash_target_binding_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_load_artifact_by_hash_target_binding_valid_but_command_dispatch_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_memory_write_authority_selftest_cases,
    RecoveryMemoryWriteAuthoritySelfTestCase,
    RECOVERY_MEMORY_WRITE_AUTHORITY_SELFTEST_CASES,
    valid_recovery_memory_write_authority_input(),
    RECOVERY_MEMORY_WRITE_AUTHORITY_CASES,
    apply_recovery_memory_write_authority_case,
    evaluate_recovery_memory_write_authority_reference
);

fn valid_recovery_memory_write_authority_input() -> CommandBindings<'static> {
    let mut input = base_input(0x61);
    input.retained_load_artifact_by_hash_target_binding_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x64));
    input.status_read_handler_hash = Some(h(0x65));
    input.rollback_preview_authorization_hash = Some(h(0x66));
    input.rollback_apply_authorization_hash = Some(h(0x67));
    input.disable_module_target_binding_hash = Some(h(0x68));
    input.restart_last_good_target_binding_hash = Some(h(0x69));
    input.load_artifact_by_hash_target_binding_hash = Some(h(0x6a));
    input.source_rollback_apply_denial_hash = Some(h(0x6b));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x6c));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0x6d));
    input.recovery_memory_write_authority_id = Some(RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID);
    input.recovery_memory_projection_hash = Some(h(0x6e));
    with_expected_hash(
        input,
        evaluate_recovery_memory_write_authority_reference,
        set_recovery_memory_write_authority_hash,
    )
}

fn set_recovery_memory_write_authority_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.recovery_memory_write_authority_hash = Some(hash);
}

fn apply_recovery_memory_write_authority_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.recovery_memory_write_authority_id =
                Some("boundary.recovery_memory_write_authority.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.recovery_memory_write_authority_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const RECOVERY_MEMORY_WRITE_AUTHORITY_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_MEMORY_WRITE_AUTHORITY_SELFTEST_CASES] = [
    case(
        "recovery_memory_write_authority_absent",
        "missing",
        "recovery_memory_write_authority_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "recovery_memory_write_authority_arity_invalid",
        "invalid_reference",
        "recovery_memory_write_authority_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_recovery_memory_write_authority",
        "stale_or_non_current_boot_reference",
        "recovery_memory_write_authority_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "recovery_memory_write_authority_id_mismatch",
        "rejected",
        "recovery_memory_write_authority_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "recovery_memory_write_authority_hash_mismatch",
        "mismatched_recovery_memory_write_authority_hash",
        "recovery_memory_write_authority_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_load_artifact_by_hash_target_binding_reference_missing",
        "rejected",
        "retained_recovery_load_artifact_by_hash_target_binding_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_recovery_memory_write_authority_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_memory_write_authority_valid_but_memory_writes_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    durable_audit_rollback_write_authority_selftest_cases,
    DurableAuditRollbackWriteAuthoritySelfTestCase,
    DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_SELFTEST_CASES,
    valid_durable_audit_rollback_write_authority_input(),
    DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_CASES,
    apply_durable_audit_rollback_write_authority_case,
    evaluate_durable_audit_rollback_write_authority_reference
);

fn valid_durable_audit_rollback_write_authority_input() -> CommandBindings<'static> {
    let mut input = base_input(0x71);
    input.retained_recovery_memory_write_authority_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x74));
    input.status_read_handler_hash = Some(h(0x75));
    input.rollback_preview_authorization_hash = Some(h(0x76));
    input.rollback_apply_authorization_hash = Some(h(0x77));
    input.disable_module_target_binding_hash = Some(h(0x78));
    input.restart_last_good_target_binding_hash = Some(h(0x79));
    input.load_artifact_by_hash_target_binding_hash = Some(h(0x7a));
    input.recovery_memory_write_authority_hash = Some(h(0x7b));
    input.source_rollback_apply_denial_hash = Some(h(0x7c));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x7d));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0x7e));
    input.durable_audit_rollback_write_authority_id =
        Some(DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID);
    input.durable_audit_rollback_projection_hash = Some(h(0x7f));
    with_expected_hash(
        input,
        evaluate_durable_audit_rollback_write_authority_reference,
        set_durable_audit_rollback_write_authority_hash,
    )
}

fn set_durable_audit_rollback_write_authority_hash(
    input: &mut CommandBindings<'static>,
    hash: [u8; 32],
) {
    input.durable_audit_rollback_write_authority_hash = Some(hash);
}

fn apply_durable_audit_rollback_write_authority_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.durable_audit_rollback_write_authority_id =
                Some("boundary.durable_audit_rollback_write_authority.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.durable_audit_rollback_write_authority_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_CASES: [CaseSpec<CommandReferenceMutation>;
    DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_SELFTEST_CASES] = [
    case(
        "durable_audit_rollback_write_authority_absent",
        "missing",
        "durable_audit_rollback_write_authority_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "durable_audit_rollback_write_authority_arity_invalid",
        "invalid_reference",
        "durable_audit_rollback_write_authority_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_durable_audit_rollback_write_authority",
        "stale_or_non_current_boot_reference",
        "durable_audit_rollback_write_authority_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "durable_audit_rollback_write_authority_id_mismatch",
        "rejected",
        "durable_audit_rollback_write_authority_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "durable_audit_rollback_write_authority_hash_mismatch",
        "mismatched_durable_audit_rollback_write_authority_hash",
        "durable_audit_rollback_write_authority_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_recovery_memory_write_authority_reference_missing",
        "rejected",
        "retained_recovery_memory_write_authority_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_durable_audit_rollback_write_authority_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "durable_audit_rollback_write_authority_valid_but_durable_writes_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_service_inventory_side_effect_boundary_selftest_cases,
    RecoveryServiceInventorySideEffectBoundarySelfTestCase,
    RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_SELFTEST_CASES,
    valid_service_inventory_side_effect_boundary_input(),
    SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_CASES,
    apply_service_inventory_side_effect_boundary_case,
    evaluate_recovery_service_inventory_side_effect_boundary_reference
);

fn valid_service_inventory_side_effect_boundary_input() -> CommandBindings<'static> {
    let mut input = base_input(0x81);
    input.retained_durable_audit_rollback_write_authority_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x84));
    input.status_read_handler_hash = Some(h(0x85));
    input.rollback_preview_authorization_hash = Some(h(0x86));
    input.rollback_apply_authorization_hash = Some(h(0x87));
    input.disable_module_target_binding_hash = Some(h(0x88));
    input.restart_last_good_target_binding_hash = Some(h(0x89));
    input.load_artifact_by_hash_target_binding_hash = Some(h(0x8a));
    input.recovery_memory_write_authority_hash = Some(h(0x8b));
    input.durable_audit_rollback_write_authority_hash = Some(h(0x8c));
    input.source_rollback_apply_denial_hash = Some(h(0x8d));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x8e));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0x8f));
    input.service_inventory_side_effect_boundary_id =
        Some(RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID);
    input.service_inventory_projection_hash = Some(h(0x90));
    with_expected_hash(
        input,
        evaluate_recovery_service_inventory_side_effect_boundary_reference,
        set_service_inventory_side_effect_boundary_hash,
    )
}

fn set_service_inventory_side_effect_boundary_hash(
    input: &mut CommandBindings<'static>,
    hash: [u8; 32],
) {
    input.service_inventory_side_effect_boundary_hash = Some(hash);
}

fn apply_service_inventory_side_effect_boundary_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.service_inventory_side_effect_boundary_id =
                Some("boundary.recovery_service_inventory_side_effect_boundary.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.service_inventory_side_effect_boundary_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_SELFTEST_CASES] = [
    case(
        "recovery_service_inventory_side_effect_boundary_absent",
        "missing",
        "recovery_service_inventory_side_effect_boundary_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "recovery_service_inventory_side_effect_boundary_arity_invalid",
        "invalid_reference",
        "recovery_service_inventory_side_effect_boundary_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_recovery_service_inventory_side_effect_boundary",
        "stale_or_non_current_boot_reference",
        "recovery_service_inventory_side_effect_boundary_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "service_inventory_side_effect_boundary_id_mismatch",
        "rejected",
        "recovery_service_inventory_side_effect_boundary_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "service_inventory_side_effect_boundary_hash_mismatch",
        "mismatched_recovery_service_inventory_side_effect_boundary_hash",
        "recovery_service_inventory_side_effect_boundary_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_durable_audit_rollback_write_authority_reference_missing",
        "rejected",
        "retained_durable_audit_rollback_write_authority_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_service_inventory_side_effect_boundary_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_service_inventory_side_effect_boundary_valid_but_service_inventory_unchanged",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_lifeline_command_dispatch_behavior_selftest_cases,
    RecoveryLifelineCommandDispatchBehaviorSelfTestCase,
    RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_SELFTEST_CASES,
    valid_command_dispatch_behavior_input(),
    COMMAND_DISPATCH_BEHAVIOR_CASES,
    apply_command_dispatch_behavior_case,
    evaluate_recovery_lifeline_command_dispatch_behavior_reference
);

fn valid_command_dispatch_behavior_input() -> CommandBindings<'static> {
    let mut input = base_input(0x91);
    input.retained_service_inventory_side_effect_boundary_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0x94));
    input.status_read_handler_hash = Some(h(0x95));
    input.rollback_preview_authorization_hash = Some(h(0x96));
    input.rollback_apply_authorization_hash = Some(h(0x97));
    input.disable_module_target_binding_hash = Some(h(0x98));
    input.restart_last_good_target_binding_hash = Some(h(0x99));
    input.load_artifact_by_hash_target_binding_hash = Some(h(0x9a));
    input.recovery_memory_write_authority_hash = Some(h(0x9b));
    input.durable_audit_rollback_write_authority_hash = Some(h(0x9c));
    input.service_inventory_side_effect_boundary_hash = Some(h(0x9d));
    input.source_rollback_apply_denial_hash = Some(h(0x9e));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0x9f));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0xa0));
    input.command_dispatch_behavior_id =
        Some(RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID);
    input.command_dispatch_behavior_projection_hash = Some(h(0xa1));
    with_expected_hash(
        input,
        evaluate_recovery_lifeline_command_dispatch_behavior_reference,
        set_command_dispatch_behavior_hash,
    )
}

fn set_command_dispatch_behavior_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.command_dispatch_behavior_hash = Some(hash);
}

fn apply_command_dispatch_behavior_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.command_dispatch_behavior_id =
                Some("boundary.recovery_lifeline_command_dispatch_behavior.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.command_dispatch_behavior_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const COMMAND_DISPATCH_BEHAVIOR_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_SELFTEST_CASES] = [
    case(
        "recovery_lifeline_command_dispatch_behavior_absent",
        "missing",
        "recovery_lifeline_command_dispatch_behavior_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "recovery_lifeline_command_dispatch_behavior_arity_invalid",
        "invalid_reference",
        "recovery_lifeline_command_dispatch_behavior_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_recovery_lifeline_command_dispatch_behavior",
        "stale_or_non_current_boot_reference",
        "recovery_lifeline_command_dispatch_behavior_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "command_dispatch_behavior_id_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_behavior_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "command_dispatch_behavior_hash_mismatch",
        "mismatched_recovery_lifeline_command_dispatch_behavior_hash",
        "recovery_lifeline_command_dispatch_behavior_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_recovery_service_inventory_side_effect_boundary_reference_missing",
        "rejected",
        "retained_recovery_service_inventory_side_effect_boundary_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_recovery_lifeline_command_dispatch_behavior_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_dispatch_behavior_valid_but_execution_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];

selftest_cases!(
    recovery_lifeline_command_executor_capability_table_selftest_cases,
    RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase,
    RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_SELFTEST_CASES,
    valid_executor_capability_table_input(),
    EXECUTOR_CAPABILITY_TABLE_CASES,
    apply_executor_capability_table_case,
    evaluate_recovery_lifeline_command_executor_capability_table_reference
);

fn valid_executor_capability_table_input() -> CommandBindings<'static> {
    let mut input = base_input(0xa1);
    input.retained_command_dispatch_behavior_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0xa4));
    input.status_read_handler_hash = Some(h(0xa5));
    input.rollback_preview_authorization_hash = Some(h(0xa6));
    input.rollback_apply_authorization_hash = Some(h(0xa7));
    input.disable_module_target_binding_hash = Some(h(0xa8));
    input.restart_last_good_target_binding_hash = Some(h(0xa9));
    input.load_artifact_by_hash_target_binding_hash = Some(h(0xaa));
    input.recovery_memory_write_authority_hash = Some(h(0xab));
    input.durable_audit_rollback_write_authority_hash = Some(h(0xac));
    input.service_inventory_side_effect_boundary_hash = Some(h(0xad));
    input.command_dispatch_behavior_hash = Some(h(0xae));
    input.source_rollback_apply_denial_hash = Some(h(0xaf));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0xb0));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0xb1));
    input.executor_capability_table_id =
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID);
    input.executor_capability_projection_hash = Some(h(0xb2));
    with_expected_hash(
        input,
        evaluate_recovery_lifeline_command_executor_capability_table_reference,
        set_executor_capability_table_hash,
    )
}

fn set_executor_capability_table_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.executor_capability_table_hash = Some(hash);
}

fn apply_executor_capability_table_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.executor_capability_table_id =
                Some("boundary.recovery_lifeline_command_executor_capability_table.wrong")
        }
        CommandReferenceMutation::HashMismatch => {
            input.executor_capability_table_hash = Some(BAD_HASH)
        }
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const EXECUTOR_CAPABILITY_TABLE_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_SELFTEST_CASES] = [
    case("recovery_lifeline_command_executor_capability_table_absent", "missing", "recovery_lifeline_command_executor_capability_table_absent", CommandReferenceMutation::Missing, false),
    case("recovery_lifeline_command_executor_capability_table_arity_invalid", "invalid_reference", "recovery_lifeline_command_executor_capability_table_arity_invalid", CommandReferenceMutation::ArityInvalid, false),
    case("previous_boot_recovery_lifeline_command_executor_capability_table", "stale_or_non_current_boot_reference", "recovery_lifeline_command_executor_capability_table_scope_must_be_current_boot", CommandReferenceMutation::PreviousBoot, false),
    case("unsupported_command_id", "rejected", "recovery_lifeline_command_id_unsupported", CommandReferenceMutation::UnsupportedCommand, false),
    case("argument_schema_mismatch", "rejected", "recovery_lifeline_command_argument_schema_mismatch", CommandReferenceMutation::ArgumentSchemaMismatch, false),
    case("dispatch_boundary_mismatch", "rejected", "recovery_lifeline_command_dispatch_boundary_mismatch", CommandReferenceMutation::DispatchBoundaryMismatch, false),
    case("executor_capability_table_id_mismatch", "rejected", "recovery_lifeline_command_executor_capability_table_id_mismatch", CommandReferenceMutation::BoundaryIdMismatch, false),
    case("executor_capability_table_hash_mismatch", "mismatched_recovery_lifeline_command_executor_capability_table_hash", "recovery_lifeline_command_executor_capability_table_hash_mismatch", CommandReferenceMutation::HashMismatch, false),
    case("retained_recovery_lifeline_command_dispatch_behavior_missing", "rejected", "retained_recovery_lifeline_command_dispatch_behavior_missing", CommandReferenceMutation::None, true),
    case("all_inputs_present_recovery_lifeline_command_executor_capability_table_still_non_executable", "valid_hash_reference_command_still_denied", "recovery_lifeline_command_executor_capability_table_valid_but_execution_disabled", CommandReferenceMutation::None, false),
];

selftest_cases!(
    recovery_lifeline_command_side_effect_gate_selftest_cases,
    RecoveryLifelineCommandSideEffectGateSelfTestCase,
    RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_SELFTEST_CASES,
    valid_side_effect_gate_input(),
    SIDE_EFFECT_GATE_CASES,
    apply_side_effect_gate_case,
    evaluate_recovery_lifeline_command_side_effect_gate_reference
);

fn valid_side_effect_gate_input() -> CommandBindings<'static> {
    let mut input = base_input(0xb1);
    input.retained_executor_capability_table_event_id = Some(EVENT_ID);
    input.handler_binding_hash = Some(h(0xb4));
    input.status_read_handler_hash = Some(h(0xb5));
    input.rollback_preview_authorization_hash = Some(h(0xb6));
    input.rollback_apply_authorization_hash = Some(h(0xb7));
    input.disable_module_target_binding_hash = Some(h(0xb8));
    input.restart_last_good_target_binding_hash = Some(h(0xb9));
    input.load_artifact_by_hash_target_binding_hash = Some(h(0xba));
    input.recovery_memory_write_authority_hash = Some(h(0xbb));
    input.durable_audit_rollback_write_authority_hash = Some(h(0xbc));
    input.service_inventory_side_effect_boundary_hash = Some(h(0xbd));
    input.command_dispatch_behavior_hash = Some(h(0xbe));
    input.executor_capability_table_hash = Some(h(0xbf));
    input.source_rollback_apply_denial_hash = Some(h(0xc0));
    input.source_durable_policy_write_authority_decision_hash = Some(h(0xc1));
    input.source_recovery_rollback_inspect_source_reference_hash = Some(h(0xc2));
    input.side_effect_gate_id = Some(RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID);
    input.side_effect_projection_hash = Some(h(0xc3));
    with_expected_hash(
        input,
        evaluate_recovery_lifeline_command_side_effect_gate_reference,
        set_side_effect_gate_hash,
    )
}

fn set_side_effect_gate_hash(input: &mut CommandBindings<'static>, hash: [u8; 32]) {
    input.side_effect_gate_hash = Some(hash);
}

fn apply_side_effect_gate_case(
    input: &mut CommandBindings<'static>,
    mutation: CommandReferenceMutation,
) {
    if apply_common(input, mutation) {
        return;
    }
    match mutation {
        CommandReferenceMutation::BoundaryIdMismatch => {
            input.side_effect_gate_id =
                Some("boundary.recovery_lifeline_command_side_effect_gate.wrong")
        }
        CommandReferenceMutation::HashMismatch => input.side_effect_gate_hash = Some(BAD_HASH),
        CommandReferenceMutation::None => {}
        _ => {}
    }
}

const SIDE_EFFECT_GATE_CASES: [CaseSpec<CommandReferenceMutation>;
    RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_SELFTEST_CASES] = [
    case(
        "recovery_lifeline_command_side_effect_gate_absent",
        "missing",
        "recovery_lifeline_command_side_effect_gate_absent",
        CommandReferenceMutation::Missing,
        false,
    ),
    case(
        "recovery_lifeline_command_side_effect_gate_arity_invalid",
        "invalid_reference",
        "recovery_lifeline_command_side_effect_gate_arity_invalid",
        CommandReferenceMutation::ArityInvalid,
        false,
    ),
    case(
        "previous_boot_recovery_lifeline_command_side_effect_gate",
        "stale_or_non_current_boot_reference",
        "recovery_lifeline_command_side_effect_gate_scope_must_be_current_boot",
        CommandReferenceMutation::PreviousBoot,
        false,
    ),
    case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandReferenceMutation::UnsupportedCommand,
        false,
    ),
    case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandReferenceMutation::ArgumentSchemaMismatch,
        false,
    ),
    case(
        "dispatch_boundary_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandReferenceMutation::DispatchBoundaryMismatch,
        false,
    ),
    case(
        "side_effect_gate_id_mismatch",
        "rejected",
        "recovery_lifeline_command_side_effect_gate_id_mismatch",
        CommandReferenceMutation::BoundaryIdMismatch,
        false,
    ),
    case(
        "side_effect_gate_hash_mismatch",
        "mismatched_recovery_lifeline_command_side_effect_gate_hash",
        "recovery_lifeline_command_side_effect_gate_hash_mismatch",
        CommandReferenceMutation::HashMismatch,
        false,
    ),
    case(
        "retained_recovery_lifeline_command_executor_capability_table_missing",
        "rejected",
        "retained_recovery_lifeline_command_executor_capability_table_missing",
        CommandReferenceMutation::None,
        true,
    ),
    case(
        "all_inputs_present_recovery_lifeline_command_side_effect_gate_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_side_effect_gate_valid_but_execution_disabled",
        CommandReferenceMutation::None,
        false,
    ),
];
