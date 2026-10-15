//! A game's own scene editor (ADR-044, PP-038j1): the editor for the engine's
//! `scene` example, which knows that example's component types, so values of
//! `scene_demo::Spin` and `scene_demo::Visits` are checked against them, and
//! either can be added to an entity in the inspector (PP-041).
//!
//! ```text
//! cargo run -p purplepie-editor --example scene_demo_editor -- assets/scenes/demo.ron
//! ```
//!
//! A real game would use its own types from its own crate; this example
//! repeats the `scene` example's (`Deserialize` to check values; `Serialize` and
//! `Default` to add them).

use std::process::ExitCode;

use purplepie_editor::GameComponents;
use serde::{Deserialize, Serialize};

/// As in `examples/scene.rs`: turns the entity at `speed` radians per second.
/// `Default` and `Serialize` let the editor add it to an entity.
#[derive(Serialize, Deserialize, Default)]
struct Spin {
    speed: f32,
    direction: Turn,
}

#[derive(Serialize, Deserialize, Default)]
enum Turn {
    #[default]
    Clockwise,
    CounterClockwise,
}

/// As in `examples/scene.rs`: how often the scene file has been loaded.
#[derive(Serialize, Deserialize, Default)]
struct Visits {
    label: String,
    count: u32,
}

fn main() -> ExitCode {
    purplepie_editor::main_with(
        GameComponents::new()
            .with_default::<Spin>("scene_demo::Spin")
            .with_default::<Visits>("scene_demo::Visits"),
    )
}
