//! GPU context and frame presentation (ADR-005, ADR-009, ADR-014).
//!
//! The only code in the engine that touches `wgpu`. It receives the window
//! as generic `raw-window-handle` providers, so it does not depend on `winit`.

use std::sync::Arc;

use super::Color;
use super::faults::{FaultSlot, GpuFault};
use crate::error::{Error, Result};

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
}

impl Renderer {
    /// Runs the wgpu initialization chain for `window`.
    ///
    /// `display` must be the display connection the window belongs to (winit's
    /// `OwnedDisplayHandle`). `width`/`height` are the window's physical size.
    pub(crate) fn new(
        display: impl wgpu::wgt::WgpuHasDisplayHandle,
        window: Arc<dyn wgpu::WindowHandle>,
        width: u32,
        height: u32,
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
        })
    }

    /// Call when the window's physical size changes. A zero width or height
    /// (minimized window) pauses rendering until a real size arrives.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
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
    pub(crate) fn render(&mut self, before_present: impl FnOnce()) -> Result<()> {
        // Faults raised since the last frame (e.g. by `resize`) are reported first.
        check(&self.faults)?;
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

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("purplepie frame"),
            });
        {
            let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("purplepie clear"),
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
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        before_present();
        self.queue.present(frame);
        check(&self.faults)
    }
}

/// Converts a recorded GPU fault into `Error::Render`.
fn check(faults: &FaultSlot) -> Result<()> {
    match faults.take() {
        Some(fault) => Err(Error::Render(Box::new(fault))),
        None => Ok(()),
    }
}
