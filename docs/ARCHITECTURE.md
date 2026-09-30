# PurplePie Architecture

Status: **Stage 0 baseline** (2026-09-30). This document describes the target
architecture. Modules listed here do not exist in `src/` until the stage that
introduces them (see [ROADMAP.md](ROADMAP.md)).

---

## 1. Goals and non-goals

PurplePie is a **2D** engine. `wgpu` is only the GPU abstraction; nothing in
the game-facing API is 3D-shaped (no meshes, no depth-sorted 3D cameras, no
`Transform3D`).

Priority order for every decision: simplicity → correctness → compilability →
clear architecture → maintainability → extensibility → performance.

Non-goals for the foreseeable future: a scheduler/plugin framework ("mini-Bevy"),
multithreaded systems, a scripting layer, web/mobile targets, an editor.
Desktop (Windows, macOS, Linux X11/Wayland) is the target.

---

## 2. Crate layout

One Cargo package, two targets ([ADR-0001](adr/0001-crate-layout.md)):

```text
PurplePie/
├── Cargo.toml          package `purplepie`
├── src/lib.rs          ── library target `purplepie`  (the ENGINE)
├── src/main.rs         ── binary target `sandbox`     (a GAME)
├── src/<modules>/      engine modules, added stage by stage
├── assets/             textures/, fonts/, shaders/
└── docs/               architecture, roadmap, status, ADRs
```

`main.rs` can only reach `pub` items of the library, exactly like an external
game crate. The engine/game boundary is enforced by the compiler, not by
convention.

---

## 3. Modules and responsibilities

| Module | Owns | Public (game-facing) | Crate-private | Depends on | Stage |
|---|---|---|---|---|---|
| `error` | The single engine error type | `Error`, `Result<T>` | — | `thiserror`, `winit`/`wgpu` error types (wrapped, not re-exported) | 1 |
| `app` | Event loop, window, frame orchestration | `Engine`, `EngineConfig`, `Game`, `Context` | `Runner` (the `winit::ApplicationHandler` impl) | everything below | 1 |
| `time` | Clock, frame delta, fixed-step accumulator | `Time` | `FixedTimestep` | `std` only | 2 |
| `math` | 2D math types | `Transform2D`, re-exported `Vec2`, `Mat4`, … | — | `glam` | 3 |
| `ecs` | The world and engine-generic components/systems | `World`, `Entity` (from `hecs`), `Velocity` | built-in systems (`integrate_velocity`) | `hecs`, `math` | 3 |
| `render` | All GPU state and drawing | `Color`, `Sprite`, `Camera2D` (plain data) | `Renderer`, `GpuContext`, pipelines, buffers | `wgpu`, `math`, `ecs` (read-only extraction) | 4–7 |
| `input` | Keyboard/mouse state for the current frame | `Input`, `KeyCode`, `MouseButton` | event translation from winit | `math` | 8 |
| `assets` | Loading & handles | `Handle<T>`, `Assets` | loaders | `render` (upload), `std::fs` | 9 |

Deliberate deviations from the initial sketch in the brief:

* **No `core/` module.** "Core" has no single responsibility, and a module
  named `core` collides with Rust's built-in `core` crate in `use` paths
  (`use core::fmt` becomes ambiguous inside the crate). Its likely contents are
  split into `error` (errors) and `app::EngineConfig` (configuration).
* **`app` is the only module that knows about `winit`.** `input` defines its
  own key/button types; `app` translates winit events into them.
* **Game-facing render types are plain data.** `Sprite`, `Color`, `Camera2D`
  contain no `wgpu` handles, so game code and ECS components never touch GPU
  types. The `Renderer` itself is `pub(crate)`.
* **`assets/` (runtime data folder) ≠ `src/assets/` (loader module).** The module
  is created in Stage 9.

Empty modules are not created ahead of time. Each module arrives with the stage
that gives it a job.

---

## 4. Dependency direction

```text
            ┌──────────────────────────────┐
  GAME      │ src/main.rs (sandbox) / games│
            └──────────────┬───────────────┘
                           │ public API only
            ┌──────────────▼───────────────┐
  FACADE    │ app  (Engine, Game, Context) │   ← only module that sees winit
            └──┬─────────┬─────────┬───────┘
               │         │         │
  SYSTEMS   ┌──▼───┐ ┌───▼───┐ ┌───▼────┐ ┌────────┐
            │render│ │ input │ │  time  │ │ assets │
            └──┬───┘ └───┬───┘ └────────┘ └───┬────┘
               │ reads   │                    │
            ┌──▼─────────▼──┐                 │
  DATA      │  ecs   math   │◄────────────────┘
            └───────┬───────┘
  EXTERNAL     hecs, glam, wgpu (render only), winit (app only)
            error ← used by every layer, depends on nothing internal
```

Rules:

1. Arrows point downward only. No module imports from a layer above it.
2. `ecs` and `math` never depend on `render`, `app`, `input`, or GPU types.
3. `render` reads the `World` (extraction); it never writes game state.
4. `time` depends on nothing but `std`, so it is fully unit-testable.
5. Forbidden cycles, explicitly: `render → ecs → render`, `app ↔ error`,
   `game → render internals → game`.

---

## 5. Ownership model

```text
Engine (owned by main, consumed by run)
└── Runner<G: Game>          implements winit::ApplicationHandler
    ├── game: G              the game's own state (owned, generic, no dyn)
    ├── world: hecs::World
    ├── time: Time
    ├── input: Input
    ├── fixed: FixedTimestep
    ├── window: Option<Arc<Window>>     None until `resumed`
    └── renderer: Option<Renderer>      None until `resumed` (Stage 4)
```

* Single-threaded. No `Mutex`, `RefCell`, `Rc`, or global state.
* The one `Arc` is `Arc<Window>`: `wgpu::Surface<'static>` must share ownership
  of the window. It never leaves `app`/`render`.
* `Window` and `Renderer` are `Option` because winit 0.30 only allows window
  creation once the event loop is running (`resumed`).
* Each callback into the game builds a short-lived `Context<'_>` from
  **disjoint field borrows** of `Runner`, so the borrow checker proves there is
  no aliasing, with no interior mutability needed.

---

## 6. Game-facing API (target shape)

Accepted direction in [ADR-0004](adr/0004-engine-game-api.md); refined in Stage 10.

```rust
use purplepie::{Context, Engine, EngineConfig, Game, Result};

#[derive(Default)]
struct Sandbox { player: Option<purplepie::Entity> }

impl Game for Sandbox {
    fn init(&mut self, ctx: &mut Context) -> Result<()> {
        self.player = Some(ctx.world.spawn((Transform2D::default(), Velocity::default())));
        Ok(())
    }
    fn fixed_update(&mut self, ctx: &mut Context) { /* gameplay at 60 Hz */ }
    fn update(&mut self, ctx: &mut Context) { /* per-frame, variable dt */ }
}

fn main() -> purplepie::Result<()> {
    let engine = Engine::new(EngineConfig::new("Sandbox").with_size(1280, 720))?;
    engine.run(Sandbox::default())
}
```

`Context` exposes `world: &mut World`, `time: &Time`, `input: &Input`, and
`request_exit()`. The engine runs **no gameplay systems implicitly**. Built-in
systems such as `ecs::integrate_velocity` are plain functions that the game
calls from `fixed_update`, so execution order is always visible in game code. It deliberately does **not** expose `wgpu` or `winit` types.
Rendering is engine-driven: the renderer draws whatever `Sprite` +
`Transform2D` entities exist after the update phase.

---

## 7. Frame lifecycle

winit 0.30 `ApplicationHandler` with `ControlFlow::Poll` (Stage 1 may use
`WaitUntil` until vsync provides pacing; see RISKS R-06).

```text
resumed ─────────────► create Window (once) ─► (Stage 4) create Renderer
                                               ─► game.init(ctx)
window_event:
  CloseRequested ─────► event_loop.exit()
  Resized ────────────► renderer.resize (ignore 0×0 / minimized)
  keyboard/mouse ─────► input.record(...)
  RedrawRequested ────► FRAME:
      time.tick()                     measure real frame delta (clamped)
      n = fixed.advance(frame_dt)     accumulator, capped at MAX_STEPS
      repeat n: game.fixed_update(ctx)   the game calls systems explicitly,
                                         e.g. ecs::integrate_velocity(ctx.world, FIXED_DT)
      game.update(ctx)                variable-rate logic
      renderer.render(world, alpha)   extract → draw → queue.present
      input.end_frame()               clear just_pressed/just_released
about_to_wait ────────► window.request_redraw()
suspended ────────────► (Stage 4) drop surface; recreate on resumed
exiting ──────────────► drop renderer before window (clean shutdown)
```

### Fixed timestep ([ADR-0005](adr/0005-game-loop.md))

```text
FIXED_DT         = 1.0 / 60.0  (f64 seconds)
MAX_FRAME_DT     = 0.25 s      clamp after a stall (debugger, window drag)
MAX_FIXED_STEPS  = 5 per frame
accumulator     += min(frame_dt, MAX_FRAME_DT)
while accumulator >= FIXED_DT && steps < MAX_FIXED_STEPS { step; accumulator -= FIXED_DT }
if steps == MAX_FIXED_STEPS { accumulator = accumulator.min(FIXED_DT) }  // drop backlog
alpha = accumulator / FIXED_DT   // for optional render interpolation later
```

Time uses `f64` for accumulated/elapsed values (no precision drift over long
sessions); `f32` is used only where values enter `glam`/GPU math.

---

## 8. Rendering architecture (Stage 4+)

Initialization order, as verified against wgpu 30.0.1:

```text
Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(event_loop.owned_display_handle())))
  → instance.create_surface(Arc<Window>)                 -> Result<_, CreateSurfaceError>
  → instance.request_adapter(&RequestAdapterOptions{ compatible_surface, ..Default })  -> Result<_, RequestAdapterError>
  → adapter.request_device(&DeviceDescriptor{..})         -> Result<(Device, Queue), RequestDeviceError>
  → surface.get_default_config(&adapter, w, h)            -> Option<SurfaceConfiguration>
  → surface.configure(&device, &config)
frame:
  surface.get_current_texture() -> CurrentSurfaceTexture::{Success, Suboptimal, Timeout,
                                   Occluded, Outdated, Lost, Validation}
  → encoder.begin_render_pass(clear purple) → queue.submit → window.pre_present_notify()
  → queue.present(frame)
```

Handling policy:

| Condition | Action |
|---|---|
| `Success` | draw and present |
| `Suboptimal` | draw and present, then reconfigure |
| `Timeout`, `Occluded` | skip the frame |
| `Outdated` | reconfigure, skip the frame |
| `Lost` | recreate the surface from the instance, reconfigure, skip |
| `Validation` | log and skip; repeated failures become `Error::Render` |
| Out-of-memory / device lost | wgpu 30 reports these through `Device::on_uncaptured_error` and `set_device_lost_callback`, not the acquire result. The renderer records them and the loop exits with `Error::Render` |
| width or height = 0 | never call `configure`; skip rendering while minimized |

Future internal structure, built only when a stage needs it:
`Renderer { gpu: GpuContext, sprites: SpriteRenderer, shapes: ShapeRenderer,
text: TextRenderer, debug: DebugRenderer }`.

Color: public `Color` values are sRGB. The renderer converts to linear when the
surface format is `*Srgb` (proposed in [ADR-0008](adr/0008-color-space.md);
the Stage 0 spike showed linear `(0.35, 0.10, 0.55)` displays as sRGB `#A059C4`).

---

## 9. Error handling ([ADR-0006](adr/0006-errors-and-logging.md))

* One public `purplepie::Error` enum (`thiserror`), `purplepie::Result<T>`.
* Variants by failure domain: `EventLoop`, `Window`, `Surface`, `Adapter`,
  `Device`, `SurfaceUnsupported`, `Render`, `Asset`, `Game(Box<dyn Error + Send + Sync>)`.
* Errors raised inside winit callbacks, which cannot return `Result`, are
  stored in the runner. The loop is asked to exit, and `Engine::run` returns
  the error after `run_app` completes.
* No `unwrap()` in engine code (`clippy::unwrap_used = "warn"`). `expect` is
  allowed only for true invariants, with a message explaining why.
* Diagnostics use the `log` facade, which wgpu and winit already depend on. The
  engine does not pick a logger; the sandbox may install one.

---

## 10. Testing strategy

| Layer | How |
|---|---|
| `time`, `math`, `ecs` systems, `input` state machine | pure unit tests, no window |
| `Game` logic | construct `World`/`Time`/`Input` headless and call game methods |
| `app` + `render` | `cargo build`, plus a smoke run under Xvfb + Mesa lavapipe (proven in the Stage 0 spike) |
| Every stage | `cargo fmt --check`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build` |
