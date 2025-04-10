#[macro_use]
extern crate anyhow;
pub mod fb;

use nvg::{renderer::*, FillType};
use slab::Slab;

mod renderer;

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
    unsafe fn load() -> anyhow::Result<Shader> {
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

enum ShaderType {
    FillGradient,
    FillImage,
    Simple,
    Image,
}

#[derive(PartialEq, Eq)]
enum CallType {
    Fill(FillType),
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
    image: Option<usize>,
    path_offset: usize,
    path_count: usize,
    triangle_offset: usize,
    triangle_count: usize,
    uniform_offset: usize,
    blend_func: Blend,
    #[cfg(feature = "wireframe")]
    wireframe: bool,
}

struct Texture {
    tex: gl::types::GLuint,
    width: usize,
    height: usize,
    texture_type: TextureType,
    flags: ImageFlags,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.tex) }
    }
}

struct GLPath {
    fill_offset: usize,
    fill_count: usize,
    stroke_offset: usize,
    stroke_count: usize,
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

pub struct RenderConfig {
    antialias: bool,
    stencil_stroke: bool,
}

impl RenderConfig {
    pub fn antialias(mut self, antialias: bool) -> Self {
        self.antialias = antialias;
        self
    }

    pub fn stencil_stroke(mut self, stencil_stroke: bool) -> Self {
        self.stencil_stroke = stencil_stroke;
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            antialias: true,
            stencil_stroke: true,
        }
    }
}

pub struct Renderer {
    shader: Shader,
    textures: Slab<Texture>,
    view: Extent,
    vert_buf: gl::types::GLuint,
    vert_arr: gl::types::GLuint,
    frag_buf: gl::types::GLuint,
    frag_size: usize,
    calls: Vec<Call>,
    paths: Vec<GLPath>,
    vertexes: Vec<Vertex>,
    uniforms: Vec<u8>,
    config: RenderConfig,
    #[cfg(feature = "wireframe")]
    wireframe: bool,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.frag_buf);
            gl::DeleteBuffers(1, &self.vert_buf);
            gl::DeleteVertexArrays(1, &self.vert_arr);
        }
    }
}

impl Renderer {
    pub fn create(config: RenderConfig) -> anyhow::Result<Renderer> {
        unsafe {
            let shader = Shader::load()?;

            let mut vert_arr: gl::types::GLuint = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vert_arr);

            let mut vert_buf: gl::types::GLuint = std::mem::zeroed();
            gl::GenBuffers(1, &mut vert_buf);

            gl::UniformBlockBinding(shader.prog, shader.loc_frag, 0);
            let mut frag_buf: gl::types::GLuint = std::mem::zeroed();
            gl::GenBuffers(1, &mut frag_buf);

            let mut align = std::mem::zeroed();
            gl::GetIntegerv(gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT, &mut align);

            let frag_size = std::mem::size_of::<FragUniforms>() + (align as usize)
                - std::mem::size_of::<FragUniforms>() % (align as usize);

            gl::Finish();

            Ok(Renderer {
                shader,
                textures: Default::default(),
                view: Default::default(),
                vert_buf,
                vert_arr,
                frag_buf,
                frag_size,
                calls: Default::default(),
                paths: Default::default(),
                vertexes: Default::default(),
                uniforms: Default::default(),
                config,
                #[cfg(feature = "wireframe")]
                wireframe: false,
            })
        }
    }

    unsafe fn set_uniforms(&self, offset: usize, img: Option<usize>) {
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

    #[inline]
    unsafe fn do_fill(&self, call: &Call, fill_type: FillType) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];

        gl::Enable(gl::STENCIL_TEST);
        gl::StencilMask(0xff);
        gl::StencilFunc(gl::ALWAYS, 0, 0xff);
        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);

        self.set_uniforms(call.uniform_offset, call.image);
        if fill_type == FillType::Winding {
            gl::StencilOpSeparate(gl::FRONT, gl::KEEP, gl::KEEP, gl::INCR_WRAP);
            gl::StencilOpSeparate(gl::BACK, gl::KEEP, gl::KEEP, gl::DECR_WRAP);
        } else {
            gl::StencilOpSeparate(gl::FRONT_AND_BACK, gl::KEEP, gl::KEEP, gl::INVERT);
        }
        gl::Disable(gl::CULL_FACE);
        for path in paths {
            gl::DrawArrays(
                gl::TRIANGLE_FAN,
                path.fill_offset as i32,
                path.fill_count as i32,
            );
        }
        gl::Enable(gl::CULL_FACE);

        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);

        self.set_uniforms(call.uniform_offset + 1, call.image);

        gl::StencilFunc(gl::EQUAL, 0x00, 0xff);
        gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
        for path in paths {
            gl::DrawArrays(
                gl::TRIANGLE_STRIP,
                path.stroke_offset as i32,
                path.stroke_count as i32,
            );
        }

        gl::StencilFunc(gl::NOTEQUAL, 0x00, 0xff);
        gl::StencilOp(gl::ZERO, gl::ZERO, gl::ZERO);
        gl::DrawArrays(
            gl::TRIANGLE_STRIP,
            call.triangle_offset as i32,
            call.triangle_count as i32,
        );

        gl::Disable(gl::STENCIL_TEST);
    }

    #[inline]
    unsafe fn do_convex_fill(&self, call: &Call) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        self.set_uniforms(call.uniform_offset, call.image);
        for path in paths {
            gl::DrawArrays(
                gl::TRIANGLE_FAN,
                path.fill_offset as i32,
                path.fill_count as i32,
            );
            if path.stroke_count > 0 {
                gl::DrawArrays(
                    gl::TRIANGLE_STRIP,
                    path.stroke_offset as i32,
                    path.stroke_count as i32,
                );
            }
        }
    }

    #[inline]
    unsafe fn do_stroke(&self, call: &Call) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        if self.config.stencil_stroke {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilMask(0xff);
            gl::StencilFunc(gl::EQUAL, 0x0, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::INCR);
            self.set_uniforms(call.uniform_offset + 1, call.image);
            for path in paths {
                gl::DrawArrays(
                    gl::TRIANGLE_STRIP,
                    path.stroke_offset as i32,
                    path.stroke_count as i32,
                );
            }

            self.set_uniforms(call.uniform_offset, call.image);
            gl::StencilFunc(gl::EQUAL, 0x0, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
            for path in paths {
                gl::DrawArrays(
                    gl::TRIANGLE_STRIP,
                    path.stroke_offset as i32,
                    path.stroke_count as i32,
                );
            }

            gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
            gl::StencilFunc(gl::ALWAYS, 0x0, 0xff);
            gl::StencilOp(gl::ZERO, gl::ZERO, gl::ZERO);
            for path in paths {
                gl::DrawArrays(
                    gl::TRIANGLE_STRIP,
                    path.stroke_offset as i32,
                    path.stroke_count as i32,
                );
            }
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);

            gl::Disable(gl::STENCIL_TEST);
        } else {
            self.set_uniforms(call.uniform_offset, call.image);
            for path in paths {
                gl::DrawArrays(
                    gl::TRIANGLE_STRIP,
                    path.stroke_offset as i32,
                    path.stroke_count as i32,
                );
            }
        }
    }

    #[inline]
    unsafe fn do_triangles(&self, call: &Call) {
        self.set_uniforms(call.uniform_offset, call.image);
        gl::DrawArrays(
            gl::TRIANGLES,
            call.triangle_offset as i32,
            call.triangle_count as i32,
        );
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    unsafe fn do_lines(&self, call: &Call) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        self.set_uniforms(call.uniform_offset, call.image);
        for path in paths {
            gl::DrawArrays(
                gl::LINE_STRIP,
                path.stroke_offset as i32,
                path.stroke_count as i32,
            );
        }
    }

    #[cfg(feature = "wireframe")]
    #[inline]
    unsafe fn do_wireframe(&self, call: &Call) {
        match call.call_type {
            CallType::Fill(_) => {
                self.set_uniforms(call.uniform_offset + 1, call.image);
                gl::Disable(gl::CULL_FACE);
                let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
                for path in paths {
                    gl::DrawArrays(
                        gl::TRIANGLE_FAN,
                        path.fill_offset as i32,
                        path.fill_count as i32,
                    );
                }
                gl::Enable(gl::CULL_FACE);
            }
            CallType::ConvexFill => {
                let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
                self.set_uniforms(call.uniform_offset, call.image);
                for path in paths {
                    gl::DrawArrays(
                        gl::TRIANGLE_FAN,
                        path.fill_offset as i32,
                        path.fill_count as i32,
                    );
                }
            }
            CallType::Stroke => {
                let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
                self.set_uniforms(call.uniform_offset, call.image);
                for path in paths {
                    gl::DrawArrays(
                        gl::TRIANGLE_STRIP,
                        path.stroke_offset as i32,
                        path.stroke_count as i32,
                    );
                }
            }
            CallType::Triangles => self.do_triangles(call),
            #[cfg(feature = "wirelines")]
            CallType::Lines => self.do_lines(call),
        }
    }

    fn convert_paint(
        &self,
        paint: &Paint,
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
                    TextureType::RGBA => {
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

#[inline]
fn premul_color(color: Color) -> Color {
    Color {
        r: color.r * color.a,
        g: color.g * color.a,
        b: color.b * color.a,
        a: color.a,
    }
}

#[inline]
fn xform_to_3x4(xform: Transform) -> [f32; 12] {
    let mut m = [0f32; 12];
    let t = &xform.0;
    m[0] = t[0];
    m[1] = t[1];
    m[2] = 0.0;
    m[3] = 0.0;
    m[4] = t[2];
    m[5] = t[3];
    m[6] = 0.0;
    m[7] = 0.0;
    m[8] = t[4];
    m[9] = t[5];
    m[10] = 1.0;
    m[11] = 0.0;
    m
}
