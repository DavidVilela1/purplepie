//! The 2D camera and the screen ↔ world mapping (ADR-018, ADR-022).

use crate::math::{Mat4, Vec2};

/// The view onto the world: which world point is at the centre of the window,
/// and how much it is magnified.
///
/// There is one camera, owned by the engine and reached through
/// [`Context::camera`](crate::Context::camera) /
/// [`Context::camera_mut`](crate::Context::camera_mut). The default camera
/// (`position` (0, 0), `zoom` 1) is the ADR-018 view: world origin at the
/// window centre, 1 world unit = 1 logical pixel. Resizing the window shows
/// more or less of the world; it never changes the zoom.
///
/// **Screen coordinates** (for [`screen_to_world`](Self::screen_to_world) and
/// [`world_to_screen`](Self::world_to_screen)) are logical pixels with the
/// origin at the top-left corner of the window's drawing area and **+Y down**,
/// like OS cursor positions. Pixel centres are at `n + 0.5`. Logical pixels
/// are physical pixels divided by the window's scale factor (DPI).
///
/// ```
/// use purplepie::math::Vec2;
/// use purplepie::render::Camera2D;
///
/// let camera = Camera2D::new(Vec2::new(100.0, 50.0), 2.0);
/// let viewport = Vec2::new(800.0, 600.0); // window size in logical pixels
/// // The window centre always shows the camera position.
/// assert_eq!(camera.screen_to_world(Vec2::new(400.0, 300.0), viewport), Vec2::new(100.0, 50.0));
/// // 10 world units right of the camera are 20 logical pixels right of the centre at zoom 2.
/// assert_eq!(camera.world_to_screen(Vec2::new(110.0, 50.0), viewport), Vec2::new(420.0, 300.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Camera2D {
    /// The world point shown at the centre of the window.
    pub position: Vec2,
    /// Magnification: logical pixels per world unit. `2.0` shows everything
    /// twice as large (and half as much of the world). Must be finite and
    /// greater than zero; any other value is treated as `1.0`.
    pub zoom: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Camera2D {
    /// The default view (ADR-018): centred on the world origin, zoom 1.
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        zoom: 1.0,
    };

    /// A camera centred on `position` with magnification `zoom`.
    pub const fn new(position: Vec2, zoom: f32) -> Self {
        Self { position, zoom }
    }

    /// A camera centred on `center` that shows the whole `size` (world
    /// units) as large as possible in a `viewport`-sized window (logical
    /// pixels, e.g. [`Context::viewport_size`](crate::Context::viewport_size)).
    /// The other axis shows extra world. Returns the default zoom of 1 when
    /// `size` or `viewport` has a zero or invalid component.
    ///
    /// ```
    /// use purplepie::math::Vec2;
    /// use purplepie::render::Camera2D;
    ///
    /// // An 800×600 playfield in a 1600×900 window: height limits, zoom 1.5.
    /// let camera = Camera2D::fit(Vec2::ZERO, Vec2::new(800.0, 600.0), Vec2::new(1600.0, 900.0));
    /// assert_eq!(camera.zoom, 1.5);
    /// ```
    pub fn fit(center: Vec2, size: Vec2, viewport: Vec2) -> Self {
        let usable = |v: Vec2| v.is_finite() && v.x > 0.0 && v.y > 0.0;
        let zoom = if usable(size) && usable(viewport) {
            (viewport / size).min_element()
        } else {
            1.0
        };
        Self::new(center, zoom)
    }

    /// The zoom actually used: `zoom` if it is finite and positive, else `1.0`.
    pub fn effective_zoom(&self) -> f32 {
        if self.zoom.is_finite() && self.zoom > 0.0 {
            self.zoom
        } else {
            1.0
        }
    }

    /// Converts a screen position (logical pixels, top-left origin, +Y down)
    /// to the world position drawn there. `viewport` is the window's size in
    /// logical pixels ([`Context::viewport_size`](crate::Context::viewport_size)).
    pub fn screen_to_world(&self, screen: Vec2, viewport: Vec2) -> Vec2 {
        let from_centre = screen - viewport * 0.5;
        self.position + Vec2::new(from_centre.x, -from_centre.y) / self.effective_zoom()
    }

    /// Converts a world position to the screen position (logical pixels,
    /// top-left origin, +Y down) where it is drawn. The inverse of
    /// [`screen_to_world`](Self::screen_to_world).
    pub fn world_to_screen(&self, world: Vec2, viewport: Vec2) -> Vec2 {
        let offset = (world - self.position) * self.effective_zoom();
        viewport * 0.5 + Vec2::new(offset.x, -offset.y)
    }

    /// The world rectangle visible in a `viewport`-sized window, as (min, max) corners.
    pub fn visible_world_rect(&self, viewport: Vec2) -> (Vec2, Vec2) {
        let half = viewport * 0.5 / self.effective_zoom();
        (self.position - half, self.position + half)
    }

    /// World → clip transform used by the renderer for a `viewport`-sized
    /// window (logical pixels).
    pub(crate) fn view_projection(&self, viewport: Vec2) -> Mat4 {
        let (min, max) = self.visible_world_rect(viewport);
        // Right-handed, Y-up view space → WebGPU NDC (Y-up, depth in [0, 1]).
        glam::camera::rh::proj::directx::orthographic(min.x, max.x, min.y, max.y, -1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    fn clip(m: Mat4, world: Vec2) -> Vec2 {
        let p = m * Vec4::new(world.x, world.y, 0.0, 1.0);
        Vec2::new(p.x / p.w, p.y / p.w)
    }

    #[test]
    fn default_camera_is_exactly_the_adr_018_view() {
        let viewport = Vec2::new(200.0, 100.0);
        let half = viewport * 0.5;
        let adr_018 = glam::camera::rh::proj::directx::orthographic(
            -half.x, half.x, -half.y, half.y, -1.0, 1.0,
        );
        assert_eq!(Camera2D::default().view_projection(viewport), adr_018);
        assert_eq!(Camera2D::default(), Camera2D::IDENTITY);
    }

    #[test]
    fn panning_moves_the_camera_position_to_the_window_centre() {
        let camera = Camera2D::new(Vec2::new(30.0, -20.0), 1.0);
        let vp = camera.view_projection(Vec2::new(200.0, 100.0));
        assert!(clip(vp, Vec2::new(30.0, -20.0)).abs_diff_eq(Vec2::ZERO, 1e-6));
        // 100 units right of the camera is the right edge (clip x = 1).
        assert!(clip(vp, Vec2::new(130.0, -20.0)).abs_diff_eq(Vec2::new(1.0, 0.0), 1e-6));
    }

    #[test]
    fn zoom_magnifies_around_the_camera_position() {
        let camera = Camera2D::new(Vec2::new(10.0, 10.0), 2.0);
        let vp = camera.view_projection(Vec2::new(200.0, 100.0));
        // At zoom 2 only 50 units fit between the centre and the right edge.
        assert!(clip(vp, Vec2::new(60.0, 10.0)).abs_diff_eq(Vec2::new(1.0, 0.0), 1e-6));
        assert!(clip(vp, Vec2::new(10.0, 35.0)).abs_diff_eq(Vec2::new(0.0, 1.0), 1e-6));
        let (min, max) = camera.visible_world_rect(Vec2::new(200.0, 100.0));
        assert_eq!((min, max), (Vec2::new(-40.0, -15.0), Vec2::new(60.0, 35.0)));
    }

    #[test]
    fn invalid_zoom_falls_back_to_one() {
        for zoom in [0.0, -2.0, f32::NAN, f32::INFINITY] {
            let camera = Camera2D::new(Vec2::ZERO, zoom);
            assert_eq!(camera.effective_zoom(), 1.0, "zoom {zoom}");
            assert_eq!(
                camera.view_projection(Vec2::new(200.0, 100.0)),
                Camera2D::IDENTITY.view_projection(Vec2::new(200.0, 100.0))
            );
        }
    }

    #[test]
    fn screen_axes_are_top_left_origin_y_down() {
        let camera = Camera2D::default();
        let viewport = Vec2::new(800.0, 600.0);
        assert_eq!(
            camera.screen_to_world(Vec2::new(400.0, 300.0), viewport),
            Vec2::ZERO
        );
        assert_eq!(
            camera.screen_to_world(Vec2::ZERO, viewport),
            Vec2::new(-400.0, 300.0)
        );
        assert_eq!(
            camera.screen_to_world(viewport, viewport),
            Vec2::new(400.0, -300.0)
        );
    }

    #[test]
    fn screen_and_world_round_trip() {
        let viewports = [
            Vec2::new(1280.0, 720.0),
            Vec2::new(333.0, 777.0),
            Vec2::new(1.0, 1.0),
        ];
        let cameras = [
            Camera2D::default(),
            Camera2D::new(Vec2::new(-123.5, 456.25), 0.25),
            Camera2D::new(Vec2::new(1e4, -1e4), 3.0),
        ];
        let points = [Vec2::ZERO, Vec2::new(17.5, -3.25), Vec2::new(-640.0, 360.0)];
        for viewport in viewports {
            for camera in cameras {
                for p in points {
                    let screen = camera.world_to_screen(p, viewport);
                    let back = camera.screen_to_world(screen, viewport);
                    assert!(
                        back.abs_diff_eq(p, 1e-2),
                        "{camera:?} {viewport} {p} → {back}"
                    );
                    let world = camera.screen_to_world(p, viewport);
                    let again = camera.world_to_screen(world, viewport);
                    assert!(
                        again.abs_diff_eq(p, 1e-2),
                        "{camera:?} {viewport} {p} → {again}"
                    );
                }
            }
        }
    }

    #[test]
    fn fit_shows_the_whole_area_and_centres_it() {
        let viewport = Vec2::new(1024.0, 768.0);
        let camera = Camera2D::fit(Vec2::new(10.0, -5.0), Vec2::new(912.0, 812.0), viewport);
        let (min, max) = camera.visible_world_rect(viewport);
        // Height is the limiting axis: exactly 812 units tall, wider than 912.
        assert!((max.y - min.y - 812.0).abs() < 1e-3);
        assert!(max.x - min.x >= 912.0);
        assert!(((min + max) / 2.0).abs_diff_eq(Vec2::new(10.0, -5.0), 1e-4));
        for bad in [Vec2::ZERO, Vec2::new(f32::NAN, 1.0)] {
            assert_eq!(Camera2D::fit(Vec2::ZERO, bad, viewport).zoom, 1.0);
            assert_eq!(Camera2D::fit(Vec2::ZERO, Vec2::ONE, bad).zoom, 1.0);
        }
    }

    /// For each DPI scale, a world point must land on the same physical pixel
    /// whether computed by the renderer's projection (clip → physical pixels)
    /// or by `world_to_screen` (logical pixels × scale).
    #[test]
    fn world_to_screen_agrees_with_the_projection_at_any_dpi() {
        let physical = Vec2::new(1920.0, 1080.0);
        let camera = Camera2D::new(Vec2::new(40.0, -25.0), 1.5);
        for scale in [1.0_f32, 1.25, 2.0] {
            let viewport = physical / scale; // what the renderer and Context use
            let vp = camera.view_projection(viewport);
            for world in [
                Vec2::new(40.0, -25.0),
                Vec2::new(300.0, 100.0),
                Vec2::new(-512.0, -64.0),
            ] {
                let c = clip(vp, world);
                // Clip (−1..1, +Y up) → physical pixels (top-left origin, +Y down).
                let from_projection = Vec2::new(
                    (c.x + 1.0) * 0.5 * physical.x,
                    (1.0 - c.y) * 0.5 * physical.y,
                );
                let from_camera = camera.world_to_screen(world, viewport) * scale;
                assert!(
                    from_projection.abs_diff_eq(from_camera, 1e-2),
                    "scale {scale}, world {world}: {from_projection} vs {from_camera}"
                );
            }
        }
    }
}
