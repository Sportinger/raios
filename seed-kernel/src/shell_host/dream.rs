//! Dream shell presentation ported from the executable UI lab specification.
//!
//! The module owns presentation state only.  It projects existing typed kernel
//! snapshots and reports bounded hit targets; it does not manufacture actions.

extern crate alloc;

use alloc::{vec, vec::Vec};

use crate::framebuffer::{Color, FramebufferSurface};
use crate::system_status::{RowState, SystemSnapshot};
use crate::{agent_build_loop, console, provider, text, wifi};

use super::genesis::{
    draw_outline, LogicalRect, APP_AMBER, APP_BLUE, APP_GREEN, APP_RED, HAIRLINE, TEXT_FAINT,
    TEXT_MAIN, TEXT_MUTED,
};
use super::{context, vault_flow};

const DRAW_SCALE: usize = 2;
const OUTER_MARGIN: usize = 14;
const RAIL_WIDTH: usize = 130;
const CLOSED_SEAM_GAP: usize = 20;
const CLOSED_CENTER_WIDTH: usize = 8;
const BRACKET_TOP: usize = 40;
const BRACKET_BOTTOM_MARGIN: usize = 18;
const BRACKET_CENTER_OFFSET: usize = 17;
const AMBIENT_BRACKET_AMP: usize = 14;
const OPEN_BRACKET_AMP: usize = 16;
const MIN_CENTER_WIDTH: usize = 240;
const MAX_CENTER_WIDTH: usize = 480;
const MIN_CENTER_CORRIDOR: usize = 132;
const COMPOSER_BOTTOM_OFFSET: usize = 44;
const CONVERSATION_TOP: usize = 70;
const CONVERSATION_COMPOSER_GAP: usize = 16;
const LAYOUT_ANIM_MS: u64 = 500;
const Q: i64 = 1_000_000;
const BUTTON_WIDTH: usize = 130;
const BUTTON_HEIGHT: usize = 16;
const CHUNKY_ADVANCE: usize = 9;
const CONVERSATION_ROW_HEIGHT: usize = 12;

const SHADE_DARK: Color = Color::new(8, 9, 11);
const SHADE_MID: Color = Color::new(11, 13, 16);
const SHADE_LIGHT: Color = Color::new(14, 16, 20);
const CANDLE_TONE: u32 = rgb(34, 41, 51);
const DIM_TEXT: Color = Color::new(92, 100, 112);
const DISABLED_TEXT: Color = Color::new(69, 77, 90);
const BLUE_LIGHT: Color = Color::new(124, 179, 245);
const BRACKET_GLOW: Color = Color::new(21, 26, 33);
const BRACKET_LOW: Color = Color::new(62, 67, 76);
const BRACKET_MID: Color = Color::new(74, 85, 104);
const BRACKET_HIGH: Color = Color::new(94, 112, 137);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tab {
    Chat,
    Console,
    Build,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HitTarget {
    Composer,
    TabChat,
    TabConsole,
    TabBuild,
    AiSetup,
    WifiSetup,
    Recovery,
    Approve,
    Inert,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HoverTarget {
    None,
    TabChat,
    TabConsole,
    TabBuild,
    AiSetup,
    WifiSetup,
    Recovery,
    Approve,
    Calculator,
    Editor,
    Playground,
}

pub(crate) struct DreamState {
    background: Option<DreamBackground>,
    physical_width: usize,
    physical_height: usize,
    layout_t: i64,
    requested_goal: i64,
    active_goal: i64,
    transition_start_t: i64,
    transition_started_ms: u64,
    transitioning: bool,
    animation_poll_divider: u8,
    tab: Tab,
    hover: HoverTarget,
}

impl DreamState {
    pub(crate) const fn new() -> Self {
        Self {
            background: None,
            physical_width: 0,
            physical_height: 0,
            layout_t: -Q,
            requested_goal: -Q,
            active_goal: -Q,
            transition_start_t: -Q,
            transition_started_ms: 0,
            transitioning: false,
            animation_poll_divider: 0,
            tab: Tab::Chat,
            hover: HoverTarget::None,
        }
    }

    pub(crate) fn update_for_frame(&mut self, uptime_ms: u64) -> bool {
        let before = self.layout_t;
        if self.requested_goal != self.active_goal {
            self.advance_active_transition(uptime_ms);
            self.transition_start_t = self.layout_t;
            self.active_goal = self.requested_goal;
            self.transition_started_ms = uptime_ms;
            self.transitioning = self.transition_start_t != self.active_goal;
        }
        self.advance_active_transition(uptime_ms);
        self.layout_t != before
    }

    pub(crate) fn update_pointer(
        &mut self,
        seen: bool,
        x: usize,
        y: usize,
        interactions_enabled: bool,
        approve_enabled: bool,
    ) -> bool {
        let geometry = self.geometry();
        let center = geometry.center_rect();
        let center_hovered = seen
            && x >= center.x.saturating_sub(20)
            && x <= center.x.saturating_add(center.w).saturating_add(20)
            && y >= geometry.bracket_top()
            && y <= geometry.bracket_bottom();
        let goal = if center_hovered { Q } else { -Q };
        let next_hover = if seen && interactions_enabled {
            self.hover_at(x, y, approve_enabled)
        } else {
            HoverTarget::None
        };
        let changed = self.requested_goal != goal || self.hover != next_hover;
        self.requested_goal = goal;
        self.hover = next_hover;
        changed
    }

    pub(crate) fn animation_frame_due(&mut self) -> bool {
        if !self.transitioning && self.requested_goal == self.active_goal {
            self.animation_poll_divider = 0;
            return false;
        }
        self.animation_poll_divider = self.animation_poll_divider.saturating_add(1);
        if self.animation_poll_divider < 2 {
            return false;
        }
        self.animation_poll_divider = 0;
        true
    }

    pub(crate) fn set_tab(&mut self, tab: Tab) -> bool {
        if self.tab == tab {
            return false;
        }
        self.tab = tab;
        true
    }

    pub(crate) const fn tab(&self) -> Tab {
        self.tab
    }

    pub(crate) fn hit_test(&self, x: usize, y: usize, approve_enabled: bool) -> Option<HitTarget> {
        let geometry = self.geometry();
        if self.layout_t > -Q / 2 {
            if point_in(x, y, composer_rect(geometry)) {
                return Some(HitTarget::Composer);
            }
            for (rect, target) in tab_rects(geometry) {
                if point_in(x, y, rect) {
                    return Some(target);
                }
            }
            let left = geometry.left_rect();
            let right = geometry.right_rect();
            let live = [
                (button_rect(left.x, 204), HitTarget::AiSetup),
                (button_rect(left.x, 226), HitTarget::WifiSetup),
                (button_rect(left.x, 248), HitTarget::Recovery),
            ];
            for (rect, target) in live {
                if point_in(x, y, rect) {
                    return Some(target);
                }
            }
            if point_in(x, y, button_rect(right.x, 170)) {
                return Some(if approve_enabled {
                    HitTarget::Approve
                } else {
                    HitTarget::Inert
                });
            }
            if point_in(x, y, item_rect(right.x, 56, right.w))
                || point_in(x, y, item_rect(right.x, 70, right.w))
                || point_in(x, y, button_rect(right.x, 222))
            {
                return Some(HitTarget::Inert);
            }
        } else {
            let left = geometry.left_rect();
            let right = geometry.right_rect();
            let live = [
                (button_rect(left.x, 146), HitTarget::AiSetup),
                (
                    button_rect(left.x + BUTTON_WIDTH + 12, 146),
                    HitTarget::WifiSetup,
                ),
                (button_rect(left.x, 168), HitTarget::Recovery),
            ];
            for (rect, target) in live {
                if point_in(x, y, rect) {
                    return Some(target);
                }
            }
            if point_in(x, y, button_rect(right.x, 116)) {
                return Some(if approve_enabled {
                    HitTarget::Approve
                } else {
                    HitTarget::Inert
                });
            }
            let column_gap = 12;
            let column_width = right.w.saturating_sub(column_gap) / 2;
            let playground_x = right.x + column_width + column_gap;
            if point_in(x, y, item_rect(right.x, 242, column_width))
                || point_in(x, y, item_rect(right.x, 255, column_width))
                || point_in(x, y, button_rect(playground_x, 242))
            {
                return Some(HitTarget::Inert);
            }
        }
        None
    }

    pub(crate) fn conversation_contains(&self, x: usize, y: usize) -> bool {
        self.tab == Tab::Chat
            && self.layout_t > -Q / 2
            && point_in(x, y, conversation_rect(self.geometry()))
    }

    pub(crate) fn conversation_max_chars(&self) -> usize {
        let center = self.geometry().center_rect();
        center.w.saturating_mul(2).saturating_sub(10) / 9
    }

    pub(crate) fn conversation_visible_rows(&self) -> usize {
        conversation_rect(self.geometry()).h / CONVERSATION_ROW_HEIGHT
    }

    pub(crate) fn composer_cursor_rect(
        &self,
        snapshot: &console::ConsoleSnapshot,
    ) -> Option<(usize, usize, usize, usize)> {
        if self.tab != Tab::Chat
            || self.layout_t <= -Q / 2
            || snapshot.view != console::UiView::Ai
            || snapshot.focus != console::UiFocus::ChatInput
        {
            return None;
        }
        let center = self.geometry().center_rect();
        let max_chars = center.w / CHUNKY_ADVANCE;
        let visible_limit = max_chars.saturating_sub(3);
        let (visible, _) = trailing_chars(snapshot.chat_input.as_str(), visible_limit);
        let cursor_x = center
            .x
            .saturating_add((2 + visible.chars().count()).saturating_mul(CHUNKY_ADVANCE));
        Some((
            cursor_x * DRAW_SCALE,
            self.geometry().composer_y().saturating_sub(3) * DRAW_SCALE,
            4,
            30,
        ))
    }

    fn advance_active_transition(&mut self, uptime_ms: u64) {
        if !self.transitioning {
            return;
        }
        let elapsed = uptime_ms.saturating_sub(self.transition_started_ms);
        if elapsed >= LAYOUT_ANIM_MS {
            self.layout_t = self.active_goal;
            self.transitioning = false;
            return;
        }
        let progress = (elapsed.saturating_mul(Q as u64) / LAYOUT_ANIM_MS) as i64;
        self.layout_t = lerp_q(
            self.transition_start_t,
            self.active_goal,
            ease_in_out_q(progress),
        );
    }

    fn geometry(&self) -> DreamGeometry {
        DreamGeometry::at(self.physical_width, self.physical_height, self.layout_t)
    }

    fn set_surface_size(&mut self, physical_width: usize, physical_height: usize) {
        self.physical_width = physical_width;
        self.physical_height = physical_height;
    }

    fn hover_at(&self, x: usize, y: usize, approve_enabled: bool) -> HoverTarget {
        let geometry = self.geometry();
        if self.layout_t > -Q / 2 {
            for (rect, target) in tab_hover_rects(geometry) {
                if point_in(x, y, rect) {
                    return target;
                }
            }
            let left = geometry.left_rect();
            let right = geometry.right_rect();
            let targets = [
                (button_rect(left.x, 204), HoverTarget::AiSetup),
                (button_rect(left.x, 226), HoverTarget::WifiSetup),
                (button_rect(left.x, 248), HoverTarget::Recovery),
                (item_rect(right.x, 56, right.w), HoverTarget::Calculator),
                (item_rect(right.x, 70, right.w), HoverTarget::Editor),
                (button_rect(right.x, 222), HoverTarget::Playground),
            ];
            for (rect, target) in targets {
                if point_in(x, y, rect) {
                    return target;
                }
            }
            if approve_enabled && point_in(x, y, button_rect(right.x, 170)) {
                return HoverTarget::Approve;
            }
        } else {
            let left = geometry.left_rect();
            let right = geometry.right_rect();
            let column_gap = 12;
            let column_width = right.w.saturating_sub(column_gap) / 2;
            let playground_x = right.x + column_width + column_gap;
            let targets = [
                (button_rect(left.x, 146), HoverTarget::AiSetup),
                (
                    button_rect(left.x + BUTTON_WIDTH + 12, 146),
                    HoverTarget::WifiSetup,
                ),
                (button_rect(left.x, 168), HoverTarget::Recovery),
                (
                    item_rect(right.x, 242, column_width),
                    HoverTarget::Calculator,
                ),
                (item_rect(right.x, 255, column_width), HoverTarget::Editor),
                (button_rect(playground_x, 242), HoverTarget::Playground),
            ];
            for (rect, target) in targets {
                if point_in(x, y, rect) {
                    return target;
                }
            }
            if approve_enabled && point_in(x, y, button_rect(right.x, 116)) {
                return HoverTarget::Approve;
            }
        }
        HoverTarget::None
    }
}

pub(crate) fn render(
    surface: &mut FramebufferSurface,
    state: &mut DreamState,
    system: &SystemSnapshot,
    console_snapshot: &console::ConsoleSnapshot,
    conversation_scroll_rows: usize,
    recovery_open: bool,
) {
    let info = surface.info();
    let physical_width = info.width as usize;
    let physical_height = info.height as usize;
    state.set_surface_size(physical_width, physical_height);
    let background_matches = match state.background.as_ref() {
        Some(background) => {
            background.width == physical_width && background.height == physical_height
        }
        None => false,
    };
    if !background_matches {
        state.background = Some(build_dream_background(physical_width, physical_height));
    }
    if let Some(background) = state.background.as_mut() {
        copy_background(surface, background);
    }
    surface.set_draw_scale(DRAW_SCALE);

    let geometry = state.geometry();
    draw_bracket(
        surface,
        geometry.bracket_left,
        -1,
        geometry.bracket_amp,
        geometry.bracket_hook,
        geometry.bracket_top(),
        geometry.bracket_bottom(),
    );
    draw_bracket(
        surface,
        geometry.bracket_right,
        1,
        geometry.bracket_amp,
        geometry.bracket_hook,
        geometry.bracket_top(),
        geometry.bracket_bottom(),
    );

    let projection = context::project(system, &provider::snapshot(), wifi::snapshot());
    let build = agent_build_loop::snapshot();
    let approve_enabled = real_approval_available();
    if state.layout_t > -Q / 2 {
        draw_tabs(surface, state, geometry);
        match state.tab {
            Tab::Chat => draw_chat(
                surface,
                geometry,
                console_snapshot,
                conversation_scroll_rows,
            ),
            Tab::Console => draw_console(surface, geometry, console_snapshot),
            Tab::Build => draw_build_center(surface, geometry, &build),
        }
        draw_ambient_rails(
            surface,
            state,
            geometry,
            system,
            projection,
            &build,
            recovery_open,
            approve_enabled,
        );
    } else {
        draw_closed_rails(
            surface,
            state,
            geometry,
            system,
            projection,
            &build,
            recovery_open,
            approve_enabled,
        );
    }
}

pub(crate) fn real_approval_available() -> bool {
    crate::agent_protocol_project_install::pending_physical_approval()
        || crate::granted_candidate_service::pending_approval()
        || crate::workspace_candidate_service::pending_approval()
        || crate::program_workspace::snapshot().present
}

#[derive(Clone, Copy)]
struct DreamGeometry {
    logical_width: usize,
    logical_height: usize,
    center_x: i64,
    center_w: i64,
    bracket_left: i64,
    bracket_right: i64,
    bracket_amp: i64,
    bracket_hook: i64,
    left_x: i64,
    left_w: i64,
    right_x: i64,
    right_w: i64,
}

impl DreamGeometry {
    fn keyframe(
        logical_width: usize,
        logical_height: usize,
        center_x: i64,
        center_w: i64,
        bracket_left: i64,
        bracket_right: i64,
        bracket_amp: i64,
        bracket_hook: i64,
        left_x: i64,
        left_w: i64,
        right_x: i64,
        right_w: i64,
    ) -> Self {
        Self {
            logical_width,
            logical_height,
            center_x: center_x * Q,
            center_w: center_w * Q,
            bracket_left: bracket_left * Q,
            bracket_right: bracket_right * Q,
            bracket_amp: bracket_amp * Q,
            bracket_hook: bracket_hook * Q,
            left_x: left_x * Q,
            left_w: left_w * Q,
            right_x: right_x * Q,
            right_w: right_w * Q,
        }
    }

    fn at(physical_width: usize, physical_height: usize, layout_t: i64) -> Self {
        let logical_width = physical_width / DRAW_SCALE;
        let logical_height = physical_height / DRAW_SCALE;
        let midpoint = logical_width / 2;
        let rail_width = responsive_rail_width(logical_width);
        let right_x = logical_width
            .saturating_sub(OUTER_MARGIN)
            .saturating_sub(rail_width);

        let closed_center_x = midpoint.saturating_sub(CLOSED_CENTER_WIDTH / 2);
        let closed_left_width = midpoint
            .saturating_sub(CLOSED_SEAM_GAP)
            .saturating_sub(OUTER_MARGIN);
        let closed_right_x = midpoint
            .saturating_add(CLOSED_SEAM_GAP)
            .min(logical_width.saturating_sub(OUTER_MARGIN));
        let closed = Self::keyframe(
            logical_width,
            logical_height,
            closed_center_x as i64,
            CLOSED_CENTER_WIDTH.min(logical_width) as i64,
            midpoint.saturating_sub(1) as i64,
            midpoint.saturating_add(1).min(logical_width) as i64,
            7,
            0,
            OUTER_MARGIN.min(logical_width) as i64,
            closed_left_width as i64,
            closed_right_x as i64,
            logical_width
                .saturating_sub(OUTER_MARGIN)
                .saturating_sub(closed_right_x) as i64,
        );

        let ambient_center_width =
            responsive_center_width(logical_width, rail_width, 27, 64, AMBIENT_BRACKET_AMP);
        let ambient_center_x = midpoint.saturating_sub(ambient_center_width / 2);
        let ambient = Self::keyframe(
            logical_width,
            logical_height,
            ambient_center_x as i64,
            ambient_center_width as i64,
            ambient_center_x.saturating_sub(BRACKET_CENTER_OFFSET) as i64,
            ambient_center_x
                .saturating_add(ambient_center_width)
                .saturating_add(BRACKET_CENTER_OFFSET.saturating_sub(2)) as i64,
            AMBIENT_BRACKET_AMP as i64,
            7,
            OUTER_MARGIN.min(logical_width) as i64,
            rail_width as i64,
            right_x as i64,
            rail_width as i64,
        );

        let open_center_width =
            responsive_center_width(logical_width, rail_width, 143, 320, OPEN_BRACKET_AMP);
        let open_center_x = midpoint.saturating_sub(open_center_width / 2);
        let open = Self::keyframe(
            logical_width,
            logical_height,
            open_center_x as i64,
            open_center_width as i64,
            open_center_x.saturating_sub(BRACKET_CENTER_OFFSET) as i64,
            open_center_x
                .saturating_add(open_center_width)
                .saturating_add(BRACKET_CENTER_OFFSET.saturating_sub(2)) as i64,
            OPEN_BRACKET_AMP as i64,
            7,
            OUTER_MARGIN.min(logical_width) as i64,
            rail_width as i64,
            right_x as i64,
            rail_width as i64,
        );

        let t = layout_t.clamp(-Q, Q);
        if t < 0 {
            Self::lerp(closed, ambient, t + Q)
        } else {
            Self::lerp(ambient, open, t)
        }
    }

    fn lerp(a: Self, b: Self, t: i64) -> Self {
        Self {
            logical_width: a.logical_width,
            logical_height: a.logical_height,
            center_x: lerp_q(a.center_x, b.center_x, t),
            center_w: lerp_q(a.center_w, b.center_w, t),
            bracket_left: lerp_q(a.bracket_left, b.bracket_left, t),
            bracket_right: lerp_q(a.bracket_right, b.bracket_right, t),
            bracket_amp: lerp_q(a.bracket_amp, b.bracket_amp, t),
            bracket_hook: lerp_q(a.bracket_hook, b.bracket_hook, t),
            left_x: lerp_q(a.left_x, b.left_x, t),
            left_w: lerp_q(a.left_w, b.left_w, t),
            right_x: lerp_q(a.right_x, b.right_x, t),
            right_w: lerp_q(a.right_w, b.right_w, t),
        }
    }

    fn center_rect(self) -> LogicalRect {
        let x = round_q(self.center_x).min(self.logical_width);
        let width = round_q(self.center_w).min(self.logical_width.saturating_sub(x));
        LogicalRect::new(x, 0, width, self.logical_height)
    }

    fn left_rect(self) -> LogicalRect {
        let x = round_q(self.left_x).min(self.logical_width);
        let width = round_q(self.left_w).min(self.logical_width.saturating_sub(x));
        LogicalRect::new(x, 0, width, self.logical_height)
    }

    fn right_rect(self) -> LogicalRect {
        let x = round_q(self.right_x).min(self.logical_width);
        let width = round_q(self.right_w).min(self.logical_width.saturating_sub(x));
        LogicalRect::new(x, 0, width, self.logical_height)
    }

    fn bracket_top(self) -> usize {
        BRACKET_TOP.min(self.logical_height.saturating_sub(1))
    }

    fn bracket_bottom(self) -> usize {
        self.logical_height
            .saturating_sub(BRACKET_BOTTOM_MARGIN)
            .max(self.bracket_top())
            .min(self.logical_height.saturating_sub(1))
    }

    fn composer_y(self) -> usize {
        self.logical_height.saturating_sub(COMPOSER_BOTTOM_OFFSET)
    }
}

fn responsive_rail_width(logical_width: usize) -> usize {
    let available_for_rails = logical_width
        .saturating_sub(OUTER_MARGIN.saturating_mul(2))
        .saturating_sub(MIN_CENTER_CORRIDOR);
    RAIL_WIDTH.min(available_for_rails / 2)
}

fn responsive_center_width(
    logical_width: usize,
    rail_width: usize,
    ratio_numerator: usize,
    ratio_denominator: usize,
    bracket_amp: usize,
) -> usize {
    let desired = logical_width
        .saturating_mul(ratio_numerator)
        .checked_div(ratio_denominator)
        .unwrap_or(0)
        .clamp(MIN_CENTER_WIDTH, MAX_CENTER_WIDTH);
    let corridor = logical_width
        .saturating_sub(OUTER_MARGIN.saturating_mul(2))
        .saturating_sub(rail_width.saturating_mul(2));
    let bracket_reserve = BRACKET_CENTER_OFFSET
        .saturating_add(bracket_amp)
        .saturating_mul(2);
    desired.min(corridor.saturating_sub(bracket_reserve))
}

fn draw_tabs(surface: &mut FramebufferSurface, state: &DreamState, geometry: DreamGeometry) {
    let tabs = [
        ("Chat", Tab::Chat, HoverTarget::TabChat),
        ("Console", Tab::Console, HoverTarget::TabConsole),
        ("Build", Tab::Build, HoverTarget::TabBuild),
    ];
    let mut x = tabs_start_x(geometry);
    for (index, (label, tab, hover)) in tabs.into_iter().enumerate() {
        let width = chunky_text_width(label);
        let active = state.tab == tab;
        draw_text(
            surface,
            x,
            46,
            label,
            if active {
                TEXT_MAIN
            } else if state.hover == hover {
                TEXT_MUTED
            } else {
                DIM_TEXT
            },
        );
        if active {
            surface.fill_rect(x, 58, width, 2, APP_BLUE);
            surface.fill_rect(
                x.saturating_sub(2),
                60,
                width.saturating_add(4),
                1,
                Color::new(30, 58, 92),
            );
        } else if state.hover == hover {
            surface.fill_rect(x, 58, width, 1, Color::new(46, 92, 148));
        }
        x = x.saturating_add(width).saturating_add(18);
        if index < 2 {
            surface.fill_rect(x.saturating_sub(11), 50, 2, 2, Color::new(51, 58, 69));
        }
    }
}

fn draw_chat(
    surface: &mut FramebufferSurface,
    geometry: DreamGeometry,
    snapshot: &console::ConsoleSnapshot,
    scroll_rows: usize,
) {
    let center = geometry.center_rect();
    let max_chunky = center.w / CHUNKY_ADVANCE;
    let max_hi = center.w.saturating_mul(2).saturating_sub(10) / 9;
    let has_chat = snapshot
        .chat_lines
        .iter()
        .any(|line| !line.text.as_str().is_empty());
    if !has_chat {
        draw_truncated_chunky(
            surface,
            center.x,
            82,
            "Welcome. What should your",
            max_chunky,
            TEXT_MAIN,
        );
        draw_truncated_chunky(
            surface,
            center.x,
            96,
            "raiOS become?",
            max_chunky,
            TEXT_MAIN,
        );
        draw_truncated_hi(
            surface,
            center.x,
            118,
            "Ask for a tool, a workflow, or a change.",
            max_hi,
            TEXT_MUTED,
        );
    } else {
        let conversation = conversation_rect(geometry);
        let visible_rows = conversation.h / CONVERSATION_ROW_HEIGHT;
        let total_rows = conversation_row_count(snapshot, max_hi);
        let scroll_rows = scroll_rows.min(total_rows.saturating_sub(visible_rows));
        let end_row = total_rows.saturating_sub(scroll_rows);
        let start_row = end_row.saturating_sub(visible_rows);
        let mut row = 0usize;
        for line in snapshot.chat_lines {
            let value = line.text.as_str();
            if value.is_empty() {
                continue;
            }
            let (label, label_color) = match line.speaker {
                console::ChatSpeaker::User => ("You", APP_BLUE),
                console::ChatSpeaker::Assistant => ("raiOS", TEXT_MAIN),
                console::ChatSpeaker::System => ("System", TEXT_MUTED),
            };
            if row >= start_row && row < end_row {
                let y = conversation.y + (row - start_row) * CONVERSATION_ROW_HEIGHT;
                surface.fill_rect(center.x, y, 3, 8, label_color);
                draw_text(surface, center.x + 10, y, label, label_color);
            }
            row = row.saturating_add(1);
            let body_color = if line.speaker == console::ChatSpeaker::Assistant {
                TEXT_MAIN
            } else {
                TEXT_MUTED
            };
            visit_wrapped_lines(value, max_hi, |wrapped| {
                if row >= start_row && row < end_row {
                    let y = conversation.y + (row - start_row) * CONVERSATION_ROW_HEIGHT;
                    draw_text_hi(surface, center.x + 10, y + 1, wrapped, body_color);
                }
                row = row.saturating_add(1);
            });
            row = row.saturating_add(1);
        }
    }

    let max_chars = center.w / CHUNKY_ADVANCE;
    let shown_max = max_chars.saturating_sub(3);
    let (shown, _) = trailing_chars(snapshot.chat_input.as_str(), shown_max);
    let composer_y = geometry.composer_y();
    draw_text(surface, center.x, composer_y, ">", APP_BLUE);
    if !shown.is_empty() {
        draw_text(
            surface,
            center.x + 2 * CHUNKY_ADVANCE,
            composer_y,
            shown,
            TEXT_MAIN,
        );
    } else if snapshot.focus != console::UiFocus::ChatInput {
        draw_text(
            surface,
            center.x + 2 * CHUNKY_ADVANCE,
            composer_y,
            "Ask anything...",
            TEXT_FAINT,
        );
    }
    let focused =
        snapshot.view == console::UiView::Ai && snapshot.focus == console::UiFocus::ChatInput;
    surface.fill_rect(
        center.x,
        composer_y.saturating_add(14),
        center.w,
        1,
        if focused {
            Color::new(46, 92, 148)
        } else {
            HAIRLINE
        },
    );
    surface.fill_rect(
        center.x,
        composer_y.saturating_add(15),
        center.w,
        1,
        if focused {
            Color::new(30, 58, 92)
        } else {
            Color::new(23, 27, 33)
        },
    );
}

fn draw_console(
    surface: &mut FramebufferSurface,
    geometry: DreamGeometry,
    snapshot: &console::ConsoleSnapshot,
) {
    let center = geometry.center_rect();
    let max_chars = center.w.saturating_mul(2).saturating_sub(10) / 9;
    let console_top = 74usize.min(geometry.composer_y());
    let conversation = conversation_rect(geometry);
    let capacity = conversation
        .y
        .saturating_add(conversation.h)
        .saturating_sub(console_top)
        / 9;
    let total = snapshot
        .lines
        .iter()
        .filter(|line| !line.as_str().is_empty())
        .count();
    let mut skip = total.saturating_sub(capacity);
    let mut row = 0usize;
    for line in snapshot.lines {
        let value = line.as_str();
        if value.is_empty() {
            continue;
        }
        if skip > 0 {
            skip -= 1;
            continue;
        }
        draw_truncated_hi(
            surface,
            center.x,
            console_top + row * 9,
            value,
            max_chars,
            TEXT_MUTED,
        );
        row += 1;
        if row == capacity {
            break;
        }
    }
    let composer_y = geometry.composer_y();
    draw_text(surface, center.x, composer_y, ">", APP_BLUE);
    draw_truncated_chunky(
        surface,
        center.x + 2 * CHUNKY_ADVANCE,
        composer_y,
        snapshot.input.as_str(),
        center.w.saturating_sub(2 * CHUNKY_ADVANCE) / CHUNKY_ADVANCE,
        TEXT_MAIN,
    );
    surface.fill_rect(
        center.x,
        composer_y.saturating_add(14),
        center.w,
        1,
        Color::new(46, 92, 148),
    );
    surface.fill_rect(
        center.x,
        composer_y.saturating_add(15),
        center.w,
        1,
        Color::new(30, 58, 92),
    );
}

fn draw_build_center(
    surface: &mut FramebufferSurface,
    geometry: DreamGeometry,
    build: &agent_build_loop::Snapshot,
) {
    let center = geometry.center_rect();
    draw_text(surface, center.x, 78, "BUILD", TEXT_MAIN);
    surface.fill_rect(
        center.x,
        88,
        chunky_text_width("BUILD"),
        1,
        Color::new(32, 37, 45),
    );
    draw_text_hi(surface, center.x, 96, build.phase.label(), APP_BLUE);
    let steps = build_steps(build);
    let mut y = 116usize;
    for step in steps {
        draw_checkbox(surface, center.x, y, step.label, step.state);
        y += 13;
    }
    if let Some(reason) = build.last_reason {
        draw_truncated_hi(
            surface,
            center.x,
            y + 8,
            reason,
            center.w.saturating_mul(2) / 9,
            APP_AMBER,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_ambient_rails(
    surface: &mut FramebufferSurface,
    state: &DreamState,
    geometry: DreamGeometry,
    system: &SystemSnapshot,
    projection: context::ContextProjection,
    build: &agent_build_loop::Snapshot,
    recovery_open: bool,
    approve_enabled: bool,
) {
    let left = geometry.left_rect();
    rail_header(surface, left.x, 42, "TRUST");
    draw_diamond(surface, left.x, 58, APP_GREEN);
    draw_text(surface, left.x + 10, 56, "Core safe", TEXT_MAIN);
    draw_text(
        surface,
        left.x + 10,
        70,
        if recovery_open {
            "Recovery open"
        } else if system.network.state == RowState::Ready {
            "Recovery ready"
        } else {
            "Recovery avail"
        },
        TEXT_MUTED,
    );

    rail_header(surface, left.x, 96, "SOURCES");
    draw_source_rows(surface, left.x, 111, 12, projection);
    rail_header(surface, left.x, 164, "PROBLEMS");
    draw_problem(surface, left.x, 178, projection);
    ghost_button(
        surface,
        state,
        left.x,
        204,
        "AI setup",
        HoverTarget::AiSetup,
        GhostMode::Normal,
    );
    ghost_button(
        surface,
        state,
        left.x,
        226,
        "WiFi setup",
        HoverTarget::WifiSetup,
        GhostMode::Normal,
    );
    ghost_button(
        surface,
        state,
        left.x,
        248,
        "Recovery",
        HoverTarget::Recovery,
        GhostMode::Normal,
    );

    let right = geometry.right_rect();
    rail_header(surface, right.x, 42, "PROGRAMS");
    ghost_item(
        surface,
        state,
        right.x,
        56,
        "Calculator",
        HoverTarget::Calculator,
        false,
    );
    ghost_item(
        surface,
        state,
        right.x,
        70,
        "Editor",
        HoverTarget::Editor,
        false,
    );
    rail_header(surface, right.x, 96, "BUILD");
    draw_build_summary(surface, right.x, right.w, 110, build, false);
    ghost_button(
        surface,
        state,
        right.x,
        170,
        "Approve + run",
        HoverTarget::Approve,
        if approve_enabled {
            GhostMode::Primary
        } else {
            GhostMode::Disabled
        },
    );
    rail_header(surface, right.x, 206, "PLAYGROUND");
    ghost_button(
        surface,
        state,
        right.x,
        222,
        "New domain",
        HoverTarget::Playground,
        GhostMode::Normal,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_closed_rails(
    surface: &mut FramebufferSurface,
    state: &DreamState,
    geometry: DreamGeometry,
    system: &SystemSnapshot,
    projection: context::ContextProjection,
    build: &agent_build_loop::Snapshot,
    recovery_open: bool,
    approve_enabled: bool,
) {
    let left = geometry.left_rect();
    let column_gap = 12;
    let left_column_width = left.w.saturating_sub(column_gap) / 2;
    let source_x = left.x + left_column_width + column_gap;
    rail_header(surface, left.x, 42, "TRUST");
    draw_diamond(surface, left.x, 58, APP_GREEN);
    draw_text(surface, left.x + 10, 56, "Core safe", TEXT_MAIN);
    draw_text_hi(
        surface,
        left.x + 10,
        72,
        if recovery_open {
            "Recovery open"
        } else if system.network.state == RowState::Ready {
            "Recovery ready"
        } else {
            "Recovery available"
        },
        TEXT_MUTED,
    );
    rail_header(surface, source_x, 42, "SOURCES");
    draw_source_rows(surface, source_x, 58, 13, projection);
    rail_header(surface, left.x, 108, "PROBLEMS");
    draw_problem(surface, left.x, 124, projection);
    ghost_button(
        surface,
        state,
        left.x,
        146,
        "AI setup",
        HoverTarget::AiSetup,
        GhostMode::Normal,
    );
    ghost_button(
        surface,
        state,
        left.x + BUTTON_WIDTH + 12,
        146,
        "WiFi setup",
        HoverTarget::WifiSetup,
        GhostMode::Normal,
    );
    ghost_button(
        surface,
        state,
        left.x,
        168,
        "Recovery",
        HoverTarget::Recovery,
        GhostMode::Normal,
    );

    let right = geometry.right_rect();
    rail_header(surface, right.x, 42, "BUILD");
    draw_build_summary(surface, right.x, right.w, 58, build, true);
    ghost_button(
        surface,
        state,
        right.x,
        116,
        "Approve + run",
        HoverTarget::Approve,
        if approve_enabled {
            GhostMode::Primary
        } else {
            GhostMode::Disabled
        },
    );
    let steps = build_steps(build);
    let mut y = 144usize;
    for step in steps {
        draw_checkbox(surface, right.x, y, step.label, step.state);
        y += 13;
    }

    let right_column_width = right.w.saturating_sub(column_gap) / 2;
    let playground_x = right.x + right_column_width + column_gap;
    rail_header(surface, right.x, 226, "PROGRAMS");
    ghost_item(
        surface,
        state,
        right.x,
        242,
        "Calculator",
        HoverTarget::Calculator,
        true,
    );
    ghost_item(
        surface,
        state,
        right.x,
        255,
        "Editor",
        HoverTarget::Editor,
        true,
    );
    rail_header(surface, playground_x, 226, "PLAYGROUND");
    ghost_button(
        surface,
        state,
        playground_x,
        242,
        "New domain",
        HoverTarget::Playground,
        GhostMode::Normal,
    );
}

fn draw_source_rows(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    row_step: usize,
    projection: context::ContextProjection,
) {
    draw_text_hi(surface, x, y, "AI", TEXT_MUTED);
    draw_text_hi(
        surface,
        x + 50,
        y,
        projection.ai_connection.value,
        context_tone_color(projection.ai_connection.tone),
    );
    draw_text_hi(surface, x, y + row_step, "Net", TEXT_MUTED);
    draw_text_hi(
        surface,
        x + 50,
        y + row_step,
        projection.network.value,
        context_tone_color(projection.network.tone),
    );
    draw_text_hi(surface, x, y + row_step * 2, "Vault", TEXT_MUTED);
    draw_text_hi(
        surface,
        x + 50,
        y + row_step * 2,
        vault_flow::VaultFlow::status_text(),
        TEXT_MUTED,
    );
}

fn draw_problem(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    projection: context::ContextProjection,
) {
    let critical = projection.problems.critical > 0;
    let color = if critical { APP_RED } else { APP_GREEN };
    draw_diamond(surface, x, y, color);
    draw_text(
        surface,
        x + 10,
        y.saturating_sub(2),
        if critical {
            "Critical present"
        } else if projection.problems.active > 0 {
            "Needs attention"
        } else {
            "None active"
        },
        color,
    );
}

fn draw_build_summary(
    surface: &mut FramebufferSurface,
    x: usize,
    width: usize,
    y: usize,
    build: &agent_build_loop::Snapshot,
    wide: bool,
) {
    let title = if build.latest_revision.is_some() {
        "current project"
    } else {
        "no active project"
    };
    if wide {
        draw_text(surface, x, y, title, TEXT_MAIN);
        draw_text_hi(surface, x, y + 15, "SOURCE", TEXT_MUTED);
        let bar_x = x + 42;
        let bar_width = width.saturating_sub(80).min(200);
        segmented_bar(
            surface,
            bar_x,
            y + 13,
            bar_width,
            build_phase_percent(build.phase),
            APP_BLUE,
            BLUE_LIGHT,
        );
        draw_text_hi(
            surface,
            x,
            y + 29,
            build.phase.label(),
            phase_color(build.phase),
        );
    } else {
        draw_truncated_chunky(surface, x, y, title, width / CHUNKY_ADVANCE, TEXT_MAIN);
        draw_text_hi(surface, x, y + 15, "SOURCE", TEXT_MUTED);
        segmented_bar(
            surface,
            x,
            y + 25,
            width.min(112),
            build_phase_percent(build.phase),
            APP_BLUE,
            BLUE_LIGHT,
        );
        draw_truncated_hi(
            surface,
            x,
            y + 37,
            build.phase.label(),
            width.saturating_mul(2) / 9,
            phase_color(build.phase),
        );
    }
}

#[derive(Clone, Copy)]
enum StepState {
    Done,
    Active,
    Waiting,
    Unwired,
}

#[derive(Clone, Copy)]
struct BuildStep {
    label: &'static str,
    state: StepState,
}

fn build_steps(build: &agent_build_loop::Snapshot) -> [BuildStep; 5] {
    use agent_build_loop::LoopPhase;
    let request_done = !matches!(build.phase, LoopPhase::Idle | LoopPhase::Rejected);
    let source_done = matches!(
        build.phase,
        LoopPhase::SourceReady | LoopPhase::RevisionNeeded | LoopPhase::VerifiedSource
    );
    let verified = build.phase == LoopPhase::VerifiedSource;
    let active_index = if !request_done {
        0
    } else if !source_done {
        1
    } else if !verified {
        2
    } else {
        3
    };
    let state_for = |index: usize, done: bool| {
        if done {
            StepState::Done
        } else if index == active_index {
            StepState::Active
        } else {
            StepState::Waiting
        }
    };
    [
        BuildStep {
            label: "Request queued",
            state: state_for(0, request_done),
        },
        BuildStep {
            label: "Source received",
            state: state_for(1, source_done),
        },
        BuildStep {
            label: "Source verified",
            state: state_for(2, verified),
        },
        BuildStep {
            label: "Build A not wired",
            state: StepState::Unwired,
        },
        BuildStep {
            label: "Build B not wired",
            state: StepState::Unwired,
        },
    ]
}

fn draw_checkbox(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    label: &str,
    state: StepState,
) {
    let color = match state {
        StepState::Done => APP_GREEN,
        StepState::Active => APP_AMBER,
        StepState::Waiting => Color::new(51, 58, 69),
        StepState::Unwired => DISABLED_TEXT,
    };
    draw_outline(surface, LogicalRect::new(x, y, 7, 7), color);
    if matches!(state, StepState::Done) {
        surface.fill_rect(x + 2, y + 3, 1, 1, APP_GREEN);
        surface.fill_rect(x + 3, y + 4, 1, 1, APP_GREEN);
        surface.fill_rect(x + 4, y + 2, 1, 1, APP_GREEN);
    }
    draw_text_hi(
        surface,
        x + 13,
        y + 1,
        label,
        match state {
            StepState::Done => TEXT_MUTED,
            StepState::Active => TEXT_MAIN,
            StepState::Waiting => DIM_TEXT,
            StepState::Unwired => DISABLED_TEXT,
        },
    );
}

fn rail_header(surface: &mut FramebufferSurface, x: usize, y: usize, label: &str) {
    draw_text(surface, x, y, label, DIM_TEXT);
    surface.fill_rect(
        x,
        y + 10,
        chunky_text_width(label),
        1,
        Color::new(32, 37, 45),
    );
}

fn draw_diamond(surface: &mut FramebufferSurface, x: usize, y: usize, color: Color) {
    surface.fill_rect(x + 2, y, 1, 1, color);
    surface.fill_rect(x + 1, y + 1, 3, 1, color);
    surface.fill_rect(x, y + 2, 5, 1, color);
    surface.fill_rect(x + 1, y + 3, 3, 1, color);
    surface.fill_rect(x + 2, y + 4, 1, 1, color);
}

#[derive(Clone, Copy)]
enum GhostMode {
    Normal,
    Primary,
    Disabled,
}

fn ghost_button(
    surface: &mut FramebufferSurface,
    state: &DreamState,
    x: usize,
    y: usize,
    label: &str,
    target: HoverTarget,
    mode: GhostMode,
) {
    let hovered = state.hover == target && !matches!(mode, GhostMode::Disabled);
    if hovered {
        surface.fill_rect(
            x,
            y,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
            if matches!(mode, GhostMode::Primary) {
                Color::new(20, 39, 61)
            } else {
                Color::new(18, 22, 28)
            },
        );
    }
    let text_color = match mode {
        GhostMode::Disabled => DISABLED_TEXT,
        GhostMode::Primary if !hovered => BLUE_LIGHT,
        _ if hovered => TEXT_MAIN,
        _ => TEXT_MUTED,
    };
    let text_width = hi_text_width(label);
    let x_physical = x * 2 + (BUTTON_WIDTH * 2).saturating_sub(text_width) / 2;
    draw_text_hi_physical(surface, x_physical, (y + 4) * 2, label, text_color);
    let baseline = match mode {
        GhostMode::Disabled => Color::new(26, 31, 38),
        GhostMode::Primary => Color::new(46, 92, 148),
        GhostMode::Normal => Color::new(32, 37, 45),
    };
    surface.fill_rect(x, y + 15, BUTTON_WIDTH, 1, baseline);
    if hovered {
        surface.fill_rect(
            x,
            y + 15,
            BUTTON_WIDTH,
            if matches!(mode, GhostMode::Primary) {
                2
            } else {
                1
            },
            APP_BLUE,
        );
    }
}

fn ghost_item(
    surface: &mut FramebufferSurface,
    state: &DreamState,
    x: usize,
    y: usize,
    label: &str,
    target: HoverTarget,
    hi: bool,
) {
    let hovered = state.hover == target;
    if hi {
        draw_text_hi(
            surface,
            x,
            y,
            ">",
            if hovered { APP_BLUE } else { DISABLED_TEXT },
        );
        draw_text_hi(surface, x + 12, y, label, TEXT_MAIN);
        if hovered {
            surface.fill_rect(x + 12, y + 9, logical_hi_text_width(label), 1, APP_BLUE);
        }
    } else {
        draw_text(
            surface,
            x,
            y,
            ">",
            if hovered { APP_BLUE } else { DISABLED_TEXT },
        );
        draw_text(surface, x + 12, y, label, TEXT_MAIN);
        if hovered {
            surface.fill_rect(x + 12, y + 10, chunky_text_width(label), 1, APP_BLUE);
        }
    }
}

fn segmented_bar(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    percent: usize,
    fill: Color,
    light: Color,
) {
    if width < 4 {
        return;
    }
    surface.fill_rect(x, y, width, 8, Color::new(13, 15, 19));
    surface.fill_rect(
        x + 1,
        y + 1,
        width.saturating_sub(2),
        1,
        Color::new(7, 9, 11),
    );
    draw_outline(
        surface,
        LogicalRect::new(x, y, width, 8),
        Color::new(30, 35, 43),
    );
    let filled = (12 * percent.min(100) + 50) / 100;
    let slot = (width.saturating_sub(4) / 12).max(2);
    for index in 0..filled {
        let segment_x = x + 2 + index * slot;
        let segment_width = slot.saturating_sub(1).max(1);
        surface.fill_rect(segment_x, y + 2, segment_width, 4, fill);
        surface.fill_rect(segment_x, y + 2, segment_width, 1, light);
    }
}

fn draw_bracket(
    surface: &mut FramebufferSurface,
    xc: i64,
    dir: i64,
    amp: i64,
    hook: i64,
    top: usize,
    bottom: usize,
) {
    if bottom <= top {
        return;
    }
    let xc_f = xc as f64 / Q as f64;
    let amp_f = amp as f64 / Q as f64;
    for y in top..=bottom {
        let t = (y - top) as f64 / (bottom - top) as f64;
        let sine = sin_approx(core::f64::consts::PI * t);
        let x = xc_f + dir as f64 * amp_f * sine;
        let width = if sine > 0.85 { 3 } else { 2 };
        let core = if sine > 0.92 {
            BRACKET_HIGH
        } else if sine > 0.55 {
            BRACKET_MID
        } else {
            BRACKET_LOW
        };
        let physical_x = round_f64(x * 2.0);
        draw_physical_rect(
            surface,
            physical_x.saturating_sub(2),
            y * 2,
            width * 2 + 4,
            2,
            BRACKET_GLOW,
        );
        draw_physical_rect(surface, physical_x, y * 2, width * 2, 2, core);
    }
    let xc = round_q(xc);
    let hook = round_q(hook);
    let hook_x = xc.saturating_sub(if dir > 0 { hook.saturating_sub(1) } else { 0 });
    surface.fill_rect(hook_x, top, hook, 1, BRACKET_LOW);
    surface.fill_rect(hook_x, bottom, hook, 1, BRACKET_LOW);
}

fn draw_physical_rect(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
) {
    for py in y..y.saturating_add(height) {
        for px in x..x.saturating_add(width) {
            surface.set_physical_pixel(px, py, color);
        }
    }
}

struct DreamBackground {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    /// In die native Byte-Reihenfolge des Framebuffers vorkodierte Zeilen —
    /// einmal beim ersten Frame gebaut, danach kopiert `blit_physical_rows`
    /// den Hintergrund als Speicherbloecke statt pro Pixel.
    encoded: Vec<u8>,
}

fn copy_background(surface: &mut FramebufferSurface, background: &mut DreamBackground) {
    surface.set_draw_scale(1);
    let expected = background
        .width
        .saturating_mul(background.height)
        .saturating_mul(4);
    if background.encoded.len() != expected {
        let mut encoded = Vec::with_capacity(expected);
        for packed in background.pixels.iter().copied() {
            let bytes = surface.encode_color(color_from_rgb(packed));
            encoded.extend_from_slice(&bytes);
        }
        background.encoded = encoded;
    }
    surface.blit_physical_rows(&background.encoded);
}

fn build_dream_background(physical_width: usize, physical_height: usize) -> DreamBackground {
    let pixel_count = physical_width.checked_mul(physical_height).unwrap_or(0);
    let mut background = DreamBackground {
        width: physical_width,
        height: physical_height,
        pixels: vec![rgb(8, 9, 11); pixel_count],
        encoded: Vec::new(),
    };
    let shades = [rgb(8, 9, 11), rgb(11, 13, 16), rgb(14, 16, 20)];
    let bayer = [
        [0usize, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5],
    ];
    let logical_width = physical_width / DRAW_SCALE;
    let logical_height = physical_height / DRAW_SCALE;
    let vertical_center = logical_height / 2 + logical_height / 40;
    let vertical_span = logical_height
        .saturating_mul(3)
        .checked_div(5)
        .unwrap_or(0)
        .max(1);
    let horizontal_center = logical_width / 2;
    let horizontal_span = logical_width
        .saturating_mul(21)
        .checked_div(32)
        .unwrap_or(0)
        .max(1);
    for y in 0..logical_height {
        let vertical = 1.0 - abs_f64(y as f64 - vertical_center as f64) / vertical_span as f64;
        for x in 0..logical_width {
            let center =
                1.0 - abs_f64(x as f64 - horizontal_center as f64) / horizontal_span as f64;
            let value =
                clamp_f64(vertical * 0.6 + center * 0.55, 0.0, 1.0) * (shades.len() - 1) as f64;
            let mut index = floor_f64(value) as usize;
            if bayer[y % 4][x % 4] as f64 / 16.0 < value - index as f64 {
                index += 1;
            }
            fill_background_rect(
                &mut background,
                x * 2,
                y * 2,
                2,
                2,
                shades[index.min(shades.len() - 1)],
            );
        }
    }

    let mut seed = 1337u32;
    const REFERENCE_BACKGROUND_AREA: u128 = 1280 * 800;
    let physical_area = (physical_width as u128).saturating_mul(physical_height as u128);
    let star_count = (physical_area
        .saturating_mul(70)
        .saturating_add(REFERENCE_BACKGROUND_AREA / 2)
        / REFERENCE_BACKGROUND_AREA)
        .min(usize::MAX as u128) as usize;
    for _ in 0..star_count {
        let x = floor_f64(lcg_random(&mut seed) * logical_width as f64) as usize;
        let y = floor_f64(lcg_random(&mut seed) * logical_height as f64) as usize;
        let bright = lcg_random(&mut seed) > 0.82;
        fill_background_rect(
            &mut background,
            x * 2,
            y * 2,
            2,
            2,
            if bright {
                rgb(35, 40, 51)
            } else {
                rgb(22, 26, 32)
            },
        );
    }
    draw_candle_into(&mut background);
    background
}

// Port of drawCandleInto from docs/design-lab/raios-ui-lab.html. Its physical
// dimensions stay fixed; only the bottom anchor follows the framebuffer height.
fn draw_candle_into(background: &mut DreamBackground) {
    const CX: f64 = 220.0;
    const REFERENCE_BASE: i32 = 786;
    let base = background.height.saturating_sub(14).min(i32::MAX as usize) as i32;
    let plate_top = base - 24;
    let stick_top = plate_top - 58;
    let body_top = stick_top - 108;
    let vertical_offset = base - REFERENCE_BASE;

    for y in plate_top..=base {
        let t = (y - plate_top) as f64 / (base - plate_top) as f64;
        let half_width = 66.0 * sin_approx(core::f64::consts::PI * (t * 1.04).min(1.0));
        candle_span(background, CX - half_width, CX + half_width, y as f64);
    }
    for (dx, width, height) in [
        (-34.0, 16.0, 5),
        (-12.0, 12.0, 4),
        (22.0, 18.0, 6),
        (44.0, 10.0, 4),
    ] {
        for index in 0..height {
            let ratio = index as f64 / height as f64;
            let half_width = (width / 2.0) * sqrt_approx(1.0 - ratio * ratio);
            candle_span(
                background,
                CX + dx - half_width,
                CX + dx + half_width,
                (plate_top - index) as f64,
            );
        }
    }

    for y in stick_top..=plate_top {
        let t = (y - stick_top) as f64 / (plate_top - stick_top) as f64;
        let mut half_width = 9.0
            + 20.0 * pow_approx(((t - 0.72).max(0.0)) / 0.28, 1.6)
            + 7.0 * exp_approx(-powi((t - 0.45) / 0.13, 2))
            + 11.0 * pow_approx(((0.16 - t).max(0.0)) / 0.16, 1.3);
        if t < 0.06 {
            half_width = 21.0;
        }
        candle_span(background, CX - half_width, CX + half_width, y as f64);
    }

    for y in body_top..=stick_top {
        let reference_y = y - vertical_offset;
        let mut half_width = 14.0
            + 2.6 * sin_approx(reference_y as f64 * 0.05)
            + 1.6 * sin_approx(reference_y as f64 * 0.12 + 1.7)
            + 3.5
                * exp_approx(-powi(
                    (reference_y - (REFERENCE_BASE - 190 + 46)) as f64 / 18.0,
                    2,
                ));
        let top = y - body_top;
        if top < 12 {
            half_width += 7.0
                * sin_approx(core::f64::consts::PI * top as f64 / 12.0)
                * (1.0 + 0.35 * sin_approx(reference_y as f64 * 0.8));
        }
        candle_span(background, CX - half_width, CX + half_width, y as f64);
    }

    for (dx, width, length) in [
        (-20.0, 5.0, 30),
        (-24.0, 4.0, 16),
        (19.0, 6.0, 72),
        (23.0, 5.0, 38),
    ] {
        for index in 0..length {
            let t = index as f64 / length as f64;
            let mut half_width = (width / 2.0) * (0.75 + 0.25 * sin_approx(index as f64 * 0.25));
            if t > 0.82 {
                half_width =
                    (width / 2.0) * sqrt_approx((1.0 - powi((t - 0.82) / 0.18, 2)).max(0.0)) * 1.15;
            }
            candle_span(
                background,
                CX + dx - half_width,
                CX + dx + half_width,
                (body_top + 6 + index) as f64,
            );
        }
    }
    for index in 0..48 {
        let t = index as f64 / 48.0;
        let mut half_width = 3.4 * (0.8 + 0.2 * sin_approx(index as f64 * 0.3));
        if t > 0.8 {
            half_width = 4.2 * sqrt_approx((1.0 - powi((t - 0.8) / 0.2, 2)).max(0.0));
        }
        candle_span(
            background,
            CX - 19.0 - half_width,
            CX - 19.0 + half_width,
            (stick_top + 2 + index) as f64,
        );
    }

    for index in 0..9 {
        let offset = round_f64(index as f64 * 0.22) as f64;
        candle_span(
            background,
            CX + offset - 1.0,
            CX + offset + 1.0,
            (body_top - index) as f64,
        );
    }

    let flame_top = body_top - 68;
    for y in flame_top..=body_top - 12 {
        let t = (y - flame_top) as f64 / (body_top - 12 - flame_top) as f64;
        let half_width = 7.2
            * sin_approx(core::f64::consts::PI * pow_approx(t, 0.55))
            * (1.0 - 0.1 * sin_approx(3.0 * t));
        let lean = 3.5 * sin_approx(core::f64::consts::PI * t) * (t - 0.45);
        candle_span(
            background,
            CX + lean - half_width,
            CX + lean + half_width,
            y as f64,
        );
    }

    let halo_center_y = flame_top as f64 + 34.0;
    let mut angle = 0.0;
    while angle < 360.0 {
        let radians = angle * core::f64::consts::PI / 180.0;
        for radius in [44.0, 43.0] {
            set_background_pixel(
                background,
                round_f64(CX + radius * cos_approx(radians)),
                round_f64(halo_center_y + radius * sin_approx(radians)),
                CANDLE_TONE,
            );
        }
        angle += 0.6;
    }

    let handle_x = CX + 74.0;
    let handle_y = (plate_top - 28) as f64;
    angle = 0.0;
    while angle < 360.0 {
        let radians = angle * core::f64::consts::PI / 180.0;
        for radius in 22..=26 {
            set_background_pixel(
                background,
                round_f64(handle_x + radius as f64 * cos_approx(radians)),
                round_f64(handle_y + radius as f64 * sin_approx(radians)),
                CANDLE_TONE,
            );
        }
        angle += 0.5;
    }
    let start_x = CX as i32 + 18;
    let end_x = handle_x as i32 - 26 + 4;
    for x in start_x..=end_x {
        let t = (x - start_x) as f64 / (end_x - start_x) as f64;
        let y = (plate_top - 38) as f64
            + 10.0 * sin_approx(core::f64::consts::PI * t - core::f64::consts::PI / 2.0);
        for offset in 0..4 {
            candle_span(background, x as f64, (x + 1) as f64, y + offset as f64);
        }
    }
}

fn candle_span(background: &mut DreamBackground, x0: f64, x1: f64, y: f64) {
    if x1 <= x0 {
        return;
    }
    let x = round_f64(x0);
    let width = round_f64(x1 - x0).max(1);
    let y = round_f64(y);
    fill_background_rect(background, x, y, width, 1, CANDLE_TONE);
}

fn fill_background_rect(
    background: &mut DreamBackground,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u32,
) {
    let max_x = x.saturating_add(width).min(background.width);
    let max_y = y.saturating_add(height).min(background.height);
    for py in y..max_y {
        for px in x..max_x {
            if let Some(pixel) = background.pixels.get_mut(py * background.width + px) {
                *pixel = color;
            }
        }
    }
}

fn set_background_pixel(background: &mut DreamBackground, x: usize, y: usize, color: u32) {
    if x < background.width && y < background.height {
        if let Some(pixel) = background.pixels.get_mut(y * background.width + x) {
            *pixel = color;
        }
    }
}

fn lcg_random(seed: &mut u32) -> f64 {
    // Match JavaScript Number multiplication and the following bitwise mask
    // from the executable spec, including its IEEE-754 rounding.
    let next = (*seed as f64 * 1_103_515_245.0 + 12_345.0) as u64;
    *seed = (next & 0x7fff_ffff) as u32;
    *seed as f64 / 0x7fff_ffffu32 as f64
}

fn draw_text(surface: &mut FramebufferSurface, x: usize, y: usize, value: &str, color: Color) {
    text::draw_text(surface, x, y, value, color, None);
}

fn draw_text_hi(surface: &mut FramebufferSurface, x: usize, y: usize, value: &str, color: Color) {
    draw_text_hi_physical(surface, x * 2, y * 2, value, color);
}

fn draw_text_hi_physical(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    value: &str,
    color: Color,
) {
    let previous_scale = surface.draw_scale();
    surface.set_draw_scale(1);
    text::draw_text(surface, x, y, value, color, None);
    surface.set_draw_scale(previous_scale);
}

fn draw_truncated_chunky(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    value: &str,
    max_chars: usize,
    color: Color,
) {
    draw_text(surface, x, y, prefix_chars(value, max_chars), color);
}

fn draw_truncated_hi(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    value: &str,
    max_chars: usize,
    color: Color,
) {
    draw_text_hi(surface, x, y, prefix_chars(value, max_chars), color);
}

fn prefix_chars(value: &str, max_chars: usize) -> &str {
    value
        .char_indices()
        .nth(max_chars)
        .map(|(index, _)| &value[..index])
        .unwrap_or(value)
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

fn context_tone_color(tone: context::ContextTone) -> Color {
    match tone {
        context::ContextTone::Neutral => TEXT_MUTED,
        context::ContextTone::Good => APP_GREEN,
        context::ContextTone::Attention => APP_AMBER,
        context::ContextTone::Critical => APP_RED,
    }
}

fn phase_color(phase: agent_build_loop::LoopPhase) -> Color {
    use agent_build_loop::LoopPhase;
    match phase {
        LoopPhase::VerifiedSource => APP_GREEN,
        LoopPhase::Rejected => APP_RED,
        LoopPhase::RevisionNeeded => APP_AMBER,
        LoopPhase::Idle => TEXT_FAINT,
        LoopPhase::RequestingSource | LoopPhase::SourceReady => APP_BLUE,
    }
}

fn build_phase_percent(phase: agent_build_loop::LoopPhase) -> usize {
    use agent_build_loop::LoopPhase;
    match phase {
        LoopPhase::Idle | LoopPhase::Rejected => 0,
        LoopPhase::RequestingSource => 25,
        LoopPhase::SourceReady => 65,
        LoopPhase::RevisionNeeded => 75,
        LoopPhase::VerifiedSource => 100,
    }
}

fn composer_rect(geometry: DreamGeometry) -> LogicalRect {
    let center = geometry.center_rect();
    let y = geometry.composer_y().saturating_sub(10);
    let bottom = geometry
        .composer_y()
        .saturating_add(18)
        .min(geometry.logical_height);
    LogicalRect::new(center.x, y, center.w, bottom.saturating_sub(y))
}

fn conversation_rect(geometry: DreamGeometry) -> LogicalRect {
    let center = geometry.center_rect();
    let bottom = geometry
        .composer_y()
        .saturating_sub(CONVERSATION_COMPOSER_GAP);
    let y = CONVERSATION_TOP.min(bottom);
    LogicalRect::new(center.x, y, center.w, bottom.saturating_sub(y))
}

fn button_rect(x: usize, y: usize) -> LogicalRect {
    LogicalRect::new(x, y, BUTTON_WIDTH, BUTTON_HEIGHT)
}

fn item_rect(x: usize, y: usize, width: usize) -> LogicalRect {
    LogicalRect::new(x, y.saturating_sub(3), width, 14)
}

fn tabs_start_x(geometry: DreamGeometry) -> usize {
    let labels_width = chunky_text_width("Chat")
        .saturating_add(chunky_text_width("Console"))
        .saturating_add(chunky_text_width("Build"));
    let total_width = labels_width.saturating_add(36);
    let center = geometry.center_rect();
    center
        .x
        .saturating_add(center.w.saturating_sub(total_width) / 2)
}

fn tab_rects(geometry: DreamGeometry) -> [(LogicalRect, HitTarget); 3] {
    let chat_x = tabs_start_x(geometry);
    let console_x = chat_x
        .saturating_add(chunky_text_width("Chat"))
        .saturating_add(18);
    let build_x = console_x
        .saturating_add(chunky_text_width("Console"))
        .saturating_add(18);
    [
        (
            LogicalRect::new(
                chat_x.saturating_sub(4),
                42,
                chunky_text_width("Chat").saturating_add(8),
                20,
            ),
            HitTarget::TabChat,
        ),
        (
            LogicalRect::new(
                console_x.saturating_sub(4),
                42,
                chunky_text_width("Console").saturating_add(8),
                20,
            ),
            HitTarget::TabConsole,
        ),
        (
            LogicalRect::new(
                build_x.saturating_sub(4),
                42,
                chunky_text_width("Build").saturating_add(8),
                20,
            ),
            HitTarget::TabBuild,
        ),
    ]
}

fn tab_hover_rects(geometry: DreamGeometry) -> [(LogicalRect, HoverTarget); 3] {
    let [chat, console, build] = tab_rects(geometry);
    [
        (chat.0, HoverTarget::TabChat),
        (console.0, HoverTarget::TabConsole),
        (build.0, HoverTarget::TabBuild),
    ]
}

fn point_in(x: usize, y: usize, rect: LogicalRect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.w)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.h)
}

fn chunky_text_width(value: &str) -> usize {
    value.chars().count().saturating_mul(CHUNKY_ADVANCE)
}

fn hi_text_width(value: &str) -> usize {
    value.chars().count().saturating_mul(9)
}

fn logical_hi_text_width(value: &str) -> usize {
    hi_text_width(value).saturating_add(1) / 2
}

fn ease_in_out_q(t: i64) -> i64 {
    let t = t.clamp(0, Q) as i128;
    let q = Q as i128;
    if t < q / 2 {
        ((4 * t * t * t) / (q * q)) as i64
    } else {
        let inverse = 2 * q - 2 * t;
        (q - inverse * inverse * inverse / (2 * q * q)) as i64
    }
}

fn lerp_q(a: i64, b: i64, t: i64) -> i64 {
    a + (((b - a) as i128 * t as i128) / Q as i128) as i64
}

fn round_q(value: i64) -> usize {
    ((value + Q / 2).max(0) / Q) as usize
}

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

fn color_from_rgb(value: u32) -> Color {
    match value {
        value if value == rgb(8, 9, 11) => SHADE_DARK,
        value if value == rgb(11, 13, 16) => SHADE_MID,
        value if value == rgb(14, 16, 20) => SHADE_LIGHT,
        _ => Color::new(
            ((value >> 16) & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            (value & 0xff) as u8,
        ),
    }
}

fn abs_f64(value: f64) -> f64 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}

fn clamp_f64(value: f64, minimum: f64, maximum: f64) -> f64 {
    if value < minimum {
        minimum
    } else if value > maximum {
        maximum
    } else {
        value
    }
}

fn floor_f64(value: f64) -> i64 {
    let integer = value as i64;
    if value < integer as f64 {
        integer - 1
    } else {
        integer
    }
}

fn round_f64(value: f64) -> usize {
    if value <= 0.0 {
        0
    } else {
        floor_f64(value + 0.5) as usize
    }
}

fn powi(value: f64, exponent: u32) -> f64 {
    let mut result = 1.0;
    for _ in 0..exponent {
        result *= value;
    }
    result
}

fn sqrt_approx(value: f64) -> f64 {
    if value <= 0.0 {
        return 0.0;
    }
    let mut result = if value > 1.0 { value } else { 1.0 };
    for _ in 0..12 {
        result = 0.5 * (result + value / result);
    }
    result
}

fn sin_approx(value: f64) -> f64 {
    let pi = core::f64::consts::PI;
    let tau = 2.0 * pi;
    let turns = floor_f64((value + pi) / tau);
    let x = value - turns as f64 * tau;
    let x2 = x * x;
    x * (1.0
        + x2 * (-1.0 / 6.0
            + x2 * (1.0 / 120.0
                + x2 * (-1.0 / 5040.0
                    + x2 * (1.0 / 362880.0
                        + x2 * (-1.0 / 39916800.0 + x2 * (1.0 / 6227020800.0)))))))
}

fn cos_approx(value: f64) -> f64 {
    sin_approx(value + core::f64::consts::PI / 2.0)
}

fn exp_approx(value: f64) -> f64 {
    if value < -50.0 {
        return 0.0;
    }
    let mut reduced = value;
    let mut squarings = 0usize;
    while abs_f64(reduced) > 0.5 {
        reduced *= 0.5;
        squarings += 1;
    }
    let mut term = 1.0;
    let mut sum = 1.0;
    for index in 1..=18 {
        term *= reduced / index as f64;
        sum += term;
    }
    for _ in 0..squarings {
        sum *= sum;
    }
    sum
}

fn ln_approx(value: f64) -> f64 {
    if value <= 0.0 {
        return -1000.0;
    }
    let mut mantissa = value;
    let mut exponent = 0i32;
    while mantissa < 1.0 {
        mantissa *= 2.0;
        exponent -= 1;
    }
    while mantissa >= 2.0 {
        mantissa *= 0.5;
        exponent += 1;
    }
    let z = (mantissa - 1.0) / (mantissa + 1.0);
    let z2 = z * z;
    let mut term = z;
    let mut sum = 0.0;
    for odd in (1..=19).step_by(2) {
        sum += term / odd as f64;
        term *= z2;
    }
    2.0 * sum + exponent as f64 * core::f64::consts::LN_2
}

fn pow_approx(value: f64, exponent: f64) -> f64 {
    if value <= 0.0 {
        0.0
    } else {
        exp_approx(exponent * ln_approx(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rects_overlap(a: LogicalRect, b: LogicalRect) -> bool {
        a.x < b.x.saturating_add(b.w)
            && b.x < a.x.saturating_add(a.w)
            && a.y < b.y.saturating_add(b.h)
            && b.y < a.y.saturating_add(a.h)
    }

    fn assert_layout_invariants(physical_width: usize, physical_height: usize) {
        let logical_width = physical_width / DRAW_SCALE;
        let logical_height = physical_height / DRAW_SCALE;
        for layout_t in [-Q, 0, Q] {
            let geometry = DreamGeometry::at(physical_width, physical_height, layout_t);
            let left = geometry.left_rect();
            let center = geometry.center_rect();
            let right = geometry.right_rect();

            assert_eq!(geometry.logical_width, logical_width);
            assert_eq!(geometry.logical_height, logical_height);
            assert!(left.x.saturating_add(left.w) <= logical_width);
            assert!(center.x.saturating_add(center.w) <= logical_width);
            assert!(right.x.saturating_add(right.w) <= logical_width);
            assert!(!rects_overlap(left, center));
            assert!(!rects_overlap(center, right));
            assert!(!rects_overlap(left, right));

            let left_bracket_min = round_q(geometry.bracket_left - geometry.bracket_amp);
            let right_bracket_max = round_q(geometry.bracket_right + geometry.bracket_amp);
            assert!(left.x.saturating_add(left.w) <= left_bracket_min);
            assert!(right_bracket_max <= right.x);
            if layout_t >= 0 {
                assert!(round_q(geometry.bracket_left) < center.x);
                assert!(center.x.saturating_add(center.w) <= round_q(geometry.bracket_right));
            }

            assert_eq!(geometry.bracket_top(), BRACKET_TOP);
            assert_eq!(
                geometry.bracket_bottom(),
                logical_height - BRACKET_BOTTOM_MARGIN
            );
            assert_eq!(
                geometry.composer_y(),
                logical_height - COMPOSER_BOTTOM_OFFSET
            );
        }

        let closed = DreamGeometry::at(physical_width, physical_height, -Q);
        let center = closed.center_rect();
        assert_eq!(center.x.saturating_add(center.w / 2), logical_width / 2);
    }

    #[test]
    fn layout_keyframes_match_the_executable_spec() {
        let closed = DreamGeometry::at(1280, 800, -Q);
        assert_eq!(closed.center_rect().x, 316);
        assert_eq!(closed.center_rect().w, 8);
        assert_eq!(round_q(closed.bracket_left), 319);
        assert_eq!(round_q(closed.bracket_right), 321);

        let ambient = DreamGeometry::at(1280, 800, 0);
        assert_eq!(ambient.center_rect().x, 185);
        assert_eq!(ambient.center_rect().w, 270);

        let open = DreamGeometry::at(1280, 800, Q);
        assert_eq!(open.center_rect().x, 177);
        assert_eq!(open.center_rect().w, 286);
        assert_eq!(round_q(open.bracket_left), 160);
        assert_eq!(round_q(open.bracket_right), 478);
    }

    #[test]
    fn layout_invariants_hold_at_1280_by_800() {
        assert_layout_invariants(1280, 800);
    }

    #[test]
    fn layout_invariants_hold_at_1920_by_1080() {
        assert_layout_invariants(1920, 1080);
    }

    #[test]
    fn undersized_viewport_clamps_to_empty_safe_regions() {
        let geometry = DreamGeometry::at(160, 80, Q);
        let center = geometry.center_rect();
        let conversation = conversation_rect(geometry);
        let composer = composer_rect(geometry);
        assert!(center.x.saturating_add(center.w) <= geometry.logical_width);
        assert!(conversation.y.saturating_add(conversation.h) <= geometry.logical_height);
        assert!(composer.y.saturating_add(composer.h) <= geometry.logical_height);
        assert_eq!(geometry.bracket_bottom(), geometry.bracket_top());
    }

    #[test]
    fn cubic_ease_has_fixed_endpoints_and_midpoint() {
        assert_eq!(ease_in_out_q(0), 0);
        assert_eq!(ease_in_out_q(Q / 2), Q / 2);
        assert_eq!(ease_in_out_q(Q), Q);
    }

    #[test]
    fn star_lcg_matches_the_javascript_number_sequence() {
        let mut seed = 1337;
        let expected = [78_628_734, 1_460_962_528, 2_037_973_760, 1_768_978_176];
        for value in expected {
            let _ = lcg_random(&mut seed);
            assert_eq!(seed, value);
        }
    }

    #[test]
    fn center_zone_opens_and_leaving_it_closes() {
        let mut state = DreamState::new();
        state.set_surface_size(1280, 800);
        assert!(state.update_pointer(true, 320, 120, true, false));
        assert_eq!(state.requested_goal, Q);
        assert!(state.update_pointer(true, 40, 120, true, false));
        assert_eq!(state.requested_goal, -Q);
    }

    #[test]
    fn transition_reaches_ambient_at_250ms_and_open_at_500ms() {
        let mut state = DreamState::new();
        state.set_surface_size(1280, 800);
        state.update_pointer(true, 320, 120, true, false);
        state.update_for_frame(100);
        state.update_for_frame(350);
        assert_eq!(state.layout_t, 0);
        state.update_for_frame(600);
        assert_eq!(state.layout_t, Q);
    }

    #[test]
    fn invented_programs_stay_inert_and_disabled_approve_cannot_escalate() {
        let mut state = DreamState::new();
        state.set_surface_size(1280, 800);
        state.layout_t = Q;
        assert_eq!(state.hit_test(500, 56, false), Some(HitTarget::Inert));
        assert_eq!(state.hit_test(500, 222, false), Some(HitTarget::Inert));
        assert_eq!(state.hit_test(500, 170, false), Some(HitTarget::Inert));
        assert_eq!(state.hit_test(500, 170, true), Some(HitTarget::Approve));
    }
}
