# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M4: GPU Foundation: VERIFIED.** Pixel-exact on Linux (Xvfb + lavapipe), and the
purple window was confirmed on Windows with a real GPU by the owner's screenshot
(pixel-checked). GPU fault handling was verified on Linux. Next milestone: **M5: First 2D Primitive: NOT_STARTED.**
M3, M2: VERIFIED (Linux). M1: FUNCTIONAL (Windows `cargo test` and Escape/close still unconfirmed). M0: VERIFIED.

## Current Stage

**Stage 4: WGPU Initialization: complete** (PP-006 + PP-014). Next: **Stage 5: First 2D Primitive** (PP-007).

## Overall State

PurplePie opens a window through its own `Engine`/`Game` API. The engine owns a
wgpu renderer that clears the window to `EngineConfig::clear_color` (default
`#6A0DAD`) every frame, with vsync (`AutoVsync`) and a 60 Hz redraw cap. Game
logic runs in a fixed-timestep `fixed_update` (60 Hz) plus a per-frame
`update`, over one engine-owned `hecs::World`. GPU faults (uncaptured wgpu
errors, device loss, lost surface) no longer panic. They end the loop cleanly
as `Error::Render`. The engine logs via `log`, and the sandbox prints logs with `PURPLEPIE_LOG`.
CI (`.github/workflows/ci.yml`) is configured but has not run yet. Entities are not drawn yet.

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | No `unwrap`/`unsafe` in `src/`. `expect` only in tests. |
| `error` (`Error`, `BoxError`, `Result`) | VERIFIED | 3 unit tests + 1 doctest. `Render` variant added. |
| `app::EngineConfig` | VERIFIED | 7 unit tests + 1 doctest |
| `app::Game` / `Context` | VERIFIED | 4 unit tests |
| `app` frame pacing (`FramePacer`) | VERIFIED | 5 unit tests. 60 Hz redraw cap (ADR-014). |
| `app::Engine` + runner (winit 0.30 lifecycle) | FUNCTIONAL | Xvfb runs pass. On Windows the window runs (owner screenshot). Escape/close and `cargo test` are unconfirmed there. |
| `time` (`Time`, `FixedTimestep`) | VERIFIED | 12 unit tests |
| `math` (`Transform2D`, `Vec2`) | VERIFIED | 2 unit tests + 1 doctest |
| `ecs` (`World`, `Entity`, `Velocity`, `integrate_velocity`) | VERIFIED | 4 unit tests + 2 doctests |
| `render::Color` | VERIFIED | 4 unit tests + 1 doctest (ADR-015) |
| `render::Renderer` (wgpu) | VERIFIED | Linux: pixel-exact, resize, unmap/map, fault exits. Windows: purple window confirmed. |
| `render::faults` (`FaultSlot`, `GpuFault`) | VERIFIED | 4 unit tests + 1 ignored GPU test (passes under lavapipe) |
| CI workflow | CONFIGURED, NOT RUN | fmt + clippy (Linux); check + test on Linux/Windows/macOS |
| `input` | NOT_STARTED | Stage 8 (Escape-to-exit is a config flag in `app` until then) |
| `assets` | NOT_STARTED | Stage 9 |

## Completed

- PP-000: architecture, verified stack, compatibility spike, scaffold.
- PP-001: engineering documentation and task-tracking system.
- PP-002: owner's Windows toolchain builds and runs the scaffold (owner-reported).
- PP-004: time and fixed update (Stage 2).
- PP-005: ECS integration (Stage 3).
- PP-006: GPU context + purple clear (Stage 4). Closed with the owner's Windows screenshot.
- PP-014: GPU fault handling + logging decision (Stage 4).

## In Progress

- **PP-003: Stage 1 minimal application.** Remaining on Windows: `cargo test`, and Escape/close exiting cleanly. The CI Windows job covers `cargo test` once it runs.

## Next

- **PP-007: Stage 5 · First 2D primitive (quad from ECS).** See [TASKS.md](TASKS.md#pp-007-first-2d-primitive-quad-from-ecs--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- Rendering is capped at 60 fps by `FramePacer`, even on high-refresh displays (ADR-014). Revisit together with render interpolation.
- `EngineConfig::exit_on_escape` hard-wires one key in `app`. Revisit when the input system exists (PP-010).
- The sandbox reads `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` for automated smoke runs. It is game-side test plumbing, not an engine feature.

## Known Limitations

- Windows: the purple window is confirmed. `cargo test`, Escape/close, vsync pacing and GPU fault paths are unverified there. macOS and Wayland are untested (CI will compile and test on macOS once it runs).
- GPU faults are not recoverable. A lost surface or device ends the game with `Error::Render` (ADR-017).
- The GPU fault test is `#[ignore]` (it needs a GPU), so CI does not run it. Run it with `cargo test -- --ignored`.
- Under lavapipe (software GPU), idle CPU is about 0.75 s per 3 s. This is GPU work done on the CPU, not a busy loop.
- Render interpolation is not implemented. `Time::alpha()` is exposed for it, but nothing uses it yet.
- Smoke runs use Xvfb with no window manager, so the close button is simulated by sending `WM_DELETE_WINDOW`.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised (R-19). The code uses let-chains (stable since 1.88).
- Logging only reaches the console if the game installs a `log` backend (ADR-016). The sandbox does, and games using the library must choose their own.
- The owner's copy is inside OneDrive (R-15). No license has been chosen (PP-013).

## Recent Changes

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

Executed in Cowork (Linux x86_64, Rust 1.95.0, Xvfb + Mesa lavapipe / llvmpipe) on 2026-09-30, after the final PP-014 code change:

| Command / check | Result |
|---|---|
| `cargo fmt --check` | ✅ PASS |
| `cargo check --all-targets` | ✅ PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ PASS |
| `cargo test` | ✅ PASS: 45 unit tests + 9 doctests, 1 ignored (GPU) |
| `cargo test -- --ignored` (lavapipe, headless) | ✅ PASS: a deliberately invalid buffer is captured as `Uncaptured(Validation)`; `destroy()` is not a fault |
| Control: same invalid call without PurplePie's handler | ✅ panics (`wgpu error: Validation Error`, exit 101), which proves the test exercises the fix |
| `cargo build` | ✅ PASS |
| Xvfb: X window destroyed under the running sandbox (3 runs) | ✅ `error: GPU rendering failed` / `caused by: the window's GPU surface was lost`, exit 1, no panic |
| Xvfb: pixels (1280×720 / 640×360 / 1600×900) | ✅ exactly `#6A0DAD`: 921,600 / 230,400 / 1,440,000 px |
| Xvfb: unmap/map, Escape, close, other key | ✅ unchanged, exit codes 0 |
| Xvfb: 300 frames timed | ✅ 5.08 s (60 Hz cap); mover = 298 · 1/60 |
| `PURPLEPIE_LOG=info` / default / invalid | ✅ GPU line printed / silent / clear error, exit 1 |
| No usable GPU backend | ✅ wgpu loader errors logged, then `error: failed to create the GPU surface`, exit 1 |
| Import check | ✅ `wgpu` only in `render/`, `winit` only in `app/` |

Windows x64, owner-provided (not executed by Claude): screenshot of the running sandbox window. Every sampled pixel in the window area is `(106, 13, 173)` = `#6A0DAD` (checked by Claude from the image).
Earlier: Stage 0 scaffold `clippy` ✅ and `run` ✅ (owner-reported).

## Last Updated

2026-09-30. PP-014 done, Stage 4 complete. PP-007 is next.
