# PurplePie Roadmap

A **living** engineering roadmap. It describes the intended order of work,
not a promise. When reality diverges, update this file ([DEVELOPMENT.md §7](DEVELOPMENT.md#7-architectural-drift)).
Task details and live status are in [TASKS.md](TASKS.md). The current state is in [PROJECT_STATUS.md](PROJECT_STATUS.md).

## Principles

Order of priority: architectural foundations → working vertical slices →
continuous compilation → testability → minimal dependencies → real integration →
incremental complexity.

Avoid: abstractions without users, whole subsystems before the layer below
works, speculative APIs, unnecessary tooling, feature creep.

**The sandbox must stay runnable after every stage.**

## Vertical slices

```text
Slice A (Stages 1–4): Window → Application loop → wgpu → Render pass → Purple clear
Slice B (Stages 3, 5): ECS → Transform2D → GPU transform → Quad on screen
Slice C (Stages 6–7):  Texture → Sprite → Sprite renderer → Camera2D
Slice D (Stages 8–10): Input → Assets → API refinement with a real example game
```

## Milestones

| Milestone | Working state | Stage | Task | Status |
|---|---|---|---|---|
| **M0: Architecture Ready** | Structure, decisions, roadmap documented; scaffold compiles | 0 | PP-000, PP-001 | VERIFIED |
| **M1: Running Application** | Window opens, lifecycle runs, clean shutdown | 1 | PP-003 | FUNCTIONAL (Linux verified; Windows pending) |
| **M2: Fixed Simulation** | Fixed timestep drives `fixed_update` correctly | 2 | PP-004 | VERIFIED (Linux) |
| **M3: ECS Integration** | World, entities, components and a system run in the loop | 3 | PP-005 | VERIFIED (Linux) |
| **M4: GPU Foundation** | wgpu initialized, purple clear, resize/minimize safe | 4 | PP-006 | NOT_STARTED |
| **M5: First 2D Primitive** | ECS entity rendered as a GPU quad | 5 | PP-007 | NOT_STARTED |
| **M6: Sprite Foundation** | Textured sprites, batched | 6 | PP-008 | NOT_STARTED |
| **M7: Camera** | World ↔ screen coordinates, resize-aware | 7 | PP-009 | NOT_STARTED |
| **M8: Input** | Game-facing input abstraction | 8 | PP-010 | NOT_STARTED |
| **M9: Resource/Asset Foundation** | Coherent asset handles and loading | 9 | PP-011 | NOT_STARTED |
| **M10: Engine API Stabilization** | Boundaries reviewed with a real example game | 10 | PP-012 | NOT_STARTED |

Each stage is one milestone. The sequence above is kept deliberately. Stage 3
(ECS) comes before Stage 4 (GPU) so the first primitive (Stage 5) can be
driven by real ECS data rather than a throwaway draw call.

---

## Stage 0: Architecture & Planning → M0 ✅

| | |
|---|---|
| **Objective** | A documented, compiling foundation that later stages can build on without rewrites |
| **Prerequisites** | — |
| **Tasks** | PP-000 architecture/scaffold, PP-001 documentation system, PP-002 Windows toolchain |
| **Files** | `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/main.rs`, `assets/`, `docs/`, `README.md` |
| **Expected result** | `cargo run` prints the sandbox version line |
| **Validation** | fmt, check, clippy `-D warnings`, test, build, run |
| **Acceptance criteria** | Versions verified; ECS chosen; ADRs recorded; roadmap, tasks, status, risks and protocol exist |
| **Risks** | Plans drifting from reality (R-16) |
| **Definition of Done** | Met on 2026-09-30 (see PROJECT_STATUS) |

## Stage 1: Minimal Application → M1 (implemented; awaiting Windows confirmation)

| | |
|---|---|
| **Objective** | A real window with a correct winit 0.30 lifecycle and clean shutdown |
| **Prerequisites** | M0 |
| **Tasks** | PP-003 |
| **Files** | `Cargo.toml` (+`winit 0.30.13`, `thiserror 2.0.21`), new `src/error.rs`, new `src/app/` (`mod.rs`, `config.rs`, `game.rs`, `pacer.rs`, `runner.rs`), `src/lib.rs`, `src/main.rs`, docs. As built, `game.rs` and `pacer.rs` were split out of the plan for testability. |
| **Expected result** | `cargo run` opens a titled window. Close button or Escape exits with code 0. |
| **Validation** | Standard checks + unit tests for `EngineConfig` defaults/builders + Xvfb smoke run (timed exit) in Cowork + owner run on Windows |
| **Acceptance criteria** | Window created only in `resumed`. `CloseRequested` → exit. Resize handled without panic. `RedrawRequested` frame hook exists. `WaitUntil` pacing (no 100% CPU spin). Callback errors are returned from `Engine::run`. Sandbox uses only `Engine`/`EngineConfig`/`Game`. PD-04 (logger) was deferred to Stage 4 because Stage 1 has nothing to log. |
| **Risks** | winit lifecycle misuse (R-02), busy loop (R-06), OneDrive build folder (R-15) |
| **Definition of Done** | Project DoD + M1 behavior confirmed by a smoke run |

## Stage 2: Time & Fixed Update → M2 ✅

| | |
|---|---|
| **Objective** | Frame-rate-independent simulation via a fixed timestep |
| **Prerequisites** | M1 |
| **Tasks** | PP-004 |
| **Files** | new `src/time/` (`mod.rs`: `Time`; `fixed.rs`: `FixedTimestep`), `src/app/runner.rs`, `src/app/game.rs` (`fixed_update`; `Context::time()`/`dt()`), `src/app/config.rs` (timestep settings), `src/lib.rs`, sandbox. As built: `Context` exposes accessor methods rather than public fields. |
| **Expected result** | Sandbox logs or shows fixed-step counts at 60 Hz independent of frame rate |
| **Validation** | Unit tests: 0 steps, 1 step, N steps, cap reached, backlog drop, `MAX_FRAME_DT` clamp, alpha range |
| **Acceptance criteria** | `FixedTimestep` has no winit dependency. Constants live in `EngineConfig`. The loop order matches ADR-010. |
| **Risks** | Spiral of death (R-07), f32 drift (use f64 for accumulated time) |
| **Definition of Done** | Project DoD |

## Stage 3: ECS → M3 ✅

| | |
|---|---|
| **Objective** | A hecs world owned by the engine, lent to the game, with a first system |
| **Prerequisites** | M2 |
| **Tasks** | PP-005 |
| **Files** | `Cargo.toml` (+`hecs 0.11.1`, `glam 0.33` with f32 types only), new `src/math/` (`Transform2D`, `Vec2`), new `src/ecs/` (re-exports, `Velocity`, `integrate_velocity`), `src/app/{game,runner}.rs` (`Context::world()`/`world_mut()`, runner-owned `World`), `src/lib.rs` (`pub mod ecs`, `pub mod math`), sandbox. As built: the world is reached through accessor methods, not a public field. |
| **Expected result** | Sandbox spawns an entity whose position advances each fixed step |
| **Validation** | Tests: spawn, attach components, query, `integrate_velocity` moves the transform by `v·dt`, `Transform2D` defaults |
| **Acceptance criteria** | `ecs`/`math` have no render/app/winit imports. The game calls the system explicitly. |
| **Risks** | Borrow conflicts in `Context` (R-04) |
| **Definition of Done** | Project DoD |

## Stage 4: WGPU Initialization → M4

| | |
|---|---|
| **Objective** | GPU renderer that clears the window to PurplePie purple |
| **Prerequisites** | M1 (M2 and M3 done by then) |
| **Tasks** | PP-006 (GPU context + purple clear), then PP-014 (GPU error/device-loss handling + logging decision). Split on 2026-09-30 when Stage 4 became current. |
| **Files** | `Cargo.toml` (+`wgpu 30.0.1`, `pollster 1.0.1`), new `src/render/` (`Renderer`, `GpuContext`, `Color`), `src/error.rs`, `src/app/runner.rs` |
| **Expected result** | Purple window. Resize, minimize and restore work. Clean shutdown. |
| **Validation** | Build + Xvfb/lavapipe smoke run with pixel color check + owner run on Windows (real GPU) |
| **Acceptance criteria** | Full init chain with typed errors. `CurrentSurfaceTexture` policy as in ARCHITECTURE §7. Device-lost/uncaptured error capture. 0×0 size never configured. Renderer dropped before window. PD-01 (color space) decided. |
| **Risks** | wgpu API churn (R-01), surface loss (R-09), color space (R-08), headless-only validation (R-11) |
| **Definition of Done** | Project DoD + owner confirms on real hardware |

## Stage 5: First 2D Primitive → M5

| | |
|---|---|
| **Objective** | Render a colored quad positioned by an ECS `Transform2D` |
| **Prerequisites** | M3, M4 |
| **Tasks** | PP-007 |
| **Files** | `assets/shaders/quad.wgsl` (or embedded), `src/render/` (pipeline, vertex/uniform buffers), a drawable component |
| **Expected result** | A moving quad on the purple background |
| **Validation** | Transform → matrix unit tests, smoke run with pixel check |
| **Acceptance criteria** | Renderer reads the world only. No wgpu types in components. Shader errors surface as `Error`. |
| **Risks** | Premature renderer abstraction (R-10) |
| **Definition of Done** | Project DoD |

## Stage 6: Sprite Rendering → M6

| | |
|---|---|
| **Objective** | Textured sprites with batching |
| **Prerequisites** | M5 |
| **Tasks** | PP-008 |
| **Files** | `src/render/` (texture, sampler, sprite batcher), `Sprite` component, `Cargo.toml` (+`image` with PNG only) |
| **Expected result** | Many sprites drawn efficiently from one or a few textures |
| **Validation** | Tests for batching and sorting. Smoke run. Rough sprite-count timing noted. |
| **Acceptance criteria** | PD-05 decided. sRGB texture format per PD-01. |
| **Risks** | Resource lifetime (R-17), per-frame allocations (R-18) |
| **Definition of Done** | Project DoD |

## Stage 7: Camera & Coordinates → M7

| | |
|---|---|
| **Objective** | `Camera2D` with a defined world coordinate system |
| **Prerequisites** | M6 |
| **Tasks** | PP-009 |
| **Files** | `src/render/camera.rs`, projection uniform, `screen_to_world` |
| **Expected result** | Resizing changes the visible area, not sprite scale. Camera pans and zooms. |
| **Validation** | Unit tests for projection and `screen_to_world` round-trips, including DPI scale |
| **Acceptance criteria** | PD-02 decided and recorded as an ADR |
| **Risks** | Cross-platform DPI behavior (R-12) |
| **Definition of Done** | Project DoD |

## Stage 8: Input System → M8

| | |
|---|---|
| **Objective** | Game-facing keyboard/mouse input independent of winit |
| **Prerequisites** | M2 (fixed steps), M7 (cursor to world) |
| **Tasks** | PP-010 |
| **Files** | new `src/input/`, winit translation in `src/app/`, `Context.input` |
| **Expected result** | Sandbox moves an entity with the arrow keys |
| **Validation** | State-machine unit tests: press, hold, release, edges with 0, 1 and N fixed steps |
| **Acceptance criteria** | PD-03 decided. No winit types in the public input API. |
| **Risks** | Edge semantics with fixed steps (R-05) |
| **Definition of Done** | Project DoD |

## Stage 9: Assets & Resources → M9

| | |
|---|---|
| **Objective** | A coherent way to load and reference textures and shaders |
| **Prerequisites** | M6 |
| **Tasks** | PP-011 |
| **Files** | new `src/assets/`, render integration, `assets/` folder conventions |
| **Expected result** | Games reference textures via handles. Missing files give a clear `Error::Asset`. |
| **Validation** | Unit tests for the handle store and error paths. Smoke run. |
| **Acceptance criteria** | PD-06 decided and recorded as an ADR |
| **Risks** | Asset lifetime and ownership (R-17) |
| **Definition of Done** | Project DoD |

## Stage 10: Engine/Game API Refinement → M10

| | |
|---|---|
| **Objective** | Validate and refine the public API with a small real game |
| **Prerequisites** | M8, M9 |
| **Tasks** | PP-012 |
| **Files** | `examples/` (e.g. Pong or Breakout), public API docs, `#![warn(missing_docs)]` |
| **Expected result** | A complete small game using only the public API |
| **Validation** | Example builds and runs. API docs build without warnings. |
| **Acceptance criteria** | ADR-008 reviewed. Workspace split decision (ADR-002 revisit) recorded. |
| **Risks** | API instability (R-13) |
| **Definition of Done** | Project DoD |

## Not scheduled

Render interpolation, shape/text/debug renderers, audio, scenes and
serialization, hot reload, editor tooling. These become tasks only when a
concrete need appears.
