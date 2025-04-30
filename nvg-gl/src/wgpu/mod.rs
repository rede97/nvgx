use std::path;

use call::{Call, GpuPath, ToBlendState};
use mesh::Mesh;
use nvg::*;
use pipeline::{PipelineBuilder, PipelineConfig, Pipelines};
use slab::Slab;
use texture::{StencilTexture, Texture};
use unifroms::{RenderCommand, Unifrom};
use wgpu::{vertex_attr_array, ShaderStages};

mod call;
mod mesh;
mod pipeline;
mod renderer;
mod texture;
mod unifroms;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    viewsize_uniform: Unifrom<Extent>,
    render_unifrom: Unifrom<Vec<RenderCommand>>,
    stencil_texture: StencilTexture,
    textures: Slab<Texture>,
    place_holder_texture: Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    calls: Vec<Call>,
    paths: Vec<GpuPath>,
    mesh: Mesh,
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

        let place_holder_texture =
            Texture::placeholder_texture(&device, &texture_bind_group_layout);

        let mesh = Mesh::new(&device, &queue, 1024);

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
            place_holder_texture,
            texture_bind_group_layout,
            calls: Vec::new(),
            paths: Vec::new(),
            mesh,
            pipeline_builder,
            pipelines,
        });
    }

    fn do_fill(&self, call: &Call) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.3,
                            b: 0.4,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.stencil_texture.view,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                    depth_ops: None,
                }),
                ..Default::default()
            });

            let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
            render_pass.set_pipeline(self.pipelines.fill_stencil.pipeline());
            render_pass.set_bind_group(0, &self.viewsize_uniform.bind_group, &[]);
            render_pass.set_bind_group(1, &self.render_unifrom.bind_group, &[0]);
            render_pass.set_bind_group(2, &self.place_holder_texture.bind_group, &[]);
            for path in paths {
                render_pass.set_stencil_reference(0);
                render_pass
                    .set_vertex_buffer(0, self.mesh.vertex_buffer.slice(path.triangle_fan_slice()));
                render_pass
                    .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..path.triangle_fan_count() * 3, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
