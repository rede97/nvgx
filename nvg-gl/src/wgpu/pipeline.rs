use nvg::PathFillType;

use super::{call::ToBlendState, mesh::VERTEX_DESC};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PipelineUsage {
    FillStencil(PathFillType),
    FillStroke,
    FillFinal,
}

impl PipelineUsage {
    fn primitive(&self) -> wgpu::PrimitiveState {
        match self {
            PipelineUsage::FillStencil(_) => wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            PipelineUsage::FillStroke | PipelineUsage::FillFinal => wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
        }
    }

    fn stencil_state(&self) -> wgpu::StencilState {
        match self {
            PipelineUsage::FillStencil(path_fill_type) => match path_fill_type {
                PathFillType::Winding => wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::IncrementWrap,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::DecrementWrap,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                PathFillType::EvenOdd => wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Invert,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Invert,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
            },
            PipelineUsage::FillStroke => wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    pass_op: wgpu::StencilOperation::Keep,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState::IGNORE,
                read_mask: 0xff,
                write_mask: 0xff,
            },
            PipelineUsage::FillFinal => wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    pass_op: wgpu::StencilOperation::Zero,
                    fail_op: wgpu::StencilOperation::Zero,
                    depth_fail_op: wgpu::StencilOperation::Zero,
                },
                back: wgpu::StencilFaceState::IGNORE,
                read_mask: 0xff,
                write_mask: 0xff,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipelineConfig {
    pub usage: PipelineUsage,
    pub blend: wgpu::BlendState,
}

impl PipelineConfig {
    fn make_pipeline(
        &self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[VERTEX_DESC],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(self.blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: self.usage.primitive(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: self.usage.stencil_state(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        return pipeline;
    }
}

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    config: PipelineConfig,
}

impl Pipeline {
    pub fn config(&self) -> &PipelineConfig {
        return &self.config;
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        return &self.pipeline;
    }
}

pub struct PipelineBuilder {
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    cache: indexmap::IndexMap<PipelineConfig, wgpu::RenderPipeline>,
}

impl PipelineBuilder {
    pub fn new(shader: wgpu::ShaderModule, layout: wgpu::PipelineLayout) -> PipelineBuilder {
        return Self {
            shader,
            layout,
            cache: indexmap::IndexMap::new(),
        };
    }

    pub fn create(&self, device: &wgpu::Device, config: PipelineConfig) -> Pipeline {
        return Pipeline {
            pipeline: config.make_pipeline(device, &self.shader, &self.layout),
            config,
        };
    }

    /// Recycle pipeline and find or create a new pipeline
    pub fn update_pipeline(
        &mut self,
        new_config: PipelineConfig,
        device: &wgpu::Device,
        pipeline: &mut Pipeline,
    ) {
        let new_pipeline = if let Some(pipeline) = self.cache.shift_remove(&new_config) {
            Pipeline {
                pipeline,
                config: new_config,
            }
        } else {
            self.create(device, new_config)
        };
        let old_pipeline = std::mem::replace(pipeline, new_pipeline);
        self.cache
            .insert(old_pipeline.config, old_pipeline.pipeline);
    }
}

pub struct Pipelines {
    pub fill_stencil: Pipeline,
    pub fill_final: Pipeline,
}

impl Pipelines {
    pub fn default(builder: &mut PipelineBuilder, device: &wgpu::Device) -> Self {
        let defaylrt_blend = (&nvg::CompositeOperationState::default()).to_wgpu_blend_state();
        let fill_stencil = builder.create(
            &device,
            PipelineConfig {
                usage: PipelineUsage::FillStencil(PathFillType::Winding),
                blend: defaylrt_blend,
            },
        );

        let fill_final = builder.create(
            &device,
            PipelineConfig {
                usage: PipelineUsage::FillFinal,
                blend: defaylrt_blend,
            },
        );

        return Self {
            fill_stencil,
            fill_final,
        };
    }
}
