# PurplePie Architecture

Last reviewed: **2026-09-30** against the repository after Stage 2 (PP-004).

Every section separates **Current** (exists and is validated in the repository)
from **Planned** (decided in [DECISIONS.md](DECISIONS.md), not yet built).
When code and this file disagree, the code plus validation wins, and this file
must be corrected ([DEVELOPMENT.md §7](DEVELOPMENT.md#7-architectural-drift)).

---

## 1. System Overview

**PurplePie is** a small, modular, desktop 2D game engine library in Rust.
Games are separate binaries that implement a `Game` trait. The engine owns the
window, event loop, time, ECS world, input state and GPU renderer.

**PurplePie is not:**
- a 3D engine, even though wgpu could do 3D;
- a general application framework or a Bevy-style plugin/scheduler system;
- a web or mobile engine (not a goal for now);
- an editor (possible much later).

**Current reality (Stage 2):** `purplepie` provides `Engine`, `EngineConfig`,
`Game`, `Context`, `Time` and `Error`. The `sandbox` game opens a window, runs a
paced 60 Hz frame loop with a fixed-timestep `fixed_update` plus a per-frame
`update`, and exits cleanly. There is no ECS, GPU or input abstraction yet.

---

## 2. Architectural Principles

Priority order when principles conflict: **simplicity → correctness →
compilability → clear architecture → maintainability → extensibility → performance.**

| Principle | What it means here |
|---|---|
| 2D-only scope | Public types are 2D (`Transform2D`, `Camera2D`, `Sprite`). No depth buffer or 3D camera in the API. |
| Rust-first | Stable Rust, edition 2024, `unsafe_code = "forbid"` (ADR-001). |
| Modularity | One responsibility per module. A module exists only once it has a user (ADR-003). |
| Composition | Behavior comes from ECS components + plain-function systems, not inheritance-like trait hierarchies. |
| Low coupling | `winit` only in `app`, `wgpu` only in `render`. ECS and math never depend on rendering. |
| Explicit APIs | No implicit systems, no global state, no hidden schedulers. Game code calls systems itself. |
| Incremental development | One verified stage at a time, each a runnable vertical slice ([ROADMAP.md](ROADMAP.md)). |
| No premature overengineering | No traits, generics, `Arc`/`Mutex`/`RefCell` or dependencies without a present need. |

---

## 3. Module Responsibilities

### Current

| Path | Responsibility | Status |
|---|---|---|
| `src/lib.rs` | Crate root: re-exports the public API, `VERSION` | VERIFIED |
| `src/error.rs` | `Error` (`#[non_exhaustive]`: `InvalidConfig`, `EventLoop`, `Window`, `Game`), `BoxError`, `Result`. winit errors are boxed sources, not public types. | VERIFIED |
| `src/app/mod.rs` | `Engine::new` (validates config, creates the `EventLoop`) and `Engine::run` (runs the runner, returns the first error) | FUNCTIONAL (Linux) |
| `src/app/config.rs` | `EngineConfig`: title, size, resizable, `exit_on_escape`, `fixed_dt`, `max_frame_dt`, `max_fixed_steps` + builders + `validate` | VERIFIED |
| `src/app/game.rs` | `Game` trait (`init`, `fixed_update`, `update`, all with defaults); `Context` (`time()`, `dt()`, `request_exit()`, `exit_requested()`) | VERIFIED |
| `src/time/mod.rs` | `Time` (public, read-only): clamped delta, elapsed game time, frame number, `fixed_dt`, total fixed steps, `alpha` | VERIFIED |
| `src/time/fixed.rs` | `FixedTimestep` (`pub(crate)`): accumulator, step cap, backlog clamp, alpha. `std` only. | VERIFIED |
| `src/app/pacer.rs` | `FramePacer`: 60 Hz `WaitUntil` deadlines, no catch-up bursts. Interim until Stage 4 vsync. | VERIFIED |
| `src/app/runner.rs` | `Runner<G>`: winit `ApplicationHandler`; the only code handling winit events | FUNCTIONAL (Linux) |
| `src/main.rs` | `sandbox` binary: a `Game` using only the public API. Optional timed exit via env var. | FUNCTIONAL (Linux) |
| `assets/{textures,fonts,shaders}/` | Runtime data folders (empty, `.gitkeep`) | Placeholder |

### Planned (ADR-003)

| Module | Responsibility | May depend on | Must NOT depend on | Future extension points | Stage |
|---|---|---|---|---|---|
| `error` | `Error` enum, `Result<T>` | `thiserror`; wraps winit/wgpu error types | any internal module | new variants per failure domain | 1 |
| `app` | `Engine`, `EngineConfig`, `Game`, `Context`, `Runner` (winit `ApplicationHandler`), frame orchestration | all engine modules, `winit` | — (top of the engine) | multiple windows (not planned), headless runner for tests | 1 |
| `time` | `Time` (delta, elapsed, frame count), `FixedTimestep` | `std` only | `winit`, `wgpu`, `hecs` | interpolation alpha, time scale/pause | 2 |
| `math` | `Transform2D`; re-exports `Vec2`, `Affine2`, `Mat4` | `glam` | everything internal | rect/AABB helpers when needed | 3 |
| `ecs` | Re-exports `World`, `Entity`; engine components (`Velocity`); systems (`integrate_velocity`) | `hecs`, `math` | `render`, `app`, `input`, `wgpu`, `winit` | hierarchy/parenting, command buffers | 3 |
| `render` | `pub(crate) Renderer`, `GpuContext`, pipelines; public data types `Color`, `Sprite`, `Camera2D` | `wgpu`, `pollster`, `math`, `ecs` (read-only) | `app`, `input`, `winit` types beyond the `Arc<Window>` handed in | `SpriteRenderer`, `ShapeRenderer`, `TextRenderer`, `DebugRenderer` | 4–7 |
| `input` | `Input` state (pressed / just_pressed / just_released), `KeyCode`, `MouseButton` | `math` | `winit` (translation lives in `app`), `render` | gamepad, text input, action mapping | 8 |
| `assets` | `Handle<T>`, `Assets` store, loaders | `render` (GPU upload), `std::fs` | `app`, `input` | hot reload, async loading | 9 |

**Not present: `core`.** See ADR-003. It had no single responsibility and
shadows Rust's built-in `core` crate.

---

## 4. Dependency Direction

Planned graph (arrows mean "depends on"). Nothing here exists in `src/` yet except the
lib/bin split.

```mermaid
graph TD
    Game["Game code<br/>(src/main.rs, examples/)"] -->|public API only| App
    App[app] --> Render[render]
    App --> Input[input]
    App --> Time[time]
    App --> Assets[assets]
    App --> ECS[ecs]
    App --> Winit[(winit)]
    Assets --> Render
    Render --> ECS
    Render --> Math[math]
    Render --> WGPU[(wgpu)]
    Input --> Math
    ECS --> Math
    ECS --> Hecs[(hecs)]
    Math --> Glam[(glam)]
    App -.-> Error["error<br/>(used by every module)"]
```

Rules:
1. Edges only point downward. An upward import is a design bug.
2. `ecs` and `math` never depend on `render`, `app`, `input` or GPU/window crates.
3. `render` reads the `World` but never writes game state (ADR-009).
4. `time` depends only on `std`.
5. Forbidden cycles: `render → ecs → render`, `app ↔ error`, `game → render internals → game`.

Currently enforced by the compiler: `main.rs` can reach only `pub` items of `purplepie` (ADR-002).

---

## 5. Runtime Flow

### Current (Stage 2)
```text
main → Engine::new(config)?          validate config, EventLoop::new()
     → engine.run(game)              EventLoop::run_app(&mut Runner)
resumed          → create Window (once) → game.init(ctx) (once; error → exit)
about_to_wait    → FramePacer: if a frame is due, request_redraw; ControlFlow::WaitUntil(next deadline)
RedrawRequested  → FRAME (skipped once exit has begun):
                     raw = now − last_frame (0 on the first frame)
                     delta = time.begin_frame(raw, max_frame_dt)       clamp to [0, max_frame_dt]
                     n = fixed.advance(delta)                          ≤ max_fixed_steps, backlog clamped
                     repeat n: game.fixed_update(ctx, dt = fixed_dt)   stop early on request_exit
                     time.set_alpha(fixed.alpha())
                     game.update(ctx, dt = delta)                      unless exit requested
CloseRequested / Escape (if enabled) / ctx.request_exit() → event_loop.exit()
exiting          → drop Window
run returns      → first stored error, else any event-loop error, else Ok(())
```
Game callbacks are never invoked after `event_loop.exiting()` becomes true.
`exit()` does not discard events that are already queued, and a bug caused by
this was found and fixed in PP-003.

### Planned full frame (ADR-008, ADR-010)
```mermaid
sequenceDiagram
    participant M as main (game)
    participant E as Engine / Runner
    participant W as winit
    participant G as Game
    participant R as Renderer
    M->>E: Engine::new(config)? then run(game)
    E->>W: EventLoop::run_app(&mut runner)
    W->>E: resumed
    E->>W: create_window (once)
    E->>R: Renderer::new (Stage 4)
    E->>G: init(ctx)
    loop every frame
        W->>E: about_to_wait → request_redraw
        W->>E: RedrawRequested
        E->>E: time.tick, fixed.advance → n
        E->>G: fixed_update(ctx) × n
        E->>G: update(ctx)
        E->>R: render(&world) → queue.present
        E->>E: input.end_frame
    end
    W->>E: CloseRequested → exit
    W->>E: exiting → drop renderer, then window
    E-->>M: Result<()>
```

Fixed-step constants: `FIXED_DT = 1/60 s`, `MAX_FRAME_DT = 0.25 s`,
`MAX_FIXED_STEPS = 5`, backlog clamped to below one step when the cap is hit.
Stages 1–3 use `ControlFlow::WaitUntil`. From Stage 4, `Poll` + vsync (`Fifo`).

---

## 6. Engine/Game Boundary

| Game code **may** use | Game code **may not** see |
|---|---|
| `Engine`, `EngineConfig`, `Game`, `Context`, `Result`/`Error`/`BoxError` | `wgpu::{Device, Queue, Surface, RenderPipeline, …}` |
| `World`, `Entity`, components (`Transform2D`, `Velocity`, `Sprite`, …) | `winit::window::Window`, raw winit events |
| `Time`, `Input`, `KeyCode`, `Color`, `Camera2D`, `Handle<T>` | `Renderer`, `Runner`, `GpuContext` (`pub(crate)`) |
| built-in system functions (`ecs::integrate_velocity`) | engine-internal state outside `Context` |

The engine runs no gameplay systems implicitly. Order is visible in the game's `fixed_update`.

---

## 7. Rendering Architecture

**Current:** none. A throwaway spike (outside `src/`) verified the wgpu 30 path
and rendered a purple clear under Xvfb + lavapipe
([spikes/stage-0-compat-spike.md](spikes/stage-0-compat-spike.md)).

**Planned (ADR-005, ADR-009):**
```text
Instance::new(InstanceDescriptor::new_with_display_handle(owned_display_handle))
 → create_surface(Arc<Window>) → request_adapter(compatible_surface)
 → request_device → surface.get_default_config → configure
frame: get_current_texture() → render pass → queue.submit → pre_present_notify → queue.present
```

| `CurrentSurfaceTexture` / condition | Action |
|---|---|
| `Success` | draw, present |
| `Suboptimal` | draw, present, then reconfigure |
| `Timeout`, `Occluded` | skip the frame |
| `Outdated` | reconfigure, skip |
| `Lost` | recreate the surface, reconfigure, skip |
| `Validation` | log, skip. Repeated failures become `Error::Render` |
| OOM / device lost | reported via `on_uncaptured_error` / `set_device_lost_callback`, which become `Error::Render` and exit |
| size 0×0 (minimized) | never configure. Skip rendering. |

Evolution, each part added only when a stage needs it:
`Renderer { gpu, sprites (Stage 5–6), shapes, text, debug (later) }`.

---

## 8. ECS Architecture

**Current:** none (no `hecs` dependency yet).

**Planned (ADR-006, ADR-008):**
- **Implementation:** `hecs 0.11.1`, archetypal storage.
- **World ownership:** exactly one `hecs::World`, owned by the runner, lent to
  the game as `ctx.world: &mut World` for each callback.
- **Components:** plain data structs. The first set is `Transform2D`, `Velocity`, then `Sprite`.
  No wgpu handles in components.
- **Systems:** free functions, e.g. `fn integrate_velocity(world: &mut World, dt: f32)`,
  called explicitly by the game. There is no scheduler.
- **Resources:** hecs has none. Engine-wide state (`Time`, `Input`, later
  `Assets`) lives in `Context`. Game-specific state lives in the `Game` struct.
- **Structural changes during iteration:** `hecs::CommandBuffer` or collect-then-apply.

---

## 9. Error Handling

**Current:** `purplepie::Error` with boxed `source`s. `Error::game(e)` wraps game errors.
Errors raised inside winit callbacks are stored by the runner (first one wins),
followed by `event_loop.exit()`, and returned from `Engine::run`. The sandbox
prints the full `source()` chain. There is no logging yet (PD-04, deferred to Stage 4).
The lints `unsafe_code = "forbid"` and `clippy::unwrap_used = "warn"` apply, and `src/` contains no `unwrap`/`expect`.

**Planned (ADR-011):** one `purplepie::Error` (`thiserror`) and
`purplepie::Result<T>`. Callback errors are stored and returned from
`Engine::run`. Diagnostics use the `log` facade, and the engine never installs a logger.

---

## 10. Async Strategy

**Current:** no async code.

**Planned (ADR-012):** only the two wgpu init futures, resolved with
`pollster::block_on` in `resumed`. No async runtime and no `async` in the public API.

---

## 11. Testing Architecture

| Layer | Approach | Current |
|---|---|---|
| `error`, `app::{config, game, pacer}`, `time` | pure unit tests + doctests | ✅ 30 unit tests + 5 doctests |
| `math`, `ecs` systems, `input` state | pure unit tests, no window/GPU | planned (Stages 3, 8) |
| Game logic | build `World`/`Time`/`Input` headless, call game methods | planned (Stage 3+) |
| `app` runner, `render` | Xvfb smoke runs in Cowork (xdotool XTEST keys, `WM_DELETE_WINDOW` close, `xwininfo`, CPU sampling; lavapipe from Stage 4); Windows by the owner | ✅ Stage 1 on Linux; Windows pending |
| Every change | `cargo fmt --check`, `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build` | ✅ in use |

Tests live next to code in `#[cfg(test)] mod tests`. Integration tests go in `tests/`
once the public API has behavior worth testing.
