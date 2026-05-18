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
const CHAT_BUBBLE_LABEL_GAP: usize = 8;
const CHAT_BUBBLE_PADDING_BOTTOM: usize = 12;
const CHAT_BUBBLE_GAP: usize = 12;
const CURSOR_WIDTH: usize = 10;
const CURSOR_HEIGHT: usize = 16;
const CONTENT_TOP: usize = 166;

const APP_BG: Color = Color::new(13, 15, 18);
const SURFACE_BG: Color = Color::new(27, 30, 35);
const SURFACE_ALT: Color = Color::new(37, 41, 47);
const HAIRLINE: Color = Color::new(62, 68, 77);
const TEXT_MAIN: Color = Color::new(243, 246, 250);
const TEXT_MUTED: Color = Color::new(164, 174, 185);
const TEXT_FAINT: Color = Color::new(112, 123, 135);
const APP_BLUE: Color = Color::new(10, 132, 255);
const APP_GREEN: Color = Color::new(52, 199, 89);
const APP_AMBER: Color = Color::new(255, 159, 10);
const APP_RED: Color = Color::new(255, 69, 58);
const SHADOW: Color = Color::new(5, 7, 10);

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
        if point_in(x, y, 28, 18, 72, 32) {
            return console::set_view(console::UiView::Ai);
        }
        if point_in(x, y, 108, 18, 116, 32) {
            return console::set_view(console::UiView::Console);
        }
        if point_in(x, y, width.saturating_sub(86), 18, 58, 32) {
            return console::set_view(console::UiView::Settings);
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
    draw_status_strip(surface, width, snapshot);
    draw_status_detail(surface, width, snapshot);

    match console_snapshot.view {
        console::UiView::Ai => draw_chat(surface, width, height, &console_snapshot),
        console::UiView::Console => draw_console(surface, width, height, &console_snapshot),
        console::UiView::Settings => draw_settings(surface, width, height, &console_snapshot),
    }

    let uptime = detail64(format_args!("UPTIME {} MS", uptime_ms));
    text::draw_text(
        surface,
        width.saturating_sub(190),
        height.saturating_sub(28),
        uptime.as_str(),
        TEXT_FAINT,
        None,
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
    surface.fill_rect(0, 75, width, 1, HAIRLINE);

    draw_tab(
        surface,
        28,
        "AI",
        snapshot.view == console::UiView::Ai,
        snapshot.focus == console::UiFocus::NavAi,
        APP_BLUE,
    );
    draw_tab(
        surface,
        108,
        "CONSOLE",
        snapshot.view == console::UiView::Console,
        snapshot.focus == console::UiFocus::NavConsole,
        APP_BLUE,
    );
    draw_tab(
        surface,
        width.saturating_sub(86),
        "SET",
        snapshot.view == console::UiView::Settings,
        snapshot.focus == console::UiFocus::NavSettings,
        APP_BLUE,
    );

    text::draw_text(surface, 248, 20, "raisOS", TEXT_MAIN, None);
    text::draw_text(surface, 248, 42, "Direct AI Host", TEXT_MUTED, None);
}

fn draw_tab(
    surface: &mut FramebufferSurface,
    x: usize,
    label: &str,
    active: bool,
    focused: bool,
    color: Color,
) {
    let bg = if active { color } else { SURFACE_ALT };
    let fg = if active { SURFACE_BG } else { TEXT_MUTED };
    let width = if label.len() > 3 { 116 } else { 72 };
    draw_soft_rect(surface, x, 18, width, 32, bg);
    if !active {
        draw_rect_outline(surface, x, 18, width, 32, HAIRLINE);
    }
    if focused {
        draw_focus_outline(surface, x, 18, width, 32);
    }
    text::draw_text(surface, x + 14, 30, label, fg, None);
}

fn draw_status_strip(surface: &mut FramebufferSurface, width: usize, snapshot: &SystemSnapshot) {
    let y = 90usize;
    draw_status_chip(surface, 40, y, 156, "Net", &snapshot.network);
    draw_status_chip(surface, 212, y, 156, "WiFi", &snapshot.wifi);
    draw_status_chip(surface, 384, y, 184, "Input", &snapshot.input);
    draw_status_chip(surface, 584, y, 188, "USB", &snapshot.usb_xhci);
    draw_status_chip(surface, 788, y, 156, "RNG", &snapshot.entropy);
    surface.fill_rect(24, 154, width.saturating_sub(48), 1, HAIRLINE);
}

fn draw_status_detail(surface: &mut FramebufferSurface, width: usize, snapshot: &SystemSnapshot) {
    text::draw_text(surface, 44, 134, "USB", TEXT_FAINT, None);
    let max_chars = width.saturating_sub(100) / FONT_ADVANCE;
    draw_truncated_text(
        surface,
        92,
        134,
        snapshot.usb_xhci.detail.as_str(),
        max_chars,
        TEXT_MUTED,
    );
}

fn draw_status_chip(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    label: &'static str,
    line: &StatusLine,
) {
    draw_soft_rect(surface, x, y, width, 30, SURFACE_BG);
    draw_rect_outline(surface, x, y, width, 30, HAIRLINE);
    surface.fill_rect(x + 12, y + 11, 8, 8, row_state_color(line.state));
    text::draw_text(surface, x + 28, y + 11, label, TEXT_MAIN, None);
    text::draw_text(
        surface,
        x + 86,
        y + 11,
        line.state.as_str(),
        TEXT_MUTED,
        None,
    );
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
    text::draw_text(surface, 44, top + 18, "Chat", TEXT_MAIN, None);
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

    let input_y = height.saturating_sub(74);
    draw_soft_rect(
        surface,
        24,
        input_y,
        width.saturating_sub(48),
        50,
        SURFACE_BG,
    );
    draw_rect_outline(surface, 24, input_y, width.saturating_sub(48), 50, HAIRLINE);
    draw_soft_rect(
        surface,
        42,
        input_y + 10,
        width.saturating_sub(118),
        30,
        SURFACE_ALT,
    );
    if snapshot.focus == console::UiFocus::ChatInput {
        draw_focus_outline(surface, 42, input_y + 10, width.saturating_sub(118), 30);
    }
    let input = snapshot.chat_input.as_str();
    let prompt = if input.is_empty() {
        "TYPE MESSAGE AND PRESS ENTER"
    } else {
        input
    };
    text::draw_text(
        surface,
        58,
        input_y + 21,
        prompt,
        if input.is_empty() {
            TEXT_FAINT
        } else {
            TEXT_MAIN
        },
        None,
    );
    draw_soft_rect(
        surface,
        width.saturating_sub(58),
        input_y + 16,
        22,
        18,
        APP_BLUE,
    );
    text::draw_text(
        surface,
        width.saturating_sub(54),
        input_y + 21,
        ">",
        SURFACE_BG,
        None,
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
            APP_BLUE,
            SURFACE_BG,
            "You",
        ),
        console::ChatSpeaker::Assistant => (48, SURFACE_BG, TEXT_MAIN, "AI"),
        console::ChatSpeaker::System => (
            width.saturating_sub(layout.width) / 2,
            Color::new(232, 236, 241),
            TEXT_MUTED,
            "Sys",
        ),
    };

    draw_bubble_rect(surface, x, y, layout.width, layout.height, color);
    text::draw_text(
        surface,
        x + CHAT_BUBBLE_PADDING_X,
        y + CHAT_BUBBLE_PADDING_TOP,
        label,
        if speaker == console::ChatSpeaker::User {
            Color::new(210, 232, 255)
        } else {
            TEXT_FAINT
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
) {
    surface.fill_rect(x + 3, y + 4, width, height, SHADOW);
    draw_soft_rect(surface, x, y, width, height, color);
    if color == SURFACE_BG {
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
    if width < 18 || height < 18 {
        surface.fill_rect(x + 3, y, width.saturating_sub(6), height, color);
        surface.fill_rect(x, y + 3, width, height.saturating_sub(6), color);
        return;
    }
    surface.fill_rect(x + 8, y, width.saturating_sub(16), height, color);
    surface.fill_rect(
        x + 4,
        y + 2,
        width.saturating_sub(8),
        height.saturating_sub(4),
        color,
    );
    surface.fill_rect(
        x + 2,
        y + 4,
        width.saturating_sub(4),
        height.saturating_sub(8),
        color,
    );
    surface.fill_rect(x, y + 8, width, height.saturating_sub(16), color);
}

fn draw_rect_outline(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
) {
    if width == 0 || height == 0 {
        return;
    }
    if width < 18 || height < 18 {
        surface.fill_rect(x + 3, y, width.saturating_sub(6), 1, color);
        surface.fill_rect(
            x + 3,
            y + height.saturating_sub(1),
            width.saturating_sub(6),
            1,
            color,
        );
        surface.fill_rect(x, y + 3, 1, height.saturating_sub(6), color);
        surface.fill_rect(
            x + width.saturating_sub(1),
            y + 3,
            1,
            height.saturating_sub(6),
            color,
        );
        return;
    }
    surface.fill_rect(x + 8, y, width.saturating_sub(16), 1, color);
    surface.fill_rect(
        x + 8,
        y + height.saturating_sub(1),
        width.saturating_sub(16),
        1,
        color,
    );
    surface.fill_rect(x, y + 8, 1, height.saturating_sub(16), color);
    surface.fill_rect(
        x + width.saturating_sub(1),
        y + 8,
        1,
        height.saturating_sub(16),
        color,
    );
    surface.set_pixel(x + 4, y + 2, color);
    surface.set_pixel(x + 2, y + 4, color);
    surface.set_pixel(x + width.saturating_sub(5), y + 2, color);
    surface.set_pixel(x + width.saturating_sub(3), y + 4, color);
    surface.set_pixel(x + 4, y + height.saturating_sub(3), color);
    surface.set_pixel(x + 2, y + height.saturating_sub(5), color);
    surface.set_pixel(
        x + width.saturating_sub(5),
        y + height.saturating_sub(3),
        color,
    );
    surface.set_pixel(
        x + width.saturating_sub(3),
        y + height.saturating_sub(5),
        color,
    );
}

fn draw_focus_outline(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    draw_rect_outline(
        surface,
        x.saturating_sub(3),
        y.saturating_sub(3),
        width.saturating_add(6),
        height.saturating_add(6),
        APP_BLUE,
    );
}

fn draw_settings_action(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    label: &str,
    focused: bool,
) {
    draw_soft_rect(surface, x, y, width, 38, SURFACE_BG);
    draw_rect_outline(surface, x, y, width, 38, HAIRLINE);
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
    text::draw_text(surface, 44, top + 18, "Console", TEXT_MAIN, None);

    let mut line_y = top + 56;
    let mut idx = 0usize;
    while idx < snapshot.lines.len() {
        let line = snapshot.lines[idx].as_str();
        if !line.is_empty() {
            text::draw_text(surface, 44, line_y, line, TEXT_MUTED, None);
        }
        line_y += 18;
        idx += 1;
    }

    let input_y = height.saturating_sub(74);
    draw_soft_rect(
        surface,
        24,
        input_y,
        width.saturating_sub(48),
        50,
        SURFACE_BG,
    );
    draw_rect_outline(surface, 24, input_y, width.saturating_sub(48), 50, HAIRLINE);
    if snapshot.focus == console::UiFocus::ConsoleInput {
        draw_focus_outline(surface, 24, input_y, width.saturating_sub(48), 50);
    }
    text::draw_text(
        surface,
        44,
        input_y + 21,
        snapshot.input.as_str(),
        TEXT_MAIN,
        None,
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
    text::draw_text(surface, 44, top + 18, "Settings", TEXT_MAIN, None);

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
    text::draw_text(surface, 56, top + 62, "Provider", TEXT_MUTED, None);
    text::draw_text(
        surface,
        220,
        top + 62,
        snapshot.provider_name,
        TEXT_MAIN,
        None,
    );
    text::draw_text(surface, 56, top + 96, "Model", TEXT_MUTED, None);
    text::draw_text(
        surface,
        220,
        top + 96,
        snapshot.provider_model,
        TEXT_MAIN,
        None,
    );
    text::draw_text(surface, 56, top + 130, "API Key", TEXT_MUTED, None);
    text::draw_text(surface, 220, top + 130, key_state, TEXT_MAIN, None);
    text::draw_text(surface, 56, top + 164, "WiFi SSID", TEXT_MUTED, None);
    text::draw_text(surface, 220, top + 164, wifi_ssid, TEXT_MAIN, None);
    text::draw_text(surface, 56, top + 198, "WiFi Key", TEXT_MUTED, None);
    text::draw_text(surface, 220, top + 198, wifi_key_state, TEXT_MAIN, None);

    draw_soft_rect(surface, 56, top + 238, 760, 226, SURFACE_ALT);
    draw_rect_outline(surface, 56, top + 238, 760, 226, HAIRLINE);
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

    let input_y = height.saturating_sub(74);
    draw_soft_rect(
        surface,
        24,
        input_y,
        width.saturating_sub(48),
        50,
        SURFACE_BG,
    );
    draw_rect_outline(surface, 24, input_y, width.saturating_sub(48), 50, HAIRLINE);
    if snapshot.settings_entry_active || snapshot.focus == console::UiFocus::ConsoleInput {
        draw_focus_outline(surface, 24, input_y, width.saturating_sub(48), 50);
    }
    text::draw_text(
        surface,
        44,
        input_y + 21,
        snapshot.input.as_str(),
        TEXT_MAIN,
        None,
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
