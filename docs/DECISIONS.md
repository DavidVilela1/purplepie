# PurplePie Architecture Decision Log

This file is the **only** place ADRs live. It replaced the per-file `docs/adr/`
directory on 2026-09-30, with no decision content changed.

* **Status values:** `Proposed`, `Accepted`, `Superseded by ADR-xxx`, `Deprecated`.
* **Implementation state** is listed separately, because an accepted decision
  may not be built yet. See [PROJECT_STATUS.md](PROJECT_STATUS.md) for what exists.
* **When to write an ADR, and when not to:** see [DEVELOPMENT.md §6](DEVELOPMENT.md#6-architecture-decisions).
* Undecided questions are listed under [Pending Decisions](#pending-decisions) at the end. They are **not** ADRs.

## Index

| ADR | Title | Status | Implemented |
|---|---|---|---|
| ADR-001 | Rust as the primary implementation language | Accepted | Yes (Stage 0) |
| ADR-002 | Single package: engine library + `sandbox` binary | Accepted | Yes (Stage 0) |
| ADR-003 | Module boundaries and dependency direction | Accepted | Partially: rules only, no modules yet |
| ADR-004 | `winit` 0.30.13 for windowing and events | Accepted | Yes (Stage 1, `src/app/` only) |
| ADR-005 | `wgpu` 30.0.1 as the GPU abstraction | Accepted | No (Stage 4) |
| ADR-006 | `hecs` as the ECS | Accepted | No (Stage 3) |
| ADR-007 | `glam` for math | Accepted | No (Stage 3) |
| ADR-008 | Engine/game API: `Game` trait + per-call `Context` | Accepted | Partially: Stages 1–2 subset |
| ADR-009 | Renderer is engine-owned, crate-private, and reads the world | Accepted | No (Stage 4) |
| ADR-010 | Fixed-timestep game loop driven by `RedrawRequested` | Accepted | Yes: Stage 1 (frame hook, pacing) + Stage 2 (timestep); vsync pacing in Stage 4 |
| ADR-011 | Error handling: one `thiserror` enum, `log` facade, no `unwrap` | Accepted | Partially: `Error` + lints (Stage 1); logging deferred |
| ADR-012 | Async: `pollster::block_on`, no async runtime | Accepted | No (Stage 4) |
| ADR-013 | Dependency admission: add per stage, pin, commit the lockfile | Accepted | Yes (Stage 0) |

---

# ADR-001: Rust as the primary implementation language

## Status
Accepted (2026-09-30). Implemented.

## Context
PurplePie targets desktop 2D games and needs predictable performance, memory
safety without a garbage collector, and a mature ecosystem for windowing and
GPU access.

## Decision
The engine and its games are written in Rust (stable channel, edition 2024).
`unsafe` code is forbidden crate-wide (`[lints.rust] unsafe_code = "forbid"`).

## Alternatives Considered
- **C++** with SDL/bgfx. Mature, but lacks memory safety and has weaker package management.
- **Zig / Odin.** Smaller ecosystems, and no stable `wgpu`-equivalent bindings.
- **C# / managed runtimes.** GC pauses, and a heavier deployment story.

## Rationale
The Rust ecosystem provides `winit`, `wgpu`, `glam` and several ECS crates of
production quality. Ownership rules make the engine/game boundary explicit.

## Consequences
### Positive
- Memory and thread safety by default. Cargo handles dependencies and builds.
- `unsafe_code = "forbid"` is achievable: wgpu 30 creates surfaces safely from `Arc<Window>`.
### Negative
- Borrow-checker constraints shape the API (see ADR-008).
- Windows builds need the MSVC linker (Visual Studio "Desktop development with C++").

## Revisit Conditions
Never for the engine core. Scripting languages may be added on top later.

---

# ADR-002: Single package with an engine library and a `sandbox` binary

## Status
Accepted (2026-09-30). Implemented: `src/lib.rs` + `src/main.rs`, `[[bin]] name = "sandbox"`.

## Context
Engine code and game code must stay separate, and the separation should be
enforced rather than conventional.

## Decision
One Cargo package `purplepie` with two targets:
- `src/lib.rs`: library `purplepie` (the engine).
- `src/main.rs`: binary `sandbox` (a game that may use only the public API).

The binary is not named `purplepie` because a lib and a bin with the same name
collide in `cargo doc` output.

## Alternatives Considered
- **One binary crate with modules.** Nothing prevents game code from reaching engine internals.
- **Cargo workspace with several crates** (`purplepie_render`, …). Premature: coordination overhead with no current benefit.

## Rationale
The compiler enforces the boundary: `main.rs` can only see `pub` items. `pub(crate)` hides wgpu and winit details.

## Consequences
### Positive
- One manifest, one lockfile, trivial builds.
- The module dependency rules (ADR-003) already match a future crate split.
### Negative
- Everything recompiles together. Compile times will grow once wgpu arrives.

## Revisit Conditions
Measured incremental build times become painful (Stage 10 review), or a
second consumer needs only part of the engine.

---

# ADR-003: Module boundaries and dependency direction

## Status
Accepted (2026-09-30). Implementation partial: the rules are fixed, and each module is created in its stage.

## Context
The initial brief sketched `app, core, ecs, input, math, render, time`.
Modules created before they have a job become empty or speculative.

## Decision
- Modules and their stages: `error` (1), `app` (1), `time` (2), `math` (3),
  `ecs` (3), `render` (4–7), `input` (8), `assets` (9).
- **No `core` module.** Its would-be contents go to `error` (the error type) and `app::EngineConfig` (settings).
- A module is created only by the stage that gives it a real responsibility.
- Dependencies point downward: `app` → `render`/`input`/`time`/`assets` → `ecs`/`math`.
  `error` has no internal dependencies. See [ARCHITECTURE.md §4](ARCHITECTURE.md#4-dependency-direction).
- Only `app` imports `winit`. Only `render` imports `wgpu`.

## Alternatives Considered
- **Create all modules as empty stubs now.** Looks organized, but the stubs have no users and drift from reality.
- **Keep `core`.** Unclear responsibility, and `core` is also the name of Rust's built-in crate, which makes `use core::…` inside the crate ambiguous.

## Rationale
Every module has one responsibility and a known first user. Confining winit
and wgpu to one module each limits the blast radius of their frequent breaking releases.

## Consequences
### Positive
- Upgrading winit or wgpu touches one module.
- The structure is always an honest picture of what exists.
### Negative
- The directory tree looks sparse early on. That is intended.

## Revisit Conditions
A module needs to depend upward, which indicates a missing abstraction or a misplaced type.

---

# ADR-004: `winit` 0.30.13 for windowing and events

## Status
Accepted (2026-09-30). Implemented in PP-003 (Stage 1): `src/app/runner.rs` is the only
code that handles winit events, and winit types appear nowhere in the public API.

## Context
PurplePie needs cross-platform windows, an event loop and input events. On
2026-09-30 the newest stable winit was 0.30.13, and 0.31.0-beta.3 was a
pre-release with a redesigned API.

## Decision
Use `winit = "0.30.13"` through the `ApplicationHandler` API
(`resumed`, `window_event`, `about_to_wait`, `exiting`). Create windows only in
`resumed`. Keep winit inside `app`.

## Alternatives Considered
- **winit 0.31 beta.** Pre-release. It would pin the foundation to a moving API.
- **SDL2/SDL3 bindings.** A C dependency, and duplicates what winit + wgpu provide.
- **GLFW bindings.** A C dependency, with weaker wgpu integration.

## Rationale
winit is the de facto Rust windowing crate and shares `raw-window-handle 0.6.2`
with wgpu 30. The Stage 0 spike compiled and ran with it
([spikes/stage-0-compat-spike.md](spikes/stage-0-compat-spike.md)).

## Consequences
### Positive
- Stable, documented API. Game code never sees it (ADR-008).
### Negative
- A 0.31 migration will be required eventually.

## Revisit Conditions
winit 0.31.0 stable is released and wgpu supports it, or a platform bug in 0.30 blocks progress.

---

# ADR-005: `wgpu` 30.0.1 as the GPU abstraction

## Status
Accepted (2026-09-30). Not implemented (Stage 4).

## Context
The renderer needs a modern, portable GPU API (Vulkan, Metal, DX12, GL) without
per-backend code.

## Decision
Use `wgpu = "30.0.1"`, pinned via `Cargo.lock`. PurplePie uses it only as a 2D
backend: orthographic projection, no depth buffer by default, no 3D concepts in
the public API.

## Alternatives Considered
- **Raw Vulkan/Metal/DX12.** Far more code, and needs `unsafe`.
- **OpenGL (glow).** Legacy API, weaker tooling, and poor macOS support.
- **A higher-level 2D crate** (e.g. macroquad). It would replace the engine instead of supporting it.

## Rationale
wgpu is safe, portable and actively maintained. The spike verified the wgpu 30 API shapes,
which differ from most tutorials (see [TECH_STACK.md](TECH_STACK.md)).

## Consequences
### Positive
- One code path for all desktop backends. Safe surface creation.
### Negative
- A breaking release roughly every quarter (26 → 30 in about 14 months).
- Most online examples are outdated for wgpu 30.

## Revisit Conditions
Each new major wgpu release: upgrade deliberately, not automatically, and note the change here.

---

# ADR-006: `hecs` as the ECS

## Status
Accepted (2026-09-30). Not implemented (Stage 3).

## Context
Exactly one ECS library is needed. Candidates were measured on 2026-09-30:

| Criterion | hecs 0.11.1 | bevy_ecs 0.19.1 |
|---|---|---|
| Unique normal dependencies (incl. itself) | 4 | 76 |
| MSRV | 1.81 | 1.95 |
| Breaking releases | 0.10 → 0.11 took about 2.5 years | 0.17 → 0.18 → 0.19 → 0.20-rc within about 11 months |
| Scope | storage + queries | storage, queries, scheduler, resources, events, observers, change detection |
| Independence | standalone | part of the Bevy release train |

## Decision
Use `hecs 0.11.1`. Re-export `hecs::World` and `hecs::Entity` from
`purplepie::ecs`. Systems are plain functions (`fn(&mut World, …)`).

## Alternatives Considered
- **bevy_ecs.** Powerful, but it brings its own app/scheduler model, a large
  dependency tree and frequent breaking changes.
- **A custom ECS.** A distraction from engine goals.

## Rationale
Simple ergonomics, low churn, and no scheduler competing with PurplePie's own
loop. Archetypal storage is more than enough for 2D entity counts.

## Consequences
### Positive
- Small, stable API. System order is explicit in game code.
### Negative
- No built-in resources, events, change detection or scheduler. Resources such
  as `Time` and `Input` live in `Context` (ADR-008).
- Game code uses hecs types directly, so switching ECS later is costly.

## Revisit Conditions
Parallel system execution or change detection becomes a measured need.

---

# ADR-007: `glam` for math

## Status
Accepted (2026-09-30). Not implemented (Stage 3).

## Context
The engine needs 2D vectors, affine transforms and 4×4 matrices for GPU upload.

## Decision
Use `glam = "0.33"`. `Vec2`, `Affine2` and `Mat4` are re-exported from
`purplepie::math`. `Transform2D { position: Vec2, rotation: f32, scale: Vec2 }`
is PurplePie's own type, converted to GPU matrices in `render`.

## Alternatives Considered
- **nalgebra.** More general, with a heavier API and slower compile times.
- **cgmath.** Largely unmaintained.
- **Hand-written math.** A needless source of bugs.

## Rationale
glam is fast (SIMD), simple, widely used with wgpu, and has no required dependencies.

## Consequences
### Positive
- Minimal footprint. Types match GPU layouts easily.
### Negative
- Public API exposes glam types. A glam breaking release affects games.

## Revisit Conditions
None expected.

---

# ADR-008: Engine/game API: `Game` trait + per-call `Context`

## Status
Accepted (2026-09-30). Partially implemented in PP-003 (Stage 1). Refined in Stage 10.

**Implementation note (Stage 1):** `Game` currently has `init` (default `Ok(())`)
and a required `update`. `Context` carries only the exit flag (`request_exit`,
`exit_requested`). `fixed_update`, `time`, `world` and `input` arrive with their
stages. `Engine::run` returns `Result<()>` as decided. `Error::game(e)` and
`BoxError` were added so games can return their own errors from `init`.

**Implementation note (Stage 2, PP-004):** `fixed_update` was added. All three
callbacks now have empty defaults (`update` was required in Stage 1, and
`fixed_update` was sketched as required above), so a game implements only what it uses.
`Context` exposes accessor methods (`time()`, `dt()`) rather than the public
fields sketched above, which keeps its internals free to change. `dt()` returns `f32`
(the step for the current callback: `fixed_dt` in `fixed_update`, the clamped
frame delta in `update`, and 0 in `init`) for direct use in glam math. Precise
`f64` values are available from `Time`. These are API-shape refinements within
this ADR, not new decisions.

## Context
Game code needs mutable ECS access plus read access to time and input, and
must not see wgpu or winit. winit 0.30 drives the application through
`ApplicationHandler` callbacks on `&mut self`, so one engine struct must own all state.

## Decision
```rust
pub trait Game {
    fn init(&mut self, ctx: &mut Context<'_>) -> Result<()> { Ok(()) }
    fn fixed_update(&mut self, ctx: &mut Context<'_>);
    fn update(&mut self, ctx: &mut Context<'_>) {}
}
pub struct Context<'a> { pub world: &'a mut World, pub time: &'a Time, pub input: &'a Input, /* exit flag */ }
impl Engine {
    pub fn new(config: EngineConfig) -> Result<Engine>;
    pub fn run<G: Game>(self, game: G) -> Result<()>;
}
```
`Context` fields appear only when their stage exists (`world` in Stage 3,
`input` in Stage 8). The engine runs **no gameplay systems implicitly**. Games
call systems such as `ecs::integrate_velocity` themselves.

## Alternatives Considered
- **Closures** (`engine.run(|ctx| …)`). Awkward with several callbacks and shared game state.
- **The game owns the World.** The renderer would depend on game structure, and borrow complexity leaks into games.
- **A Bevy-style plugin/scheduler app.** Framework-scale machinery (a "mini-Bevy").

## Rationale
Static dispatch (`G: Game`). `Context` is built from disjoint field borrows,
so there is no `Rc`, `RefCell` or `Mutex`. Game logic can be unit-tested
headless by building a `Context` from plain data.

## Consequences
### Positive
- A small, explicit API. Game code never sees GPU or window types.
### Negative
- Games cannot issue custom GPU work. A restricted draw hook would need a new ADR.

## Revisit Conditions
Stage 10 review with a real example game, or a borrow conflict that disjoint borrows cannot solve.

---

# ADR-009: Renderer is engine-owned, crate-private, and reads the world

## Status
Accepted (2026-09-30). Not implemented (Stage 4).

## Context
The renderer holds `Surface`, `Device`, `Queue` and pipelines. Its lifetime is
tied to the window (created in `resumed`, dropped on `suspended`/`exiting`).

## Decision
- `Renderer` is `pub(crate)`, owned by the app runner as `Option<Renderer>`.
- Each frame the renderer **reads** the `World` (entities with `Transform2D` +
  `Sprite`, and later other drawables) after the update phase. It never mutates game state.
- Public render types (`Color`, `Sprite`, `Camera2D`) are plain data with no wgpu handles.
- The one shared-ownership type is `Arc<Window>`, required for `wgpu::Surface<'static>`.
- Drop order on shutdown: renderer (surface) before window.

## Alternatives Considered
- **Game-owned renderer or immediate-mode draw calls from the game.** Exposes GPU lifetimes to games.
- **Renderer inside the ECS as a resource.** hecs has no resources, and it would create an ECS ↔ render cycle.

## Rationale
Keeps wgpu churn inside one module and keeps the ECS independent of rendering.
Matches winit's create-on-resume lifecycle.

## Consequences
### Positive
- ECS and game code are testable without a GPU.
### Negative
- Everything drawable must be representable as component data.

## Revisit Conditions
Debug drawing or UI needs immediate-mode calls. If so, add a restricted
command buffer, not raw wgpu access.

---

# ADR-010: Fixed-timestep game loop driven by `RedrawRequested`

## Status
Accepted (2026-09-30). Partially implemented in PP-003 (Stage 1).

**Implementation note (Stage 1):** the frame runs in `RedrawRequested` as decided.
Stage 1 pacing is a pure `FramePacer` (60 Hz `WaitUntil` deadlines; after a
stall it restarts the schedule instead of bursting), measured at about 0.04 s CPU
per 3 s idle under Xvfb. The fixed timestep itself is Stage 2 (PP-004).

**Implementation note (Stage 2, PP-004):** implemented as decided, with
`FixedTimestep` (`std` only) and `Time`. The constants are configurable through
`EngineConfig` and validated (`fixed_dt > 0`, `max_frame_dt ≥ fixed_dt`,
`max_fixed_steps ≥ 1`). When the cap is hit, the backlog is reduced with `%= dt`,
which keeps the sub-step phase. The first frame uses a zero delta, and
non-finite or negative deltas are ignored. Verified by 12 unit tests and Xvfb
probes (59.8 Hz measured; a 1 s stall gives 5 steps).

## Context
Gameplay must be frame-rate independent. Rendering should run at display rate.
winit 0.30 recommends drawing in `RedrawRequested` and requesting redraws from `about_to_wait`.

## Decision
- The whole frame runs in `RedrawRequested`, in this order: time tick → 0..N
  `fixed_update` → `update` → render → end the input frame.
- `FIXED_DT = 1/60 s` (f64), `MAX_FRAME_DT = 0.25 s` clamp,
  `MAX_FIXED_STEPS = 5`. When the cap is hit, the backlog is clamped to below one step.
- `alpha = accumulator / FIXED_DT` is exposed. Interpolation is not implemented until needed.
- `FixedTimestep` is a pure struct with no winit/wgpu dependency.
- Stages 1–3 use `ControlFlow::WaitUntil` (no swapchain yet, so no busy loop).
  From Stage 4, `Poll` + `Fifo` present mode provides frame pacing.

## Alternatives Considered
- **Variable timestep only.** Non-deterministic and frame-rate dependent.
- **Simulation on a separate thread.** Synchronization complexity with no current need.
- **Simulation in `about_to_wait`, render in `RedrawRequested`.** Splits one
  logical frame across callbacks, which makes ordering harder to reason about.

## Rationale
The standard, well-understood pattern. It prevents the spiral of death and is unit-testable.

## Consequences
### Positive
- Decoupled simulation and rendering rates. No update burst after a debugger pause.
### Negative
- Edge-triggered input needs care with 0 or N fixed steps per frame (Pending Decision PD-03).

## Revisit Conditions
Visible stutter that interpolation cannot fix, or a need for simulation rates other than 60 Hz.

---

# ADR-011: Error handling: one `thiserror` enum, `log` facade, no `unwrap`

## Status
Accepted (2026-09-30). Partially implemented: lints (Stage 0); `Error`/`BoxError`/`Result` and callback-error propagation (Stage 1, PP-003). Logging is deferred (PD-04).

## Context
Window, GPU and asset initialization can fail. winit callbacks cannot return errors.

## Decision
- One public `purplepie::Error` (`thiserror`) and `purplepie::Result<T>`.
  Variants follow failure domains: event loop, window, surface, adapter, device,
  unsupported surface, render, asset, and game (boxed).
- Third-party errors are wrapped (`#[from]`/`#[source]`), not re-exported.
- Errors inside callbacks are stored by the runner, which then calls
  `event_loop.exit()`. `Engine::run` returns the error.
- `clippy::unwrap_used = "warn"` (active). `expect` is allowed only for
  invariants, with a message explaining why.
- Diagnostics use the `log` facade, which wgpu already depends on. The engine never installs a logger.

**Correction (2026-09-30, PP-003):** this ADR originally said winit also depends
on `log`. In fact winit 0.30 logs through `tracing`. `log` is in the graph only via
`calloop` on Linux, and via wgpu from Stage 4. The decision is unchanged. The
choice between `log` and `tracing` for the engine's own diagnostics is part of
PD-04, which was deferred to PP-006 because Stage 1 has nothing to log.

## Alternatives Considered
- **`anyhow` in the engine.** Loses typed matching for games.
- **Panics on init failure.** Poor user experience and untestable.
- **`tracing`.** More than current needs.

## Rationale
Typed, matchable errors with minimal dependencies.

## Consequences
### Positive
- One error type for games to handle.
### Negative
- Variants must be maintained as stages add failure modes.

## Revisit Conditions
Structured diagnostics or profiling spans become a measured need.

---

# ADR-012: Async: `pollster::block_on`, no async runtime

## Status
Accepted (2026-09-30). Not implemented (Stage 4).

## Context
`Instance::request_adapter` and `Adapter::request_device` return futures. On
native platforms they resolve promptly and do no I/O that benefits from a scheduler.

## Decision
Use `pollster = "1.0.1"` to block on these futures during renderer creation. No
Tokio, async-std or smol.

## Alternatives Considered
- **Tokio.** A large runtime for two one-off futures.
- **Hand-written executor.** Reinventing pollster.

## Rationale
A tiny, dependency-free crate that keeps initialization synchronous and readable.

## Consequences
### Positive
- No runtime, no `async` in the engine API.
### Negative
- The web target, which is a non-goal, would need a different init path.

## Revisit Conditions
Asynchronous asset streaming or a web target becomes a goal.

---

# ADR-013: Dependency admission: add per stage, pin, commit the lockfile

## Status
Accepted (2026-09-30). Implemented: Stage 0 `Cargo.toml` has no dependencies.

## Context
Unused dependencies slow builds and create maintenance burden. wgpu and winit
break often.

## Decision
- A dependency is added only in the stage that uses it (planned list in `Cargo.toml` comments).
- Before adding one, check its current version and MSRV and confirm the API
  against the crate source or docs. Record it in [TECH_STACK.md](TECH_STACK.md).
- `Cargo.lock` is committed. Upgrades are deliberate changes, noted in PROJECT_STATUS.
- `rust-version` reflects the highest MSRV in the resolved graph (currently 1.90, only 1.95 exercised).

## Alternatives Considered
- **Declare all planned dependencies up front.** Bloats Stage 0–2 builds with unused crates.
- **Loose version ranges without a lockfile.** Non-reproducible builds.

## Rationale
Every dependency has a current user. Builds are reproducible.

## Consequences
### Positive
- Fast early builds and a clear audit trail.
### Negative
- Each stage must re-verify versions (a small, recurring cost).

## Revisit Conditions
None expected.

---

# Pending Decisions

These questions have a proposed direction but have **not** been decided. Each
one is resolved (and becomes an ADR) inside the task listed.

| ID | Question | Proposed direction | Decide in |
|---|---|---|---|
| PD-01 | Color space of public `Color` | Public colors are sRGB. The renderer converts to linear once. The spike showed linear `(0.35, 0.10, 0.55)` displays as `#A059C4` on an sRGB surface. | PP-006 / PP-007 (Stage 4–5) |
| PD-02 | World coordinate system | +X right, +Y up, world units, `Camera2D { position, zoom, pixels_per_unit }` centered, radians counter-clockwise, `z`/layer ordering without a depth buffer | PP-009 (Stage 7) |
| PD-03 | Input model | Own `KeyCode`/`MouseButton` enums mapped from winit. Edges latched until the first fixed step of the frame consumes them. | PP-010 (Stage 8) |
| PD-04 | Engine diagnostics (`log` vs `tracing`) and a logger in the sandbox | Use the `log` facade (wgpu uses it). Possibly `env_logger` in the sandbox only. Deferred from PP-003 because Stage 1 emits no diagnostics. | PP-006 (Stage 4) |
| PD-05 | Sprite batching strategy | Instanced quads per texture, sorted by layer | PP-008 (Stage 6) |
| PD-06 | Asset handle design | Typed `Handle<T>` + `Assets` store, synchronous loading | PP-011 (Stage 9) |
| PD-07 | Project license | MIT OR Apache-2.0 is the ecosystem norm | Owner decision (PP-013) |
