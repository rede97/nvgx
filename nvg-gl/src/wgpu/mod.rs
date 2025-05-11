use std::ops::Deref;

use call::{Call, CallType, GpuPath};
use mesh::Mesh;
use nvg::*;
use pipeline::{PipelineManager, PipelineUsage};
use texture::TextureManager;
use unifroms::{RenderCommand, Unifrom};
use wgpu::{util::DeviceExt, TextureView};

use crate::RenderConfig;

mod call;
pub mod fb;
mod instance;
mod mesh;
mod pipeline;
mod renderer;
mod texture;
mod unifroms;

pub struct RenderResource {
    mesh: Mesh,
    paths: Vec<GpuPath>,
    calls: Vec<Call>,
    viewsize_uniform: Unifrom<Extent>,
    render_unifrom: Unifrom<Vec<RenderCommand>>,
    texture_manager: TextureManager,
    default_instace: wgpu::Buffer,
}

impl RenderResource {
    #[inline]
    fn do_fill(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let paths = &self.paths[call.path_range.clone()];
        let buffer = call
            .vertex_buffer
            .as_ref()
            .map(|v| v.deref())
            .unwrap_or(&self.mesh.vertex_buffer);
        let (instance_buffer, instance_slice) = call
            .instances
            .as_ref()
            .map(|i| (i.0.deref(), i.1.clone()))
            .unwrap_or((&self.default_instace, 0..1));
        {
            {
                // Fill stencil pass
                render_pass.set_pipeline(pipeline_manager.fill_stencil.pipeline());
                render_pass.set_stencil_reference(0);
                render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
                render_pass.set_bind_group(
                    1,
                    &self.render_unifrom.bind_group,
                    &[call.uniform_offset(0)],
                );
                render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
                render_pass
                    .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(0, buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                for path in paths {
                    let count = path.triangle_fan_count();
                    render_pass.draw_indexed(
                        0..(count * 3),
                        path.triangle_fan_offset(),
                        instance_slice.clone(),
                    );
                }
            }
            {
                // Stroke border pass
                render_pass.set_pipeline(pipeline_manager.fill_stroke.pipeline());
                render_pass.set_stencil_reference(0);
                render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
                render_pass.set_bind_group(
                    1,
                    &self.render_unifrom.bind_group,
                    &[call.uniform_offset(1)],
                );
                render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
                render_pass.set_vertex_buffer(0, buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

                for path in paths {
                    render_pass.draw(path.stroke_vert(), instance_slice.clone());
                }
            }
            {
                // Fill Content pass
                render_pass.set_pipeline(pipeline_manager.fill_inner.pipeline());
                render_pass.set_stencil_reference(0);
                render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
                render_pass.set_bind_group(
                    1,
                    &self.render_unifrom.bind_group,
                    &[call.uniform_offset(1)],
                );
                render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
                render_pass.set_vertex_buffer(0, buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

                render_pass.draw(call.triangle_vert(), instance_slice.clone());
            }
        }
    }

    #[inline]
    fn do_convex_fill(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let paths = &self.paths[call.path_range.clone()];
        let buffer = call
            .vertex_buffer
            .as_ref()
            .map(|v| v.deref())
            .unwrap_or(&self.mesh.vertex_buffer);
        let (instance_buffer, instance_slice) = call
            .instances
            .as_ref()
            .map(|i| (i.0.deref(), i.1.clone()))
            .unwrap_or((&self.default_instace, 0..1));
        {
            render_pass.set_pipeline(pipeline_manager.fill_convex.pipeline());
            render_pass.set_stencil_reference(0);
            render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
            render_pass.set_bind_group(
                1,
                &self.render_unifrom.bind_group,
                &[call.uniform_offset(0)],
            );
            render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
            render_pass
                .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

            for path in paths {
                render_pass.draw_indexed(
                    0..path.triangle_fan_count() * 3,
                    path.triangle_fan_offset(),
                    instance_slice.clone(),
                );
            }
        }

        {
            render_pass.set_pipeline(pipeline_manager.fill_stroke.pipeline());
            render_pass.set_stencil_reference(0);
            render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
            render_pass.set_bind_group(
                1,
                &self.render_unifrom.bind_group,
                &[call.uniform_offset(0)],
            );
            render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

            for path in paths {
                render_pass.draw(path.stroke_vert(), instance_slice.clone());
            }
        }
    }

    #[inline]
    fn do_stroke(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let paths = &self.paths[call.path_range.clone()];
        let buffer = call
            .vertex_buffer
            .as_ref()
            .map(|v| v.deref())
            .unwrap_or(&self.mesh.vertex_buffer);
        let (instance_buffer, instance_slice) = call
            .instances
            .as_ref()
            .map(|i| (i.0.deref(), i.1.clone()))
            .unwrap_or((&self.default_instace, 0..1));
        render_pass.set_pipeline(pipeline_manager.fill_stroke.pipeline());
        render_pass.set_stencil_reference(0);
        render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.render_unifrom.bind_group,
            &[call.uniform_offset(0)],
        );
        render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
        render_pass.set_vertex_buffer(0, buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        for path in paths {
            render_pass.draw(path.stroke_vert(), instance_slice.clone());
        }
    }

    #[inline]
    fn do_triangles(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let buffer = call
            .vertex_buffer
            .as_ref()
            .map(|v| v.deref())
            .unwrap_or(&self.mesh.vertex_buffer);
        let (instance_buffer, instance_slice) = call
            .instances
            .as_ref()
            .map(|i| (i.0.deref(), i.1.clone()))
            .unwrap_or((&self.default_instace, 0..1));
        render_pass.set_pipeline(pipeline_manager.triangles.pipeline());
        render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.render_unifrom.bind_group,
            &[call.uniform_offset(0)],
        );
        render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
        render_pass.set_vertex_buffer(0, buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        render_pass.draw(call.triangle_vert(), instance_slice.clone());
    }

    #[inline]
    #[cfg(feature = "wirelines")]
    fn do_lines(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let paths = &self.paths[call.path_range.clone()];
        let buffer = call
            .vertex_buffer
            .as_ref()
            .map(|v| v.deref())
            .unwrap_or(&self.mesh.vertex_buffer);
        let (instance_buffer, instance_slice) = call
            .instances
            .as_ref()
            .map(|i| (i.0.deref(), i.1.clone()))
            .unwrap_or((&self.default_instace, 0..1));
        render_pass.set_pipeline(pipeline_manager.wirelines.pipeline());
        render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.render_unifrom.bind_group,
            &[call.uniform_offset(0)],
        );
        render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
        render_pass.set_vertex_buffer(0, buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        for path in paths {
            render_pass.draw(path.stroke_vert(), instance_slice.clone());
        }
    }

    fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &TextureView,
        stencil_view: &TextureView,
        pipeline_manager: &mut PipelineManager,
        clear_cmd: Option<wgpu::Color>,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Nvg Flush Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("NVG Render Pass"),
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: stencil_view,
                    stencil_ops: Some(wgpu::Operations {
                        load: if clear_cmd.is_some() {
                            wgpu::LoadOp::Clear(0)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                    depth_ops: None,
                }),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if let Some(color) = clear_cmd {
                            wgpu::LoadOp::Clear(color)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],

                ..Default::default()
            });
            for call in &self.calls {
                match call.call_type {
                    CallType::Fill(t) => {
                        pipeline_manager.update_pipeline(&device, PipelineUsage::FillStencil(t));
                        pipeline_manager.update_pipeline(
                            &device,
                            PipelineUsage::FillStroke(call.blend_func.clone()),
                        );
                        pipeline_manager
                            .update_pipeline(&device, PipelineUsage::FillInner(call.blend_func));
                        self.do_fill(call, &mut render_pass, &pipeline_manager);
                    }
                    CallType::ConvexFill => {
                        pipeline_manager.update_pipeline(
                            &device,
                            PipelineUsage::FillConvex(call.blend_func.clone()),
                        );
                        pipeline_manager
                            .update_pipeline(&device, PipelineUsage::FillStroke(call.blend_func));
                        self.do_convex_fill(call, &mut render_pass, &pipeline_manager);
                    }
                    CallType::Stroke => {
                        pipeline_manager
                            .update_pipeline(&device, PipelineUsage::FillStroke(call.blend_func));
                        self.do_stroke(call, &mut render_pass, &pipeline_manager);
                    }
                    CallType::Triangles => {
                        pipeline_manager
                            .update_pipeline(&device, PipelineUsage::Triangles(call.blend_func));
                        self.do_triangles(call, &mut render_pass, &pipeline_manager);
                    }
                    #[cfg(feature = "wirelines")]
                    CallType::Lines => {
                        pipeline_manager
                            .update_pipeline(&device, PipelineUsage::Lines(call.blend_func));
                        self.do_lines(call, &mut render_pass, &pipeline_manager);
                    }
                }
            }
        }
        queue.submit(std::iter::once(encoder.finish()));
    }
}

pub struct Renderer {
    config: RenderConfig,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline_manager: PipelineManager,
    target_fb: Option<(ImageId, TextureView)>,
    clear_cmd: Option<wgpu::Color>,
    resources: RenderResource,
}

impl Renderer {
    pub fn create(
        config: RenderConfig,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
    ) -> anyhow::Result<Self> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NVG Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let viewsize_uniform: Unifrom<Extent> =
            Unifrom::new(&device, 0, wgpu::ShaderStages::VERTEX, None);
        let render_unifrom: Unifrom<Vec<RenderCommand>> =
            Unifrom::new(&device, 0, wgpu::ShaderStages::FRAGMENT, Some(64));

        let mesh = Mesh::new(&device, &queue, 1024);
        let texture_manager = TextureManager::new(&device, &surface_config);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("NVG Render Pipeline Layout"),
            bind_group_layouts: &[
                &viewsize_uniform.layout,
                &render_unifrom.layout,
                &texture_manager.layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline_manager = PipelineManager::new(shader, pipeline_layout, &device);

        let identity_instance = Transform::identity();
        let default_instace = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Default Instace"),
            contents: bytemuck::bytes_of(&identity_instance),
            usage: wgpu::BufferUsages::VERTEX,
        });

        return Ok(Self {
            config,
            device,
            queue,
            surface,
            surface_config,
            target_fb: None,
            pipeline_manager,
            clear_cmd: None,
            resources: RenderResource {
                mesh,
                paths: Vec::new(),
                calls: Vec::new(),
                viewsize_uniform,
                render_unifrom,
                texture_manager,
                default_instace,
            },
        });
    }
}
