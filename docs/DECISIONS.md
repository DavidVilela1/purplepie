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
| ADR-002 | Single package: engine library + `sandbox` binary | Accepted (reaffirmed by ADR-026) | Yes (Stage 0); `examples/` added in Stage 10 |
| ADR-003 | Module boundaries and dependency direction | Accepted | Yes for `error`, `app`, `time`, `math`, `ecs`, `render`, `input` (keyboard, Stage 8); `assets` in Stage 9; `audio` in PP-022; `ui` in PP-023 |
| ADR-004 | `winit` 0.30.13 for windowing and events | Accepted | Yes (Stage 1, `src/app/` only) |
| ADR-005 | `wgpu` 30.0.1 as the GPU abstraction | Accepted | Yes (Stage 4, `src/render/` only) |
| ADR-006 | `hecs` as the ECS | Accepted | Yes (Stage 3, `src/ecs/`) |
| ADR-007 | `glam` for math | Accepted | Yes (Stage 3, `src/math/`) |
| ADR-008 | Engine/game API: `Game` trait + per-call `Context` | Accepted (reviewed by ADR-026) | Yes: `init`/`fixed_update`/`update`; `Context` = world, time, dt, input, cursor, camera, viewport, textures, fonts, text measuring, asset root, window title, exit |
| ADR-009 | Renderer is engine-owned, crate-private, and reads the world | Accepted | Yes: ownership/lifecycle (Stage 4); reads `&World` for quads (Stage 5) and sprites (Stage 6) |
| ADR-010 | Fixed-timestep game loop driven by `RedrawRequested` | Accepted (pacing clause superseded by ADR-014) | Yes: Stages 1–2 |
| ADR-011 | Error handling: one `thiserror` enum, `log` facade, no `unwrap` | Accepted | Yes: `Error` (Stages 1–4), lints; logging via ADR-016 |
| ADR-012 | Async: `pollster::block_on`, no async runtime | Accepted | Yes (Stage 4, renderer init) |
| ADR-013 | Dependency admission: add per stage, pin, commit the lockfile | Accepted | Yes (Stage 0) |
| ADR-014 | Frame pacing: vsync (`AutoVsync`) plus a 60 Hz redraw cap | Accepted | Yes (Stage 4) |
| ADR-015 | Public colors are sRGB; the renderer converts per target format | Accepted | Yes (Stage 4) |
| ADR-016 | Diagnostics: `log` facade in the engine; games choose the logger | Accepted | Yes (Stage 4, PP-014) |
| ADR-017 | GPU faults are fatal and reported as `Error::Render` | Accepted | Yes (Stage 4, PP-014) |
| ADR-018 | World coordinates: +X right, +Y up, origin at the window centre, 1 unit = 1 logical pixel | Accepted (now the default camera, ADR-022) | Yes (Stage 5, PP-007) |
| ADR-019 | Quads: one instanced pipeline, CPU-built clip matrices, embedded WGSL | Accepted ("one draw call for all quads" is now one per layer, ADR-021) | Yes (Stage 5, PP-007); instance code shared with sprites since PP-008 |
| ADR-020 | Textures: `TextureId` handles, decode on load, upload in the renderer; `image` (PNG only) | Accepted (the "sprites drawn after quads" clause is superseded by ADR-021; extended by PP-019: texture regions) | Yes (Stage 6, PP-008; regions + `SpriteGrid`: PP-019) |
| ADR-021 | Draw order and batching: optional `Layer` component, one sorted draw list, one draw call per (layer, material) run | Accepted | Yes (Stage 6, PP-015) |
| ADR-022 | One engine-owned `Camera2D { position, zoom }` in `Context`; screen ↔ world in logical pixels | Accepted | Yes (Stage 7, PP-009) |
| ADR-023 | License: MIT OR Apache-2.0 | Accepted | Yes (PP-013) |
| ADR-024 | Keyboard input: own `KeyCode` (physical keys), `Input` with per-callback edges latched for fixed steps | Accepted (extended to the mouse by PP-016) | Yes (Stage 8, PP-010 keyboard, PP-016 mouse) |
| ADR-025 | Asset root: relative asset paths resolve against one folder chosen at startup (config, else next to the executable, else the working directory) | Accepted | Yes (Stage 9, PP-011) |
| ADR-026 | API review after Breakout: keep `Game` + `Context` and a single crate; add `Hidden`, `Camera2D::fit`, `Context::set_window_title`; `missing_docs` enforced | Accepted | Yes (Stage 10, PP-017) |
| ADR-027 | Text: `ab_glyph` rasterizes outline fonts into one glyph atlas drawn by the sprite pipeline; `Text` component (with `TextAnchor`); `FontId` handles; `Context::measure_text`; Poppins shipped (OFL) | Accepted (extended by PP-018b) | Yes (PP-018a: fonts, atlas, `Text`; PP-018b: anchors, measuring, Breakout HUD) |
| ADR-028 | Sprite animation: `SpriteAnimation` component (grid + frame range + fps + loop/once) advanced by the game-called `render::advance_animations`; the draw list shows its current frame | Accepted | Yes (PP-020) |
| ADR-029 | Screen space: a `ScreenSpace { anchor }` component draws quads, sprites and text in logical window pixels from a window anchor, ignoring the camera, after all world content | Accepted | Yes (PP-021) |
| ADR-030 | Audio: `cpal` output + `hound` WAV decoding + PurplePie's own mixer; `SoundId`; `Context::load_sound` / `play_sound`; silent fallback without a device; generic handles (PD-06) deferred again | Accepted | Yes (PP-022: one-shot sound effects) |
| ADR-031 | UI interaction: `ui` module with a data-only `Button` component in screen space, a `Pointer` snapshot and the game-called `update_buttons` system; topmost button wins | Accepted | Yes (PP-023) |
| ADR-032 | Audio playback control: `PlaybackId` per playback, `loop_sound`, `stop_sound`, `set_sound_volume`, `stop_all_sounds`, master volume, all as commands to the mixer; loops survive the voice limit | Accepted | Yes (PP-024a) |
| ADR-033 | OGG Vorbis: `lewton` decodes `.ogg` completely on load; `load_sound` picks the decoder from the file's first bytes; no streaming yet | Accepted | Yes (PP-024b) |

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
Accepted (2026-09-30). Implemented in PP-006 (Stage 4): `src/render/renderer.rs` is the only code that uses wgpu. It receives the window through wgpu's `WindowHandle` trait and the display through `wgt::WgpuHasDisplayHandle`, so `render` never imports winit.

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
Accepted (2026-09-30). Implemented in PP-005 (Stage 3): `purplepie::ecs` re-exports `World`, `Entity` and the `hecs` crate. The first component is `Velocity`, and the first system is `integrate_velocity`.

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
Accepted (2026-09-30). Implemented in PP-005 (Stage 3): `purplepie::math::{Transform2D, Vec2}`. Per ADR-013 minimal-footprint rules, glam is built with `default-features = false, features = ["std"]`, which gives the f32 types only (`Vec2`, `Affine2`, `Mat4`).

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

**Implementation note (Stage 3, PP-005):** the world is reached through
`ctx.world()` / `ctx.world_mut()`, consistent with the Stage 2 accessor style.
Known ergonomic cost, verified with the compiler: `integrate_velocity(ctx.world_mut(), ctx.dt())`
fails with E0502, so games write `let dt = ctx.dt();` first. This is documented in the API docs
and a doctest. Revisit in Stage 10 together with input access.

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
Accepted (2026-09-30). Partially implemented in PP-006 (Stage 4): `Renderer` is `pub(crate)`, owned by the runner as `Option<Renderer>`, created in `resumed`, and dropped in `suspended` and in `exiting` (before the window). Public `render::Color` is plain data. The renderer does not read the `World` yet, because Stage 4 only clears the frame. Reading the world starts with the first primitive (Stage 5).

**Implementation note (Stage 5, PP-007):** `Renderer::render(&World, …)` takes the world by shared reference. It queries `(Transform2D, Quad)` and never writes. The public `render::Quad` is plain data (size + `Color`).

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
  ~~From Stage 4, `Poll` + `Fifo` present mode provides frame pacing.~~
  **Superseded by ADR-014 (2026-09-30):** measurements showed vsync alone does
  not reliably pace the loop, so the `WaitUntil` cap stays.

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

**Implemented (PP-014):** logging per ADR-016. GPU runtime failures surface as `Error::Render` per ADR-017.

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
Accepted (2026-09-30). Implemented in PP-006 (Stage 4): `pollster::block_on` wraps `request_adapter` and `request_device` in `Renderer::new`, called from `resumed`.

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

# ADR-014: Frame pacing: vsync (`AutoVsync`) plus a 60 Hz redraw cap

## Status
Accepted (2026-09-30, PP-006). Supersedes the pacing clause of ADR-010.

## Context
ADR-010 planned to drop the Stage 1 `FramePacer` (`ControlFlow::WaitUntil`,
60 Hz) once a swapchain existed and to rely on `Poll` + `Fifo` for pacing.
Before implementing that, the Stage 4 spike was measured under Xvfb + Mesa lavapipe:
- With `Fifo` and `Poll`, the loop ran at **544 fps**, using about one full CPU
  core. `Fifo` does not block on this presentation path.
- The adapter's **first** reported present mode was `Immediate`, and
  `Surface::get_default_config` picks the first mode. So the default
  configuration would have had no vsync at all.
- `CurrentSurfaceTexture::Occluded` and `Timeout` return immediately, so an
  occluded or minimized window would spin under `Poll` even on real hardware.

## Decision
- The surface uses `PresentMode::AutoVsync` explicitly (Fifo-family, always supported).
- The `FramePacer` stays. `about_to_wait` requests a redraw at most every
  1/60 s and sleeps with `ControlFlow::WaitUntil` in between.
- Minimized (0×0) frames skip rendering but keep the same pacing.

## Alternatives Considered
- **`Poll` + `Fifo` only (the original plan):** busy-loops where Fifo doesn't block
  (measured) and while occluded.
- **`Poll` + `Fifo`, falling back to `WaitUntil` only after a skipped frame:** more
  states to reason about, and it still busy-loops where Fifo doesn't block.
- **Uncapped `Immediate`/`Mailbox`:** tearing or wasted power, with no benefit for a 60 Hz simulation.

## Rationale
It is correct everywhere that was measured, and it is simple: one pacing
mechanism, with vsync only preventing tearing. The fixed simulation already
runs at 60 Hz (ADR-010), so rendering faster adds no new game state.

## Consequences
### Positive
- No busy loop in any measured state. 300 frames took 5.1 s under Xvfb.
- Behavior is identical with and without a vsync-capable presentation path.
### Negative
- Rendering is capped at 60 fps even on 120/144 Hz displays. Smoother
  high-refresh output would need both a higher cap and render interpolation
  (`Time::alpha`).
- On a 60 Hz display, the timer and vsync are two clocks. Occasional
  frame-time jitter is possible (not measurable here; owner to observe on real hardware).

## Revisit Conditions
High-refresh support is wanted (then add a configurable cap plus interpolation),
or the owner sees stutter on real hardware.

---

# ADR-015: Public colors are sRGB; the renderer converts per target format

## Status
Accepted (2026-09-30, PP-006). Resolves pending decision PD-01.

## Context
Surfaces commonly use an `*Srgb` format (lavapipe offered `Bgra8UnormSrgb`
first), and the GPU then encodes shader/clear values from linear light. The
Stage 0 spike showed intended `#591A8C` appearing as `#A059C4` when sRGB numbers
were passed straight through. Games and artists think in sRGB hex values.

## Decision
- `render::Color` stores **sRGB-encoded** components (`0.0..=1.0`) with
  straight alpha. Constructors: `rgb`, `rgba`, `rgb8`, `rgba8`, `hex(0xRRGGBB)`.
  Constants: `PURPLEPIE` (`#6A0DAD`), `BLACK`, `WHITE`, `TRANSPARENT`.
- Conversion happens in one place: `Color::to_wgpu(target_is_srgb)`. For sRGB
  targets it applies the IEC 61966-2-1 transfer function (`to_linear`). For
  `Unorm` targets it passes the values unchanged. Alpha is never gamma-converted.
- The clear color is configurable: `EngineConfig::clear_color` (default `Color::PURPLEPIE`).

## Alternatives Considered
- **Linear public colors:** mathematically convenient, but surprising for users (hex values look wrong).
- **Force a non-sRGB surface format:** avoids conversion for clears, but blending
  and texture filtering would then happen in the wrong space.

## Rationale
Colors look the same as in image editors, and blending stays correct in linear space.

## Consequences
### Positive
- Verified: the sandbox window is exactly `#6A0DAD` (921,600 of 921,600 pixels) under Xvfb + lavapipe.
### Negative
- Sprite textures (Stage 6) must be uploaded as `*Srgb` formats to match.

## Revisit Conditions
HDR or wide-gamut output (`SurfaceColorSpace`) becomes a goal.

---

# ADR-016: Diagnostics: `log` facade in the engine; games choose the logger

## Status
Accepted (2026-09-30, PP-014). Resolves pending decision PD-04.

## Context
Stage 4 is the first stage with diagnostics worth reporting: the chosen GPU,
surface state changes, and the details of GPU faults. wgpu already logs through
the `log` crate. winit 0.30 logs through `tracing`. The engine must not force a
logging backend on games (ADR-011), and new dependencies must earn their place (ADR-013).

## Decision
- Engine code logs with the **`log` facade** (`log = "0.4"`, already in the tree
  via wgpu, so no new crate). Levels: `error` for fatal GPU faults, `warn` for
  degraded operation, `info` for one-time facts (GPU name/backend, surface
  format, present mode), `debug` for per-event detail (suboptimal/outdated surface).
- The engine **never installs a logger**. Choosing one is the game's decision.
- The `sandbox` game installs a ~20-line built-in stderr logger. Its level comes
  from `PURPLEPIE_LOG=off|error|warn|info|debug|trace`, default `warn`. No `env_logger` dependency.
- `tracing` is not adopted. winit's `tracing` events are dropped when no subscriber is installed.

## Alternatives Considered
- **`tracing` for the engine:** richer (spans), but adds a dependency and a second
  ecosystem, and wgpu would still log via `log`.
- **`env_logger` in the sandbox:** it cannot be a sandbox-only dependency, because
  the library and binary share one package, so every game using the library would
  pull it in (several extra crates).
- **No logging at all:** hides wgpu's own explanations (e.g. "Found no drivers!").

## Rationale
It costs nothing new in the dependency graph and keeps games free to choose.
wgpu's diagnostics show up alongside PurplePie's in whatever logger the game installs.

## Consequences
### Positive
- Verified: with `PURPLEPIE_LOG=info` the sandbox prints
  `GPU: llvmpipe (…) (Vulkan, Cpu); surface Bgra8UnormSrgb, AutoVsync`. With no
  GPU driver, wgpu's own loader errors now appear before the PurplePie error.
### Negative
- winit's `tracing` diagnostics stay invisible unless a game installs a `tracing` subscriber itself.

## Revisit Conditions
Structured or span-based profiling becomes a need, or the package is split into a
workspace (ADR-002), at which point the sandbox could use `env_logger`.

---

# ADR-017: GPU faults are fatal and reported as `Error::Render`

## Status
Accepted (2026-09-30, PP-014).

## Context
wgpu 30's default uncaptured-error handler **panics**
(`backend/wgpu_core.rs: default_error_handler`, verified in source and by a
control run that panics with `wgpu error: Validation Error`). Device loss has a
separate callback. PP-006 skipped `Validation` acquire results and recreated
the surface on `Lost`. Testing PP-014 showed the second is unsafe: destroying
the X11 window under a running sandbox produced `Lost`, and recreating the
surface **panicked inside wgpu-hal 30.0.1** (`vulkan/instance.rs:407`, an
`expect` on `create_xlib_surface`: `ERROR_OUT_OF_HOST_MEMORY`). PurplePie cannot catch that cleanly.

## Decision
- Right after device creation, the renderer installs its own
  `on_uncaptured_error` and `set_device_lost_callback` handlers. They record the
  **first** fault in a `FaultSlot` (`Arc<Mutex<Option<GpuFault>>>`, required
  because wgpu callbacks must be `Send + Sync + 'static`) and log it at `error`.
- `Renderer::new` and `Renderer::render` (before and after each frame) turn a
  recorded fault into `Error::Render(source)`. The runner then exits the event loop
  cleanly, and `Engine::run` returns the error.
- Fatal: any uncaptured wgpu error (validation, out-of-memory, internal),
  device loss with reason `Unknown`, `CurrentSurfaceTexture::Validation`, and
  **`CurrentSurfaceTexture::Lost`** (no recreation attempt).
- Not fatal: `DeviceLostReason::Destroyed` (only caused by PurplePie itself),
  and `Timeout`, `Occluded`, `Outdated`, `Suboptimal`, minimized (unchanged from PP-006).
- No device recreation. No "N validation failures in a row" counter: wgpu
  routes acquire validation through the uncaptured-error handler, so the first one is already recorded.

## Alternatives Considered
- **Keep wgpu's panic:** abrupt, and no `Error` for the game to report.
- **Log and continue after validation errors:** they indicate engine bugs, and later frames would render garbage.
- **Recreate the surface on `Lost` (wgpu's documented recovery):** panics in wgpu-hal when the window is gone (measured).
- **`catch_unwind` around surface recreation:** unwinding through wgpu internals
  leaves unknown state, and the panic hook still prints a crash report.
- **Error scopes (`push_error_scope`) around every call:** more code on every
  path, with no benefit over a global handler when every error is fatal anyway.

## Rationale
Fail fast, but cleanly: one error type, one exit path, and a readable cause chain.

## Consequences
### Positive
- Verified: destroying the window mid-run → `error: GPU rendering failed` /
  `caused by: the window's GPU surface was lost`, exit 1, no panic (3/3 runs).
- The ignored GPU test proves an invalid call is captured instead of panicking.
### Negative
- A transient `Lost` on a still-living window (possible with some drivers on
  display changes) ends the game instead of recovering.
- One `Arc<Mutex<…>>` in the renderer (justified above).

## Revisit Conditions
`Lost` is observed on live windows on real hardware (then recover, but only
while the window is known to exist), or a wgpu upgrade makes surface recreation
return errors instead of panicking.

---

# ADR-018: World coordinates: +X right, +Y up, origin at the window centre, 1 unit = 1 logical pixel

## Status
Accepted (2026-10-01, PP-007). Resolves the core of PD-02. Camera controls remain pending (Stage 7). *Update 2026-10-01 (PP-009):* this view is now `Camera2D::IDENTITY`, the default camera (ADR-022); "origin at the window centre" holds for the default camera.

## Context
The first visible primitive needs a defined mapping from `Transform2D` to the
screen. PD-02 proposed +Y up, a centred camera and world units, with
`Camera2D` and `pixels_per_unit` in Stage 7. Stage 5 needs only the core.

## Decision
- **Axes:** +X right, **+Y up**. Rotation is in radians, **counter-clockwise** positive.
- **Origin:** the centre of the window.
- **Scale:** with the default view, **1 world unit = 1 logical pixel**. Logical
  pixels = physical pixels / window scale factor (DPI), so sizes look the same on
  high-DPI displays.
- **Resize:** a larger window shows **more of the world**. It does not stretch it.
- **Projection:** `glam::camera::rh::proj::directx::orthographic` (right-handed,
  Y-up view → WebGPU NDC with Y up and depth in [0, 1]).
- **Draw order:** not specified yet (ECS query order). Layering comes in Stage 6.

## Alternatives Considered
- **+Y down, top-left origin (screen/UI convention):** natural for UI, but
  unnatural for physics and maths and inconsistent with counter-clockwise rotation.
- **Normalized units (e.g. the window spans −1..1):** resolution-independent, but
  sizes become awkward fractions and aspect ratio leaks into game code.
- **Physical pixels:** objects shrink on high-DPI displays.
- **Deciding everything (`Camera2D`, `pixels_per_unit`) now:** no user for zoom or
  panning yet, so it waits for Stage 7.

## Rationale
It's the standard 2D game/maths convention and gives predictable sizes ("a
64×64 quad is 64×64 logical pixels"). Stage 7 can add a camera on top without
changing this default.

## Consequences
### Positive
- Verified under Xvfb: a 200×100 quad at (−300, 200) occupies exactly x 240..439,
  y 110..209 of a 1280×720 window (20,000 px). After resizing to 1000×600 it is still
  200×100, now at x 100..299, y 50..149.
### Negative
- UI-style code (top-left, +Y down) must convert. Stage 7's `screen_to_world`
  will provide that conversion for cursor input.

## Revisit Conditions
Stage 7 adds `Camera2D`. The default view must keep these semantics.

---

# ADR-019: Quads: one instanced pipeline, CPU-built clip matrices, embedded WGSL

## Status
Accepted (2026-10-01, PP-007).

## Context
Stage 5 draws solid-colour rectangles from ECS data. The first GPU drawing
code sets the pattern Stage 6 (sprites, batching) extends. Simplicity first (ARCHITECTURE §2).

## Decision
- One render pipeline, one draw call for **all** quads: `draw(0..6, 0..n)`,
  with the unit-square corners generated from `vertex_index`. There is **no vertex
  buffer and no bind group**.
- One **per-instance buffer** of `QuadInstance { clip_from_local: mat4, color: vec4 }`
  (renamed `Instance` in `src/render/instance.rs` by PP-008, which shares it with sprites; ADR-020)
  (80 bytes, `#[repr(C)]`, `bytemuck::Pod`; 96 bytes with `uv_rect` since ADR-027). The CPU computes
  `view_projection × Transform2D::to_mat4() × scale(size)` per quad. The buffer grows
  to the next power of two when needed, and the instance `Vec` is reused every frame.
- Colours are converted per target format on the CPU (ADR-015). Blending is
  `ALPHA_BLENDING` (straight alpha). No culling, because negative scale mirrors quads. No MSAA.
- The WGSL source lives in `src/render/quad.wgsl` and is **embedded with
  `include_str!`**. Engine shaders are code, not runtime assets. `assets/` stays for game data.
- `bytemuck` (1.25, `derive`) becomes a direct dependency. It was already in
  the tree via wgpu, so no new crate (ADR-013).

## Alternatives Considered
- **Uniform buffer + bind group for the view-projection, model matrices on the GPU:**
  saves a CPU matrix multiply per quad, but adds a bind group layout with no current need.
- **One draw call per quad:** simplest, but it scales poorly and has to be undone in Stage 6.
- **Shader files loaded from `assets/` at runtime:** path and lifetime handling
  (Stage 9 material), and a missing file would become a runtime error.

## Rationale
It has the fewest GPU objects that still batch. All maths is on the CPU, where it is unit-tested
(corner positions, rotation, scale, colour conversion).

## Consequences
### Positive
- Verified: the pipeline builds for 3 surface formats with zero GPU errors
  (ignored GPU test), and a broken WGSL shader is captured as a fault, not a panic.
- The screen output is pixel-exact (every pixel is one of the four expected colours).
### Negative
- CPU cost is O(n) matrix multiplies per frame. That's fine for thousands of quads, so measure before optimizing.
- Draw order is query order (unspecified) until Stage 6 adds layers. *(Resolved by ADR-021.)*

## Revisit Conditions
Stage 6 sprites (textures need bind groups), or a profiling result showing the per-quad CPU work matters.
*Update 2026-10-01 (PP-008):* sprites arrived as a second pipeline (ADR-020). This ADR still holds for quads.

---

# ADR-020: Textures: `TextureId` handles, decode on load, upload in the renderer; `image` (PNG only)

## Status
Accepted (2026-10-01, PP-008). Pulls the smallest piece of PD-06 (asset handles) forward. The rest of PD-06 (generic `Handle<T>`, unloading, other asset kinds) stays open for PP-011.

## Context
Sprites need textures. Game code must be able to name a texture inside a
component without touching wgpu (ADR-009: the renderer is crate-private, and
`Context` exposes no GPU types). The renderer can also be dropped and recreated
(`suspended`/`resumed`), so GPU objects cannot be the source of truth.

## Decision
- **Handle:** `render::TextureId`, a `Copy` newtype over a `u32` index with a private field, so it
  can only come from the engine. It lives in the public `Sprite { texture, size, tint }` component.
- **Loading:** `Context::load_texture(path) -> Result<TextureId>` reads and decodes the PNG
  **immediately**, so a missing or broken file is returned to the caller as
  `Error::Asset { path, source }`. A path is resolved against the working directory *(superseded by ADR-025: the asset root)*. The same path
  spelling returns the same id (no second read). Failed loads store nothing. `Context::texture_size(id)`
  returns the size in texels.
- **Store:** a crate-private `Textures` (owned by the runner, lent to `Context`) keeps every decoded
  image as RGBA8 sRGB with straight alpha, in load order. **Nothing is ever unloaded** in this stage.
- **Upload:** at the start of `Renderer::render`, the sprite pipeline uploads every store entry it has
  not seen yet (`Textures::since(n)`). A recreated renderer starts at 0 and re-uploads everything,
  which is why the CPU copies are kept. A texture larger than `max_texture_dimension_2d` becomes
  `Error::Asset` (fatal, like other render errors) before wgpu sees it.
- **GPU format:** `Rgba8UnormSrgb` when the surface is sRGB, `Rgba8Unorm` otherwise. This mirrors
  ADR-015: blending happens in the same space as quad colours on either kind of target.
- **Sampling:** one sampler, `Nearest` min/mag, `ClampToEdge`, no mipmaps. Crisp texels and exact pixel tests.
- **Pipeline:** a second instanced pipeline (`sprite.wgsl`) with the same `Instance` layout as quads
  (ADR-019). The instance colour is the tint, which multiplies the sample. UV `v = 0` is the top row of the image.
  One bind group (texture + sampler) per texture. Consecutive sprites with the same texture share one
  draw call. **Sprites are drawn after quads.** Sorting by layer and by texture is PP-015 (PD-05, PD-08).
  *(Superseded on 2026-10-01 by ADR-021: order now comes from `Layer`; within a layer quads still come first.)*
- **Decoder:** `image 0.25.10`, `default-features = false, features = ["png"]`. +12 crates
  (116 → 128 unique normal dependencies), with a highest `rust-version` of 1.88.

## Alternatives Considered
- **Load requests queued and decoded by the renderer:** errors would surface a frame later, away from
  the `?` in game code. Rejected: immediate, typed errors matter more.
- **`Context` holds a GPU handle and uploads directly:** breaks ADR-009, and breaks on renderer recreation.
- **Generic `Handle<T>` + `Assets` store now (full PD-06):** more design than one asset kind justifies. Revisit in PP-011.
- **`png` crate directly:** 4 fewer crates (no `moxcms`, `pxfm`, `byteorder-lite`, `num-traits`), but we would
  hand-write grey/RGB/16-bit → RGBA conversion, and Stage 9 would likely switch to `image` anyway.
- **Linear filtering:** smoother when scaled, but blurs pixel art and makes pixel tests inexact. A per-texture
  option can come later.

## Rationale
It's the smallest design that keeps wgpu out of game code, survives renderer recreation, and reports
bad files where the game can handle them.

## Consequences
### Positive
- Verified under Xvfb/lavapipe: a 16×16 PNG at 128×128 logical px is pixel-exact (each texel 8×8 px),
  including transparent texels (background shows through) and a 50% alpha quadrant that blends to the
  value computed in linear space. Missing and corrupt files exit with `Error::Asset` and a cause chain, no panic.
### Negative
- Every texture stays in RAM (CPU copy) and VRAM until the engine stops.
- Relative paths depend on the working directory. (The sandbox uses `CARGO_MANIFEST_DIR`.) *Resolved by ADR-025.*
- An oversized texture is only detected at upload, so it stops the engine rather than returning from `load_texture`.

## Revisit Conditions
PP-011 (asset system), memory pressure from many or large textures, the need for texture atlases or UV
rectangles, or a need for linear filtering.

## Extension: texture regions / sprite sheets (2026-10-06, PP-019)
- **`Sprite::region: Option<TextureRegion>`** (builder `with_region`; `None` = whole texture, the previous behaviour).
  `TextureRegion { x, y, width, height }` is in **texels**, top-left origin, rows down, like an image editor, matching how sheets are drawn and
  avoiding float maths in game code. The draw list converts it to the instance's `uv_rect` (ADR-027) using the texture's size from the CPU
  store; a region that sticks out is cut to the texture, one completely outside (or for an unknown texture) is not drawn.
- **`SpriteGrid { cell_width, cell_height, columns, rows, spacing, margin }`** finds a cell's region by `cell(column,
  row)` or by row-major `frame(index)`; overflow and out-of-range cells return `None`.
- **No flip API**: mirroring is a negative transform scale (already supported, culling is off). **No new material**:
  regions of one texture batch together (one draw call), so a whole sheet costs what one sprite texture did.
- **Alternatives:** normalized UV rectangles in the public API (fragile and float-y for pixel art); a separate
  `SpriteSheet` asset type with its own handle (a second handle kind for the same pixels; revisit with PD-06 generic
  handles); per-sprite flip flags (duplicates negative scale).
- **Verified:** an offscreen GPU test draws four cells at 4× (one mirrored) and every pixel equals its own texel (no
  neighbouring-cell texel); the sandbox shows the same cells and matches the whole-frame model at zoom 1, 1.5 and 2.
  Sprites without a region are pixel-identical to before (Breakout's lose screen unchanged).
- **Revisit:** frame animation (PP-020) builds on `SpriteGrid::frame`; linear filtering (F9) would need padding between
  cells (sampling bleeds across cell edges), so it stays `Nearest`-only for now.

---

# ADR-021: Draw order and batching: optional `Layer` component, one sorted draw list, one draw call per (layer, material) run

## Status
Accepted (2026-10-01, PP-015). Resolves PD-08 (draw order) and PD-05 (sprite batching). Supersedes ADR-020's "sprites after quads" clause.

## Context
After PP-008, quads always drew under sprites and the order inside each kind was hecs query order. That order
depends on archetypes, so it changes when a component is added to or removed from an entity. Games had no way
to put a quad (a health bar, a fade overlay) over a sprite, and sprites alternating between textures cost one draw call each.

## Decision
- **Public API:** `render::Layer(pub i32)`, an optional ECS component. No `Layer` means layer 0. Higher layers draw on top.
  `Quad` and `Sprite` are unchanged.
- **Order:** every frame the renderer builds one draw list (`render/draw.rs`) from all `(Transform2D, Quad)` and
  `(Transform2D, Sprite)` entities and sorts it by the key **(layer, material rank, entity index)**:
  - the material rank puts quads (0) before sprites, and sprites by texture (`1 + TextureId`);
  - the entity index (`Entity::id()`) is a tie-breaker that makes the order total and independent of query order.
    It is deterministic but **not part of the API**: overlapping drawables whose order matters should use different layers.
- **Batching:** consecutive items with the same (layer, material) form one batch = one draw call
  (`draw(0..6, range)`). Within a layer that's 1 call for all quads + 1 per distinct texture.
- **One instance buffer** for quads and sprites in draw order (same `Instance` layout, ADR-019/020). The pipelines
  only switch when the material kind changes; the bind group changes per texture batch.
- No depth buffer: order is painter's order (alpha blending needs back-to-front anyway).

## Alternatives Considered
- **A `layer` field on `Quad` and `Sprite`:** discoverable, but changes both public structs and has to be repeated
  for every future drawable (text, shapes). A component is optional and shared.
- **`z: f32`:** allows in-betweens, but floats need a total order (NaN) and invite "z-fighting" style confusion. `i32` is enough for 2D.
- **Keep spawn/query order within a layer (no texture sort):** intuitive for overlap, but alternating textures cost
  a draw call each, and query order isn't stable anyway.
- **Depth buffer + z test:** breaks for translucent sprites, which must be sorted anyway.
- **Stable sort by query order instead of an entity tie-breaker:** order would change when a component is added
  (archetype move). Verified: a test that moves an entity to a new archetype fails without the tie-breaker.

## Rationale
It's a single sort over plain keys, testable without a GPU, and it gives both requirements (control over overlap,
few draw calls) with one optional component.

## Consequences
### Positive
- Verified under Xvfb: a layer-1 quad covers a layer-0 sprite, and a layer-0 quad shows only through the sprite's
  transparent texels (0 mismatches over 40,000 checked pixels, also after a resize). In a control run with the quad on
  layer −1 it went under the sprite.
- Draw calls = (layer, material) runs, unit-tested (5 sprites alternating between 2 textures + 2 quads → 3 calls, not 7).
- Rough cost (release build, Cowork CPU): building and sorting the list takes ~0.08 ms for 1,000 drawables,
  ~1.8 ms for 10,000 and ~7.9 ms for 50,000 (8 textures, 3 layers → 21 draw calls).
### Negative
- An O(n log n) sort every frame, even when nothing changed.
- Same-layer overlap between sprites with different textures follows texture ids, which can surprise users. Documented on `Layer`.
- Entity indices are reused after despawn, so a respawned entity may change its place among equals.

## Revisit Conditions
Profiling shows the per-frame sort matters (cache the sorted list or sort only on change), a need for
y-sorting or sub-layer order, or more drawable kinds (text, shapes) that need their own material ranks.

---

# ADR-022: One engine-owned `Camera2D { position, zoom }` in `Context`; screen ↔ world in logical pixels

## Status
Accepted (2026-10-01, PP-009). Resolves PD-02 (camera controls). Builds on ADR-018, which becomes the default camera.

## Context
ADR-018 fixed the view: world origin at the window centre, 1 unit = 1 logical pixel. Games need to follow a
player, zoom, and (from Stage 8) turn cursor positions into world positions. The renderer must stay
crate-private and read-only (ADR-009), and game code must not see wgpu or winit types.

## Decision
- **Type:** public `render::Camera2D { position: Vec2, zoom: f32 }` (plain `Copy` data). `position` is the world point at
  the window centre; `zoom` is logical pixels per world unit. `Camera2D::IDENTITY` / `Default` = ADR-018's view, so
  games that ignore the camera see no change. A non-finite or non-positive zoom is treated as 1 (`effective_zoom`).
  No rotation.
- **Ownership:** exactly **one** camera, owned by the runner, lent to the game through `Context::camera()` /
  `camera_mut()`, and passed read-only to `Renderer::render`. Changes apply to the next frame drawn.
- **Projection:** `Camera2D::view_projection(viewport)` (crate-private) = orthographic over the visible world rectangle
  `position ± viewport / 2 / zoom`, same glam function as ADR-018. It replaced `quad::view_projection`.
- **Screen space:** logical pixels, origin at the top-left of the drawing area, +Y down (OS cursor convention), pixel
  centres at `n + 0.5`. `Camera2D::screen_to_world(screen, viewport)` and `world_to_screen` are exact inverses.
  Physical pixels → logical: divide by the window scale factor (input in Stage 8 will do this before calling them).
- **Viewport:** `Context::viewport_size()` = window size in logical pixels. The runner updates it **only from window
  events** (creation, `Resized`, `ScaleFactorChanged`), never by querying the window per frame (see Consequences).

## Alternatives Considered
- **Camera as an ECS component/entity (Bevy style):** allows several cameras, but needs "which camera is active" rules
  and a query in the renderer. One camera covers a small 2D engine; split screen is not a goal.
- **`pixels_per_unit` separate from `zoom`:** a second scale knob with no current user. `zoom` already maps world units to pixels.
- **Camera rotation:** no current need; it complicates `screen_to_world` and pixel-exact tests. Can be added without breaking callers.
- **Screen coordinates with +Y up or in physical pixels:** +Y down matches what the OS reports for the cursor, and
  logical pixels match how sizes are specified everywhere else (ADR-018).
- **`screen_to_world` on `Context` only:** the pure methods on `Camera2D` are unit-testable without a window; a
  `Context` convenience can be added with input (Stage 8).

## Rationale
It is the smallest change that gives panning, zooming and a tested screen ↔ world mapping, keeps ADR-018 as the
default, and keeps the renderer read-only.

## Consequences
### Positive
- Verified under Xvfb: whole-frame per-pixel models for cameras (0, 0)×1, (0, 120)×2 and (200, 0)×0.5, plus (0, 120)×2
  after a resize to 1000×600, all with **0 mismatches** (moving/rotated objects excluded). A 1-unit camera offset in the
  model produces ~3,000 mismatches, so the check is sensitive.
- Unit tests: default camera equals the ADR-018 matrix exactly; pan; zoom; invalid zoom; round trips; DPI 1.0/1.25/2.0
  agreement between the renderer's projection and `world_to_screen`.
- **Found while testing:** reading the window size every frame (`Window::inner_size`) panicked inside winit 0.30.13 on
  X11 once the window had been destroyed (`GetGeometry` → `unwrap`), turning the clean ADR-017 exit into a crash. The
  viewport is therefore event-driven (R-22).
### Negative
- Non-integer zoom with `Nearest` sampling makes texels uneven in size (1 or 2 pixels at zoom 1.5).
- One camera only; no UI/screen-space layer that ignores the camera.

## Revisit Conditions
Split screen or minimaps (several cameras), a screen-space UI layer, a need for camera rotation, or pixel-art games that need integer-zoom snapping.

---

# ADR-023: License: MIT OR Apache-2.0

## Status
Accepted (2026-10-01, PP-013, owner decision). Resolves PD-07.

## Context
The repository is public on GitHub and meant as a portfolio project others can read and reuse. Without a license,
nobody may legally reuse the code. Every dependency PurplePie uses is available under MIT and/or Apache-2.0.

## Decision
Dual license, **MIT OR Apache-2.0**, at the user's option: `license = "MIT OR Apache-2.0"` in `Cargo.toml`,
`LICENSE-MIT` (copyright 2026 David Vilela) and `LICENSE-APACHE` (canonical Apache-2.0 text; its appendix line reads "Copyright 2026 David Vilela", filled in by the owner on 2026-10-02), plus the usual Rust
"Contribution" clause in the README (contributions are dual licensed under the same terms).

## Alternatives Considered
- **MIT only:** simplest, but no explicit patent grant.
- **Apache-2.0 only:** patent grant, but incompatible with GPLv2 projects.
- **No license / all rights reserved:** the code would be visible but not reusable.

## Rationale
It is the Rust ecosystem convention (the compiler, wgpu, glam, hecs, winit-adjacent crates), so it adds no friction
for anyone combining PurplePie with other crates.

## Consequences
- Anyone may use, modify and redistribute PurplePie under either license.
- Changing the license later would need the agreement of every contributor (today: only the owner).

## Revisit Conditions
None expected.

---

# ADR-024: Keyboard input: own `KeyCode` (physical keys), `Input` with per-callback edges latched for fixed steps

## Status
Accepted (2026-10-02, PP-010). Resolves PD-03 for the keyboard. Mouse buttons follow the same model in PP-016.

## Context
Games need held-key state and press/release edges. The engine runs 0..N `fixed_update`s and one `update` per
frame (ADR-010), and window events arrive between frames. A naive "edges are cleared each frame" design loses a
press when a frame has no fixed step and reports it N times when it has several (R-05). winit types must not leak
into the public API (ADR-003).

## Decision
- **Public module `input`:** `KeyCode` (99 physical keys named by US-layout position: letters, digits, F1–F12, arrows,
  editing keys, modifiers, punctuation, numpad; `#[non_exhaustive]`, `#[repr(u8)]`) and `Input` with `pressed`,
  `just_pressed`, `just_released`, `axis(negative, positive)` and `pressed_keys`. Keys are stored in 128-bit masks.
- **Physical, not logical, keys:** `KeyCode::W` is the key above `S` on every layout (it's `Z` on AZERTY), which is
  what movement controls need. Text input (logical characters) is out of scope.
- **Two edge sets:** `update` sees every edge since the previous frame (cleared after `update`). `fixed_update` sees
  edges latched until the **first fixed step that runs** after the event (cleared after that step). Result: every
  edge reaches each callback exactly once, whatever the step count. `pressed` is the same everywhere.
- **Rules:** OS auto-repeat is ignored (a held key can't be pressed again). A release without a press is ignored.
  A press and release in one frame is `just_pressed` + `just_released` but never `pressed`. Losing focus releases
  every held key (with release edges). X11's synthetic presses on focus gain are ignored.
- **Translation in `app/keymap.rs`** (the only file that sees both key types; a test checks it is a bijection onto
  `KeyCode::ALL`). Unknown keys are dropped. Key events are logged at `debug` level.
- **`Context::input()`**, read-only. `Context` now borrows one `EngineState` (world, time, textures, camera, input,
  viewport) instead of a growing argument list.
- `EngineConfig::exit_on_escape` stays: Escape (logical) exits before reaching `Input`, as before.

## Alternatives Considered
- **Edges only in `update`:** simple, but gameplay in `fixed_update` (where it belongs) couldn't react to presses
  reliably. This was R-05's fallback.
- **Clear all edges at frame end:** loses presses on 0-step frames and doubles them on N-step frames (a mutation of
  the implementation into this design fails 3 unit tests).
- **Event queue instead of state:** more flexible, but every game must fold events into state itself.
- **Logical keys / re-exporting winit's `KeyCode`:** layout-dependent controls, and winit in the public API.

## Rationale
Gameplay code gets the obvious API (`just_pressed` works anywhere), and the subtle fixed-step behaviour is in one
small, unit-tested, platform-free module.

## Consequences
### Positive
- Verified: 10 unit tests (0/1/N steps, repeat, tap, focus release, axis), 3 keymap tests, and Xvfb runs with XTEST
  keys: 3 `=` taps counted exactly 3 times in `fixed_update` and 3 in `update` (zoom 1 → 8 → ÷2 = 4), held arrows
  panned the camera and the resulting frame matched the per-pixel model (0 mismatches). Holding → Right and removing
  focus stopped the pan at the focus loss (camera 170 ≈ 0.576 s × 300 px/s), not at the later key release.
### Negative
- An edge reaches `fixed_update` one frame late when the frame that saw it had no fixed step (inherent to fixed steps).
- Only 99 keys; anything else is ignored until added.
- `exit_on_escape` remains a hard-wired shortcut in `app` (now redundant with `Input`, kept for compatibility).

## Revisit Conditions
Text input, gamepads, key rebinding / action maps, or more than 128 keys.

## Extension: mouse (2026-10-06, PP-016)
Same model, no new decision needed, so no separate ADR:
- `MouseButton { Left, Right, Middle, Back, Forward }` (extra buttons ignored) with `mouse_pressed` / `mouse_just_pressed` /
  `mouse_just_released`. Keys and buttons share one implementation (a `Buttons` bit-mask set with both edge sets), so
  the 0/1/N-step guarantees are identical. Focus loss releases mouse buttons too.
- **Cursor** = plain state: `Input::cursor_position()` in **logical screen pixels** (top-left origin, +Y down, ADR-022),
  converted from winit's physical position with the window's DPI scale (tracked from window events, never queried per
  frame); `None` after `CursorLeft`. `Context::cursor_world()` = `camera.screen_to_world(cursor, viewport)`.
- **Wheel** = `Input::scroll()` in lines (`y > 0` = away from the user), delivered once to `fixed_update` and once to
  `update` like an edge. Touchpad pixel deltas: physical → logical → ÷ 20 px per line.
- **Observed (X11 + XTEST only):** winit 0.30.13 reports **two** `MouseWheel` events per synthetic wheel click (it maps
  both the button-4/5 press and release, and XTEST events are not flagged as emulated). Real X11 wheels arrive as XI2
  axis motion instead. PurplePie passes events through unchanged, so under xdotool one click = 2 lines. Unverified on
  real hardware (R-12).

---

# ADR-025: Asset root: relative asset paths resolve against one folder chosen at startup

## Status
Accepted (2026-10-06, PP-011). Resolves the file-location part of PD-06 and mitigates R-24. Generic handles and unloading
(the rest of PD-06) are deferred until a second asset kind exists.

## Context
Since PP-008, `Context::load_texture` resolved relative paths against the process's working directory. That works for
`cargo run` from the project folder, but a shipped game started by double-clicking its executable (working directory
= wherever the OS chooses) could not find its files. The sandbox worked around it with a compile-time absolute path.

## Decision
- New crate-private module `assets` with `AssetRoot`, resolved **once** when the engine starts:
  1. `EngineConfig::asset_root` / `with_asset_root(path)` if set (a relative root is joined to the working directory at
     startup; it is used even if it doesn't exist, so typos surface as file-not-found errors naming the full path);
  2. else `<executable's folder>/assets` if it is a directory;
  3. else `<working directory>/assets` if it is a directory (the `cargo run` case);
  4. else no root: loading a **relative** path fails with `Error::Asset` whose source lists both searched folders and
     suggests `with_asset_root`. The engine logs the chosen root at `info` (or a `warn` when none).
- `Context::load_texture(path)`: absolute paths are used unchanged; relative paths are joined to the root. The texture
  cache key is the resolved path, so a relative and an absolute spelling of the same file share one `TextureId`.
- `Context::asset_root()` exposes the chosen folder (`None` if none).
- Path resolution lives in `assets`, called from `app`; `render::Textures` still receives a full path and stays
  file-location-agnostic.

## Alternatives Considered
- **Working directory only (status quo):** breaks double-click launches.
- **Executable folder only:** breaks `cargo run`, where the executable lives in `target/debug/`.
- **Compile-time `CARGO_MANIFEST_DIR`:** an absolute path of the build machine; useless for shipped games.
- **Embedding assets in the binary (`include_bytes!`):** no files to lose, but every asset change needs a rebuild and the
  binary grows; can be added later as an option.
- **Resolving the root per load:** the working directory could change mid-run and give inconsistent results.

## Rationale
The two common layouts (development: project folder; release: `assets/` next to the executable) both work without
configuration, and every failure names exactly where PurplePie looked.

## Consequences
### Positive
- Verified end to end: started from the project folder, via `cargo run`, as a "shipped" copy (binary + `assets/`)
  launched from an unrelated folder, and as a moved binary run from the project folder (working-directory fallback).
  With no `assets/` anywhere the sandbox exits 1 with the searched folders in the error, no panic.
- The sandbox no longer needs `CARGO_MANIFEST_DIR`.
### Negative
- `cargo run` from a subfolder of the project (e.g. `src/`) finds no root unless the game sets one.
- A release build must ship the `assets/` folder next to the executable (documented in the README).

## Revisit Conditions
Platform bundles (macOS `.app` `Resources/`), embedded assets, or a second asset kind (generic handles, PD-06).

---

# ADR-026: API review after Breakout: keep `Game` + `Context` and a single crate; add `Hidden`, `Camera2D::fit`, `Context::set_window_title`

## Status
Accepted (2026-10-06, PP-017). Reviews ADR-008 (engine/game API) and ADR-002 (single package). Completes Stage 10.

## Context
PP-012 built a complete game (`examples/breakout.rs`) on the public API with no engine changes and recorded ten friction
points (F1–F10, TASKS PP-012). Stage 10 asks for a decision on each, a check of the `Game`/`Context` design, the fate of
`EngineConfig::exit_on_escape`, and whether to split the crate.

## Decision
**ADR-008 stands.** `Game` (`init` / `fixed_update` / `update`, all optional) plus a per-call `&mut Context` worked for a full
game: every interaction Breakout needed went through `Context`, and keeping gameplay in `fixed_update` made it deterministic.

| Friction | Decision |
|---|---|
| F1 No text rendering | **Deferred**, post-portfolio feature PP-018 (needs a font dependency and a glyph atlas; too big for a review task). F2 covers the most urgent need meanwhile. **Resolved 2026-10-06 by ADR-027 (PP-018a/b).** |
| F2 No runtime window title | **Added** `Context::set_window_title(title)`. Applied by the runner after the callback, only when requested (no per-frame window call, R-22). |
| F3 No visibility toggle | **Added** the `render::Hidden` marker component. The draw list skips entities that have it (`hecs::Without`). |
| F4 Copying values out of `Context` before mutating the world | **No change.** It's the borrow checker enforcing one `&mut Context`; splitting `Context` into simultaneous borrows would complicate the API for a few saved lines. Documented on `Context::world_mut`. |
| F5 Verbose per-entity component access | **No change.** It's hecs's API (`world.get::<&mut T>(e)`), re-exported deliberately (ADR-006); wrapping it would duplicate hecs. |
| F6 No "cursor moved" signal | **No change.** Two lines in the game (`last != current`); revisit if a second game needs it. |
| F7 No camera fit helper | **Added** `Camera2D::fit(center, size, viewport)`: the largest zoom that shows the whole area (invalid sizes → zoom 1). |
| F8 `request_exit` has no exit code | **No change.** `Engine::run` returns `Result<()>`; a game that needs a process exit code can call `std::process::exit` after `run` returns. |
| F9 `Nearest` sampling aliases scaled-down sprites | **Deferred** with sprite sheets / per-texture sampling options (post-portfolio). |
| F10 No randomness helper | **No change, by design.** Games bring their own generator (determinism and choice of algorithm stay with the game). |

- **`exit_on_escape` stays**, default `true`: a convenience for prototypes, handled before input reaches the game; games that
  want Escape (pause menus) turn it off and read `Input`. Its doc comment now says so.
- **Single crate stays** (ADR-002 reaffirmed): ~5,800 lines in `src/` (tests included), clean builds dominated by
  dependency compilation (wgpu), and module boundaries already enforced by convention and the per-task import checks. Revisit when a feature brings
  a heavy optional dependency (text, audio) that should be feature-gated or separately versioned.
- **`#![warn(missing_docs)]`** is enabled in `lib.rs`; with clippy's `-D warnings` in CI, undocumented public items fail the build.

## Alternatives Considered
- **Fix every friction item now:** F1 (text) alone is a stage of work; F4/F5 would add API surface that mostly duplicates
  Rust/hecs. The review should leave the API smaller and clearer, not bigger.
- **A `Visible(bool)` component instead of a `Hidden` marker:** every drawable would need it or a default; a marker is
  absent by default and costs nothing for the common case.
- **Workspace split now (core / render / input / app crates):** more `Cargo.toml`s and public re-exports to maintain with
  one consumer; no measured build-time problem yet.

## Rationale
Change the API only where a real game showed a missing capability that is small, general and testable; record everything
else so later work starts from the evidence.

## Consequences
### Positive
- Breakout uses all three additions: the end-screen overlay is one entity toggled with `Hidden`, the camera uses
  `Camera2D::fit`, and the window title shows score and lives. Its autoplay results are unchanged (the gameplay code is).
- Public items are fully documented and kept so by CI.
### Negative
- Text, sprite sheets and sampling options remain missing for now (PP-018 first).
- `exit_on_escape` remains a second way to handle one key.

## Revisit Conditions
A second example game (re-check F4–F6 with fresh evidence), text rendering (PP-018), or a heavy optional dependency
(workspace split).

---

# ADR-027: Text: `ab_glyph` + one glyph atlas drawn by the sprite pipeline; `Text` component; `FontId` handles

## Status
Accepted (2026-10-06, PP-018a; extended by PP-018b the same day: anchors, measuring, Breakout HUD). Resolves friction F1 (ADR-026).
Extends ADR-019/ADR-020 (instance data gains a UV rectangle) and ADR-021 (a third material, drawn last in each layer).
Decides PD-06 for now: fonts get their own `FontId` like `TextureId`; generic handles stay deferred.

## Context
Breakout had to fake score and lives with sprites and the window title. Games need on-screen text that follows the camera,
layers and DPI like everything else, without exposing GPU types (ADR-009). Options were a font-rasterizing dependency or a
hand-made bitmap font.

## Decision
- **Rasterizer: `ab_glyph` 0.2.32** (Apache-2.0; `ab_glyph_rasterizer`, `owned_ttf_parser`, `ttf-parser`), default
  features off (`std` only). Measured with `cargo tree -e normal` on 2026-10-06: **+0 crates on Linux** (winit's Wayland
  decorations already depend on it) and **+4 on Windows** (99 → 103). `fontdue` 0.9.4 would add 4 on Linux and 5 on
  Windows (its `hashbrown` 0.15 is a second copy) and needs `--no-default-features` work to trim. Used only in `src/render/`.
- **Fonts are an asset kind like textures** (ADR-020 pattern): `Context::load_font(path) -> Result<FontId>` resolves the
  path through the asset root (ADR-025), reads and parses the file immediately (`Error::Asset` on failure), and deduplicates
  by path. The CPU store (`render::Fonts`) lives in `EngineState`; fonts are never unloaded.
- **`render::Text { content, font, size, color }`** is a component drawn with `Transform2D`. The transform's position is
  the left end of the first line's baseline; `size` is the em size in world units (like CSS `font-size`); `'\n'` starts
  a new line (ascent − descent + line gap, rounded); other control characters are skipped; no shaping (one glyph per
  `char`), kerning only from a `kern` table. `Hidden` and `Layer` apply. Within a layer: quads, then sprites, then text.
- **Rasterize at the on-screen size.** Per text, the draw list computes physical pixels per local unit
  (camera zoom × DPI × transform scale; the larger axis if uneven), rasterizes glyphs at `size` × that (cached per font,
  glyph, and em size in 1/64 px), lays them out with pen positions rounded to whole pixels, and moves the text origin to
  the nearest physical pixel corner. Axis-aligned text therefore maps atlas texels 1:1 onto pixels. Text larger than
  512 physical pixels is skipped.
- **One glyph atlas**, 1024×1024 RGBA (4 MiB), owned by the renderer: white RGB with coverage in alpha, a 1-texel
  transparent border per glyph, shelf packing, dirty rows uploaded before drawing. When it is full mid-frame it is
  cleared once and the frame's text is laid out again; glyphs that still don't fit are skipped. It is sampled with a
  **linear** sampler (exact at texel centres; smooth for rotated or scaled text) through the **sprite pipeline**: the
  shared `Instance` gains `uv_rect` (96 bytes now); sprites use the whole texture, quads ignore it. One draw call per
  (layer, text) run.
- **Font shipped:** `assets/fonts/Poppins-Regular.ttf` (160 KB, unmodified) with `assets/fonts/OFL.txt` (SIL Open Font
  License 1.1; copyright line from the font's `name` table, licence text from the Rust toolchain's copy). Games may use any
  `.ttf`/`.otf`.

## Alternatives Considered
- **Bitmap font (pre-rendered PNG):** no dependency, but one size, ASCII only, blurry or blocky when the camera zooms, and
  a custom format to maintain.
- **`fontdue`:** fast and popular; slightly larger dependency cost here and nothing `ab_glyph` lacks for this scope.
- **`glyphon`/`cosmic-text` (shaping, wgpu text renderer):** proper shaping and font fallback, but a large dependency tree,
  its own wgpu pipeline (version lock-step with wgpu), and text outside our draw list and layers.
- **Rasterize once at a fixed size and scale on the GPU:** blurry when zoomed in, aliased when zoomed out.
- **An `R8` atlas and a dedicated text pipeline:** a quarter of the memory, but a third pipeline and shader; the RGBA atlas
  reuses the sprite pipeline unchanged.

## Rationale
The cheapest dependency that rasterizes real fonts, and a design that keeps text inside the existing draw list, layers,
camera and DPI handling, so it behaves like every other drawable and is testable pixel for pixel.

## Consequences
### Positive
- Verified: an offscreen GPU test renders text at fractional positions (zoom 1 and 2) and matches the CPU rasterization
  within 1/255 on every pixel; the Linux sandbox shows its label with the rest of the frame unchanged (whole-frame model
  outside the label: 0 mismatches) and close to FreeType's rendering of the same font.
- `uv_rect` also enables sprite sheets later without another instance-format change.
### Negative
- No shaping (no ligatures, complex scripts, right-to-left), no font fallback. (Measuring and alignment: see the extension below.)
- Coverage is blended in linear space on sRGB targets (ADR-015), so light-on-dark text looks slightly bolder than
  gamma-space renderers at small sizes.
- Continuous zooming rasterizes every new size; a full atlas costs a re-layout (logged once at `info`).
- Fonts and their glyph caches are never unloaded (like textures).

## Revisit Conditions
A need for shaping or non-Latin scripts (consider `cosmic-text`/`swash`), many sizes or very large text (signed distance
fields), a third asset kind or unloading (generic handles, PD-06), or atlas resets showing up in profiles.

## Extension: anchors and measuring (2026-10-06, PP-018b)
- **`Text::anchor: TextAnchor { horizontal, vertical }`** (default `BASELINE_LEFT`, the PP-018a behaviour; builder
  `with_anchor`; 12 constants such as `TOP_LEFT`, `CENTER`, `BOTTOM_RIGHT`). `HorizontalAnchor` = `Left | Center |
  Right` puts that edge of **every line** at the position, so it also left/centre/right-aligns multi-line text.
  `VerticalAnchor` = `Baseline | Top | Middle | Bottom`, measured from the font's ascent and descent lines (the same for
  any string), so text does not jump when its characters change (e.g. a score going from 9 to 10 keeps its height).
- **Layout** stays pixel-exact: each line's shift (`−width × 0 | ½ | 1`) and the block's vertical shift are rounded to
  whole pixels before the glyphs are placed. The line widths come from a first pass over the string into a reused
  scratch buffer (no per-frame allocation).
- **`Context::measure_text(&Text) -> Option<TextMetrics>`** returns world-unit `width` (widest line, advance widths +
  kerning), `height`, `ascent`, `descent`, `line_height` and `lines`, independent of camera and window; and
  **`TextMetrics::bounds(anchor)`** gives the block's rectangle relative to the entity position (+Y up), e.g. for a
  panel behind the text. Drawn text can differ by up to a pixel (rounding).
- **Alternatives:** a separate `align` field next to an anchor (two concepts for one need in a 2D HUD; can be split later
  if text boxes with wrapping arrive); ink bounds instead of font metrics (would make text move as characters change);
  measuring through the renderer (would need a frame and depend on the camera).
- **Breakout** now shows its score, the win/lose headline and the next action as `Text` (centred), verified: all pixels
  that changed on the lose screen lie inside the rectangles `TextMetrics::bounds` predicts, and the autoplay results are
  unchanged.

---

# ADR-028: Sprite animation: a component advanced by a game-called system; the renderer draws its current frame

## Status
Accepted (2026-10-07, PP-020). Builds on the ADR-020 extension (texture regions, `SpriteGrid`).

## Context
Sprite sheets (PP-019) cover tiles but not motion. Every game would otherwise write the same timer and frame-index code,
and get the fixed-step edge cases wrong (frames a step late from `f32` sums, huge `dt` after a stall).

## Decision
- **`render::SpriteAnimation`** (component): public `grid`, `first`, `last` (inclusive; `last < first` plays backwards),
  `fps`, `mode: AnimationMode { Loop (default), Once }`; private playback state (frame step + seconds in the current
  frame, `f64`). Methods: `new`, `once`, `frame`, `region`, `len`, `is_finished`, `restart`, `advance(dt)`.
- **`render::advance_animations(&mut World, dt)`** advances every animation; the game calls it from `fixed_update`,
  exactly like `ecs::integrate_velocity` (ADR-008: the game owns the schedule). Playback depends only on the summed
  `dt`, so it is deterministic with the fixed timestep.
- **The draw list reads the animation** (`Option<&SpriteAnimation>` in the sprite query): while an entity has one, its
  sprite shows the animation's current frame and `Sprite::region` is ignored. So the right frame is drawn from the very
  first frame, before any fixed step has run, and there is no second copy of the frame to keep in sync.
- **Timing:** frames advance when the time in the current frame reaches `1/fps` (minus a 1 µs slack). Without the
  slack, 50 Hz steps at 25 fps show a frame one step late (`f32` sums fall a hair short); with it, every tested pair
  (60/10, 60/12, 60/15, 60/30, 120/24, 144/24, 50/25, 30/6 Hz/fps) switches on exactly the expected step for 20 loops.
  Long `dt` skips frames like many short steps; invalid `fps` or `dt` holds the frame; absurd `dt` saturates.

## Alternatives Considered
- **The system writes `Sprite::region`:** two sources of truth, and the first rendered frame shows the whole sheet until
  a fixed step runs.
- **Advance animations automatically inside the engine loop:** hides scheduling from the game (pause menus, slow motion
  and hit-stop would need engine options); every other system is game-called.
- **Seconds-since-start state:** loses precision over long sessions; the per-frame remainder stays bounded.
- **Per-frame durations / named clips / events:** not needed yet; a game can switch `first`/`last`/`fps` or swap the
  component.

## Rationale
Smallest API that removes the repeated timer code, keeps scheduling in the game, and is exactly testable.

## Consequences
### Positive
- Verified: unit tests for timing, ranges, reversal, once/restart, long steps, invalid values; the sandbox's animated cell
  shows exactly one sheet frame in every screenshot, and timed runs print the frame `⌊steps / 15⌋ mod 8` (4 fps at
  60 Hz) after 119, 399 and 630 steps.
### Negative
- `Sprite::region` is silently ignored while an animation is attached (documented on both types).
- Playback stops if the game forgets to call `advance_animations` (documented in the examples).

## Revisit Conditions
Variable frame durations, animation events (e.g. "footstep on frame 3"), blending between clips, or many animated
entities showing up in profiles.

---

# ADR-029: Screen space: a `ScreenSpace` component draws in window pixels from an anchor, after all world content

## Status
Accepted (2026-10-07, PP-021). Extends ADR-021 (draw order) and ADR-022 (camera); first step of the in-game UI plan
(ROADMAP P2).

## Context
Everything was drawn through the one `Camera2D`, so a HUD moves and scales with the camera. Breakout works around it by
fitting its camera so that the HUD stays put; a game that scrolls or zooms cannot. Every UI element (panels, buttons,
menus) needs a camera-independent space first.

## Decision
- **`render::ScreenSpace { anchor: ScreenAnchor }`** (marker-like component; constants `TOP_LEFT` … `BOTTOM_RIGHT`,
  default `CENTER`). An entity with it has its `Transform2D` read as **logical pixels from that window point**, with the
  world's axes (**+X right, +Y up**), so rotation, scale, `TextAnchor` and every drawable behave exactly as in the world.
  `ScreenSpace::CENTER` is, by construction, the default camera's view (unit-tested equality).
- **Projection:** per entity, `orthographic` over the logical viewport with the anchor as origin (same convention as
  `Camera2D::view_projection`); text keeps its pixel snapping because `TextPlacement` takes the projection as input.
- **Order:** the sort key gains a leading space bucket (world 0, screen 1): screen-space content is drawn after **all**
  world content, regardless of `Layer`; within screen space, `Layer` orders as usual. Batching is unchanged.
- **Helpers for UI hit tests:** `anchor_point(viewport)`, `from_window(window_pos, viewport)` and `to_window(pos,
  viewport)` convert between window coordinates (`Input::cursor_position`, top-left origin, +Y down) and the space.

## Alternatives Considered
- **+Y down screen coordinates** (common in UI toolkits): would make transforms, rotation and text anchors behave
  differently from the world; anchoring to corners already gives natural offsets.
- **A second camera / a `RenderLayer` with per-layer cameras:** more general (minimaps, split screen) but more API than
  a HUD needs; can be added later and `ScreenSpace` mapped onto it.
- **Ordering screen space by `Layer` together with the world:** a HUD would need huge layer numbers to stay on top.

## Rationale
The smallest change that gives UI a stable coordinate system, reuses every existing drawable unchanged, and keeps the
draw list single-pass.

## Consequences
### Positive
- Verified: unit tests (all 9 anchors, resize, DPI 1/1.25/2, order, camera independence, pixel-aligned screen text); a
  GPU test (a screen-space square covers exactly its window pixels over a layer-100 world quad, with two cameras;
  fails when screen space is ignored); Xvfb: the sandbox HUD crops are pixel-identical at three cameras and the corner
  square follows a resize.
### Negative
- Screen-space content cannot be placed between world layers (always on top).
- No UI widgets yet (hit testing is the game's job via the helpers).

## Revisit Conditions
Minimaps or split screen (several cameras), world-anchored UI (labels following entities), or a UI widget layer.

---

# ADR-030: Audio: `cpal` + `hound` + an own mixer; sound effects through `Context`; silence when there is no device

## Status
Accepted (2026-10-07, PP-022, part 1: one-shot sound effects). Defers PD-06 (generic handles) again. Extended by
ADR-032 (playback control, PP-024a) and ADR-033 (OGG Vorbis, PP-024b).

## Context
PurplePie had no sound. Games need at least short sound effects (hits, clicks, jingles), loaded like other assets and
played from game code without blocking, on Windows, macOS and Linux, and a game must keep running when a machine has no
audio device (CI, servers, Cowork's container).

## Decision
- **Crates (measured 2026-10-07 with `cargo tree -e normal --target …`):** `cpal` 0.18.2 (device output; Apache-2.0;
  default features) + `hound` 3.5.1 (WAV decoding; Apache-2.0): **+5 crates on Linux (128 → 133), +3 on Windows
  (103 → 106), +9 on macOS (99 → 108)**. Compared: `rodio` 0.22.2 with only `playback` + `wav` +20/+18/+22 (and its
  decoders come from `symphonia`, MPL-2.0); `kira` 0.12.5 +28/+25/+29. `cpal` is used only in `src/audio/output.rs`,
  `hound` only in `src/audio/sound.rs`.
- **Linux builds need the ALSA headers** (`libasound2-dev`, through `alsa-sys`): CI installs them in both Linux jobs;
  developers on Linux install them once (DEVELOPMENT §9). Windows (WASAPI) and macOS (CoreAudio) need nothing extra.
- **Own mixer** (`audio/mixer.rs`, pure code): up to 32 voices (the oldest stops first), per-voice volume 0..4,
  linear-interpolation resampling to the device rate, mono → every channel, stereo → left/right (averaged for mono
  outputs), output clamped to −1..1. Unit-tested without a device.
- **Threading:** the game sends `Play` commands over an `mpsc` channel; the `cpal` callback owns the mixer, drains the
  channel with `try_recv`, mixes into a reused `f32` buffer and converts to the device format (f32/i16/u16/i32). No locks
  on the audio thread; sound data is shared as `Arc<SoundData>` (the store keeps a reference, so the audio thread never
  frees memory).
- **API:** `audio::SoundId`; `Context::load_sound(path) -> Result<SoundId>` (asset root, decode now, `Error::Asset`,
  dedup by path); `Context::play_sound(id, volume)`; `Context::audio_available()`; `EngineConfig::audio` /
  `with_audio(bool)` (default `true`). The runner opens the default output device at startup; **any failure is logged
  and leaves a silent output** — loading still works, playing does nothing, the game never errors or panics for audio.
- **Formats:** WAV only (PCM 8/16/24/32-bit, 32-bit float; mono/stereo; any rate). Music/streaming, OGG and spatial audio
  are later tasks. *(OGG Vorbis since PP-024b, ADR-033.)*
- **PD-06 (generic handles):** sounds get their own `SoundId` like `TextureId`/`FontId`. Three handle types with the same
  shape are still simple; a generic `Handle<T>` is deferred until unloading or hot reload needs a shared store.

## Alternatives Considered
- **`rodio`:** convenient (sinks, many decoders), but 4× the dependencies for the same feature here, MPL-2.0 decoders,
  and its mixer is harder to verify sample-for-sample.
- **`kira`:** excellent for games (tweens, clocks, tracks), but the largest tree; worth revisiting for music features.
- **Opening the device lazily on first `play_sound`:** hides startup cost but makes the first sound late and the
  failure surface unpredictable.
- **Failing `Engine::run` when no device exists:** would break CI and headless use for a non-essential feature.

## Rationale
The smallest stack that plays sound on all three platforms, keeps the audio thread lock-free, and makes the part that
matters (mixing) exactly testable.

## Consequences
### Positive
- Verified in Cowork: unit tests (decoding, store, mixer: channels, resampling, clamping, voice limit, invalid
  volumes; `Context` without a device); end to end through ALSA's `file` plugin (2 ch, 48 kHz, f32): one click in the
  sandbox produced exactly the expected resampled blip at volume 0.8 in both channels (max difference 0.0 over 2,880
  frames). Without a device the sandbox logs `audio disabled` and runs normally; Breakout's autoplay results are
  identical with an (ALSA null) device and without one.
### Negative
- Linux developers and CI need `libasound2-dev`; libasound prints its own errors to stderr when no device exists.
- Windows and macOS audio are compiled only by CI and not heard by anyone yet (owner check requested).
- No music streaming, no looping, no stopping individual sounds yet.

## Revisit Conditions
Music/looping/stop control (consider `kira`), compressed formats (OGG via `lewton`), latency complaints, or a fourth
asset kind (generic handles, PD-06).

---

# ADR-031: UI interaction: a `ui` module with a data-only `Button`, a `Pointer` snapshot and a game-called system

## Status
Accepted (2026-10-07, PP-023). Builds on ADR-029 (screen space) and ADR-024 (input).

## Context
Screen-space drawing exists (PP-021), but every menu or HUD button needs the same hit testing and the same
press/release rules, which are easy to get subtly wrong (press on one button, release on another; overlapping buttons;
focus loss while held).

## Decision
- **New public module `ui`** (depends on `ecs`, `math`, `input`, `render`; nothing depends on it).
- **`ui::Button { size, .. }`** component: a rectangle of `size` logical pixels centred on the entity's `Transform2D`
  (scale applies, rotation is ignored), in the entity's `ScreenSpace`. Read-only state: `is_hovered`, `is_pressed`
  (pressed on it, still held, cursor over it) and `clicked` (press **and** release on it; true for one update). Buttons
  without `ScreenSpace` and `Hidden` buttons never react. **Visuals stay the game's choice** (any quad/sprite/text).
- **`ui::Pointer`** is a `Copy` snapshot of the cursor and the left mouse button (`Pointer::from_input(ctx.input())`),
  so it can be read before borrowing the world mutably (the ADR-026 F4 pattern).
- **`ui::update_buttons(&mut World, Pointer, viewport)`** — called by the game (like `integrate_velocity` and
  `advance_animations`), normally once per `update`. Only the **topmost** button under the cursor reacts: highest
  `Layer`, then the most recently spawned entity. A release without a seen edge (focus loss) disarms.

## Alternatives Considered
- **Immediate-mode UI (`if ui.button("Reset") { … }`)**: compact, but needs a UI context, layout and its own drawing;
  that is the editor/debug-overlay phase (P4, probably egui), not in-game UI.
- **Callbacks or events on click:** closures in components are awkward with hecs and determinism; polling `clicked()`
  is simpler and testable.
- **All overlapping buttons react:** a dialog over a button would click both.

## Rationale
The smallest piece that removes repeated, error-prone code while keeping the engine free of a widget toolkit.

## Consequences
### Positive
- Verified: 9 unit tests (edges, one-update clicks, quick click, drag off/onto, unseen release, topmost by layer and
  age, hidden/unanchored, anchors + scale + window size, `Pointer::from_input`); Xvfb XTEST in the sandbox: idle →
  hover → pressed colours exact, one click resets the camera (`reset button clicks: 1`, camera back to (0,0)×1), press
  on the button and release outside does not click, and clicks on the button never stamp the world.
### Negative
- No keyboard focus/navigation, no layout, no text input, no disabled state yet.
- Hit areas are axis-aligned (rotation ignored).

## Revisit Conditions
Menus with keyboard/gamepad navigation, many widgets needing layout, or the editor/debug overlay (P4).

---

# ADR-032: Audio playback control: playback ids, looping and volume as mixer commands

## Status
Accepted (2026-10-07, PP-024a). Extends ADR-030. OGG Vorbis decoding (music files) followed in PP-024b (ADR-033).

## Context
PP-022 could only fire sounds. Music and ambience need a sound that loops seamlessly, can be stopped, and can change
volume while playing; games also want one master volume (options menus, pausing).

## Decision
- **`audio::PlaybackId`**: `Context::play_sound` now **returns** an id (callers that ignore it keep compiling), and
  `Context::loop_sound(sound, volume)` starts a looping playback. `stop_sound(id)`, `set_sound_volume(id, volume)`,
  `stop_all_sounds()`, `set_master_volume(v)` / `master_volume()`, and `sound_duration(sound)` complete the API.
  Ids come from a counter on the game side (so they exist without a device too); an id of a playback that already
  ended is harmless.
- **Everything is a command** on the existing lock-free channel (`Play`, `Stop`, `SetVolume`, `SetMaster`, `StopAll`),
  handled by the pure `Mixer::apply`, so the whole behaviour is unit-tested without a device.
- **Looping** wraps the source position and interpolates across the seam (last frame → first frame), so a loop whose
  samples are periodic plays without a click. **Volumes change instantly** (no ramp).
- **Voice limit (32):** the oldest one-shot is dropped first; loops only when every voice loops. A silent volume now
  still starts a voice (it can be raised later).
- **No "is it still playing?" query yet:** that would need a channel back from the audio thread; `sound_duration` lets
  games estimate one-shots, and loops play until stopped.

## Alternatives Considered
- **A separate `play_music` API with one music slot:** simpler for one track, but cross-fades and ambience layers need
  several looping playbacks anyway.
- **Shared state (`Arc<Mutex<…>>`) to query playbacks:** would put a lock on the audio thread.
- **Volume ramps:** avoid clicks on abrupt changes, but add per-voice state and timing; deferred until needed.

## Rationale
The smallest extension of ADR-030 that covers music and options-menu needs, keeping the audio thread lock-free and the
logic exactly testable.

## Consequences
### Positive
- Verified: mixer unit tests (seamless loop across buffers, interpolation across the seam, stop, volume change,
  master volume, stop-all, unknown ids, loop survives the voice limit) and `Context` calls without a device; end to end
  through ALSA's `file` plugin, the sandbox's `M` loop matched the model sample for sample for 3,522,960 frames
  (73.4 loops) and was exactly silent after the second `M`.
### Negative
- Abrupt volume changes and stops can click (no fades yet).
- WAV only until PP-024b, so music files were large (the 1 s test loop is 44 KB as WAV, 4.9 KB as OGG; ADR-033).

## Revisit Conditions
Fades/cross-fades, "finished" events or queries, or many simultaneous long sounds (streaming instead of decoding fully).

---

# ADR-033: OGG Vorbis decoding with `lewton`, decoded fully on load

## Status
Accepted (2026-10-07, PP-024b). Extends ADR-030 (formats) and ADR-032 (music).

## Context
Uncompressed music is about 10 MB per minute (44.1 kHz mono 16-bit; twice that in stereo). Games ship music and long
ambience as OGG Vorbis. ADR-030 left OGG out to keep PP-022 small; ADR-032 made music possible but WAV-only.

## Decision
- **Decoder: `lewton` 0.10.2** (pure Rust, MIT OR Apache-2.0) with its default `ogg` feature. Measured with
  `cargo tree -e normal --target …`: **+4 crates on every platform** (`lewton`, `ogg` 0.8.0 BSD-3-Clause,
  `tinyvec` 1.13.3 Zlib/Apache-2.0/MIT, `byteorder` 1.5.0 Unlicense/MIT): Linux 133 → 137, Windows 106 → 110,
  macOS 108 → 112. Used only in `src/audio/sound.rs`, like `hound`.
- **The format comes from the content:** `load_sound` reads the file and looks at its first four bytes: `RIFF` → WAV
  (`hound`), `OggS` → OGG Vorbis (`lewton`), anything else → `Error::Asset` "not a WAV or OGG Vorbis file". A file's
  extension is never consulted, so a misnamed file still loads.
- **Decoded completely on load** into the same `SoundData` (interleaved `f32`, clamped to −1..1) that WAV uses, so the
  mixer, looping, volumes and the voice limit are unchanged. Mono and stereo, any sample rate; more channels are
  rejected like WAV. Chained OGG streams are joined only if every stream keeps the first one's channel count and
  sample rate (otherwise `Error::Asset`); an OGG with no audio is an error.
- **New asset `assets/sounds/loop.ogg`:** `loop.wav` encoded by ffmpeg/libvorbis at quality 4 (bit-exact flags, no
  metadata), 4.9 KB instead of 44 KB. It decodes to exactly the source's 22,050 frames, so it still loops seamlessly;
  the sandbox's `M` loop now plays it. `loop.wav` stays as the reference the unit test compares against.

## Alternatives Considered
- **`symphonia`** (many formats incl. MP3/FLAC): MPL-2.0 and a larger tree; more than one format needs today.
- **`rodio` / `kira`** decoders: bring their whole playback stacks (ADR-030 rejected them for size).
- **Streaming decode** (decode while playing): keeps memory flat for long tracks, but adds a decoder per voice on the
  audio thread or a feeder thread plus buffering. Not needed until a game has several minutes of music loaded at once.
- **Choosing by extension:** simpler, but fails on misnamed files and says nothing useful about broken ones.

## Rationale
The smallest pure-Rust, permissively licensed Vorbis decoder; decoding up front reuses the whole verified mixer path
and keeps the audio thread free of decoding work.

## Consequences
### Positive
- Verified: unit tests decode the shipped OGG (format and exact length equal to `loop.wav`, RMS difference 0.0019
  against a 0.32 peak), detect formats by content (an OGG named `.wav` loads), and reject broken OGG files (magic only,
  garbage, cut headers, damaged pages) as `Error::Asset`; `Context::load_sound("sounds/loop.ogg")` reports 1.0 s. End to
  end through ALSA's `file` plugin, the sandbox's `M` loop matched the mixer model applied to `lewton`'s output sample
  for sample (max diff 0) for 6,290,400 frames (131 loops), both channels equal, silent after the stop. A scratch check
  (not shipped) decoded a stereo file (left 440 Hz, right silent: right channel exactly 0), rejected a 3-channel file,
  joined a chained file and rejected a chain that changes the sample rate.
### Negative
- Memory: a decoded sound takes about 10 MB per minute of 44.1 kHz mono (`f32`), twice that in stereo, and decoding
  happens on the calling thread during `load_sound`. Measured for `loop.ogg` (1 s, 22.05 kHz mono): 9.9 ms in a
  debug build, 1.7 ms in release, so roughly 4× that per second of 44.1 kHz stereo (a 3-minute track: ~7 s debug,
  ~1.2 s release). Load music during startup or a loading screen.
- `lewton` does not drop a stream's leading samples when its first granule position asks for it (ffmpeg does), so
  some files decode a few milliseconds longer than other players play them. Files encoded like `loop.ogg` are exact.
  At the join of a chained file, `lewton` drops the first packet of the next stream.
- `lewton`'s last release is from 2021 (stable, but slow-moving); `ogg` is BSD-3-Clause (permissive; keep its notice
  when redistributing binaries, like the other dependencies' licences).

## Revisit Conditions
Long music tracks or many loaded at once (stream instead), MP3/FLAC requests (consider `symphonia`), or a `lewton`
bug or security advisory.

---

# Pending Decisions

PD-01 (color space) was resolved by ADR-015 and PD-04 (logging) by ADR-016, both on 2026-09-30. The core of PD-02 (coordinates) was resolved by ADR-018 on 2026-10-01. PD-05 (batching) and PD-08 (draw order) were resolved by ADR-021, PD-02 (camera) by ADR-022 and PD-07 (license) by ADR-023, all on 2026-10-01. PD-03 (input) was resolved by ADR-024 on 2026-10-02. The file-location part of PD-06 was resolved by ADR-025 on 2026-10-06; fonts, the second asset kind, follow the texture pattern with their own `FontId` (ADR-027), and sounds, the third, with `SoundId` (ADR-030).

These questions have a proposed direction but have **not** been decided. Each
one is resolved (and becomes an ADR) inside the task listed.

| ID | Question | Proposed direction | Decide in |
|---|---|---|---|
| PD-06 | Generic asset handles and unloading (textures: ADR-020; where files are found: ADR-025; fonts: ADR-027; sounds: ADR-030) | Generalize `TextureId`/`FontId`/`SoundId` into a typed `Handle<T>` + store when unloading or hot reload needs a shared store (deferred again by ADR-030) | Not scheduled: the first task that needs unloading or hot reload |
