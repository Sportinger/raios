//! State owner for the signed current-boot personal-shell proof surface.
//!
//! The service adapter is the sole Wasm boundary. This module only retains a
//! frame it accepted, owns personal focus, and returns to Genesis on any
//! rejected invocation or core secure-attention signal.

use alloc::vec::Vec;

use raios_core::personal_shell_abi::{
    PersonalShellContext, PersonalShellInput, SanitizedInputEvent,
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
}

impl PersonalSurface {
    pub(crate) const fn new() -> Self {
        Self {
            frame: None,
            health: PersonalSurfaceHealth::Genesis,
            personal_focus: false,
            next_invocation_id: 1,
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
        self.invoke(context, None, PersonalSurfaceRoute::Entered)
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

    fn rejected(reason: &'static str, fuel_used: u64) -> PersonalShellAttempt {
        PersonalShellAttempt {
            frame: PersonalShellAttemptFrame::Rejected,
            health: PersonalShellHealth::Unhealthy,
            reason,
            fuel_used,
        }
    }
}
