//! Game components in scene files (ADR-036).
//!
//! A game registers each of its component types under a stable name with
//! [`Context::register_scene_component`](crate::Context::register_scene_component).
//! The registry keeps, per type, monomorphised functions that test for the
//! component on an entity, write it as a RON fragment ([`RawValue`]) and read
//! it back. No `dyn Serialize` is needed, and the engine's own components keep
//! their private mirror types (ADR-035).

use std::any::TypeId;
use std::collections::BTreeMap;

use ron::value::RawValue;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::ecs::{Entity, World};
use crate::error::BoxError;

/// Adds one decoded component to an entity being spawned.
pub(crate) type Inserter = Box<dyn FnOnce(&mut World, Entity)>;

/// A component written as a RON fragment, if the entity has it.
type Saved = Option<Result<Box<RawValue>, BoxError>>;

/// One registered component type.
struct Registered {
    name: String,
    type_id: TypeId,
    has: fn(&hecs::EntityRef<'_>) -> bool,
    save: fn(&hecs::EntityRef<'_>) -> Saved,
    load: fn(&RawValue) -> Result<Inserter, BoxError>,
}

/// The game component types a scene can save and load, by name.
#[derive(Default)]
pub(crate) struct Registry {
    entries: Vec<Registered>,
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.entries.iter().map(|e| &e.name))
            .finish()
    }
}

fn has<T: hecs::Component>(e: &hecs::EntityRef<'_>) -> bool {
    e.has::<T>()
}

fn save<T: hecs::Component + Serialize>(e: &hecs::EntityRef<'_>) -> Saved {
    e.get::<&T>().map(|c| to_raw(&*c))
}

/// Writes a game component as one line of RON spaced like the engine's parts
/// of the file (`(speed: 2.0, mode: Fast)`, not `(speed:2.0,mode:Fast)`; ADR-040).
/// `RawValue::from_rust` writes the compact form.
fn to_raw<T: Serialize>(value: &T) -> Result<Box<RawValue>, BoxError> {
    let one_line = ron::ser::PrettyConfig::new()
        .new_line("\n")
        .struct_names(false)
        .compact_structs(true)
        .compact_maps(true)
        .compact_arrays(true);
    let text = ron::ser::to_string_pretty(value, one_line)?;
    Ok(RawValue::from_boxed_ron(text.into_boxed_str())?)
}

fn load<T: hecs::Component + DeserializeOwned>(raw: &RawValue) -> Result<Inserter, BoxError> {
    let component: T = raw.into_rust()?;
    Ok(Box::new(move |world: &mut World, entity: Entity| {
        if world.insert_one(entity, component).is_err() {
            log::warn!("scene: entity {entity:?} vanished while loading");
        }
    }))
}

impl Registry {
    /// Registers `T` under `name`. Registering the same type under the same
    /// name again does nothing; a name or a type that is already taken by a
    /// different registration is an error.
    pub(crate) fn register<T>(&mut self, name: &str) -> Result<(), &'static str>
    where
        T: hecs::Component + Serialize + DeserializeOwned,
    {
        if name.is_empty() || name.trim() != name {
            return Err("a scene component name must be non-empty without surrounding spaces");
        }
        let type_id = TypeId::of::<T>();
        for entry in &self.entries {
            match (entry.name == name, entry.type_id == type_id) {
                (true, true) => return Ok(()),
                (true, false) => {
                    return Err("this scene component name is already registered for another type");
                }
                (false, true) => {
                    return Err("this component type is already registered under another name");
                }
                (false, false) => {}
            }
        }
        self.entries.push(Registered {
            name: name.to_owned(),
            type_id,
            has: has::<T>,
            save: save::<T>,
            load: load::<T>,
        });
        Ok(())
    }

    /// Number of registered types.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` if a type is registered under `name`.
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|r| r.name == name)
    }

    /// `true` if the entity has any registered component.
    pub(crate) fn any_on(&self, e: &hecs::EntityRef<'_>) -> bool {
        self.entries.iter().any(|r| (r.has)(e))
    }

    /// The entity's registered components as RON fragments, by name (sorted,
    /// so files are deterministic).
    pub(crate) fn save(
        &self,
        e: &hecs::EntityRef<'_>,
    ) -> Result<BTreeMap<String, Box<RawValue>>, BoxError> {
        let mut out = BTreeMap::new();
        for r in &self.entries {
            if let Some(value) = (r.save)(e) {
                let value = value
                    .map_err(|err| format!("component `{}` could not be written: {err}", r.name))?;
                out.insert(r.name.clone(), value);
            }
        }
        Ok(out)
    }

    /// The entity's registered components for the debug overlay's inspector
    /// (ADR-043): each one's name with the one-line RON a scene file would
    /// hold, or the error that stopped it from being written. Sorted by name;
    /// unlike [`save`](Self::save), one failing component does not hide the
    /// others.
    #[cfg(feature = "egui")]
    pub(crate) fn describe(
        &self,
        e: &hecs::EntityRef<'_>,
    ) -> Vec<(String, Result<String, String>)> {
        let mut out: Vec<_> = self
            .entries
            .iter()
            .filter_map(|r| {
                let value = (r.save)(e)?
                    .map(|raw| raw.get_ron().to_owned())
                    .map_err(|err| err.to_string());
                Some((r.name.clone(), value))
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        // Components a lenient load kept as text (ADR-044), after them.
        if let Some(unknown) = e.get::<&super::UnknownComponents>() {
            out.extend(
                unknown
                    .iter()
                    .map(|(name, text)| (format!("{name} (kept as text)"), Ok(text.to_owned()))),
            );
        }
        out
    }

    /// Decodes the component saved under `name`.
    pub(crate) fn load(&self, name: &str, raw: &RawValue) -> Result<Inserter, BoxError> {
        let entry = self
            .entries
            .iter()
            .find(|r| r.name == name)
            .ok_or_else(|| {
                format!(
                    "unknown component `{name}`: the game must register it with \
                 Context::register_scene_component before loading the scene"
                )
            })?;
        (entry.load)(raw).map_err(|err| format!("component `{name}`: {err}").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Mode {
        Calm,
        Chase(u32),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Enemy {
        name: String,
        speed: f32,
        mode: Mode,
        path: Vec<(f32, f32)>,
        loot: BTreeMap<String, u8>,
        boss: Option<bool>,
    }

    #[test]
    fn game_components_are_one_spaced_line_and_read_back_exactly() {
        let enemy = Enemy {
            name: "bat, \"big\"".into(),
            speed: 2.5,
            mode: Mode::Chase(3),
            path: vec![(0.0, 1.0), (2.0, -3.5)],
            loot: BTreeMap::from([("coins".into(), 5), ("keys".into(), 1)]),
            boss: Some(false),
        };
        let raw = to_raw(&enemy).expect("write");
        assert_eq!(
            raw.get_ron(),
            r#"(name: "bat, \"big\"", speed: 2.5, mode: Chase(3), path: [(0.0, 1.0), (2.0, -3.5)], loot: {"coins": 5, "keys": 1}, boss: Some(false))"#
        );
        let back: Enemy = raw.into_rust().expect("read");
        assert_eq!(back, enemy);
        // Unit structs (markers) stay `()`.
        #[derive(Serialize)]
        struct Marker;
        assert_eq!(to_raw(&Marker).expect("marker").get_ron(), "()");
    }

    #[cfg(feature = "egui")]
    #[test]
    fn describe_lists_each_registered_component_with_its_ron_or_its_error() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Speed(f32);
        /// Serializes to an error, like a component holding a non-finite value
        /// for a format that rejects it.
        #[derive(Debug, Deserialize)]
        struct Broken;
        impl Serialize for Broken {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("cannot write a Broken"))
            }
        }
        #[derive(Debug, Serialize, Deserialize)]
        struct Unused;

        let mut registry = Registry::default();
        registry.register::<Speed>("demo::Speed").expect("register");
        registry
            .register::<Broken>("demo::Broken")
            .expect("register");
        registry
            .register::<Unused>("demo::Unused")
            .expect("register");
        let mut world = World::new();
        let e = world.spawn((Speed(2.5), Broken, 7_u32));

        let rows = registry.describe(&world.entity(e).expect("alive"));
        assert_eq!(rows.len(), 2, "only components the entity has: {rows:?}");
        assert_eq!(rows[0].0, "demo::Broken", "sorted by name");
        let error = rows[0].1.as_ref().expect_err("the error is kept");
        assert!(error.contains("cannot write a Broken"), "{error}");
        assert_eq!(rows[1], ("demo::Speed".to_owned(), Ok("(2.5)".to_owned())));
        assert!(
            registry.save(&world.entity(e).expect("alive")).is_err(),
            "save stops at the error; describe shows the rest"
        );

        // Components a lenient load kept as text follow, marked as such.
        let text = crate::scene::tests_support::kept(&[("other::Odd", "(x: 1)")]);
        let k = world.spawn((Speed(1.0), text));
        let rows = registry.describe(&world.entity(k).expect("alive"));
        assert_eq!(
            rows,
            [
                ("demo::Speed".to_owned(), Ok("(1.0)".to_owned())),
                (
                    "other::Odd (kept as text)".to_owned(),
                    Ok("(x: 1)".to_owned())
                ),
            ]
        );
    }
}
