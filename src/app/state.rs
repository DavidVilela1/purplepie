//! Engine-owned state lent to the game through [`Context`](super::Context).

use crate::assets::AssetRoot;
use crate::ecs::World;
use crate::input::Input;
use crate::math::Vec2;
use crate::render::{Camera2D, Textures};
use crate::time::Time;

/// Everything a game callback can see, owned by the runner for the whole run.
/// Grouped so that `Context` borrows one value instead of a growing list.
pub(crate) struct EngineState {
    pub(crate) exit_requested: bool,
    pub(crate) time: Time,
    /// The single game world (ADR-008).
    pub(crate) world: World,
    /// Every texture the game loaded (CPU copies; ADR-020). Outlives renderers.
    pub(crate) textures: Textures,
    /// The single camera, read by the renderer (ADR-022).
    pub(crate) camera: Camera2D,
    /// Keyboard state, fed from window events (ADR-024).
    pub(crate) input: Input,
    /// Window drawing area in logical pixels, kept up to date from window events.
    pub(crate) viewport: Vec2,
    /// Where relative asset paths are resolved (ADR-025), chosen at startup.
    pub(crate) assets: AssetRoot,
    /// A window title requested by the game, applied by the runner after the callback.
    pub(crate) window_title: Option<String>,
}

impl EngineState {
    pub(crate) fn new(fixed_dt: f64, assets: AssetRoot) -> Self {
        Self {
            exit_requested: false,
            time: Time::new(fixed_dt),
            world: World::new(),
            textures: Textures::default(),
            camera: Camera2D::default(),
            input: Input::default(),
            viewport: Vec2::ZERO,
            assets,
            window_title: None,
        }
    }
}

/// A usable DPI scale: `scale_factor` if finite and positive, else 1.
pub(crate) fn sanitize_scale_factor(scale_factor: f64) -> f64 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

/// Physical pixels (what winit reports for sizes and the cursor) → logical
/// pixels (what games see, ADR-018/022).
pub(crate) fn physical_to_logical(x: f64, y: f64, scale_factor: f64) -> Vec2 {
    let scale = sanitize_scale_factor(scale_factor);
    Vec2::new((x / scale) as f32, (y / scale) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Camera2D;

    #[test]
    fn physical_to_logical_divides_by_a_sane_scale() {
        assert_eq!(
            physical_to_logical(250.0, 100.0, 1.25),
            Vec2::new(200.0, 80.0)
        );
        assert_eq!(
            physical_to_logical(250.0, 100.0, 0.0),
            Vec2::new(250.0, 100.0)
        );
        assert_eq!(
            physical_to_logical(250.0, 100.0, f64::NAN),
            Vec2::new(250.0, 100.0)
        );
    }

    /// The cursor path end to end: a physical cursor position becomes logical
    /// pixels and then a world position; the renderer must draw that world point
    /// under the same physical pixel, at any DPI.
    #[test]
    fn cursor_maps_to_the_world_point_drawn_under_it_at_any_dpi() {
        let physical_window = Vec2::new(1920.0, 1080.0);
        let camera = Camera2D::new(Vec2::new(-30.0, 12.5), 1.5);
        for scale in [1.0_f64, 1.25, 2.0] {
            let viewport = physical_window / scale as f32;
            for cursor in [(0.0, 0.0), (960.0, 540.0), (1500.25, 33.0)] {
                let logical = physical_to_logical(cursor.0, cursor.1, scale);
                let world = camera.screen_to_world(logical, viewport);
                let back = camera.world_to_screen(world, viewport) * scale as f32;
                assert!(
                    back.abs_diff_eq(Vec2::new(cursor.0 as f32, cursor.1 as f32), 1e-2),
                    "scale {scale}, cursor {cursor:?}: {back}"
                );
            }
        }
    }
}
