//! Debug overlay (ADR-043): an egui window drawn over the game, shown and
//! hidden with F12. Compiled only with the `debug-overlay` feature and turned
//! on with [`EngineConfig::with_debug_overlay`](crate::EngineConfig::with_debug_overlay).
//!
//! Crate-private: no egui type reaches the public API. The overlay belongs to
//! the engine, not to the game's world: it never spawns entities, so entity
//! counts, queries, `World::clear` and saved scenes are unaffected.
//!
//! - [`Toggle`] decides what F12 does (no egui, unit-tested).
//! - [`Overlay`] owns the egui context and its wgpu renderer and draws one
//!   frame of the overlay in its own render pass, after the game's (owned by
//!   the renderer).
//! - [`OverlayInput`] turns window events into egui input (egui-winit; owned
//!   by the runner), and [`reaches_game`] decides which events the game still
//!   sees while the overlay is visible (no egui, unit-tested).
//!
//! - [`Stats`] is one frame's numbers for the stats window, gathered from the
//!   engine's state ([`Stats::gather`]) with a smoothed frame time
//!   ([`FrameTimer`]) and formatted by [`stats_rows`] (no egui, unit-tested).
//!
//! - [`inspect`] lists the world's entities and formats one entity's engine
//!   components for the "Entities" window (no egui, unit-tested).
//!
//! PP-037b1 drew the window, PP-037b2 routed the mouse to it, PP-037c filled
//! it with statistics, PP-037d1 added the entity inspector (game components as
//! RON: PP-037d2).

mod inspect;

use std::time::Instant;

use crate::ecs::{Entity, World};
use crate::input::{Input, KeyCode};
use crate::math::Vec2;
use crate::render::Camera2D;

/// The key that shows and hides the overlay.
pub(crate) const TOGGLE_KEY: KeyCode = KeyCode::F12;

/// Whether the overlay is visible, and what the toggle key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Toggle {
    visible: bool,
}

impl Toggle {
    /// An enabled overlay starts visible.
    pub(crate) fn new() -> Self {
        Self { visible: true }
    }

    pub(crate) fn visible(self) -> bool {
        self.visible
    }

    /// Handles one key event. Returns `true` when the event belongs to the
    /// overlay (both the press and the release of [`TOGGLE_KEY`]), so the
    /// game's input must not see it. Only a fresh press toggles; held-key
    /// repeats and synthetic presses do not.
    pub(crate) fn key(&mut self, key: KeyCode, pressed: bool, repeat_or_synthetic: bool) -> bool {
        if key != TOGGLE_KEY {
            return false;
        }
        if pressed && !repeat_or_synthetic {
            self.visible = !self.visible;
        }
        true
    }
}

/// One frame's numbers for the stats window. The runner fills it with
/// [`gather`](Self::gather); the renderer adds this frame's draw counts.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Stats {
    /// Real time since the previous frame, in seconds (not clamped).
    pub(crate) frame_seconds: f64,
    /// [`FrameTimer`]'s smoothed frame time, in seconds.
    pub(crate) average_frame_seconds: f64,
    /// Fixed steps run in this frame.
    pub(crate) fixed_steps: u32,
    /// Length of one fixed step, in seconds.
    pub(crate) fixed_dt: f64,
    pub(crate) entities: u32,
    /// Draw calls and instances (quads, sprites and glyphs) of this frame.
    pub(crate) batches: usize,
    pub(crate) instances: usize,
    pub(crate) camera_position: Vec2,
    /// The camera's effective zoom.
    pub(crate) camera_zoom: f32,
    /// The window's size in logical pixels and its DPI scale.
    pub(crate) viewport: Vec2,
    pub(crate) scale_factor: f64,
    /// The cursor in logical window pixels and in world units (`None`
    /// outside the window).
    pub(crate) cursor_screen: Option<Vec2>,
    pub(crate) cursor_world: Option<Vec2>,
}

/// Inputs to [`Stats::gather`] that only the runner's frame loop knows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct FrameNumbers {
    pub(crate) frame_seconds: f64,
    pub(crate) average_frame_seconds: f64,
    pub(crate) fixed_steps: u32,
    pub(crate) fixed_dt: f64,
    pub(crate) scale_factor: f64,
}

impl Stats {
    /// This frame's numbers from the engine's state (draw counts stay 0
    /// until the renderer sets them).
    pub(crate) fn gather(
        frame: FrameNumbers,
        world: &World,
        camera: &Camera2D,
        viewport: Vec2,
        input: &Input,
    ) -> Self {
        let cursor_screen = input.cursor_position();
        Self {
            frame_seconds: frame.frame_seconds,
            average_frame_seconds: frame.average_frame_seconds,
            fixed_steps: frame.fixed_steps,
            fixed_dt: frame.fixed_dt,
            entities: world.len(),
            batches: 0,
            instances: 0,
            camera_position: camera.position,
            camera_zoom: camera.effective_zoom(),
            viewport,
            scale_factor: frame.scale_factor,
            cursor_screen,
            cursor_world: cursor_screen.map(|screen| camera.screen_to_world(screen, viewport)),
        }
    }
}

/// A smoothed frame time: an exponential moving average, so the shown FPS
/// does not flicker from frame to frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct FrameTimer {
    average: Option<f64>,
}

impl FrameTimer {
    /// How much each new frame moves the average (about the last 10 frames).
    const WEIGHT: f64 = 0.1;

    /// Adds one frame's real duration and returns the new average. Zero,
    /// negative and non-finite durations (the first frame has none) are
    /// skipped.
    pub(crate) fn record(&mut self, seconds: f64) -> f64 {
        if seconds.is_finite() && seconds > 0.0 {
            self.average = Some(match self.average {
                None => seconds,
                Some(average) => average + (seconds - average) * Self::WEIGHT,
            });
        }
        self.average.unwrap_or(0.0)
    }
}

/// The stats window's rows, as (label, value).
pub(crate) fn stats_rows(stats: &Stats) -> Vec<(&'static str, String)> {
    let ms = |seconds: f64| seconds * 1000.0;
    let frame = if stats.average_frame_seconds > 0.0 {
        format!(
            "{:.2} ms ({:.0} FPS)",
            ms(stats.average_frame_seconds),
            1.0 / stats.average_frame_seconds
        )
    } else {
        "-".to_owned()
    };
    let point = |p: Vec2| format!("({:.1}, {:.1})", p.x, p.y);
    let cursor = match (stats.cursor_screen, stats.cursor_world) {
        (Some(screen), Some(world)) => format!("{} px, world {}", point(screen), point(world)),
        _ => "outside the window".to_owned(),
    };
    vec![
        ("Frame", frame),
        ("Last frame", format!("{:.2} ms", ms(stats.frame_seconds))),
        (
            "Fixed steps",
            format!("{} × {:.2} ms", stats.fixed_steps, ms(stats.fixed_dt)),
        ),
        ("Entities", stats.entities.to_string()),
        (
            "Drawn",
            format!("{} batches, {} instances", stats.batches, stats.instances),
        ),
        (
            "Camera",
            format!(
                "{} zoom {:.2}",
                point(stats.camera_position),
                stats.camera_zoom
            ),
        ),
        (
            "Window",
            format!(
                "{:.0} × {:.0} px, scale {:.2}",
                stats.viewport.x, stats.viewport.y, stats.scale_factor
            ),
        ),
        ("Cursor", cursor),
    ]
}

/// What kind of window event the game's input might receive, for
/// [`reaches_game`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Routed {
    /// A mouse button press, a wheel turn or a cursor move.
    Pointer,
    /// A mouse button release; `game_holds` is whether the game's input has
    /// that button down (it saw the press).
    Release { game_holds: bool },
    /// Keys, focus, resize, close and everything else.
    Other,
}

/// While the overlay is visible: does the game's input get this event, given
/// whether egui consumed it (ADR-043)?
///
/// - Pointer events egui consumes (the pointer is over an overlay window, or
///   egui is dragging one) belong to the overlay.
/// - A release always reaches the game when the game holds that button, so a
///   press that started in the game is never left stuck down.
/// - Keys always reach the game: the overlay has no text fields yet, and egui
///   claims Tab and keeps keyboard focus after a click, which would otherwise
///   swallow game keys. (F12 is handled before this, by [`Toggle`].)
pub(crate) fn reaches_game(event: Routed, egui_consumed: bool) -> bool {
    match event {
        Routed::Other | Routed::Release { game_holds: true } => true,
        Routed::Pointer | Routed::Release { game_holds: false } => !egui_consumed,
    }
}

/// Window events → egui input, for the visible overlay (egui-winit).
pub(crate) struct OverlayInput {
    state: egui_winit::State,
}

impl OverlayInput {
    /// Input for the overlay drawn with `context` in `window`.
    pub(crate) fn new(context: egui::Context, window: &winit::window::Window) -> Self {
        Self {
            state: egui_winit::State::new(
                context,
                egui::ViewportId::ROOT,
                window,
                Some(window.scale_factor() as f32),
                window.theme(),
                None,
            ),
        }
    }

    /// Gives one window event to egui; returns whether egui consumed it.
    pub(crate) fn on_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    /// Everything egui needs for the next frame (events since the last one,
    /// screen size, DPI scale, time).
    pub(crate) fn take(&mut self, window: &winit::window::Window) -> egui::RawInput {
        self.state.take_egui_input(window)
    }

    /// Applies what the last overlay frame asked of the window (the cursor icon).
    pub(crate) fn apply(&mut self, window: &winit::window::Window, output: egui::PlatformOutput) {
        self.state.handle_platform_output(window, output);
    }
}

/// The overlay's egui side, without the GPU: input in, layout out.
pub(crate) struct OverlayUi {
    context: egui::Context,
    started: Instant,
    /// Input for the next frame, from [`OverlayInput`]; without it a frame
    /// gets [`basic_input`] (no events).
    pending_input: Option<egui::RawInput>,
    /// What the last frame asked of the window, for [`OverlayInput::apply`].
    platform_output: Option<egui::PlatformOutput>,
    /// Where the overlay's window was laid out last frame, in points.
    #[cfg_attr(not(test), allow(dead_code))]
    last_window: Option<egui::Rect>,
    /// The numbers the next frame shows.
    stats: Stats,
    /// The entity the inspector shows, chosen by clicking it in the list.
    selected: Option<Entity>,
}

impl OverlayUi {
    pub(crate) fn new() -> Self {
        let context = egui::Context::default();
        // No fades: the overlay appears and disappears at once with F12, and a
        // frame looks the same however fast frames come.
        context.global_style_mut(|style| style.animation_time = 0.0);
        Self {
            context,
            started: Instant::now(),
            pending_input: None,
            platform_output: None,
            last_window: None,
            stats: Stats::default(),
            selected: None,
        }
    }

    /// Lays out one frame from the input set with [`Overlay::set_input`], or
    /// from [`basic_input`] for a target of `size_in_pixels` at
    /// `pixels_per_point` when there is none.
    ///
    /// egui's scale is the input's native scale times its zoom factor (left
    /// at 1), so the scale is never set on the context directly: that would
    /// apply it twice.
    fn run(
        &mut self,
        size_in_pixels: [u32; 2],
        pixels_per_point: f32,
        world: &World,
    ) -> egui::FullOutput {
        let pixels_per_point = if pixels_per_point.is_finite() && pixels_per_point > 0.0 {
            pixels_per_point
        } else {
            1.0
        };
        let input = self.pending_input.take().unwrap_or_else(|| {
            basic_input(
                size_in_pixels,
                pixels_per_point,
                self.started.elapsed().as_secs_f64(),
            )
        });
        let rows = stats_rows(&self.stats);
        let selected = &mut self.selected;
        let mut window = None;
        let mut output = self.context.run_ui(input, |ui| {
            window = egui::Window::new("PurplePie")
                .default_pos([12.0, 12.0])
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    egui::Grid::new("purplepie stats")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for (label, value) in &rows {
                                ui.label(*label);
                                ui.monospace(value.as_str());
                                ui.end_row();
                            }
                        });
                    ui.small(concat!(
                        "PurplePie ",
                        env!("CARGO_PKG_VERSION"),
                        " - F12 hides"
                    ));
                })
                .map(|response| response.response.rect);
            entities_window(ui.ctx(), world, selected);
        });
        self.last_window = window;
        self.platform_output = Some(std::mem::take(&mut output.platform_output));
        output
    }
}

/// Heights of the inspector's entity list and component grid (each scrolls),
/// so the window fits under the stats window in a 720-pixel-high window.
const LIST_HEIGHT: f32 = 160.0;
const COMPONENTS_HEIGHT: f32 = 180.0;

/// The "Entities" window: every entity (only the visible rows are laid out),
/// and the engine components of the one clicked. It starts collapsed, and a
/// collapsed window costs nothing (egui skips its contents).
fn entities_window(context: &egui::Context, world: &World, selected: &mut Option<Entity>) {
    egui::Window::new("Entities")
        .default_pos([12.0, 260.0])
        .default_width(340.0)
        .default_open(false)
        .show(context, |ui| {
            let ids = inspect::entity_ids(world);
            ui.label(format!("{} entities - click one to inspect it", ids.len()));
            let row_height = ui.spacing().interact_size.y;
            egui::ScrollArea::vertical()
                .id_salt("purplepie entity list")
                .max_height(LIST_HEIGHT)
                .auto_shrink([false, true])
                .show_rows(ui, row_height, ids.len(), |ui, range| {
                    for &entity in &ids[range] {
                        let label = inspect::summary(world, entity);
                        if ui
                            .selectable_label(*selected == Some(entity), label)
                            .clicked()
                        {
                            *selected = Some(entity);
                        }
                    }
                });
            ui.separator();
            let Some(entity) = *selected else {
                ui.label("No entity selected.");
                return;
            };
            match inspect::component_rows(world, entity) {
                None => {
                    ui.label(format!("#{} was despawned.", entity.id()));
                }
                Some(rows) if rows.is_empty() => {
                    ui.label(format!("#{} has no components.", entity.id()));
                }
                Some(rows) => {
                    ui.strong(format!("#{}", entity.id()));
                    egui::ScrollArea::vertical()
                        .id_salt("purplepie components")
                        .max_height(COMPONENTS_HEIGHT)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            egui::Grid::new("purplepie components")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    for (name, value) in rows {
                                        ui.label(name);
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(value).monospace(),
                                            )
                                            .wrap(),
                                        );
                                        ui.end_row();
                                    }
                                });
                        });
                }
            }
        });
}

/// Input without events: the screen size in points, the DPI scale and the time.
fn basic_input(size_in_pixels: [u32; 2], pixels_per_point: f32, time: f64) -> egui::RawInput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                size_in_pixels[0] as f32 / pixels_per_point,
                size_in_pixels[1] as f32 / pixels_per_point,
            ),
        )),
        time: Some(time),
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .native_pixels_per_point = Some(pixels_per_point);
    input
}

/// Where [`Overlay::draw`] draws one frame.
pub(crate) struct Target<'a> {
    /// A view of the frame in the overlay's [`format`](Overlay::format).
    pub(crate) view: &'a wgpu::TextureView,
    /// The frame's physical size.
    pub(crate) size_in_pixels: [u32; 2],
    /// The window's DPI scale, used only without input from [`OverlayInput`]
    /// (which carries its own).
    pub(crate) pixels_per_point: f32,
}

/// The overlay's egui side and its wgpu renderer for one GPU device.
pub(crate) struct Overlay {
    ui: OverlayUi,
    renderer: egui_wgpu::Renderer,
    /// The format of the views this overlay draws into.
    format: wgpu::TextureFormat,
}

impl Overlay {
    /// Creates the overlay for views of `format`. Prefer a non-sRGB format:
    /// egui blends in gamma space and warns on sRGB targets (it still draws
    /// correct colours there).
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            ui: OverlayUi::new(),
            renderer: egui_wgpu::Renderer::new(
                device,
                format,
                egui_wgpu::RendererOptions::default(),
            ),
            format,
        }
    }

    /// The egui context (shared with [`OverlayInput`]).
    pub(crate) fn context(&self) -> egui::Context {
        self.ui.context.clone()
    }

    /// Sets the input for the next [`draw`](Self::draw).
    pub(crate) fn set_input(&mut self, input: egui::RawInput) {
        self.ui.pending_input = Some(input);
    }

    /// Sets the numbers the next [`draw`](Self::draw) shows.
    pub(crate) fn set_stats(&mut self, stats: Stats) {
        self.ui.stats = stats;
    }

    /// Adds this frame's draw counts to the numbers set with
    /// [`set_stats`](Self::set_stats).
    pub(crate) fn set_draw_counts(&mut self, batches: usize, instances: usize) {
        self.ui.stats.batches = batches;
        self.ui.stats.instances = instances;
    }

    /// What the last frame asked of the window, once.
    pub(crate) fn take_platform_output(&mut self) -> Option<egui::PlatformOutput> {
        self.ui.platform_output.take()
    }

    /// The format of the views [`draw`](Self::draw) expects.
    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Records one overlay frame into `encoder`: lays it out
    /// ([`OverlayUi::run`]), uploads egui's textures and buffers, and draws into
    /// `target` on top of what is already there. `world` is only read, by the
    /// entity inspector.
    pub(crate) fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: Target<'_>,
        world: &World,
    ) {
        let Target {
            view,
            size_in_pixels,
            pixels_per_point,
        } = target;
        let mut output = self.ui.run(size_in_pixels, pixels_per_point, world);

        let shapes = std::mem::take(&mut output.shapes);
        let jobs = self.ui.context.tessellate(shapes, output.pixels_per_point);
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.renderer.update_texture(device, queue, *id, delta);
            }
        }
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels,
            pixels_per_point: output.pixels_per_point,
        };
        // egui-wgpu returns command buffers only for paint callbacks, which the
        // overlay does not use.
        let _ = self
            .renderer
            .update_buffers(device, queue, encoder, &jobs, &screen);
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("purplepie debug overlay"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            self.renderer.render(&mut pass, &jobs, &screen);
        }
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
        // Every texture change has now been applied (egui asserts this).
        output.textures_delta.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f12_toggles_on_fresh_presses_and_is_never_passed_to_the_game() {
        let mut toggle = Toggle::new();
        assert!(toggle.visible(), "an enabled overlay starts visible");

        assert!(toggle.key(KeyCode::F12, true, false), "press is consumed");
        assert!(!toggle.visible());
        assert!(toggle.key(KeyCode::F12, true, true), "repeat is consumed");
        assert!(!toggle.visible(), "a held key does not toggle again");
        assert!(
            toggle.key(KeyCode::F12, false, false),
            "release is consumed"
        );
        assert!(!toggle.visible());

        assert!(toggle.key(KeyCode::F12, true, false));
        assert!(toggle.visible(), "the next press shows it again");
    }

    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 400;
    /// What the target holds before the overlay draws (bytes, written through
    /// the non-sRGB view so they are exact).
    const CLEAR: [u8; 4] = [64, 16, 128, 255];

    /// Clears an `Rgba8UnormSrgb` target (like a typical surface), draws one
    /// overlay frame through a view of `overlay.format()`, and reads the bytes
    /// back.
    fn draw_frame(overlay: &mut Overlay, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u8> {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("overlay test target"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let linear_view = target.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        });
        let overlay_view = target.create_view(&wgpu::TextureViewDescriptor {
            format: Some(overlay.format()),
            ..Default::default()
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let byte = |b: u8| f64::from(b) / 255.0;
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &linear_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: byte(CLEAR[0]),
                            g: byte(CLEAR[1]),
                            b: byte(CLEAR[2]),
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        overlay.draw(
            device,
            queue,
            &mut encoder,
            Target {
                view: &overlay_view,
                size_in_pixels: [WIDTH, HEIGHT],
                pixels_per_point: 1.0,
            },
            &World::new(),
        );
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay readback"),
            size: u64::from(WIDTH * HEIGHT * 4),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(WIDTH * 4),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
        readback.map_async(wgpu::MapMode::Read, .., |result| {
            result.expect("map readback buffer");
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU finished");
        readback.get_mapped_range(..).expect("mapped").to_vec()
    }

    /// Draws two overlay frames (egui lays a new window out invisibly on its
    /// first frame) and returns the second frame's bytes, the window rectangle
    /// and the colour egui fills windows with.
    fn second_frame(format: wgpu::TextureFormat) -> (Vec<u8>, egui::Rect, egui::Color32) {
        let (device, queue, faults) = crate::render::headless_device_and_queue();
        let mut overlay = Overlay::new(&device, format);
        let _ = draw_frame(&mut overlay, &device, &queue);
        let pixels = draw_frame(&mut overlay, &device, &queue);
        assert!(faults.take().is_none(), "no GPU errors");
        let window = overlay.ui.last_window.expect("the window was shown");
        let fill = overlay.ui.context.global_style().visuals.window_fill;
        (pixels, window, fill)
    }

    fn pixel(pixels: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * WIDTH + x) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    }

    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test --features debug-overlay -- --ignored`"]
    fn the_window_is_drawn_over_the_frame_with_the_same_colours_through_either_view() {
        for format in [
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ] {
            let (pixels, window, fill) = second_frame(format);
            assert!(
                window.min.x >= 0.0 && window.max.x < WIDTH as f32 && window.max.y < HEIGHT as f32,
                "{format:?}: window {window:?} inside the target"
            );
            // Inside the window, just above its bottom edge: egui's window fill
            // (sRGB bytes), whichever view egui drew through.
            let inside = pixel(&pixels, window.center().x as u32, window.max.y as u32 - 4);
            for c in 0..3 {
                let expected = fill.to_array()[c];
                assert!(
                    inside[c].abs_diff(expected) <= 2,
                    "{format:?}: window fill {inside:?}, expected {fill:?}"
                );
            }
            // Away from the window and its shadow, the frame is untouched.
            let shadow = 40.0;
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let (fx, fy) = (x as f32, y as f32);
                    let near = fx > window.min.x - shadow
                        && fx < window.max.x + shadow
                        && fy > window.min.y - shadow
                        && fy < window.max.y + shadow;
                    if !near {
                        assert_eq!(pixel(&pixels, x, y), CLEAR, "{format:?}: ({x}, {y})");
                    }
                }
            }
        }
    }

    #[test]
    fn pointer_events_egui_consumes_stay_with_the_overlay() {
        assert!(
            !reaches_game(Routed::Pointer, true),
            "click/wheel/drag on the overlay"
        );
        assert!(
            reaches_game(Routed::Pointer, false),
            "pointer over the game"
        );
        assert!(
            !reaches_game(Routed::Release { game_holds: false }, true),
            "the release of a click that started on the overlay"
        );
        assert!(reaches_game(Routed::Release { game_holds: false }, false));
    }

    #[test]
    fn a_button_the_game_holds_is_always_released_and_keys_always_pass() {
        for consumed in [false, true] {
            assert!(
                reaches_game(Routed::Release { game_holds: true }, consumed),
                "a press that started in the game must not stay stuck (consumed: {consumed})"
            );
            assert!(reaches_game(Routed::Other, consumed), "keys, focus, resize");
        }
    }

    #[test]
    fn egui_uses_the_windows_scale_exactly_once() {
        // The scale must come only from the input's native scale (times a zoom
        // of 1); setting it on the context as well would apply it twice.
        let mut ui = OverlayUi::new();
        let mut windows = Vec::new();
        for frame in 0..3 {
            let mut output = ui.run([640, 400], 2.0, &World::new());
            output.textures_delta.clear(); // no renderer here
            assert_eq!(output.pixels_per_point, 2.0, "basic input, frame {frame}");
            assert_eq!(ui.context.viewport_rect().size(), egui::vec2(320.0, 200.0));
            windows.extend(ui.last_window);
        }
        // Input like egui-winit's (it carries its own native scale) wins over
        // the size and scale passed to `run`.
        ui.pending_input = Some(basic_input([640, 400], 2.0, 1.0));
        let mut output = ui.run([640, 400], 1.0, &World::new());
        output.textures_delta.clear();
        assert_eq!(output.pixels_per_point, 2.0, "input from the window");
        windows.extend(ui.last_window);
        assert!(windows.len() >= 3, "the window is laid out");
        assert!(
            windows.windows(2).all(|pair| pair[0] == pair[1]),
            "same place and size in points every frame: {windows:?}"
        );
    }

    #[test]
    fn the_frame_timer_smooths_and_skips_frames_without_a_duration() {
        let mut timer = FrameTimer::default();
        assert_eq!(timer.record(0.0), 0.0, "the first frame has no duration");
        assert_eq!(
            timer.record(0.020),
            0.020,
            "the first real frame is the average"
        );
        let average = timer.record(0.010);
        assert!(
            (average - 0.019).abs() < 1e-12,
            "{average}: moves 10 % of the way"
        );
        for skipped in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(timer.record(skipped), average, "{skipped} is skipped");
        }
        for _ in 0..200 {
            timer.record(0.010);
        }
        assert!(
            (timer.record(0.010) - 0.010).abs() < 1e-9,
            "settles on a steady rate"
        );
    }

    #[test]
    fn stats_rows_read_as_the_window_shows_them() {
        let stats = Stats {
            frame_seconds: 0.0175,
            average_frame_seconds: 0.016_667,
            fixed_steps: 2,
            fixed_dt: 1.0 / 60.0,
            entities: 19,
            batches: 7,
            instances: 120,
            camera_position: Vec2::new(-12.5, 40.0),
            camera_zoom: 2.0,
            viewport: Vec2::new(1280.0, 720.0),
            scale_factor: 1.25,
            cursor_screen: Some(Vec2::new(640.0, 360.0)),
            cursor_world: Some(Vec2::new(-12.5, 40.0)),
        };
        let rows = stats_rows(&stats);
        let expected = [
            ("Frame", "16.67 ms (60 FPS)"),
            ("Last frame", "17.50 ms"),
            ("Fixed steps", "2 × 16.67 ms"),
            ("Entities", "19"),
            ("Drawn", "7 batches, 120 instances"),
            ("Camera", "(-12.5, 40.0) zoom 2.00"),
            ("Window", "1280 × 720 px, scale 1.25"),
            ("Cursor", "(640.0, 360.0) px, world (-12.5, 40.0)"),
        ];
        assert_eq!(rows.len(), expected.len());
        for ((label, value), (want_label, want_value)) in rows.iter().zip(expected) {
            assert_eq!((*label, value.as_str()), (want_label, want_value));
        }

        let empty = stats_rows(&Stats::default());
        assert_eq!(empty[0].1, "-", "no FPS before the first timed frame");
        assert_eq!(empty[7].1, "outside the window");
    }

    #[test]
    fn gather_reads_the_world_camera_and_cursor() {
        let mut world = World::new();
        for i in 0..3 {
            world.spawn((i,));
        }
        let camera = Camera2D::new(Vec2::new(100.0, 50.0), 2.0);
        let viewport = Vec2::new(800.0, 600.0);
        let mut input = Input::default();
        input.set_cursor(Some(Vec2::new(400.0, 300.0)));
        let frame = FrameNumbers {
            frame_seconds: 0.02,
            average_frame_seconds: 0.016,
            fixed_steps: 1,
            fixed_dt: 1.0 / 60.0,
            scale_factor: 1.0,
        };

        let stats = Stats::gather(frame, &world, &camera, viewport, &input);
        assert_eq!(stats.entities, 3);
        assert_eq!(
            (stats.camera_position, stats.camera_zoom),
            (camera.position, 2.0)
        );
        assert_eq!(stats.viewport, viewport);
        assert_eq!(stats.cursor_screen, Some(Vec2::new(400.0, 300.0)));
        assert_eq!(
            stats.cursor_world,
            Some(camera.position),
            "the window's centre is where the camera looks"
        );
        assert_eq!((stats.fixed_steps, stats.frame_seconds), (1, 0.02));
        assert_eq!(
            (stats.batches, stats.instances),
            (0, 0),
            "the renderer adds these"
        );

        input.set_cursor(None);
        let outside = Stats::gather(frame, &world, &camera, viewport, &input);
        assert_eq!((outside.cursor_screen, outside.cursor_world), (None, None));
    }

    #[test]
    fn other_keys_pass_through_unchanged() {
        let mut toggle = Toggle::new();
        for key in [KeyCode::F11, KeyCode::Escape, KeyCode::A, KeyCode::Space] {
            assert!(!toggle.key(key, true, false), "{key:?}");
            assert!(!toggle.key(key, false, false), "{key:?}");
        }
        assert!(toggle.visible());
    }
}
