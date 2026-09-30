# PurplePie Roadmap

Every stage leaves the repository compiling, formatted, tested and documented,
and ends with a `PurplePie-stage-N.zip` archive. **Stages never start
automatically.** Each one begins only on explicit request.

Per-stage procedure: state goal → inspect repo → list files and dependencies
to change → implement only this stage → validate (`fmt --check`, `check`,
`clippy`, `test`, `build`, plus a smoke run where possible) → update docs and
[STATUS.md](STATUS.md) → archive → stop.

---

### Stage 0: Architecture & Planning ✅
* Architecture, module map, dependency rules, target API, ADRs, risks.
* Verified crate versions and a compile-and-run compatibility spike.
* Scaffold: `lib` + `sandbox` bin, no dependencies.

### Stage 1: Minimal Application
* **Adds:** `winit 0.30.13`, `thiserror 2`, optionally `env_logger` in the sandbox only (decide then).
* **Creates:** `src/error.rs`, `src/app/` (`Engine`, `EngineConfig`, `Game`, `Context` skeleton, `Runner`).
* Window from `EngineConfig` (title, size), `CloseRequested` → exit, Escape → exit
  (temporary, inside `app`), `Resized` logged, `RedrawRequested` frame hook,
  `exiting` cleanup, errors from callbacks surfaced from `Engine::run`.
* **Out:** ECS, GPU, fixed timestep.
* **Exit:** sandbox opens a window and closes cleanly. Xvfb smoke run returns exit code 0 via a timed exit hook.

### Stage 2: Time & Fixed Update
* **Adds:** nothing (std only).
* **Creates:** `src/time/` (`Time`, `FixedTimestep`).
* Frame delta, clamp (`MAX_FRAME_DT`), accumulator, `MAX_FIXED_STEPS`,
  `alpha`, `Game::fixed_update` / `Game::update` wired into the frame.
* **Exit:** unit tests for 0, 1, N, and capped step cases and backlog drop.

### Stage 3: ECS
* **Adds:** `hecs 0.11.1`, `glam 0.33`.
* **Creates:** `src/math/` (`Transform2D`), `src/ecs/` (re-exports, `Velocity`, `integrate_velocity`).
* `Context.world` becomes a real `hecs::World`; sandbox spawns an entity.
* **Exit:** tests prove spawn / insert / query / system update, with no render dependency.

### Stage 4: WGPU Initialization
* **Adds:** `wgpu 30.0.1`, `pollster 1.0.1`.
* **Creates:** `src/render/` (`Renderer`, `GpuContext`, public `Color`).
* Full init chain and acquire-result policy (see ARCHITECTURE §8), resize,
  minimize, suspend/resume, device-lost/uncaptured-error capture, purple clear.
* **Exit:** purple window. Xvfb + lavapipe smoke run.

### Stage 5: First 2D Primitive
* **Adds:** `bytemuck` (probably).
* WGSL shader in `assets/shaders/`, one pipeline, a colored quad from an ECS
  entity (`Transform2D` + a `Shape`/`Quad` component). Fix the color-space ADR.

### Stage 6: Sprite Rendering
* **Adds:** `image` (PNG only, minimal features).
* Texture upload, sampler, `Sprite` component, instanced batching of many sprites.

### Stage 7: Camera & Coordinates
* `Camera2D`, orthographic projection, world units vs pixels, resize-aware
  viewport, DPI scale factor. Fix the coordinate-system ADR.

### Stage 8: Input System
* `src/input/`: `Input`, own `KeyCode`/`MouseButton`, pressed / just_pressed /
  just_released, cursor position in window and world space. Fix the input ADR
  (edge events vs fixed steps).

### Stage 9: Assets & Resources
* `src/assets/`: typed `Handle<T>`, `Assets` store, synchronous loading from
  `assets/`, texture/shader caching, clear error reporting.

### Stage 10: Engine/Game API Refinement
* Review ergonomics with a small real game (e.g. Pong/Breakout in `examples/`).
  Finalize `Context`, docs, `#![warn(missing_docs)]`, and consider a workspace
  split ([ADR-0001](adr/0001-crate-layout.md)).

### Later (not scheduled)
Render interpolation, shapes/text/debug renderers, audio, scenes, hot reload, editor tooling.
