# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M2: Fixed Simulation: VERIFIED** (unit tests + Xvfb runs on Linux).
**M1: Running Application: FUNCTIONAL.** Verified on Linux and awaiting the owner's
Windows run (PP-003 AC 10). M0: VERIFIED.

## Current Stage

**Stage 2: Time & Fixed Update: complete** (PP-004). Next: **Stage 3: ECS** (PP-005).

## Overall State

PurplePie opens a window through its own `Engine`/`Game` API. It runs a paced
60 Hz frame loop without busy-waiting, and drives the game with a fixed-timestep
`fixed_update` (default 60 Hz, clamped and capped) followed by a per-frame
`update`. It shuts down cleanly on close, on Escape, on `Context::request_exit`,
or on error. There is no ECS, renderer or input abstraction yet.

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | No `unwrap`/`expect`/`unsafe` in `src/` |
| `error` (`Error`, `BoxError`, `Result`) | VERIFIED | 3 unit tests + 1 doctest |
| `app::EngineConfig` | VERIFIED | 7 unit tests + 1 doctest. Validates size and timestep settings. |
| `app::Game` / `Context` | VERIFIED | 3 unit tests. `init`, `fixed_update`, `update` (all optional); `time()`, `dt()`, `request_exit()` |
| `app` frame pacing (`FramePacer`) | VERIFIED | 5 unit tests. About 1% CPU idle under Xvfb. Temporary until Stage 4 vsync. |
| `app::Engine` + runner (winit 0.30 lifecycle) | FUNCTIONAL | Xvfb smoke runs pass. Windows not yet confirmed. |
| `time` (`Time`, `FixedTimestep`) | VERIFIED | 12 unit tests. Xvfb: 59.8 Hz fixed rate. Stall clamp and cap confirmed by probe. |
| `math`, `ecs` | NOT_STARTED | Stage 3, next (PP-005) |
| `render` | NOT_STARTED | Stage 4 |
| `input` | NOT_STARTED | Stage 8 (Escape-to-exit is a config flag in `app` until then) |
| `assets` | NOT_STARTED | Stage 9 |

## Completed

- PP-000: architecture, verified stack, compatibility spike, scaffold.
- PP-001: engineering documentation and task-tracking system.
- PP-002: owner's Windows toolchain builds and runs the scaffold (owner-reported).
- PP-004: time and fixed update (Stage 2).

## In Progress

- **PP-003: Stage 1 minimal application.** Implemented and verified on Linux. Waiting only for the owner to run `cargo test` and `cargo run` on Windows. Any current build covers it.

## Next

- **PP-005: Stage 3 · ECS integration.** See [TASKS.md](TASKS.md#pp-005-ecs-integration--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- `FramePacer` + `ControlFlow::WaitUntil` is interim pacing. Stage 4 (PP-006) replaces it with vsync and removes the pacer if nothing else uses it.
- `EngineConfig::exit_on_escape` hard-wires one key in `app`. Revisit when the input system exists (PP-010).
- The sandbox reads `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` for automated smoke runs. It is game-side test plumbing, not an engine feature.

## Known Limitations

- Windows (the owner's platform) is untested for Stage 1–2 behavior. macOS and Wayland are also untested.
- Render interpolation is not implemented. `Time::alpha()` is exposed for it, but nothing uses it yet.
- Smoke runs use Xvfb with no window manager, so the close button is simulated by sending `WM_DELETE_WINDOW`.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised (R-19). The code uses let-chains (stable since 1.88).
- No logging yet (PD-04 deferred to PP-006). Errors are reported through `Result` and the sandbox prints the source chain.
- The owner's copy is inside OneDrive (R-15). No license has been chosen (PP-013).

## Recent Changes

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

Executed in Cowork (Linux x86_64, Rust 1.95.0) on 2026-09-30, after the final PP-004 code change:

| Command / check | Result |
|---|---|
| `cargo fmt --check` | ✅ PASS |
| `cargo check --all-targets` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo test` | ✅ PASS: 30 unit tests + 5 doctests (2 are compile-only `no_run`) |
| `cargo build` | ✅ PASS |
| Xvfb: sandbox, 120 frames | ✅ 117 fixed steps, 1.966 s game time, exit 0, 1.99 s wall |
| Xvfb: sandbox, 300 frames | ✅ 297 fixed steps / 4.966 s game time (59.8 Hz); simulated time trails game time by one pending step (0.016 s) |
| Xvfb probe: 1 s stall at frame 30 | ✅ delta clamped to 0.250 s, 5 steps (cap), no burst after |
| Xvfb probe: `fixed_dt = 1/120`, cap 8 | ✅ ≈1.95 steps per frame, 8 after the stall |
| Xvfb probe: `request_exit` in `fixed_update` | ✅ 0 further fixed calls, no `update` that frame, `Ok(())` |
| Xvfb probe: `init` context | ✅ `dt = 0`, `frame = 0` |
| Stage 1 regression (idle CPU, `xwininfo`, resize, Escape, other key, close) | ✅ unchanged: 0.04 s CPU / 3 s, 1280×720, all exits code 0 |

Owner-reported on Windows x64 (not executed by Claude): Stage 0 scaffold `clippy` ✅ and `run` ✅.
**Stage 1–2 on Windows has not been reported yet.**

## Last Updated

2026-09-30. PP-004 done. PP-003 still awaits Windows confirmation.
