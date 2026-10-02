//! Engine-owned state lent to the game through [`Context`](super::Context).

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
}

impl EngineState {
    pub(crate) fn new(fixed_dt: f64) -> Self {
        Self {
            exit_requested: false,
            time: Time::new(fixed_dt),
            world: World::new(),
            textures: Textures::default(),
            camera: Camera2D::default(),
            input: Input::default(),
            viewport: Vec2::ZERO,
        }
    }
}
