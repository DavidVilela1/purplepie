//! PurplePie sandbox.
//!
//! This binary plays the role of a *game*: it may only use the public
//! `purplepie` API, exactly like an external game crate would.
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
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Camera2D, Color, Layer, Quad, Sprite};
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
/// The sandbox's test image (16×16 texels). The path is fixed at compile time,
/// so `cargo run` works from any directory.
const SPRITE_TEXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/textures/sandbox_quadrants.png"
);
/// Turn rate of the spinning sprite, in radians per second.
const SPIN_SPEED: f32 = 1.0;

struct Sandbox {
    exit_after_frames: Option<u64>,
    /// Simulated seconds, advanced only by fixed steps.
    simulated_seconds: f64,
    mover: Option<Entity>,
    spinner: Option<Entity>,
    camera: Camera2D,
}

impl Game for Sandbox {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        *ctx.camera_mut() = self.camera;
        let texture = ctx.load_texture(SPRITE_TEXTURE)?;
        let world = ctx.world_mut();
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
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        self.simulated_seconds += f64::from(ctx.dt());
        let dt = ctx.dt();
        ecs::integrate_velocity(ctx.world_mut(), dt);
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
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
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
        camera,
    };
    let result = Engine::new(EngineConfig::new("PurplePie Sandbox")).and_then(|e| e.run(game));

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
