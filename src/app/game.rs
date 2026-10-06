//! The game-facing callback trait and the per-call context (ADR-008).

use std::path::Path;

use super::state::EngineState;
use crate::ecs::World;
use crate::error::Result;
use crate::input::Input;
use crate::math::Vec2;
use crate::render::{Camera2D, FontId, Text, TextMetrics, TextureId};
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
/// renderer (ADR-020), the camera is plain data (ADR-022), and keys are
/// PurplePie's own [`KeyCode`](crate::input::KeyCode)s (ADR-024).
pub struct Context<'a> {
    state: &'a mut EngineState,
    dt: f64,
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("exit_requested", &self.state.exit_requested)
            .field("time", &self.state.time)
            .field("entities", &self.state.world.len())
            .field("textures", &self.state.textures.len())
            .field("fonts", &self.state.fonts.len())
            .field("camera", &self.state.camera)
            .field("input", &self.state.input)
            .field("viewport", &self.state.viewport)
            .field("dt", &self.dt)
            .finish()
    }
}

impl<'a> Context<'a> {
    pub(crate) fn new(state: &'a mut EngineState, dt: f64) -> Self {
        Self { state, dt }
    }

    /// The game world (read-only).
    pub fn world(&self) -> &World {
        &self.state.world
    }

    /// The game world: spawn entities, attach components, run queries and systems.
    ///
    /// Read [`dt`](Self::dt) into a local first when passing both to a system,
    /// because the world borrow is exclusive:
    /// `let dt = ctx.dt(); ecs::integrate_velocity(ctx.world_mut(), dt);`
    /// The same applies to [`input`](Self::input), [`cursor_world`](Self::cursor_world)
    /// and [`camera`](Self::camera): copy what you need out of them first (ADR-026).
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.state.world
    }

    /// Timing information for the current frame.
    pub fn time(&self) -> &Time {
        &self.state.time
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
    /// [`Error::Asset`](crate::Error::Asset) (missing file, not a PNG, no asset
    /// folder, …). The GPU upload happens before the next frame is drawn
    /// (ADR-020). Relative paths such as `"textures/player.png"` are resolved
    /// against the asset root ([`asset_root`](Self::asset_root), ADR-025);
    /// absolute paths are used as they are. Loading the same file again returns
    /// the same [`TextureId`] without reading it.
    /// Usually called from [`Game::init`].
    ///
    /// A texture larger than the GPU supports (at least 2048×2048 everywhere)
    /// is reported as `Error::Asset` when the next frame is drawn, which stops
    /// the engine.
    pub fn load_texture(&mut self, path: impl AsRef<Path>) -> Result<TextureId> {
        let file = self.state.assets.locate(path.as_ref())?;
        self.state.textures.load(&file)
    }

    /// Loads a TrueType (`.ttf`) or OpenType (`.otf`) font and returns a
    /// handle for [`Text`](crate::render::Text) components (ADR-027).
    ///
    /// The file is read and parsed now, so a problem is reported here as
    /// [`Error::Asset`](crate::Error::Asset). Paths resolve like
    /// [`load_texture`](Self::load_texture): relative to the asset root, and
    /// loading the same file again returns the same [`FontId`]. Glyphs are
    /// rasterized later, at the size they are drawn. PurplePie ships one font,
    /// `assets/fonts/Poppins-Regular.ttf` (SIL Open Font License; see
    /// `assets/fonts/OFL.txt` before redistributing it).
    pub fn load_font(&mut self, path: impl AsRef<Path>) -> Result<FontId> {
        let file = self.state.assets.locate(path.as_ref())?;
        self.state.fonts.load(&file)
    }

    /// The size of `text` in world units: its widest line, its height and the
    /// font's line metrics (ADR-027). Use it with
    /// [`TextMetrics::bounds`](crate::render::TextMetrics::bounds) to place a
    /// panel behind text, or to lay out text next to other things.
    /// `None` only for a font id that came from a different engine run.
    ///
    /// The result does not depend on the camera or the window; drawn text is
    /// placed on whole pixels, so it can differ by up to a pixel.
    pub fn measure_text(&self, text: &Text) -> Option<TextMetrics> {
        let font = self.state.fonts.get(text.font)?;
        Some(crate::render::measure_text(&text.content, font, text.size))
    }

    /// The folder relative asset paths are loaded from (ADR-025), or `None` if
    /// no `assets` folder was found and none was configured with
    /// [`EngineConfig::with_asset_root`](crate::EngineConfig::with_asset_root).
    pub fn asset_root(&self) -> Option<&Path> {
        self.state.assets.path()
    }

    /// Width and height of a loaded texture in texels, e.g. to give a
    /// [`Sprite`](crate::render::Sprite) its image's natural size.
    /// `None` only for an id that came from a different engine run.
    pub fn texture_size(&self, texture: TextureId) -> Option<Vec2> {
        self.state.textures.size(texture)
    }

    /// The camera the next frame is drawn with (ADR-022).
    pub fn camera(&self) -> &Camera2D {
        &self.state.camera
    }

    /// The camera, to pan or zoom: `ctx.camera_mut().position.x += 10.0;`.
    /// Changes apply to the next frame drawn and persist until changed again.
    pub fn camera_mut(&mut self) -> &mut Camera2D {
        &mut self.state.camera
    }

    /// The window's drawing area in logical pixels (physical pixels ÷ DPI
    /// scale), as used by [`Camera2D::screen_to_world`]. Zero while the window
    /// is minimized.
    pub fn viewport_size(&self) -> Vec2 {
        self.state.viewport
    }

    /// The keyboard (ADR-024). `just_pressed` / `just_released` report each
    /// key edge exactly once in `fixed_update` and once in `update`, however
    /// many fixed steps a frame runs; see [`input`](crate::input).
    pub fn input(&self) -> &Input {
        &self.state.input
    }

    /// The world position under the mouse cursor, through the current camera
    /// (ADR-022), or `None` while the cursor is outside the window. Screen
    /// coordinates are in [`Input::cursor_position`].
    pub fn cursor_world(&self) -> Option<Vec2> {
        let state = &*self.state;
        state
            .input
            .cursor_position()
            .map(|screen| state.camera.screen_to_world(screen, state.viewport))
    }

    /// Changes the window title, e.g. to show the score. Applied after the
    /// current callback returns; calling it again in the same callback replaces
    /// the earlier request.
    pub fn set_window_title(&mut self, title: impl Into<String>) {
        self.state.window_title = Some(title.into());
    }

    /// Asks the engine to shut down cleanly. No further game callbacks are made.
    pub fn request_exit(&mut self) {
        self.state.exit_requested = true;
    }

    /// Whether [`request_exit`](Self::request_exit) has been called.
    pub fn exit_requested(&self) -> bool {
        self.state.exit_requested
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetRoot;
    use std::path::PathBuf;

    #[test]
    fn request_exit_sets_the_engine_flag() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let mut ctx = Context::new(&mut state, 0.0);
        assert!(!ctx.exit_requested());
        ctx.request_exit();
        assert!(ctx.exit_requested());
        assert!(state.exit_requested);
    }

    #[test]
    fn exposes_time_and_callback_dt() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let ctx = Context::new(&mut state, 0.25);
        assert_eq!(ctx.dt(), 0.25_f32);
        assert_eq!(ctx.time().fixed_dt(), 0.25);
    }

    #[test]
    fn world_changes_made_through_the_context_persist() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        {
            let mut ctx = Context::new(&mut state, 0.0);
            ctx.world_mut().spawn((1_u32,));
            assert_eq!(ctx.world().len(), 1);
        }
        assert_eq!(state.world.len(), 1);
    }

    #[test]
    fn load_texture_errors_are_typed_and_sizes_are_reported() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let mut ctx = Context::new(&mut state, 0.0);
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
    fn fonts_load_relative_to_the_asset_root_with_typed_errors() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let mut ctx = Context::new(&mut state, 0.0);
        let err = ctx
            .load_font("fonts/no-such-font.ttf")
            .expect_err("missing file");
        assert!(matches!(err, crate::Error::Asset { .. }), "{err}");
        // A PNG is not a font.
        let err = ctx
            .load_font("textures/sandbox_quadrants.png")
            .expect_err("not a font");
        assert!(matches!(err, crate::Error::Asset { .. }), "{err}");
        let relative = ctx.load_font("fonts/Poppins-Regular.ttf").expect("font");
        let absolute = ctx
            .load_font(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/fonts/Poppins-Regular.ttf"
            ))
            .expect("absolute");
        assert_eq!(relative, absolute, "same file, same font");
        assert_eq!(state.fonts.len(), 1);
    }

    #[test]
    fn measure_text_reports_world_unit_metrics() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let mut ctx = Context::new(&mut state, 0.0);
        let font = ctx.load_font("fonts/Poppins-Regular.ttf").expect("font");
        let one = ctx
            .measure_text(&Text::new("HH", font, 100.0))
            .expect("known font");
        // Poppins: 'H' advances 692/1000 em; ascent 1050, descent 350, gap 100 (units per 1000).
        assert!((one.width - 138.4).abs() < 1e-3, "{one:?}");
        assert!((one.ascent - 105.0).abs() < 1e-3 && (one.descent - 35.0).abs() < 1e-3);
        assert!((one.height - 140.0).abs() < 1e-3 && (one.line_height - 150.0).abs() < 1e-3);
        assert_eq!(one.lines, 1);
        let two = ctx
            .measure_text(&Text::new("HH\nH", font, 100.0))
            .expect("known font");
        assert_eq!(two.lines, 2);
        assert!((two.width - one.width).abs() < 1e-3, "the widest line");
        assert!((two.height - 290.0).abs() < 1e-3);
    }

    #[test]
    fn camera_changes_persist_and_viewport_is_reported() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        state.viewport = Vec2::new(800.0, 600.0);
        {
            let mut ctx = Context::new(&mut state, 0.0);
            assert_eq!(ctx.camera(), &Camera2D::IDENTITY);
            assert_eq!(ctx.viewport_size(), Vec2::new(800.0, 600.0));
            ctx.camera_mut().position = Vec2::new(5.0, 6.0);
            ctx.camera_mut().zoom = 2.0;
        }
        assert_eq!(state.camera, Camera2D::new(Vec2::new(5.0, 6.0), 2.0));
    }

    #[test]
    fn input_is_visible_through_the_context() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        state.input.key_down(crate::input::KeyCode::Space);
        let ctx = Context::new(&mut state, 0.0);
        assert!(ctx.input().pressed(crate::input::KeyCode::Space));
        assert!(ctx.input().just_pressed(crate::input::KeyCode::Space));
    }

    #[test]
    fn cursor_world_goes_through_the_camera() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        state.viewport = Vec2::new(800.0, 600.0);
        state.camera = Camera2D::new(Vec2::new(100.0, 50.0), 2.0);
        assert_eq!(
            Context::new(&mut state, 0.0).cursor_world(),
            None,
            "no cursor yet"
        );
        state.input.set_cursor(Some(Vec2::new(420.0, 280.0)));
        let ctx = Context::new(&mut state, 0.0);
        // 20 px right / 20 px up of the centre at zoom 2 = 10 world units each way.
        assert_eq!(ctx.cursor_world(), Some(Vec2::new(110.0, 60.0)));
    }

    #[test]
    fn relative_texture_paths_load_from_the_asset_root() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let mut ctx = Context::new(&mut state, 0.0);
        let relative = ctx
            .load_texture("textures/sandbox_quadrants.png")
            .expect("relative to the root");
        let absolute = ctx
            .load_texture(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/textures/sandbox_quadrants.png"),
            )
            .expect("absolute");
        assert_eq!(relative, absolute, "same file, same texture");
        assert!(
            ctx.asset_root()
                .is_some_and(|root| root.ends_with("assets"))
        );
    }

    #[test]
    fn without_an_asset_root_relative_loads_fail_but_absolute_ones_work() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::NotFound {
                searched: vec![PathBuf::from("nowhere/assets")],
            },
        );
        let mut ctx = Context::new(&mut state, 0.0);
        let err = ctx
            .load_texture("textures/sandbox_quadrants.png")
            .expect_err("no root");
        assert!(matches!(err, crate::Error::Asset { .. }));
        assert_eq!(ctx.asset_root(), None);
        ctx.load_texture(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/textures/sandbox_quadrants.png"
        ))
        .expect("absolute paths need no root");
    }

    #[test]
    fn window_title_requests_are_kept_until_applied() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::NotFound {
                searched: Vec::new(),
            },
        );
        {
            let mut ctx = Context::new(&mut state, 0.0);
            ctx.set_window_title("first");
            ctx.set_window_title(String::from("Score: 10"));
        }
        assert_eq!(state.window_title.take().as_deref(), Some("Score: 10"));
        assert_eq!(state.window_title, None);
    }

    #[test]
    fn default_callbacks_do_nothing() {
        struct Minimal;
        impl Game for Minimal {}
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let mut ctx = Context::new(&mut state, 0.0);
        let mut game = Minimal;
        assert!(game.init(&mut ctx).is_ok());
        game.fixed_update(&mut ctx);
        game.update(&mut ctx);
        assert!(!state.exit_requested);
    }
}
