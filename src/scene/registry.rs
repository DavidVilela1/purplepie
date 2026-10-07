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
    e.get::<&T>()
        .map(|c| RawValue::from_rust(&*c).map_err(BoxError::from))
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
