//! Fixed-timestep accumulator (ADR-010, R-07).

/// Converts variable frame deltas into a whole number of fixed simulation steps.
///
/// Per frame: `accumulator += delta`, then run steps while
/// `accumulator >= dt`, up to `max_steps`. If the cap is hit, the backlog is
/// clamped below one step (keeping its phase), so the simulation slows down
/// instead of spiralling into ever longer frames.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FixedTimestep {
    dt: f64,
    max_steps: u32,
    accumulator: f64,
}

impl FixedTimestep {
    /// `dt` must be finite and > 0, and `max_steps` ≥ 1. `EngineConfig::validate`
    /// guarantees both before this is constructed.
    pub(crate) fn new(dt: f64, max_steps: u32) -> Self {
        debug_assert!(dt.is_finite() && dt > 0.0, "fixed dt must be positive");
        debug_assert!(max_steps >= 1, "max fixed steps must be at least 1");
        Self {
            dt,
            max_steps,
            accumulator: 0.0,
        }
    }

    /// Adds a frame delta (already clamped by `Time::begin_frame`) and returns
    /// how many fixed steps to run this frame (`0..=max_steps`).
    pub(crate) fn advance(&mut self, delta: f64) -> u32 {
        if delta.is_finite() && delta > 0.0 {
            self.accumulator += delta;
        }
        let mut steps = 0;
        while self.accumulator >= self.dt && steps < self.max_steps {
            self.accumulator -= self.dt;
            steps += 1;
        }
        if self.accumulator >= self.dt {
            // Cap reached: drop whole steps of backlog, keep the fractional phase.
            self.accumulator %= self.dt;
        }
        steps
    }

    /// Fraction of a step left in the accumulator, in `[0, 1)`.
    pub(crate) fn alpha(&self) -> f64 {
        self.accumulator / self.dt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Powers of two keep every value exact in binary floating point.
    const DT: f64 = 0.25;

    #[test]
    fn zero_delta_runs_no_steps() {
        let mut fixed = FixedTimestep::new(DT, 5);
        assert_eq!(fixed.advance(0.0), 0);
        assert_eq!(fixed.alpha(), 0.0);
    }

    #[test]
    fn one_step_per_exact_dt() {
        let mut fixed = FixedTimestep::new(DT, 5);
        assert_eq!(fixed.advance(DT), 1);
        assert_eq!(fixed.alpha(), 0.0);
    }

    #[test]
    fn remainder_carries_over_between_frames() {
        let mut fixed = FixedTimestep::new(DT, 5);
        assert_eq!(fixed.advance(0.125), 0);
        assert_eq!(fixed.alpha(), 0.5);
        assert_eq!(fixed.advance(0.25), 1);
        assert_eq!(fixed.alpha(), 0.5);
        assert_eq!(fixed.advance(0.125), 1);
        assert_eq!(fixed.alpha(), 0.0);
    }

    #[test]
    fn long_frame_runs_several_steps() {
        let mut fixed = FixedTimestep::new(DT, 5);
        assert_eq!(fixed.advance(0.625), 2);
        assert_eq!(fixed.alpha(), 0.5);
    }

    #[test]
    fn step_cap_limits_steps_and_clamps_backlog() {
        let mut fixed = FixedTimestep::new(DT, 3);
        // 2.125 s = 8.5 steps of work; only 3 may run, and the backlog drops to half a step.
        assert_eq!(fixed.advance(2.125), 3);
        assert_eq!(fixed.alpha(), 0.5);
        // No burst on the following frame.
        assert_eq!(fixed.advance(0.0), 0);
    }

    #[test]
    fn negative_or_non_finite_deltas_are_ignored() {
        let mut fixed = FixedTimestep::new(DT, 5);
        for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(fixed.advance(bad), 0);
        }
        assert_eq!(fixed.alpha(), 0.0);
    }

    #[test]
    fn sixty_hertz_frames_average_one_step_each() {
        let dt = 1.0 / 60.0;
        let mut fixed = FixedTimestep::new(dt, 5);
        let total: u32 = (0..600).map(|_| fixed.advance(dt)).sum();
        // Floating-point rounding may leave the last step pending, never more.
        assert!((599..=600).contains(&total), "total = {total}");
    }

    #[test]
    fn alpha_stays_in_unit_range_for_irregular_frames() {
        let mut fixed = FixedTimestep::new(1.0 / 60.0, 5);
        for i in 0..10_000u32 {
            let delta = f64::from(i % 97) / 1000.0; // 0 ms .. 96 ms
            fixed.advance(delta);
            let alpha = fixed.alpha();
            assert!((0.0..1.0).contains(&alpha), "alpha {alpha} at frame {i}");
        }
    }
}
