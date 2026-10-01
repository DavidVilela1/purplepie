// PurplePie: textured sprites (ADR-019, ADR-020).
//
// Same instancing scheme as quad.wgsl: one instance per sprite, corners from
// the vertex index, no vertex buffer. The instance colour is a tint that
// multiplies the texture sample. The texture's view format matches the target
// (sRGB or not), so the result needs no further conversion here.

struct Instance {
    @location(0) clip_from_local_0: vec4<f32>,
    @location(1) clip_from_local_1: vec4<f32>,
    @location(2) clip_from_local_2: vec4<f32>,
    @location(3) clip_from_local_3: vec4<f32>,
    @location(4) tint: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@group(0) @binding(0) var sprite_texture: texture_2d<f32>;
@group(0) @binding(1) var sprite_sampler: sampler;

// Two triangles covering the unit square centred on the origin.
const CORNERS = array<vec2<f32>, 6>(
    vec2<f32>(-0.5, -0.5),
    vec2<f32>( 0.5, -0.5),
    vec2<f32>( 0.5,  0.5),
    vec2<f32>(-0.5, -0.5),
    vec2<f32>( 0.5,  0.5),
    vec2<f32>(-0.5,  0.5),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: Instance) -> VertexOutput {
    let clip_from_local = mat4x4<f32>(
        instance.clip_from_local_0,
        instance.clip_from_local_1,
        instance.clip_from_local_2,
        instance.clip_from_local_3,
    );
    let corner = CORNERS[vertex_index];
    var out: VertexOutput;
    out.position = clip_from_local * vec4<f32>(corner, 0.0, 1.0);
    // World +Y is up but image rows run top to bottom: the top edge samples v = 0.
    out.uv = vec2<f32>(corner.x + 0.5, 0.5 - corner.y);
    out.tint = instance.tint;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, in.uv) * in.tint;
}
