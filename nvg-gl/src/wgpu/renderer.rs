use nvg::Vertex;
use wgpu::{Extent3d, Origin2d};

use crate::wgpu::{
    call::{Call, GpuPath},
    unifroms::{RenderCommand, ShaderType},
};

use super::{call::CallType, Renderer};

impl nvg::RendererDevice for Renderer {
    fn edge_antialias(&self) -> bool {
        return self.config.antialias;
    }

    fn resize(&mut self, _width: u32, _height: u32) -> anyhow::Result<()> {
        self.surface_config.width = _width;
        self.surface_config.height = _height;
        self.surface.configure(&self.device, &self.surface_config);
        self.resources
            .texture_manager
            .configure_stencil(&self.device, &self.surface_config);
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
        Ok(self.resources.texture_manager.create(
            &self.device,
            &self.queue,
            wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            flags,
            texture_type,
            data,
        ) as nvg::ImageId)
    }

    fn delete_texture(&mut self, img: nvg::ImageId) -> anyhow::Result<()> {
        self.resources.texture_manager.remove(img as usize);
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
            .resources
            .texture_manager
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
            .resources
            .texture_manager
            .get(img as usize)
            .ok_or_else(|| anyhow::anyhow!("Texture not found"))?;
        let size = texture.size();
        Ok((size.width, size.height))
    }

    fn viewport(&mut self, extent: nvg::Extent, _device_pixel_ratio: f32) -> anyhow::Result<()> {
        self.resources.viewsize_uniform.value = extent;
        Ok(())
    }

    #[inline]
    fn cancel(&mut self) -> anyhow::Result<()> {
        self.resources.calls.clear();
        self.resources.paths.clear();
        self.resources.mesh.clear();
        self.resources.render_unifrom.value.clear();
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.resources.mesh.update_buffer(&self.device, &self.queue);
        self.resources
            .viewsize_uniform
            .update_buffer(&self.device, &self.queue);
        self.resources
            .render_unifrom
            .update_buffer(&self.device, &self.queue);

        if let Some((image, stencil_view)) = self.target_fb.as_ref() {
            let texture = self.resources.texture_manager.get(*image).unwrap();
            self.resources.render(
                &self.device,
                &self.queue,
                &texture.view,
                &stencil_view,
                &mut self.pipeline_manager,
            );
        } else {
            let output = self.surface.get_current_texture().unwrap();
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.resources.render(
                &self.device,
                &self.queue,
                &view,
                self.resources.texture_manager.stencil_view(),
                &mut self.pipeline_manager,
            );
            output.present();
        };
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
        let path_offset = self.resources.paths.len();
        let mut offset = self.resources.mesh.vertices.len();
        for path in paths {
            let fill = path.get_fill();
            let mut wgpu_path = GpuPath::default();
            if !fill.is_empty() {
                wgpu_path.fill_offset = offset;
                wgpu_path.fill_count = fill.len();
                self.resources.mesh.vertices.extend(fill);
                offset += fill.len()
            }

            let stroke = path.get_stroke();
            if !stroke.is_empty() {
                wgpu_path.stroke_offset = offset;
                wgpu_path.stroke_count = stroke.len();
                self.resources.mesh.vertices.extend(stroke);
                offset += stroke.len();
            }
            self.resources.paths.push(wgpu_path);
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
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
        };

        if let CallType::Fill(_) = call.call_type {
            self.resources.mesh.vertices.extend([
                Vertex::new(bounds.max.x, bounds.max.y, 0.5, 1.0),
                Vertex::new(bounds.max.x, bounds.min.y, 0.5, 1.0),
                Vertex::new(bounds.min.x, bounds.max.y, 0.5, 1.0),
                Vertex::new(bounds.min.x, bounds.min.y, 0.5, 1.0),
            ]);
            self.resources.render_unifrom.value.push(RenderCommand {
                stroke_thr: -1.0,
                render_type: ShaderType::Simple as u32,
                ..Default::default()
            });
            self.resources.render_unifrom.value.push(RenderCommand::new(
                &self, paint, scissor, fringe, fringe, -1.0,
            ));
        } else {
            self.resources.render_unifrom.value.push(RenderCommand::new(
                &self, paint, scissor, fringe, fringe, -1.0,
            ));
        }
        self.resources.calls.push(call);
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
        let call = Call {
            call_type: CallType::Stroke,
            image: paint.image,
            path_offset: self.resources.paths.len(),
            path_count: paths.len(),
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
            ..Default::default()
        };

        let mut offset = self.resources.mesh.vertices.len();
        for path in paths {
            let mut gl_path = GpuPath {
                fill_offset: 0,
                fill_count: 0,
                stroke_offset: 0,
                stroke_count: 0,
            };

            let stroke = path.get_stroke();
            if !stroke.is_empty() {
                gl_path.stroke_offset = offset;
                gl_path.stroke_count = stroke.len();
                self.resources.mesh.vertices.extend(stroke);
                offset += stroke.len();
                self.resources.paths.push(gl_path);
            }
        }
        self.resources.render_unifrom.value.push(RenderCommand::new(
            &self,
            paint,
            scissor,
            stroke_width,
            fringe,
            -1.0,
        ));
        self.resources.calls.push(call);
        Ok(())
    }

    fn triangles(
        &mut self,
        paint: &nvg::PaintPattern,
        composite_operation: nvg::CompositeOperationState,
        scissor: &nvg::Scissor,
        vertexes: &[nvg::Vertex],
    ) -> anyhow::Result<()> {
        let call = Call {
            call_type: CallType::Triangles,
            image: paint.image,
            triangle_offset: self.resources.mesh.vertices.len(),
            triangle_count: vertexes.len(),
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
            ..Default::default()
        };

        self.resources.calls.push(call);
        self.resources.mesh.vertices.extend(vertexes);

        self.resources.render_unifrom.value.push(
            RenderCommand::new(&self, paint, scissor, 1.0, 1.0, -1.0).set_type(ShaderType::Image),
        );
        Ok(())
    }

    fn clear(&mut self, color: nvg::Color) -> anyhow::Result<()> {
        self.cancel()?;
        self.resources.clear_cmd = Some(wgpu::Color {
            r: color.r as f64,
            g: color.g as f64,
            b: color.b as f64,
            a: color.a as f64,
        });
        Ok(())
    }

    fn wirelines(
        &mut self,
        paint: &nvg::PaintPattern,
        composite_operation: nvg::CompositeOperationState,
        scissor: &nvg::Scissor,
        paths: &[nvg::PathInfo],
    ) -> anyhow::Result<()> {
        let call = Call {
            call_type: CallType::Lines,
            image: paint.image,
            path_offset: self.resources.paths.len(),
            path_count: paths.len(),
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
            ..Default::default()
        };

        let mut offset = self.resources.mesh.vertices.len();
        for path in paths {
            let line = path.get_line();
            if !line.is_empty() {
                let gl_path = GpuPath {
                    stroke_offset: offset,
                    stroke_count: line.len(),
                    ..Default::default()
                };
                self.resources.mesh.vertices.extend(line);
                offset += line.len();
                self.resources.paths.push(gl_path);
            }
        }

        self.resources.calls.push(call);

        self.resources
            .render_unifrom
            .value
            .push(RenderCommand::new(&self, paint, scissor, 1.0, 1.0, -1.0));
        Ok(())
    }
}
