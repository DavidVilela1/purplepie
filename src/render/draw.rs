//! Draw order and batching (ADR-021, ADR-027).
//!
//! Every frame, all drawables (quads, sprites and text glyphs) are collected
//! into one list, sorted by [`Layer`], then by material (quads, then sprites
//! grouped by texture, then text), then by entity. Consecutive items with the
//! same layer and material become one [`Batch`], which is one draw call.

use std::ops::Range;

use super::atlas::GlyphAtlas;
use super::font::Fonts;
use super::instance::Instance;
use super::quad::Quad;
use super::sprite::Sprite;
use super::text::{self, MAX_EM_PIXELS, Text};
use super::texture::TextureId;
use crate::ecs::{Entity, World, hecs::Without};
use crate::math::{Mat4, Transform2D, Vec2};
use glam::Vec3;

/// Draw order for an entity's [`Quad`], [`Sprite`] or [`Text`]: higher layers
/// are drawn on top of lower ones. Entities without a `Layer` are on layer 0.
///
/// Within one layer, quads are drawn first, then sprites grouped by texture,
/// then text.
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

/// Marks an entity that is not drawn, without removing its [`Quad`],
/// [`Sprite`] or [`Text`]: insert it to hide, remove it to show again.
///
/// ```
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::{Color, Hidden, Quad};
///
/// let mut world = purplepie::ecs::World::new();
/// let overlay = world.spawn((Transform2D::default(), Quad::new(Vec2::splat(100.0), Color::BLACK), Hidden));
/// world.remove_one::<Hidden>(overlay).expect("shown again");
/// world.insert_one(overlay, Hidden).expect("hidden again");
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hidden;

/// What a batch is drawn with: the solid-colour quad pipeline, or the sprite
/// pipeline with one texture or the glyph atlas bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Material {
    Color,
    Texture(TextureId),
    Glyphs,
}

impl Material {
    /// Sort rank within a layer: quads first, then sprites by texture, then text.
    fn rank(self) -> u32 {
        match self {
            Material::Color => 0,
            Material::Texture(id) => u32::try_from(id.index())
                .map_or(u32::MAX - 1, |i| i.saturating_add(1).min(u32::MAX - 1)),
            Material::Glyphs => u32::MAX,
        }
    }
}

/// How this frame is viewed: everything the draw list needs besides the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct View {
    /// World → clip transform (ADR-022).
    pub(crate) view_projection: Mat4,
    /// Drawing area in physical pixels, for pixel-exact text.
    pub(crate) physical_size: Vec2,
    /// Colours are converted to linear for sRGB targets (ADR-015).
    pub(crate) target_is_srgb: bool,
}

/// A run of consecutive instances with the same layer and material: one draw call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Batch {
    pub(crate) layer: i32,
    pub(crate) material: Material,
    pub(crate) instances: Range<u32>,
}

/// Total order of a drawable: layer, then material, then entity index (the
/// tie-breaker that makes the order independent of ECS query order), then the
/// glyph's position in its text (0 for quads and sprites).
type SortKey = (i32, u32, u32, u32);

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
    /// Scratch space for text layout (one entry per line).
    line_widths: Vec<f32>,
}

impl DrawList {
    /// Rebuilds the list from every `(Transform2D, Quad)`, `(Transform2D,
    /// Sprite)` and `(Transform2D, Text)` entity in `world` (read-only,
    /// ADR-009). Glyphs missing from `atlas` are rasterized into it; if it
    /// fills up, it is cleared and this frame's text is laid out again.
    pub(crate) fn build(
        &mut self,
        world: &World,
        view: &View,
        fonts: &Fonts,
        atlas: &mut GlyphAtlas,
    ) {
        self.items.clear();
        let view_projection = &view.view_projection;
        let target_is_srgb = view.target_is_srgb;
        for (entity, transform, quad, layer) in world
            .query::<Without<(Entity, &Transform2D, &Quad, Option<&Layer>), &Hidden>>()
            .iter()
        {
            let instance = Instance::new(
                view_projection,
                transform,
                quad.size,
                quad.color,
                target_is_srgb,
            );
            self.push(entity, layer, Material::Color, 0, instance);
        }
        for (entity, transform, sprite, layer) in world
            .query::<Without<(Entity, &Transform2D, &Sprite, Option<&Layer>), &Hidden>>()
            .iter()
        {
            let instance = Instance::new(
                view_projection,
                transform,
                sprite.size,
                sprite.tint,
                target_is_srgb,
            );
            self.push(
                entity,
                layer,
                Material::Texture(sprite.texture),
                0,
                instance,
            );
        }
        let text_start = self.items.len();
        if self.push_text(world, view, fonts, atlas, false).is_err() {
            // The atlas filled up mid-frame: glyphs placed so far may be
            // evicted by a reset, so start this frame's text over.
            self.items.truncate(text_start);
            atlas.reset();
            log::debug!("glyph atlas full; cleared (reset #{})", atlas.resets());
            if atlas.resets() == 1 {
                log::info!(
                    "glyph atlas was full and has been cleared; text may cost more this frame"
                );
            }
            let skipped = self.push_text(world, view, fonts, atlas, true);
            debug_assert!(skipped.is_ok());
        }

        // Keys are unique (entity index, then glyph index; an entity's quad,
        // sprite and text differ in rank), so an unstable sort is still deterministic.
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

    fn push(
        &mut self,
        entity: Entity,
        layer: Option<&Layer>,
        material: Material,
        index: u32,
        instance: Instance,
    ) {
        let layer = layer.copied().unwrap_or_default().0;
        self.items.push(Item {
            key: (layer, material.rank(), entity.id(), index),
            material,
            instance,
        });
    }

    /// Lays out every visible `Text` and pushes one item per glyph.
    /// Returns `Err` if the atlas fills up (only when `skip_when_full` is false).
    fn push_text(
        &mut self,
        world: &World,
        view: &View,
        fonts: &Fonts,
        atlas: &mut GlyphAtlas,
        skip_when_full: bool,
    ) -> Result<(), super::atlas::AtlasFull> {
        let atlas_size = atlas.size() as f32;
        for (entity, transform, text, layer) in world
            .query::<Without<(Entity, &Transform2D, &Text, Option<&Layer>), &Hidden>>()
            .iter()
        {
            let Some(font) = fonts.get(text.font) else {
                continue;
            };
            let Some(placement) = TextPlacement::new(view, transform, text.size) else {
                continue;
            };
            let key_base = (
                layer.copied().unwrap_or_default().0,
                Material::Glyphs.rank(),
                entity.id(),
            );
            let mut index = 0_u32;
            let items = &mut self.items;
            text::layout(
                &text.content,
                text.font,
                font,
                placement.em_px,
                text.anchor,
                atlas,
                skip_when_full,
                &mut self.line_widths,
                |glyph| {
                    let (w, h) = glyph.image.size;
                    let (u, v) = glyph.image.texel;
                    let uv_rect = [
                        u as f32 / atlas_size,
                        v as f32 / atlas_size,
                        w as f32 / atlas_size,
                        h as f32 / atlas_size,
                    ];
                    let clip_from_local = placement.glyph_matrix(glyph.position, (w, h));
                    items.push(Item {
                        key: (key_base.0, key_base.1, key_base.2, index),
                        material: Material::Glyphs,
                        instance: Instance::from_matrix(
                            &clip_from_local,
                            text.color,
                            uv_rect,
                            view.target_is_srgb,
                        ),
                    });
                    index = index.saturating_add(1);
                },
            )?;
        }
        Ok(())
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

/// Where a text's pixel layout goes: the text's local space (its transform)
/// mapped to clip space, nudged so the origin sits on a physical pixel corner.
struct TextPlacement {
    clip_from_text: Mat4,
    /// Physical pixels per local unit (the larger axis if scaled unevenly).
    pixels_per_unit: f32,
    /// Em size in physical pixels: what glyphs are rasterized at.
    em_px: f32,
}

impl TextPlacement {
    /// `None` if the text would be invisible (zero scale, empty viewport) or
    /// too large to rasterize.
    fn new(view: &View, transform: &Transform2D, size: f32) -> Option<Self> {
        let half = view.physical_size * 0.5;
        if !(half.x > 0.0 && half.y > 0.0) {
            return None;
        }
        let clip_from_text = view.view_projection * transform.to_mat4();
        // Length of each local axis in physical pixels.
        let axis_px = |axis: glam::Vec4| Vec2::new(axis.x * half.x, axis.y * half.y).length();
        let pixels_per_unit = axis_px(clip_from_text.x_axis).max(axis_px(clip_from_text.y_axis));
        let em_px = size * pixels_per_unit;
        let usable = |v: f32| v.is_finite() && v > 0.0;
        if !usable(pixels_per_unit) || !usable(em_px) {
            return None;
        }
        if em_px > MAX_EM_PIXELS {
            log::debug!("text at {em_px} px is larger than {MAX_EM_PIXELS} px; not drawn");
            return None;
        }
        // The origin in physical pixels (top-left origin, +Y down), moved to
        // the nearest pixel corner so glyph texels land exactly on pixels.
        let origin = clip_from_text.w_axis;
        let p = Vec2::new((origin.x + 1.0) * half.x, (1.0 - origin.y) * half.y);
        let nudge = p.round() - p;
        let snap = Mat4::from_translation(Vec3::new(nudge.x / half.x, -nudge.y / half.y, 0.0));
        Some(Self {
            clip_from_text: snap * clip_from_text,
            pixels_per_unit,
            em_px,
        })
    }

    /// The unit square → clip transform for a `size` glyph bitmap whose
    /// top-left corner is `position` pixels from the origin (+Y down).
    fn glyph_matrix(&self, position: (i32, i32), size: (u32, u32)) -> Mat4 {
        let k = self.pixels_per_unit;
        let (w, h) = (size.0 as f32, size.1 as f32);
        let centre = Vec3::new(
            (position.0 as f32 + w * 0.5) / k,
            -(position.1 as f32 + h * 0.5) / k,
            0.0,
        );
        self.clip_from_text
            * Mat4::from_translation(centre)
            * Mat4::from_scale(Vec3::new(w / k, h / k, 1.0))
    }
}

#[cfg(test)]
impl View {
    /// A non-sRGB 200×100 px view through `view_projection`.
    pub(crate) fn flat(view_projection: Mat4) -> Self {
        Self {
            view_projection,
            physical_size: Vec2::new(200.0, 100.0),
            target_is_srgb: false,
        }
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
        list.build(
            world,
            &View::flat(Mat4::IDENTITY),
            &Fonts::default(),
            &mut GlyphAtlas::new(16),
        );
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
    fn hidden_entities_are_not_drawn() {
        let [tex] = texture_ids();
        let mut world = World::new();
        world.spawn((Transform2D::default(), quad(0x000001), Hidden));
        world.spawn((Transform2D::default(), Sprite::new(tex, Vec2::ONE), Hidden));
        let shown = world.spawn((Transform2D::default(), quad(0x000002)));
        let list = built(&world);
        assert_eq!(colors(&list), [0x000002]);
        world.insert_one(shown, Hidden).expect("entity exists");
        assert!(built(&world).instances().is_empty());
    }

    #[test]
    fn rebuilding_replaces_the_previous_frame() {
        let mut world = World::new();
        let e = world.spawn((Transform2D::default(), quad(0x000001)));
        let mut list = DrawList::default();
        let (fonts, mut atlas) = (Fonts::default(), GlyphAtlas::new(16));
        list.build(&world, &View::flat(Mat4::IDENTITY), &fonts, &mut atlas);
        world.despawn(e).expect("entity exists");
        list.build(&world, &View::flat(Mat4::IDENTITY), &fonts, &mut atlas);
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
        let vp = super::super::Camera2D::default().view_projection(Vec2::new(200.0, 100.0));
        let mut list = DrawList::default();
        let view = View {
            target_is_srgb: true,
            ..View::flat(vp)
        };
        list.build(&world, &view, &Fonts::default(), &mut GlyphAtlas::new(16));
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

    /// A 256×128 px view (1 logical = 1 physical pixel) through `camera`.
    fn text_view(camera: super::super::Camera2D) -> View {
        let size = Vec2::new(256.0, 128.0);
        View {
            view_projection: camera.view_projection(size),
            physical_size: size,
            target_is_srgb: false,
        }
    }

    fn built_with_text(
        world: &World,
        view: &View,
        fonts: &Fonts,
        atlas: &mut GlyphAtlas,
    ) -> DrawList {
        let mut list = DrawList::default();
        list.build(world, view, fonts, atlas);
        list
    }

    /// The physical-pixel rectangle (left, top, right, bottom) an instance covers.
    fn pixel_rect(instance: &Instance, view: &View) -> [f32; 4] {
        let m = Mat4::from_cols_array_2d(&instance.clip_from_local);
        let to_px = |x: f32, y: f32| {
            let c = m * glam::Vec4::new(x, y, 0.0, 1.0);
            Vec2::new(
                (c.x + 1.0) * 0.5 * view.physical_size.x,
                (1.0 - c.y) * 0.5 * view.physical_size.y,
            )
        };
        let (a, b) = (to_px(-0.5, 0.5), to_px(0.5, -0.5));
        [a.x, a.y, b.x, b.y]
    }

    #[test]
    fn text_is_drawn_after_quads_and_sprites_in_one_batch_per_layer() {
        let [tex] = texture_ids();
        let (fonts, font) = super::super::font::tests::poppins();
        let mut atlas = GlyphAtlas::new(256);
        let mut world = World::new();
        world.spawn((Transform2D::default(), Text::new("Hi", font, 16.0)));
        world.spawn((Transform2D::default(), Text::new("yo", font, 16.0)));
        world.spawn((Transform2D::default(), Sprite::new(tex, Vec2::ONE)));
        world.spawn((Transform2D::default(), quad(0x000001)));
        world.spawn((
            Transform2D::default(),
            Text::new("top", font, 16.0),
            Layer(1),
        ));
        world.spawn((
            Transform2D::default(),
            Text::new("gone", font, 16.0),
            Hidden,
        ));
        let view = text_view(super::super::Camera2D::default());
        let list = built_with_text(&world, &view, &fonts, &mut atlas);
        let batches: Vec<(i32, Material, u32)> = list
            .batches()
            .iter()
            .map(|b| (b.layer, b.material, b.instances.end - b.instances.start))
            .collect();
        assert_eq!(
            batches,
            [
                (0, Material::Color, 1),
                (0, Material::Texture(tex), 1),
                (0, Material::Glyphs, 4),
                (1, Material::Glyphs, 3),
            ]
        );
    }

    #[test]
    fn glyph_quads_land_on_whole_pixels_at_their_bitmap_size() {
        let (fonts, font) = super::super::font::tests::poppins();
        let mut world = World::new();
        // A fractional position: the origin is snapped to the nearest pixel corner.
        world.spawn((
            Transform2D::from_position(Vec2::new(-100.3, 10.6)),
            Text::new("Ag", font, 20.0),
        ));
        for zoom in [1.0, 2.0, 0.75] {
            let camera = super::super::Camera2D::new(Vec2::new(3.25, -1.5), zoom);
            let view = text_view(camera);
            let mut atlas = GlyphAtlas::new(256);
            let list = built_with_text(&world, &view, &fonts, &mut atlas);
            assert_eq!(list.instances().len(), 2);
            // The same glyphs laid out directly at the on-screen size.
            let mut expected = Vec::new();
            text::layout(
                "Ag",
                font,
                fonts.get(font).expect("font"),
                20.0 * zoom,
                crate::render::TextAnchor::BASELINE_LEFT,
                &mut atlas,
                false,
                &mut Vec::new(),
                |g| expected.push(g),
            )
            .expect("fits");
            let origin = camera
                .world_to_screen(Vec2::new(-100.3, 10.6), view.physical_size)
                .round();
            for (instance, glyph) in list.instances().iter().zip(&expected) {
                let [l, t, r, b] = pixel_rect(instance, &view);
                let (w, h) = glyph.image.size;
                let want = [
                    origin.x + glyph.position.0 as f32,
                    origin.y + glyph.position.1 as f32,
                    origin.x + (glyph.position.0 + w as i32) as f32,
                    origin.y + (glyph.position.1 + h as i32) as f32,
                ];
                for (got, want) in [l, t, r, b].into_iter().zip(want) {
                    assert!((got - want).abs() < 1e-3, "zoom {zoom}: {got} vs {want}");
                }
                let s = atlas.size() as f32;
                assert_eq!(
                    instance.uv_rect,
                    [
                        glyph.image.texel.0 as f32 / s,
                        glyph.image.texel.1 as f32 / s,
                        w as f32 / s,
                        h as f32 / s
                    ]
                );
            }
        }
    }

    #[test]
    fn text_is_rasterized_for_physical_pixels_at_any_dpi() {
        let (fonts, font) = super::super::font::tests::poppins();
        let mut world = World::new();
        world.spawn((
            Transform2D::from_position(Vec2::new(-50.4, 7.3)),
            Text::new("Hx", font, 16.0),
        ));
        for scale in [1.0_f32, 1.25, 2.0] {
            let logical = Vec2::new(256.0, 128.0);
            let view = View {
                view_projection: super::super::Camera2D::default().view_projection(logical),
                physical_size: logical * scale,
                target_is_srgb: false,
            };
            let mut atlas = GlyphAtlas::new(256);
            let list = built_with_text(&world, &view, &fonts, &mut atlas);
            let mut expected = Vec::new();
            text::layout(
                "Hx",
                font,
                fonts.get(font).expect("font"),
                16.0 * scale,
                crate::render::TextAnchor::BASELINE_LEFT,
                &mut atlas,
                false,
                &mut Vec::new(),
                |g| expected.push(g.image.size),
            )
            .expect("fits");
            assert_eq!(list.instances().len(), 2);
            for (instance, (w, h)) in list.instances().iter().zip(expected) {
                let [l, t, r, b] = pixel_rect(instance, &view);
                for edge in [l, t, r, b] {
                    assert!(
                        (edge - edge.round()).abs() < 1e-3,
                        "scale {scale}: edge {edge}"
                    );
                }
                assert!(((r - l) - w as f32).abs() < 1e-3 && ((b - t) - h as f32).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn invisible_or_oversized_text_is_skipped() {
        let (fonts, font) = super::super::font::tests::poppins();
        let mut atlas = GlyphAtlas::new(256);
        let mut world = World::new();
        world.spawn((
            Transform2D::default().with_scale(Vec2::ZERO),
            Text::new("A", font, 16.0),
        ));
        world.spawn((Transform2D::default(), Text::new("A", font, 0.0)));
        world.spawn((Transform2D::default(), Text::new("A", font, f32::NAN)));
        world.spawn((Transform2D::default(), Text::new("A", font, 600.0)));
        let view = text_view(super::super::Camera2D::default());
        assert!(
            built_with_text(&world, &view, &fonts, &mut atlas)
                .instances()
                .is_empty()
        );
        let empty = View {
            physical_size: Vec2::ZERO,
            ..view
        };
        world.spawn((Transform2D::default(), Text::new("A", font, 16.0)));
        assert!(
            built_with_text(&world, &empty, &fonts, &mut atlas)
                .instances()
                .is_empty()
        );
    }

    #[test]
    fn a_full_atlas_is_cleared_once_and_the_frame_is_laid_out_again() {
        let (fonts, font) = super::super::font::tests::poppins();
        let mut atlas = GlyphAtlas::new(128);
        let mut world = World::new();
        // Each text fits alone; together they overflow a 128×128 atlas.
        world.spawn((
            Transform2D::default(),
            Text::new("ABCDEFGHIJKLM", font, 40.0),
        ));
        world.spawn((
            Transform2D::default(),
            Text::new("NOPQRSTUVWXYZ", font, 40.0),
        ));
        let view = text_view(super::super::Camera2D::default());
        let list = built_with_text(&world, &view, &fonts, &mut atlas);
        assert_eq!(atlas.resets(), 1);
        let drawn = list.instances().len();
        assert!(drawn > 0 && drawn < 26, "{drawn}");
        // Every glyph drawn this frame refers to the atlas as it is now: its
        // texels are not all transparent.
        for instance in list.instances() {
            let s = atlas.size() as f32;
            let (x, y) = (
                (instance.uv_rect[0] * s) as u32,
                (instance.uv_rect[1] * s) as u32,
            );
            let (w, h) = (
                (instance.uv_rect[2] * s) as u32,
                (instance.uv_rect[3] * s) as u32,
            );
            let any = (0..h).any(|j| (0..w).any(|i| atlas.texel(x + i, y + j)[3] > 0));
            assert!(any, "glyph at {x},{y} is in the atlas");
        }
    }
}
