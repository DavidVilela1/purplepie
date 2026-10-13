//! Picking (ADR-044, PP-038d): which entities are drawn at a world point,
//! topmost first, in the same order the draw list uses (ADR-021).

use crate::ecs::{Entity, World};
use crate::math::{Transform2D, Vec2};

use super::draw::Material;
use super::font::Fonts;
use super::texture::Textures;
use super::{Hidden, Layer, Quad, ScreenSpace, Sprite, SpriteAnimation, Text};

/// Every world-space entity whose drawn area contains `point` (world units),
/// topmost first: higher [`Layer`] first; within a layer text before
/// sprites before quads, then later entity ids first, exactly the reverse of
/// the draw order. An entity is listed once, by its topmost drawable.
///
/// The drawn area is the quad's or sprite's rectangle, or the text's
/// [`TextMetrics::bounds`](super::TextMetrics::bounds), turned and scaled by
/// the entity's [`Transform2D`]. [`Hidden`] and [`ScreenSpace`] entities, and
/// things the draw list skips (a sprite region or animation frame outside its
/// texture, a text in a font that is not loaded), are never picked.
pub(crate) fn entities_at(
    world: &World,
    textures: &Textures,
    fonts: &Fonts,
    point: Vec2,
) -> Vec<Entity> {
    // (layer, material rank, entity id) of each hit, as in the draw list.
    let mut hits: Vec<((i32, u32, u32), Entity)> = Vec::new();
    let mut hit = |entity: Entity, layer: Option<&Layer>, material: Material| {
        let layer = layer.copied().unwrap_or_default().0;
        hits.push(((layer, material.rank(), entity.id()), entity));
    };

    for (entity, transform, quad, layer, screen) in world
        .query::<hecs::Without<
            (
                Entity,
                &Transform2D,
                &Quad,
                Option<&Layer>,
                Option<&ScreenSpace>,
            ),
            &Hidden,
        >>()
        .iter()
    {
        if screen.is_some() {
            continue;
        }
        let half = quad.size.abs() * 0.5;
        if contains(transform, -half, half, point) {
            hit(entity, layer, Material::Color);
        }
    }
    for (entity, transform, sprite, animation, layer, screen) in world
        .query::<hecs::Without<
            (
                Entity,
                &Transform2D,
                &Sprite,
                Option<&SpriteAnimation>,
                Option<&Layer>,
                Option<&ScreenSpace>,
            ),
            &Hidden,
        >>()
        .iter()
    {
        if screen.is_some() {
            continue;
        }
        // Skipped by the draw list: an animation frame outside its grid, or
        // a region with nothing left inside the texture.
        let region = match animation {
            Some(animation) => match animation.region() {
                Some(region) => Some(region),
                None => continue,
            },
            None => sprite.region,
        };
        if let Some(region) = region
            && textures
                .get(sprite.texture)
                .and_then(|t| region.uv_rect(t.width, t.height))
                .is_none()
        {
            continue;
        }
        let half = sprite.size.abs() * 0.5;
        if contains(transform, -half, half, point) {
            hit(entity, layer, Material::Texture(sprite.texture));
        }
    }
    for (entity, transform, text, layer, screen) in world
        .query::<hecs::Without<
            (
                Entity,
                &Transform2D,
                &Text,
                Option<&Layer>,
                Option<&ScreenSpace>,
            ),
            &Hidden,
        >>()
        .iter()
    {
        if screen.is_some() {
            continue;
        }
        let Some(font) = fonts.get(text.font) else {
            continue;
        };
        let (min, max) = super::text::measure(&text.content, font, text.size).bounds(text.anchor);
        if contains(transform, min, max, point) {
            hit(entity, layer, Material::Glyphs);
        }
    }

    // Topmost first; keys are unique per (entity, material), so the first
    // occurrence of an entity is its topmost drawable.
    hits.sort_unstable_by_key(|&(key, _)| std::cmp::Reverse(key));
    let mut entities: Vec<Entity> = Vec::with_capacity(hits.len());
    for (_, entity) in hits {
        if !entities.contains(&entity) {
            entities.push(entity);
        }
    }
    entities
}

/// Whether `point` lies in the rectangle `min..=max` of `transform`'s local
/// space. An empty rectangle or a transform that flattens it (zero scale)
/// contains nothing.
fn contains(transform: &Transform2D, min: Vec2, max: Vec2, point: Vec2) -> bool {
    if !(max.x > min.x && max.y > min.y) {
        return false;
    }
    let matrix = transform.to_mat4();
    let determinant = matrix.determinant();
    if !determinant.is_finite() || determinant == 0.0 {
        return false;
    }
    let local = matrix.inverse().transform_point3(point.extend(0.0));
    let local = Vec2::new(local.x, local.y);
    // A little slack, so a point on an edge counts despite rounding.
    let slack = 1e-4 * (max - min).max_element();
    local.x >= min.x - slack
        && local.x <= max.x + slack
        && local.y >= min.y - slack
        && local.y <= max.y + slack
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_4;

    use super::*;
    use crate::render::atlas::GlyphAtlas;
    use crate::render::draw::{DrawList, View};
    use crate::render::texture::tests::texture_ids;
    use crate::render::{Color, SpriteGrid, TextAnchor, TextureRegion};
    use glam::Mat4;

    fn at(world: &World, point: Vec2) -> Vec<Entity> {
        entities_at(world, &Textures::default(), &Fonts::default(), point)
    }

    fn quad(color: u32, size: f32) -> Quad {
        Quad::new(Vec2::splat(size), Color::hex(color))
    }

    #[test]
    fn overlapping_entities_come_topmost_first_in_draw_order() {
        let [texture] = texture_ids();
        let mut world = World::new();
        // Distinct colours, so the draw list's order can be read back.
        let low = world.spawn((Transform2D::default(), quad(0x000001, 10.0), Layer(-1)));
        let sprite = world.spawn((
            Transform2D::default(),
            Sprite::new(texture, Vec2::splat(10.0)).with_tint(Color::hex(0x000002)),
        ));
        let quad_a = world.spawn((Transform2D::default(), quad(0x000003, 10.0)));
        let quad_b = world.spawn((Transform2D::default(), quad(0x000004, 10.0)));
        let high = world.spawn((Transform2D::default(), quad(0x000005, 10.0), Layer(3)));
        let elsewhere = world.spawn((
            Transform2D::from_position(Vec2::new(100.0, 0.0)),
            quad(0x000006, 10.0),
        ));

        let picked = at(&world, Vec2::new(1.0, -2.0));
        assert_eq!(picked, [high, sprite, quad_b, quad_a, low]);
        assert_eq!(at(&world, Vec2::new(100.0, 4.0)), [elsewhere]);
        assert!(at(&world, Vec2::new(50.0, 0.0)).is_empty());

        // The renderer draws the same entities in exactly the reverse order.
        let mut list = DrawList::default();
        list.build(
            &world,
            &View::flat(Mat4::IDENTITY),
            &Textures::default(),
            &Fonts::default(),
            &mut GlyphAtlas::new(16),
        );
        let drawn: Vec<u32> = list
            .instances()
            .iter()
            .map(|i| (i.color[2] * 255.0).round() as u32)
            .filter(|&c| c != 6)
            .collect();
        let expected: Vec<u32> = [high, sprite, quad_b, quad_a, low]
            .iter()
            .rev()
            .map(|&e| match e {
                e if e == low => 1,
                e if e == sprite => 2,
                e if e == quad_a => 3,
                e if e == quad_b => 4,
                _ => 5,
            })
            .collect();
        assert_eq!(drawn, expected, "bottom to top");
    }

    #[test]
    fn hidden_screen_space_and_transformless_entities_are_never_picked() {
        let mut world = World::new();
        world.spawn((Transform2D::default(), quad(1, 10.0), Hidden));
        world.spawn((Transform2D::default(), quad(2, 10.0), ScreenSpace::CENTER));
        world.spawn((quad(3, 10.0),)); // not drawn without a transform
        let flat = world.spawn((
            Transform2D::default().with_scale(Vec2::new(0.0, 1.0)),
            quad(4, 10.0),
        ));
        assert!(at(&world, Vec2::ZERO).is_empty());
        world
            .remove_one::<Hidden>(world.iter().next().map(|e| e.entity()).expect("first"))
            .expect("hidden");
        assert_eq!(at(&world, Vec2::ZERO).len(), 1, "shown again: picked");
        assert!(
            !at(&world, Vec2::ZERO).contains(&flat),
            "zero scale draws nothing"
        );
    }

    #[test]
    fn rotated_scaled_and_mirrored_shapes_pick_by_their_drawn_area() {
        let mut world = World::new();
        // A 10 × 2 bar turned 45°: its far end is near (3.5, 3.5).
        let bar = world.spawn((
            Transform2D::default().with_rotation(FRAC_PI_4),
            Quad::new(Vec2::new(10.0, 2.0), Color::WHITE),
        ));
        assert_eq!(at(&world, Vec2::new(3.4, 3.4)), [bar]);
        assert!(
            at(&world, Vec2::new(4.0, 0.0)).is_empty(),
            "outside the turned bar"
        );

        // Scaled ×3 and mirrored on X: 30 × 3 world units around (50, 0).
        let mirrored = world.spawn((
            Transform2D::from_position(Vec2::new(50.0, 0.0)).with_scale(Vec2::new(-3.0, 3.0)),
            Quad::new(Vec2::new(10.0, 1.0), Color::WHITE),
        ));
        assert_eq!(at(&world, Vec2::new(64.0, 1.4)), [mirrored]);
        assert_eq!(at(&world, Vec2::new(36.0, -1.4)), [mirrored]);
        assert!(at(&world, Vec2::new(66.0, 0.0)).is_empty());
        assert!(at(&world, Vec2::new(50.0, 1.6)).is_empty());
    }

    #[test]
    fn sprites_the_draw_list_skips_are_not_picked() {
        let mut textures = Textures::default();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/textures/sandbox_sheet.png"); // 32 × 16
        let sheet = textures
            .load(&path, crate::render::TextureOptions::NEAREST)
            .expect("sheet");
        let mut world = World::new();
        let inside = world.spawn((
            Transform2D::default(),
            Sprite::new(sheet, Vec2::splat(8.0)).with_region(TextureRegion::new(8, 0, 8, 8)),
        ));
        // A region entirely outside the 32 × 16 texture: not drawn.
        world.spawn((
            Transform2D::default(),
            Sprite::new(sheet, Vec2::splat(8.0)).with_region(TextureRegion::new(64, 64, 8, 8)),
            Layer(5),
        ));
        // An animation whose frame is outside its grid: not drawn either.
        world.spawn((
            Transform2D::default(),
            Sprite::new(sheet, Vec2::splat(8.0)),
            SpriteAnimation::new(SpriteGrid::new(8, 8, 4, 2), 9, 9, 10.0),
            Layer(6),
        ));
        let picked = entities_at(&world, &textures, &Fonts::default(), Vec2::ZERO);
        assert_eq!(picked, [inside]);
    }

    #[test]
    fn texts_pick_by_their_measured_bounds_around_the_anchor() {
        let mut fonts = Fonts::default();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/fonts/Poppins-Regular.ttf");
        let font = fonts.load(&path).expect("font");
        let mut world = World::new();
        let text = Text::new("Hello", font, 20.0).with_anchor(TextAnchor::CENTER);
        let metrics = super::super::text::measure("Hello", fonts.get(font).expect("loaded"), 20.0);
        let label = world.spawn((
            Transform2D::from_position(Vec2::new(10.0, 10.0)),
            text.clone(),
        ));
        let pick = |p: Vec2| entities_at(&world, &Textures::default(), &fonts, p);
        let half = Vec2::new(metrics.width, metrics.height) * 0.5;
        assert_eq!(pick(Vec2::new(10.0, 10.0)), [label]);
        assert_eq!(pick(Vec2::new(10.0, 10.0) + half * 0.9), [label]);
        assert!(pick(Vec2::new(10.0, 10.0) + half * 1.1).is_empty());

        // Doubled in size, the same text covers twice the distance.
        let mut world2 = World::new();
        let big = world2.spawn((
            Transform2D::from_position(Vec2::new(10.0, 10.0)).with_scale(Vec2::splat(2.0)),
            text,
        ));
        let pick2 = |p: Vec2| entities_at(&world2, &Textures::default(), &fonts, p);
        assert_eq!(pick2(Vec2::new(10.0, 10.0) + half * 1.8), [big]);
        // A font from another engine run (not loaded here): never picked.
        assert!(at(&world2, Vec2::new(10.0, 10.0)).is_empty());
    }

    #[test]
    fn an_entity_drawn_twice_is_listed_once_by_its_top_drawable() {
        let [texture] = texture_ids();
        let mut world = World::new();
        let both = world.spawn((
            Transform2D::default(),
            quad(1, 10.0),
            Sprite::new(texture, Vec2::splat(4.0)),
        ));
        let between = world.spawn((Transform2D::default(), quad(2, 10.0)));
        // At the centre the sprite (drawn after quads) is on top of `between`;
        // at the edge only the two quads are.
        assert_eq!(at(&world, Vec2::ZERO), [both, between]);
        assert_eq!(at(&world, Vec2::new(4.0, 4.0)), [between, both]);
    }
}
