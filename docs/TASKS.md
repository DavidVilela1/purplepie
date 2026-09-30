# PurplePie Tasks

Active task tracker. Rules are in [DEVELOPMENT.md §5](DEVELOPMENT.md#5-tasks).

* **Status:** `TODO`, `IN_PROGRESS`, `BLOCKED`, `DONE`, `CANCELLED`.
* **Priority:** `P1` (next), `P2` (upcoming stages), `P3` (later).
* A task is `DONE` only when the [Definition of Done](DEVELOPMENT.md#4-definition-of-done) is met,
  not merely because code exists.
* Future stages have one coarse task each. Split a task only when its stage becomes current.

## Current

- [ ] **PP-005: Stage 3 · ECS integration** · P1 · TODO ← **next task**

## In Progress

- [ ] **PP-003: Stage 1 · Minimal application (window + lifecycle)** · P1 · IN_PROGRESS
  All work is implemented and verified on Linux/Xvfb. **The only remaining item is AC 10: the owner runs `cargo test` + `cargo run` on Windows.**

## Blocked

_None._

## Completed

- [x] **PP-000: Stage 0 · Architecture, version verification, compatibility spike, scaffold** · DONE
- [x] **PP-001: Stage 0 · Engineering documentation and task-tracking system** · DONE
- [x] **PP-002: Stage 0 · Windows toolchain able to build and run the scaffold** · DONE
- [x] **PP-004: Stage 2 · Time & fixed update** · DONE (2026-09-30)

## Future

- [ ] **PP-006: Stage 4 · wgpu initialization + purple clear** · P2 · TODO
- [ ] **PP-007: Stage 5 · First 2D primitive (quad from ECS)** · P3 · TODO
- [ ] **PP-008: Stage 6 · Textures & sprite rendering** · P3 · TODO
- [ ] **PP-009: Stage 7 · Camera2D & coordinates** · P3 · TODO
- [ ] **PP-010: Stage 8 · Input system** · P3 · TODO
- [ ] **PP-011: Stage 9 · Assets & resources** · P3 · TODO
- [ ] **PP-012: Stage 10 · Engine/game API refinement with an example game** · P3 · TODO
- [ ] **PP-013: Owner · Choose project license** · P3 · TODO

---

## Task details

### PP-000: Architecture, version verification, compatibility spike, scaffold
| Field | Value |
|---|---|
| Stage | 0 · Priority P1 · **DONE** (2026-09-30) |
| Dependencies | — |
| Acceptance criteria | ✅ Crate versions verified on crates.io and in source. ✅ Spike compiled, built and ran headless with a purple clear. ✅ ECS chosen (ADR-006). ✅ lib + `sandbox` scaffold passes fmt, check, clippy `-D warnings`, test and build. |
| Evidence | [spikes/stage-0-compat-spike.md](spikes/stage-0-compat-spike.md), PROJECT_STATUS validation log |

### PP-001: Engineering documentation and task-tracking system
| Field | Value |
|---|---|
| Stage | 0 · Priority P1 · **DONE** (2026-09-30) |
| Dependencies | PP-000 |
| Acceptance criteria | ✅ ARCHITECTURE, ROADMAP, PROJECT_STATUS, TASKS, DECISIONS, RISKS and DEVELOPMENT exist and match the repository. ✅ ADRs consolidated into DECISIONS.md, with unmade decisions listed as pending. ✅ One next task selected. ✅ Validation re-run. ✅ `PurplePie-stage-0.zip` rebuilt and verified. |

### PP-002: Windows toolchain able to build and run the scaffold
| Field | Value |
|---|---|
| Stage | 0 · Owner environment · **DONE** (2026-09-30, owner-reported) |
| Dependencies | PP-000 |
| Acceptance criteria | ✅ Owner-reported: `cargo clippy --all-targets -- -D warnings` passed. After the VS C++ workload was installed, `cargo run` printed the sandbox line, which proves linking works. ⚠️ Owner has not reported `cargo test` on Windows since the fix. That check is included in PP-003. |

### PP-003: Minimal application (window + lifecycle)
| Field | Value |
|---|---|
| Stage | 1 → Milestone M1 · Priority P1 · **IN_PROGRESS**. Implemented 2026-09-30. AC 1–9 and 11 met. AC 10 awaits the owner. |
| Dependencies | PP-000, PP-002 (both DONE). No blockers. |
| Scope | Add `winit 0.30.13` and `thiserror 2`. Create `src/error.rs` (`Error`, `Result`) and `src/app/` (`EngineConfig` with title/size builders; `Engine::new`/`run`; `Game` trait with `init` + `update` only; `Context` with `request_exit()`; `Runner` implementing `ApplicationHandler`). Update the sandbox to `Engine::new(..)?.run(Sandbox)`. Decide PD-04 (logger). |
| Out of scope | Fixed timestep, ECS, GPU, input abstraction |
| Result | `src/error.rs`, `src/app/{mod,config,game,pacer,runner}.rs`, sandbox rewritten, 15 unit tests + 5 doctests. Xvfb smoke runs: window 1280×720 titled "PurplePie Sandbox". About 0.04 s CPU over 3 s idle. Resize OK. Escape → exit 0. Close (WM_DELETE_WINDOW) → exit 0. Other keys ignored. 120 frames in 1.99 s. `init` error returned as `Error::Game` with no `update` afterwards. `update` never runs after `request_exit`. Zero-size config rejected. Second `Engine::new` → `Error::EventLoop`. No display → error chain, exit 1. PD-04 deferred to PP-006 (nothing to log yet). |
| Bug found and fixed | `event_loop.exit()` does not stop queued events, so `update` ran after a failed `init`. Game callbacks are now gated on `!event_loop.exiting()`. |
| Acceptance criteria | 1. Window created only in `resumed`, exactly once. 2. Close button and Escape exit with code 0. 3. `Resized` handled (0×0 tolerated). 4. `about_to_wait` → `request_redraw`; `RedrawRequested` calls `game.update`. 5. `ControlFlow::WaitUntil` pacing, no busy loop. 6. An error raised in a callback is returned from `Engine::run`. 7. No `unwrap()` in engine code. 8. Unit tests for `EngineConfig`. 9. Checks in Cowork: fmt, check, clippy `-D warnings`, test, build, plus an Xvfb smoke run with automated exit. 10. Owner runs `cargo test` + `cargo run` on Windows and confirms the window. 11. Docs, PROJECT_STATUS and TASKS updated. `PurplePie-stage-1.zip` delivered. |

### PP-004: Time & fixed update
| Field | Value |
|---|---|
| Stage | 2 → Milestone M2 · Priority P1 · **DONE** (2026-09-30) |
| Dependencies | PP-003 (implemented; its Windows run is still outstanding and did not block this work) |
| Scope | New `src/time/` (`Time`; pure `FixedTimestep` in `time/fixed.rs`). `Game::fixed_update`. `Context::time()` / `Context::dt()`. `EngineConfig::{fixed_dt, max_frame_dt, max_fixed_steps}` with validation. Runner wired per ADR-010. Sandbox reports fixed steps. |
| Acceptance criteria | ✅ 1. `time` depends only on `std`. ✅ 2. Unit tests: 0, 1 and N steps, carry-over, cap + backlog clamp, bad deltas, 60 Hz long run, alpha in [0, 1) over 10k irregular frames, `Time` clamping/counters, config validation. ✅ 3. Loop order: tick → fixed × n → update, and no callbacks after exit is requested. ✅ 4. Xvfb: 297 fixed steps / 4.966 s game time (59.8 Hz). ✅ 5. Docs updated. |
| Extra verification | Throwaway probe game under Xvfb: 1 s stall → delta clamped to 0.25 s → 5 steps (cap). At 120 Hz: ~1.95 steps per frame and a cap of 8 after the stall. `request_exit` inside `fixed_update` → 0 further fixed calls and no `update` that frame. `init` sees `dt = 0`, `frame = 0`. |
| API notes | `update` became a default method (it was required in Stage 1), so all three callbacks are optional. `EngineConfig` lost `Eq` (it now has `f64` fields), keeping `PartialEq`. |

### PP-005: ECS integration ← NEXT
| Field | Value |
|---|---|
| Stage | 3 → Milestone M3 · Priority P1 · **TODO** |
| Dependencies | PP-004 (DONE) |
| Scope | Add `hecs 0.11.1` + `glam 0.33` (re-verify versions first, per ADR-013). New `src/math/` (`Transform2D { position: Vec2, rotation: f32, scale: Vec2 }`, re-export `Vec2`). New `src/ecs/` (re-export `World`/`Entity`, `Velocity`, `integrate_velocity(world, dt)`). Runner owns one `World`, and `Context::world()` gives `&mut World`. Sandbox spawns an entity and moves it in `fixed_update`. |
| Acceptance criteria | 1. Tests: spawn, attach components, query, `integrate_velocity` moves by `v·dt`, `Transform2D` defaults. 2. `ecs`/`math` have no `app`/`winit`/render imports. 3. The game calls systems explicitly (no implicit engine systems). 4. Xvfb smoke: the sandbox entity's position after N fixed steps equals `N·v·fixed_dt`. 5. DoD and docs updated. |

### PP-006: wgpu initialization + purple clear
| Stage 4 → M4 · P2 · TODO | Depends on PP-003 (PP-004/005 expected done first) |
|---|---|
| Acceptance criteria | Init chain with typed errors. Acquire-result policy (ARCHITECTURE §7). Resize/minimize safe. Device-lost capture. PD-01 and PD-04 decided. Replace `FramePacer`/`WaitUntil` with vsync pacing (remove the pacer if unused). Xvfb pixel check. Owner confirms on a real GPU. |

### PP-007: First 2D primitive
| Stage 5 → M5 · P3 · TODO | Depends on PP-005, PP-006 |
|---|---|
| Acceptance criteria | A quad driven by an ECS `Transform2D`. WGSL pipeline. Renderer read-only on the world. Transform → matrix tests. |

### PP-008: Textures & sprite rendering
| Stage 6 → M6 · P3 · TODO | Depends on PP-007 |
|---|---|
| Acceptance criteria | PNG textures, `Sprite` component, instanced batching (PD-05 decided). Smoke run. |

### PP-009: Camera2D & coordinates
| Stage 7 → M7 · P3 · TODO | Depends on PP-008 |
|---|---|
| Acceptance criteria | PD-02 decided as an ADR. Projection + `screen_to_world` tests, including DPI. |

### PP-010: Input system
| Stage 8 → M8 · P3 · TODO | Depends on PP-004, PP-009 |
|---|---|
| Acceptance criteria | PD-03 decided. Own key types. Edge semantics tested with 0, 1 and N fixed steps. |

### PP-011: Assets & resources
| Stage 9 → M9 · P3 · TODO | Depends on PP-008 |
|---|---|
| Acceptance criteria | PD-06 decided. Handle store tests. Clear `Error::Asset` on missing files. |

### PP-012: Engine/game API refinement
| Stage 10 → M10 · P3 · TODO | Depends on PP-010, PP-011 |
|---|---|
| Acceptance criteria | Example game in `examples/`. ADR-008 reviewed. Workspace-split decision recorded. |

### PP-013: Choose project license
| Owner decision · P3 · TODO | No dependencies. Does not block engineering. |
|---|---|
| Acceptance criteria | `license` in `Cargo.toml` + `LICENSE` file(s). PD-07 closed. |
