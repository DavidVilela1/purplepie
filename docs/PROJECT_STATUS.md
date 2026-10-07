# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**M10: Engine API Stabilization: VERIFIED (Linux).** All milestones M0–M10 are reached: the portfolio scope is complete.
A complete game (Breakout) runs on the reviewed public API (ADR-026). M5–M10 have not been looked at on Windows yet
(M1 and M4 are owner-confirmed there; CI builds and tests on Linux, Windows and macOS).

## Current Stage

**Post-portfolio phase P2 (runtime essentials): complete** (2026-10-07, verified on Linux): PP-019 sprite sheets,
PP-020 sprite animation, PP-021 screen-space drawing, PP-022 sound effects, PP-023 UI buttons, PP-024a playback
control, PP-024b OGG Vorbis decoding and PP-025 per-texture sampling. Stages 0–10 and phase P1 (Text) were completed
earlier. **Phase P3 (editor foundations) in progress:** PP-026a scene files (RON format, drawing components;
ADR-035) done; **PP-026b** (animation, velocity and UI buttons in scenes) is next.
The long-term plan (in-game UI and an editor) is in
[ROADMAP.md](ROADMAP.md#after-stage-10).

## Overall State

PurplePie opens a window through its own `Engine`/`Game` API and draws game
state. Every entity with `Transform2D` + `render::Quad` is drawn as a solid-colour
rectangle and every entity with `Transform2D` + `render::Sprite` as a textured rectangle, from one
draw list sorted by the optional `render::Layer` component and batched by texture (ADR-021); entities with `Transform2D` +
`render::Text` are drawn as glyphs from a font loaded with `Context::load_font`, rasterized by `ab_glyph` at their
on-screen size into one glyph atlas and pixel-aligned, anchored by `render::TextAnchor` and measurable with
`Context::measure_text` (ADR-027), all as seen through one
engine-owned `render::Camera2D` that games pan and zoom via `Context::camera_mut()` (ADR-022).
Keyboard and mouse state come through `Context::input()` with engine-owned `KeyCode` / `MouseButton`; every press,
click and wheel movement reaches `fixed_update` and `update` exactly once, and `Context::cursor_world()` maps the
cursor through the camera (ADR-024). Games load PNGs with `Context::load_texture`, which returns a plain
`TextureId` or a typed `Error::Asset`; the renderer uploads textures itself (ADR-020). World coordinates are +X right, +Y up,
origin at the window centre, and 1 unit = 1 logical pixel (ADR-018). Game logic
runs in a 60 Hz fixed-timestep `fixed_update` plus a per-frame `update` over one
engine-owned `hecs::World`. GPU faults end the loop cleanly as `Error::Render`.
The engine logs via `log`. CI passes on Linux, Windows and macOS. Licensed MIT OR Apache-2.0.
Relative asset paths resolve against an asset root: `EngineConfig::with_asset_root`, else `assets/` next to the
executable, else `assets/` in the working directory (ADR-025).

| Component | State | Notes |
|---|---|---|
| Crate layout (`purplepie` lib + `sandbox` bin) | VERIFIED | ADR-002 |
| Lints (`unsafe_code = forbid`, `unwrap_used = warn`) | VERIFIED | No `unwrap`/`unsafe` in `src/`. `expect` only in tests. |
| `error` (`Error`, `BoxError`, `Result`) | VERIFIED | 3 unit tests + 1 doctest. `Error::Asset` added (PP-008), covered by texture tests. |
| `app::EngineConfig` | VERIFIED | 8 unit tests + 1 doctest (`asset_root` added in PP-011) |
| `app::Game` / `Context` | VERIFIED | 11 unit tests. `set_window_title` (PP-017). `asset_root`, relative `load_texture` (PP-011). `cursor_world` (PP-016). `load_texture`, `texture_size` (PP-008); `camera`, `camera_mut`, `viewport_size` (PP-009); `input` (PP-010). Borrows one `EngineState`. |
| `app` frame pacing (`FramePacer`) | VERIFIED | 5 unit tests. 60 Hz redraw cap (ADR-014). |
| `app::Engine` + runner (winit 0.30 lifecycle) | VERIFIED | Xvfb runs pass. Passes the DPI scale factor to the renderer (also on `ScaleFactorChanged`). Owns the camera and an event-driven logical viewport (PP-009). Windows: window, Escape and close confirmed by the owner. |
| `time` (`Time`, `FixedTimestep`) | VERIFIED | 12 unit tests |
| `math` (`Transform2D`, `Vec2`, `Mat4`) | VERIFIED | 5 unit tests + 2 doctests. `to_mat4()` added. |
| `ecs` (`World`, `Entity`, `Velocity`, `integrate_velocity`) | VERIFIED | 4 unit tests + 2 doctests |
| `render::Color` | VERIFIED | 4 unit tests + 1 doctest (ADR-015) |
| `render::Quad` + quad pipeline | VERIFIED (Linux) | 6 unit tests + 1 doctest + 2 ignored GPU tests. Xvfb pixel-exact (ADR-018/019). |
| `render::Camera2D` | VERIFIED (Linux) | 7 unit tests + 1 doctest (incl. DPI 1.0/1.25/2.0). Xvfb whole-frame checks (ADR-022). |
| `render::Layer` + draw list (`render/draw.rs`) | VERIFIED (Linux) | 13 unit tests + 2 doctests (incl. text ordering, pixel snapping at zoom/DPI, atlas overflow). Xvfb overlap checks, 0 mismatches (ADR-021, ADR-027). |
| `render::instance` (`Instance`, `InstanceBuffer`) | VERIFIED | 1 unit test. Shared by quads, sprites and glyphs; 96 B with `uv_rect` since PP-018a. |
| `render::Text` + `TextAnchor` + `TextMetrics` + `FontId` + font store + glyph atlas (`text.rs`, `font.rs`, `atlas.rs`) | VERIFIED (Linux) | 19 unit tests (+1 in `app::game` for `measure_text`) + 1 doctest (compile-only) + 3 ignored GPU tests (readback = CPU raster; anchored text inside measured bounds). Xvfb sandbox label, Breakout HUD (ADR-027). |
| `render::TextureId` + texture store + PNG decode | VERIFIED | 9 unit tests (ADR-020) |
| `render::Sprite` + sprite pipeline | VERIFIED (Linux) | 2 unit tests + 1 doctest (compile-only) + 2 ignored GPU tests. Xvfb pixel-exact. (The batching test moved to `draw.rs`.) |
| `render::Renderer` (wgpu) | VERIFIED | Linux: pixel-exact quads, sprites and text, resize, unmap/map, fault exits, texture upload/sync. Windows: purple window confirmed (Stage 4). |
| `render::faults` (`FaultSlot`, `GpuFault`) | VERIFIED | 4 unit tests + 1 ignored GPU test |
| CI workflow | VERIFIED (owner-reported) | fmt + clippy (Linux); check + test on Linux/Windows/macOS: first run all green, 2026-10-01 |
| `input` (`Input`, `KeyCode`, `MouseButton`) + `app::{keymap, state}` | VERIFIED (Linux) | 15 + 5 + 2 unit tests + 1 doctest; Xvfb XTEST key/mouse runs (ADR-024). |
| `examples/breakout.rs` | VERIFIED (Linux) | Complete game on the public API; deterministic autoplay (win: 7135 steps, score 220; lose: 892 steps) and XTEST paddle/launch/restart checks (PP-012). Uses `Hidden`, `Camera2D::fit` and the window title since PP-017. |
| `assets` (`AssetRoot`) | VERIFIED (Linux) | 5 unit tests + end-to-end launch layouts (ADR-025). Generic handles/unloading deferred (PD-06). |

## Completed

- PP-000: architecture, verified stack, compatibility spike, scaffold.
- PP-001: engineering documentation and task-tracking system.
- PP-002: owner's Windows toolchain builds and runs the scaffold (owner-reported).
- PP-004: time and fixed update (Stage 2).
- PP-005: ECS integration (Stage 3).
- PP-006: GPU context + purple clear (Stage 4).
- PP-014: GPU fault handling + logging decision (Stage 4).
- PP-007: first 2D primitive, quads from ECS (Stage 5).
- PP-008: textures + `Sprite` component with a minimal texture handle (Stage 6, part 1).
- PP-015: draw order (`Layer`) + batching by texture (Stage 6, part 2).
- PP-009: `Camera2D` + screen ↔ world mapping (Stage 7).
- PP-003: Stage 1 closed after the owner confirmed Windows `cargo test`, Escape and close.
- PP-013: license chosen: MIT OR Apache-2.0.
- PP-010: keyboard input (Stage 8, part 1).
- PP-016: mouse input (Stage 8, part 2).
- PP-011: asset root (Stage 9).
- PP-012: Breakout example game (Stage 10, part 1).
- PP-017: API review (Stage 10, part 2). Portfolio scope (Stages 0–10) complete.
- PP-018a: text rendering part 1: fonts, glyph atlas, `Text` (ADR-027).
- PP-018b: text anchors/alignment, `Context::measure_text`, Breakout HUD text. Phase P1 (Text) complete.
- PP-019: sprite sheets: `Sprite` regions (`TextureRegion`) + `SpriteGrid`.
- PP-020: sprite frame animation: `SpriteAnimation` + `render::advance_animations` (ADR-028).
- PP-021: screen-space drawing for HUD/UI: `ScreenSpace` (ADR-029).
- PP-022: audio part 1, sound effects: `cpal` + `hound` + own mixer, `Context::load_sound` / `play_sound` (ADR-030).
- PP-023: UI buttons: `ui::Button`, `ui::Pointer`, `ui::update_buttons` (ADR-031).
- PP-024a: audio playback control and looping: `PlaybackId`, `loop_sound`, `stop_sound`, volumes (ADR-032).
- PP-024b: OGG Vorbis decoding with `lewton`; format chosen by content; sandbox music is `loop.ogg` (ADR-033).
- PP-026a: scene files, part 1a: `Context::save_scene` / `load_scene` (RON via private serde types; drawing components; assets by relative path), `examples/scene.rs` (ADR-035).
- PP-025: per-texture sampling: `TextureFilter`/`TextureOptions`, `Context::load_texture_with`; Breakout's ball is `Linear` (ADR-034). Phase P2 complete.

## In Progress

- Nothing. (PP-003 closed on 2026-10-01.)

## Next

- **PP-026b: Scene files: animation, velocity and UI buttons** (phase P3). See [TASKS.md](TASKS.md#pp-026b-scene-files-animation-velocity-and-ui-buttons--next).

## Blocked

- Nothing is blocked.

## Technical Debt

- Rendering is capped at 60 fps by `FramePacer`, even on high-refresh displays (ADR-014). Revisit together with render interpolation.
- The draw list is rebuilt and sorted every frame (O(n log n)), even when nothing changed: ~1.8 ms per 10,000 drawables in release (R-18).
- Textures are never unloaded and keep a CPU copy (ADR-020, R-17). Deferred with generic handles (PD-06) until a second asset kind exists.
- `EngineConfig::exit_on_escape` is a second way to handle Escape; kept on purpose as a prototyping convenience (ADR-026).
- Text: the glyph atlas is a fixed 1024² RGBA texture that is cleared (and the frame's text re-laid out) when full; continuous zooming rasterizes every new size (R-25). Fonts are never unloaded.
- The sandbox reads `PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES` for automated smoke runs. It is game-side test plumbing, not an engine feature.

## Known Limitations

- Windows (owner): the purple window (Stage 4), `cargo test`, Escape and the close button are confirmed. Stages 5–7 rendering, vsync pacing and GPU fault paths are unverified there. macOS compiles and passes `cargo test` in CI but its rendering has never been seen; Wayland is untested.
- Input: 99 physical keys and 5 mouse buttons only (no text input, gamepads, touch or rebinding). Under X11 + XTEST, winit reports each synthetic wheel click twice (2 lines); real hardware unverified. An edge reaches `fixed_update` one frame late when the frame that saw it ran no fixed step.
- One camera only, without rotation. Screen-space content (`ScreenSpace`, PP-021) is always drawn above world content. UI widgets: buttons only (PP-023; no keyboard focus, layout, text input or disabled state). A non-integer zoom with `Nearest` sampling makes texels uneven (1 vs 2 pixels at zoom 1.5).
- Quads and sprites are drawn without MSAA, so rotated quad edges are aliased. Textures are sampled `Nearest` (default) or `Linear` (chosen per texture at load, ADR-034); there are no mipmaps, so even `Linear` sprites alias when shrunk below half size, and cells of a `Linear` sprite sheet blend with neighbouring texels unless the sheet has spacing.
- Textures: PNG only. A sprite shows its whole texture or one rectangular region of it (sprite sheets, PP-019); regions can be animated with `SpriteAnimation` (fixed frame rate per animation; no per-frame durations or events), and sheets need `Nearest` sampling (linear filtering would bleed across cells). A texture larger than the GPU limit (≥ 2048 everywhere) stops the engine with `Error::Asset` at the next frame rather than failing in `load_texture`.
- Linux/X11 only: if another X client destroys the window, winit 0.30.13 can intermittently panic in its own IME cleanup instead of PurplePie's clean `Error::Render` exit. Pre-existing (the Stage 8 build does it too); see R-22.
- Text: no wrapping or text boxes, no shaping (one glyph per `char`: no ligatures, complex or right-to-left scripts), no font fallback, kerning only from a `kern` table; small light-on-dark text looks a little bolder than in gamma-space renderers (linear blending, R-25); text larger than 512 physical pixels is not drawn. Sprites use `Nearest` sampling only (scaled-down sprites alias). Other Breakout friction points were decided in ADR-026 (some deliberately unchanged).
- GPU faults are not recoverable. A lost surface or device ends the game with `Error::Render` (ADR-017).
- The GPU fault test is `#[ignore]` (it needs a GPU), so CI does not run it. Run it with `cargo test -- --ignored`.
- Under lavapipe (software GPU), idle CPU is about 0.75 s per 3 s (1.4 s per 5 s with sprites, PP-008). This is GPU work done on the CPU, not a busy loop.
- Render interpolation is not implemented. `Time::alpha()` is exposed for it, but nothing uses it yet.
- Smoke runs use Xvfb with no window manager, so the close button is simulated by sending `WM_DELETE_WINDOW`.
- `rust-version = "1.90"` comes from dependency metadata. Only Rust 1.95.0 has been exercised in Cowork (R-19); CI uses the latest stable (1.99 on 2026-10-06), whose newer clippy lints Cowork cannot run. The code uses let-chains (stable since 1.88).
- Audio (PP-022/024a/024b): WAV and OGG Vorbis only (no MP3/FLAC); sounds are decoded completely when loaded (no streaming: ~10 MB per minute of 44.1 kHz mono, and a 3-minute stereo track takes ~7 s to load in a debug build, ~1.2 s in release); `lewton` ignores a stream's leading-sample trim, so some OGG files play a few ms longer than elsewhere; no fades (volume changes and stops are instant), no "still playing?" query; Windows/macOS audio compiled by CI but not heard yet; without a device the game is silent and libasound prints its own errors on Linux.
- Scenes (PP-026a): only `Transform2D`, `Quad`, `Sprite`, `Text`, `Layer`, `Hidden` and `ScreenSpace` are saved, and only on entities with a drawable; `SpriteAnimation`, `Velocity`, `ui::Button` (PP-026b) and game components (registry, later) are dropped. Loading adds to the world (no replace/unload). Same-layer draw-order ties use entity ids, which scenes do not store (ADR-035).
- Logging only reaches the console if the game installs a `log` backend (ADR-016). The sandbox does, and games using the library must choose their own.
- The owner's copy is inside OneDrive (R-15).

## Recent Changes

- **2026-10-07: PP-026a Scene files, part 1a (phase P3 started).**
  - New dependencies `ron` 0.12.2 + `serde` 1.0.229 (ADR-035; +5 crates on every platform). serde is used only inside the new crate-private `scene` module; no public type derives it.
  - New `Context::save_scene(path)` and `Context::load_scene(path) -> Vec<Entity>`: RON text, `version: 1`, assets by path relative to the asset root, all-or-nothing loading. New `Error::Save` for write failures.
  - New example `examples/scene.rs` and `assets/scenes/demo.ron` (14 entities: quads, sprites incl. regions/mirroring/rotation/Linear, world and screen-space text, layers, a hidden quad).
  - Follow-up fix: on the owner's Windows machine the shipped-scene test failed because `ron` writes `\r\n` line ends on Windows by default (only the header line used `\n`). Scene files are now always written with `\n`; the round-trip test asserts it (forcing `\r\n` on Linux reproduces the Windows failure).

- **2026-10-07: PP-025 Per-texture sampling (phase P2 complete).**
  - New public API (ADR-034): `render::TextureFilter` (`Nearest` default, `Linear`), `render::TextureOptions` (`NEAREST`, `LINEAR`, `with_filter`; non-exhaustive) and `Context::load_texture_with(path, options)`. `load_texture` is unchanged (Nearest).
  - The texture store keys its cache by (path, options); `Linear` textures get their transparent texels' colour bled from their neighbours on load (no dark fringes with straight alpha). The sprite pipeline has a `Nearest` and a `Linear` sampler and picks one per texture's bind group.
  - Breakout's ball loads `Linear` (friction point F9). No new dependencies.
  - Follow-up fix: the owner's clippy 1.98 rejected six `chunks_exact(4)` calls in `render/texture.rs` (`chunks_exact_to_as_chunks`, a lint Cowork's clippy 1.95 lacks); replaced with `as_chunks::<4>()`, and a pre-delivery grep was added to DEVELOPMENT §8 (R-19).

- **2026-10-07: PP-024b OGG Vorbis decoding.**
  - New dependency `lewton` 0.10.2 (ADR-033; +4 crates on every platform: `lewton`, `ogg`, `tinyvec`, `byteorder`).
  - `Context::load_sound` decodes WAV or OGG Vorbis, chosen from the file's first bytes (not its extension); OGG is decoded completely on load into the same samples the mixer already plays.
  - New asset `assets/sounds/loop.ogg` (`loop.wav` encoded with ffmpeg/libvorbis, 4.9 KB instead of 44 KB, same 22,050 frames); the sandbox's `M` loop plays it.

- **2026-10-07: PP-024a Audio playback control and looping (PP-024 split into a + b).**
  - `Context::play_sound` returns an `audio::PlaybackId`; new `loop_sound`, `stop_sound`, `set_sound_volume`, `stop_all_sounds`, `set_master_volume`, `master_volume`, `sound_duration` (ADR-032).
  - Mixer handles commands, seamless loops (interpolated across the seam), a master volume, and keeps loops when the 32-voice limit is hit.
  - New asset `assets/sounds/loop.wav`; the sandbox toggles it with `M`. No new dependencies.

- **2026-10-07: PP-023 UI buttons.**
  - New public module `ui` (ADR-031): `Button` component (hovered / pressed / clicked), `Pointer` snapshot (`from_input`), `update_buttons(world, pointer, viewport)` called by the game; topmost button wins.
  - Sandbox: "Reset camera" button in the top-right corner (colours show idle/hover/pressed; clicks on it don't stamp). No new dependencies.

- **2026-10-07: PP-022 Audio, part 1 (sound effects).**
  - New dependencies `cpal` 0.18.2 + `hound` 3.5.1 (ADR-030; +5 Linux / +3 Windows / +9 macOS crates). Linux builds need `libasound2-dev`; CI installs it.
  - New public module `audio` (`SoundId`), `Context::load_sound`, `Context::play_sound(id, volume)`, `Context::audio_available`, `EngineConfig::with_audio`. Own lock-free mixer (32 voices, resampling); silent fallback without a device.
  - Assets `assets/sounds/{blip,hit,lose}.wav` (generated). Sandbox clicks and Breakout bounces/bricks/lost balls play sounds; gameplay unchanged.

- **2026-10-07: PP-021 Screen-space drawing for HUD/UI.**
  - New public API: `render::{ScreenSpace, ScreenAnchor}` (ADR-029). Entities with `ScreenSpace` are drawn in logical window pixels from a window anchor (+Y up), ignoring the camera, after all world content; `from_window` / `to_window` convert cursor positions for hit tests.
  - Draw list: sort key gains a space bucket; `View` gains the logical size. Sandbox: a top-left HUD panel with text and a bottom-right square. No new dependencies.

- **2026-10-07: PP-020 Sprite frame animation.**
  - New public API: `render::{SpriteAnimation, AnimationMode, advance_animations}` (ADR-028). The game calls `advance_animations(world, dt)` from `fixed_update`; the draw list shows each animation's current frame (overriding `Sprite::region`).
  - Deterministic timing with a 1 µs boundary slack (needed: 50 Hz steps at 25 fps were a step late without it).
  - Sandbox: an animated sheet cell at (580, −250); the timed exit prints its frame. No new dependencies.

- **2026-10-06: PP-019 Sprite sheets (phase P2 starts).**
  - New public API: `render::{TextureRegion, SpriteGrid}`, `Sprite::region` + `Sprite::with_region`. Regions are texel rectangles turned into the instance's `uv_rect`; one texture is still one draw call.
  - `DrawList::build` takes the texture store (to know texture sizes). Sprites without a region render exactly as before.
  - New asset `assets/textures/sandbox_sheet.png`; the sandbox shows four of its cells (one mirrored with a negative scale).
  - ADR-020 extended. No new dependencies.

- **2026-10-06: CI fix after PP-018b.** GitHub Actions' clippy (Rust 1.99) rejected `chunks_exact_mut(4)` in `render/atlas.rs` (new lint `chunks_exact_to_as_chunks`); replaced by `as_chunks_mut::<4>()`. Behaviour unchanged. Cowork cannot install Rust 1.99, so CI is the only check for lints newer than 1.95 (R-19).

- **2026-10-06: PP-018b Text anchors, measuring and Breakout HUD text (phase P1 complete).**
  - New public API: `render::{TextAnchor, HorizontalAnchor, VerticalAnchor, TextMetrics}`, `Text::anchor` + `with_anchor` (default `BASELINE_LEFT` = PP-018a behaviour), `Context::measure_text`, `TextMetrics::bounds`.
  - Horizontal anchors align every line of multi-line text; vertical anchors use the font's ascent/descent; all shifts are whole pixels.
  - Breakout: centred score label, `YOU WIN!` / `GAME OVER` headline and an action hint drawn as text (window title kept). Gameplay unchanged.
  - ADR-027 extended; ADR-026's friction F1 resolved. No new dependencies.

- **2026-10-06: PP-018a Text rendering, part 1 (post-portfolio; PP-018 split into a + b).**
  - ADR-027: `ab_glyph` 0.2.32 (+0 crates on Linux, +4 on Windows) rasterizes TrueType/OpenType glyphs into one 1024² glyph atlas drawn by the sprite pipeline.
  - New public API: `render::Text { content, font, size, color }` (`Text::new(..).with_color(..)`), `render::FontId`, `Context::load_font`.
  - `Instance` gains `uv_rect` (96 B); text is a third material, drawn after quads and sprites in each layer.
  - Font shipped: `assets/fonts/Poppins-Regular.ttf` + `OFL.txt`. The sandbox shows a help label.
  - ROADMAP: long-term plan after Stage 10 (text → runtime essentials incl. in-game UI → editor foundations → debug overlay → scene editor → bigger example game).

- **2026-10-06: PP-017 Stage 10 API review (Stages 0–10 complete).**
  - ADR-026: `Game` + `Context` kept (ADR-008 reviewed), single crate kept (ADR-002 reaffirmed), `exit_on_escape` kept; decisions for Breakout's friction points F1–F10.
  - New public API: `render::Hidden` (skip drawing an entity), `Camera2D::fit(center, size, viewport)`, `Context::set_window_title`.
  - `#![warn(missing_docs)]` enforced (CI fails on undocumented public items).
  - Breakout: `Hidden` overlay, `Camera2D::fit`, score and lives in the window title; autoplay results unchanged.
  - README rewritten (features, getting started, example). Next: PP-018 text rendering.
- **2026-10-06: PP-012 Stage 10 Breakout example.**
  - New `examples/breakout.rs` (`cargo run --example breakout`): a complete game (paddle, ball, 60 bricks, lives, score, win/lose/restart, HUD, camera fitted to the window) using only the public API. No engine changes were needed.
  - New assets `assets/textures/breakout/{ball,brick}.png`.
  - Deterministic test modes `PURPLEPIE_BREAKOUT_AUTOPLAY=win|lose`.
  - Friction list F1–F10 recorded in TASKS for PP-017.
- **2026-10-06: PP-011 Stage 9 asset root (Stage 9 complete).**
  - New crate-private `src/assets/` (`AssetRoot`): relative asset paths resolve against `EngineConfig::with_asset_root`, else `assets/` next to the executable, else `assets/` in the working directory; chosen once at startup and logged (ADR-025, R-24 mitigated).
  - `Context::load_texture` takes paths relative to that root (absolute paths unchanged); new `Context::asset_root()`; `EngineConfig::asset_root` / `with_asset_root`.
  - Sandbox loads `textures/sandbox_quadrants.png`; README explains shipping `assets/` next to the executable.
  - Stage 9 narrowed (generic handles + unloading wait for a second asset kind, PD-06). Stage 10 split into PP-012 (Breakout example) and PP-017 (API review). No new dependencies.
- **2026-10-06: PP-016 Stage 8 mouse input (Stage 8 complete).**
  - `input::MouseButton` with `mouse_pressed` / `mouse_just_pressed` / `mouse_just_released`, `Input::cursor_position()` (logical px, `None` outside), `Input::scroll()` (lines), `Context::cursor_world()`. Keys and buttons share one edge implementation (ADR-024 extended).
  - The runner tracks the DPI scale from window events and handles mouse, cursor and wheel events; `app/state.rs` gained the physical → logical helper.
  - Sandbox: cursor marker, click stamps, wheel zoom; timed exit prints clicks and the cursor.
  - Found during validation: a pre-existing, intermittent winit panic when another X11 client destroys the window (R-22); winit/XTEST doubles synthetic wheel clicks.
  - Owner's GitHub edit mirrored: `LICENSE-APACHE` appendix line "Copyright 2026 David Vilela".
  - Stage 9 narrowed to PP-011 (asset root). No new dependencies.
- **2026-10-02: PP-010 Stage 8 keyboard input.**
  - New public module `input`: `KeyCode` (99 physical keys) and `Input` (`pressed`, `just_pressed`, `just_released`, `axis`, `pressed_keys`); `Context::input()`.
  - Edges are kept separately for `fixed_update` (latched until the first fixed step) and `update` (per frame), so each press reaches each callback exactly once (ADR-024, resolves PD-03; R-05 mitigated).
  - New `app/keymap.rs` (winit → `KeyCode`) and `app/state.rs` (`EngineState`; `Context::new` takes it instead of six arguments). Focus loss releases held keys; repeats and X11 synthetic presses are ignored; key/focus events logged at `debug`.
  - Sandbox: arrows pan the camera, `=` / `-` zoom; the timed exit prints the `=` press counts per callback. No new dependencies.
- **2026-10-01: owner confirmations recorded (no code changes).**
  - PP-003 DONE: on Windows, `cargo test` passes and Escape / the close button exit cleanly. M1 is VERIFIED.
  - CI: the first GitHub Actions run passed every job (fmt + clippy on Linux; check + test on Linux, Windows, macOS). R-23 closed.
  - PP-013 DONE: MIT OR Apache-2.0 (ADR-023): `license` in `Cargo.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, README License section.
- **2026-10-01: PP-009 Stage 7 camera (Stage 7 complete).**
  - New public `render::Camera2D { position, zoom }` with `screen_to_world` / `world_to_screen` (logical pixels, top-left origin, +Y down). Default = the ADR-018 view.
  - The runner owns one camera and a logical viewport; new `Context::camera()`, `camera_mut()`, `viewport_size()`. `Renderer::render` takes `&Camera2D`; `quad::view_projection` moved to `Camera2D::view_projection`.
  - Regression found and fixed before delivery: a per-frame `window.inner_size()` panicked inside winit when the X11 window was destroyed. The viewport is now updated from window events only (R-22, TECH_STACK).
  - Sandbox: `PURPLEPIE_SANDBOX_CAMERA=x,y,zoom`; the timed exit prints viewport and camera.
  - ADR-022 resolves PD-02. Stage 8 split into PP-010 (keyboard) and PP-016 (mouse). No new dependencies.
- **2026-10-01: PP-015 Stage 6 draw order + batching (Stage 6 complete).**
  - New public `render::Layer(pub i32)` component (optional, default 0, higher = on top).
  - New crate-private `render/draw.rs`: `DrawList` sorts quads and sprites by (layer, material, entity index) and builds one batch per (layer, material) run.
  - One shared instance buffer for quads and sprites; the pipelines became pipeline + bind-group holders; `collect_instances` / `collect_sprites` removed.
  - Sandbox: a yellow quad (layer 0) under the sprite's top-left corner and a pink quad (layer 1) over its bottom-right corner.
  - ADR-021 resolves PD-05 and PD-08 and supersedes ADR-020's "sprites after quads" clause. No new dependencies.
- **2026-10-01: PP-008 Stage 6 textures + sprites.**
  - Added `image 0.25.10` (PNG only; +12 crates, 116 → 128 unique normal dependencies; highest `rust-version` still 1.90).
  - New `Error::Asset { path, source }`. Public `render::TextureId`, `render::Sprite { texture, size, tint }`, `Context::load_texture`, `Context::texture_size`.
  - New crate-private `render/texture.rs` (`Textures` store, `decode_png`), `render/instance.rs` (`Instance` + `InstanceBuffer`, extracted from `quad.rs`), `render/sprite.rs` + `sprite.wgsl`.
  - The runner owns the texture store and lends it to `Context`; the renderer uploads new textures before each frame and draws sprites after quads.
  - Refactor: `QuadInstance` → shared `Instance`, quad pipeline builder → shared `rect_pipeline`. Quad behaviour unchanged (pixel counts identical).
  - Sandbox: `assets/textures/sandbox_quadrants.png` (generated 16×16 test image), one static sprite and one tinted spinning sprite.
  - ADR-020 (texture handle; pulls part of PD-06 forward). PD-05/PD-08 re-targeted to PP-015. New risk R-24 (working-directory-relative asset paths).
  - Drift fixed: ARCHITECTURE's `src/error.rs` row listed only 4 of the 9 error variants.
- **2026-10-01: PP-007 Stage 5 first 2D primitive.**
  - `bytemuck 1.25` added as a direct dependency (already in the tree, so no new crate).
  - `math::Mat4` and `Transform2D::to_mat4()`.
  - Public `render::Quad`. Crate-private `render/quad.rs` + `quad.wgsl`, with an instanced pipeline and no vertex buffer or bind group.
  - The renderer takes `&World` and the DPI scale factor.
  - The sandbox shows three reference quads (one bouncing).
  - ADR-018 (coordinates, resolves the PD-02 core) and ADR-019 (quad pipeline). New pending decision PD-08 (draw order).
  - Stage 6 was split into PP-008 (textures + sprites) and PP-015 (layers + batching).
- **2026-09-30: PP-014 GPU fault handling + logging.**
  - New `src/render/faults.rs` (`FaultSlot`, `GpuFault`) replaces wgpu's panicking uncaptured-error handler and captures device loss.
  - New `Error::Render`. `Validation` and `Lost` acquire results are now fatal (ADR-017).
  - Found and fixed: surface recreation after `Lost` panicked inside wgpu-hal when the window was gone.
  - `log` added as a direct dependency (no new crate). The sandbox has a stderr logger with `PURPLEPIE_LOG` (ADR-016, resolves PD-04).
  - Removed the renderer's unused `instance`/`window` fields, which existed only for recreation.
- **2026-09-30: CI added (owner request, separate task).** `.github/workflows/ci.yml`: fmt + clippy on Linux; `cargo check` + `cargo test` on Linux, Windows and macOS. Not run yet.
- **2026-09-30: Windows confirmation.** The owner's screenshot shows the sandbox window on Windows, and all sampled pixels are `#6A0DAD`. PP-006 is closed.
- **2026-09-30: PP-006 Stage 4 GPU context + purple clear.**
  - Added `wgpu 30.0.1` and `pollster 1.0.1` (116 unique normal dependencies on Linux).
  - New `src/render/`: public `Color` (sRGB) and crate-private `Renderer`.
  - New `Error::{Surface, Adapter, Device, SurfaceUnsupported}`. New `EngineConfig::clear_color` / `with_clear_color`.
  - The runner owns `Arc<Window>` + `Option<Renderer>`: created in `resumed`, resized on `Resized`, rendered after `update`, dropped on `suspended`/`exiting` before the window.
  - ADR-014 (vsync + 60 Hz cap; the pacer is kept) supersedes ADR-010's pacing clause. ADR-015 (sRGB colors) resolves PD-01.
  - Fixed stale doc entries: the ADR-003 index said "no modules yet", and ARCHITECTURE §5 still described `Poll` + `Fifo`.
- **2026-09-30: PP-005 Stage 3 ECS integration.**
  - Added `hecs 0.11.1` and `glam 0.33` (f32 types only), 77 unique normal dependencies on Linux.
  - New `src/math/` (`Transform2D`, `Vec2`) and `src/ecs/` (`World`, `Entity`, `hecs` re-export, `Velocity`, `integrate_velocity`). Both modules are public: `purplepie::math`, `purplepie::ecs`.
  - The runner owns one `World`, exposed via `Context::world()` / `world_mut()`.
  - The sandbox spawns a moving entity and reports its position at exit.
  - Stage 4 was split into PP-006 (GPU context + clear) and PP-014 (GPU errors, device loss, logging), and PD-04 moved to PP-014.
  - Delivery rule: every delivery starts with the `Remove-Item … Claude outputs` command (DEVELOPMENT §10).
- **2026-09-30: PP-004 Stage 2 time & fixed update.**
  - New `src/time/` (`Time`, `FixedTimestep`), `std` only.
  - `Game::fixed_update`. All `Game` callbacks now have defaults (`update` was required before).
  - `Context::time()` / `Context::dt()` (`f32`, the step for the current callback).
  - `EngineConfig::{fixed_dt, max_frame_dt, max_fixed_steps}` with validation. `EngineConfig` no longer derives `Eq`.
  - Runner frame: tick → fixed × n → update. No callbacks after `request_exit`, including mid-way through fixed steps.
  - Public API adds `Time`. No new dependencies.
  - DEVELOPMENT §3/§10: deliveries go only to the chat, and nothing is saved on the owner's computer.
- **2026-09-30: PP-003 Stage 1 implementation.**
  - Added `winit 0.30.13` and `thiserror 2.0.21` (72 unique normal dependencies on Linux).
  - New `src/error.rs` and `src/app/` (`mod`, `config`, `game`, `pacer`, `runner`).
  - Public API: `Engine`, `EngineConfig`, `Game`, `Context`, `Error`, `BoxError`, `Result`, `VERSION`.
  - The sandbox is now a `Game`.
  - Fixed a bug found in testing: game callbacks could run after exit had started. They are now gated on `event_loop.exiting()`.
  - Docs corrected: winit 0.30 uses `tracing`, not `log` (ADR-011 note, TECH_STACK). The ADR-008/010 implementation notes were updated.
  - Owner-requested process rule added to DEVELOPMENT §3: no AI attribution lines in commit messages.
  - Owner rule added to DEVELOPMENT §2/§3/§10: Claude never touches the owner's computer. Every delivery is a full-project ZIP plus the `Expand-Archive` command.
- **2026-09-30: PP-001 documentation system** (DECISIONS/TASKS/PROJECT_STATUS/DEVELOPMENT added; `docs/adr/` merged).
- **2026-09-30: PP-000 Stage 0** architecture and scaffold.

## Validation

Executed in Cowork (Linux x86_64, Rust 1.95.0, Xvfb + Mesa lavapipe / llvmpipe, `libasound2-dev` installed) on 2026-10-07,
after the final PP-026a change (CI additionally runs the latest stable clippy and builds Windows/macOS, which Cowork cannot; R-19):

| Command / check | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ PASS |
| `cargo check --locked --all-targets --all-features` | ✅ PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` (Rust 1.95) | ✅ PASS (0 warnings) |
| `cargo test --locked` | ✅ PASS: 204 unit tests + 29 doctests, 11 ignored (GPU) |
| `cargo test --locked -- --ignored` (lavapipe) | ✅ PASS: 11/11 |
| `grep -rnE "chunks_exact(_mut)?\([0-9]" src examples` (R-19 guard) | ✅ no matches |
| Scene example (`PURPLEPIE_SCENE_EXAMPLE=save`, `build`, default `load`) under Xvfb | ✅ the three 1024×600 frames are pixel-identical; `save` wrote `assets/scenes/demo.ron`, which the unit tests read back byte-for-byte |
| Scene mutation checks | ✅ dropping `ScreenSpace` on load or the tint on save fails the round-trip tests |
| `cargo doc --no-deps` (`-D warnings`) | ✅ no warnings |
| `cargo build --locked`, `cargo build --release --example breakout` | ✅ PASS |
| Sandbox whole-frame model + HUD (camera (0,0)×1) | ✅ 0 mismatches; HUD panel, corner square and button crops identical to the PP-025 run |
| Breakout autoplay `win` / `lose` (debug, ALSA null device) | ✅ unchanged: `Won after 7135 fixed steps … score 220`; `Lost after 892 fixed steps … score 14` |
| Breakout lose screen vs PP-025 | ✅ pixel-identical |
| Window destroyed (fresh display) | ✅ `error: GPU rendering failed` / `surface was lost`, exit 1, no panic |

Owner-provided (not executed by Claude): Windows x64: the Stage 4 purple window was confirmed by screenshot (pixel-checked); `cargo test`, Escape and the close button confirmed on 2026-10-01. GitHub Actions: first run all green on 2026-10-01.
Stages 5–10 and text (PP-018a/b) on Windows have not been seen yet.

## Last Updated

2026-10-07. PP-026a done (scene files: format + drawing components; ADR-035). PP-026b (animation, velocity, buttons in scenes) is next.
