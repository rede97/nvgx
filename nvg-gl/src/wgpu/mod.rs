use call::{Call, CallType, GpuPath};
use mesh::Mesh;
use nvg::*;
use pipeline::{PipelineManager, PipelineUsage};
use texture::TextureManager;
use unifroms::{RenderCommand, Unifrom};
use wgpu::TextureView;

use crate::RenderConfig;

mod call;
pub mod fb;
mod mesh;
mod pipeline;
mod renderer;
mod texture;
mod unifroms;

pub struct RenderResource {
    mesh: Mesh,
    paths: Vec<GpuPath>,
    calls: Vec<Call>,
    clear_cmd: Option<wgpu::Color>,
    viewsize_uniform: Unifrom<Extent>,
    render_unifrom: Unifrom<Vec<RenderCommand>>,
    texture_manager: TextureManager,
}

impl RenderResource {
    #[inline]
    fn do_fill(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
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
                for path in paths {
                    let count = path.triangle_fan_count();
                    if count != 0 {
                        render_pass.set_vertex_buffer(
                            0,
                            self.mesh.vertex_buffer.slice(path.triangle_fan_slice()),
                        );
                        render_pass.draw_indexed(0..(count * 3), 0, 0..1);
                    }
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
                render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
                for path in paths {
                    let count = path.stroke_count();
                    if count >= 3 {
                        render_pass.set_vertex_buffer(
                            0,
                            self.mesh.vertex_buffer.slice(path.stroke_slice()),
                        );
                        render_pass.draw(0..count, 0..1);
                    }
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
                render_pass
                    .set_vertex_buffer(0, self.mesh.vertex_buffer.slice(call.triangle_slice()));
                render_pass.draw(0..call.triangle_count(), 0..1);
            }
        }
    }

    fn do_convex_fill(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
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
            for path in paths {
                render_pass
                    .set_vertex_buffer(0, self.mesh.vertex_buffer.slice(path.triangle_fan_slice()));
                render_pass.draw_indexed(0..path.triangle_fan_count() * 3, 0, 0..1);
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
            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            for path in paths {
                render_pass.draw(
                    (path.stroke_offset as u32)..((path.stroke_offset + path.stroke_count) as u32),
                    0..1,
                );
            }
        }
    }

    fn do_stroke(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        render_pass.set_pipeline(pipeline_manager.fill_stroke.pipeline());
        render_pass.set_stencil_reference(0);
        render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.render_unifrom.bind_group,
            &[call.uniform_offset(0)],
        );
        render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        for path in paths {
            render_pass.draw(
                (path.stroke_offset as u32)..((path.stroke_offset + path.stroke_count) as u32),
                0..1,
            );
        }
    }

    fn do_triangles(
        &self,
        call: &Call,
        render_pass: &mut wgpu::RenderPass<'_>,
        pipeline_manager: &PipelineManager,
    ) {
        render_pass.set_pipeline(pipeline_manager.triangles.pipeline());
        render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.render_unifrom.bind_group,
            &[call.uniform_offset(0)],
        );
        render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(call.triangle_slice()));
        render_pass.draw(0..call.triangle_count(), 0..1);
    }

    fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &TextureView,
        stencil_view: &TextureView,
        pipeline_manager: &mut PipelineManager,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Nvg Flush Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("NVG Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if let Some(color) = self.clear_cmd {
                            wgpu::LoadOp::Clear(color)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: stencil_view,
                    stencil_ops: Some(wgpu::Operations {
                        load: if self.clear_cmd.is_some() {
                            wgpu::LoadOp::Clear(0)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                    depth_ops: None,
                }),
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
                    _ => {
                        println!("call: {:?}, todo", call.call_type);
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

        return Ok(Self {
            config,
            device,
            queue,
            surface,
            surface_config,
            target_fb: None,
            pipeline_manager,
            resources: RenderResource {
                mesh,
                clear_cmd: None,
                paths: Vec::new(),
                calls: Vec::new(),
                viewsize_uniform,
                render_unifrom,
                texture_manager,
            },
        });
    }
}
