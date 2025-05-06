use call::{Call, GpuPath};
use mesh::Mesh;
use nvg::*;
use pipeline::PipelineManager;
use texture::TextureManager;
use unifroms::{RenderCommand, Unifrom};

use crate::RenderConfig;

mod fb;
mod call;
mod mesh;
mod pipeline;
mod renderer;
mod texture;
mod unifroms;

pub struct Renderer {
    config: RenderConfig,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    viewsize_uniform: Unifrom<Extent>,
    render_unifrom: Unifrom<Vec<RenderCommand>>,
    texture_manager: TextureManager,
    clear_cmd: Option<wgpu::Color>,
    calls: Vec<Call>,
    paths: Vec<GpuPath>,
    mesh: Mesh,
    pipeline_manager: PipelineManager,
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
            viewsize_uniform,
            render_unifrom,
            texture_manager,
            clear_cmd: None,
            calls: Vec::new(),
            paths: Vec::new(),
            mesh,
            pipeline_manager,
        });
    }

    #[inline]
    fn do_fill(&self, call: &Call, render_pass: &mut wgpu::RenderPass<'_>) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        {
            {
                // Fill stencil pass
                render_pass.set_pipeline(self.pipeline_manager.fill_stencil.pipeline());
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
                render_pass.set_pipeline(self.pipeline_manager.fill_stroke.pipeline());
                render_pass.set_stencil_reference(0);
                render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
                render_pass.set_bind_group(
                    1,
                    &self.render_unifrom.bind_group,
                    &[call.uniform_offset(1)],
                );
                render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
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
                render_pass.set_pipeline(self.pipeline_manager.fill_inner.pipeline());
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

    fn do_convex_fill(&self, call: &Call, render_pass: &mut wgpu::RenderPass<'_>) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        {
            render_pass.set_pipeline(self.pipeline_manager.fill_convex.pipeline());
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
            render_pass.set_pipeline(self.pipeline_manager.fill_stroke.pipeline());
            render_pass.set_stencil_reference(0);
            render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
            render_pass.set_bind_group(
                1,
                &self.render_unifrom.bind_group,
                &[call.uniform_offset(0)],
            );
            render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
            for path in paths {
                render_pass
                    .set_vertex_buffer(0, self.mesh.vertex_buffer.slice(path.stroke_slice()));
                render_pass.draw(0..path.stroke_count(), 0..1);
            }
        }
    }

    fn do_stroke(&self, call: &Call, render_pass: &mut wgpu::RenderPass<'_>) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        render_pass.set_pipeline(self.pipeline_manager.fill_stroke.pipeline());
        render_pass.set_stencil_reference(0);
        render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.render_unifrom.bind_group,
            &[call.uniform_offset(0)],
        );
        render_pass.set_bind_group(2, self.texture_manager.get_bindgroup(call.image), &[]);
        for path in paths {
            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(path.stroke_slice()));
            render_pass.draw(0..path.stroke_count(), 0..1);
        }
    }

    fn do_triangles(&self, call: &Call, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(self.pipeline_manager.triangles.pipeline());
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
}
