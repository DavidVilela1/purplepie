// src/bin/editor.rs: `cargo run --bin editor -- assets/levels/one.ron`
use std::process::ExitCode;

use purplepie_editor::GameComponents;

#[path = "../components.rs"]
#[allow(dead_code)] // the editor only checks values; the game uses them
mod components;

fn main() -> ExitCode {
    // The same names the game registers with `register_scene_component`.
    purplepie_editor::main_with(GameComponents::new().with::<components::Door>("my_game::Door"))
}
