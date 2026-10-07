//! Screen-space drawing for HUDs and UI (ADR-029).
//!
//! An entity with [`ScreenSpace`] is drawn in window coordinates instead of
//! through the camera: its `Transform2D` is in logical pixels, measured from
//! an anchor point of the window, and it is drawn after (on top of) all
//! world content.

use crate::math::{Mat4, Vec2};

/// A point of the window that screen-space positions are measured from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ScreenAnchor {
    /// Top-left corner.
    TopLeft,
    /// Middle of the top edge.
    Top,
    /// Top-right corner.
    TopRight,
    /// Middle of the left edge.
    Left,
    /// Centre of the window. The default.
    #[default]
    Center,
    /// Middle of the right edge.
    Right,
    /// Bottom-left corner.
    BottomLeft,
    /// Middle of the bottom edge.
    Bottom,
    /// Bottom-right corner.
    BottomRight,
}

impl ScreenAnchor {
    /// The anchor as a fraction of the window, from the bottom-left corner
    /// (+Y up): (0, 0) bottom-left … (1, 1) top-right.
    const fn fraction(self) -> (f32, f32) {
        match self {
            ScreenAnchor::TopLeft => (0.0, 1.0),
            ScreenAnchor::Top => (0.5, 1.0),
            ScreenAnchor::TopRight => (1.0, 1.0),
            ScreenAnchor::Left => (0.0, 0.5),
            ScreenAnchor::Center => (0.5, 0.5),
            ScreenAnchor::Right => (1.0, 0.5),
            ScreenAnchor::BottomLeft => (0.0, 0.0),
            ScreenAnchor::Bottom => (0.5, 0.0),
            ScreenAnchor::BottomRight => (1.0, 0.0),
        }
    }
}

/// Draws the entity's [`Quad`](super::Quad), [`Sprite`](super::Sprite) or
/// [`Text`](super::Text) in **screen space**, unaffected by the camera: for
/// HUDs, menus and other UI.
///
/// The entity's [`Transform2D`](crate::math::Transform2D) is then in logical
/// pixels relative to [`anchor`](Self::anchor), a point of the window, with the
/// same axes as the world (**+X right, +Y up**). So `ScreenSpace::TOP_LEFT`
/// with position (20, −30) is 20 px right of and 30 px below the top-left
/// corner, and stays there when the camera moves or the window is resized.
/// Screen-space entities are drawn after all world entities; among
/// themselves they are ordered by [`Layer`](super::Layer) as usual.
///
/// ```
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::{Color, Quad, ScreenSpace};
///
/// let mut world = purplepie::ecs::World::new();
/// // A 200×40 bar whose top-left corner is 10 px from the window's top-left corner.
/// world.spawn((
///     Transform2D::from_position(Vec2::new(110.0, -30.0)),
///     Quad::new(Vec2::new(200.0, 40.0), Color::hex(0x1D3557)),
///     ScreenSpace::TOP_LEFT,
/// ));
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ScreenSpace {
    /// The window point that the entity's position is measured from.
    pub anchor: ScreenAnchor,
}

impl ScreenSpace {
    /// Measured from the top-left corner.
    pub const TOP_LEFT: Self = Self::new(ScreenAnchor::TopLeft);
    /// Measured from the middle of the top edge.
    pub const TOP: Self = Self::new(ScreenAnchor::Top);
    /// Measured from the top-right corner.
    pub const TOP_RIGHT: Self = Self::new(ScreenAnchor::TopRight);
    /// Measured from the middle of the left edge.
    pub const LEFT: Self = Self::new(ScreenAnchor::Left);
    /// Measured from the centre of the window (the default).
    pub const CENTER: Self = Self::new(ScreenAnchor::Center);
    /// Measured from the middle of the right edge.
    pub const RIGHT: Self = Self::new(ScreenAnchor::Right);
    /// Measured from the bottom-left corner.
    pub const BOTTOM_LEFT: Self = Self::new(ScreenAnchor::BottomLeft);
    /// Measured from the middle of the bottom edge.
    pub const BOTTOM: Self = Self::new(ScreenAnchor::Bottom);
    /// Measured from the bottom-right corner.
    pub const BOTTOM_RIGHT: Self = Self::new(ScreenAnchor::BottomRight);

    /// Screen space measured from `anchor`.
    pub const fn new(anchor: ScreenAnchor) -> Self {
        Self { anchor }
    }

    /// The anchor point in window coordinates (logical pixels, top-left
    /// origin, +Y down, like [`Input::cursor_position`](crate::input::Input::cursor_position))
    /// for a `viewport`-sized window
    /// ([`Context::viewport_size`](crate::Context::viewport_size)).
    pub fn anchor_point(&self, viewport: Vec2) -> Vec2 {
        let (fx, fy) = self.anchor.fraction();
        Vec2::new(fx * viewport.x, (1.0 - fy) * viewport.y)
    }

    /// Converts a window position (logical pixels, top-left origin, +Y down,
    /// e.g. the cursor) to this space's coordinates (from the anchor, +Y up):
    /// the inverse of [`to_window`](Self::to_window). Use it to hit-test UI.
    pub fn from_window(&self, window: Vec2, viewport: Vec2) -> Vec2 {
        let from_anchor = window - self.anchor_point(viewport);
        Vec2::new(from_anchor.x, -from_anchor.y)
    }

    /// Converts a position in this space (from the anchor, +Y up) to window
    /// coordinates (logical pixels, top-left origin, +Y down).
    pub fn to_window(&self, position: Vec2, viewport: Vec2) -> Vec2 {
        self.anchor_point(viewport) + Vec2::new(position.x, -position.y)
    }

    /// Screen → clip transform for `anchor` in a `viewport`-sized window
    /// (logical pixels): the same kind of matrix the camera builds, with the
    /// anchor point as the origin and 1 unit = 1 logical pixel.
    pub(crate) fn view_projection(anchor: ScreenAnchor, viewport: Vec2) -> Mat4 {
        let (fx, fy) = anchor.fraction();
        let min = Vec2::new(-fx * viewport.x, -fy * viewport.y);
        let max = min + viewport;
        glam::camera::rh::proj::directx::orthographic(min.x, max.x, min.y, max.y, -1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Camera2D;
    use glam::Vec4;

    /// Clip space → window pixels (top-left origin, +Y down).
    fn to_pixels(m: Mat4, p: Vec2, viewport: Vec2) -> Vec2 {
        let c = m * Vec4::new(p.x, p.y, 0.0, 1.0);
        Vec2::new(
            (c.x + 1.0) * 0.5 * viewport.x,
            (1.0 - c.y) * 0.5 * viewport.y,
        )
    }

    #[test]
    fn the_centre_anchor_is_the_default_camera_view() {
        let viewport = Vec2::new(1280.0, 720.0);
        assert_eq!(ScreenSpace::default(), ScreenSpace::CENTER);
        assert_eq!(
            ScreenSpace::view_projection(ScreenAnchor::Center, viewport),
            Camera2D::default().view_projection(viewport)
        );
    }

    #[test]
    fn positions_are_pixels_from_the_anchor_point() {
        let viewport = Vec2::new(800.0, 600.0);
        let cases = [
            (
                ScreenSpace::TOP_LEFT,
                Vec2::new(20.0, -30.0),
                Vec2::new(20.0, 30.0),
            ),
            (ScreenSpace::TOP, Vec2::ZERO, Vec2::new(400.0, 0.0)),
            (
                ScreenSpace::TOP_RIGHT,
                Vec2::new(-10.0, -10.0),
                Vec2::new(790.0, 10.0),
            ),
            (
                ScreenSpace::LEFT,
                Vec2::new(5.0, 0.0),
                Vec2::new(5.0, 300.0),
            ),
            (
                ScreenSpace::CENTER,
                Vec2::new(1.0, 1.0),
                Vec2::new(401.0, 299.0),
            ),
            (ScreenSpace::RIGHT, Vec2::ZERO, Vec2::new(800.0, 300.0)),
            (
                ScreenSpace::BOTTOM_LEFT,
                Vec2::new(10.0, 10.0),
                Vec2::new(10.0, 590.0),
            ),
            (
                ScreenSpace::BOTTOM,
                Vec2::new(0.0, 4.0),
                Vec2::new(400.0, 596.0),
            ),
            (
                ScreenSpace::BOTTOM_RIGHT,
                Vec2::new(-30.0, 30.0),
                Vec2::new(770.0, 570.0),
            ),
        ];
        for (space, position, window) in cases {
            let vp = ScreenSpace::view_projection(space.anchor, viewport);
            let pixels = to_pixels(vp, position, viewport);
            assert!(pixels.abs_diff_eq(window, 1e-3), "{space:?}: {pixels}");
            assert_eq!(space.to_window(position, viewport), window, "{space:?}");
            assert_eq!(space.from_window(window, viewport), position, "{space:?}");
        }
    }

    #[test]
    fn corner_anchors_follow_the_window_when_it_is_resized() {
        let space = ScreenSpace::BOTTOM_RIGHT;
        let offset = Vec2::new(-30.0, 30.0);
        for viewport in [
            Vec2::new(800.0, 600.0),
            Vec2::new(1280.0, 720.0),
            Vec2::new(321.0, 123.0),
        ] {
            let vp = ScreenSpace::view_projection(space.anchor, viewport);
            let pixels = to_pixels(vp, offset, viewport);
            assert!(
                pixels.abs_diff_eq(viewport - Vec2::splat(30.0), 1e-3),
                "{viewport}"
            );
        }
    }

    /// At any DPI the renderer maps logical pixels to physical ones by the
    /// scale factor (the viewport is physical / scale), so a screen-space
    /// position lands on `scale ×` its logical window position.
    #[test]
    fn screen_space_is_in_logical_pixels_at_any_dpi() {
        let physical = Vec2::new(1920.0, 1080.0);
        for scale in [1.0_f32, 1.25, 2.0] {
            let viewport = physical / scale;
            let space = ScreenSpace::TOP_LEFT;
            let vp = ScreenSpace::view_projection(space.anchor, viewport);
            let position = Vec2::new(40.0, -24.0);
            let on_screen = to_pixels(vp, position, physical);
            assert!(
                on_screen.abs_diff_eq(space.to_window(position, viewport) * scale, 1e-2),
                "scale {scale}: {on_screen}"
            );
        }
    }
}
