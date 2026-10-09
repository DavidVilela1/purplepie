//! Enemies: what they are, how waves grow, and how they move.

use purplepie::ecs::{Velocity, World};
use purplepie::math::{Rng, Transform2D, Vec2};
use purplepie::render::{Color, Sprite};

/// Seconds per wave.
pub const WAVE_LENGTH: f32 = 20.0;
/// No new enemy spawns while this many are alive.
pub const MAX_ENEMIES: usize = 120;

/// The two enemy types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// Slow, walks straight at the player.
    Drifter,
    /// Waits, then dashes in a straight line.
    Dasher,
}

impl Kind {
    pub const fn hit_points(self) -> u32 {
        match self {
            Kind::Drifter => 1,
            Kind::Dasher => 2,
        }
    }

    pub const fn points(self) -> u32 {
        match self {
            Kind::Drifter => 10,
            Kind::Dasher => 25,
        }
    }

    pub const fn size(self) -> f32 {
        match self {
            Kind::Drifter => 34.0,
            Kind::Dasher => 38.0,
        }
    }
}

/// What a Dasher is doing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dash {
    /// Standing still for this many seconds.
    Waiting(f32),
    /// Moving with this velocity for this many seconds.
    Dashing(f32, Vec2),
}

/// An enemy entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    pub kind: Kind,
    pub hit_points: u32,
    /// Seconds of white flash left after being hit.
    pub flash: f32,
    pub dash: Dash,
}

impl Enemy {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            hit_points: kind.hit_points(),
            flash: 0.0,
            dash: Dash::Waiting(1.0),
        }
    }

    /// Collision radius (a little smaller than the sprite, which feels fair).
    pub fn radius(&self) -> f32 {
        self.kind.size() * 0.4
    }
}

/// Seconds between spawns during `wave` (1, 2, …): faster each wave.
pub fn spawn_interval(wave: u32) -> f32 {
    (1.6 - 0.15 * wave.saturating_sub(1) as f32).max(0.25)
}

/// Chance that a new enemy is a Dasher during `wave`: none in wave 1.
pub fn dasher_chance(wave: u32) -> f32 {
    (0.08 * wave.saturating_sub(1) as f32).min(0.4)
}

/// Picks the kind of the next enemy.
pub fn pick_kind(rng: &mut Rng, wave: u32) -> Kind {
    if rng.chance(dasher_chance(wave)) {
        Kind::Dasher
    } else {
        Kind::Drifter
    }
}

/// Steers every enemy for one step toward `player`: Drifters walk at it,
/// Dashers wait and then dash at where it was. Also counts down hit flashes.
pub fn steer(world: &mut World, player: Vec2, wave: u32, dt: f32) {
    let drift_speed = 90.0 + 6.0 * wave as f32;
    for (t, enemy, velocity) in world.query_mut::<(&Transform2D, &mut Enemy, &mut Velocity)>() {
        let to_player = (player - t.position).normalize_or_zero();
        velocity.0 = match enemy.kind {
            Kind::Drifter => to_player * drift_speed,
            Kind::Dasher => match enemy.dash {
                Dash::Waiting(left) if left > dt => {
                    enemy.dash = Dash::Waiting(left - dt);
                    Vec2::ZERO
                }
                Dash::Waiting(_) => {
                    let v = to_player * 430.0;
                    enemy.dash = Dash::Dashing(0.6, v);
                    v
                }
                Dash::Dashing(left, v) if left > dt => {
                    enemy.dash = Dash::Dashing(left - dt, v);
                    v
                }
                Dash::Dashing(..) => {
                    enemy.dash = Dash::Waiting(0.9);
                    Vec2::ZERO
                }
            },
        };
        enemy.flash = (enemy.flash - dt).max(0.0);
    }
}

/// Shows hit flashes: an enemy that was just hit is drawn half transparent.
pub fn show_flashes(world: &mut World) {
    for (enemy, sprite) in world.query_mut::<(&Enemy, &mut Sprite)>() {
        sprite.tint = if enemy.flash > 0.0 {
            Color::rgba(1.0, 1.0, 1.0, 0.5)
        } else {
            Color::WHITE
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waves_get_faster_and_bring_dashers_later() {
        assert_eq!(spawn_interval(1), 1.6);
        assert!(spawn_interval(2) < spawn_interval(1));
        assert_eq!(spawn_interval(50), 0.25, "never faster than 4 per second");
        assert_eq!(dasher_chance(1), 0.0, "wave 1 is Drifters only");
        assert!(dasher_chance(3) > 0.0);
        assert_eq!(dasher_chance(30), 0.4);
    }

    #[test]
    fn a_dasher_waits_dashes_and_waits_again() {
        let mut world = World::new();
        let e = world.spawn((
            Transform2D::from_position(Vec2::new(-100.0, 0.0)),
            Enemy::new(Kind::Dasher),
            Velocity(Vec2::ZERO),
        ));
        let dt = 0.1;
        let mut speeds = Vec::new();
        for _ in 0..30 {
            steer(&mut world, Vec2::ZERO, 1, dt);
            speeds.push(world.get::<&Velocity>(e).expect("velocity").0.length());
        }
        // ~1 s still, ~0.6 s at dash speed, then still again.
        assert!(speeds[..9].iter().all(|&s| s == 0.0), "{speeds:?}");
        assert!(
            speeds[10..15].iter().all(|&s| (s - 430.0).abs() < 1e-3),
            "{speeds:?}"
        );
        assert!(speeds[17..25].iter().all(|&s| s == 0.0), "{speeds:?}");
    }
}
