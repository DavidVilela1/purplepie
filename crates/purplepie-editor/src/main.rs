//! `purplepie-editor [--assets <folder>] <scene.ron>`: opens a scene file to
//! view, edit and save it (ADR-044). It knows no game's component types, so
//! their values are checked as RON syntax only; a game's own editor calls
//! [`purplepie_editor::main_with`] with its types.
//!
//! `PURPLEPIE_EDITOR_EXIT_AFTER_FRAMES=N` closes it after N frames (for
//! automated checks).

use std::process::ExitCode;

use purplepie_editor::GameComponents;

fn main() -> ExitCode {
    purplepie_editor::main_with(GameComponents::new())
}
