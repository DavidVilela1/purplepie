# Technology Stack

Versions were verified on **2026-09-30** against crates.io and by compiling a
throwaway spike ([spikes/stage-0-compat-spike.md](spikes/stage-0-compat-spike.md)).

| Crate | Version | Added in | Why | MSRV (crate metadata) |
|---|---|---|---|---|
| Rust (edition 2024) | stable, **1.95.0** used | 0 | — | — |
| `winit` | **0.30.13** ✅ added | 1 | window + event loop | 1.70 |
| `thiserror` | **2.0.21** ✅ added | 1 | structured `Error` enum | 1.77 |
| `hecs` | **0.11.1** ✅ added | 3 | ECS (ADR-006 in [DECISIONS.md](DECISIONS.md)), default features (`std`) | 1.81 |
| `glam` | **0.33** (0.33.11) ✅ added | 3 | `Vec2`, `Mat4`, `Affine2`; `default-features = false, features = ["std"]`, which omits the f64/integer types of the default `all-types` feature | 1.68.2 |
| `wgpu` | **30.0.1** ✅ added | 4 | GPU abstraction; default features (all native backends + `wgsl`) | 1.87 |
| `pollster` | **1.0.1** ✅ added | 4 | block on wgpu init futures (ADR-012 in [DECISIONS.md](DECISIONS.md)) | 1.69 |
| `log` | **0.4** ✅ added (0.4.34; already in the tree via wgpu, so no new crate) | 4 | diagnostics facade (ADR-016). Note that winit 0.30 itself logs through `tracing`. | — |
| `bytemuck` | TBD | 5 | vertex/uniform casting | decide in Stage 5 |
| `image` | TBD (PNG only) | 6/9 | texture decoding | decide in Stage 6 |

**Declared `rust-version = "1.90"`.** This is the highest `rust-version` found
in the resolved Stage 1–4 dependency graph (`ordered-float 5.5.0` via
`wgpu-hal`). Only 1.95.0 has actually been exercised: the sandbox could not
download other toolchains. Raise the value if a lower toolchain fails.

## Versions deliberately not chosen

| Candidate | Reason |
|---|---|
| `winit 0.31.0-beta.3` | Pre-release. 0.31 redesigns the API (trait-based windows, `can_create_surfaces`). Revisit when 0.31.0 is stable, as a dedicated migration (ADR-004 in [DECISIONS.md](DECISIONS.md)). |
| `bevy_ecs 0.19.1` / `0.20.0-rc` | ~76 transitive deps vs 4 for hecs, MSRV 1.95, breaking release every ~3–5 months (ADR-006 in [DECISIONS.md](DECISIONS.md)). |
| `tokio` / `async-std` | No async I/O requirement (ADR-012 in [DECISIONS.md](DECISIONS.md)). |
| `anyhow` in the engine | The engine exposes typed errors. Games may use anyhow themselves. |

## Compatibility evidence

* One `raw-window-handle` in the graph (0.6.2), shared by winit 0.30.13 and
  wgpu 30.0.1. `cargo tree -d` shows no duplicate here.
* The spike compiled with `cargo check`, `cargo clippy` and `cargo build`
  (Rust 1.95.0, Linux x86_64). It **ran** under Xvfb with Mesa lavapipe
  (software Vulkan): an 800×600 window cleared to purple (480,000 pixels
  `#A059C4`).
* Not exercised: Windows, macOS, Wayland, real GPUs.

## wgpu 30 API notes (differ from most tutorials)

| Area | wgpu 30.0.1 | Older tutorials |
|---|---|---|
| Instance | `InstanceDescriptor::new_with_display_handle(Box::new(event_loop.owned_display_handle()))`, which GLES/Wayland needs | `InstanceDescriptor::default()` |
| Adapter | `request_adapter(..)` → `Result<Adapter, RequestAdapterError>`; `RequestAdapterOptions` has `apply_limit_buckets`, so use `..Default::default()` | `Option<Adapter>` |
| Device | `request_device(&DeviceDescriptor)` (no trace path arg); fields include `experimental_features`, `memory_hints`, `trace` | `request_device(&desc, None)` |
| Acquire | `surface.get_current_texture()` → `CurrentSurfaceTexture::{Success, Suboptimal, Timeout, Occluded, Outdated, Lost, Validation}` | `Result<SurfaceTexture, SurfaceError>` |
| Present | `queue.present(surface_texture)` | `surface_texture.present()` |
| Surface config | `SurfaceConfiguration` has `color_space` | — |
| Render pass | `RenderPassDescriptor::multiview_mask`, `RenderPassColorAttachment::depth_slice` | — |
| OOM / device loss | `Device::on_uncaptured_error`, `push_error_scope(ErrorFilter::OutOfMemory)`, `set_device_lost_callback` | `SurfaceError::OutOfMemory` |

## winit 0.30 API notes

* `EventLoop::new()? .run_app(&mut impl ApplicationHandler)`
* Create windows only inside `resumed` (`ActiveEventLoop::create_window`).
* Frame driving: `about_to_wait` → `window.request_redraw()`; draw in
  `WindowEvent::RedrawRequested`; call `window.pre_present_notify()` before
  presenting.
* `ActiveEventLoop::exit()` for shutdown; `exiting` for final cleanup.

## hecs 0.11 API note

`World::query_mut::<(&mut A, &B)>()` yields component tuples **without** the
`Entity` (verified in the spike). Add `Entity` to the query tuple when the id
is needed.

## wgpu observations from PP-006 (Xvfb + Mesa lavapipe, llvmpipe LLVM 20)

- Surface formats offered: `[Bgra8UnormSrgb, Bgra8Unorm]`. Present modes: `[Immediate, Mailbox, Fifo, FifoRelaxed]`.
  `get_default_config` picks the **first** present mode (`Immediate`), so PurplePie sets `AutoVsync` explicitly (ADR-014).
- `Fifo` did not throttle on this path: 544 fps with `ControlFlow::Poll`.
- wgpu 30's default uncaptured-error handler panics (`backend/wgpu_core.rs: default_error_handler`). PP-014 replaces it (ADR-017).
- **wgpu-hal 30.0.1 panics** when recreating a Vulkan surface for a destroyed X11 window (`vulkan/instance.rs:407`, `create_xlib_surface … ERROR_OUT_OF_HOST_MEMORY`), rather than returning an error. PurplePie therefore treats `Lost` as fatal (ADR-017).
- Passing the display through `InstanceDescriptor` means surfaces are created with `SurfaceTarget::from_window_without_display`.
  `wgpu::WindowHandle` is `HasWindowHandle + Send + Sync`, so `Arc<dyn wgpu::WindowHandle>` works as a winit-free window parameter.
- With no usable backend (Vulkan ICDs and EGL vendors hidden), `create_surface` fails with "Failed to create surface for any enabled backend", which becomes `Error::Surface` and exit 1.

