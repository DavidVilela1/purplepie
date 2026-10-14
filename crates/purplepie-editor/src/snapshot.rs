//! Everything an entity has, copied out of the world so that deleting it can be
//! undone and it can be duplicated (ADR-044, PP-038i1).
//!
//! The editor registers no game components, so the entities it opens hold only
//! what a scene file can: the engine's saved components and the game's kept as
//! text ([`UnknownComponents`]). A [`Snapshot`] copies exactly those; an entity
//! holding anything else is refused rather than copied in part.

use purplepie::ecs::hecs::EntityBuilder;
use purplepie::ecs::{Entity, Velocity, World};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Hidden, Layer, Quad, ScreenSpace, Sprite, SpriteAnimation, Text};
use purplepie::scene::UnknownComponents;
use purplepie::ui::Button;

/// How far a duplicate is placed from the original, in world units (right
/// and down), so both can be seen.
pub(crate) const DUPLICATE_OFFSET: Vec2 = Vec2::new(16.0, -16.0);

/// One entity's components, by value.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Snapshot {
    transform: Option<Transform2D>,
    quad: Option<Quad>,
    sprite: Option<Sprite>,
    text: Option<Text>,
    layer: Option<Layer>,
    hidden: bool,
    screen_space: Option<ScreenSpace>,
    animation: Option<SpriteAnimation>,
    velocity: Option<Velocity>,
    button: Option<Button>,
    kept: Option<UnknownComponents>,
}

/// Why an entity cannot be copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotCopied {
    /// It no longer exists.
    Gone,
    /// It has components the editor does not know (a game's own types).
    Unknown(usize),
}

impl Snapshot {
    /// Copies `entity`'s components, or says why it cannot.
    pub(crate) fn of(world: &World, entity: Entity) -> Result<Self, NotCopied> {
        let e = world.entity(entity).map_err(|_| NotCopied::Gone)?;
        let snapshot = Self {
            transform: e.get::<&Transform2D>().map(|c| *c),
            quad: e.get::<&Quad>().map(|c| *c),
            sprite: e.get::<&Sprite>().map(|c| *c),
            text: e.get::<&Text>().map(|c| (*c).clone()),
            layer: e.get::<&Layer>().map(|c| *c),
            hidden: e.has::<Hidden>(),
            screen_space: e.get::<&ScreenSpace>().map(|c| *c),
            animation: e.get::<&SpriteAnimation>().map(|c| *c),
            velocity: e.get::<&Velocity>().map(|c| *c),
            button: e.get::<&Button>().map(|c| *c),
            kept: e.get::<&UnknownComponents>().map(|c| (*c).clone()),
        };
        let unknown = e.len().saturating_sub(snapshot.count());
        if unknown > 0 {
            return Err(NotCopied::Unknown(unknown));
        }
        Ok(snapshot)
    }

    /// How many components this snapshot holds.
    fn count(&self) -> usize {
        [
            self.transform.is_some(),
            self.quad.is_some(),
            self.sprite.is_some(),
            self.text.is_some(),
            self.layer.is_some(),
            self.hidden,
            self.screen_space.is_some(),
            self.animation.is_some(),
            self.velocity.is_some(),
            self.button.is_some(),
            self.kept.is_some(),
        ]
        .into_iter()
        .filter(|&present| present)
        .count()
    }

    /// The same components moved by `offset` (when there is a transform).
    pub(crate) fn moved_by(mut self, offset: Vec2) -> Self {
        if let Some(t) = &mut self.transform {
            t.position += offset;
        }
        self
    }

    fn builder(&self) -> EntityBuilder {
        let mut b = EntityBuilder::new();
        if let Some(c) = self.transform {
            b.add(c);
        }
        if let Some(c) = self.quad {
            b.add(c);
        }
        if let Some(c) = self.sprite {
            b.add(c);
        }
        if let Some(c) = &self.text {
            b.add(c.clone());
        }
        if let Some(c) = self.layer {
            b.add(c);
        }
        if self.hidden {
            b.add(Hidden);
        }
        if let Some(c) = self.screen_space {
            b.add(c);
        }
        if let Some(c) = self.animation {
            b.add(c);
        }
        if let Some(c) = self.velocity {
            b.add(c);
        }
        if let Some(c) = self.button {
            b.add(c);
        }
        if let Some(c) = &self.kept {
            b.add(c.clone());
        }
        b
    }

    /// A new entity with these components.
    pub(crate) fn spawn(&self, world: &mut World) -> Entity {
        world.spawn(self.builder().build())
    }

    /// Brings back `entity` (the same handle, so earlier edits and the save
    /// order still refer to it) with these components. Refused (`false`) if
    /// any live entity uses its id: `World::spawn_at` would replace it.
    pub(crate) fn spawn_at(&self, world: &mut World, entity: Entity) -> bool {
        if world.iter().any(|e| e.entity().id() == entity.id()) {
            return false;
        }
        world.spawn_at(entity, self.builder().build());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use purplepie::render::Color;

    fn components(world: &World, entity: Entity) -> usize {
        world.entity(entity).map_or(0, |e| e.len())
    }

    #[test]
    fn a_snapshot_copies_every_component_and_brings_the_same_entity_back() {
        let mut world = World::new();
        let mut kept = UnknownComponents::default();
        kept.set("game::Speed", "(value: 2.0)").expect("valid");
        let full = world.spawn((
            Transform2D::from_position(Vec2::new(5.0, 6.0)),
            Quad::new(Vec2::new(10.0, 20.0), Color::WHITE),
            Layer(3),
            Hidden,
            ScreenSpace::TOP_LEFT,
            Velocity(Vec2::X),
            Button::new(Vec2::ONE),
            kept,
        ));
        let snapshot = Snapshot::of(&world, full).expect("copyable");
        assert_eq!(snapshot.count(), 8);
        assert_eq!(snapshot.count(), components(&world, full));

        world.despawn(full).expect("alive");
        assert!(snapshot.spawn_at(&mut world, full));
        assert_eq!(
            Snapshot::of(&world, full),
            Ok(snapshot.clone()),
            "same values back"
        );
        assert_eq!(components(&world, full), 8);

        // A copy is a new entity with the same components, moved.
        let copy = snapshot
            .clone()
            .moved_by(DUPLICATE_OFFSET)
            .spawn(&mut world);
        assert_ne!(copy, full);
        let position = world.get::<&Transform2D>(copy).expect("t").position;
        assert_eq!(position, Vec2::new(21.0, -10.0));
        assert_eq!(components(&world, copy), 8);
    }

    #[test]
    fn unknown_components_and_taken_ids_are_refused() {
        #[derive(Debug)]
        struct GameThing;
        let mut world = World::new();
        let odd = world.spawn((Transform2D::default(), GameThing));
        assert_eq!(Snapshot::of(&world, odd), Err(NotCopied::Unknown(1)));
        world.despawn(odd).expect("alive");
        assert_eq!(Snapshot::of(&world, odd), Err(NotCopied::Gone));

        // The id of a deleted entity was given to a new one: no resurrection.
        let empty = Snapshot::default();
        let reused = world.spawn((Transform2D::default(),));
        assert_eq!(reused.id(), odd.id(), "hecs reuses the id");
        assert!(!empty.spawn_at(&mut world, odd));
        assert!(world.contains(reused), "the new entity is untouched");
    }
}
