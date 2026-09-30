//! The game-facing callback trait and the per-call context (ADR-008).

use crate::error::Result;

/// Implemented by a game. The engine owns the game value and calls these
/// methods from its main loop.
///
/// Stage 1 provides `init` and a per-frame `update`. `fixed_update` arrives
/// with the fixed timestep in Stage 2.
pub trait Game {
    /// Called once, after the window exists and before the first frame.
    ///
    /// Returning an error stops the engine, and [`Engine::run`](crate::Engine::run)
    /// returns that error.
    fn init(&mut self, ctx: &mut Context<'_>) -> Result<()> {
        let _ = ctx;
        Ok(())
    }

    /// Called once per rendered frame.
    fn update(&mut self, ctx: &mut Context<'_>);
}

/// What game code can see and do during a callback.
///
/// Built fresh for every callback from engine-owned state. It never exposes
/// `winit` or GPU types. More fields (time, world, input) are added by later stages.
#[derive(Debug)]
pub struct Context<'a> {
    exit_requested: &'a mut bool,
}

impl<'a> Context<'a> {
    pub(crate) fn new(exit_requested: &'a mut bool) -> Self {
        Self { exit_requested }
    }

    /// Asks the engine to shut down cleanly after the current callback.
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
        let mut flag = false;
        let mut ctx = Context::new(&mut flag);
        assert!(!ctx.exit_requested());
        ctx.request_exit();
        assert!(ctx.exit_requested());
        assert!(flag);
    }

    #[test]
    fn default_init_succeeds() {
        struct Minimal;
        impl Game for Minimal {
            fn update(&mut self, _ctx: &mut Context<'_>) {}
        }
        let mut flag = false;
        assert!(Minimal.init(&mut Context::new(&mut flag)).is_ok());
        assert!(!flag);
    }
}
