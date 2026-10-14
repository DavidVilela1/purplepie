//! A game's own component types, given to its editor so that their values are
//! checked against them (ADR-044, PP-038j1).
//!
//! The editor keeps every game component as RON text
//! ([`UnknownComponents`](purplepie::scene::UnknownComponents)), whether or
//! not its type is known: the text is saved back exactly, and deleting,
//! duplicating and undoing copy it like any other value. Knowing the types
//! only adds checks, with
//! [`scene::check_component`](purplepie::scene::check_component), which reads
//! a value exactly as the game's scene load will.

use std::collections::BTreeMap;
use std::fmt;

use purplepie::scene::{InvalidRon, check_component};
use serde::de::DeserializeOwned;

type Check = fn(&str) -> Result<(), InvalidRon>;

/// The game's scene component types, by the names it registers them under
/// with [`Context::register_scene_component`](purplepie::Context::register_scene_component).
///
/// ```
/// use purplepie_editor::GameComponents;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Speed {
///     value: f32,
/// }
///
/// let components = GameComponents::new().with::<Speed>("my_game::Speed");
/// assert_eq!(components.names().collect::<Vec<_>>(), ["my_game::Speed"]);
/// ```
#[derive(Clone, Default)]
pub struct GameComponents {
    checks: BTreeMap<String, Check>,
}

impl GameComponents {
    /// No types: game components are checked as RON syntax only.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the type `T`, registered by the game as `name`. Only reading is
    /// needed (`Deserialize`), as the editor never builds the values.
    #[must_use]
    pub fn with<T: DeserializeOwned>(mut self, name: &str) -> Self {
        self.checks.insert(name.to_owned(), check_component::<T>);
        self
    }

    /// The names given, sorted.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.checks.keys().map(String::as_str)
    }

    /// `true` when no type was given.
    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }

    /// Checks `ron` against the type named `name`; `None` if that name was
    /// not given.
    pub(crate) fn check(&self, name: &str, ron: &str) -> Option<Result<(), InvalidRon>> {
        self.checks.get(name).map(|check| check(ron))
    }
}

impl fmt::Debug for GameComponents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.names()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[allow(dead_code)] // only read
    struct Spin {
        speed: f32,
    }

    #[test]
    fn given_types_check_values_and_other_names_are_not_checked() {
        let components = GameComponents::new().with::<Spin>("demo::Spin");
        assert!(!components.is_empty());
        assert_eq!(components.check("demo::Spin", "(speed: 2.0)"), Some(Ok(())));
        let err = components
            .check("demo::Spin", "(speed: fast)")
            .expect("known")
            .expect_err("not a number");
        assert!(
            err.to_string().starts_with("does not fit the type"),
            "{err}"
        );
        assert_eq!(components.check("demo::Other", "anything"), None);
        assert!(GameComponents::new().is_empty());
        assert_eq!(format!("{components:?}"), r#"["demo::Spin"]"#);
    }
}
