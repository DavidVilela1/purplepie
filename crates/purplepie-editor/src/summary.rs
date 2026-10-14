//! One line per entity for the editor's list.

use purplepie::ecs::{Entity, World};
use purplepie::math::Transform2D;
use purplepie::render::{Quad, ScreenSpace, Sprite, Text};
use purplepie::scene::UnknownComponents;
use purplepie::ui::Button;

/// The longest piece of a text shown in a summary.
const TEXT_CHARS: usize = 20;

/// Every entity, by id (stable while components change).
pub(crate) fn entity_ids(world: &World) -> Vec<Entity> {
    let mut ids: Vec<Entity> = world.iter().map(|e| e.entity()).collect();
    ids.sort_unstable_by_key(|e| e.id());
    ids
}

/// `#id kind at (x, y)`, plus "screen" for screen-space entities and the
/// number of game components kept as text.
pub(crate) fn summary(world: &World, entity: Entity) -> String {
    let Ok(e) = world.entity(entity) else {
        return format!("#{} (gone)", entity.id());
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
        "no drawing".to_owned()
    };
    let mut line = format!("#{} {kind}", entity.id());
    if let Some(t) = e.get::<&Transform2D>() {
        line.push_str(&format!(" at ({:.0}, {:.0})", t.position.x, t.position.y));
    }
    if e.has::<ScreenSpace>() {
        line.push_str(" (screen)");
    }
    if let Some(kept) = e.get::<&UnknownComponents>() {
        match kept.len() {
            0 => {}
            1 => line.push_str(" +1 game component"),
            n => line.push_str(&format!(" +{n} game components")),
        }
    }
    line
}

/// The first line of `text`, cut to [`TEXT_CHARS`] characters.
fn shorten(text: &str) -> String {
    let line = text.lines().next().unwrap_or("");
    if line.chars().count() > TEXT_CHARS || line.len() < text.trim_end().len() {
        let cut: String = line.chars().take(TEXT_CHARS).collect();
        format!("{cut}…")
    } else {
        line.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use purplepie::math::Vec2;
    use purplepie::render::Color;

    #[test]
    fn summaries_name_the_drawable_its_place_and_its_space() {
        let mut world = World::new();
        let quad = world.spawn((
            Transform2D::from_position(Vec2::new(12.4, -3.6)),
            Quad::new(Vec2::ONE, Color::WHITE),
        ));
        let hud = world.spawn((
            Transform2D::default(),
            Quad::new(Vec2::ONE, Color::WHITE),
            ScreenSpace::TOP_LEFT,
        ));
        let bare = world.spawn((7_u32,));
        assert_eq!(
            summary(&world, quad),
            format!("#{} quad at (12, -4)", quad.id())
        );
        assert_eq!(
            summary(&world, hud),
            format!("#{} quad at (0, 0) (screen)", hud.id())
        );
        assert_eq!(summary(&world, bare), format!("#{} no drawing", bare.id()));
        assert_eq!(entity_ids(&world), [quad, hud, bare]);
        world.despawn(bare).expect("alive");
        assert_eq!(summary(&world, bare), format!("#{} (gone)", bare.id()));
        assert_eq!(shorten("a\nb"), "a…");
        assert_eq!(shorten(&"x".repeat(25)), format!("{}…", "x".repeat(20)));
    }
}
