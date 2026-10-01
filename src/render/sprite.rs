//! Textured rectangles drawn from ECS data (ADR-019, ADR-020).
//!
//! `Sprite` is the public component. The rest is crate-private: uploading
//! textures and the textured pipeline. Collection, sorting and batching live
//! in `draw.rs` (ADR-021).

use wgpu::util::DeviceExt as _;

use super::Color;
use super::quad::rect_pipeline;
use super::texture::{TextureData, TextureId, Textures};
use crate::error::{Error, Result};
use crate::math::Vec2;

/// A textured rectangle, drawn centred on the entity's [`Transform2D`](crate::math::Transform2D).
///
/// The whole texture is stretched over `size` world units (before the
/// transform's scale). `tint` multiplies every texel: [`Color::WHITE`] shows
/// the texture unchanged, and a lower alpha fades it. An entity needs both
/// `Transform2D` and `Sprite` to be drawn. Draw order comes from the entity's
/// [`Layer`](super::Layer) (default 0); within a layer, sprites are drawn after quads.
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

/// A texture on the GPU. The bind group keeps the texture view alive.
struct GpuTexture {
    bind_group: wgpu::BindGroup,
}

/// The textured pipeline, the GPU copies of every loaded texture, and the
pub(crate) struct SpritePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Texture formats follow the target: sRGB targets sample sRGB textures
    /// (decoded to linear), others sample the raw values (ADR-015).
    texture_format: wgpu::TextureFormat,
    /// Indexed by `TextureId`. Grows as the store does (`sync_textures`).
    textures: Vec<GpuTexture>,
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

    /// Selects this pipeline for the following draws.
    pub(crate) fn bind(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
    }

    /// Binds `texture` for the following draws. Returns `false` (and logs) if it
    /// is not on the GPU, which cannot happen after `sync_textures`.
    pub(crate) fn bind_texture(&self, pass: &mut wgpu::RenderPass<'_>, texture: TextureId) -> bool {
        match self.textures.get(texture.index()) {
            Some(gpu) => {
                pass.set_bind_group(0, &gpu.bind_group, &[]);
                true
            }
            None => {
                log::warn!("sprite texture {texture:?} is not on the GPU; skipped");
                false
            }
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
    use super::super::draw::DrawList;
    use super::super::quad::tests::headless_device_and_queue;
    use super::super::texture::tests::encode_png;
    use super::*;
    use crate::math::Transform2D;

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
    fn sprite_instance_uses_size_transform_and_tint() {
        let mut textures = Textures::default();
        let id = load(&mut textures, "inst.png", 1, 1);
        let mut world = crate::ecs::World::new();
        world.spawn((
            Transform2D::from_position(Vec2::new(50.0, 0.0)),
            Sprite::new(id, Vec2::new(40.0, 20.0)).with_tint(Color::rgba(1.0, 0.5, 0.0, 0.25)),
        ));
        let vp = super::super::Camera2D::default().view_projection(Vec2::new(200.0, 100.0));
        let mut list = DrawList::default();
        list.build(&world, &vp, false);
        let instance = list.instances()[0];
        let m = glam::Mat4::from_cols_array_2d(&instance.clip_from_local);
        // Unit-square corner (0.5, 0.5) → world (70, 10) → clip (0.7, 0.2).
        let corner = m * glam::Vec4::new(0.5, 0.5, 0.0, 1.0);
        assert!(
            corner
                .truncate()
                .truncate()
                .abs_diff_eq(Vec2::new(0.7, 0.2), 1e-6)
        );
        assert_eq!(instance.color, [1.0, 0.5, 0.0, 0.25]);
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
