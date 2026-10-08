//! PurplePie sandbox.
//!
//! This binary plays the role of a *game*: it may only use the public
//! `purplepie` API, exactly like an external game crate would.
//!
//! Controls: arrow keys pan the camera, `=` / `-` or the mouse wheel zoom in / out,
//! `M` starts / stops a music loop,
//! left click stamps a square at the cursor (with a blip sound), Escape quits. A green marker follows the cursor.
//! A text label at the bottom left lists these controls; four sprite-sheet cells
//! (one mirrored) and one animated cell sit at the bottom right. A screen-space
//! HUD panel (top-left) and square (bottom-right corner) ignore the camera; a
//! "Reset camera" button (top-right) resets pan and zoom (clicks on it do not stamp).
//! Asset hot reload is on: saving a changed texture, font or sound under
//! `assets/` takes effect while the sandbox runs.
//!
//! Environment variables:
//! - `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES=N`: request exit after N frames
//!   (used for automated smoke runs).
//! - `PURPLEPIE_SANDBOX_CAMERA=x,y,zoom`: start with this camera instead of
//!   the default view (used by smoke tests to check panning and zooming).
//! - `PURPLEPIE_LOG=off|error|warn|info|debug|trace`: log level for messages
//!   from PurplePie, wgpu and other crates using the `log` facade (default `warn`).

use std::error::Error as _;
use std::process::ExitCode;

use purplepie::ecs::{self, Entity, Velocity};
use purplepie::input::{KeyCode, MouseButton};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{
    self, Camera2D, Color, Layer, Quad, ScreenSpace, Sprite, SpriteAnimation, SpriteGrid, Text,
    TextAnchor,
};
use purplepie::ui::{self, Button, Pointer};
use purplepie::{Context, Engine, EngineConfig, Game};

const EXIT_AFTER_FRAMES_VAR: &str = "PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES";
const LOG_LEVEL_VAR: &str = "PURPLEPIE_LOG";
const CAMERA_VAR: &str = "PURPLEPIE_SANDBOX_CAMERA";

/// Minimal `log` backend that prints to stderr. The engine never installs a
/// logger (ADR-016). Choosing one is the game's job, and this is the sandbox's choice.
struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("[{} {}] {}", record.level(), record.target(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: StderrLogger = StderrLogger;

/// Installs [`StderrLogger`] with the level from `PURPLEPIE_LOG` (default `warn`).
fn init_logging() -> Result<(), String> {
    let level = match std::env::var(LOG_LEVEL_VAR) {
        Ok(value) => value.parse::<log::LevelFilter>().map_err(|_| {
            format!("{LOG_LEVEL_VAR} must be off|error|warn|info|debug|trace, got {value:?}")
        })?,
        Err(_) => log::LevelFilter::Warn,
    };
    log::set_logger(&LOGGER).map_err(|e| e.to_string())?;
    log::set_max_level(level);
    Ok(())
}

/// Parses `x,y,zoom` (e.g. `0,120,2`) into a camera.
fn parse_camera(value: &str) -> Result<Camera2D, String> {
    let parts: Vec<f32> = value
        .split(',')
        .map(|p| p.trim().parse::<f32>())
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{CAMERA_VAR}: {e}"))?;
    match parts[..] {
        [x, y, zoom] if zoom.is_finite() && zoom > 0.0 => Ok(Camera2D::new(Vec2::new(x, y), zoom)),
        _ => Err(format!(
            "{CAMERA_VAR} must be x,y,zoom with zoom > 0, got {value:?}"
        )),
    }
}

/// The moving test entity: 120 world units (logical pixels) per second along +X.
const MOVER_VELOCITY: Vec2 = Vec2::new(120.0, 0.0);
/// The mover bounces between `-MOVER_LIMIT` and `+MOVER_LIMIT` on the X axis.
const MOVER_LIMIT: f32 = 500.0;
/// The sandbox's test image (16×16 texels), relative to the asset root
/// (ADR-025): `assets/` next to the executable, else `assets/` in the working
/// directory, which is the project folder under `cargo run`.
const SPRITE_TEXTURE: &str = "textures/sandbox_quadrants.png";
/// A 32×16 sprite sheet: 4 × 2 cells of 8×8 texels, each with its own border
/// colour, inner colour and a white texel in its top-left corner (PP-019).
const SHEET_TEXTURE: &str = "textures/sandbox_sheet.png";
/// Sheet cells shown in the sandbox: (frame, centre x, mirrored), all at y = −250, 32×32 world units.
const SHEET_CELLS: [(u32, f32, bool); 4] = [
    (0, 400.0, false),
    (5, 440.0, false),
    (6, 480.0, true),
    (3, 520.0, false),
];
/// Played when a click stamps a square (ADR-030).
const CLICK_SOUND: &str = "sounds/blip.wav";
/// Looped while music is on (`M`, ADR-032): OGG Vorbis (ADR-033).
const MUSIC_LOOP: &str = "sounds/loop.ogg";
/// The reset button (ADR-031): idle, hovered and pressed colours.
const BUTTON_COLORS: [u32; 3] = [0x3A86FF, 0x6FA8FF, 0x1D5FCC];
/// The animated cell (ADR-028): all 8 sheet frames in a loop, at (580, −250).
const ANIMATED_FPS: f32 = 4.0;
/// The font shipped with PurplePie (SIL Open Font License, `assets/fonts/OFL.txt`).
const FONT: &str = "fonts/Poppins-Regular.ttf";
/// The help label: its baseline starts here (world units) and its size is the em size.
const LABEL_POSITION: Vec2 = Vec2::new(-600.0, -300.0);
const LABEL_SIZE: f32 = 20.0;
const LABEL: &str = "PurplePie sandbox: arrows pan, = / - or wheel zoom, click stamps, Esc quits";
/// Camera pan speed with the arrow keys, in logical pixels per second (so it
/// feels the same at any zoom).
const PAN_SPEED: f32 = 300.0;
/// Zoom limits for the `=` / `-` keys (each press doubles or halves the zoom).
const ZOOM_RANGE: (f32, f32) = (0.125, 8.0);
/// Size of the green cursor marker and of the cyan click stamps, in world units.
const MARKER_SIZE: f32 = 10.0;
const STAMP_SIZE: f32 = 16.0;
/// Turn rate of the spinning sprite, in radians per second.
const SPIN_SPEED: f32 = 1.0;

struct Sandbox {
    exit_after_frames: Option<u64>,
    /// Simulated seconds, advanced only by fixed steps.
    simulated_seconds: f64,
    mover: Option<Entity>,
    spinner: Option<Entity>,
    /// The animated sprite-sheet cell.
    animated: Option<Entity>,
    /// The "Reset camera" button (its quad shows the state).
    reset_button: Option<Entity>,
    /// Clicks on the reset button.
    reset_clicks: u32,
    /// The music loop, and its playback while it plays.
    music: Option<purplepie::audio::SoundId>,
    music_playback: Option<purplepie::audio::PlaybackId>,
    /// Played on every stamp.
    click_sound: Option<purplepie::audio::SoundId>,
    camera: Camera2D,
    /// `=` presses seen by `fixed_update` and by `update` (each press must count once in each).
    zoom_in_presses: (u32, u32),
    /// The quad that follows the cursor.
    marker: Option<Entity>,
    /// Left clicks seen by `fixed_update` (each spawns a stamp) and by `update`.
    left_clicks: (u32, u32),
}

impl Game for Sandbox {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        *ctx.camera_mut() = self.camera;
        let texture = ctx.load_texture(SPRITE_TEXTURE)?;
        let font = ctx.load_font(FONT)?;
        let sheet = ctx.load_texture(SHEET_TEXTURE)?;
        self.click_sound = Some(ctx.load_sound(CLICK_SOUND)?);
        self.music = Some(ctx.load_sound(MUSIC_LOOP)?);
        let world = ctx.world_mut();
        // Sprite sheet cells (regions of one texture, one draw call); one mirrored by a negative scale.
        let grid = SpriteGrid::new(8, 8, 4, 2);
        for (frame, x, mirrored) in SHEET_CELLS {
            if let Some(region) = grid.frame(frame) {
                let scale = Vec2::new(if mirrored { -1.0 } else { 1.0 }, 1.0);
                world.spawn((
                    Transform2D::from_position(Vec2::new(x, -250.0)).with_scale(scale),
                    Sprite::new(sheet, region.size() * 4.0).with_region(region),
                ));
            }
        }
        // Screen space (ADR-029): window pixels from an anchor, unaffected by the camera.
        world.spawn((
            Transform2D::from_position(Vec2::new(140.0, -30.0)),
            Quad::new(Vec2::new(260.0, 40.0), Color::hex(0x1D3557)),
            ScreenSpace::TOP_LEFT,
        ));
        world.spawn((
            Transform2D::from_position(Vec2::new(22.0, -30.0)),
            Text::new("Screen-space HUD", font, 20.0)
                .with_color(Color::hex(0xF1FAEE))
                .with_anchor(TextAnchor::CENTER_LEFT),
            ScreenSpace::TOP_LEFT,
            Layer(1),
        ));
        world.spawn((
            Transform2D::from_position(Vec2::new(-30.0, 30.0)),
            Quad::new(Vec2::splat(40.0), Color::hex(0xFF006E)),
            ScreenSpace::BOTTOM_RIGHT,
        ));
        // A UI button (ADR-031): 160×40, 20 px from the top-right corner.
        self.reset_button = Some(world.spawn((
            Transform2D::from_position(Vec2::new(-100.0, -40.0)),
            ScreenSpace::TOP_RIGHT,
            Button::new(Vec2::new(160.0, 40.0)),
            Quad::new(Vec2::new(160.0, 40.0), Color::hex(BUTTON_COLORS[0])),
        )));
        world.spawn((
            Transform2D::from_position(Vec2::new(-100.0, -40.0)),
            ScreenSpace::TOP_RIGHT,
            Text::new("Reset camera", font, 18.0)
                .with_color(Color::hex(0xF1FAEE))
                .with_anchor(TextAnchor::CENTER),
            Layer(1),
        ));
        self.animated = Some(world.spawn((
            Transform2D::from_position(Vec2::new(580.0, -250.0)),
            Sprite::new(sheet, Vec2::splat(32.0)),
            SpriteAnimation::new(grid, 0, grid.len() - 1, ANIMATED_FPS),
        )));
        // Text (ADR-027): a help label in the bottom-left corner of the default view.
        world.spawn((
            Transform2D::from_position(LABEL_POSITION),
            Text::new(LABEL, font, LABEL_SIZE).with_color(Color::hex(0xF1FAEE)),
        ));
        // Static reference shapes; their screen positions are checked by smoke tests.
        world.spawn((
            Transform2D::from_position(Vec2::new(-300.0, 200.0)),
            Quad::new(Vec2::new(200.0, 100.0), Color::hex(0xFFB000)),
        ));
        world.spawn((
            Transform2D::from_position(Vec2::new(300.0, 200.0))
                .with_rotation(std::f32::consts::FRAC_PI_4),
            Quad::new(Vec2::new(100.0, 100.0), Color::hex(0x00C2A8)),
        ));
        let mover = world.spawn((
            Transform2D::from_position(Vec2::new(0.0, -150.0)),
            Velocity(MOVER_VELOCITY),
            Quad::new(Vec2::new(80.0, 80.0), Color::WHITE),
        ));
        self.mover = Some(mover);
        // Static reference sprite: each texel covers exactly 8×8 logical pixels.
        world.spawn((
            Transform2D::from_position(Vec2::new(0.0, 120.0)),
            Sprite::new(texture, Vec2::new(128.0, 128.0)),
        ));
        // Draw-order references (ADR-021), overlapping two corners of that sprite:
        // yellow, layer 0: under the sprite (quads before sprites within a layer),
        // so it only shows through the sprite's transparent border;
        world.spawn((
            Transform2D::from_position(Vec2::new(-64.0, 184.0)),
            Quad::new(Vec2::new(48.0, 48.0), Color::hex(0xFFE600)),
        ));
        // pink, layer 1: always on top of the layer-0 sprite.
        world.spawn((
            Transform2D::from_position(Vec2::new(64.0, 56.0)),
            Quad::new(Vec2::new(48.0, 48.0), Color::hex(0xFF3EA5)),
            Layer(1),
        ));
        // The same texture, tinted half-transparent cyan and spinning.
        let spinner = world.spawn((
            Transform2D::from_position(Vec2::new(-300.0, -20.0)),
            Sprite::new(texture, Vec2::new(96.0, 96.0)).with_tint(Color::rgba(0.5, 1.0, 1.0, 0.6)),
        ));
        self.spinner = Some(spinner);
        // Follows the cursor (positioned in `update`); zero scale hides it while
        // the cursor is outside the window.
        let marker = world.spawn((
            Transform2D::default().with_scale(Vec2::ZERO),
            Quad::new(Vec2::splat(MARKER_SIZE), Color::hex(0x39FF14)),
            Layer(4),
        ));
        self.marker = Some(marker);
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        self.simulated_seconds += f64::from(ctx.dt());
        let dt = ctx.dt();
        ecs::integrate_velocity(ctx.world_mut(), dt);
        render::advance_animations(ctx.world_mut(), dt);
        // Bounce the mover between the limits (game logic, not an engine feature).
        for (transform, velocity) in ctx.world_mut().query_mut::<(&Transform2D, &mut Velocity)>() {
            let x = transform.position.x;
            if (x > MOVER_LIMIT && velocity.0.x > 0.0) || (x < -MOVER_LIMIT && velocity.0.x < 0.0) {
                velocity.0.x = -velocity.0.x;
            }
        }
        if let Some(mut transform) = self
            .spinner
            .and_then(|e| ctx.world_mut().get::<&mut Transform2D>(e).ok())
        {
            transform.rotation += SPIN_SPEED * dt;
        }

        // Controls: arrow keys pan (while held), `=` / `-` zoom (once per press).
        let input = ctx.input();
        let pan = Vec2::new(
            input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight),
            input.axis(KeyCode::ArrowDown, KeyCode::ArrowUp),
        );
        let toggle_music = input.just_pressed(KeyCode::M);
        let zoom_in = input.just_pressed(KeyCode::Equal);
        let zoom_out = input.just_pressed(KeyCode::Minus);
        let wheel = input.scroll().y;
        let click = input.mouse_just_pressed(MouseButton::Left);
        let cursor = ctx.cursor_world();
        self.zoom_in_presses.0 += u32::from(zoom_in);
        let camera = ctx.camera_mut();
        camera.position += pan * PAN_SPEED * dt / camera.effective_zoom();
        if zoom_in {
            camera.zoom = (camera.effective_zoom() * 2.0).min(ZOOM_RANGE.1);
        }
        if zoom_out {
            camera.zoom = (camera.effective_zoom() / 2.0).max(ZOOM_RANGE.0);
        }
        if wheel != 0.0 {
            // One wheel notch doubles or halves the zoom, like `=` / `-`.
            camera.zoom =
                (camera.effective_zoom() * 2.0_f32.powf(wheel)).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        }
        // Left click: stamp a cyan square at the cursor (world coordinates, read
        // before this step's camera change).
        // M: start or stop the music loop (ADR-032).
        if toggle_music {
            match self.music_playback.take() {
                Some(playback) => ctx.stop_sound(playback),
                None => self.music_playback = self.music.map(|m| ctx.loop_sound(m, 0.5)),
            }
        }
        // Clicks on the UI button are the button's, not the world's.
        let over_button = self
            .reset_button
            .and_then(|e| ctx.world().get::<&Button>(e).ok().map(|b| b.is_hovered()))
            .unwrap_or(false);
        if click {
            self.left_clicks.0 += 1;
            if let Some(position) = cursor.filter(|_| !over_button) {
                ctx.world_mut().spawn((
                    Transform2D::from_position(position),
                    Quad::new(Vec2::splat(STAMP_SIZE), Color::hex(0x00E0FF)),
                    Layer(3),
                ));
                if let Some(sound) = self.click_sound {
                    ctx.play_sound(sound, 0.8);
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        // UI first (ADR-031): copy the mouse state, then update the buttons.
        let pointer = Pointer::from_input(ctx.input());
        let viewport = ctx.viewport_size();
        ui::update_buttons(ctx.world_mut(), pointer, viewport);
        if let Some(entity) = self.reset_button {
            let state = ctx.world().get::<&Button>(entity).ok().map(|b| *b);
            if let Some(button) = state {
                let color = if button.is_pressed() {
                    BUTTON_COLORS[2]
                } else if button.is_hovered() {
                    BUTTON_COLORS[1]
                } else {
                    BUTTON_COLORS[0]
                };
                if let Ok(mut quad) = ctx.world_mut().get::<&mut Quad>(entity) {
                    quad.color = Color::hex(color);
                }
                if button.clicked() {
                    self.reset_clicks += 1;
                    *ctx.camera_mut() = Camera2D::default();
                }
            }
        }
        self.zoom_in_presses.1 += u32::from(ctx.input().just_pressed(KeyCode::Equal));
        self.left_clicks.1 += u32::from(ctx.input().mouse_just_pressed(MouseButton::Left));
        // The marker follows the cursor every frame (hidden outside the window).
        let cursor = ctx.cursor_world();
        if let Some(mut marker) = self
            .marker
            .and_then(|e| ctx.world_mut().get::<&mut Transform2D>(e).ok())
        {
            match cursor {
                Some(position) => {
                    marker.position = position;
                    marker.scale = Vec2::ONE;
                }
                None => marker.scale = Vec2::ZERO,
            }
        }
        let time = ctx.time();
        if Some(time.frame()) == self.exit_after_frames {
            println!(
                "sandbox: {} frames, {} fixed steps, {:.3} s game time, {:.3} s simulated",
                time.frame(),
                time.fixed_steps(),
                time.elapsed(),
                self.simulated_seconds,
            );
            let position = self
                .mover
                .and_then(|e| ctx.world().get::<&Transform2D>(e).ok().map(|t| t.position));
            match position {
                Some(p) => println!(
                    "sandbox: mover at ({:.4}, {:.4}); requesting exit",
                    p.x, p.y
                ),
                None => println!("sandbox: mover missing; requesting exit"),
            }
            let viewport = ctx.viewport_size();
            let camera = ctx.camera();
            println!(
                "sandbox: viewport {}x{} logical px, camera at ({}, {}) zoom {}",
                viewport.x, viewport.y, camera.position.x, camera.position.y, camera.zoom
            );
            println!(
                "sandbox: '=' presses seen: {} in fixed_update, {} in update",
                self.zoom_in_presses.0, self.zoom_in_presses.1
            );
            let frame = self.animated.and_then(|e| {
                ctx.world()
                    .get::<&SpriteAnimation>(e)
                    .ok()
                    .map(|a| a.frame())
            });
            println!("sandbox: animated cell shows frame {frame:?}");
            println!("sandbox: audio output available: {}", ctx.audio_available());
            println!("sandbox: reset button clicks: {}", self.reset_clicks);
            println!("sandbox: music playing: {}", self.music_playback.is_some());
            let cursor = ctx.input().cursor_position();
            println!(
                "sandbox: left clicks seen: {} in fixed_update, {} in update; cursor {:?} screen, {:?} world",
                self.left_clicks.0,
                self.left_clicks.1,
                cursor,
                ctx.cursor_world()
            );
            ctx.request_exit();
        }
    }
}

fn main() -> ExitCode {
    if let Err(message) = init_logging() {
        eprintln!("error: {message}");
        return ExitCode::FAILURE;
    }
    let exit_after_frames = match std::env::var(EXIT_AFTER_FRAMES_VAR) {
        Ok(value) => match value.parse::<u64>() {
            Ok(frames) if frames > 0 => Some(frames),
            _ => {
                eprintln!(
                    "error: {EXIT_AFTER_FRAMES_VAR} must be a positive integer, got {value:?}"
                );
                return ExitCode::FAILURE;
            }
        },
        Err(_) => None,
    };

    let camera = match std::env::var(CAMERA_VAR) {
        Ok(value) => match parse_camera(&value) {
            Ok(camera) => camera,
            Err(message) => {
                eprintln!("error: {message}");
                return ExitCode::FAILURE;
            }
        },
        Err(_) => Camera2D::default(),
    };

    println!("PurplePie sandbox v{}", purplepie::VERSION);
    let game = Sandbox {
        exit_after_frames,
        simulated_seconds: 0.0,
        mover: None,
        spinner: None,
        animated: None,
        click_sound: None,
        music: None,
        music_playback: None,
        reset_button: None,
        reset_clicks: 0,
        camera,
        zoom_in_presses: (0, 0),
        marker: None,
        left_clicks: (0, 0),
    };
    // Hot reload (ADR-037): edit a texture, font or sound under assets/ while
    // the sandbox runs and the change shows up within about half a second.
    let config = EngineConfig::new("PurplePie Sandbox").with_hot_reload(true);
    let result = Engine::new(config).and_then(|e| e.run(game));

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            let mut source = error.source();
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }
            ExitCode::FAILURE
        }
    }
}
