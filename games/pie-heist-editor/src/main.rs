//! Pie Heist's own scene editor (docs/GAME3.md): PurplePie's editor with the
//! game's component types, so their values are checked and any of them can be
//! added to an entity ("Game component: Add…").
//!
//! Run it from the game's folder, on a level:
//!
//! ```text
//! cd games/pie-heist
//! cargo run -p pie-heist-editor -- assets/levels/level1.ron
//! ```

use std::process::ExitCode;

use pie_heist::{Door, Exit, Guard, Key, Pie, PlayerStart, Wall, names};
use purplepie_editor::GameComponents;

fn main() -> ExitCode {
    // The same names the game registers (`pie_heist::register`).
    purplepie_editor::main_with(
        GameComponents::new()
            .with_default::<Wall>(names::WALL)
            .with_default::<Pie>(names::PIE)
            .with_default::<Key>(names::KEY)
            .with_default::<Door>(names::DOOR)
            .with_default::<Exit>(names::EXIT)
            .with_default::<Guard>(names::GUARD)
            .with_default::<PlayerStart>(names::PLAYER_START),
    )
}
