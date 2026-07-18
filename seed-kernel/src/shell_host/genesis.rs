//! Core-owned Genesis presentation.  It renders only typed snapshots and delegates
//! existing setup actions to the current console/provider adapters.

use crate::agent_protocol::recovery_lifeline;
use crate::framebuffer::{Color, FramebufferInfo, FramebufferSurface};
use crate::system_status::{RowState, SnapshotStates, StatusLine, SystemSnapshot};
use crate::{
    agent_protocol_project_install, console, granted_candidate_service, input,
    personal_shell_service, program_persistence, program_workspace, provider, secret_vault, serial,
    text, wifi, workspace_candidate_service,
};
use raios_core::{
    genesis_layout::{GenesisLayout, Point, Size},
    personal_shell_abi::{PersonalShellContext, SanitizedInputEvent, SanitizedInputKind},
    ui_frame::{self, Command as PersonalFrameCommand, Viewport as PersonalViewport},
};

use super::{
    context,
    dream::{self, DreamState, HitTarget as DreamHitTarget, Tab as DreamTab},
    personal_surface::{PersonalSurface, PersonalSurfaceRoute},
    recovery, vault_flow, wifi_flow,
};

const CONTAINED_QEMU_POWER_CUT_KEYCODE_F9: u16 = 67;
const CONVERSATION_WHEEL_ROWS: usize = 3;
const COMPOSER_CURSOR_BLINK_MS: u64 = 500;

pub(crate) const FONT_ADVANCE: usize = 9;
pub(crate) const SURFACE_BG: Color = Color::new(29, 32, 38);
pub(crate) const SURFACE_ALT: Color = Color::new(39, 43, 51);
pub(crate) const HAIRLINE: Color = Color::new(62, 67, 76);
pub(crate) const TEXT_MAIN: Color = Color::new(238, 241, 245);
pub(crate) const TEXT_MUTED: Color = Color::new(169, 177, 189);
pub(crate) const TEXT_FAINT: Color = Color::new(117, 126, 140);
pub(crate) const APP_BLUE: Color = Color::new(72, 151, 242);
pub(crate) const APP_GREEN: Color = Color::new(84, 187, 125);
pub(crate) const APP_AMBER: Color = Color::new(222, 164, 71);
pub(crate) const APP_RED: Color = Color::new(218, 88, 82);

#[derive(Clone, Copy)]
pub(crate) struct LogicalRect {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) w: usize,
    pub(crate) h: usize,
}

impl LogicalRect {
    pub(crate) const fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }
}

pub struct ShellHost {
    surface: Option<FramebufferSurface>,
    last_states: Option<SnapshotStates>,
    last_draw_states: Option<SnapshotStates>,
    last_mouse_buttons: u8,
    last_cursor_rect: Option<CursorRect>,
    last_composer_cursor_rect: Option<CursorRect>,
    wifi: wifi_flow::GuidedWifi,
    recovery: recovery::RecoveryView,
    recovery_open: bool,
    conversation_scroll_rows: usize,
    last_composer_cursor_phase: bool,
    dream: DreamState,
    personal: PersonalSurface,
    vault: vault_flow::VaultFlow,
}

impl ShellHost {
    pub fn new(surface: Option<FramebufferSurface>) -> Self {
        if let Some(surface) = surface.as_ref() {
            let info = surface.info();
            input::set_pointer_bounds(info.width as usize, info.height as usize);
        }
        Self {
            surface,
            last_states: None,
            last_draw_states: None,
            last_mouse_buttons: 0,
            last_cursor_rect: None,
            last_composer_cursor_rect: None,
            wifi: wifi_flow::GuidedWifi::new(),
            recovery: recovery::RecoveryView::new(),
            recovery_open: false,
            conversation_scroll_rows: 0,
            last_composer_cursor_phase: false,
            dream: DreamState::new(),
            personal: PersonalSurface::new(),
            vault: vault_flow::VaultFlow::new(),
        }
    }

    pub fn render(&mut self, uptime_ms: u64, runtime: crate::system_status::RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, false);
    }

    pub fn render_forced(&mut self, uptime_ms: u64, runtime: crate::system_status::RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, true);
    }

    fn render_inner(
        &mut self,
        uptime_ms: u64,
        runtime: crate::system_status::RuntimeStatus,
        force_draw: bool,
    ) {
        let reconnect_started =
            self.vault.take_normal_reconnect_request() && self.wifi.begin_normal_saved_reconnect();
        let flow_changed = reconnect_started | self.wifi.advance();
        let framebuffer = self.surface.as_ref().map(|surface| surface.info());
        let snapshot = SystemSnapshot::collect(framebuffer, runtime);
        let personal_changed = if self.vault.is_active() {
            false
        } else {
            self.activate_pending_personal_shell(&snapshot)
        };
        self.log_transitions(&snapshot);
        let states = snapshot.states();
        let dream_changed = self.dream.update_for_frame(uptime_ms);
        if flow_changed
            || personal_changed
            || dream_changed
            || force_draw
            || self.last_draw_states != Some(states)
        {
            if let Some(surface) = self.surface.as_mut() {
                if self.personal.has_personal_focus() {
                    let Some(layout) = genesis_layout(surface.info()) else {
                        return;
                    };
                    let rendered = self
                        .personal
                        .frame()
                        .is_some_and(|frame| draw_personal_frame(surface, layout, frame));
                    if rendered {
                        // The Dream shell has no secure/title strip.  The area
                        // outside the bounded personal viewport remains blank.
                        let scale = surface.draw_scale();
                        surface.fill_rect(
                            0,
                            0,
                            layout.logical_size.width as usize,
                            layout.personal_surface.y as usize,
                            Color::new(8, 9, 11),
                        );
                        let personal = rect_from_layout(layout.personal_surface);
                        surface.present_rect(
                            personal.x.saturating_mul(scale),
                            personal.y.saturating_mul(scale),
                            personal.w.saturating_mul(scale),
                            personal.h.saturating_mul(scale),
                        );
                        surface.present_rect(
                            0,
                            0,
                            layout.logical_size.width as usize * scale,
                            layout.personal_surface.y as usize * scale,
                        );
                    } else {
                        self.personal.exit();
                        console::write_event(format_args!(
                            "PERSONAL SHELL FALLBACK: validated frame unavailable"
                        ));
                        draw_genesis(
                            surface,
                            uptime_ms,
                            &snapshot,
                            self.conversation_scroll_rows,
                            &self.wifi,
                            &self.recovery,
                            self.recovery_open,
                            &mut self.vault,
                            &mut self.dream,
                        );
                        surface.present();
                    }
                } else {
                    draw_genesis(
                        surface,
                        uptime_ms,
                        &snapshot,
                        self.conversation_scroll_rows,
                        &self.wifi,
                        &self.recovery,
                        self.recovery_open,
                        &mut self.vault,
                        &mut self.dream,
                    );
                    surface.present();
                }
                self.vault.note_presented();
                self.last_composer_cursor_rect = None;
                self.last_cursor_rect = None;
                draw_current_cursor(surface, &mut self.last_cursor_rect);
                self.last_draw_states = Some(states);
            }
        }
        self.render_composer_cursor(uptime_ms);
    }

    pub fn render_pointer(&mut self) {
        if let Some(surface) = self.surface.as_mut() {
            if let Some(rect) = self.last_cursor_rect.take() {
                surface.restore_from_back_rect(rect.x, rect.y, rect.w, rect.h);
            }
            if let Some(rect) = self.last_composer_cursor_rect {
                draw_front_rect(surface, rect, APP_BLUE);
            }
            draw_current_cursor(surface, &mut self.last_cursor_rect);
        }
    }

    fn render_composer_cursor(&mut self, uptime_ms: u64) {
        let phase = composer_cursor_phase(uptime_ms);
        let active = !self.personal.has_personal_focus()
            && self.dream.tab() == DreamTab::Chat
            && console::composer_active();
        let visible = active && phase;
        if !active && self.last_composer_cursor_rect.is_none() {
            self.last_composer_cursor_phase = phase;
            return;
        }
        if visible == self.last_composer_cursor_rect.is_some()
            && phase == self.last_composer_cursor_phase
        {
            return;
        }
        let snapshot = console::snapshot();
        if let Some(surface) = self.surface.as_mut() {
            if let Some(rect) = self.last_cursor_rect.take() {
                surface.restore_from_back_rect(rect.x, rect.y, rect.w, rect.h);
            }
            if let Some(rect) = self.last_composer_cursor_rect.take() {
                surface.restore_from_back_rect(rect.x, rect.y, rect.w, rect.h);
            }
            if visible {
                self.last_composer_cursor_rect =
                    draw_composer_cursor_front(surface, &self.dream, &snapshot);
            }
            draw_current_cursor(surface, &mut self.last_cursor_rect);
        }
        self.last_composer_cursor_phase = phase;
    }

    pub fn handle_pointer_interaction(
        &mut self,
        runtime: crate::system_status::RuntimeStatus,
    ) -> bool {
        let Some(info) = self.surface.as_ref().map(|surface| surface.info()) else {
            return false;
        };
        let mouse = input::mouse_snapshot();
        let left_down = mouse.buttons & 1 != 0;
        let left_was_down = self.last_mouse_buttons & 1 != 0;
        self.last_mouse_buttons = mouse.buttons;
        let x = mouse.x / 2;
        let y = mouse.y / 2;
        let view = console::snapshot().view;
        let dream_interactions_enabled = !self.vault.is_active()
            && !self.personal.has_personal_focus()
            && !self.wifi.is_active()
            && view != console::UiView::Settings;
        let approve_enabled = dream::real_approval_available();
        let visual_changed = self.dream.update_pointer(
            mouse.seen,
            x,
            y,
            dream_interactions_enabled,
            approve_enabled,
        );
        let animation_frame_due = self.dream.animation_frame_due();
        if !mouse.seen || !left_down || left_was_down {
            return visual_changed || animation_frame_due;
        }

        let Some(layout) = genesis_layout(info) else {
            return visual_changed;
        };
        let width = layout.logical_size.width as usize;
        let height = layout.logical_size.height as usize;

        if self.vault.is_active() {
            return self.vault.handle_pointer(x, y, layout) || visual_changed;
        }
        if self.personal.has_personal_focus() {
            return visual_changed;
        }
        if self.wifi.is_active() {
            return self.wifi.handle_pointer(x, y, width, height) || visual_changed;
        }
        if view == console::UiView::Settings {
            return self.handle_setup_pointer(layout, x, y, width, height) || visual_changed;
        }

        let hit = self.dream.hit_test(x, y, approve_enabled);
        if hit == Some(DreamHitTarget::Recovery) {
            return self.toggle_recovery(runtime);
        }
        if self.recovery_open {
            return self.handle_recovery_pointer(layout, x, y, runtime) || visual_changed;
        }

        match hit {
            Some(DreamHitTarget::Composer | DreamHitTarget::TabChat) => {
                self.dream.set_tab(DreamTab::Chat);
                let _ = console::set_view(console::UiView::Ai);
                true
            }
            Some(DreamHitTarget::TabConsole) => {
                self.dream.set_tab(DreamTab::Console);
                let _ = console::set_view(console::UiView::Console);
                true
            }
            Some(DreamHitTarget::TabBuild) => {
                self.dream.set_tab(DreamTab::Build);
                true
            }
            Some(DreamHitTarget::AiSetup) => {
                self.dream.set_tab(DreamTab::Chat);
                open_setup()
            }
            Some(DreamHitTarget::WifiSetup) => self.wifi.begin(),
            Some(DreamHitTarget::Approve) => self.handle_real_approval(runtime),
            Some(DreamHitTarget::Inert) => true,
            Some(DreamHitTarget::Recovery) => true,
            None => visual_changed,
        }
    }

    fn handle_real_approval(&mut self, runtime: crate::system_status::RuntimeStatus) -> bool {
        if agent_protocol_project_install::pending_physical_approval() {
            return agent_protocol_project_install::approve_from_pointer();
        }
        if granted_candidate_service::pending_approval() {
            return granted_candidate_service::approve_and_run_from_pointer();
        }
        if workspace_candidate_service::pending_approval() {
            return workspace_candidate_service::approve_and_run_from_pointer();
        }
        let Some(program) = program_workspace::retained_program() else {
            return false;
        };
        let identity = program.identity();
        let Some(context) = self.personal_context(runtime) else {
            console::write_event(format_args!(
                "PROGRAM START DENIED: framebuffer unavailable"
            ));
            return true;
        };
        let route = self.personal.enter_program(context, program);
        let approved = if route == PersonalSurfaceRoute::Entered {
            match program_workspace::approve_retained_program() {
                Ok(approved) => Some(approved),
                Err(reason) => {
                    console::write_event(format_args!("PROGRAM INSTALL READY DENIED: {reason}"));
                    None
                }
            }
        } else {
            None
        };
        note_program_route(identity.sha256, route, approved.as_ref());
        true
    }

    /// Called by the console before normal text handling. Secure attention and
    /// all input while focused are consumed here; Genesis still receives input
    /// only after this route declines it.
    pub fn handle_input_event(
        &mut self,
        event: input::InputEvent,
        runtime: crate::system_status::RuntimeStatus,
    ) -> bool {
        if matches!(
            event.kind,
            input::InputEventKind::Key {
                code: CONTAINED_QEMU_POWER_CUT_KEYCODE_F9,
                pressed: true
            }
        ) && secret_vault::contained_qemu_wifi_test_available()
        {
            if let Err(error) = secret_vault::arm_contained_qemu_wifi_power_cut_test() {
                serial::write_fmt(format_args!(
                    "C1_VAULT_POWER_CUT_PRECOMMIT_REJECTED reason={} test_infrastructure=true\r\n",
                    error.recovery_reason()
                ));
            }
            return true;
        }
        if self.vault.is_active() {
            return self.vault.handle_physical_input(event);
        }
        if self.wifi.handle_physical_input(event) {
            return true;
        }
        let console_snapshot = console::snapshot();
        if !self.personal.has_personal_focus()
            && !self.recovery_open
            && self.dream.tab() == DreamTab::Chat
            && console_snapshot.view == console::UiView::Ai
            && self.handle_conversation_scroll(event, &console_snapshot)
        {
            return true;
        }
        if !self.personal.has_personal_focus()
            && console_snapshot.view == console::UiView::Settings
            && matches!(
                event.kind,
                input::InputEventKind::Key {
                    code: 28,
                    pressed: true
                }
            )
        {
            match console_snapshot.focus {
                console::UiFocus::SettingsVault => return self.vault.begin_explicit(),
                console::UiFocus::SettingsApiKey => {
                    return self.vault.begin_provider_explicit();
                }
                console::UiFocus::SettingsWifiSsid
                    if secret_vault::contained_qemu_wifi_test_available() =>
                {
                    return self.vault.begin_contained_wifi_explicit();
                }
                _ => {}
            }
        }
        if matches!(event.kind, input::InputEventKind::SecureAttention) {
            let granted_dropped = granted_candidate_service::secure_attention_drop();
            let workspace_dropped = workspace_candidate_service::secure_attention_drop();
            if self.personal.has_personal_focus() {
                let route = self.personal.handle_secure_attention();
                note_personal_route(route);
                return true;
            }
            return self.toggle_recovery(runtime) || granted_dropped || workspace_dropped;
        }
        if !self.personal.has_personal_focus() {
            return false;
        }
        let Some(context) = self.personal_context(runtime) else {
            self.personal.exit();
            console::write_event(format_args!(
                "PERSONAL SHELL FALLBACK: framebuffer unavailable"
            ));
            return true;
        };
        let Some(layout) = self
            .surface
            .as_ref()
            .and_then(|surface| genesis_layout(surface.info()))
        else {
            self.personal.exit();
            console::write_event(format_args!(
                "PERSONAL SHELL FALLBACK: framebuffer unavailable"
            ));
            return true;
        };
        let Some(event) = sanitize_personal_input(event, layout) else {
            return true;
        };
        let route = self.personal.route_sanitized_event(context, event);
        note_personal_route(route);
        true
    }

    fn handle_conversation_scroll(
        &mut self,
        event: input::InputEvent,
        snapshot: &console::ConsoleSnapshot,
    ) -> bool {
        let visible_rows = self.dream.conversation_visible_rows();
        let max_scroll = conversation_row_count(snapshot, self.dream.conversation_max_chars())
            .saturating_sub(visible_rows);
        let delta = match event.kind {
            input::InputEventKind::Relative(input::RelativeAxis::Wheel, value) => {
                let mouse = input::mouse_snapshot();
                if !self.dream.conversation_contains(mouse.x / 2, mouse.y / 2) {
                    return false;
                }
                isize::try_from(value)
                    .unwrap_or(if value < 0 { isize::MIN } else { isize::MAX })
                    .saturating_mul(CONVERSATION_WHEEL_ROWS as isize)
            }
            input::InputEventKind::Key {
                code: 104,
                pressed: true,
            } => visible_rows.saturating_sub(1) as isize,
            input::InputEventKind::Key {
                code: 109,
                pressed: true,
            } => -(visible_rows.saturating_sub(1) as isize),
            _ => return false,
        };
        let previous = self.conversation_scroll_rows;
        if delta > 0 {
            self.conversation_scroll_rows = self
                .conversation_scroll_rows
                .saturating_add(delta as usize)
                .min(max_scroll);
        } else {
            self.conversation_scroll_rows = self
                .conversation_scroll_rows
                .saturating_sub(delta.unsigned_abs());
        }
        self.conversation_scroll_rows != previous
    }

    fn activate_pending_personal_shell(&mut self, snapshot: &SystemSnapshot) -> bool {
        let Some(mode) = personal_shell_service::take_current_boot_proof_start() else {
            return false;
        };
        let Some(context) = self.personal_context_from_snapshot(snapshot) else {
            console::write_event(format_args!(
                "PERSONAL SHELL START DENIED: framebuffer unavailable"
            ));
            return true;
        };
        let route = self.personal.enter_mode(context, mode);
        note_personal_route(route);
        true
    }

    fn personal_context(
        &self,
        runtime: crate::system_status::RuntimeStatus,
    ) -> Option<PersonalShellContext> {
        let framebuffer = self.surface.as_ref().map(|surface| surface.info());
        let snapshot = SystemSnapshot::collect(framebuffer, runtime);
        self.personal_context_from_snapshot(&snapshot)
    }

    fn personal_context_from_snapshot(
        &self,
        snapshot: &SystemSnapshot,
    ) -> Option<PersonalShellContext> {
        let info = self.surface.as_ref()?.info();
        let layout = genesis_layout(info)?;
        let width = layout.personal_surface.width;
        let height = layout.personal_surface.height;
        if width == 0 || height == 0 || width > u32::from(u16::MAX) || height > u32::from(u16::MAX)
        {
            return None;
        }
        let problem_count = context::project(snapshot, &provider::snapshot(), wifi::snapshot())
            .problems
            .active;
        Some(PersonalShellContext::new(
            0,
            width as u16,
            height as u16,
            crate::service_inventory::SERVICES
                .len()
                .min(u16::MAX as usize) as u16,
            problem_count as u16,
            crate::agent_protocol_system::denied_capability_count(),
            false,
            true,
            0,
        ))
    }

    fn toggle_recovery(&mut self, runtime: crate::system_status::RuntimeStatus) -> bool {
        if self.recovery_open {
            self.recovery_open = false;
            serial::write_line("GENESIS_RECOVERY_VIEW_CLOSED current_boot=true");
        } else {
            self.recovery.refresh(runtime);
            self.recovery_open = true;
            serial::write_line("GENESIS_RECOVERY_VIEW_OPENED current_boot=true");
        }
        true
    }

    fn handle_recovery_pointer(
        &mut self,
        layout: GenesisLayout,
        x: usize,
        y: usize,
        runtime: crate::system_status::RuntimeStatus,
    ) -> bool {
        let Some(selection) = self.recovery.pointer_selection(layout, x, y, true) else {
            return false;
        };
        let action = match selection {
            recovery::RecoveryActionSelection::RestartLastGood => {
                recovery_lifeline::GenesisRecoveryAction::RestartLastGoodDemoEcho
            }
            recovery::RecoveryActionSelection::DisableModule => {
                recovery_lifeline::GenesisRecoveryAction::DisableDemoEcho
            }
            recovery::RecoveryActionSelection::LoadLocalHash
            | recovery::RecoveryActionSelection::Rollback => return false,
        };
        let result = recovery_lifeline::execute_genesis_action(action);
        console::write_event(format_args!(
            "GENESIS RECOVERY {} {}: {}",
            selection.method(),
            if result.performed {
                "APPLIED"
            } else {
                "DENIED"
            },
            result.reason
        ));
        self.recovery.refresh(runtime);
        true
    }

    fn handle_setup_pointer(
        &mut self,
        layout: GenesisLayout,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> bool {
        let rect = rect_from_layout(layout.trusted_overlay);
        let action = setup_action_rects(rect);
        if point_in(x, y, setup_vault_rect(rect)) {
            return self.vault.begin_explicit();
        }
        if point_in(x, y, setup_keyboard_rect(rect)) {
            return console::activate_focus(console::UiFocus::SettingsKeyboardLayout);
        }
        if point_in(x, y, action[0]) {
            return self.vault.begin_provider_explicit();
        }
        if point_in(x, y, action[1]) {
            if secret_vault::contained_qemu_wifi_test_available() {
                return self.vault.begin_contained_wifi_explicit();
            }
            return console::activate_focus(console::UiFocus::SettingsWifiSsid);
        }
        if point_in(x, y, action[2]) {
            return self.wifi.begin();
        }
        if point_in(x, y, action[3]) {
            return console::activate_focus(console::UiFocus::SettingsClose);
        }
        // A visible setup entry field remains focusable; no private state is held here.
        point_in(
            x,
            y,
            LogicalRect::new(rect.x + 20, rect.y + 58, rect.w - 40, 34),
        ) && console::set_view(console::UiView::Settings)
            && width > 0
            && height > 0
    }

    fn log_transitions(&mut self, snapshot: &SystemSnapshot) {
        let states = snapshot.states();
        let previous = self.last_states;
        log_transition(
            previous.map(|state| state.framebuffer),
            &snapshot.framebuffer,
        );
        log_transition(previous.map(|state| state.entropy), &snapshot.entropy);
        log_transition(previous.map(|state| state.usb_xhci), &snapshot.usb_xhci);
        log_transition(
            previous.map(|state| state.usb_hotplug),
            &snapshot.usb_hotplug,
        );
        log_transition(previous.map(|state| state.wifi), &snapshot.wifi);
        log_transition(previous.map(|state| state.network), &snapshot.network);
        log_transition(previous.map(|state| state.input), &snapshot.input);
        log_transition(previous.map(|state| state.iommu), &snapshot.iommu);
        self.last_states = Some(states);
    }
}

fn open_setup() -> bool {
    console::set_view(console::UiView::Settings)
}

fn draw_genesis(
    surface: &mut FramebufferSurface,
    _uptime_ms: u64,
    snapshot: &SystemSnapshot,
    conversation_scroll_rows: usize,
    wifi: &wifi_flow::GuidedWifi,
    recovery: &recovery::RecoveryView,
    recovery_open: bool,
    vault: &mut vault_flow::VaultFlow,
    dream_state: &mut DreamState,
) {
    let Some(layout) = genesis_layout(surface.info()) else {
        return;
    };
    let console_snapshot = console::snapshot();
    dream::render(
        surface,
        dream_state,
        snapshot,
        &console_snapshot,
        conversation_scroll_rows,
        recovery_open,
    );
    if recovery_open {
        recovery.draw_context(surface, layout, true);
    }
    if console_snapshot.view == console::UiView::Settings {
        draw_setup_overlay(surface, layout, &console_snapshot, vault);
    }
    wifi.draw(
        surface,
        layout.logical_size.width as usize,
        layout.logical_size.height as usize,
    );
    vault.draw(surface, layout);
}

fn genesis_layout(info: FramebufferInfo) -> Option<GenesisLayout> {
    GenesisLayout::new(Size::new(info.width as u32, info.height as u32)).ok()
}

fn conversation_row_count(snapshot: &console::ConsoleSnapshot, max_chars: usize) -> usize {
    let mut rows = 0usize;
    for line in snapshot.chat_lines {
        if line.text.as_str().is_empty() {
            continue;
        }
        rows = rows.saturating_add(2);
        visit_wrapped_lines(line.text.as_str(), max_chars, |_| {
            rows = rows.saturating_add(1);
        });
    }
    rows.saturating_sub(1)
}

fn visit_wrapped_lines<'a>(value: &'a str, max_chars: usize, mut visit: impl FnMut(&'a str)) {
    if max_chars == 0 {
        return;
    }
    let mut start = 0usize;
    let mut chars = 0usize;
    for (index, ch) in value.char_indices() {
        if ch == '\n' {
            visit(&value[start..index]);
            start = index + ch.len_utf8();
            chars = 0;
        } else {
            if chars == max_chars {
                visit(&value[start..index]);
                start = index;
                chars = 0;
            }
            chars += 1;
        }
    }
    visit(&value[start..]);
}

fn draw_personal_frame(
    surface: &mut FramebufferSurface,
    layout: GenesisLayout,
    bytes: &[u8],
) -> bool {
    let area = rect_from_layout(layout.personal_surface);
    let viewport = PersonalViewport {
        width: area.w.min(u16::MAX as usize) as u16,
        height: area.h.min(u16::MAX as usize) as u16,
    };
    let Ok(frame) = ui_frame::validate_frame(bytes, viewport) else {
        return false;
    };
    surface.set_draw_scale(2);
    for command in frame.commands() {
        match command {
            PersonalFrameCommand::Clear { rgba } => {
                surface.fill_rect(area.x, area.y, area.w, area.h, personal_color(*rgba));
            }
            PersonalFrameCommand::FillRect { rect, rgba } => {
                surface.fill_rect(
                    area.x + rect.x as usize,
                    area.y + rect.y as usize,
                    rect.width as usize,
                    rect.height as usize,
                    personal_color(*rgba),
                );
            }
            PersonalFrameCommand::StrokeRect { rect, rgba } => draw_outline(
                surface,
                LogicalRect::new(
                    area.x + rect.x as usize,
                    area.y + rect.y as usize,
                    rect.width as usize,
                    rect.height as usize,
                ),
                personal_color(*rgba),
            ),
            PersonalFrameCommand::Text {
                x,
                y,
                rgba,
                text: value,
            } => text::draw_text(
                surface,
                area.x + *x as usize,
                area.y + *y as usize,
                value,
                personal_color(*rgba),
                None,
            ),
            PersonalFrameCommand::FocusHint { rect } => draw_outline(
                surface,
                LogicalRect::new(
                    area.x + rect.x as usize,
                    area.y + rect.y as usize,
                    rect.width as usize,
                    rect.height as usize,
                ),
                APP_AMBER,
            ),
        }
    }
    true
}

fn personal_color(rgba: u32) -> Color {
    Color {
        r: (rgba >> 24) as u8,
        g: (rgba >> 16) as u8,
        b: (rgba >> 8) as u8,
        a: rgba as u8,
    }
}

fn sanitize_personal_input(
    event: input::InputEvent,
    layout: GenesisLayout,
) -> Option<SanitizedInputEvent> {
    let (kind, pressed, code, x, y, dx, dy) = match event.kind {
        input::InputEventKind::SecureAttention => return None,
        input::InputEventKind::Key { code, pressed } if (272..=274).contains(&code) => {
            let position = current_personal_pointer(layout);
            if pressed && position.is_none() {
                return None;
            }
            let (x, y) = position.unwrap_or((0, 0));
            (SanitizedInputKind::PointerButton, pressed, code, x, y, 0, 0)
        }
        input::InputEventKind::Key { code, pressed } => {
            (SanitizedInputKind::Key, pressed, code, 0, 0, 0, 0)
        }
        input::InputEventKind::Relative(input::RelativeAxis::X, value) => (
            SanitizedInputKind::PointerMove,
            false,
            0,
            0,
            0,
            clamp_input_axis(value),
            0,
        ),
        input::InputEventKind::Relative(input::RelativeAxis::Y, value) => (
            SanitizedInputKind::PointerMove,
            false,
            0,
            0,
            0,
            0,
            clamp_input_axis(value),
        ),
        input::InputEventKind::Relative(input::RelativeAxis::Wheel, _) => return None,
        input::InputEventKind::Absolute { .. } => {
            let (x, y) = current_personal_pointer(layout)?;
            (SanitizedInputKind::PointerMove, false, 0, x, y, 0, 0)
        }
    };
    Some(SanitizedInputEvent::new(
        kind, pressed, false, code, x, y, dx, dy, 0,
    ))
}

fn current_personal_pointer(layout: GenesisLayout) -> Option<(i16, i16)> {
    let mouse = input::mouse_snapshot();
    if !mouse.seen {
        return None;
    }
    localize_personal_pointer(layout, mouse.x, mouse.y)
}

fn localize_personal_pointer(
    layout: GenesisLayout,
    physical_x: usize,
    physical_y: usize,
) -> Option<(i16, i16)> {
    let logical = Point::new((physical_x / 2) as u32, (physical_y / 2) as u32);
    if !layout.personal_surface.contains(logical) {
        return None;
    }
    Some((
        i16::try_from(logical.x - layout.personal_surface.x).ok()?,
        i16::try_from(logical.y - layout.personal_surface.y).ok()?,
    ))
}

fn clamp_input_axis(value: i32) -> i16 {
    value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

fn note_personal_route(route: PersonalSurfaceRoute) {
    match route {
        PersonalSurfaceRoute::Ignored => {}
        PersonalSurfaceRoute::Entered => {
            console::write_event(format_args!("PERSONAL SHELL ACTIVE current_boot proof"));
        }
        PersonalSurfaceRoute::FrameUpdated => {
            serial::write_line("PERSONAL SHELL FRAME UPDATED sanitized_input");
        }
        PersonalSurfaceRoute::ExitedToGenesis => {
            console::write_event(format_args!("PERSONAL SHELL EXIT F12 genesis"));
        }
        PersonalSurfaceRoute::GenesisFallback { reason, fuel_used } => {
            console::write_event(format_args!(
                "PERSONAL SHELL FALLBACK {} fuel_used={}",
                reason, fuel_used
            ));
        }
    }
}

fn note_program_route(
    sha256: [u8; 32],
    route: PersonalSurfaceRoute,
    approved: Option<&program_workspace::ApprovedProgramApproval>,
) {
    serial::write_raw_str(
        "PROGRAM_CURRENT_BOOT_ACTIVATION physical_approval=pointer program_sha256=sha256:",
    );
    for byte in sha256 {
        serial::write_raw_fmt(format_args!("{byte:02x}"));
    }
    match route {
        PersonalSurfaceRoute::Entered => serial::write_raw_str(
            " engine=svc.user.shell capability_surface=ui_only wasm=true result=accepted\r\n",
        ),
        PersonalSurfaceRoute::GenesisFallback { reason, fuel_used } => serial::write_raw_fmt(
            format_args!(
                " engine=svc.user.shell capability_surface=ui_only wasm=true result=denied reason={reason} fuel_used={fuel_used}\r\n"
            ),
        ),
        _ => serial::write_raw_str(
            " engine=svc.user.shell capability_surface=ui_only wasm=true result=denied reason=unexpected_route\r\n",
        ),
    }
    if let Some(approved) = approved {
        program_persistence::emit_install_ready_marker(approved);
    }
    note_personal_route(route);
}

fn composer_cursor_phase(uptime_ms: u64) -> bool {
    (uptime_ms / COMPOSER_CURSOR_BLINK_MS) % 2 == 0
}

fn draw_composer_cursor_front(
    surface: &mut FramebufferSurface,
    dream: &DreamState,
    snapshot: &console::ConsoleSnapshot,
) -> Option<CursorRect> {
    let (x, y, w, h) = dream.composer_cursor_rect(snapshot)?;
    let cursor = CursorRect { x, y, w, h };
    draw_front_rect(surface, cursor, APP_BLUE);
    Some(cursor)
}

fn draw_front_rect(surface: &mut FramebufferSurface, rect: CursorRect, color: Color) {
    for y in rect.y..rect.y.saturating_add(rect.h) {
        for x in rect.x..rect.x.saturating_add(rect.w) {
            surface.set_front_pixel(x, y, color);
        }
    }
}

fn draw_setup_overlay(
    surface: &mut FramebufferSurface,
    layout: GenesisLayout,
    snapshot: &console::ConsoleSnapshot,
    vault: &vault_flow::VaultFlow,
) {
    let rect = rect_from_layout(layout.trusted_overlay);
    surface.fill_rect(
        0,
        layout.personal_surface.y as usize,
        layout.logical_size.width as usize,
        layout.personal_surface.height as usize,
        Color::new(12, 14, 18),
    );
    draw_panel(surface, rect, "Trusted setup");
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 42,
        "Existing provider and WiFi setup",
        TEXT_MUTED,
        None,
    );
    text::draw_text(
        surface,
        rect.x + 20,
        rect.y + 70,
        vault.provider_status_text(),
        TEXT_MAIN,
        None,
    );
    draw_button(
        surface,
        setup_vault_rect(rect),
        vault.action_label(),
        snapshot.focus == console::UiFocus::SettingsVault,
    );
    let keyboard_label = match snapshot.keyboard_layout {
        input::KeyboardLayout::Us => "Keyboard: US / switch to DE",
        input::KeyboardLayout::German => "Keyboard: DE / switch to US",
    };
    draw_button(
        surface,
        setup_keyboard_rect(rect),
        keyboard_label,
        snapshot.focus == console::UiFocus::SettingsKeyboardLayout,
    );
    let actions = setup_action_rects(rect);
    draw_button(
        surface,
        actions[0],
        vault.provider_action_label(),
        snapshot.focus == console::UiFocus::SettingsApiKey,
    );
    draw_button(surface, actions[1], "Set WiFi", false);
    draw_button(surface, actions[2], "Scan WiFi", false);
    draw_button(surface, actions[3], "Close", false);
}

fn setup_action_rects(rect: LogicalRect) -> [LogicalRect; 4] {
    let width = rect.w.saturating_sub(52) / 2;
    let left = rect.x + 20;
    let right = left + width + 12;
    let first = rect.y + rect.h.saturating_sub(82);
    [
        LogicalRect::new(left, first, width, 24),
        LogicalRect::new(right, first, width, 24),
        LogicalRect::new(left, first + 32, width, 24),
        LogicalRect::new(right, first + 32, width, 24),
    ]
}

fn setup_vault_rect(rect: LogicalRect) -> LogicalRect {
    LogicalRect::new(rect.x + 20, rect.y + 104, rect.w.saturating_sub(40), 24)
}

fn setup_keyboard_rect(rect: LogicalRect) -> LogicalRect {
    LogicalRect::new(rect.x + 20, rect.y + 136, rect.w.saturating_sub(40), 24)
}

pub(crate) fn draw_panel(surface: &mut FramebufferSurface, rect: LogicalRect, title: &str) {
    surface.fill_rect(rect.x, rect.y, rect.w, rect.h, SURFACE_BG);
    draw_outline(surface, rect, HAIRLINE);
    text::draw_text(surface, rect.x + 14, rect.y + 16, title, TEXT_MAIN, None);
    surface.fill_rect(
        rect.x + 14,
        rect.y + 31,
        rect.w.saturating_sub(28),
        1,
        HAIRLINE,
    );
}

pub(crate) fn draw_button(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    label: &str,
    primary: bool,
) {
    surface.fill_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if primary { APP_BLUE } else { SURFACE_ALT },
    );
    draw_outline(surface, rect, if primary { APP_BLUE } else { HAIRLINE });
    let x = rect.x + rect.w.saturating_sub(text_width(label)) / 2;
    text::draw_text(surface, x, rect.y + 8, label, TEXT_MAIN, None);
}

pub(crate) fn draw_outline(surface: &mut FramebufferSurface, rect: LogicalRect, color: Color) {
    if rect.w == 0 || rect.h == 0 {
        return;
    }
    surface.fill_rect(rect.x, rect.y, rect.w, 1, color);
    surface.fill_rect(rect.x, rect.y + rect.h.saturating_sub(1), rect.w, 1, color);
    surface.fill_rect(rect.x, rect.y, 1, rect.h, color);
    surface.fill_rect(rect.x + rect.w.saturating_sub(1), rect.y, 1, rect.h, color);
}

pub(crate) fn draw_truncated_text(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    value: &str,
    max_chars: usize,
    color: Color,
) {
    if value.chars().count() <= max_chars {
        text::draw_text(surface, x, y, value, color, None);
        return;
    }
    if max_chars < 4 {
        return;
    }
    let mut end = 0usize;
    for (count, (index, ch)) in value.char_indices().enumerate() {
        if count == max_chars - 3 {
            break;
        }
        end = index + ch.len_utf8();
    }
    text::draw_text(surface, x, y, &value[..end], color, None);
    text::draw_text(
        surface,
        x + (max_chars - 3) * FONT_ADVANCE,
        y,
        "...",
        color,
        None,
    );
}

pub(crate) fn text_width(value: &str) -> usize {
    value.chars().count().saturating_mul(FONT_ADVANCE)
}

pub(crate) fn point_in(x: usize, y: usize, rect: LogicalRect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.w)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.h)
}

pub(crate) fn rect_from_layout(rect: raios_core::genesis_layout::Rect) -> LogicalRect {
    LogicalRect::new(
        rect.x as usize,
        rect.y as usize,
        rect.width as usize,
        rect.height as usize,
    )
}

pub(crate) fn row_color(state: RowState) -> Color {
    match state {
        RowState::Ready | RowState::Configured => APP_GREEN,
        RowState::Detected => APP_BLUE,
        RowState::Waiting | RowState::Degraded => APP_AMBER,
        RowState::Missing => APP_RED,
    }
}

fn log_transition(previous: Option<RowState>, line: &StatusLine) {
    if previous == Some(line.state) {
        return;
    }
    serial::write_fmt(format_args!(
        "status {}: {} - {}\r\n",
        line.label,
        line.state.as_str(),
        line.detail.as_str()
    ));
    console::record_event(format_args!(
        "STATUS {} {}",
        line.label,
        line.state.as_str()
    ));
}

#[derive(Clone, Copy)]
struct CursorRect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

fn draw_current_cursor(surface: &mut FramebufferSurface, last_rect: &mut Option<CursorRect>) {
    let mouse = input::mouse_snapshot();
    if !mouse.seen {
        return;
    }
    let x = mouse.x;
    let y = mouse.y;
    let info = surface.info();
    if x >= info.width as usize || y >= info.height as usize {
        return;
    }
    let fill = if mouse.buttons & 1 != 0 {
        APP_BLUE
    } else {
        TEXT_MAIN
    };
    let outline = Color::new(5, 8, 12);
    let scale = surface.draw_scale();
    let shape = [
        "X",
        "XX",
        "XOX",
        "XOOX",
        "XOOOX",
        "XOOOOX",
        "XOOOOOX",
        "XOOOOOOX",
        "XOOOOOOOX",
        "XOOOOX",
        "XOOXOX",
        "XOXXOX",
        "XX  XOX",
        "X    XOX",
        "     XOX",
        "      X",
    ];
    for (row, pattern) in shape.iter().enumerate() {
        for (col, pixel) in pattern.bytes().enumerate() {
            let color = match pixel {
                b'X' => outline,
                b'O' => fill,
                _ => continue,
            };
            draw_front_block(surface, x, y, col, row, scale, color);
        }
    }
    *last_rect = Some(CursorRect {
        x,
        y,
        w: usize::min(10usize.saturating_mul(scale), info.width as usize - x),
        h: usize::min(16usize.saturating_mul(scale), info.height as usize - y),
    });
}

fn draw_front_block(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    col: usize,
    row: usize,
    scale: usize,
    color: Color,
) {
    let start_x = x + col.saturating_mul(scale);
    let start_y = y + row.saturating_mul(scale);
    for dy in 0..scale {
        for dx in 0..scale {
            surface.set_front_pixel(start_x + dx, start_y + dy, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personal_pointer_coordinates_are_viewport_local_and_exclude_secure_ui() {
        let layout = GenesisLayout::new(Size::new(1920, 1080)).unwrap();
        assert_eq!(
            localize_personal_pointer(layout, 81 * 2, (38 + 105) * 2),
            Some((81, 105))
        );
        assert_eq!(localize_personal_pointer(layout, 10, 10), None);
    }
}
