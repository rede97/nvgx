struct VertexInput {
    @location(0) vertex: vec2f,
    @location(1) tcoord: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) ftcoord: vec2f,
    @location(1) fpos: vec2f,
}

@group(0) @binding(0)
var<uniform> view_size: vec2f;

@vertex
fn vs_main(vert_in: VertexInput,) -> VertexOutput {
    var out: VertexOutput;
    out.ftcoord = vert_in.tcoord;
    out.fpos = vert_in.vertex;
    out.clip_position = vec4f(2.0 * vert_in.vertex.x / view_size.x - 1.0, 1.0 - 2.0 * vert_in.vertex.y / view_size.y, 0.0, 1.0);
    return out;
}