use bytemuck::NoUninit;
use wgpu::{util::DeviceExt, Device};

enum ShaderType {
    FillGradient,
    FillImage,
    Simple,
    Image,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RenderUnifrom {
    pub scissor_mat: [[f32; 3]; 3],
    pub paint_mat: [[f32; 3]; 3],
    pub inner_color: [f32; 4],
    pub outer_color: [f32; 4],
    pub scissor_ext: [f32; 2],
    pub scissor_scale: [f32; 2],
    pub extend: [f32; 2],
    pub radius: f32,
    pub feather: f32,
    pub stroke_mult: f32,
    pub stroke_thr: f32,
    pub texture_type: u32,
    pub render_type: u32,
}


pub(crate) struct Unifrom<T: Default + Copy + NoUninit> {
    pub value: T,
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl<T: Default + Copy + NoUninit> Unifrom<T> {
    pub fn new(
        device: &Device,
        binding: u32,
        visibility: wgpu::ShaderStages,
        dyn_offset: bool,
    ) -> Self {
        let default_value: T = Default::default();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[default_value]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
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
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        return Self {
            value: default_value,
            buffer: uniform_buffer,
            layout: bind_group_layout,
            bind_group,
        };
    }
}

