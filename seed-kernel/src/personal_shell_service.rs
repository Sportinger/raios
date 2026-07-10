//! Stateless adapter for the one checked-in `svc.user.shell` proof invocation.
//!
//! It owns V1 packet staging only. Wasm linkage, frame validation, and rendering
//! remain in their respective runtime, core-model, and ShellHost boundaries.

use alloc::vec::Vec;
use raios_core::{
    personal_shell_abi::{
        validate_invocation_pair, PersonalShellContext, PersonalShellInput, CONTEXT_LEN,
        MAX_INPUT_LEN,
    },
    ui_frame::Viewport,
};

use crate::wasm_runtime::{self, PersonalShellFrameResult, PersonalShellRuntimeResult};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PersonalShellAttemptFrame {
    Accepted(Vec<u8>),
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonalShellHealth {
    Healthy,
    Unhealthy,
}

pub(crate) struct PersonalShellAttempt {
    pub(crate) frame: PersonalShellAttemptFrame,
    pub(crate) health: PersonalShellHealth,
    pub(crate) reason: &'static str,
    pub(crate) fuel_used: u64,
}

/// Stages one typed, bounded context/input pair and invokes a fresh proof guest.
/// This adapter owns no retained Wasm state and no framebuffer access.
pub(crate) fn invoke_current_boot_proof(
    context: &PersonalShellContext,
    input: &PersonalShellInput,
) -> PersonalShellAttempt {
    if validate_invocation_pair(context, input).is_err() {
        return rejected("invocation_id_mismatch");
    }
    if context.viewport_width == 0 || context.viewport_height == 0 {
        return rejected("viewport_invalid");
    }

    let context_bytes = context.encode();
    let input_bytes = input.encode();
    if context_bytes.len() != CONTEXT_LEN || input_bytes.as_bytes().len() > MAX_INPUT_LEN {
        return rejected("invocation_packet_out_of_bounds");
    }
    if PersonalShellContext::decode(&context_bytes).is_err()
        || PersonalShellInput::decode(input_bytes.as_bytes()).is_err()
    {
        return rejected("invocation_packet_invalid");
    }

    from_runtime(wasm_runtime::run_personal_shell_proof(
        &context_bytes,
        input_bytes.as_bytes(),
        Viewport {
            width: context.viewport_width,
            height: context.viewport_height,
        },
    ))
}

fn from_runtime(runtime: PersonalShellRuntimeResult) -> PersonalShellAttempt {
    match runtime.frame {
        PersonalShellFrameResult::Accepted(frame) if runtime.run_outcome == "success" => {
            PersonalShellAttempt {
                frame: PersonalShellAttemptFrame::Accepted(frame),
                health: PersonalShellHealth::Healthy,
                reason: "accepted",
                fuel_used: runtime.fuel_used,
            }
        }
        PersonalShellFrameResult::Accepted(_) => {
            rejected_with_fuel("runtime_result_inconsistent", runtime.fuel_used)
        }
        PersonalShellFrameResult::Rejected => {
            rejected_with_fuel(runtime.run_outcome, runtime.fuel_used)
        }
    }
}

fn rejected(reason: &'static str) -> PersonalShellAttempt {
    rejected_with_fuel(reason, 0)
}

fn rejected_with_fuel(reason: &'static str, fuel_used: u64) -> PersonalShellAttempt {
    PersonalShellAttempt {
        frame: PersonalShellAttemptFrame::Rejected,
        health: PersonalShellHealth::Unhealthy,
        reason,
        fuel_used,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatched_packets_are_denied_before_wasm() {
        let context = PersonalShellContext::new(1, 640, 480, 0, 0, 0, true, true, 0);
        let input = PersonalShellInput::new(2);

        let attempt = invoke_current_boot_proof(&context, &input);

        assert_eq!(attempt.frame, PersonalShellAttemptFrame::Rejected);
        assert_eq!(attempt.health, PersonalShellHealth::Unhealthy);
        assert_eq!(attempt.reason, "invocation_id_mismatch");
        assert_eq!(attempt.fuel_used, 0);
    }

    #[test]
    fn fuel_budget_stays_frozen_at_the_runtime_boundary() {
        assert_eq!(
            crate::wasm_runtime::PERSONAL_SHELL_WASM_FUEL_BUDGET,
            250_000
        );
    }
}
