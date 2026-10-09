//! Axis-aligned rectangles and circle tests for gameplay overlap checks
//! (ADR-041). Not a physics engine: they answer "do these touch?", nothing more.

use glam::Vec2;

/// An axis-aligned rectangle given by its centre and half its size, in the
/// same units as [`Transform2D`](super::Transform2D) positions.
///
/// Built from the same numbers as a [`Quad`](crate::render::Quad) or
/// [`Sprite`](crate::render::Sprite) (position and size), it answers
/// overlap questions for bullets, pickups and walls. Rotation is ignored.
///
/// ```
/// use purplepie::math::{Rect, Vec2};
///
/// let player = Rect::from_center_size(Vec2::new(0.0, 0.0), Vec2::new(40.0, 40.0));
/// let coin = Rect::from_center_size(Vec2::new(30.0, 0.0), Vec2::new(24.0, 24.0));
/// assert!(player.overlaps(coin));
/// assert!(player.contains(Vec2::new(19.0, -19.0)));
/// assert!(player.overlaps_circle(Vec2::new(25.0, 25.0), 8.0));
///
/// // How far they overlap: push the player out along the smaller axis.
/// let overlap = player.intersection(coin).expect("they overlap");
/// assert_eq!(overlap.size(), Vec2::new(2.0, 24.0)); // 2 units deep along x
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// The centre.
    pub center: Vec2,
    /// Half the width and half the height (non-negative).
    pub half_size: Vec2,
}

impl Rect {
    /// A rectangle of `size` (width, height) centred on `center`. Negative
    /// sizes count as their absolute value.
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        Self {
            center,
            half_size: size.abs() * 0.5,
        }
    }

    /// The rectangle spanning two corners, in any order.
    pub fn from_corners(a: Vec2, b: Vec2) -> Self {
        let (min, max) = (a.min(b), a.max(b));
        Self {
            center: (min + max) * 0.5,
            half_size: (max - min) * 0.5,
        }
    }

    /// The bottom-left corner (smallest x and y).
    pub fn min(self) -> Vec2 {
        self.center - self.half_size
    }

    /// The top-right corner (largest x and y).
    pub fn max(self) -> Vec2 {
        self.center + self.half_size
    }

    /// Width and height.
    pub fn size(self) -> Vec2 {
        self.half_size * 2.0
    }

    /// `true` if `point` is inside or on the edge.
    pub fn contains(self, point: Vec2) -> bool {
        let d = (point - self.center).abs();
        d.x <= self.half_size.x && d.y <= self.half_size.y
    }

    /// `true` if the two rectangles share some area. Rectangles that only
    /// touch along an edge or a corner do not overlap.
    pub fn overlaps(self, other: Rect) -> bool {
        let d = (other.center - self.center).abs();
        let reach = self.half_size + other.half_size;
        d.x < reach.x && d.y < reach.y
    }

    /// The shared area of two overlapping rectangles, or `None` if they do
    /// not overlap. Its size is how far one would have to move to separate them
    /// along each axis.
    pub fn intersection(self, other: Rect) -> Option<Rect> {
        if !self.overlaps(other) {
            return None;
        }
        let min = self.min().max(other.min());
        let max = self.max().min(other.max());
        Some(Rect::from_corners(min, max))
    }

    /// The point of the rectangle (inside or on the edge) closest to `point`.
    pub fn closest_point(self, point: Vec2) -> Vec2 {
        point.clamp(self.min(), self.max())
    }

    /// `true` if the circle at `center` with `radius` shares some area with
    /// the rectangle (touching only does not count).
    pub fn overlaps_circle(self, center: Vec2, radius: f32) -> bool {
        self.closest_point(center).distance_squared(center) < radius * radius
    }

    /// The rectangle grown by `amount` on every side (shrunk if negative,
    /// never below zero size).
    pub fn expand(self, amount: f32) -> Rect {
        Rect {
            center: self.center,
            half_size: (self.half_size + Vec2::splat(amount)).max(Vec2::ZERO),
        }
    }
}

/// `true` if two circles share some area (touching only does not count).
///
/// ```
/// use purplepie::math::{circles_overlap, Vec2};
///
/// assert!(circles_overlap(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 6.0));
/// assert!(!circles_overlap(Vec2::ZERO, 10.0, Vec2::new(16.0, 0.0), 6.0));
/// ```
pub fn circles_overlap(a: Vec2, radius_a: f32, b: Vec2, radius_b: f32) -> bool {
    let reach = radius_a + radius_b;
    a.distance_squared(b) < reach * reach
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(cx: f32, cy: f32, w: f32, h: f32) -> Rect {
        Rect::from_center_size(Vec2::new(cx, cy), Vec2::new(w, h))
    }

    #[test]
    fn corners_size_and_construction_agree() {
        let a = r(10.0, -5.0, 20.0, 6.0);
        assert_eq!(
            (a.min(), a.max()),
            (Vec2::new(0.0, -8.0), Vec2::new(20.0, -2.0))
        );
        assert_eq!(a.size(), Vec2::new(20.0, 6.0));
        assert_eq!(
            Rect::from_corners(a.max(), a.min()),
            a,
            "corners in any order"
        );
        assert_eq!(
            r(0.0, 0.0, -4.0, 2.0).size(),
            Vec2::new(4.0, 2.0),
            "negative size"
        );
    }

    #[test]
    fn contains_includes_edges() {
        let a = r(0.0, 0.0, 10.0, 10.0);
        assert!(a.contains(Vec2::ZERO));
        assert!(a.contains(Vec2::new(5.0, -5.0)), "corner");
        assert!(!a.contains(Vec2::new(5.01, 0.0)));
    }

    #[test]
    fn overlap_needs_shared_area() {
        let a = r(0.0, 0.0, 10.0, 10.0);
        assert!(a.overlaps(r(9.0, 9.0, 10.0, 10.0)));
        assert!(!a.overlaps(r(10.0, 0.0, 10.0, 10.0)), "edge to edge");
        assert!(!a.overlaps(r(10.0, 10.0, 10.0, 10.0)), "corner to corner");
        assert!(!a.overlaps(r(30.0, 0.0, 10.0, 10.0)));
        assert!(a.overlaps(r(0.0, 0.0, 2.0, 2.0)), "one inside the other");
        assert_eq!(
            a.overlaps(r(9.0, 9.0, 10.0, 10.0)),
            r(9.0, 9.0, 10.0, 10.0).overlaps(a)
        );
    }

    #[test]
    fn intersection_is_the_shared_area() {
        let a = r(0.0, 0.0, 10.0, 10.0);
        let b = r(8.0, 1.0, 10.0, 4.0);
        let i = a.intersection(b).expect("overlap");
        assert_eq!(
            (i.min(), i.max()),
            (Vec2::new(3.0, -1.0), Vec2::new(5.0, 3.0))
        );
        assert_eq!(a.intersection(r(20.0, 0.0, 2.0, 2.0)), None);
    }

    #[test]
    fn circles_against_rects_and_circles() {
        let a = r(0.0, 0.0, 10.0, 10.0);
        assert!(
            a.overlaps_circle(Vec2::new(7.0, 0.0), 2.5),
            "beside the right edge"
        );
        assert!(
            !a.overlaps_circle(Vec2::new(7.0, 0.0), 2.0),
            "touching only"
        );
        assert!(
            !a.overlaps_circle(Vec2::new(7.0, 7.0), 2.5),
            "near the corner but outside"
        );
        assert!(
            a.overlaps_circle(Vec2::new(6.0, 6.0), 1.5),
            "inside the corner's reach"
        );
        assert!(a.overlaps_circle(Vec2::ZERO, 0.1), "centre inside");
        assert_eq!(a.closest_point(Vec2::new(20.0, 2.0)), Vec2::new(5.0, 2.0));
        assert!(circles_overlap(Vec2::ZERO, 1.0, Vec2::new(1.5, 0.0), 1.0));
        assert!(
            !circles_overlap(Vec2::ZERO, 1.0, Vec2::new(2.0, 0.0), 1.0),
            "touching"
        );
    }

    #[test]
    fn expand_grows_and_shrinks_but_not_below_zero() {
        let a = r(1.0, 1.0, 4.0, 2.0);
        assert_eq!(a.expand(1.0).size(), Vec2::new(6.0, 4.0));
        assert_eq!(a.expand(-1.5).size(), Vec2::new(1.0, 0.0));
        assert_eq!(a.expand(-1.5).center, a.center);
    }
}
