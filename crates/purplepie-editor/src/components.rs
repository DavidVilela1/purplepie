//! A game's own component types, given to its editor so that their values are
//! checked against them (ADR-044, PP-038j1).
//!
//! The editor keeps every game component as RON text
//! ([`UnknownComponents`](purplepie::scene::UnknownComponents)), whether or
//! not its type is known: the text is saved back exactly, and deleting,
//! duplicating and undoing copy it like any other value. Knowing the types
//! only adds checks, with
//! [`scene::check_component`](purplepie::scene::check_component), which reads
//! a value exactly as the game's scene load will. A type given with a default
//! value ([`GameComponents::with_default`]) can also be added to an entity in
//! the inspector (PP-041), written with
//! [`scene::component_text`](purplepie::scene::component_text).

use std::collections::BTreeMap;
use std::fmt;

use purplepie::scene::{InvalidRon, check_component, component_text};
use serde::Serialize;
use serde::de::DeserializeOwned;

type Check = fn(&str) -> Result<(), InvalidRon>;

/// One of the game's types: how to check a value, and the text of its
/// default value if it can be added.
#[derive(Clone)]
struct Entry {
    check: Check,
    default: Option<Result<String, InvalidRon>>,
}

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
    entries: BTreeMap<String, Entry>,
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
        let entry = Entry {
            check: check_component::<T>,
            default: None,
        };
        self.entries.insert(name.to_owned(), entry);
        self
    }

    /// Adds the type `T` like [`with`](Self::with), and lets the inspector
    /// add it to an entity, starting from `T::default()`.
    ///
    /// ```
    /// use purplepie_editor::GameComponents;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize, Default)]
    /// struct Wall;
    ///
    /// let components = GameComponents::new().with_default::<Wall>("my_game::Wall");
    /// assert_eq!(components.addable().collect::<Vec<_>>(), ["my_game::Wall"]);
    /// ```
    #[must_use]
    pub fn with_default<T: Serialize + DeserializeOwned + Default>(mut self, name: &str) -> Self {
        let entry = Entry {
            check: check_component::<T>,
            default: Some(component_text(&T::default())),
        };
        self.entries.insert(name.to_owned(), entry);
        self
    }

    /// The names given, sorted.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// The names that can be added to an entity (given with a default),
    /// sorted.
    pub fn addable(&self) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.default.is_some())
            .map(|(name, _)| name.as_str())
    }

    /// `true` when no type was given.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Checks `ron` against the type named `name`; `None` if that name was
    /// not given.
    pub(crate) fn check(&self, name: &str, ron: &str) -> Option<Result<(), InvalidRon>> {
        self.entries.get(name).map(|entry| (entry.check)(ron))
    }

    /// The text of the default value of the type named `name`; `None` if it
    /// was not given with a default.
    pub(crate) fn default_text(&self, name: &str) -> Option<Result<String, InvalidRon>> {
        self.entries
            .get(name)
            .and_then(|entry| entry.default.clone())
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

    #[derive(serde::Serialize, Deserialize, Default)]
    struct Tag {
        label: String,
        level: u8,
    }

    #[test]
    fn types_with_a_default_can_be_added_with_its_text() {
        let components = GameComponents::new()
            .with::<Spin>("demo::Spin")
            .with_default::<Tag>("demo::Tag");
        assert_eq!(components.addable().collect::<Vec<_>>(), ["demo::Tag"]);
        assert_eq!(
            components.default_text("demo::Tag"),
            Some(Ok(r#"(label: "", level: 0)"#.to_owned()))
        );
        assert_eq!(components.default_text("demo::Spin"), None, "no default");
        let text = components
            .default_text("demo::Tag")
            .expect("given")
            .expect("written");
        assert_eq!(components.check("demo::Tag", &text), Some(Ok(())));
    }

    #[test]
    fn the_guide_shows_the_compiled_game_editor_files() {
        let guide = include_str!("../../../docs/GUIDE.md");
        for (file, text) in [
            (
                "examples/guide/src/components.rs",
                include_str!("../examples/guide/src/components.rs"),
            ),
            (
                "examples/guide/src/bin/editor.rs",
                include_str!("../examples/guide/src/bin/editor.rs"),
            ),
        ] {
            assert!(
                guide.contains(text),
                "docs/GUIDE.md section 15 must show {file} exactly"
            );
        }
    }
}
