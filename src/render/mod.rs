//! Rendering (ADR-005, ADR-009).
//!
//! Public: plain-data types that games use (`Color`, the `Quad` component).
//! Crate-private: the `Renderer`, which owns every `wgpu` object. Game code
//! never touches the GPU. The engine draws whatever the game state describes.
//!
//! Each frame the window is cleared to
//! [`EngineConfig::clear_color`](crate::EngineConfig::clear_color), then every
//! entity with [`Transform2D`](crate::math::Transform2D) + [`Quad`] is drawn
//! (Stage 5). Sprites follow in Stage 6. Coordinates: ADR-018.

mod color;
mod faults;
mod quad;
mod renderer;

pub use color::Color;
pub use quad::Quad;
pub(crate) use renderer::Renderer;
