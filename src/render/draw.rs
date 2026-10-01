//! Draw order and batching (ADR-021).
//!
//! Every frame, all drawables (quads and sprites) are collected into one list,
//! sorted by [`Layer`], then by material (quads, then sprites grouped by
//! texture), then by entity. Consecutive items with the same layer and material
//! become one [`Batch`], which is one draw call.

use std::ops::Range;

use super::instance::Instance;
use super::quad::Quad;
use super::sprite::Sprite;
use super::texture::TextureId;
use crate::ecs::{Entity, World};
use crate::math::{Mat4, Transform2D};

/// Draw order for an entity's [`Quad`] or [`Sprite`]: higher layers are drawn
/// on top of lower ones. Entities without a `Layer` are on layer 0.
///
/// Within one layer, quads are drawn first, then sprites grouped by texture.
/// The order inside such a group is deterministic but not part of the API, so
/// give overlapping drawables different layers when their order matters.
///
/// ```
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::{Color, Layer, Quad};
///
/// let mut world = purplepie::ecs::World::new();
/// // A health bar that always covers the sprites on the default layer.
/// world.spawn((
///     Transform2D::from_position(Vec2::new(0.0, 200.0)),
///     Quad::new(Vec2::new(100.0, 10.0), Color::hex(0xE63946)),
///     Layer(10),
/// ));
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Layer(pub i32);

/// What a batch is drawn with: the solid-colour quad pipeline, or the sprite
/// pipeline with one texture bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Material {
    Color,
    Texture(TextureId),
}

impl Material {
    /// Sort rank within a layer: quads first, then sprites by texture.
    fn rank(self) -> u32 {
        match self {
            Material::Color => 0,
            Material::Texture(id) => {
                u32::try_from(id.index()).map_or(u32::MAX, |i| i.saturating_add(1))
            }
        }
    }
}

/// A run of consecutive instances with the same layer and material: one draw call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Batch {
    pub(crate) layer: i32,
    pub(crate) material: Material,
    pub(crate) instances: Range<u32>,
}

/// Total order of a drawable: layer, then material, then entity index (the
/// tie-breaker that makes the order independent of ECS query order).
type SortKey = (i32, u32, u32);

struct Item {
    key: SortKey,
    material: Material,
    instance: Instance,
}

/// This frame's sorted instances and their batches. Reused every frame, so
/// steady-state frames do not allocate.
#[derive(Default)]
pub(crate) struct DrawList {
    items: Vec<Item>,
    instances: Vec<Instance>,
    batches: Vec<Batch>,
}

impl DrawList {
    /// Rebuilds the list from every `(Transform2D, Quad)` and `(Transform2D,
    /// Sprite)` entity in `world` (read-only, ADR-009).
    pub(crate) fn build(&mut self, world: &World, view_projection: &Mat4, target_is_srgb: bool) {
        self.items.clear();
        let mut push = |entity: Entity, layer: Option<&Layer>, material: Material, instance| {
            let layer = layer.copied().unwrap_or_default().0;
            self.items.push(Item {
                key: (layer, material.rank(), entity.id()),
                material,
                instance,
            });
        };
        for (entity, transform, quad, layer) in world
            .query::<(Entity, &Transform2D, &Quad, Option<&Layer>)>()
            .iter()
        {
            let instance = Instance::new(
                view_projection,
                transform,
                quad.size,
                quad.color,
                target_is_srgb,
            );
            push(entity, layer, Material::Color, instance);
        }
        for (entity, transform, sprite, layer) in world
            .query::<(Entity, &Transform2D, &Sprite, Option<&Layer>)>()
            .iter()
        {
            let instance = Instance::new(
                view_projection,
                transform,
                sprite.size,
                sprite.tint,
                target_is_srgb,
            );
            push(entity, layer, Material::Texture(sprite.texture), instance);
        }

        // Keys are unique (they end with the entity index, and an entity has at
        // most one quad and one sprite, which differ in rank), so an unstable
        // sort is still deterministic.
        self.items.sort_unstable_by_key(|item| item.key);

        self.instances.clear();
        self.batches.clear();
        for item in &self.items {
            let index = u32::try_from(self.instances.len()).unwrap_or(u32::MAX);
            self.instances.push(item.instance);
            let layer = item.key.0;
            match self.batches.last_mut() {
                Some(batch) if batch.layer == layer && batch.material == item.material => {
                    batch.instances.end = index + 1;
                }
                _ => self.batches.push(Batch {
                    layer,
                    material: item.material,
                    instances: index..index + 1,
                }),
            }
        }
    }

    /// All instances in draw order.
    pub(crate) fn instances(&self) -> &[Instance] {
        &self.instances
    }

    /// The draw calls, in order.
    pub(crate) fn batches(&self) -> &[Batch] {
        &self.batches
    }
}

#[cfg(test)]
mod tests {
    use super::super::Color;
    use super::super::texture::tests::texture_ids;
    use super::*;
    use crate::math::Vec2;

    fn quad(color: u32) -> Quad {
        Quad::new(Vec2::ONE, Color::hex(color))
    }

    fn built(world: &World) -> DrawList {
        let mut list = DrawList::default();
        list.build(world, &Mat4::IDENTITY, false);
        list
    }

    /// The instance colours in draw order, as 0xRRGGBB (non-sRGB target: exact).
    fn colors(list: &DrawList) -> Vec<u32> {
        list.instances()
            .iter()
            .map(|i| {
                let c = |v: f32| (v * 255.0).round() as u32;
                (c(i.color[0]) << 16) | (c(i.color[1]) << 8) | c(i.color[2])
            })
            .collect()
    }

    #[test]
    fn layers_sort_across_quads_and_sprites() {
        let [tex] = texture_ids();
        let mut world = World::new();
        world.spawn((Transform2D::default(), quad(0x000001), Layer(2)));
        world.spawn((
            Transform2D::default(),
            Sprite::new(tex, Vec2::ONE).with_tint(Color::hex(0x000002)),
            Layer(1),
        ));
        world.spawn((Transform2D::default(), quad(0x000003))); // no Layer: layer 0
        world.spawn((Transform2D::default(), quad(0x000004), Layer(-5)));
        let list = built(&world);
        assert_eq!(colors(&list), [0x000004, 0x000003, 0x000002, 0x000001]);
        let layers: Vec<i32> = list.batches().iter().map(|b| b.layer).collect();
        assert_eq!(layers, [-5, 0, 1, 2]);
    }

    #[test]
    fn within_a_layer_quads_come_before_sprites() {
        let [tex] = texture_ids();
        let mut world = World::new();
        // Spawned sprite first, so query order alone would not explain the result.
        world.spawn((
            Transform2D::default(),
            Sprite::new(tex, Vec2::ONE).with_tint(Color::hex(0x0000AA)),
        ));
        world.spawn((Transform2D::default(), quad(0x0000BB)));
        let list = built(&world);
        assert_eq!(colors(&list), [0x0000BB, 0x0000AA]);
        assert_eq!(list.batches()[0].material, Material::Color);
        assert_eq!(list.batches()[1].material, Material::Texture(tex));
    }

    #[test]
    fn one_draw_call_per_layer_and_material_run() {
        let [a, b] = texture_ids();
        let mut world = World::new();
        // Layer 0: sprites alternating between two textures, plus two quads.
        for (i, texture) in [a, b, a, b, a].into_iter().enumerate() {
            world.spawn((
                Transform2D::from_position(Vec2::new(i as f32, 0.0)),
                Sprite::new(texture, Vec2::ONE),
            ));
        }
        world.spawn((Transform2D::default(), quad(0xFFFFFF)));
        world.spawn((Transform2D::default(), quad(0xFFFFFF)));
        // Layer 1: one more sprite with texture a, so a must split by layer.
        world.spawn((Transform2D::default(), Sprite::new(a, Vec2::ONE), Layer(1)));

        let list = built(&world);
        assert_eq!(list.instances().len(), 8);
        let batches: Vec<(i32, Material, Range<u32>)> = list
            .batches()
            .iter()
            .map(|b| (b.layer, b.material, b.instances.clone()))
            .collect();
        assert_eq!(
            batches,
            [
                (0, Material::Color, 0..2),
                (0, Material::Texture(a), 2..5),
                (0, Material::Texture(b), 5..7),
                (1, Material::Texture(a), 7..8),
            ],
            "draw calls = (layer, material) runs, not one per sprite"
        );
    }

    #[test]
    fn order_within_a_group_follows_entity_index_not_query_order() {
        let mut world = World::new();
        let first = world.spawn((Transform2D::default(), quad(0x000001)));
        let second = world.spawn((Transform2D::default(), quad(0x000002)));
        // Adding a component moves `first` to another archetype, which changes
        // hecs's iteration order. The draw order must not change.
        world.insert_one(first, Layer(0)).expect("entity exists");
        assert!(first.id() < second.id());
        let list = built(&world);
        assert_eq!(colors(&list), [0x000001, 0x000002]);
        assert_eq!(list.batches().len(), 1);
    }

    #[test]
    fn an_entity_with_both_a_quad_and_a_sprite_draws_both() {
        let [tex] = texture_ids();
        let mut world = World::new();
        world.spawn((
            Transform2D::default(),
            quad(0x123456),
            Sprite::new(tex, Vec2::ONE),
        ));
        world.spawn((Transform2D::default(),)); // nothing to draw
        let list = built(&world);
        assert_eq!(list.instances().len(), 2);
        assert_eq!(list.batches().len(), 2);
    }

    #[test]
    fn rebuilding_replaces_the_previous_frame() {
        let mut world = World::new();
        let e = world.spawn((Transform2D::default(), quad(0x000001)));
        let mut list = DrawList::default();
        list.build(&world, &Mat4::IDENTITY, false);
        world.despawn(e).expect("entity exists");
        list.build(&world, &Mat4::IDENTITY, false);
        assert!(list.instances().is_empty());
        assert!(list.batches().is_empty());
    }

    #[test]
    fn instances_carry_the_transform_size_and_converted_colour() {
        let mut world = World::new();
        world.spawn((
            Transform2D::from_position(Vec2::new(50.0, 0.0)),
            Quad::new(Vec2::new(40.0, 20.0), Color::rgba(0.5, 0.5, 0.5, 0.5)),
        ));
        let vp = super::super::quad::view_projection(Vec2::new(200.0, 100.0));
        let mut list = DrawList::default();
        list.build(&world, &vp, true);
        let instance = list.instances()[0];
        let m = Mat4::from_cols_array_2d(&instance.clip_from_local);
        let corner = m * glam::Vec4::new(0.5, 0.5, 0.0, 1.0);
        // Unit-square corner (0.5, 0.5) → world (70, 10) → clip (0.7, 0.2).
        assert!(
            corner
                .truncate()
                .truncate()
                .abs_diff_eq(Vec2::new(0.7, 0.2), 1e-6)
        );
        assert!(
            (instance.color[0] - 0.214_041).abs() < 1e-5,
            "sRGB target: linear colour"
        );
        assert_eq!(instance.color[3], 0.5);
    }
}
