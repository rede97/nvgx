use std::num::NonZero;

use nvgx::{utils::{premul_color, xform_to_3x4}, Color, Extent, ImageFlags, PaintPattern, Scissor, TextureType, Transform};
use wgpu::Device;


pub enum ShaderType {
    FillGradient,
    FillImage,
    Simple,
    Image,
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
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
    pub _padding: AlignPadding<80>,
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
            if let Some(texture) = render.resources.texture_manager.textures.get(img) {
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
                    TextureType::RGBA | TextureType::BGRA => {
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

    pub fn set_type(mut self, render_type: ShaderType) -> Self {
        self.render_type = render_type as u32;
        self
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexCommand {
    pub viewsize: Extent,
    pub transfrom_mat: [f32; 12],
}

pub struct Unifrom<T: WgpuUnifromContent> {
    pub value: T,
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub binding: u32,
}

impl<T: WgpuUnifromContent> Unifrom<T> {
    pub fn new(
        device: &Device,
        binding: u32,
        visibility: wgpu::ShaderStages,
        dyn_offset_with_init_size: Option<usize>,
    ) -> Self {
        // let elem_size = T::
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("NVG Uniform Buffer"),
            size: (T::elem_size() * dyn_offset_with_init_size.unwrap_or(1)) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("NVG Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: dyn_offset_with_init_size.is_some(),
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("NVG Unifrom Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: NonZero::new(T::elem_size() as u64),
                }),
            }],
        });

        return Self {
            value: Default::default(),
            buffer: uniform_buffer,
            layout: bind_group_layout,
            bind_group,
            binding,
        };
    }

    pub fn update_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let content = self.value.as_contents();
        if self.buffer.size() < content.len() as u64 {
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("NVG Expand Uniform Buffer"),
                size: (content.len() * 2) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("NVg Expand Unifrom Bind Group"),
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: self.binding,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.buffer,
                        offset: 0,
                        size: NonZero::new(T::elem_size() as u64),
                    }),
                }],
            });
        }
        queue.write_buffer(&self.buffer, 0, self.value.as_contents());
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.value.offset()
    }
}

pub trait WgpuUnifromContent: Default {
    fn elem_size() -> usize;
    fn as_contents(&self) -> &[u8];
    #[inline]
    fn offset(&self) -> usize {
        0
    }
}

impl WgpuUnifromContent for Vec<RenderCommand> {
    fn elem_size() -> usize {
        return size_of::<RenderCommand>();
    }

    fn as_contents(&self) -> &[u8] {
        return bytemuck::cast_slice(self.as_slice());
    }

    fn offset(&self) -> usize {
        return self.len();
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

#[repr(transparent)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AlignPadding<const S: usize> {
    _padding: [u8; S],
}

impl<const S: usize> Default for AlignPadding<S> {
    fn default() -> Self {
        Self { _padding: [0; S] }
    }
}
