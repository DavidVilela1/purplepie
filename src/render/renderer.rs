//! GPU context and frame presentation (ADR-005, ADR-009, ADR-014, ADR-019, ADR-020, ADR-021).
//!
//! The only code in the engine that touches `wgpu`. It receives the window
//! as generic `raw-window-handle` providers, so it does not depend on `winit`.

use std::sync::Arc;

use super::Color;
use super::Textures;
use super::draw::{Batch, DrawList, Material};
use super::faults::{FaultSlot, GpuFault};
use super::instance::InstanceBuffer;
use super::quad::{QuadPipeline, view_projection};
use super::sprite::SpritePipeline;
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
    /// One GPU instance buffer shared by quads and sprites, in draw order.
    instances: InstanceBuffer,
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
        let sprites = SpritePipeline::new(&device, config.format);
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
    /// `Transform2D` + `Quad` or `Sprite`, ordered by `Layer` (ADR-021).
    /// Textures in `textures` that are not on the GPU yet are uploaded first;
    /// one the GPU cannot hold returns `Err(Error::Asset)` (ADR-020).
    pub(crate) fn render(
        &mut self,
        world: &World,
        textures: &Textures,
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

        let logical_size = Vec2::new(self.config.width as f32, self.config.height as f32)
            / self.scale_factor as f32;
        self.draw_list.build(
            world,
            &view_projection(logical_size),
            self.config.format.is_srgb(),
        );
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
