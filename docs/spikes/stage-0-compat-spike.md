# Stage 0 Compatibility Spike

**Purpose:** prove that the chosen crate versions compile together and run,
using the API shapes PurplePie's architecture assumes. This is **reference
material, not engine code**. It is not compiled as part of PurplePie and
deliberately ignores the module structure. It was built in a scratch
directory outside the project.

## Result (2026-09-30, Rust 1.95.0, Linux x86_64)

| Check | Result |
|---|---|
| `cargo check` | ✅ passed (after one fix: `RequestAdapterOptions` gained `apply_limit_buckets` in wgpu 30, so use `..Default::default()`) |
| `cargo clippy` | ✅ no warnings |
| `cargo build` | ✅ passed |
| Run without a display | ✅ clean `Err(EventLoop(..))` ("neither WAYLAND_DISPLAY nor … DISPLAY is set"), no panic |
| Run under `xvfb-run` + Mesa lavapipe | ✅ 800×600 window, 480,000 pixels `#A059C4` (purple clear) |
| `cargo tree -d` | one `raw-window-handle` (0.6.2); no conflicting windowing crates |

Manifest:

```toml
[package]
name = "spike"
edition = "2024"

[dependencies]
glam = "0.33"
hecs = "0.11.1"
pollster = "1.0.1"
thiserror = "2.0"
wgpu = "30.0.1"
winit = "0.30.13"
```

## Source (`src/main.rs`)

```rust
//! Throwaway Stage 0 compatibility spike. NOT part of PurplePie.
//! Proves that the chosen versions of winit, wgpu, hecs, glam, pollster and
//! thiserror compile together with the API shapes the architecture assumes.

use std::sync::Arc;
use std::time::Instant;

use glam::Vec2;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Debug, thiserror::Error)]
enum SpikeError {
    #[error("event loop error: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error("window creation failed: {0}")]
    Window(#[from] winit::error::OsError),
    #[error("surface creation failed: {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error("no suitable GPU adapter: {0}")]
    Adapter(#[from] wgpu::RequestAdapterError),
    #[error("device request failed: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("surface is not supported by the selected adapter")]
    UnsupportedSurface,
}

#[derive(Debug, Clone, Copy)]
struct Transform2D {
    position: Vec2,
    rotation: f32,
    scale: Vec2,
}

#[derive(Debug, Clone, Copy)]
struct Velocity(Vec2);

const FIXED_DT: f64 = 1.0 / 60.0;
const MAX_STEPS: u32 = 5;

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
}

impl Gpu {
    fn new(
        display: winit::event_loop::OwnedDisplayHandle,
        window: Arc<Window>,
    ) -> Result<Self, SpikeError> {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display)));
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("spike device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                .using_resolution(adapter.limits()),
            ..Default::default()
        }))?;
        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or(SpikeError::UnsupportedSurface)?;
        surface.configure(&device, &config);
        Ok(Self { surface, device, queue, config, window })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) => f,
            wgpu::CurrentSurfaceTexture::Suboptimal(f) => {
                // Present this one; reconfigure next frame.
                f
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.35, g: 0.1, b: 0.55, a: 1.0 }),
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
        self.window.pre_present_notify();
        self.queue.present(frame);
    }
}

struct App {
    gpu: Option<Gpu>,
    world: hecs::World,
    last: Instant,
    acc: f64,
    error: Option<SpikeError>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let result = event_loop
            .create_window(Window::default_attributes().with_title("spike"))
            .map_err(SpikeError::from)
            .and_then(|w| Gpu::new(event_loop.owned_display_handle(), Arc::new(w)));
        match result {
            Ok(gpu) => self.gpu = Some(gpu),
            Err(e) => {
                self.error = Some(e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                self.acc += (now - self.last).as_secs_f64();
                self.last = now;
                let mut steps = 0;
                while self.acc >= FIXED_DT && steps < MAX_STEPS {
                    for (t, v) in self.world.query_mut::<(&mut Transform2D, &Velocity)>() {
                        t.position += v.0 * FIXED_DT as f32;
                        t.rotation += 0.0;
                        let _ = t.scale;
                    }
                    self.acc -= FIXED_DT;
                    steps += 1;
                }
                if steps == MAX_STEPS {
                    self.acc = 0.0;
                }
                if let Some(gpu) = &mut self.gpu {
                    gpu.render();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(gpu) = &self.gpu {
            gpu.window.request_redraw();
        }
    }
}

fn main() -> Result<(), SpikeError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut world = hecs::World::new();
    world.spawn((
        Transform2D { position: Vec2::ZERO, rotation: 0.0, scale: Vec2::ONE },
        Velocity(Vec2::new(1.0, 0.0)),
    ));
    let mut app = App { gpu: None, world, last: Instant::now(), acc: 0.0, error: None };
    event_loop.run_app(&mut app)?;
    match app.error.take() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}
```
