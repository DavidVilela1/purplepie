//! The game-facing callback trait and the per-call context (ADR-008).

use std::path::Path;

use super::state::EngineState;
use crate::audio::{PlaybackId, SoundId};
use crate::ecs::{Entity, World};
use crate::error::Result;
use crate::input::Input;
use crate::math::Vec2;
use crate::render::{Camera2D, FontId, Text, TextMetrics, TextureId, TextureOptions};
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
            .field("sounds", &self.state.sounds.len())
            .field("audio", &self.state.audio)
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
    ///
    /// The texture is sampled with [`TextureFilter::Nearest`](crate::render::TextureFilter::Nearest)
    /// (crisp pixel art); use [`load_texture_with`](Self::load_texture_with)
    /// for smooth, linear sampling.
    pub fn load_texture(&mut self, path: impl AsRef<Path>) -> Result<TextureId> {
        self.load_texture_with(path, TextureOptions::NEAREST)
    }

    /// Like [`load_texture`](Self::load_texture), with `options` choosing how
    /// the texture is sampled (ADR-034): [`TextureOptions::LINEAR`] for smooth
    /// sprites that are scaled, rotated or moved by fractions of a pixel,
    /// [`TextureOptions::NEAREST`] for pixel art.
    ///
    /// Loading the same file again with the same options returns the same
    /// [`TextureId`]; with other options it is a separate texture. Sprite-sheet
    /// cells sampled with `Linear` blend with the texels around them, so leave
    /// [`spacing`](crate::render::SpriteGrid::spacing) between cells (or repeat
    /// each cell's edge texels) when a sheet uses it.
    ///
    /// ```no_run
    /// use purplepie::render::TextureOptions;
    /// # fn init(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
    /// let ball = ctx.load_texture_with("textures/ball.png", TextureOptions::LINEAR)?;
    /// # let _ = ball;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_texture_with(
        &mut self,
        path: impl AsRef<Path>,
        options: TextureOptions,
    ) -> Result<TextureId> {
        let file = self.state.assets.locate(path.as_ref())?;
        self.state.textures.load(&file, options)
    }

    /// Saves the engine's components of every entity that has a
    /// [`Quad`](crate::render::Quad), [`Sprite`](crate::render::Sprite),
    /// [`Text`](crate::render::Text),
    /// [`SpriteAnimation`](crate::render::SpriteAnimation) or
    /// [`Button`](crate::ui::Button) to a scene file (ADR-035): their
    /// `Transform2D`, `Quad`, `Sprite` (texture by asset path, filter, tint,
    /// region), `Text` (font by asset path), `Layer`, `Hidden`, `ScreenSpace`,
    /// `SpriteAnimation` (including how far it has played), `Velocity` and
    /// `Button` (its size; hover and click state start fresh), plus every
    /// game component registered with
    /// [`register_scene_component`](Self::register_scene_component). Entities
    /// with none of the five engine components above and no registered
    /// component are skipped.
    ///
    /// The file is human-readable [RON](https://docs.rs/ron) text. A relative
    /// `path` is resolved against the asset root like
    /// [`load_texture`](Self::load_texture) (e.g. `"scenes/level1.ron"`); its
    /// folder must exist. Asset references are written relative to the asset
    /// root, so the scene keeps working when the game folder moves. Failures
    /// are [`Error::Save`](crate::Error::Save).
    pub fn save_scene(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let file = self
            .state
            .assets
            .locate(path)
            .map_err(|e| crate::Error::Save {
                path: path.to_path_buf(),
                source: Box::new(e),
            })?;
        // Report the resolved file, so a missing folder shows where (ADR-040).
        let save_error = |source: crate::error::BoxError| crate::Error::Save {
            path: file.clone(),
            source,
        };
        let state = &*self.state;
        let scene = crate::scene::capture(
            &state.world,
            &state.textures,
            &state.fonts,
            state.assets.path(),
            &state.scene_components,
        )
        .map_err(save_error)?;
        let text = crate::scene::to_text(&scene).map_err(save_error)?;
        std::fs::write(&file, text).map_err(|e| save_error(Box::new(e)))?;
        log::debug!(
            "saved {} entities to scene {}",
            scene.entities.len(),
            file.display()
        );
        Ok(())
    }

    /// Lets scene files save and load the game's own component `T` under
    /// `name` (ADR-036). Call it before [`save_scene`](Self::save_scene) or
    /// [`load_scene`](Self::load_scene), usually in [`Game::init`](crate::Game::init).
    ///
    /// `T` is written and read with its `serde` implementation, so derive
    /// `Serialize` and `Deserialize` for it (add `serde = { version = "1",
    /// features = ["derive"] }` to the game's `Cargo.toml`). In the file it
    /// appears under `components: { "name": (...) }` in plain RON (options as
    /// `Some(…)`). Entities with a registered component are saved even without
    /// anything drawable. Choose a stable name (e.g. `"mygame::Health"`): it is
    /// what files refer to, so renaming the Rust type later does not break them.
    ///
    /// Registering the same type under the same name again does nothing. A
    /// name already used for another type, a type already registered under
    /// another name, or an empty name is
    /// [`Error::InvalidConfig`](crate::Error::InvalidConfig). Components that
    /// hold an [`Entity`] are not supported (entity ids change between runs).
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Health {
    ///     current: u32,
    ///     max: u32,
    /// }
    ///
    /// # fn init(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
    /// ctx.register_scene_component::<Health>("mygame::Health")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn register_scene_component<T>(&mut self, name: &str) -> Result<()>
    where
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned,
    {
        self.state
            .scene_components
            .register::<T>(name)
            .map_err(crate::Error::InvalidConfig)
    }

    /// Loads a scene file written by [`save_scene`](Self::save_scene) (or by
    /// hand) and spawns its entities into the world, after any that are
    /// already there. Returns the new entities in file order.
    ///
    /// Every texture and font the scene names is loaded first (cached like
    /// [`load_texture`](Self::load_texture) and [`load_font`](Self::load_font));
    /// if one fails, the file is not a valid scene of a supported version, or it
    /// holds a game component that is not
    /// [registered](Self::register_scene_component) (or does not decode), the
    /// error is [`Error::Asset`](crate::Error::Asset) and nothing is spawned.
    ///
    /// ```no_run
    /// # fn init(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
    /// let entities = ctx.load_scene("scenes/level1.ron")?;
    /// println!("spawned {} entities", entities.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_scene(&mut self, path: impl AsRef<Path>) -> Result<Vec<Entity>> {
        let path = path.as_ref();
        let file = self.state.assets.locate(path)?;
        let asset_error = |source: crate::error::BoxError| crate::Error::Asset {
            path: path.to_path_buf(),
            source,
        };
        let text = std::fs::read_to_string(&file).map_err(|e| asset_error(Box::new(e)))?;
        let scene = crate::scene::from_text(&text).map_err(asset_error)?;
        let state = &mut *self.state;
        let components = crate::scene::decode_components(&scene, &state.scene_components)
            .map_err(asset_error)?;
        let (assets, textures, fonts) = (&state.assets, &mut state.textures, &mut state.fonts);
        let resolved = crate::scene::resolve_assets(
            &scene,
            |p, options| textures.load(&assets.locate(p)?, options),
            |p| fonts.load(&assets.locate(p)?),
        )?;
        let entities = crate::scene::spawn(scene, resolved, components, &mut state.world);
        log::debug!(
            "loaded {} entities from scene {}",
            entities.len(),
            file.display()
        );
        Ok(entities)
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

    /// Loads a sound and returns a handle for [`play_sound`](Self::play_sound)
    /// and [`loop_sound`](Self::loop_sound) (ADR-030). Formats: **WAV** (PCM
    /// 8/16/24/32-bit or 32-bit float) and **OGG Vorbis** (ADR-033), mono or
    /// stereo, any sample rate. The format is recognised from the file's
    /// content, not its extension.
    ///
    /// The whole file is read and decoded now (OGG too: about 10 MB of memory
    /// per minute of 44.1 kHz mono, twice that for stereo, and a long track
    /// takes a noticeable moment, so load music at startup), and a problem is
    /// reported here as
    /// [`Error::Asset`](crate::Error::Asset). Paths resolve like
    /// [`load_texture`](Self::load_texture); loading the same file again
    /// returns the same [`SoundId`]. Works without an audio device.
    pub fn load_sound(&mut self, path: impl AsRef<Path>) -> Result<SoundId> {
        let file = self.state.assets.locate(path.as_ref())?;
        self.state.sounds.load(&file)
    }

    /// Starts playing `sound` from the beginning at `volume` (1.0 = as
    /// recorded, 0 = silent; at most 4.0; invalid values are silent).
    /// Sounds overlap freely, up to 32 at once (the oldest stops first).
    /// Returns immediately; without an audio device it does nothing.
    ///
    /// The returned [`PlaybackId`] can stop it ([`stop_sound`](Self::stop_sound))
    /// or change its volume ([`set_sound_volume`](Self::set_sound_volume));
    /// ignore it for fire-and-forget effects. An unknown `sound` plays nothing.
    pub fn play_sound(&mut self, sound: SoundId, volume: f32) -> PlaybackId {
        self.start_sound(sound, volume, false)
    }

    /// Plays `sound` over and over, seamlessly (music, ambience), until
    /// [`stop_sound`](Self::stop_sound) or [`stop_all_sounds`](Self::stop_all_sounds).
    /// Otherwise like [`play_sound`](Self::play_sound). Looping sounds are the
    /// last to be cut when more than 32 sounds play at once.
    pub fn loop_sound(&mut self, sound: SoundId, volume: f32) -> PlaybackId {
        self.start_sound(sound, volume, true)
    }

    fn start_sound(&mut self, sound: SoundId, volume: f32, looping: bool) -> PlaybackId {
        let data = self.state.sounds.get(sound).map(std::sync::Arc::clone);
        match data {
            Some(data) => self.state.audio.play(data, volume, looping),
            // An id from another engine run: hand out an id that plays nothing.
            None => self.state.audio.unused_id(),
        }
    }

    /// Stops a playback now. Does nothing if it already ended.
    pub fn stop_sound(&mut self, playback: PlaybackId) {
        self.state.audio.stop(playback);
    }

    /// Changes a playback's volume (same range as [`play_sound`](Self::play_sound)),
    /// immediately and without a fade.
    pub fn set_sound_volume(&mut self, playback: PlaybackId, volume: f32) {
        self.state.audio.set_volume(playback, volume);
    }

    /// Stops every playing sound, looping or not.
    pub fn stop_all_sounds(&mut self) {
        self.state.audio.stop_all();
    }

    /// Sets the volume applied on top of every sound's own volume (1.0 at
    /// start, 0 mutes everything, at most 4.0; invalid values mute).
    pub fn set_master_volume(&mut self, volume: f32) {
        self.state.audio.set_master_volume(volume);
    }

    /// The master volume (see [`set_master_volume`](Self::set_master_volume)).
    pub fn master_volume(&self) -> f32 {
        self.state.audio.master_volume()
    }

    /// Length of a loaded sound in seconds (at its own sample rate), or
    /// `None` for an id from another engine run.
    pub fn sound_duration(&self, sound: SoundId) -> Option<f32> {
        self.state
            .sounds
            .get(sound)
            .map(|s| s.frames() as f32 / s.sample_rate as f32)
    }

    /// `true` if sounds reach an audio device: `false` when audio is turned
    /// off in [`EngineConfig`](crate::EngineConfig) or no device could be opened.
    pub fn audio_available(&self) -> bool {
        self.state.audio.is_active()
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
        // Options are part of the identity (ADR-034): Nearest is the default.
        assert_eq!(
            ctx.load_texture_with(sandbox, TextureOptions::NEAREST)
                .expect("nearest"),
            id
        );
        let linear = ctx
            .load_texture_with("textures/sandbox_quadrants.png", TextureOptions::LINEAR)
            .expect("linear");
        assert_ne!(linear, id, "another filter is another texture");
        assert_eq!(ctx.texture_size(linear), Some(Vec2::new(16.0, 16.0)));
        let filter = |id| state_filter(&ctx, id);
        assert_eq!(
            (filter(id), filter(linear)),
            (
                crate::render::TextureFilter::Nearest,
                crate::render::TextureFilter::Linear
            )
        );
    }

    fn state_filter(ctx: &Context<'_>, id: TextureId) -> crate::render::TextureFilter {
        ctx.state.textures.get(id).expect("loaded").filter
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
    fn game_components_register_once_and_travel_through_scene_files() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Health(u32);
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Other;
        let root = std::env::temp_dir().join(format!("purplepie-scene-reg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("mkdir");
        let mut state = EngineState::new(0.25, AssetRoot::Found(root.clone()));
        let mut ctx = Context::new(&mut state, 0.0);
        ctx.register_scene_component::<Health>("test::Health")
            .expect("register");
        ctx.register_scene_component::<Health>("test::Health")
            .expect("again");
        let err = ctx
            .register_scene_component::<Other>("test::Health")
            .expect_err("clash");
        assert!(matches!(err, crate::Error::InvalidConfig(_)), "{err}");
        ctx.world_mut().spawn((Health(42),));
        ctx.save_scene("game.ron").expect("save");
        let text = std::fs::read_to_string(root.join("game.ron")).expect("written");
        assert!(text.contains("\"test::Health\": (42)"), "{text}");
        let spawned = ctx.load_scene("game.ron").expect("load");
        assert_eq!(
            ctx.world().get::<&Health>(spawned[0]).map(|h| h.0).ok(),
            Some(42)
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn scenes_save_and_load_relative_to_the_asset_root() {
        use crate::math::Transform2D;
        use crate::render::{Color, Quad, Sprite};
        // A scratch asset root holding one texture, so the scene is written there.
        let root = std::env::temp_dir().join(format!("purplepie-scene-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("textures")).expect("mkdir");
        std::fs::create_dir_all(root.join("scenes")).expect("mkdir");
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/textures/sandbox_quadrants.png"
            ),
            root.join("textures/q.png"),
        )
        .expect("copy texture");
        let mut state = EngineState::new(0.25, AssetRoot::Found(root.clone()));
        let mut ctx = Context::new(&mut state, 0.0);
        let texture = ctx
            .load_texture_with("textures/q.png", TextureOptions::LINEAR)
            .expect("texture");
        ctx.world_mut().spawn((
            Transform2D::from_position(Vec2::new(3.0, 4.0)),
            Sprite::new(texture, Vec2::splat(16.0)),
        ));
        ctx.world_mut().spawn((Quad::new(Vec2::ONE, Color::WHITE),));
        ctx.save_scene("scenes/a.ron").expect("save");
        let text = std::fs::read_to_string(root.join("scenes/a.ron")).expect("written");
        assert!(
            text.contains("\"textures/q.png\""),
            "relative reference: {text}"
        );
        assert!(text.contains("filter: Linear"), "{text}");

        let spawned = ctx.load_scene("scenes/a.ron").expect("load");
        assert_eq!(spawned.len(), 2);
        assert_eq!(ctx.world().len(), 4, "loading adds to the world");
        let sprite = *ctx.world().get::<&Sprite>(spawned[0]).expect("sprite");
        assert_eq!(
            sprite.texture, texture,
            "same file + options: the cached texture"
        );

        let err = ctx
            .save_scene("no-such-folder/a.ron")
            .expect_err("missing folder");
        assert!(
            matches!(&err, crate::Error::Save { path, .. } if path.ends_with("no-such-folder/a.ron")),
            "{err}"
        );
        assert!(err.to_string().starts_with("failed to save"), "{err}");
        let err = ctx
            .load_scene("scenes/missing.ron")
            .expect_err("missing file");
        assert!(matches!(err, crate::Error::Asset { .. }), "{err}");
        std::fs::write(root.join("scenes/bad.ron"), "(version: 99, entities: [])").expect("write");
        let err = ctx
            .load_scene("scenes/bad.ron")
            .expect_err("future version");
        let reason = std::error::Error::source(&err).expect("reason").to_string();
        assert!(reason.contains("version 99"), "{reason}");
        assert_eq!(ctx.world().len(), 4, "failed loads spawn nothing");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sounds_load_relative_to_the_asset_root_and_play_silently_without_a_device() {
        let mut state = EngineState::new(
            0.25,
            AssetRoot::Found(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        );
        let mut ctx = Context::new(&mut state, 0.0);
        let err = ctx.load_sound("sounds/no-such.wav").expect_err("missing");
        assert!(matches!(err, crate::Error::Asset { .. }), "{err}");
        let err = ctx
            .load_sound("textures/sandbox_quadrants.png")
            .expect_err("a PNG is not a sound");
        assert!(matches!(err, crate::Error::Asset { .. }), "{err}");
        let blip = ctx.load_sound("sounds/blip.wav").expect("blip");
        assert_eq!(ctx.load_sound("sounds/blip.wav").expect("again"), blip);
        assert!(!ctx.audio_available(), "tests never open a device");
        let a = ctx.play_sound(blip, 1.0); // silent, must not panic
        let b = ctx.loop_sound(blip, f32::NAN);
        assert_ne!(a, b, "every playback gets its own id");
        ctx.set_sound_volume(b, 0.5);
        ctx.stop_sound(a);
        ctx.stop_all_sounds();
        ctx.set_master_volume(0.25);
        assert_eq!(ctx.master_volume(), 0.25);
        ctx.set_master_volume(f32::NAN);
        assert_eq!(ctx.master_volume(), 0.0, "invalid volumes mute");
        // blip.wav: 1323 frames at 22,050 Hz = 60 ms.
        let duration = ctx.sound_duration(blip).expect("loaded");
        assert!((duration - 0.06).abs() < 1e-6, "{duration}");
        // OGG Vorbis loads the same way (ADR-033): loop.ogg is 22,050 frames = 1 s.
        let music = ctx.load_sound("sounds/loop.ogg").expect("loop.ogg");
        assert_ne!(music, blip);
        let duration = ctx.sound_duration(music).expect("loaded");
        assert!((duration - 1.0).abs() < 1e-6, "{duration}");
        let _ = ctx.loop_sound(music, 0.5);
        assert_eq!(state.sounds.len(), 2);
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
