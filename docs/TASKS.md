# PurplePie Tasks

Active task tracker. Rules are in [DEVELOPMENT.md §5](DEVELOPMENT.md#5-tasks).

* **Status:** `TODO`, `IN_PROGRESS`, `BLOCKED`, `DONE`, `CANCELLED`.
* **Priority:** `P1` (next), `P2` (upcoming stages), `P3` (later).
* A task is `DONE` only when the [Definition of Done](DEVELOPMENT.md#4-definition-of-done) is met,
  not merely because code exists.
* Future stages have one coarse task each. Split a task only when its stage becomes current.

## Current

- [ ] **PP-036a: Second game, part 1: choose and specify it (phase P3.5, item 6)** · P1 · TODO ← **next task**

## In Progress

_None._

## Blocked

_None._

## Completed

- [x] **PP-000: Stage 0 · Architecture, version verification, compatibility spike, scaffold** · DONE
- [x] **PP-001: Stage 0 · Engineering documentation and task-tracking system** · DONE
- [x] **PP-002: Stage 0 · Windows toolchain able to build and run the scaffold** · DONE
- [x] **PP-003: Stage 1 · Minimal application (window + lifecycle)** · DONE (2026-10-01; Windows `cargo test` + Escape/close confirmed by the owner)
- [x] **PP-004: Stage 2 · Time & fixed update** · DONE (2026-09-30)
- [x] **PP-005: Stage 3 · ECS integration** · DONE (2026-09-30)
- [x] **PP-006: Stage 4 · GPU context + purple clear** · DONE (2026-09-30; Windows purple window confirmed by owner screenshot)
- [x] **PP-014: Stage 4 · GPU error & device-loss handling + logging decision** · DONE (2026-09-30)
- [x] **PP-007: Stage 5 · First 2D primitive (quad from ECS)** · DONE (2026-10-01; verified on Linux; owner's Windows look pending, non-blocking)
- [x] **PP-008: Stage 6 · Textures + `Sprite` component (minimal texture handle)** · DONE (2026-10-01; verified on Linux)
- [x] **PP-015: Stage 6 · Draw order/layers (PD-08) + sprite batching by texture (PD-05)** · DONE (2026-10-01; verified on Linux)
- [x] **PP-009: Stage 7 · Camera2D & coordinates** · DONE (2026-10-01; verified on Linux)
- [x] **PP-013: Owner · Choose project license** · DONE (2026-10-01; MIT OR Apache-2.0)
- [x] **PP-010: Stage 8 · Keyboard input** · DONE (2026-10-02; verified on Linux)
- [x] **PP-016: Stage 8 · Mouse input** · DONE (2026-10-06; verified on Linux)
- [x] **PP-011: Stage 9 · Asset root** · DONE (2026-10-06; verified on Linux)
- [x] **PP-012: Stage 10 · Example game (Breakout)** · DONE (2026-10-06; verified on Linux)
- [x] **PP-017: Stage 10 · API review** · DONE (2026-10-06; verified on Linux). Stages 0–10 (the portfolio scope) are complete.
- [x] **PP-018a: Text rendering, part 1 (ADR-027): fonts, glyph atlas, `Text` component** · DONE (2026-10-06; verified on Linux)
- [x] **PP-018b: Text anchors/alignment, `measure_text`, Breakout HUD text** · DONE (2026-10-06; verified on Linux)
- [x] **PP-019: Sprite sheets: `Sprite` regions + `SpriteGrid`** · DONE (2026-10-06; verified on Linux)
- [x] **PP-020: Sprite frame animation (ADR-028)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-021: Screen-space drawing for HUD/UI (ADR-029)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-022: Audio, part 1: sound effects (ADR-030)** · DONE (2026-10-07; verified on Linux; Windows/macOS compiled by CI only)
- [x] **PP-023: UI buttons (ADR-031)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-024a: Audio, part 2a: playback control and looping (ADR-032)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-024b: Audio, part 2b: OGG Vorbis decoding for music (ADR-033)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-025: Per-texture sampling, Nearest or Linear (ADR-034; F9; phase P2 complete)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-026a: Scene files, part 1a: format decision + drawing components (ADR-035)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-026b: Scene files, part 1b: animation, velocity and UI buttons (ADR-035 extension)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-027: Scene files, part 2: game components through a registry (ADR-036)** · DONE (2026-10-07; verified on Linux)
- [x] **PP-028: Asset hot reload, part 1: textures (ADR-037)** · DONE (2026-10-08; verified on Linux)
- [x] **PP-029: Asset hot reload, part 2: fonts and sounds (ADR-037 extension)** · DONE (2026-10-08; verified on Linux)
- [x] **PP-030: Workspace layout decision (ADR-038; phase P3 complete)** · DONE (2026-10-08; verified on Linux)
- [x] **PP-031: Outside-crate trial (phase P3.5, item 1)** · DONE (2026-10-08; verified on Linux; findings in [USABILITY.md](USABILITY.md))
- [x] **PP-032a: Newcomer guide `docs/GUIDE.md`, compiled by `cargo test` (phase P3.5, item 2, part 1)** · DONE (2026-10-08; verified on Linux)
- [x] **PP-032b: Module docs for game authors + starter-template decision (ADR-039; phase P3.5, item 2 complete)** · DONE (2026-10-09; verified on Linux)
- [x] **PP-035b: Release 0.1.0: version 0.1.0, `repository` field, CHANGELOG `[0.1.0] - 2026-10-09`, git dependency line (phase P3.5, item 5 complete)** · DONE (2026-10-09; verified on Linux; the owner pushes the `v0.1.0` tag)
- [x] **PP-033b: Owner run of `docs/CHECKLIST.md` on Windows (phase P3.5, item 3 complete)** · DONE (2026-10-09; owner-reported)
- [x] **PP-035a: Release preparation: `docs/RELEASING.md` (compatibility policy + procedure), `CHANGELOG.md`, CI `msrv` job and `latest-deps.yml` (phase P3.5, item 5, part 1)** · DONE (2026-10-09; verified on Linux; the two new CI jobs run first on GitHub)
- [x] **PP-034b: Scene component formatting (U-12) + opt-in console logger `EngineConfig::console_log` (U-13) (phase P3.5, item 4 complete)** · DONE (2026-10-09; verified on Linux)
- [x] **PP-034a: Second public API review: ADR-040 (anchor names, readable `Error` debug output, resolved `Save` path, `#[non_exhaustive]` policy) (phase P3.5, item 4, part 1)** · DONE (2026-10-09; verified on Linux)
- [x] **PP-033a: Platform checklist `docs/CHECKLIST.md` written and run on Linux (phase P3.5, item 3, part 1)** · DONE (2026-10-09; verified on Linux, PowerShell blocks under PowerShell 7)

## Future

- [ ] **PP-036: A second game as an outside crate (phase P3.5, item 6; PP-036a first)** · P2 · TODO

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
| Stage | 1 → Milestone M1 · Priority P1 · **DONE** (2026-10-01). Implemented 2026-09-30. AC 1–11 met; AC 10 confirmed by the owner on 2026-10-01 (Windows: `cargo test` passes, Escape and the close button exit cleanly; `cargo run` window seen 2026-09-30). |
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

### PP-008: Textures + `Sprite` component (minimal texture handle)
| Field | Value |
|---|---|
| Stage | 6 → Milestone M6 · Priority P1 · **DONE** (2026-10-01). Stage 6 was split into PP-008 + PP-015 on 2026-10-01. |
| Dependencies | PP-007 (DONE) |
| Scope (as built) | `image 0.25.10` (`default-features = false`, `png`; +12 crates, 116 → 128). New `Error::Asset { path, source }`. Public `render::TextureId` (Copy, private field) and `render::Sprite { texture, size, tint }` (`new`, `with_tint`). `Context::load_texture(path)` decodes immediately (typed errors; same path → same id) and `Context::texture_size(id)`. Crate-private `render/texture.rs` (`Textures` store owned by the runner, `decode_png`), `render/instance.rs` (`Instance` + `InstanceBuffer`, shared with quads), `render/sprite.rs` + `sprite.wgsl` (second instanced pipeline, one bind group per texture, `Nearest`/`ClampToEdge` sampler, texture format follows the surface's sRGB-ness, consecutive same-texture sprites share a draw call). The renderer uploads new store entries before each frame and re-uploads all of them after recreation; oversized textures → `Error::Asset`. Sprites draw after quads. Sandbox: `assets/textures/sandbox_quadrants.png` (16×16, generated) as a static 128×128 sprite at (0, 120) and a tinted, spinning 96×96 copy at (−300, −20). ADR-020. |
| Acceptance criteria | ✅ 1. Unit tests: handle bookkeeping (sequential ids, same path → same id, `since`, sizes), PNG decode (RGBA rows, grey/RGB gain alpha, 16-bit → 8-bit, sandbox PNG quadrants), batching of consecutive textures, sprite instance maths and tint. ✅ 2. Missing / non-PNG file → `Error::Asset` naming the path with the I/O or decode error as source; failures aren't cached. End to end: the sandbox exits 1 with the cause chain, no panic. ✅ 3. Xvfb: every sprite region is exact (maxdiff 0, tolerance ±2): transparent border = background, three opaque quadrants = PNG colours, 50% alpha quadrant = linear-space blend `#5662DB`; no bleed outside the 128×128 rectangle. ✅ 4. Ignored GPU tests: sprite pipeline builds and uploads for 3 formats with zero GPU errors; incremental sync uploads once; oversized texture → `Error::Asset` before wgpu. ✅ 5. Stage 1–5 regressions unchanged. ✅ 6. Docs updated, ADR-020. |
| Notes | Owner check on Windows: a four-colour square above the centre and a smaller tinted copy spinning on the left. |

### PP-015: Draw order/layers + sprite batching by texture
| Field | Value |
|---|---|
| Stage | 6 → Milestone M6 · Priority P1 · **DONE** (2026-10-01). Completes Stage 6. |
| Dependencies | PP-008 (DONE) |
| Scope (as built) | Public `render::Layer(pub i32)` component (optional, default 0, higher = on top). New crate-private `render/draw.rs`: `DrawList` collects quads and sprites, sorts by (layer, material rank, entity index), and builds `Batch { layer, material, instances }` runs. One shared `InstanceBuffer` for quads and sprites; `QuadPipeline`/`SpritePipeline` became pipeline + bind-group holders (`bind`, `bind_texture`); `renderer.rs` records one draw call per batch and switches pipelines only on material-kind changes. Removed `collect_instances`/`collect_sprites`. Sandbox: yellow quad (layer 0) under the sprite's top-left corner, pink quad (layer 1) over its bottom-right corner. ADR-021 resolves PD-05 and PD-08. |
| Acceptance criteria | ✅ 1. Overlap pixel check under Xvfb: quad-over-sprite (layer 1 pink covers 576 px of the sprite) and sprite-over-quad (layer 0 yellow visible only through the transparent border: 2,048 px; red covers 256 px); 0 mismatches over 40,000 px at 1280×720 and again at 1000×600. Control run (pink on layer −1) puts it under the sprite. ✅ 2. Draw calls = (layer, material) runs, unit-tested. ✅ 3. Equal layers: quads, then sprites by texture, then entity index, documented on `Layer`; a test moves an entity to another archetype and the order is unchanged (and fails without the tie-breaker). ✅ 4. Stage 1–6 regressions unchanged. ✅ 5. ADR-021, docs updated. |
| Notes | Rough cost (release): draw-list build + sort ~0.08 ms / 1k, ~1.8 ms / 10k, ~7.9 ms / 50k drawables. Owner check on Windows: a yellow square peeking out under the top-left corner of the four-colour sprite, and a pink square on top of its bottom-right corner. |

### PP-009: Camera2D & coordinates
| Field | Value |
|---|---|
| Stage | 7 → Milestone M7 · Priority P1 · **DONE** (2026-10-01). Completes Stage 7. |
| Dependencies | PP-015 (DONE), ADR-018 |
| Scope (as built) | New `src/render/camera.rs`: public `Camera2D { position, zoom }` (`IDENTITY`/`Default` = ADR-018 view, `new`, `effective_zoom`, `screen_to_world`, `world_to_screen`, `visible_world_rect`; crate-private `view_projection`, which replaced `quad::view_projection`). The runner owns one camera and a logical viewport; `Context::camera()`, `camera_mut()`, `viewport_size()`. `Renderer::render` takes `&Camera2D`. The viewport is updated only from window events (creation, `Resized`, `ScaleFactorChanged`). Sandbox: `PURPLEPIE_SANDBOX_CAMERA=x,y,zoom`, and the timed exit prints the viewport and camera. ADR-022 resolves PD-02. |
| Acceptance criteria | ✅ 1. ADR-022. ✅ 2. Unit tests: default camera = ADR-018 matrix exactly; pan; zoom; invalid zoom → 1; screen axes; round trips over 3 viewports × 3 cameras × 3 points; DPI 1.0/1.25/2.0 agreement between the projection and `world_to_screen`; `Context` camera persistence + viewport. ✅ 3. Xvfb whole-frame per-pixel model, 0 mismatches for cameras (0,0)×1, (0,120)×2, (200,0)×0.5, and (0,120)×2 after resizing to 1000×600; a 1-unit model offset gives ~3,000 mismatches (sensitivity control). ✅ 4. Stage 1–6 regressions unchanged, after fixing the regression below. |
| Notes | **Regression found and fixed during validation:** the first version read `window.inner_size()` every frame; once the X11 window was destroyed, winit 0.30.13 panicked inside `inner_size` (`GetGeometry` unwrap) instead of the clean `Error::Render` exit. The viewport is now event-driven (ADR-022, R-22). |

### PP-010: Keyboard input
| Field | Value |
|---|---|
| Stage | 8 → Milestone M8 · Priority P1 · **DONE** (2026-10-02). Stage 8 was split into PP-010 (keyboard) + PP-016 (mouse) on 2026-10-01. |
| Dependencies | PP-004, PP-009 (DONE) |
| Scope (as built) | New public module `input`: `KeyCode` (99 physical keys, US-layout names, `#[non_exhaustive]`) and `Input` (`pressed`, `just_pressed`, `just_released`, `axis`, `pressed_keys`) with separate frame / fixed-step edge sets (128-bit masks). New `app/keymap.rs` (winit → `KeyCode`) and `app/state.rs` (`EngineState`: `Context` now borrows one struct instead of six arguments). The runner feeds key events (repeats and X11 synthetic presses ignored), releases all keys on focus loss, and brackets every fixed step / frame for the edge sets; key and focus events are logged at `debug`. `Context::input()`. `exit_on_escape` kept as before. Sandbox: arrows pan the camera (300 px/s), `=` / `-` zoom ×2 / ÷2; the timed exit prints how many `=` presses each callback saw. ADR-024 resolves PD-03. No new dependencies. |
| Acceptance criteria | ✅ 1. ADR-024. ✅ 2. Unit tests: press/hold/release, OS repeat, tap within a frame, 0 / 1 / N fixed steps, focus release, release-without-press, axis, key bits; keymap bijection. A mutation to per-frame edge clearing fails 3 tests. ✅ 3. No winit types in the public API (`input` has no dependencies). ✅ 4. Xvfb XTEST: 3 `=` taps → seen 3× in `fixed_update` and 3× in `update`, zoom 4 after one `-`; held arrows panned (155, 95); that frame matched the per-pixel model with 0 mismatches (3 runs, identical results); focus removed while → was held stopped the pan at the focus loss; Escape exits, other keys don't. ✅ 5. Stage 1–7 regressions unchanged. |
| Notes | One early run (before debug logging) lost the → press; it never reproduced in 14 later runs. Probably a focus race in WM-less Xvfb, unconfirmed (R-11). Owner check on Windows: arrows pan, `=` / `-` zoom, Escape quits. |

### PP-016: Mouse input
| Field | Value |
|---|---|
| Stage | 8 → Milestone M8 · Priority P1 · **DONE** (2026-10-06). Completes Stage 8. |
| Dependencies | PP-010, PP-009 (DONE) |
| Scope (as built) | `input::MouseButton` (Left, Right, Middle, Back, Forward) with `mouse_pressed` / `mouse_just_pressed` / `mouse_just_released`; keys and buttons now share one `Buttons` edge implementation. `Input::cursor_position()` (logical screen px, `None` outside the window) and `Input::scroll()` (lines, same once-per-callback delivery). `Context::cursor_world()`. `app/keymap.rs` translates buttons and wheel deltas; `app/state.rs` gained `physical_to_logical` / `sanitize_scale_factor`; the runner tracks the DPI scale from window events and handles `MouseInput`, `CursorMoved`, `CursorLeft`, `MouseWheel` (wheel logged at `debug`). Sandbox: green marker follows the cursor, left click stamps a cyan square, wheel zooms; the timed exit prints click counts and the cursor. ADR-024 extended (no new ADR). |
| Acceptance criteria | ✅ 1. Button edges unit-tested with 0/1/N steps via the shared implementation; no key/button bit sharing; focus release; cursor state; scroll once per callback. ✅ 2. Physical → logical → world at DPI 1.0/1.25/2.0 round-trips to the physical pixel. ✅ 3. Xvfb: camera (100,50)×2, clicks at (400,300) and (900,500), cursor at (700,200): 2 clicks seen in each callback, stamps and marker exactly where predicted (whole frame: 0 mismatches over 798,632 px); cursor outside the window → `None`, marker hidden, outside clicks ignored. ✅ 4. No winit types in the public API. ✅ 5. Stage 1–8 regressions unchanged (keyboard script identical). |
| Notes | XTEST wheel clicks arrive twice from winit on X11 (ADR-024 extension). While re-running the window-destroy test, a **pre-existing** intermittent winit panic was found (IME cleanup on an externally destroyed X11 window); the Stage 8 build reproduces it too (R-22). Owner check on Windows: the green marker tracks the cursor, clicks stamp squares, one wheel notch = one zoom step. |

### PP-011: Asset root
| Field | Value |
|---|---|
| Stage | 9 → Milestone M9 · Priority P1 · **DONE** (2026-10-06). Completes Stage 9 (narrowed on 2026-10-06). |
| Dependencies | PP-008 (DONE) |
| Scope (as built) | New crate-private `src/assets/mod.rs`: `AssetRoot` (`resolve`, `for_process`, `path`, `locate`). `EngineConfig::asset_root` + `with_asset_root`. `EngineState` holds the root (resolved once in `Runner::new`, logged). `Context::load_texture` resolves relative paths against it (absolute paths unchanged; the cache key is the resolved path); new `Context::asset_root()`. Sandbox loads `textures/sandbox_quadrants.png` (no more `CARGO_MANIFEST_DIR`). ADR-025. No new dependencies. |
| Acceptance criteria | ✅ 1. ADR-025 (file location decided; generic handles + unloading deferred in PD-06). ✅ 2. Unit tests: exe-folder-first order with working-directory fallback, a file named `assets` is not a folder, explicit root (relative → absolute, kept if missing), relative/absolute locate, no-root error lists the searched folders, relative and absolute spellings share a `TextureId`, config builder. ✅ 3. End to end: project folder, `cargo run`, shipped copy launched from another folder, moved binary from the project folder; no `assets/` anywhere → exit 1 with both searched folders named; missing file inside the root → full path in the error. ✅ 4. Stage 1–8 regressions unchanged. |
| Notes | Release builds must ship `assets/` next to the executable (README). |

### PP-012: Example game (Breakout)
| Field | Value |
|---|---|
| Stage | 10 → Milestone M10 · Priority P1 · **DONE** (2026-10-06). Stage 10 was split into PP-012 (game) + PP-017 (API review) on 2026-10-06. |
| Dependencies | PP-011, PP-016 (DONE) |
| Scope (as built) | `examples/breakout.rs` (531 lines, `cargo run --example breakout`): paddle (←/→, A/D or mouse), sprite ball, 10×6 tinted sprite bricks with row points, walls, 3 lives, speed-up per brick, win/lose overlays (translucent quads), restart with Space/↑/click, HUD (lives as ball sprites, a progress bar), camera zoom fitted to the window every frame. All collisions are AABB in game code. Seeded LCG + simulation only in `fixed_update` → deterministic `PURPLEPIE_BREAKOUT_AUTOPLAY=win|lose` test modes (bot / parked paddle) that print a summary and exit. New assets `assets/textures/breakout/{ball,brick}.png` (generated). **No engine changes were needed** (`src/` identical to Stage 9). |
| Acceptance criteria | ✅ 1. Full game playable; restart flow verified (3 balls lost via XTEST → `0` lives → Space → `new game`). ✅ 2. Example compiles as an external crate (only public items), no winit/wgpu/hecs paths. ✅ 3. Xvfb: cursor at x = 300 → paddle centre exactly 300 px; click launches; holding → moves the paddle 293 px in ~0.5 s; autoplay **win** = `Won after 7135 fixed steps; bricks 60/60; lives 3; score 220` (3 runs identical), **lose** = `Lost after 892 fixed steps; bricks 8/60; lives 0; score 14` (4 runs identical); lose end screen: overlay pixels exactly the linear-space blend `(161, 33, 47)`. ✅ 4. CI: `cargo check/clippy --all-targets` include the example. ✅ 5. Friction list below. |
| API friction found (input for PP-017) | **F1** No text rendering: score/lives drawn with sprites, events printed to the console (biggest gap for games). **F2** Window title can't change at runtime. **F3** No way to hide a drawable without despawning it (overlay respawned; the sandbox uses scale 0). **F4** Reading `ctx.input()` / `cursor_world()` while mutating the world needs values copied out first (one `&mut Context`); fine but noisy. **F5** Per-entity component access is verbose (`world.get::<&mut T>(e).ok()` helpers written by hand). **F6** "Cursor moved this frame?" needs manual tracking (no `cursor_delta`). **F7** Fitting a world rectangle to the window is manual math (`Camera2D` has no `fit`). **F8** `request_exit` can't carry an exit code; test modes rely on stdout. **F9** Small sprites from larger textures alias with `Nearest` sampling (ball). **F10** No randomness helper (an LCG in the example; probably right to keep out of the engine). Working well: fixed-step determinism, `Layer`, tinted shared texture batching, asset root, input edges. |

### PP-017: API review
| Field | Value |
|---|---|
| Stage | 10 → Milestone M10 · Priority P1 · **DONE** (2026-10-06). Completes Stage 10 and the portfolio scope (Stages 0–10). |
| Dependencies | PP-012 (DONE) |
| Scope (as built) | `#![warn(missing_docs)]` in `lib.rs` (4 gaps fixed: `KeyCode` variants get generated docs, `MouseButton` variants). Friction decisions F1–F10 in ADR-026: **added** `render::Hidden` (marker; the draw list skips it via `hecs::Without`), `Camera2D::fit(center, size, viewport)` and `Context::set_window_title` (applied by the runner only when requested); **deferred** text (PP-018) and sampling options; **no change** for F4/F5/F6/F8/F10 with reasons. `exit_on_escape` kept (documented). Single crate kept (ADR-002 reaffirmed). Breakout uses `Hidden` for its end-screen overlay, `Camera2D::fit` and a score/lives window title. README rewritten: features, getting started (compile-checked), example. |
| Acceptance criteria | ✅ 1. Zero `missing_docs` warnings (clippy `-D warnings` passes with the lint on). ✅ 2. Every friction item has a recorded decision (ADR-026). ✅ 3. Workspace decision recorded. ✅ 4. README matches the example and compiles. ✅ 5. Regressions unchanged; Breakout autoplay identical (`win` 7135 steps / score 220; `lose` 892 / 14) and the lose overlay has exactly the same pixels (347,496 × `(161,33,47)`) now that it is a `Hidden`-toggled entity. |

### PP-018: Text rendering (split on 2026-10-06 into PP-018a + PP-018b)
Friction F1 from Breakout (ADR-026): score, lives and messages had to be faked with quads, the window title and the
console. Split when started because the decision, the atlas and the drawing path were already a full task.

### PP-018a: Text rendering, part 1: fonts, glyph atlas, `Text`
| Field | Value |
|---|---|
| Stage | Post-portfolio · Priority P1 · **DONE** (2026-10-06) |
| Dependencies | PP-017 (DONE) |
| Scope (as built) | ADR-027. New dependency `ab_glyph` 0.2.32 (`std` only; +0 crates on Linux, +4 on Windows). `Context::load_font(path) -> Result<FontId>` (asset root, parse on load, `Error::Asset`, dedup by path); crate-private `render::Fonts` in `EngineState`. Public `render::{Text, FontId}`: `Text::new(content, font, size).with_color(..)`; origin = left end of the first baseline, `size` = em size in world units, `\n` = new line, control chars skipped, kerning from `kern`. Crate-private `render/atlas.rs` (1024² RGBA shelf-packed glyph atlas, white + coverage alpha, 1-texel border, dirty-row uploads, cleared and re-laid out once per frame when full) and `render/text.rs` (layout in whole pixels). The draw list rasterizes at the on-screen size (zoom × DPI × transform scale), snaps the origin to a physical pixel and emits one instance per glyph as a third material (`Glyphs`, drawn after sprites in each layer, one draw per run). `Instance` gains `uv_rect` (80 → 96 B); `sprite.wgsl` uses it; glyphs are drawn by the sprite pipeline with the atlas bound through a linear sampler. Font shipped: `assets/fonts/Poppins-Regular.ttf` + `OFL.txt`. Sandbox shows a help label. |
| Acceptance criteria | ✅ 1. ADR-027 records the approach, the measured dependency cost and the alternatives. ✅ 2. Unit tests: font store (ids, dedup, `Error::Asset` for missing/garbage files, via `Context` too), atlas (white + alpha, padding, cache per size, no overlaps, full → reset, dirty rows), layout (advance, baseline, spaces, newlines, control chars), draw list (text after quads/sprites, one batch per layer, `Hidden`, zero scale / zero size / NaN / oversize skipped, glyph rectangles on whole physical pixels at zoom 0.75/1/2 and DPI 1/1.25/2, atlas overflow → one reset). ✅ 3. GPU (`--ignored`, lavapipe): offscreen render of text at fractional positions (zoom 1 and 2) matches the CPU rasterization within 1/255 on every pixel; layer order and colour checked (red text over a blue quad, hidden under a layer-1 green quad). ✅ 4. Xvfb sandbox: whole-frame model outside the label 0 mismatches at camera (0,0)×1 (only the known cursor-marker pixels at (−300,−280)×2 and (100,40)×0.5); the label matches FreeType's rendering of the same font closely (IoU 0.77/0.86/0.59 at 20/40/10 px, mean difference 5/3/14 per 255; visually identical layout, slightly bolder because of linear blending). ✅ 5. Regressions: Breakout autoplay `win` 7135 / 220 and `lose` 892 / 14 unchanged, lose overlay still 347,496 px of `(161,33,47)`; window-destroy fault test clean `Error::Render` (2/2). |

### PP-018b: Text anchors/alignment, measuring, Breakout HUD text
| Field | Value |
|---|---|
| Stage | Post-portfolio · Priority P1 · **DONE** (2026-10-06). Completes PP-018 and friction F1. |
| Dependencies | PP-018a (DONE) |
| Scope (as built) | ADR-027 extension. Public `render::{TextAnchor, HorizontalAnchor, VerticalAnchor, TextMetrics}`; `Text::anchor` (default `BASELINE_LEFT`, unchanged behaviour) + `Text::with_anchor`; 12 anchor constants; horizontal anchors align every line (left/centre/right); vertical anchors use the font's ascent/descent (baseline/top/middle/bottom). Layout: first pass for line widths into a reused scratch buffer, line and block shifts rounded to whole pixels. `Context::measure_text(&Text) -> Option<TextMetrics>` (world units: width, height, ascent, descent, line height, lines) and `TextMetrics::bounds(anchor)`. Breakout: centred `SCORE n` label in the HUD, `YOU WIN!` / `GAME OVER` headline and a "Space, Up or click to …" hint (also shown while serving), each hidden with `Hidden` when empty; window title kept as a secondary display. No new dependencies. Coverage gamma for small text left as is until the owner's Windows look (R-25). |
| Acceptance criteria | ✅ 1. Unit tests: metrics against the font tables (`H H`, multi-line, empty, trailing newline, invalid sizes), `bounds` for 6 anchors and 2 lines, rigid whole-pixel shifts for 6 anchors, per-line centring/right alignment, `Context::measure_text`. ✅ 2. GPU (`--ignored`): for 6 anchors, every lit pixel of a two-line text lies inside the `bounds` rectangle (±1 px); a deliberately wrong anchor makes it fail (mutation check). ✅ 3. Xvfb Breakout: every pixel that changed on the lose screen versus PP-018a (9,185 px) lies inside the measured boxes of `SCORE 14`, `GAME OVER` and the hint (0 outside); serve screen shows score and launch hint; the overlay keeps 339,277 px of `(161, 33, 47)` (347,496 before, the rest is now text). ✅ 4. Autoplay unchanged: `win` 7135 / 220, `lose` 892 / 14. ✅ 5. Sandbox label strip pixel-identical to PP-018a (default anchor); whole-frame model 0 mismatches; window-destroy fault test clean. Owner's Windows look still requested. |

### PP-019: Sprite sheets
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 (runtime essentials) · Priority P1 · **DONE** (2026-10-06) |
| Dependencies | PP-018a (`Instance::uv_rect`) |
| Scope (as built) | ADR-020 extension. New `src/render/region.rs`: public `TextureRegion { x, y, width, height }` (texels, rows down; `size()`; crate-private `uv_rect` that cuts to the texture) and `SpriteGrid` (`new`, `with_spacing`, `with_margin`, `cell`, `frame`, `len`, `is_empty`; overflow-safe). `Sprite::region` + `with_region` (default `None` = whole texture). `DrawList::build` now also takes `&Textures` to turn regions into `uv_rect`; regions outside the texture or of unknown textures are skipped. Mirroring stays a negative transform scale (no flip flags). New asset `assets/textures/sandbox_sheet.png` (32×16, 4×2 cells of 8×8, each with its own border/inner colour and a white corner texel); the sandbox shows four cells at (400…520, −250), 32×32, the third mirrored. No new dependencies. |
| Acceptance criteria | ✅ 1. Unit tests: region → UV (whole, partial, cut, empty, outside, overflow), grid cells with margin/spacing, row-major frames, degenerate/overflowing grids, draw list (regions as `uv_rect`, one batch per texture, skipped regions). ✅ 2. GPU (`--ignored`): four cells drawn at 4× (one mirrored) — every one of 4,096 pixels equals its own texel; fails when regions are ignored (mutation check). ✅ 3. Xvfb sandbox whole-frame model incl. the four cells: 0 mismatches at (0,0)×1; only the known cursor-marker pixels at (450,−250)×2 and (460,−240)×1.5; the model detects a wrong mirror (32 px) or a wrong cell (1,008 px). ✅ 4. Sprites without regions unchanged: Breakout lose screen pixel-identical to PP-018b; autoplay `win` 7135 / 220, `lose` 892 / 14. |

### PP-020: Sprite frame animation
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 · Priority P1 · **DONE** (2026-10-07) |
| Dependencies | PP-019 (DONE) |
| Scope (as built) | ADR-028. New `src/render/animation.rs`: public `SpriteAnimation` (grid, `first`, `last` (reverse when smaller), `fps`, `mode`; `new`, `once`, `frame`, `region`, `len`, `is_finished`, `restart`, `advance`), `AnimationMode { Loop, Once }` and the system `advance_animations(&mut World, dt)` the game calls from `fixed_update`. Playback state = frame step + `f64` seconds in frame, with a 1 µs boundary slack. The draw list queries `Option<&SpriteAnimation>` and draws its current frame instead of `Sprite::region`. Sandbox: an animated cell (all 8 sheet frames, 4 fps) at (580, −250); the timed exit prints its frame. No new dependencies. |
| Acceptance criteria | ✅ 1. Unit tests (10 + 1 draw list): exact frame switching for 8 step-rate/fps pairs over 20 loops (fails without the slack at 50 Hz/25 fps: checked), no drift over 8 loops, sub-ranges, reverse, once/finished/restart, one long step = many short steps, huge `dt`, invalid fps/dt, single frame, `advance_animations` over several entities, draw list uses the current frame and skips frames outside the grid. ✅ 2. Xvfb: the animated cell equals exactly one sheet frame in each screenshot (frame 1 at zoom 1, frame 4 at zoom 2; all other frames 1,008 / 4,032 px off); rest of the frame 0 mismatches (cursor marker aside). ✅ 3. Timed runs: frame 7 after 119 steps, 2 after 399, 2 after 630 = ⌊steps/15⌋ mod 8. ✅ 4. Regressions: Breakout autoplay unchanged, lose screen pixel-identical to PP-019; GPU tests 9/9; window-destroy fault test clean. |

### PP-021: Screen-space drawing for HUD/UI
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 (in-game UI basics) · Priority P1 · **DONE** (2026-10-07) |
| Dependencies | PP-018b, PP-020 (DONE) |
| Scope (as built) | ADR-029. New `src/render/screen.rs`: public `ScreenSpace { anchor }` (9 constants, default `CENTER`; `anchor_point`, `from_window`, `to_window`) and `ScreenAnchor`. `View` gains `logical_size`; quads, sprites and text with `ScreenSpace` use an anchor-origin orthographic projection in logical pixels (+Y up); `TextPlacement` takes the projection, so screen text stays pixel-aligned. Sort key `(space, layer, material, entity, glyph)`: screen space after all world content. Sandbox: a top-left HUD panel with text and a bottom-right square. Breakout unchanged (its world-space HUD still works; moving it is optional future polish). No new dependencies. |
| Acceptance criteria | ✅ 1. Unit tests: 9 anchors (projection, `to_window`, `from_window`), resize, DPI 1/1.25/2, `CENTER` = default camera, order (screen above world layer 100), camera independence, pixel-aligned screen text. ✅ 2. GPU (`--ignored`): a `BOTTOM_RIGHT` square covers exactly x 236..252, y 116..124 over a layer-100 world quad with two cameras (all 32,768 pixels checked); all three screen-space tests fail when the projection is ignored (mutation check). ✅ 3. Xvfb: HUD crops pixel-identical at cameras (0,0)×1, (450,−250)×2, (−600,300)×0.5; panel border and corner square exact; after resizing to 900×500 the square is at exactly (850..889, 450..489); world model unchanged (0 mismatches apart from the cursor marker). ✅ 4. Breakout autoplay unchanged, lose screen pixel-identical to PP-020; window-destroy fault test clean. |

### PP-022: Audio, part 1: sound effects
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 (runtime essentials) · Priority P1 · **DONE** (2026-10-07) |
| Dependencies | none |
| Scope (as built) | ADR-030. Dependencies `cpal` 0.18.2 + `hound` 3.5.1 (+5 Linux / +3 Windows / +9 macOS crates). New public module `audio` (`SoundId`); crate-private `audio/sound.rs` (`Sounds` store, `decode_wav`: PCM 8–32-bit + float, mono/stereo), `audio/mixer.rs` (32 voices, volume 0..4, linear resampling, channel mapping, clamping), `audio/output.rs` (default device via `cpal`, lock-free `mpsc` commands, f32/i16/u16/i32 devices, silent fallback). `Context::load_sound`, `play_sound(id, volume)`, `audio_available`; `EngineConfig::audio` + `with_audio`. CI installs `libasound2-dev` on Linux. Generated assets `assets/sounds/{blip,hit,lose}.wav` (22,050 Hz mono). Sandbox: blip on every click stamp, exit line prints audio availability. Breakout: bounce/brick/lost sounds queued by the simulation and played after each fixed step. PD-06 deferred again. |
| Acceptance criteria | ✅ 1. Unit tests: WAV decoding (16-bit mono, 24-bit stereo, float, garbage, 3 channels), store (dedup, `Error::Asset`, failed loads not cached), shipped sounds decode, mixer (volume, buffer continuity, sum + clamp, stereo/mono/surround mapping, 2× up- and down-resampling, invalid volume, empty sound, voice limit), `Context` load/play without a device. ✅ 2. End to end via ALSA `file` plugin (2 ch, 48 kHz, f32): one sandbox click → exactly the expected resampled blip × 0.8 in both channels (max difference 0.0, 2,880 frames), nothing else. ✅ 3. No device: `audio disabled` warning, sandbox runs and exits 0, `audio output available: false`. ✅ 4. Breakout autoplay identical with an ALSA null device and without a device (`win` 7135 / 220, `lose` 892 / 14); lose screen pixel-identical to PP-021. ⏳ 5. Windows/macOS: compiled by CI only; the owner should hear the Breakout sounds on Windows. |

### PP-023: UI buttons
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 (in-game UI basics) · Priority P1 · **DONE** (2026-10-07) |
| Dependencies | PP-021, PP-018b |
| Scope (as built) | ADR-031. New public module `src/ui/mod.rs`: `Button { size }` (`is_hovered`, `is_pressed`, `clicked`), `Pointer` (`from_input`: cursor + left button) and `update_buttons(world, pointer, viewport)`; topmost button (layer, then newest) wins; hidden/unanchored buttons inert; unseen release disarms. Sandbox: a "Reset camera" button (top-right, 160×40, idle/hover/pressed colours); clicks on it don't stamp; the timed exit prints `reset button clicks`. No new dependencies. |
| Acceptance criteria | ✅ 1. Unit tests (9 + 2 doctests): edges, one-update click, quick press+release, drag off/onto, unseen release, topmost by layer and age, hidden/unanchored, anchors + scale + two window sizes, `Pointer::from_input`. ✅ 2. Xvfb XTEST (camera started at (0,120)×2): button pixel `#3A86FF` idle → `#6FA8FF` hover → `#1D5FCC` held → `#6FA8FF` after release; exactly 1 click recorded, camera reset to (0,0)×1; press on the button + release outside = no click (back to idle colour); 3 presses seen, but only the click away from the button stamped a square. ✅ 3. Regressions: Breakout autoplay unchanged, lose screen pixel-identical to PP-022; sandbox model 0 mismatches (button area excluded), HUD exact; GPU tests 10/10; fault test clean. |

### PP-024: Audio, part 2 (split on 2026-10-07 into PP-024a + PP-024b)
Music needs two independent things: control over running playbacks (looping, stop, volume) and a compressed format.
Control is the architectural part (ids, commands, mixer); the decoder is a dependency decision. Split to keep each
verifiable on its own.

### PP-024a: Playback control and looping
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 · Priority P1 · **DONE** (2026-10-07) |
| Dependencies | PP-022 (DONE) |
| Scope (as built) | ADR-032. `audio::PlaybackId`; `Context::play_sound` returns it; new `loop_sound`, `stop_sound`, `set_sound_volume`, `stop_all_sounds`, `set_master_volume`, `master_volume`, `sound_duration`. Mixer: commands `Play`/`Stop`/`SetVolume`/`SetMaster`/`StopAll` via `Mixer::apply`, seamless looping with interpolation across the seam, master volume, voice limit drops one-shots before loops. New asset `assets/sounds/loop.wav` (generated 1 s periodic chord). Sandbox: `M` toggles the loop at volume 0.5; the exit line prints `music playing`. No new dependencies. |
| Acceptance criteria | ✅ 1. Unit tests: loop across buffers, seam interpolation, stop, set volume, master, stop-all, unknown ids, voice-limit priority, silent voices, `Context` API without a device (ids distinct, master clamps, duration 0.06 s for blip). ✅ 2. ALSA `file`-plugin capture: `M`…`M` → the loop matches the model exactly (max diff 0) for 3,522,960 frames (73.4 loops), both channels equal, all zero after the stop. ✅ 3. Regressions: Breakout autoplay unchanged, lose screen pixel-identical to PP-023; sandbox model/HUD/animation unchanged; GPU tests 10/10; fault test clean. |

### PP-024b: OGG Vorbis decoding
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 · Priority P1 · **DONE** (2026-10-07; verified on Linux) |
| Dependencies | PP-024a (DONE) |
| Why now | Music as WAV is ~10 MB per minute; games ship music as OGG Vorbis. |
| Scope (as built) | ADR-033. New dependency `lewton` 0.10.2 (+4 crates on every platform). `audio/sound.rs`: `decode` picks the decoder from the first bytes (`RIFF` → `hound`, `OggS` → `lewton`, else `Error::Asset` "not a WAV or OGG Vorbis file"); `decode_ogg` decodes fully on load to interleaved `f32` (mono/stereo, consistent chained streams). New asset `assets/sounds/loop.ogg` (`loop.wav` via ffmpeg/libvorbis q4, 4.9 KB, exactly 22,050 frames); the sandbox's `M` loop plays it. `load_sound` docs list both formats and the memory/load-time cost. |
| Acceptance criteria | ✅ 1. Unit tests (+3, and `loop.ogg` added to the shipped-sounds test): the shipped OGG has the source's channels, rate and exact length, RMS difference 0.0019 (limit 0.005); format by content (an OGG named `.wav` loads; unknown magic rejected); broken OGG (magic only, garbage, cut headers, damaged page) → error / `Error::Asset`; `Context::load_sound("sounds/loop.ogg")` = 1.0 s. ✅ 2. ALSA `file`-plugin capture: `M`…`M` plays `loop.ogg` exactly as the mixer model predicts from `lewton`'s output (max diff 0) for 6,290,400 frames (131 loops), L = R, silent after the stop. ✅ 3. Scratch files: stereo channel order, 3 channels rejected, chained file joined, rate-changing chain rejected. ✅ 4. Regressions: Breakout autoplay unchanged, lose screen pixel-identical to PP-024a; sandbox model 0 mismatches (cursor marker aside), HUD/button crops identical at 3 cameras; fault test clean. |

### PP-025: Per-texture sampling
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P2 · Priority P1 · **DONE** (2026-10-07; verified on Linux). Last P2 item: phase P2 complete. |
| Dependencies | PP-019 (DONE) |
| Why now | The last planned P2 item. Friction point F9: scaled-down or rotated sprites alias because every texture is sampled with `Nearest` (Breakout's ball). Pixel art wants `Nearest`, smooth art wants `Linear`. |
| Scope | Decide the API (ADR): a sampling choice per texture chosen at load (e.g. `Context::load_texture_with(path, TextureOptions { filter })`, default `Nearest` so existing games are unchanged); one sampler per filter mode, chosen when the texture's bind group is built (no extra draw-list state beyond the texture batch). Document that sprite-sheet cells with `Linear` bleed into neighbours unless the sheet has spacing (ADR-020 extension). Use `Linear` where F9 was seen (Breakout's ball) only if it does not change the deterministic gameplay output. |
| Scope (as built) | ADR-034. Public `render::TextureFilter { Nearest (default), Linear }`, `render::TextureOptions { filter }` (non-exhaustive; `NEAREST`, `LINEAR`, `with_filter`), `Context::load_texture_with(path, options)`; `load_texture` = Nearest. `Textures` cache keyed by (path, options); `Linear` textures get `bleed_transparent_texels` on load (transparent texels take their neighbours' colour, alpha 0 kept). `SpritePipeline` holds a `Nearest` and a `Linear` sampler and binds each texture with its own. Breakout's ball loads `Linear`. No new dependencies; draw list, shader and batching unchanged. |
| Acceptance criteria | ✅ 1. Unit tests (+3, one extended): defaults, cache by (path, options), Nearest keeps the file's texels, bleeding (rings, averages, alpha > 0 sources, empty textures, the ball becomes all white with unchanged alpha), `Context::load_texture_with` + filters; doctest for `TextureOptions`. ✅ 2. Ignored GPU readback test (2×2 texture at 32×, the 2×2-at-4× idea made larger so the blend is measurable): `Nearest` exact, `Linear` = CPU bilinear within 2/255, and a `Linear` white→transparent-black strip over blue has no fringe (blue ≥ 254); mutation checks fail as expected (no bleeding → blue 251; wrong sampler → blend mismatch). ✅ 3. Regressions: all earlier GPU tests pass; sandbox model 0 mismatches (cursor marker aside), HUD/button crops identical at 3 cameras; Breakout autoplay unchanged; lose screen pixel-identical (no ball on it); mid-game screenshots: ball edges blended, nothing darker than the background; fault test clean. |

### PP-026: Scene files, part 1 (split on 2026-10-07 into PP-026a + PP-026b)
The format decision, the asset references and the drawing components are one verifiable slice (a scene that renders
the same picture); `SpriteAnimation` (private playback state), `Velocity` and `ui::Button` add questions of their own
(save playback position or not, entities without drawables). Original definition kept below.

| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3 (editor foundations) · Priority P1 · split |
| Dependencies | PP-025 (DONE; phase P2 complete) |
| Why now | First item of P3. An editor (P5) and a debug overlay (P4) both need entities that can be written to and read from files; the engine's own components are the part that does not need a component registry yet. |
| Scope | ADR first: file format and dependency (measure per ADR-013; candidates `serde` + `ron`, `serde` + JSON/TOML, or a hand-written format), how assets are referenced (relative asset paths plus `TextureOptions`, never per-run ids), and versioning. Then save and load the engine's components (`Transform2D`, `Quad`, `Sprite` incl. region, `Layer`, `Hidden`, `ScreenSpace`, `Text` incl. font, `SpriteAnimation`, `Velocity`) for a whole `World`, through `Context` (load resolves assets via the asset root). Game components and a registry are later tasks. Split again if the ADR shows it is too large. |
| Acceptance criteria | Round-trip unit tests (save → load → equal components, including asset references); typed `Error::Asset` for broken or unknown-version files; crate counts measured; the sandbox (or a small example) saves and reloads a scene and renders the same picture (Xvfb pixel comparison); CI green. |

### PP-026a: Scene files: format decision + drawing components
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3 · Priority P1 · **DONE** (2026-10-07; verified on Linux) |
| Dependencies | PP-025 (DONE) |
| Scope (as built) | ADR-035. New dependencies `ron` 0.12.2 + `serde` 1.0.229 (`derive`): +5 crates on every platform (Linux 137 → 142, Windows 110 → 115, macOS 112 → 117); JSON (+6/+7) and TOML (+9) measured and rejected. New crate-private `src/scene/mod.rs`: serde mirror types (`SceneFile`, `EntityFile`, `*File`), `capture` (entities with `Quad`/`Sprite`/`Text`, sorted by entity id; asset paths relative to the asset root), `to_text`/`from_text` (version checked first, unknown fields rejected, `implicit_some`), `resolve_assets` (all assets before any spawn) and `spawn`. New `Context::save_scene(path)` / `Context::load_scene(path) -> Vec<Entity>`; new `Error::Save { path, source }`. New `examples/scene.rs` (modes `load`/`build`/`save`) and `assets/scenes/demo.ron` (14 entities, written by the example). |
| Acceptance criteria | ✅ 1. Unit tests (+7): full round trip of every saved component/option into a fresh world and back to identical text; relative `/` asset paths, absolute outside the root; hand-written defaults, empty entities, CRLF; bad version / garbage / missing version / unknown fields named / extra top-level field; missing asset → `Error::Asset`, nothing spawned; shipped `demo.ron` loads and saves back byte-for-byte; `Context` save/load relative to the asset root, cache reuse, `Error::Save`, failed loads spawn nothing. Mutations (drop `ScreenSpace` on load, drop tint on save) fail the tests. A real bug was caught on the way: asset ids were consumed for entities without a sprite (fixed with per-entity resolution). ✅ 2. Xvfb: the example's `build`, `save` and `load` modes render pixel-identical 1024×600 frames. ✅ 3. Crate counts measured. ✅ 4. Regressions: GPU tests 11/11; sandbox model 0 mismatches, HUD/button crops unchanged; Breakout autoplay unchanged, lose screen pixel-identical; fault test clean. |

### PP-026b: Scene files: animation, velocity and UI buttons
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3 · Priority P1 · **DONE** (2026-10-07; verified on Linux) |
| Dependencies | PP-026a (DONE) |
| Why now | Completes "the engine's own components" in scenes before game components (registry) are tackled: the sandbox and Breakout use `SpriteAnimation`, `Velocity` and `ui::Button`, which PP-026a drops on save. |
| Scope | Extend scene version 1 compatibly (new optional fields, old files still load): `SpriteAnimation` (grid, range, fps, mode; decide in the ADR-035 extension whether the playback position is saved — probably yes, for editor snapshots), `ecs::Velocity`, `ui::Button` (size). Decide whether entities whose only engine components are these (no drawable) are saved. Keep serde private. |
| Acceptance criteria | Round-trip unit tests for the new components (including an animation mid-frame); a version-1 file from PP-026a (the shipped `demo.ron`) still loads unchanged; the scene example gains an animated sprite and a button and still renders `load` = `build` (Xvfb); CI green. |
| Scope (as built) | ADR-035 extension. Scene version 1 gains optional `animation` (grid, range, fps, mode, `step`, `time_in_frame`, `finished`), `velocity` and `button` (size) fields; saved entities: those with `Quad`/`Sprite`/`Text`/`SpriteAnimation`/`Button`. Crate-private `SpriteAnimation::playback` / `with_playback` (validating). PP-026a's `demo.ron` kept as the fixture `src/scene/fixtures/demo_v1_pp026a.ron`. Example: animation saved at frame 5, drifting square (`Velocity`), "Click me" button; `PURPLEPIE_SCENE_FREEZE=1` stops animation and movement; `assets/scenes/demo.ron` regenerated (18 entities). No new dependencies. |
| Result | ✅ 1. Unit tests (+3, two extended): full round trip incl. mid-frame reversed animation on a spaced grid, finished one-shot without sprite, velocity, button (whole-value equality); restored animations match the originals for 40 steps; PP-026a fixture loads and saves back byte-for-byte; `with_playback` validation. Mutations (drop `time_in_frame`, drop `Velocity`) fail the tests. ✅ 2. Xvfb: frozen `save`/`build`/`load` pixel-identical; loaded animated cell = frame 5 (all unoccluded pixels exact); unfrozen it animates and moves; 2 XTEST clicks on the loaded button counted, 1 elsewhere ignored. ✅ 3. Regressions: GPU tests 11/11; sandbox model 0 mismatches, HUD/button unchanged; Breakout autoplay unchanged, lose screen identical; fault test clean. |

### PP-027: Scene files, part 2: game components
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3 (component registry / reflection item) · Priority P1 · **DONE** (2026-10-07; verified on Linux) |
| Dependencies | PP-026b (DONE) |
| Why now | Scenes now hold every engine component, but a game's own components (Breakout's bricks, the sandbox's mover) are dropped, so no real level can live in a file yet; the editor (P5) needs the same mechanism to show and edit them. |
| Scope | ADR first: how a game registers a component type for scenes (e.g. `Context`/`EngineConfig` registration with a name and save/load functions; whether `serde` becomes part of the public API or games convert to/from a small engine-owned value type), how unknown component names in a file are handled (error vs. kept), and how entity references inside components are written. Then implement registration + save/load for registered components, keeping existing version-1 files loading. Split if the ADR shows it is too large. |
| Acceptance criteria | A game component registered by an example round-trips through a scene file; files without game components still load (PP-026a fixture); unregistered names give a clear error; crate counts measured if dependencies change; CI green. |
| Scope (as built) | ADR-036. New `Context::register_scene_component::<T>(name)` (`T: hecs::Component + Serialize + DeserializeOwned`; serde enters the public API via this bound). New crate-private `src/scene/registry.rs` (`Registry` in `EngineState`: name + monomorphised has/save/load fns; clash rules → `Error::InvalidConfig`). Scene version 1 gains the optional `components: { "name": <RON> }` field (`ron::value::RawValue`, trimmed on parse); entities with a registered component are saved; unknown names / bad values → `Error::Asset` before anything loads or spawns. No new dependencies (no crate-count change). Example: `Spin` (two rotating squares) and `Visits` (an entity with nothing drawn, counted on load) registered; `demo.ron` regenerated (19 entities). |
| Result | ✅ 1. Unit tests (+4, one replaced): round trip of structs/enums/options/vecs/unit/newtype components incl. a game-only entity, back to identical text; unregistered dropped/skipped; unknown name and bad value errors spawn nothing; PP-026a fixture still loads; registration rules; `Context` registration + `InvalidConfig` + save/load. Mutation (skip inserting decoded components) fails 3 tests. ✅ 2. Xvfb: frozen `save`/`build`/`load` pixel-identical (and identical to PP-026b's frame); unfrozen, the loaded squares rotate and the text does not change; the example prints "demo scene loaded 1 time(s)" and "2 spinning sprites". ✅ 3. Regressions: GPU tests 11/11; sandbox model 0 mismatches; Breakout autoplay unchanged, lose screen identical; fault test clean. |

### PP-028: Asset hot reload, part 1: textures
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3 (asset hot reload item) · Priority P1 · **DONE** (2026-10-08; verified on Linux) |
| Dependencies | PP-027 (DONE) |
| Why now | The next P3 item. An editor (P5) and fast iteration on art both need changed files to show up without restarting the game; textures are the most common case and already have a CPU store and a GPU re-upload path (`sync_textures`, renderer recreation). |
| Scope | ADR first: how changes are detected (polling modification times on a timer vs. a file-watcher dependency such as `notify`, measured per ADR-013), whether reload is opt-in (`EngineConfig`) or a `Context` call, and what happens on a broken file (keep the old pixels, log). Then reload a changed PNG under the same `TextureId` (same size or not; filter kept, `Linear` bleeding re-applied) and re-upload it. Fonts, sounds and scenes are later parts. |
| Acceptance criteria | Unit tests for change detection and replacement (same id, new pixels/size, broken file keeps the old texture); an Xvfb run where overwriting a PNG while the sandbox runs changes the picture without a restart; no change in behaviour when reload is off; CI green. |
| Scope (as built) | ADR-037. `EngineConfig::hot_reload` / `with_hot_reload` (default off; the sandbox turns it on). `Textures::reload_changed` (file stamp = mtime + length per entry, stamped before reading; same id, filter and bleeding; `revision` bump; broken file logged once and kept; missing file ignored) called by the runner every 0.5 s before drawing. `SpritePipeline::sync_textures` re-uploads entries whose revision changed; an oversized replacement keeps the old GPU copy. `notify` measured (+7/+9/+5 crates) and rejected: no new dependencies. Also: ROADMAP gains phase **P3.5 Usability and first release** between P3 and P4 (owner decision). |
| Result | ✅ 1. Unit tests (+1, two extended): reload with new size/pixels under the same id for a Nearest and a Linear copy, bleeding re-applied, revision bumps; broken file reported once, old pixels kept; fixed file reloads; deleted file ignored; config default/builder. ✅ 2. Ignored GPU test: re-upload under the same id; oversized replacement kept without error or retry. ✅ 3. Xvfb: overwriting `sandbox_quadrants.png` in the running sandbox inverted the sprite (sample (230,57,70) → (25,198,185)), a half-written file changed nothing (1 warning), restoring the file restored the original pixels exactly; with hot reload off (scene example) nothing changed and nothing was logged. ✅ 4. Regressions: GPU tests 12/12; sandbox model 0 mismatches; Breakout autoplay unchanged, lose screen identical; fault test clean. |

### PP-029: Asset hot reload, part 2: fonts and sounds
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3 (asset hot reload item) · Priority P1 · **DONE** (2026-10-08; verified on Linux) |
| Dependencies | PP-028 (DONE) |
| Why now | Completes hot reload for every asset kind the engine loads (scenes are spawned data, not handles, and stay out). Same mechanism as ADR-037, so it is a small, contained step before the workspace split. |
| Scope | Extend the ADR-037 poll to the font and sound stores: a changed font file replaces the parsed font under the same `FontId` and drops that font's glyphs from the atlas cache so text is re-rasterized; a changed sound replaces the decoded samples under the same `SoundId` (voices already playing keep the old samples via their `Arc`; new plays use the new ones). Broken files keep the old asset, logged once. Share the stamp/poll code with textures rather than copying it. |
| Acceptance criteria | Unit tests per store (same id, new data, broken file kept, reported once); a glyph-cache test showing the old font's glyphs are not reused; an Xvfb run where replacing the sandbox font file changes the help label; a capture run where replacing `blip.wav` changes the next click's sound; CI green. |
| Scope (as built) | ADR-037 extension. New crate-private `src/assets/watch.rs` (`FileWatch`, `ReloadReport<Id>`, `reload_if_changed`; the texture store now uses it too). `Fonts::reload_changed` + store `revision`; the renderer clears the glyph atlas when it changes (new `GlyphAtlas::clear`, not counted as full). `Sounds::reload_changed` (new `Arc` per changed sound; sounds not from a file are skipped). The runner polls all three stores every 0.5 s. No new dependencies; no public API change. |
| Result | ✅ 1. Unit tests (+4): shared watcher, font reload, sound reload (playing copy keeps old samples), atlas clear. ✅ 2. Xvfb: swapping the sandbox font for DejaVu Sans changed the help label and HUD text; a broken font file changed nothing (1 warning); restoring the file restored the original pixels exactly. ✅ 3. ALSA capture: click → exactly `blip.wav`; after replacing it with `hit.wav`, the next click → exactly `hit.wav` (max diff 0, L = R). ✅ 4. Regressions: GPU tests 12/12; sandbox model 0 mismatches, label IoU unchanged (0.767); Breakout autoplay unchanged, lose screen identical; fault test clean. All test assets restored byte-for-byte. |

### PP-030: Workspace split (decision: declare the workspace, no move)
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3 (workspace split item, the last P3 item) · Priority P1 · **DONE** (2026-10-08; verified on Linux). Phase P3 complete. |
| Dependencies | PP-029 (DONE) |
| Why now | ADR-026 kept one crate until an editor or another consumer needs a separate one; P4 (debug overlay, likely egui behind a feature) and P5 (editor app) will, and P3.5's guide and release must describe the final crate layout. Doing the split last in P3 lets P3.5 document a stable structure. |
| Scope | ADR first: target layout (e.g. a Cargo workspace with the `purplepie` engine crate unchanged for games, examples and the sandbox as before, and room for `purplepie-editor` later), what moves (probably nothing in the engine API; sandbox/examples placement; shared `assets/`), CI and `cargo run` commands, Windows paths in the owner's command block. Then perform the move with no behaviour change. If the ADR concludes a split brings nothing yet, record that and close the item instead. |
| Acceptance criteria | `cargo test --workspace`, clippy, docs and all examples build and pass; sandbox/Breakout/scene example behave identically (existing Xvfb regressions); asset root still found from the new layout; CI updated; game-facing API unchanged. |
| Scope (as built) | ADR-038. A scratch trial of moving the engine to `crates/purplepie` measured the cost (library stops compiling: 3 `include_bytes!`; 25 `CARGO_MANIFEST_DIR` asset paths in 7 files; 8 tests still failing after a naive rewrite; changed run commands). Decision: no move; the root `Cargo.toml` gains `[workspace] members = []` (engine stays the only, default member; future crates under `crates/`). No source, API, dependency, `Cargo.lock` or CI change. |
| Result | ✅ 1. Gates unchanged with and without `--workspace`; `Cargo.lock` byte-identical; `cargo metadata`: workspace root = repository root, single default member `purplepie`. ✅ 2. A copy nested inside a folder with its own `[workspace]` failed to build before ("believes it's in a workspace when it's not") and builds after. ✅ 3. Regressions: GPU tests 12/12; sandbox model 0 mismatches and asset root found at `assets/`; scene example identical to PP-027's frame; Breakout autoplay unchanged, lose screen identical. |

### PP-032b: Module docs for game authors + starter-template decision
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3.5, item 2, part 2 · Priority P1 · **DONE** (2026-10-09; verified on Linux) |
| Dependencies | PP-032a (DONE) |
| Acceptance criteria | ✅ Every public module page (`audio`, `ecs`, `input`, `math`, `render`, `ui`) starts with a one-line summary for game authors, a paragraph naming the `Context` methods and guide section, and a compiled example (4 new doctests: 43 → 47); engine internals and the ADR trail moved under "Engine notes". ✅ `cargo doc --no-deps` with `-D warnings` clean; rendered crate page checked. ✅ U-07 marked fixed. ✅ Template decision recorded as ADR-039 (no template crate; the guide is the template), backed by a scratch trial: a template crate inside the repository fails `cargo check` ("believes it's in a workspace"). No API change. |

### PP-032a: Newcomer guide, compiled by `cargo test`
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3.5, item 2, part 1 · Priority P1 · **DONE** (2026-10-08; verified on Linux) |
| Dependencies | PP-031 (DONE) |
| Acceptance criteria | ✅ `docs/GUIDE.md`: 14 sections from `cargo new` to shipping (crate setup and assets, game loop, coordinates and camera, entities and queries, quads and sprites, sheets and animation, text, input names, HUD and buttons, sound, scene files, hot reload / errors / logs, shipping, a complete game). ✅ Every Rust block (12) is compiled by `cargo test` through a `#[cfg(doctest)]` include in `src/lib.rs`; a mutation (`KeyCode::KeyA`) fails the test; CRLF line endings also pass. ✅ Linked from the README (Getting started, layout) and the crate page. ✅ Fresh outside crate from the guide alone: builds without warnings, clippy clean, runs under Xvfb (first frame, brick collected with blip captured, Restart click; screenshots checked). ✅ U-08 fixed. |

### PP-031: Outside-crate trial
| Field | Value |
|---|---|
| Stage | Post-portfolio phase P3.5 (Usability and first release), item 1 · Priority P1 · **DONE** (2026-10-08; verified on Linux) |
| Dependencies | PP-030 (DONE; phase P3 complete) |
| Acceptance criteria | ✅ A trial crate `pie_catch` outside the package (path dependency, own `assets/`) built from the README and API docs alone: sprites, text, keyboard, a sound, a scene file with two registered game components; ran under Xvfb (screenshots checked: play, game over, restart; sound captured), and its release build ran shipped with `assets/` from another working directory. ✅ `docs/USABILITY.md` lists 17 findings (U-01…U-17) with class and owner task. ✅ README ("A new game crate", font copying, serde, scene folder, `cargo doc --no-deps`), crate-page and `ecs` docs fixed (new doctest); re-checked by a fresh `cargo new` crate following the README word for word. ✅ Follow-ups recorded (PP-032…PP-036). No engine API change. |

### PP-033a: Platform checklist, written and run on Linux
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 3, part 1 · Priority P1 · **DONE** (2026-10-09; verified on Linux) |
| Dependencies | PP-032b (DONE) |
| Acceptance criteria | ✅ `docs/CHECKLIST.md`: setup + 8 numbered steps in PowerShell (tests, GPU tests on real hardware, sandbox incl. input/sound/window, hot reload with `git checkout` restore, Breakout play + autoplay lines, scene example, a new crate from GUIDE.md section 14 outside the repository and OneDrive, shipping its release build) with what to expect and what to report, plus a macOS/Linux appendix. ✅ Every step run on Linux: tests 216 + 47 and 12/12 GPU (lavapipe); sandbox frame, `GPU:`/`audio:` lines, click + music audible in an ALSA capture; hot reload swapped and restored the texture (two `reloaded texture` lines, file byte-identical afterwards); autoplay lines `Lost after 892 …` / `Won after 7135 …`; scene: 19 entities, 2 clicks counted, build mode; steps 7–8 executed **as PowerShell** with PowerShell 7.4.6 for Linux (only `cargo run`→`cargo build`, `.exe` and one `.NET` path separator adapted): crate built without warnings, game frame correct, release build shipped and run from `/` showed the same frame; bash appendix extraction line tested. ✅ Linked from DEVELOPMENT §8 and the README. |

### PP-033b: Owner run of the checklist
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 3, part 2 · Priority P1 · **DONE** (2026-10-09; owner-reported) |
| Dependencies | PP-033a (DONE) |
| Result | Owner, Windows: step 1 `cargo test` 216 + 47 green; step 2 GPU tests 12/12 on an AMD Radeon integrated GPU (Vulkan); step 3 GPU and audio lines as expected; all remaining steps (hot reload, Breakout incl. autoplay, scene example, new crate from the guide, shipped release build) reported "as expected". The exact autoplay lines were not quoted, so cross-platform determinism of the Breakout simulation is taken as matching but not recorded number-for-number. |
### PP-034b: Scene component formatting + opt-in console logger
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 4, part 2 · Priority P1 · **DONE** (2026-10-09; verified on Linux) |
| Dependencies | PP-034a (DONE) |
| Acceptance criteria | ✅ U-12: registered components are written as one spaced RON line (unit test with nested struct, enum, tuple array, map, `Option`, escaped string and a unit marker; exact text + exact read-back); `demo.ron` regenerated (3 lines differ) and the shipped-scene round-trip test passes; scene example `save`/`load`/`build` frames pixel-identical to the PP-026b baseline. ✅ U-13: `EngineConfig::console_log` / `with_console_log` (doctest); `app::logging` (the sandbox's logger moved in), level from `PURPLEPIE_LOG` with empty = `warn` and invalid = `Error::InvalidConfig` (unit test; sandbox run with `loud` exits 1 with the message); sandbox, Breakout and scene opt in: `PURPLEPIE_LOG=info` Breakout prints the asset-root, audio and GPU lines, without it nothing extra; the outside trial crate (no opt-in) prints nothing even with `PURPLEPIE_LOG=info`. ✅ Breakout `Lost after 892 …` and lose screen pixel-identical. ✅ README, guide (new compiled block), ARCHITECTURE §9, ADR-016 amendment, ADR-040 implementation note updated. 220 unit tests + 49 doctests. |

### PP-034a: Second public API review
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 4, part 1 · Priority P1 · **DONE** (2026-10-09; verified on Linux) |
| Dependencies | PP-032b (DONE) |
| Acceptance criteria | ✅ ADR-040: review table over every public area added since ADR-026 with a verdict each. ✅ U-09: `ScreenSpace::TOP_CENTER`/`CENTER_LEFT`/`CENTER_RIGHT`/`BOTTOM_CENTER` and `ScreenAnchor::TopCenter`/… (breaking rename, no aliases); scene files write the new names and read the old ones (serde aliases; unit test `edge_anchors_load_under_their_old_and_new_names`; the trial's pre-rename `level.ron` still loads). ✅ U-11: `Error`'s `Debug` = message + "Caused by:" chain (unit test; trial output checked). ✅ U-10: `Error::Save` carries the resolved path (trial: save into a missing folder). ✅ `#[non_exhaustive]` on `EngineConfig`, `Sprite`, `Quad`, `Text`, `TextMetrics`, `SpriteAnimation`, `AnimationMode`, `TextureFilter`, `Camera2D`, `ui::Button`, `ui::Pointer`; sandbox, examples, doctests and the guide compile unchanged. ✅ Regressions: scene example frame and Breakout lose screen pixel-identical to their baselines; `Lost after 892 …` unchanged. 218 unit tests + 47 doctests. |

### PP-035a: Release preparation
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 5, part 1 · Priority P1 · **DONE** (2026-10-09; verified on Linux) |
| Dependencies | PP-034b (DONE) |
| Acceptance criteria | ✅ `docs/RELEASING.md`: distribution by git tag (`publish = false` kept, reasons and revisit point), 0.x compatibility policy (public API incl. `#[non_exhaustive]` rules and re-exported hecs 0.11 / glam 0.33 / serde 1, scene format 1, assets, MSRV 1.90, platforms; what is not covered), release procedure, CI guards. ✅ `CHANGELOG.md` (Keep a Changelog): `[Unreleased]` = the 0.1.0 content, pre-release breaking changes (ADR-040), known limitations. ✅ CI: `msrv` job (`cargo +1.90 check --locked`) in `ci.yml`; new `latest-deps.yml` (`cargo update` + check + test; push, weekly, manual); both valid YAML. ✅ Locally, in a scratch copy: `cargo update` (12 packages newer) + check + test pass (220 + 49); committed `Cargo.lock` unchanged; MSRV evidence: `incompatible_msrv` clean at 1.90 and firing at 1.85, highest dependency `rust-version` 1.90. ⚠️ Cowork cannot install Rust 1.90 (download blocked), so the `msrv` job's first real run is on GitHub. |

### PP-035b: Release 0.1.0
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 5, part 2 · Priority P1 · **DONE** (2026-10-09; verified on Linux; tag pushed by the owner) |
| Dependencies | PP-035a ✅, PP-033b ✅ (owner: checklist as expected), repository URL ✅ (`https://github.com/DavidVilela1/purplepie`), green CI incl. `msrv` and latest-deps ✅ (owner-reported) |
| Acceptance criteria | ✅ `version = "0.1.0"`, `repository` set; `Cargo.lock` changed only in purplepie's own version line. ✅ CHANGELOG `[0.1.0] - 2026-10-09` + empty `[Unreleased]` + compare/tag links. ✅ README (status, dependency line, asset source), guide section 1 and RELEASING give the git-tag dependency. ✅ The git form was verified against the real repository: a fresh crate with `git = "https://github.com/DavidVilela1/purplepie", branch = "main"` (the tag does not exist yet) built and ran the guide game, frame identical to the PP-032a guide frame; the same crate by path against the 0.1.0 workspace too. ✅ GitHub `main` was byte-identical to the PP-035a archive before this change. Owner action: commit, push, wait for green CI, then `git tag -a v0.1.0` + `git push origin v0.1.0` (RELEASING §3). |
### PP-036a: Second game, part 1: choose and specify it ← NEXT
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 6, part 1 · Priority P1 · TODO |
| Dependencies | PP-035a (DONE). Not blocked by the release (PP-035b) or the owner's checklist: the game is an outside crate depending on PurplePie by path |
| Why now | The remaining P3.5 work that does not wait on the owner. A second, different game written as an outside crate is the test of "someone can make a game without knowing the code": it decides which missing features are real (candidates: random numbers U-14, rectangle overlap U-15, world clearing / scene replacement, tilemaps, timers, camera helpers, gamepad). |
| Scope | Choose the game (different from Breakout and Pie Catch, e.g. a top-down arena shooter or a tile-based puzzle). Write a one-page spec (new `docs/GAME2.md`): mechanics, screens, assets, which engine features it uses and which it lacks. Classify every gap as "write in game code" or "engine feature" and split the engine features into small tasks (PP-036b…). No engine code in this task. |
| Acceptance criteria | GAME2.md exists with a feature/gap table; each engine-feature gap has a task with scope and acceptance; the trial crate skeleton (empty game opening a window) builds outside the repository. |

### PP-036: A second game as an outside crate (umbrella)
| Field | Value |
|---|---|
| Stage | Phase P3.5, item 6 · Split into PP-036a (spec) and follow-ups |
| Scope | A second, different game written as an outside crate, adding only the engine features it needs. Candidates from the trial: random numbers (U-14) and rectangle-overlap helpers (U-15); from the roadmap: world clearing / scene replacement, tilemaps, timers, camera helpers, gamepad. |

### PP-013: Choose project license
| Owner decision · P3 · **DONE** (2026-10-01) | No dependencies. |
|---|---|
| Acceptance criteria | ✅ `license = "MIT OR Apache-2.0"` in `Cargo.toml`. ✅ `LICENSE-MIT` (copyright holder David Vilela, 2026) + `LICENSE-APACHE` (canonical Apache-2.0 text from the cargo registry; the owner then filled in the appendix's copyright line, "Copyright 2026 David Vilela", on GitHub on 2026-10-02). ✅ README License + Contribution sections. ✅ PD-07 closed → ADR-023. |
