//! Seedable random numbers (ADR-041): PCG32 (PCG-XSH-RR, 64-bit state,
//! 32-bit output), no dependency.

use std::f32::consts::TAU;
use std::hash::{BuildHasher, Hasher};

use glam::Vec2;

/// The PCG32 multiplier.
const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
/// The stream every `Rng` uses (PCG's own example stream), so a seed alone
/// decides the sequence.
const STREAM: u64 = 54;

/// A small, fast random number generator for gameplay: spawn points, damage
/// rolls, drops, shuffles.
///
/// The same seed always gives the same sequence on every platform, so a game
/// that draws from one `Rng` inside `fixed_update` plays out identically each
/// run, which makes replays, tests and bug reports reproducible. Use
/// [`from_entropy`](Self::from_entropy) for a different game every time. It
/// is not suitable for cryptography.
///
/// ```
/// use purplepie::math::{Rng, Vec2};
///
/// let mut rng = Rng::new(7);
/// let x = rng.range_f32(-400.0, 400.0); // somewhere in [-400, 400)
/// assert!((-400.0..400.0).contains(&x));
/// let damage = rng.range_u32(5, 11); // 5..=10
/// assert!((5..11).contains(&damage));
/// if rng.chance(0.1) {
///     // a 10 % drop
/// }
/// let enemy = rng.pick(&["drifter", "dasher"]).copied();
/// assert!(enemy.is_some());
/// let heading: Vec2 = rng.unit_vec2(); // length 1, any direction
/// assert!((heading.length() - 1.0).abs() < 1e-5);
///
/// // Same seed, same numbers.
/// assert_eq!(Rng::new(7).next_u32(), Rng::new(7).next_u32());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
    increment: u64,
}

impl Rng {
    /// A generator whose sequence is fully decided by `seed`.
    pub const fn new(seed: u64) -> Self {
        let increment = (STREAM << 1) | 1;
        let mut rng = Self {
            state: 0,
            increment,
        };
        rng.step();
        rng.state = rng.state.wrapping_add(seed);
        rng.step();
        rng
    }

    /// A generator with an unpredictable seed (different every run), for
    /// games that should not repeat. The seed comes from the standard
    /// library's per-process hash keys and the clock.
    pub fn from_entropy() -> Self {
        let mut hasher = std::hash::RandomState::new().build_hasher();
        if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            hasher.write_u128(now.as_nanos());
        }
        Self::new(hasher.finish())
    }

    const fn step(&mut self) {
        self.state = self
            .state
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(self.increment);
    }

    /// The next 32 random bits.
    pub const fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.step();
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rotation = (old >> 59) as u32;
        xorshifted.rotate_right(rotation)
    }

    /// A number in `[0, 1)`, with 24 bits of precision (every value is an
    /// exact multiple of 2⁻²⁴, the same on every platform).
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }

    /// A number in `[min, max)`. Returns `min` when `max <= min`.
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        if max <= min {
            return min;
        }
        let value = min + (max - min) * self.f32();
        // Rounding can land exactly on `max` for some ranges; keep it half-open.
        if value < max { value } else { min }
    }

    /// An integer in `[min, max)`, every value equally likely. Returns `min`
    /// when `max <= min`.
    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        if max <= min {
            return min;
        }
        let span = max - min;
        // Reject the few values that would make some results more likely.
        let threshold = span.wrapping_neg() % span;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return min + r % span;
            }
        }
    }

    /// `true` with probability `p` (0 never, 1 always).
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    /// A random element of `items`, or `None` if it is empty.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        let len = u32::try_from(items.len()).unwrap_or(u32::MAX);
        if len == 0 {
            return None;
        }
        items.get(self.range_u32(0, len) as usize)
    }

    /// Puts `items` in a random order (Fisher–Yates), each order equally likely.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let bound = u32::try_from(i + 1).unwrap_or(u32::MAX);
            let j = self.range_u32(0, bound) as usize;
            items.swap(i, j);
        }
    }

    /// A vector of length 1 pointing in a random direction. (Uses `sin`/`cos`,
    /// whose last bits can differ between platforms.)
    pub fn unit_vec2(&mut self) -> Vec2 {
        let angle = self.f32() * TAU;
        Vec2::new(angle.cos(), angle.sin())
    }
}

impl Default for Rng {
    /// Same as `Rng::new(0)`.
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_pcg32_reference_sequence() {
        // pcg32_srandom_r(&rng, 42, 54) from the PCG reference demo.
        let mut rng = Rng::new(42);
        let got: Vec<u32> = (0..6).map(|_| rng.next_u32()).collect();
        assert_eq!(
            got,
            [
                0xa15c_02b7,
                0x7b47_f409,
                0xba1d_3330,
                0x83d2_f293,
                0xbfa4_784b,
                0xcbed_606e
            ]
        );
    }

    #[test]
    fn same_seed_same_numbers_different_seed_different_numbers() {
        let a: Vec<u32> = {
            let mut r = Rng::new(1);
            (0..16).map(|_| r.next_u32()).collect()
        };
        let b: Vec<u32> = {
            let mut r = Rng::new(1);
            (0..16).map(|_| r.next_u32()).collect()
        };
        let c: Vec<u32> = {
            let mut r = Rng::new(2);
            (0..16).map(|_| r.next_u32()).collect()
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(Rng::default(), Rng::new(0));
    }

    #[test]
    fn ranges_stay_inside_and_cover_every_value() {
        let mut rng = Rng::new(3);
        let mut seen = [0u32; 6];
        for _ in 0..6000 {
            let v = rng.range_u32(10, 16);
            assert!((10..16).contains(&v));
            seen[(v - 10) as usize] += 1;
        }
        // Each value about 1000 times (loose bounds: this is a smoke test).
        assert!(seen.iter().all(|&n| (850..1150).contains(&n)), "{seen:?}");
        for _ in 0..10_000 {
            let f = rng.f32();
            assert!((0.0..1.0).contains(&f));
            let r = rng.range_f32(-2.5, 7.5);
            assert!((-2.5..7.5).contains(&r));
        }
        assert_eq!(rng.range_u32(5, 5), 5, "empty range: min");
        assert_eq!(rng.range_f32(1.0, -1.0), 1.0, "reversed range: min");
        assert!(!rng.chance(0.0));
        assert!(rng.chance(1.0));
    }

    #[test]
    fn pick_shuffle_and_unit_vectors() {
        let mut rng = Rng::new(9);
        let empty: [u8; 0] = [];
        assert_eq!(rng.pick(&empty), None);
        assert_eq!(rng.pick(&[4]), Some(&4));
        let mut items: Vec<u32> = (0..20).collect();
        rng.shuffle(&mut items);
        let mut sorted = items.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>(), "a permutation");
        assert_ne!(items, sorted, "and actually shuffled");
        for _ in 0..100 {
            assert!((rng.unit_vec2().length() - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn entropy_seeds_differ() {
        // Two generators created in a row get different seeds (hash keys
        // and the clock differ), so their first numbers almost surely differ.
        let a: Vec<u32> = {
            let mut r = Rng::from_entropy();
            (0..4).map(|_| r.next_u32()).collect()
        };
        let b: Vec<u32> = {
            let mut r = Rng::from_entropy();
            (0..4).map(|_| r.next_u32()).collect()
        };
        assert_ne!(a, b);
    }
}
