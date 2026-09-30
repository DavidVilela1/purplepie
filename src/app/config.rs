//! Engine configuration supplied by the game.

use crate::error::{Error, Result};
use crate::render::Color;

/// Settings for [`Engine::new`](crate::Engine::new).
///
/// ```
/// use purplepie::EngineConfig;
///
/// let config = EngineConfig::new("My Game").with_size(1280, 720);
/// assert_eq!(config.title, "My Game");
/// assert_eq!((config.width, config.height), (1280, 720));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EngineConfig {
    /// Window title.
    pub title: String,
    /// Initial inner width of the window in logical pixels. Must be > 0.
    pub width: u32,
    /// Initial inner height of the window in logical pixels. Must be > 0.
    pub height: u32,
    /// Whether the user can resize the window.
    pub resizable: bool,
    /// Whether pressing Escape closes the application.
    ///
    /// Convenience for early development. Games will be able to handle keys
    /// themselves once the input system exists (Stage 8).
    pub exit_on_escape: bool,
    /// Length of one fixed simulation step in seconds (default `1/60`). Must be
    /// finite and > 0.
    pub fixed_dt: f64,
    /// Longest frame delta the engine accepts, in seconds (default `0.25`).
    /// Longer frames (debugger pause, window drag) are clamped to this.
    /// Must be finite and ≥ `fixed_dt`.
    pub max_frame_dt: f64,
    /// Maximum fixed steps run in one frame (default `5`). Any backlog beyond
    /// this is dropped rather than caught up. Must be ≥ 1.
    pub max_fixed_steps: u32,
    /// Color the window is cleared to every frame (default [`Color::PURPLEPIE`]).
    pub clear_color: Color,
}

impl EngineConfig {
    /// Default window size used by [`EngineConfig::new`].
    pub const DEFAULT_SIZE: (u32, u32) = (1280, 720);
    /// Default fixed simulation step: 60 Hz.
    pub const DEFAULT_FIXED_DT: f64 = 1.0 / 60.0;
    /// Default clamp for a single frame delta.
    pub const DEFAULT_MAX_FRAME_DT: f64 = 0.25;
    /// Default cap on fixed steps per frame.
    pub const DEFAULT_MAX_FIXED_STEPS: u32 = 5;

    /// Creates a configuration with the given window title and defaults for
    /// everything else: 1280×720, resizable, Escape exits, 60 Hz fixed step,
    /// 0.25 s frame clamp, at most 5 fixed steps per frame, purple clear color.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: Self::DEFAULT_SIZE.0,
            height: Self::DEFAULT_SIZE.1,
            resizable: true,
            exit_on_escape: true,
            fixed_dt: Self::DEFAULT_FIXED_DT,
            max_frame_dt: Self::DEFAULT_MAX_FRAME_DT,
            max_fixed_steps: Self::DEFAULT_MAX_FIXED_STEPS,
            clear_color: Color::PURPLEPIE,
        }
    }

    /// Sets the initial window size in logical pixels.
    #[must_use]
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets whether the window is resizable.
    #[must_use]
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets whether pressing Escape closes the application.
    #[must_use]
    pub fn with_exit_on_escape(mut self, exit_on_escape: bool) -> Self {
        self.exit_on_escape = exit_on_escape;
        self
    }

    /// Sets the fixed simulation step in seconds, e.g. `1.0 / 120.0` for 120 Hz.
    #[must_use]
    pub fn with_fixed_dt(mut self, fixed_dt: f64) -> Self {
        self.fixed_dt = fixed_dt;
        self
    }

    /// Sets the longest frame delta accepted before clamping, in seconds.
    #[must_use]
    pub fn with_max_frame_dt(mut self, max_frame_dt: f64) -> Self {
        self.max_frame_dt = max_frame_dt;
        self
    }

    /// Sets the maximum number of fixed steps run in one frame.
    #[must_use]
    pub fn with_max_fixed_steps(mut self, max_fixed_steps: u32) -> Self {
        self.max_fixed_steps = max_fixed_steps;
        self
    }

    /// Sets the color the window is cleared to every frame.
    #[must_use]
    pub fn with_clear_color(mut self, clear_color: Color) -> Self {
        self.clear_color = clear_color;
        self
    }

    /// Checks the configuration for values the engine cannot use.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.width == 0 {
            return Err(Error::InvalidConfig("width must be greater than zero"));
        }
        if self.height == 0 {
            return Err(Error::InvalidConfig("height must be greater than zero"));
        }
        if !(self.fixed_dt.is_finite() && self.fixed_dt > 0.0) {
            return Err(Error::InvalidConfig(
                "fixed_dt must be a finite number greater than zero",
            ));
        }
        if !(self.max_frame_dt.is_finite() && self.max_frame_dt >= self.fixed_dt) {
            return Err(Error::InvalidConfig(
                "max_frame_dt must be finite and at least fixed_dt",
            ));
        }
        if self.max_fixed_steps == 0 {
            return Err(Error::InvalidConfig("max_fixed_steps must be at least 1"));
        }
        Ok(())
    }
}

impl Default for EngineConfig {
    /// Same as `EngineConfig::new("PurplePie")`.
    fn default() -> Self {
        Self::new("PurplePie")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_documented_defaults() {
        let config = EngineConfig::new("Test");
        assert_eq!(config.title, "Test");
        assert_eq!((config.width, config.height), EngineConfig::DEFAULT_SIZE);
        assert!(config.resizable);
        assert!(config.exit_on_escape);
        assert_eq!(config.fixed_dt, 1.0 / 60.0);
        assert_eq!(config.max_frame_dt, 0.25);
        assert_eq!(config.max_fixed_steps, 5);
        assert_eq!(config.clear_color, Color::PURPLEPIE);
    }

    #[test]
    fn default_is_named_purplepie() {
        assert_eq!(EngineConfig::default(), EngineConfig::new("PurplePie"));
    }

    #[test]
    fn builders_set_fields() {
        let config = EngineConfig::new("Test")
            .with_size(640, 480)
            .with_resizable(false)
            .with_exit_on_escape(false)
            .with_fixed_dt(0.5)
            .with_max_frame_dt(2.0)
            .with_max_fixed_steps(3)
            .with_clear_color(Color::BLACK);
        assert_eq!(config.clear_color, Color::BLACK);
        assert_eq!(
            (config.fixed_dt, config.max_frame_dt, config.max_fixed_steps),
            (0.5, 2.0, 3)
        );
        assert_eq!((config.width, config.height), (640, 480));
        assert!(!config.resizable);
        assert!(!config.exit_on_escape);
    }

    #[test]
    fn validate_accepts_defaults() {
        assert!(EngineConfig::default().validate().is_ok());
    }

    #[test]
    fn validate_rejects_zero_width_or_height() {
        for (w, h) in [(0, 720), (1280, 0), (0, 0)] {
            let result = EngineConfig::default().with_size(w, h).validate();
            assert!(
                matches!(result, Err(Error::InvalidConfig(_))),
                "{w}x{h} should be rejected"
            );
        }
    }

    #[test]
    fn validate_rejects_bad_timestep_settings() {
        let base = EngineConfig::default();
        let bad = [
            base.clone().with_fixed_dt(0.0),
            base.clone().with_fixed_dt(-1.0),
            base.clone().with_fixed_dt(f64::NAN),
            base.clone().with_fixed_dt(f64::INFINITY),
            base.clone().with_max_frame_dt(0.001), // below fixed_dt
            base.clone().with_max_frame_dt(f64::INFINITY),
            base.clone().with_max_fixed_steps(0),
        ];
        for config in bad {
            assert!(
                matches!(config.validate(), Err(Error::InvalidConfig(_))),
                "{config:?} should be rejected"
            );
        }
    }

    #[test]
    fn validate_accepts_max_frame_dt_equal_to_fixed_dt() {
        let config = EngineConfig::default()
            .with_fixed_dt(0.25)
            .with_max_frame_dt(0.25);
        assert!(config.validate().is_ok());
    }
}
