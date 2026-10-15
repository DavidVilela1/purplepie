# PurplePie Project Status

Factual snapshot. States: `NOT_STARTED`, `IN_PROGRESS`, `FUNCTIONAL`,
`NEEDS_REFACTOR`, `BLOCKED`, `VERIFIED`. `VERIFIED` means validated by executed
commands, and the evidence is listed below.

## Current Milestone

**Phase P5 (scene editor): complete (2026-10-14, verified on Linux).** `crates/purplepie-editor` (ADR-044) opens a
scene file, shows it, edits every saved value, adds, duplicates and deletes entities, undoes every change and saves
the file back in the same format; a game builds its own editor that checks its component types in three lines, and
GUIDE §15 explains both. Before it, phase P4 (2026-10-13) added the optional debug overlay (ADR-043). Before that, phase P3.5 (2026-10-12) delivered release 0.1.0 (tag `v0.1.0` on GitHub)
and a second game, Purple Swarm, built on the public API; M10 (Engine API Stabilization), the portfolio scope,
runtime essentials (P1–P2) and editor foundations (P3) were reached earlier. Windows: the owner's checklist passed
(PP-033b; again on 2026-10-14 with step 9, the overlay and the editor, owner-reported). Released as **0.2.0**
(PP-039). macOS is built and tested in CI but has not been seen
running.
**Phase P5 (scene editor): complete** (PP-038a–PP-038j2, ADR-044, 2026-10-13 to 2026-10-14): the editor is a
separate crate written as a `Game` on a public egui hook. It took: the design, a fix it needed first (scene loading was
quadratic, PP-038b), the public egui hook (`egui` feature, `Game::egui`, PP-038c) and picking
(`Context::entities_at`, PP-038d), lenient scene loading (`load_scene_keeping_unknown`, PP-038e) and the editor
crate's first slice (`crates/purplepie-editor`: open, list, pan/zoom, save, PP-038f; select, move, undo/redo,
PP-038g) and an inspector that edits every saved value: transform, layer and hidden (PP-038h1), quads, sprites,
texts and game components kept as RON text (PP-038h2), deleting and duplicating entities with undo (PP-038i1) and
adding quads, sprites and texts (PP-038i2), a game's own editor that checks its component types (PP-038j1), and the
guide section with the checklist step (PP-038j2). It was released in **0.2.0** (PP-039, 2026-10-14; tag checked in
PP-040). **Phase P6 (proof): in progress** since PP-040 (2026-10-15), which planned Pie Heist ([GAME3.md](GAME3.md)), a
top-down heist whose five levels are drawn in the editor. PP-041 let the editor add and remove game components. Next:
PP-042a, the Pie Heist skeleton and its editor, with level 1 drawn in the editor.

## Current Stage

**Post-portfolio phase P2 (runtime essentials): complete** (2026-10-07, verified on Linux): PP-019 sprite sheets,
PP-020 sprite animation, PP-021 screen-space drawing, PP-022 sound effects, PP-023 UI buttons, PP-024a playback
control, PP-024b OGG Vorbis decoding and PP-025 per-texture sampling. Stages 0–10 and phase P1 (Text) were completed
earlier. **Phase P3 (editor foundations): complete** (2026-10-08): scene files (PP-026a/b, ADR-035), game components in
scenes (PP-027, ADR-036), asset hot reload for textures, fonts and sounds (PP-028/029, ADR-037) and the workspace layout
decision (PP-030, ADR-038). **Phase P3.5 (usability and first release): complete** (2026-10-12). PP-031, the outside-crate
trial, is done: a small game built from the README alone worked after three README gaps were fixed, and its 17
findings are in [USABILITY.md](USABILITY.md). PP-032a added the newcomer guide [GUIDE.md](GUIDE.md), compiled by
`cargo test`, and PP-032b rewrote the API module pages for game authors (item 2 complete; no template crate, ADR-039).
PP-033a wrote the Windows/macOS [CHECKLIST.md](CHECKLIST.md) and ran it on Linux; the owner's run (PP-033b)
passed on Windows (owner-reported, PP-033b; see Validation). PP-034a/b, the second API review (ADR-040), renamed the
edge anchors, made errors print readably, marked growing types `#[non_exhaustive]`, formatted game components in
scene files and added the opt-in console logger (item 4 complete). PP-035a prepared the release (compatibility
policy in [RELEASING.md](RELEASING.md), `CHANGELOG.md`, CI jobs for the minimum Rust version and the latest
dependencies) and PP-035b made it: **version 0.1.0**, `repository` set, dependency by git tag. Items 1–5 of P3.5 are
done. PP-036a specified the second game, Purple Swarm ([GAME2.md](GAME2.md)), and PP-036b
added the two engine features it lacked: `math::Rng` and `math::Rect` (ADR-041). PP-036c1 started the game in
`games/purple-swarm` (a workspace member built by CI, ADR-042): arena, player, camera, shooting. PP-036c2 made it
playable: enemies, waves, collisions, score and sounds. PP-036d1 added the title, pause and game-over screens with
restart, and PP-036d2 the best-score file and a difficulty curve measured with the bot, which completes item 6 and
the phase. **Phase P4 (debug overlay): complete** (2026-10-13). PP-037a decided the approach after a measured throwaway spike:
egui inside the engine behind an off-by-default `debug-overlay` feature (ADR-043). PP-037b1 implemented the feature,
`EngineConfig::with_debug_overlay`, the F12 toggle and the overlay's render pass (a placeholder window); PP-037b2
routed the mouse to it (clicks and the wheel on the overlay's window stay with it); PP-037c replaced the placeholder
with a stats window (frame time and FPS, fixed steps, entities, draws, camera, window, cursor); PP-037d1 added an
"Entities" window (list, engine components of the selected entity) and PP-037d2 its registered game components as
RON, which completes **phase P4**. **Phase P5 (scene editor): complete** (2026-10-14). PP-038a decided its shape after a
throwaway spike (ADR-044): a separate `crates/purplepie-editor` crate written as a `Game` on a public `Game::egui`
hook behind a new `egui` feature, with undo as editor commands. The spike also found that scene loading was quadratic
(ron 0.12.2 scans the rest of the file for every number it reads without a known type: 1 000 quads took 1.5 s to
parse). PP-038b fixed that: ron is held at 0.12.1 (the scan came with 0.12.2) and a scene is parsed once, so 10 000
entities with game components load in 0.12 s instead of 12 s. PP-038c added the public egui hook: with the
optional `egui` feature and `EngineConfig::with_egui(true)`, `Game::egui(ctx, ui)` draws the game's own egui UI
over its frame (`purplepie::egui`), and the debug overlay now builds on the same egui frame. PP-038d added picking:
`Context::entities_at(point)` lists the entities drawn at a world point, topmost first in draw order. PP-038e added
`Context::load_scene_keeping_unknown`, which keeps game components nobody registered as text
(`scene::UnknownComponents`) and saves them back unchanged. PP-038f started the editor itself,
`crates/purplepie-editor` (`cargo run -p purplepie-editor -- <scene.ron>`): it opens a scene, lists its entities, pans
and zooms beside its panel and saves it back. PP-038g completed the first vertical slice ADR-044 set: entities are
selected by click or in the list, outlined, inspected, moved by dragging or with the arrow keys, and moves are undone
and redone, with an unsaved-changes marker. PP-038h1 made the inspector edit position, rotation, scale, layer and
hidden, each change one undo step. PP-038h2 added quads (size, colour), sprites (size, tint, region), texts
(content, size, colour, anchor) and the game's own components, edited as RON text that is checked when the field is
left (`UnknownComponents::set`). PP-038i1 added deleting (Delete) and duplicating (Ctrl+D), each one undo step;
undoing a delete brings back the same entity with everything it had. PP-038i2 added an "Add" row (a quad, a sprite
chosen from the asset folder's images, a text in one of its fonts) and a plain "new scene" message for a file that
does not exist yet. PP-038j1 lets a game build its own editor in three lines (`purplepie_editor::main_with` with
`GameComponents`): values of its components are checked against its types when the scene opens and when edited
(engine: `scene::check_component`). PP-038j2 added GUIDE §15 (using the editor, and a game's own editor binary whose
code is a compiled example kept identical to the guide by a test), the checklist step for the example editor, and
closed **phase P5**. PP-039 released it as **version 0.2.0**; the owner tagged `v0.2.0`, and PP-040 checked the tag and
planned **phase P6**: Pie Heist ([GAME3.md](GAME3.md)), whose levels are made in the editor. Next: **PP-041**, adding and
removing game components in the editor, the one gap every Pie Heist level needs; PP-041 built it ("Game component:
Add…" with the game's defaults, an "x" to remove; engine `scene::component_text`). Next: **PP-042a**, the Pie Heist
skeleton and its editor.
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
- PP-041: editor adds a game component (the game's types given with `GameComponents::with_default`) and removes one, each one undo step; engine `scene::component_text`.
- PP-040: P6 plan: tag `v0.2.0` checked (identical tree; outside crate by the tag builds); GAME3.md (Pie Heist), gaps, P6 split into PP-041 and PP-042a–e.
- PP-039: release 0.2.0: version, CHANGELOG `[0.2.0] - 2026-10-14`, dependency lines on `tag = "v0.2.0"` (README, GUIDE §1/§15, RELEASING), RELEASING covers the editor's library; preconditions owner-reported green.
- PP-038j2: GUIDE §15 "The scene editor" (controls; a game's own editor binary, its files compiled as the editor crate's `guide_editor` example and kept identical to the guide by a test; checked as an outside crate depending on GitHub `main`); CHECKLIST step 9 with the example editor; RISKS reviewed. Phase P5 complete.
- PP-038j1: a game's own editor: `purplepie_editor::GameComponents`, `run_with`, `main_with`; kept game component values checked against the game's types on open and on edit (still saved as text); engine `scene::check_component::<T>`; example `scene_demo_editor`.
- PP-038i2: editor adds quads, sprites (image from the asset folder) and texts (font from the asset folder) at the view's centre, each one undo step; a missing scene file opens as a new scene with a plain message.
- PP-038i1: editor deletes (Delete) and duplicates (Ctrl+D) the selection, each one undo step; undoing a delete brings back the same entity (`World::spawn_at`) with all its components; entities with components the editor does not know are refused.
- PP-038h2: editor inspector edits quads (size, colour), sprites (size, tint, region), texts (content, size, colour, anchor) and kept game components as RON text checked on leaving the field (engine: `scene::UnknownComponents::set`, `scene::InvalidRon`); one undo step per field.
- PP-038h1: editor inspector edits position, rotation, scale, layer and hidden; undo history generalized to before/after component values; one step per field change.
- PP-038g: scene editor first slice complete: select (click, cycling, list), outline, read-only inspector, move (drag, arrow keys), undo/redo, unsaved-changes marker.
- PP-038f: scene editor crate, part 1: `crates/purplepie-editor` opens a scene (lenient), lists its entities, pans/zooms/fits beside its panel, saves with Ctrl+S, shows errors in a status line, never overwrites a file it could not open.
- PP-038e: lenient scene loading: `Context::load_scene_keeping_unknown`, public `scene::UnknownComponents`; kept components saved back unchanged.
- PP-038d: picking: `Context::entities_at(point)`, topmost first in draw order; hidden and screen-space entities excluded; rotation, scale and text bounds.
- PP-038c: public egui hook: `egui` feature, `purplepie::egui`, `Game::egui(ctx, ui)`, `EngineConfig::with_egui`; one egui frame laid out by the runner (game UI, then the overlay's windows) and painted by the renderer; keys withheld only while an egui text field has the focus; `egui_panel` example.
- PP-038b: scene loading linear again: ron held at exactly 0.12.1 (0.12.2 made parsing quadratic), scene parsed once before the version check; regression test (R-31 mitigated).
- PP-038a: scene editor design: ADR-044 (separate editor crate on a public `Game::egui` hook behind an `egui` feature; command undo; engine picking and lenient loading); throwaway spike measured; P5 split into PP-038b–PP-038j.
- PP-037d2: inspector shows registered game components as RON (`Registry::describe`, prepared by the runner so `debug` and `render` stay independent of `scene`); "Entities" window at the top right. Phase P4 complete.
- PP-037d1: debug overlay "Entities" window: entities by id (virtualized list), the selected entity's engine components, game components counted; collapsed by default.
- PP-037c: debug overlay stats window: frame time (last, smoothed) and FPS, fixed steps per frame, entity count, draw batches and instances, camera, window size and scale, cursor in screen and world coordinates.
- PP-037b2: debug overlay input routing (egui-winit): pointer events the overlay consumes stay with it; releases of buttons the game holds and all keys reach the game; egui scale applied once.
- PP-037b1: optional `debug-overlay` feature (egui + egui-wgpu 0.36; Rust 1.95 for the feature), `EngineConfig::with_debug_overlay`, F12 toggle (never reaches the game), overlay pass through a non-sRGB view of the surface, placeholder window; frames without it unchanged; CI `msrv` without features.
- PP-037a: debug overlay design (ADR-043): egui 0.36 inside the engine behind an off-by-default `debug-overlay` feature, engine-owned and outside the ECS world; measured in a throwaway spike (exact wgpu 30 / winit 0.30 match, +32 lock entries, +6.1 MB release, Rust 1.95 for the feature); R-30.
- PP-036d2: Purple Swarm best score (file next to the executable, written via temporary file + rename, broken or missing = 0) and difficulty curve (bot survives 208 s / wave 11 instead of 373 s / wave 19; `survive` autoplay mode). Phase P3.5 complete.
- PP-036d1: Purple Swarm screens: title, pause (world frozen, music muted), game over with restart via `World::clear`; engine-free, unit-tested screen flow; banner backdrop; red HP at 2 or fewer.
- PP-036c2: Purple Swarm playable: Drifters and Dashers in 20 s waves, circle collisions, 5 hit points with blinking invulnerability, score, health orbs, five sounds, game over; deterministic autoplay summary line.
- PP-036c1: `games/purple-swarm` (ADR-042: example games as workspace members; CI `--workspace`): arena, player, camera, shooting, HUD, autoplay bot.
- PP-036b: `math::Rng` (PCG32, seedable, deterministic) and `math::Rect` + `circles_overlap` (ADR-041); the guide's game uses them.
- PP-036a: second game specified: Purple Swarm, a top-down arena shooter ([GAME2.md](GAME2.md)); engine gaps: random numbers and overlap tests (PP-036b); skeleton built from the v0.1.0 tag.
- PP-035b: release 0.1.0: version, `repository = "https://github.com/DavidVilela1/purplepie"`, CHANGELOG `[0.1.0] - 2026-10-09`, git-tag dependency line in README/guide/RELEASING.
- PP-033b: owner's Windows run of `docs/CHECKLIST.md`: all steps as expected.
- PP-035a: release preparation: `docs/RELEASING.md` (distribution by git tag, 0.x compatibility policy, procedure), `CHANGELOG.md`, CI `msrv` job and `latest-deps.yml` workflow.
- PP-034b: game components in scene files written as one spaced line; opt-in `EngineConfig::console_log` (stderr logger, level from `PURPLEPIE_LOG`), used by the sandbox and both examples.
- PP-034a: second API review (ADR-040): `ScreenSpace::TOP_CENTER`/`CENTER_LEFT`/`CENTER_RIGHT`/`BOTTOM_CENTER` (scene files read the old names), `Error` `Debug` prints message + causes, `Error::Save` has the resolved path, `#[non_exhaustive]` on growing public types.
- PP-033a: platform checklist `docs/CHECKLIST.md` (PowerShell, 8 steps + macOS/Linux appendix), every step run on Linux, the PowerShell blocks under PowerShell 7.
- PP-032b: module pages for game authors (summary, guide link, compiled example; internals under "Engine notes"); starter template decision ADR-039 (the guide is the template).
- PP-032a: newcomer guide `docs/GUIDE.md` (14 sections, every Rust block a doctest); checked as an outside crate (phase P3.5 item 2, part 1).
- PP-031: outside-crate trial: a game crate outside the package built from the README alone; findings in `docs/USABILITY.md`; README, crate-page and `ecs` docs fixed (phase P3.5 item 1).
- PP-030: workspace layout: root `Cargo.toml` declares the workspace; the engine stays at the root; future crates under `crates/` (ADR-038). Phase P3 complete.
- PP-029: font and sound hot reload: shared `assets::watch`, glyph atlas cleared on font change, sounds swapped for new plays (ADR-037 extension).
- PP-028: texture hot reload: opt-in `EngineConfig::hot_reload`, 0.5 s polling of file stamps, same `TextureId`, GPU re-upload (ADR-037).
- PP-027: scene files, part 2: game components via `Context::register_scene_component` (serde bound), stored by name (ADR-036).
- PP-026b: scene files, part 1b: `SpriteAnimation` (with playback position), `Velocity`, `ui::Button` in scenes; PP-026a files still load (fixture).
- PP-025: per-texture sampling: `TextureFilter`/`TextureOptions`, `Context::load_texture_with`; Breakout's ball is `Linear` (ADR-034). Phase P2 complete.

## In Progress

- Nothing. (PP-003 closed on 2026-10-01.)

## Next

- **PP-042a: Pie Heist skeleton and its editor** (phase P6, item 3): `games/pie-heist` (components, level loading, movement with wall collision, camera) and `games/pie-heist-editor`; level 1 drawn in the editor; grid-snapping decision. May be split at its start. See [TASKS.md](TASKS.md#pp-042a-pie-heist-skeleton-and-its-editor--next).

## Blocked

- Nothing is blocked. (The `v0.1.0` tag is on GitHub and equals this tree before PP-036a.)

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
- Picking (`Context::entities_at`, PP-038d): world-space entities only; transparent sprite texels count; text by its measured box (drawn text is snapped to whole pixels, up to a pixel off); linear in drawable entities and measures every text per call.
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
- `ron` is held at exactly 0.12.1 (R-31): 0.12.2 parses in quadratic time. A game that itself requires ron 0.12.2 or later cannot build with PurplePie until ron releases a fix and the pin moves.
- Scenes (PP-026a/b, PP-027): every engine component and every *registered* game component is saved; unregistered components are dropped silently, and entities with neither a drawable/animation/button nor a registered component are skipped. A file with game components loads with `load_scene` only where those components are registered (unknown names are errors); `load_scene_keeping_unknown` keeps them as text instead (PP-038e). Components holding an `Entity` cannot be saved. Button hover/click state is not saved (it starts idle). Loading adds to the world (no replace/unload). Same-layer draw-order ties use entity ids, which scenes do not store (ADR-035).
- Hot reload (PP-028/029): textures, fonts and sounds (not scenes); off by default; changes appear after up to ~0.5 s; a texture too large for the GPU keeps the old one; sounds already playing (including loops) keep their old samples until started again; a font change re-rasterizes all text once.
- Logging reaches the console only if the game opts in with `EngineConfig::with_console_log(true)` (ADR-040; the sandbox and examples do) or installs its own `log` backend; otherwise engine warnings are silent.
- Debug overlay (optional feature): stats and a read-only entity inspector (engine components, registered game components as RON; unregistered ones only counted; long `Debug` values for `Text`/`Sprite`; a new selection's game components appear one frame later); at the top left over any HUD there (it can be collapsed); in debug builds on a software GPU it costs about a quarter of the frame rate (sandbox 59 → 43 fps; release 56 → 53 fps), mostly unoptimized egui; keys reach the game except while an egui text field has the focus; the feature needs Rust 1.95, and binaries built with it embed egui's fonts (R-26). Seen only on Linux/lavapipe.
- `egui` hook (optional feature, PP-038c): an egui pass runs every frame while `with_egui` is on (cost not measured separately; the visible overlay costs about a quarter of the frame rate in debug builds on a software GPU); the game's UI covers the scene drawn under it (no separate viewport); the egui context is recreated when the platform suspends the app, so panel sizes and positions reset then. Seen only on Linux/lavapipe.
- Scene editor (PP-038f, PP-038g): selects, moves (world-space entities only), edits transform, layer, hidden, quads, sprites (not the texture), texts (not the font) and kept game components (PP-038h1/h2), undoes and saves; game components are RON text checked as syntax, and against the game's types only in a game's own editor (PP-038j1), with no typed widgets; game components are added from a game's own editor's defaults (`with_default`) and removed with the "x" (PP-041); velocity, animation, buttons and screen-space anchors are read-only; entities can be added (quads, sprites, texts), deleted and duplicated (PP-038i1/i2); asset lists are read when the scene opens; there is no "New", "Open" or "Save as" (one file per run); screen-space entities are drawn relative to the whole window, so its panel can cover them; it runs no game logic (animations stay at their saved frame); Linux only so far (CHECKLIST step 9 has a Windows check).
- The owner's copy is inside OneDrive (R-15).

## Recent Changes

- **2026-10-15: PP-041 Editor adds and removes game components.**
  - In a game's own editor, "Game component: Add…" in the inspector gives the selected entity one of the game's components with its default value (types given with the new `GameComponents::with_default`); the "x" after a component removes it. Both are one undo step.
  - Engine: `scene::component_text(&value)` writes a value exactly as a scene save does. 1 engine test; the editor crate has 24 unit tests and 4 doctests.

- **2026-10-15: PP-040 Phase P6 planned.**
  - The `v0.2.0` tag equals the released tree, and an outside crate following the guide builds against it.
  - [GAME3.md](GAME3.md) specifies Pie Heist: five levels drawn in the editor, with walls, pies, keys, doors, exits, guards and the player start as the game's components. The one gap that blocks it, adding and removing game components in the editor, is PP-041. Documentation only.

- **2026-10-14: PP-039 Release 0.2.0.**
  - Version 0.2.0 for the engine and the editor crate; CHANGELOG `[0.2.0] - 2026-10-14`; README, GUIDE and RELEASING now point to `tag = "v0.2.0"`. The owner tags it after pushing.
  - Owner-reported before the release: every checklist step green (including step 9 on Windows), all tests and all CI jobs green. New owner rule in DEVELOPMENT §3: owner-run checks are assumed green unless the owner reports otherwise.

- **2026-10-14: PP-038j2 Scene editor in the guide; phase P5 complete.**
  - GUIDE §15 "The scene editor": how to open a level, the controls, and a game's own editor (a second binary calling `purplepie_editor::main_with`). The two files it shows are compiled as the editor crate's `guide_editor` example, and a test fails if the guide's text differs from them. Followed word for word as a fresh outside crate with both dependencies from GitHub `main`: it built, saved a level with a `my_game::Door`, and its editor checked that value against the type.
  - CHECKLIST step 9 runs the example editor; RISKS reviewed (R-29, R-30 notes). No engine or editor behaviour change.

- **2026-10-14: PP-038j1 A game's own scene editor.**
  - A game builds its own editor with `purplepie_editor::main_with(GameComponents::new().with::<Spin>("my_game::Spin"))`: values of its components are checked against its types when a scene opens (those that do not fit are shown in red) and when edited (refused). The values stay text, so they are saved exactly as written.
  - Engine: `scene::check_component::<T>(ron)`. Example `cargo run -p purplepie-editor --example scene_demo_editor -- assets/scenes/demo.ron`. PP-038j was split; the guide section and P5's close-out are PP-038j2.

- **2026-10-14: PP-038i2 Editor adds quads, sprites and texts.**
  - An "Add" row in the panel: Quad, Sprite (pick an image from the asset folder) and Text (pick a font); the new entity appears at the centre of the view, selected, and is one undo step. Asset paths are saved relative to the asset folder.
  - Opening a file that does not exist now says "new scene: … saving (Ctrl+S) creates it" instead of showing a red error. 20 unit tests in the crate; no engine change.

- **2026-10-14: PP-038i1 Editor deletes and duplicates entities.**
  - Delete (key or button) removes the selection; Ctrl+D (or Duplicate) adds a copy 16 units to the right and down and selects it. Each is one undo step; undoing a delete brings back the same entity with all its components, so earlier undo steps still work.
  - An entity with a component the editor does not know is refused with a message rather than copied in part. PP-038i was split; adding new entities is PP-038i2. 19 unit tests in the crate; no engine change.

- **2026-10-14: PP-038h2 Editor inspector edits drawables and game components.**
  - The inspector now edits quads (size, colour), sprites (size, tint, region), texts (content, size, colour, anchor) and the game's own components, each as a RON text field checked when it is left: invalid text is refused with a message and the old value stays. Every field change is one undo step.
  - Engine: `scene::UnknownComponents::set(name, ron)` with the new `scene::InvalidRon` error. 1 engine test, 16 unit tests in the editor crate.

- **2026-10-14: PP-038h1 Editor inspector edits transform, layer and hidden.**
  - The inspector now has number fields for position, rotation (degrees) and scale, a layer checkbox and number, and a hidden checkbox; drag a number or click it to type. A change shows at once and is one undo step when the field is let go.
  - The undo history records any component change (before and after), not only moves. 14 unit tests in the crate; no engine change.

- **2026-10-14: PP-038g Scene editor: select, move, undo (first vertical slice complete).**
  - Click an entity in the view to select it (clicking again on the same spot selects the one below), or click its row in the list; Escape deselects. The selection gets an orange outline (drawn by egui, not saved) and a read-only inspector under the list.
  - Drag the selection (or the topmost entity under the pointer) with the left button, or step it with the arrow keys (Shift: 10 units). Ctrl+Z / Ctrl+Shift+Z (or Ctrl+Y) and the Undo/Redo buttons undo and redo moves; `*` in the title and "unsaved changes" in the status line show when the scene differs from the file. Screen-space entities are not moved yet.
  - New `history.rs` and `select.rs`; 13 unit tests in the crate. No engine change.

- **2026-10-14: PP-038f Scene editor, first part.**
  - New workspace member `crates/purplepie-editor` (ADR-044): `cargo run -p purplepie-editor -- assets/scenes/demo.ron`. It is a `Game` on the public API with the `egui` feature: it opens the scene with `load_scene_keeping_unknown`, lists its entities in a left panel, pans (middle or right drag), zooms (wheel, around the cursor) and fits (F) the view beside the panel, and saves with Ctrl+S (byte for byte when nothing changed). Errors appear in a red status line; a file that exists but could not be opened is never overwritten.
  - CI: the `msrv` job excludes the editor (Rust 1.95). Because Cargo unifies features, `cargo test --workspace` now builds the engine with `egui`; use `cargo test` at the root or `--exclude purplepie-editor` for the feature-free engine (DEVELOPMENT.md). No engine change.

- **2026-10-13: PP-038e Lenient scene loading.**
  - New `Context::load_scene_keeping_unknown(path)`: like `load_scene`, but a game component nobody registered is kept as its RON text in the new `scene::UnknownComponents` component (new public module `scene`) instead of failing the load; `save_scene` writes it back unchanged (a registered value of the same name wins). `load_scene` itself is unchanged.
  - The shipped demo scene round-trips byte for byte with none or some of its game types registered. The debug overlay's inspector lists kept components as "name (kept as text)". 4 new unit tests, extended inspector/describe tests and a doctest; README, GUIDE §11, CHANGELOG.

- **2026-10-13: PP-038d Picking.**
  - New `Context::entities_at(point)`: the entities drawn at a world point, topmost first, in the reverse of the draw order (higher layer first; within a layer text, then sprites, then quads; later entities first), each once. Quads and sprites count by their rectangle and texts by their measured bounds, turned and scaled with their transform; hidden, screen-space and undrawn entities never count.
  - 7 unit tests (one compares with the draw list's actual order) and a doctest; README, GUIDE §8, CHANGELOG. Cost: 1.3 ms per call for 10 000 quads + 1 000 texts in release (34 ms in debug), so it is for clicks, not per-frame scans.

- **2026-10-13: PP-038c Public egui hook.**
  - New optional `egui` Cargo feature (Rust 1.95): `EngineConfig::with_egui(true)` makes the engine call the new `Game::egui(ctx, ui)` once per frame after `update`, with a `Context` and the root egui `Ui`, so a game draws its own panels and windows over its frame; `purplepie::egui` re-exports egui 0.36. `debug-overlay` now turns on `egui`.
  - The egui frame is laid out by the runner (the game's UI first, then the overlay's windows on top) and painted by the renderer after the game's pass. The stats window's draw counts are now the previous frame's. A frame that is never painted (minimized window) keeps its texture changes for the next one.
  - Input: egui gets window events while the overlay is visible or the hook is on; pointer events it uses stay with it (as before), and keys are withheld only while an egui text field has the focus. F12 still toggles while typing.
  - New `examples/egui_panel.rs` (needs `--features egui`). 3 new unit tests (routing test extended) and 1 doctest. Without the features nothing changes (frames pixel-identical).

- **2026-10-13: PP-038b Scene loading is linear again.**
  - Loading a scene was quadratic in its size: in a release build 1 000 entities took 1.5–2.2 s and 10 000 with game components over 12 s. Cause: ron 0.12.2 (number-range support) searches the rest of the text for every number read without a known type; 0.12.1 does not.
  - Cargo.toml now requires exactly `ron = "=0.12.1"`, and `scene::from_text` parses the whole file before checking its version (only a failed parse is read again for the version), so error messages are unchanged and a file is parsed once. Release: 10 000 entities with game components 117 ms, 1 000 quads 5 ms.
  - New unit test `parsing_time_grows_linearly_with_the_scene` (fails on ron 0.12.2: checked). Scene files and the scene example's frame are unchanged. CHANGELOG `[Unreleased]` → Fixed.

- **2026-10-13: PP-038a Scene editor design (ADR-044); phase P5 started.**
  - Decision: the editor is a separate crate, `crates/purplepie-editor`, written as an ordinary `Game`. The engine gets a public, optional `egui` feature with a `Game::egui(ctx, ui)` hook (turned on by `EngineConfig::with_egui`) and re-exports egui; the debug overlay will share that egui pass. The scene is drawn by the engine under the editor's egui panels; undo is a stack of editor commands.
  - A throwaway spike (not merged, [spikes/pp-038a-editor-spike.md](spikes/pp-038a-editor-spike.md)) proved the hook: an example opened `demo.ron`, picked, dragged, undid and saved a file that differed only in the moved entity's position.
  - The spike found that scene loading is quadratic because of ron 0.12.2 (1 000 quads: 1.39 s to parse; parsing the whole scene first: 9.3 ms). PP-038b fixes it first; game component values stay slow until ron releases its fix (R-31).
  - P5 is split into PP-038b–PP-038j. No code, API or dependency change in this task.

- **2026-10-13: PP-037d2 Inspector shows registered game components; phase P4 complete.**
  - Selecting an entity in the overlay's "Entities" window now also shows its game components registered for scenes, under their registered names and as the same one-line RON a scene file holds (e.g. `scene_demo::Spin (speed: 0.8, direction: Clockwise)`); a component that cannot be written shows its error; unregistered ones are counted with a hint to register them.
  - The runner asks the scene registry (`Registry::describe`) for the selected entity and hands the overlay plain text, so `debug` and `render` never depend on `scene`. The scene example enables the overlay when built with the feature.
  - The "Entities" window now starts at the top right; under the stats window it did not fit in a 600-pixel-high window.
  - Phase P4 (debug overlay) is complete. Next phase: P5, scene editor, starting with its design decision (PP-038a).

- **2026-10-13: PP-037d1 Debug overlay entity inspector, part 1.**
  - A second overlay window, "Entities" (collapsed by default): the entity count, every entity by id with a one-line summary (what it draws and where), and for the clicked entity its engine components (transform, layer, hidden, screen space, velocity, quad, sprite, text, animation, button) plus a count of the game's own components. Read-only.
  - The list is virtualized; listing and summarizing cost 0.3 ms per frame for 5 000 entities in a debug build (0.025 ms release). The inspector's text comes from plain functions in `debug::inspect`, unit-tested on a headless world.
  - PP-037d was split: registered game components as RON are PP-037d2. No engine API or dependency change.

- **2026-10-13: PP-037c Debug overlay stats window.**
  - The overlay's placeholder is now a "PurplePie" window with eight rows: frame time (smoothed) and FPS, the last frame's time, fixed steps × step length, entity count, draw batches and instances, camera position and zoom, window size and DPI scale, and the cursor in window pixels and world units.
  - The runner gathers the numbers into a crate-private `Stats` snapshot each visible frame (the renderer adds the draw counts); formatting and gathering are unit-tested without a GPU or window.
  - Measured cost of the visible overlay in the sandbox (lavapipe): debug 58.8 → 43.4 fps, release 56.3 → 53.3 fps. No engine API or dependency change.

- **2026-10-12: PP-037b2 Debug overlay, part 1b: input routing.**
  - New optional dependency `egui-winit` 0.36.2 (`x11`, `wayland` only; +2 lock entries). While the overlay is visible, window events go to egui first: clicks, wheel turns and drags on its window stay with it; the game still gets every key, every click elsewhere, and the release of any button it saw pressed. Hidden, nothing changes.
  - Fixed before it could bite: PP-037b1 set egui's scale on the context, which egui-winit's own scale would have doubled on HiDPI screens; now the scale comes only from the input (unit test at 2×).
  - Verified in the sandbox under Xvfb (collapse arrow, wheel, drag-release, F12) with and without the feature; frames unchanged without it.

- **2026-10-12: PP-037b1 Debug overlay, part 1a.**
  - New optional Cargo feature `debug-overlay` (egui and egui-wgpu 0.36.2; needs Rust 1.95) and `EngineConfig::with_debug_overlay(true)`: a small "PurplePie" window drawn over the game in its own render pass, shown and hidden with F12. While the overlay is on, F12 never reaches the game. Without the feature, asking for it logs one warning.
  - On an sRGB surface the overlay draws through a non-sRGB view (`Bgra8Unorm` on the test machine), so egui's colours are exact and its warning is gone; egui animations are off so frames do not depend on timing.
  - Without the feature (or with the overlay off) nothing changes: no egui in the build, frames pixel-identical to before. CI's `msrv` job now builds without features. +30 `Cargo.lock` entries; +31 crates with the feature on Linux. Binaries built with the feature embed egui's fonts (OFL / Ubuntu Font Licence, R-26). PP-037b was split: input routing is PP-037b2.

- **2026-10-12: PP-037a Debug overlay design (ADR-043); phase P4 started.**
  - Decision: egui inside the engine behind an optional, off-by-default `debug-overlay` feature. The overlay is engine-owned and drawn in its own pass after the game's; it never spawns entities. Games turn it on with `EngineConfig::with_debug_overlay(true)` (from PP-037b), and F12 toggles it.
  - Measured in a throwaway spike on a scratch copy: egui 0.36.2 matches wgpu 30.0.1 and winit 0.30.13 exactly; +32 lock entries; release sandbox 12.8 → 18.9 MB and about 3 extra minutes of cold release build with the feature; egui needs Rust 1.95, so the feature will be excluded from the 1.90 `msrv` job. An egui window rendered over the sandbox frame on lavapipe.
  - Rejected: an own text/quad overlay (fallback if egui's coupling hurts), a `crates/` overlay crate (would need public wgpu/winit hooks), overlay entities in the game's world. New risk R-30. No repository code changed.

- **2026-10-12: PP-036d2 Purple Swarm best score and difficulty; phase P3.5 complete.**
  - Best score in `purple-swarm-best.txt` next to the executable (under `target/` for `cargo run`): shown on the title and game-over screens; a missing or broken file is 0; written through a temporary file and a rename; autoplay never touches it. 4 new unit tests (the game has 10).
  - Difficulty measured with a new `PURPLE_SWARM_AUTOPLAY=survive` mode: the bot used to survive 373 s (wave 19) because spawning stopped speeding up at wave 10; spawning now speeds up 20 % per wave (min 0.15 s) and Drifters by 10 units/s per wave, and the bot dies at 208 s (wave 11). Wave 1 is unchanged. New 60 s line `60.0 s; wave 4; score 475; kills 43; shots 151; hit points 5`.
  - P3.5 item 6 and the phase are complete (ROADMAP). Next phase: P4 debug overlay, starting with its design decision (PP-037a). No engine code change.

- **2026-10-09: PP-036d1 Purple Swarm screens.**
  - PP-036d was split: screens and restart now, best score and balance in PP-036d2.
  - Title ("Click to start"), pause (Escape; world and clock frozen, music muted; click resumes, Escape or Q quits) and game over (score and wave; click plays again). Restart clears the world and respawns the arena; per-round numbers live in one `Round` struct.
  - The screen flow is a small engine-free module with 3 unit tests (the game now has 6). Autoplay skips the title; its summary line is unchanged.
  - New observation U-20 (sounds can be muted but not paused). No engine code change.

- **2026-10-09: PP-036c2 Purple Swarm, part 2.**
  - The game is playable: Drifters and Dashers spawn on the arena edge in waves every 20 s; bullets, enemies, the player and health orbs collide with `circles_overlap`; 5 hit points with half a second of blinking invulnerability; score and wave in the HUD; game over stops the music and freezes the world.
  - Autoplay is deterministic (`Rng::new(2026)`) and prints `swarm: autoplay finished: 60.0 s; wave 4; score 415; kills 37; shots 133; hit points 5` (identical over three runs).
  - New observation U-19 (no `TextureId` for headless tests). No engine code change.

- **2026-10-09: PP-036c1 Purple Swarm, part 1 (ADR-042).**
  - Example games now live in `games/<name>/` as workspace members with their own `assets/`, using only the public API. The root package stays the only default member, so root commands are unchanged; CI uses `--workspace` in every job.
  - `games/purple-swarm`: tiled arena, walls, WASD/arrow movement, mouse aim, shooting with sound, a camera clamped to the walls, a HUD and an autoplay bot (`PURPLE_SWARM_AUTOPLAY=1`). Run it from its folder.
  - New observation U-18: `cargo run -p purple-swarm` from the root uses the engine's `assets/` (documented). No engine code change; `Cargo.lock` gained only the game's own entry.

- **2026-10-09: PP-036b Random numbers and overlap tests (ADR-041).**
  - New `math::Rng` (PCG32: `new(seed)`, `from_entropy`, ranges, `chance`, `pick`, `shuffle`, `unit_vec2`). The same seed gives the same numbers everywhere; verified against the PCG32 reference outputs.
  - New `math::Rect` (contains / overlaps / intersection / overlaps_circle / expand) and `math::circles_overlap`.
  - Additive, so 0.1.x-compatible; CHANGELOG `[Unreleased]`. The guide's game now uses both, and §4 mentions `World::clear`. No new dependency. 231 unit tests + 52 doctests.

- **2026-10-09: PP-036a Second game specified.**
  - New `docs/GAME2.md`: Purple Swarm, a top-down arena shooter. It lists the 0.1.0 features it uses and a gap table: random numbers and overlap tests become engine work (PP-036b); timers, screen switching with `World::clear`, the tiled floor, the camera clamp, the best-score file and screen shake stay in game code; gamepad is out of scope.
  - Skeleton built outside the repository from the published `tag = "v0.1.0"`. Release check: the tag on GitHub equals the PP-035b archive. No engine change.

- **2026-10-09: PP-035b Release 0.1.0 (P3.5 item 5 complete).**
  - `Cargo.toml`: `version = "0.1.0"`, `repository = "https://github.com/DavidVilela1/purplepie"` (`Cargo.lock`: only purplepie's own version line).
  - `CHANGELOG.md`: `[0.1.0] - 2026-10-09` and an empty `[Unreleased]`. README, guide section 1 and RELEASING now depend by git tag (`tag = "v0.1.0"`), with the path form as the alternative.
  - PP-033b closed: the owner reports the whole Windows checklist as expected, and all CI jobs (including `msrv` and latest-deps) green.
  - The owner pushes the tag (RELEASING §3).

- **2026-10-09: PP-035a Release preparation.**
  - New `docs/RELEASING.md`:
    - releases are git tags (no crates.io yet);
    - the 0.x compatibility policy: public API including the `#[non_exhaustive]` rules and the re-exported hecs 0.11 / glam 0.33 / serde 1, scene format 1, assets, MSRV 1.90, platforms;
    - the release procedure and the CI guards.
  - New `CHANGELOG.md`: `[Unreleased]` holds the 0.1.0 content and the pre-release breaking changes.
  - **CI:**
    - new `msrv` job (`cargo +1.90 check --locked`);
    - new `latest-deps.yml` workflow (`cargo update`, then check + test; on push, weekly, and on demand).

    Cowork cannot install Rust 1.90, so the `msrv` job is first verified on GitHub. The local evidence is in RISKS R-19.
  - No code change.

- **2026-10-09: PP-034b Scene component formatting and opt-in console logger (P3.5 item 4 complete).**
  - Registered game components are written as one spaced RON line (`(speed: 0.8, direction: Clockwise)`); `assets/scenes/demo.ron` regenerated (3 lines). Format version unchanged.
  - New `EngineConfig::console_log` / `with_console_log(true)`: `Engine::new` installs a stderr logger (crate-private `app::logging`, the sandbox's former logger) with the level from `PURPLEPIE_LOG` (empty/unset = `warn`, invalid = `Error::InvalidConfig`); does nothing if a logger exists. Sandbox, Breakout and scene examples opt in. Amends ADR-016. No new dependency.
  - 2 unit tests + 2 doctests (one in the guide): 220 + 49.

- **2026-10-09: PP-034a Second API review (ADR-040).**
  - **Breaking (pre-0.1.0):** `ScreenSpace::TOP`/`LEFT`/`RIGHT`/`BOTTOM` → `TOP_CENTER`/`CENTER_LEFT`/`CENTER_RIGHT`/`BOTTOM_CENTER`, and `ScreenAnchor::Top`/… → `TopCenter`/…, named like `TextAnchor`. Scene files write the new names and still read the old ones (format version 1 unchanged).
  - `Error`'s `Debug` prints the message plus a "Caused by:" chain, so `fn main() -> purplepie::Result<()>` reports failures readably; `Error::Save` now holds the resolved path.
  - `#[non_exhaustive]` on `EngineConfig`, `Sprite`, `Quad`, `Text`, `TextMetrics`, `SpriteAnimation`, `AnimationMode`, `TextureFilter`, `Camera2D`, `ui::Button`, `ui::Pointer` (constructors and builders unchanged; struct literals of these types are no longer possible outside the crate).
  - 2 new unit tests (218). No new dependency. U-12 (component formatting) and U-13 (console logger) decided, built next in PP-034b.

- **2026-10-09: PP-033a Platform checklist.**
  - New `docs/CHECKLIST.md`: what the owner runs on Windows (PowerShell) to see PurplePie on real hardware: tests and GPU tests, sandbox (drawing, input, sound, window), hot reload (restored with `git checkout`), Breakout play and autoplay lines, scene example, a new game crate from the guide outside the repository and OneDrive, and shipping it. macOS/Linux appendix. Each step says what to expect and what to report.
  - Every step was run on Linux first; the PowerShell blocks for steps 7–8 ran under PowerShell 7.4.6 for Linux (unpacked into `/opt/pwsh` in Cowork for this). No code change.
  - PP-033 split: PP-033a done, PP-033b (owner run) pending; PP-034 next; the release (PP-035) waits for PP-033b.

- **2026-10-09: PP-032b Module pages for game authors (P3.5 item 2 complete).**
  - `audio`, `ecs`, `input`, `math`, `render` and `ui` module docs now open with a one-line summary for game authors (the crate page's module list reads as a map), a paragraph naming the `Context` methods and the guide section, and a compiled example; engine internals and ADR references moved under "Engine notes". Doctests 43 → 47. No code or API change.
  - ADR-039: no starter-template crate in the repository; the guide's sections 1 and 14 are the template (a template crate inside the repository fails `cargo check` as a non-member of the workspace, verified in a scratch copy). Revisit at 0.1.0.

- **2026-10-08: PP-032a Newcomer guide.**
  - New `docs/GUIDE.md`: 14 short sections from `cargo new` to shipping, ending with a complete small game ("Collector"); it uses only assets copied from PurplePie's `assets/`.
  - `src/lib.rs` includes the guide into a `#[cfg(doctest)]` module, so `cargo test` compiles all 12 Rust blocks (doctests 31 → 43); the guide cannot drift from the API. No public API change.
  - README links the guide (Getting started, layout); the crate page mentions it. PP-032 split into PP-032a (done) and PP-032b (module docs, next).

- **2026-10-08: PP-031 Outside-crate trial (phase P3.5 started).**
  - A scratch game crate (`pie_catch`: catch falling pies; sprites, text, keyboard, a sound, a scene file with two registered game components) was written outside the package from the README and API docs only. It ran under Xvfb and shipped as a release build with its own `assets/`.
  - New `docs/USABILITY.md`: 17 findings (U-01…U-17): 8 doc fixes, 4 API issues, 1 both (logging), 2 missing features, 2 observations.
  - README: new "A new game crate" section (path/git dependency, serde for scene components, the game's own `assets/` and copying the bundled font, first-build time, `cargo doc -p purplepie --no-deps --open`, how a missing asset shows up, how to see engine logs with any `log` backend); the smallest-game example now says which two asset files it needs; the Scenes bullet names serde and the existing-folder rule.
  - Crate page: how a game crate is laid out and that `Context` is the entry point. `ecs` module: new doctest for queries with `Entity`, despawning after a query, and `query_mut`.
  - TASKS: follow-ups PP-032 (guide, next) to PP-036 recorded. No engine code or API change.

- **2026-10-08: PP-030 Workspace layout (phase P3 complete).**
  - ADR-038: a scratch trial measured what moving the engine into `crates/purplepie` would break (3 `include_bytes!`, 25 asset paths in tests, commands); decision: no move. The root `Cargo.toml` now declares `[workspace] members = []`, so later crates (debug overlay, editor) can join under `crates/`, and the project no longer fails to build when nested inside another Cargo workspace. No source, API, `Cargo.lock` or command change.

- **2026-10-08: PP-029 Asset hot reload, part 2: fonts and sounds.**
  - File-change detection moved to the shared crate-private `assets::watch` (also used by textures). Fonts reload under the same `FontId` and the renderer clears the glyph atlas; sounds reload under the same `SoundId` for new plays (ADR-037 extension). No public API change, no new dependencies.

- **2026-10-08: PP-028 Asset hot reload, part 1: textures.**
  - New `EngineConfig::hot_reload` / `with_hot_reload` (default off; ADR-037). The runner checks texture files' modification time and size every 0.5 s and reloads changed PNGs under the same `TextureId`; the renderer re-uploads them. Broken files keep the old texture (one warning). No new dependencies (`notify` measured and rejected).
  - The sandbox runs with hot reload on.
  - ROADMAP: new phase **P3.5 Usability and first release** between P3 and P4 (owner decision).

- **2026-10-07: PP-027 Scene files, part 2: game components.**
  - New `Context::register_scene_component::<T>(name)` (ADR-036): `T` needs serde's `Serialize` + `DeserializeOwned`, so serde now appears in one public trait bound; engine types still do not derive it.
  - Crate-private `src/scene/registry.rs`; scene version 1 gains the optional `components: { "name": … }` field. Unknown names or bad values fail the load before anything spawns. No new dependencies.
  - `examples/scene.rs` registers `Spin` (rotating squares) and `Visits` (a game-state entity); `demo.ron` regenerated (19 entities).

- **2026-10-07: PP-026b Scene files, part 1b.**
  - Scene version 1 gains optional `animation` (incl. playback position), `velocity` and `button` fields (ADR-035 extension); PP-026a files load and save back unchanged (fixture `src/scene/fixtures/demo_v1_pp026a.ron`).
  - Crate-private `SpriteAnimation::playback` / `with_playback` (validating restore); no public API change.
  - `examples/scene.rs`: animation saved mid-play, drifting square, "Click me" button, `PURPLEPIE_SCENE_FREEZE=1`; `assets/scenes/demo.ron` regenerated (18 entities). No new dependencies.

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

Executed in Cowork (Linux x86_64, Rust 1.95.0, Xvfb + Mesa lavapipe / llvmpipe, `libasound2-dev` installed) on 2026-10-13,
after the final PP-041 change (CI additionally runs the latest stable clippy and builds Windows/macOS, which Cowork cannot; R-19):

| Command / check | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ PASS |
| `cargo check --locked --all-targets --all-features` | ✅ PASS |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` (Rust 1.95) | ✅ PASS (0 warnings) |
| `cargo test --locked` (and `--workspace`) | ✅ PASS (engine without features: `cargo test` or `--workspace --exclude purplepie-editor`): 246 unit tests + 59 doctests (14 of them the guide's blocks) with 1 doctest ignored (the guide's editor program, compiled as an example of the editor crate), 12 unit tests ignored (GPU); with `--workspace --all-features` 262 + 60, 13 ignored; editor 24 + 4 doctests; Purple Swarm 10 |
| `cargo test --locked -- --ignored` (lavapipe) | ✅ PASS: 12/12; with `--all-features` 13/13 (the overlay pixel test) |
| Add/remove game components (PP-041) | ✅ GitHub `main` (`704a417`) identical to this tree before the task. clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc` with `--workspace --all-features` and without features (`-D warnings`). Xvfb with `scene_demo_editor`: Spin added to `#1` and removed from `#8` → saved diff exactly those; two undos → identical file; `#8`'s menu offered only Visits |
| P6 plan (PP-040) | ✅ GitHub `main` (`55bb6a5`) identical to this tree before the task; tag `v0.2.0` (annotated, `55bb6a5`) cloned and identical. Outside crate with GUIDE §15's dependency lines (`purplepie` and `purplepie-editor` at `tag = "v0.2.0"`) and its two files: built against the tag in 3 min 14 s with 0 warnings and one `purplepie` in its lock file; under Xvfb its game saved a level with a `my_game::Door` and its editor opened it ("1 game component value(s) checked against the game's types"). Documentation-only change otherwise (`cargo fmt --check`) |
| Release 0.2.0 (PP-039) | ✅ GitHub `main` (`290d87f`) identical to this tree before the task. Public items compared with a clone of the `v0.1.0` tag. clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc --workspace --all-features` (`-D warnings`); `cargo build`; sandbox under Xvfb prints `PurplePie sandbox v0.2.0`; R-19 `chunks_exact` grep empty; outside crate (GUIDE §15 files) by path builds against 0.2.0 |
| Guide section and P5 close-out (PP-038j2) | ✅ GitHub `main` (`4f0dc83`) identical to this tree before the task. Guide doctests 14 passed, 1 ignored (the editor program, compiled as an example); the guide-files test fails when the example differs from the guide (mutation). Outside crate built from section 15 word for word with both dependencies from GitHub `main` (see PP-038j2 in TASKS for the run). clippy `--workspace --all-targets --all-features`; `cargo doc --workspace --all-features` (`-D warnings`). CHECKLIST step 9's new example-editor check run under Xvfb |
| A game's own editor (PP-038j1) | ✅ GitHub `main` (`f31060d`) identical to this tree before the task. clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc` with `--workspace --all-features` and without features (`-D warnings`). Xvfb with the `scene_demo_editor` example: a string speed refused, a valid value accepted, the entity duplicated, saved; that file loaded strictly by the engine's `scene` example (20 entities, 3 spinning sprites); `count: -1` → red status and label; plain editor unchanged, `--help` exit 0, bad option exit 2 |
| Editor adding (PP-038i2) | ✅ GitHub `main` (`80e08de`) identical to this tree before the task (engine untouched). clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc --workspace --all-features` (`-D warnings`). Xvfb on a copy of `demo.ron`: Quad, Sprite (`textures/breakout/ball.png`), Text added → saved diff exactly three new entities with relative asset paths; three undos → identical file; missing `fresh.ron` → "new scene" message, quad + text saved, file reopens with 2 entities |
| Editor delete and duplicate (PP-038i1) | ✅ GitHub `main` (`b7f7c5c`) identical to this tree before the task (engine untouched). clippy `--workspace --all-targets --all-features`; `cargo doc --workspace --all-features` (`-D warnings`). Xvfb on a copy of `demo.ron`: duplicate + two deletes → saved diff shows exactly those; undo ×3 → identical file, redo ×3 → identical to the first save, undo ×3 → identical; delete + undo of the kept-component-only `#10` → identical |
| Editor inspector editing, part 2 (PP-038h2) | ✅ GitHub `main` (`8f70598`) identical to this tree before the task. clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc` with `--workspace --all-features` and without features (`-D warnings`). Xvfb on a copy of `demo.ron`: quad colour, sprite region, kept component (an invalid value refused in red), text content → saved file differs in exactly those four lines; four undos → identical file; anchor change + undo → identical file |
| Editor inspector editing, part 1 (PP-038h1) | ✅ GitHub `main` (`8b3796d`) identical to this tree before the task (engine untouched). clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc --workspace --all-features` (0 warnings). Xvfb on a copy of `demo.ron`: field edits saved as exactly those values; five undos → file identical, title clean; arrows in a field moved nothing |
| Scene editor: select, move, undo (PP-038g) | ✅ GitHub `main` (`9e02674`) identical to this tree before the task (engine untouched: only `crates/purplepie-editor` and docs changed). clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc --workspace --all-features` (0 warnings). Xvfb on a copy of `demo.ron`: click select, drag, merged arrow steps, undo/redo (Ctrl+Z, Ctrl+Shift+Z, Ctrl+Y), title `* demo.ron - …` then clean, click cycling, list selection, screen-space move refused, save identical after undoing; drag-and-save diff: one line (`#7` position) |
| Scene editor, part 1 (PP-038f) | ✅ GitHub `main` (`c84a821`) identical to this tree before the task. clippy `--workspace --all-targets --all-features` and, without features, `--exclude purplepie-editor` (+ `incompatible_msrv`); `cargo doc --workspace --all-features` (0 warnings); GPU tests 13/13. Xvfb with the editor binary on a copy of `demo.ron`: 19 entities, pan, zoom, fit, Ctrl+S → `cmp` identical; missing file → red cause, Ctrl+S creates an empty scene; broken file → save refused, file untouched; `--help` exit 0, bad arguments exit 2 |
| Lenient scene loading (PP-038e) | ✅ GitHub `main` (`38df37c`) identical to this tree before the task. clippy with no features (+ `incompatible_msrv`), `--features egui` and `--all-features`; `cargo doc` with and without features (0 warnings); GPU tests 13/13. Mutation (save only registered components) fails 4 tests. Scene example frozen frame 0 pixels different |
| Picking (PP-038d) | ✅ GitHub `main` (`8ed4aac`) identical to this tree before the task. clippy with no features (+ `incompatible_msrv`) and `--all-features`; `cargo doc` with and without features (0 warnings). New tests pass; mutation (ignore the transform) fails 2. Timing (temporary test, removed): 10 000 quads + 1 000 texts 34 ms debug / 1.3 ms release per call. GPU tests 13/13. Breakout lose frame 0 pixels different |
| Public egui hook (PP-038c) | ✅ GitHub `main` (`3e2d391`) identical to this tree before the task. clippy with no features (+ `incompatible_msrv`), `--features egui` and `--all-features`; tests in all three; GPU tests 13/13 (overlay pixel test through `layout` + `paint`). Mutation: dropping unpainted texture changes instead of merging fails `texture_changes_of_unpainted_frames_are_kept_for_the_next_paint`. Xvfb: `egui_panel` (`clicks 2, key frames 32, stamps 0, size 195, title "PurplePie egui panel X"`; window name followed the field), with `debug-overlay` overlay on top and F12 while typing; sandbox overlay routing and inspector as before (1 game click, zoom 1). Without features: Breakout lose frame and frozen scene frame 0 pixels different |
| Scene loading speed (PP-038b) | ✅ GitHub `main` (`68d5adc`) identical to this tree before the task. clippy with and without `--all-features` (+ `incompatible_msrv`); `cargo doc`. Timing test (temporary, removed) before/after in debug and release, 1 000 and 10 000 entities (ADR-044 implementation notes); new regression test passes with ron 0.12.1 (ratio 4.2) and fails with 0.12.2 (14.5× debug, 17.9× release). Scene example `load`, frozen, under Xvfb: 0 pixels differ from the frame built from GitHub `main` |
| Scene editor design (PP-038a) | ✅ GitHub `main` (`1ab36b1`) identical to this tree before the task; documentation-only change (no file under `src/`, `examples/`, `games/` or `Cargo.*` changed). Throwaway spike in a scratch copy ([spikes/pp-038a-editor-spike.md](spikes/pp-038a-editor-spike.md)): `Game::egui` hook + editor-like example under Xvfb (pick, drag, Ctrl+Z, save; saved file differs from `demo.ron` only in the moved entity's position); scene read/write timing (release, mean of 5): 1 000 quads parse 1.39 s, 9.3 ms with the whole scene parsed first; callgrind: 92 % in ron's `..` search. After the spike, engine artifacts cleaned (`cargo clean -p purplepie -p purple-swarm`) and every check above re-run: 231 + 53 (12 ignored), `--all-features` 244 + 53 (13 ignored), GPU 13/13 |
| Inspector game components (PP-037d2) | ✅ GitHub `main` (`cd8bdf0`) identical to this tree before the task. clippy with and without `--all-features` (+ `incompatible_msrv`); new unit tests: `Registry::describe` (working, failing, absent component), `inspect::game_rows`, singular summary. Overlay GPU test 13/13 with both windows excluded. Scene example with the feature under Xvfb: `#8` → `scene_demo::Spin (speed: 0.8, direction: Clockwise)` (same text as `demo.ron`), `#10` → `scene_demo::Visits (label: "demo scene", count: 1)`. Breakout lose frame and frozen scene frame pixel-identical without the feature |
| Debug overlay entity inspector (PP-037d1) | ✅ GitHub `main` (`86c5d04`) identical to this tree before the task. clippy with and without `--all-features` (+ `incompatible_msrv`); 3 new `debug::inspect` unit tests. Timing (temporary test, removed): 5 000 entities 0.30 ms (debug) / 0.025 ms (release) per frame, 50 000 entities 2.7 / 0.20 ms. Sandbox with the feature under Xvfb: Entities window expanded with 19 entities, `#5` selected shows its four engine components, a stamp adds `#19 quad at (260.0, -160.0)` (count 20), wheel over the list scrolls it (camera zoom stays 1), one left click reached the game. Breakout lose frame and frozen scene frame pixel-identical without the feature |
| Debug overlay stats (PP-037c) | ✅ GitHub `main` (`b53551a`) identical to this tree before the task. clippy with and without `--all-features` (+ `incompatible_msrv`); 3 new `debug` unit tests (timer, rows, gather). Sandbox with the feature under Xvfb: stats window shows cursor `(640.0, 360.0) px, world (0.0, 0.0)`, entities 19 → 20 and batches 8 → 9 after a click, zoom 1.00 → 4.00 after a wheel notch (the sandbox's exit line agrees). 600-frame timing, lavapipe: debug 58.8 fps without the feature vs 43.4 with the overlay visible; release 56.3 vs 53.3 |
| Debug overlay input routing (PP-037b2) | ✅ clippy with and without `--all-features` (+ `incompatible_msrv`). 5 `debug` unit tests incl. routing and the 2× scale test (fails with PP-037b1's `set_pixels_per_point` put back). Sandbox with the feature under Xvfb: overlay body and collapse-arrow clicks withheld (window collapsed, no stamp), wheel over the overlay withheld (0 `wheel` lines) and over the game delivered (zoom 4), drag-release over the overlay delivered (next click counted), F12 then a click where the overlay was: `left clicks seen: 4 in fixed_update, 4 in update`. Same session without the feature: 6 clicks, zoom 8. Breakout lose frame and frozen scene frame pixel-identical to the baselines without the feature and with it compiled in but off |
| Debug overlay part 1a (PP-037b1) | ✅ `cargo clippy --locked --workspace --all-targets` with and without `--all-features`, also `-W clippy::incompatible_msrv`; `cargo tree -e normal -i egui`: absent without the feature. Overlay GPU test (sRGB and non-sRGB views): fill within 2/255, rest untouched. Breakout lose frame pixel-identical to the PP-036 baseline and frozen scene frame pixel-identical to GitHub `main` (`bf98e71`), both without the feature and with it compiled in but off. Sandbox with the feature under Xvfb: `view_formats: [Bgra8Unorm]`, overlay shown → F12 hidden → F12 shown, no `key F12` in `Input`'s debug log, `A` delivered. Throwaway example asking for the overlay without the feature: one warning, exit 0. Both workflow files parse as YAML |
| Debug overlay spike (PP-037a; scratch copy, not merged) | ✅ `cargo info`: egui/egui-wgpu/egui-winit 0.36.2, `rust-version` 1.95. Resolver: wgpu 30.0.1 and winit 0.30.13 only (one version each). Scratch copy with the optional feature: `cargo build` without it unchanged; `Cargo.lock` +32 entries; `cargo build --release --bin sandbox` 1 min 4 s → 3 min 56 s with `--features debug-overlay`, 12 763 312 → 18 866 072 bytes; debug 204 900 112 → 305 093 912. Sandbox under Xvfb with the spike: egui window with `entities: 19` drawn over the frame; egui-wgpu warning about the `Bgra8UnormSrgb` surface. GitHub `main` (`0a50e14`) identical to this tree before the task |
| Purple Swarm best score + difficulty (PP-036d2) | ✅ GitHub `main` (`a35d4fe`, PP-036d1) byte-identical to this tree before the task. `cargo clippy --locked --workspace --all-targets --all-features -D warnings` (+ `incompatible_msrv`), `cargo test --locked --workspace` (game: 10 tests). `survive` autoplay: `208.4 s; wave 11; score 6550; kills 421; shots 1062; hit points 0` twice (before retuning: `373.3 s; wave 19`; at 15 %/wave: `308.7 s; wave 16`); 60 s line `60.0 s; wave 4; score 475; kills 43; shots 151; hit points 5` twice; no best-score file after four autoplay runs. Xvfb sessions: sweep-firing round → `New best! Score 20`, file `20`; idle round → `Score 0   Best 20`, file unchanged; fresh launch → `Best 20` on the title; file `garbage` → plain title, one warning, file untouched |
| Purple Swarm screens (PP-036d1) | ✅ `cargo clippy --locked --workspace --all-targets --all-features -D warnings` (+ `incompatible_msrv`), `cargo test --locked --workspace` (game: 6 tests). Autoplay summary unchanged: `60.0 s; wave 4; score 415; kills 37; shots 133; hit points 5` (two runs). Scripted Xvfb session: title frame → click → Escape: pause frames 1.5 s apart pixel-identical → click → idle until `swarm: game over at 11.5 s; …` (pause not counted), `HP 0/5` red → click: fresh round (`Score 0 Wave 1 0:01`, `HP 5/5`, no enemies) → Escape → Q: exit 0. Escape on the title: exit 0 |
| Purple Swarm part 2 (PP-036c2) | ✅ `cargo clippy --locked --workspace --all-targets --all-features -D warnings` (+ `incompatible_msrv`), `cargo test --locked --workspace` (game: 3 tests). Autoplay under Xvfb from `games/purple-swarm`: summary `60.0 s; wave 4; score 415; kills 37; shots 133; hit points 5`, identical over three runs; frames at 30 s / 50 s show an enemy, bullets, `Score 160 Wave 2 0:30` / `Score 365 Wave 3 0:50` and `HP 5/5`. Idle manual run: `swarm: game over at 9.9 s; wave 1; score 0; …`, frame shows the banner and `HP 0/5`. ALSA capture (25 s): continuous music with louder effect transients |
| Purple Swarm part 1 (PP-036c1) | ✅ `cargo fmt --all --check`, `cargo check/clippy --locked --workspace --all-targets --all-features` (+ `incompatible_msrv`), `cargo test --locked --workspace` pass. Under Xvfb from `games/purple-swarm`: asset root `…/games/purple-swarm/assets`; autoplay frame after ~8 s shows `Shots 62`, bullets, tiled floor, bottom wall with the camera clamped; manual run (mouse at top right, W + left button held 1 s) shows `Shots 8` and the camera moved. From the repository root: `Error: failed to load asset …/PurplePie/assets/textures/floor.png` (U-18). Both workflow files parse as YAML |
| Rng + Rect (PP-036b) | ✅ PCG32 reference sequence for seed 42 matches (`a15c02b7 7b47f409 ba1d3330 83d2f293 bfa4784b cbed606e`); mutating the stream constant fails the test. Guide §14 game rebuilt as an outside crate (path to this tree): clippy clean; Xvfb: bricks placed by `Rng::new(7)`, moving right into one brick gives `Score 1` and a respawn. `clippy -W clippy::incompatible_msrv` clean |
| Second game spec (PP-036a) | ✅ GitHub tag `v0.1.0` (`bf03aeb`) cloned: identical to this tree before PP-036a; a crate with `purplepie = { git = …, tag = "v0.1.0" }` builds and prints `purplepie 0.1.0`. Purple Swarm skeleton from that tag: 0 warnings, clippy clean; Xvfb frames before/after moving (player turns to the cursor, camera follows, HUD drawn); console logger shows asset root and GPU |
| Release 0.1.0 (PP-035b) | ✅ `cargo check --locked` with version 0.1.0; sandbox prints `PurplePie sandbox v0.1.0`. GitHub `main` (`e3abead`, PP-035a) byte-identical to the PP-035a archive. A fresh crate with `purplepie = { git = "https://github.com/DavidVilela1/purplepie", branch = "main" }` fetched, built and ran the guide game (frame identical to the PP-032a guide frame); the same crate by path against this 0.1.0 tree built and ran (only the animated corner differs). The tag form itself is checked after the owner pushes `v0.1.0` |
| Release preparation (PP-035a) | ✅ scratch copy with `cargo update` (12 packages newer, e.g. hecs 0.11.2, glam 0.33.12): `cargo check --all-targets --all-features` and `cargo test --all-features` pass (220 + 49); committed `Cargo.lock` unchanged. `cargo clippy --locked --all-targets -- -D warnings -W clippy::incompatible_msrv` clean at `rust-version` 1.90, and it fires at 1.85 (mutation). Highest dependency `rust-version` in the lock file: 1.90. Both workflow files parse as YAML. Rust 1.90 itself could not be installed (blocked download) |
| Component formatting + console logger (PP-034b) | ✅ scene example `save` rewrote `demo.ron` (only the 3 component lines changed); `save`/`load`/`build` frozen frames pixel-identical to the PP-026b baseline. Breakout with `PURPLEPIE_LOG=info` prints `asset root:`, `audio:` and `GPU:` lines, without it no log lines; autoplay `Lost after 892 …`, lose screen pixel-identical. Sandbox with `PURPLEPIE_LOG=loud` exits 1 with `invalid engine configuration: PURPLEPIE_LOG must be …`. Outside trial crate (no opt-in) prints no log lines with `PURPLEPIE_LOG=info` |
| API review (PP-034a) | ✅ trial crate `pie_catch` (outside crate): old `ScreenSpace::TOP` fails to compile (expected), its pre-rename `level.ron` (`screen_space: Top`) loads with the title centred; missing `pie.png` prints ``Error: failed to load asset `…/pie.png` `` + `Caused by: No such file or directory (os error 2)`; save into a missing folder names the full resolved path. Scene example frozen frame and Breakout lose screen pixel-identical to their baselines; `Lost after 892 fixed steps …` unchanged |
| Platform checklist (PP-033a) | ✅ all steps on Linux: sandbox frame with `GPU:`/`audio:` log lines, click + `M` music in an ALSA capture; hot reload swap and `git`-style restore (file byte-identical); autoplay `Lost after 892 …` / `Won after 7135 …`; scene: 19 entities, 2 button clicks counted, build mode; steps 7–8 run as PowerShell 7.4.6 scripts: new crate builds with 0 warnings, release build shipped and run from `/` shows the same frame (only the animated corner differs) |
| Module pages (PP-032b) | ✅ `cargo doc --no-deps` (`-D warnings`) clean; rendered crate page lists the six modules with the new game-author summaries; the 4 new module examples pass as doctests |
| Starter template trial (ADR-039) | ✅ reproduced: a `templates/starter` crate inside a scratch copy of the repository fails `cargo check` with "current package believes it's in a workspace when it's not" |
| Guide (PP-032a) | ✅ `cargo test --doc guide`: 12/12; a deliberate `KeyCode::KeyA` in the guide fails its block; CRLF copy of the guide also passes. Fresh outside crate following the guide word for word (section 1 `Cargo.toml` + asset folders, section 14 game): no warnings, clippy clean; Xvfb: first frame correct, a brick collected (score 1, blip in ALSA capture), Restart click reset score and position |
| Outside-crate trial (PP-031) | ✅ README example copied verbatim into a fresh crate: built with no warnings, failed only on the two assets the README did not mention (now documented); trial game `pie_catch` (path dependency, own `assets/`): scene save + load (5 entities, 2 registered components), autoplay frame (score 5), game over and Enter restart frames checked, catch sounds present in an ALSA `file` capture; release build + `assets/` copied to a new folder ran from working directory `/` |
| README re-check (PP-031) | ✅ fresh `cargo new my_game` beside a PurplePie folder, following the fixed README word for word: builds and runs (sprite, bar and text in the screenshot); `cargo doc -p purplepie --no-deps` 1 min from a cold target directory |
| Workspace declaration (PP-030) | ✅ `cargo check/test` also with `--workspace`; `Cargo.lock` byte-identical; `cargo metadata` workspace root = repo root, sole default member `purplepie`; a copy nested in a folder with its own `[workspace]` failed to build before and builds after |
| Font hot reload end to end (sandbox, Xvfb) | ✅ replacing `Poppins-Regular.ttf` with DejaVu Sans changed the help label and HUD text within 1.5 s; a broken font file changed nothing (one warning); restoring the file restored the exact original pixels (earlier run; no engine code changed since) |
| Sound hot reload end to end (sandbox, ALSA `file` plugin) | ✅ the click before replacing `blip.wav` with `hit.wav` played exactly `blip.wav`, the click after it exactly `hit.wav` (max diff 0 against the mixer model; L = R) (earlier run; no engine code changed since) |
| `grep -rnE "chunks_exact(_mut)?\([0-9]" src examples` (R-19 guard) | ✅ no matches |
| Scene example (`PURPLEPIE_SCENE_FREEZE=1`, modes `save`, `build`, `load`) under Xvfb | ✅ the three 1024×600 frames are pixel-identical, and identical to PP-026b's frame; `save` wrote `assets/scenes/demo.ron` (19 entities, 2 game component types), which the unit tests read back byte-for-byte (earlier run; no engine code changed since) |
| Scene example unfrozen (load) | ✅ the two squares loaded with `Spin` rotate between screenshots, the static text does not; prints `demo scene loaded 1 time(s)` and `2 spinning sprites` (earlier run; no engine code changed since) |
| Scene mutation check | ✅ not inserting decoded game components fails 3 tests (earlier run; no engine code changed since) |
| `cargo doc --no-deps` (`-D warnings`) | ✅ no warnings |
| `cargo build --locked`, `cargo build --release --example breakout` | ✅ PASS |
| Sandbox whole-frame model + HUD (camera (0,0)×1) | ✅ 0 mismatches; HUD panel, corner square and button crops identical to the PP-025 run (earlier run; no engine code changed since) |
| Breakout autoplay `win` / `lose` (debug, ALSA null device) | ✅ unchanged: `Won after 7135 fixed steps … score 220`; `Lost after 892 fixed steps … score 14` (earlier run; no engine code changed since) |
| Breakout lose screen vs PP-025 | ✅ pixel-identical (earlier run; no engine code changed since) |
| Window destroyed (fresh display) | ✅ `error: GPU rendering failed` / `surface was lost`, exit 1, no panic (earlier run; no engine code changed since) |

Owner-provided (not executed by Claude): Windows x64: the Stage 4 purple window was confirmed by screenshot (pixel-checked); `cargo test`, Escape and the close button confirmed on 2026-10-01. GitHub Actions: first run all green on 2026-10-01. Windows, 2026-10-09 (PP-033a/PP-034a trees, `docs/CHECKLIST.md`): step 1 `cargo test` 216 unit tests + 47 doctests passed, 0 failed, 12 ignored; step 2 `cargo test -- --ignored` 12/12 passed on a real GPU, the first hardware run of the pixel-exact GPU tests; step 3 `GPU: AMD Radeon(TM) Graphics (Vulkan, IntegratedGpu); surface Bgra8UnormSrgb, AutoVsync` (laptop on battery, so the RTX 3060 was idle) and `audio: 2 channels at 48000 Hz (f32)`; the owner reports everything else working. Later the same day the owner reported the remaining steps (4–8) "as expected" (exact autoplay lines not quoted) and all CI jobs green, including `msrv` (Rust 1.90) and latest-deps. Windows, 2026-10-14 (PP-038j2 tree, before the 0.2.0 release): the owner reports every checklist step green, including step 9 (debug overlay, `egui_panel`, scene editor, example editor), all tests green and all CI jobs green. From then on, owner-run checks are assumed green unless the owner reports a failure (DEVELOPMENT §3).
Stages 5–10 and text (PP-018a/b) on Windows have not been seen yet.

## Last Updated

2026-10-15. PP-041 done (editor adds and removes game components). PP-042a (Pie Heist skeleton and its editor) is next.
