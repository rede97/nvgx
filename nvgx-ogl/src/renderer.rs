use std::cell::Cell;
use std::ffi::c_void;
use std::ops::Range;
use std::sync::Arc;

use super::{Call, CallType, FragUniforms, GLPath, ShaderType, Texture};
use super::{GLSlice, Renderer};
use nvgx::*;

#[derive(Default)]
pub struct GLArrayBuffer {
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
    attached_inst: Cell<(gl::types::GLuint, i32)>,
}

impl GLArrayBuffer {
    pub(crate) fn new(usage: BufferUsage) -> Self {
        unsafe {
            let mut vbo: gl::types::GLuint = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            let mut vao: gl::types::GLuint = std::mem::zeroed();
            if usage == BufferUsage::Vertex {
                gl::GenVertexArrays(1, &mut vao);
                Self::set_vertex_binding(vao, vbo);
            }
            return Self {
                vbo,
                vao,
                attached_inst: Cell::new(std::mem::zeroed()),
            };
        }
    }

    pub(crate) fn set_vertex_binding(vao: gl::types::GLuint, vertex_vbo: gl::types::GLuint) {
        unsafe {
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_vbo);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                std::ptr::null(),
            );
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                (2 * std::mem::size_of::<f32>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        }
    }

    pub(crate) fn set_inst_binding(&self, inst_vbo: gl::types::GLuint, offset: i32) {
        if self.attached_inst.get() == (inst_vbo, offset) {
            unsafe {
                gl::BindVertexArray(self.vao);
            }
            return;
        }
        unsafe {
            self.attached_inst.set((inst_vbo, offset));
            let elem_size = std::mem::size_of::<f32>();
            let stride = elem_size as i32 * 6;
            let offset_ptr = offset * stride;
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, inst_vbo);
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_ptr as *const c_void,
            );
            gl::VertexAttribPointer(
                3,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (4 * elem_size + offset_ptr as usize) as *const c_void,
            );
            gl::EnableVertexAttribArray(2);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribDivisor(2, 1);
            gl::VertexAttribDivisor(3, 1);
        }
    }

    pub fn update_data(&self, dat: &[u8]) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (dat.len()) as isize,
                dat.as_ptr() as *const c_void,
                gl::STREAM_DRAW,
            );
        }
    }
}

impl Drop for GLArrayBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl nvgx::RendererDevice for Renderer {
    type VertexBuffer = Arc<GLArrayBuffer>;

    fn edge_antialias(&self) -> bool {
        self.config.antialias
    }

    fn resize(&mut self, _width: u32, _height: u32) -> anyhow::Result<()> {
        Ok(())
    }

    fn create_vertex_buffer(
        &mut self,
        _buffer_size: usize,
        usage: BufferUsage,
    ) -> anyhow::Result<Self::VertexBuffer> {
        return Ok(Arc::new(GLArrayBuffer::new(usage)));
    }

    fn update_vertex_buffer(
        &mut self,
        buffer: Option<&Self::VertexBuffer>,
        dat: &[u8],
    ) -> anyhow::Result<()> {
        if let Some(buffer) = buffer {
            buffer.update_data(dat);
        } else {
            self.vert_buf.update_data(dat);
        }
        Ok(())
    }

    fn create_texture(
        &mut self,
        texture_type: TextureType,
        width: u32,
        height: u32,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<ImageId> {
        let tex = unsafe {
            let mut tex: gl::types::GLuint = std::mem::zeroed();
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            match texture_type {
                TextureType::RGBA => {
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA as i32,
                        width as i32,
                        height as i32,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        match data {
                            Some(data) => data.as_ptr() as *const c_void,
                            None => std::ptr::null(),
                        },
                    );
                }
                TextureType::Alpha => {
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::R8 as i32,
                        width as i32,
                        height as i32,
                        0,
                        gl::RED,
                        gl::UNSIGNED_BYTE,
                        match data {
                            Some(data) => data.as_ptr() as *const c_void,
                            None => std::ptr::null(),
                        },
                    );
                }
            }

            if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
                if flags.contains(ImageFlags::NEAREST) {
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_MIN_FILTER,
                        gl::NEAREST_MIPMAP_NEAREST as i32,
                    );
                } else {
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_MIN_FILTER,
                        gl::LINEAR_MIPMAP_LINEAR as i32,
                    );
                }
            } else {
                if flags.contains(ImageFlags::NEAREST) {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                } else {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                }
            }

            if flags.contains(ImageFlags::NEAREST) {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            } else {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }

            if flags.contains(ImageFlags::REPEATX) {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            } else {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            }

            if flags.contains(ImageFlags::REPEATY) {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            } else {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            }

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);

            if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);
            tex
        };

        let id = self.textures.insert(Texture {
            tex,
            width,
            height,
            texture_type,
            flags,
        });
        Ok(id)
    }

    fn delete_texture(&mut self, img: ImageId) -> anyhow::Result<()> {
        if let Some(texture) = self.textures.get(img) {
            unsafe { gl::DeleteTextures(1, &texture.tex) }
            self.textures.remove(img);
            Ok(())
        } else {
            bail!("texture '{}' not found", img);
        }
    }

    fn update_texture(
        &mut self,
        img: ImageId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> anyhow::Result<()> {
        if let Some(texture) = self.textures.get(img) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, texture.tex);
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

                match texture.texture_type {
                    TextureType::RGBA => gl::TexSubImage2D(
                        gl::TEXTURE_2D,
                        0,
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        data.as_ptr() as *const c_void,
                    ),
                    TextureType::Alpha => gl::TexSubImage2D(
                        gl::TEXTURE_2D,
                        0,
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                        gl::RED,
                        gl::UNSIGNED_BYTE,
                        data.as_ptr() as *const c_void,
                    ),
                }

                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }
            Ok(())
        } else {
            bail!("texture '{}' not found", img);
        }
    }

    fn texture_size(&self, img: ImageId) -> anyhow::Result<(u32, u32)> {
        if let Some(texture) = self.textures.get(img) {
            Ok((texture.width, texture.height))
        } else {
            bail!("texture '{}' not found", img);
        }
    }

    fn viewport(&mut self, extent: Extent, _device_pixel_ratio: f32) -> anyhow::Result<()> {
        self.view = extent;
        Ok(())
    }

    fn cancel(&mut self) -> anyhow::Result<()> {
        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        if !self.calls.is_empty() {
            unsafe {
                gl::UseProgram(self.shader.prog);

                gl::Enable(gl::CULL_FACE);
                gl::CullFace(gl::BACK);
                gl::FrontFace(gl::CCW);
                gl::Enable(gl::BLEND);
                gl::Disable(gl::DEPTH_TEST);
                gl::Disable(gl::SCISSOR_TEST);
                gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                gl::StencilMask(0xffffffff);
                gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
                gl::StencilFunc(gl::ALWAYS, 0, 0xffffffff);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, 0);

                gl::BindBuffer(gl::UNIFORM_BUFFER, self.frag_buf);
                gl::BufferData(
                    gl::UNIFORM_BUFFER,
                    self.uniforms.len() as isize,
                    self.uniforms.as_ptr() as *const c_void,
                    gl::STREAM_DRAW,
                );

                gl::Uniform1i(self.shader.loc_tex, 0);
                gl::Uniform2fv(
                    self.shader.loc_viewsize,
                    1,
                    &self.view as *const Extent as *const f32,
                );

                gl::BindBuffer(gl::UNIFORM_BUFFER, self.frag_buf);

                for call in &self.calls {
                    let (inst_vbo, inst_slice) =
                        call.instances.as_ref().map(|v| (v.0.vbo, v.1)).unwrap_or((
                            self.inst_buf.vbo,
                            GLSlice {
                                offset: 0,
                                count: 1,
                            },
                        ));

                    let vertex_buffer = call
                        .vert_buff
                        .as_ref()
                        .map(|v| v.as_ref())
                        .unwrap_or(&self.vert_buf);
                    vertex_buffer.set_inst_binding(inst_vbo, inst_slice.offset as i32);

                    let blend = &call.blend_func;

                    gl::BlendFuncSeparate(
                        blend.src_rgb,
                        blend.dst_rgb,
                        blend.src_alpha,
                        blend.dst_alpha,
                    );

                    let inst_count = inst_slice.count as i32;
                    match call.call_type {
                        CallType::Fill(ft) => self.do_fill(&call, ft, inst_count),
                        CallType::ConvexFill => self.do_convex_fill(&call, inst_count),
                        CallType::Stroke => self.do_stroke(&call, inst_count),
                        CallType::Triangles => self.do_triangles(&call, inst_count),
                        #[cfg(feature = "wirelines")]
                        CallType::Lines => self.do_lines(&call, inst_count),
                    }
                }

                gl::BindVertexArray(0);
                gl::Disable(gl::CULL_FACE);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::UseProgram(0);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }
        }

        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
        Ok(())
    }

    fn fill(
        &mut self,
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        fill_type: nvgx::PathFillType,
        scissor: &nvgx::Scissor,
        fringe: f32,
        bounds_offset: Option<usize>,
        paths: &[nvgx::PathSlice],
    ) -> anyhow::Result<()> {
        let path_offset = self.paths.len();

        for path in paths {
            let fill = path.get_fill();
            if fill.count < 3 {
                continue;
            }

            let stroke = path.get_stroke();

            let gl_path = GLPath {
                fill: fill.into(),
                stroke: stroke.into(),
            };

            self.paths.push(gl_path);
        }

        let mut call = Call {
            call_type: if bounds_offset.is_some() {
                CallType::Fill(fill_type)
            } else {
                CallType::ConvexFill
            },
            image: paint.image,
            path_range: path_offset..self.paths.len(),
            triangle: GLSlice {
                offset: 0,
                count: 4,
            },
            uniform_offset: self.get_uniform_offset(),
            blend_func: composite_operation.into(),
            vert_buff: vertex_buffer,
            instances: instances.map(|(insts, r)| (insts, r.into())),
        };

        if let Some(offset) = bounds_offset {
            call.triangle.offset = offset as u32;
            self.append_uniforms(FragUniforms {
                stroke_thr: -1.0,
                type_: ShaderType::Simple as i32,
                ..FragUniforms::default()
            });
            self.append_uniforms(self.convert_paint(paint, scissor, fringe, fringe, -1.0));
        } else {
            self.append_uniforms(self.convert_paint(paint, scissor, fringe, fringe, -1.0));
        }

        self.calls.push(call);
        Ok(())
    }

    fn stroke(
        &mut self,
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        scissor: &nvgx::Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[nvgx::PathSlice],
    ) -> anyhow::Result<()> {
        let path_offset = self.paths.len();

        for path in paths {
            let stroke = path.get_stroke();
            let gl_path = GLPath {
                stroke: stroke.into(),
                ..Default::default()
            };
            self.paths.push(gl_path);
        }
        let call = Call {
            call_type: CallType::Stroke,
            image: paint.image,
            path_range: path_offset..self.paths.len(),
            triangle: Default::default(),
            uniform_offset: self.get_uniform_offset(),
            blend_func: composite_operation.into(),
            vert_buff: vertex_buffer,
            instances: instances.map(|(insts, r)| (insts, r.into())),
        };

        self.append_uniforms(self.convert_paint(paint, scissor, stroke_width, fringe, -1.0));
        self.calls.push(call);
        Ok(())
    }

    fn triangles(
        &mut self,
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        scissor: &nvgx::Scissor,
        slice: VertexSlice,
    ) -> anyhow::Result<()> {
        let call = Call {
            call_type: CallType::Triangles,
            image: paint.image,
            path_range: 0..0,
            triangle: slice.into(),
            uniform_offset: self.get_uniform_offset(),
            blend_func: composite_operation.into(),
            vert_buff: vertex_buffer,
            instances: instances.map(|(insts, r)| (insts, r.into())),
        };
        self.calls.push(call);

        let mut uniforms = self.convert_paint(paint, scissor, 1.0, 1.0, -1.0);
        uniforms.type_ = ShaderType::Image as i32;
        self.append_uniforms(uniforms);
        Ok(())
    }

    #[cfg(feature = "wirelines")]
    fn wirelines(
        &mut self,
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        scissor: &nvgx::Scissor,
        paths: &[nvgx::PathSlice],
    ) -> anyhow::Result<()> {
        let path_offset = self.paths.len();

        for path in paths {
            let line = path.get_stroke();
            let gl_path = GLPath {
                stroke: line.into(),
                ..Default::default()
            };
            self.paths.push(gl_path);
        }
        let call = Call {
            call_type: CallType::Lines,
            image: paint.image,
            path_range: path_offset..self.paths.len(),
            triangle: Default::default(),
            uniform_offset: self.get_uniform_offset(),
            blend_func: composite_operation.into(),
            vert_buff: vertex_buffer,
            instances: instances.map(|(insts, r)| (insts, r.into())),
        };

        self.append_uniforms(self.convert_paint(paint, scissor, 1.0, 1.0, -1.0));

        self.calls.push(call);
        Ok(())
    }

    fn clear(&mut self, clear_color: Color) -> anyhow::Result<()> {
        unsafe {
            gl::Viewport(0, 0, self.view.width as i32, self.view.height as i32);
            gl::ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
        Ok(())
    }
}
