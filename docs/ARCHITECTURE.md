# PurplePie Architecture

Last reviewed: **2026-10-06** against the repository after PP-019 (sprite sheets).

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

**Current reality (Stages 0–10 complete; post-portfolio text phase P1, PP-018a/b):** `purplepie` provides `Engine`,
`EngineConfig` (incl. `with_asset_root`), `Game`, `Context` (incl. `load_texture`, `load_font`, `measure_text`, `asset_root`, `camera`/`camera_mut`, `viewport_size`, `input`, `cursor_world`), `Time`, `Error`, and the public modules `ecs`
(`World`, `Entity`, `Velocity`, `integrate_velocity`), `math` (`Transform2D`, `Vec2`, `Mat4`), `input`
(`Input`, `KeyCode`, `MouseButton`) and `render`
(`Color`, `Camera2D`, `Quad`, `Sprite`, `Text`, `TextAnchor`, `TextMetrics`, `TextureId`, `TextureRegion`, `SpriteGrid`, `FontId`, `Layer`, `Hidden`). Every frame the engine clears the window and draws each entity
that has `Transform2D` + `Quad` (solid colour, ADR-019), `Transform2D` + `Sprite` (textured, ADR-020) or `Transform2D` +
`Text` (glyphs from a font atlas, ADR-027), sorted by `Layer` and batched by texture (ADR-021), as seen through the one engine-owned `Camera2D` (ADR-022; default view ADR-018). The `sandbox` game shows reference quads (one moving,
two overlapping the sprite to show draw order) and two sprites from one PNG (one tinted and spinning); its camera can be
set with an env var, and the arrow keys, `=` / `-` and the mouse wheel pan and zoom it; a marker follows the cursor and clicks stamp squares
(ADR-024: keyboard and mouse); a text label lists the controls (ADR-027); four sprite-sheet cells, one mirrored, show texture regions (PP-019).

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
| `src/error.rs` | `Error` (`#[non_exhaustive]`: `InvalidConfig`, `EventLoop`, `Window`, `Surface`, `Adapter`, `Device`, `SurfaceUnsupported`, `Render`, `Asset { path, source }`, `Game`), `BoxError`, `Result`. winit/wgpu/image errors are boxed sources, not public types. | VERIFIED |
| `src/app/mod.rs` | `Engine::new` (validates config, creates the `EventLoop`) and `Engine::run` (runs the runner, returns the first error) | VERIFIED (Linux; Windows lifecycle confirmed by the owner) |
| `src/app/config.rs` | `EngineConfig`: title, size, resizable, `exit_on_escape`, `fixed_dt`, `max_frame_dt`, `max_fixed_steps`, `clear_color`, `asset_root` + builders + `validate` | VERIFIED |
| `src/assets/mod.rs` | Crate-private `AssetRoot` (`resolve`, `for_process`, `path`, `locate`): the asset root chosen once at startup (ADR-025) | VERIFIED (5 unit tests + end-to-end layouts) |
| `src/app/game.rs` | `Game` trait (`init`, `fixed_update`, `update`, all with defaults); `Context` (`world()`, `world_mut()`, `time()`, `dt()`, `load_texture(path)`, `texture_size(id)`, `camera()`, `camera_mut()`, `viewport_size()`, `input()`, `cursor_world()`, `asset_root()`, `set_window_title()`, `request_exit()`, `exit_requested()`), borrowing one `EngineState` | VERIFIED |
| `src/app/state.rs` | `EngineState` (`pub(crate)`): exit flag, `Time`, `World`, `Textures`, `Camera2D`, `Input`, viewport, `AssetRoot`, pending window title; owned by the runner. `physical_to_logical`, `sanitize_scale_factor` | VERIFIED (2 unit tests incl. cursor DPI path) |
| `src/app/keymap.rs` | winit → PurplePie input: `PhysicalKey` → `KeyCode` (bijection test), mouse buttons, wheel deltas → lines (ADR-024) | VERIFIED (5 unit tests) |
| `src/input/mod.rs` | `pub mod input`: `KeyCode` (99 physical keys), `MouseButton`, `Input` (keys: `pressed`, `just_pressed`, `just_released`, `axis`, `pressed_keys`; mouse: `mouse_pressed`, `mouse_just_pressed`, `mouse_just_released`, `cursor_position`, `scroll`); one shared `Buttons` edge implementation (ADR-024) | VERIFIED (15 unit tests + doctest, Xvfb XTEST runs) |
| `src/time/mod.rs` | `Time` (public, read-only): clamped delta, elapsed game time, frame number, `fixed_dt`, total fixed steps, `alpha` | VERIFIED |
| `src/time/fixed.rs` | `FixedTimestep` (`pub(crate)`): accumulator, step cap, backlog clamp, alpha. `std` only. | VERIFIED |
| `src/math/mod.rs` | `pub mod math`: `Transform2D { position, rotation, scale }` (+ `IDENTITY`, builders, `to_mat4()`), re-exported `glam::{Vec2, Mat4}` | VERIFIED |
| `src/ecs/mod.rs` | `pub mod ecs`: re-exports `hecs::{World, Entity}` and the `hecs` crate; `Velocity(Vec2)`; `integrate_velocity(&mut World, dt)` | VERIFIED |
| `src/render/mod.rs` | `pub mod render`: public `Color`, `Camera2D`, `Quad`, `Sprite`, `TextureRegion`, `SpriteGrid`, `Text`, `TextAnchor`, `HorizontalAnchor`, `VerticalAnchor`, `TextMetrics`, `TextureId`, `FontId`, `Layer`, `Hidden`; crate-private `Renderer`, `Textures`, `Fonts`, `measure_text` | VERIFIED |
| `src/render/camera.rs` | Public `Camera2D { position, zoom }`: `IDENTITY`, `fit`, `effective_zoom`, `screen_to_world`, `world_to_screen`, `visible_world_rect`; crate-private `view_projection` (ADR-018, ADR-022) | VERIFIED (7 unit tests + doctest, Xvfb whole-frame checks) |
| `src/render/draw.rs` | Public `Layer(i32)` and `Hidden` components (hidden entities are skipped). Crate-private `View`, `DrawList` (collect quads + sprites + text glyphs, sort by (layer, material, entity, glyph), build `Batch` runs; atlas full → one reset and re-layout), `TextPlacement` (on-screen glyph size, pixel-snapped origin) and `Material` (`Color`, `Texture`, `Glyphs`; ADR-021, ADR-027) | VERIFIED (13 unit tests + doctests, Xvfb checks) |
| `src/render/quad.rs` | Public `Quad { size, color }` component. Crate-private `QuadPipeline` (`bind`), `rect_pipeline` (pipeline builder shared with sprites; ADR-019) | VERIFIED (unit tests, 2 ignored GPU tests, Xvfb pixel checks) |
| `src/render/instance.rs` | `Instance` (96 B Pod: clip matrix + colour/tint + `uv_rect`) and `InstanceBuffer` (growable vertex buffer), shared by quads, sprites and glyphs | VERIFIED |
| `src/render/font.rs` | Public `FontId`. Crate-private `Fonts` store (parsed `ab_glyph::FontArc` in load order, path → id cache; owned by the runner) and `decode_font` (ADR-027) | VERIFIED (unit tests) |
| `src/render/text.rs` | Public `Text { content, font, size, color, anchor }` component, `TextAnchor` (+ `HorizontalAnchor`, `VerticalAnchor`), `TextMetrics` (+ `bounds`). Crate-private `walk` (shared pen logic), `layout` (anchored, whole-pixel glyph positions, newlines, kerning), `measure` (world-unit metrics for `Context::measure_text`) and `MAX_EM_PIXELS` (ADR-027) | VERIFIED (unit tests, 3 GPU readback tests, Xvfb) |
| `src/render/atlas.rs` | Crate-private `GlyphAtlas`: 1024² RGBA shelf-packed glyph cache (white + coverage alpha, 1-texel border, per-size cache, dirty rows, reset when full) and `px_scale` (em → ab_glyph scale) (ADR-027) | VERIFIED (unit tests) |
| `src/render/region.rs` | Public `TextureRegion { x, y, width, height }` (texels, rows down; `size`; crate-private `uv_rect` cut to the texture) and `SpriteGrid` (`cell`, `frame`, margin/spacing; ADR-020 extension, PP-019) | VERIFIED (4 unit tests + 2 doctests, GPU readback test, Xvfb) |
| `src/render/texture.rs` | Public `TextureId`. Crate-private `Textures` store (decoded RGBA8 in load order, path → id cache; owned by the runner) and `decode_png` (ADR-020) | VERIFIED (unit tests) |
| `src/render/sprite.rs` | Public `Sprite { texture, size, tint, region }` (`with_tint`, `with_region`). Crate-private `SpritePipeline` (bind group per texture, `Nearest` sampler, `sync_textures` uploads new store entries, `bind`/`bind_texture`; ADR-020) plus the GPU glyph atlas (linear sampler, `upload_glyphs` writes dirty rows, `bind_glyphs`; ADR-027) | VERIFIED (unit tests, 2 ignored GPU tests, Xvfb pixel checks) |
| `src/render/sprite.wgsl` | Sprite + glyph shader: the quad scheme plus UVs from the instance's `uv_rect` (top row = v 0) and `textureSample × tint` | VERIFIED |
| `src/render/quad.wgsl` | Quad shader (corners from `vertex_index`, per-instance clip matrix + colour), embedded with `include_str!` | VERIFIED |
| `src/render/color.rs` | `Color` (sRGB, straight alpha): constructors, `PURPLEPIE` `#6A0DAD`, `to_linear`, `to_wgpu(target_is_srgb)` (ADR-015) | VERIFIED |
| `src/render/renderer.rs` | `Renderer` (`pub(crate)`): wgpu surface/device/queue/config; `new`, `resize`, `render(&World, &Textures, &Fonts, &Camera2D, before_present)`; texture sync; owns the CPU `GlyphAtlas`; one shared instance buffer; `record_batches`; acquire-result policy; `AutoVsync` (ADR-014); fault checks (ADR-017) | VERIFIED (Linux/lavapipe incl. 2 ignored offscreen text tests; purple window confirmed on Windows by the owner) |
| `src/render/faults.rs` | `FaultSlot` (first-fault-wins `Arc<Mutex<Option<GpuFault>>>`) + `GpuFault`; installs wgpu's uncaptured-error and device-lost callbacks (ADR-017) | VERIFIED (unit tests + ignored GPU test under lavapipe) |
| `src/app/pacer.rs` | `FramePacer`: 60 Hz `WaitUntil` deadlines, no catch-up bursts. Interim until Stage 4 vsync. | VERIFIED |
| `src/app/runner.rs` | `Runner<G>`: winit `ApplicationHandler`; the only code handling winit events. Owns world, textures, camera and the event-driven logical viewport. | VERIFIED (Linux; Windows lifecycle confirmed by the owner) |
| `examples/breakout.rs` | Breakout, a complete game on the public API only (`cargo run --example breakout`); deterministic autoplay test modes. The engine needed no changes for it (PP-012) | VERIFIED (Xvfb autoplay + XTEST runs) |
| `src/main.rs` | `sandbox` binary: a `Game` using only the public API. Optional timed exit via env var. | VERIFIED (Linux; Windows lifecycle confirmed by the owner) |
| `assets/textures/sandbox_sheet.png` | 32×16 sprite sheet for the sandbox and tests: 4×2 cells of 8×8, each with its own border and inner colour and a white texel at its inner top-left (shows mirroring) | VERIFIED |
| `assets/textures/sandbox_quadrants.png` | 16×16 test image for the sandbox and unit tests (four colour quadrants, transparent border, one 50% alpha quadrant) | VERIFIED |
| `assets/fonts/Poppins-Regular.ttf`, `assets/fonts/OFL.txt` | The shipped font (unmodified, 160 KB) and its SIL Open Font License 1.1 (ADR-027); used by the sandbox and unit tests | VERIFIED |
| `assets/shaders/` | Runtime data folder (empty, `.gitkeep`) | Placeholder |

### Planned (ADR-003)

| Module | Responsibility | May depend on | Must NOT depend on | Future extension points | Stage |
|---|---|---|---|---|---|
| `error` | `Error` enum, `Result<T>` | `thiserror`; wraps winit/wgpu error types | any internal module | new variants per failure domain | 1 |
| `app` | `Engine`, `EngineConfig`, `Game`, `Context`, `Runner` (winit `ApplicationHandler`), frame orchestration | all engine modules, `winit` | — (top of the engine) | multiple windows (not planned), headless runner for tests | 1 |
| `time` | `Time` (delta, elapsed, frame count), `FixedTimestep` | `std` only | `winit`, `wgpu`, `hecs` | interpolation alpha, time scale/pause | 2 |
| `math` | `Transform2D`; re-exports `Vec2`, `Affine2`, `Mat4` | `glam` | everything internal | rect/AABB helpers when needed | 3 |
| `ecs` | Re-exports `World`, `Entity`; engine components (`Velocity`); systems (`integrate_velocity`) | `hecs`, `math` | `render`, `app`, `input`, `wgpu`, `winit` | hierarchy/parenting, command buffers | 3 |
| `render` | `pub(crate) Renderer`, pipelines, texture and font stores, glyph atlas, draw list; public data types `Color`, `Camera2D`, `Quad`, `Sprite`, `Text`, `TextureId`, `FontId`, `Layer`, `Hidden` | `wgpu`, `pollster`, `image` (PNG decode), `ab_glyph` (fonts, ADR-027), `math`, `ecs` (read-only) | `app`, `input`, `winit` (the window arrives as `Arc<dyn wgpu::WindowHandle>`, the display as `impl wgpu::wgt::WgpuHasDisplayHandle`) | `ShapeRenderer`, `DebugRenderer`, sprite animation (PP-020), text wrapping | 4–7, P2 |
| `input` | `Input` state (pressed / just_pressed / just_released, cursor, wheel), `KeyCode`, `MouseButton` | `math` | `winit` (translation lives in `app`), `render` | gamepad, text input, action mapping | 8 |
| `assets` | Today: `AssetRoot` (where relative asset paths resolve, ADR-025), used by `app` for textures and fonts. Later (PD-06): `Handle<T>`, store, unloading | `error`, `std::fs` | `render`, `input`, `winit`, `wgpu` | hot reload, async loading, embedded assets | 9 |

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

### Current (Stage 10, PP-017)
```text
main → Engine::new(config)?          validate config, EventLoop::new()
     → engine.run(game)              EventLoop::run_app(&mut Runner)
resumed          → create Arc<Window> (once) → Renderer::new (if none; error → exit)
                   → game.init(ctx) (once; error → exit). ctx.load_texture / ctx.load_font: AssetRoot::locate
                     (relative → root, chosen once in Runner::new, ADR-025) → decode/parse now → Textures / Fonts
                     (CPU, owned by the runner; outlive renderers: a new one re-uploads textures, re-rasterizes glyphs)
about_to_wait    → FramePacer: if a frame is due, request_redraw; ControlFlow::WaitUntil(next deadline)
RedrawRequested  → FRAME (skipped once exit has begun):
                     raw = now − last_frame (0 on the first frame)
                     delta = time.begin_frame(raw, max_frame_dt)       clamp to [0, max_frame_dt]
                     n = fixed.advance(delta)                          ≤ max_fixed_steps, backlog clamped
                     repeat n: input.begin_fixed_step → game.fixed_update(ctx, dt = fixed_dt)
                               → input.end_fixed_step (fixed edges consumed)   stop early on request_exit
                     time.set_alpha(fixed.alpha())
                     game.update(ctx, dt = delta)                      unless exit requested
                     apply a requested window title (set_window_title; also after init)
                     input.end_frame                                   frame edges consumed
                     renderer.render(&world, &textures, &fonts, &camera, pre_present_notify)   unless exit requested; Err → exit
                       (upload new textures → DrawList: collect quads + sprites + text glyphs (rasterizing new glyphs
                        into the atlas), sort by (Layer, material, entity, glyph) → upload dirty atlas rows + one
                        instance buffer → clear + one draw per batch)
Resized / ScaleFactorChanged → viewport = size / scale (for Context) + renderer.resize(w, h, scale_factor)
                               0×0 = minimized: skip rendering. The window is never queried per frame (ADR-022).
KeyboardInput   → Escape (logical, if exit_on_escape) → exit; else keymap::translate(physical) → input.key_down/up
                  (repeats and X11 synthetic presses ignored). Focused(false) → input.release_all (keys + buttons)
MouseInput / CursorMoved / CursorLeft / MouseWheel → input (cursor: physical ÷ tracked DPI scale → logical px;
                  CursorLeft → None; wheel → lines). Context::cursor_world = camera.screen_to_world(cursor, viewport)
CloseRequested / ctx.request_exit() → event_loop.exit()
suspended        → drop Renderer (recreated in resumed; world and time are kept)
exiting          → drop Renderer, then Window
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
        E->>R: render(&world, &textures) → queue.present
        E->>E: input.end_frame
    end
    W->>E: CloseRequested → exit
    W->>E: exiting → drop renderer, then window
    E-->>M: Result<()>
```

Fixed-step constants: `FIXED_DT = 1/60 s`, `MAX_FRAME_DT = 0.25 s`,
`MAX_FIXED_STEPS = 5`, backlog clamped to below one step when the cap is hit.
All stages use `ControlFlow::WaitUntil` with a 60 Hz redraw cap, plus `PresentMode::AutoVsync` from Stage 4 (ADR-014, which supersedes the original plan of switching to `Poll` + `Fifo`).

---

## 6. Engine/Game Boundary

| Game code **may** use | Game code **may not** see |
|---|---|
| `Engine`, `EngineConfig`, `Game`, `Context`, `Result`/`Error`/`BoxError` | `wgpu::{Device, Queue, Surface, RenderPipeline, …}` |
| `World`, `Entity`, components (`Transform2D`, `Velocity`, `Sprite`, `Text`, …) | `winit::window::Window`, raw winit events |
| `Time`, `Input`, `KeyCode`, `MouseButton`, `Color`, `TextureId`, `FontId`, `Text`, `TextAnchor`, `TextMetrics`, `Layer`, `Hidden`, `Camera2D`, `Handle<T>` (planned) | `Renderer`, `Runner`, `Textures`, `Fonts`, `GlyphAtlas` (`pub(crate)`), `image` and `ab_glyph` types |
| built-in system functions (`ecs::integrate_velocity`) | engine-internal state outside `Context` |

The engine runs no gameplay systems implicitly. Order is visible in the game's `fixed_update`.

---

## 7. Rendering Architecture

**Current (Stages 4–6; ADR-005, ADR-009, ADR-014, ADR-015, ADR-017, ADR-018, ADR-019, ADR-020, ADR-021):** a single
crate-private `Renderer` that clears the window to `EngineConfig::clear_color`
every frame, then draws all `(Transform2D, Quad)` and `(Transform2D, Sprite)` entities from one sorted draw list,
one draw call per (layer, material) run (see "Quads and coordinates", "Sprites and textures" and "Draw order and batching" below). It is verified pixel-exact under Xvfb + lavapipe, and the owner confirmed the
same purple on Windows with a real GPU. The chain below is implemented as
shown, with `SurfaceTarget::from_window_without_display` because the display
handle goes through the `InstanceDescriptor`. Right after `request_device`,
PurplePie's `FaultSlot` replaces wgpu's panicking uncaptured-error handler
and the device-lost callback.

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
| `Lost` | **fatal:** `Error::Render(SurfaceLost)`. No recreation, because it panics in wgpu-hal 30.0.1 when the window is gone (ADR-017) |
| `Validation` | **fatal:** `Error::Render` with the recorded uncaptured error (or `AcquireValidation`) |
| Uncaptured wgpu error (validation / OOM / internal) | recorded by `FaultSlot`, checked before and after each frame and after init, becomes `Error::Render`, clean exit |
| Device lost, `Unknown` | same as above. `Destroyed` (self-inflicted) is ignored |
| size 0×0 (minimized) | never configure. Skip rendering. |

### Quads, coordinates and camera (Stages 5, 7)

```text
world (ADR-018): +X right, +Y up; default camera: origin = window centre, 1 unit = 1 logical px
camera (ADR-022): visible world = camera.position ± logical_size / 2 / zoom   (one engine-owned Camera2D)
logical size = physical size / scale_factor     resize → more world visible, same zoom
screen (cursor) space: logical px, top-left origin, +Y down; screen_to_world / world_to_screen are inverses
per quad (CPU): clip_from_local = camera.view_projection(logical) × Transform2D::to_mat4() × scale(size)
                color = Color::to_wgpu(surface is sRGB)            (ADR-015)
GPU: one pipeline, no vertex buffer, no bind group. draw(0..6, 0..n). Corners from vertex_index.
```
Draw order: see "Draw order and batching" below (ADR-021). There is no MSAA,
and alpha blending is straight alpha.

### Sprites and textures (Stage 6, ADR-020)

```text
game:  id = ctx.load_texture(path)?        relative → asset root (ADR-025); read + decode PNG now → Error::Asset on failure
       spawn((Transform2D, Sprite::new(id, size).with_tint(c)))      optional .with_region(TextureRegion | SpriteGrid::frame(i))
store: Textures (runner-owned, CPU): RGBA8 sRGB straight alpha, load order = TextureId, never unloaded
frame: sprites.sync_textures(store)        upload entries ≥ uploaded count; > max_texture_dimension_2d → Error::Asset
       texture format = Rgba8UnormSrgb if surface is sRGB else Rgba8Unorm   (same blend space as quads, ADR-015)
       per sprite: Instance { clip_from_local, tint, uv_rect }  (same 96 B layout as quads)
                   uv_rect = whole texture, or region / texture size (cut to the texture; nothing left → not drawn)
       consecutive sprites with the same texture → one draw(0..6, range) with that texture's bind group
       sampler: Nearest, ClampToEdge, no mipmaps.   UV: v = 0 at the top edge (image row 0)
order: see "Draw order and batching" (ADR-021); regions of one texture share its batch
mirror: negative Transform2D scale (no flip flags; culling is off)
```

### Text (post-portfolio, ADR-027)

```text
game:  font = ctx.load_font("fonts/Poppins-Regular.ttf")?   asset root; read + parse now (ab_glyph) → Error::Asset on failure
       spawn((Transform2D at the left end of the baseline, Text::new("Score: 0", font, 32.0).with_color(c)))
store: Fonts (runner-owned, CPU): FontArc per FontId, never unloaded
frame: per Text (Without<Hidden>): M = view_projection × transform
         k = physical px per local unit (larger axis of M × viewport/2);  em_px = size × k  (skip if ≤ 0, NaN, > 512)
         origin → nearest physical pixel corner (clip-space nudge)
         layout: pass 1 = line widths (scratch Vec reused); line shift = round(−width × 0|½|1) (TextAnchor horizontal),
                 block shift = round(0 | ascent | (ascent − below)/2 | −below) (vertical; below = (lines−1)·L + descent)
                 pass 2 = pen x rounded per glyph, '\n' → next line (L = round(ascent − descent + gap)), kern table
         glyph bitmap from GlyphAtlas (1024² RGBA, key = font, glyph, em in 1/64 px; rasterized on first use)
         Instance { M × translate(centre/k) × scale(size/k), colour, uv_rect = atlas texels / 1024 }
       atlas full mid-frame → reset once, lay out this frame's text again (skip what still doesn't fit)
       dirty atlas rows → queue.write_texture; drawn by the sprite pipeline with the atlas bind group (Linear sampler)
order: material rank Glyphs = u32::MAX → after quads and sprites in each layer
game:  ctx.measure_text(&text) → TextMetrics (world units, unrounded; camera-independent); metrics.bounds(anchor)
```

### Draw order and batching (Stage 6, ADR-021)

```text
key per drawable = (Layer (default 0), material rank (quad = 0, sprite = 1 + TextureId, text = u32::MAX), entity index, glyph index)
DrawList::build(world, view, &Textures (sizes for regions), &Fonts, &mut GlyphAtlas):
                 query (Entity, Transform2D, Quad | Sprite | Text, Option<Layer>), each Without<Hidden>
                 → sort_unstable_by_key (keys are unique → deterministic) → instances in draw order
                 → batches = runs of equal (layer, material)
record_batches:  one shared instance buffer bound once; pipeline switched only when the material kind changes;
                 texture (or glyph atlas) bind group per batch; draw(0..6, batch range) per batch
draw calls     = number of (layer, material) runs; painter's order, no depth buffer
```
The entity-index tie-breaker keeps the order independent of hecs query/archetype order. It is deterministic
but not part of the API: overlapping drawables whose order matters should use different layers.

Evolution, each part added only when a stage needs it:
`Renderer { quads (Stage 5 ✅), sprites + textures (Stage 6 ✅ PP-008), layers + batching (Stage 6 ✅ PP-015), camera (Stage 7 ✅ PP-009), text (PP-018a ✅), debug (later) }`.

---

## 8. ECS Architecture

**Current (Stage 3, ADR-006, ADR-008):**
- **Implementation:** `hecs 0.11.1`, archetypal storage.
- **World ownership:** exactly one `hecs::World`, owned by the runner and lent to
  the game in every callback through `ctx.world()` / `ctx.world_mut()`. Because
  `world_mut()` borrows the context exclusively, read `ctx.dt()` into a local first.
- **Components so far:** `math::Transform2D`, `ecs::Velocity`, `render::Quad`, `render::Sprite`, `render::Text`, `render::Layer` and `render::Hidden` (read by the renderer). `Sprite` holds a plain `TextureId` and `Text` a plain `FontId`, never a GPU object.
- **Systems so far:** `ecs::integrate_velocity(world, dt)`, called by the sandbox from `fixed_update`.
- **Components:** plain data structs. The first set is `Transform2D`, `Velocity`, then `Sprite`.
  No wgpu handles in components.
- **Systems:** free functions, e.g. `fn integrate_velocity(world: &mut World, dt: f32)`,
  called explicitly by the game. There is no scheduler.
- **Resources:** hecs has none. Engine-wide state (`Time`, `Input`, `Camera2D`, textures; later
  `Assets`) lives in the runner's `EngineState` and is reached through `Context`. Game-specific state lives in the `Game` struct.
- **Structural changes during iteration:** `hecs::CommandBuffer` or collect-then-apply.

---

## 9. Error Handling

**Current:** `purplepie::Error` with boxed `source`s. `Error::game(e)` wraps game errors.
Errors raised inside winit callbacks are stored by the runner (first one wins),
followed by `event_loop.exit()`, and returned from `Engine::run`. The sandbox
prints the full `source()` chain. GPU setup failures map to `Error::{Surface,
Adapter, Device, SurfaceUnsupported}`. GPU runtime faults map to `Error::Render`
(ADR-017) and end the loop the same way. Diagnostics use the `log` facade
(ADR-016). The engine never installs a logger, and the sandbox has a built-in stderr
logger controlled by `PURPLEPIE_LOG` (default `warn`).
The lints `unsafe_code = "forbid"` and `clippy::unwrap_used = "warn"` apply. `src/` contains no `unwrap`, and `expect` is used only in tests.

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
| `error`, `app::{config, game, pacer}`, `time`, `math`, `ecs`, `render::{Color, camera, faults, quad, instance, texture, region, font, atlas, text, sprite, draw}`, `input`, `app::{keymap, state}`, `assets` | pure unit tests + doctests | ✅ 149 unit tests + 21 doctests (PP-019) |
| GPU-dependent code paths (`FaultSlot`, quad and sprite pipelines, texture upload and size limit, shader errors on a real device, offscreen text and sprite-sheet rendering read back and compared with the CPU rasterization / texels) | `#[ignore]` tests, run with `cargo test -- --ignored` where a GPU/lavapipe exists (not in CI) | ✅ 9 ignored tests pass under lavapipe |
| Rendered output | Xvfb screenshots analysed per pixel (`docs/DEVELOPMENT.md` §8): exact rectangles, colours, texels, alpha blends, motion, resize behaviour | ✅ Stages 5–6 |
| Every push | GitHub Actions `.github/workflows/ci.yml`: fmt + clippy (Linux); `cargo check` + `cargo test` on Linux, Windows, macOS | ✅ first run all green (owner-reported, 2026-10-01) |
| `input` state | pure unit tests, no window/GPU; Xvfb XTEST key, click, move and wheel runs | ✅ keyboard (PP-010) + mouse (PP-016) |
| Game logic | build `World`/`Time`/`Input` headless, call game methods | planned (Stage 3+) |
| `app` runner, `render::Renderer` | Xvfb + lavapipe smoke runs in Cowork (xdotool XTEST keys, `WM_DELETE_WINDOW` close, `xwininfo`, CPU sampling, screenshot color histograms); Windows by the owner | ✅ Stages 1–6 on Linux; Windows pending |
| Every change | `cargo fmt --check`, `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build` | ✅ in use |

Tests live next to code in `#[cfg(test)] mod tests`. Integration tests go in `tests/`
once the public API has behavior worth testing.
