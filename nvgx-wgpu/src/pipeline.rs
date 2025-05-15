use nvgx::{BlendFactor, CompositeOperationState, PathFillType};

use crate::texture::texture_type_map;

use super::{instance::INSTANCE_DESC, mesh::VERTEX_DESC};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PipelineUsage {
    FillStencil(PathFillType),
    FillStroke(CompositeOperationState),
    FillInner(CompositeOperationState),
    FillConvex(CompositeOperationState),
    Triangles(CompositeOperationState),
    Lines(CompositeOperationState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipelineConfig {
    pub format: nvgx::TextureType,
    pub usage: PipelineUsage,
}

impl PipelineConfig {
    fn convert_blend_factor(factor: BlendFactor) -> wgpu::BlendFactor {
        match factor {
            BlendFactor::Zero => wgpu::BlendFactor::Zero,
            BlendFactor::One => wgpu::BlendFactor::One,
            BlendFactor::SrcColor => wgpu::BlendFactor::Src,
            BlendFactor::OneMinusSrcColor => wgpu::BlendFactor::OneMinusSrc,
            BlendFactor::DstColor => wgpu::BlendFactor::Dst,
            BlendFactor::OneMinusDstColor => wgpu::BlendFactor::OneMinusDst,
            BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
            BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
            BlendFactor::SrcAlphaSaturate => wgpu::BlendFactor::SrcAlphaSaturated,
        }
    }

    fn to_wgpu_blend_state(composite: &CompositeOperationState) -> wgpu::BlendState {
        return wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: Self::convert_blend_factor(composite.src_rgb),
                dst_factor: Self::convert_blend_factor(composite.dst_rgb),
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: Self::convert_blend_factor(composite.src_alpha),
                dst_factor: Self::convert_blend_factor(composite.dst_alpha),
                operation: wgpu::BlendOperation::Add,
            },
        };
    }

    #[inline]
    fn get_color_format(&self) -> wgpu::TextureFormat {
        assert_ne!(self.format, nvgx::TextureType::Alpha);
        texture_type_map(self.format)
    }

    #[inline]
    fn fragment_target(&self) -> wgpu::ColorTargetState {
        match &self.usage {
            PipelineUsage::FillStencil(_) => wgpu::ColorTargetState {
                format: self.get_color_format(),
                blend: None,
                write_mask: wgpu::ColorWrites::empty(),
            },
            PipelineUsage::FillStroke(blend)
            | PipelineUsage::FillInner(blend)
            | PipelineUsage::FillConvex(blend)
            | PipelineUsage::Triangles(blend)
            | PipelineUsage::Lines(blend) => wgpu::ColorTargetState {
                format: self.get_color_format(),
                blend: Some(Self::to_wgpu_blend_state(blend)),
                write_mask: wgpu::ColorWrites::ALL,
            },
        }
    }

    #[inline]
    fn primitive(&self) -> wgpu::PrimitiveState {
        match &self.usage {
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
        match &self.usage {
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

    #[inline]
    fn make_pipeline(
        &self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
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
                targets: &[Some(self.fragment_target())],
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
    config: PipelineConfig,
}

impl Pipeline {
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        return &self.pipeline;
    }
}

struct PipelineBuilder {
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    cache: indexmap::IndexMap<PipelineConfig, wgpu::RenderPipeline>,
}

impl PipelineBuilder {
    fn new(shader: wgpu::ShaderModule, layout: wgpu::PipelineLayout) -> PipelineBuilder {
        return Self {
            shader,
            layout,
            cache: indexmap::IndexMap::new(),
        };
    }

    fn create(&self, device: &wgpu::Device, config: PipelineConfig) -> Pipeline {
        return Pipeline {
            pipeline: config.make_pipeline(device, &self.shader, &self.layout),
            config,
        };
    }

    /// Recycle pipeline and find or create a new pipeline
    #[inline]
    fn update_pipeline(
        &mut self,
        config: PipelineConfig,
        device: &wgpu::Device,
        pipeline: &mut Pipeline,
    ) {
        let new_pipeline = if let Some(pipeline) = self.cache.shift_remove(&config) {
            Pipeline { pipeline, config }
        } else {
            self.create(device, config)
        };
        let old_pipeline = std::mem::replace(pipeline, new_pipeline);
        self.cache
            .insert(old_pipeline.config, old_pipeline.pipeline);
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
        format: nvgx::TextureType,
    ) -> Self {
        let builder = PipelineBuilder::new(shader, pipeline_layout);
        let default_blend = nvgx::CompositeOperationState::default();
        let fill_stencil = builder.create(
            &device,
            PipelineConfig {
                format,
                usage: PipelineUsage::FillStencil(PathFillType::Winding),
            },
        );
        let fill_stroke = builder.create(
            &device,
            PipelineConfig {
                format,
                usage: PipelineUsage::FillStroke(default_blend.clone()),
            },
        );
        let fill_inner = builder.create(
            &device,
            PipelineConfig {
                format,
                usage: PipelineUsage::FillInner(default_blend),
            },
        );
        let fill_convex = builder.create(
            &device,
            PipelineConfig {
                format,
                usage: PipelineUsage::FillConvex(default_blend),
            },
        );
        let triangles = builder.create(
            &device,
            PipelineConfig {
                format,
                usage: PipelineUsage::Triangles(default_blend),
            },
        );
        let wirelines = builder.create(
            &device,
            PipelineConfig {
                format,
                usage: PipelineUsage::Lines(default_blend),
            },
        );
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
    pub fn update_pipeline(&mut self, device: &wgpu::Device, config: PipelineConfig) {
        match &config.usage {
            PipelineUsage::FillStencil(_) => {
                if self.fill_stencil.config != config {
                    self.builder
                        .update_pipeline(config, device, &mut self.fill_stencil);
                }
            }
            PipelineUsage::FillStroke(_) => {
                if self.fill_stroke.config != config {
                    self.builder
                        .update_pipeline(config, device, &mut self.fill_stroke);
                }
            }
            PipelineUsage::FillInner(_) => {
                if self.fill_inner.config != config {
                    self.builder
                        .update_pipeline(config, device, &mut self.fill_inner);
                }
            }
            PipelineUsage::FillConvex(_) => {
                if self.fill_convex.config != config {
                    self.builder
                        .update_pipeline(config, device, &mut self.fill_convex);
                }
            }
            PipelineUsage::Triangles(_) => {
                if self.triangles.config != config {
                    self.builder
                        .update_pipeline(config, device, &mut self.triangles);
                }
            }
            PipelineUsage::Lines(_) => {
                if self.wirelines.config != config {
                    self.builder
                        .update_pipeline(config, device, &mut self.wirelines);
                }
            }
        }
    }
}
