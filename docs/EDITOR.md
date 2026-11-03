# The engine editor (phase P7)

P7 makes PurplePie's scene editor (`crates/purplepie-editor`, ADR-044) the engine's development interface: one
window from which a game is built. You open a game's project, browse its assets, open, create and save its scenes,
run the game, and read what the game and the editor report. The owner chose this direction on 2026-11-03, over
improving only the scene editor, the debug overlay or the code API. Planned in PP-043 against the editor as it was
after P6's level work. The decision is recorded in **ADR-046**.

It stays a small tool on the engine's public API, not an editor framework: no plugin system, no scripting, no
docking layout, no asset pipeline. Each feature below exists because building Pie Heist's levels needed it, or
because running and checking a game should not need a terminal.

## Where the editor is today (end of P5, plus PP-041)

- **One scene per run:** the file comes from the command line, and there is no New, Open or Save As. Switching scenes
  means closing the window and starting again.
- **Assets:** the asset folder is found from the scene (the nearest folder named `assets`). Images and fonts are
  listed only inside the Add → Sprite and Add → Text menus.
- **Messages:** one status line, also printed to the terminal.
- **Running the game:** not possible from the editor. A level is tried with `cargo run` in another terminal.
- **A game's own types:** a game's editor crate gives them with `main_with(GameComponents…)`. They are checked, and
  can be added to entities (PP-038j1, PP-041).
- **Findings from drawing four Pie Heist levels (GAME3.md):**
  - typing exact numbers is quick;
  - guard paths and door colours cannot be seen in the editor;
  - the inspector's place moves as the entity list grows;
  - each level took several editor runs.

## What the engine editor is

| Area | What it does | Why |
|---|---|---|
| **Project** | The project is the folder holding the asset folder (for `games/pie-heist/assets/levels/level1.ron`, the project is `games/pie-heist`). No project file: the editor finds it from the scene given, or from `--project <folder>`. | Every game here is already laid out this way (ADR-025, ADR-042), so no new file to keep in sync. |
| **Scenes** | New, Open and Save As inside one run. A changed scene asks "Save / Don't save / Cancel" before another is opened or the window is closed. One scene is open at a time (no tabs), and undo history belongs to the scene. | Building a game means moving between its levels. |
| **Project panel** | The asset folder as a tree: scenes (`.ron`), images, fonts, sounds. A click on a scene opens it, through the save question. Refreshed when files change on disk. | Replaces typing paths on the command line, and shows what a game ships. |
| **Console** | The editor's messages with their time, warnings and errors coloured, plus the running game's output. The status line stays, showing the latest message. | One place to read what happened, without a terminal. |
| **Run** | A Run button (F5) saves the scene, then starts the game as a **separate process** with the open scene. Its output goes to the console, a Stop button ends it, and its exit status is reported. The command is set by the project: by default `cargo run --` in the project folder, followed by the scene's path relative to the asset folder (as Pie Heist accepts it). A game's editor crate sets its own command in code; the plain `purplepie-editor` binary takes `--run <command>`. | Trying a level is the most common thing a level designer does. |
| **Game hints in the view** | A game's editor can draw simple shapes over the view for its own component types, from their values: for example a guard's patrol as a line, or a door in its key's colour. They are lines, rectangles and labels in world units, drawn by the editor. | The two largest gaps found while building Pie Heist. |
| **Image previews** | The project panel and the sprite picker show small previews of images. | Choosing textures by file name alone is slow. |

### Not planned

These become tasks only if building games shows a need:

- running the game inside the editor's window (see ADR-046);
- tabs or several scenes open at once;
- docking or moving panels;
- grid snapping (GAME3.md: typing numbers was enough);
- multi-select;
- an asset import pipeline;
- editing sounds or fonts;
- a project file;
- a plugin API.

## Engine and editor work it needs

| Need | Where | Notes |
|---|---|---|
| Opening another scene in one run | Editor | `World::clear` then `load_scene_keeping_unknown`, as Pie Heist switches levels. History, selection and pending edits reset per scene. |
| Asking before the window closes | **Engine (likely)** | Checked in PP-043: the runner exits on the window's close request (`WindowEvent::CloseRequested`), so no game can refuse it. PP-044a decides: a small engine API letting a game see and decline the request (an ADR-008 note), or the editor saving a recovery copy instead. |
| Reading a project's files | Editor (`library.rs`, std) | Already lists images and fonts. It grows to scenes and sounds, and a tree. |
| Running the game, reading its output | Editor (std `process` + a reader thread per pipe, a channel read each frame) | No new dependency (ADR-013). The PP-043 spike: `cargo build -p pie-heist` while `cargo run -p pie-heist-editor` was running was not blocked by Cargo's build lock (Linux). Windows behaviour is checked by the owner when the task lands (R-32). |
| Game hints | Editor (`GameComponents` learns a drawing function per type; values read with `scene::check_component`'s decoding) | The editor already decodes typed values to check them. The API is designed in its own task (an ADR-044 note). |
| Image previews | Editor (egui textures) | Needs the image's pixels. Decide in the task: decode the PNG in the editor (a new dependency) or a small engine API giving a loaded texture's RGBA. |
| Splitting `editor.rs` (1 477 lines) | Editor | Each new panel goes in its own module; the inspector and the view move out as they are touched. |

Two engine API changes are possible: handling the window's close request (PP-044a) and texture pixels for previews (PP-044f). Anything else that turns out to need
the engine gets an ADR note when its task starts.

## Plan

Each item is one task, split again at its start if it does not fit one session:

1. **PP-043 (done):** this plan, ADR-046, P6 parked.
2. **PP-044a:** scenes in one run: New, Open (a path field, with paths relative to the project), Save As, and the
   "Save / Don't save / Cancel" question before switching scenes or closing; the window title shows the scene and
   whether it has unsaved changes.
3. **PP-044b:** the project panel: the asset folder as a tree, click a scene to open it, refresh on changes.
4. **PP-044c:** the console panel: the editor's messages with time and level; the status line shows the latest.
5. **PP-044d:** Run and Stop: the project's run command, the game's output in the console, exit status,
   `--run`, and the code option for a game's editor; Pie Heist's editor configured.
6. **PP-044e:** game hints in the view (API and an ADR-044 note); Pie Heist's editor draws guard paths and door
   colours.
7. **PP-044f:** image previews (decide decoding in the editor or an engine API).
8. **PP-044g:** GUIDE §15 rewritten for the engine editor, a CHECKLIST step, README, CHANGELOG. Phase P7 complete,
   then release 0.3.0 (which also carries `Headless`, `component_text` and `GameComponents::with_default`).

## P6 is parked

Pie Heist has four of its five levels. Every mechanic and screen is done, and a test checks every level. Level 5, the
balance pass (PP-042d2c) and P6's wrap-up (PP-042e) are parked, not dropped. They resume when the owner asks, and
the engine editor will make drawing level 5 easier.
