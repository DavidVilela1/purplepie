//! The entity inspector's text (ADR-043, PP-037d1): which entities exist, a
//! one-line summary of each, and the engine components of one entity. Plain
//! functions over the world, without egui, so they are unit-tested on a
//! headless world.
//!
//! The engine's own components are formatted here. A game's components are
//! counted; the ones registered for scenes arrive as RON from the scene
//! registry (the runner asks it, so `debug` never depends on `scene`) and are
//! turned into rows by [`game_rows`].

use std::any::TypeId;

use crate::ecs::{Entity, Velocity, World};
use crate::math::{Transform2D, Vec2};
use crate::render::{Hidden, Layer, Quad, ScreenSpace, Sprite, SpriteAnimation, Text};
use crate::ui::Button;

/// The longest piece of a `Text`'s content shown in a summary.
const SUMMARY_TEXT_CHARS: usize = 24;

/// Every entity in the world, by id, so the list keeps its order from frame
/// to frame (hecs iterates by archetype, which changes as components change).
pub(crate) fn entity_ids(world: &World) -> Vec<Entity> {
    let mut ids: Vec<Entity> = world.iter().map(|e| e.entity()).collect();
    ids.sort_unstable_by_key(|e| e.id());
    ids
}

/// One line for the entity list: id, what it draws (text, sprite, quad or
/// button) and where.
pub(crate) fn summary(world: &World, entity: Entity) -> String {
    let Ok(e) = world.entity(entity) else {
        return format!("#{} (despawned)", entity.id());
    };
    let kind = if let Some(text) = e.get::<&Text>() {
        format!("text \"{}\"", shorten(&text.content))
    } else if e.has::<Sprite>() {
        "sprite".to_owned()
    } else if e.has::<Quad>() {
        "quad".to_owned()
    } else if e.has::<Button>() {
        "button".to_owned()
    } else {
        match e.len() {
            1 => "1 component".to_owned(),
            n => format!("{n} components"),
        }
    };
    match e.get::<&Transform2D>() {
        Some(t) => format!("#{} {kind} at {}", entity.id(), point(t.position)),
        None => format!("#{} {kind}", entity.id()),
    }
}

/// What the inspector shows of one entity besides its game components.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Inspected {
    /// The engine components as (name, value) rows, in a fixed order.
    pub(crate) engine: Vec<(&'static str, String)>,
    /// How many of the entity's components the engine does not know (the
    /// game's own, registered for scenes or not).
    pub(crate) other_components: usize,
}

/// The selected entity's engine components and a count of the rest; `None`
/// when the entity no longer exists.
pub(crate) fn inspect_entity(world: &World, entity: Entity) -> Option<Inspected> {
    let e = world.entity(entity).ok()?;
    let mut rows = Vec::new();
    if let Some(t) = e.get::<&Transform2D>() {
        rows.push((
            "Transform2D",
            format!(
                "position {}, rotation {:.1}°, scale {}",
                point(t.position),
                t.rotation.to_degrees(),
                point(t.scale)
            ),
        ));
    }
    if let Some(layer) = e.get::<&Layer>() {
        rows.push(("Layer", layer.0.to_string()));
    }
    if e.has::<Hidden>() {
        rows.push(("Hidden", "yes".to_owned()));
    }
    if let Some(screen) = e.get::<&ScreenSpace>() {
        rows.push(("ScreenSpace", format!("{:?}", *screen)));
    }
    if let Some(velocity) = e.get::<&Velocity>() {
        rows.push(("Velocity", format!("{} per second", point(velocity.0))));
    }
    if let Some(quad) = e.get::<&Quad>() {
        rows.push(("Quad", format!("{:?}", *quad)));
    }
    if let Some(sprite) = e.get::<&Sprite>() {
        rows.push(("Sprite", format!("{:?}", *sprite)));
    }
    if let Some(text) = e.get::<&Text>() {
        rows.push(("Text", format!("{:?}", *text)));
    }
    if let Some(animation) = e.get::<&SpriteAnimation>() {
        rows.push(("SpriteAnimation", format!("{:?}", *animation)));
    }
    if let Some(button) = e.get::<&Button>() {
        rows.push(("Button", format!("{:?}", *button)));
    }
    let engine = engine_types();
    Some(Inspected {
        engine: rows,
        other_components: e.component_types().filter(|t| !engine.contains(t)).count(),
    })
}

/// The selected entity's game components as (name, value) rows: each one
/// registered for scenes under its registered name with its one-line RON (or
/// why it could not be written), then a note for those that are not
/// registered. `registered` comes from the scene registry; `other_components`
/// from [`inspect_entity`].
pub(crate) fn game_rows(
    registered: &[(String, Result<String, String>)],
    other_components: usize,
) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> = registered
        .iter()
        .map(|(name, value)| match value {
            Ok(ron) => (name.clone(), ron.clone()),
            Err(error) => (name.clone(), format!("could not be written: {error}")),
        })
        .collect();
    let unregistered = other_components.saturating_sub(registered.len());
    if unregistered > 0 {
        let what = if unregistered == 1 {
            "component"
        } else {
            "components"
        };
        rows.push((
            "Not registered".to_owned(),
            format!("{unregistered} {what}: register them for scenes to see them here"),
        ));
    }
    rows
}

/// The engine components [`inspect_entity`] shows, and
/// [`UnknownComponents`](crate::scene::UnknownComponents), whose entries the
/// runner shows with the game components (`Registry::describe`).
fn engine_types() -> [TypeId; 11] {
    [
        TypeId::of::<crate::scene::UnknownComponents>(),
        TypeId::of::<Transform2D>(),
        TypeId::of::<Layer>(),
        TypeId::of::<Hidden>(),
        TypeId::of::<ScreenSpace>(),
        TypeId::of::<Velocity>(),
        TypeId::of::<Quad>(),
        TypeId::of::<Sprite>(),
        TypeId::of::<Text>(),
        TypeId::of::<SpriteAnimation>(),
        TypeId::of::<Button>(),
    ]
}

fn point(p: Vec2) -> String {
    format!("({:.1}, {:.1})", p.x, p.y)
}

/// The first line of `text`, cut to [`SUMMARY_TEXT_CHARS`] characters.
fn shorten(text: &str) -> String {
    let line = text.lines().next().unwrap_or("");
    if line.chars().count() > SUMMARY_TEXT_CHARS || line.len() < text.trim_end().len() {
        let cut: String = line.chars().take(SUMMARY_TEXT_CHARS).collect();
        format!("{cut}…")
    } else {
        line.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{Color, FontId};

    /// A game component the engine does not know.
    struct Health(#[allow(dead_code)] u32);

    /// A real font id (the bundled font), since `FontId` has no constructor.
    fn font() -> FontId {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/fonts/Poppins-Regular.ttf");
        crate::render::Fonts::default()
            .load(&path)
            .expect("the bundled font loads")
    }

    #[test]
    fn entities_are_listed_by_id_whatever_their_archetype() {
        let mut world = World::new();
        let a = world.spawn((Transform2D::default(),));
        let b = world.spawn((Health(3),));
        let c = world.spawn((Transform2D::default(), Layer(2)));
        // Moving `a` to another archetype must not move it in the list.
        world.insert_one(a, Hidden).expect("alive");
        assert_eq!(entity_ids(&world), [a, b, c]);
        world.despawn(b).expect("alive");
        assert_eq!(entity_ids(&world), [a, c]);
    }

    #[test]
    fn summaries_name_what_an_entity_draws_and_where() {
        let mut world = World::new();
        let font = font();
        let label = world.spawn((
            Transform2D::from_position(Vec2::new(12.0, -8.0)),
            Text::new("Score 10", font, 20.0),
        ));
        let long = world.spawn((Text::new(
            "A rather long piece of text\nline two",
            font,
            20.0,
        ),));
        let quad = world.spawn((
            Transform2D::from_position(Vec2::new(1.0, 2.5)),
            Quad::new(Vec2::splat(4.0), Color::WHITE),
        ));
        let bare = world.spawn((Health(1), Velocity(Vec2::ZERO)));
        let single = world.spawn((Health(2),));

        assert_eq!(
            summary(&world, label),
            format!("#{} text \"Score 10\" at (12.0, -8.0)", label.id())
        );
        assert_eq!(
            summary(&world, long),
            format!("#{} text \"A rather long piece of t…\"", long.id())
        );
        assert_eq!(
            summary(&world, quad),
            format!("#{} quad at (1.0, 2.5)", quad.id())
        );
        assert_eq!(
            summary(&world, bare),
            format!("#{} 2 components", bare.id())
        );
        assert_eq!(
            summary(&world, single),
            format!("#{} 1 component", single.id())
        );
        world.despawn(bare).expect("alive");
        assert_eq!(summary(&world, bare), format!("#{} (despawned)", bare.id()));
    }

    #[test]
    fn inspect_entity_shows_engine_components_and_counts_the_rest() {
        let mut world = World::new();
        let e = world.spawn((
            Transform2D {
                position: Vec2::new(3.0, 4.0),
                rotation: std::f32::consts::FRAC_PI_2,
                scale: Vec2::new(2.0, 1.0),
            },
            Layer(-5),
            Hidden,
            Velocity(Vec2::new(10.0, 0.0)),
            Quad::new(Vec2::splat(1.0), Color::BLACK),
            Health(9),
        ));
        let inspected = inspect_entity(&world, e).expect("alive");
        let rows = &inspected.engine;
        let names: Vec<&str> = rows.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            names,
            ["Transform2D", "Layer", "Hidden", "Velocity", "Quad"]
        );
        assert_eq!(
            rows[0].1,
            "position (3.0, 4.0), rotation 90.0°, scale (2.0, 1.0)"
        );
        assert_eq!(rows[1].1, "-5");
        assert_eq!(rows[2].1, "yes");
        assert_eq!(rows[3].1, "(10.0, 0.0) per second");
        assert!(rows[4].1.starts_with("Quad {"), "{}", rows[4].1);
        assert_eq!(inspected.other_components, 1, "Health is the game's");

        let only_engine = world.spawn((Transform2D::default(),));
        let inspected = inspect_entity(&world, only_engine).expect("alive");
        assert_eq!((inspected.engine.len(), inspected.other_components), (1, 0));
        // Components kept as text are shown with the game's, not counted as
        // unregistered.
        let kept = world.spawn((crate::scene::tests_support::kept(&[("a::B", "()")]),));
        let inspected = inspect_entity(&world, kept).expect("alive");
        assert_eq!(inspected.other_components, 0);

        world.despawn(e).expect("alive");
        assert_eq!(inspect_entity(&world, e), None);
    }

    #[test]
    fn game_rows_show_registered_values_errors_and_what_is_not_registered() {
        let registered = vec![
            (
                "demo::Broken".to_owned(),
                Err("cannot write a Broken".to_owned()),
            ),
            ("demo::Spin".to_owned(), Ok("(speed: 1.5)".to_owned())),
        ];
        let rows = game_rows(&registered, 3);
        let rows: Vec<(&str, &str)> = rows.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        assert_eq!(
            rows,
            [
                (
                    "demo::Broken",
                    "could not be written: cannot write a Broken"
                ),
                ("demo::Spin", "(speed: 1.5)"),
                (
                    "Not registered",
                    "1 component: register them for scenes to see them here"
                ),
            ]
        );
        assert!(game_rows(&[], 0).is_empty(), "no game components, no rows");
        assert_eq!(
            game_rows(&[], 2)[0].1,
            "2 components: register them for scenes to see them here"
        );
    }
}
