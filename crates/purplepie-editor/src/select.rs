//! Choosing what a click selects, and how far keys and drags move it.

use purplepie::ecs::Entity;
use purplepie::math::Vec2;

/// What a click selects among `picked` (the entities under it, topmost first):
/// the one after the current selection when that is among them, so clicking
/// the same spot again cycles through a stack; otherwise the topmost one, or
/// nothing on an empty spot.
pub(crate) fn after_click(picked: &[Entity], current: Option<Entity>) -> Option<Entity> {
    let position = current.and_then(|c| picked.iter().position(|&e| e == c));
    match position {
        Some(i) => picked.get((i + 1) % picked.len()).copied(),
        None => picked.first().copied(),
    }
}

/// What a drag starting on `picked` moves: the current selection if it is
/// under the pointer, else the topmost entity there.
pub(crate) fn drag_target(picked: &[Entity], current: Option<Entity>) -> Option<Entity> {
    match current {
        Some(c) if picked.contains(&c) => Some(c),
        _ => picked.first().copied(),
    }
}

/// A drag of `delta` window pixels (+Y down) as a world offset at `zoom`.
pub(crate) fn drag_to_world(delta: Vec2, zoom: f32) -> Vec2 {
    Vec2::new(delta.x, -delta.y) / zoom
}

/// One arrow-key step: 1 world unit, 10 with Shift.
pub(crate) fn key_step(direction: Vec2, shift: bool) -> Vec2 {
    direction * if shift { 10.0 } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use purplepie::ecs::World;

    #[test]
    fn clicks_pick_the_top_then_cycle_and_drags_keep_the_selection() {
        let mut world = World::new();
        let [a, b, c] = [(), (), ()].map(|_| world.spawn((0_u8,)));
        let stack = [a, b, c];
        assert_eq!(after_click(&stack, None), Some(a));
        assert_eq!(after_click(&stack, Some(a)), Some(b));
        assert_eq!(after_click(&stack, Some(c)), Some(a), "wraps around");
        let elsewhere = world.spawn((0_u8,));
        assert_eq!(after_click(&stack, Some(elsewhere)), Some(a));
        assert_eq!(after_click(&[], Some(a)), None, "an empty spot clears");

        assert_eq!(drag_target(&stack, Some(c)), Some(c), "keeps the selection");
        assert_eq!(drag_target(&stack, Some(elsewhere)), Some(a));
        assert_eq!(drag_target(&[], Some(a)), None);
    }

    #[test]
    fn drags_and_keys_move_in_world_units() {
        assert_eq!(
            drag_to_world(Vec2::new(20.0, 10.0), 2.0),
            Vec2::new(10.0, -5.0)
        );
        assert_eq!(key_step(Vec2::new(0.0, 1.0), false), Vec2::new(0.0, 1.0));
        assert_eq!(key_step(Vec2::new(-1.0, 0.0), true), Vec2::new(-10.0, 0.0));
    }
}
