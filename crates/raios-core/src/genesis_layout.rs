//! Pure responsive geometry and hit testing for the trusted Genesis shell.

pub const LOGICAL_SCALE: u32 = 2;
pub const OUTER_MARGIN: u32 = 16;
pub const PANEL_GAP: u32 = 12;
pub const SECURE_STRIP_HEIGHT: u32 = 38;
pub const COMPOSER_HEIGHT: u32 = 48;

const MIN_LOGICAL_WIDTH: u32 = 480;
const MIN_LOGICAL_HEIGHT: u32 = 360;
const MIN_CONVERSATION_WIDTH: u32 = 300;
const MIN_CONTEXT_WIDTH: u32 = 180;
const MIN_CONVERSATION_HEIGHT: u32 = 128;
const MIN_CONTEXT_HEIGHT: u32 = 96;
const MAX_OVERLAY_WIDTH: u32 = 480;
const MAX_OVERLAY_HEIGHT: u32 = 280;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn right(self) -> u32 {
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(self) -> u32 {
        self.y.saturating_add(self.height)
    }

    pub const fn contains(self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() && point.y >= self.y && point.y < self.bottom()
    }

    pub const fn overlaps(self, other: Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelFlow {
    SideBySide,
    Stacked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitLayer {
    Genesis,
    Personal,
    TrustedOverlay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitTarget {
    SecureStrip,
    Conversation,
    Context,
    Composer,
    PersonalSurface,
    TrustedOverlay,
    OverlayScrim,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutError {
    ViewportTooSmall,
    GeometryOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenesisLayout {
    pub physical_size: Size,
    pub logical_size: Size,
    pub panel_flow: PanelFlow,
    pub secure_strip: Rect,
    pub conversation: Rect,
    pub context: Rect,
    pub composer: Rect,
    pub personal_surface: Rect,
    pub trusted_overlay: Rect,
}

impl GenesisLayout {
    pub fn new(physical_size: Size) -> Result<Self, LayoutError> {
        let logical_size = Size::new(
            physical_size
                .width
                .checked_div(LOGICAL_SCALE)
                .ok_or(LayoutError::GeometryOverflow)?,
            physical_size
                .height
                .checked_div(LOGICAL_SCALE)
                .ok_or(LayoutError::GeometryOverflow)?,
        );
        if logical_size.width < MIN_LOGICAL_WIDTH || logical_size.height < MIN_LOGICAL_HEIGHT {
            return Err(LayoutError::ViewportTooSmall);
        }

        let inner_width = logical_size
            .width
            .saturating_sub(OUTER_MARGIN.saturating_mul(2));
        let composer_y = logical_size
            .height
            .checked_sub(OUTER_MARGIN.saturating_add(COMPOSER_HEIGHT))
            .ok_or(LayoutError::GeometryOverflow)?;
        let content_y = SECURE_STRIP_HEIGHT.saturating_add(OUTER_MARGIN);
        let content_bottom = composer_y
            .checked_sub(PANEL_GAP)
            .ok_or(LayoutError::GeometryOverflow)?;
        let content_height = content_bottom
            .checked_sub(content_y)
            .ok_or(LayoutError::GeometryOverflow)?;

        let secure_strip = Rect::new(0, 0, logical_size.width, SECURE_STRIP_HEIGHT);
        let composer = Rect::new(OUTER_MARGIN, composer_y, inner_width, COMPOSER_HEIGHT);
        let personal_surface = Rect::new(
            0,
            SECURE_STRIP_HEIGHT,
            logical_size.width,
            logical_size.height.saturating_sub(SECURE_STRIP_HEIGHT),
        );

        let can_use_columns = inner_width
            >= MIN_CONVERSATION_WIDTH
                .saturating_add(PANEL_GAP)
                .saturating_add(MIN_CONTEXT_WIDTH);
        let (panel_flow, conversation, context) = if can_use_columns {
            let usable_width = inner_width.saturating_sub(PANEL_GAP);
            let conversation_width = usable_width.saturating_mul(2) / 3;
            let context_width = usable_width.saturating_sub(conversation_width);
            (
                PanelFlow::SideBySide,
                Rect::new(OUTER_MARGIN, content_y, conversation_width, content_height),
                Rect::new(
                    OUTER_MARGIN
                        .saturating_add(conversation_width)
                        .saturating_add(PANEL_GAP),
                    content_y,
                    context_width,
                    content_height,
                ),
            )
        } else {
            let usable_height = content_height.saturating_sub(PANEL_GAP);
            let context_height = (usable_height / 3).max(MIN_CONTEXT_HEIGHT);
            let conversation_height = usable_height.saturating_sub(context_height);
            if conversation_height < MIN_CONVERSATION_HEIGHT {
                return Err(LayoutError::ViewportTooSmall);
            }
            (
                PanelFlow::Stacked,
                Rect::new(OUTER_MARGIN, content_y, inner_width, conversation_height),
                Rect::new(
                    OUTER_MARGIN,
                    content_y
                        .saturating_add(conversation_height)
                        .saturating_add(PANEL_GAP),
                    inner_width,
                    context_height,
                ),
            )
        };

        let overlay_width = inner_width.min(MAX_OVERLAY_WIDTH);
        let overlay_height = personal_surface
            .height
            .saturating_sub(OUTER_MARGIN.saturating_mul(2))
            .min(MAX_OVERLAY_HEIGHT);
        let trusted_overlay = Rect::new(
            logical_size.width.saturating_sub(overlay_width) / 2,
            personal_surface
                .y
                .saturating_add(personal_surface.height.saturating_sub(overlay_height) / 2),
            overlay_width,
            overlay_height,
        );

        Ok(Self {
            physical_size,
            logical_size,
            panel_flow,
            secure_strip,
            conversation,
            context,
            composer,
            personal_surface,
            trusted_overlay,
        })
    }

    pub fn hit_test_physical(&self, layer: HitLayer, point: Point) -> Option<HitTarget> {
        if point.x >= self.physical_size.width || point.y >= self.physical_size.height {
            return None;
        }
        self.hit_test_logical(
            layer,
            Point::new(point.x / LOGICAL_SCALE, point.y / LOGICAL_SCALE),
        )
    }

    pub fn hit_test_logical(&self, layer: HitLayer, point: Point) -> Option<HitTarget> {
        let viewport = Rect::new(0, 0, self.logical_size.width, self.logical_size.height);
        if !viewport.contains(point) {
            return None;
        }
        if self.secure_strip.contains(point) {
            return Some(HitTarget::SecureStrip);
        }

        match layer {
            HitLayer::Genesis => {
                if self.composer.contains(point) {
                    Some(HitTarget::Composer)
                } else if self.conversation.contains(point) {
                    Some(HitTarget::Conversation)
                } else if self.context.contains(point) {
                    Some(HitTarget::Context)
                } else {
                    None
                }
            }
            HitLayer::Personal => self
                .personal_surface
                .contains(point)
                .then_some(HitTarget::PersonalSurface),
            HitLayer::TrustedOverlay => {
                if self.trusted_overlay.contains(point) {
                    Some(HitTarget::TrustedOverlay)
                } else if self.personal_surface.contains(point) {
                    Some(HitTarget::OverlayScrim)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORTS: [Size; 3] = [
        Size::new(1024, 768),
        Size::new(1280, 800),
        Size::new(1920, 1080),
    ];

    fn assert_inside(inner: Rect, outer: Rect) {
        assert!(inner.width > 0 && inner.height > 0);
        assert!(inner.x >= outer.x && inner.y >= outer.y);
        assert!(inner.right() <= outer.right());
        assert!(inner.bottom() <= outer.bottom());
    }

    #[test]
    fn supported_viewports_stay_bounded_and_reserve_the_secure_strip() {
        for physical in VIEWPORTS {
            let layout = GenesisLayout::new(physical).unwrap();
            let viewport = Rect::new(0, 0, layout.logical_size.width, layout.logical_size.height);

            assert_eq!(layout.secure_strip, Rect::new(0, 0, viewport.width, 38));
            assert_inside(layout.conversation, viewport);
            assert_inside(layout.context, viewport);
            assert_inside(layout.composer, viewport);
            assert_inside(layout.personal_surface, viewport);
            assert_inside(layout.trusted_overlay, layout.personal_surface);
            assert!(!layout.secure_strip.overlaps(layout.conversation));
            assert!(!layout.secure_strip.overlaps(layout.context));
            assert!(!layout.secure_strip.overlaps(layout.composer));
            assert!(!layout.secure_strip.overlaps(layout.personal_surface));
            assert!(!layout.secure_strip.overlaps(layout.trusted_overlay));
        }
    }

    #[test]
    fn panels_do_not_overlap_and_keep_minimum_usable_areas() {
        for physical in VIEWPORTS {
            let layout = GenesisLayout::new(physical).unwrap();

            assert!(!layout.conversation.overlaps(layout.context));
            assert!(!layout.conversation.overlaps(layout.composer));
            assert!(!layout.context.overlaps(layout.composer));
            assert!(layout.conversation.width >= MIN_CONVERSATION_WIDTH);
            assert!(layout.conversation.height >= MIN_CONVERSATION_HEIGHT);
            assert!(layout.context.width >= MIN_CONTEXT_WIDTH);
            assert!(layout.context.height >= MIN_CONTEXT_HEIGHT);
            assert!(layout.composer.width >= MIN_LOGICAL_WIDTH - OUTER_MARGIN * 2);
            assert_eq!(layout.composer.height, COMPOSER_HEIGHT);
        }
    }

    #[test]
    fn narrow_viewport_stacks_context_and_wide_viewports_use_columns() {
        assert_eq!(
            GenesisLayout::new(VIEWPORTS[0]).unwrap().panel_flow,
            PanelFlow::Stacked
        );
        assert_eq!(
            GenesisLayout::new(VIEWPORTS[1]).unwrap().panel_flow,
            PanelFlow::SideBySide
        );
        assert_eq!(
            GenesisLayout::new(VIEWPORTS[2]).unwrap().panel_flow,
            PanelFlow::SideBySide
        );
    }

    #[test]
    fn hit_targets_are_half_open_layered_and_deterministic() {
        let layout = GenesisLayout::new(Size::new(1280, 800)).unwrap();
        let inside = |rect: Rect| Point::new(rect.x, rect.y);

        assert_eq!(
            layout.hit_test_logical(HitLayer::Genesis, inside(layout.secure_strip)),
            Some(HitTarget::SecureStrip)
        );
        assert_eq!(
            layout.hit_test_logical(HitLayer::Genesis, inside(layout.conversation)),
            Some(HitTarget::Conversation)
        );
        assert_eq!(
            layout.hit_test_logical(HitLayer::Genesis, inside(layout.context)),
            Some(HitTarget::Context)
        );
        assert_eq!(
            layout.hit_test_logical(HitLayer::Genesis, inside(layout.composer)),
            Some(HitTarget::Composer)
        );
        assert_eq!(
            layout.hit_test_logical(HitLayer::Personal, inside(layout.secure_strip)),
            Some(HitTarget::SecureStrip)
        );
        assert_eq!(
            layout.hit_test_logical(HitLayer::Personal, inside(layout.personal_surface)),
            Some(HitTarget::PersonalSurface)
        );
        assert_eq!(
            layout.hit_test_logical(HitLayer::TrustedOverlay, inside(layout.trusted_overlay)),
            Some(HitTarget::TrustedOverlay)
        );
        assert_eq!(
            layout.hit_test_logical(
                HitLayer::TrustedOverlay,
                Point::new(0, layout.personal_surface.y)
            ),
            Some(HitTarget::OverlayScrim)
        );

        let boundary = Point::new(layout.conversation.right(), layout.conversation.y);
        assert_eq!(layout.hit_test_logical(HitLayer::Genesis, boundary), None);
        assert_eq!(
            layout.hit_test_physical(HitLayer::Genesis, Point::new(0, 0)),
            Some(HitTarget::SecureStrip)
        );
        assert_eq!(
            layout.hit_test_physical(HitLayer::Genesis, Point::new(1280, 0)),
            None
        );
    }

    #[test]
    fn undersized_viewports_fail_closed() {
        assert_eq!(
            GenesisLayout::new(Size::new(959, 719)),
            Err(LayoutError::ViewportTooSmall)
        );
    }
}
