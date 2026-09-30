//! Application layer: the [`Engine`] entry point, its configuration, and the
//! [`Game`] trait. The only module that depends on `winit` (ADR-003).

mod config;
mod game;
mod pacer;
mod runner;

pub use config::EngineConfig;
pub use game::{Context, Game};

use winit::event_loop::EventLoop;

use crate::error::{Error, Result};
use runner::Runner;

/// The engine: owns the platform event loop and runs a [`Game`].
///
/// ```no_run
/// use purplepie::{Context, Engine, EngineConfig, Game};
///
/// struct MyGame;
/// impl Game for MyGame {
///     fn update(&mut self, _ctx: &mut Context<'_>) {}
/// }
///
/// fn main() -> purplepie::Result<()> {
///     Engine::new(EngineConfig::new("My Game"))?.run(MyGame)
/// }
/// ```
#[derive(Debug)]
pub struct Engine {
    config: EngineConfig,
    event_loop: EventLoop<()>,
}

impl Engine {
    /// Validates `config` and creates the platform event loop.
    ///
    /// Must be called on the main thread. The platform allows only one event
    /// loop per process, so a second `Engine` returns [`Error::EventLoop`].
    pub fn new(config: EngineConfig) -> Result<Self> {
        config.validate()?;
        let event_loop = EventLoop::new().map_err(|e| Error::EventLoop(Box::new(e)))?;
        Ok(Self { config, event_loop })
    }

    /// Opens the window and runs `game` until it exits: window closed, Escape
    /// pressed (if enabled), [`Context::request_exit`], or an error.
    ///
    /// Blocks the calling thread. Returns the first error raised by the game
    /// or the platform.
    pub fn run<G: Game>(self, game: G) -> Result<()> {
        let mut runner = Runner::new(self.config, game);
        let loop_result = self.event_loop.run_app(&mut runner);
        // A game or window error is the root cause, so report it before any
        // event-loop error that followed from it.
        runner.finish()?;
        loop_result.map_err(|e| Error::EventLoop(Box::new(e)))
    }
}
