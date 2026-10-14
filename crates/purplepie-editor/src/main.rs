//! `purplepie-editor [--assets <folder>] <scene.ron>`: opens a scene file to
//! view and save it (ADR-044).
//!
//! `PURPLEPIE_EDITOR_EXIT_AFTER_FRAMES=N` closes it after N frames (for
//! automated checks).

use std::process::ExitCode;

use purplepie_editor::{Options, USAGE};

fn main() -> ExitCode {
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
    match purplepie_editor::run_frames(options, frames) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
