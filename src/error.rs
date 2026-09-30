//! The engine's single public error type (ADR-011).
//!
//! Third-party errors (winit today, wgpu later) are kept as boxed
//! [`source`](std::error::Error::source)s, so their messages survive but their
//! types are not part of PurplePie's public API.

/// A boxed, thread-safe error. Used as the `source` of [`Error`] variants and
/// accepted by [`Error::game`].
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// `Result` specialised to PurplePie's [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can make the engine fail.
///
/// Variants are added as stages introduce new failure domains, so the enum is
/// `#[non_exhaustive]`: always keep a wildcard arm when matching.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// [`EngineConfig`](crate::EngineConfig) contains an unusable value.
    #[error("invalid engine configuration: {0}")]
    InvalidConfig(&'static str),

    /// The platform event loop could not be created or failed while running.
    #[error("event loop error")]
    EventLoop(#[source] BoxError),

    /// The operating system refused to create the window.
    #[error("failed to create the window")]
    Window(#[source] BoxError),

    /// The GPU drawing surface for the window could not be created.
    #[error("failed to create the GPU surface")]
    Surface(#[source] BoxError),

    /// No GPU adapter (graphics card or software renderer) can draw to the window.
    #[error("no suitable GPU adapter found")]
    Adapter(#[source] BoxError),

    /// The GPU adapter refused to open a device.
    #[error("failed to open the GPU device")]
    Device(#[source] BoxError),

    /// The selected GPU adapter cannot present to this window's surface.
    #[error("the GPU adapter does not support presenting to this window")]
    SurfaceUnsupported,

    /// An error returned by game code, e.g. from [`Game::init`](crate::Game::init).
    #[error("game error")]
    Game(#[source] BoxError),
}

impl Error {
    /// Wraps any error produced by game code.
    ///
    /// ```
    /// let err = purplepie::Error::game("level file is missing");
    /// assert_eq!(err.to_string(), "game error");
    /// ```
    pub fn game(error: impl Into<BoxError>) -> Self {
        Self::Game(error.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn error_is_send_sync_static() {
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn game_error_keeps_its_source() {
        let err = Error::game("boom");
        let source = err.source().map(ToString::to_string);
        assert_eq!(source.as_deref(), Some("boom"));
    }

    #[test]
    fn invalid_config_message_names_the_problem() {
        let err = Error::InvalidConfig("width must be greater than zero");
        assert_eq!(
            err.to_string(),
            "invalid engine configuration: width must be greater than zero"
        );
    }
}
