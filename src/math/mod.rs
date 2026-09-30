//! 2D math types (ADR-007).
//!
//! Built on `glam` (f32 types only). Independent of windowing, ECS and GPU
//! code. Conversion to GPU matrices happens in the renderer (Stage 4+).

pub use glam::Vec2;

/// Position, rotation and scale of something in 2D world space.
///
/// Rotation is in radians. Units and axis directions are defined in Stage 7
/// (pending decision PD-02); until then, treat them as abstract world units.
///
/// ```
/// use purplepie::math::{Transform2D, Vec2};
///
/// let t = Transform2D::from_position(Vec2::new(3.0, 4.0)).with_rotation(0.5);
/// assert_eq!(t.position, Vec2::new(3.0, 4.0));
/// assert_eq!(t.scale, Vec2::ONE);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    /// Position in world units.
    pub position: Vec2,
    /// Rotation in radians.
    pub rotation: f32,
    /// Per-axis scale factor. `Vec2::ONE` means unscaled.
    pub scale: Vec2,
}

impl Transform2D {
    /// The identity transform: origin, no rotation, unit scale.
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    /// An unrotated, unscaled transform at `position`.
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }

    /// Returns a copy with the given rotation in radians.
    #[must_use]
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Returns a copy with the given per-axis scale.
    #[must_use]
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }
}

impl Default for Transform2D {
    /// Same as [`Transform2D::IDENTITY`].
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_identity() {
        let t = Transform2D::default();
        assert_eq!(t.position, Vec2::ZERO);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale, Vec2::ONE);
        assert_eq!(t, Transform2D::IDENTITY);
    }

    #[test]
    fn builders_set_fields() {
        let t = Transform2D::from_position(Vec2::new(1.0, 2.0))
            .with_rotation(1.5)
            .with_scale(Vec2::new(2.0, 3.0));
        assert_eq!(t.position, Vec2::new(1.0, 2.0));
        assert_eq!(t.rotation, 1.5);
        assert_eq!(t.scale, Vec2::new(2.0, 3.0));
    }
}
