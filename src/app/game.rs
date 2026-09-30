//! The game-facing callback trait and the per-call context (ADR-008).

use crate::error::Result;
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
/// `winit` or GPU types. More fields (world, input) are added by later stages.
#[derive(Debug)]
pub struct Context<'a> {
    exit_requested: &'a mut bool,
    time: &'a Time,
    dt: f64,
}

impl<'a> Context<'a> {
    pub(crate) fn new(exit_requested: &'a mut bool, time: &'a Time, dt: f64) -> Self {
        Self {
            exit_requested,
            time,
            dt,
        }
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
        let mut flag = false;
        let mut ctx = Context::new(&mut flag, &time, 0.0);
        assert!(!ctx.exit_requested());
        ctx.request_exit();
        assert!(ctx.exit_requested());
        assert!(flag);
    }

    #[test]
    fn exposes_time_and_callback_dt() {
        let time = Time::new(0.25);
        let mut flag = false;
        let ctx = Context::new(&mut flag, &time, 0.25);
        assert_eq!(ctx.dt(), 0.25_f32);
        assert_eq!(ctx.time().fixed_dt(), 0.25);
    }

    #[test]
    fn default_callbacks_do_nothing() {
        struct Minimal;
        impl Game for Minimal {}
        let time = Time::new(0.25);
        let mut flag = false;
        let mut ctx = Context::new(&mut flag, &time, 0.0);
        let mut game = Minimal;
        assert!(game.init(&mut ctx).is_ok());
        game.fixed_update(&mut ctx);
        game.update(&mut ctx);
        assert!(!flag);
    }
}
