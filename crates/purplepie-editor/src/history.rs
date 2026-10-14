//! Undo and redo (ADR-044): every change is a reversible [`Edit`] (one
//! component's value on one entity, before and after) kept in a [`History`],
//! which also knows whether the scene differs from the last save.

use purplepie::ecs::hecs;
use purplepie::ecs::{Entity, World};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Hidden, Layer, Quad, Sprite, Text};
use purplepie::scene::UnknownComponents;

/// One reversible change to the scene: one component of one entity.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Edit {
    /// The entity's transform (moves, and the inspector's transform fields).
    Transform {
        entity: Entity,
        before: Transform2D,
        after: Transform2D,
    },
    /// The entity's `Layer`, `None` meaning no `Layer` component.
    Layer {
        entity: Entity,
        before: Option<Layer>,
        after: Option<Layer>,
    },
    /// Whether the entity has `Hidden`.
    Hidden {
        entity: Entity,
        before: bool,
        after: bool,
    },
    /// The entity's quad (size, colour).
    Quad {
        entity: Entity,
        before: Quad,
        after: Quad,
    },
    /// The entity's sprite (size, tint, region; the texture is not edited).
    Sprite {
        entity: Entity,
        before: Sprite,
        after: Sprite,
    },
    /// The entity's text (content, size, colour, anchor; the font is not edited).
    Text {
        entity: Entity,
        before: Box<Text>,
        after: Box<Text>,
    },
    /// The RON text of a game component kept as text (`UnknownComponents`).
    Kept {
        entity: Entity,
        name: String,
        before: String,
        after: String,
    },
}

/// Replaces `entity`'s `T`; `false` if it has none.
fn replace<T: hecs::Component + Clone>(world: &mut World, entity: Entity, value: &T) -> bool {
    match world.get::<&mut T>(entity) {
        Ok(mut current) => {
            *current = value.clone();
            true
        }
        Err(_) => false,
    }
}

/// `entity`'s `T` now, or `fallback` if it has none.
fn current<T: hecs::Component + Clone>(world: &World, entity: Entity, fallback: &T) -> T {
    world
        .get::<&T>(entity)
        .map_or_else(|_| fallback.clone(), |c| (*c).clone())
}

impl Edit {
    /// A move of `entity` from `from` to `to`, other transform parts as in
    /// `transform` (its value before the move).
    pub(crate) fn moved(entity: Entity, transform: Transform2D, to: Vec2) -> Self {
        Edit::Transform {
            entity,
            before: transform,
            after: Transform2D {
                position: to,
                ..transform
            },
        }
    }

    /// The entity this edit changes.
    pub(crate) fn entity(&self) -> Entity {
        match *self {
            Edit::Transform { entity, .. }
            | Edit::Layer { entity, .. }
            | Edit::Hidden { entity, .. }
            | Edit::Quad { entity, .. }
            | Edit::Sprite { entity, .. }
            | Edit::Text { entity, .. }
            | Edit::Kept { entity, .. } => entity,
        }
    }

    /// Whether `other` changes the same thing: the same kind of edit on the
    /// same entity (and the same game component).
    pub(crate) fn same_target(&self, other: &Edit) -> bool {
        match (self, other) {
            (
                Edit::Kept { name, .. },
                Edit::Kept {
                    name: other_name, ..
                },
            ) => name == other_name && self.entity() == other.entity(),
            _ => {
                std::mem::discriminant(self) == std::mem::discriminant(other)
                    && self.entity() == other.entity()
            }
        }
    }

    /// Makes this edit end where it starts (`after` = `before`): an edit
    /// about to begin.
    pub(crate) fn revert_to_before(&mut self) {
        match self {
            Edit::Transform { before, after, .. } => *after = *before,
            Edit::Layer { before, after, .. } => *after = *before,
            Edit::Hidden { before, after, .. } => *after = *before,
            Edit::Quad { before, after, .. } => *after = *before,
            Edit::Sprite { before, after, .. } => *after = *before,
            Edit::Text { before, after, .. } => after.clone_from(before),
            Edit::Kept { before, after, .. } => after.clone_from(before),
        }
    }

    /// `true` when before and after are the same.
    pub(crate) fn changes_nothing(&self) -> bool {
        match self {
            Edit::Transform { before, after, .. } => before == after,
            Edit::Layer { before, after, .. } => before == after,
            Edit::Hidden { before, after, .. } => before == after,
            Edit::Quad { before, after, .. } => before == after,
            Edit::Sprite { before, after, .. } => before == after,
            Edit::Text { before, after, .. } => before == after,
            Edit::Kept { before, after, .. } => before == after,
        }
    }

    /// The same edit ending in `entity`'s current value of this component
    /// (an edit started in the inspector, finished when the field is left).
    pub(crate) fn ending_now(self, world: &World) -> Self {
        match self {
            Edit::Transform { entity, before, .. } => Edit::Transform {
                entity,
                before,
                after: current(world, entity, &before),
            },
            Edit::Layer { entity, before, .. } => Edit::Layer {
                entity,
                before,
                after: world.get::<&Layer>(entity).ok().map(|l| *l),
            },
            Edit::Hidden { entity, before, .. } => Edit::Hidden {
                entity,
                before,
                after: world.get::<&Hidden>(entity).is_ok(),
            },
            Edit::Quad { entity, before, .. } => Edit::Quad {
                entity,
                before,
                after: current(world, entity, &before),
            },
            Edit::Sprite { entity, before, .. } => Edit::Sprite {
                entity,
                before,
                after: current(world, entity, &before),
            },
            Edit::Text { entity, before, .. } => {
                let after = Box::new(current(world, entity, &*before));
                Edit::Text {
                    entity,
                    before,
                    after,
                }
            }
            Edit::Kept {
                entity,
                name,
                before,
                ..
            } => {
                let after = world
                    .get::<&UnknownComponents>(entity)
                    .ok()
                    .and_then(|kept| kept.get(&name).map(str::to_owned))
                    .unwrap_or_else(|| before.clone());
                Edit::Kept {
                    entity,
                    name,
                    before,
                    after,
                }
            }
        }
    }

    /// Puts the world in the state after this edit (`forward`) or before it.
    /// `false` if the entity is gone (or does not have the component).
    pub(crate) fn apply(&self, world: &mut World, forward: bool) -> bool {
        if !world.contains(self.entity()) {
            return false;
        }
        match self {
            &Edit::Transform {
                entity,
                before,
                after,
            } => replace(world, entity, if forward { &after } else { &before }),
            &Edit::Layer {
                entity,
                before,
                after,
            } => match if forward { after } else { before } {
                Some(layer) => world.insert_one(entity, layer).is_ok(),
                None => {
                    let _ = world.remove_one::<Layer>(entity);
                    true
                }
            },
            &Edit::Hidden {
                entity,
                before,
                after,
            } => {
                if if forward { after } else { before } {
                    world.insert_one(entity, Hidden).is_ok()
                } else {
                    let _ = world.remove_one::<Hidden>(entity);
                    true
                }
            }
            Edit::Quad {
                entity,
                before,
                after,
            } => replace(world, *entity, if forward { after } else { before }),
            Edit::Sprite {
                entity,
                before,
                after,
            } => replace(world, *entity, if forward { after } else { before }),
            Edit::Text {
                entity,
                before,
                after,
            } => replace::<Text>(world, *entity, if forward { after } else { before }),
            Edit::Kept {
                entity,
                name,
                before,
                after,
            } => match world.get::<&mut UnknownComponents>(*entity) {
                Ok(mut kept) => kept.set(name, if forward { after } else { before }).is_ok(),
                Err(_) => false,
            },
        }
    }

    /// Extends this edit with a newer one of the same kind on the same
    /// entity (keeping this one's `before`); `false` if they do not match.
    fn absorb(&mut self, newer: &Edit) -> bool {
        if !self.same_target(newer) {
            return false;
        }
        match (self, newer) {
            (Edit::Transform { after, .. }, Edit::Transform { after: newer, .. }) => {
                *after = *newer
            }
            (Edit::Layer { after, .. }, Edit::Layer { after: newer, .. }) => *after = *newer,
            (Edit::Hidden { after, .. }, Edit::Hidden { after: newer, .. }) => *after = *newer,
            (Edit::Quad { after, .. }, Edit::Quad { after: newer, .. }) => *after = *newer,
            (Edit::Sprite { after, .. }, Edit::Sprite { after: newer, .. }) => *after = *newer,
            (Edit::Text { after, .. }, Edit::Text { after: newer, .. }) => after.clone_from(newer),
            (Edit::Kept { after, .. }, Edit::Kept { after: newer, .. }) => after.clone_from(newer),
            _ => return false,
        }
        true
    }

    /// A short description for the status line: "the move of #7 to (1.0, 2.0)".
    pub(crate) fn describe(&self) -> String {
        match self {
            &Edit::Transform {
                entity,
                before,
                after,
            } => {
                let only_moved = before.rotation == after.rotation && before.scale == after.scale;
                if only_moved {
                    format!(
                        "the move of #{} to ({:.1}, {:.1})",
                        entity.id(),
                        after.position.x,
                        after.position.y
                    )
                } else {
                    format!("the transform of #{}", entity.id())
                }
            }
            &Edit::Layer { entity, after, .. } => match after {
                Some(layer) => format!("layer {} of #{}", layer.0, entity.id()),
                None => format!("removing the layer of #{}", entity.id()),
            },
            &Edit::Hidden { entity, after, .. } => {
                let what = if after { "hiding" } else { "showing" };
                format!("{what} #{}", entity.id())
            }
            Edit::Quad { entity, .. } => format!("the quad of #{}", entity.id()),
            Edit::Sprite { entity, .. } => format!("the sprite of #{}", entity.id()),
            Edit::Text { entity, .. } => format!("the text of #{}", entity.id()),
            Edit::Kept { entity, name, .. } => format!("{name} of #{}", entity.id()),
        }
    }
}

/// Edits done and undone, and where the last save was.
#[derive(Debug)]
pub(crate) struct History {
    done: Vec<Edit>,
    undone: Vec<Edit>,
    /// How many edits were done when the scene was last saved (or opened);
    /// `None` once that state can no longer be reached by undo or redo.
    saved: Option<usize>,
}

impl History {
    /// An empty history of a scene just opened (not modified).
    pub(crate) fn new() -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            saved: Some(0),
        }
    }

    /// Records an edit that has already been applied to the world. With
    /// `merge`, an edit of the same kind on the same entity as the last one
    /// extends that edit instead (a burst of arrow-key steps is undone at
    /// once). An edit that changes nothing is not recorded. Recording forgets
    /// what was undone.
    pub(crate) fn record(&mut self, edit: Edit, merge: bool) {
        if self.saved.is_some_and(|saved| saved > self.done.len()) {
            // The saved state was among the undone edits, which are dropped.
            self.saved = None;
        }
        self.undone.clear();
        if merge
            && let Some(last) = self.done.last_mut()
            && last.absorb(&edit)
        {
            if self.saved == Some(self.done.len()) {
                // The saved state was the end of the edit just changed.
                self.saved = None;
            }
            return;
        }
        if !edit.changes_nothing() {
            self.done.push(edit);
        }
    }

    /// The edit to revert, moved to the redo list.
    pub(crate) fn undo(&mut self) -> Option<Edit> {
        let edit = self.done.pop()?;
        self.undone.push(edit.clone());
        Some(edit)
    }

    /// The edit to apply again, moved back to the undo list.
    pub(crate) fn redo(&mut self) -> Option<Edit> {
        let edit = self.undone.pop()?;
        self.done.push(edit.clone());
        Some(edit)
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// The scene was just saved.
    pub(crate) fn mark_saved(&mut self) {
        self.saved = Some(self.done.len());
    }

    /// Whether the scene differs from the last save.
    pub(crate) fn is_dirty(&self) -> bool {
        self.saved != Some(self.done.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32) -> Transform2D {
        Transform2D::from_position(Vec2::new(x, 0.0))
    }

    fn moved(entity: Entity, from: f32, to: f32) -> Edit {
        Edit::moved(entity, at(from), Vec2::new(to, 0.0))
    }

    fn x(world: &World, entity: Entity) -> f32 {
        world.get::<&Transform2D>(entity).expect("t").position.x
    }

    #[test]
    fn undo_and_redo_walk_the_edits_and_track_the_saved_state() {
        let mut world = World::new();
        let a = world.spawn((Transform2D::default(),));
        let mut history = History::new();
        assert!(!history.is_dirty() && !history.can_undo() && !history.can_redo());

        history.record(moved(a, 0.0, 5.0), false);
        history.record(moved(a, 5.0, 9.0), false);
        assert!(history.is_dirty());
        history.mark_saved();
        assert!(!history.is_dirty());

        let edit = history.undo().expect("one to undo");
        assert_eq!(edit, moved(a, 5.0, 9.0));
        assert!(edit.apply(&mut world, false));
        assert_eq!(x(&world, a), 5.0);
        assert!(history.is_dirty(), "one step away from the save");
        let again = history.redo().expect("one to redo");
        assert!(again.apply(&mut world, true));
        assert!(!history.is_dirty(), "back at the saved state");
        assert_eq!(x(&world, a), 9.0);

        // Undo past the save, then a new edit: the save is out of reach.
        history.undo();
        history.undo();
        history.record(moved(a, 0.0, 1.0), false);
        assert!(!history.can_redo(), "a new edit drops what was undone");
        history.undo();
        assert!(history.is_dirty(), "the saved state cannot come back");
        assert!(history.undo().is_none());
    }

    #[test]
    fn merged_steps_undo_together_and_empty_edits_are_not_recorded() {
        let mut world = World::new();
        let (a, b) = (
            world.spawn((Transform2D::default(),)),
            world.spawn((Transform2D::default(),)),
        );
        let mut history = History::new();
        history.record(moved(a, 0.0, 0.0), false);
        assert!(!history.can_undo(), "nothing moved");
        history.record(moved(a, 0.0, 1.0), true);
        history.record(moved(a, 1.0, 2.0), true);
        history.record(moved(a, 2.0, 3.0), true);
        history.record(moved(b, 0.0, 1.0), true); // another entity: separate
        let hide = Edit::Hidden {
            entity: b,
            before: false,
            after: true,
        };
        history.record(hide.clone(), true); // another kind: separate
        assert_eq!(history.undo(), Some(hide));
        assert_eq!(history.undo(), Some(moved(b, 0.0, 1.0)));
        assert_eq!(history.undo(), Some(moved(a, 0.0, 3.0)), "one step");
        assert_eq!(history.undo(), None);

        // Merging into the edit the last save ended with makes it dirty.
        history.redo();
        history.mark_saved();
        history.record(moved(a, 3.0, 4.0), true);
        assert!(history.is_dirty());
        history.undo();
        assert!(history.is_dirty(), "the merged edit is not the saved one");
    }

    #[test]
    fn every_kind_of_edit_applies_and_reverts() {
        let mut world = World::new();
        let e = world.spawn((at(1.0),));
        let turned = Transform2D {
            rotation: 0.5,
            scale: Vec2::new(2.0, -1.0),
            ..at(4.0)
        };
        let transform = Edit::Transform {
            entity: e,
            before: at(1.0),
            after: turned,
        };
        assert!(transform.apply(&mut world, true));
        assert_eq!(*world.get::<&Transform2D>(e).expect("t"), turned);
        assert!(transform.apply(&mut world, false));
        assert_eq!(*world.get::<&Transform2D>(e).expect("t"), at(1.0));

        let layer = Edit::Layer {
            entity: e,
            before: None,
            after: Some(Layer(3)),
        };
        assert!(layer.apply(&mut world, true));
        assert_eq!(world.get::<&Layer>(e).map(|l| *l).ok(), Some(Layer(3)));
        assert!(layer.apply(&mut world, false));
        assert!(world.get::<&Layer>(e).is_err(), "no layer component again");

        let hidden = Edit::Hidden {
            entity: e,
            before: false,
            after: true,
        };
        assert!(hidden.apply(&mut world, true));
        assert!(world.get::<&Hidden>(e).is_ok());
        assert!(hidden.apply(&mut world, false));
        assert!(world.get::<&Hidden>(e).is_err());

        // An edit started earlier ends with the current value.
        world.insert_one(e, Layer(-2)).expect("alive");
        let started = Edit::Layer {
            entity: e,
            before: None,
            after: None,
        };
        assert_eq!(
            started.ending_now(&world),
            Edit::Layer {
                entity: e,
                before: None,
                after: Some(Layer(-2))
            }
        );
        assert_eq!(layer.describe(), format!("layer 3 of #{}", e.id()));
        assert_eq!(
            moved(e, 0.0, 2.5).describe(),
            format!("the move of #{} to (2.5, 0.0)", e.id())
        );
    }

    #[test]
    fn edits_on_missing_entities_report_failure() {
        let mut world = World::new();
        let a = world.spawn((Transform2D::default(),));
        let bare = world.spawn((1_u8,));
        world.despawn(a).expect("alive");
        assert!(!moved(a, 0.0, 1.0).apply(&mut world, true));
        assert!(!moved(bare, 0.0, 1.0).apply(&mut world, true));
        let hidden = Edit::Hidden {
            entity: a,
            before: false,
            after: true,
        };
        assert!(!hidden.apply(&mut world, true), "gone");
    }

    #[test]
    fn quad_and_kept_component_edits_apply_revert_and_merge_per_target() {
        use purplepie::render::Color;
        let mut world = World::new();
        let mut kept = UnknownComponents::default();
        kept.set("game::Speed", "(value: 1.0)").expect("valid");
        kept.set("game::Tag", "Boss").expect("valid");
        let small = Quad::new(Vec2::new(10.0, 10.0), Color::WHITE);
        let e = world.spawn((Transform2D::default(), small, kept));
        let red = Quad::new(Vec2::new(10.0, 20.0), Color::rgb8(255, 0, 0));
        let quad = Edit::Quad {
            entity: e,
            before: small,
            after: red,
        };
        assert!(quad.apply(&mut world, true));
        assert_eq!(*world.get::<&Quad>(e).expect("quad"), red);
        assert!(quad.apply(&mut world, false));
        assert_eq!(*world.get::<&Quad>(e).expect("quad"), small);

        let speed = |before: &str, after: &str| Edit::Kept {
            entity: e,
            name: "game::Speed".to_owned(),
            before: before.to_owned(),
            after: after.to_owned(),
        };
        let faster = speed("(value: 1.0)", "(value: 3.0)");
        let text = |world: &World, name: &str| {
            world
                .get::<&UnknownComponents>(e)
                .expect("kept")
                .get(name)
                .map(str::to_owned)
        };
        assert!(faster.apply(&mut world, true));
        assert_eq!(text(&world, "game::Speed").as_deref(), Some("(value: 3.0)"));
        assert!(faster.apply(&mut world, false));
        assert_eq!(text(&world, "game::Speed").as_deref(), Some("(value: 1.0)"));
        assert_eq!(
            text(&world, "game::Tag").as_deref(),
            Some("Boss"),
            "untouched"
        );
        assert_eq!(faster.describe(), format!("game::Speed of #{}", e.id()));
        assert_eq!(quad.describe(), format!("the quad of #{}", e.id()));

        // Ending now reads the current value; reverting empties the change.
        world
            .get::<&mut UnknownComponents>(e)
            .expect("kept")
            .set("game::Speed", "(value: 5.0)")
            .expect("valid");
        assert_eq!(
            speed("(value: 1.0)", "(value: 1.0)").ending_now(&world),
            speed("(value: 1.0)", "(value: 5.0)")
        );
        let mut start = quad.clone();
        start.revert_to_before();
        assert!(start.changes_nothing() && quad.same_target(&start));

        // Another component name, kind or entity is another target.
        let tag = Edit::Kept {
            entity: e,
            name: "game::Tag".to_owned(),
            before: "Boss".to_owned(),
            after: "Minion".to_owned(),
        };
        assert!(!faster.same_target(&tag) && !faster.same_target(&quad));
        let mut history = History::new();
        history.record(faster.clone(), true);
        history.record(speed("(value: 3.0)", "(value: 4.0)"), true);
        history.record(tag.clone(), true);
        assert_eq!(history.undo(), Some(tag));
        assert_eq!(
            history.undo(),
            Some(speed("(value: 1.0)", "(value: 4.0)")),
            "merged on the same name"
        );

        // Without the component (or the entity), nothing is applied.
        let bare = world.spawn((Transform2D::default(),));
        let on_bare = Edit::Quad {
            entity: bare,
            before: small,
            after: red,
        };
        assert!(!on_bare.apply(&mut world, true));
        let kept_on_bare = Edit::Kept {
            entity: bare,
            name: "game::Speed".to_owned(),
            before: String::new(),
            after: "1".to_owned(),
        };
        assert!(!kept_on_bare.apply(&mut world, true));
    }
}
