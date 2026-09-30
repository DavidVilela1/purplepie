//! Rendering (ADR-005, ADR-009).
//!
//! Public: plain-data types that games use (`Color`). Crate-private: the
//! `Renderer`, which owns every `wgpu` object. Game code never touches the
//! GPU. The engine draws whatever the game state describes.
//!
//! Stage 4 clears the window to [`EngineConfig::clear_color`](crate::EngineConfig::clear_color).
//! Drawing primitives and sprites from ECS data follows in Stages 5–6.

mod color;
mod renderer;

pub use color::Color;
pub(crate) use renderer::Renderer;
