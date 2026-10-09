//! The engine's single public error type (ADR-011).
//!
//! Third-party errors (winit, wgpu, image) are kept as boxed
//! [`source`](std::error::Error::source)s, so their messages survive but their
//! types are not part of PurplePie's public API.
//!
//! `Debug` prints the message and its chain of causes (ADR-040), so
//! `fn main() -> purplepie::Result<()>` reports a failure readably:
//!
//! ```text
//! Error: failed to load asset `/home/me/my_game/assets/textures/player.png`
//!
//! Caused by:
//!     No such file or directory (os error 2)
//! ```

/// A boxed, thread-safe error. Used as the `source` of [`Error`] variants and
/// accepted by [`Error::game`].
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// `Result` specialised to PurplePie's [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can make the engine fail.
///
/// Variants are added as stages introduce new failure domains, so the enum is
/// `#[non_exhaustive]`: always keep a wildcard arm when matching.
#[derive(thiserror::Error)]
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

    /// The GPU failed while running: a wgpu validation, out-of-memory or
    /// internal error, a lost device, or a surface that could not be recreated.
    /// The source describes the details.
    #[error("GPU rendering failed")]
    Render(#[source] BoxError),

    /// A file the game asked for could not be used: it is missing, unreadable,
    /// not a supported format, or too large for the GPU. `source` says which.
    #[error("failed to load asset `{}`", .path.display())]
    Asset {
        /// The file that was tried: the game's path resolved against the asset
        /// root (or as given when no asset folder was found).
        path: std::path::PathBuf,
        /// The underlying I/O, decoding or size error.
        #[source]
        source: BoxError,
    },

    /// A file could not be written, e.g. by
    /// [`Context::save_scene`](crate::Context::save_scene): the folder is
    /// missing or read-only, or the data could not be encoded. `source` says which.
    #[error("failed to save `{}`", .path.display())]
    Save {
        /// The file that was written: the game's path resolved against the
        /// asset root (or as given when no asset folder was found).
        path: std::path::PathBuf,
        /// The underlying I/O or encoding error.
        #[source]
        source: BoxError,
    },

    /// An error returned by game code, e.g. from [`Game::init`](crate::Game::init).
    #[error("game error")]
    Game(#[source] BoxError),
}

/// The message, then each cause on its own line, like `anyhow` (ADR-040):
/// this is what `fn main() -> purplepie::Result<()>` prints on failure. The
/// variant and its fields are available by matching on the error.
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")?;
        let mut source = std::error::Error::source(self);
        if source.is_some() {
            write!(f, "\n\nCaused by:")?;
        }
        while let Some(cause) = source {
            write!(f, "\n    {cause}")?;
            source = cause.source();
        }
        Ok(())
    }
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
    fn debug_prints_the_message_and_every_cause() {
        // What `fn main() -> purplepie::Result<()>` prints after "Error: ".
        #[derive(Debug)]
        struct Outer(std::io::Error);
        impl std::fmt::Display for Outer {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("could not read the level")
            }
        }
        impl std::error::Error for Outer {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "level.ron is missing");
        let err = Error::game(Outer(io));
        assert_eq!(
            format!("{err:?}"),
            "game error\n\nCaused by:\n    could not read the level\n    level.ron is missing"
        );
        let plain = Error::InvalidConfig("width must be greater than zero");
        assert_eq!(
            format!("{plain:?}"),
            plain.to_string(),
            "no causes: just the message"
        );
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
