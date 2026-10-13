# PP-038a spike: scene editor on an egui hook

Date: 2026-10-13. Throwaway code in a scratch copy of the repository at commit `1ab36b1` (PP-037d2); nothing was
merged. Linux, Rust 1.95, lavapipe under Xvfb. Decision: [ADR-044](../DECISIONS.md).

## Questions
1. Can a crate outside the engine build an editor as a `Game` if the engine hands it egui each frame?
2. Is undo by whole-scene snapshots (save to text, reload) cheap enough, or are commands needed?

## 1. The egui hook

**Change in the scratch copy** (about 60 lines, all behind `debug-overlay`):
- `Game::egui(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui)` with an empty default, and `pub use egui;`.
- The runner fed every window event to egui (not only while the overlay was visible), ran `Context::run_ui` itself
  after `update`, called the hook inside it, and stored the `FullOutput`. The renderer painted that output in the
  overlay pass. The overlay's own windows were left out of this pass for the spike.

A first version handed the game an `&egui::Context`; it did not compile against egui 0.36, whose panels take a
`&mut Ui` (`egui::Panel::left(..).show(ui, ..)`). The hook must run inside `run_ui` and pass the root `Ui`.

**Example** `examples/editor_spike.rs` (about 200 lines, public API only): registers the scene example's `Spin` and
`Visits`, loads `scenes/demo.ron`, shows a left panel (Save, Undo, entity list), an interaction area over the rest of
the window (click to pick, drag to move, `drag_delta` divided by the camera zoom, Y flipped), an outline of the
selection drawn with `ui.painter()` from `Camera2D::world_to_screen`, and a command stack of moves.

**Run** (1024 × 600 window, xdotool): the scene was drawn under the panel; a click picked an entity, a drag moved it
by (89, −191), Ctrl+Z restored (0, 0), a second drag moved it by (0, −241), and Save wrote a file whose only
difference from `demo.ron` was that entity's `position` line.

Findings:
- **Picking:** the naive rule (highest `Layer` whose box contains the point) picked entity #4, the demo's
  2 000 × 2 000 `Hidden` quad on layer 100, wherever one clicked. Picking must skip `Hidden` and `ScreenSpace`
  entities and follow the draw order (layer, material, entity; ADR-021), with rotation, scale and text bounds.
- **Drag start:** egui reports `drag_started` after the pointer has moved a few pixels, so picking at
  `interact_pointer_pos` then uses a point a few pixels from the press; use the press origin.
- **Shortcuts:** `i.modifiers.command && i.key_pressed(Z)` missed a fast Ctrl+Z (xdotool pressed and released Ctrl
  within one frame, so the frame's final modifiers had no Ctrl). `i.consume_key(Modifiers::COMMAND, Key::Z)` uses each
  event's own modifiers and worked. The raw events arrived correctly.
- The panel covers the left of the scene: the editor must shift the camera to centre the scene in the free area.

## 2. Snapshot cost and a parser problem

A test in the scratch copy built N entities (half quads on a layer, half sprites with a texture region and a
registered game component), then timed `capture` + `to_text` and `from_text` + `resolve_assets` +
`decode_components` + `spawn` (release build, mean of 5).

| Case | Text | Write | Read |
|---|---|---|---|
| 250 mixed | 85 KB | 2 ms | 143 ms |
| 1 000 mixed | 340 KB | 5.5 ms | 2.14 s |
| 1 000 quads only | 253 KB | 3.7 ms | 1.39 s |

Reading is quadratic, and almost all of it is `from_text` (resolve, decode and spawn together were under 2 ms).
Callgrind (150 quads) put 92 % of all instructions in `core::str::pattern::TwoWaySearcher::next`, called from ron's
`deserialize_any`. Cause: ron 0.12.2 (the latest release, 2026-06-22), `Parser::next_bytes_is_float`, runs
`self.src()[skip..].find("..")` over the **whole remaining input** for every number whose type is not known in
advance, and only then clamps the result to the number's length. PurplePie reaches it in two places:
- `from_text` first parses `VersionOnly`, which skips the rest of the file with serde's `IgnoredAny`, so every number
  in the file goes through `deserialize_any`;
- game component values are kept as `ron::value::RawValue`, which also reads through `deserialize_any`.

ron's main branch already limits the search to the number (a comment there describes this exact quadratic case); it
is not released yet.

Trying the first fix (parse the whole `SceneFile` first, and parse `VersionOnly` only if that fails, to explain a
wrong version):

| Case | Read before | Read after |
|---|---|---|
| 1 000 quads only | 1.39 s | 9.3 ms (parse) / 10.0 ms (all) |
| 1 000 mixed | 2.14 s | 139 ms |
| 10 000 mixed | not run (minutes) | 12.5 s |

The remaining time in the mixed cases is the game components' raw values, which only a ron release fixes.

**Commands instead:** moving a component's position and back cost about 40 ns. A snapshot undo of 1 000 entities
costs about 13 ms with engine components only (after the fix) and about 150 ms with game components, and respawns
every entity with new ids.

## Outcome
ADR-044: a separate `crates/purplepie-editor` crate written as a `Game` on a public `Game::egui` hook behind an `egui`
feature; undo as a command stack; engine tasks for the parser fix (PP-038b), the hook (PP-038c), picking (PP-038d) and
lenient loading (PP-038e) before the editor slice (PP-038f, PP-038g).

## Note for future spikes
The scratch copy was built with the repository's own `CARGO_TARGET_DIR` to reuse compiled dependencies. Cargo then
treated the spike's `purplepie` artifacts as up to date for the real tree (a workspace member's artifact names do not
depend on its folder, and the spike's files were newer), so the next `cargo test` in the real tree ran the spike's
test binary (one extra ignored test gave it away). `cargo clean -p purplepie -p purple-swarm` fixed it. Give a spike its
own target directory, or clean the engine's artifacts afterwards.
