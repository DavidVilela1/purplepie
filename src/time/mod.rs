//! Time keeping and the fixed simulation timestep (ADR-010).
//!
//! Depends only on `std`. Nothing here knows about windows, events or the GPU,
//! so all of it is unit-tested directly.

mod fixed;

pub(crate) use fixed::FixedTimestep;

/// Read-only timing information for the current frame, available to game code
/// through [`Context::time`](crate::Context::time).
///
/// All values are in seconds as `f64`, which avoids precision drift over long
/// sessions. For per-callback gameplay math, use
/// [`Context::dt`](crate::Context::dt), which is already the right step for the
/// callback you are in.
#[derive(Debug, Clone, PartialEq)]
pub struct Time {
    delta: f64,
    elapsed: f64,
    frame: u64,
    fixed_dt: f64,
    fixed_steps: u64,
    alpha: f64,
}

impl Time {
    pub(crate) fn new(fixed_dt: f64) -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
            frame: 0,
            fixed_dt,
            fixed_steps: 0,
            alpha: 0.0,
        }
    }

    /// Duration of the current frame, clamped to `EngineConfig::max_frame_dt`
    /// so a stall (debugger, window drag) does not produce a huge step.
    pub fn delta(&self) -> f64 {
        self.delta
    }

    /// Sum of all clamped frame deltas since the game started ("game time").
    ///
    /// Time lost to stalls beyond `max_frame_dt` is not counted.
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// Number of the current frame. The first frame is `1`. It is `0` during `Game::init`.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Length of one fixed simulation step (default `1/60` s).
    pub fn fixed_dt(&self) -> f64 {
        self.fixed_dt
    }

    /// Total fixed steps run since the game started, including the current one
    /// when read inside `fixed_update`.
    pub fn fixed_steps(&self) -> u64 {
        self.fixed_steps
    }

    /// How far the simulation is between the last fixed step and the next one,
    /// in `[0, 1)`. Updated after the fixed steps of each frame. It is intended
    /// for render interpolation, which is not implemented yet.
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Starts a new frame. `raw_delta` is the measured wall-clock time since
    /// the previous frame. Returns the clamped delta that was recorded.
    pub(crate) fn begin_frame(&mut self, raw_delta: f64, max_frame_dt: f64) -> f64 {
        let delta = if raw_delta.is_finite() {
            raw_delta.clamp(0.0, max_frame_dt)
        } else {
            0.0
        };
        self.delta = delta;
        self.elapsed += delta;
        self.frame += 1;
        delta
    }

    pub(crate) fn record_fixed_step(&mut self) {
        self.fixed_steps += 1;
    }

    pub(crate) fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero() {
        let time = Time::new(0.25);
        assert_eq!(
            (
                time.delta(),
                time.elapsed(),
                time.frame(),
                time.fixed_steps()
            ),
            (0.0, 0.0, 0, 0)
        );
        assert_eq!(time.fixed_dt(), 0.25);
    }

    #[test]
    fn begin_frame_records_delta_and_counts_frames() {
        let mut time = Time::new(0.25);
        assert_eq!(time.begin_frame(0.125, 1.0), 0.125);
        assert_eq!(time.begin_frame(0.25, 1.0), 0.25);
        assert_eq!(time.delta(), 0.25);
        assert_eq!(time.elapsed(), 0.375);
        assert_eq!(time.frame(), 2);
    }

    #[test]
    fn begin_frame_clamps_stalls_and_rejects_bad_values() {
        let mut time = Time::new(0.25);
        assert_eq!(time.begin_frame(30.0, 0.5), 0.5);
        assert_eq!(time.begin_frame(-1.0, 0.5), 0.0);
        assert_eq!(time.begin_frame(f64::NAN, 0.5), 0.0);
        assert_eq!(time.begin_frame(f64::INFINITY, 0.5), 0.0);
        assert_eq!(time.elapsed(), 0.5);
        assert_eq!(time.frame(), 4);
    }

    #[test]
    fn fixed_step_counter_and_alpha() {
        let mut time = Time::new(0.25);
        time.record_fixed_step();
        time.record_fixed_step();
        time.set_alpha(0.5);
        assert_eq!(time.fixed_steps(), 2);
        assert_eq!(time.alpha(), 0.5);
    }
}
