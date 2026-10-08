//! # PurplePie
//!
//! A small, modular 2D game engine written in Rust.
//!
//! A game implements [`Game`] and hands it to an [`Engine`]. The engine owns
//! the window and the main loop and calls the game back with a [`Context`]:
//! `fixed_update` at a fixed rate (default 60 Hz) for simulation, then
//! `update` once per frame.
//! Game state lives in an ECS [`ecs::World`] owned by the engine and reached
//! through [`Context::world_mut`]. The engine owns the GPU (`wgpu`) and draws
//! every frame. Game code never touches `winit` or `wgpu` types.
//!
//! ```no_run
//! use purplepie::{Context, Engine, EngineConfig, Game};
//!
//! struct Hello;
//!
//! impl Game for Hello {
//!     fn fixed_update(&mut self, ctx: &mut Context<'_>) {
//!         let _step_seconds = ctx.dt(); // 1/60 s by default
//!     }
//! }
//!
//! fn main() -> purplepie::Result<()> {
//!     Engine::new(EngineConfig::new("Hello").with_size(800, 600))?.run(Hello)
//! }
//! ```
//!
//! A game is its own crate that depends on `purplepie` (by path or git; see
//! "A new game crate" in the README) and keeps its own `assets/` folder:
//! asset paths such as `"textures/player.png"` are relative to it. Everything
//! a game calls goes through [`Context`]: assets, scenes, sound, input, the
//! camera and exiting. [`ecs`] shows how to query entities.
//!
//! Current status, architecture and roadmap: `docs/PROJECT_STATUS.md`,
//! `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`.

#![warn(missing_docs)]

mod app;
mod assets;
pub mod audio;
pub mod ecs;
mod error;
pub mod input;
pub mod math;
pub mod render;
mod scene;
mod time;
pub mod ui;

pub use app::{Context, Engine, EngineConfig, Game};
pub use error::{BoxError, Error, Result};
pub use time::Time;

/// The PurplePie crate version, taken from `Cargo.toml`.
///
/// ```
/// assert!(!purplepie::VERSION.is_empty());
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
