//! A game's own scene editor (ADR-044, PP-038j1): the editor for the engine's
//! `scene` example, which knows that example's component types, so values of
//! `scene_demo::Spin` and `scene_demo::Visits` are checked against them.
//!
//! ```text
//! cargo run -p purplepie-editor --example scene_demo_editor -- assets/scenes/demo.ron
//! ```
//!
//! A real game would use its own types from its own crate; this example
//! repeats the `scene` example's (only reading is needed, so `Deserialize`).

use std::process::ExitCode;

use purplepie_editor::GameComponents;
use serde::Deserialize;

/// As in `examples/scene.rs`: turns the entity at `speed` radians per second.
#[derive(Deserialize)]
#[allow(dead_code)] // only read, to check values
struct Spin {
    speed: f32,
    direction: Turn,
}

#[derive(Deserialize)]
enum Turn {
    Clockwise,
    CounterClockwise,
}

/// As in `examples/scene.rs`: how often the scene file has been loaded.
#[derive(Deserialize)]
#[allow(dead_code)] // only read, to check values
struct Visits {
    label: String,
    count: u32,
}

fn main() -> ExitCode {
    purplepie_editor::main_with(
        GameComponents::new()
            .with::<Spin>("scene_demo::Spin")
            .with::<Visits>("scene_demo::Visits"),
    )
}
