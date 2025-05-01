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
const ShaderTypeSimple: u32 = 2; // for stencil
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

fn sdroundrect(pt: vec2f, ext: vec2f, rad: f32) -> f32 {
    let ext2: vec2f = ext - vec2f(rad, rad);
    let d = abs(pt) - ext2;
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2f(0.0, 0.0))) - rad;
}

fn scissor_mask(p: vec2f) -> f32 {
    let sc = (abs((render_uniform.scissor_mat * vec3(p, 1.0)).xy) - render_uniform.scissor_ext);
    let sc2 = vec2(0.5, 0.5) - sc * render_uniform.scissor_scale;
    return clamp(sc2.x, 0.0, 1.0) * clamp(sc2.y, 0.0, 1.0);
}

fn stroke_mask(ftcoord: vec2f) -> f32 {
    return min(1.0, (1.0 - abs(ftcoord.x * 2.0 - 1.0)) * render_uniform.stroke_mult) * min(1.0, ftcoord.y);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let scissor = scissor_mask(in.fpos);
    let stroke_alpha = stroke_mask(in.ftcoord);
    if stroke_alpha < render_uniform.stroke_thr {
        discard;
    }

    let render_type = render_uniform.render_type;
    if render_type == ShaderTypeFillGradient {
        let pt = (render_uniform.paint_mat * vec3(in.fpos, 1.0)).xy;
        let d = clamp((sdroundrect(pt, render_uniform.extent, render_uniform.radius) + render_uniform.feather * 0.5) / render_uniform.feather, 0.0, 1.0);
        return mix(render_uniform.inner_color, render_uniform.outer_color, d) * stroke_alpha * scissor;
    } else if render_type == ShaderTypeFillImage {
        let pt = (render_uniform.paint_mat * vec3(in.fpos, 1.0)).xy / render_uniform.extent;
        var color = textureSample(frag_texture, frag_sampler, pt);
        if (render_uniform.texture_type == 1) {
            color = vec4(color.xyz * color.w, color.w);
        }
        if (render_uniform.texture_type == 2) {
            color = vec4(color.x);
        }
        return color * render_uniform.inner_color * stroke_alpha * scissor;
    } else if render_type == ShaderTypeImage {
        var color = textureSample(frag_texture, frag_sampler, in.ftcoord);
        if (render_uniform.texture_type == 1) {
            color = vec4(color.xyz * color.w, color.w);
        }
        if (render_uniform.texture_type == 2) { 
            color = vec4(color.x);
        }
        return color * scissor * render_uniform.inner_color;
    }
    // for stencil
    return vec4f(1.0, 1.0, 1.0, 1.0);
}