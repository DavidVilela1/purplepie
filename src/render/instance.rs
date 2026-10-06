//! Per-instance GPU data and its growable buffer, shared by the quad and
//! sprite pipelines (ADR-019, ADR-020, ADR-027).

use super::Color;
use crate::math::{Mat4, Transform2D, Vec2};

/// One drawn rectangle: matches `Instance` in `quad.wgsl` and `sprite.wgsl`
/// (6 × vec4<f32> = 96 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Instance {
    /// view-projection × model × size: maps the unit square to clip space.
    pub(crate) clip_from_local: [[f32; 4]; 4],
    /// Fill colour (quads) or tint (sprites), already converted for the target format.
    pub(crate) color: [f32; 4],
    /// Texture region shown (sprites, glyphs): top-left u, v and width, height
    /// in texture coordinates (0..1, v down). Quads ignore it.
    pub(crate) uv_rect: [f32; 4],
}

impl Instance {
    pub(crate) const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4,
        5 => Float32x4
    ];

    /// The whole texture.
    pub(crate) const FULL_UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    /// A `size`-sized rectangle centred on `transform`, filled or tinted with `color`.
    pub(crate) fn new(
        view_projection: &Mat4,
        transform: &Transform2D,
        size: Vec2,
        color: Color,
        target_is_srgb: bool,
    ) -> Self {
        let local = Mat4::from_scale(size.extend(1.0));
        let clip_from_local = *view_projection * transform.to_mat4() * local;
        Self::from_matrix(&clip_from_local, color, Self::FULL_UV, target_is_srgb)
    }

    /// A rectangle that is the unit square mapped by `clip_from_local`,
    /// showing the `uv_rect` region of its texture.
    pub(crate) fn from_matrix(
        clip_from_local: &Mat4,
        color: Color,
        uv_rect: [f32; 4],
        target_is_srgb: bool,
    ) -> Self {
        let c = color.to_wgpu(target_is_srgb);
        Self {
            clip_from_local: clip_from_local.to_cols_array_2d(),
            color: [c.r as f32, c.g as f32, c.b as f32, c.a as f32],
            uv_rect,
        }
    }

    /// The vertex-buffer layout: one `Instance` per drawn rectangle.
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// A GPU vertex buffer of [`Instance`]s that grows (to the next power of two) as needed.
pub(crate) struct InstanceBuffer {
    label: &'static str,
    buffer: Option<wgpu::Buffer>,
    capacity: usize,
}

impl InstanceBuffer {
    pub(crate) const fn new(label: &'static str) -> Self {
        Self {
            label,
            buffer: None,
            capacity: 0,
        }
    }

    /// Copies `instances` to the GPU, reallocating first if they do not fit.
    pub(crate) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[Instance],
    ) {
        if instances.is_empty() {
            return;
        }
        if instances.len() > self.capacity || self.buffer.is_none() {
            self.capacity = instances.len().next_power_of_two();
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: (self.capacity * std::mem::size_of::<Instance>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    /// The whole buffer, or `None` before the first upload.
    pub(crate) fn slice(&self) -> Option<wgpu::BufferSlice<'_>> {
        self.buffer.as_ref().map(|b| b.slice(..))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_layout_matches_the_shaders() {
        assert_eq!(std::mem::size_of::<Instance>(), 96);
        assert_eq!(Instance::layout().array_stride, 96);
    }
}
