//! What gets drawn: rectangles, images and text, their order, the camera,
//! colours and textures.
//!
//! Drawing is declarative: every entity with a
//! [`Transform2D`](crate::math::Transform2D) and a [`Quad`] (solid rectangle),
//! [`Sprite`] (image) or [`Text`] is drawn each frame, until it is despawned or
//! given [`Hidden`]. [`Layer`] orders drawing (higher on top), [`ScreenSpace`] pins
//! an entity to the window for HUDs, and the [`Camera2D`]
//! ([`Context::camera_mut`](crate::Context::camera_mut)) moves and zooms the
//! world. Textures come from [`Context::load_texture`](crate::Context::load_texture)
//! (PNG), fonts from [`Context::load_font`](crate::Context::load_font). Sprite
//! sheets: [`SpriteGrid`] and [`SpriteAnimation`] with [`advance_animations`].
//! Guide: sections 5–7 and 9 (`docs/GUIDE.md`).
//!
//! ```
//! use purplepie::Context;
//! use purplepie::math::{Transform2D, Vec2};
//! use purplepie::render::{Color, Layer, Quad, ScreenSpace, Sprite, Text, TextAnchor};
//!
//! fn spawn_title_screen(ctx: &mut Context<'_>) -> purplepie::Result<()> {
//!     let logo = ctx.load_texture("textures/logo.png")?;
//!     let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
//!     let world = ctx.world_mut();
//!     world.spawn((
//!         Transform2D::default(),
//!         Quad::new(Vec2::new(640.0, 360.0), Color::hex(0x1D3557)),
//!         Layer(-1), // behind the logo
//!     ));
//!     world.spawn((
//!         Transform2D::from_position(Vec2::new(0.0, 60.0)),
//!         Sprite::new(logo, Vec2::new(256.0, 128.0)), // drawn at this size
//!     ));
//!     world.spawn((
//!         Transform2D::from_position(Vec2::new(0.0, 40.0)), // 40 px above the bottom edge
//!         Text::new("Press Space", font, 24.0).with_anchor(TextAnchor::BOTTOM_CENTER),
//!         ScreenSpace::BOTTOM, // follows the window, not the camera
//!     ));
//!     ctx.camera_mut().zoom = 1.5;
//!     Ok(())
//! }
//! ```
//!
//! # Engine notes
//!
//! Public: plain-data types (ADR-005, ADR-009). Crate-private: the `Renderer`,
//! which owns every `wgpu` object, the CPU-side texture and font stores, and the
//! glyph atlas. Game code never touches the GPU; the engine draws whatever the game
//! state describes.
//!
//! Each frame the window is cleared to
//! [`EngineConfig::clear_color`](crate::EngineConfig::clear_color), then every
//! drawable is drawn, lowest [`Layer`] first; within a layer, quads, then
//! sprites, then text; screen-space entities after all world entities.
//! Coordinates: ADR-018. Camera: ADR-022. Textures: ADR-020, sampling per
//! texture: ADR-034. Draw order: ADR-021. Text: ADR-027. Screen space: ADR-029.

mod animation;
mod atlas;
mod camera;
mod color;
mod draw;
mod faults;
mod font;
mod instance;
mod quad;
mod region;
mod renderer;
mod screen;
mod sprite;
mod text;
mod texture;

pub use animation::{AnimationMode, SpriteAnimation, advance_animations};
pub use camera::Camera2D;
pub use color::Color;
pub use draw::{Hidden, Layer};
pub use font::FontId;
pub(crate) use font::Fonts;
pub use quad::Quad;
pub use region::{SpriteGrid, TextureRegion};
pub(crate) use renderer::Renderer;
pub use screen::{ScreenAnchor, ScreenSpace};
pub use sprite::Sprite;
pub(crate) use text::measure as measure_text;
pub use text::{HorizontalAnchor, Text, TextAnchor, TextMetrics, VerticalAnchor};
pub(crate) use texture::Textures;
pub use texture::{TextureFilter, TextureId, TextureOptions};
