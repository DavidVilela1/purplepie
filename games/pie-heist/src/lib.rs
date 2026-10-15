//! Pie Heist (docs/GAME3.md): the game's components and level rules, shared by
//! the game (`src/main.rs`) and its scene editor (`games/pie-heist-editor`).
//!
//! A level is a scene file drawn in the editor: walls, pies, keys, doors,
//! exits, guards and the player's start are the game components below, put on
//! ordinary quads and sprites. The game registers them ([`register`]) and
//! loads a level strictly, so a level that uses a misspelt or unknown
//! component does not load.

use purplepie::Context;
use purplepie::math::{Rect, Transform2D, Vec2};
use purplepie::render::Quad;
use serde::{Deserialize, Serialize};

/// The names the components are saved under in level files.
pub mod names {
    pub const WALL: &str = "pie_heist::Wall";
    pub const PIE: &str = "pie_heist::Pie";
    pub const KEY: &str = "pie_heist::Key";
    pub const DOOR: &str = "pie_heist::Door";
    pub const EXIT: &str = "pie_heist::Exit";
    pub const GUARD: &str = "pie_heist::Guard";
    pub const PLAYER_START: &str = "pie_heist::PlayerStart";
}

/// A quad the player cannot cross (axis-aligned: not rotated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Wall;

/// Collected by touching it; every pie of a level opens its exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Pie;

/// The colour that ties keys to doors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KeyColor {
    #[default]
    Red,
    Blue,
    Gold,
}

/// Picked up by touching it; opens every door of its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Key {
    pub color: KeyColor,
}

/// A wall until a key of its colour is picked up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Door {
    pub color: KeyColor,
}

/// Leads to the next level (a level file relative to the asset folder), or
/// ends the game when `next` is empty.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Exit {
    pub next: String,
}

/// Walks `path` (points relative to where the guard starts, looping) at
/// `speed` units per second; touching it restarts the level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guard {
    pub path: Vec<(f32, f32)>,
    pub speed: f32,
}

impl Default for Guard {
    /// A short back-and-forth walk, so a guard added in the editor moves.
    fn default() -> Self {
        Self {
            path: vec![(0.0, 0.0), (200.0, 0.0)],
            speed: 90.0,
        }
    }
}

/// Where the player appears; the game hides the entity that carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerStart;

/// Registers every component under its name in [`names`], before a level is
/// loaded.
pub fn register(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    ctx.register_scene_component::<Wall>(names::WALL)?;
    ctx.register_scene_component::<Pie>(names::PIE)?;
    ctx.register_scene_component::<Key>(names::KEY)?;
    ctx.register_scene_component::<Door>(names::DOOR)?;
    ctx.register_scene_component::<Exit>(names::EXIT)?;
    ctx.register_scene_component::<Guard>(names::GUARD)?;
    ctx.register_scene_component::<PlayerStart>(names::PLAYER_START)?;
    Ok(())
}

/// The area a wall's quad covers, or `None` if the quad is rotated (walls
/// must be axis-aligned).
pub fn wall_rect(transform: &Transform2D, quad: &Quad) -> Option<Rect> {
    if transform.rotation.rem_euclid(std::f32::consts::PI) > 1e-4
        && (std::f32::consts::PI - transform.rotation.rem_euclid(std::f32::consts::PI)) > 1e-4
    {
        return None;
    }
    // Half turns keep a rectangle axis-aligned with the same size.
    Some(Rect::from_center_size(
        transform.position,
        quad.size * transform.scale.abs(),
    ))
}

/// Moves a box of `size` centred at `position` by `delta`, stopping at
/// `walls` and sliding along them: the move is done along x, then along y,
/// and each part stops at the face of the nearest wall in the way, however
/// long the step (nothing is skipped through). Walls the box already overlaps
/// do not block, so a box placed inside one can walk out.
pub fn move_and_slide(position: Vec2, size: Vec2, delta: Vec2, walls: &[Rect]) -> Vec2 {
    let half = size * 0.5;
    let mut at = position;
    for axis in 0..2 {
        let step = delta[axis];
        if step == 0.0 {
            continue;
        }
        let other = 1 - axis;
        let mut to = at[axis] + step;
        for wall in walls {
            // Only walls beside the box's path across the other axis.
            let gap = (wall.center[other] - at[other]).abs();
            if gap >= wall.half_size[other] + half[other] {
                continue;
            }
            if step > 0.0 {
                let face = wall.min()[axis] - half[axis];
                if face >= at[axis] {
                    to = to.min(face);
                }
            } else {
                let face = wall.max()[axis] + half[axis];
                if face <= at[axis] {
                    to = to.max(face);
                }
            }
        }
        at[axis] = to;
    }
    at
}

/// What blocks the player: the walls, and every door whose colour is not
/// among the keys held.
pub fn blocking(walls: &[Rect], doors: &[(KeyColor, Rect)], keys: &[KeyColor]) -> Vec<Rect> {
    walls
        .iter()
        .copied()
        .chain(
            doors
                .iter()
                .filter(|(color, _)| !keys.contains(color))
                .map(|(_, rect)| *rect),
        )
        .collect()
}

impl KeyColor {
    /// How keys and doors of this colour are drawn (the game colours them
    /// when a level loads, whatever the level file says).
    pub fn color(self) -> purplepie::render::Color {
        use purplepie::render::Color;
        match self {
            KeyColor::Red => Color::hex(0xEF476F),
            KeyColor::Blue => Color::hex(0x4CC9F0),
            KeyColor::Gold => Color::hex(0xFFD60A),
        }
    }
}

/// Where a guard is, relative to where it starts, `seconds` after the level
/// began: it walks from `path`'s first point to the next, then on to the
/// last, and back to the first (the path loops; two points walk back and
/// forth) at `speed` units per second. A path of one point, a speed that is
/// zero, negative or not a number, or a path whose points are all the same
/// keeps the guard on the first point; an empty path, on its start.
pub fn patrol_offset(path: &[(f32, f32)], speed: f32, seconds: f32) -> Vec2 {
    let points: Vec<Vec2> = path.iter().map(|&(x, y)| Vec2::new(x, y)).collect();
    let Some(&first) = points.first() else {
        return Vec2::ZERO;
    };
    let legs = || {
        points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .map(|(&from, &to)| (from, to))
    };
    let length: f32 = legs().map(|(from, to)| from.distance(to)).sum();
    let walked = speed * seconds;
    if !(length > 0.0 && walked.is_finite() && speed > 0.0) {
        return first;
    }
    let mut left = walked.rem_euclid(length);
    for (from, to) in legs() {
        let leg = from.distance(to);
        if left <= leg {
            return from.lerp(to, if leg > 0.0 { left / leg } else { 0.0 });
        }
        left -= leg;
    }
    first
}

/// The area a sprite or quad of `size` covers with `transform` (scale applied,
/// rotation ignored): what the player touches to pick something up.
pub fn drawn_rect(transform: &Transform2D, size: Vec2) -> Rect {
    Rect::from_center_size(transform.position, size * transform.scale.abs())
}

/// The indices of `items` that `player` overlaps (touching edges do not
/// count), in order.
pub fn touched(player: Rect, items: &[Rect]) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| player.overlaps(**item))
        .map(|(i, _)| i)
        .collect()
}

/// The level an exit leads to, or `None` when it ends the game.
pub fn next_level(exit: &Exit) -> Option<&str> {
    let next = exit.next.trim();
    (!next.is_empty()).then_some(next)
}

/// "Level 1" for `levels/level1.ron`: the file's name without folder and
/// extension, with a space before its number.
pub fn level_title(path: &str) -> String {
    let stem = std::path::Path::new(path)
        .file_stem()
        .map_or_else(|| path.to_owned(), |s| s.to_string_lossy().into_owned());
    let split = stem
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(stem.len());
    let (word, number) = stem.split_at(split);
    let mut title: String = word
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect();
    if !number.is_empty() {
        title.push(' ');
        title.push_str(number);
    }
    title
}

/// The smallest rectangle around every wall: the level's bounds for the
/// camera. `None` without walls.
pub fn level_bounds(walls: &[Rect]) -> Option<Rect> {
    let first = walls.first()?;
    let (mut min, mut max) = (first.min(), first.max());
    for wall in &walls[1..] {
        min = min.min(wall.min());
        max = max.max(wall.max());
    }
    Some(Rect::from_corners(min, max))
}

/// Where the camera looks to follow `target` without showing outside
/// `bounds`: along an axis where the level is smaller than the view, it stays
/// on the level's centre.
pub fn camera_target(target: Vec2, half_view: Vec2, bounds: Rect) -> Vec2 {
    let room = (bounds.half_size - half_view).max(Vec2::ZERO);
    target.clamp(bounds.center - room, bounds.center + room)
}

#[cfg(test)]
mod tests {
    use super::*;
    use purplepie::render::Color;

    fn wall(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_center_size(Vec2::new(x, y), Vec2::new(w, h))
    }

    #[test]
    fn the_player_stops_at_walls_and_slides_along_them() {
        let size = Vec2::splat(20.0);
        // A wall to the right: x stops at its face, y still moves.
        let walls = [wall(50.0, 0.0, 20.0, 200.0)];
        let to = move_and_slide(Vec2::ZERO, size, Vec2::new(100.0, 30.0), &walls);
        assert_eq!(to, Vec2::new(30.0, 30.0));
        // Moving left, away from it, is free.
        let back = move_and_slide(to, size, Vec2::new(-10.0, 0.0), &walls);
        assert_eq!(back, Vec2::new(20.0, 30.0));
        // A floor below: y stops on top of it.
        let floor = [wall(0.0, -50.0, 400.0, 20.0)];
        let down = move_and_slide(Vec2::ZERO, size, Vec2::new(5.0, -100.0), &floor);
        assert_eq!(down, Vec2::new(5.0, -30.0));
        // Touching a wall is not overlapping it: standing there is allowed.
        let stay = move_and_slide(Vec2::new(30.0, 0.0), size, Vec2::new(0.0, 10.0), &walls);
        assert_eq!(stay, Vec2::new(30.0, 10.0));
        // A corner: both axes stop.
        let corner = [wall(50.0, 0.0, 20.0, 200.0), wall(0.0, 50.0, 200.0, 20.0)];
        let into = move_and_slide(Vec2::ZERO, size, Vec2::new(100.0, 100.0), &corner);
        assert_eq!(into, Vec2::new(30.0, 30.0));
        // A step longer than the wall is thick does not pass through it.
        let thin = [wall(50.0, 0.0, 2.0, 200.0)];
        let long = move_and_slide(Vec2::ZERO, size, Vec2::new(500.0, 0.0), &thin);
        assert_eq!(long, Vec2::new(39.0, 0.0));
        // A box that starts inside a wall can walk out of it.
        let out = move_and_slide(Vec2::new(50.0, 0.0), size, Vec2::new(-5.0, 0.0), &thin);
        assert_eq!(out, Vec2::new(45.0, 0.0));
        // Walls beyond the path on the other axis do not block.
        let above = [wall(50.0, 100.0, 20.0, 20.0)];
        let past = move_and_slide(Vec2::ZERO, size, Vec2::new(100.0, 0.0), &above);
        assert_eq!(past, Vec2::new(100.0, 0.0));
    }

    #[test]
    fn walls_are_axis_aligned_quads_and_bound_the_level() {
        let quad = Quad::new(Vec2::new(100.0, 20.0), Color::WHITE);
        let straight = Transform2D {
            scale: Vec2::new(-2.0, 1.0),
            ..Transform2D::from_position(Vec2::new(10.0, 5.0))
        };
        assert_eq!(
            wall_rect(&straight, &quad),
            Some(wall(10.0, 5.0, 200.0, 20.0)),
            "mirroring keeps the size"
        );
        let half_turn = Transform2D {
            rotation: std::f32::consts::PI,
            ..straight
        };
        assert!(wall_rect(&half_turn, &quad).is_some());
        let tilted = Transform2D {
            rotation: 0.3,
            ..straight
        };
        assert_eq!(wall_rect(&tilted, &quad), None);

        let walls = [wall(0.0, 0.0, 20.0, 20.0), wall(100.0, -50.0, 40.0, 10.0)];
        let bounds = level_bounds(&walls).expect("walls");
        assert_eq!(bounds.min(), Vec2::new(-10.0, -55.0));
        assert_eq!(bounds.max(), Vec2::new(120.0, 10.0));
        assert_eq!(level_bounds(&[]), None);
    }

    #[test]
    fn the_camera_follows_inside_the_level() {
        let bounds = wall(0.0, 0.0, 1000.0, 400.0);
        let half_view = Vec2::new(300.0, 300.0);
        // x follows up to the edge; y is smaller than the view: centred.
        assert_eq!(
            camera_target(Vec2::new(100.0, 150.0), half_view, bounds),
            Vec2::new(100.0, 0.0)
        );
        assert_eq!(
            camera_target(Vec2::new(480.0, 0.0), half_view, bounds),
            Vec2::new(200.0, 0.0)
        );
    }

    #[test]
    fn pickups_exits_and_titles() {
        let player = wall(0.0, 0.0, 28.0, 28.0);
        let items = [
            wall(20.0, 0.0, 16.0, 16.0),  // overlaps
            wall(22.0, 0.0, 16.0, 16.0),  // touches the edge only
            wall(-10.0, 5.0, 16.0, 16.0), // overlaps
        ];
        assert_eq!(touched(player, &items), [0, 2]);
        let scaled = Transform2D {
            scale: Vec2::new(-2.0, 1.0),
            ..Transform2D::from_position(Vec2::new(5.0, 6.0))
        };
        assert_eq!(
            drawn_rect(&scaled, Vec2::splat(10.0)).size(),
            Vec2::new(20.0, 10.0)
        );

        let onward = Exit {
            next: " levels/level2.ron ".to_owned(),
        };
        assert_eq!(next_level(&onward), Some("levels/level2.ron"));
        assert_eq!(next_level(&Exit::default()), None, "the last level");

        assert_eq!(level_title("levels/level1.ron"), "Level 1");
        assert_eq!(level_title("levels/vault12.ron"), "Vault 12");
        assert_eq!(level_title("bonus.ron"), "Bonus");
    }

    #[test]
    fn keys_open_only_the_doors_of_their_colour() {
        let walls = [wall(0.0, 0.0, 10.0, 10.0)];
        let red = wall(100.0, 0.0, 10.0, 50.0);
        let gold = wall(200.0, 0.0, 10.0, 50.0);
        let doors = [(KeyColor::Red, red), (KeyColor::Gold, gold)];
        assert_eq!(blocking(&walls, &doors, &[]), [walls[0], red, gold]);
        assert_eq!(blocking(&walls, &doors, &[KeyColor::Gold]), [walls[0], red]);
        assert_eq!(
            blocking(&walls, &doors, &[KeyColor::Blue, KeyColor::Red]),
            [walls[0], gold]
        );
        // A closed door stops the player like a wall.
        let size = Vec2::splat(20.0);
        let solid = blocking(&walls, &doors, &[]);
        let stopped = move_and_slide(Vec2::new(50.0, 0.0), size, Vec2::new(200.0, 0.0), &solid);
        assert_eq!(stopped, Vec2::new(85.0, 0.0));
        let open = blocking(&walls, &doors, &[KeyColor::Red]);
        let through = move_and_slide(Vec2::new(50.0, 0.0), size, Vec2::new(100.0, 0.0), &open);
        assert_eq!(through, Vec2::new(150.0, 0.0));
        assert_ne!(KeyColor::Red.color(), KeyColor::Gold.color());
    }

    #[test]
    fn guards_walk_their_path_in_a_loop() {
        let close = |a: Vec2, b: Vec2| (a - b).length() < 1e-3;
        // Two points: back and forth, 400 units a round at 100 units/s.
        let line = [(0.0, 0.0), (200.0, 0.0)];
        assert_eq!(patrol_offset(&line, 100.0, 0.0), Vec2::ZERO);
        assert!(close(
            patrol_offset(&line, 100.0, 1.5),
            Vec2::new(150.0, 0.0)
        ));
        assert!(close(
            patrol_offset(&line, 100.0, 3.0),
            Vec2::new(100.0, 0.0)
        ));
        assert!(close(
            patrol_offset(&line, 100.0, 4.5),
            Vec2::new(50.0, 0.0)
        ));
        // Three points: the last leg goes back to the first point.
        let triangle = [(0.0, 0.0), (30.0, 0.0), (30.0, 40.0)];
        assert!(close(
            patrol_offset(&triangle, 10.0, 5.0),
            Vec2::new(30.0, 20.0)
        ));
        assert!(close(
            patrol_offset(&triangle, 10.0, 9.5),
            Vec2::new(15.0, 20.0)
        ));
        assert!(close(patrol_offset(&triangle, 10.0, 12.0), Vec2::ZERO));
        // A path that does not start at (0, 0) starts the guard elsewhere.
        let offset = [(0.0, -50.0), (0.0, 50.0)];
        assert_eq!(patrol_offset(&offset, 25.0, 0.0), Vec2::new(0.0, -50.0));
        // Standing still: empty, one point, repeated points, no speed.
        assert_eq!(patrol_offset(&[], 100.0, 3.0), Vec2::ZERO);
        assert_eq!(
            patrol_offset(&[(5.0, 6.0)], 100.0, 3.0),
            Vec2::new(5.0, 6.0)
        );
        assert_eq!(
            patrol_offset(&[(5.0, 6.0), (5.0, 6.0)], 100.0, 3.0),
            Vec2::new(5.0, 6.0)
        );
        for speed in [0.0, -10.0, f32::NAN, f32::INFINITY] {
            assert_eq!(patrol_offset(&line, speed, 3.0), Vec2::ZERO, "{speed}");
        }
        // A long time is still on the path.
        let late = patrol_offset(&line, 100.0, 3600.0 + 1.0);
        assert!(close(late, Vec2::new(100.0, 0.0)), "{late}");
    }

    #[test]
    fn components_have_the_documented_defaults() {
        assert_eq!(Key::default().color, KeyColor::Red);
        assert_eq!(Guard::default().path, vec![(0.0, 0.0), (200.0, 0.0)]);
        assert_eq!(Exit::default().next, "");
    }
}
