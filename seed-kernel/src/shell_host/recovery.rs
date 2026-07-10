//! Read-only Recovery projection for the core-owned Genesis shell.
//!
//! This module deliberately has no serial parsing and no recovery authority. It
//! reads the same typed sources as `recovery_lifeline`, caches one bounded BOOTCTL
//! read on entry/refresh, and returns typed action selections for the shared
//! recovery executor to evaluate. A caller must never treat a selection as a
//! successful recovery action.

use raios_core::boot_control::{BootPosture, BootSlotId};
use raios_core::recovery_lifeline_table as lifeline;

use crate::framebuffer::FramebufferSurface;
use crate::system_status::{RuntimeStatus, SystemSnapshot};
use crate::{boot_control, echo_service, provider, service_inventory, text};

use super::genesis::{
    draw_button, draw_panel, draw_truncated_text, point_in, rect_from_layout, LogicalRect,
    APP_AMBER, APP_BLUE, APP_GREEN, TEXT_MUTED,
};

pub(crate) const RECOVERY_ACTION_COUNT: usize = 4;
const FULL_RECOVERY_CONTEXT_HEIGHT: usize = 260;

/// A core-owned cache. `refresh` is the only operation that performs the bounded
/// BOOTCTL read, so regular framebuffer redraws never touch AHCI.
pub(crate) struct RecoveryView {
    snapshot: Option<RecoverySnapshot>,
}

impl RecoveryView {
    pub(crate) const fn new() -> Self {
        Self { snapshot: None }
    }

    /// Call when Recovery opens, on explicit refresh, or after a relevant recovery
    /// executor result. It does not invoke Wasm, the provider, or a serial parser.
    pub(crate) fn refresh(&mut self, runtime: RuntimeStatus) {
        self.snapshot = Some(RecoverySnapshot::collect(runtime));
    }

    pub(crate) fn snapshot(&self) -> Option<&RecoverySnapshot> {
        self.snapshot.as_ref()
    }

    /// Draws only the cached, redacted projection. The caller owns mode transitions
    /// and is responsible for sending any returned selection to the shared executor.
    pub(crate) fn draw_context(
        &self,
        surface: &mut FramebufferSurface,
        layout: raios_core::genesis_layout::GenesisLayout,
        shared_executor_ready: bool,
    ) {
        let rect = rect_from_layout(layout.context);
        draw_panel(surface, rect, "Recovery context");
        let Some(snapshot) = self.snapshot() else {
            draw_truncated_text(
                surface,
                rect.x + 14,
                rect.y + 50,
                "Refresh recovery facts",
                rect.w.saturating_sub(28) / super::genesis::FONT_ADVANCE,
                APP_AMBER,
            );
            return;
        };
        if !supports_full_recovery_context(rect) {
            draw_compact_context(surface, rect, snapshot);
            return;
        }

        let rows = [
            (
                "Posture",
                snapshot.posture.label(),
                snapshot.posture.color(),
            ),
            (
                "Current problem",
                snapshot.primary_problem.label(),
                APP_AMBER,
            ),
            ("Last known good", snapshot.last_good.label(), TEXT_MUTED),
            ("Crashed services", snapshot.crashed.label(), TEXT_MUTED),
            ("Disabled modules", snapshot.disabled.label(), TEXT_MUTED),
        ];
        let mut y = rect.y + 45;
        for (label, value, color) in rows {
            text::draw_text(surface, rect.x + 14, y, label, TEXT_MUTED, None);
            draw_truncated_text(
                surface,
                rect.x + 14,
                y + 10,
                value,
                rect.w.saturating_sub(28) / super::genesis::FONT_ADVANCE,
                color,
            );
            y = y.saturating_add(29);
        }

        let actions = action_rects(rect);
        for (index, selection) in RecoveryActionSelection::ALL.iter().copied().enumerate() {
            let offer = recovery_action_offer(selection);
            draw_button(
                surface,
                actions[index],
                offer.label,
                shared_executor_ready
                    && offer.availability == RecoveryActionAvailability::RequiresSharedExecutor,
            );
        }
    }

    /// Returns only a typed selection. It performs no evaluator call and makes no
    /// authority claim; its integration point must invoke the shared lifeline executor.
    pub(crate) fn pointer_selection(
        &self,
        layout: raios_core::genesis_layout::GenesisLayout,
        x: usize,
        y: usize,
        shared_executor_ready: bool,
    ) -> Option<RecoveryActionSelection> {
        if !shared_executor_ready {
            return None;
        }
        let context = rect_from_layout(layout.context);
        if !supports_full_recovery_context(context) {
            return None;
        }
        let rects = action_rects(context);
        for (index, selection) in RecoveryActionSelection::ALL.iter().copied().enumerate() {
            if point_in(x, y, rects[index])
                && recovery_action_offer(selection).availability
                    == RecoveryActionAvailability::RequiresSharedExecutor
            {
                return Some(selection);
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RecoverySnapshot {
    pub(crate) posture: RecoveryPosture,
    pub(crate) last_good: LastGoodView,
    pub(crate) crashed: CurrentBootServiceView,
    pub(crate) disabled: CurrentBootServiceView,
    pub(crate) service_health: RecoveryServiceHealth,
    pub(crate) primary_problem: RecoveryProblem,
}

impl RecoverySnapshot {
    /// Reads the same typed state as the serial lifeline snapshot. This is the one
    /// recovery projection call that may read BOOTCTL; `RecoveryView` caches it.
    fn collect(runtime: RuntimeStatus) -> Self {
        let system = SystemSnapshot::collect(None, runtime);
        let provider = provider::snapshot();
        let (posture, decision, control) = boot_control::current_boot_last_good_view();
        let crashed = CurrentBootServiceView::from_crashed(echo_service::crash_view());
        let disabled = CurrentBootServiceView::from_disabled(echo_service::disabled_view());
        let service_health = RecoveryServiceHealth::collect(&system, &provider);
        let primary_problem =
            RecoveryProblem::from_sources(posture, crashed, disabled, service_health);

        Self {
            posture: RecoveryPosture::from(posture),
            last_good: LastGoodView::from_read(
                decision.reason,
                control.map(|record| record.last_good),
            ),
            crashed,
            disabled,
            service_health,
            primary_problem,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryPosture {
    value: BootPosture,
}

impl From<BootPosture> for RecoveryPosture {
    fn from(value: BootPosture) -> Self {
        Self { value }
    }
}

impl RecoveryPosture {
    pub(crate) const fn label(self) -> &'static str {
        match self.value {
            BootPosture::Normal => "Normal",
            BootPosture::Probation => "Probation",
            BootPosture::Safe => "SAFE",
            BootPosture::PersistenceUnavailable => "Storage unavailable",
        }
    }

    const fn color(self) -> crate::framebuffer::Color {
        match self.value {
            BootPosture::Normal => APP_GREEN,
            BootPosture::Probation | BootPosture::PersistenceUnavailable => APP_AMBER,
            BootPosture::Safe => super::genesis::APP_RED,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LastGoodView {
    slot: Option<BootSlotId>,
    reason: &'static str,
}

impl LastGoodView {
    fn from_read(reason: &'static str, slot: Option<BootSlotId>) -> Self {
        Self { slot, reason }
    }

    pub(crate) fn label(self) -> &'static str {
        self.slot
            .and_then(BootSlotId::as_str)
            .unwrap_or(self.reason)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CurrentBootServiceView {
    id: Option<&'static str>,
}

impl CurrentBootServiceView {
    const fn none() -> Self {
        Self { id: None }
    }

    fn from_crashed(
        value: Option<(
            &'static str,
            &'static str,
            Option<crate::event_log::EventId>,
        )>,
    ) -> Self {
        Self {
            id: value.map(|(id, _, _)| id),
        }
    }

    fn from_disabled(
        value: Option<(
            &'static str,
            &'static str,
            Option<crate::event_log::EventId>,
        )>,
    ) -> Self {
        Self {
            id: value.map(|(id, _, _)| id),
        }
    }

    pub(crate) fn label(self) -> &'static str {
        self.id.unwrap_or("None")
    }

    const fn is_present(self) -> bool {
        self.id.is_some()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryServiceHealth {
    pub(crate) unhealthy_count: u16,
}

impl RecoveryServiceHealth {
    fn collect(system: &SystemSnapshot, provider: &provider::Snapshot) -> Self {
        let mut unhealthy_count = 0u16;
        for service in service_inventory::SERVICES {
            let health = service_inventory::service_health(service, system, provider);
            if health.state != "healthy" && health.state != "starting" {
                unhealthy_count = unhealthy_count.saturating_add(1);
            }
        }
        Self { unhealthy_count }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum RecoveryProblem {
    None,
    SafeBoot,
    RecoveryStorageUnavailable,
    CurrentBootServiceCrashed,
    CurrentBootModuleDisabled,
    ServiceHealthDegraded,
}

impl RecoveryProblem {
    fn from_sources(
        posture: BootPosture,
        crashed: CurrentBootServiceView,
        disabled: CurrentBootServiceView,
        service_health: RecoveryServiceHealth,
    ) -> Self {
        match posture {
            BootPosture::Safe => Self::SafeBoot,
            BootPosture::PersistenceUnavailable => Self::RecoveryStorageUnavailable,
            BootPosture::Normal | BootPosture::Probation if crashed.is_present() => {
                Self::CurrentBootServiceCrashed
            }
            BootPosture::Normal | BootPosture::Probation if disabled.is_present() => {
                Self::CurrentBootModuleDisabled
            }
            BootPosture::Normal | BootPosture::Probation if service_health.unhealthy_count > 0 => {
                Self::ServiceHealthDegraded
            }
            BootPosture::Normal | BootPosture::Probation => Self::None,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::None => "No active recovery problem",
            Self::SafeBoot => "SAFE boot posture",
            Self::RecoveryStorageUnavailable => "Recovery storage unavailable",
            Self::CurrentBootServiceCrashed => "Current-boot service crashed",
            Self::CurrentBootModuleDisabled => "Current-boot module disabled",
            Self::ServiceHealthDegraded => "Service health needs attention",
        }
    }
}

/// A typed choice from the core-owned Recovery surface. It is intentionally smaller
/// than the lifeline table: table/snapshot are reads, while rollback remains visibly
/// unavailable until the frozen table says otherwise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryActionSelection {
    RestartLastGood,
    DisableModule,
    LoadLocalHash,
    Rollback,
}

impl RecoveryActionSelection {
    const ALL: [Self; RECOVERY_ACTION_COUNT] = [
        Self::RestartLastGood,
        Self::DisableModule,
        Self::LoadLocalHash,
        Self::Rollback,
    ];

    pub(crate) const fn method(self) -> &'static str {
        match self {
            Self::RestartLastGood => lifeline::METHOD_RESTART_LAST_GOOD,
            Self::DisableModule => lifeline::METHOD_DISABLE_MODULE,
            Self::LoadLocalHash => lifeline::METHOD_LOAD_ARTIFACT_BY_HASH,
            Self::Rollback => lifeline::METHOD_ROLLBACK,
        }
    }

    pub(crate) const fn immediate_intent(self) -> Option<RecoveryActionIntent> {
        match self {
            Self::RestartLastGood => Some(RecoveryActionIntent::RestartLastGood {
                target: RecoveryTarget::DemoEcho,
            }),
            Self::DisableModule => Some(RecoveryActionIntent::DisableModule {
                target: RecoveryTarget::DemoEcho,
            }),
            Self::LoadLocalHash | Self::Rollback => None,
        }
    }
}

/// The only mutating recovery requests Genesis may form. Their execution belongs in
/// the existing lifeline evaluator; this enum cannot mutate state by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryActionIntent {
    RestartLastGood { target: RecoveryTarget },
    DisableModule { target: RecoveryTarget },
    LoadLocalArtifactByHash { artifact_sha256: [u8; 32] },
}

impl RecoveryActionIntent {
    pub(crate) const fn method(self) -> &'static str {
        match self {
            Self::RestartLastGood { .. } => lifeline::METHOD_RESTART_LAST_GOOD,
            Self::DisableModule { .. } => lifeline::METHOD_DISABLE_MODULE,
            Self::LoadLocalArtifactByHash { .. } => lifeline::METHOD_LOAD_ARTIFACT_BY_HASH,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryTarget {
    DemoEcho,
}

impl RecoveryTarget {
    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::DemoEcho => "svc.demo.echo",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryActionAvailability {
    /// The fixed lifeline recognizes the action, but the shared evaluator still
    /// decides it from live posture, target, durable evidence, and re-verification.
    RequiresSharedExecutor,
    Unavailable,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryActionOffer {
    pub(crate) label: &'static str,
    pub(crate) availability: RecoveryActionAvailability,
}

pub(crate) fn recovery_action_offer(selection: RecoveryActionSelection) -> RecoveryActionOffer {
    let label = match selection {
        RecoveryActionSelection::RestartLastGood => "Restart last good",
        RecoveryActionSelection::DisableModule => "Disable module",
        RecoveryActionSelection::LoadLocalHash => "Load local hash",
        RecoveryActionSelection::Rollback => "Rollback unavailable",
    };
    let availability = match lifeline::lookup(selection.method()) {
        Some(method) if method.implemented => RecoveryActionAvailability::RequiresSharedExecutor,
        _ => RecoveryActionAvailability::Unavailable,
    };
    RecoveryActionOffer {
        label,
        availability,
    }
}

fn action_rects(context: LogicalRect) -> [LogicalRect; RECOVERY_ACTION_COUNT] {
    let x = context.x + 12;
    let width = context.w.saturating_sub(24);
    let first_y = context.y + context.h.saturating_sub(94);
    [
        LogicalRect::new(x, first_y, width, 18),
        LogicalRect::new(x, first_y + 20, width, 18),
        LogicalRect::new(x, first_y + 40, width, 18),
        LogicalRect::new(x, first_y + 60, width, 18),
    ]
}

fn supports_full_recovery_context(context: LogicalRect) -> bool {
    context.h >= FULL_RECOVERY_CONTEXT_HEIGHT
}

fn draw_compact_context(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    snapshot: &RecoverySnapshot,
) {
    draw_truncated_text(
        surface,
        rect.x + 14,
        rect.y + 48,
        snapshot.posture.label(),
        rect.w.saturating_sub(28) / super::genesis::FONT_ADVANCE,
        snapshot.posture.color(),
    );
    draw_truncated_text(
        surface,
        rect.x + 14,
        rect.y + 68,
        snapshot.primary_problem.label(),
        rect.w.saturating_sub(28) / super::genesis::FONT_ADVANCE,
        APP_AMBER,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_follow_the_pinned_lifeline_flags() {
        assert_eq!(
            recovery_action_offer(RecoveryActionSelection::RestartLastGood).availability,
            RecoveryActionAvailability::RequiresSharedExecutor
        );
        assert_eq!(
            recovery_action_offer(RecoveryActionSelection::Rollback).availability,
            RecoveryActionAvailability::Unavailable
        );
    }

    #[test]
    fn immediate_intents_are_typed_and_hash_loading_requires_real_input() {
        let restart = RecoveryActionSelection::RestartLastGood
            .immediate_intent()
            .expect("restart has one fixed target");
        assert_eq!(restart.method(), lifeline::METHOD_RESTART_LAST_GOOD);
        assert_eq!(RecoveryTarget::DemoEcho.id(), "svc.demo.echo");
        assert!(RecoveryActionSelection::LoadLocalHash
            .immediate_intent()
            .is_none());
    }

    #[test]
    fn compact_context_never_exposes_invisible_recovery_actions() {
        assert!(!supports_full_recovery_context(LogicalRect::new(
            0, 0, 480, 96
        )));
        assert!(supports_full_recovery_context(LogicalRect::new(
            0, 0, 220, 270
        )));
    }
}
