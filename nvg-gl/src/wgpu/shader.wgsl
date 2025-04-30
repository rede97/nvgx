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

const ShaderTypeFillGradient: u32 = 0;
const ShaderTypeFillImage: u32 = 1;
const ShaderTypeSimple: u32 = 2;
const ShaderTypeImage: u32 = 3;

struct RenderUnifrom {
    scissor_mat: mat3x3f,
    paint_mat: mat3x3f,
    inner_color: vec4f,
    outer_color: vec4f,
    scissor_ext: vec2f,
    scissor_scale: vec2f,
    extent: vec2f,
    radius: f32,
    feather: f32,
    stroke_mult: f32,
    stroke_thr: f32,
    texture_type: u32,
    render_type: u32,
}

@group(1) @binding(0)
var<uniform> render_uniform: RenderUnifrom;
@group(2) @binding(0)
var frag_texture: texture_2d<f32>;
@group(2) @binding(1)
var frag_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    if render_uniform.render_type == 3 {
        return textureSample(frag_texture, frag_sampler, in.ftcoord);
    } else {
        return vec4f(1.0, 1.0, 1.0, 1.0);
    }
}
