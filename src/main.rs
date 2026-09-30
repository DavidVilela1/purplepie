//! PurplePie sandbox.
//!
//! This binary plays the role of a *game*: it may only use the public
//! `purplepie` API, exactly like an external game crate would.
//!
//! Environment variables:
//! - `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES=N`: request exit after N frames
//!   (used for automated smoke runs).

use std::error::Error as _;
use std::process::ExitCode;

use purplepie::{Context, Engine, EngineConfig, Game};

const EXIT_AFTER_FRAMES_VAR: &str = "PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES";

struct Sandbox {
    exit_after_frames: Option<u64>,
    /// Simulated seconds, advanced only by fixed steps.
    simulated_seconds: f64,
}

impl Game for Sandbox {
    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        self.simulated_seconds += f64::from(ctx.dt());
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        let time = ctx.time();
        if Some(time.frame()) == self.exit_after_frames {
            println!(
                "sandbox: {} frames, {} fixed steps, {:.3} s game time, {:.3} s simulated; requesting exit",
                time.frame(),
                time.fixed_steps(),
                time.elapsed(),
                self.simulated_seconds,
            );
            ctx.request_exit();
        }
    }
}

fn main() -> ExitCode {
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
