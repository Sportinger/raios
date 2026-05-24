use crate::{
    agent_protocol_recovery_command_authorization_types::*,
    agent_protocol_recovery_command_dispatch_types::*,
    agent_protocol_recovery_command_effect_types::*,
    agent_protocol_recovery_command_eval::{
        evaluate_recovery_lifeline_command_dispatch,
        recovery_lifeline_command_dispatch_candidate_from_retained,
    },
    agent_protocol_recovery_constants::*,
    agent_protocol_recovery_lifeline::{
        recovery_lifeline_command_spec, RecoveryLifelineCommandSpec,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    },
    agent_protocol_support::{
        current_boot_event_id_str, method_eq, parse_current_boot_event_id, parse_sha256_ref,
    },
    event_log, module_evidence,
};

pub(crate) fn parse_recovery_lifeline_command_envelope_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineCommandEnvelopeReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let command_envelope_reference_hash = parts.next();
    let retained_lifeline_request_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let required_capability = parts.next();
    let target_locator = parts.next();
    let command_admission_boundary_id = parts.next();
    let lifeline_request_reference_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineCommandEnvelopeReferenceInput {
        has_reference: command_envelope_reference_hash.is_some(),
        arity_valid: command_envelope_reference_hash.is_some()
            && retained_lifeline_request_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && required_capability.is_some()
            && target_locator.is_some()
            && command_admission_boundary_id.is_some()
            && lifeline_request_reference_hash.is_some()
            && extra.is_none(),
        scope,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        retained_lifeline_request_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        required_capability,
        target_locator,
        command_admission_boundary_id,
        lifeline_request_reference_hash: lifeline_request_reference_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_command_envelope_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_command_envelope_reference(
    input: RecoveryLifelineCommandEnvelopeReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineCommandEnvelopeReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_lifeline_command_envelope_reference_absent",
            false,
        );
    }
    let Some(retained_lifeline_request_event_id) = input.retained_lifeline_request_event_id else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    let Some(required_capability) = input.required_capability else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    let Some(command_admission_boundary_id) = input.command_admission_boundary_id else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    let Some(lifeline_request_reference_hash) = input.lifeline_request_reference_hash else {
        return recovery_lifeline_command_envelope_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_envelope_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_envelope_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_lifeline_request_event_id) {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_lifeline_request_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(required_capability, spec.required_capability) {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_required_capability_mismatch",
            false,
        );
    }
    if !method_eq(
        command_admission_boundary_id,
        RECOVERY_COMMAND_ADMISSION_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_admission_boundary_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_lifeline_command_envelope_reference_hash(
        module_evidence::RecoveryLifelineCommandEnvelopeReferenceHashInput {
            retained_lifeline_request_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            required_capability: spec.required_capability,
            target_locator,
            command_admission_boundary_id: RECOVERY_COMMAND_ADMISSION_BOUNDARY_ID,
            lifeline_request_reference_hash,
        },
    );
    if input.command_envelope_reference_hash != Some(expected) {
        return recovery_lifeline_command_envelope_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_command_envelope_reference_hash",
            "recovery_lifeline_command_envelope_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_lifeline_command_envelope_live_chain_mismatch(&input) {
            return recovery_lifeline_command_envelope_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_command_envelope_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_envelope_reference_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_lifeline_command_envelope_invalid(
    input: RecoveryLifelineCommandEnvelopeReferenceInput<'_>,
) -> RecoveryLifelineCommandEnvelopeReferenceCheck<'_> {
    recovery_lifeline_command_envelope_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_lifeline_command_envelope_reference_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_command_envelope_reference_check<'a>(
    input: RecoveryLifelineCommandEnvelopeReferenceInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_command_envelope_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineCommandEnvelopeReferenceCheck<'a> {
    RecoveryLifelineCommandEnvelopeReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        expected_command_envelope_reference_hash,
        retained_lifeline_request_event_id: input.retained_lifeline_request_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        required_capability: input.required_capability,
        target_locator: input.target_locator,
        command_admission_boundary_id: input.command_admission_boundary_id,
        lifeline_request_reference_hash: input.lifeline_request_reference_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_command_envelope_live_chain_mismatch(
    input: &RecoveryLifelineCommandEnvelopeReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_event_id = parse_current_boot_event_id(input.retained_lifeline_request_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_request_reference()
    else {
        return Some("retained_recovery_lifeline_request_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_recovery_lifeline_request_event_id_stale_or_dropped");
    }
    if input.lifeline_request_reference_hash
        != Some(latest_reference.lifeline_request_reference_hash)
    {
        return Some("retained_recovery_lifeline_request_reference_hash_mismatch");
    }
    None
}

pub(crate) fn recovery_lifeline_command_envelope_binding_from_check(
    check: &RecoveryLifelineCommandEnvelopeReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineCommandEnvelopeReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryLifelineCommandEnvelopeReference {
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        retained_lifeline_request_event_id: parse_current_boot_event_id(
            check.retained_lifeline_request_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        required_capability: spec.required_capability,
        target_locator: check.target_locator_value?,
        command_admission_boundary_id: RECOVERY_COMMAND_ADMISSION_BOUNDARY_ID,
        lifeline_request_reference_hash: check.lifeline_request_reference_hash?,
    })
}

pub(crate) fn parse_recovery_lifeline_command_body_canonicalization_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let command_body_canonicalization_hash = parts.next();
    let retained_command_envelope_reference_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineCommandBodyCanonicalizationInput {
        has_reference: command_body_canonicalization_hash.is_some(),
        arity_valid: command_body_canonicalization_hash.is_some()
            && retained_command_envelope_reference_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && extra.is_none(),
        scope,
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        retained_command_envelope_reference_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
    };
    evaluate_recovery_lifeline_command_body_canonicalization_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_command_body_canonicalization_reference(
    input: RecoveryLifelineCommandBodyCanonicalizationInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_lifeline_command_body_canonicalization_absent",
            false,
        );
    }
    let Some(retained_command_envelope_reference_event_id) =
        input.retained_command_envelope_reference_event_id
    else {
        return recovery_lifeline_command_body_canonicalization_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_command_body_canonicalization_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_command_body_canonicalization_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_command_body_canonicalization_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_command_body_canonicalization_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_lifeline_command_body_canonicalization_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_lifeline_command_body_canonicalization_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_body_canonicalization_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_body_canonicalization_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_command_envelope_reference_event_id) {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_lifeline_command_envelope_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_lifeline_command_body_canonicalization_hash(
        module_evidence::RecoveryLifelineCommandBodyCanonicalizationHashInput {
            retained_command_envelope_reference_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        },
    );
    if input.command_body_canonicalization_hash != Some(expected) {
        return recovery_lifeline_command_body_canonicalization_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_command_body_canonicalization_hash",
            "recovery_lifeline_command_body_canonicalization_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) =
            recovery_lifeline_command_body_canonicalization_live_chain_mismatch(&input)
        {
            return recovery_lifeline_command_body_canonicalization_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_command_body_canonicalization_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_body_canonicalization_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_invalid(
    input: RecoveryLifelineCommandBodyCanonicalizationInput<'_>,
) -> RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_> {
    recovery_lifeline_command_body_canonicalization_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_lifeline_command_body_canonicalization_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_reference_check<'a>(
    input: RecoveryLifelineCommandBodyCanonicalizationInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_command_body_canonicalization_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'a> {
    RecoveryLifelineCommandBodyCanonicalizationReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        expected_command_body_canonicalization_hash,
        retained_command_envelope_reference_event_id: input
            .retained_command_envelope_reference_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_live_chain_mismatch(
    input: &RecoveryLifelineCommandBodyCanonicalizationInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_command_envelope_reference_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_command_envelope_reference()
    else {
        return Some("retained_recovery_lifeline_command_envelope_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_recovery_lifeline_command_envelope_event_id_stale_or_dropped");
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
    {
        return Some("recovery_lifeline_command_envelope_reference_binding_mismatch");
    }
    let target_locator = input.target_locator?;
    if !method_eq(target_locator, latest_reference.target_locator.as_str()) {
        return Some("recovery_lifeline_command_envelope_reference_target_mismatch");
    }
    let dispatch_candidate = recovery_lifeline_command_dispatch_candidate_from_retained(
        Some((latest_event_id, latest_reference)),
        event_log::latest_recovery_lifeline_request_reference(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let dispatch_check = evaluate_recovery_lifeline_command_dispatch(dispatch_candidate);
    if !method_eq(
        dispatch_check.status,
        "denied_missing_lifeline_command_dispatch_boundary",
    ) || !method_eq(
        dispatch_check.reason,
        "recovery_lifeline_command_body_canonicalization_missing",
    ) {
        return Some(dispatch_check.reason);
    }
    None
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_binding_from_check(
    check: &RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineCommandBodyCanonicalizationReference> {
    let spec = check.normalized_spec?;
    Some(
        event_log::RecoveryLifelineCommandBodyCanonicalizationReference {
            command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
            retained_command_envelope_reference_event_id: parse_current_boot_event_id(
                check.retained_command_envelope_reference_event_id?,
            )?,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash: check.argument_hash?,
            target_locator: check.target_locator_value?,
            command_envelope_reference_hash: check.command_envelope_reference_hash?,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        },
    )
}

pub(crate) fn parse_recovery_lifeline_command_handler_binding_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineCommandHandlerBindingReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let handler_binding_hash = parts.next();
    let retained_command_body_canonicalization_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let handler_id = parts.next();
    let handler_input_binding_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineCommandHandlerBindingInput {
        has_reference: handler_binding_hash.is_some(),
        arity_valid: handler_binding_hash.is_some()
            && retained_command_body_canonicalization_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && handler_id.is_some()
            && handler_input_binding_hash.is_some()
            && extra.is_none(),
        scope,
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        retained_command_body_canonicalization_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        handler_id,
        handler_input_binding_hash: handler_input_binding_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_command_handler_binding_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_command_handler_binding_reference(
    input: RecoveryLifelineCommandHandlerBindingInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineCommandHandlerBindingReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_lifeline_command_handler_binding_absent",
            false,
        );
    }
    let Some(retained_body_event_id) = input.retained_command_body_canonicalization_event_id else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(handler_id) = input.handler_id else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    let Some(handler_input_binding_hash) = input.handler_input_binding_hash else {
        return recovery_lifeline_command_handler_binding_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_handler_binding_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_handler_binding_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_body_event_id) {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_lifeline_command_body_canonicalization_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(handler_id, RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID) {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_handler_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_lifeline_command_handler_binding_hash(
        module_evidence::RecoveryLifelineCommandHandlerBindingHashInput {
            retained_command_body_canonicalization_event_id: retained_body_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            handler_id: RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID,
            handler_input_binding_hash,
        },
    );
    if input.handler_binding_hash != Some(expected) {
        return recovery_lifeline_command_handler_binding_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_command_handler_binding_hash",
            "recovery_lifeline_command_handler_binding_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_lifeline_command_handler_binding_live_chain_mismatch(&input)
        {
            return recovery_lifeline_command_handler_binding_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_command_handler_binding_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_handler_binding_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_lifeline_command_handler_binding_invalid(
    input: RecoveryLifelineCommandHandlerBindingInput<'_>,
) -> RecoveryLifelineCommandHandlerBindingReferenceCheck<'_> {
    recovery_lifeline_command_handler_binding_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_lifeline_command_handler_binding_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_command_handler_binding_reference_check<'a>(
    input: RecoveryLifelineCommandHandlerBindingInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_handler_binding_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineCommandHandlerBindingReferenceCheck<'a> {
    RecoveryLifelineCommandHandlerBindingReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        handler_binding_hash: input.handler_binding_hash,
        expected_handler_binding_hash,
        retained_command_body_canonicalization_event_id: input
            .retained_command_body_canonicalization_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        handler_id: input.handler_id,
        handler_input_binding_hash: input.handler_input_binding_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_command_handler_binding_live_chain_mismatch(
    input: &RecoveryLifelineCommandHandlerBindingInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_command_body_canonicalization_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_command_body_canonicalization_reference()
    else {
        return Some("retained_recovery_lifeline_command_body_canonicalization_missing");
    };
    if latest_event_id != retained_event_id {
        return Some(
            "retained_recovery_lifeline_command_body_canonicalization_event_id_stale_or_dropped",
        );
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
    {
        return Some("recovery_lifeline_command_body_canonicalization_binding_mismatch");
    }
    None
}

pub(crate) fn recovery_lifeline_command_handler_binding_from_check(
    check: &RecoveryLifelineCommandHandlerBindingReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineCommandHandlerBindingReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryLifelineCommandHandlerBindingReference {
        handler_binding_hash: check.handler_binding_hash?,
        retained_command_body_canonicalization_event_id: parse_current_boot_event_id(
            check.retained_command_body_canonicalization_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        handler_id: RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID,
        handler_input_binding_hash: check.handler_input_binding_hash?,
    })
}

pub(crate) fn parse_recovery_lifeline_status_read_handler_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineStatusReadHandlerReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let status_read_handler_hash = parts.next();
    let retained_command_handler_binding_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let status_handler_id = parts.next();
    let status_read_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineStatusReadHandlerInput {
        has_reference: status_read_handler_hash.is_some(),
        arity_valid: status_read_handler_hash.is_some()
            && retained_command_handler_binding_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && status_handler_id.is_some()
            && status_read_projection_hash.is_some()
            && extra.is_none(),
        scope,
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        retained_command_handler_binding_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        status_handler_id,
        status_read_projection_hash: status_read_projection_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_status_read_handler_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_status_read_handler_reference(
    input: RecoveryLifelineStatusReadHandlerInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineStatusReadHandlerReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_lifeline_status_read_handler_absent",
            false,
        );
    }
    let Some(retained_handler_event_id) = input.retained_command_handler_binding_event_id else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(status_handler_id) = input.status_handler_id else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    let Some(status_read_projection_hash) = input.status_read_projection_hash else {
        return recovery_lifeline_status_read_handler_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_status_read_handler_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_status_read_handler_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_handler_event_id) {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_lifeline_command_handler_binding_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(status_handler_id, RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID) {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_status_read_handler_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_lifeline_status_read_handler_hash(
        module_evidence::RecoveryLifelineStatusReadHandlerHashInput {
            retained_command_handler_binding_event_id: retained_handler_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            status_handler_id: RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID,
            status_read_projection_hash,
        },
    );
    if input.status_read_handler_hash != Some(expected) {
        return recovery_lifeline_status_read_handler_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_status_read_handler_hash",
            "recovery_lifeline_status_read_handler_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_lifeline_status_read_handler_live_chain_mismatch(&input) {
            return recovery_lifeline_status_read_handler_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_status_read_handler_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_status_read_handler_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_lifeline_status_read_handler_invalid(
    input: RecoveryLifelineStatusReadHandlerInput<'_>,
) -> RecoveryLifelineStatusReadHandlerReferenceCheck<'_> {
    recovery_lifeline_status_read_handler_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_lifeline_status_read_handler_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_status_read_handler_reference_check<'a>(
    input: RecoveryLifelineStatusReadHandlerInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_status_read_handler_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineStatusReadHandlerReferenceCheck<'a> {
    RecoveryLifelineStatusReadHandlerReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        status_read_handler_hash: input.status_read_handler_hash,
        expected_status_read_handler_hash,
        retained_command_handler_binding_event_id: input.retained_command_handler_binding_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        status_handler_id: input.status_handler_id,
        status_read_projection_hash: input.status_read_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_status_read_handler_live_chain_mismatch(
    input: &RecoveryLifelineStatusReadHandlerInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_command_handler_binding_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_command_handler_binding_reference()
    else {
        return Some("retained_recovery_lifeline_command_handler_binding_missing");
    };
    if latest_event_id != retained_event_id {
        return Some(
            "retained_recovery_lifeline_command_handler_binding_event_id_stale_or_dropped",
        );
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_lifeline_command_handler_binding_mismatch");
    }
    None
}

pub(crate) fn recovery_lifeline_status_read_handler_from_check(
    check: &RecoveryLifelineStatusReadHandlerReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineStatusReadHandlerReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryLifelineStatusReadHandlerReference {
        status_read_handler_hash: check.status_read_handler_hash?,
        retained_command_handler_binding_event_id: parse_current_boot_event_id(
            check.retained_command_handler_binding_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        status_handler_id: RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID,
        status_read_projection_hash: check.status_read_projection_hash?,
    })
}

pub(crate) fn parse_recovery_rollback_preview_authorization_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryRollbackPreviewAuthorizationReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let rollback_preview_authorization_hash = parts.next();
    let retained_status_read_handler_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let rollback_preview_authorization_id = parts.next();
    let rollback_preview_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryRollbackPreviewAuthorizationInput {
        has_reference: rollback_preview_authorization_hash.is_some(),
        arity_valid: rollback_preview_authorization_hash.is_some()
            && retained_status_read_handler_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && rollback_preview_authorization_id.is_some()
            && rollback_preview_projection_hash.is_some()
            && extra.is_none(),
        scope,
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        retained_status_read_handler_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        rollback_preview_authorization_id,
        rollback_preview_projection_hash: rollback_preview_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_recovery_rollback_preview_authorization_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_rollback_preview_authorization_reference(
    input: RecoveryRollbackPreviewAuthorizationInput<'_>,
    require_live_retained: bool,
) -> RecoveryRollbackPreviewAuthorizationReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_rollback_preview_authorization_absent",
            false,
        );
    }
    let Some(retained_status_event_id) = input.retained_status_read_handler_event_id else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(rollback_preview_authorization_id) = input.rollback_preview_authorization_id else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    let Some(rollback_preview_projection_hash) = input.rollback_preview_projection_hash else {
        return recovery_rollback_preview_authorization_invalid(input);
    };
    if !input.arity_valid {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_rollback_preview_authorization_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_rollback_preview_authorization_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_status_event_id) {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_lifeline_status_read_handler_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        rollback_preview_authorization_id,
        RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID,
    ) {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_rollback_preview_authorization_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_rollback_preview_authorization_hash(
        module_evidence::RecoveryRollbackPreviewAuthorizationHashInput {
            retained_status_read_handler_event_id: retained_status_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            rollback_preview_authorization_id: RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID,
            rollback_preview_projection_hash,
        },
    );
    if input.rollback_preview_authorization_hash != Some(expected) {
        return recovery_rollback_preview_authorization_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_rollback_preview_authorization_hash",
            "recovery_rollback_preview_authorization_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_rollback_preview_authorization_live_chain_mismatch(&input) {
            return recovery_rollback_preview_authorization_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_rollback_preview_authorization_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_rollback_preview_authorization_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_rollback_preview_authorization_invalid(
    input: RecoveryRollbackPreviewAuthorizationInput<'_>,
) -> RecoveryRollbackPreviewAuthorizationReferenceCheck<'_> {
    recovery_rollback_preview_authorization_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_rollback_preview_authorization_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_rollback_preview_authorization_reference_check<'a>(
    input: RecoveryRollbackPreviewAuthorizationInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_rollback_preview_authorization_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryRollbackPreviewAuthorizationReferenceCheck<'a> {
    RecoveryRollbackPreviewAuthorizationReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        expected_rollback_preview_authorization_hash,
        retained_status_read_handler_event_id: input.retained_status_read_handler_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        rollback_preview_authorization_id: input.rollback_preview_authorization_id,
        rollback_preview_projection_hash: input.rollback_preview_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_rollback_preview_authorization_live_chain_mismatch(
    input: &RecoveryRollbackPreviewAuthorizationInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_status_read_handler_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_status_read_handler_reference()
    else {
        return Some("retained_recovery_lifeline_status_read_handler_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_recovery_lifeline_status_read_handler_event_id_stale_or_dropped");
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_lifeline_status_read_handler_mismatch");
    }
    None
}

pub(crate) fn recovery_rollback_preview_authorization_from_check(
    check: &RecoveryRollbackPreviewAuthorizationReferenceCheck<'_>,
) -> Option<event_log::RecoveryRollbackPreviewAuthorizationReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryRollbackPreviewAuthorizationReference {
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        retained_status_read_handler_event_id: parse_current_boot_event_id(
            check.retained_status_read_handler_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        rollback_preview_authorization_id: RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID,
        rollback_preview_projection_hash: check.rollback_preview_projection_hash?,
    })
}

pub(crate) fn parse_recovery_rollback_apply_authorization_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryRollbackApplyAuthorizationReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let rollback_apply_authorization_hash = parts.next();
    let retained_rollback_preview_authorization_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let rollback_apply_authorization_id = parts.next();
    let rollback_apply_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryRollbackApplyAuthorizationInput {
        has_reference: rollback_apply_authorization_hash.is_some(),
        arity_valid: rollback_apply_authorization_hash.is_some()
            && retained_rollback_preview_authorization_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && rollback_apply_authorization_id.is_some()
            && rollback_apply_projection_hash.is_some()
            && extra.is_none(),
        scope,
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        retained_rollback_preview_authorization_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        rollback_apply_authorization_id,
        rollback_apply_projection_hash: rollback_apply_projection_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_rollback_apply_authorization_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_rollback_apply_authorization_reference(
    input: RecoveryRollbackApplyAuthorizationInput<'_>,
    require_live_retained: bool,
) -> RecoveryRollbackApplyAuthorizationReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_rollback_apply_authorization_absent",
            false,
        );
    }
    let Some(retained_preview_event_id) = input.retained_rollback_preview_authorization_event_id
    else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(rollback_apply_authorization_id) = input.rollback_apply_authorization_id else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    let Some(rollback_apply_projection_hash) = input.rollback_apply_projection_hash else {
        return recovery_rollback_apply_authorization_invalid(input);
    };
    if !input.arity_valid {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_rollback_apply_authorization_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_rollback_apply_authorization_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_preview_event_id) {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_rollback_preview_authorization_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        rollback_apply_authorization_id,
        RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID,
    ) {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_rollback_apply_authorization_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_rollback_apply_authorization_hash(
        module_evidence::RecoveryRollbackApplyAuthorizationHashInput {
            retained_rollback_preview_authorization_event_id: retained_preview_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            rollback_apply_authorization_id: RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID,
            rollback_apply_projection_hash,
        },
    );
    if input.rollback_apply_authorization_hash != Some(expected) {
        return recovery_rollback_apply_authorization_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_rollback_apply_authorization_hash",
            "recovery_rollback_apply_authorization_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_rollback_apply_authorization_live_chain_mismatch(&input) {
            return recovery_rollback_apply_authorization_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_rollback_apply_authorization_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_rollback_apply_authorization_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_rollback_apply_authorization_invalid(
    input: RecoveryRollbackApplyAuthorizationInput<'_>,
) -> RecoveryRollbackApplyAuthorizationReferenceCheck<'_> {
    recovery_rollback_apply_authorization_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_rollback_apply_authorization_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_rollback_apply_authorization_reference_check<'a>(
    input: RecoveryRollbackApplyAuthorizationInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_rollback_apply_authorization_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryRollbackApplyAuthorizationReferenceCheck<'a> {
    RecoveryRollbackApplyAuthorizationReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        expected_rollback_apply_authorization_hash,
        retained_rollback_preview_authorization_event_id: input
            .retained_rollback_preview_authorization_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        rollback_apply_authorization_id: input.rollback_apply_authorization_id,
        rollback_apply_projection_hash: input.rollback_apply_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_rollback_apply_authorization_live_chain_mismatch(
    input: &RecoveryRollbackApplyAuthorizationInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_rollback_preview_authorization_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_rollback_preview_authorization_reference()
    else {
        return Some("retained_recovery_rollback_preview_authorization_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_recovery_rollback_preview_authorization_event_id_stale_or_dropped");
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_rollback_preview_authorization_mismatch");
    }
    None
}

pub(crate) fn recovery_rollback_apply_authorization_from_check(
    check: &RecoveryRollbackApplyAuthorizationReferenceCheck<'_>,
) -> Option<event_log::RecoveryRollbackApplyAuthorizationReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryRollbackApplyAuthorizationReference {
        rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
        retained_rollback_preview_authorization_event_id: parse_current_boot_event_id(
            check.retained_rollback_preview_authorization_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        rollback_apply_authorization_id: RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID,
        rollback_apply_projection_hash: check.rollback_apply_projection_hash?,
    })
}

pub(crate) fn parse_recovery_disable_module_target_binding_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryDisableModuleTargetBindingReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let disable_module_target_binding_hash = parts.next();
    let retained_rollback_apply_authorization_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let disable_module_target_id = parts.next();
    let disable_module_target_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryDisableModuleTargetBindingInput {
        has_reference: disable_module_target_binding_hash.is_some(),
        arity_valid: disable_module_target_binding_hash.is_some()
            && retained_rollback_apply_authorization_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && disable_module_target_id.is_some()
            && disable_module_target_projection_hash.is_some()
            && extra.is_none(),
        scope,
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        retained_rollback_apply_authorization_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        disable_module_target_id,
        disable_module_target_projection_hash: disable_module_target_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_recovery_disable_module_target_binding_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_disable_module_target_binding_reference(
    input: RecoveryDisableModuleTargetBindingInput<'_>,
    require_live_retained: bool,
) -> RecoveryDisableModuleTargetBindingReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_disable_module_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_disable_module_target_binding_absent",
            false,
        );
    }
    let Some(retained_apply_event_id) = input.retained_rollback_apply_authorization_event_id else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(disable_module_target_id) = input.disable_module_target_id else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    let Some(disable_module_target_projection_hash) = input.disable_module_target_projection_hash
    else {
        return recovery_disable_module_target_binding_invalid(input);
    };
    if !input.arity_valid {
        return recovery_disable_module_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_disable_module_target_binding_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_disable_module_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_disable_module_target_binding_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_apply_event_id) {
        return recovery_disable_module_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_rollback_apply_authorization_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_disable_module_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_disable_module_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_disable_module_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        disable_module_target_id,
        RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID,
    ) {
        return recovery_disable_module_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_disable_module_target_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_disable_module_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_disable_module_target_binding_hash(
        module_evidence::RecoveryDisableModuleTargetBindingHashInput {
            retained_rollback_apply_authorization_event_id: retained_apply_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            disable_module_target_id: RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID,
            disable_module_target_projection_hash,
        },
    );
    if input.disable_module_target_binding_hash != Some(expected) {
        return recovery_disable_module_target_binding_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_disable_module_target_binding_hash",
            "recovery_disable_module_target_binding_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_disable_module_target_binding_live_chain_mismatch(&input) {
            return recovery_disable_module_target_binding_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_disable_module_target_binding_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_disable_module_target_binding_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_disable_module_target_binding_invalid(
    input: RecoveryDisableModuleTargetBindingInput<'_>,
) -> RecoveryDisableModuleTargetBindingReferenceCheck<'_> {
    recovery_disable_module_target_binding_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_disable_module_target_binding_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_disable_module_target_binding_reference_check<'a>(
    input: RecoveryDisableModuleTargetBindingInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_disable_module_target_binding_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryDisableModuleTargetBindingReferenceCheck<'a> {
    RecoveryDisableModuleTargetBindingReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        expected_disable_module_target_binding_hash,
        retained_rollback_apply_authorization_event_id: input
            .retained_rollback_apply_authorization_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        disable_module_target_id: input.disable_module_target_id,
        disable_module_target_projection_hash: input.disable_module_target_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_disable_module_target_binding_live_chain_mismatch(
    input: &RecoveryDisableModuleTargetBindingInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_rollback_apply_authorization_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_rollback_apply_authorization_reference()
    else {
        return Some("retained_recovery_rollback_apply_authorization_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_recovery_rollback_apply_authorization_event_id_stale_or_dropped");
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_rollback_apply_authorization_mismatch");
    }
    None
}

pub(crate) fn recovery_disable_module_target_binding_from_check(
    check: &RecoveryDisableModuleTargetBindingReferenceCheck<'_>,
) -> Option<event_log::RecoveryDisableModuleTargetBindingReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryDisableModuleTargetBindingReference {
        disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
        retained_rollback_apply_authorization_event_id: parse_current_boot_event_id(
            check.retained_rollback_apply_authorization_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        disable_module_target_id: RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID,
        disable_module_target_projection_hash: check.disable_module_target_projection_hash?,
    })
}

pub(crate) fn parse_recovery_restart_last_good_target_binding_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryRestartLastGoodTargetBindingReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let restart_last_good_target_binding_hash = parts.next();
    let retained_disable_module_target_binding_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let restart_last_good_target_id = parts.next();
    let restart_last_good_target_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryRestartLastGoodTargetBindingInput {
        has_reference: restart_last_good_target_binding_hash.is_some(),
        arity_valid: restart_last_good_target_binding_hash.is_some()
            && retained_disable_module_target_binding_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && restart_last_good_target_id.is_some()
            && restart_last_good_target_projection_hash.is_some()
            && extra.is_none(),
        scope,
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        retained_disable_module_target_binding_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        restart_last_good_target_id,
        restart_last_good_target_projection_hash: restart_last_good_target_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_recovery_restart_last_good_target_binding_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_restart_last_good_target_binding_reference(
    input: RecoveryRestartLastGoodTargetBindingInput<'_>,
    require_live_retained: bool,
) -> RecoveryRestartLastGoodTargetBindingReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_restart_last_good_target_binding_absent",
            false,
        );
    }
    let Some(retained_disable_event_id) = input.retained_disable_module_target_binding_event_id
    else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(restart_last_good_target_id) = input.restart_last_good_target_id else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    let Some(restart_last_good_target_projection_hash) =
        input.restart_last_good_target_projection_hash
    else {
        return recovery_restart_last_good_target_binding_invalid(input);
    };
    if !input.arity_valid {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_restart_last_good_target_binding_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_restart_last_good_target_binding_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_disable_event_id) {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_disable_module_target_binding_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        restart_last_good_target_id,
        RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID,
    ) {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_restart_last_good_target_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_restart_last_good_target_binding_hash(
        module_evidence::RecoveryRestartLastGoodTargetBindingHashInput {
            retained_disable_module_target_binding_event_id: retained_disable_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            restart_last_good_target_id: RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID,
            restart_last_good_target_projection_hash,
        },
    );
    if input.restart_last_good_target_binding_hash != Some(expected) {
        return recovery_restart_last_good_target_binding_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_restart_last_good_target_binding_hash",
            "recovery_restart_last_good_target_binding_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_restart_last_good_target_binding_live_chain_mismatch(&input)
        {
            return recovery_restart_last_good_target_binding_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_restart_last_good_target_binding_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_restart_last_good_target_binding_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_restart_last_good_target_binding_invalid(
    input: RecoveryRestartLastGoodTargetBindingInput<'_>,
) -> RecoveryRestartLastGoodTargetBindingReferenceCheck<'_> {
    recovery_restart_last_good_target_binding_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_restart_last_good_target_binding_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_restart_last_good_target_binding_reference_check<'a>(
    input: RecoveryRestartLastGoodTargetBindingInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_restart_last_good_target_binding_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryRestartLastGoodTargetBindingReferenceCheck<'a> {
    RecoveryRestartLastGoodTargetBindingReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        expected_restart_last_good_target_binding_hash,
        retained_disable_module_target_binding_event_id: input
            .retained_disable_module_target_binding_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        restart_last_good_target_id: input.restart_last_good_target_id,
        restart_last_good_target_projection_hash: input.restart_last_good_target_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_restart_last_good_target_binding_live_chain_mismatch(
    input: &RecoveryRestartLastGoodTargetBindingInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_disable_module_target_binding_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_disable_module_target_binding_reference()
    else {
        return Some("retained_recovery_disable_module_target_binding_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_recovery_disable_module_target_binding_event_id_stale_or_dropped");
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_disable_module_target_binding_mismatch");
    }
    None
}

pub(crate) fn recovery_restart_last_good_target_binding_from_check(
    check: &RecoveryRestartLastGoodTargetBindingReferenceCheck<'_>,
) -> Option<event_log::RecoveryRestartLastGoodTargetBindingReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryRestartLastGoodTargetBindingReference {
        restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
        retained_disable_module_target_binding_event_id: parse_current_boot_event_id(
            check.retained_disable_module_target_binding_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
        disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        restart_last_good_target_id: RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID,
        restart_last_good_target_projection_hash: check.restart_last_good_target_projection_hash?,
    })
}

pub(crate) fn parse_recovery_load_artifact_by_hash_target_binding_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let retained_restart_last_good_target_binding_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let load_artifact_by_hash_target_id = parts.next();
    let load_artifact_by_hash_target_artifact_hash = parts.next();
    let load_artifact_by_hash_target_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLoadArtifactByHashTargetBindingInput {
        has_reference: load_artifact_by_hash_target_binding_hash.is_some(),
        arity_valid: load_artifact_by_hash_target_binding_hash.is_some()
            && retained_restart_last_good_target_binding_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && load_artifact_by_hash_target_id.is_some()
            && load_artifact_by_hash_target_artifact_hash.is_some()
            && load_artifact_by_hash_target_projection_hash.is_some()
            && extra.is_none(),
        scope,
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        retained_restart_last_good_target_binding_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        load_artifact_by_hash_target_id,
        load_artifact_by_hash_target_artifact_hash: load_artifact_by_hash_target_artifact_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_projection_hash: load_artifact_by_hash_target_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_recovery_load_artifact_by_hash_target_binding_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_load_artifact_by_hash_target_binding_reference(
    input: RecoveryLoadArtifactByHashTargetBindingInput<'_>,
    require_live_retained: bool,
) -> RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_load_artifact_by_hash_target_binding_absent",
            false,
        );
    }
    let Some(retained_restart_event_id) = input.retained_restart_last_good_target_binding_event_id
    else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(load_artifact_by_hash_target_id) = input.load_artifact_by_hash_target_id else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(load_artifact_by_hash_target_artifact_hash) =
        input.load_artifact_by_hash_target_artifact_hash
    else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    let Some(load_artifact_by_hash_target_projection_hash) =
        input.load_artifact_by_hash_target_projection_hash
    else {
        return recovery_load_artifact_by_hash_target_binding_invalid(input);
    };
    if !input.arity_valid {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_load_artifact_by_hash_target_binding_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_load_artifact_by_hash_target_binding_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_restart_event_id) {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_restart_last_good_target_binding_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        load_artifact_by_hash_target_id,
        RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID,
    ) {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_load_artifact_by_hash_target_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_load_artifact_by_hash_target_binding_hash(
        module_evidence::RecoveryLoadArtifactByHashTargetBindingHashInput {
            retained_restart_last_good_target_binding_event_id: retained_restart_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            restart_last_good_target_binding_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            load_artifact_by_hash_target_id:
                RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID,
            load_artifact_by_hash_target_artifact_hash,
            load_artifact_by_hash_target_projection_hash,
        },
    );
    if input.load_artifact_by_hash_target_binding_hash != Some(expected) {
        return recovery_load_artifact_by_hash_target_binding_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_load_artifact_by_hash_target_binding_hash",
            "recovery_load_artifact_by_hash_target_binding_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) =
            recovery_load_artifact_by_hash_target_binding_live_chain_mismatch(&input)
        {
            return recovery_load_artifact_by_hash_target_binding_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_load_artifact_by_hash_target_binding_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_load_artifact_by_hash_target_binding_valid_but_command_dispatch_disabled",
        true,
    )
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_invalid(
    input: RecoveryLoadArtifactByHashTargetBindingInput<'_>,
) -> RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_> {
    recovery_load_artifact_by_hash_target_binding_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_load_artifact_by_hash_target_binding_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_reference_check<'a>(
    input: RecoveryLoadArtifactByHashTargetBindingInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_load_artifact_by_hash_target_binding_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'a> {
    RecoveryLoadArtifactByHashTargetBindingReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        expected_load_artifact_by_hash_target_binding_hash,
        retained_restart_last_good_target_binding_event_id: input
            .retained_restart_last_good_target_binding_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        load_artifact_by_hash_target_id: input.load_artifact_by_hash_target_id,
        load_artifact_by_hash_target_artifact_hash: input
            .load_artifact_by_hash_target_artifact_hash,
        load_artifact_by_hash_target_projection_hash: input
            .load_artifact_by_hash_target_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_live_chain_mismatch(
    input: &RecoveryLoadArtifactByHashTargetBindingInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_restart_last_good_target_binding_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_restart_last_good_target_binding_reference()
    else {
        return Some("retained_recovery_restart_last_good_target_binding_missing");
    };
    if latest_event_id != retained_event_id {
        return Some(
            "retained_recovery_restart_last_good_target_binding_event_id_stale_or_dropped",
        );
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || input.restart_last_good_target_binding_hash
            != Some(latest_reference.restart_last_good_target_binding_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_restart_last_good_target_binding_mismatch");
    }
    None
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_from_check(
    check: &RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_>,
) -> Option<event_log::RecoveryLoadArtifactByHashTargetBindingReference> {
    let spec = check.normalized_spec?;
    Some(
        event_log::RecoveryLoadArtifactByHashTargetBindingReference {
            load_artifact_by_hash_target_binding_hash: check
                .load_artifact_by_hash_target_binding_hash?,
            retained_restart_last_good_target_binding_event_id: parse_current_boot_event_id(
                check.retained_restart_last_good_target_binding_event_id?,
            )?,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash: check.argument_hash?,
            target_locator: check.target_locator_value?,
            command_envelope_reference_hash: check.command_envelope_reference_hash?,
            command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
            handler_binding_hash: check.handler_binding_hash?,
            status_read_handler_hash: check.status_read_handler_hash?,
            rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
            rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
            disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
            restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            load_artifact_by_hash_target_id:
                RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID,
            load_artifact_by_hash_target_artifact_hash: check
                .load_artifact_by_hash_target_artifact_hash?,
            load_artifact_by_hash_target_projection_hash: check
                .load_artifact_by_hash_target_projection_hash?,
        },
    )
}

pub(crate) fn parse_recovery_memory_write_authority_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryMemoryWriteAuthorityReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let recovery_memory_write_authority_hash = parts.next();
    let retained_load_artifact_by_hash_target_binding_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let recovery_memory_write_authority_id = parts.next();
    let recovery_memory_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryMemoryWriteAuthorityInput {
        has_reference: recovery_memory_write_authority_hash.is_some(),
        arity_valid: recovery_memory_write_authority_hash.is_some()
            && retained_load_artifact_by_hash_target_binding_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && load_artifact_by_hash_target_binding_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && recovery_memory_write_authority_id.is_some()
            && recovery_memory_projection_hash.is_some()
            && extra.is_none(),
        scope,
        recovery_memory_write_authority_hash: recovery_memory_write_authority_hash
            .and_then(parse_sha256_ref),
        retained_load_artifact_by_hash_target_binding_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        recovery_memory_write_authority_id,
        recovery_memory_projection_hash: recovery_memory_projection_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_memory_write_authority_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_memory_write_authority_reference(
    input: RecoveryMemoryWriteAuthorityInput<'_>,
    require_live_retained: bool,
) -> RecoveryMemoryWriteAuthorityReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_memory_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_memory_write_authority_absent",
            false,
        );
    }
    let Some(retained_load_target_event_id) =
        input.retained_load_artifact_by_hash_target_binding_event_id
    else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(load_artifact_by_hash_target_binding_hash) =
        input.load_artifact_by_hash_target_binding_hash
    else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(recovery_memory_write_authority_id) = input.recovery_memory_write_authority_id else {
        return recovery_memory_write_authority_invalid(input);
    };
    let Some(recovery_memory_projection_hash) = input.recovery_memory_projection_hash else {
        return recovery_memory_write_authority_invalid(input);
    };
    if !input.arity_valid {
        return recovery_memory_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_memory_write_authority_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_memory_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_memory_write_authority_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_load_target_event_id) {
        return recovery_memory_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_load_artifact_by_hash_target_binding_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_memory_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_memory_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_memory_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        recovery_memory_write_authority_id,
        RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID,
    ) {
        return recovery_memory_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_memory_write_authority_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_memory_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_memory_write_authority_hash(
        module_evidence::RecoveryMemoryWriteAuthorityHashInput {
            retained_load_artifact_by_hash_target_binding_event_id: retained_load_target_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            restart_last_good_target_binding_hash,
            load_artifact_by_hash_target_binding_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            recovery_memory_write_authority_id: RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID,
            recovery_memory_projection_hash,
        },
    );
    if input.recovery_memory_write_authority_hash != Some(expected) {
        return recovery_memory_write_authority_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_recovery_memory_write_authority_hash",
            "recovery_memory_write_authority_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_memory_write_authority_live_chain_mismatch(&input) {
            return recovery_memory_write_authority_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_memory_write_authority_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_memory_write_authority_valid_but_memory_writes_disabled",
        true,
    )
}

pub(crate) fn recovery_memory_write_authority_invalid(
    input: RecoveryMemoryWriteAuthorityInput<'_>,
) -> RecoveryMemoryWriteAuthorityReferenceCheck<'_> {
    recovery_memory_write_authority_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_memory_write_authority_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_memory_write_authority_reference_check<'a>(
    input: RecoveryMemoryWriteAuthorityInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_recovery_memory_write_authority_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryMemoryWriteAuthorityReferenceCheck<'a> {
    RecoveryMemoryWriteAuthorityReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        recovery_memory_write_authority_hash: input.recovery_memory_write_authority_hash,
        expected_recovery_memory_write_authority_hash,
        retained_load_artifact_by_hash_target_binding_event_id: input
            .retained_load_artifact_by_hash_target_binding_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        recovery_memory_write_authority_id: input.recovery_memory_write_authority_id,
        recovery_memory_projection_hash: input.recovery_memory_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_memory_write_authority_live_chain_mismatch(
    input: &RecoveryMemoryWriteAuthorityInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_load_artifact_by_hash_target_binding_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_load_artifact_by_hash_target_binding_reference()
    else {
        return Some("retained_recovery_load_artifact_by_hash_target_binding_missing");
    };
    if latest_event_id != retained_event_id {
        return Some(
            "retained_recovery_load_artifact_by_hash_target_binding_event_id_stale_or_dropped",
        );
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || input.restart_last_good_target_binding_hash
            != Some(latest_reference.restart_last_good_target_binding_hash)
        || input.load_artifact_by_hash_target_binding_hash
            != Some(latest_reference.load_artifact_by_hash_target_binding_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_load_artifact_by_hash_target_binding_mismatch");
    }
    None
}

pub(crate) fn recovery_memory_write_authority_from_check(
    check: &RecoveryMemoryWriteAuthorityReferenceCheck<'_>,
) -> Option<event_log::RecoveryMemoryWriteAuthorityReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryMemoryWriteAuthorityReference {
        recovery_memory_write_authority_hash: check.recovery_memory_write_authority_hash?,
        retained_load_artifact_by_hash_target_binding_event_id: parse_current_boot_event_id(
            check.retained_load_artifact_by_hash_target_binding_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
        disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
        restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
        load_artifact_by_hash_target_binding_hash: check
            .load_artifact_by_hash_target_binding_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        recovery_memory_write_authority_id: RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID,
        recovery_memory_projection_hash: check.recovery_memory_projection_hash?,
    })
}

pub(crate) fn parse_durable_audit_rollback_write_authority_reference(
    arg: &str,
    require_live_retained: bool,
) -> DurableAuditRollbackWriteAuthorityReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let durable_audit_rollback_write_authority_hash = parts.next();
    let retained_recovery_memory_write_authority_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let recovery_memory_write_authority_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let durable_audit_rollback_write_authority_id = parts.next();
    let durable_audit_rollback_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = DurableAuditRollbackWriteAuthorityInput {
        has_reference: durable_audit_rollback_write_authority_hash.is_some(),
        arity_valid: durable_audit_rollback_write_authority_hash.is_some()
            && retained_recovery_memory_write_authority_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && load_artifact_by_hash_target_binding_hash.is_some()
            && recovery_memory_write_authority_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && durable_audit_rollback_write_authority_id.is_some()
            && durable_audit_rollback_projection_hash.is_some()
            && extra.is_none(),
        scope,
        durable_audit_rollback_write_authority_hash: durable_audit_rollback_write_authority_hash
            .and_then(parse_sha256_ref),
        retained_recovery_memory_write_authority_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        recovery_memory_write_authority_hash: recovery_memory_write_authority_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        durable_audit_rollback_write_authority_id,
        durable_audit_rollback_projection_hash: durable_audit_rollback_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_durable_audit_rollback_write_authority_reference(input, require_live_retained)
}

pub(crate) fn evaluate_durable_audit_rollback_write_authority_reference(
    input: DurableAuditRollbackWriteAuthorityInput<'_>,
    require_live_retained: bool,
) -> DurableAuditRollbackWriteAuthorityReferenceCheck<'_> {
    if !input.has_reference {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "durable_audit_rollback_write_authority_absent",
            false,
        );
    }
    let Some(retained_memory_event_id) = input.retained_recovery_memory_write_authority_event_id
    else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(load_artifact_by_hash_target_binding_hash) =
        input.load_artifact_by_hash_target_binding_hash
    else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(recovery_memory_write_authority_hash) = input.recovery_memory_write_authority_hash
    else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(durable_audit_rollback_write_authority_id) =
        input.durable_audit_rollback_write_authority_id
    else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    let Some(durable_audit_rollback_projection_hash) = input.durable_audit_rollback_projection_hash
    else {
        return durable_audit_rollback_write_authority_invalid(input);
    };
    if !input.arity_valid {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "durable_audit_rollback_write_authority_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "durable_audit_rollback_write_authority_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_memory_event_id) {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_memory_write_authority_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        durable_audit_rollback_write_authority_id,
        DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID,
    ) {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "durable_audit_rollback_write_authority_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_durable_audit_rollback_write_authority_hash(
        module_evidence::DurableAuditRollbackWriteAuthorityHashInput {
            retained_recovery_memory_write_authority_event_id: retained_memory_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            restart_last_good_target_binding_hash,
            load_artifact_by_hash_target_binding_hash,
            recovery_memory_write_authority_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            durable_audit_rollback_write_authority_id:
                DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID,
            durable_audit_rollback_projection_hash,
        },
    );
    if input.durable_audit_rollback_write_authority_hash != Some(expected) {
        return durable_audit_rollback_write_authority_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_durable_audit_rollback_write_authority_hash",
            "durable_audit_rollback_write_authority_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = durable_audit_rollback_write_authority_live_chain_mismatch(&input) {
            return durable_audit_rollback_write_authority_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    durable_audit_rollback_write_authority_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "durable_audit_rollback_write_authority_valid_but_durable_writes_disabled",
        true,
    )
}

pub(crate) fn durable_audit_rollback_write_authority_invalid(
    input: DurableAuditRollbackWriteAuthorityInput<'_>,
) -> DurableAuditRollbackWriteAuthorityReferenceCheck<'_> {
    durable_audit_rollback_write_authority_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "durable_audit_rollback_write_authority_invalid_hash",
        false,
    )
}

pub(crate) fn durable_audit_rollback_write_authority_reference_check<'a>(
    input: DurableAuditRollbackWriteAuthorityInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_durable_audit_rollback_write_authority_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> DurableAuditRollbackWriteAuthorityReferenceCheck<'a> {
    DurableAuditRollbackWriteAuthorityReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        durable_audit_rollback_write_authority_hash: input
            .durable_audit_rollback_write_authority_hash,
        expected_durable_audit_rollback_write_authority_hash,
        retained_recovery_memory_write_authority_event_id: input
            .retained_recovery_memory_write_authority_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        recovery_memory_write_authority_hash: input.recovery_memory_write_authority_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        durable_audit_rollback_write_authority_id: input.durable_audit_rollback_write_authority_id,
        durable_audit_rollback_projection_hash: input.durable_audit_rollback_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn durable_audit_rollback_write_authority_live_chain_mismatch(
    input: &DurableAuditRollbackWriteAuthorityInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_recovery_memory_write_authority_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_memory_write_authority_reference()
    else {
        return Some("retained_recovery_memory_write_authority_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_recovery_memory_write_authority_event_id_stale_or_dropped");
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || input.restart_last_good_target_binding_hash
            != Some(latest_reference.restart_last_good_target_binding_hash)
        || input.load_artifact_by_hash_target_binding_hash
            != Some(latest_reference.load_artifact_by_hash_target_binding_hash)
        || input.recovery_memory_write_authority_hash
            != Some(latest_reference.recovery_memory_write_authority_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_memory_write_authority_mismatch");
    }
    None
}

pub(crate) fn durable_audit_rollback_write_authority_from_check(
    check: &DurableAuditRollbackWriteAuthorityReferenceCheck<'_>,
) -> Option<event_log::DurableAuditRollbackWriteAuthorityReference> {
    let spec = check.normalized_spec?;
    Some(event_log::DurableAuditRollbackWriteAuthorityReference {
        durable_audit_rollback_write_authority_hash: check
            .durable_audit_rollback_write_authority_hash?,
        retained_recovery_memory_write_authority_event_id: parse_current_boot_event_id(
            check.retained_recovery_memory_write_authority_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
        disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
        restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
        load_artifact_by_hash_target_binding_hash: check
            .load_artifact_by_hash_target_binding_hash?,
        recovery_memory_write_authority_hash: check.recovery_memory_write_authority_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        durable_audit_rollback_write_authority_id:
            DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID,
        durable_audit_rollback_projection_hash: check.durable_audit_rollback_projection_hash?,
    })
}

pub(crate) fn parse_recovery_service_inventory_side_effect_boundary_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let service_inventory_side_effect_boundary_hash = parts.next();
    let retained_durable_audit_rollback_write_authority_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let recovery_memory_write_authority_hash = parts.next();
    let durable_audit_rollback_write_authority_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let service_inventory_side_effect_boundary_id = parts.next();
    let service_inventory_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryServiceInventorySideEffectBoundaryInput {
        has_reference: service_inventory_side_effect_boundary_hash.is_some(),
        arity_valid: service_inventory_side_effect_boundary_hash.is_some()
            && retained_durable_audit_rollback_write_authority_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && load_artifact_by_hash_target_binding_hash.is_some()
            && recovery_memory_write_authority_hash.is_some()
            && durable_audit_rollback_write_authority_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && service_inventory_side_effect_boundary_id.is_some()
            && service_inventory_projection_hash.is_some()
            && extra.is_none(),
        scope,
        service_inventory_side_effect_boundary_hash: service_inventory_side_effect_boundary_hash
            .and_then(parse_sha256_ref),
        retained_durable_audit_rollback_write_authority_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        recovery_memory_write_authority_hash: recovery_memory_write_authority_hash
            .and_then(parse_sha256_ref),
        durable_audit_rollback_write_authority_hash: durable_audit_rollback_write_authority_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        service_inventory_side_effect_boundary_id,
        service_inventory_projection_hash: service_inventory_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_recovery_service_inventory_side_effect_boundary_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_service_inventory_side_effect_boundary_reference(
    input: RecoveryServiceInventorySideEffectBoundaryInput<'_>,
    require_live_retained: bool,
) -> RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_service_inventory_side_effect_boundary_absent",
            false,
        );
    }
    let Some(retained_durable_event_id) =
        input.retained_durable_audit_rollback_write_authority_event_id
    else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(load_artifact_by_hash_target_binding_hash) =
        input.load_artifact_by_hash_target_binding_hash
    else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(recovery_memory_write_authority_hash) = input.recovery_memory_write_authority_hash
    else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(durable_audit_rollback_write_authority_hash) =
        input.durable_audit_rollback_write_authority_hash
    else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(service_inventory_side_effect_boundary_id) =
        input.service_inventory_side_effect_boundary_id
    else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    let Some(service_inventory_projection_hash) = input.service_inventory_projection_hash else {
        return recovery_service_inventory_side_effect_boundary_invalid(input);
    };
    if !input.arity_valid {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_service_inventory_side_effect_boundary_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_service_inventory_side_effect_boundary_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_durable_event_id) {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_durable_audit_rollback_write_authority_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        service_inventory_side_effect_boundary_id,
        RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID,
    ) {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_service_inventory_side_effect_boundary_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_service_inventory_side_effect_boundary_hash(
        module_evidence::RecoveryServiceInventorySideEffectBoundaryHashInput {
            retained_durable_audit_rollback_write_authority_event_id: retained_durable_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            restart_last_good_target_binding_hash,
            load_artifact_by_hash_target_binding_hash,
            recovery_memory_write_authority_hash,
            durable_audit_rollback_write_authority_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            service_inventory_side_effect_boundary_id:
                RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID,
            service_inventory_projection_hash,
        },
    );
    if input.service_inventory_side_effect_boundary_hash != Some(expected) {
        return recovery_service_inventory_side_effect_boundary_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_recovery_service_inventory_side_effect_boundary_hash",
            "recovery_service_inventory_side_effect_boundary_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) =
            recovery_service_inventory_side_effect_boundary_live_chain_mismatch(&input)
        {
            return recovery_service_inventory_side_effect_boundary_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_service_inventory_side_effect_boundary_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_service_inventory_side_effect_boundary_valid_but_service_inventory_unchanged",
        true,
    )
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_invalid(
    input: RecoveryServiceInventorySideEffectBoundaryInput<'_>,
) -> RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'_> {
    recovery_service_inventory_side_effect_boundary_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_service_inventory_side_effect_boundary_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_reference_check<'a>(
    input: RecoveryServiceInventorySideEffectBoundaryInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_service_inventory_side_effect_boundary_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'a> {
    RecoveryServiceInventorySideEffectBoundaryReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        service_inventory_side_effect_boundary_hash: input
            .service_inventory_side_effect_boundary_hash,
        expected_service_inventory_side_effect_boundary_hash,
        retained_durable_audit_rollback_write_authority_event_id: input
            .retained_durable_audit_rollback_write_authority_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        recovery_memory_write_authority_hash: input.recovery_memory_write_authority_hash,
        durable_audit_rollback_write_authority_hash: input
            .durable_audit_rollback_write_authority_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        service_inventory_side_effect_boundary_id: input.service_inventory_side_effect_boundary_id,
        service_inventory_projection_hash: input.service_inventory_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_live_chain_mismatch(
    input: &RecoveryServiceInventorySideEffectBoundaryInput<'_>,
) -> Option<&'static str> {
    let retained_event_id = parse_current_boot_event_id(
        input.retained_durable_audit_rollback_write_authority_event_id?,
    )?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_durable_audit_rollback_write_authority_reference()
    else {
        return Some("retained_durable_audit_rollback_write_authority_missing");
    };
    if latest_event_id != retained_event_id {
        return Some("retained_durable_audit_rollback_write_authority_event_id_stale_or_dropped");
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || input.restart_last_good_target_binding_hash
            != Some(latest_reference.restart_last_good_target_binding_hash)
        || input.load_artifact_by_hash_target_binding_hash
            != Some(latest_reference.load_artifact_by_hash_target_binding_hash)
        || input.recovery_memory_write_authority_hash
            != Some(latest_reference.recovery_memory_write_authority_hash)
        || input.durable_audit_rollback_write_authority_hash
            != Some(latest_reference.durable_audit_rollback_write_authority_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("durable_audit_rollback_write_authority_mismatch");
    }
    None
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_from_check(
    check: &RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'_>,
) -> Option<event_log::RecoveryServiceInventorySideEffectBoundaryReference> {
    let spec = check.normalized_spec?;
    Some(
        event_log::RecoveryServiceInventorySideEffectBoundaryReference {
            service_inventory_side_effect_boundary_hash: check
                .service_inventory_side_effect_boundary_hash?,
            retained_durable_audit_rollback_write_authority_event_id: parse_current_boot_event_id(
                check.retained_durable_audit_rollback_write_authority_event_id?,
            )?,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash: check.argument_hash?,
            target_locator: check.target_locator_value?,
            command_envelope_reference_hash: check.command_envelope_reference_hash?,
            command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
            handler_binding_hash: check.handler_binding_hash?,
            status_read_handler_hash: check.status_read_handler_hash?,
            rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
            rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
            disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
            restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
            load_artifact_by_hash_target_binding_hash: check
                .load_artifact_by_hash_target_binding_hash?,
            recovery_memory_write_authority_hash: check.recovery_memory_write_authority_hash?,
            durable_audit_rollback_write_authority_hash: check
                .durable_audit_rollback_write_authority_hash?,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            service_inventory_side_effect_boundary_id:
                RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID,
            service_inventory_projection_hash: check.service_inventory_projection_hash?,
        },
    )
}

pub(crate) fn parse_recovery_lifeline_command_dispatch_behavior_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let command_dispatch_behavior_hash = parts.next();
    let retained_service_inventory_side_effect_boundary_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let recovery_memory_write_authority_hash = parts.next();
    let durable_audit_rollback_write_authority_hash = parts.next();
    let service_inventory_side_effect_boundary_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let command_dispatch_behavior_id = parts.next();
    let command_dispatch_behavior_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineCommandDispatchBehaviorInput {
        has_reference: command_dispatch_behavior_hash.is_some(),
        arity_valid: command_dispatch_behavior_hash.is_some()
            && retained_service_inventory_side_effect_boundary_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && load_artifact_by_hash_target_binding_hash.is_some()
            && recovery_memory_write_authority_hash.is_some()
            && durable_audit_rollback_write_authority_hash.is_some()
            && service_inventory_side_effect_boundary_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && command_dispatch_behavior_id.is_some()
            && command_dispatch_behavior_projection_hash.is_some()
            && extra.is_none(),
        scope,
        command_dispatch_behavior_hash: command_dispatch_behavior_hash.and_then(parse_sha256_ref),
        retained_service_inventory_side_effect_boundary_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        recovery_memory_write_authority_hash: recovery_memory_write_authority_hash
            .and_then(parse_sha256_ref),
        durable_audit_rollback_write_authority_hash: durable_audit_rollback_write_authority_hash
            .and_then(parse_sha256_ref),
        service_inventory_side_effect_boundary_hash: service_inventory_side_effect_boundary_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        command_dispatch_behavior_id,
        command_dispatch_behavior_projection_hash: command_dispatch_behavior_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_command_dispatch_behavior_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_command_dispatch_behavior_reference(
    input: RecoveryLifelineCommandDispatchBehaviorInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_lifeline_command_dispatch_behavior_absent",
            false,
        );
    }
    let Some(retained_service_event_id) =
        input.retained_service_inventory_side_effect_boundary_event_id
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(load_artifact_by_hash_target_binding_hash) =
        input.load_artifact_by_hash_target_binding_hash
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(recovery_memory_write_authority_hash) = input.recovery_memory_write_authority_hash
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(durable_audit_rollback_write_authority_hash) =
        input.durable_audit_rollback_write_authority_hash
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(service_inventory_side_effect_boundary_hash) =
        input.service_inventory_side_effect_boundary_hash
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(command_dispatch_behavior_id) = input.command_dispatch_behavior_id else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    let Some(command_dispatch_behavior_projection_hash) =
        input.command_dispatch_behavior_projection_hash
    else {
        return recovery_lifeline_command_dispatch_behavior_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_dispatch_behavior_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_dispatch_behavior_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_service_event_id) {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_service_inventory_side_effect_boundary_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_behavior_id,
        RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_behavior_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_lifeline_command_dispatch_behavior_hash(
        module_evidence::RecoveryLifelineCommandDispatchBehaviorHashInput {
            retained_service_inventory_side_effect_boundary_event_id: retained_service_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            restart_last_good_target_binding_hash,
            load_artifact_by_hash_target_binding_hash,
            recovery_memory_write_authority_hash,
            durable_audit_rollback_write_authority_hash,
            service_inventory_side_effect_boundary_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            command_dispatch_behavior_id: RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID,
            command_dispatch_behavior_projection_hash,
        },
    );
    if input.command_dispatch_behavior_hash != Some(expected) {
        return recovery_lifeline_command_dispatch_behavior_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_recovery_lifeline_command_dispatch_behavior_hash",
            "recovery_lifeline_command_dispatch_behavior_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) =
            recovery_lifeline_command_dispatch_behavior_live_chain_mismatch(&input)
        {
            return recovery_lifeline_command_dispatch_behavior_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_command_dispatch_behavior_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_dispatch_behavior_valid_but_execution_disabled",
        true,
    )
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_invalid(
    input: RecoveryLifelineCommandDispatchBehaviorInput<'_>,
) -> RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_> {
    recovery_lifeline_command_dispatch_behavior_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_lifeline_command_dispatch_behavior_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_reference_check<'a>(
    input: RecoveryLifelineCommandDispatchBehaviorInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_command_dispatch_behavior_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'a> {
    RecoveryLifelineCommandDispatchBehaviorReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        command_dispatch_behavior_hash: input.command_dispatch_behavior_hash,
        expected_command_dispatch_behavior_hash,
        retained_service_inventory_side_effect_boundary_event_id: input
            .retained_service_inventory_side_effect_boundary_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        recovery_memory_write_authority_hash: input.recovery_memory_write_authority_hash,
        durable_audit_rollback_write_authority_hash: input
            .durable_audit_rollback_write_authority_hash,
        service_inventory_side_effect_boundary_hash: input
            .service_inventory_side_effect_boundary_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        command_dispatch_behavior_id: input.command_dispatch_behavior_id,
        command_dispatch_behavior_projection_hash: input.command_dispatch_behavior_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_live_chain_mismatch(
    input: &RecoveryLifelineCommandDispatchBehaviorInput<'_>,
) -> Option<&'static str> {
    let retained_event_id = parse_current_boot_event_id(
        input.retained_service_inventory_side_effect_boundary_event_id?,
    )?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_service_inventory_side_effect_boundary_reference()
    else {
        return Some("retained_recovery_service_inventory_side_effect_boundary_missing");
    };
    if latest_event_id != retained_event_id {
        return Some(
            "retained_recovery_service_inventory_side_effect_boundary_event_id_stale_or_dropped",
        );
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || input.restart_last_good_target_binding_hash
            != Some(latest_reference.restart_last_good_target_binding_hash)
        || input.load_artifact_by_hash_target_binding_hash
            != Some(latest_reference.load_artifact_by_hash_target_binding_hash)
        || input.recovery_memory_write_authority_hash
            != Some(latest_reference.recovery_memory_write_authority_hash)
        || input.durable_audit_rollback_write_authority_hash
            != Some(latest_reference.durable_audit_rollback_write_authority_hash)
        || input.service_inventory_side_effect_boundary_hash
            != Some(latest_reference.service_inventory_side_effect_boundary_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_service_inventory_side_effect_boundary_mismatch");
    }
    None
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_from_check(
    check: &RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineCommandDispatchBehaviorReference> {
    let spec = check.normalized_spec?;
    Some(
        event_log::RecoveryLifelineCommandDispatchBehaviorReference {
            command_dispatch_behavior_hash: check.command_dispatch_behavior_hash?,
            retained_service_inventory_side_effect_boundary_event_id: parse_current_boot_event_id(
                check.retained_service_inventory_side_effect_boundary_event_id?,
            )?,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash: check.argument_hash?,
            target_locator: check.target_locator_value?,
            command_envelope_reference_hash: check.command_envelope_reference_hash?,
            command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
            handler_binding_hash: check.handler_binding_hash?,
            status_read_handler_hash: check.status_read_handler_hash?,
            rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
            rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
            disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
            restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
            load_artifact_by_hash_target_binding_hash: check
                .load_artifact_by_hash_target_binding_hash?,
            recovery_memory_write_authority_hash: check.recovery_memory_write_authority_hash?,
            durable_audit_rollback_write_authority_hash: check
                .durable_audit_rollback_write_authority_hash?,
            service_inventory_side_effect_boundary_hash: check
                .service_inventory_side_effect_boundary_hash?,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            command_dispatch_behavior_id: RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID,
            command_dispatch_behavior_projection_hash: check
                .command_dispatch_behavior_projection_hash?,
        },
    )
}

pub(crate) fn parse_recovery_lifeline_command_executor_capability_table_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let executor_capability_table_hash = parts.next();
    let retained_command_dispatch_behavior_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let recovery_memory_write_authority_hash = parts.next();
    let durable_audit_rollback_write_authority_hash = parts.next();
    let service_inventory_side_effect_boundary_hash = parts.next();
    let command_dispatch_behavior_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let executor_capability_table_id = parts.next();
    let executor_capability_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineCommandExecutorCapabilityTableInput {
        has_reference: executor_capability_table_hash.is_some(),
        arity_valid: executor_capability_table_hash.is_some()
            && retained_command_dispatch_behavior_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && load_artifact_by_hash_target_binding_hash.is_some()
            && recovery_memory_write_authority_hash.is_some()
            && durable_audit_rollback_write_authority_hash.is_some()
            && service_inventory_side_effect_boundary_hash.is_some()
            && command_dispatch_behavior_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && executor_capability_table_id.is_some()
            && executor_capability_projection_hash.is_some()
            && extra.is_none(),
        scope,
        executor_capability_table_hash: executor_capability_table_hash.and_then(parse_sha256_ref),
        retained_command_dispatch_behavior_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        recovery_memory_write_authority_hash: recovery_memory_write_authority_hash
            .and_then(parse_sha256_ref),
        durable_audit_rollback_write_authority_hash: durable_audit_rollback_write_authority_hash
            .and_then(parse_sha256_ref),
        service_inventory_side_effect_boundary_hash: service_inventory_side_effect_boundary_hash
            .and_then(parse_sha256_ref),
        command_dispatch_behavior_hash: command_dispatch_behavior_hash.and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        executor_capability_table_id,
        executor_capability_projection_hash: executor_capability_projection_hash
            .and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_command_executor_capability_table_reference(
        input,
        require_live_retained,
    )
}

pub(crate) fn evaluate_recovery_lifeline_command_executor_capability_table_reference(
    input: RecoveryLifelineCommandExecutorCapabilityTableInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_lifeline_command_executor_capability_table_absent",
            false,
        );
    }
    let Some(retained_behavior_event_id) = input.retained_command_dispatch_behavior_event_id else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(load_artifact_by_hash_target_binding_hash) =
        input.load_artifact_by_hash_target_binding_hash
    else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(recovery_memory_write_authority_hash) = input.recovery_memory_write_authority_hash
    else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(durable_audit_rollback_write_authority_hash) =
        input.durable_audit_rollback_write_authority_hash
    else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(service_inventory_side_effect_boundary_hash) =
        input.service_inventory_side_effect_boundary_hash
    else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(command_dispatch_behavior_hash) = input.command_dispatch_behavior_hash else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(executor_capability_table_id) = input.executor_capability_table_id else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    let Some(executor_capability_projection_hash) = input.executor_capability_projection_hash
    else {
        return recovery_lifeline_command_executor_capability_table_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_executor_capability_table_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_executor_capability_table_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_behavior_event_id) {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_command_dispatch_behavior_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        executor_capability_table_id,
        RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_executor_capability_table_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected =
        module_evidence::computed_recovery_lifeline_command_executor_capability_table_hash(
            module_evidence::RecoveryLifelineCommandExecutorCapabilityTableHashInput {
                retained_command_dispatch_behavior_event_id: retained_behavior_event_id,
                command_id: spec.command_id,
                argument_schema: spec.argument_schema,
                argument_hash,
                target_locator,
                command_envelope_reference_hash,
                command_body_canonicalization_hash,
                handler_binding_hash,
                status_read_handler_hash,
                rollback_preview_authorization_hash,
                rollback_apply_authorization_hash,
                disable_module_target_binding_hash,
                restart_last_good_target_binding_hash,
                load_artifact_by_hash_target_binding_hash,
                recovery_memory_write_authority_hash,
                durable_audit_rollback_write_authority_hash,
                service_inventory_side_effect_boundary_hash,
                command_dispatch_behavior_hash,
                command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
                executor_capability_table_id:
                    RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID,
                executor_capability_projection_hash,
            },
        );
    if input.executor_capability_table_hash != Some(expected) {
        return recovery_lifeline_command_executor_capability_table_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_recovery_lifeline_command_executor_capability_table_hash",
            "recovery_lifeline_command_executor_capability_table_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) =
            recovery_lifeline_command_executor_capability_table_live_chain_mismatch(&input)
        {
            return recovery_lifeline_command_executor_capability_table_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_command_executor_capability_table_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_executor_capability_table_valid_but_execution_disabled",
        true,
    )
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_invalid(
    input: RecoveryLifelineCommandExecutorCapabilityTableInput<'_>,
) -> RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_> {
    recovery_lifeline_command_executor_capability_table_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_lifeline_command_executor_capability_table_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_reference_check<'a>(
    input: RecoveryLifelineCommandExecutorCapabilityTableInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_executor_capability_table_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'a> {
    RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        executor_capability_table_hash: input.executor_capability_table_hash,
        expected_executor_capability_table_hash,
        retained_command_dispatch_behavior_event_id: input
            .retained_command_dispatch_behavior_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        recovery_memory_write_authority_hash: input.recovery_memory_write_authority_hash,
        durable_audit_rollback_write_authority_hash: input
            .durable_audit_rollback_write_authority_hash,
        service_inventory_side_effect_boundary_hash: input
            .service_inventory_side_effect_boundary_hash,
        command_dispatch_behavior_hash: input.command_dispatch_behavior_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        executor_capability_table_id: input.executor_capability_table_id,
        executor_capability_projection_hash: input.executor_capability_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_live_chain_mismatch(
    input: &RecoveryLifelineCommandExecutorCapabilityTableInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_command_dispatch_behavior_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_command_dispatch_behavior_reference()
    else {
        return Some("retained_recovery_lifeline_command_dispatch_behavior_missing");
    };
    if latest_event_id != retained_event_id {
        return Some(
            "retained_recovery_lifeline_command_dispatch_behavior_event_id_stale_or_dropped",
        );
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || input.restart_last_good_target_binding_hash
            != Some(latest_reference.restart_last_good_target_binding_hash)
        || input.load_artifact_by_hash_target_binding_hash
            != Some(latest_reference.load_artifact_by_hash_target_binding_hash)
        || input.recovery_memory_write_authority_hash
            != Some(latest_reference.recovery_memory_write_authority_hash)
        || input.durable_audit_rollback_write_authority_hash
            != Some(latest_reference.durable_audit_rollback_write_authority_hash)
        || input.service_inventory_side_effect_boundary_hash
            != Some(latest_reference.service_inventory_side_effect_boundary_hash)
        || input.command_dispatch_behavior_hash
            != Some(latest_reference.command_dispatch_behavior_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_lifeline_command_dispatch_behavior_mismatch");
    }
    None
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_from_check(
    check: &RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineCommandExecutorCapabilityTableReference> {
    let spec = check.normalized_spec?;
    Some(
        event_log::RecoveryLifelineCommandExecutorCapabilityTableReference {
            executor_capability_table_hash: check.executor_capability_table_hash?,
            retained_command_dispatch_behavior_event_id: parse_current_boot_event_id(
                check.retained_command_dispatch_behavior_event_id?,
            )?,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash: check.argument_hash?,
            target_locator: check.target_locator_value?,
            command_envelope_reference_hash: check.command_envelope_reference_hash?,
            command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
            handler_binding_hash: check.handler_binding_hash?,
            status_read_handler_hash: check.status_read_handler_hash?,
            rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
            rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
            disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
            restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
            load_artifact_by_hash_target_binding_hash: check
                .load_artifact_by_hash_target_binding_hash?,
            recovery_memory_write_authority_hash: check.recovery_memory_write_authority_hash?,
            durable_audit_rollback_write_authority_hash: check
                .durable_audit_rollback_write_authority_hash?,
            service_inventory_side_effect_boundary_hash: check
                .service_inventory_side_effect_boundary_hash?,
            command_dispatch_behavior_hash: check.command_dispatch_behavior_hash?,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            executor_capability_table_id:
                RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID,
            executor_capability_projection_hash: check.executor_capability_projection_hash?,
        },
    )
}

pub(crate) fn parse_recovery_lifeline_command_side_effect_gate_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineCommandSideEffectGateReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let side_effect_gate_hash = parts.next();
    let retained_executor_capability_table_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let recovery_memory_write_authority_hash = parts.next();
    let durable_audit_rollback_write_authority_hash = parts.next();
    let service_inventory_side_effect_boundary_hash = parts.next();
    let command_dispatch_behavior_hash = parts.next();
    let executor_capability_table_hash = parts.next();
    let command_dispatch_boundary_id = parts.next();
    let side_effect_gate_id = parts.next();
    let side_effect_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineCommandSideEffectGateInput {
        has_reference: side_effect_gate_hash.is_some(),
        arity_valid: side_effect_gate_hash.is_some()
            && retained_executor_capability_table_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && load_artifact_by_hash_target_binding_hash.is_some()
            && recovery_memory_write_authority_hash.is_some()
            && durable_audit_rollback_write_authority_hash.is_some()
            && service_inventory_side_effect_boundary_hash.is_some()
            && command_dispatch_behavior_hash.is_some()
            && executor_capability_table_hash.is_some()
            && command_dispatch_boundary_id.is_some()
            && side_effect_gate_id.is_some()
            && side_effect_projection_hash.is_some()
            && extra.is_none(),
        scope,
        side_effect_gate_hash: side_effect_gate_hash.and_then(parse_sha256_ref),
        retained_executor_capability_table_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        recovery_memory_write_authority_hash: recovery_memory_write_authority_hash
            .and_then(parse_sha256_ref),
        durable_audit_rollback_write_authority_hash: durable_audit_rollback_write_authority_hash
            .and_then(parse_sha256_ref),
        service_inventory_side_effect_boundary_hash: service_inventory_side_effect_boundary_hash
            .and_then(parse_sha256_ref),
        command_dispatch_behavior_hash: command_dispatch_behavior_hash.and_then(parse_sha256_ref),
        executor_capability_table_hash: executor_capability_table_hash.and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        side_effect_gate_id,
        side_effect_projection_hash: side_effect_projection_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_command_side_effect_gate_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_command_side_effect_gate_reference(
    input: RecoveryLifelineCommandSideEffectGateInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineCommandSideEffectGateReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "recovery_lifeline_command_side_effect_gate_absent",
            false,
        );
    }
    let Some(retained_executor_event_id) = input.retained_executor_capability_table_event_id else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(load_artifact_by_hash_target_binding_hash) =
        input.load_artifact_by_hash_target_binding_hash
    else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(recovery_memory_write_authority_hash) = input.recovery_memory_write_authority_hash
    else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(durable_audit_rollback_write_authority_hash) =
        input.durable_audit_rollback_write_authority_hash
    else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(service_inventory_side_effect_boundary_hash) =
        input.service_inventory_side_effect_boundary_hash
    else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(command_dispatch_behavior_hash) = input.command_dispatch_behavior_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(executor_capability_table_hash) = input.executor_capability_table_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(side_effect_gate_id) = input.side_effect_gate_id else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    let Some(side_effect_projection_hash) = input.side_effect_projection_hash else {
        return recovery_lifeline_command_side_effect_gate_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_side_effect_gate_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_command_side_effect_gate_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_executor_event_id) {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_executor_capability_table_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(
        side_effect_gate_id,
        RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_side_effect_gate_id_mismatch",
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_lifeline_command_side_effect_gate_hash(
        module_evidence::RecoveryLifelineCommandSideEffectGateHashInput {
            retained_executor_capability_table_event_id: retained_executor_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            restart_last_good_target_binding_hash,
            load_artifact_by_hash_target_binding_hash,
            recovery_memory_write_authority_hash,
            durable_audit_rollback_write_authority_hash,
            service_inventory_side_effect_boundary_hash,
            command_dispatch_behavior_hash,
            executor_capability_table_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            side_effect_gate_id: RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID,
            side_effect_projection_hash,
        },
    );
    if input.side_effect_gate_hash != Some(expected) {
        return recovery_lifeline_command_side_effect_gate_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            "mismatched_recovery_lifeline_command_side_effect_gate_hash",
            "recovery_lifeline_command_side_effect_gate_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_lifeline_command_side_effect_gate_live_chain_mismatch(&input)
        {
            return recovery_lifeline_command_side_effect_gate_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_command_side_effect_gate_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_side_effect_gate_valid_but_execution_disabled",
        true,
    )
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_invalid(
    input: RecoveryLifelineCommandSideEffectGateInput<'_>,
) -> RecoveryLifelineCommandSideEffectGateReferenceCheck<'_> {
    recovery_lifeline_command_side_effect_gate_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        "recovery_lifeline_command_side_effect_gate_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_reference_check<'a>(
    input: RecoveryLifelineCommandSideEffectGateInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_side_effect_gate_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineCommandSideEffectGateReferenceCheck<'a> {
    RecoveryLifelineCommandSideEffectGateReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        side_effect_gate_hash: input.side_effect_gate_hash,
        expected_side_effect_gate_hash,
        retained_executor_capability_table_event_id: input
            .retained_executor_capability_table_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        recovery_memory_write_authority_hash: input.recovery_memory_write_authority_hash,
        durable_audit_rollback_write_authority_hash: input
            .durable_audit_rollback_write_authority_hash,
        service_inventory_side_effect_boundary_hash: input
            .service_inventory_side_effect_boundary_hash,
        command_dispatch_behavior_hash: input.command_dispatch_behavior_hash,
        executor_capability_table_hash: input.executor_capability_table_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        side_effect_gate_id: input.side_effect_gate_id,
        side_effect_projection_hash: input.side_effect_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_live_chain_mismatch(
    input: &RecoveryLifelineCommandSideEffectGateInput<'_>,
) -> Option<&'static str> {
    let retained_event_id =
        parse_current_boot_event_id(input.retained_executor_capability_table_event_id?)?;
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_command_executor_capability_table_reference()
    else {
        return Some("retained_recovery_lifeline_command_executor_capability_table_missing");
    };
    if latest_event_id != retained_event_id {
        return Some(
            "retained_recovery_lifeline_command_executor_capability_table_event_id_stale_or_dropped",
        );
    }
    if !method_eq(input.command_id?, latest_reference.command_id)
        || !method_eq(input.argument_schema?, latest_reference.argument_schema)
        || input.argument_hash != Some(latest_reference.argument_hash)
        || input.command_envelope_reference_hash
            != Some(latest_reference.command_envelope_reference_hash)
        || input.command_body_canonicalization_hash
            != Some(latest_reference.command_body_canonicalization_hash)
        || input.handler_binding_hash != Some(latest_reference.handler_binding_hash)
        || input.status_read_handler_hash != Some(latest_reference.status_read_handler_hash)
        || input.rollback_preview_authorization_hash
            != Some(latest_reference.rollback_preview_authorization_hash)
        || input.rollback_apply_authorization_hash
            != Some(latest_reference.rollback_apply_authorization_hash)
        || input.disable_module_target_binding_hash
            != Some(latest_reference.disable_module_target_binding_hash)
        || input.restart_last_good_target_binding_hash
            != Some(latest_reference.restart_last_good_target_binding_hash)
        || input.load_artifact_by_hash_target_binding_hash
            != Some(latest_reference.load_artifact_by_hash_target_binding_hash)
        || input.recovery_memory_write_authority_hash
            != Some(latest_reference.recovery_memory_write_authority_hash)
        || input.durable_audit_rollback_write_authority_hash
            != Some(latest_reference.durable_audit_rollback_write_authority_hash)
        || input.service_inventory_side_effect_boundary_hash
            != Some(latest_reference.service_inventory_side_effect_boundary_hash)
        || input.command_dispatch_behavior_hash
            != Some(latest_reference.command_dispatch_behavior_hash)
        || input.executor_capability_table_hash
            != Some(latest_reference.executor_capability_table_hash)
        || !method_eq(
            input.target_locator?,
            latest_reference.target_locator.as_str(),
        )
        || !method_eq(
            input.command_dispatch_boundary_id?,
            latest_reference.command_dispatch_boundary_id,
        )
    {
        return Some("recovery_lifeline_command_executor_capability_table_mismatch");
    }
    None
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_from_check(
    check: &RecoveryLifelineCommandSideEffectGateReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineCommandSideEffectGateReference> {
    let spec = check.normalized_spec?;
    Some(event_log::RecoveryLifelineCommandSideEffectGateReference {
        side_effect_gate_hash: check.side_effect_gate_hash?,
        retained_executor_capability_table_event_id: parse_current_boot_event_id(
            check.retained_executor_capability_table_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
        disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
        restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
        load_artifact_by_hash_target_binding_hash: check
            .load_artifact_by_hash_target_binding_hash?,
        recovery_memory_write_authority_hash: check.recovery_memory_write_authority_hash?,
        durable_audit_rollback_write_authority_hash: check
            .durable_audit_rollback_write_authority_hash?,
        service_inventory_side_effect_boundary_hash: check
            .service_inventory_side_effect_boundary_hash?,
        command_dispatch_behavior_hash: check.command_dispatch_behavior_hash?,
        executor_capability_table_hash: check.executor_capability_table_hash?,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        side_effect_gate_id: RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID,
        side_effect_projection_hash: check.side_effect_projection_hash?,
    })
}
