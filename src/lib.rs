//! # PurplePie
//!
//! A small, modular 2D game engine written in Rust.
//!
//! **Status: Stage 0 (architecture & planning).** This crate is an intentionally
//! empty scaffold. Engine modules are added by the stage that needs them; see
//! `docs/ROADMAP.md` and `docs/ARCHITECTURE.md` for the planned layout:
//!
//! | Module   | Responsibility                                          | Stage |
//! |----------|---------------------------------------------------------|-------|
//! | `error`  | The engine's single public `Error` type                 | 1     |
//! | `app`    | `Engine`, `EngineConfig`, `Game` trait, winit event loop | 1     |
//! | `time`   | `Time` and the fixed-timestep accumulator               | 2     |
//! | `math`   | `Transform2D` and re-exported `glam` types              | 3     |
//! | `ecs`    | `hecs` re-exports, engine components, built-in systems  | 3     |
//! | `render` | wgpu renderer (private) + `Color`, `Sprite`, `Camera2D` | 4–7   |
//! | `input`  | Keyboard/mouse state exposed to game code               | 8     |
//! | `assets` | Handles and loading for textures, shaders, fonts        | 9     |
//!
//! ```
//! assert!(!purplepie::VERSION.is_empty());
//! ```

/// The PurplePie crate version, taken from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_matches_manifest() {
        assert_eq!(VERSION, "0.0.0");
    }
}
