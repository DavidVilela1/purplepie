# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M6: Sprite Foundation: IN_PROGRESS.** PP-008 (textures + `Sprite`) is VERIFIED on Linux:
sprites from a PNG are pixel-exact under Xvfb + lavapipe. PP-015 (draw order + batching) completes M6.
M5: VERIFIED (Linux; Windows look pending, non-blocking). M4: VERIFIED (Linux + Windows purple window).
M3, M2: VERIFIED (Linux). M1: FUNCTIONAL (Windows `cargo test` and Escape/close still unconfirmed). M0: VERIFIED.

## Current Stage

**Stage 6: Sprite Rendering: in progress.** PP-008 done (2026-10-01). Next: **PP-015** (draw order/layers + batching), which completes Stage 6.

## Overall State

PurplePie opens a window through its own `Engine`/`Game` API and draws game
state. Every entity with `Transform2D` + `render::Quad` is drawn as a solid-colour
rectangle by one instanced wgpu pipeline, then every entity with `Transform2D` + `render::Sprite`
as a textured rectangle. Games load PNGs with `Context::load_texture`, which returns a plain
`TextureId` or a typed `Error::Asset`; the renderer uploads textures itself (ADR-020). World coordinates are +X right, +Y up,
origin at the window centre, and 1 unit = 1 logical pixel (ADR-018). Game logic
runs in a 60 Hz fixed-timestep `fixed_update` plus a per-frame `update` over one
engine-owned `hecs::World`. GPU faults end the loop cleanly as `Error::Render`.
The engine logs via `log`. CI is configured but has not run yet.
There is no draw-order control (quads always under sprites), camera or input abstraction yet.

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | No `unwrap`/`unsafe` in `src/`. `expect` only in tests. |
| `error` (`Error`, `BoxError`, `Result`) | VERIFIED | 3 unit tests + 1 doctest. `Error::Asset` added (PP-008), covered by texture tests. |
| `app::EngineConfig` | VERIFIED | 7 unit tests + 1 doctest |
| `app::Game` / `Context` | VERIFIED | 5 unit tests. `load_texture`, `texture_size` (PP-008). |
| `app` frame pacing (`FramePacer`) | VERIFIED | 5 unit tests. 60 Hz redraw cap (ADR-014). |
| `app::Engine` + runner (winit 0.30 lifecycle) | FUNCTIONAL | Xvfb runs pass. Passes the DPI scale factor to the renderer (also on `ScaleFactorChanged`). Windows: window runs, Escape/close unconfirmed. |
| `time` (`Time`, `FixedTimestep`) | VERIFIED | 12 unit tests |
| `math` (`Transform2D`, `Vec2`, `Mat4`) | VERIFIED | 5 unit tests + 2 doctests. `to_mat4()` added. |
| `ecs` (`World`, `Entity`, `Velocity`, `integrate_velocity`) | VERIFIED | 4 unit tests + 2 doctests |
| `render::Color` | VERIFIED | 4 unit tests + 1 doctest (ADR-015) |
| `render::Quad` + quad pipeline | VERIFIED (Linux) | 6 unit tests + 1 doctest + 2 ignored GPU tests. Xvfb pixel-exact (ADR-018/019). |
| `render::instance` (`Instance`, `InstanceBuffer`) | VERIFIED | 1 unit test. Shared by quads and sprites (PP-008). |
| `render::TextureId` + texture store + PNG decode | VERIFIED | 9 unit tests (ADR-020) |
| `render::Sprite` + sprite pipeline | VERIFIED (Linux) | 3 unit tests + 1 doctest (compile-only) + 2 ignored GPU tests. Xvfb pixel-exact. |
| `render::Renderer` (wgpu) | VERIFIED | Linux: pixel-exact quads and sprites, resize, unmap/map, fault exits, texture upload/sync. Windows: purple window confirmed (Stage 4). |
| `render::faults` (`FaultSlot`, `GpuFault`) | VERIFIED | 4 unit tests + 1 ignored GPU test |
| CI workflow | CONFIGURED, NOT RUN | fmt + clippy (Linux); check + test on Linux/Windows/macOS |
| `input` | NOT_STARTED | Stage 8 (Escape-to-exit is a config flag in `app` until then) |
| `assets` | NOT_STARTED | Stage 9. Textures already have handles and `Error::Asset` (PP-008, ADR-020). |

## Completed

- PP-000: architecture, verified stack, compatibility spike, scaffold.
- PP-001: engineering documentation and task-tracking system.
- PP-002: owner's Windows toolchain builds and runs the scaffold (owner-reported).
- PP-004: time and fixed update (Stage 2).
- PP-005: ECS integration (Stage 3).
- PP-006: GPU context + purple clear (Stage 4).
- PP-014: GPU fault handling + logging decision (Stage 4).
- PP-007: first 2D primitive, quads from ECS (Stage 5).
- PP-008: textures + `Sprite` component with a minimal texture handle (Stage 6, part 1).

## In Progress

- **PP-003: Stage 1 minimal application.** Remaining on Windows: `cargo test`, and Escape/close exiting cleanly. The CI Windows job covers `cargo test` once it runs.

## Next

- **PP-015: Stage 6 · Draw order/layers (PD-08) + sprite batching by texture (PD-05).** See [TASKS.md](TASKS.md#pp-015-draw-orderlayers--sprite-batching-by-texture--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- Rendering is capped at 60 fps by `FramePacer`, even on high-refresh displays (ADR-014). Revisit together with render interpolation.
- Draw order: quads always under sprites, and within each kind ECS query order (unspecified). Sprites alternating between textures cost one draw call each. Fix in PP-015 (PD-08, PD-05).
- Textures are never unloaded and keep a CPU copy (ADR-020, R-17). Relative asset paths depend on the working directory (R-24). Both belong to PD-06 / PP-011.
- `EngineConfig::exit_on_escape` hard-wires one key in `app`. Revisit when the input system exists (PP-010).
- The sandbox reads `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` for automated smoke runs. It is game-side test plumbing, not an engine feature.

## Known Limitations

- Windows: the purple window is confirmed. `cargo test`, Escape/close, vsync pacing and GPU fault paths are unverified there. macOS and Wayland are untested (CI will compile and test on macOS once it runs).
- Quads and sprites are drawn without MSAA, so rotated edges are aliased. Sprites use `Nearest` sampling only (no linear filtering, no mipmaps), so scaled-down or rotated sprites shimmer. There is no camera yet (fixed default view, ADR-018).
- Textures: PNG only. One texture per sprite (no atlas/UV rectangles). A texture larger than the GPU limit (≥ 2048 everywhere) stops the engine with `Error::Asset` at the next frame rather than failing in `load_texture`.
- GPU faults are not recoverable. A lost surface or device ends the game with `Error::Render` (ADR-017).
- The GPU fault test is `#[ignore]` (it needs a GPU), so CI does not run it. Run it with `cargo test -- --ignored`.
- Under lavapipe (software GPU), idle CPU is about 0.75 s per 3 s (1.4 s per 5 s with sprites, PP-008). This is GPU work done on the CPU, not a busy loop.
- Render interpolation is not implemented. `Time::alpha()` is exposed for it, but nothing uses it yet.
- Smoke runs use Xvfb with no window manager, so the close button is simulated by sending `WM_DELETE_WINDOW`.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised (R-19). The code uses let-chains (stable since 1.88).
- Logging only reaches the console if the game installs a `log` backend (ADR-016). The sandbox does, and games using the library must choose their own.
- The owner's copy is inside OneDrive (R-15). No license has been chosen (PP-013).

## Recent Changes

- **2026-10-01: PP-008 Stage 6 textures + sprites.**
  - Added `image 0.25.10` (PNG only; +12 crates, 116 → 128 unique normal dependencies; highest `rust-version` still 1.90).
  - New `Error::Asset { path, source }`. Public `render::TextureId`, `render::Sprite { texture, size, tint }`, `Context::load_texture`, `Context::texture_size`.
  - New crate-private `render/texture.rs` (`Textures` store, `decode_png`), `render/instance.rs` (`Instance` + `InstanceBuffer`, extracted from `quad.rs`), `render/sprite.rs` + `sprite.wgsl`.
  - The runner owns the texture store and lends it to `Context`; the renderer uploads new textures before each frame and draws sprites after quads.
  - Refactor: `QuadInstance` → shared `Instance`, quad pipeline builder → shared `rect_pipeline`. Quad behaviour unchanged (pixel counts identical).
  - Sandbox: `assets/textures/sandbox_quadrants.png` (generated 16×16 test image), one static sprite and one tinted spinning sprite.
  - ADR-020 (texture handle; pulls part of PD-06 forward). PD-05/PD-08 re-targeted to PP-015. New risk R-24 (working-directory-relative asset paths).
  - Drift fixed: ARCHITECTURE's `src/error.rs` row listed only 4 of the 9 error variants.
- **2026-10-01: PP-007 Stage 5 first 2D primitive.**
  - `bytemuck 1.25` added as a direct dependency (already in the tree, so no new crate).
  - `math::Mat4` and `Transform2D::to_mat4()`.
  - Public `render::Quad`. Crate-private `render/quad.rs` + `quad.wgsl`, with an instanced pipeline and no vertex buffer or bind group.
  - The renderer takes `&World` and the DPI scale factor.
  - The sandbox shows three reference quads (one bouncing).
  - ADR-018 (coordinates, resolves the PD-02 core) and ADR-019 (quad pipeline). New pending decision PD-08 (draw order).
  - Stage 6 was split into PP-008 (textures + sprites) and PP-015 (layers + batching).
- **2026-09-30: PP-014 GPU fault handling + logging.**
  - New `src/render/faults.rs` (`FaultSlot`, `GpuFault`) replaces wgpu's panicking uncaptured-error handler and captures device loss.
  - New `Error::Render`. `Validation` and `Lost` acquire results are now fatal (ADR-017).
  - Found and fixed: surface recreation after `Lost` panicked inside wgpu-hal when the window was gone.
  - `log` added as a direct dependency (no new crate). The sandbox has a stderr logger with `PURPLEPIE_LOG` (ADR-016, resolves PD-04).
  - Removed the renderer's unused `instance`/`window` fields, which existed only for recreation.
- **2026-09-30: CI added (owner request, separate task).** `.github/workflows/ci.yml`: fmt + clippy on Linux; `cargo check` + `cargo test` on Linux, Windows and macOS. Not run yet.
- **2026-09-30: Windows confirmation.** The owner's screenshot shows the sandbox window on Windows, and all sampled pixels are `#6A0DAD`. PP-006 is closed.
- **2026-09-30: PP-006 Stage 4 GPU context + purple clear.**
  - Added `wgpu 30.0.1` and `pollster 1.0.1` (116 unique normal dependencies on Linux).
  - New `src/render/`: public `Color` (sRGB) and crate-private `Renderer`.
  - New `Error::{Surface, Adapter, Device, SurfaceUnsupported}`. New `EngineConfig::clear_color` / `with_clear_color`.
  - The runner owns `Arc<Window>` + `Option<Renderer>`: created in `resumed`, resized on `Resized`, rendered after `update`, dropped on `suspended`/`exiting` before the window.
  - ADR-014 (vsync + 60 Hz cap; the pacer is kept) supersedes ADR-010's pacing clause. ADR-015 (sRGB colors) resolves PD-01.
  - Fixed stale doc entries: the ADR-003 index said "no modules yet", and ARCHITECTURE §5 still described `Poll` + `Fifo`.
- **2026-09-30: PP-005 Stage 3 ECS integration.**
  - Added `hecs 0.11.1` and `glam 0.33` (f32 types only), 77 unique normal dependencies on Linux.
  - New `src/math/` (`Transform2D`, `Vec2`) and `src/ecs/` (`World`, `Entity`, `hecs` re-export, `Velocity`, `integrate_velocity`). Both modules are public: `purplepie::math`, `purplepie::ecs`.
  - The runner owns one `World`, exposed via `Context::world()` / `world_mut()`.
  - The sandbox spawns a moving entity and reports its position at exit.
  - Stage 4 was split into PP-006 (GPU context + clear) and PP-014 (GPU errors, device loss, logging), and PD-04 moved to PP-014.
  - Delivery rule: every delivery starts with the `Remove-Item … Claude outputs` command (DEVELOPMENT §10).
- **2026-09-30: PP-004 Stage 2 time & fixed update.**
  - New `src/time/` (`Time`, `FixedTimestep`), `std` only.
  - `Game::fixed_update`. All `Game` callbacks now have defaults (`update` was required before).
  - `Context::time()` / `Context::dt()` (`f32`, the step for the current callback).
  - `EngineConfig::{fixed_dt, max_frame_dt, max_fixed_steps}` with validation. `EngineConfig` no longer derives `Eq`.
  - Runner frame: tick → fixed × n → update. No callbacks after `request_exit`, including mid-way through fixed steps.
  - Public API adds `Time`. No new dependencies.
  - DEVELOPMENT §3/§10: deliveries go only to the chat, and nothing is saved on the owner's computer.
- **2026-09-30: PP-003 Stage 1 implementation.**
  - Added `winit 0.30.13` and `thiserror 2.0.21` (72 unique normal dependencies on Linux).
  - New `src/error.rs` and `src/app/` (`mod`, `config`, `game`, `pacer`, `runner`).
  - Public API: `Engine`, `EngineConfig`, `Game`, `Context`, `Error`, `BoxError`, `Result`, `VERSION`.
  - The sandbox is now a `Game`.
  - Fixed a bug found in testing: game callbacks could run after exit had started. They are now gated on `event_loop.exiting()`.
  - Docs corrected: winit 0.30 uses `tracing`, not `log` (ADR-011 note, TECH_STACK). The ADR-008/010 implementation notes were updated.
  - Owner-requested process rule added to DEVELOPMENT §3: no AI attribution lines in commit messages.
  - Owner rule added to DEVELOPMENT §2/§3/§10: Claude never touches the owner's computer. Every delivery is a full-project ZIP plus the `Expand-Archive` command.
- **2026-09-30: PP-001 documentation system** (DECISIONS/TASKS/PROJECT_STATUS/DEVELOPMENT added; `docs/adr/` merged).
- **2026-09-30: PP-000 Stage 0** architecture and scaffold.

## Validation

Executed in Cowork (Linux x86_64, Rust 1.95.0, Xvfb + Mesa lavapipe / llvmpipe) on 2026-10-01, after the final PP-008 code change:

| Command / check | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ PASS |
| `cargo check --locked --all-targets --all-features` | ✅ PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | ✅ PASS |
| `cargo test --locked` | ✅ PASS: 68 unit tests + 12 doctests, 5 ignored (GPU) |
| `cargo test --locked -- --ignored` (lavapipe) | ✅ PASS: 5/5 (fault capture; quad pipeline 3 formats; broken WGSL captured; sprite pipeline builds + uploads for 3 formats with zero GPU errors and syncs incrementally; oversized texture → `Error::Asset` before wgpu) |
| `cargo build` | ✅ PASS |
| Xvfb screenshot, 1280×720: sprite | ✅ 128×128 at x 576..703, y 176..303. Border 3,840 px = `#6A0DAD` (transparent texels); quadrants 3,136 px each: `#E63946`, `#2A9D8F`, `#F4A261`, and the 50% alpha quadrant = `#5662DB` (linear-space blend, computed independently). **maxdiff 0** in every region. The 1-px ring outside is all background. |
| Xvfb screenshot: quads (Stage 5 regression) | ✅ amber 20,000 px at x 240..439, y 110..209; teal 9,940 px; white 6,400 px (identical to Stage 5) |
| Xvfb: resize to 640×360 | ✅ sprite quadrants still exact (3,136 px each) relative to the new centre; parts outside the smaller window are clipped as expected |
| Xvfb: unmap/map, 1×1, then 800×600 | ✅ keeps running; sprite and quads exact afterwards; Escape → exit 0 |
| Xvfb: 120 frames timed | ✅ 2.12 s, 119 fixed steps, mover at x = 238.0 (= 119 × 2) |
| Texture file missing (moved away) | ✅ `error: failed to load asset `…/sandbox_quadrants.png`` / `caused by: No such file or directory (os error 2)`, exit 1, no panic |
| Texture file not a PNG | ✅ `error: failed to load asset …` / `caused by: Format error decoding Png: Invalid PNG signature.`, exit 1, no panic |
| Xvfb: close button (`WM_DELETE_WINDOW`) | ✅ exit 0 |
| Xvfb: window destroyed mid-run | ✅ `error: GPU rendering failed` / `caused by: the window's GPU surface was lost`, exit 1 |
| No usable GPU backend | ✅ `error: failed to create the GPU surface`, exit 1 |
| Idle CPU (lavapipe) | ✅ 1.40 s per 5 s (Stage 5: ~0.75 s per 3 s); software GPU work, not a busy loop |
| Import check | ✅ `wgpu` only in `render/`, `winit` only in `app/`, `image` only in `render/texture.rs` |

Windows x64, owner-provided (not executed by Claude): the Stage 4 purple window was confirmed by screenshot (pixel-checked).
Stage 5 quads and Stage 6 sprites on Windows have not been seen yet.

## Last Updated

2026-10-01. PP-008 done (Stage 6, part 1). PP-015 is next.
