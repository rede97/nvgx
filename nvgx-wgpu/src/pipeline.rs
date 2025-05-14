use nvgx::{CompositeOperationState, PathFillType};

use super::{call::ToBlendState, instance::INSTANCE_DESC, mesh::VERTEX_DESC};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PipelineUsage {
    FillStencil(PathFillType),
    FillStroke(CompositeOperationState),
    FillInner(CompositeOperationState),
    FillConvex(CompositeOperationState),
    Triangles(CompositeOperationState),
    Lines(CompositeOperationState),
}

impl PipelineUsage {
    #[inline]
    fn fragment_target(&self, format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
        match self {
            PipelineUsage::FillStencil(_) => wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::empty(),
            },
            PipelineUsage::FillStroke(blend)
            | PipelineUsage::FillInner(blend)
            | PipelineUsage::FillConvex(blend)
            | PipelineUsage::Triangles(blend)
            | PipelineUsage::Lines(blend) => wgpu::ColorTargetState {
                format,
                blend: Some(blend.to_wgpu_blend_state()),
                write_mask: wgpu::ColorWrites::ALL,
            },
        }
    }

    #[inline]
    fn primitive(&self) -> wgpu::PrimitiveState {
        match self {
            PipelineUsage::FillStencil(_)
            | PipelineUsage::FillConvex(_)
            | PipelineUsage::Triangles(_) => wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            PipelineUsage::FillStroke(_) | PipelineUsage::FillInner(_) => wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            PipelineUsage::Lines(_) => wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Line,
                conservative: false,
            },
        }
    }

    #[inline]
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
            PipelineUsage::FillStroke(_) | PipelineUsage::FillConvex(_) => wgpu::StencilState {
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
            PipelineUsage::FillInner(_) => wgpu::StencilState {
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
            PipelineUsage::Triangles(_) | PipelineUsage::Lines(_) => wgpu::StencilState {
                front: wgpu::StencilFaceState::IGNORE,
                back: wgpu::StencilFaceState::IGNORE,
                read_mask: 0xff,
                write_mask: 0xff,
            },
        }
    }
}

impl PipelineUsage {
    #[inline]
    fn make_pipeline(
        &self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
        fmt: &wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NVG Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[VERTEX_DESC, INSTANCE_DESC],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(self.fragment_target(*fmt))],
            }),
            primitive: self.primitive(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: self.stencil_state(),
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
    usage: PipelineUsage,
}

impl Pipeline {
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        return &self.pipeline;
    }
}

struct PipelineBuilder {
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    cache: indexmap::IndexMap<PipelineUsage, wgpu::RenderPipeline>,
}

impl PipelineBuilder {
    fn new(shader: wgpu::ShaderModule, layout: wgpu::PipelineLayout) -> PipelineBuilder {
        return Self {
            shader,
            layout,
            cache: indexmap::IndexMap::new(),
        };
    }

    fn create(
        &self,
        device: &wgpu::Device,
        usage: PipelineUsage,
        fmt: &wgpu::TextureFormat,
    ) -> Pipeline {
        return Pipeline {
            pipeline: usage.make_pipeline(device, &self.shader, &self.layout, fmt),
            usage,
        };
    }

    /// Recycle pipeline and find or create a new pipeline
    #[inline]
    fn update_pipeline(
        &mut self,
        new_usage: PipelineUsage,
        device: &wgpu::Device,
        pipeline: &mut Pipeline,
        fmt: &wgpu::TextureFormat,
    ) {
        let new_pipeline = if let Some(pipeline) = self.cache.shift_remove(&new_usage) {
            Pipeline {
                pipeline,
                usage: new_usage,
            }
        } else {
            self.create(device, new_usage, fmt)
        };
        let old_pipeline = std::mem::replace(pipeline, new_pipeline);
        self.cache.insert(old_pipeline.usage, old_pipeline.pipeline);
    }
}

pub struct PipelineManager {
    builder: PipelineBuilder,
    pub fill_stencil: Pipeline,
    pub fill_stroke: Pipeline,
    pub fill_inner: Pipeline,
    pub fill_convex: Pipeline,
    pub triangles: Pipeline,
    pub wirelines: Pipeline,
}

impl PipelineManager {
    pub fn new(
        shader: wgpu::ShaderModule,
        pipeline_layout: wgpu::PipelineLayout,
        device: &wgpu::Device,
        fmt: &wgpu::TextureFormat,
    ) -> Self {
        let builder = PipelineBuilder::new(shader, pipeline_layout);
        let default_blend = nvgx::CompositeOperationState::default();
        let fill_stencil = builder.create(
            &device,
            PipelineUsage::FillStencil(PathFillType::Winding),
            fmt,
        );
        let fill_stroke = builder.create(
            &device,
            PipelineUsage::FillStroke(default_blend.clone()),
            fmt,
        );
        let fill_inner = builder.create(&device, PipelineUsage::FillInner(default_blend), fmt);
        let fill_convex = builder.create(&device, PipelineUsage::FillConvex(default_blend), fmt);
        let triangles = builder.create(&device, PipelineUsage::Triangles(default_blend), fmt);
        let wirelines = builder.create(&device, PipelineUsage::Lines(default_blend), fmt);
        return Self {
            builder,
            fill_stencil,
            fill_stroke,
            fill_inner,
            fill_convex,
            triangles,
            wirelines,
        };
    }

    #[inline]
    pub fn update_pipeline(
        &mut self,
        device: &wgpu::Device,
        usage: PipelineUsage,
        fmt: &wgpu::TextureFormat,
    ) {
        match &usage {
            PipelineUsage::FillStencil(_) => {
                if self.fill_stencil.usage != usage {
                    self.builder
                        .update_pipeline(usage, device, &mut self.fill_stencil, fmt);
                }
            }
            PipelineUsage::FillStroke(_) => {
                if self.fill_stroke.usage != usage {
                    self.builder
                        .update_pipeline(usage, device, &mut self.fill_stroke, fmt);
                }
            }
            PipelineUsage::FillInner(_) => {
                if self.fill_inner.usage != usage {
                    self.builder
                        .update_pipeline(usage, device, &mut self.fill_inner, fmt);
                }
            }
            PipelineUsage::FillConvex(_) => {
                if self.fill_convex.usage != usage {
                    self.builder
                        .update_pipeline(usage, device, &mut self.fill_convex, fmt);
                }
            }
            PipelineUsage::Triangles(_) => {
                if self.triangles.usage != usage {
                    self.builder
                        .update_pipeline(usage, device, &mut self.triangles, fmt);
                }
            }
            PipelineUsage::Lines(_) => {
                if self.wirelines.usage != usage {
                    self.builder
                        .update_pipeline(usage, device, &mut self.wirelines, fmt);
                }
            }
        }
    }
}
