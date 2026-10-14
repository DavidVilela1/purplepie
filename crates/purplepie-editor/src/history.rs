//! Undo and redo (ADR-044): every change is a reversible [`Edit`] kept in a
//! [`History`], which also knows whether the scene differs from the last save.

use purplepie::ecs::{Entity, World};
use purplepie::math::{Transform2D, Vec2};

/// One reversible change to the scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Edit {
    /// An entity's position went from `from` to `to`.
    Move {
        entity: Entity,
        from: Vec2,
        to: Vec2,
    },
}

impl Edit {
    /// Puts the world in the state after this edit (`forward`) or before it.
    /// `false` if the entity is gone or has no transform.
    pub(crate) fn apply(&self, world: &mut World, forward: bool) -> bool {
        match *self {
            Edit::Move { entity, from, to } => match world.get::<&mut Transform2D>(entity) {
                Ok(mut transform) => {
                    transform.position = if forward { to } else { from };
                    true
                }
                Err(_) => false,
            },
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
    /// `merge`, a move of the same entity as the last edit extends that edit
    /// instead (a burst of arrow-key steps is undone at once). A move that
    /// changes nothing is not recorded. Recording forgets what was undone.
    pub(crate) fn record(&mut self, edit: Edit, merge: bool) {
        let Edit::Move { entity, from, to } = edit;
        if self.saved.is_some_and(|saved| saved > self.done.len()) {
            // The saved state was among the undone edits, which are dropped.
            self.saved = None;
        }
        self.undone.clear();
        if merge
            && let Some(Edit::Move {
                entity: last_entity,
                to: last_to,
                ..
            }) = self.done.last_mut()
            && *last_entity == entity
        {
            *last_to = to;
            if self.saved == Some(self.done.len()) {
                // The saved state was the end of the edit just changed.
                self.saved = None;
            }
            return;
        }
        if from != to {
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

    fn moved(entity: Entity, from: f32, to: f32) -> Edit {
        Edit::Move {
            entity,
            from: Vec2::new(from, 0.0),
            to: Vec2::new(to, 0.0),
        }
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
        assert_eq!(world.get::<&Transform2D>(a).expect("t").position.x, 5.0);
        assert!(history.is_dirty(), "one step away from the save");
        let again = history.redo().expect("one to redo");
        assert!(again.apply(&mut world, true));
        assert!(!history.is_dirty(), "back at the saved state");
        assert_eq!(world.get::<&Transform2D>(a).expect("t").position.x, 9.0);

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
    fn merged_steps_undo_together_and_empty_moves_are_not_recorded() {
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
    fn edits_on_missing_entities_report_failure() {
        let mut world = World::new();
        let a = world.spawn((Transform2D::default(),));
        let bare = world.spawn((1_u8,));
        world.despawn(a).expect("alive");
        assert!(!moved(a, 0.0, 1.0).apply(&mut world, true));
        assert!(!moved(bare, 0.0, 1.0).apply(&mut world, true));
    }
}
