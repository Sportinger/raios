use crate::framebuffer::{Color, FramebufferInfo, FramebufferSurface};
use crate::system_status::{RowState, SnapshotStates, StatusLine, SystemSnapshot, TextBuf};
use crate::{console, input, marvell_wifi_pcie, serial, text, wifi};
use core::fmt::{self, Write};
use raios_core::dot11_scan::Dot11Security;

pub use crate::system_status::RuntimeStatus;

const FONT_ADVANCE: usize = 9;
const CHAT_LINE_HEIGHT: usize = 15;
const CHAT_BUBBLE_PADDING_X: usize = 16;
const CHAT_BUBBLE_PADDING_TOP: usize = 10;
const CHAT_BUBBLE_LABEL_HEIGHT: usize = 10;
const CHAT_BUBBLE_LABEL_GAP: usize = 6;
const CHAT_BUBBLE_PADDING_BOTTOM: usize = 12;
const CHAT_BUBBLE_GAP: usize = 16;
const CURSOR_WIDTH: usize = 10;
const CURSOR_HEIGHT: usize = 16;
const STATUS_DETAIL_START_Y: usize = 120;
const STATUS_DETAIL_LINE_H: usize = 16;
const STATUS_DETAIL_LABEL_X: usize = 44;
const STATUS_DETAIL_VALUE_X: usize = 132;
const STATUS_HAIRLINE_Y: usize = 274;
const CONTENT_TOP: usize = STATUS_HAIRLINE_Y + 6;
const PANEL_TITLE_H: usize = 44;
const HEADER_TAB_START_X: usize = 236;
const HEADER_TAB_GAP: usize = 24;
const HEADER_TAB_HIT_Y: usize = 18;
const HEADER_TAB_HIT_H: usize = 58;
const HEADER_TAB_LABEL_Y: usize = 36;
const HEADER_TAB_UNDERLINE_Y: usize = 73;
const INPUT_FIELD_X: usize = 24;
const INPUT_FIELD_RIGHT: usize = 72;
const INPUT_FIELD_H: usize = 36;
const SETTINGS_ACTION_H: usize = 38;
const SETTINGS_ACTION_ROW_STEP: usize = 54;
const WIFI_FLOW_LIST_LIMIT: usize = 8;
const WIFI_FLOW_LIST_ROW_H: usize = 42;
const R8_INSETS: [usize; 8] = [8, 4, 3, 2, 1, 1, 0, 0];
const R6_INSETS: [usize; 6] = [6, 3, 2, 1, 0, 0];

const APP_BG: Color = Color::new(17, 18, 22);
const SURFACE_BG: Color = Color::new(26, 28, 33);
const SURFACE_ALT: Color = Color::new(36, 39, 45);
const HAIRLINE: Color = Color::new(45, 49, 56);
const HAIRLINE_HI: Color = Color::new(66, 71, 80);
const TEXT_MAIN: Color = Color::new(232, 236, 241);
const TEXT_MUTED: Color = Color::new(156, 164, 175);
const TEXT_FAINT: Color = Color::new(106, 114, 126);
const APP_BLUE: Color = Color::new(10, 132, 255);
const APP_GREEN: Color = Color::new(52, 199, 89);
const APP_AMBER: Color = Color::new(255, 159, 10);
const APP_RED: Color = Color::new(255, 69, 58);
const USER_BUBBLE: Color = Color::new(21, 93, 204);

#[derive(Clone, Copy, PartialEq, Eq)]
enum WifiFlow {
    Idle,
    Starting {
        scan_started: bool,
    },
    Selecting,
    Password {
        network_index: usize,
        remember_for_boot: bool,
        rejected: bool,
    },
    Configured {
        network_index: usize,
        remember_for_boot: bool,
    },
    Failed(&'static str),
}

#[derive(Clone, Copy)]
struct LogicalRect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

pub struct StatusUi {
    surface: Option<FramebufferSurface>,
    last_states: Option<SnapshotStates>,
    last_draw_states: Option<SnapshotStates>,
    last_mouse_buttons: u8,
    last_cursor_rect: Option<CursorRect>,
    wifi_chip_rect: Option<LogicalRect>,
    wifi_flow: WifiFlow,
}

impl StatusUi {
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
            wifi_chip_rect: None,
            wifi_flow: WifiFlow::Idle,
        }
    }

    pub fn render(&mut self, uptime_ms: u64, runtime: RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, false);
    }

    pub fn render_forced(&mut self, uptime_ms: u64, runtime: RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, true);
    }

    fn render_inner(&mut self, uptime_ms: u64, runtime: RuntimeStatus, force_draw: bool) {
        let flow_changed = self.advance_wifi_flow();
        let framebuffer = self.surface.as_ref().map(|surface| surface.info());
        let snapshot = SystemSnapshot::collect(framebuffer, runtime);
        self.log_transitions(&snapshot);
        self.wifi_chip_rect = Some(wifi_status_chip_rect(&snapshot));

        let states = snapshot.states();
        let should_draw = flow_changed || force_draw || self.last_draw_states != Some(states);

        if should_draw {
            if let Some(surface) = self.surface.as_mut() {
                draw(
                    surface,
                    uptime_ms,
                    &snapshot,
                    self.wifi_flow,
                    self.wifi_chip_rect,
                );
                surface.present();
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

    pub fn handle_pointer_interaction(&mut self) -> bool {
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

        let scale = display_scale(surface.info());
        let width = logical_width(surface.info());
        let height = logical_height(surface.info());
        let x = mouse.x / scale;
        let y = mouse.y / scale;

        if self.wifi_flow != WifiFlow::Idle {
            return self.handle_wifi_flow_pointer(x, y, width, height);
        }
        if let Some(rect) = self.wifi_chip_rect {
            if point_in(x, y, rect.x, rect.y, rect.w, rect.h)
                && wifi::snapshot().state == wifi::WifiState::Detected
            {
                return self.begin_wifi_flow();
            }
        }

        let ai_tab_x = header_tab_x(0);
        let ai_tab_w = header_tab_width("AI");
        let console_tab_x = header_tab_x(1);
        let console_tab_w = header_tab_width("CONSOLE");
        let settings_tab_x = header_tab_x(2);
        let settings_tab_w = header_tab_width("SET");
        if point_in(x, y, ai_tab_x, HEADER_TAB_HIT_Y, ai_tab_w, HEADER_TAB_HIT_H) {
            return console::set_view(console::UiView::Ai);
        }
        if point_in(
            x,
            y,
            console_tab_x,
            HEADER_TAB_HIT_Y,
            console_tab_w,
            HEADER_TAB_HIT_H,
        ) {
            return console::set_view(console::UiView::Console);
        }
        if point_in(
            x,
            y,
            settings_tab_x,
            HEADER_TAB_HIT_Y,
            settings_tab_w,
            HEADER_TAB_HIT_H,
        ) {
            return console::set_view(console::UiView::Settings);
        }

        let snapshot = console::snapshot();
        if snapshot.view == console::UiView::Ai {
            let input_y = input_field_y(logical_height(surface.info()));
            if point_in(
                x,
                y,
                INPUT_FIELD_X,
                input_y,
                input_field_width(width),
                INPUT_FIELD_H,
            ) {
                return console::set_view(console::UiView::Ai);
            }
        }
        if snapshot.view == console::UiView::Settings && !snapshot.settings_entry_active {
            let top = CONTENT_TOP;
            let (action_x, action_x2, action_w) = settings_action_columns(width);
            let action_y = top + 62;
            if point_in(x, y, action_x, action_y, action_w, SETTINGS_ACTION_H) {
                return console::activate_focus(console::UiFocus::SettingsProvider);
            }
            if point_in(x, y, action_x2, action_y, action_w, SETTINGS_ACTION_H) {
                return console::activate_focus(console::UiFocus::SettingsApiKey);
            }
            if point_in(
                x,
                y,
                action_x,
                action_y + SETTINGS_ACTION_ROW_STEP,
                action_w,
                SETTINGS_ACTION_H,
            ) {
                return console::activate_focus(console::UiFocus::SettingsClear);
            }
            if point_in(
                x,
                y,
                action_x2,
                action_y + SETTINGS_ACTION_ROW_STEP,
                action_w,
                SETTINGS_ACTION_H,
            ) {
                return console::activate_focus(console::UiFocus::SettingsWifiSsid);
            }
            if point_in(
                x,
                y,
                action_x,
                action_y + SETTINGS_ACTION_ROW_STEP * 2,
                action_w,
                SETTINGS_ACTION_H,
            ) {
                return console::activate_focus(console::UiFocus::SettingsWifiPassphrase);
            }
            if point_in(
                x,
                y,
                action_x2,
                action_y + SETTINGS_ACTION_ROW_STEP * 2,
                action_w,
                SETTINGS_ACTION_H,
            ) {
                return console::activate_focus(console::UiFocus::SettingsWifiClear);
            }
            if point_in(
                x,
                y,
                action_x,
                action_y + SETTINGS_ACTION_ROW_STEP * 3,
                action_w,
                SETTINGS_ACTION_H,
            ) {
                return console::activate_focus(console::UiFocus::SettingsWifiFirmware);
            }
            if point_in(
                x,
                y,
                action_x2,
                action_y + SETTINGS_ACTION_ROW_STEP * 3,
                action_w,
                SETTINGS_ACTION_H,
            ) {
                return console::activate_focus(console::UiFocus::SettingsWifiScan);
            }
            if point_in(
                x,
                y,
                action_x,
                action_y + SETTINGS_ACTION_ROW_STEP * 4,
                action_w,
                SETTINGS_ACTION_H,
            ) {
                return console::activate_focus(console::UiFocus::SettingsClose);
            }
        }

        false
    }

    fn begin_wifi_flow(&mut self) -> bool {
        if wifi::snapshot().state != wifi::WifiState::Detected {
            self.wifi_flow = WifiFlow::Failed("wifi_device_not_detected");
            return true;
        }

        let firmware = marvell_wifi_pcie::snapshot();
        if firmware.is_failed() {
            self.wifi_flow = WifiFlow::Failed(
                firmware
                    .result
                    .map(|result| result.label())
                    .unwrap_or("firmware_failed"),
            );
            return true;
        }
        self.wifi_flow = WifiFlow::Starting {
            scan_started: false,
        };
        if firmware.is_ready() || firmware.running {
            return true;
        }

        let result = marvell_wifi_pcie::start_bring_up_firmware();
        console::write_event(format_args!("WIFI UI START: {}", result.label()));
        match result {
            marvell_wifi_pcie::FirmwareBringupTriggerResult::Started
            | marvell_wifi_pcie::FirmwareBringupTriggerResult::AlreadyRunning => true,
            marvell_wifi_pcie::FirmwareBringupTriggerResult::AlreadyAttempted => {
                self.wifi_flow = WifiFlow::Failed("firmware_already_attempted");
                true
            }
            marvell_wifi_pcie::FirmwareBringupTriggerResult::Failed(error) => {
                self.wifi_flow = WifiFlow::Failed(error.label());
                true
            }
        }
    }

    fn advance_wifi_flow(&mut self) -> bool {
        match self.wifi_flow {
            WifiFlow::Starting { scan_started } => {
                let firmware = marvell_wifi_pcie::snapshot();
                if firmware.is_failed() {
                    self.wifi_flow = WifiFlow::Failed(
                        firmware
                            .result
                            .map(|result| result.label())
                            .unwrap_or("firmware_failed"),
                    );
                    return true;
                }
                if !firmware.is_ready() {
                    return false;
                }

                let hw_spec = marvell_wifi_pcie::hw_spec_snapshot();
                if hw_spec.is_failed() {
                    self.wifi_flow = WifiFlow::Failed(
                        hw_spec
                            .result
                            .map(|result| result.label())
                            .unwrap_or("hw_spec_failed"),
                    );
                    return true;
                }
                if !hw_spec.is_ready() {
                    return false;
                }

                if !scan_started {
                    let result = marvell_wifi_pcie::start_scan_ext_24ghz();
                    console::write_event(format_args!("WIFI UI SCAN: {}", result.label()));
                    return match result {
                        marvell_wifi_pcie::ScanCmdTriggerResult::Started
                        | marvell_wifi_pcie::ScanCmdTriggerResult::AlreadyRunning => {
                            self.wifi_flow = WifiFlow::Starting { scan_started: true };
                            true
                        }
                        marvell_wifi_pcie::ScanCmdTriggerResult::Failed(error) => {
                            self.wifi_flow = WifiFlow::Failed(error.label());
                            true
                        }
                    };
                }

                let scan = marvell_wifi_pcie::scan_cmd_snapshot();
                if scan.stage == marvell_wifi_pcie::ScanCmdStage::Failed {
                    self.wifi_flow = WifiFlow::Failed(
                        scan.result
                            .map(|result| result.label())
                            .unwrap_or("scan_failed"),
                    );
                    return true;
                }
                if scan.stage == marvell_wifi_pcie::ScanCmdStage::Done {
                    self.wifi_flow = WifiFlow::Selecting;
                    return true;
                }
                false
            }
            WifiFlow::Password {
                network_index,
                remember_for_boot,
                ..
            } => {
                let console_snapshot = console::snapshot();
                match console_snapshot.wifi_passphrase_entry_result {
                    console::WifiPassphraseEntryResult::Set => {
                        self.wifi_flow = WifiFlow::Configured {
                            network_index,
                            remember_for_boot,
                        };
                        true
                    }
                    console::WifiPassphraseEntryResult::Cancelled => {
                        self.wifi_flow = WifiFlow::Selecting;
                        true
                    }
                    console::WifiPassphraseEntryResult::Rejected => {
                        let _ = console::activate_focus(console::UiFocus::SettingsWifiPassphrase);
                        self.wifi_flow = WifiFlow::Password {
                            network_index,
                            remember_for_boot,
                            rejected: true,
                        };
                        true
                    }
                    console::WifiPassphraseEntryResult::None => false,
                }
            }
            WifiFlow::Idle
            | WifiFlow::Selecting
            | WifiFlow::Configured { .. }
            | WifiFlow::Failed(_) => false,
        }
    }

    fn handle_wifi_flow_pointer(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> bool {
        match self.wifi_flow {
            WifiFlow::Idle | WifiFlow::Starting { .. } => false,
            WifiFlow::Selecting => {
                let scan = wifi::scan_results();
                let rect = wifi_selection_dialog_rect(
                    width,
                    height,
                    scan.count,
                    self.wifi_chip_rect.map(|rect| rect.x).unwrap_or(24),
                );
                let visible = usize::min(scan.count, WIFI_FLOW_LIST_LIMIT);
                let mut index = 0usize;
                while index < visible {
                    let row_y = rect.y + 66 + index * WIFI_FLOW_LIST_ROW_H;
                    if point_in(x, y, rect.x + 24, row_y, rect.w - 48, 34) {
                        let network = scan.networks[index];
                        if network.hidden_ssid || network.ssid.is_empty() {
                            self.wifi_flow = WifiFlow::Failed("hidden_ssid_requires_manual_entry");
                            return true;
                        }
                        wifi::clear_config();
                        if wifi::set_ssid(network.ssid.as_bytes()).is_err() {
                            self.wifi_flow = WifiFlow::Failed("ssid_rejected");
                            return true;
                        }
                        if network.security == Dot11Security::Open {
                            self.wifi_flow = WifiFlow::Configured {
                                network_index: index,
                                remember_for_boot: false,
                            };
                            return true;
                        }
                        wifi::set_remember_passphrase_for_boot(true);
                        let _ = console::activate_focus(console::UiFocus::SettingsWifiPassphrase);
                        self.wifi_flow = WifiFlow::Password {
                            network_index: index,
                            remember_for_boot: true,
                            rejected: false,
                        };
                        return true;
                    }
                    index += 1;
                }

                let (left, right) = wifi_dialog_action_rects(rect);
                if point_in(x, y, left.x, left.y, left.w, left.h) {
                    self.wifi_flow = WifiFlow::Starting {
                        scan_started: false,
                    };
                    return true;
                }
                if point_in(x, y, right.x, right.y, right.w, right.h) {
                    self.wifi_flow = WifiFlow::Idle;
                    return true;
                }
                false
            }
            WifiFlow::Password {
                network_index,
                remember_for_boot,
                rejected,
            } => {
                let rect = wifi_password_dialog_rect(width, height);
                if point_in(x, y, rect.x + 28, rect.y + 150, rect.w - 56, 34) {
                    let remember_for_boot = !remember_for_boot;
                    wifi::set_remember_passphrase_for_boot(remember_for_boot);
                    self.wifi_flow = WifiFlow::Password {
                        network_index,
                        remember_for_boot,
                        rejected,
                    };
                    return true;
                }

                let (left, right) = wifi_dialog_action_rects(rect);
                if point_in(x, y, left.x, left.y, left.w, left.h) {
                    let _ = console::cancel_wifi_passphrase_entry();
                    self.wifi_flow = WifiFlow::Selecting;
                    return true;
                }
                if point_in(x, y, right.x, right.y, right.w, right.h) {
                    wifi::set_remember_passphrase_for_boot(remember_for_boot);
                    return console::submit_wifi_passphrase_entry();
                }
                false
            }
            WifiFlow::Configured { .. } => {
                let rect = wifi_result_dialog_rect(width, height);
                let (_, right) = wifi_dialog_action_rects(rect);
                if point_in(x, y, right.x, right.y, right.w, right.h) {
                    self.wifi_flow = WifiFlow::Idle;
                    return true;
                }
                false
            }
            WifiFlow::Failed(_) => {
                let rect = wifi_result_dialog_rect(width, height);
                let (left, right) = wifi_dialog_action_rects(rect);
                if point_in(x, y, left.x, left.y, left.w, left.h) {
                    return self.begin_wifi_flow();
                }
                if point_in(x, y, right.x, right.y, right.w, right.h) {
                    self.wifi_flow = WifiFlow::Idle;
                    return true;
                }
                false
            }
        }
    }

    fn log_transitions(&mut self, snapshot: &SystemSnapshot) {
        let states = snapshot.states();
        let previous = self.last_states;

        log_transition(previous.map(|prev| prev.framebuffer), &snapshot.framebuffer);
        log_transition(previous.map(|prev| prev.entropy), &snapshot.entropy);
        log_transition(previous.map(|prev| prev.usb_xhci), &snapshot.usb_xhci);
        log_transition(previous.map(|prev| prev.usb_hotplug), &snapshot.usb_hotplug);
        log_transition(previous.map(|prev| prev.wifi), &snapshot.wifi);
        log_transition(previous.map(|prev| prev.network), &snapshot.network);
        log_transition(previous.map(|prev| prev.input), &snapshot.input);
        log_transition(previous.map(|prev| prev.iommu), &snapshot.iommu);

        self.last_states = Some(states);
    }
}

fn point_in(px: usize, py: usize, x: usize, y: usize, w: usize, h: usize) -> bool {
    px >= x && px < x.saturating_add(w) && py >= y && py < y.saturating_add(h)
}

fn centered_rect(
    screen_width: usize,
    screen_height: usize,
    width: usize,
    height: usize,
) -> LogicalRect {
    let width = usize::min(width, screen_width.saturating_sub(48));
    let height = usize::min(height, screen_height.saturating_sub(96));
    LogicalRect {
        x: screen_width.saturating_sub(width) / 2,
        y: usize::max(88, screen_height.saturating_sub(height) / 2),
        w: width,
        h: height,
    }
}

fn wifi_selection_dialog_rect(
    screen_width: usize,
    screen_height: usize,
    network_count: usize,
    anchor_x: usize,
) -> LogicalRect {
    let visible = usize::min(network_count, WIFI_FLOW_LIST_LIMIT);
    let mut rect = centered_rect(
        screen_width,
        screen_height,
        560,
        170usize.saturating_add(visible.saturating_mul(WIFI_FLOW_LIST_ROW_H)),
    );
    rect.x = usize::min(
        usize::max(24, anchor_x),
        screen_width.saturating_sub(rect.w).saturating_sub(24),
    );
    if 128usize.saturating_add(rect.h).saturating_add(24) <= screen_height {
        rect.y = 128;
    }
    rect
}

fn wifi_password_dialog_rect(screen_width: usize, screen_height: usize) -> LogicalRect {
    centered_rect(screen_width, screen_height, 560, 260)
}

fn wifi_result_dialog_rect(screen_width: usize, screen_height: usize) -> LogicalRect {
    centered_rect(screen_width, screen_height, 520, 210)
}

fn wifi_progress_dialog_rect(screen_width: usize, screen_height: usize) -> LogicalRect {
    centered_rect(screen_width, screen_height, 520, 190)
}

fn wifi_dialog_action_rects(rect: LogicalRect) -> (LogicalRect, LogicalRect) {
    let gap = 14usize;
    let width = rect.w.saturating_sub(62).saturating_sub(gap) / 2;
    let y = rect.y.saturating_add(rect.h).saturating_sub(50);
    (
        LogicalRect {
            x: rect.x + 24,
            y,
            w: width,
            h: 34,
        },
        LogicalRect {
            x: rect.x + 24 + width + gap,
            y,
            w: width,
            h: 34,
        },
    )
}

fn status_chip_width(label: &str, line: &StatusLine) -> usize {
    28usize
        .saturating_add(label.len().saturating_mul(FONT_ADVANCE))
        .saturating_add(8)
        .saturating_add(row_state_text_len(line.state).saturating_mul(FONT_ADVANCE))
        .saturating_add(14)
}

fn wifi_status_chip_rect(snapshot: &SystemSnapshot) -> LogicalRect {
    LogicalRect {
        x: 24usize
            .saturating_add(status_chip_width("Net", &snapshot.network))
            .saturating_add(12),
        y: 90,
        w: status_chip_width("WiFi", &snapshot.wifi),
        h: 30,
    }
}

fn settings_action_columns(width: usize) -> (usize, usize, usize) {
    let x = usize::max(430, width / 2);
    let gap = 16usize;
    let button_w = width.saturating_sub(x.saturating_add(72).saturating_add(gap)) / 2;
    (x, x.saturating_add(button_w).saturating_add(gap), button_w)
}

#[derive(Clone, Copy)]
struct CursorRect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
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

fn draw(
    surface: &mut FramebufferSurface,
    uptime_ms: u64,
    snapshot: &SystemSnapshot,
    wifi_flow: WifiFlow,
    wifi_chip_rect: Option<LogicalRect>,
) {
    let info = surface.info();
    let scale = display_scale(info);
    surface.set_draw_scale(scale);
    let width = logical_width(info);
    let height = logical_height(info);
    let console_snapshot = console::snapshot();

    surface.fill(APP_BG);
    draw_header(surface, width, &console_snapshot);
    draw_status_strip(surface, width, uptime_ms, snapshot);
    draw_status_detail(surface, width, console_snapshot.view, snapshot);

    match console_snapshot.view {
        console::UiView::Ai => draw_chat(surface, width, height, &console_snapshot),
        console::UiView::Console => draw_console(surface, width, height, &console_snapshot),
        console::UiView::Settings => draw_settings(surface, width, height, &console_snapshot),
    }

    draw_wifi_flow(
        surface,
        width,
        height,
        wifi_flow,
        wifi_chip_rect,
        &console_snapshot,
    );
}

fn display_scale(info: FramebufferInfo) -> usize {
    if info.width >= 2200 || info.height >= 1400 {
        2
    } else {
        1
    }
}

fn logical_width(info: FramebufferInfo) -> usize {
    usize::max(1, info.width as usize / display_scale(info))
}

fn logical_height(info: FramebufferInfo) -> usize {
    usize::max(1, info.height as usize / display_scale(info))
}

fn draw_header(
    surface: &mut FramebufferSurface,
    width: usize,
    snapshot: &console::ConsoleSnapshot,
) {
    surface.fill_rect(0, 0, width, 76, SURFACE_BG);
    surface.fill_rect(0, 75, width, 1, HAIRLINE_HI);

    text::draw_text(surface, 24, HEADER_TAB_LABEL_Y, "raiOS", TEXT_MAIN, None);
    text::draw_text(
        surface,
        84,
        HEADER_TAB_LABEL_Y,
        "Direct AI Host",
        TEXT_FAINT,
        None,
    );

    draw_tab(
        surface,
        header_tab_x(0),
        "AI",
        snapshot.view == console::UiView::Ai,
    );
    draw_tab(
        surface,
        header_tab_x(1),
        "CONSOLE",
        snapshot.view == console::UiView::Console,
    );
    draw_tab(
        surface,
        header_tab_x(2),
        "SET",
        snapshot.view == console::UiView::Settings,
    );
}

fn header_tab_width(label: &str) -> usize {
    label.len().saturating_mul(FONT_ADVANCE).saturating_add(24)
}

fn header_tab_x(index: usize) -> usize {
    match index {
        0 => HEADER_TAB_START_X,
        1 => HEADER_TAB_START_X
            .saturating_add(header_tab_width("AI"))
            .saturating_add(HEADER_TAB_GAP),
        _ => HEADER_TAB_START_X
            .saturating_add(header_tab_width("AI"))
            .saturating_add(HEADER_TAB_GAP)
            .saturating_add(header_tab_width("CONSOLE"))
            .saturating_add(HEADER_TAB_GAP),
    }
}

fn draw_tab(surface: &mut FramebufferSurface, x: usize, label: &str, active: bool) {
    text::draw_text(
        surface,
        x + 12,
        HEADER_TAB_LABEL_Y,
        label,
        if active { TEXT_MAIN } else { TEXT_MUTED },
        None,
    );
    if active {
        surface.fill_rect(
            x,
            HEADER_TAB_UNDERLINE_Y,
            header_tab_width(label),
            3,
            APP_BLUE,
        );
    }
}

fn draw_status_strip(
    surface: &mut FramebufferSurface,
    width: usize,
    uptime_ms: u64,
    snapshot: &SystemSnapshot,
) {
    let y = 90usize;
    let mut x = 24usize;
    x = draw_status_chip(surface, width, x, y, "Net", &snapshot.network);
    let wifi_x = x;
    x = draw_status_chip(surface, width, x, y, "WiFi", &snapshot.wifi);
    if snapshot.wifi.state == RowState::Detected {
        draw_rect_outline_r6(
            surface,
            wifi_x,
            y,
            status_chip_width("WiFi", &snapshot.wifi),
            30,
            APP_BLUE,
        );
    }
    x = draw_status_chip(surface, width, x, y, "Input", &snapshot.input);
    x = draw_status_chip(surface, width, x, y, "USB", &snapshot.usb_xhci);
    let _ = draw_status_chip(surface, width, x, y, "RNG", &snapshot.entropy);

    let uptime = detail64(format_args!("UPTIME {} MS", uptime_ms));
    let uptime_width = text_width(uptime.as_str());
    if uptime_width.saturating_add(48) <= width {
        text::draw_text(
            surface,
            width.saturating_sub(24).saturating_sub(uptime_width),
            y + 11,
            uptime.as_str(),
            TEXT_FAINT,
            None,
        );
    }
    surface.fill_rect(
        24,
        STATUS_HAIRLINE_Y,
        width.saturating_sub(48),
        1,
        HAIRLINE_HI,
    );
}

fn draw_status_detail(
    surface: &mut FramebufferSurface,
    width: usize,
    view: console::UiView,
    snapshot: &SystemSnapshot,
) {
    if view == console::UiView::Settings {
        return;
    }

    let mut y = STATUS_DETAIL_START_Y;
    draw_status_detail_line(surface, width, y, "INPUT", &snapshot.input);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);
    draw_status_detail_line(surface, width, y, "USB", &snapshot.usb_xhci);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);
    draw_status_detail_line(surface, width, y, "HOTPLUG", &snapshot.usb_hotplug);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);
    draw_status_detail_line(surface, width, y, "WIFI", &snapshot.wifi);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);
    draw_status_detail_line(surface, width, y, "NET", &snapshot.network);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);
    draw_status_detail_line(surface, width, y, "IOMMU", &snapshot.iommu);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);
    y = draw_wifi_firmware_detail(surface, width, y);
    draw_wifi_scan_detail(surface, width, y);
}

fn draw_status_detail_line(
    surface: &mut FramebufferSurface,
    width: usize,
    y: usize,
    label: &'static str,
    line: &StatusLine,
) {
    text::draw_text(surface, STATUS_DETAIL_LABEL_X, y, label, TEXT_FAINT, None);
    let max_chars = width.saturating_sub(STATUS_DETAIL_VALUE_X + 24) / FONT_ADVANCE;
    draw_truncated_text(
        surface,
        STATUS_DETAIL_VALUE_X,
        y,
        line.detail.as_str(),
        max_chars,
        row_state_color(line.state),
    );
}

fn draw_wifi_scan_detail(surface: &mut FramebufferSurface, width: usize, mut y: usize) {
    let scan = wifi::scan_results();
    if scan.count == 0 {
        let mut line = TextBuf::<192>::new();
        let _ = write!(
            line,
            "WIFI SCAN: NOT AVAILABLE ({})",
            scan.unavailable_reason
        );
        draw_status_text_line(surface, width, y, "SCAN", line.as_str(), APP_AMBER);
        return;
    }

    let mut header = TextBuf::<192>::new();
    if scan.scan_available {
        let _ = write!(header, "WIFI SCAN: RESULTS");
    } else {
        let _ = write!(
            header,
            "WIFI SCAN: SELF-TEST RESULTS; LIVE NOT AVAILABLE ({})",
            scan.unavailable_reason
        );
    }
    draw_status_text_line(surface, width, y, "SCAN", header.as_str(), APP_AMBER);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);

    let mut idx = 0usize;
    let count = usize::min(scan.count, 3);
    while idx < count {
        let line = wifi_scan_network_line(scan.networks[idx]);
        draw_status_text_line(surface, width, y, "SSID", line.as_str(), TEXT_MUTED);
        y = y.saturating_add(STATUS_DETAIL_LINE_H);
        idx += 1;
    }
}

fn draw_wifi_firmware_detail(
    surface: &mut FramebufferSurface,
    width: usize,
    mut y: usize,
) -> usize {
    let firmware = marvell_wifi_pcie::snapshot();
    let color = firmware_status_color(firmware);
    draw_status_text_line(
        surface,
        width,
        y,
        "FW",
        "WIFI FW: firmware bring-up ATTEMPT (unaudited blob; DMA not IOMMU-confined)",
        color,
    );
    y = y.saturating_add(STATUS_DETAIL_LINE_H);

    let ladder = firmware_ladder_line(firmware);
    draw_status_text_line(surface, width, y, "FW", ladder.as_str(), color);
    y = y.saturating_add(STATUS_DETAIL_LINE_H);

    if firmware.is_failed() || firmware.is_ready() {
        let regs = firmware_register_line(firmware);
        draw_status_text_line(surface, width, y, "FWREG", regs.as_str(), color);
        y = y.saturating_add(STATUS_DETAIL_LINE_H);
    }
    y
}

fn draw_status_text_line(
    surface: &mut FramebufferSurface,
    width: usize,
    y: usize,
    label: &'static str,
    value: &str,
    color: Color,
) {
    text::draw_text(surface, STATUS_DETAIL_LABEL_X, y, label, TEXT_FAINT, None);
    let max_chars = width.saturating_sub(STATUS_DETAIL_VALUE_X + 24) / FONT_ADVANCE;
    draw_truncated_text(surface, STATUS_DETAIL_VALUE_X, y, value, max_chars, color);
}

fn wifi_scan_network_line(network: wifi::ScannedNetwork) -> TextBuf<160> {
    let mut line = TextBuf::new();
    let ssid = if network.hidden_ssid || network.ssid.is_empty() {
        "<hidden>"
    } else {
        network.ssid.as_str()
    };
    if network.channel == 0 {
        let _ = write!(
            line,
            "SSID {}  CH?  {}  {}",
            ssid,
            wifi::scan_security_label(network.security),
            network.source.tag()
        );
    } else {
        let _ = write!(
            line,
            "SSID {}  CH{}  {}  {}",
            ssid,
            network.channel,
            wifi::scan_security_label(network.security),
            network.source.tag()
        );
    }
    line
}

fn draw_status_chip(
    surface: &mut FramebufferSurface,
    screen_width: usize,
    x: usize,
    y: usize,
    label: &'static str,
    line: &StatusLine,
) -> usize {
    let width = status_chip_width(label, line);
    if x.saturating_add(width) > screen_width.saturating_sub(24) {
        return x;
    }

    draw_soft_rect_r6(surface, x, y, width, 30, SURFACE_ALT);
    draw_rect_outline_r6(surface, x, y, width, 30, HAIRLINE);
    draw_status_dot(surface, x + 12, y + 12, row_state_color(line.state));
    text::draw_text(surface, x + 28, y + 11, label, TEXT_MAIN, None);
    text::draw_text(
        surface,
        x + 28 + label.len().saturating_mul(FONT_ADVANCE) + 8,
        y + 11,
        line.state.as_str(),
        row_state_color(line.state),
        None,
    );
    x.saturating_add(width).saturating_add(12)
}

fn draw_status_dot(surface: &mut FramebufferSurface, x: usize, y: usize, color: Color) {
    surface.fill_rect(x + 1, y, 4, 6, color);
    surface.fill_rect(x, y + 1, 6, 4, color);
}

fn text_width(value: &str) -> usize {
    value.chars().count().saturating_mul(FONT_ADVANCE)
}

fn row_state_text_len(state: RowState) -> usize {
    match state {
        RowState::Ready => 5,
        RowState::Waiting => 7,
        RowState::Configured => 10,
        RowState::Detected => 8,
        RowState::Degraded => 8,
        RowState::Missing => 7,
    }
}

fn draw_panel_title(surface: &mut FramebufferSurface, x: usize, y: usize, title: &str) {
    surface.fill_rect(x, y.saturating_sub(3), 3, 14, APP_BLUE);
    text::draw_text(surface, x + 12, y, title, TEXT_MAIN, None);
}

fn draw_panel(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    title: &str,
) {
    draw_soft_rect(surface, x, y, width, height, SURFACE_BG);
    if height > PANEL_TITLE_H.saturating_add(8) {
        draw_soft_rect(surface, x, y, width, PANEL_TITLE_H, SURFACE_ALT);
        surface.fill_rect(
            x,
            y.saturating_add(PANEL_TITLE_H).saturating_sub(8),
            width,
            8,
            SURFACE_ALT,
        );
        surface.fill_rect(
            x + 1,
            y.saturating_add(PANEL_TITLE_H),
            width.saturating_sub(2),
            1,
            HAIRLINE,
        );
    }
    draw_rect_outline(surface, x, y, width, height, HAIRLINE);
    draw_panel_title(surface, x + 20, y + 18, title);
}

fn draw_centered_text(
    surface: &mut FramebufferSurface,
    width: usize,
    y: usize,
    value: &str,
    color: Color,
) {
    let value_width = text_width(value);
    text::draw_text(
        surface,
        width.saturating_sub(value_width) / 2,
        y,
        value,
        color,
        None,
    );
}

fn draw_centered_text_in(
    surface: &mut FramebufferSurface,
    x: usize,
    width: usize,
    y: usize,
    value: &str,
    color: Color,
) {
    let value_width = text_width(value);
    text::draw_text(
        surface,
        x.saturating_add(width.saturating_sub(value_width) / 2),
        y,
        value,
        color,
        None,
    );
}

fn chat_has_messages(snapshot: &console::ConsoleSnapshot) -> bool {
    let mut idx = 0usize;
    while idx < snapshot.chat_lines.len() {
        if !snapshot.chat_lines[idx].text.as_str().is_empty() {
            return true;
        }
        idx += 1;
    }
    false
}

fn input_field_y(height: usize) -> usize {
    height.saturating_sub(74).saturating_add(7)
}

fn input_field_width(width: usize) -> usize {
    width.saturating_sub(INPUT_FIELD_X.saturating_add(INPUT_FIELD_RIGHT))
}

fn draw_input_bar(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    focused: bool,
    value: &str,
    placeholder: &str,
    console_prompt: bool,
) {
    let y = input_field_y(height);
    let field_w = input_field_width(width);
    draw_soft_rect_r6(
        surface,
        INPUT_FIELD_X,
        y,
        field_w,
        INPUT_FIELD_H,
        SURFACE_ALT,
    );
    draw_rect_outline_r6(surface, INPUT_FIELD_X, y, field_w, INPUT_FIELD_H, HAIRLINE);
    if focused {
        draw_focus_outline(surface, INPUT_FIELD_X, y, field_w, INPUT_FIELD_H);
    }

    let mut text_x = INPUT_FIELD_X + 16;
    if console_prompt {
        text::draw_text(surface, text_x, y + 13, ">", APP_BLUE, None);
        text_x += 18;
    }

    let visible = if value.is_empty() { placeholder } else { value };
    if !visible.is_empty() {
        let text_room = field_w.saturating_sub(text_x.saturating_sub(INPUT_FIELD_X) + 14);
        draw_truncated_text(
            surface,
            text_x,
            y + 13,
            visible,
            text_room / FONT_ADVANCE,
            if value.is_empty() {
                TEXT_FAINT
            } else {
                TEXT_MAIN
            },
        );
    }

    let button_x = width.saturating_sub(56);
    let button_y = y + 4;
    draw_soft_rect_r6(surface, button_x, button_y, 28, 28, APP_BLUE);
    text::draw_text(surface, button_x + 9, button_y + 10, ">", TEXT_MAIN, None);
}

fn draw_settings_row(
    surface: &mut FramebufferSurface,
    width: usize,
    y: usize,
    key: &str,
    value: &str,
    is_set: bool,
) {
    text::draw_text(surface, 56, y, key, TEXT_MUTED, None);
    text::draw_text(
        surface,
        220,
        y,
        value,
        if is_set { APP_GREEN } else { APP_AMBER },
        None,
    );
    surface.fill_rect(48, y + 25, width.saturating_sub(120), 1, HAIRLINE);
}

fn row_state_color(state: RowState) -> Color {
    match state {
        RowState::Ready => APP_GREEN,
        RowState::Waiting => APP_AMBER,
        RowState::Configured => APP_BLUE,
        RowState::Detected => APP_BLUE,
        RowState::Degraded => APP_AMBER,
        RowState::Missing => APP_RED,
    }
}

fn draw_chat(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    snapshot: &console::ConsoleSnapshot,
) {
    let top = CONTENT_TOP;
    let bottom = height.saturating_sub(88);
    draw_panel(
        surface,
        24,
        top,
        width.saturating_sub(48),
        bottom.saturating_sub(top),
        "Chat",
    );
    text::draw_text(
        surface,
        116,
        top + 18,
        snapshot.provider_phase,
        TEXT_MUTED,
        None,
    );
    let key_state = if snapshot.api_key_set {
        "KEY SET"
    } else {
        "KEY MISSING"
    };
    let provider = detail64(format_args!(
        "{} {} {}",
        snapshot.provider_name, snapshot.provider_model, key_state
    ));
    let provider_max_chars = width.saturating_sub(364) / FONT_ADVANCE;
    let provider_chars = usize::min(provider.as_str().chars().count(), provider_max_chars);
    draw_truncated_text(
        surface,
        width
            .saturating_sub(44)
            .saturating_sub(provider_chars.saturating_mul(FONT_ADVANCE)),
        top + 18,
        provider.as_str(),
        provider_max_chars,
        TEXT_MUTED,
    );

    let has_messages = chat_has_messages(snapshot);
    let content_top = top.saturating_add(PANEL_TITLE_H).saturating_add(1);
    if !has_messages {
        draw_centered_text(
            surface,
            width,
            content_top.saturating_add(bottom.saturating_sub(content_top) / 2),
            "No messages yet - type below to talk to the AI",
            TEXT_FAINT,
        );
    }

    let mut cursor_y = bottom.saturating_sub(18);
    let min_y = top + 52;
    let mut idx = snapshot.chat_lines.len();
    while idx > 0 {
        idx -= 1;
        let line = snapshot.chat_lines[idx];
        let value = line.text.as_str();
        if value.is_empty() {
            continue;
        }

        let max_width = chat_bubble_max_width(width, line.speaker);
        let layout = chat_bubble_layout(value, max_width);
        let Some(y) = cursor_y.checked_sub(layout.height) else {
            break;
        };
        if y < min_y {
            break;
        }

        draw_chat_bubble(surface, width, y, line.speaker, value, layout);
        cursor_y = y.saturating_sub(CHAT_BUBBLE_GAP);
    }

    draw_input_bar(
        surface,
        width,
        height,
        snapshot.focus == console::UiFocus::ChatInput,
        snapshot.chat_input.as_str(),
        "Type a message and press Enter",
        false,
    );
}

fn draw_chat_bubble(
    surface: &mut FramebufferSurface,
    width: usize,
    y: usize,
    speaker: console::ChatSpeaker,
    value: &str,
    layout: BubbleLayout,
) {
    let (x, color, text_color, label) = match speaker {
        console::ChatSpeaker::User => (
            width.saturating_sub(layout.width + 48),
            USER_BUBBLE,
            TEXT_MAIN,
            "You",
        ),
        console::ChatSpeaker::Assistant => (48, SURFACE_BG, TEXT_MAIN, "AI"),
        console::ChatSpeaker::System => (
            width.saturating_sub(layout.width) / 2,
            SURFACE_ALT,
            TEXT_MUTED,
            "Sys",
        ),
    };

    draw_bubble_rect(surface, x, y, layout.width, layout.height, color, speaker);
    text::draw_text(
        surface,
        x + CHAT_BUBBLE_PADDING_X,
        y + CHAT_BUBBLE_PADDING_TOP,
        label,
        match speaker {
            console::ChatSpeaker::User => Color::new(200, 224, 255),
            console::ChatSpeaker::Assistant => TEXT_FAINT,
            console::ChatSpeaker::System => TEXT_MUTED,
        },
        None,
    );
    draw_wrapped_text(
        surface,
        x + CHAT_BUBBLE_PADDING_X,
        y + CHAT_BUBBLE_PADDING_TOP + CHAT_BUBBLE_LABEL_HEIGHT + CHAT_BUBBLE_LABEL_GAP,
        value,
        layout.max_chars,
        text_color,
        layout.line_count,
    );
}

#[derive(Clone, Copy)]
struct BubbleLayout {
    width: usize,
    height: usize,
    max_chars: usize,
    line_count: usize,
}

fn chat_bubble_max_width(screen_width: usize, speaker: console::ChatSpeaker) -> usize {
    match speaker {
        console::ChatSpeaker::User => usize::max(220, screen_width.saturating_mul(44) / 100),
        console::ChatSpeaker::Assistant => usize::max(360, screen_width.saturating_mul(64) / 100),
        console::ChatSpeaker::System => usize::max(260, screen_width.saturating_mul(52) / 100),
    }
}

fn chat_bubble_layout(value: &str, max_width: usize) -> BubbleLayout {
    let inner_max = max_width.saturating_sub(CHAT_BUBBLE_PADDING_X * 2);
    let max_chars = usize::max(8, inner_max / FONT_ADVANCE);
    let (line_count, longest_line) = wrap_metrics(value, max_chars);
    let text_width = longest_line.saturating_mul(FONT_ADVANCE);
    let label_width = 3usize.saturating_mul(FONT_ADVANCE);
    let min_width = 104usize;
    let width = usize::min(
        max_width,
        usize::max(
            min_width,
            usize::max(text_width, label_width).saturating_add(CHAT_BUBBLE_PADDING_X * 2),
        ),
    );
    let height = CHAT_BUBBLE_PADDING_TOP
        + CHAT_BUBBLE_LABEL_HEIGHT
        + CHAT_BUBBLE_LABEL_GAP
        + line_count.saturating_mul(CHAT_LINE_HEIGHT)
        + CHAT_BUBBLE_PADDING_BOTTOM;

    BubbleLayout {
        width,
        height,
        max_chars: usize::max(
            8,
            width.saturating_sub(CHAT_BUBBLE_PADDING_X * 2) / FONT_ADVANCE,
        ),
        line_count,
    }
}

fn wrap_metrics(value: &str, max_chars: usize) -> (usize, usize) {
    let mut offset = 0usize;
    let mut lines = 0usize;
    let mut longest = 0usize;

    while offset < value.len() {
        offset = skip_whitespace(value, offset);
        if offset >= value.len() {
            break;
        }

        let end = wrapped_line_end(value, offset, max_chars);
        let len = value[offset..end].chars().count();
        longest = usize::max(longest, len);
        lines += 1;
        offset = end;
    }

    if lines == 0 {
        (1, 0)
    } else {
        (lines, longest)
    }
}

fn draw_wrapped_text(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    value: &str,
    max_chars: usize,
    color: Color,
    max_lines: usize,
) {
    let mut offset = 0usize;
    let mut line = 0usize;

    while offset < value.len() && line < max_lines {
        offset = skip_whitespace(value, offset);
        if offset >= value.len() {
            break;
        }

        let end = wrapped_line_end(value, offset, max_chars);
        text::draw_text(
            surface,
            x,
            y + line * CHAT_LINE_HEIGHT,
            &value[offset..end],
            color,
            None,
        );
        offset = end;
        line += 1;
    }
}

fn draw_truncated_text(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    value: &str,
    max_chars: usize,
    color: Color,
) {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        text::draw_text(surface, x, y, value, color, None);
        return;
    }
    if max_chars <= 3 {
        return;
    }

    let prefix_len = max_chars - 3;
    let prefix_end = nth_char_boundary(value, prefix_len);
    let prefix = &value[..prefix_end];
    text::draw_text(surface, x, y, prefix, color, None);
    text::draw_text(
        surface,
        x + prefix_len.saturating_mul(FONT_ADVANCE),
        y,
        "...",
        color,
        None,
    );
}

fn skip_whitespace(value: &str, mut offset: usize) -> usize {
    while offset < value.len() {
        let Some(ch) = value[offset..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        offset += ch.len_utf8();
    }
    offset
}

fn wrapped_line_end(value: &str, start: usize, max_chars: usize) -> usize {
    let mut offset = start;
    let mut count = 0usize;
    let mut last_space = None;

    while offset < value.len() && count < max_chars {
        let Some(ch) = value[offset..].chars().next() else {
            break;
        };
        if ch.is_whitespace() && offset > start {
            last_space = Some(offset);
        }
        offset += ch.len_utf8();
        count += 1;
    }

    if offset >= value.len() {
        return offset;
    }

    last_space.unwrap_or(offset)
}

fn nth_char_boundary(value: &str, count: usize) -> usize {
    match value.char_indices().nth(count) {
        Some((idx, _)) => idx,
        None => value.len(),
    }
}

fn draw_bubble_rect(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
    speaker: console::ChatSpeaker,
) {
    draw_soft_rect(surface, x, y, width, height, color);
    if speaker == console::ChatSpeaker::Assistant {
        draw_rect_outline(surface, x, y, width, height, HAIRLINE);
    }
}

fn draw_soft_rect(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
) {
    draw_soft_rect_with_insets(surface, x, y, width, height, color, &R8_INSETS);
}

fn draw_soft_rect_r6(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
) {
    draw_soft_rect_with_insets(surface, x, y, width, height, color, &R6_INSETS);
}

fn draw_soft_rect_with_insets(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
    insets: &[usize],
) {
    if width == 0 || height == 0 {
        return;
    }

    let rows = usize::min(insets.len(), height / 2);
    let middle_h = height.saturating_sub(rows.saturating_mul(2));
    if middle_h > 0 {
        surface.fill_rect(x, y + rows, width, middle_h, color);
    }

    let mut dy = 0usize;
    while dy < rows {
        let inset = usize::min(insets[dy], width / 2);
        let row_w = width.saturating_sub(inset.saturating_mul(2));
        if row_w > 0 {
            surface.fill_rect(x + inset, y + dy, row_w, 1, color);
            surface.fill_rect(
                x + inset,
                y + height.saturating_sub(1).saturating_sub(dy),
                row_w,
                1,
                color,
            );
        }
        dy += 1;
    }
}

fn draw_rect_outline(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
) {
    draw_rect_outline_with_insets(surface, x, y, width, height, color, &R8_INSETS);
}

fn draw_rect_outline_r6(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
) {
    draw_rect_outline_with_insets(surface, x, y, width, height, color, &R6_INSETS);
}

fn draw_rect_outline_with_insets(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
    insets: &[usize],
) {
    if width == 0 || height == 0 {
        return;
    }

    let rows = usize::min(insets.len(), height / 2);
    let top_inset = if rows > 0 {
        usize::min(insets[0], width / 2)
    } else {
        0
    };
    let row_w = width.saturating_sub(top_inset.saturating_mul(2));
    if row_w > 0 {
        surface.fill_rect(x + top_inset, y, row_w, 1, color);
        surface.fill_rect(x + top_inset, y + height.saturating_sub(1), row_w, 1, color);
    }

    let side_h = height.saturating_sub(rows.saturating_mul(2));
    if side_h > 0 {
        surface.fill_rect(x, y + rows, 1, side_h, color);
        surface.fill_rect(x + width.saturating_sub(1), y + rows, 1, side_h, color);
    }

    let mut dy = 0usize;
    while dy < rows {
        let inset = usize::min(insets[dy], width / 2);
        draw_corner_row(surface, x, y + dy, width, inset, color);
        draw_corner_row(
            surface,
            x,
            y + height.saturating_sub(1).saturating_sub(dy),
            width,
            inset,
            color,
        );
        dy += 1;
    }
}

fn draw_corner_row(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    inset: usize,
    color: Color,
) {
    if width == 0 {
        return;
    }

    let left = usize::min(inset, width.saturating_sub(1));
    surface.set_pixel(x + left, y, color);
    if left.saturating_add(1) < width {
        surface.set_pixel(x + left + 1, y, color);
    }

    let right = width.saturating_sub(1).saturating_sub(left);
    surface.set_pixel(x + right, y, color);
    if right > 0 {
        surface.set_pixel(x + right - 1, y, color);
    }
}

fn draw_focus_outline(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    draw_rect_outline_r6(surface, x, y, width, height, APP_BLUE);
}

fn draw_settings_action(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    label: &str,
    focused: bool,
) {
    draw_soft_rect_r6(surface, x, y, width, SETTINGS_ACTION_H, SURFACE_ALT);
    draw_rect_outline_r6(surface, x, y, width, SETTINGS_ACTION_H, HAIRLINE);
    if focused {
        draw_focus_outline(surface, x, y, width, SETTINGS_ACTION_H);
    }
    draw_truncated_text(
        surface,
        x + 18,
        y + 15,
        label,
        width.saturating_sub(36) / FONT_ADVANCE,
        TEXT_MAIN,
    );
}

fn draw_console(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    snapshot: &console::ConsoleSnapshot,
) {
    let top = CONTENT_TOP;
    draw_panel(
        surface,
        24,
        top,
        width.saturating_sub(48),
        height.saturating_sub(top + 42),
        "Console",
    );

    let input_y = input_field_y(height);
    let max_chars = usize::max(8, width.saturating_sub(88) / FONT_ADVANCE);
    let min_y = top + 56;
    let mut cursor_y = input_y.saturating_sub(22);
    let mut idx = snapshot.lines.len();
    while idx > 0 {
        idx -= 1;
        let line = snapshot.lines[idx].as_str();
        if line.is_empty() {
            continue;
        }

        let is_user_command = line.starts_with("> ");
        let (line_count, _) = if is_user_command {
            wrap_metrics(&line[2..], max_chars.saturating_sub(2))
        } else {
            wrap_metrics(line, max_chars)
        };
        let y = cursor_y.saturating_sub(line_count.saturating_sub(1) * CHAT_LINE_HEIGHT);
        if y < min_y {
            break;
        }
        if is_user_command {
            text::draw_text(surface, 44, y, ">", APP_BLUE, None);
            draw_wrapped_text(
                surface,
                62,
                y,
                &line[2..],
                max_chars.saturating_sub(2),
                TEXT_MAIN,
                line_count,
            );
        } else {
            draw_wrapped_text(surface, 44, y, line, max_chars, TEXT_MUTED, line_count);
        }
        cursor_y = y.saturating_sub(CHAT_LINE_HEIGHT);
    }

    draw_input_bar(
        surface,
        width,
        height,
        snapshot.focus == console::UiFocus::ConsoleInput,
        snapshot.input.as_str(),
        "",
        true,
    );
}

fn draw_settings(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    snapshot: &console::ConsoleSnapshot,
) {
    let top = CONTENT_TOP;
    draw_panel(
        surface,
        24,
        top,
        width.saturating_sub(48),
        height.saturating_sub(top + 42),
        "Settings",
    );

    let key_state = if snapshot.api_key_set {
        "SET"
    } else {
        "MISSING"
    };
    let wifi_ssid = if snapshot.wifi_ssid.is_empty() {
        "NONE"
    } else {
        snapshot.wifi_ssid.as_str()
    };
    let wifi_key_state = if snapshot.wifi_passphrase_set {
        "SET"
    } else {
        "MISSING"
    };
    let (action_x, action_x2, action_w) = settings_action_columns(width);
    let action_y = top + 62;
    draw_settings_row(
        surface,
        action_x,
        top + 62,
        "Provider",
        snapshot.provider_name,
        true,
    );
    draw_settings_row(
        surface,
        action_x,
        top + 102,
        "Model",
        snapshot.provider_model,
        true,
    );
    draw_settings_row(
        surface,
        action_x,
        top + 142,
        "API Key",
        key_state,
        snapshot.api_key_set,
    );
    draw_settings_row(
        surface,
        action_x,
        top + 182,
        "WiFi SSID",
        wifi_ssid,
        !snapshot.wifi_ssid.is_empty(),
    );
    draw_settings_row(
        surface,
        action_x,
        top + 222,
        "WiFi Key",
        wifi_key_state,
        snapshot.wifi_passphrase_set,
    );

    draw_settings_action(
        surface,
        action_x,
        action_y,
        action_w,
        "Provider Status",
        snapshot.focus == console::UiFocus::SettingsProvider,
    );
    draw_settings_action(
        surface,
        action_x2,
        action_y,
        action_w,
        "Enter API Key",
        snapshot.focus == console::UiFocus::SettingsApiKey,
    );
    draw_settings_action(
        surface,
        action_x,
        action_y + SETTINGS_ACTION_ROW_STEP,
        action_w,
        "Clear API Key",
        snapshot.focus == console::UiFocus::SettingsClear,
    );
    draw_settings_action(
        surface,
        action_x2,
        action_y + SETTINGS_ACTION_ROW_STEP,
        action_w,
        "WiFi SSID",
        snapshot.focus == console::UiFocus::SettingsWifiSsid,
    );
    draw_settings_action(
        surface,
        action_x,
        action_y + SETTINGS_ACTION_ROW_STEP * 2,
        action_w,
        "WiFi Key",
        snapshot.focus == console::UiFocus::SettingsWifiPassphrase,
    );
    draw_settings_action(
        surface,
        action_x2,
        action_y + SETTINGS_ACTION_ROW_STEP * 2,
        action_w,
        "Clear WiFi",
        snapshot.focus == console::UiFocus::SettingsWifiClear,
    );
    draw_settings_action(
        surface,
        action_x,
        action_y + SETTINGS_ACTION_ROW_STEP * 3,
        action_w,
        "Start WiFi FW",
        snapshot.focus == console::UiFocus::SettingsWifiFirmware,
    );
    draw_settings_action(
        surface,
        action_x2,
        action_y + SETTINGS_ACTION_ROW_STEP * 3,
        action_w,
        "Scan networks",
        snapshot.focus == console::UiFocus::SettingsWifiScan,
    );
    draw_settings_action(
        surface,
        action_x,
        action_y + SETTINGS_ACTION_ROW_STEP * 4,
        action_w,
        "Close Settings",
        snapshot.focus == console::UiFocus::SettingsClose,
    );

    let networks_y = draw_settings_wifi_firmware(surface, width, top + 340);
    draw_settings_wifi_networks(surface, width, height, networks_y.saturating_add(14));

    draw_input_bar(
        surface,
        width,
        height,
        snapshot.settings_entry_active || snapshot.focus == console::UiFocus::ConsoleInput,
        snapshot.input.as_str(),
        "",
        false,
    );
}

fn draw_wifi_flow(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    flow: WifiFlow,
    wifi_chip_rect: Option<LogicalRect>,
    console_snapshot: &console::ConsoleSnapshot,
) {
    match flow {
        WifiFlow::Idle => {}
        WifiFlow::Starting { .. } => draw_wifi_progress_dialog(surface, width, height),
        WifiFlow::Selecting => draw_wifi_selection_dialog(
            surface,
            width,
            height,
            wifi_chip_rect.map(|rect| rect.x).unwrap_or(24),
        ),
        WifiFlow::Password {
            network_index,
            remember_for_boot,
            rejected,
        } => draw_wifi_password_dialog(
            surface,
            width,
            height,
            network_index,
            remember_for_boot,
            rejected,
            console_snapshot,
        ),
        WifiFlow::Configured {
            network_index,
            remember_for_boot,
        } => draw_wifi_configured_dialog(surface, width, height, network_index, remember_for_boot),
        WifiFlow::Failed(reason) => draw_wifi_failed_dialog(surface, width, height, reason),
    }
}

fn draw_wifi_progress_dialog(surface: &mut FramebufferSurface, width: usize, height: usize) {
    let rect = wifi_progress_dialog_rect(width, height);
    draw_panel(surface, rect.x, rect.y, rect.w, rect.h, "Starting WiFi");

    let (percent, label) = wifi_flow_progress();
    draw_truncated_text(
        surface,
        rect.x + 28,
        rect.y + 67,
        label,
        rect.w.saturating_sub(56) / FONT_ADVANCE,
        TEXT_MAIN,
    );
    let bar_x = rect.x + 28;
    let bar_y = rect.y + 98;
    let bar_w = rect.w.saturating_sub(56);
    draw_soft_rect_r6(surface, bar_x, bar_y, bar_w, 18, SURFACE_ALT);
    let fill_w = bar_w.saturating_mul(percent) / 100;
    if fill_w > 0 {
        draw_soft_rect_r6(surface, bar_x, bar_y, fill_w, 18, APP_BLUE);
    }
    draw_rect_outline_r6(surface, bar_x, bar_y, bar_w, 18, HAIRLINE_HI);

    let mut progress = TextBuf::<32>::new();
    let _ = write!(progress, "{}%", percent);
    draw_centered_text_in(
        surface,
        rect.x,
        rect.w,
        rect.y + 132,
        progress.as_str(),
        TEXT_MUTED,
    );
}

fn wifi_flow_progress() -> (usize, &'static str) {
    let firmware = marvell_wifi_pcie::snapshot();
    if !firmware.is_ready() {
        let percent = if firmware.total == 0 {
            4
        } else {
            4usize.saturating_add(
                firmware
                    .downloaded
                    .saturating_mul(76)
                    .checked_div(firmware.total)
                    .unwrap_or(0),
            )
        };
        return (usize::min(percent, 80), "Loading radio firmware");
    }

    let hw_spec = marvell_wifi_pcie::hw_spec_snapshot();
    if !hw_spec.is_ready() {
        return (88, "Reading radio identity");
    }

    let scan = marvell_wifi_pcie::scan_cmd_snapshot();
    if scan.stage == marvell_wifi_pcie::ScanCmdStage::Done {
        (100, "Networks ready")
    } else {
        (94, "Scanning networks")
    }
}

fn draw_wifi_selection_dialog(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    anchor_x: usize,
) {
    let scan = wifi::scan_results();
    let rect = wifi_selection_dialog_rect(width, height, scan.count, anchor_x);
    draw_panel(surface, rect.x, rect.y, rect.w, rect.h, "WiFi networks");

    let visible = usize::min(scan.count, WIFI_FLOW_LIST_LIMIT);
    if visible == 0 {
        draw_centered_text_in(
            surface,
            rect.x,
            rect.w,
            rect.y + 76,
            "No networks found",
            TEXT_MUTED,
        );
    } else {
        let mut index = 0usize;
        while index < visible {
            let network = scan.networks[index];
            let row_y = rect.y + 66 + index * WIFI_FLOW_LIST_ROW_H;
            draw_soft_rect_r6(surface, rect.x + 24, row_y, rect.w - 48, 34, SURFACE_ALT);
            draw_rect_outline_r6(surface, rect.x + 24, row_y, rect.w - 48, 34, HAIRLINE);
            let line = wifi_scan_settings_network_line(network);
            draw_truncated_text(
                surface,
                rect.x + 40,
                row_y + 13,
                line.as_str(),
                rect.w.saturating_sub(80) / FONT_ADVANCE,
                if network.hidden_ssid {
                    TEXT_FAINT
                } else {
                    TEXT_MAIN
                },
            );
            index += 1;
        }
        if scan.count > visible {
            let mut more = TextBuf::<48>::new();
            let _ = write!(more, "+{} more", scan.count - visible);
            text::draw_text(
                surface,
                rect.x + rect.w - 24 - text_width(more.as_str()),
                rect.y + 51,
                more.as_str(),
                TEXT_FAINT,
                None,
            );
        }
    }

    let (left, right) = wifi_dialog_action_rects(rect);
    draw_wifi_dialog_button(surface, left, "Scan again", false);
    draw_wifi_dialog_button(surface, right, "Close", false);
}

fn draw_wifi_password_dialog(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    network_index: usize,
    remember_for_boot: bool,
    rejected: bool,
    console_snapshot: &console::ConsoleSnapshot,
) {
    let rect = wifi_password_dialog_rect(width, height);
    draw_panel(surface, rect.x, rect.y, rect.w, rect.h, "WiFi password");
    let scan = wifi::scan_results();
    let ssid = scan
        .networks
        .get(network_index)
        .map(|network| network.ssid.as_str())
        .unwrap_or("Unknown network");
    draw_truncated_text(
        surface,
        rect.x + 28,
        rect.y + 64,
        ssid,
        rect.w.saturating_sub(56) / FONT_ADVANCE,
        TEXT_MAIN,
    );

    draw_soft_rect_r6(
        surface,
        rect.x + 28,
        rect.y + 88,
        rect.w - 56,
        36,
        SURFACE_ALT,
    );
    draw_rect_outline_r6(surface, rect.x + 28, rect.y + 88, rect.w - 56, 36, APP_BLUE);
    draw_truncated_text(
        surface,
        rect.x + 42,
        rect.y + 102,
        console_snapshot.input.as_str(),
        rect.w.saturating_sub(84) / FONT_ADVANCE,
        TEXT_MAIN,
    );
    text::draw_text(
        surface,
        rect.x + 28,
        rect.y + 132,
        if rejected {
            "Password must contain 8-63 printable characters"
        } else {
            "8-63 printable characters"
        },
        if rejected { APP_RED } else { TEXT_FAINT },
        None,
    );

    draw_wifi_checkbox(
        surface,
        rect.x + 28,
        rect.y + 157,
        remember_for_boot,
        "Remember for this boot (RAM only)",
    );

    let (left, right) = wifi_dialog_action_rects(rect);
    draw_wifi_dialog_button(surface, left, "Back", false);
    draw_wifi_dialog_button(surface, right, "Set credentials", true);
}

fn draw_wifi_configured_dialog(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    network_index: usize,
    remember_for_boot: bool,
) {
    let rect = wifi_result_dialog_rect(width, height);
    draw_panel(surface, rect.x, rect.y, rect.w, rect.h, "WiFi setup");
    let scan = wifi::scan_results();
    let ssid = scan
        .networks
        .get(network_index)
        .map(|network| network.ssid.as_str())
        .unwrap_or("Selected network");
    draw_truncated_text(
        surface,
        rect.x + 28,
        rect.y + 66,
        ssid,
        rect.w.saturating_sub(56) / FONT_ADVANCE,
        APP_GREEN,
    );
    text::draw_text(
        surface,
        rect.x + 28,
        rect.y + 92,
        if remember_for_boot {
            "Credentials ready in RAM for this boot"
        } else {
            "Network selected for the pending connection"
        },
        TEXT_MAIN,
        None,
    );
    text::draw_text(
        surface,
        rect.x + 28,
        rect.y + 114,
        "Connection not established",
        APP_AMBER,
        None,
    );
    let (_, right) = wifi_dialog_action_rects(rect);
    draw_wifi_dialog_button(surface, right, "Done", true);
}

fn draw_wifi_failed_dialog(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    reason: &'static str,
) {
    let rect = wifi_result_dialog_rect(width, height);
    draw_panel(surface, rect.x, rect.y, rect.w, rect.h, "WiFi unavailable");
    draw_truncated_text(
        surface,
        rect.x + 28,
        rect.y + 72,
        reason,
        rect.w.saturating_sub(56) / FONT_ADVANCE,
        APP_RED,
    );
    text::draw_text(
        surface,
        rect.x + 28,
        rect.y + 102,
        "No network state was granted",
        TEXT_MUTED,
        None,
    );
    let (left, right) = wifi_dialog_action_rects(rect);
    draw_wifi_dialog_button(surface, left, "Retry", false);
    draw_wifi_dialog_button(surface, right, "Close", false);
}

fn draw_wifi_dialog_button(
    surface: &mut FramebufferSurface,
    rect: LogicalRect,
    label: &str,
    primary: bool,
) {
    draw_soft_rect_r6(
        surface,
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if primary { APP_BLUE } else { SURFACE_ALT },
    );
    draw_rect_outline_r6(
        surface,
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if primary { APP_BLUE } else { HAIRLINE_HI },
    );
    draw_centered_text_in(surface, rect.x, rect.w, rect.y + 13, label, TEXT_MAIN);
}

fn draw_wifi_checkbox(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    checked: bool,
    label: &str,
) {
    draw_soft_rect_r6(
        surface,
        x,
        y,
        18,
        18,
        if checked { APP_BLUE } else { SURFACE_ALT },
    );
    draw_rect_outline_r6(
        surface,
        x,
        y,
        18,
        18,
        if checked { APP_BLUE } else { HAIRLINE_HI },
    );
    if checked {
        text::draw_text(surface, x + 5, y + 5, "x", TEXT_MAIN, None);
    }
    text::draw_text(surface, x + 30, y + 5, label, TEXT_MUTED, None);
}

fn draw_settings_wifi_firmware(
    surface: &mut FramebufferSurface,
    width: usize,
    mut y: usize,
) -> usize {
    let firmware = marvell_wifi_pcie::snapshot();
    let max_chars = usize::max(8, width.saturating_sub(112) / FONT_ADVANCE);
    let color = firmware_status_color(firmware);
    draw_truncated_text(
        surface,
        56,
        y,
        "WIFI FW - ATTEMPT ONLY (unaudited blob; DMA not IOMMU-confined)",
        max_chars,
        color,
    );
    surface.fill_rect(48, y + 21, width.saturating_sub(120), 1, HAIRLINE);
    y = y.saturating_add(38);

    let ladder = firmware_ladder_line(firmware);
    draw_truncated_text(surface, 72, y, ladder.as_str(), max_chars, color);
    y = y.saturating_add(18);

    if firmware.is_failed() || firmware.is_ready() {
        let regs = firmware_register_line(firmware);
        draw_truncated_text(surface, 72, y, regs.as_str(), max_chars, color);
        y = y.saturating_add(18);
    }
    if firmware.is_ready() {
        let hw_spec = marvell_wifi_pcie::hw_spec_snapshot();
        if hw_spec.stage != marvell_wifi_pcie::HwSpecStage::Idle {
            let line = hw_spec_line(hw_spec);
            draw_truncated_text(
                surface,
                72,
                y,
                line.as_str(),
                max_chars,
                hw_spec_status_color(hw_spec),
            );
            y = y.saturating_add(18);
        }
        let scan_cmd = marvell_wifi_pcie::scan_cmd_snapshot();
        if scan_cmd.attempted {
            let line = scan_cmd_line(scan_cmd);
            draw_truncated_text(
                surface,
                72,
                y,
                line.as_str(),
                max_chars,
                scan_cmd_status_color(scan_cmd),
            );
            y = y.saturating_add(18);
        }
        let event_ring = marvell_wifi_pcie::event_ring_snapshot();
        if event_ring.attempted {
            let line = event_ring_line(event_ring);
            draw_truncated_text(
                surface,
                72,
                y,
                line.as_str(),
                max_chars,
                event_ring_status_color(event_ring),
            );
            y = y.saturating_add(18);
        }
        let rx_ring = marvell_wifi_pcie::rx_ring_snapshot();
        if rx_ring.attempted {
            let line = rx_ring_line(rx_ring);
            draw_truncated_text(
                surface,
                72,
                y,
                line.as_str(),
                max_chars,
                rx_ring_status_color(rx_ring),
            );
            y = y.saturating_add(18);
        }
    }
    y
}

fn draw_settings_wifi_networks(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    y: usize,
) {
    let scan = wifi::scan_results();
    let max_chars = usize::max(8, width.saturating_sub(112) / FONT_ADVANCE);
    let header = wifi_networks_header(scan);
    draw_truncated_text(
        surface,
        56,
        y,
        header.as_str(),
        max_chars,
        if scan.scan_available {
            APP_GREEN
        } else {
            APP_AMBER
        },
    );
    surface.fill_rect(48, y + 21, width.saturating_sub(120), 1, HAIRLINE);

    let mut row_y = y.saturating_add(38);
    let bottom = input_field_y(height).saturating_sub(20);
    if scan.count == 0 {
        draw_truncated_text(
            surface,
            72,
            row_y,
            "No live networks listed; Scan networks starts live command or fallback",
            max_chars.saturating_sub(2),
            TEXT_FAINT,
        );
        return;
    }

    let mut idx = 0usize;
    while idx < scan.count && idx < scan.networks.len() && row_y <= bottom {
        let line = wifi_scan_settings_network_line(scan.networks[idx]);
        draw_truncated_text(
            surface,
            72,
            row_y,
            line.as_str(),
            max_chars.saturating_sub(2),
            TEXT_MUTED,
        );
        row_y = row_y.saturating_add(18);
        idx += 1;
    }

    if idx < scan.count && row_y <= bottom {
        let mut line = TextBuf::<64>::new();
        let _ = write!(line, "+{} more networks", scan.count.saturating_sub(idx));
        draw_truncated_text(
            surface,
            72,
            row_y,
            line.as_str(),
            max_chars.saturating_sub(2),
            TEXT_FAINT,
        );
    }
}

fn wifi_networks_header(scan: wifi::ScanResultsSnapshot) -> TextBuf<192> {
    let mut header = TextBuf::new();
    if scan.scan_available {
        let _ = write!(header, "WIFI NETWORKS - LIVE SCAN AVAILABLE");
    } else if scan.count == 0 {
        let _ = write!(
            header,
            "WIFI NETWORKS - LIVE RESULTS NOT AVAILABLE ({}); Scan networks starts live command or fallback",
            scan.unavailable_reason
        );
    } else {
        let _ = write!(
            header,
            "WIFI NETWORKS - LIVE SCAN NOT AVAILABLE ({}); showing SELF-TEST",
            scan.unavailable_reason
        );
    }
    header
}

fn wifi_scan_settings_network_line(network: wifi::ScannedNetwork) -> TextBuf<160> {
    let mut line = TextBuf::new();
    let ssid = if network.hidden_ssid || network.ssid.is_empty() {
        "(hidden)"
    } else {
        network.ssid.as_str()
    };
    if network.channel == 0 {
        let _ = write!(
            line,
            "SSID {}  CH?  {}  {}",
            ssid,
            wifi::scan_security_label(network.security),
            network.source.tag()
        );
    } else {
        let _ = write!(
            line,
            "SSID {}  CH{}  {}  {}",
            ssid,
            network.channel,
            wifi::scan_security_label(network.security),
            network.source.tag()
        );
    }
    line
}

fn firmware_status_color(firmware: marvell_wifi_pcie::FirmwareBringupSnapshot) -> Color {
    if firmware.is_ready() {
        APP_GREEN
    } else if firmware.is_failed() {
        APP_RED
    } else if firmware.running || firmware.attempted {
        APP_AMBER
    } else {
        TEXT_FAINT
    }
}

fn firmware_ladder_line(firmware: marvell_wifi_pcie::FirmwareBringupSnapshot) -> TextBuf<192> {
    let mut line = TextBuf::new();
    let status = match firmware.result {
        Some(result) => result.label(),
        None if firmware.running => "running",
        None if firmware.attempted => "pending",
        None => "not_started",
    };
    let failed_at = match firmware.failed_stage {
        Some(stage) => stage.label(),
        None => firmware.stage.label(),
    };
    let _ = write!(
        line,
        "DETECT {} -> BAR2 {} -> DOWNLOAD {}/{} -> FW_STATUS=0x{:08X} -> {}@{}",
        detect_ladder_label(firmware),
        bar2_ladder_label(firmware),
        firmware.downloaded,
        firmware.total,
        firmware.registers.fw_status,
        status,
        failed_at
    );
    line
}

fn firmware_register_line(firmware: marvell_wifi_pcie::FirmwareBringupSnapshot) -> TextBuf<192> {
    let mut line = TextBuf::new();
    if firmware.registers.valid {
        let _ = write!(
            line,
            "REG C40=0x{:08X} C44=0x{:08X} CF0=0x{:08X} C30=0x{:08X}",
            firmware.registers.cmd_size,
            firmware.registers.fw_status,
            firmware.registers.drv_ready,
            firmware.registers.host_int_status
        );
    } else {
        let _ = write!(line, "REG unavailable before MMIO");
    }
    line
}

fn hw_spec_line(hw_spec: marvell_wifi_pcie::HwSpecSnapshot) -> TextBuf<192> {
    let mut line = TextBuf::new();
    if hw_spec.is_ready() {
        if let (Some(mac), Some(fw_release)) = (hw_spec.mac, hw_spec.fw_release) {
            let _ = write!(
                line,
                "HW_SPEC: MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}  FW=0x{:08x}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5], fw_release
            );
            return line;
        }
    }
    if hw_spec.running {
        let _ = write!(line, "HW_SPEC: querying chip (GET_HW_SPEC)...");
        return line;
    }
    match hw_spec.result {
        Some(marvell_wifi_pcie::HwSpecResult::Response(
            raios_core::marvell_wifi_cmd::HwSpecCmdError::BadCommand { got },
        ))
        | Some(marvell_wifi_pcie::HwSpecResult::CommandBuild(
            raios_core::marvell_wifi_cmd::HwSpecCmdError::BadCommand { got },
        )) => {
            let _ = write!(
                line,
                "HW_SPEC: bad_command 0x{:04x} (HOST_INT=0x{:08x})",
                got, hw_spec.host_int_status
            );
        }
        Some(marvell_wifi_pcie::HwSpecResult::Response(
            raios_core::marvell_wifi_cmd::HwSpecCmdError::FwResult { code },
        ))
        | Some(marvell_wifi_pcie::HwSpecResult::CommandBuild(
            raios_core::marvell_wifi_cmd::HwSpecCmdError::FwResult { code },
        )) => {
            let _ = write!(
                line,
                "HW_SPEC: fw_result 0x{:04x} (HOST_INT=0x{:08x})",
                code, hw_spec.host_int_status
            );
        }
        Some(result) => {
            let _ = write!(
                line,
                "HW_SPEC: {} (HOST_INT=0x{:08x})",
                result.label(),
                hw_spec.host_int_status
            );
        }
        None => {
            let _ = write!(
                line,
                "HW_SPEC: {} (HOST_INT=0x{:08x})",
                hw_spec.stage.label(),
                hw_spec.host_int_status
            );
        }
    }
    line
}

fn hw_spec_status_color(hw_spec: marvell_wifi_pcie::HwSpecSnapshot) -> Color {
    if hw_spec.is_ready() {
        APP_GREEN
    } else if hw_spec.is_failed() {
        APP_RED
    } else {
        APP_AMBER
    }
}

fn scan_cmd_line(scan_cmd: marvell_wifi_pcie::ScanCmdSnapshot) -> TextBuf<192> {
    let mut line = TextBuf::new();
    let result = scan_cmd
        .result
        .map(|result| result.label())
        .unwrap_or("pending");
    let _ = write!(
        line,
        "SCAN: {} result={} len={} HOST_INT=0x{:08x}",
        scan_cmd.stage.label(),
        result,
        scan_cmd.command_len,
        scan_cmd.host_int_status
    );
    line
}

fn scan_cmd_status_color(scan_cmd: marvell_wifi_pcie::ScanCmdSnapshot) -> Color {
    if scan_cmd.is_done() {
        APP_GREEN
    } else if scan_cmd.is_failed() {
        APP_RED
    } else {
        APP_AMBER
    }
}

fn event_ring_line(event_ring: marvell_wifi_pcie::EventRingSnapshot) -> TextBuf<192> {
    let mut line = TextBuf::new();
    let result = event_ring
        .result
        .map(|result| result.label())
        .unwrap_or("pending");
    let _ = write!(
        line,
        "EVENT_RING: {} result={} rd=0x{:x} wr=0x{:x} type=0x{:04x} cause=0x{:08x} len={} HOST_INT=0x{:08x}",
        event_ring.stage.label(),
        result,
        event_ring.rdptr,
        event_ring.wrptr,
        event_ring.event_type,
        event_ring.event_cause,
        event_ring.event_len,
        event_ring.host_int_status
    );
    line
}

fn event_ring_status_color(event_ring: marvell_wifi_pcie::EventRingSnapshot) -> Color {
    if event_ring.has_event() {
        APP_GREEN
    } else if event_ring.is_failed() {
        APP_RED
    } else {
        APP_AMBER
    }
}

fn rx_ring_line(rx_ring: marvell_wifi_pcie::RxRingSnapshot) -> TextBuf<192> {
    let mut line = TextBuf::new();
    let result = rx_ring
        .result
        .map(|result| result.label())
        .unwrap_or("pending");
    let _ = write!(
        line,
        "RX_RING: {} result={} rd=0x{:x} wr=0x{:x} type=0x{:04x} len={} HOST_INT=0x{:08x}",
        rx_ring.stage.label(),
        result,
        rx_ring.rdptr,
        rx_ring.wrptr,
        rx_ring.rx_type,
        rx_ring.rx_len,
        rx_ring.host_int_status
    );
    line
}

fn rx_ring_status_color(rx_ring: marvell_wifi_pcie::RxRingSnapshot) -> Color {
    if rx_ring.has_packet() {
        APP_GREEN
    } else if rx_ring.is_failed() {
        APP_RED
    } else {
        APP_AMBER
    }
}

fn detect_ladder_label(firmware: marvell_wifi_pcie::FirmwareBringupSnapshot) -> &'static str {
    if firmware.stage == marvell_wifi_pcie::FirmwareStage::Preflight
        || firmware.stage == marvell_wifi_pcie::FirmwareStage::Idle
    {
        "pending"
    } else if firmware.result == Some(marvell_wifi_pcie::FirmwareDownloadResult::NotPresent) {
        "missing"
    } else {
        "ok"
    }
}

fn bar2_ladder_label(firmware: marvell_wifi_pcie::FirmwareBringupSnapshot) -> &'static str {
    match firmware.result {
        Some(marvell_wifi_pcie::FirmwareDownloadResult::Bar2Missing)
        | Some(marvell_wifi_pcie::FirmwareDownloadResult::Bar2NotMmio)
        | Some(marvell_wifi_pcie::FirmwareDownloadResult::MmioMapFailed(_))
        | Some(marvell_wifi_pcie::FirmwareDownloadResult::MmioProbeAllOnes) => "failed",
        _ if firmware.stage == marvell_wifi_pcie::FirmwareStage::Bar2MapOk
            || firmware.stage == marvell_wifi_pcie::FirmwareStage::Downloading
            || firmware.stage == marvell_wifi_pcie::FirmwareStage::DoorbellAck
            || firmware.stage == marvell_wifi_pcie::FirmwareStage::PollingReady
            || firmware.stage == marvell_wifi_pcie::FirmwareStage::DrvReadyQuarantined
            || firmware.stage == marvell_wifi_pcie::FirmwareStage::Ready =>
        {
            "ok"
        }
        _ => "pending",
    }
}

fn draw_current_cursor(surface: &mut FramebufferSurface, last_rect: &mut Option<CursorRect>) {
    let mouse = input::mouse_snapshot();
    let Some(rect) = mouse_cursor_rect(surface.info(), mouse) else {
        return;
    };

    draw_mouse_cursor_front(surface, mouse);
    *last_rect = Some(rect);
}

fn mouse_cursor_rect(info: FramebufferInfo, mouse: input::MouseSnapshot) -> Option<CursorRect> {
    if !mouse.seen {
        return None;
    }
    let width = info.width as usize;
    let height = info.height as usize;
    if mouse.x >= width || mouse.y >= height {
        return None;
    }
    let scale = display_scale(info);
    Some(CursorRect {
        x: mouse.x,
        y: mouse.y,
        w: usize::min(
            CURSOR_WIDTH.saturating_mul(scale),
            width.saturating_sub(mouse.x),
        ),
        h: usize::min(
            CURSOR_HEIGHT.saturating_mul(scale),
            height.saturating_sub(mouse.y),
        ),
    })
}

fn draw_mouse_cursor_front(surface: &mut FramebufferSurface, mouse: input::MouseSnapshot) {
    let fill = if mouse.buttons & 1 != 0 {
        Color::new(92, 204, 255)
    } else {
        Color::new(245, 248, 250)
    };
    let outline = Color::new(4, 8, 12);
    let x = mouse.x;
    let y = mouse.y;
    let scale = display_scale(surface.info());
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
        for (col, byte) in pattern.as_bytes().iter().copied().enumerate() {
            match byte {
                b'X' => draw_front_block(surface, x, y, col, row, scale, outline),
                b'O' => draw_front_block(surface, x, y, col, row, scale, fill),
                _ => {}
            }
        }
    }
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
    let mut dy = 0usize;
    while dy < scale {
        let mut dx = 0usize;
        while dx < scale {
            surface.set_front_pixel(start_x + dx, start_y + dy, color);
            dx += 1;
        }
        dy += 1;
    }
}

fn detail64(args: fmt::Arguments<'_>) -> TextBuf<64> {
    let mut buffer = TextBuf::new();
    let _ = buffer.write_fmt(args);
    buffer
}
