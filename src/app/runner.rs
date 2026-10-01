//! The winit `ApplicationHandler` that drives a [`Game`] (winit 0.30 lifecycle).
//!
//! This is the only place in the engine that handles winit events.

use std::sync::Arc;
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
use crate::ecs::World;
use crate::error::{Error, Result};
use crate::render::Renderer;
use crate::time::{FixedTimestep, Time};

/// Owns the game and all engine state for the lifetime of the event loop.
pub(crate) struct Runner<G: Game> {
    config: EngineConfig,
    game: G,
    /// `None` until winit calls `resumed`. Windows may only be created then.
    /// `Arc` because the GPU surface shares ownership of the window (ADR-009).
    window: Option<Arc<Window>>,
    /// `None` until `resumed`, and dropped again on `suspended` and `exiting`.
    renderer: Option<Renderer>,
    initialized: bool,
    exit_requested: bool,
    pacer: FramePacer,
    time: Time,
    fixed: FixedTimestep,
    /// The single game world, lent to the game in every callback (ADR-008).
    world: World,
    /// When the previous frame started; `None` before the first frame.
    last_frame: Option<Instant>,
    /// First error raised inside a callback, returned by `Engine::run`.
    error: Option<Error>,
}

impl<G: Game> Runner<G> {
    pub(crate) fn new(config: EngineConfig, game: G) -> Self {
        Self {
            game,
            window: None,
            renderer: None,
            initialized: false,
            exit_requested: false,
            pacer: FramePacer::new(FRAME_INTERVAL, Instant::now()),
            time: Time::new(config.fixed_dt),
            fixed: FixedTimestep::new(config.fixed_dt, config.max_fixed_steps),
            world: World::new(),
            last_frame: None,
            error: None,
            config,
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
        self.window = Some(Arc::new(window));
        Ok(())
    }

    /// Creates the GPU renderer for the current window (ADR-012: blocks briefly).
    fn create_renderer(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let size = window.inner_size();
        let renderer = Renderer::new(
            event_loop.owned_display_handle(),
            window.clone(),
            size.width,
            size.height,
            window.scale_factor(),
            self.config.clear_color,
        )?;
        self.renderer = Some(renderer);
        Ok(())
    }

    /// Draws the current frame, if a renderer exists.
    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) else {
            return;
        };
        if let Err(error) = renderer.render(&self.world, || window.pre_present_notify()) {
            self.fail(event_loop, error);
        }
    }

    /// One frame (ADR-010): measure time → fixed updates × n → update → render.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        // The first frame has no predecessor, so it gets a zero delta rather
        // than the time spent creating the window and running `init`.
        let raw_delta = self
            .last_frame
            .map_or(0.0, |previous| (now - previous).as_secs_f64());
        self.last_frame = Some(now);
        let delta = self.time.begin_frame(raw_delta, self.config.max_frame_dt);

        let steps = self.fixed.advance(delta);
        let fixed_dt = self.config.fixed_dt;
        for _ in 0..steps {
            self.time.record_fixed_step();
            let mut ctx = Context::new(
                &mut self.exit_requested,
                &self.time,
                &mut self.world,
                fixed_dt,
            );
            self.game.fixed_update(&mut ctx);
            if self.exit_requested {
                break;
            }
        }
        self.time.set_alpha(self.fixed.alpha());

        if !self.exit_requested {
            let mut ctx =
                Context::new(&mut self.exit_requested, &self.time, &mut self.world, delta);
            self.game.update(&mut ctx);
        }
        if self.exit_requested {
            event_loop.exit();
        } else {
            self.render(event_loop);
        }
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
        if self.renderer.is_none()
            && let Err(error) = self.create_renderer(event_loop)
        {
            self.fail(event_loop, error);
            return;
        }
        if !self.initialized {
            self.initialized = true;
            let mut ctx = Context::new(&mut self.exit_requested, &self.time, &mut self.world, 0.0);
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
        if self.window.as_ref().map(|w| w.id()) != Some(window_id) {
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
            WindowEvent::Resized(size) => {
                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    renderer.resize(size.width, size.height, window.scale_factor());
                }
            }
            // winit follows a DPI change with `Resized` if the physical size changes.
            // Update the scale here too, in case it doesn't.
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    let size = window.inner_size();
                    renderer.resize(size.width, size.height, scale_factor);
                }
            }
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

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Platforms may invalidate surfaces while suspended. Drop the renderer
        // and recreate it in `resumed`. Game state (world, time) is kept.
        self.renderer = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // Drop order matters: GPU surface first, then the window it draws to,
        // both while the event loop is still alive.
        self.renderer = None;
        self.window = None;
    }
}
