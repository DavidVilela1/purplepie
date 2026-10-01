//! The game-facing callback trait and the per-call context (ADR-008).

use std::path::Path;

use crate::ecs::World;
use crate::error::Result;
use crate::math::Vec2;
use crate::render::{Camera2D, TextureId, Textures};
use crate::time::Time;

/// Implemented by a game. The engine owns the game value and calls these
/// methods from its main loop, in this order each frame:
///
/// 1. [`fixed_update`](Game::fixed_update), 0 or more times (fixed timestep);
/// 2. [`update`](Game::update), once.
///
/// All methods have empty defaults, so a game implements only what it needs.
/// Once exit has begun (for example after [`Context::request_exit`]), no
/// further callbacks are made.
pub trait Game {
    /// Called once, after the window exists and before the first frame.
    ///
    /// Returning an error stops the engine, and [`Engine::run`](crate::Engine::run)
    /// returns that error.
    fn init(&mut self, ctx: &mut Context<'_>) -> Result<()> {
        let _ = ctx;
        Ok(())
    }

    /// Called at a fixed rate (`EngineConfig::fixed_dt`, default 60 Hz),
    /// independent of the frame rate. Put gameplay and simulation here.
    /// [`Context::dt`] equals the fixed step.
    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        let _ = ctx;
    }

    /// Called once per frame, after the fixed updates. [`Context::dt`] is the
    /// (clamped) frame delta.
    fn update(&mut self, ctx: &mut Context<'_>) {
        let _ = ctx;
    }
}

/// What game code can see and do during a callback.
///
/// Built fresh for every callback from engine-owned state. It never exposes
/// `winit` or GPU types: textures are loaded here but uploaded by the
/// renderer (ADR-020), and the camera is plain data (ADR-022). Input is
/// added in Stage 8.
pub struct Context<'a> {
    exit_requested: &'a mut bool,
    time: &'a Time,
    world: &'a mut World,
    textures: &'a mut Textures,
    camera: &'a mut Camera2D,
    /// Window size in logical pixels (zero while there is no window or it is minimized).
    viewport: Vec2,
    dt: f64,
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("exit_requested", &self.exit_requested)
            .field("time", &self.time)
            .field("entities", &self.world.len())
            .field("textures", &self.textures.len())
            .field("camera", &self.camera)
            .field("viewport", &self.viewport)
            .field("dt", &self.dt)
            .finish()
    }
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        exit_requested: &'a mut bool,
        time: &'a Time,
        world: &'a mut World,
        textures: &'a mut Textures,
        camera: &'a mut Camera2D,
        viewport: Vec2,
        dt: f64,
    ) -> Self {
        Self {
            exit_requested,
            time,
            world,
            textures,
            camera,
            viewport,
            dt,
        }
    }

    /// The game world (read-only).
    pub fn world(&self) -> &World {
        self.world
    }

    /// The game world: spawn entities, attach components, run queries and systems.
    ///
    /// Read [`dt`](Self::dt) into a local first when passing both to a system,
    /// because the world borrow is exclusive:
    /// `let dt = ctx.dt(); ecs::integrate_velocity(ctx.world_mut(), dt);`
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    /// Timing information for the current frame.
    pub fn time(&self) -> &Time {
        self.time
    }

    /// Seconds to advance by in *this* callback: the fixed step inside
    /// `fixed_update`, the frame delta inside `update`, and `0.0` in `init`.
    ///
    /// Returned as `f32` for direct use in gameplay/vector math. Precise `f64`
    /// values are available from [`time`](Self::time).
    pub fn dt(&self) -> f32 {
        self.dt as f32
    }

    /// Loads a PNG image and returns a handle for [`Sprite`](crate::render::Sprite)s.
    ///
    /// The file is read and decoded now, so a problem is reported here as
    /// [`Error::Asset`](crate::Error::Asset) (missing file, not a PNG, …). The
    /// GPU upload happens before the next frame is drawn (ADR-020). Relative
    /// paths are resolved against the current working directory. Loading the
    /// same path again returns the same [`TextureId`] without reading the file.
    /// Usually called from [`Game::init`].
    ///
    /// A texture larger than the GPU supports (at least 2048×2048 everywhere)
    /// is reported as `Error::Asset` when the next frame is drawn, which stops
    /// the engine.
    pub fn load_texture(&mut self, path: impl AsRef<Path>) -> Result<TextureId> {
        self.textures.load(path.as_ref())
    }

    /// Width and height of a loaded texture in texels, e.g. to give a
    /// [`Sprite`](crate::render::Sprite) its image's natural size.
    /// `None` only for an id that came from a different engine run.
    pub fn texture_size(&self, texture: TextureId) -> Option<Vec2> {
        self.textures.size(texture)
    }

    /// The camera the next frame is drawn with (ADR-022).
    pub fn camera(&self) -> &Camera2D {
        self.camera
    }

    /// The camera, to pan or zoom: `ctx.camera_mut().position.x += 10.0;`.
    /// Changes apply to the next frame drawn and persist until changed again.
    pub fn camera_mut(&mut self) -> &mut Camera2D {
        self.camera
    }

    /// The window's drawing area in logical pixels (physical pixels ÷ DPI
    /// scale), as used by [`Camera2D::screen_to_world`]. Zero while the window
    /// is minimized.
    pub fn viewport_size(&self) -> Vec2 {
        self.viewport
    }

    /// Asks the engine to shut down cleanly. No further game callbacks are made.
    pub fn request_exit(&mut self) {
        *self.exit_requested = true;
    }

    /// Whether [`request_exit`](Self::request_exit) has been called.
    pub fn exit_requested(&self) -> bool {
        *self.exit_requested
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_exit_sets_the_engine_flag() {
        let time = Time::new(0.25);
        let mut world = World::new();
        let mut textures = Textures::default();
        let mut camera = Camera2D::default();
        let mut flag = false;
        let mut ctx = Context::new(
            &mut flag,
            &time,
            &mut world,
            &mut textures,
            &mut camera,
            Vec2::new(800.0, 600.0),
            0.0,
        );
        assert!(!ctx.exit_requested());
        ctx.request_exit();
        assert!(ctx.exit_requested());
        assert!(flag);
    }

    #[test]
    fn exposes_time_and_callback_dt() {
        let time = Time::new(0.25);
        let mut world = World::new();
        let mut textures = Textures::default();
        let mut camera = Camera2D::default();
        let mut flag = false;
        let ctx = Context::new(
            &mut flag,
            &time,
            &mut world,
            &mut textures,
            &mut camera,
            Vec2::new(800.0, 600.0),
            0.25,
        );
        assert_eq!(ctx.dt(), 0.25_f32);
        assert_eq!(ctx.time().fixed_dt(), 0.25);
    }

    #[test]
    fn world_changes_made_through_the_context_persist() {
        let time = Time::new(0.25);
        let mut world = World::new();
        let mut textures = Textures::default();
        let mut camera = Camera2D::default();
        let mut flag = false;
        {
            let mut ctx = Context::new(
                &mut flag,
                &time,
                &mut world,
                &mut textures,
                &mut camera,
                Vec2::new(800.0, 600.0),
                0.0,
            );
            ctx.world_mut().spawn((1_u32,));
            assert_eq!(ctx.world().len(), 1);
        }
        assert_eq!(world.len(), 1);
    }

    #[test]
    fn load_texture_errors_are_typed_and_sizes_are_reported() {
        let time = Time::new(0.25);
        let mut world = World::new();
        let mut textures = Textures::default();
        let mut camera = Camera2D::default();
        let mut flag = false;
        let mut ctx = Context::new(
            &mut flag,
            &time,
            &mut world,
            &mut textures,
            &mut camera,
            Vec2::new(800.0, 600.0),
            0.0,
        );
        let err = ctx
            .load_texture("this/file/does/not/exist.png")
            .expect_err("missing file");
        assert!(matches!(err, crate::Error::Asset { .. }), "{err}");

        let sandbox = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/textures/sandbox_quadrants.png"
        );
        let id = ctx.load_texture(sandbox).expect("sandbox texture");
        assert_eq!(ctx.texture_size(id), Some(Vec2::new(16.0, 16.0)));
        assert_eq!(ctx.load_texture(sandbox).expect("again"), id);
    }

    #[test]
    fn camera_changes_persist_and_viewport_is_reported() {
        let time = Time::new(0.25);
        let mut world = World::new();
        let mut textures = Textures::default();
        let mut camera = Camera2D::default();
        let mut flag = false;
        {
            let mut ctx = Context::new(
                &mut flag,
                &time,
                &mut world,
                &mut textures,
                &mut camera,
                Vec2::new(800.0, 600.0),
                0.0,
            );
            assert_eq!(ctx.camera(), &Camera2D::IDENTITY);
            assert_eq!(ctx.viewport_size(), Vec2::new(800.0, 600.0));
            ctx.camera_mut().position = Vec2::new(5.0, 6.0);
            ctx.camera_mut().zoom = 2.0;
        }
        assert_eq!(camera, Camera2D::new(Vec2::new(5.0, 6.0), 2.0));
    }

    #[test]
    fn default_callbacks_do_nothing() {
        struct Minimal;
        impl Game for Minimal {}
        let time = Time::new(0.25);
        let mut world = World::new();
        let mut textures = Textures::default();
        let mut camera = Camera2D::default();
        let mut flag = false;
        let mut ctx = Context::new(
            &mut flag,
            &time,
            &mut world,
            &mut textures,
            &mut camera,
            Vec2::new(800.0, 600.0),
            0.0,
        );
        let mut game = Minimal;
        assert!(game.init(&mut ctx).is_ok());
        game.fixed_update(&mut ctx);
        game.update(&mut ctx);
        assert!(!flag);
    }
}
