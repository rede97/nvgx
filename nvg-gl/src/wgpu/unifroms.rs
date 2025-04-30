use nvg::{Color, Extent, ImageFlags, PaintPattern, Scissor, TextureType, Transform};
use wgpu::{util::DeviceExt, Device};

use crate::{premul_color, xform_to_3x4};

pub enum ShaderType {
    FillGradient,
    FillImage,
    Simple,
    Image,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderCommand {
    pub scissor_mat: [f32; 12],
    pub paint_mat: [f32; 12],
    pub inner_color: Color,
    pub outer_color: Color,
    pub scissor_ext: [f32; 2],
    pub scissor_scale: [f32; 2],
    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,
    pub stroke_mult: f32,
    pub stroke_thr: f32,
    pub texture_type: u32,
    pub render_type: u32,
}

impl RenderCommand {
    pub fn new(
        render: &crate::Renderer,
        paint: &PaintPattern,
        scissor: &Scissor,
        width: f32,
        fringe: f32,
        stroke_thr: f32,
    ) -> Self {
        let mut frag = RenderCommand {
            inner_color: premul_color(paint.inner_color),
            outer_color: premul_color(paint.outer_color),
            stroke_thr,
            ..Default::default()
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
            if let Some(texture) = render.textures.get(img) {
                if texture.image_flags.contains(ImageFlags::FLIPY) {
                    let m1 = Transform::translate(0.0, frag.extent[1] * 0.5) * paint.xform;
                    let m2 = Transform::scale(1.0, -1.0) * m1;
                    let m1 = Transform::translate(0.0, -frag.extent[1] * 0.5) * m2;
                    invxform = m1.inverse();
                } else {
                    invxform = paint.xform.inverse();
                };

                frag.render_type = ShaderType::FillImage as u32;
                frag.texture_type = match texture.texture_type() {
                    TextureType::RGBA => {
                        if texture.image_flags.contains(ImageFlags::PREMULTIPLIED) {
                            0
                        } else {
                            1
                        }
                    }
                    TextureType::Alpha => 2,
                };
            }
        } else {
            frag.render_type = ShaderType::FillGradient as u32;
            frag.radius = paint.radius;
            frag.feather = paint.feather;
            invxform = paint.xform.inverse();
        }

        frag.paint_mat = xform_to_3x4(invxform);

        frag
    }
}

pub struct Unifrom<T: WgpuUnifromContent> {
    pub value: T,
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl<T: WgpuUnifromContent> Unifrom<T> {
    pub fn new(
        device: &Device,
        binding: u32,
        visibility: wgpu::ShaderStages,
        dyn_offset: bool,
    ) -> Self {
        // let elem_size = T::
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: vec![0; T::elem_size()].as_slice(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: dyn_offset,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Unifrom Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        return Self {
            value: Default::default(),
            buffer: uniform_buffer,
            layout: bind_group_layout,
            bind_group,
        };
    }

    pub fn update_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, self.value.as_contents());
    }
}

pub trait WgpuUnifromContent: Default {
    fn elem_size() -> usize;
    fn as_contents(&self) -> &[u8];
}

impl WgpuUnifromContent for Vec<RenderCommand> {
    fn elem_size() -> usize {
        return size_of::<RenderCommand>();
    }

    fn as_contents(&self) -> &[u8] {
        return bytemuck::cast_slice(self.as_slice());
    }
}

impl WgpuUnifromContent for Extent {
    fn elem_size() -> usize {
        return size_of::<Extent>();
    }

    fn as_contents(&self) -> &[u8] {
        return bytemuck::bytes_of(self);
    }
}
