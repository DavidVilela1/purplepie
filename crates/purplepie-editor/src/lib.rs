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
//! Ctrl+Shift+Z or Ctrl+Y), and save (Ctrl+S). Since PP-038h1 the inspector
//! edits position, rotation, scale, layer and hidden, each change one undoable
//! step; the drawables' values and game components come next.
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

mod editor;
mod history;
mod options;
mod select;
mod summary;
mod view;

use std::path::Path;

pub use options::{Options, USAGE, asset_folder_of};

use purplepie::{Engine, EngineConfig};

/// Runs the editor on `options.scene` until its window is closed.
///
/// The scene's asset paths are resolved against `options.assets`, or else
/// the nearest folder named `assets` holding the scene ([`asset_folder_of`]),
/// or else the engine's usual lookup. A scene that cannot be read is shown
/// as an error in the window, not returned.
pub fn run(options: Options) -> purplepie::Result<()> {
    run_frames(options, None)
}

/// [`run`], exiting after `frames` frames when given (automated checks).
#[doc(hidden)]
pub fn run_frames(options: Options, frames: Option<u32>) -> purplepie::Result<()> {
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
    Engine::new(config)?.run(editor::Editor::new(scene, frames))
}

/// `path` relative to the current folder made absolute, so it does not depend
/// on the asset folder.
fn absolute(path: &Path) -> std::path::PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}
