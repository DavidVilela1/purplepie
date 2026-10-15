// src/bin/editor.rs: `cargo run --bin editor -- assets/levels/one.ron`
use std::process::ExitCode;

use purplepie_editor::GameComponents;

#[path = "../components.rs"]
#[allow(dead_code)] // the editor only checks values; the game uses them
mod components;

fn main() -> ExitCode {
    // The same names the game registers with `register_scene_component`;
    // `with_default` also lets the inspector add the component to an entity.
    purplepie_editor::main_with(
        GameComponents::new().with_default::<components::Door>("my_game::Door"),
    )
}
