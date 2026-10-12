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
//!   frame of the overlay in its own render pass, after the game's.
//!
//! Part 1 (PP-037b1) draws a small placeholder window and takes no mouse or
//! keyboard input except F12; input routing to egui is PP-037b2, the real
//! contents PP-037c/d.

use std::time::Instant;

use crate::input::KeyCode;

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

/// The egui context and its wgpu renderer for one GPU device.
pub(crate) struct Overlay {
    context: egui::Context,
    renderer: egui_wgpu::Renderer,
    /// The format of the views this overlay draws into.
    format: wgpu::TextureFormat,
    started: Instant,
    /// Where the overlay's window was drawn last frame, in logical pixels.
    #[cfg_attr(not(test), allow(dead_code))]
    last_window: Option<egui::Rect>,
}

impl Overlay {
    /// Creates the overlay for views of `format`. Prefer a non-sRGB format:
    /// egui blends in gamma space and warns on sRGB targets (it still draws
    /// correct colours there).
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let context = egui::Context::default();
        // No fades: the overlay appears and disappears at once with F12, and a
        // frame looks the same however fast frames come.
        context.global_style_mut(|style| style.animation_time = 0.0);
        Self {
            context,
            renderer: egui_wgpu::Renderer::new(
                device,
                format,
                egui_wgpu::RendererOptions::default(),
            ),
            format,
            started: Instant::now(),
            last_window: None,
        }
    }

    /// The format of the views [`draw`](Self::draw) expects.
    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Records one overlay frame into `encoder`: lays out the window, uploads
    /// egui's textures and buffers, and draws into `view` (format
    /// [`format`](Self::format)) on top of what is already there.
    /// `size_in_pixels` is the target's physical size and `pixels_per_point`
    /// the window's DPI scale.
    pub(crate) fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        size_in_pixels: [u32; 2],
        pixels_per_point: f32,
    ) {
        let pixels_per_point = if pixels_per_point.is_finite() && pixels_per_point > 0.0 {
            pixels_per_point
        } else {
            1.0
        };
        self.context.set_pixels_per_point(pixels_per_point);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(
                    size_in_pixels[0] as f32 / pixels_per_point,
                    size_in_pixels[1] as f32 / pixels_per_point,
                ),
            )),
            time: Some(self.started.elapsed().as_secs_f64()),
            ..Default::default()
        };
        let mut window = None;
        let mut output = self.context.run_ui(input, |ui| {
            window = egui::Window::new("PurplePie")
                .default_pos([12.0, 12.0])
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(concat!("PurplePie ", env!("CARGO_PKG_VERSION")));
                    ui.label("F12 hides this overlay");
                })
                .map(|response| response.response.rect);
        });
        self.last_window = window;

        let shapes = std::mem::take(&mut output.shapes);
        let jobs = self.context.tessellate(shapes, output.pixels_per_point);
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

    const WIDTH: u32 = 320;
    const HEIGHT: u32 = 200;
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
            &overlay_view,
            [WIDTH, HEIGHT],
            1.0,
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
        let window = overlay.last_window.expect("the window was shown");
        let fill = overlay.context.global_style().visuals.window_fill;
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
    fn other_keys_pass_through_unchanged() {
        let mut toggle = Toggle::new();
        for key in [KeyCode::F11, KeyCode::Escape, KeyCode::A, KeyCode::Space] {
            assert!(!toggle.key(key, true, false), "{key:?}");
            assert!(!toggle.key(key, false, false), "{key:?}");
        }
        assert!(toggle.visible());
    }
}
