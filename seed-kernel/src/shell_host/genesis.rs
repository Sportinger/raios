//! Core-owned Genesis presentation.  It renders only typed snapshots and delegates
//! existing setup actions to the current console/provider adapters.

use crate::agent_protocol::recovery_lifeline;
use crate::framebuffer::{Color, FramebufferInfo, FramebufferSurface};
use crate::system_status::{RowState, SnapshotStates, StatusLine, SystemSnapshot};
use crate::{console, input, personal_shell_service, provider, secret_vault, serial, text, wifi};
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
    wifi: wifi_flow::GuidedWifi,
    recovery: recovery::RecoveryView,
    recovery_open: bool,
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
            wifi: wifi_flow::GuidedWifi::new(),
            recovery: recovery::RecoveryView::new(),
            recovery_open: false,
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
                    } else {
                        self.personal.exit();
                        console::write_event(format_args!(
                            "PERSONAL SHELL FALLBACK: validated frame unavailable"
                        ));
                        draw_genesis(
                            surface,
                            uptime_ms,
                            &snapshot,
                            &self.wifi,
                            &self.recovery,
                            self.recovery_open,
                            &mut self.vault,
                        );
                    }
                } else {
                    draw_genesis(
                        surface,
                        uptime_ms,
                        &snapshot,
                        &self.wifi,
                        &self.recovery,
                        self.recovery_open,
                        &mut self.vault,
                    );
                }
                surface.present();
                self.vault.note_presented();
                self.last_cursor_rect = None;
                draw_current_cursor(surface, &mut self.last_cursor_rect);
                self.last_draw_states = Some(states);
            }
        }
    }

    pub fn render_pointer(&mut self) {
        if let Some(surface) = self.surface.as_mut() {
            if let Some(rect) = self.last_cursor_rect.take() {
                surface.restore_from_back_rect(rect.x, rect.y, rect.w, rect.h);
            }
            draw_current_cursor(surface, &mut self.last_cursor_rect);
        }
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
            if self.personal.has_personal_focus() {
                let route = self.personal.handle_secure_attention();
                note_personal_route(route);
                return true;
            }
            return self.toggle_recovery(runtime);
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
        let Some(event) = sanitize_personal_input(event) else {
            return true;
        };
        let route = self.personal.route_sanitized_event(context, event);
        note_personal_route(route);
        true
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
    draw_conversation(surface, layout, &console_snapshot);
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

    let mut y = rect.y + rect.h.saturating_sub(24);
    let mut index = snapshot.chat_lines.len();
    while index > 0 && y > rect.y + 42 {
        index -= 1;
        let line = snapshot.chat_lines[index];
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
        let line_y = y.saturating_sub(18);
        text::draw_text(surface, rect.x + 18, line_y, label, color, None);
        draw_truncated_text(
            surface,
            rect.x + 18,
            line_y + 11,
            text_value,
            rect.w.saturating_sub(36) / FONT_ADVANCE,
            TEXT_MUTED,
        );
        y = line_y.saturating_sub(18);
    }
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
    draw_button(
        surface,
        context_personal_shell_rect(layout),
        "Run signed shell proof",
        false,
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
    for (label, value, color) in rows {
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
    draw_button(surface, context_setup_rect(layout), "AI setup", false);
    draw_button(surface, context_wifi_rect(layout), "WiFi setup", true);
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

fn sanitize_personal_input(event: input::InputEvent) -> Option<SanitizedInputEvent> {
    let (kind, pressed, code, x, y, dx, dy) = match event.kind {
        input::InputEventKind::SecureAttention => return None,
        input::InputEventKind::Key { code, pressed } if (272..=274).contains(&code) => {
            (SanitizedInputKind::PointerButton, pressed, code, 0, 0, 0, 0)
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
        input::InputEventKind::Absolute { x, y, .. } => (
            SanitizedInputKind::PointerMove,
            false,
            0,
            x.min(i16::MAX as u16) as i16,
            y.min(i16::MAX as u16) as i16,
            0,
            0,
        ),
    };
    Some(SanitizedInputEvent::new(
        kind, pressed, false, code, x, y, dx, dy, 0,
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
            console::write_event(format_args!("PERSONAL SHELL FRAME UPDATED sanitized_input"));
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
    if text_value.is_empty() {
        text::draw_text(
            surface,
            rect.x + 14,
            rect.y + 18,
            "Describe what raiOS should become...",
            TEXT_FAINT,
            None,
        );
    } else {
        draw_truncated_text(
            surface,
            rect.x + 14,
            rect.y + 18,
            text_value,
            rect.w.saturating_sub(58) / FONT_ADVANCE,
            TEXT_MAIN,
        );
    }
    draw_button(
        surface,
        LogicalRect::new(rect.x + rect.w.saturating_sub(38), rect.y + 8, 30, 30),
        ">",
        true,
    );
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
    let shape = [
        "X", "XX", "XOX", "XOOX", "XOOOX", "XOOOOX", "XOOOOOX", "XX  XOX", "X    X",
    ];
    for (row, pattern) in shape.iter().enumerate() {
        for (col, pixel) in pattern.bytes().enumerate() {
            let color = match pixel {
                b'X' => outline,
                b'O' => fill,
                _ => continue,
            };
            surface.set_front_pixel(x + col, y + row, color);
        }
    }
    *last_rect = Some(CursorRect { x, y, w: 10, h: 10 });
}
