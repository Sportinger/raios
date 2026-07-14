//! State owner for the signed current-boot personal-shell proof surface.
//!
//! The service adapter is the sole Wasm boundary. This module only retains a
//! frame it accepted, owns personal focus, and returns to Genesis on any
//! rejected invocation or core secure-attention signal.

use alloc::vec::Vec;

use raios_core::personal_shell_abi::{
    PersonalShellContext, PersonalShellInput, SanitizedInputEvent,
};
use raios_core::{
    ui_frame::Viewport,
    ui_program::{DispatchOutcome, Program, ProgramState, RuntimeError, UiInput},
};

use crate::personal_shell_service::{
    self, PersonalShellAttempt, PersonalShellAttemptFrame, PersonalShellHealth,
    PersonalShellProofMode, PROOF_TEST_KEY_FUEL_EXHAUSTION, PROOF_TEST_KEY_TRAP,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonalSurfaceHealth {
    Genesis,
    Active,
    GenesisFallback { reason: &'static str },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonalSurfaceRoute {
    Ignored,
    Entered,
    FrameUpdated,
    ExitedToGenesis,
    GenesisFallback {
        reason: &'static str,
        fuel_used: u64,
    },
}

/// The sole owner of a validated personal frame and personal input focus.
pub(crate) struct PersonalSurface {
    frame: Option<Vec<u8>>,
    health: PersonalSurfaceHealth,
    personal_focus: bool,
    next_invocation_id: u32,
    program: Option<Program>,
    program_state: Option<ProgramState>,
    shift_active: bool,
    pointer_blocked_until_release: bool,
}

impl PersonalSurface {
    pub(crate) const fn new() -> Self {
        Self {
            frame: None,
            health: PersonalSurfaceHealth::Genesis,
            personal_focus: false,
            next_invocation_id: 1,
            program: None,
            program_state: None,
            shift_active: false,
            pointer_blocked_until_release: false,
        }
    }

    pub(crate) fn health(&self) -> PersonalSurfaceHealth {
        self.health
    }

    pub(crate) fn has_personal_focus(&self) -> bool {
        self.personal_focus
    }

    /// Bytes returned here were accepted by `personal_shell_service`; callers
    /// present them only while `has_personal_focus` remains true.
    pub(crate) fn frame(&self) -> Option<&[u8]> {
        self.frame.as_deref()
    }

    /// Starts the non-default signed proof with an empty, typed input packet.
    pub(crate) fn enter(&mut self, context: PersonalShellContext) -> PersonalSurfaceRoute {
        self.program = None;
        self.program_state = None;
        self.shift_active = false;
        self.pointer_blocked_until_release = false;
        self.invoke(context, None, PersonalSurfaceRoute::Entered)
    }

    /// Starts one validated current-boot program through the existing signed
    /// shell artifact. Its state stays here across fresh Wasmi invocations.
    pub(crate) fn enter_program(
        &mut self,
        context: PersonalShellContext,
        program: Program,
    ) -> PersonalSurfaceRoute {
        self.program_state = Some(program.initial_state());
        self.program = Some(program);
        self.shift_active = false;
        self.pointer_blocked_until_release = true;
        self.invoke_program(context, None, PersonalSurfaceRoute::Entered)
    }

    /// Starts only a named built-in proof case. The trap/fuel variants are
    /// fixed test infrastructure, not caller-provided guest bytes.
    pub(crate) fn enter_mode(
        &mut self,
        context: PersonalShellContext,
        mode: PersonalShellProofMode,
    ) -> PersonalSurfaceRoute {
        let event = match mode {
            PersonalShellProofMode::Normal => return self.enter(context),
            PersonalShellProofMode::Trap => proof_key(PROOF_TEST_KEY_TRAP),
            PersonalShellProofMode::FuelExhaustion => proof_key(PROOF_TEST_KEY_FUEL_EXHAUSTION),
        };
        self.program = None;
        self.program_state = None;
        self.pointer_blocked_until_release = false;
        self.invoke(context, Some(event), PersonalSurfaceRoute::Entered)
    }

    /// Routes exactly one already-sanitized event while the personal surface owns focus.
    pub(crate) fn route_sanitized_event(
        &mut self,
        context: PersonalShellContext,
        event: SanitizedInputEvent,
    ) -> PersonalSurfaceRoute {
        if !self.personal_focus {
            return PersonalSurfaceRoute::Ignored;
        }
        if self.program.is_some() {
            let Some(input) = self.program_input(event) else {
                return PersonalSurfaceRoute::Ignored;
            };
            return self.invoke_program(context, Some(input), PersonalSurfaceRoute::FrameUpdated);
        }
        self.invoke(context, Some(event), PersonalSurfaceRoute::FrameUpdated)
    }

    /// Consumes core secure attention before a guest input packet can be staged.
    pub(crate) fn handle_secure_attention(&mut self) -> PersonalSurfaceRoute {
        if !self.personal_focus {
            return PersonalSurfaceRoute::Ignored;
        }
        self.exit();
        PersonalSurfaceRoute::ExitedToGenesis
    }

    pub(crate) fn exit(&mut self) {
        self.frame = None;
        self.health = PersonalSurfaceHealth::Genesis;
        self.personal_focus = false;
        self.shift_active = false;
        self.pointer_blocked_until_release = false;
        personal_shell_service::note_personal_shell_stopped("f12_exit");
    }

    fn invoke(
        &mut self,
        source: PersonalShellContext,
        event: Option<SanitizedInputEvent>,
        accepted_route: PersonalSurfaceRoute,
    ) -> PersonalSurfaceRoute {
        // A stale frame may never survive a rejected fresh invocation.
        self.frame = None;
        self.personal_focus = false;

        let context = self.next_context(source);
        let mut input = PersonalShellInput::new(context.invocation_id);
        if let Some(event) = event {
            if input.push(event).is_err() {
                return self.fallback("input_packet_limit", 0);
            }
        }

        self.apply_attempt(
            personal_shell_service::invoke_current_boot_proof(&context, &input),
            accepted_route,
        )
    }

    fn invoke_program(
        &mut self,
        source: PersonalShellContext,
        event: Option<UiInput>,
        accepted_route: PersonalSurfaceRoute,
    ) -> PersonalSurfaceRoute {
        let context = self.next_context(source);
        let input = PersonalShellInput::new(context.invocation_id);
        let Some(mut candidate_state) = self.program_state else {
            return self.fallback("program_state_missing", 0);
        };
        if let Some(event) = event {
            let Some(program) = self.program.as_ref() else {
                return self.fallback("program_missing", 0);
            };
            match program.dispatch(&mut candidate_state, event) {
                Ok(DispatchOutcome::Handled { .. }) | Ok(DispatchOutcome::Edited) => {}
                Ok(DispatchOutcome::Ignored) => return PersonalSurfaceRoute::Ignored,
                Err(error) => return self.fallback(program_error_reason(error), 0),
            }
        }

        // A stale frame may never survive a rejected fresh invocation, but an
        // ignored event above must leave the active surface untouched.
        self.frame = None;
        self.personal_focus = false;
        let attempt = {
            let Some(program) = self.program.as_ref() else {
                return self.fallback("program_missing", 0);
            };
            personal_shell_service::invoke_current_boot_program(
                context.invocation_id,
                Viewport {
                    width: context.viewport_width,
                    height: context.viewport_height,
                },
                program,
                candidate_state,
                &input,
            )
        };
        if attempt.health == PersonalShellHealth::Healthy
            && matches!(&attempt.frame, PersonalShellAttemptFrame::Accepted(_))
        {
            self.program_state = Some(candidate_state);
        }
        self.apply_attempt(attempt, accepted_route)
    }

    fn program_input(&mut self, event: SanitizedInputEvent) -> Option<UiInput> {
        use raios_core::personal_shell_abi::SanitizedInputKind;

        if self.pointer_blocked_until_release {
            match event.kind {
                SanitizedInputKind::PointerButton if !event.pressed => {
                    self.pointer_blocked_until_release = false;
                    return None;
                }
                SanitizedInputKind::PointerButton | SanitizedInputKind::PointerMove => return None,
                SanitizedInputKind::Key => {}
            }
        }

        match event.kind {
            SanitizedInputKind::Key if event.code == 42 || event.code == 54 => {
                self.shift_active = event.pressed;
                None
            }
            SanitizedInputKind::Key if event.pressed => {
                let code = match event.code {
                    14 => 8,
                    28 => 13,
                    57 => u16::from(b' '),
                    code => u16::from(crate::input::keycode_to_ascii(code, self.shift_active)?),
                };
                Some(UiInput::Key { code, modifiers: 0 })
            }
            SanitizedInputKind::PointerButton if event.pressed && event.code == 272 => {
                Some(UiInput::PointerClick {
                    x: u16::try_from(event.x).ok()?,
                    y: u16::try_from(event.y).ok()?,
                })
            }
            _ => None,
        }
    }

    fn next_context(&mut self, source: PersonalShellContext) -> PersonalShellContext {
        let invocation_id = self.next_invocation_id;
        self.next_invocation_id = self.next_invocation_id.wrapping_add(1);
        PersonalShellContext::new(
            invocation_id,
            source.viewport_width,
            source.viewport_height,
            source.service_count,
            source.problem_count,
            source.denied_capability_count,
            true,
            source.recovery_ready,
            source.active_task_id,
        )
    }

    fn apply_attempt(
        &mut self,
        attempt: PersonalShellAttempt,
        accepted_route: PersonalSurfaceRoute,
    ) -> PersonalSurfaceRoute {
        let reason = attempt.reason;
        let fuel_used = attempt.fuel_used;
        match (attempt.health, attempt.frame) {
            (PersonalShellHealth::Healthy, PersonalShellAttemptFrame::Accepted(frame)) => {
                self.frame = Some(frame);
                self.health = PersonalSurfaceHealth::Active;
                self.personal_focus = true;
                personal_shell_service::note_personal_shell_started();
                accepted_route
            }
            _ => self.fallback(reason, fuel_used),
        }
    }

    fn fallback(&mut self, reason: &'static str, fuel_used: u64) -> PersonalSurfaceRoute {
        self.frame = None;
        self.health = PersonalSurfaceHealth::GenesisFallback { reason };
        self.personal_focus = false;
        personal_shell_service::note_personal_shell_stopped(reason);
        PersonalSurfaceRoute::GenesisFallback { reason, fuel_used }
    }
}

fn program_error_reason(error: RuntimeError) -> &'static str {
    match error {
        RuntimeError::StateMismatch => "program_state_mismatch",
        RuntimeError::AmbiguousEvent => "program_event_ambiguous",
        RuntimeError::Overflow => "program_arithmetic_overflow",
        RuntimeError::DivisionByZero => "program_division_by_zero",
    }
}

fn proof_key(code: u16) -> SanitizedInputEvent {
    SanitizedInputEvent::new(
        raios_core::personal_shell_abi::SanitizedInputKind::Key,
        true,
        false,
        code,
        0,
        0,
        0,
        0,
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use raios_core::{personal_shell_abi::SanitizedInputKind, ui_program::calculator_program};

    #[test]
    fn secure_attention_exits_without_staging_a_guest_event() {
        let mut surface = PersonalSurface::new();
        surface.personal_focus = true;
        surface.health = PersonalSurfaceHealth::Active;
        surface.frame = Some(alloc::vec![1]);

        assert_eq!(
            surface.handle_secure_attention(),
            PersonalSurfaceRoute::ExitedToGenesis
        );
        assert!(!surface.has_personal_focus());
        assert!(surface.frame().is_none());
        assert_eq!(surface.health(), PersonalSurfaceHealth::Genesis);
    }

    #[test]
    fn trap_and_fuel_rejections_fall_back_to_genesis() {
        for (reason, fuel_used) in [("trap", 17), ("fuel_exhausted", 250_000)] {
            let mut surface = PersonalSurface::new();
            surface.personal_focus = true;
            surface.frame = Some(alloc::vec![1]);

            assert_eq!(
                surface.apply_attempt(rejected(reason, fuel_used), PersonalSurfaceRoute::Entered),
                PersonalSurfaceRoute::GenesisFallback { reason, fuel_used }
            );
            assert!(!surface.has_personal_focus());
            assert!(surface.frame().is_none());
            assert_eq!(
                surface.health(),
                PersonalSurfaceHealth::GenesisFallback { reason }
            );
        }
    }

    #[test]
    fn calculator_keys_and_local_pointer_map_to_program_inputs() {
        assert_eq!(crate::input::keycode_to_ascii(2, false), Some(b'1'));
        assert_eq!(crate::input::keycode_to_ascii(11, false), Some(b'0'));
        assert_eq!(crate::input::keycode_to_ascii(13, true), Some(b'+'));
        assert_eq!(crate::input::keycode_to_ascii(9, true), Some(b'*'));
        assert_eq!(crate::input::keycode_to_ascii(46, false), Some(b'c'));

        let mut surface = PersonalSurface::new();
        assert_eq!(
            surface.program_input(SanitizedInputEvent::new(
                SanitizedInputKind::Key,
                true,
                false,
                28,
                0,
                0,
                0,
                0,
                0,
            )),
            Some(UiInput::Key {
                code: 13,
                modifiers: 0
            })
        );

        assert_eq!(
            surface.program_input(SanitizedInputEvent::new(
                SanitizedInputKind::PointerButton,
                true,
                false,
                272,
                81,
                105,
                0,
                0,
                0,
            )),
            Some(UiInput::PointerClick { x: 81, y: 105 })
        );

        let program = calculator_program();
        let mut state = program.initial_state();
        program
            .dispatch(&mut state, UiInput::PointerClick { x: 81, y: 105 })
            .unwrap();
        assert_eq!(state.values()[0], 8);
    }

    #[test]
    fn physical_activation_click_is_ignored_until_pointer_release() {
        let mut surface = PersonalSurface::new();
        surface.pointer_blocked_until_release = true;
        let pointer = |pressed| {
            SanitizedInputEvent::new(
                SanitizedInputKind::PointerButton,
                pressed,
                false,
                272,
                81,
                105,
                0,
                0,
                0,
            )
        };

        assert_eq!(surface.program_input(pointer(true)), None);
        assert_eq!(surface.program_input(pointer(false)), None);
        assert_eq!(
            surface.program_input(pointer(true)),
            Some(UiInput::PointerClick { x: 81, y: 105 })
        );
    }

    #[test]
    fn ignored_program_input_keeps_the_active_frame_and_focus() {
        let program = calculator_program();
        let mut surface = PersonalSurface::new();
        surface.program_state = Some(program.initial_state());
        surface.program = Some(program);
        surface.personal_focus = true;
        surface.health = PersonalSurfaceHealth::Active;
        surface.frame = Some(alloc::vec![1]);

        assert_eq!(
            surface.invoke_program(
                PersonalShellContext::new(0, 640, 480, 0, 0, 0, true, true, 0),
                Some(UiInput::PointerClick { x: 500, y: 400 }),
                PersonalSurfaceRoute::FrameUpdated,
            ),
            PersonalSurfaceRoute::Ignored
        );
        assert!(surface.has_personal_focus());
        assert_eq!(surface.frame(), Some(&[1][..]));
    }

    fn rejected(reason: &'static str, fuel_used: u64) -> PersonalShellAttempt {
        PersonalShellAttempt {
            frame: PersonalShellAttemptFrame::Rejected,
            health: PersonalShellHealth::Unhealthy,
            reason,
            fuel_used,
        }
    }
}
