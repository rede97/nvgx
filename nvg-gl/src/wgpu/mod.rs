use call::{Call, GpuPath, ToBlendState};
use nvg::*;
use pipeline::{PipelineBuilder, PipelineConfig, Pipelines};
use slab::Slab;
use texture::{StencilTexture, Texture};
use unifroms::{RenderCommand, Unifrom};
use wgpu::{vertex_attr_array, ShaderStages};

mod call;
mod pipeline;
mod renderer;
mod texture;
mod unifroms;

struct VertexIn {
    vertex: [f32; 2],
    texcoord: [f32; 2],
}

impl VertexIn {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    viewsize_uniform: Unifrom<Extent>,
    render_unifrom: Unifrom<Vec<RenderCommand>>,
    stencil_texture: StencilTexture,
    textures: Slab<Texture>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    calls: Vec<Call>,
    paths: Vec<GpuPath>,
    vertexes: Vec<Vertex>,
    pipeline_builder: PipelineBuilder,
    pipelines: Pipelines,
}

impl Renderer {
    pub fn create(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
    ) -> anyhow::Result<Self> {
        let stencil_texture = StencilTexture::new(&device, &surface_config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let viewsize_uniform: Unifrom<Extent> =
            Unifrom::new(&device, 0, ShaderStages::VERTEX, false);
        let render_unifrom: Unifrom<Vec<RenderCommand>> =
            Unifrom::new(&device, 0, ShaderStages::FRAGMENT, true);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &viewsize_uniform.layout,
                &render_unifrom.layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let mut pipeline_builder = PipelineBuilder::new(shader, pipeline_layout);
        let pipelines = Pipelines::default(&mut pipeline_builder, &device);

        return Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            viewsize_uniform,
            render_unifrom,
            stencil_texture,
            textures: Slab::default(),
            texture_bind_group_layout,
            calls: Vec::new(),
            paths: Vec::new(),
            vertexes: Vec::new(),
            pipeline_builder,
            pipelines,
        });
    }

    pub fn do_fill(&mut self) {}
}
