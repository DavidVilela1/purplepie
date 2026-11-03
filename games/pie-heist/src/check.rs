//! What makes a level file playable, checked on every shipped level by
//! `tests/levels.rs` (PP-042d1, ADR-045): the level is loaded strictly with
//! [`purplepie::Headless`], exactly as the game loads it, and then
//! [`problems`] and [`order`] say what is wrong in words.

use std::collections::{BTreeMap, BTreeSet};

use purplepie::ecs::{Entity, World};
use purplepie::math::{Rect, Transform2D, Vec2};
use purplepie::render::{Quad, Sprite};

use crate::{Door, Exit, Guard, Key, PLAYER_SIZE, Pie, PlayerStart, Wall, wall_rect};

/// What is wrong with a loaded level, one sentence each; empty when it is
/// playable as far as the rules can tell:
///
/// - exactly one `PlayerStart` and one `Exit`;
/// - walls and doors are quads, not rotated;
/// - pies, keys, exits, guards and the start have a sprite or a quad (what
///   the player touches) and a transform;
/// - the player, placed at the start, touches no wall or door;
/// - every door's colour has a key in the level.
///
/// Whether every pie and key can be reached is not checked (that is a
/// path-finding question, left to playing the level).
pub fn problems(world: &World) -> Vec<String> {
    let mut found = Vec::new();
    let at = |e: Entity| {
        world
            .get::<&Transform2D>(e)
            .map(|t| format!("at ({}, {})", t.position.x, t.position.y))
            .unwrap_or_else(|_| "without a transform".to_owned())
    };

    let starts = entities::<PlayerStart>(world);
    if starts.len() != 1 {
        found.push(format!(
            "{} player starts (a level needs exactly one)",
            starts.len()
        ));
    }
    let exits = entities::<Exit>(world);
    if exits.len() != 1 {
        found.push(format!("{} exits (a level needs exactly one)", exits.len()));
    }

    let mut solid = Vec::new();
    for (kind, list) in [
        ("wall", entities::<Wall>(world)),
        ("door", entities::<Door>(world)),
    ] {
        for e in list {
            let quad = world.get::<&Quad>(e).ok().map(|q| *q);
            let transform = world.get::<&Transform2D>(e).ok().map(|t| *t);
            match (transform, quad) {
                (Some(t), Some(q)) => match wall_rect(&t, &q) {
                    Some(rect) => solid.push(rect),
                    None => found.push(format!("a {kind} {} is rotated", at(e))),
                },
                _ => found.push(format!("a {kind} {} is not a quad with a transform", at(e))),
            }
        }
    }

    for (kind, list) in [
        ("pie", entities::<Pie>(world)),
        ("key", entities::<Key>(world)),
        ("exit", exits.clone()),
        ("guard", entities::<Guard>(world)),
        ("player start", starts.clone()),
    ] {
        for e in list {
            let drawn = world.get::<&Sprite>(e).is_ok() || world.get::<&Quad>(e).is_ok();
            if !drawn || world.get::<&Transform2D>(e).is_err() {
                found.push(format!("a {kind} {} has no sprite or quad to touch", at(e)));
            }
        }
    }

    for &start in &starts {
        if let Ok(t) = world.get::<&Transform2D>(start) {
            let player = Rect::from_center_size(t.position, Vec2::splat(PLAYER_SIZE));
            if solid.iter().any(|wall| player.overlaps(*wall)) {
                found.push(format!(
                    "the player start {} touches a wall or a door",
                    at(start)
                ));
            }
        }
    }

    let keys: BTreeSet<String> = world
        .query::<&Key>()
        .iter()
        .map(|key| format!("{:?}", key.color))
        .collect();
    let doors: BTreeSet<String> = world
        .query::<&Door>()
        .iter()
        .map(|door| format!("{:?}", door.color))
        .collect();
    for color in doors.difference(&keys) {
        found.push(format!("a {color} door, but no {color} key"));
    }
    found
}

/// The entities with a `T`, in entity order.
fn entities<T: purplepie::ecs::hecs::Component>(world: &World) -> Vec<Entity> {
    let mut list: Vec<Entity> = world
        .query::<(Entity, &T)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    list.sort_by_key(|e| e.id());
    list
}

/// The levels in the order their exits lead, from `first`, given each level
/// file and where its exit leads (`None`: the last level). Fails, in words,
/// when an exit leads to a file that is not a level, when the exits loop,
/// when `first` is not a level, or when a level is never reached.
pub fn order(first: &str, next: &BTreeMap<String, Option<String>>) -> Result<Vec<String>, String> {
    let mut visited = Vec::new();
    let mut current = first.to_owned();
    loop {
        let Some(leads_to) = next.get(&current) else {
            return Err(match visited.last() {
                Some(from) => {
                    format!("the exit of {from} leads to {current}, which is not a level")
                }
                None => format!("the first level {current} is not a level"),
            });
        };
        if visited.contains(&current) {
            return Err(format!("the exits loop back to {current}"));
        }
        visited.push(current);
        match leads_to {
            Some(level) => current = level.clone(),
            None => break,
        }
    }
    let unreached: Vec<&str> = next
        .keys()
        .filter(|level| !visited.contains(level))
        .map(String::as_str)
        .collect();
    if unreached.is_empty() {
        Ok(visited)
    } else {
        Err(format!("no exit leads to {}", unreached.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyColor;
    use purplepie::render::Color;

    fn quad(world: &mut World, x: f32, y: f32, w: f32, h: f32) -> Entity {
        world.spawn((
            Transform2D::from_position(Vec2::new(x, y)),
            Quad::new(Vec2::new(w, h), Color::WHITE),
        ))
    }

    /// A playable room: walls around, a start, a pie, a red door and key, an exit.
    fn room() -> World {
        let mut world = World::new();
        for (x, y, w, h) in [
            (0.0, 200.0, 620.0, 20.0),
            (0.0, -200.0, 620.0, 20.0),
            (-300.0, 0.0, 20.0, 400.0),
            (300.0, 0.0, 20.0, 400.0),
        ] {
            let wall = quad(&mut world, x, y, w, h);
            world.insert_one(wall, Wall).expect("wall");
        }
        let door = quad(&mut world, 100.0, 0.0, 20.0, 380.0);
        world
            .insert_one(
                door,
                Door {
                    color: KeyColor::Red,
                },
            )
            .expect("door");
        for (x, component) in [(-200.0, 0), (0.0, 1), (-100.0, 2), (200.0, 3)] {
            let e = quad(&mut world, x, 0.0, 32.0, 32.0);
            match component {
                0 => world.insert_one(e, PlayerStart),
                1 => world.insert_one(e, Pie),
                2 => world.insert_one(
                    e,
                    Key {
                        color: KeyColor::Red,
                    },
                ),
                _ => world.insert_one(e, Exit::default()),
            }
            .expect("component");
        }
        world
    }

    #[test]
    fn a_playable_room_has_no_problems() {
        assert_eq!(problems(&room()), Vec::<String>::new());
    }

    #[test]
    fn starts_exits_rotations_touching_and_keys_are_reported() {
        let mut world = room();
        // A second start, inside the left wall.
        let start = quad(&mut world, -295.0, 0.0, 32.0, 32.0);
        world.insert_one(start, PlayerStart).expect("start");
        // No exit.
        let exits: Vec<Entity> = entities::<Exit>(&world);
        for e in exits {
            world.despawn(e).expect("exit");
        }
        // A rotated wall, and a wall with no quad.
        let tilted = quad(&mut world, 0.0, 100.0, 50.0, 10.0);
        world
            .insert(
                tilted,
                (
                    Wall,
                    Transform2D {
                        rotation: 0.5,
                        ..Transform2D::from_position(Vec2::new(0.0, 100.0))
                    },
                ),
            )
            .expect("tilted");
        world.spawn((Transform2D::default(), Wall));
        // A gold door without a gold key, and a pie nobody can touch.
        let gold = quad(&mut world, 150.0, 0.0, 20.0, 100.0);
        world
            .insert_one(
                gold,
                Door {
                    color: KeyColor::Gold,
                },
            )
            .expect("door");
        world.spawn((Transform2D::default(), Pie));

        assert_eq!(
            problems(&world),
            [
                "2 player starts (a level needs exactly one)",
                "0 exits (a level needs exactly one)",
                "a wall at (0, 100) is rotated",
                "a wall at (0, 0) is not a quad with a transform",
                "a pie at (0, 0) has no sprite or quad to touch",
                "the player start at (-295, 0) touches a wall or a door",
                "a Gold door, but no Gold key",
            ]
        );
    }

    fn chain(links: &[(&str, Option<&str>)]) -> BTreeMap<String, Option<String>> {
        links
            .iter()
            .map(|(level, next)| ((*level).to_owned(), next.map(str::to_owned)))
            .collect()
    }

    #[test]
    fn exits_lead_through_every_level_to_a_last_one() {
        let good = chain(&[("a", Some("b")), ("b", Some("c")), ("c", None)]);
        assert_eq!(
            order("a", &good),
            Ok(vec!["a".into(), "b".into(), "c".into()])
        );
        assert_eq!(
            order("a", &chain(&[("a", Some("x")), ("b", None)])),
            Err("the exit of a leads to x, which is not a level".into())
        );
        assert_eq!(
            order("a", &chain(&[("a", Some("b")), ("b", Some("a"))])),
            Err("the exits loop back to a".into())
        );
        assert_eq!(
            order("a", &chain(&[("a", None), ("b", None), ("c", None)])),
            Err("no exit leads to b, c".into())
        );
        assert_eq!(
            order("z", &good),
            Err("the first level z is not a level".into())
        );
    }
}
