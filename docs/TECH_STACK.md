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
| `bytemuck` | **1.25** ✅ added (1.25.2, `derive`; already in the tree via wgpu, so no new crate) | 5 | `Pod` instance data → bytes (ADR-019) | — |
| `log` | **0.4** ✅ added (0.4.34; already in the tree via wgpu, so no new crate) | 4 | diagnostics facade (ADR-016). Note that winit 0.30 itself logs through `tracing`. | — |
| `image` | **0.25.10** ✅ added (`default-features = false, features = ["png"]`; +12 crates: `png`, `moxcms`, `pxfm`, `flate2`, `fdeflate`, `miniz_oxide` ×2, …; 116 → 128 unique normal dependencies) | 6 | PNG decoding for textures (ADR-020) | 1.88 |
| `ab_glyph` | **0.2.32** ✅ added (`default-features = false, features = ["std"]`; pulls `ab_glyph_rasterizer` 0.1.10, `owned_ttf_parser` 0.25.1, `ttf-parser` 0.25.1; Apache-2.0 / MIT OR Apache-2.0). **+0 crates on Linux** (already in the tree via winit's Wayland decorations, still 128) and **+4 on Windows** (99 → 103), measured with `cargo tree -e normal --target …` on 2026-10-06 | PP-018a | font parsing + glyph rasterization (ADR-027) | 1.63 (`ttf-parser`; the others declare none) |
| `cpal` | **0.18.2** ✅ added (default features; Linux: `alsa` 0.11.0 + `alsa-sys` 0.4.0, needs `libasound2-dev` at build time; macOS: CoreAudio crates; Windows: WASAPI via the `windows` crate already in the tree). With `hound`: **+5 crates Linux (128 → 133), +3 Windows (103 → 106), +9 macOS (99 → 108)**, measured 2026-10-07 | PP-022 | audio output device (ADR-030) | 1.85 |
| `hound` | **3.5.1** ✅ added (Apache-2.0, no dependencies) | PP-022 | WAV decoding (ADR-030) | — |
| `lewton` | **0.10.2** ✅ added (MIT OR Apache-2.0, pure Rust, default `ogg` feature; pulls `ogg` 0.8.0 **BSD-3-Clause**, `tinyvec` 1.13.3 Zlib/Apache-2.0/MIT, `byteorder` 1.5.0 Unlicense/MIT). **+4 crates on every platform: Linux 133 → 137, Windows 106 → 110, macOS 108 → 112**, measured with `cargo tree -e normal --target …` on 2026-10-07 | PP-024b | OGG Vorbis decoding (ADR-033) | — (`byteorder` 1.60) |
| `ron` | **0.12.2** ✅ added (MIT OR Apache-2.0) | PP-026a | scene file format (ADR-035) | 1.64 |
| `serde` | **1.0.229** ✅ added (`derive`; pulls `serde_core`, `serde_derive`; MIT OR Apache-2.0). With `ron` (+`typeid`): **+5 crates on every platform: Linux 137 → 142, Windows 110 → 115, macOS 112 → 117**, measured 2026-10-07. Used only by the crate-private `scene` module: no public type derives serde | PP-026a | scene (de)serialization via private mirror types (ADR-035) | 1.56 (`serde_derive` 1.71) |

**Declared `rust-version = "1.90"`.** This is the highest `rust-version` found
in the resolved Stage 1–4 dependency graph (`ordered-float 5.5.0` via
`wgpu-hal`). Still the highest after PP-008 (`image` declares 1.88, `moxcms`/`pxfm` 1.85) and PP-018a (`ttf-parser` 1.63) and PP-022 (`cpal` 1.85) and PP-024b (`byteorder` 1.60; `lewton`, `ogg`, `tinyvec` declare none) and PP-026a (`serde_derive` 1.71, `ron` 1.64). Only 1.95.0 has actually been exercised: the sandbox could not
download other toolchains. Raise the value if a lower toolchain fails.

## Versions deliberately not chosen

| Candidate | Reason |
|---|---|
| `winit 0.31.0-beta.3` | Pre-release. 0.31 redesigns the API (trait-based windows, `can_create_surfaces`). Revisit when 0.31.0 is stable, as a dedicated migration (ADR-004 in [DECISIONS.md](DECISIONS.md)). |
| `bevy_ecs 0.19.1` / `0.20.0-rc` | ~76 transitive deps vs 4 for hecs, MSRV 1.95, breaking release every ~3–5 months (ADR-006 in [DECISIONS.md](DECISIONS.md)). |
| `tokio` / `async-std` | No async I/O requirement (ADR-012 in [DECISIONS.md](DECISIONS.md)). |
| `anyhow` in the engine | The engine exposes typed errors. Games may use anyhow themselves. |
| `fontdue 0.9.4` | Fine rasterizer, but +4 crates on Linux and +5 on Windows (a second `hashbrown`) where `ab_glyph` costs +0/+4 (ADR-027). |
| `rodio 0.22.2` | Convenient playback API, but +20/+18/+22 crates (Linux/Windows/macOS) even with only `playback` + `wav`, and its decoders come from `symphonia` (MPL-2.0) (ADR-030). |
| `symphonia` | Many formats (MP3, FLAC, Vorbis, …), but MPL-2.0 and a larger tree than one format needs (ADR-033). Revisit for MP3/FLAC. |
| `serde_json` 1.0.151 / `toml` 1.1.6 (scenes) | +6/+7 and +9 crates with serde (vs +5 for `ron`); JSON has no comments and is noisy to hand-edit, TOML is awkward for lists of nested tables (ADR-035). |
| `kira 0.12.5` | Game-oriented audio (tweens, clocks, tracks), but +28/+25/+29 crates; revisit for music features (ADR-030). |
| `glyphon` / `cosmic-text` | Shaping and font fallback, but a large tree, its own wgpu pipeline locked to wgpu versions, and text outside the draw list (ADR-027). Revisit for non-Latin scripts. |

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
* **X11 pitfall (PP-009):** `Window::inner_size()` makes an X server round trip and `unwrap`s the result, so it panics
  if the window was destroyed externally. Take sizes from `WindowEvent::Resized` instead of querying every frame.
* **X11 wheel (PP-016):** wheel buttons 4–7 are turned into `MouseWheel` on both press **and** release unless XI2 flags them as
  emulated; XTEST clicks are not flagged, so `xdotool click 4` produces two `LineDelta(0, 1)` events.
* **X11 external destroy (PP-016):** `DestroyNotify` handling calls `remove_context(..).expect(..)` for the IME input context, which can
  panic with BadDrawable when the window was destroyed by another client (intermittent; see R-22).

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

## glam 0.33 note (PP-007)

`Mat4::orthographic_rh` and the other projection constructors on `Mat4` are deprecated in glam 0.33.
Use `glam::camera::{lh,rh}::proj::{opengl,vulkan,directx}::*` instead. For WebGPU (Y-up NDC, depth [0, 1]), use
`glam::camera::rh::proj::directx::orthographic`. It is available with `default-features = false, features = ["std"]`.
`cargo add bytemuck@1 --features derive` failed against the newest index entry ("unrecognized feature"), so the
dependency was written by hand as `{ version = "1.25", features = ["derive"] }`, matching the locked 1.25.2.

## image 0.25 notes (PP-008)

- The default features pull in every format plus `rayon`. PurplePie uses `default-features = false, features = ["png"]`.
  Even then, `image` 0.25.10 depends on `moxcms` (colour management) and `pxfm`, which is why it costs 4 crates more than
  the `png` crate alone (`png` 0.18.1: +8). ADR-020 records why `image` was still chosen.
- `image::load_from_memory_with_format(bytes, ImageFormat::Png)?.into_rgba8()` handles every PNG colour type: grey,
  palette and RGB gain an opaque alpha, 16-bit channels become 8-bit (`(v + 128) / 257`). The `png` feature includes the encoder,
  which the unit tests use to build PNGs in memory.
- `zlib-rs` appears in `Cargo.lock` (an optional `flate2` backend) but is not compiled: `cargo tree -i zlib-rs` prints nothing.
- wgpu 30: `wgpu::util::DeviceExt::create_texture_with_data(queue, desc, TextureDataOrder::LayerMajor, bytes)` uploads
  a texture in one call. `write_texture` needs no 256-byte row alignment (only buffer-to-texture copies do).
