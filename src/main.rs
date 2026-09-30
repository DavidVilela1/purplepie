//! PurplePie sandbox.
//!
//! This binary plays the role of a *game*: it may only use the public
//! `purplepie` API, exactly like an external game crate would.
//!
//! Environment variables:
//! - `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES=N`: request exit after N frames
//!   (used for automated smoke runs).
//! - `PURPLEPIE_LOG=off|error|warn|info|debug|trace`: log level for messages
//!   from PurplePie, wgpu and other crates using the `log` facade (default `warn`).

use std::error::Error as _;
use std::process::ExitCode;

use purplepie::ecs::{self, Entity, Velocity};
use purplepie::math::{Transform2D, Vec2};
use purplepie::{Context, Engine, EngineConfig, Game};

const EXIT_AFTER_FRAMES_VAR: &str = "PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES";
const LOG_LEVEL_VAR: &str = "PURPLEPIE_LOG";

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

/// The sandbox's moving test entity: 1 world unit per second along +X.
const MOVER_VELOCITY: Vec2 = Vec2::new(1.0, 0.0);

struct Sandbox {
    exit_after_frames: Option<u64>,
    /// Simulated seconds, advanced only by fixed steps.
    simulated_seconds: f64,
    mover: Option<Entity>,
}

impl Game for Sandbox {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        let mover = ctx
            .world_mut()
            .spawn((Transform2D::default(), Velocity(MOVER_VELOCITY)));
        self.mover = Some(mover);
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        self.simulated_seconds += f64::from(ctx.dt());
        let dt = ctx.dt();
        ecs::integrate_velocity(ctx.world_mut(), dt);
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

    println!("PurplePie sandbox v{}", purplepie::VERSION);
    let game = Sandbox {
        exit_after_frames,
        simulated_seconds: 0.0,
        mover: None,
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
