//! The winit `ApplicationHandler` that drives a [`Game`] (winit 0.30 lifecycle).
//!
//! This is the only place in the engine that handles winit events.

use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use super::config::EngineConfig;
use super::game::{Context, Game};
use super::pacer::{FRAME_INTERVAL, FramePacer};
use crate::error::{Error, Result};

/// Owns the game and all engine state for the lifetime of the event loop.
pub(crate) struct Runner<G: Game> {
    config: EngineConfig,
    game: G,
    /// `None` until winit calls `resumed`. Windows may only be created then.
    window: Option<Window>,
    initialized: bool,
    exit_requested: bool,
    pacer: FramePacer,
    /// First error raised inside a callback, returned by `Engine::run`.
    error: Option<Error>,
}

impl<G: Game> Runner<G> {
    pub(crate) fn new(config: EngineConfig, game: G) -> Self {
        Self {
            config,
            game,
            window: None,
            initialized: false,
            exit_requested: false,
            pacer: FramePacer::new(FRAME_INTERVAL, Instant::now()),
            error: None,
        }
    }

    /// Consumes the runner after the event loop has returned.
    pub(crate) fn finish(self) -> Result<()> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Records the first error and asks the event loop to stop.
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Error) {
        self.error.get_or_insert(error);
        event_loop.exit();
    }

    fn exit_if_requested(&self, event_loop: &ActiveEventLoop) {
        if self.exit_requested {
            event_loop.exit();
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let attributes = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(self.config.resizable);
        let window = event_loop
            .create_window(attributes)
            .map_err(|e| Error::Window(Box::new(e)))?;
        self.window = Some(window);
        Ok(())
    }

    /// One frame. Stage 2 adds time and fixed updates, and Stage 4 adds rendering.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let mut ctx = Context::new(&mut self.exit_requested);
        self.game.update(&mut ctx);
        self.exit_if_requested(event_loop);
    }
}

impl<G: Game> ApplicationHandler for Runner<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none()
            && let Err(error) = self.create_window(event_loop)
        {
            self.fail(event_loop, error);
            return;
        }
        if !self.initialized {
            self.initialized = true;
            let mut ctx = Context::new(&mut self.exit_requested);
            if let Err(error) = self.game.init(&mut ctx) {
                self.fail(event_loop, error);
                return;
            }
            self.exit_if_requested(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(Window::id) != Some(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } if self.config.exit_on_escape => event_loop.exit(),
            // Nothing to resize yet. Stage 4 reconfigures the GPU surface here
            // and must ignore 0×0 sizes (minimized window).
            WindowEvent::Resized(_) => {}
            // `exit()` does not stop the loop immediately: already-queued events
            // still arrive. Never call the game again once exit has begun
            // (after `request_exit`, a failed `init`, Escape or close).
            WindowEvent::RedrawRequested if self.initialized && !event_loop.exiting() => {
                self.frame(event_loop);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &self.window else {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        };
        if event_loop.exiting() {
            return;
        }
        let (frame_due, wake_at) = self.pacer.poll(Instant::now());
        if frame_due {
            window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(wake_at));
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // Drop the window while the event loop is still alive. Stage 4 must
        // drop the renderer (surface) before this line.
        self.window = None;
    }
}
