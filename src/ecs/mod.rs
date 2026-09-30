//! Entity-component-system support (ADR-006), built on `hecs`.
//!
//! The engine owns one [`World`] and lends it to the game through
//! [`Context::world_mut`](crate::Context::world_mut). Systems are plain
//! functions that the game calls explicitly, usually from `fixed_update`.
//! The engine never runs gameplay systems on its own.
//!
//! This module depends only on `hecs` and [`crate::math`]. It must never
//! depend on windowing or rendering code (ADR-003).
//!
//! ```
//! use purplepie::ecs::{self, Velocity, World};
//! use purplepie::math::{Transform2D, Vec2};
//!
//! let mut world = World::new();
//! let ball = world.spawn((Transform2D::default(), Velocity(Vec2::new(2.0, 0.0))));
//!
//! ecs::integrate_velocity(&mut world, 0.5);
//!
//! let t = world.get::<&Transform2D>(ball).expect("ball exists");
//! assert_eq!(t.position, Vec2::new(1.0, 0.0));
//! ```

pub use hecs::{Entity, World};

/// The full `hecs` API, for queries and tools beyond the re-exported basics
/// (e.g. `hecs::CommandBuffer` for spawning or despawning while iterating).
pub use hecs;

use crate::math::{Transform2D, Vec2};

/// Linear velocity in world units per second.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity(pub Vec2);

/// Moves every entity that has both a [`Transform2D`] and a [`Velocity`] by
/// `velocity * dt`.
///
/// Call it from `fixed_update` with `ctx.dt()` for frame-rate-independent motion:
///
/// ```
/// use purplepie::{ecs, Context, Game};
///
/// struct MyGame;
///
/// impl Game for MyGame {
///     fn fixed_update(&mut self, ctx: &mut Context<'_>) {
///         let dt = ctx.dt(); // read first: `world_mut` borrows the context exclusively
///         ecs::integrate_velocity(ctx.world_mut(), dt);
///     }
/// }
/// ```
pub fn integrate_velocity(world: &mut World, dt: f32) {
    for (transform, velocity) in world.query_mut::<(&mut Transform2D, &Velocity)>() {
        transform.position += velocity.0 * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_attach_and_query_components() {
        let mut world = World::new();
        let a = world.spawn((Transform2D::default(),));
        world
            .insert_one(a, Velocity(Vec2::X))
            .expect("entity a exists");
        let _b = world.spawn((Transform2D::default(),)); // no velocity

        let with_velocity = world
            .query_mut::<(&Transform2D, &Velocity)>()
            .into_iter()
            .count();
        let with_transform = world.query_mut::<&Transform2D>().into_iter().count();
        assert_eq!((with_velocity, with_transform), (1, 2));
    }

    #[test]
    fn integrate_velocity_moves_by_velocity_times_dt() {
        let mut world = World::new();
        let e = world.spawn((
            Transform2D::from_position(Vec2::new(1.0, 1.0)),
            Velocity(Vec2::new(4.0, -2.0)),
        ));
        integrate_velocity(&mut world, 0.25);
        integrate_velocity(&mut world, 0.25);
        let t = world.get::<&Transform2D>(e).expect("entity exists");
        assert_eq!(t.position, Vec2::new(3.0, 0.0));
    }

    #[test]
    fn integrate_velocity_ignores_entities_without_both_components() {
        let mut world = World::new();
        let still = world.spawn((Transform2D::from_position(Vec2::ONE),));
        let orphan_velocity = world.spawn((Velocity(Vec2::X),));
        integrate_velocity(&mut world, 1.0);
        let t = world.get::<&Transform2D>(still).expect("entity exists");
        assert_eq!(t.position, Vec2::ONE);
        assert!(world.contains(orphan_velocity));
    }

    #[test]
    fn rotation_and_scale_are_untouched() {
        let mut world = World::new();
        let start = Transform2D::default()
            .with_rotation(0.75)
            .with_scale(Vec2::splat(2.0));
        let e = world.spawn((start, Velocity(Vec2::Y)));
        integrate_velocity(&mut world, 1.0);
        let t = world.get::<&Transform2D>(e).expect("entity exists");
        assert_eq!((t.rotation, t.scale), (0.75, Vec2::splat(2.0)));
    }
}
