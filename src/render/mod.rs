//! Rendering (ADR-005, ADR-009).
//!
//! Public: plain-data types that games use (`Color`, `Camera2D`, the `Quad`,
//! `Sprite`, `Text`, `Layer` and `Hidden` components, `TextureId`, `FontId`).
//! Crate-private: the `Renderer`, which owns every `wgpu` object, the CPU-side
//! texture and font stores, and the glyph atlas. Game code never touches the
//! GPU. The engine draws whatever the game state describes.
//!
//! Each frame the window is cleared to
//! [`EngineConfig::clear_color`](crate::EngineConfig::clear_color), then every
//! entity with [`Transform2D`](crate::math::Transform2D) + [`Quad`], [`Sprite`]
//! or [`Text`] is drawn, lowest [`Layer`] first; within a layer, quads, then
//! sprites, then text. Coordinates: ADR-018. Camera: ADR-022. Textures:
//! ADR-020. Draw order: ADR-021. Text: ADR-027.

mod atlas;
mod camera;
mod color;
mod draw;
mod faults;
mod font;
mod instance;
mod quad;
mod renderer;
mod sprite;
mod text;
mod texture;

pub use camera::Camera2D;
pub use color::Color;
pub use draw::{Hidden, Layer};
pub use font::FontId;
pub(crate) use font::Fonts;
pub use quad::Quad;
pub(crate) use renderer::Renderer;
pub use sprite::Sprite;
pub use text::Text;
pub use texture::TextureId;
pub(crate) use texture::Textures;
