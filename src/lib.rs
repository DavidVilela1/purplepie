//! # PurplePie
//!
//! A small, modular 2D game engine written in Rust.
//!
//! A game implements [`Game`] and hands it to an [`Engine`]. The engine owns
//! the window and the main loop and calls the game back with a [`Context`].
//! Game code never touches `winit` (or, later, `wgpu`) types.
//!
//! ```no_run
//! use purplepie::{Context, Engine, EngineConfig, Game};
//!
//! struct Hello;
//!
//! impl Game for Hello {
//!     fn update(&mut self, _ctx: &mut Context<'_>) {}
//! }
//!
//! fn main() -> purplepie::Result<()> {
//!     Engine::new(EngineConfig::new("Hello").with_size(800, 600))?.run(Hello)
//! }
//! ```
//!
//! Current status, architecture and roadmap: `docs/PROJECT_STATUS.md`,
//! `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`.

mod app;
mod error;

pub use app::{Context, Engine, EngineConfig, Game};
pub use error::{BoxError, Error, Result};

/// The PurplePie crate version, taken from `Cargo.toml`.
///
/// ```
/// assert!(!purplepie::VERSION.is_empty());
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
