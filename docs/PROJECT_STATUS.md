# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M4: GPU Foundation: IN_PROGRESS.** PP-006 (GPU context + purple clear) is
FUNCTIONAL and verified pixel-exact on Linux (Xvfb + lavapipe). PP-014 (GPU error
and device-loss handling) is next. The owner's Windows run is pending.
M3, M2: VERIFIED (Linux). M1: FUNCTIONAL (Windows run pending). M0: VERIFIED.

## Current Stage

**Stage 4: WGPU Initialization: in progress** (PP-006 implemented; PP-014 next).

## Overall State

PurplePie opens a window through its own `Engine`/`Game` API. The engine owns a
wgpu renderer that clears the window to `EngineConfig::clear_color` (default
`#6A0DAD`) every frame, with vsync (`AutoVsync`) and a 60 Hz redraw cap. Game
logic runs in a fixed-timestep `fixed_update` (60 Hz) plus a per-frame
`update`, over one engine-owned `hecs::World`. It shuts down cleanly on close,
on Escape, on `request_exit`, or on error, including GPU setup failure.
Entities are not drawn yet. **Known gap:** an uncaptured wgpu error currently
panics through wgpu's default handler (fix: PP-014).

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | No `unwrap`/`unsafe` in `src/`. `expect` only in tests. |
| `error` (`Error`, `BoxError`, `Result`) | VERIFIED | 3 unit tests + 1 doctest. GPU variants added. |
| `app::EngineConfig` | VERIFIED | 7 unit tests + 1 doctest. Size, timestep, `clear_color`. |
| `app::Game` / `Context` | VERIFIED | 4 unit tests |
| `app` frame pacing (`FramePacer`) | VERIFIED | 5 unit tests. Kept as the 60 Hz redraw cap (ADR-014). |
| `app::Engine` + runner (winit 0.30 lifecycle) | FUNCTIONAL | Xvfb runs pass. Windows not yet confirmed. |
| `time` (`Time`, `FixedTimestep`) | VERIFIED | 12 unit tests. |
| `math` (`Transform2D`, `Vec2`) | VERIFIED | 2 unit tests + 1 doctest |
| `ecs` (`World`, `Entity`, `Velocity`, `integrate_velocity`) | VERIFIED | 4 unit tests + 2 doctests |
| `render::Color` | VERIFIED | 4 unit tests + 1 doctest. sRGB → linear per target format (ADR-015). |
| `render::Renderer` (wgpu) | FUNCTIONAL | Xvfb + lavapipe: pixel-exact clear, resize, unmap/map. Windows and real GPUs not confirmed. Uncaptured-error handling missing (PP-014). |
| `input` | NOT_STARTED | Stage 8 (Escape-to-exit is a config flag in `app` until then) |
| `assets` | NOT_STARTED | Stage 9 |

## Completed

- PP-000: architecture, verified stack, compatibility spike, scaffold.
- PP-001: engineering documentation and task-tracking system.
- PP-002: owner's Windows toolchain builds and runs the scaffold (owner-reported).
- PP-004: time and fixed update (Stage 2).
- PP-005: ECS integration (Stage 3).

## In Progress

- **PP-003: Stage 1 minimal application.** Waiting only for the owner's Windows run (`cargo test` + `cargo run`).
- **PP-006: Stage 4 GPU context + purple clear.** Waiting only for the owner to confirm the purple window on Windows (real GPU). The same run closes PP-003.

## Next

- **PP-014: Stage 4 · GPU error & device-loss handling + logging decision.** See [TASKS.md](TASKS.md#pp-014-gpu-error--device-loss-handling--logging-decision--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- Rendering is capped at 60 fps by `FramePacer`, even on high-refresh displays (ADR-014). Revisit together with render interpolation.
- wgpu's default uncaptured-error handler panics, device loss is undetected, and `Validation` acquire results are skipped silently (PP-014).
- `EngineConfig::exit_on_escape` hard-wires one key in `app`. Revisit when the input system exists (PP-010).
- The sandbox reads `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` for automated smoke runs. It is game-side test plumbing, not an engine feature.

## Known Limitations

- Windows (the owner's platform) is untested for Stage 1–4 behavior. macOS, Wayland and real GPUs are also untested. Vsync on real hardware is unverified.
- Under lavapipe (software GPU), idle CPU is about 0.75 s per 3 s. This is GPU work done on the CPU, not a busy loop.
- Render interpolation is not implemented. `Time::alpha()` is exposed for it, but nothing uses it yet.
- Smoke runs use Xvfb with no window manager, so the close button is simulated by sending `WM_DELETE_WINDOW`.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised (R-19). The code uses let-chains (stable since 1.88).
- No logging yet (PD-04 → PP-014). Errors are reported through `Result` and the sandbox prints the source chain.
- The owner's copy is inside OneDrive (R-15). No license has been chosen (PP-013).

## Recent Changes

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

Executed in Cowork (Linux x86_64, Rust 1.95.0, Xvfb + Mesa lavapipe / llvmpipe) on 2026-09-30, after the final PP-006 code change:

| Command / check | Result |
|---|---|
| `cargo fmt --check` | ✅ PASS |
| `cargo check --all-targets` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo test` | ✅ PASS: 41 unit tests + 9 doctests, 0 ignored |
| `cargo build` | ✅ PASS |
| Import check | ✅ `wgpu` only in `render/`, `winit` only in `app/`; `math`/`ecs`/`time` unchanged |
| Xvfb: initial frame, screenshot histogram | ✅ 921,600 px (= 1280×720) exactly `#6A0DAD` |
| Xvfb: resize 640×360 / 1×1 → 1600×900 | ✅ 230,400 / 1,440,000 px exactly `#6A0DAD`; no panic |
| Xvfb: unmap 2 s, then map | ✅ keeps running, repaints purple |
| Xvfb: 300 frames timed | ✅ 5.1 s (60 Hz cap holds); mover = 299 · 1/60 = 4.9833 |
| Xvfb: idle CPU | ✅ ~0.75 s / 3 s (software GPU work; uncapped `Fifo` measured 544 fps) |
| Stage 1–3 regression (Escape, close, other key, resize, `xwininfo`) | ✅ unchanged, exit codes 0 |
| No usable GPU backend (Vulkan ICDs + EGL vendors hidden) | ✅ `error: failed to create the GPU surface` + cause, exit 1, no panic |
| Sandbox without a display | ✅ error chain, exit 1 |

Owner-reported on Windows x64 (not executed by Claude): Stage 0 scaffold `clippy` ✅ and `run` ✅.
**Stages 1–4 on Windows have not been reported yet.**

## Last Updated

2026-09-30. PP-006 implemented. PP-003 and PP-006 await Windows confirmation. PP-014 is next.
