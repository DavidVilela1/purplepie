//! Undo and redo (ADR-044): every change is a reversible [`Edit`] (one
//! component's value on one entity, before and after) kept in a [`History`],
//! which also knows whether the scene differs from the last save.

use purplepie::ecs::{Entity, World};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Hidden, Layer};

/// One reversible change to the scene: one component of one entity.
#[derive(Debug, Clone, Copy, PartialEq)]
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
            | Edit::Hidden { entity, .. } => entity,
        }
    }

    /// `true` when before and after are the same.
    pub(crate) fn changes_nothing(&self) -> bool {
        match self {
            Edit::Transform { before, after, .. } => before == after,
            Edit::Layer { before, after, .. } => before == after,
            Edit::Hidden { before, after, .. } => before == after,
        }
    }

    /// The same edit ending in `entity`'s current value of this component
    /// (an edit started in the inspector, finished when the field is left).
    pub(crate) fn ending_now(self, world: &World) -> Self {
        match self {
            Edit::Transform { entity, before, .. } => Edit::Transform {
                entity,
                before,
                after: world.get::<&Transform2D>(entity).map_or(before, |t| *t),
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
        }
    }

    /// Puts the world in the state after this edit (`forward`) or before it.
    /// `false` if the entity is gone (or, for a transform, has none).
    pub(crate) fn apply(&self, world: &mut World, forward: bool) -> bool {
        if !world.contains(self.entity()) {
            return false;
        }
        match *self {
            Edit::Transform {
                entity,
                before,
                after,
            } => match world.get::<&mut Transform2D>(entity) {
                Ok(mut transform) => {
                    *transform = if forward { after } else { before };
                    true
                }
                Err(_) => false,
            },
            Edit::Layer {
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
            Edit::Hidden {
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
        }
    }

    /// Extends this edit with a newer one of the same kind on the same
    /// entity (keeping this one's `before`); `false` if they do not match.
    fn absorb(&mut self, newer: &Edit) -> bool {
        match (self, newer) {
            (
                Edit::Transform { entity, after, .. },
                Edit::Transform {
                    entity: e,
                    after: newer_after,
                    ..
                },
            ) if entity == e => {
                *after = *newer_after;
                true
            }
            (
                Edit::Layer { entity, after, .. },
                Edit::Layer {
                    entity: e,
                    after: newer_after,
                    ..
                },
            ) if entity == e => {
                *after = *newer_after;
                true
            }
            (
                Edit::Hidden { entity, after, .. },
                Edit::Hidden {
                    entity: e,
                    after: newer_after,
                    ..
                },
            ) if entity == e => {
                *after = *newer_after;
                true
            }
            _ => false,
        }
    }

    /// A short description for the status line: "the move of #7 to (1.0, 2.0)".
    pub(crate) fn describe(&self) -> String {
        match *self {
            Edit::Transform {
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
            Edit::Layer { entity, after, .. } => match after {
                Some(layer) => format!("layer {} of #{}", layer.0, entity.id()),
                None => format!("removing the layer of #{}", entity.id()),
            },
            Edit::Hidden { entity, after, .. } => {
                let what = if after { "hiding" } else { "showing" };
                format!("{what} #{}", entity.id())
            }
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
        self.undone.push(edit);
        Some(edit)
    }

    /// The edit to apply again, moved back to the undo list.
    pub(crate) fn redo(&mut self) -> Option<Edit> {
        let edit = self.undone.pop()?;
        self.done.push(edit);
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
        history.record(hide, true); // another kind: separate
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
}
