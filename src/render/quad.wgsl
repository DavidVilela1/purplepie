// PurplePie: solid-colour quads (ADR-019).
//
// One instance per quad. The CPU supplies the full local-to-clip matrix
// (view-projection × model × size) and a colour already converted for the
// target format. The unit square's corners come from the vertex index, so
// there is no vertex buffer.

struct Instance {
    @location(0) clip_from_local_0: vec4<f32>,
    @location(1) clip_from_local_1: vec4<f32>,
    @location(2) clip_from_local_2: vec4<f32>,
    @location(3) clip_from_local_3: vec4<f32>,
    @location(4) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

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
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
