#[macro_use]
#[allow(unused)]
extern crate anyhow;

pub mod fb;

use std::{ops::Range, sync::Arc};

use nvgx::{
    utils::{premul_color, xform_to_3x4},
    *,
};
use renderer::GLArrayBuffer;
use slab::Slab;

mod renderer;

pub struct RenderConfig {
    antialias: bool,
}

impl RenderConfig {
    pub fn antialias(mut self, antialias: bool) -> Self {
        self.antialias = antialias;
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self { antialias: true }
    }
}

struct Shader {
    prog: gl::types::GLuint,
    frag: gl::types::GLuint,
    vert: gl::types::GLuint,
    loc_viewsize: i32,
    loc_tex: i32,
    loc_frag: u32,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.prog);
            gl::DeleteShader(self.vert);
            gl::DeleteShader(self.frag);
        }
    }
}

impl Shader {
    fn load() -> anyhow::Result<Shader> {
        unsafe {
            let mut status: gl::types::GLint = std::mem::zeroed();
            let prog = gl::CreateProgram();
            let vert = gl::CreateShader(gl::VERTEX_SHADER);
            let frag = gl::CreateShader(gl::FRAGMENT_SHADER);
            let vert_source =
                std::ffi::CString::from_vec_unchecked(include_bytes!("shader.vert").to_vec());
            let frag_source =
                std::ffi::CString::from_vec_unchecked(include_bytes!("shader.frag").to_vec());

            gl::ShaderSource(
                vert,
                1,
                [vert_source.as_ptr()].as_ptr() as *const *const i8,
                std::ptr::null(),
            );
            gl::ShaderSource(
                frag,
                1,
                [frag_source.as_ptr()].as_ptr() as *const *const i8,
                std::ptr::null(),
            );

            gl::CompileShader(vert);
            gl::GetShaderiv(vert, gl::COMPILE_STATUS, &mut status);
            if status != gl::TRUE as i32 {
                return Err(shader_error(vert, "shader.vert"));
            }

            gl::CompileShader(frag);
            gl::GetShaderiv(frag, gl::COMPILE_STATUS, &mut status);
            if status != gl::TRUE as i32 {
                return Err(shader_error(vert, "shader.frag"));
            }

            gl::AttachShader(prog, vert);
            gl::AttachShader(prog, frag);

            let name_vertex = std::ffi::CString::new("vertex").unwrap();
            let name_tcoord = std::ffi::CString::new("tcoord").unwrap();
            gl::BindAttribLocation(prog, 0, name_vertex.as_ptr() as *const i8);
            gl::BindAttribLocation(prog, 1, name_tcoord.as_ptr() as *const i8);

            gl::LinkProgram(prog);
            gl::GetProgramiv(prog, gl::LINK_STATUS, &mut status);
            if status != gl::TRUE as i32 {
                return Err(program_error(prog));
            }

            let name_viewsize = std::ffi::CString::new("viewSize").unwrap();
            let name_tex = std::ffi::CString::new("tex").unwrap();
            let name_frag = std::ffi::CString::new("frag").unwrap();

            Ok(Shader {
                prog,
                frag,
                vert,
                loc_viewsize: gl::GetUniformLocation(prog, name_viewsize.as_ptr() as *const i8),
                loc_tex: gl::GetUniformLocation(prog, name_tex.as_ptr() as *const i8),
                loc_frag: gl::GetUniformBlockIndex(prog, name_frag.as_ptr() as *const i8),
            })
        }
    }
}

enum ShaderType {
    FillGradient,
    FillImage,
    Simple,
    Image,
}

#[derive(PartialEq, Eq)]
enum CallType {
    Fill(PathFillType),
    ConvexFill,
    Stroke,
    Triangles,
    #[cfg(feature = "wirelines")]
    Lines,
}

struct Blend {
    src_rgb: gl::types::GLenum,
    dst_rgb: gl::types::GLenum,
    src_alpha: gl::types::GLenum,
    dst_alpha: gl::types::GLenum,
}

impl From<CompositeOperationState> for Blend {
    fn from(state: CompositeOperationState) -> Self {
        Blend {
            src_rgb: convert_blend_factor(state.src_rgb),
            dst_rgb: convert_blend_factor(state.dst_rgb),
            src_alpha: convert_blend_factor(state.src_alpha),
            dst_alpha: convert_blend_factor(state.dst_alpha),
        }
    }
}

struct Call {
    call_type: CallType,
    vert_buff: Option<Arc<GLArrayBuffer>>,
    instances: Option<(Arc<GLArrayBuffer>, GLSlice)>,
    image: Option<usize>,
    path_range: Range<usize>,
    triangle: GLSlice,
    uniform_offset: usize,
    blend_func: Blend,
}

struct Texture {
    tex: gl::types::GLuint,
    width: u32,
    height: u32,
    texture_type: TextureType,
    flags: ImageFlags,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.tex) }
    }
}

#[derive(Default, Clone, Copy)]
struct GLSlice {
    offset: u32,
    count: u32,
}

impl From<VertexSlice> for GLSlice {
    fn from(value: VertexSlice) -> Self {
        return Self {
            offset: value.offset as u32,
            count: value.count as u32,
        };
    }
}

impl From<Range<u32>> for GLSlice {
    fn from(value: Range<u32>) -> Self {
        return Self {
            offset: value.start as u32,
            count: value.count() as u32,
        };
    }
}

#[derive(Default)]
struct GLPath {
    fill: GLSlice,
    stroke: GLSlice,
}

#[derive(Default)]
#[allow(dead_code)]
struct FragUniforms {
    scissor_mat: [f32; 12],
    paint_mat: [f32; 12],
    inner_color: Color,
    outer_color: Color,
    scissor_ext: [f32; 2],
    scissor_scale: [f32; 2],
    extent: [f32; 2],
    radius: f32,
    feather: f32,
    stroke_mult: f32,
    stroke_thr: f32,
    tex_type: i32,
    type_: i32,
}

#[derive(Clone, Copy, Debug)]
struct DefaultFBO {
    fbo: gl::types::GLint,
    rbo: gl::types::GLint,
}

pub struct Renderer {
    shader: Shader,
    textures: Slab<Texture>,
    view: Extent,
    vert_buf: GLArrayBuffer,
    inst_buf: GLArrayBuffer,
    frag_buf: gl::types::GLuint,
    frag_size: usize,
    calls: Vec<Call>,
    paths: Vec<GLPath>,
    uniforms: Vec<u8>,
    config: RenderConfig,
    default_fbo: DefaultFBO,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.frag_buf);
        }
    }
}

impl Renderer {
    pub fn create(config: RenderConfig) -> anyhow::Result<Renderer> {
        unsafe {
            let shader = Shader::load()?;

            let vert_buf = GLArrayBuffer::new(BufferUsage::Vertex);
            let inst_buf = GLArrayBuffer::new(BufferUsage::Instance);
            inst_buf.update_data(bytemuck::bytes_of(&Transform::identity()));

            gl::UniformBlockBinding(shader.prog, shader.loc_frag, 0);
            let mut frag_buf: gl::types::GLuint = std::mem::zeroed();
            gl::GenBuffers(1, &mut frag_buf);

            let mut align = std::mem::zeroed();
            gl::GetIntegerv(gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT, &mut align);

            let frag_size = std::mem::size_of::<FragUniforms>() + (align as usize)
                - std::mem::size_of::<FragUniforms>() % (align as usize);

            gl::Finish();

            let mut default_fbo: gl::types::GLint = 0;
            let mut default_rbo: gl::types::GLint = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut default_fbo as *mut _);
            gl::GetIntegerv(gl::RENDERBUFFER_BINDING, &mut default_rbo as *mut _);

            Ok(Renderer {
                shader,
                textures: Default::default(),
                view: Default::default(),
                vert_buf,
                inst_buf,
                frag_buf,
                frag_size,
                calls: Default::default(),
                paths: Default::default(),
                uniforms: Default::default(),
                config,
                default_fbo: DefaultFBO {
                    fbo: default_fbo,
                    rbo: default_rbo,
                },
            })
        }
    }

    fn set_uniforms(&self, offset: usize, img: Option<usize>) {
        unsafe {
            gl::BindBufferRange(
                gl::UNIFORM_BUFFER,
                0,
                self.frag_buf,
                (offset * self.frag_size) as isize,
                std::mem::size_of::<FragUniforms>() as isize,
            );

            if let Some(img) = img {
                if let Some(texture) = self.textures.get(img) {
                    gl::BindTexture(gl::TEXTURE_2D, texture.tex);
                }
            } else {
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }
        }
    }

    #[inline]
    fn do_fill(&self, call: &Call, fill_type: PathFillType, inst_count: i32) {
        let paths = &self.paths[call.path_range.clone()];
        unsafe {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilMask(0xff);
            gl::StencilFunc(gl::ALWAYS, 0, 0xff);
            gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);

            self.set_uniforms(call.uniform_offset, call.image);
            if fill_type == PathFillType::Winding {
                gl::StencilOpSeparate(gl::FRONT, gl::KEEP, gl::KEEP, gl::INCR_WRAP);
                gl::StencilOpSeparate(gl::BACK, gl::KEEP, gl::KEEP, gl::DECR_WRAP);
            } else {
                gl::StencilOpSeparate(gl::FRONT_AND_BACK, gl::KEEP, gl::KEEP, gl::INVERT);
            }
            gl::Disable(gl::CULL_FACE);
            for path in paths {
                gl::DrawArraysInstanced(
                    gl::TRIANGLE_FAN,
                    path.fill.offset as i32,
                    path.fill.count as i32,
                    inst_count,
                );
            }
            gl::Enable(gl::CULL_FACE);

            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);

            self.set_uniforms(call.uniform_offset + 1, call.image);

            gl::StencilFunc(gl::EQUAL, 0x00, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
            for path in paths {
                gl::DrawArraysInstanced(
                    gl::TRIANGLE_STRIP,
                    path.stroke.offset as i32,
                    path.stroke.count as i32,
                    inst_count,
                );
            }

            gl::StencilFunc(gl::NOTEQUAL, 0x00, 0xff);
            gl::StencilOp(gl::ZERO, gl::ZERO, gl::ZERO);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                call.triangle.offset as i32,
                call.triangle.count as i32,
                inst_count,
            );

            gl::Disable(gl::STENCIL_TEST);
        }
    }

    #[inline]
    unsafe fn do_convex_fill(&self, call: &Call, inst_count: i32) {
        let paths = &self.paths[call.path_range.clone()];
        self.set_uniforms(call.uniform_offset, call.image);
        for path in paths {
            unsafe {
                gl::DrawArraysInstanced(
                    gl::TRIANGLE_FAN,
                    path.fill.offset as i32,
                    path.fill.count as i32,
                    inst_count
                );
                if path.stroke.count > 0 {
                    gl::DrawArraysInstanced(
                        gl::TRIANGLE_STRIP,
                        path.stroke.offset as i32,
                        path.stroke.count as i32,
                        inst_count
                    );
                }
            }
        }
    }

    #[inline]
    unsafe fn do_stroke(&self, call: &Call, inst_count: i32) {
        let paths = &self.paths[call.path_range.clone()];
        self.set_uniforms(call.uniform_offset, call.image);
        for path in paths {
            unsafe {
                gl::DrawArraysInstanced(
                    gl::TRIANGLE_STRIP,
                    path.stroke.offset as i32,
                    path.stroke.count as i32,
                    inst_count
                );
            }
        }
    }

    #[inline]
    fn do_triangles(&self, call: &Call, inst_count: i32) {
        self.set_uniforms(call.uniform_offset, call.image);
        unsafe {
            gl::DrawArraysInstanced(
                gl::TRIANGLES,
                call.triangle.offset as i32,
                call.triangle.count as i32,
                inst_count
            );
        }
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    unsafe fn do_lines(&self, call: &Call, inst_count: i32) {
        let paths = &self.paths[call.path_range.clone()];
        self.set_uniforms(call.uniform_offset, call.image);
        for path in paths {
            unsafe {
                gl::DrawArraysInstanced(
                    gl::LINE_STRIP,
                    path.stroke.offset as i32,
                    path.stroke.count as i32,
                    inst_count
                );
            }
        }
    }

    fn convert_paint(
        &self,
        paint: &PaintPattern,
        scissor: &Scissor,
        width: f32,
        fringe: f32,
        stroke_thr: f32,
    ) -> FragUniforms {
        let mut frag = FragUniforms {
            scissor_mat: Default::default(),
            paint_mat: Default::default(),
            inner_color: premul_color(paint.inner_color),
            outer_color: premul_color(paint.outer_color),
            scissor_ext: Default::default(),
            scissor_scale: Default::default(),
            extent: Default::default(),
            radius: 0.0,
            feather: 0.0,
            stroke_mult: 0.0,
            stroke_thr,
            tex_type: 0,
            type_: 0,
        };

        if scissor.extent.width < -0.5 || scissor.extent.height < -0.5 {
            frag.scissor_ext[0] = 1.0;
            frag.scissor_ext[1] = 1.0;
            frag.scissor_scale[0] = 1.0;
            frag.scissor_scale[1] = 1.0;
        } else {
            frag.scissor_mat = xform_to_3x4(scissor.xform.inverse());
            frag.scissor_ext[0] = scissor.extent.width;
            frag.scissor_ext[1] = scissor.extent.height;
            frag.scissor_scale[0] = (scissor.xform.0[0] * scissor.xform.0[0]
                + scissor.xform.0[2] * scissor.xform.0[2])
                .sqrt()
                / fringe;
            frag.scissor_scale[1] = (scissor.xform.0[1] * scissor.xform.0[1]
                + scissor.xform.0[3] * scissor.xform.0[3])
                .sqrt()
                / fringe;
        }

        frag.extent = [paint.extent.width, paint.extent.height];
        frag.stroke_mult = (width * 0.5 + fringe * 0.5) / fringe;

        let mut invxform = Transform::default();

        if let Some(img) = paint.image {
            if let Some(texture) = self.textures.get(img) {
                if texture.flags.contains(ImageFlags::FLIPY) {
                    let m1 = Transform::translate(0.0, frag.extent[1] * 0.5) * paint.xform;
                    let m2 = Transform::scale(1.0, -1.0) * m1;
                    let m1 = Transform::translate(0.0, -frag.extent[1] * 0.5) * m2;
                    invxform = m1.inverse();
                } else {
                    invxform = paint.xform.inverse();
                };

                frag.type_ = ShaderType::FillImage as i32;
                match texture.texture_type {
                    TextureType::RGBA | TextureType::BGRA => {
                        frag.tex_type = if texture.flags.contains(ImageFlags::PREMULTIPLIED) {
                            0
                        } else {
                            1
                        }
                    }
                    TextureType::Alpha => frag.tex_type = 2,
                }
            }
        } else {
            frag.type_ = ShaderType::FillGradient as i32;
            frag.radius = paint.radius;
            frag.feather = paint.feather;
            invxform = paint.xform.inverse();
        }

        frag.paint_mat = xform_to_3x4(invxform);

        frag
    }

    fn append_uniforms(&mut self, uniforms: FragUniforms) {
        self.uniforms
            .resize(self.uniforms.len() + self.frag_size, 0);
        unsafe {
            let idx = self.uniforms.len() - self.frag_size;
            let p = self.uniforms.as_mut_ptr().add(idx) as *mut FragUniforms;
            *p = uniforms;
        }
    }

    #[inline]
    fn get_uniform_offset(&self) -> usize {
        return self.uniforms.len() / self.frag_size;
    }
}

fn shader_error(shader: gl::types::GLuint, filename: &str) -> anyhow::Error {
    unsafe {
        let mut data: [gl::types::GLchar; 512 + 1] = std::mem::zeroed();
        let mut len: gl::types::GLsizei = std::mem::zeroed();
        gl::GetShaderInfoLog(shader, 512, &mut len, data.as_mut_ptr());
        if len > 512 {
            len = 512;
        }
        data[len as usize] = 0;
        let err_msg = std::ffi::CStr::from_ptr(data.as_ptr());
        anyhow!(
            "failed to compile shader: {}: {}",
            filename,
            err_msg.to_string_lossy()
        )
    }
}

fn program_error(prog: gl::types::GLuint) -> anyhow::Error {
    unsafe {
        let mut data: [gl::types::GLchar; 512 + 1] = std::mem::zeroed();
        let mut len: gl::types::GLsizei = std::mem::zeroed();
        gl::GetProgramInfoLog(prog, 512, &mut len, data.as_mut_ptr());
        if len > 512 {
            len = 512;
        }
        data[len as usize] = 0;
        let err_msg = std::ffi::CStr::from_ptr(data.as_ptr());
        anyhow!("failed to link program: {}", err_msg.to_string_lossy())
    }
}

fn convert_blend_factor(factor: BlendFactor) -> gl::types::GLenum {
    match factor {
        BlendFactor::Zero => gl::ZERO,
        BlendFactor::One => gl::ONE,
        BlendFactor::SrcColor => gl::SRC_COLOR,
        BlendFactor::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
        BlendFactor::DstColor => gl::DST_COLOR,
        BlendFactor::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
        BlendFactor::SrcAlpha => gl::SRC_ALPHA,
        BlendFactor::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
        BlendFactor::DstAlpha => gl::DST_ALPHA,
        BlendFactor::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
        BlendFactor::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE,
    }
}
