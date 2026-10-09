//! The opt-in console logger (ADR-040, amending ADR-016).
//!
//! PurplePie reports through the `log` facade, which prints nothing until a
//! backend is installed. With [`EngineConfig::console_log`](super::EngineConfig::console_log)
//! the engine installs this small one: every message at or above the level in
//! `PURPLEPIE_LOG` (default `warn`) goes to stderr as `[LEVEL target] message`.
//! Without the setting the engine still never installs a logger.

use log::LevelFilter;

use crate::error::{Error, Result};

/// The environment variable that sets the console log level.
pub(crate) const LEVEL_VAR: &str = "PURPLEPIE_LOG";

/// Prints log records to stderr.
struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("[{} {}] {}", record.level(), record.target(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: ConsoleLogger = ConsoleLogger;

/// The level for a `PURPLEPIE_LOG` value (`None` when it is not set; an
/// empty value counts as not set).
pub(crate) fn level_from(value: Option<&str>) -> Result<LevelFilter> {
    match value.map(str::trim) {
        None | Some("") => Ok(LevelFilter::Warn),
        Some(v) => v.parse().map_err(|_| {
            Error::InvalidConfig("PURPLEPIE_LOG must be off, error, warn, info, debug or trace")
        }),
    }
}

/// Installs the console logger at the level from `PURPLEPIE_LOG`. Returns
/// `false`, changing nothing, if the process already has a logger (the
/// game's own, or this one from an earlier `Engine`).
pub(crate) fn install() -> Result<bool> {
    let value = std::env::var(LEVEL_VAR).ok();
    let level = level_from(value.as_deref())?;
    if log::set_logger(&LOGGER).is_err() {
        return Ok(false);
    }
    log::set_max_level(level);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_parse_case_insensitively_and_default_to_warn() {
        assert_eq!(level_from(None).expect("unset"), LevelFilter::Warn);
        assert_eq!(level_from(Some("info")).expect("info"), LevelFilter::Info);
        assert_eq!(
            level_from(Some(" DEBUG ")).expect("debug"),
            LevelFilter::Debug
        );
        assert_eq!(level_from(Some("off")).expect("off"), LevelFilter::Off);
        assert_eq!(level_from(Some(" ")).expect("empty"), LevelFilter::Warn);
        assert!(matches!(
            level_from(Some("loud")),
            Err(Error::InvalidConfig(_))
        ));
    }
}
