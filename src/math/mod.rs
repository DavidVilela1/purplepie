//! Vectors and transforms: [`Vec2`] for positions, sizes and velocities, and
//! the [`Transform2D`] component that places an entity.
//!
//! Coordinates: +X right, **+Y up**, the origin at the window centre, one unit is
//! one logical pixel at zoom 1. A `Transform2D` is the entity's centre, its
//! rotation in radians (counter-clockwise) and its scale (negative mirrors).
//! `Vec2` is [`glam`](https://docs.rs/glam)'s, with all its methods
//! (`length`, `normalize_or_zero`, `clamp`, `abs`, `lerp` …). Guide: section 3
//! (`docs/GUIDE.md`).
//!
//! ```
//! use purplepie::math::{Transform2D, Vec2};
//!
//! let mut ship = Transform2D::from_position(Vec2::new(0.0, -200.0))
//!     .with_rotation(std::f32::consts::FRAC_PI_2) // a quarter turn counter-clockwise
//!     .with_scale(Vec2::splat(2.0));
//! let velocity = Vec2::new(30.0, 40.0);
//! ship.position += velocity * 0.5; // half a second later
//! assert_eq!(ship.position, Vec2::new(15.0, -180.0));
//! assert_eq!(velocity.length(), 50.0);
//! ```
//!
//! # Engine notes
//!
//! Built on `glam` (f32 types only; ADR-007). Independent of windowing, ECS and
//! GPU code; conversion to GPU matrices ([`Transform2D::to_mat4`]) is used by the renderer.

pub use glam::{Mat4, Vec2};

use glam::{Quat, Vec3};

/// Position, rotation and scale of something in 2D world space.
///
/// Coordinates follow ADR-018: +X right, +Y up, rotation in radians
/// counter-clockwise. With the default view, one world unit is one logical
/// pixel and the origin is the center of the window.
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

    /// The local-to-world matrix: scale first, then rotate, then translate.
    ///
    /// ```
    /// use purplepie::math::{Transform2D, Vec2};
    ///
    /// let t = Transform2D::from_position(Vec2::new(10.0, 0.0)).with_scale(Vec2::splat(2.0));
    /// let m = t.to_mat4();
    /// assert_eq!(m.w_axis.x, 10.0); // translation
    /// assert_eq!(m.x_axis.x, 2.0); // scale (no rotation)
    /// ```
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.extend(1.0),
            Quat::from_rotation_z(self.rotation),
            Vec3::new(self.position.x, self.position.y, 0.0),
        )
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

    fn apply(t: &Transform2D, x: f32, y: f32) -> Vec2 {
        let p = t.to_mat4().transform_point3(Vec3::new(x, y, 0.0));
        Vec2::new(p.x, p.y)
    }

    #[test]
    fn identity_matrix_leaves_points_unchanged() {
        assert_eq!(Transform2D::IDENTITY.to_mat4(), Mat4::IDENTITY);
    }

    #[test]
    fn matrix_translates_scales_and_rotates_counter_clockwise() {
        let t = Transform2D::from_position(Vec2::new(5.0, -3.0));
        assert_eq!(apply(&t, 1.0, 1.0), Vec2::new(6.0, -2.0));

        let t = Transform2D::default().with_scale(Vec2::new(2.0, 3.0));
        assert_eq!(apply(&t, 1.0, 1.0), Vec2::new(2.0, 3.0));

        // +90° (counter-clockwise, ADR-018): +X goes to +Y.
        let t = Transform2D::default().with_rotation(std::f32::consts::FRAC_PI_2);
        assert!(apply(&t, 1.0, 0.0).abs_diff_eq(Vec2::new(0.0, 1.0), 1e-6));
    }

    #[test]
    fn scale_is_applied_before_rotation_and_translation() {
        let t = Transform2D::from_position(Vec2::new(10.0, 0.0))
            .with_rotation(std::f32::consts::FRAC_PI_2)
            .with_scale(Vec2::new(2.0, 1.0));
        // (1, 0) → scale (2, 0) → rotate (0, 2) → translate (10, 2).
        assert!(apply(&t, 1.0, 0.0).abs_diff_eq(Vec2::new(10.0, 2.0), 1e-5));
    }
}
