//! Solid-colour rectangles drawn from ECS data (ADR-018, ADR-019).
//!
//! `Quad` is the public component. Everything else here is crate-private GPU
//! plumbing: the quad pipeline (the view projection lives in `camera.rs`). Collection, sorting and
//! batching live in `draw.rs` (ADR-021).

use super::Color;
use super::instance::Instance;
use crate::math::Vec2;

/// A solid-colour rectangle, drawn centred on the entity's [`Transform2D`](crate::math::Transform2D).
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

/// The solid-colour quad pipeline. It has no bind groups: everything comes
/// from the shared instance buffer (ADR-019, ADR-021).
pub(crate) struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl QuadPipeline {
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
        let pipeline = rect_pipeline(device, "purplepie quad pipeline", &layout, &shader, format);
        Self { pipeline }
    }

    /// Selects this pipeline for the following draws.
    pub(crate) fn bind(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
    }
}

/// A render pipeline that draws instanced unit squares (`vs_main`/`fs_main` in
/// `shader`) with straight-alpha blending. Shared by quads and sprites.
pub(crate) fn rect_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(Instance::layout())],
        },
        primitive: wgpu::PrimitiveState {
            // Negative scale mirrors a rectangle, which flips its winding, so never cull.
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
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
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::math::{Mat4, Transform2D};
    use glam::Vec4;

    fn clip(m: &[[f32; 4]; 4], local: Vec2) -> Vec2 {
        let p = Mat4::from_cols_array_2d(m) * Vec4::new(local.x, local.y, 0.0, 1.0);
        Vec2::new(p.x / p.w, p.y / p.w)
    }

    fn quad_instance(vp: &Mat4, t: &Transform2D, quad: &Quad, srgb: bool) -> Instance {
        Instance::new(vp, t, quad.size, quad.color, srgb)
    }

    #[test]
    fn view_maps_window_center_to_clip_origin_with_y_up() {
        let vp = super::super::Camera2D::default().view_projection(Vec2::new(200.0, 100.0));
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
        let p = super::super::Camera2D::default().view_projection(Vec2::new(200.0, 100.0))
            * Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!((0.0..=1.0).contains(&p.z));
    }

    #[test]
    fn quad_corners_land_where_expected() {
        let vp = super::super::Camera2D::default().view_projection(Vec2::new(200.0, 100.0));
        let quad = Quad::new(Vec2::new(40.0, 20.0), Color::WHITE);
        let transform = Transform2D::from_position(Vec2::new(50.0, 0.0));
        let inst = quad_instance(&vp, &transform, &quad, false);
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
        let vp = super::super::Camera2D::default().view_projection(Vec2::new(200.0, 200.0));
        let quad = Quad::new(Vec2::new(20.0, 20.0), Color::WHITE);
        let transform = Transform2D::default()
            .with_rotation(std::f32::consts::FRAC_PI_2)
            .with_scale(Vec2::splat(2.0));
        let inst = quad_instance(&vp, &transform, &quad, false);
        // Corner (0.5, 0) → size (10, 0) → scale (20, 0) → +90° (0, 20) → clip (0, 0.2).
        assert!(
            clip(&inst.clip_from_local, Vec2::new(0.5, 0.0)).abs_diff_eq(Vec2::new(0.0, 0.2), 1e-6)
        );
    }

    #[test]
    fn color_is_converted_for_the_target_format() {
        let vp = super::super::Camera2D::default().view_projection(Vec2::new(100.0, 100.0));
        let quad = Quad::new(Vec2::ONE, Color::rgba(0.5, 0.5, 0.5, 0.5));
        let t = Transform2D::default();
        let srgb = quad_instance(&vp, &t, &quad, true);
        let unorm = quad_instance(&vp, &t, &quad, false);
        assert!((srgb.color[0] - 0.214_041).abs() < 1e-5);
        assert_eq!(unorm.color[0], 0.5);
        assert_eq!(srgb.color[3], 0.5); // alpha is never gamma-converted
    }

    #[test]
    fn only_entities_with_transform_and_quad_are_drawn() {
        let mut world = crate::ecs::World::new();
        world.spawn((Transform2D::default(), Quad::new(Vec2::ONE, Color::WHITE)));
        world.spawn((Transform2D::default(),));
        world.spawn((Quad::new(Vec2::ONE, Color::WHITE),));
        let mut list = super::super::draw::DrawList::default();
        list.build(&world, &Mat4::IDENTITY, false);
        assert_eq!(list.instances().len(), 1);
    }

    /// Headless device and queue with PurplePie's fault capture installed.
    /// Needs a GPU adapter (hardware or software, e.g. Mesa lavapipe).
    pub(crate) fn headless_device_and_queue()
    -> (wgpu::Device, wgpu::Queue, super::super::faults::FaultSlot) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("a GPU adapter is required for this ignored test");
        // The same limits the engine requests (renderer.rs).
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        }))
        .expect("device");
        let faults = super::super::faults::FaultSlot::default();
        faults.install(&device);
        (device, queue, faults)
    }

    fn headless_device() -> (wgpu::Device, super::super::faults::FaultSlot) {
        let (device, _queue, faults) = headless_device_and_queue();
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
