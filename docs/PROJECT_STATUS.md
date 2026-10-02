# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M8: Input: IN_PROGRESS.** Keyboard input (PP-010) is VERIFIED on Linux; mouse input (PP-016) is next.
M7, M6, M5: VERIFIED (Linux; Windows look pending). M4: VERIFIED (Linux + Windows purple window).
M3, M2: VERIFIED (Linux). M1: VERIFIED (Linux + Windows, owner-confirmed). M0: VERIFIED.

## Current Stage

**Stage 8: Input System: in progress.** PP-010 (keyboard) done 2026-10-02. Next: **PP-016** (mouse), which completes Stage 8.

## Overall State

PurplePie opens a window through its own `Engine`/`Game` API and draws game
state. Every entity with `Transform2D` + `render::Quad` is drawn as a solid-colour
rectangle and every entity with `Transform2D` + `render::Sprite` as a textured rectangle, from one
draw list sorted by the optional `render::Layer` component and batched by texture (ADR-021), as seen through one
engine-owned `render::Camera2D` that games pan and zoom via `Context::camera_mut()` (ADR-022).
Keyboard state comes through `Context::input()` with engine-owned `KeyCode`s; every press reaches `fixed_update`
and `update` exactly once (ADR-024). Games load PNGs with `Context::load_texture`, which returns a plain
`TextureId` or a typed `Error::Asset`; the renderer uploads textures itself (ADR-020). World coordinates are +X right, +Y up,
origin at the window centre, and 1 unit = 1 logical pixel (ADR-018). Game logic
runs in a 60 Hz fixed-timestep `fixed_update` plus a per-frame `update` over one
engine-owned `hecs::World`. GPU faults end the loop cleanly as `Error::Render`.
The engine logs via `log`. CI passes on Linux, Windows and macOS. Licensed MIT OR Apache-2.0.
There is no mouse input yet.

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | No `unwrap`/`unsafe` in `src/`. `expect` only in tests. |
| `error` (`Error`, `BoxError`, `Result`) | VERIFIED | 3 unit tests + 1 doctest. `Error::Asset` added (PP-008), covered by texture tests. |
| `app::EngineConfig` | VERIFIED | 7 unit tests + 1 doctest |
| `app::Game` / `Context` | VERIFIED | 7 unit tests. `load_texture`, `texture_size` (PP-008); `camera`, `camera_mut`, `viewport_size` (PP-009); `input` (PP-010). Borrows one `EngineState`. |
| `app` frame pacing (`FramePacer`) | VERIFIED | 5 unit tests. 60 Hz redraw cap (ADR-014). |
| `app::Engine` + runner (winit 0.30 lifecycle) | VERIFIED | Xvfb runs pass. Passes the DPI scale factor to the renderer (also on `ScaleFactorChanged`). Owns the camera and an event-driven logical viewport (PP-009). Windows: window, Escape and close confirmed by the owner. |
| `time` (`Time`, `FixedTimestep`) | VERIFIED | 12 unit tests |
| `math` (`Transform2D`, `Vec2`, `Mat4`) | VERIFIED | 5 unit tests + 2 doctests. `to_mat4()` added. |
| `ecs` (`World`, `Entity`, `Velocity`, `integrate_velocity`) | VERIFIED | 4 unit tests + 2 doctests |
| `render::Color` | VERIFIED | 4 unit tests + 1 doctest (ADR-015) |
| `render::Quad` + quad pipeline | VERIFIED (Linux) | 6 unit tests + 1 doctest + 2 ignored GPU tests. Xvfb pixel-exact (ADR-018/019). |
| `render::Camera2D` | VERIFIED (Linux) | 7 unit tests + 1 doctest (incl. DPI 1.0/1.25/2.0). Xvfb whole-frame checks (ADR-022). |
| `render::Layer` + draw list (`render/draw.rs`) | VERIFIED (Linux) | 7 unit tests + 1 doctest. Xvfb overlap checks, 0 mismatches (ADR-021). |
| `render::instance` (`Instance`, `InstanceBuffer`) | VERIFIED | 1 unit test. Shared by quads and sprites (PP-008). |
| `render::TextureId` + texture store + PNG decode | VERIFIED | 9 unit tests (ADR-020) |
| `render::Sprite` + sprite pipeline | VERIFIED (Linux) | 2 unit tests + 1 doctest (compile-only) + 2 ignored GPU tests. Xvfb pixel-exact. (The batching test moved to `draw.rs`.) |
| `render::Renderer` (wgpu) | VERIFIED | Linux: pixel-exact quads and sprites, resize, unmap/map, fault exits, texture upload/sync. Windows: purple window confirmed (Stage 4). |
| `render::faults` (`FaultSlot`, `GpuFault`) | VERIFIED | 4 unit tests + 1 ignored GPU test |
| CI workflow | VERIFIED (owner-reported) | fmt + clippy (Linux); check + test on Linux/Windows/macOS: first run all green, 2026-10-01 |
| `input` (`Input`, `KeyCode`) + `app::keymap` | VERIFIED (Linux) | 10 + 3 unit tests + 1 doctest; Xvfb XTEST runs (ADR-024). Mouse: PP-016. |
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
- PP-015: draw order (`Layer`) + batching by texture (Stage 6, part 2).
- PP-009: `Camera2D` + screen ↔ world mapping (Stage 7).
- PP-003: Stage 1 closed after the owner confirmed Windows `cargo test`, Escape and close.
- PP-013: license chosen: MIT OR Apache-2.0.
- PP-010: keyboard input (Stage 8, part 1).

## In Progress

- Nothing. (PP-003 closed on 2026-10-01.)

## Next

- **PP-016: Stage 8 · Mouse input.** See [TASKS.md](TASKS.md#pp-016-mouse-input--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- Rendering is capped at 60 fps by `FramePacer`, even on high-refresh displays (ADR-014). Revisit together with render interpolation.
- The draw list is rebuilt and sorted every frame (O(n log n)), even when nothing changed: ~1.8 ms per 10,000 drawables in release (R-18).
- Textures are never unloaded and keep a CPU copy (ADR-020, R-17). Relative asset paths depend on the working directory (R-24). Both belong to PD-06 / PP-011.
- `EngineConfig::exit_on_escape` hard-wires one key in `app`, now redundant with `Input` but kept for compatibility (ADR-024). Revisit in Stage 10 (API refinement).
- The sandbox reads `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` for automated smoke runs. It is game-side test plumbing, not an engine feature.

## Known Limitations

- Windows (owner): the purple window (Stage 4), `cargo test`, Escape and the close button are confirmed. Stages 5–7 rendering, vsync pacing and GPU fault paths are unverified there. macOS compiles and passes `cargo test` in CI but its rendering has never been seen; Wayland is untested.
- Keyboard: 99 physical keys only (no text input, gamepads or rebinding). An edge reaches `fixed_update` one frame late when the frame that saw it ran no fixed step.
- One camera only, without rotation, and no screen-space (UI) layer that ignores it. A non-integer zoom with `Nearest` sampling makes texels uneven (1 vs 2 pixels at zoom 1.5).
- Quads and sprites are drawn without MSAA, so rotated edges are aliased. Sprites use `Nearest` sampling only (no linear filtering, no mipmaps), so scaled-down or rotated sprites shimmer. 
- Textures: PNG only. One texture per sprite (no atlas/UV rectangles). A texture larger than the GPU limit (≥ 2048 everywhere) stops the engine with `Error::Asset` at the next frame rather than failing in `load_texture`.
- GPU faults are not recoverable. A lost surface or device ends the game with `Error::Render` (ADR-017).
- The GPU fault test is `#[ignore]` (it needs a GPU), so CI does not run it. Run it with `cargo test -- --ignored`.
- Under lavapipe (software GPU), idle CPU is about 0.75 s per 3 s (1.4 s per 5 s with sprites, PP-008). This is GPU work done on the CPU, not a busy loop.
- Render interpolation is not implemented. `Time::alpha()` is exposed for it, but nothing uses it yet.
- Smoke runs use Xvfb with no window manager, so the close button is simulated by sending `WM_DELETE_WINDOW`.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised (R-19). The code uses let-chains (stable since 1.88).
- Logging only reaches the console if the game installs a `log` backend (ADR-016). The sandbox does, and games using the library must choose their own.
- The owner's copy is inside OneDrive (R-15).

## Recent Changes

- **2026-10-02: PP-010 Stage 8 keyboard input.**
  - New public module `input`: `KeyCode` (99 physical keys) and `Input` (`pressed`, `just_pressed`, `just_released`, `axis`, `pressed_keys`); `Context::input()`.
  - Edges are kept separately for `fixed_update` (latched until the first fixed step) and `update` (per frame), so each press reaches each callback exactly once (ADR-024, resolves PD-03; R-05 mitigated).
  - New `app/keymap.rs` (winit → `KeyCode`) and `app/state.rs` (`EngineState`; `Context::new` takes it instead of six arguments). Focus loss releases held keys; repeats and X11 synthetic presses are ignored; key/focus events logged at `debug`.
  - Sandbox: arrows pan the camera, `=` / `-` zoom; the timed exit prints the `=` press counts per callback. No new dependencies.
- **2026-10-01: owner confirmations recorded (no code changes).**
  - PP-003 DONE: on Windows, `cargo test` passes and Escape / the close button exit cleanly. M1 is VERIFIED.
  - CI: the first GitHub Actions run passed every job (fmt + clippy on Linux; check + test on Linux, Windows, macOS). R-23 closed.
  - PP-013 DONE: MIT OR Apache-2.0 (ADR-023): `license` in `Cargo.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, README License section.
- **2026-10-01: PP-009 Stage 7 camera (Stage 7 complete).**
  - New public `render::Camera2D { position, zoom }` with `screen_to_world` / `world_to_screen` (logical pixels, top-left origin, +Y down). Default = the ADR-018 view.
  - The runner owns one camera and a logical viewport; new `Context::camera()`, `camera_mut()`, `viewport_size()`. `Renderer::render` takes `&Camera2D`; `quad::view_projection` moved to `Camera2D::view_projection`.
  - Regression found and fixed before delivery: a per-frame `window.inner_size()` panicked inside winit when the X11 window was destroyed. The viewport is now updated from window events only (R-22, TECH_STACK).
  - Sandbox: `PURPLEPIE_SANDBOX_CAMERA=x,y,zoom`; the timed exit prints viewport and camera.
  - ADR-022 resolves PD-02. Stage 8 split into PP-010 (keyboard) and PP-016 (mouse). No new dependencies.
- **2026-10-01: PP-015 Stage 6 draw order + batching (Stage 6 complete).**
  - New public `render::Layer(pub i32)` component (optional, default 0, higher = on top).
  - New crate-private `render/draw.rs`: `DrawList` sorts quads and sprites by (layer, material, entity index) and builds one batch per (layer, material) run.
  - One shared instance buffer for quads and sprites; the pipelines became pipeline + bind-group holders; `collect_instances` / `collect_sprites` removed.
  - Sandbox: a yellow quad (layer 0) under the sprite's top-left corner and a pink quad (layer 1) over its bottom-right corner.
  - ADR-021 resolves PD-05 and PD-08 and supersedes ADR-020's "sprites after quads" clause. No new dependencies.
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

Executed in Cowork (Linux x86_64, Rust 1.95.0, Xvfb + Mesa lavapipe / llvmpipe) on 2026-10-02, after the final PP-010 code change:

| Command / check | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ PASS |
| `cargo check --locked --all-targets --all-features` | ✅ PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | ✅ PASS |
| `cargo test --locked` | ✅ PASS: 96 unit tests + 15 doctests, 5 ignored (GPU) |
| `cargo test --locked -- --ignored` (lavapipe) | ✅ PASS: 5/5 |
| `cargo doc --no-deps` | ✅ no warnings |
| `cargo build --locked` | ✅ PASS |
| Mutation check | ✅ clearing fixed-step edges per frame (the naive design) fails 3 input tests |
| Xvfb XTEST keys (focus, 0.5 s settle; hold → 0.5 s, hold ↑ 0.3 s, tap `=` ×3, tap `-`) | ✅ 3/3 runs identical: camera (155, 95) zoom 4; `'=' presses seen: 3 in fixed_update, 3 in update`; screenshot vs per-pixel model at that camera: **0 mismatches** (844,776 px). 5 more runs without debug logging and without the settle delay: identical |
| Xvfb: focus removed while → held | ✅ pan stopped at focus loss: camera x = 170 ≈ 0.576 s × 300 px/s (the real key-up came 1.5 s later); log shows the synthetic release + `focus lost` |
| Xvfb: OS auto-repeat | ✅ 15 repeat events logged while → was held; no extra presses |
| One anomalous run | ⚠️ the very first keyboard run lost the → press (camera x = 0, y = 35). Not reproduced in 14 later runs; probably a focus race in WM-less Xvfb, unconfirmed (R-11) |
| Xvfb: Escape exits; `q`, Space, Enter, F1 don't | ✅ |
| Xvfb: whole-frame model at cameras (0,0)×1 and (0,120)×2 | ✅ 0 mismatches (Stage 7 regression) |
| Xvfb: window destroyed mid-run | ✅ `error: GPU rendering failed` / `caused by: the window's GPU surface was lost`, exit 1 |
| Xvfb: 120 frames timed; unmap/map, 1×1 → 800×600; close button | ✅ exit 0; 119 steps, mover at 238.0; keeps running; close → exit 0 |
| Texture file missing / no GPU backend | ✅ clean errors, exit 1 |
| Idle CPU (lavapipe) | ✅ 1.46 s per 5 s; software GPU work, not a busy loop |

Owner-provided (not executed by Claude): Windows x64: the Stage 4 purple window was confirmed by screenshot (pixel-checked); `cargo test`, Escape and the close button confirmed on 2026-10-01. GitHub Actions: first run all green on 2026-10-01.
Stages 5–8 on Windows have not been seen yet.

## Last Updated

2026-10-02. PP-010 done (Stage 8, part 1). PP-016 is next.
