//! Engine configuration supplied by the game.

use crate::error::{Error, Result};

/// Settings for [`Engine::new`](crate::Engine::new).
///
/// ```
/// use purplepie::EngineConfig;
///
/// let config = EngineConfig::new("My Game").with_size(1280, 720);
/// assert_eq!(config.title, "My Game");
/// assert_eq!((config.width, config.height), (1280, 720));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl EngineConfig {
    /// Default window size used by [`EngineConfig::new`].
    pub const DEFAULT_SIZE: (u32, u32) = (1280, 720);

    /// Creates a configuration with the given window title and defaults for
    /// everything else: 1280×720, resizable, Escape exits.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: Self::DEFAULT_SIZE.0,
            height: Self::DEFAULT_SIZE.1,
            resizable: true,
            exit_on_escape: true,
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

    /// Checks the configuration for values the engine cannot use.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.width == 0 {
            return Err(Error::InvalidConfig("width must be greater than zero"));
        }
        if self.height == 0 {
            return Err(Error::InvalidConfig("height must be greater than zero"));
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
            .with_exit_on_escape(false);
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
}
