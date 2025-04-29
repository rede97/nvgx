use call::Call;
use nvg::*;
use slab::Slab;
use texture::Texture;
use unifroms::{RenderUnifrom, Unifrom};
use wgpu::{vertex_attr_array, ShaderStages};

mod call;
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
    view: Extent,
    render_unifrom: Unifrom<RenderUnifrom>,
    viewsize_uniform: Unifrom<[f32; 2]>,
    pipeline_layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    calls: Vec<Call>,
    textures: Slab<Texture>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl Renderer {
    pub fn create(device: wgpu::Device, queue: wgpu::Queue) -> anyhow::Result<Self> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let viewsize_uniform: Unifrom<[f32; 2]> =
            Unifrom::new(&device, 0, ShaderStages::VERTEX, false);
        let render_unifrom: Unifrom<RenderUnifrom> =
            Unifrom::new(&device, 0, ShaderStages::FRAGMENT, true);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&viewsize_uniform.layout, &render_unifrom.layout],
            push_constant_ranges: &[],
        });

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

        return Ok(Self {
            device,
            queue,
            view: Extent::default(),
            viewsize_uniform,
            render_unifrom,
            pipeline_layout,
            shader,
            calls: Vec::new(),
            textures: Slab::default(),
            texture_bind_group_layout,
        });
    }

    #[inline]
    pub fn device(&self) -> &wgpu::Device {
        return &self.device;
    }

    pub fn do_fill(&mut self) {

        // let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("Render Pipeline"),
        //     layout: Some(&render_pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: Some("vs_main"),
        //         compilation_options: Default::default(),
        //         buffers: &[VertexIn::desc()],
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: Some("fs_main"),
        //         compilation_options: Default::default(),
        //         targets: &[Some(wgpu::ColorTargetState {
        //             format: wgpu::TextureFormat::Bgra8UnormSrgb,
        //             blend: Some(wgpu::BlendState {
        //                 color: wgpu::BlendComponent::OVER,
        //                 alpha: wgpu::BlendComponent::OVER,
        //             }),
        //             write_mask: wgpu::ColorWrites::ALL,
        //         })],
        //     }),
        //     primitive: wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleStrip,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         cull_mode: Some(wgpu::Face::Back),
        //         unclipped_depth: false,
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //         conservative: false,
        //     },
        //     // depth_stencil: Some(wgpu::DepthStencilState {
        //     //     format: wgpu::TextureFormat::Stencil8,
        //     //     depth_write_enabled: false,
        //     //     depth_compare: wgpu::CompareFunction::Never,
        //     //     stencil: wgpu::StencilState::default(),
        //     //     bias: wgpu::DepthBiasState::default(),
        //     // }),
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState {
        //         count: 1,
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        //     multiview: None,
        //     cache: None,
        // });
    }
}
