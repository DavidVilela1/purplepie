# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M1: Running Application: FUNCTIONAL.** Verified on Linux/Xvfb. It becomes
VERIFIED when the owner confirms the window on Windows (PP-003 AC 10).
M0: VERIFIED.

## Current Stage

**Stage 1: Minimal Application.** Implementation complete, awaiting the owner's Windows run.
Next: Stage 2 (Time & Fixed Update, PP-004).

## Overall State

PurplePie opens a window through its own `Engine`/`Game` API. It runs a paced
60 Hz frame loop without busy-waiting, and shuts down cleanly on close, on
Escape, on `Context::request_exit`, or on error. There is no timestep, ECS,
renderer or input abstraction yet.

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | No `unwrap`/`expect`/`unsafe` in `src/` |
| `error` (`Error`, `BoxError`, `Result`) | VERIFIED | 3 unit tests + 1 doctest |
| `app::EngineConfig` | VERIFIED | 5 unit tests + 1 doctest. Rejects 0×N sizes. |
| `app::Game` / `Context` | VERIFIED | 2 unit tests. `init` + `update`; `request_exit` |
| `app` frame pacing (`FramePacer`) | VERIFIED | 5 unit tests. About 1% CPU idle under Xvfb. Temporary until Stage 4 vsync. |
| `app::Engine` + runner (winit 0.30 lifecycle) | FUNCTIONAL | Xvfb smoke runs pass (see Validation). Windows not yet confirmed. |
| `time` (fixed timestep) | NOT_STARTED | Stage 2, next |
| `math`, `ecs` | NOT_STARTED | Stage 3 |
| `render` | NOT_STARTED | Stage 4 |
| `input` | NOT_STARTED | Stage 8 (Escape-to-exit is a config flag in `app` until then) |
| `assets` | NOT_STARTED | Stage 9 |

## Completed

- PP-000: architecture, verified stack, compatibility spike, scaffold.
- PP-001: engineering documentation and task-tracking system.
- PP-002: owner's Windows toolchain builds and runs the scaffold (owner-reported).

## In Progress

- **PP-003: Stage 1 minimal application.** Implemented and verified on Linux. Waiting only for the owner to run `cargo test` and `cargo run` on Windows.

## Next

- **PP-004: Stage 2 · Time & fixed update.** See [TASKS.md](TASKS.md#pp-004-time--fixed-update--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- `FramePacer` + `ControlFlow::WaitUntil` is interim pacing. Stage 4 (PP-006) replaces it with vsync and removes the pacer if nothing else uses it.
- `EngineConfig::exit_on_escape` hard-wires one key in `app`. Revisit when the input system exists (PP-010).
- The sandbox reads `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` for automated smoke runs. It is game-side test plumbing, not an engine feature.
- Resolved this session: the hard-coded `"0.0.0"` version test was removed, and the duplicated module table in `src/lib.rs` was removed (R-16).

## Known Limitations

- Windows (the owner's platform) is untested for Stage 1 behavior. macOS and Wayland are also untested.
- Smoke runs use Xvfb with no window manager, so the close button is simulated by sending `WM_DELETE_WINDOW`.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised (R-19). The code uses let-chains (stable since 1.88).
- No logging yet (PD-04 deferred to PP-006). Errors are reported through `Result` and the sandbox prints the source chain.
- The owner's copy is inside OneDrive (R-15). No license has been chosen (PP-013).

## Recent Changes

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

Executed in Cowork (Linux x86_64, Rust 1.95.0) on 2026-09-30, after the final code change:

| Command / check | Result |
|---|---|
| `cargo fmt --check` | ✅ PASS |
| `cargo check --all-targets` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo test` | ✅ PASS: 15 unit tests + 5 doctests (2 are compile-only `no_run`) |
| `cargo build` | ✅ PASS |
| Sandbox without a display | ✅ `error: event loop error` + cause chain, exit 1 |
| Sandbox with invalid `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` | ✅ clear message, exit 1 |
| Xvfb: 120 frames with timed exit | ✅ exit 0 in 1.99 s (≈ 60 Hz) |
| Xvfb: window attributes (`xwininfo`) | ✅ "PurplePie Sandbox", 1280×720 |
| Xvfb: idle CPU over ~3 s | ✅ 0.04 s CPU (no busy loop) |
| Xvfb: resize to 640×360 | ✅ keeps running |
| Xvfb: Escape (XTEST) | ✅ exit 0 |
| Xvfb: non-Escape key | ✅ keeps running |
| Xvfb: close request (`WM_DELETE_WINDOW`) | ✅ exit 0 |
| Throwaway game: `init` returns error | ✅ `Engine::run` → `Error::Game("level file missing")`, `update` never called |
| Throwaway game: `request_exit` in first `update` | ✅ `update` ran exactly once, `Ok(())` |
| Zero-width config / second `Engine::new` | ✅ `InvalidConfig` / `EventLoop("EventLoop can't be recreated")` |

Owner-reported on Windows x64 (not executed by Claude): Stage 0 scaffold `clippy` ✅ and `run` ✅.
**Stage 1 on Windows has not been reported yet.**

## Last Updated

2026-09-30. PP-003 implemented, awaiting Windows confirmation.
