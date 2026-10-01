//! Solid-colour rectangles drawn from ECS data (ADR-018, ADR-019).
//!
//! `Quad` is the public component. Everything else here is crate-private GPU
//! plumbing: the view projection, per-instance data, and the instanced pipeline.

use super::Color;
use crate::ecs::World;
use crate::math::{Mat4, Transform2D, Vec2};

/// A solid-colour rectangle, drawn centred on the entity's [`Transform2D`].
///
/// `size` is in world units before the transform's scale. Rotation turns the
/// rectangle around its centre. An entity needs both `Transform2D` and `Quad`
/// to be drawn.
///
/// ```
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::{Color, Quad};
///
/// let mut world = purplepie::ecs::World::new();
/// world.spawn((
///     Transform2D::from_position(Vec2::new(0.0, 100.0)),
///     Quad::new(Vec2::new(64.0, 32.0), Color::WHITE),
/// ));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    /// Width and height in world units (logical pixels with the default view).
    pub size: Vec2,
    /// Fill colour (sRGB, straight alpha; ADR-015).
    pub color: Color,
}

impl Quad {
    /// A quad of `size` world units filled with `color`.
    pub const fn new(size: Vec2, color: Color) -> Self {
        Self { size, color }
    }
}

/// World → clip transform for the default view (ADR-018): origin at the window
/// centre, +X right, +Y up, one world unit per logical pixel. A larger window
/// shows more of the world rather than scaling it.
pub(crate) fn view_projection(logical_size: Vec2) -> Mat4 {
    let half = logical_size * 0.5;
    // Right-handed, Y-up view space → WebGPU NDC (Y-up, depth in [0, 1]).
    glam::camera::rh::proj::directx::orthographic(-half.x, half.x, -half.y, half.y, -1.0, 1.0)
}

/// Per-quad GPU data: matches `Instance` in `quad.wgsl` (5 × vec4<f32> = 80 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct QuadInstance {
    clip_from_local: [[f32; 4]; 4],
    color: [f32; 4],
}

impl QuadInstance {
    pub(crate) fn new(
        view_projection: &Mat4,
        transform: &Transform2D,
        quad: &Quad,
        target_is_srgb: bool,
    ) -> Self {
        let local = Mat4::from_scale(quad.size.extend(1.0));
        let clip_from_local = *view_projection * transform.to_mat4() * local;
        let c = quad.color.to_wgpu(target_is_srgb);
        Self {
            clip_from_local: clip_from_local.to_cols_array_2d(),
            color: [c.r as f32, c.g as f32, c.b as f32, c.a as f32],
        }
    }
}

/// Reads `(Transform2D, Quad)` from the world into `out` (reused every frame).
/// Read-only access to the world (ADR-009).
pub(crate) fn collect_instances(
    world: &World,
    view_projection: &Mat4,
    target_is_srgb: bool,
    out: &mut Vec<QuadInstance>,
) {
    out.clear();
    for (transform, quad) in world.query::<(&Transform2D, &Quad)>().iter() {
        out.push(QuadInstance::new(
            view_projection,
            transform,
            quad,
            target_is_srgb,
        ));
    }
}

/// The instanced quad pipeline plus its growable instance buffer.
pub(crate) struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
    instances: Option<wgpu::Buffer>,
    capacity: usize,
}

impl QuadPipeline {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4
    ];

    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("purplepie quad shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("purplepie quad layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("purplepie quad pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &Self::ATTRIBUTES,
                })],
            },
            primitive: wgpu::PrimitiveState {
                // Negative scale mirrors a quad, which flips its winding, so never cull.
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            instances: None,
            capacity: 0,
        }
    }

    /// Uploads `instances`, growing the GPU buffer (to the next power of two) when needed.
    pub(crate) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[QuadInstance],
    ) {
        if instances.is_empty() {
            return;
        }
        if instances.len() > self.capacity || self.instances.is_none() {
            self.capacity = instances.len().next_power_of_two();
            self.instances = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("purplepie quad instances"),
                size: (self.capacity * std::mem::size_of::<QuadInstance>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if let Some(buffer) = &self.instances {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    /// Records the draw for `count` uploaded instances.
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>, count: usize) {
        let Some(buffer) = &self.instances else {
            return;
        };
        if count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..6, 0..count as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    fn clip(m: &[[f32; 4]; 4], local: Vec2) -> Vec2 {
        let p = Mat4::from_cols_array_2d(m) * Vec4::new(local.x, local.y, 0.0, 1.0);
        Vec2::new(p.x / p.w, p.y / p.w)
    }

    #[test]
    fn instance_layout_matches_the_shader() {
        assert_eq!(std::mem::size_of::<QuadInstance>(), 80);
    }

    #[test]
    fn view_maps_window_center_to_clip_origin_with_y_up() {
        let vp = view_projection(Vec2::new(200.0, 100.0));
        let at = |x: f32, y: f32| {
            let p = vp * Vec4::new(x, y, 0.0, 1.0);
            Vec2::new(p.x, p.y)
        };
        assert_eq!(at(0.0, 0.0), Vec2::ZERO);
        assert_eq!(at(100.0, 50.0), Vec2::new(1.0, 1.0)); // top-right corner
        assert_eq!(at(-100.0, -50.0), Vec2::new(-1.0, -1.0)); // bottom-left corner
        assert!(at(0.0, 10.0).y > 0.0, "+Y is up");
    }

    #[test]
    fn view_depth_for_z0_is_inside_the_clip_volume() {
        let p = view_projection(Vec2::new(200.0, 100.0)) * Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!((0.0..=1.0).contains(&p.z));
    }

    #[test]
    fn quad_corners_land_where_expected() {
        let vp = view_projection(Vec2::new(200.0, 100.0));
        let quad = Quad::new(Vec2::new(40.0, 20.0), Color::WHITE);
        let transform = Transform2D::from_position(Vec2::new(50.0, 0.0));
        let inst = QuadInstance::new(&vp, &transform, &quad, false);
        // Unit-square corner (0.5, 0.5) → world (70, 10) → clip (0.7, 0.2).
        assert!(
            clip(&inst.clip_from_local, Vec2::splat(0.5)).abs_diff_eq(Vec2::new(0.7, 0.2), 1e-6)
        );
        assert!(
            clip(&inst.clip_from_local, Vec2::splat(-0.5)).abs_diff_eq(Vec2::new(0.3, -0.2), 1e-6)
        );
    }

    #[test]
    fn scale_and_rotation_apply_to_the_quad() {
        let vp = view_projection(Vec2::new(200.0, 200.0));
        let quad = Quad::new(Vec2::new(20.0, 20.0), Color::WHITE);
        let transform = Transform2D::default()
            .with_rotation(std::f32::consts::FRAC_PI_2)
            .with_scale(Vec2::splat(2.0));
        let inst = QuadInstance::new(&vp, &transform, &quad, false);
        // Corner (0.5, 0) → size (10, 0) → scale (20, 0) → +90° (0, 20) → clip (0, 0.2).
        assert!(
            clip(&inst.clip_from_local, Vec2::new(0.5, 0.0)).abs_diff_eq(Vec2::new(0.0, 0.2), 1e-6)
        );
    }

    #[test]
    fn color_is_converted_for_the_target_format() {
        let vp = view_projection(Vec2::new(100.0, 100.0));
        let quad = Quad::new(Vec2::ONE, Color::rgba(0.5, 0.5, 0.5, 0.5));
        let t = Transform2D::default();
        let srgb = QuadInstance::new(&vp, &t, &quad, true);
        let unorm = QuadInstance::new(&vp, &t, &quad, false);
        assert!((srgb.color[0] - 0.214_041).abs() < 1e-5);
        assert_eq!(unorm.color[0], 0.5);
        assert_eq!(srgb.color[3], 0.5); // alpha is never gamma-converted
    }

    #[test]
    fn collect_reads_only_entities_with_transform_and_quad() {
        let mut world = World::new();
        world.spawn((Transform2D::default(), Quad::new(Vec2::ONE, Color::WHITE)));
        world.spawn((Transform2D::default(),));
        world.spawn((Quad::new(Vec2::ONE, Color::WHITE),));
        let mut out = vec![QuadInstance::zeroed(); 5];
        collect_instances(&world, &Mat4::IDENTITY, false, &mut out);
        assert_eq!(out.len(), 1);
    }

    use bytemuck::Zeroable;

    /// Headless device with PurplePie's fault capture installed. Needs a GPU
    /// adapter (hardware or software, e.g. Mesa lavapipe).
    fn headless_device() -> (wgpu::Device, super::super::faults::FaultSlot) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("a GPU adapter is required for this ignored test");
        let (device, _queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("device");
        let faults = super::super::faults::FaultSlot::default();
        faults.install(&device);
        (device, faults)
    }

    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn quad_pipeline_builds_on_a_real_device_without_gpu_errors() {
        let (device, faults) = headless_device();
        for format in [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ] {
            let _pipeline = QuadPipeline::new(&device, format);
            assert!(
                faults.take().is_none(),
                "quad pipeline raised a GPU error for {format:?}"
            );
        }
    }

    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn invalid_shader_is_captured_as_a_fault_not_a_panic() {
        let (device, faults) = headless_device();
        let _broken = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("deliberately broken"),
            source: wgpu::ShaderSource::Wgsl("@vertex fn vs_main( -> {".into()),
        });
        assert!(
            faults.take().is_some(),
            "a WGSL syntax error must be recorded"
        );
    }
}
