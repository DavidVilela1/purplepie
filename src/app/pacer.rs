//! Frame pacing for Stages 1–3, before a vsync swapchain exists (ADR-010, R-06).
//!
//! Without vsync, `ControlFlow::Poll` would spin a CPU core at 100%. Instead
//! the runner sleeps with `ControlFlow::WaitUntil(deadline)` and requests a
//! redraw only when a frame is due. Stage 4 replaces this with `Fifo` presentation.

use std::time::{Duration, Instant};

/// Target interval between redraws while no vsync is available (60 Hz).
pub(crate) const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// Decides when the next frame is due. Pure logic: tested without a window.
#[derive(Debug, Clone)]
pub(crate) struct FramePacer {
    interval: Duration,
    next_frame: Instant,
}

impl FramePacer {
    /// A pacer whose first frame is due immediately at `now`.
    pub(crate) fn new(interval: Duration, now: Instant) -> Self {
        Self {
            interval,
            next_frame: now,
        }
    }

    /// Returns `(frame_due, wake_at)`.
    ///
    /// When a frame is due, the deadline advances by one interval. If the
    /// runner fell more than one interval behind, the schedule restarts from
    /// `now` instead of producing a burst of catch-up frames.
    pub(crate) fn poll(&mut self, now: Instant) -> (bool, Instant) {
        if now < self.next_frame {
            return (false, self.next_frame);
        }
        self.next_frame += self.interval;
        if self.next_frame <= now {
            self.next_frame = now + self.interval;
        }
        (true, self.next_frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: Duration = Duration::from_millis(1);

    #[test]
    fn frame_interval_is_sixty_hertz() {
        assert_eq!(FRAME_INTERVAL.as_nanos(), 16_666_666);
    }

    #[test]
    fn first_frame_is_due_immediately() {
        let t0 = Instant::now();
        let mut pacer = FramePacer::new(10 * MS, t0);
        assert_eq!(pacer.poll(t0), (true, t0 + 10 * MS));
    }

    #[test]
    fn no_frame_before_the_deadline() {
        let t0 = Instant::now();
        let mut pacer = FramePacer::new(10 * MS, t0);
        pacer.poll(t0);
        assert_eq!(pacer.poll(t0 + 5 * MS), (false, t0 + 10 * MS));
    }

    #[test]
    fn keeps_a_steady_cadence_when_slightly_late() {
        let t0 = Instant::now();
        let mut pacer = FramePacer::new(10 * MS, t0);
        pacer.poll(t0);
        // Woken 2 ms late: the next deadline stays on the 10 ms grid.
        assert_eq!(pacer.poll(t0 + 12 * MS), (true, t0 + 20 * MS));
    }

    #[test]
    fn resets_instead_of_bursting_after_a_long_stall() {
        let t0 = Instant::now();
        let mut pacer = FramePacer::new(10 * MS, t0);
        pacer.poll(t0);
        let late = t0 + 100 * MS;
        assert_eq!(pacer.poll(late), (true, late + 10 * MS));
        assert_eq!(pacer.poll(late + MS), (false, late + 10 * MS));
    }
}
