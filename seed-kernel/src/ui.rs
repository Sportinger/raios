use crate::framebuffer::{Color, FramebufferInfo, FramebufferSurface};
use crate::system_status::{RowState, SnapshotStates, StatusLine, SystemSnapshot, TextBuf};
use crate::{console, input, serial, text};
use core::fmt::{self, Write};

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
const CONTENT_TOP: usize = 166;
const HEADER_TAB_START_X: usize = 236;
const HEADER_TAB_GAP: usize = 24;
const HEADER_TAB_HIT_Y: usize = 18;
const HEADER_TAB_HIT_H: usize = 58;
const HEADER_TAB_LABEL_Y: usize = 36;
const HEADER_TAB_UNDERLINE_Y: usize = 73;
const INPUT_FIELD_X: usize = 24;
const INPUT_FIELD_RIGHT: usize = 72;
const INPUT_FIELD_H: usize = 36;
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

pub struct StatusUi {
    surface: Option<FramebufferSurface>,
    last_states: Option<SnapshotStates>,
    last_draw_states: Option<SnapshotStates>,
    last_mouse_buttons: u8,
    last_cursor_rect: Option<CursorRect>,
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
        }
    }

    pub fn render(&mut self, uptime_ms: u64, runtime: RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, false);
    }

    pub fn render_forced(&mut self, uptime_ms: u64, runtime: RuntimeStatus) {
        self.render_inner(uptime_ms, runtime, true);
    }

    fn render_inner(&mut self, uptime_ms: u64, runtime: RuntimeStatus, force_draw: bool) {
        let framebuffer = self.surface.as_ref().map(|surface| surface.info());
        let snapshot = SystemSnapshot::collect(framebuffer, runtime);
        self.log_transitions(&snapshot);

        let states = snapshot.states();
        let should_draw = force_draw || self.last_draw_states != Some(states);

        if should_draw {
            if let Some(surface) = self.surface.as_mut() {
                draw(surface, uptime_ms, &snapshot);
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
        let x = mouse.x / scale;
        let y = mouse.y / scale;
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
            if point_in(x, y, 72, top + 250, 342, 38) {
                return console::activate_focus(console::UiFocus::SettingsProvider);
            }
            if point_in(x, y, 430, top + 250, 342, 38) {
                return console::activate_focus(console::UiFocus::SettingsApiKey);
            }
            if point_in(x, y, 72, top + 304, 342, 38) {
                return console::activate_focus(console::UiFocus::SettingsClear);
            }
            if point_in(x, y, 430, top + 304, 342, 38) {
                return console::activate_focus(console::UiFocus::SettingsWifiSsid);
            }
            if point_in(x, y, 72, top + 358, 342, 38) {
                return console::activate_focus(console::UiFocus::SettingsWifiPassphrase);
            }
            if point_in(x, y, 430, top + 358, 342, 38) {
                return console::activate_focus(console::UiFocus::SettingsWifiClear);
            }
            if point_in(x, y, 72, top + 412, 700, 38) {
                return console::activate_focus(console::UiFocus::SettingsClose);
            }
        }

        false
    }

    fn log_transitions(&mut self, snapshot: &SystemSnapshot) {
        let states = snapshot.states();
        let previous = self.last_states;

        log_transition(previous.map(|prev| prev.framebuffer), &snapshot.framebuffer);
        log_transition(previous.map(|prev| prev.entropy), &snapshot.entropy);
        log_transition(previous.map(|prev| prev.usb_xhci), &snapshot.usb_xhci);
        log_transition(previous.map(|prev| prev.wifi), &snapshot.wifi);
        log_transition(previous.map(|prev| prev.network), &snapshot.network);
        log_transition(previous.map(|prev| prev.input), &snapshot.input);

        self.last_states = Some(states);
    }
}

fn point_in(px: usize, py: usize, x: usize, y: usize, w: usize, h: usize) -> bool {
    px >= x && px < x.saturating_add(w) && py >= y && py < y.saturating_add(h)
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

fn draw(surface: &mut FramebufferSurface, uptime_ms: u64, snapshot: &SystemSnapshot) {
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

    text::draw_text(surface, 24, 20, "raiOS", TEXT_MAIN, None);
    text::draw_text(surface, 84, 20, "Direct AI Host", TEXT_FAINT, None);

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
    x = draw_status_chip(surface, width, x, y, "WiFi", &snapshot.wifi);
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
    surface.fill_rect(24, 154, width.saturating_sub(48), 1, HAIRLINE_HI);
}

fn draw_status_detail(
    surface: &mut FramebufferSurface,
    width: usize,
    view: console::UiView,
    snapshot: &SystemSnapshot,
) {
    if view != console::UiView::Console {
        return;
    }
    text::draw_text(surface, 44, 134, "USB", TEXT_FAINT, None);
    let max_chars = width.saturating_sub(100) / FONT_ADVANCE;
    draw_truncated_text(
        surface,
        92,
        134,
        snapshot.usb_xhci.detail.as_str(),
        max_chars,
        TEXT_FAINT,
    );
}

fn draw_status_chip(
    surface: &mut FramebufferSurface,
    screen_width: usize,
    x: usize,
    y: usize,
    label: &'static str,
    line: &StatusLine,
) -> usize {
    let width = 28usize
        .saturating_add(label.len().saturating_mul(FONT_ADVANCE))
        .saturating_add(8)
        .saturating_add(row_state_text_len(line.state).saturating_mul(FONT_ADVANCE))
        .saturating_add(14);
    if x.saturating_add(width) > screen_width.saturating_sub(24) {
        return x;
    }

    draw_soft_rect_r6(surface, x, y, width, 30, SURFACE_BG);
    draw_rect_outline_r6(surface, x, y, width, 30, HAIRLINE);
    draw_status_dot(surface, x + 12, y + 12, row_state_color(line.state));
    text::draw_text(surface, x + 28, y + 11, label, TEXT_MAIN, None);
    text::draw_text(
        surface,
        x + 28 + label.len().saturating_mul(FONT_ADVANCE) + 8,
        y + 11,
        line.state.as_str(),
        TEXT_MUTED,
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
    surface.fill_rect(x, y + 1, 3, 14, APP_BLUE);
    text::draw_text(surface, x + 12, y, title, TEXT_MAIN, None);
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
    draw_soft_rect(
        surface,
        24,
        top,
        width.saturating_sub(48),
        bottom.saturating_sub(top),
        SURFACE_BG,
    );
    draw_rect_outline(
        surface,
        24,
        top,
        width.saturating_sub(48),
        bottom.saturating_sub(top),
        HAIRLINE,
    );
    draw_panel_title(surface, 44, top + 18, "Chat");
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
    text::draw_text(
        surface,
        width.saturating_sub(360),
        top + 18,
        provider.as_str(),
        TEXT_MUTED,
        None,
    );

    let has_messages = chat_has_messages(snapshot);
    if !has_messages {
        draw_centered_text(
            surface,
            width,
            top.saturating_add(bottom.saturating_sub(top) / 2),
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
    draw_soft_rect_r6(surface, x, y, width, 38, SURFACE_ALT);
    draw_rect_outline_r6(surface, x, y, width, 38, HAIRLINE);
    if focused {
        draw_focus_outline(surface, x, y, width, 38);
    }
    text::draw_text(surface, x + 18, y + 15, label, TEXT_MAIN, None);
}

fn draw_console(
    surface: &mut FramebufferSurface,
    width: usize,
    height: usize,
    snapshot: &console::ConsoleSnapshot,
) {
    let top = CONTENT_TOP;
    draw_soft_rect(
        surface,
        24,
        top,
        width.saturating_sub(48),
        height.saturating_sub(top + 42),
        SURFACE_BG,
    );
    draw_rect_outline(
        surface,
        24,
        top,
        width.saturating_sub(48),
        height.saturating_sub(top + 42),
        HAIRLINE,
    );
    draw_panel_title(surface, 44, top + 18, "Console");

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
    draw_soft_rect(
        surface,
        24,
        top,
        width.saturating_sub(48),
        height.saturating_sub(top + 42),
        SURFACE_BG,
    );
    draw_rect_outline(
        surface,
        24,
        top,
        width.saturating_sub(48),
        height.saturating_sub(top + 42),
        HAIRLINE,
    );
    draw_panel_title(surface, 44, top + 18, "Settings");

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
    draw_settings_row(
        surface,
        width,
        top + 62,
        "Provider",
        snapshot.provider_name,
        true,
    );
    draw_settings_row(
        surface,
        width,
        top + 102,
        "Model",
        snapshot.provider_model,
        true,
    );
    draw_settings_row(
        surface,
        width,
        top + 142,
        "API Key",
        key_state,
        snapshot.api_key_set,
    );
    draw_settings_row(
        surface,
        width,
        top + 182,
        "WiFi SSID",
        wifi_ssid,
        !snapshot.wifi_ssid.is_empty(),
    );
    draw_settings_row(
        surface,
        width,
        top + 222,
        "WiFi Key",
        wifi_key_state,
        snapshot.wifi_passphrase_set,
    );

    draw_settings_action(
        surface,
        72,
        top + 250,
        342,
        "Provider Status",
        snapshot.focus == console::UiFocus::SettingsProvider,
    );
    draw_settings_action(
        surface,
        430,
        top + 250,
        342,
        "Enter API Key",
        snapshot.focus == console::UiFocus::SettingsApiKey,
    );
    draw_settings_action(
        surface,
        72,
        top + 304,
        342,
        "Clear API Key",
        snapshot.focus == console::UiFocus::SettingsClear,
    );
    draw_settings_action(
        surface,
        430,
        top + 304,
        342,
        "WiFi SSID",
        snapshot.focus == console::UiFocus::SettingsWifiSsid,
    );
    draw_settings_action(
        surface,
        72,
        top + 358,
        342,
        "WiFi Key",
        snapshot.focus == console::UiFocus::SettingsWifiPassphrase,
    );
    draw_settings_action(
        surface,
        430,
        top + 358,
        342,
        "Clear WiFi",
        snapshot.focus == console::UiFocus::SettingsWifiClear,
    );
    draw_settings_action(
        surface,
        72,
        top + 412,
        700,
        "Close Settings",
        snapshot.focus == console::UiFocus::SettingsClose,
    );

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
