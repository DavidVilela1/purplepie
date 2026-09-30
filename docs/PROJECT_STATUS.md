# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M0: Architecture Ready: VERIFIED.** Next milestone: **M1: Running Application: NOT_STARTED.**

## Current Stage

**Stage 0: Architecture & Planning: complete.** Stage 1 has not started and begins only on explicit request.

## Overall State

PurplePie is a compiling, dependency-free scaffold with a complete
architecture, roadmap, decision log and task tracker. There is **no engine
functionality yet**: no window, loop, ECS, renderer or input.

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | Active in `Cargo.toml` |
| Documentation system | VERIFIED | This file, TASKS, DECISIONS, ROADMAP, ARCHITECTURE, RISKS, DEVELOPMENT |
| Version/compatibility research | VERIFIED | Spike built and ran headless (outside `src/`) |
| `error` module | NOT_STARTED | Stage 1 |
| `app` (window, lifecycle) | NOT_STARTED | Stage 1 |
| `time` (fixed timestep) | NOT_STARTED | Stage 2 |
| `math`, `ecs` | NOT_STARTED | Stage 3 |
| `render` | NOT_STARTED | Stage 4 |
| `input` | NOT_STARTED | Stage 8 |
| `assets` | NOT_STARTED | Stage 9 |

## Completed

- PP-000: architecture, verified stack (winit 0.30.13, wgpu 30.0.1, hecs 0.11.1, glam 0.33, pollster 1.0.1, thiserror 2), compatibility spike, scaffold.
- PP-001: engineering documentation and task-tracking system.
- PP-002: owner's Windows toolchain builds and runs the scaffold (owner-reported).

## In Progress

- None.

## Next

- **PP-003: Stage 1 · Minimal application (window + lifecycle).** See [TASKS.md](TASKS.md#pp-003-minimal-application-window--lifecycle--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- `version_matches_manifest` asserts the literal `"0.0.0"`, so it will fail on the first version bump. Replace it or remove it when real tests exist (Stage 1–2).
- The module table in `src/lib.rs` duplicates ARCHITECTURE §3, which is a drift risk (R-16). Replace it with real module docs as modules appear.

## Known Limitations

- No runtime engine features. `sandbox` only prints its version.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised (R-19).
- Cowork validation is Linux-only and headless. Windows validation depends on the owner (R-11).
- Windows `cargo test` has not been reported since the MSVC fix (it is part of PP-003).
- The owner's working copy is inside OneDrive (R-15).
- No license has been chosen (PP-013).
- No git repository is initialized in the Cowork workspace. History lives in stage archives.

## Recent Changes

- **2026-09-30:** PP-001 documentation system.
  - Added `PROJECT_STATUS.md`, which replaces `STATUS.md`.
  - Added `TASKS.md`, `DECISIONS.md` and `DEVELOPMENT.md`.
  - `DECISIONS.md` replaces `docs/adr/` (10 files). ADR-001…013 now include Rust, glam, module boundaries, renderer ownership and dependency policy, which were previously implicit.
  - The former "Proposed" ADRs (color space, coordinates, input) became Pending Decisions PD-01…03.
  - ARCHITECTURE, ROADMAP and RISKS were rewritten to separate current from planned state, and milestones M0–M10 were added.
  - `Cargo.toml` and `README.md` doc links were updated.
- **2026-09-30:** Owner hit `link.exe not found` on Windows. Resolved by installing the VS C++ workload (R-14).
- **2026-09-30:** Stage 0 architecture and scaffold (PP-000).

## Validation

Executed in Cowork (Linux x86_64, Rust 1.95.0) on 2026-09-30 after the documentation update:

| Command | Result |
|---|---|
| `cargo fmt --check` | ✅ exit 0 |
| `cargo check --all-targets` | ✅ exit 0 |
| `cargo clippy --all-targets -- -D warnings` | ✅ exit 0 |
| `cargo test` | ✅ 1 unit test + 1 doctest passed |
| `cargo build` | ✅ exit 0 |
| `cargo run` | ✅ `PurplePie sandbox v0.0.0 (Stage 0 scaffold: no window yet, see docs/ROADMAP.md)` |

Owner-reported on Windows x64 (not executed by Claude):
`cargo clippy --all-targets -- -D warnings` ✅. `cargo test` ❌ `link.exe not found` before the fix.
`cargo run` ✅ after installing the VS C++ workload.

## Last Updated

2026-09-30. PP-001 completed, end of Stage 0.
