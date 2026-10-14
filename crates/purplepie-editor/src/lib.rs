//! PurplePie's scene editor (phase P5, ADR-044).
//!
//! The editor is an ordinary PurplePie [`Game`](purplepie::Game) on the
//! engine's public API with the `egui` feature: the engine draws the scene,
//! and egui panels around it list the entities, save, and report what
//! happened. It opens a scene with
//! [`Context::load_scene_keeping_unknown`](purplepie::Context::load_scene_keeping_unknown),
//! so game components it does not know are kept and saved back unchanged.
//!
//! So far (PP-038f, PP-038g): open a scene, list its entities, pan (middle or
//! right drag) and zoom (wheel) the view, fit it (F), select an entity (click it
//! in the view, again to reach the one below, or in the list), inspect it, move
//! it (left drag, arrow keys, Shift for 10 units), undo and redo moves (Ctrl+Z,
//! Ctrl+Shift+Z or Ctrl+Y), and save (Ctrl+S). The inspector edits position,
//! rotation, scale, layer and hidden (PP-038h1), quads, sprites and texts, and
//! the game's components as RON text checked when the field is left (PP-038h2),
//! each field change one undoable step. Delete removes the selection and Ctrl+D
//! duplicates it (PP-038i1); undoing a delete brings the same entity back. The
//! Add row places a quad, a sprite or a text at the centre of the view
//! (PP-038i2); a file that does not exist yet opens as a new scene. A game's
//! own editor gives its component types ([`GameComponents`], [`main_with`]),
//! and their values are checked against them (PP-038j1).
//!
//! ```no_run
//! use purplepie_editor::Options;
//!
//! fn main() -> purplepie::Result<()> {
//!     purplepie_editor::run(Options {
//!         scene: "assets/scenes/level1.ron".into(),
//!         assets: None,
//!     })
//! }
//! ```

mod components;
mod editor;
mod history;
mod library;
mod options;
mod select;
mod snapshot;
mod summary;
mod view;

use std::path::Path;

pub use components::GameComponents;
pub use options::{Options, USAGE, asset_folder_of};

use std::process::ExitCode;

use purplepie::{Engine, EngineConfig};

/// Runs the editor on `options.scene` until its window is closed.
///
/// The scene's asset paths are resolved against `options.assets`, or else
/// the nearest folder named `assets` holding the scene ([`asset_folder_of`]),
/// or else the engine's usual lookup. A scene that cannot be read is shown
/// as an error in the window, not returned.
pub fn run(options: Options) -> purplepie::Result<()> {
    run_with(options, GameComponents::new())
}

/// [`run`] for a game whose component types are known: their values are
/// checked against them when the scene opens and when they are edited.
pub fn run_with(options: Options, components: GameComponents) -> purplepie::Result<()> {
    run_frames(options, components, None)
}

/// The whole `purplepie-editor` program (arguments, `--help`, errors, exit
/// code) with `components` known: a game's own editor is a `main` that calls
/// this.
///
/// ```no_run
/// use purplepie_editor::GameComponents;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Speed {
///     value: f32,
/// }
///
/// fn main() -> std::process::ExitCode {
///     purplepie_editor::main_with(GameComponents::new().with::<Speed>("my_game::Speed"))
/// }
/// ```
///
/// `PURPLEPIE_EDITOR_EXIT_AFTER_FRAMES=N` closes it after N frames (for
/// automated checks).
pub fn main_with(components: GameComponents) -> ExitCode {
    let options = match Options::from_args(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(Some(problem)) => {
            eprintln!("error: {problem}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let frames = std::env::var("PURPLEPIE_EDITOR_EXIT_AFTER_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok());
    match run_frames(options, components, frames) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// [`run_with`], exiting after `frames` frames when given (automated checks).
fn run_frames(
    options: Options,
    components: GameComponents,
    frames: Option<u32>,
) -> purplepie::Result<()> {
    let scene = absolute(&options.scene);
    let assets = options
        .assets
        .map(|folder| absolute(&folder))
        .or_else(|| asset_folder_of(&scene));
    let mut config = EngineConfig::new("PurplePie Editor")
        .with_size(1280, 720)
        .with_egui(true)
        .with_console_log(true)
        // Escape must not throw away the work: close the window to quit.
        .with_exit_on_escape(false);
    if let Some(assets) = assets {
        config = config.with_asset_root(assets);
    }
    Engine::new(config)?.run(editor::Editor::new(scene, components, frames))
}

/// `path` relative to the current folder made absolute, so it does not depend
/// on the asset folder.
fn absolute(path: &Path) -> std::path::PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}
