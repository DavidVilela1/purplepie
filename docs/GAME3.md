# Third game: Pie Heist (phase P6)

P6 "Proof" (ROADMAP): a bigger example game **whose levels are made with the scene editor**. Breakout builds its
level in code, and Purple Swarm generates its arena. Pie Heist is the first game where the content lives in scene
files that a designer draws in the editor (`crates/purplepie-editor`, ADR-044) and the game only loads. It answers
whether the editor and the scene format are good enough to build a real game's levels. Each gap below is either game
code or a small engine or editor task. Planned in PP-040 (2026-10-15) against release 0.2.0.

## The game

- **Goal:** a top-down heist through a building, one level per scene file. Collect every pie in the level, then
  reach the exit to go to the next level. After the last level, a "You win" screen shows the time and the number of
  times the player was caught.
- **Player:** moves with WASD or the arrows at 220 units/s and stops at walls (it slides along them). The camera
  follows the player inside the level's bounds.
- **Walls:** solid, axis-aligned rectangles (quads). Rotated walls are not supported: the game says so when it loads
  one.
- **Pies:** collected by touching them. The exit stays locked (dimmed) until every pie in the level is collected.
- **Keys and doors:** coloured red, blue or gold. A key opens every door of its colour; doors are walls until then.
- **Guards:** walk a patrol path (a list of points, looping) at their own speed. Touching a guard restarts the level
  (pies and keys come back) and adds one to "caught".
- **Hints:** text in the level, placed in the editor (a plain `Text` entity).
- **Screens:** title ("Click to start"), play (HUD: pies left, level, time, caught), level complete (a short banner),
  win. Escape pauses; Escape again or Q quits.
- **Sound:** pie, key, door, caught, exit (short WAVs); music optional.
- **Size:** five levels, each bigger than the window (about 2 000 × 1 500 units), built only in the editor.

## How the levels are made

Each level is `games/pie-heist/assets/levels/levelN.ron`, a normal scene file (ADR-035). The game's components are
registered under `pie_heist::…` names, and the game's own editor checks them while editing (`GameComponents`,
PP-038j1):

| Component | RON in the editor | On |
|---|---|---|
| `Wall` | `()` | quads that block movement |
| `Pie` | `()` | sprites to collect |
| `Key { color }` | `(color: Red)` | sprites |
| `Door { color }` | `(color: Red)` | quads (block until opened) |
| `Exit { next }` | `(next: "levels/level2.ron")`, `""` on the last level | a sprite |
| `Guard { path, speed }` | `(path: [(0.0, 0.0), (300.0, 0.0)], speed: 90.0)` | sprites; the path is relative to the guard's position |
| `PlayerStart` | `()` | one entity per level (a sprite the game hides) |

Everything else in a level is ordinary engine data the editor already edits: positions, sizes, colours, sprites,
layers, texts.

## Engine and editor features it uses (all in 0.2.0)

| Need | PurplePie 0.2.0 |
|---|---|
| Levels as files, with the game's components | `load_scene`, `register_scene_component` (ADR-035/036) |
| Building and checking levels | the game's own editor: `purplepie_editor::main_with(GameComponents…)` (GUIDE §15) |
| Switching levels, restarting | `World::clear` + `load_scene` |
| Collisions (player ↔ walls, pies, keys, doors, guards, exit) | `math::Rect`, `circles_overlap` (ADR-041) |
| Camera follow and clamp to the level | `Camera2D`, `visible_world_rect` |
| HUD, banners, buttons | `Text`, `ScreenSpace`, `ui::Button` |
| Sounds | `play_sound`, `loop_sound` |
| Inspecting a running level | the debug overlay (F12, `debug-overlay` feature) |

## Gaps and decisions

| Gap | Where it goes | Why |
|---|---|---|
| **The editor cannot add or remove a game component** (it edits only the ones a file already has). A new quad cannot become a `Wall`; a designer would have to duplicate an existing wall. | **Editor (+ a small engine helper): PP-041, done** (2026-10-15: "Game component: Add…" with `with_default`, an "x" to remove; `scene::component_text`) | Making levels from scratch is the point of P6. An "Add component" menu needs a default value for each of the game's types, as RON text: `GameComponents` would take `T: Default + Serialize` for that, and the engine would write the text exactly as `save_scene` does (the editor has no `ron` dependency, on purpose: R-31). Removing is `UnknownComponents::remove` plus an undo step. |
| Wall collision response (stop and slide) | Game code | Axis-separated push-out against `Rect`s, a few dozen lines. Walls are axis-aligned by rule, so no rotated boxes. Not engine physics (out of scope for PurplePie). |
| Level bounds for the camera clamp | Game code | The union of the wall rectangles, computed when the level loads. |
| Grid snapping in the editor (aligning maze walls) | **Decided in PP-042a: not now** | Level 1's walls were placed by typing four numbers each (position x/y, size x/y) and came out exact; no wall needed dragging. Snapping only matters when walls are placed with the mouse. Revisit in PP-042d if drawing levels 2–5 by mouse is slow. |
| Seeing guard paths in the editor | Not planned | The path is RON text relative to the guard; the designer reads it in the inspector. Revisit only if building levels 2–5 shows it is the slowest part. |
| A test that every shipped level loads, with one `PlayerStart` and one `Exit` | **Decide in PP-042d** | Games cannot load a scene without a running engine (`load_scene` is a `Context` method). Either a timed Xvfb run like Purple Swarm's autoplay, or a small engine API to read a scene file's entities without a window. The choice is made when the five levels exist. |
| Changing a sprite's texture in the editor | Not planned | Delete and add again with the right image. Revisit if it shows up while building levels. |
| Art | Game assets | Small PNGs made for the game (as Purple Swarm's) plus quads; the font and sounds copied from the engine. |

## Where it lives

`games/pie-heist/`, a workspace member like Purple Swarm (ADR-042), with `purplepie = { path = "../.." }`:

- `src/lib.rs`: the components and the level rules, so they can be unit-tested and shared;
- `src/main.rs`: the game;
- the game's editor in its own small crate, `games/pie-heist-editor/` (depends on `pie-heist` for the types and on
  `purplepie-editor`), as GUIDE §15 recommends, so the game itself is built without egui and keeps Rust 1.90. The
  CI `msrv` job then excludes `pie-heist-editor` too.

This is one more workspace member of the kind ADR-042 already allows, plus a second editor crate; it gets a note
in ADR-042 (not a new ADR) in PP-042a.

## Building level 1 (PP-042a)

Level 1 (`assets/levels/level1.ron`) was drawn with `pie-heist-editor` on an empty file, driven under Xvfb: five
quads (an 820 × 520 room and an inner wall with a gap at the bottom), each given its position and size in the
inspector and `pie_heist::Wall` from "Game component: Add…", and a sprite with `textures/start.png` given
`pie_heist::PlayerStart`. Saving wrote exactly those six entities. Findings:

- **Fixed in the editor:** "Add → Quad" made a quad in `Color::PURPLEPIE`, the engine's default background, so it was
  invisible on a new scene. New quads are now light lavender (`#E0D7F5`).
- Typing exact numbers was quick and precise; no grid snapping needed (see the gaps table).
- The inspector rows move down as the entity list grows (until the list reaches its maximum height); harmless for a
  person, but scripted checks have to compute positions from the entity count.

In the game, the player starts at the hidden start marker, stops at the inner wall (touching it), slides along
the floor and walks under the inner wall to the other side; the camera stays centred because level 1 is smaller
than the window.

## Guards (PP-042c1)

A guard's path is typed in the editor's field as RON, relative to where the guard is placed: level 2's guard at
(-80, 0) has `(path: [(0.0, -160.0), (0.0, 160.0)], speed: 160.0)`, so it walks up and down across the way to the
door, a round every 4 s. A path loops: after its last point the guard walks back to the first, so two points are a
back-and-forth walk. Guards walk through walls (the designer keeps paths inside rooms); they start again from the
first point whenever the level restarts. Typing the path was quick for a straight walk; a longer path is a list of
numbers that is only seen in the game (see "Seeing guard paths in the editor" above, still not planned).

## Plan

1. **PP-041 (done):** editor: add and remove game components (with defaults from the game's types), each one undo step;
   the engine helper that writes a component as scene RON. Updates GUIDE §15 and the example editor.
2. **PP-042a (done):** `games/pie-heist` skeleton and `games/pie-heist-editor`: components registered, level loading,
   player movement with wall collision, camera; level 1 drawn in the editor; the grid-snapping decision.
3. **PP-042b:** split in two:
   - **PP-042b1 (done):** pies, the exit and level progression, the HUD and two sounds; level 1 completed and level 2
     made in the editor; both played through by a script;
   - **PP-042b2 (done):** keys and doors; level 2 got a gold door before the exit and a gold key, in the editor.
4. **PP-042c:** split in two:
   - **PP-042c1 (done):** guards walk their paths (`patrol_offset`); touching one restarts the level and counts as
     caught; level 2 got a guard in the editor, its path typed as RON;
   - **PP-042c2:** title, pause, level-complete and win screens, "Play again", the time in the HUD.
5. **PP-042d:** levels 2–5 built in the editor; the level-check decision and its test; a balance pass.
6. **PP-042e:** findings from building the levels (what the editor made easy or slow), CHECKLIST step, README and
   CHANGELOG; phase P6 complete. A 0.3.0 release follows if engine or editor APIs changed.

Each step is split again at its start if it does not fit one session.
