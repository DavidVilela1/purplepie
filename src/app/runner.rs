//! The winit `ApplicationHandler` that drives a [`Game`] (winit 0.30 lifecycle).
//!
//! This is the only place in the engine that handles winit events.

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use super::config::EngineConfig;
use super::game::{Context, Game};
use super::keymap;
use super::pacer::{FRAME_INTERVAL, FramePacer};
use super::state::{EngineState, physical_to_logical, sanitize_scale_factor};
use crate::assets::AssetRoot;
use crate::audio::AudioOutput;
use crate::error::{Error, Result};
use crate::render::Renderer;
use crate::time::FixedTimestep;

/// How often hot reload checks texture files for changes (ADR-037).
const RELOAD_INTERVAL: Duration = Duration::from_millis(500);

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
    pacer: FramePacer,
    fixed: FixedTimestep,
    /// World, time, textures, camera, input and viewport: lent to the game
    /// through `Context`, read by the renderer.
    state: EngineState,
    /// The window's DPI scale, kept up to date from window events (like the viewport).
    scale_factor: f64,
    /// When the previous frame started; `None` before the first frame.
    last_frame: Option<Instant>,
    /// First error raised inside a callback, returned by `Engine::run`.
    error: Option<Error>,
    /// When hot reload last checked the texture files (ADR-037).
    last_reload_check: Option<Instant>,
    /// The debug overlay's F12 state, when the game turned it on (ADR-043).
    #[cfg(feature = "debug-overlay")]
    overlay: Option<crate::debug::Toggle>,
    /// Window events → egui for the visible overlay; created with the renderer.
    #[cfg(feature = "debug-overlay")]
    overlay_input: Option<crate::debug::OverlayInput>,
    /// Smoothed frame time and this frame's numbers, for the overlay's stats.
    #[cfg(feature = "debug-overlay")]
    frame_timer: crate::debug::FrameTimer,
    #[cfg(feature = "debug-overlay")]
    frame_numbers: crate::debug::FrameNumbers,
}

impl<G: Game> Runner<G> {
    pub(crate) fn new(config: EngineConfig, game: G) -> Self {
        let assets = AssetRoot::for_process(config.asset_root.as_deref());
        match &assets {
            AssetRoot::Found(root) => log::info!("asset root: {}", root.display()),
            AssetRoot::NotFound { searched } => {
                log::warn!("no asset folder found (looked for {searched:?})");
            }
        }
        if config.debug_overlay && !cfg!(feature = "debug-overlay") {
            log::warn!(
                "debug overlay requested, but PurplePie was built without its `debug-overlay` feature"
            );
        }
        Self {
            game,
            window: None,
            renderer: None,
            initialized: false,
            pacer: FramePacer::new(FRAME_INTERVAL, Instant::now()),
            fixed: FixedTimestep::new(config.fixed_dt, config.max_fixed_steps),
            state: {
                let mut state = EngineState::new(config.fixed_dt, assets);
                if config.audio {
                    state.audio = AudioOutput::open();
                }
                state
            },
            scale_factor: 1.0,
            last_frame: None,
            error: None,
            last_reload_check: None,
            #[cfg(feature = "debug-overlay")]
            overlay: config.debug_overlay.then(crate::debug::Toggle::new),
            #[cfg(feature = "debug-overlay")]
            overlay_input: None,
            #[cfg(feature = "debug-overlay")]
            frame_timer: crate::debug::FrameTimer::default(),
            #[cfg(feature = "debug-overlay")]
            frame_numbers: crate::debug::FrameNumbers::default(),
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
        if self.state.exit_requested {
            event_loop.exit();
        }
    }

    /// Recomputes the logical viewport from a physical size and DPI scale.
    ///
    /// Called only from window events and window creation, never per frame:
    /// querying a window that the platform has already destroyed can panic
    /// inside winit (X11 `inner_size`), which a per-frame query hit in testing.
    fn set_viewport(&mut self, width: u32, height: u32, scale_factor: f64) {
        self.scale_factor = sanitize_scale_factor(scale_factor);
        self.state.viewport =
            physical_to_logical(f64::from(width), f64::from(height), self.scale_factor);
    }

    /// Applies a title the game requested with `Context::set_window_title`.
    /// Only called when there is a request, never as a per-frame window query.
    fn apply_window_title(&mut self) {
        if let (Some(title), Some(window)) = (self.state.window_title.take(), &self.window) {
            window.set_title(&title);
        }
    }

    /// Feeds a key event to `Input`, after the optional Escape-to-exit shortcut.
    fn keyboard(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent, is_synthetic: bool) {
        let pressed = event.state == ElementState::Pressed;
        if self.config.exit_on_escape
            && pressed
            && !event.repeat
            && event.logical_key == Key::Named(NamedKey::Escape)
        {
            event_loop.exit();
            return;
        }
        let Some(key) = keymap::translate(event.physical_key) else {
            return;
        };
        // F12 belongs to the debug overlay while it is on (ADR-043).
        #[cfg(feature = "debug-overlay")]
        if let Some(overlay) = &mut self.overlay
            && overlay.key(key, pressed, event.repeat || is_synthetic)
        {
            log::debug!("debug overlay visible: {}", overlay.visible());
            // egui may have left a resize or grab cursor behind.
            if !overlay.visible()
                && let Some(window) = &self.window
            {
                window.set_cursor(winit::window::CursorIcon::Default);
            }
            return;
        }
        log::debug!(
            "key {key:?} {} (repeat: {}, synthetic: {is_synthetic})",
            if pressed { "down" } else { "up" },
            event.repeat
        );
        if pressed {
            // Synthetic presses (X11 reports keys already held when the window
            // gains focus) are not real presses: ignore them, so they never
            // become `just_pressed` edges.
            if !is_synthetic {
                self.state.input.key_down(key);
            }
        } else {
            self.state.input.key_up(key);
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
        let size = window.inner_size();
        self.set_viewport(size.width, size.height, window.scale_factor());
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
            self.config.debug_overlay,
        )?;
        #[cfg(feature = "debug-overlay")]
        let mut renderer = renderer;
        #[cfg(feature = "debug-overlay")]
        {
            self.overlay_input = renderer
                .overlay_mut()
                .map(|overlay| crate::debug::OverlayInput::new(overlay.context(), window));
        }
        self.renderer = Some(renderer);
        Ok(())
    }

    /// Draws the current frame, if a renderer exists.
    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) else {
            return;
        };
        let state = &self.state;
        #[cfg(feature = "debug-overlay")]
        let overlay_visible = self.overlay.is_some_and(crate::debug::Toggle::visible);
        // The overlay's input for this frame: everything since the last one.
        #[cfg(feature = "debug-overlay")]
        if overlay_visible
            && let (Some(input), Some(overlay)) = (&mut self.overlay_input, renderer.overlay_mut())
        {
            overlay.set_input(input.take(window));
            overlay.set_stats(crate::debug::Stats::gather(
                self.frame_numbers,
                &state.world,
                &state.camera,
                state.viewport,
                &state.input,
            ));
        }
        #[cfg(not(feature = "debug-overlay"))]
        let overlay_visible = false;
        if let Err(error) = renderer.render(
            &state.world,
            &state.textures,
            &state.fonts,
            &state.camera,
            overlay_visible,
            || {
                window.pre_present_notify();
            },
        ) {
            self.fail(event_loop, error);
        }
        // Apply what the overlay asked of the window (its cursor icon).
        #[cfg(feature = "debug-overlay")]
        if let (Some(input), Some(renderer), Some(window)) =
            (&mut self.overlay_input, &mut self.renderer, &self.window)
            && let Some(output) = renderer
                .overlay_mut()
                .and_then(crate::debug::Overlay::take_platform_output)
        {
            input.apply(window, output);
        }
    }

    /// While the overlay is visible, gives `event` to egui first and returns
    /// whether the game should still see it (ADR-043, [`crate::debug::reaches_game`]).
    #[cfg(feature = "debug-overlay")]
    fn game_sees(&mut self, event: &WindowEvent) -> bool {
        use crate::debug::Routed;
        let visible = self.overlay.is_some_and(crate::debug::Toggle::visible);
        let (true, Some(input), Some(window)) = (visible, &mut self.overlay_input, &self.window)
        else {
            return true;
        };
        let consumed = input.on_event(window, event);
        let routed = match event {
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } => Routed::Release {
                game_holds: keymap::translate_button(*button)
                    .is_some_and(|button| self.state.input.mouse_pressed(button)),
            },
            WindowEvent::MouseInput { .. }
            | WindowEvent::MouseWheel { .. }
            | WindowEvent::CursorMoved { .. } => Routed::Pointer,
            _ => Routed::Other,
        };
        let reaches = crate::debug::reaches_game(routed, consumed);
        if !reaches {
            log::trace!("debug overlay took {routed:?}");
        }
        reaches
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
        let delta = self
            .state
            .time
            .begin_frame(raw_delta, self.config.max_frame_dt);

        let steps = self.fixed.advance(delta);
        let fixed_dt = self.config.fixed_dt;
        #[cfg(feature = "debug-overlay")]
        {
            self.frame_numbers = crate::debug::FrameNumbers {
                frame_seconds: raw_delta,
                average_frame_seconds: self.frame_timer.record(raw_delta),
                fixed_steps: steps,
                fixed_dt,
                scale_factor: self.scale_factor,
            };
        }
        for _ in 0..steps {
            self.state.time.record_fixed_step();
            // Key edges are visible to the first fixed step after they happened
            // only (ADR-024).
            self.state.input.begin_fixed_step();
            self.game
                .fixed_update(&mut Context::new(&mut self.state, fixed_dt));
            self.state.input.end_fixed_step();
            if self.state.exit_requested {
                break;
            }
        }
        self.state.time.set_alpha(self.fixed.alpha());

        if !self.state.exit_requested {
            self.game.update(&mut Context::new(&mut self.state, delta));
        }
        self.state.input.end_frame();
        self.apply_window_title();
        if self.state.exit_requested {
            event_loop.exit();
        } else {
            self.reload_changed_assets(now);
            self.render(event_loop);
        }
    }

    /// Hot reload (ADR-037): at most every [`RELOAD_INTERVAL`], re-read the
    /// textures, fonts and sounds whose files changed; the renderer re-uploads
    /// textures and clears the glyph atlas for fonts this frame.
    fn reload_changed_assets(&mut self, now: Instant) {
        if !self.config.hot_reload {
            return;
        }
        if self
            .last_reload_check
            .is_some_and(|last| now.duration_since(last) < RELOAD_INTERVAL)
        {
            return;
        }
        self.last_reload_check = Some(now);
        self.state.textures.reload_changed();
        self.state.fonts.reload_changed();
        self.state.sounds.reload_changed();
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
            if let Err(error) = self.game.init(&mut Context::new(&mut self.state, 0.0)) {
                self.fail(event_loop, error);
                return;
            }
            self.apply_window_title();
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
        #[cfg(feature = "debug-overlay")]
        if !self.game_sees(&event) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => self.keyboard(event_loop, &event, is_synthetic),
            // Releases that happen while another window has focus never arrive,
            // so treat every held key and mouse button as released (ADR-024).
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) = keymap::translate_button(button) {
                    match state {
                        ElementState::Pressed => self.state.input.mouse_down(button),
                        ElementState::Released => self.state.input.mouse_up(button),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let logical = physical_to_logical(position.x, position.y, self.scale_factor);
                self.state.input.set_cursor(Some(logical));
            }
            WindowEvent::CursorLeft { .. } => self.state.input.set_cursor(None),
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let lines = keymap::scroll_lines(delta, self.scale_factor);
                log::debug!("wheel {delta:?} ({phase:?}) → {lines} lines");
                self.state.input.add_scroll(lines);
            }
            WindowEvent::Focused(false) => {
                log::debug!("focus lost: releasing all keys");
                self.state.input.release_all();
            }
            WindowEvent::Resized(size) => {
                let Some(scale_factor) = self.window.as_ref().map(|w| w.scale_factor()) else {
                    return;
                };
                self.set_viewport(size.width, size.height, scale_factor);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height, scale_factor);
                }
            }
            // winit follows a DPI change with `Resized` if the physical size changes.
            // Update the scale here too, in case it doesn't.
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let Some(size) = self.window.as_ref().map(|w| w.inner_size()) else {
                    return;
                };
                self.set_viewport(size.width, size.height, scale_factor);
                if let Some(renderer) = &mut self.renderer {
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
        #[cfg(feature = "debug-overlay")]
        {
            self.overlay_input = None;
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // Drop order matters: GPU surface first, then the window it draws to,
        // both while the event loop is still alive.
        self.renderer = None;
        self.window = None;
    }
}
