//! Rendering (ADR-005, ADR-009).
//!
//! Public: plain-data types that games use (`Color`, the `Quad` and `Sprite`
//! components, `TextureId`). Crate-private: the `Renderer`, which owns every
//! `wgpu` object, and the CPU-side texture store. Game code never touches the
//! GPU. The engine draws whatever the game state describes.
//!
//! Each frame the window is cleared to
//! [`EngineConfig::clear_color`](crate::EngineConfig::clear_color), then every
//! entity with [`Transform2D`](crate::math::Transform2D) + [`Quad`] is drawn,
//! then every entity with `Transform2D` + [`Sprite`] on top (Stage 6).
//! Coordinates: ADR-018. Textures: ADR-020.

mod color;
mod faults;
mod instance;
mod quad;
mod renderer;
mod sprite;
mod texture;

pub use color::Color;
pub use quad::Quad;
pub(crate) use renderer::Renderer;
pub use sprite::Sprite;
pub use texture::TextureId;
pub(crate) use texture::Textures;
