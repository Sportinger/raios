//! Core-owned Genesis presentation.  It renders only typed snapshots and delegates
//! existing setup actions to the current console/provider adapters.

use alloc::format;

use crate::agent_protocol::recovery_lifeline;
use crate::framebuffer::{Color, FramebufferInfo, FramebufferSurface};
use crate::system_status::{RowState, SnapshotStates, StatusLine, SystemSnapshot};
use crate::{
    agent_build_loop, agent_protocol_project_install, console, granted_candidate_service, input,
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
    personal_surface::{PersonalSurface, PersonalSurfaceRoute},
    recovery, vault_flow, wifi_flow,
};

const CONTAINED_QEMU_POWER_CUT_KEYCODE_F9: u16 = 67;
const CONVERSATION_ROW_HEIGHT: usize = 12;
const CONVERSATION_WHEEL_ROWS: usize = 3;
const COMPOSER_CURSOR_BLINK_MS: u64 = 500;

pub(crate) const FONT_ADVANCE: usize = 9;
pub(crate) const APP_BG: Color = Color::new(20, 22, 26);
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
            personal: PersonalSurface::new(),
            vault: vault_flow::VaultFlow::new(),
        }
    }

    const EARLY_BOOT_MAX_SCALE: usize = 4;
    const EARLY_BOOT_GLYPH_WIDTH: usize = 8;
    const EARLY_BOOT_GLYPH_HEIGHT: usize = 8;
    const EARLY_BOOT_GLYPH_ADVANCE: usize = 9;

    pub fn render_early_boot_before_surface(&mut self) {
        self.render_early_boot_checkpoint("EB1", APP_BLUE);
    }

    pub fn render_early_boot_after_provider_config(&mut self) {
        self.render_early_boot_checkpoint("EB1P", SURFACE_BG);
    }

    pub fn render_early_boot_after_console(&mut self) {
        self.render_early_boot_checkpoint("EB1C", HAIRLINE);
    }

    pub fn render_early_boot_before_usb(&mut self) {
        self.render_early_boot_checkpoint("EB2", APP_AMBER);
    }

    pub fn render_early_boot_after_usb(&mut self) {
        self.render_early_boot_checkpoint("EB3", APP_GREEN);
    }

    pub fn render_early_boot_after_persist(&mut self) {
        self.render_early_boot_checkpoint("EB4P", SURFACE_ALT);
    }

    pub fn render_early_boot_persist_failed(&mut self) {
        self.render_early_boot_checkpoint("EB4E", APP_BG);
    }

    pub fn render_early_boot_surface_failed(&mut self) {
        self.render_early_boot_checkpoint("EB4F", APP_RED);
    }

    fn render_early_boot_checkpoint(&mut self, code: &'static str, background: Color) {
        if let Some(surface) = self.surface.as_mut() {
            let (scale, x, y, text_fits) =
                Self::early_boot_checkpoint_layout(surface.info(), code.len());
            surface.set_draw_scale(scale);
            surface.fill(background);
            if text_fits {
                text::draw_text(surface, x, y, code, TEXT_MAIN, None);
            }
            surface.present();
        }
    }

    fn early_boot_checkpoint_layout(
        info: FramebufferInfo,
        code_len: usize,
    ) -> (usize, usize, usize, bool) {
        let text_width = code_len
            .saturating_sub(1)
            .saturating_mul(Self::EARLY_BOOT_GLYPH_ADVANCE)
            .saturating_add(Self::EARLY_BOOT_GLYPH_WIDTH);
        let width = info.width as usize;
        let height = info.height as usize;
        let fit_scale = usize::min(
            width / text_width.max(1),
            height / Self::EARLY_BOOT_GLYPH_HEIGHT,
        );
        let text_fits = width >= text_width && height >= Self::EARLY_BOOT_GLYPH_HEIGHT;
        let scale = usize::min(Self::EARLY_BOOT_MAX_SCALE, fit_scale).max(1);
        let x = width.saturating_sub(text_width.saturating_mul(scale)) / scale / 2;
        let y =
            height.saturating_sub(Self::EARLY_BOOT_GLYPH_HEIGHT.saturating_mul(scale)) / scale / 2;
        (scale, x, y, text_fits)
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
        if flow_changed || personal_changed || force_draw || self.last_draw_states != Some(states) {
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
                        // Genesis owns this strip. Draw it after every personal
                        // command so the guest cannot cover recovery/secure UI.
                        draw_secure_strip(surface, layout, uptime_ms, &snapshot, false);
                        let scale = surface.draw_scale();
                        let personal = rect_from_layout(layout.personal_surface);
                        surface.present_rect(
                            personal.x.saturating_mul(scale),
                            personal.y.saturating_mul(scale),
                            personal.w.saturating_mul(scale),
                            personal.h.saturating_mul(scale),
                        );
                        let strip = rect_from_layout(layout.secure_strip);
                        surface.present_rect(
                            strip.x.saturating_mul(scale),
                            strip.y.saturating_mul(scale),
                            strip.w.saturating_mul(scale),
                            strip.h.saturating_mul(scale),
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
        let active = !self.personal.has_personal_focus() && console::composer_active();
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
                self.last_composer_cursor_rect = genesis_layout(surface.info())
                    .and_then(|layout| draw_composer_cursor_front(surface, layout, &snapshot));
            }
            draw_current_cursor(surface, &mut self.last_cursor_rect);
        }
        self.last_composer_cursor_phase = phase;
    }

    pub fn handle_pointer_interaction(
        &mut self,
        runtime: crate::system_status::RuntimeStatus,
    ) -> bool {
        let Some(surface) = self.surface.as_ref() else {
            return false;
        };
        let mouse = input::mouse_snapshot();
        let left_down = mouse.buttons & 1 != 0;
        let left_was_down = self.last_mouse_buttons & 1 != 0;
        self.last_mouse_buttons = mouse.buttons;
        if !mouse.seen || !left_down || left_was_down {
            return false;
        }

        let Some(layout) = genesis_layout(surface.info()) else {
            return false;
        };
        let x = mouse.x / 2;
        let y = mouse.y / 2;
        let width = layout.logical_size.width as usize;
        let height = layout.logical_size.height as usize;

        if self.vault.is_active() {
            return self.vault.handle_pointer(x, y, layout);
        }
        if self.personal.has_personal_focus() {
            return false;
        }
        if self.wifi.is_active() {
            return self.wifi.handle_pointer(x, y, width, height);
        }
        let view = console::snapshot().view;
        if view == console::UiView::Settings {
            return self.handle_setup_pointer(layout, x, y, width, height);
        }
        if point_in(x, y, recovery_strip_rect(layout)) {
            return self.toggle_recovery(runtime);
        }
        if self.recovery_open {
            return self.handle_recovery_pointer(layout, x, y, runtime);
        }
        if point_in(x, y, context_personal_shell_rect(layout)) {
            if agent_protocol_project_install::pending_physical_approval() {
                return agent_protocol_project_install::approve_from_pointer();
            }
            if granted_candidate_service::pending_approval() {
                return granted_candidate_service::approve_and_run_from_pointer();
            }
            if workspace_candidate_service::pending_approval() {
                return workspace_candidate_service::approve_and_run_from_pointer();
            }
            if let Some(program) = program_workspace::retained_program() {
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
                            console::write_event(format_args!(
                                "PROGRAM INSTALL READY DENIED: {reason}"
                            ));
                            None
                        }
                    }
                } else {
                    None
                };
                note_program_route(identity.sha256, route, approved.as_ref());
                return true;
            }
            return personal_shell_service::request_current_boot_proof_start(
                personal_shell_service::PersonalShellProofMode::Normal,
            );
        }
        if layout.composer.contains(Point::new(x as u32, y as u32)) {
            return console::set_view(console::UiView::Ai);
        }
        if point_in(x, y, context_setup_rect(layout)) {
            return open_setup();
        }
        if point_in(x, y, context_wifi_rect(layout)) {
            return self.wifi.begin();
        }
        false
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
        let Some(layout) = self
            .surface
            .as_ref()
            .and_then(|surface| genesis_layout(surface.info()))
        else {
            return false;
        };
        let max_scroll = conversation_max_scroll(layout, snapshot);
        let delta = match event.kind {
            input::InputEventKind::Relative(input::RelativeAxis::Wheel, value) => {
                let mouse = input::mouse_snapshot();
                let point = Point::new((mouse.x / 2) as u32, (mouse.y / 2) as u32);
                if !layout.conversation.contains(point) {
                    return false;
                }
                isize::try_from(value)
                    .unwrap_or(if value < 0 { isize::MIN } else { isize::MAX })
                    .saturating_mul(CONVERSATION_WHEEL_ROWS as isize)
            }
            input::InputEventKind::Key {
                code: 104,
                pressed: true,
            } => conversation_visible_rows(layout).saturating_sub(1) as isize,
            input::InputEventKind::Key {
                code: 109,
                pressed: true,
            } => -(conversation_visible_rows(layout).saturating_sub(1) as isize),
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
    uptime_ms: u64,
    snapshot: &SystemSnapshot,
    conversation_scroll_rows: usize,
    wifi: &wifi_flow::GuidedWifi,
    recovery: &recovery::RecoveryView,
    recovery_open: bool,
    vault: &mut vault_flow::VaultFlow,
) {
    let Some(layout) = genesis_layout(surface.info()) else {
        return;
    };
    surface.set_draw_scale(2);
    surface.fill(APP_BG);
    let console_snapshot = console::snapshot();
    draw_secure_strip(surface, layout, uptime_ms, snapshot, recovery_open);
    draw_conversation(surface, layout, &console_snapshot, conversation_scroll_rows);
    if recovery_open {
        recovery.draw_context(surface, layout, true);
    } else {
        draw_context(surface, layout, snapshot);
    }
    draw_composer(surface, layout, &console_snapshot);
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

fn draw_secure_strip(
    surface: &mut FramebufferSurface,
    layout: GenesisLayout,
    _uptime_ms: u64,
    snapshot: &SystemSnapshot,
    recovery_open: bool,
) {
    let rect = rect_from_layout(layout.secure_strip);
    surface.fill_rect(rect.x, rect.y, rect.w, rect.h, SURFACE_BG);
    surface.fill_rect(
        rect.x,
        rect.y + rect.h.saturating_sub(1),
        rect.w,
        1,
        HAIRLINE,
    );
    text::draw_text(surface, 12, 14, "raiOS / Genesis", TEXT_MAIN, None);
    let right = if recovery_open {
        "Recovery context / Click to close"
    } else if snapshot.network.state == RowState::Ready {
        "Core safe / Recovery ready"
    } else {
        "Core safe / Recovery available"
    };
    let right_x = rect.x + rect.w.saturating_sub(text_width(right) + 12);
    text::draw_text(surface, right_x, 14, right, TEXT_MUTED, None);
}

fn draw_conversation(
    surface: &mut FramebufferSurface,
    layout: GenesisLayout,
    snapshot: &console::ConsoleSnapshot,
    scroll_rows: usize,
) {
    let rect = rect_from_layout(layout.conversation);
    draw_panel(surface, rect, "Conversation");
    if !has_chat(snapshot) {
        text::draw_text(
            surface,
            rect.x + 18,
            rect.y + 54,
            "Welcome. What should your raiOS become?",
            TEXT_MAIN,
            None,
        );
        text::draw_text(
            surface,
            rect.x + 18,
            rect.y + 74,
            "Ask for a tool, workflow, or a change.",
            TEXT_MUTED,
            None,
        );
        return;
    }

    let max_chars = rect.w.saturating_sub(48) / FONT_ADVANCE;
    let visible_rows = conversation_visible_rows(layout);
    let total_rows = conversation_row_count(snapshot, max_chars);
    let scroll_rows = scroll_rows.min(total_rows.saturating_sub(visible_rows));
    let end_row = total_rows.saturating_sub(scroll_rows);
    let start_row = end_row.saturating_sub(visible_rows);
    let content_y = rect.y + 42;
    let mut row = 0usize;
    for line in snapshot.chat_lines {
        let text_value = line.text.as_str();
        if text_value.is_empty() {
            continue;
        }
        let color = match line.speaker {
            console::ChatSpeaker::User => APP_BLUE,
            console::ChatSpeaker::Assistant => TEXT_MAIN,
            console::ChatSpeaker::System => TEXT_MUTED,
        };
        let label = match line.speaker {
            console::ChatSpeaker::User => "You",
            console::ChatSpeaker::Assistant => "raiOS",
            console::ChatSpeaker::System => "System",
        };
        draw_conversation_row(
            surface,
            rect.x + 18,
            content_y,
            row,
            start_row,
            end_row,
            label,
            color,
        );
        row += 1;
        let body_color = if line.speaker == console::ChatSpeaker::Assistant {
            TEXT_MAIN
        } else {
            TEXT_MUTED
        };
        visit_wrapped_lines(text_value, max_chars, |wrapped| {
            draw_conversation_row(
                surface,
                rect.x + 18,
                content_y,
                row,
                start_row,
                end_row,
                wrapped,
                body_color,
            );
            row += 1;
        });
        row += 1;
    }
    draw_conversation_scrollbar(surface, rect, total_rows, visible_rows, start_row);
}

fn conversation_visible_rows(layout: GenesisLayout) -> usize {
    let rect = rect_from_layout(layout.conversation);
    rect.h
        .saturating_sub(54)
        .checked_div(CONVERSATION_ROW_HEIGHT)
        .unwrap_or(0)
        .max(1)
}

fn conversation_max_scroll(layout: GenesisLayout, snapshot: &console::ConsoleSnapshot) -> usize {
    let rect = rect_from_layout(layout.conversation);
    let max_chars = rect.w.saturating_sub(48) / FONT_ADVANCE;
    conversation_row_count(snapshot, max_chars).saturating_sub(conversation_visible_rows(layout))
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

#[allow(clippy::too_many_arguments)]
fn draw_conversation_row(
    surface: &mut FramebufferSurface,
    x: usize,
    content_y: usize,
    row: usize,
    start_row: usize,
    end_row: usize,
    value: &str,
    color: Color,
) {
    if row >= start_row && row < end_row {
        text::draw_text(
            surface,
            x,
            content_y + (row - start_row) * CONVERSATION_ROW_HEIGHT,
            value,
            color,
            None,
        );
    }
}

fn draw_conversation_scrollbar(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    total_rows: usize,
    visible_rows: usize,
    start_row: usize,
) {
    if total_rows <= visible_rows {
        return;
    }
    let track_y = rect.y + 42;
    let track_h = rect.h.saturating_sub(54);
    if track_h == 0 {
        return;
    }
    let thumb_h = (track_h * visible_rows / total_rows).max(12).min(track_h);
    let max_start = total_rows.saturating_sub(visible_rows).max(1);
    let thumb_y = track_y + (track_h - thumb_h) * start_row.min(max_start) / max_start;
    surface.fill_rect(
        rect.x + rect.w.saturating_sub(7),
        track_y,
        2,
        track_h,
        HAIRLINE,
    );
    surface.fill_rect(
        rect.x + rect.w.saturating_sub(8),
        thumb_y,
        4,
        thumb_h,
        APP_BLUE,
    );
}

fn draw_context(
    surface: &mut FramebufferSurface,
    layout: GenesisLayout,
    snapshot: &SystemSnapshot,
) {
    let rect = rect_from_layout(layout.context);
    draw_panel(surface, rect, "Context");
    let context = context::project(snapshot, &provider::snapshot(), wifi::snapshot());
    let problem_value = if context.problems.critical == 0 {
        "No critical problems"
    } else {
        "Critical problem present"
    };
    let problem_color = if context.problems.critical == 0 {
        APP_GREEN
    } else {
        APP_RED
    };
    let install_preview = agent_protocol_project_install::snapshot();
    let install_pending = agent_protocol_project_install::pending_physical_approval();
    let granted_preview = granted_candidate_service::approval_preview();
    let granted_pending = granted_preview.is_some();
    let workspace_preview = workspace_candidate_service::snapshot();
    let workspace_pending = workspace_candidate_service::pending_approval();
    let program_snapshot = program_workspace::snapshot();
    let program_ready = program_snapshot.present;
    draw_button(
        surface,
        context_personal_shell_rect(layout),
        if install_pending {
            if install_preview.install_source == Some("ui_program") {
                "Approve + persist program"
            } else {
                match install_preview.kind {
                    Some(agent_protocol_project_install::PreviewKind::Install) => {
                        "Approve + install app"
                    }
                    Some(agent_protocol_project_install::PreviewKind::Uninstall) => {
                        "Approve + uninstall app"
                    }
                    None => "Signed project action",
                }
            }
        } else if granted_pending {
            "Approve + run downloaded app"
        } else if workspace_pending {
            "Approve + run workspace app"
        } else if program_ready {
            "Approve + run program"
        } else {
            "Run signed shell proof"
        },
        install_pending || granted_pending || workspace_pending || program_ready,
    );
    let rows = [
        (
            "AI connection",
            context.ai_connection.value,
            context_tone_color(context.ai_connection.tone),
        ),
        (
            "Network",
            context.network.value,
            context_tone_color(context.network.tone),
        ),
        (
            "Secret Vault",
            vault_flow::VaultFlow::status_text(),
            TEXT_MUTED,
        ),
        ("Problems", problem_value, problem_color),
    ];
    let mut y = rect.y + 78;
    if install_pending {
        let downloaded = install_preview.install_source == Some("granted_candidate");
        let program = install_preview.install_source == Some("ui_program");
        text::draw_text(
            surface,
            rect.x + 14,
            y,
            if downloaded {
                "[INSTALL] Granted candidate"
            } else if program {
                "[PERSIST] RUIP program"
            } else {
                "[INSTALL] Signed W6 install"
            },
            APP_AMBER,
            None,
        );
        y = y.saturating_add(14);
        let effect = if downloaded {
            format!("generation {} / durable target", install_preview.generation)
        } else {
            format!(
                "{} generation {} / durable target",
                install_preview
                    .kind
                    .map(|kind| kind.label())
                    .unwrap_or("project"),
                install_preview.generation
            )
        };
        draw_truncated_text(
            surface,
            rect.x + 14,
            y,
            &effect,
            rect.w.saturating_sub(28) / FONT_ADVANCE,
            TEXT_MUTED,
        );
        y = y.saturating_add(14);
        let subject = install_preview
            .candidate_sha256
            .or(install_preview.previous_commit_sha256)
            .unwrap_or([0; 32]);
        draw_short_hash(
            surface,
            rect.x + 14,
            y,
            if downloaded {
                "candidate"
            } else if program {
                "program"
            } else {
                "project"
            },
            subject,
            TEXT_MUTED,
        );
        y = y.saturating_add(14);
        let binding = if downloaded || program {
            install_preview
                .activation_approval_sha256
                .unwrap_or([0; 32])
        } else {
            install_preview
                .receipt_sha256
                .or(install_preview.previous_commit_sha256)
                .unwrap_or([0; 32])
        };
        let binding_label = if downloaded || program {
            "approval"
        } else if install_preview.receipt_sha256.is_some() {
            "receipt"
        } else {
            "install head"
        };
        draw_short_hash(surface, rect.x + 14, y, binding_label, binding, TEXT_MUTED);
        y = y.saturating_add(14);
        text::draw_text(
            surface,
            rect.x + 14,
            y,
            "durable autostart / dev key not owner sealed",
            APP_AMBER,
            None,
        );
        y = y.saturating_add(20);
    } else if let Some(preview) = granted_preview {
        text::draw_text(
            surface,
            rect.x + 14,
            y,
            "[RUN] Granted candidate",
            APP_AMBER,
            None,
        );
        y = y.saturating_add(14);
        let candidate = preview.candidate_sha256;
        draw_short_hash(surface, rect.x + 14, y, "candidate", candidate, TEXT_MUTED);
        y = y.saturating_add(14);
        if let Some(receipt) = preview.receipt_sha256 {
            draw_short_hash(surface, rect.x + 14, y, "receipt", receipt, TEXT_MUTED);
        } else {
            text::draw_text(
                surface,
                rect.x + 14,
                y,
                "serial source / no receipt",
                TEXT_MUTED,
                None,
            );
        }
        y = y.saturating_add(20);
    } else if workspace_pending {
        text::draw_text(
            surface,
            rect.x + 14,
            y,
            "[RUN] Workspace candidate",
            APP_BLUE,
            None,
        );
        y = y.saturating_add(14);
        if let Some(binding) = workspace_preview.binding {
            let candidate = binding.candidate_sha256;
            draw_short_hash(surface, rect.x + 14, y, "candidate", candidate, TEXT_MUTED);
            y = y.saturating_add(14);
            let receipt = binding.receipt_sha256;
            draw_short_hash(surface, rect.x + 14, y, "receipt", receipt, TEXT_MUTED);
            y = y.saturating_add(20);
        }
    } else if program_ready {
        text::draw_text(
            surface,
            rect.x + 14,
            y,
            "[RUN + PERSIST] RUIP program",
            APP_GREEN,
            None,
        );
        y = y.saturating_add(14);
    }
    y = draw_program_retention(surface, rect, y, program_snapshot);
    let source_y = context_setup_rect(layout).y.saturating_sub(76);
    let source_visible = source_y >= y.saturating_add(4);
    for (label, value, color) in rows {
        if source_visible && y.saturating_add(31) > source_y.saturating_sub(4) {
            break;
        }
        text::draw_text(surface, rect.x + 14, y, label, TEXT_MUTED, None);
        draw_truncated_text(
            surface,
            rect.x + 14,
            y + 11,
            value,
            rect.w.saturating_sub(28) / FONT_ADVANCE,
            color,
        );
        y = y.saturating_add(31);
    }
    if source_visible {
        draw_source_status(surface, rect, source_y, &agent_build_loop::snapshot());
    }
    draw_button(surface, context_setup_rect(layout), "AI setup", false);
    draw_button(surface, context_wifi_rect(layout), "WiFi setup", true);
}

fn draw_source_status(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    y: usize,
    snapshot: &agent_build_loop::Snapshot,
) {
    let x = rect.x + 14;
    draw_source_value(
        surface,
        rect,
        y,
        "SOURCE / ",
        snapshot.phase.label(),
        APP_BLUE,
        TEXT_MAIN,
    );
    if let Some(revision) = snapshot.latest_revision.as_ref() {
        let hash = revision.revision_sha256;
        let line = format!(
            "rev {:02x}{:02x}{:02x}{:02x}{:02x}{:02x} files={}",
            hash[0],
            hash[1],
            hash[2],
            hash[3],
            hash[4],
            hash[5],
            revision.entries.len()
        );
        draw_truncated_text(
            surface,
            x,
            y + 13,
            &line,
            rect.w.saturating_sub(28) / FONT_ADVANCE,
            TEXT_MUTED,
        );
    } else {
        text::draw_text(surface, x, y + 13, "rev none", TEXT_FAINT, None);
    }
    let verifier = snapshot.verifier_result;
    let feedback = snapshot.feedback_packet;
    let rows = [
        (
            "origin ",
            snapshot.answer_origin.unwrap_or("none"),
            if snapshot.answer_origin.is_some() {
                TEXT_MUTED
            } else {
                TEXT_FAINT
            },
        ),
        (
            match verifier {
                Some(result) if result.passed => "check PASS ",
                Some(_) => "check FAIL ",
                None => "check ",
            },
            verifier.map(|result| result.reason).unwrap_or("not verified"),
            match verifier {
                Some(result) if result.passed => APP_GREEN,
                Some(_) => APP_RED,
                None => TEXT_FAINT,
            },
        ),
        (
            if feedback.is_some() {
                "feedback retained "
            } else {
                "feedback "
            },
            feedback.map(|packet| packet.reason).unwrap_or("none"),
            if feedback.is_some() {
                APP_AMBER
            } else {
                TEXT_FAINT
            },
        ),
    ];
    for (index, (label, value, color)) in rows.into_iter().enumerate() {
        draw_source_value(
            surface,
            rect,
            y + 25 + index * 12,
            label,
            value,
            TEXT_MUTED,
            color,
        );
    }
}

fn draw_source_value(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    y: usize,
    label: &str,
    value: &str,
    label_color: Color,
    color: Color,
) {
    let x = rect.x + 14;
    let value_x = x + text_width(label);
    text::draw_text(surface, x, y, label, label_color, None);
    draw_truncated_text(
        surface,
        value_x,
        y,
        value,
        rect.x
            .saturating_add(rect.w)
            .saturating_sub(value_x.saturating_add(14))
            / FONT_ADVANCE,
        color,
    );
}

fn draw_program_retention(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    mut y: usize,
    snapshot: program_workspace::Snapshot,
) -> usize {
    let Some(hash) = snapshot.sha256 else {
        return y;
    };
    let durable = snapshot.retention == "durable";
    draw_short_hash(
        surface,
        rect.x + 14,
        y,
        if durable {
            "Program installed:"
        } else {
            "Program:"
        },
        hash,
        if durable { APP_GREEN } else { TEXT_MUTED },
    );
    y = y.saturating_add(14);
    text::draw_text(
        surface,
        rect.x + 14,
        y,
        if durable {
            "durable (survives reboot)"
        } else {
            "current boot only"
        },
        if durable { APP_GREEN } else { TEXT_MUTED },
        None,
    );
    y.saturating_add(20)
}

fn draw_short_hash(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    label: &str,
    hash: [u8; 32],
    color: Color,
) {
    let line = format!(
        "{} {:02x}{:02x}{:02x}{:02x}...",
        label, hash[0], hash[1], hash[2], hash[3]
    );
    text::draw_text(surface, x, y, &line, color, None);
}

/// Renders an already accepted display-list within the logical personal area.
/// A second validation keeps the render boundary fail-closed if a caller ever
/// hands this surface stale or corrupt retained bytes.
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

fn context_tone_color(tone: context::ContextTone) -> Color {
    match tone {
        context::ContextTone::Neutral => TEXT_MUTED,
        context::ContextTone::Good => APP_GREEN,
        context::ContextTone::Attention => APP_AMBER,
        context::ContextTone::Critical => APP_RED,
    }
}

fn draw_composer(
    surface: &mut FramebufferSurface,
    layout: GenesisLayout,
    snapshot: &console::ConsoleSnapshot,
) {
    let rect = rect_from_layout(layout.composer);
    surface.fill_rect(rect.x, rect.y, rect.w, rect.h, SURFACE_ALT);
    draw_outline(surface, rect, HAIRLINE);
    let text_value = snapshot.chat_input.as_str();
    let text_x = rect.x + 14;
    let max_chars = rect.w.saturating_sub(58) / FONT_ADVANCE;
    if text_value.is_empty() {
        text::draw_text(
            surface,
            text_x,
            rect.y + 18,
            "Ask anything, or /build <program>...",
            TEXT_FAINT,
            None,
        );
    } else {
        let (visible, truncated) = trailing_chars(text_value, max_chars.saturating_sub(1));
        if truncated {
            text::draw_text(surface, text_x, rect.y + 18, "<", TEXT_FAINT, None);
            text::draw_text(
                surface,
                text_x + FONT_ADVANCE,
                rect.y + 18,
                visible,
                TEXT_MAIN,
                None,
            );
        } else {
            text::draw_text(surface, text_x, rect.y + 18, visible, TEXT_MAIN, None);
        }
    }
    draw_button(
        surface,
        LogicalRect::new(rect.x + rect.w.saturating_sub(38), rect.y + 8, 30, 30),
        ">",
        true,
    );
}

fn composer_cursor_phase(uptime_ms: u64) -> bool {
    (uptime_ms / COMPOSER_CURSOR_BLINK_MS) % 2 == 0
}

fn draw_composer_cursor_front(
    surface: &mut FramebufferSurface,
    layout: GenesisLayout,
    snapshot: &console::ConsoleSnapshot,
) -> Option<CursorRect> {
    if snapshot.view != console::UiView::Ai || snapshot.focus != console::UiFocus::ChatInput {
        return None;
    }
    let rect = rect_from_layout(layout.composer);
    let max_chars = rect.w.saturating_sub(58) / FONT_ADVANCE;
    let text_value = snapshot.chat_input.as_str();
    let cursor_chars = if text_value.is_empty() {
        0
    } else {
        let (visible, truncated) = trailing_chars(text_value, max_chars.saturating_sub(1));
        visible.chars().count() + usize::from(truncated)
    };
    let scale = surface.draw_scale();
    let cursor = CursorRect {
        x: (rect.x + 14 + cursor_chars * FONT_ADVANCE).saturating_mul(scale),
        y: (rect.y + 15).saturating_mul(scale),
        w: 2usize.saturating_mul(scale),
        h: 15usize.saturating_mul(scale),
    };
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

fn trailing_chars(value: &str, max_chars: usize) -> (&str, bool) {
    let count = value.chars().count();
    if count <= max_chars {
        return (value, false);
    }
    let skip = count - max_chars;
    let start = value
        .char_indices()
        .nth(skip)
        .map(|(index, _)| index)
        .unwrap_or(value.len());
    (&value[start..], true)
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

fn context_setup_rect(layout: GenesisLayout) -> LogicalRect {
    let context = rect_from_layout(layout.context);
    LogicalRect::new(
        context.x + 12,
        context.y + context.h.saturating_sub(62),
        context.w - 24,
        22,
    )
}

fn context_personal_shell_rect(layout: GenesisLayout) -> LogicalRect {
    let context = rect_from_layout(layout.context);
    LogicalRect::new(context.x + 12, context.y + 42, context.w - 24, 24)
}

fn context_wifi_rect(layout: GenesisLayout) -> LogicalRect {
    let context = rect_from_layout(layout.context);
    LogicalRect::new(
        context.x + 12,
        context.y + context.h.saturating_sub(34),
        context.w - 24,
        22,
    )
}

fn recovery_strip_rect(layout: GenesisLayout) -> LogicalRect {
    let strip = rect_from_layout(layout.secure_strip);
    LogicalRect::new(strip.x + strip.w.saturating_sub(238), strip.y, 238, strip.h)
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

fn has_chat(snapshot: &console::ConsoleSnapshot) -> bool {
    snapshot
        .chat_lines
        .iter()
        .any(|line| !line.text.as_str().is_empty())
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
