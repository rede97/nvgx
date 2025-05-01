use std::thread::sleep_ms;

use anyhow::Ok;
use nvg::Vertex;
use wgpu::{Extent3d, Origin2d};

use crate::wgpu::{
    call::{Call, GpuPath, ToBlendState},
    texture,
    unifroms::{RenderCommand, ShaderType},
};

use super::{call::CallType, pipeline::PipelineUsage, Renderer};

impl nvg::Renderer for Renderer {
    fn edge_antialias(&self) -> bool {
        return true;
    }

    fn resize(&mut self, _width: u32, _height: u32) -> anyhow::Result<()> {
        self.surface_config.width = _width;
        self.surface_config.height = _height;
        self.surface.configure(&self.device, &self.surface_config);
        self.stencil_texture = texture::StencilTexture::new(&self.device, &self.surface_config);
        Ok(())
    }

    fn create_texture(
        &mut self,
        texture_type: nvg::TextureType,
        width: u32,
        height: u32,
        flags: nvg::ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<nvg::ImageId> {
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };
        let texture = texture::Texture::new(
            &self.device,
            size,
            flags,
            texture_type,
            &self.texture_bind_group_layout,
        );
        if let Some(data) = data {
            texture.update(&self.queue, data, Origin2d::ZERO, size);
        }
        let id = self.textures.insert(texture);
        Ok(id as nvg::ImageId)
    }

    fn delete_texture(&mut self, img: nvg::ImageId) -> anyhow::Result<()> {
        self.textures.remove(img as usize);
        Ok(())
    }

    fn update_texture(
        &mut self,
        img: nvg::ImageId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let texture = self
            .textures
            .get_mut(img as usize)
            .ok_or_else(|| anyhow::anyhow!("Texture not found"))?;
        texture.update(
            &self.queue,
            data,
            Origin2d { x, y },
            Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
        );
        Ok(())
    }

    fn texture_size(&self, img: nvg::ImageId) -> anyhow::Result<(u32, u32)> {
        let texture = self
            .textures
            .get(img as usize)
            .ok_or_else(|| anyhow::anyhow!("Texture not found"))?;
        let size = texture.size();
        Ok((size.width, size.height))
    }

    fn viewport(&mut self, extent: nvg::Extent, _device_pixel_ratio: f32) -> anyhow::Result<()> {
        self.viewsize_uniform.value = extent;
        Ok(())
    }

    fn cancel(&mut self) -> anyhow::Result<()> {
        self.calls.clear();
        self.paths.clear();
        self.mesh.clear();
        self.render_unifrom.value.clear();
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.mesh.update_buffer(&self.device, &self.queue);
        self.viewsize_uniform
            .update_buffer(&self.device, &self.queue);
        self.render_unifrom.update_buffer(&self.device, &self.queue);

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Nvg Flush Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("NVG Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                    view: &self.stencil_texture.view,
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
                        self.pipeline_manager
                            .update_pipeline(&self.device, PipelineUsage::FillStencil(t));
                        self.pipeline_manager.update_pipeline(
                            &self.device,
                            PipelineUsage::FillStroke(call.blend_func.clone()),
                        );
                        self.pipeline_manager.update_pipeline(
                            &self.device,
                            PipelineUsage::FillInner(call.blend_func.clone()),
                        );
                        self.do_fill(call, &mut render_pass);
                    }
                    _ => {
                        println!("call: {:?}, todo", call.call_type);
                    }
                }
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        return self.cancel();
    }

    fn fill(
        &mut self,
        paint: &nvg::PaintPattern,
        composite_operation: nvg::CompositeOperationState,
        fill_type: nvg::PathFillType,
        scissor: &nvg::Scissor,
        fringe: f32,
        bounds: nvg::Bounds,
        paths: &[nvg::PathInfo],
    ) -> anyhow::Result<()> {
        let path_offset = self.paths.len();
        let mut offset = self.mesh.vertices.len();
        for path in paths {
            let fill = path.get_fill();
            let mut wgpu_path = GpuPath::default();
            if !fill.is_empty() {
                wgpu_path.fill_offset = offset;
                wgpu_path.fill_count = fill.len();
                self.mesh.vertices.extend(fill);
                offset += fill.len()
            }

            let stroke = path.get_stroke();
            if !stroke.is_empty() {
                wgpu_path.stroke_offset = offset;
                wgpu_path.stroke_count = stroke.len();
                self.mesh.vertices.extend(stroke);
                offset += stroke.len();
            }
            self.paths.push(wgpu_path);
        }

        let call = Call {
            call_type: if paths.len() == 1 && paths[0].convex {
                crate::wgpu::call::CallType::ConvexFill
            } else {
                crate::wgpu::call::CallType::Fill(fill_type)
            },
            image: paint.image,
            path_offset,
            path_count: paths.len(),
            triangle_offset: offset,
            triangle_count: 4,
            uniform_offset: self.render_unifrom.value.len(),
            blend_func: composite_operation,
            wireframe: false,
        };

        if let CallType::Fill(_) = call.call_type {
            self.mesh.vertices.extend([
                Vertex::new(bounds.max.x, bounds.max.y, 0.5, 1.0),
                Vertex::new(bounds.max.x, bounds.min.y, 0.5, 1.0),
                Vertex::new(bounds.min.x, bounds.max.y, 0.5, 1.0),
                Vertex::new(bounds.min.x, bounds.min.y, 0.5, 1.0),
            ]);
            self.render_unifrom.value.push(RenderCommand {
                stroke_thr: -1.0,
                render_type: ShaderType::Simple as u32,
                ..Default::default()
            });
            self.render_unifrom.value.push(RenderCommand::new(
                &self, paint, scissor, fringe, fringe, -1.0,
            ));
        } else {
            self.render_unifrom.value.push(RenderCommand::new(
                &self, paint, scissor, fringe, fringe, -1.0,
            ));
        }
        self.calls.push(call);
        Ok(())
    }

    fn stroke(
        &mut self,
        paint: &nvg::PaintPattern,
        composite_operation: nvg::CompositeOperationState,
        scissor: &nvg::Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[nvg::PathInfo],
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn triangles(
        &mut self,
        paint: &nvg::PaintPattern,
        composite_operation: nvg::CompositeOperationState,
        scissor: &nvg::Scissor,
        vertexes: &[nvg::Vertex],
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn clear(&mut self, color: nvg::Color) -> anyhow::Result<()> {
        self.cancel()?;
        self.clear_cmd = Some(wgpu::Color {
            r: color.r as f64,
            g: color.g as f64,
            b: color.b as f64,
            a: color.a as f64,
        });
        Ok(())
    }

    fn wireframe(&mut self, _enable: bool) -> anyhow::Result<()> {
        todo!()
    }

    fn wirelines(
        &mut self,
        _paint: &nvg::PaintPattern,
        _composite_operation: nvg::CompositeOperationState,
        _scissor: &nvg::Scissor,
        _path: &[nvg::PathInfo],
    ) -> anyhow::Result<()> {
        todo!()
    }
}
