//! Textured rectangles drawn from ECS data (ADR-019, ADR-020).
//!
//! `Sprite` is the public component. The rest is crate-private: collecting
//! sprite instances, uploading textures, and the textured pipeline.

use std::ops::Range;

use wgpu::util::DeviceExt as _;

use super::Color;
use super::instance::{Instance, InstanceBuffer};
use super::quad::rect_pipeline;
use super::texture::{TextureData, TextureId, Textures};
use crate::ecs::World;
use crate::error::{Error, Result};
use crate::math::{Mat4, Transform2D, Vec2};

/// A textured rectangle, drawn centred on the entity's [`Transform2D`].
///
/// The whole texture is stretched over `size` world units (before the
/// transform's scale). `tint` multiplies every texel: [`Color::WHITE`] shows
/// the texture unchanged, and a lower alpha fades it. An entity needs both
/// `Transform2D` and `Sprite` to be drawn. Sprites are drawn after (on top
/// of) quads; the order among sprites is not defined yet (PP-015).
///
/// ```no_run
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::Sprite;
/// # fn init(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
/// let texture = ctx.load_texture("assets/textures/player.png")?;
/// ctx.world_mut().spawn((
///     Transform2D::from_position(Vec2::new(0.0, 100.0)),
///     Sprite::new(texture, Vec2::new(64.0, 64.0)),
/// ));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    /// The image to draw, from [`Context::load_texture`](crate::Context::load_texture).
    pub texture: TextureId,
    /// Width and height in world units (logical pixels with the default view).
    pub size: Vec2,
    /// Multiplies the texture's colour and alpha (sRGB, straight alpha; ADR-015).
    pub tint: Color,
}

impl Sprite {
    /// A sprite showing `texture` over `size` world units, untinted.
    pub const fn new(texture: TextureId, size: Vec2) -> Self {
        Self {
            texture,
            size,
            tint: Color::WHITE,
        }
    }

    /// The same sprite with `tint` applied.
    pub const fn with_tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }
}

/// A run of consecutive instances that share one texture: one draw call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Batch {
    pub(crate) texture: TextureId,
    pub(crate) instances: Range<u32>,
}

/// Reads `(Transform2D, Sprite)` from the world into `instances`, and groups
/// consecutive sprites with the same texture into `batches` (both reused every
/// frame). Read-only access to the world (ADR-009). Sprites keep query order;
/// sorting by layer and texture is PP-015.
pub(crate) fn collect_sprites(
    world: &World,
    view_projection: &Mat4,
    target_is_srgb: bool,
    instances: &mut Vec<Instance>,
    batches: &mut Vec<Batch>,
) {
    instances.clear();
    batches.clear();
    for (transform, sprite) in world.query::<(&Transform2D, &Sprite)>().iter() {
        let index = u32::try_from(instances.len()).unwrap_or(u32::MAX);
        instances.push(Instance::new(
            view_projection,
            transform,
            sprite.size,
            sprite.tint,
            target_is_srgb,
        ));
        match batches.last_mut() {
            Some(batch) if batch.texture == sprite.texture => batch.instances.end = index + 1,
            _ => batches.push(Batch {
                texture: sprite.texture,
                instances: index..index + 1,
            }),
        }
    }
}

/// A texture on the GPU. The bind group keeps the texture view alive.
struct GpuTexture {
    bind_group: wgpu::BindGroup,
}

/// The textured pipeline, the GPU copies of every loaded texture, and the
/// sprite instance buffer.
pub(crate) struct SpritePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Texture formats follow the target: sRGB targets sample sRGB textures
    /// (decoded to linear), others sample the raw values (ADR-015).
    texture_format: wgpu::TextureFormat,
    /// Indexed by `TextureId`. Grows as the store does (`sync_textures`).
    textures: Vec<GpuTexture>,
    instances: InstanceBuffer,
}

impl SpritePipeline {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("purplepie sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("purplepie sprite texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("purplepie sprite layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = rect_pipeline(
            device,
            "purplepie sprite pipeline",
            &layout,
            &shader,
            format,
        );
        // Nearest filtering: texels stay crisp (pixel art) and pixel tests are exact.
        // Clamp so the edges never pick up texels from the opposite side.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("purplepie sprite sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture_format = if format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };
        Self {
            pipeline,
            bind_group_layout,
            sampler,
            texture_format,
            textures: Vec::new(),
            instances: InstanceBuffer::new("purplepie sprite instances"),
        }
    }

    /// Uploads every texture in `store` that is not on the GPU yet.
    ///
    /// Fails with [`Error::Asset`] if a texture is larger than this GPU allows.
    pub(crate) fn sync_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        store: &Textures,
    ) -> Result<()> {
        for (id, data) in store.since(self.textures.len()) {
            debug_assert_eq!(id.index(), self.textures.len());
            let gpu = self.upload_texture(device, queue, data)?;
            self.textures.push(gpu);
        }
        Ok(())
    }

    fn upload_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &TextureData,
    ) -> Result<GpuTexture> {
        let max = device.limits().max_texture_dimension_2d;
        if data.width > max || data.height > max {
            return Err(Error::Asset {
                path: data.source.clone(),
                source: format!(
                    "the image is {}×{} pixels, but this GPU supports at most {max}×{max}",
                    data.width, data.height
                )
                .into(),
            });
        }
        let label = data.source.display().to_string();
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(&label),
                size: wgpu::Extent3d {
                    width: data.width,
                    height: data.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.texture_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &data.pixels,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&label),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        Ok(GpuTexture { bind_group })
    }

    /// Uploads this frame's instances.
    pub(crate) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[Instance],
    ) {
        self.instances.upload(device, queue, instances);
    }

    /// Records one draw call per batch.
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>, batches: &[Batch]) {
        let Some(buffer) = self.instances.slice() else {
            return;
        };
        if batches.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, buffer);
        for batch in batches {
            // Every stored texture is synced before drawing, so this always succeeds.
            let Some(gpu) = self.textures.get(batch.texture.index()) else {
                log::warn!(
                    "sprite texture {:?} is not on the GPU; skipped",
                    batch.texture
                );
                continue;
            };
            pass.set_bind_group(0, &gpu.bind_group, &[]);
            pass.draw(0..6, batch.instances.clone());
        }
    }

    /// Number of textures on the GPU.
    #[cfg(test)]
    pub(crate) fn texture_count(&self) -> usize {
        self.textures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::super::quad::tests::headless_device_and_queue;
    use super::super::quad::view_projection;
    use super::super::texture::tests::encode_png;
    use super::*;

    fn load(textures: &mut Textures, name: &str, w: u32, h: u32) -> TextureId {
        let path = std::env::temp_dir().join(format!("purplepie-{}-{name}", std::process::id()));
        let pixels = vec![255; (w * h * 4) as usize];
        std::fs::write(&path, encode_png(w, h, &pixels)).expect("write");
        let id = textures.load(&path).expect("load");
        std::fs::remove_file(&path).ok();
        id
    }

    #[test]
    fn sprite_defaults_to_an_untinted_texture() {
        let mut textures = Textures::default();
        let id = load(&mut textures, "default.png", 1, 1);
        let sprite = Sprite::new(id, Vec2::new(8.0, 4.0));
        assert_eq!(sprite.tint, Color::WHITE);
        assert_eq!(sprite.with_tint(Color::BLACK).tint, Color::BLACK);
    }

    #[test]
    fn consecutive_sprites_with_the_same_texture_share_a_batch() {
        let mut textures = Textures::default();
        let a = load(&mut textures, "batch-a.png", 1, 1);
        let b = load(&mut textures, "batch-b.png", 1, 1);
        let mut world = World::new();
        // Same archetype, so hecs yields them in spawn order.
        for texture in [a, a, b, b, b, a] {
            world.spawn((Transform2D::default(), Sprite::new(texture, Vec2::ONE)));
        }
        world.spawn((Transform2D::default(),)); // no sprite: ignored
        let (mut instances, mut batches) = (Vec::new(), Vec::new());
        collect_sprites(&world, &Mat4::IDENTITY, false, &mut instances, &mut batches);
        assert_eq!(instances.len(), 6);
        assert_eq!(
            batches,
            [
                Batch {
                    texture: a,
                    instances: 0..2
                },
                Batch {
                    texture: b,
                    instances: 2..5
                },
                Batch {
                    texture: a,
                    instances: 5..6
                },
            ]
        );
    }

    #[test]
    fn sprite_instance_uses_size_transform_and_tint() {
        let mut textures = Textures::default();
        let id = load(&mut textures, "inst.png", 1, 1);
        let mut world = World::new();
        world.spawn((
            Transform2D::from_position(Vec2::new(50.0, 0.0)),
            Sprite::new(id, Vec2::new(40.0, 20.0)).with_tint(Color::rgba(1.0, 0.5, 0.0, 0.25)),
        ));
        let vp = view_projection(Vec2::new(200.0, 100.0));
        let (mut instances, mut batches) = (Vec::new(), Vec::new());
        collect_sprites(&world, &vp, false, &mut instances, &mut batches);
        let m = Mat4::from_cols_array_2d(&instances[0].clip_from_local);
        // Unit-square corner (0.5, 0.5) → world (70, 10) → clip (0.7, 0.2).
        let corner = m * glam::Vec4::new(0.5, 0.5, 0.0, 1.0);
        assert!(
            corner
                .truncate()
                .truncate()
                .abs_diff_eq(Vec2::new(0.7, 0.2), 1e-6)
        );
        assert_eq!(instances[0].color, [1.0, 0.5, 0.0, 0.25]);
    }

    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn sprite_pipeline_builds_and_uploads_textures_without_gpu_errors() {
        let (device, queue, faults) = headless_device_and_queue();
        let mut textures = Textures::default();
        let first = load(&mut textures, "gpu-a.png", 3, 5);
        for format in [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ] {
            let mut sprites = SpritePipeline::new(&device, format);
            sprites
                .sync_textures(&device, &queue, &textures)
                .expect("sync");
            assert_eq!(sprites.texture_count(), 1);
            assert!(
                faults.take().is_none(),
                "sprite pipeline raised a GPU error for {format:?}"
            );
        }
        // Textures loaded later are picked up by the next sync, and only once.
        let mut sprites = SpritePipeline::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
        sprites
            .sync_textures(&device, &queue, &textures)
            .expect("sync");
        let second = load(&mut textures, "gpu-b.png", 2, 2);
        sprites
            .sync_textures(&device, &queue, &textures)
            .expect("sync");
        sprites
            .sync_textures(&device, &queue, &textures)
            .expect("sync again");
        assert_eq!(sprites.texture_count(), 2);
        assert_ne!(first, second);
        assert!(faults.take().is_none());
    }

    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn texture_larger_than_the_gpu_limit_is_an_asset_error() {
        let (device, queue, faults) = headless_device_and_queue();
        let max = device.limits().max_texture_dimension_2d;
        let mut textures = Textures::default();
        // One texel wider than allowed, one texel tall: small to encode.
        let id = load(&mut textures, "too-wide.png", max + 1, 1);
        let mut sprites = SpritePipeline::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
        let err = sprites
            .sync_textures(&device, &queue, &textures)
            .expect_err("oversized texture must fail");
        assert!(matches!(err, Error::Asset { .. }), "{err}");
        assert_eq!(sprites.texture_count(), 0);
        assert!(faults.take().is_none(), "rejected before reaching wgpu");
        let _ = id;
    }
}
