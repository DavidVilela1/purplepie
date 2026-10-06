//! GPU context and frame presentation (ADR-005, ADR-009, ADR-014, ADR-019, ADR-020, ADR-021, ADR-022, ADR-027).
//!
//! The only code in the engine that touches `wgpu`. It receives the window
//! as generic `raw-window-handle` providers, so it does not depend on `winit`.

use std::sync::Arc;

use super::atlas::GlyphAtlas;
use super::draw::{Batch, DrawList, Material, View};
use super::faults::{FaultSlot, GpuFault};
use super::instance::InstanceBuffer;
use super::quad::QuadPipeline;
use super::sprite::SpritePipeline;
use super::{Camera2D, Color};
use super::{Fonts, Textures};
use crate::ecs::World;
use crate::error::{Error, Result};
use crate::math::Vec2;

/// Owns the GPU objects for one window. Crate-private: game code never sees it.
pub(crate) struct Renderer {
    /// Holds a clone of the window `Arc`, which keeps the window alive for as long as the surface exists.
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    clear: wgpu::Color,
    /// Width or height is 0 (minimized). Never configure or render in this state.
    minimized: bool,
    /// The last frame was `Suboptimal`: reconfigure before the next one.
    needs_reconfigure: bool,
    /// First fatal GPU fault reported by wgpu's callbacks (ADR-017).
    faults: FaultSlot,
    /// Physical pixels per logical pixel (window DPI scale).
    scale_factor: f64,
    quads: QuadPipeline,
    sprites: SpritePipeline,
    /// This frame's sorted instances and batches, reused every frame (ADR-021).
    draw_list: DrawList,
    /// One GPU instance buffer shared by quads, sprites and glyphs, in draw order.
    instances: InstanceBuffer,
    /// Rasterized glyphs (CPU copy; the GPU copy lives in `sprites`, ADR-027).
    glyph_atlas: GlyphAtlas,
}

impl Renderer {
    /// Runs the wgpu initialization chain for `window`.
    ///
    /// `display` must be the display connection the window belongs to (winit's
    /// `OwnedDisplayHandle`). `width`/`height` are the window's physical size,
    /// and `scale_factor` is physical pixels per logical pixel.
    pub(crate) fn new(
        display: impl wgpu::wgt::WgpuHasDisplayHandle,
        window: Arc<dyn wgpu::WindowHandle>,
        width: u32,
        height: u32,
        scale_factor: f64,
        clear_color: Color,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::from_window_without_display(window))
            .map_err(|e| Error::Surface(Box::new(e)))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .map_err(|e| Error::Adapter(Box::new(e)))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("purplepie device"),
            required_features: wgpu::Features::empty(),
            // Ask only for what a 2D renderer needs, so older GPUs and GL work too.
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        }))
        .map_err(|e| Error::Device(Box::new(e)))?;
        // Replace wgpu's default handler, which panics, before any further GPU call.
        let faults = FaultSlot::default();
        faults.install(&device);

        let mut config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or(Error::SurfaceUnsupported)?;
        // The default is the adapter's first present mode, which can be `Immediate`
        // (no vsync; observed with lavapipe). Request vsync explicitly (ADR-014).
        config.present_mode = wgpu::PresentMode::AutoVsync;

        let clear = clear_color.to_wgpu(config.format.is_srgb());
        let minimized = width == 0 || height == 0;
        if !minimized {
            surface.configure(&device, &config);
        }
        // Shader or pipeline errors are reported through `faults` and caught just below.
        let quads = QuadPipeline::new(&device, config.format);
        let glyph_atlas = GlyphAtlas::default();
        let sprites = SpritePipeline::new(&device, config.format, glyph_atlas.size());
        check(&faults)?;

        let info = adapter.get_info();
        log::info!(
            "GPU: {} ({:?}, {:?}); surface {:?}, {:?}",
            info.name,
            info.backend,
            info.device_type,
            config.format,
            config.present_mode
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            clear,
            minimized,
            needs_reconfigure: false,
            faults,
            scale_factor: sanitize_scale_factor(scale_factor),
            quads,
            sprites,
            draw_list: DrawList::default(),
            instances: InstanceBuffer::new("purplepie instances"),
            glyph_atlas,
        })
    }

    /// Call when the window's physical size or DPI scale changes. A zero width
    /// or height (minimized window) pauses rendering until a real size arrives.
    pub(crate) fn resize(&mut self, width: u32, height: u32, scale_factor: f64) {
        self.scale_factor = sanitize_scale_factor(scale_factor);
        if width == 0 || height == 0 {
            self.minimized = true;
            return;
        }
        self.minimized = false;
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.needs_reconfigure = false;
    }

    /// Draws and presents one frame. `before_present` runs right before the
    /// frame is handed to the compositor (winit's `pre_present_notify`).
    ///
    /// Returns `Err(Error::Render)` for fatal GPU faults (ADR-017): any
    /// uncaptured wgpu error, device loss, a lost surface, or an acquire
    /// validation failure. Minimized, occluded, timeout and outdated surfaces
    /// skip the frame, and the next frame retries (ARCHITECTURE §7).
    ///
    /// Reads `world` (never writes it, ADR-009) and draws every entity with
    /// `Transform2D` + `Quad`, `Sprite` or `Text`, ordered by `Layer` (ADR-021),
    /// as seen through `camera` (ADR-022). Text uses the fonts in `fonts` (ADR-027).
    /// Textures in `textures` that are not on the GPU yet are uploaded first;
    /// one the GPU cannot hold returns `Err(Error::Asset)` (ADR-020).
    pub(crate) fn render(
        &mut self,
        world: &World,
        textures: &Textures,
        fonts: &Fonts,
        camera: &Camera2D,
        before_present: impl FnOnce(),
    ) -> Result<()> {
        // Faults raised since the last frame (e.g. by `resize`) are reported first.
        check(&self.faults)?;
        // Upload new textures even while minimized, so load errors surface early.
        self.sprites
            .sync_textures(&self.device, &self.queue, textures)?;
        if self.minimized {
            return Ok(());
        }
        if self.needs_reconfigure {
            self.surface.configure(&self.device, &self.config);
            self.needs_reconfigure = false;
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                // Still usable. Present it, then reconfigure before the next frame
                // (never while a SurfaceTexture is alive, or `configure` panics).
                log::debug!("surface suboptimal; reconfiguring after this frame");
                self.needs_reconfigure = true;
                frame
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                // The uncaptured-error callback normally recorded the cause already.
                let fault = self.faults.take().unwrap_or(GpuFault::AcquireValidation);
                return Err(Error::Render(Box::new(fault)));
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                log::debug!("surface outdated; reconfiguring");
                self.surface.configure(&self.device, &self.config);
                return check(&self.faults);
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // Fatal by design (ADR-017): recreating the surface for a window that is
                // gone panics inside wgpu-hal 30.0.1 instead of returning an error.
                return Err(Error::Render(Box::new(GpuFault::SurfaceLost)));
            }
        };

        let physical_size = Vec2::new(self.config.width as f32, self.config.height as f32);
        let logical_size = physical_size / self.scale_factor as f32;
        let view = View {
            view_projection: camera.view_projection(logical_size),
            physical_size,
            target_is_srgb: self.config.format.is_srgb(),
        };
        self.draw_list
            .build(world, &view, fonts, &mut self.glyph_atlas);
        self.sprites
            .upload_glyphs(&self.queue, &mut self.glyph_atlas);
        self.instances
            .upload(&self.device, &self.queue, self.draw_list.instances());

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("purplepie frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("purplepie main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some(instances) = self.instances.slice() {
                pass.set_vertex_buffer(0, instances);
                record_batches(
                    &mut pass,
                    self.draw_list.batches(),
                    &self.quads,
                    &self.sprites,
                );
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        before_present();
        self.queue.present(frame);
        check(&self.faults)
    }
}

/// Records one draw call per batch, switching pipelines only when the material
/// kind changes. The shared instance buffer must already be bound.
fn record_batches(
    pass: &mut wgpu::RenderPass<'_>,
    batches: &[Batch],
    quads: &QuadPipeline,
    sprites: &SpritePipeline,
) {
    #[derive(PartialEq)]
    enum Bound {
        Nothing,
        Quads,
        Sprites,
    }
    let mut bound = Bound::Nothing;
    for batch in batches {
        match batch.material {
            Material::Color => {
                if bound != Bound::Quads {
                    quads.bind(pass);
                    bound = Bound::Quads;
                }
            }
            Material::Texture(texture) => {
                if bound != Bound::Sprites {
                    sprites.bind(pass);
                    bound = Bound::Sprites;
                }
                if !sprites.bind_texture(pass, texture) {
                    continue;
                }
            }
            Material::Glyphs => {
                if bound != Bound::Sprites {
                    sprites.bind(pass);
                    bound = Bound::Sprites;
                }
                sprites.bind_glyphs(pass);
            }
        }
        pass.draw(0..6, batch.instances.clone());
    }
}

/// Guards against a zero, negative or non-finite DPI scale (would break the projection).
fn sanitize_scale_factor(scale_factor: f64) -> f64 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

/// Converts a recorded GPU fault into `Error::Render`.
fn check(faults: &FaultSlot) -> Result<()> {
    match faults.take() {
        Some(fault) => Err(Error::Render(Box::new(fault))),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::quad::tests::headless_device_and_queue;
    use super::super::text::{self, Text};
    use super::*;
    use crate::math::Transform2D;

    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 128;

    /// Draws `world` with the real pipelines into an offscreen `format`
    /// texture cleared to opaque black, and reads the pixels back (RGBA8).
    fn render_offscreen(
        world: &World,
        fonts: &Fonts,
        camera: &Camera2D,
        format: wgpu::TextureFormat,
        atlas: &mut GlyphAtlas,
    ) -> Vec<u8> {
        let (device, queue, faults) = headless_device_and_queue();
        let quads = QuadPipeline::new(&device, format);
        let sprites = SpritePipeline::new(&device, format, atlas.size());
        let mut draw_list = DrawList::default();
        let physical_size = Vec2::new(WIDTH as f32, HEIGHT as f32);
        let view = View {
            view_projection: camera.view_projection(physical_size),
            physical_size,
            target_is_srgb: format.is_srgb(),
        };
        draw_list.build(world, &view, fonts, atlas);
        sprites.upload_glyphs(&queue, atlas);
        let mut instances = InstanceBuffer::new("test instances");
        instances.upload(&device, &queue, draw_list.instances());

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen target"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(WIDTH * HEIGHT * 4),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some(slice) = instances.slice() {
                pass.set_vertex_buffer(0, slice);
                record_batches(&mut pass, draw_list.batches(), &quads, &sprites);
            }
        }
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
        let pixels = readback.get_mapped_range(..).expect("mapped").to_vec();
        assert!(faults.take().is_none(), "no GPU errors");
        pixels
    }

    /// The pixels the text should produce: each glyph's coverage from the
    /// atlas, at whole-pixel offsets from the snapped origin.
    fn expected_coverage(
        content: &str,
        fonts: &Fonts,
        font: super::super::FontId,
        em_px: f32,
        origin: (i32, i32),
        atlas: &mut GlyphAtlas,
    ) -> Vec<u8> {
        let mut glyphs = Vec::new();
        text::layout(
            content,
            font,
            fonts.get(font).expect("font"),
            em_px,
            crate::render::TextAnchor::BASELINE_LEFT,
            atlas,
            false,
            &mut Vec::new(),
            |g| glyphs.push(g),
        )
        .expect("fits");
        let mut alpha = vec![0_u8; (WIDTH * HEIGHT) as usize];
        for glyph in glyphs {
            for j in 0..glyph.image.size.1 {
                for i in 0..glyph.image.size.0 {
                    let x = origin.0 + glyph.position.0 + i as i32;
                    let y = origin.1 + glyph.position.1 + j as i32;
                    let a = atlas.texel(glyph.image.texel.0 + i, glyph.image.texel.1 + j)[3];
                    if !(0..WIDTH as i32).contains(&x) || !(0..HEIGHT as i32).contains(&y) {
                        continue;
                    }
                    let px = &mut alpha[(y as u32 * WIDTH + x as u32) as usize];
                    // Overlapping glyphs blend: a + b(1 − a).
                    let (s, d) = (f32::from(a) / 255.0, f32::from(*px) / 255.0);
                    *px = ((s + d * (1.0 - s)) * 255.0).round() as u8;
                }
            }
        }
        alpha
    }

    /// White text on black, non-sRGB target: every pixel's red channel must
    /// equal the glyph coverage rasterized on the CPU, placed at whole pixels.
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn gpu_text_matches_the_cpu_rasterization_pixel_for_pixel() {
        let (fonts, font) = super::super::font::tests::poppins();
        let content = "PurplePie: Ag!";
        for (zoom, size, position) in [
            (1.0, 20.0, Vec2::new(-120.3, 10.6)),
            (2.0, 12.0, Vec2::new(-60.2, 2.4)),
        ] {
            let camera = Camera2D::new(Vec2::ZERO, zoom);
            let mut world = World::new();
            world.spawn((
                Transform2D::from_position(position),
                Text::new(content, font, size),
            ));
            let mut atlas = GlyphAtlas::new(256);
            let pixels = render_offscreen(
                &world,
                &fonts,
                &camera,
                wgpu::TextureFormat::Rgba8Unorm,
                &mut atlas,
            );
            let viewport = Vec2::new(WIDTH as f32, HEIGHT as f32);
            let origin = camera.world_to_screen(position, viewport).round();
            let expected = expected_coverage(
                content,
                &fonts,
                font,
                size * zoom,
                (origin.x as i32, origin.y as i32),
                &mut atlas,
            );
            let mut worst = 0;
            let mut lit = 0;
            for (i, want) in expected.iter().enumerate() {
                let got = &pixels[i * 4..i * 4 + 4];
                assert_eq!(got[0], got[1], "white text: r = g");
                assert_eq!(got[0], got[2], "white text: r = b");
                worst = worst.max(got[0].abs_diff(*want));
                lit += u32::from(*want > 0);
            }
            assert!(lit > 300, "zoom {zoom}: the text covers pixels ({lit})");
            assert!(worst <= 1, "zoom {zoom}: worst difference {worst}/255");
        }
    }

    /// Text sits in the same pass as quads: a quad on a higher layer covers
    /// it, and text on the same layer is drawn over a quad.
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn text_follows_layer_order_and_colour() {
        use super::super::{Layer, Quad};
        let (fonts, font) = super::super::font::tests::poppins();
        let mut world = World::new();
        // Left half: blue quad under red text (same layer). Right half: the
        // same text, covered by a green quad on layer 1.
        world.spawn((
            Transform2D::from_position(Vec2::new(-64.0, 0.0)),
            Quad::new(Vec2::new(128.0, 128.0), Color::rgb(0.0, 0.0, 1.0)),
        ));
        for x in [-120.0, 8.0] {
            world.spawn((
                Transform2D::from_position(Vec2::new(x, 0.0)),
                Text::new("HHHH", font, 40.0).with_color(Color::rgb(1.0, 0.0, 0.0)),
            ));
        }
        world.spawn((
            Transform2D::from_position(Vec2::new(64.0, 0.0)),
            Quad::new(Vec2::new(128.0, 128.0), Color::rgb(0.0, 1.0, 0.0)),
            Layer(1),
        ));
        let mut atlas = GlyphAtlas::new(256);
        let pixels = render_offscreen(
            &world,
            &fonts,
            &Camera2D::default(),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &mut atlas,
        );
        let at = |x: u32, y: u32| {
            let i = ((y * WIDTH + x) * 4) as usize;
            [pixels[i], pixels[i + 1], pixels[i + 2]]
        };
        let left: Vec<[u8; 3]> = (0..128)
            .flat_map(|x| (0..HEIGHT).map(move |y| (x, y)))
            .map(|(x, y)| at(x, y))
            .collect();
        assert!(
            left.contains(&[255, 0, 0]),
            "solid red stems over the blue quad"
        );
        assert!(left.contains(&[0, 0, 255]), "blue around the text");
        for x in 128..WIDTH {
            for y in 0..HEIGHT {
                assert_eq!(at(x, y), [0, 255, 0], "the layer-1 quad covers the text");
            }
        }
    }

    /// Anchored text stays inside the rectangle `TextMetrics::bounds` predicts
    /// (±1 px), on the side of the position the anchor says.
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn anchored_text_lands_inside_its_measured_bounds() {
        use super::super::text::{TextAnchor, measure};
        let (fonts, font) = super::super::font::tests::poppins();
        let content = "Hg\nPie";
        let metrics = measure(content, fonts.get(font).expect("font"), 24.0);
        let centre = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
        for anchor in [
            TextAnchor::TOP_LEFT,
            TextAnchor::TOP_RIGHT,
            TextAnchor::BOTTOM_LEFT,
            TextAnchor::BOTTOM_RIGHT,
            TextAnchor::CENTER,
            TextAnchor::BASELINE_CENTER,
        ] {
            let mut world = World::new();
            world.spawn((
                Transform2D::default(),
                Text::new(content, font, 24.0).with_anchor(anchor),
            ));
            let mut atlas = GlyphAtlas::new(256);
            let pixels = render_offscreen(
                &world,
                &fonts,
                &Camera2D::default(),
                wgpu::TextureFormat::Rgba8Unorm,
                &mut atlas,
            );
            // World (+Y up, origin at the centre) → screen pixels (+Y down).
            let (min, max) = metrics.bounds(anchor);
            let (left, right) = (centre.x + min.x - 1.0, centre.x + max.x + 1.0);
            let (top, bottom) = (centre.y - max.y - 1.0, centre.y - min.y + 1.0);
            let mut lit = 0;
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if pixels[((y * WIDTH + x) * 4) as usize] == 0 {
                        continue;
                    }
                    lit += 1;
                    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                    assert!(
                        (left..=right).contains(&px) && (top..=bottom).contains(&py),
                        "{anchor:?}: lit pixel ({x}, {y}) outside {left}..{right} × {top}..{bottom}"
                    );
                }
            }
            assert!(lit > 100, "{anchor:?}: text drawn ({lit} px)");
        }
    }
}
