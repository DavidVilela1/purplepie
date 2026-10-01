# PurplePie Tasks

Active task tracker. Rules are in [DEVELOPMENT.md §5](DEVELOPMENT.md#5-tasks).

* **Status:** `TODO`, `IN_PROGRESS`, `BLOCKED`, `DONE`, `CANCELLED`.
* **Priority:** `P1` (next), `P2` (upcoming stages), `P3` (later).
* A task is `DONE` only when the [Definition of Done](DEVELOPMENT.md#4-definition-of-done) is met,
  not merely because code exists.
* Future stages have one coarse task each. Split a task only when its stage becomes current.

## Current

- [ ] **PP-008: Stage 6 · Textures + `Sprite` component (minimal texture handle)** · P1 · TODO ← **next task**

## In Progress

- [ ] **PP-003: Stage 1 · Minimal application (window + lifecycle)** · P1 · IN_PROGRESS
  Implemented and verified on Linux/Xvfb. On Windows, `cargo run` is confirmed (owner screenshot of the running window, 2026-09-30). **Still unconfirmed on Windows: `cargo test`, and Escape/close exiting cleanly.** The CI Windows job (`cargo test`) covers the first once the workflow runs.

## Blocked

_None._

## Completed

- [x] **PP-000: Stage 0 · Architecture, version verification, compatibility spike, scaffold** · DONE
- [x] **PP-001: Stage 0 · Engineering documentation and task-tracking system** · DONE
- [x] **PP-002: Stage 0 · Windows toolchain able to build and run the scaffold** · DONE
- [x] **PP-004: Stage 2 · Time & fixed update** · DONE (2026-09-30)
- [x] **PP-005: Stage 3 · ECS integration** · DONE (2026-09-30)
- [x] **PP-006: Stage 4 · GPU context + purple clear** · DONE (2026-09-30; Windows purple window confirmed by owner screenshot)
- [x] **PP-014: Stage 4 · GPU error & device-loss handling + logging decision** · DONE (2026-09-30)
- [x] **PP-007: Stage 5 · First 2D primitive (quad from ECS)** · DONE (2026-10-01; verified on Linux; owner's Windows look pending, non-blocking)

## Future

- [ ] **PP-015: Stage 6 · Draw order/layers (PD-08) + sprite batching by texture (PD-05)** · P2 · TODO
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

### PP-005: ECS integration
| Field | Value |
|---|---|
| Stage | 3 → Milestone M3 · Priority P1 · **DONE** (2026-09-30) |
| Dependencies | PP-004 (DONE) |
| Scope | Added `hecs 0.11.1` + `glam 0.33` (f32 types only: `default-features = false, features = ["std"]`). New `src/math/` (`Transform2D`, re-exported `Vec2`). New `src/ecs/` (re-exports `World`, `Entity`, the `hecs` crate; `Velocity`; `integrate_velocity`). Runner owns one `World`. `Context::world()` / `Context::world_mut()`. Sandbox spawns a mover in `init` and integrates it in `fixed_update`. |
| Acceptance criteria | ✅ 1. Tests: spawn/attach/query, `integrate_velocity` = `v·dt`, entities lacking a component are ignored, rotation/scale untouched, `Transform2D` defaults and builders, world changes via `Context` persist, plus 2 compiled doctests. ✅ 2. `math`/`ecs` import only `glam`/`hecs`/`crate::math`. ✅ 3. The game calls the system explicitly. ✅ 4. Xvfb: after 297 fixed steps the mover is at x = 4.9500 (= 297 · 1/60 · 1.0). ✅ 5. Docs updated. |
| Notes | Borrow pattern: `let dt = ctx.dt(); ecs::integrate_velocity(ctx.world_mut(), dt);`, because `world_mut` borrows the context exclusively. This is documented in the API docs and doctest. `Context` now implements `Debug` by hand (it prints the entity count) because `hecs::World` is not `Debug`. |

### PP-006: GPU context + purple clear
| Field | Value |
|---|---|
| Stage | 4 → Milestone M4 · Priority P1 · **DONE** (2026-09-30). All AC met. AC 5: the owner's Windows screenshot shows the running sandbox, and every sampled pixel in the window area is `(106, 13, 173)` = `#6A0DAD`. Stage 4 was split into PP-006 + PP-014. |
| Dependencies | PP-003 (implemented; Windows run outstanding), PP-004, PP-005 (DONE) |
| Scope (as built) | Added `wgpu 30.0.1` + `pollster 1.0.1`. New `src/render/`: public `Color` (sRGB) and a crate-private `Renderer` that owns instance, surface, device, queue and config. A separate `GpuContext` was planned but dropped: a single struct suffices. The window is received as `Arc<dyn wgpu::WindowHandle>` and the display as `impl wgpu::wgt::WgpuHasDisplayHandle`, so `render` never imports winit. `PresentMode::AutoVsync`. Acquire policy for every `CurrentSurfaceTexture` variant (`Lost` → recreate the surface from the kept instance and window; **superseded by PP-014 / ADR-017: `Lost` is now fatal**). 0×0 skips rendering. New `Error::{Surface, Adapter, Device, SurfaceUnsupported}`. `EngineConfig::clear_color` (default `Color::PURPLEPIE` = `#6A0DAD`). The runner creates the renderer in `resumed`, drops it in `suspended`/`exiting` before the window, resizes on `Resized`, and renders after `update`. |
| Deviation | **`FramePacer` kept** instead of removed (ADR-014). Measured under Xvfb: `Fifo` + `Poll` ran at 544 fps using about one core, the default present mode would be `Immediate`, and `Occluded` returns immediately. |
| Acceptance criteria | ✅ 1. Typed error variants. With no usable backend, the sandbox exits 1 with "failed to create the GPU surface" and its cause, no panic. ✅ 2. Xvfb + lavapipe: 921,600 of 921,600 window pixels are `#6A0DAD` (PD-01 → ADR-015). ✅ 3. Resize to 640×360, 1×1 → 1600×900, and unmap/map: no panic, repainted with exact pixel counts. ✅ 4. Stage 1–3 runs unchanged: 300 frames in 5.1 s (60 Hz cap), mover = 299 · 1/60, Escape/close/other key OK. ✅ 5. Owner confirmed the purple window on Windows (screenshot, pixel-checked). Window ≈1600×900 physical at the owner's display scaling (logical 1280×720), as expected. ✅ 6. Docs updated. |
| Known gap → fixed in PP-014 | wgpu's default uncaptured-error handler panicked, and device loss was not handled. The `Lost` → recreate policy from this task was later found to panic in wgpu-hal and was replaced (ADR-017). |

### PP-014: GPU error & device-loss handling + logging decision
| Field | Value |
|---|---|
| Stage | 4 → Milestone M4 · Priority P1 · **DONE** (2026-09-30) |
| Dependencies | PP-006 |
| Scope (as built) | New `src/render/faults.rs`: `FaultSlot` (first-fault-wins `Arc<Mutex<Option<GpuFault>>>`) installs `on_uncaptured_error` + `set_device_lost_callback` right after `request_device`. `GpuFault::{Uncaptured, DeviceLost, SurfaceLost, AcquireValidation}`. New `Error::Render`. The renderer checks faults after init and before/after each frame. `Validation` and `Lost` acquire results are fatal. Engine logging via `log` (ADR-016), and the sandbox has a stderr logger with `PURPLEPIE_LOG`. PD-04 → ADR-016, fault policy → ADR-017. |
| Deviations | (1) The planned "N validation failures in a row" counter was dropped: wgpu routes acquire validation through the uncaptured-error handler, so the first failure is already recorded. (2) **`Lost` became fatal**: an end-to-end test (destroying the X window mid-run) showed that surface recreation panics inside wgpu-hal 30.0.1 (`vulkan/instance.rs:407`). |
| Acceptance criteria | ✅ 1. No panic path from wgpu callbacks. The control run without the handler panics, and with it the error is captured. ✅ 2. 4 unit tests (first-fault-wins, shared clones, `Destroyed` ignored, `Unknown` message) + 1 `#[ignore]` GPU test (`cargo test -- --ignored`, passes under lavapipe): a deliberately invalid buffer → `Uncaptured(Validation)`, `destroy()` → no fault. ✅ 3. Forced fault end to end: the window destroyed under a running sandbox → `error: GPU rendering failed` / `caused by: the window's GPU surface was lost`, exit 1 (3/3 runs). ✅ 4. Stage 1–4 runs unchanged (pixel-exact purple, resize, unmap, 60 Hz cap, Escape, close, no-GPU, no-display). ✅ 5. Docs updated. |

### PP-007: First 2D primitive (quad from ECS)
| Field | Value |
|---|---|
| Stage | 5 → Milestone M5 · Priority P1 · **DONE** (2026-10-01) |
| Dependencies | PP-005, PP-006, PP-014 |
| Scope (as built) | `bytemuck 1.25` (`derive`; already in the tree). `math::Mat4` + `Transform2D::to_mat4()`. Public `render::Quad { size, color }`. Crate-private `render/quad.rs`: `view_projection` (ADR-018), `QuadInstance` (80 B, Pod), `collect_instances(&World, …)`, `QuadPipeline` (one instanced pipeline, growable instance buffer). Corners come from `vertex_index`, with no vertex buffer or bind group. `render/quad.wgsl` is embedded with `include_str!`. The renderer takes `&World` and the window scale factor (also on `ScaleFactorChanged`). The sandbox shows amber/teal/white reference quads, and the white one bounces between x = ±500. ADR-018 (coordinates), ADR-019 (quad pipeline). |
| Acceptance criteria | ✅ 1. Unit tests: `to_mat4` (identity/translate/scale/CCW rotation/order), view (centre → 0, corners → ±1, +Y up, depth in [0, 1]), quad corners, scale+rotation, colour per format, instance size 80, world query filter. ✅ 2. Xvfb screenshot: amber 200×100 exactly at x 240..439 / y 110..209 (20,000 px); teal rotated 45° = 9,940 px, bbox 140×140; white 80×80 moved between frames; **0 stray pixels**. ✅ 3. The renderer takes `&World` only. Components hold no wgpu types. ✅ 4. Ignored GPU tests: the pipeline builds for 3 formats with zero errors, and broken WGSL is captured as a fault (no panic). ✅ 5. Stage 1–4 regressions unchanged (exits, timing, window-destroy → exit 1, unmap/1×1/resize, no-GPU). ✅ 6. Docs updated, and PD-02 core → ADR-018. |
| Notes | glam 0.33 deprecated `Mat4::orthographic_rh`, so the code uses `glam::camera::rh::proj::directx::orthographic`. Draw order is ECS query order (PD-08 → PP-015). The owner may want to look at it on Windows: three shapes on purple, the white one moving. |

### PP-008: Textures + `Sprite` component (minimal texture handle) ← NEXT
| Field | Value |
|---|---|
| Stage | 6 → Milestone M6 · Priority P1 · **TODO**. Stage 6 was split into PP-008 + PP-015 on 2026-10-01. |
| Dependencies | PP-007 (DONE) |
| Why now | It's the next vertical slice (texture → sprite → screen). It also forces the first answer to "how does game code refer to a GPU resource without touching wgpu", which every later stage builds on. |
| Scope | Re-verify and add `image` (PNG only, minimal features; ADR-013). A **minimal texture handle**: game code asks the engine to load a PNG and gets back a plain-data handle (e.g. `TextureId`). The upload happens inside `render` (ADR-009 holds: no wgpu in `Context`). This is the smallest piece of PD-06 needed now, so record it as an ADR. Public `render::Sprite { texture, size, tint }` component. Textures upload as `Rgba8UnormSrgb` (ADR-015), with a sampler and a bind group per texture. The quad pipeline grows a textured variant, or a sprite pipeline reuses the instancing pattern (ADR-019). The sandbox shows a sprite from `assets/textures/` (a small generated PNG committed to the repo). |
| Acceptance criteria | 1. Unit tests for handle bookkeeping, plus PNG decode → expected dimensions and pixels (a tiny test PNG). 2. Missing or invalid file → a clear typed error, not a panic. 3. Xvfb: the sprite's pixels match the PNG's colours at the expected rectangle (sRGB round-trip exact within ±2). 4. Ignored GPU test: the sprite pipeline builds with zero GPU errors. 5. Stage 1–5 regressions unchanged. 6. DoD and docs updated, and an ADR for the texture handle. |

### PP-015: Draw order/layers + sprite batching by texture
| Stage 6 → M6 · P2 · TODO | Depends on PP-008 |
|---|---|
| Acceptance criteria | PD-08 decided (layer/z on drawables, sorted per frame) and PD-05 decided (batch instances per texture), each recorded as an ADR. Deterministic draw order is tested (overlap pixel check). Draw calls = number of (texture, layer) batches. |

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
