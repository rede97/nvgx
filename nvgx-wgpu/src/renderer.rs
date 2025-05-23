use std::ops::Range;
use std::sync::Arc;

use nvgx::{BufferUsage, VertexSlice};
use wgpu::{Extent3d, Origin2d};

use crate::{
    call::{Call, GpuPath},
    unifroms::{RenderCommand, ShaderType},
};

use super::{call::CallType, mesh::Mesh, Renderer};

impl nvgx::RendererDevice for Renderer {
    type VertexBuffer = Arc<wgpu::Buffer>;

    fn edge_antialias(&self) -> bool {
        return self.resources.config.antialias;
    }

    fn create_vertex_buffer(
        &mut self,
        buffer_size: usize,
        _usage: BufferUsage,
    ) -> anyhow::Result<Self::VertexBuffer> {
        return Ok(Arc::new(Mesh::create_buffer(&self.device, buffer_size)));
    }

    fn update_vertex_buffer(
        &mut self,
        buffer: Option<&Self::VertexBuffer>,
        data: &[u8],
    ) -> anyhow::Result<()> {
        if let Some(buffer) = buffer {
            self.resources
                .mesh
                .update_buffer(&self.queue, buffer.as_ref(), data)?;
        } else {
            self.resources
                .mesh
                .update_inner_buffer(&self.device, &self.queue, data);
        };
        Ok(())
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
        texture_type: nvgx::TextureType,
        width: u32,
        height: u32,
        flags: nvgx::ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<nvgx::ImageId> {
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
        ) as nvgx::ImageId)
    }

    fn delete_texture(&mut self, img: nvgx::ImageId) -> anyhow::Result<()> {
        self.resources.texture_manager.remove(img as usize);
        Ok(())
    }

    fn update_texture(
        &mut self,
        img: nvgx::ImageId,
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

    fn texture_size(&self, img: nvgx::ImageId) -> anyhow::Result<(u32, u32)> {
        let texture = self
            .resources
            .texture_manager
            .get(img as usize)
            .ok_or_else(|| anyhow::anyhow!("Texture not found"))?;
        let size = texture.size();
        Ok((size.width, size.height))
    }

    fn viewport(&mut self, extent: nvgx::Extent, _device_pixel_ratio: f32) -> anyhow::Result<()> {
        self.resources.viewsize_uniform.value = extent;
        Ok(())
    }

    #[inline]
    fn cancel(&mut self) -> anyhow::Result<()> {
        self.resources.calls.clear();
        self.resources.paths.clear();
        self.resources.render_unifrom.value.clear();
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
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
                texture.texture_type,
                &stencil_view,
                &mut self.pipeline_manager,
                self.clear_cmd.take(),
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
                self.resources.config.format,
                self.resources.texture_manager.stencil_view(),
                &mut self.pipeline_manager,
                self.clear_cmd.take(),
            );
            output.present();
        };
        return self.cancel();
    }

    fn fill(
        &mut self,
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        fill_type: nvgx::PathFillType,
        scissor: &nvgx::Scissor,
        fringe: f32,
        bounds_offset: Option<usize>,
        paths: &[nvgx::PathSlice],
    ) -> anyhow::Result<()> {
        let path_offset = self.resources.paths.len();
        let mut fill_vertex_count = 0;
        self.resources.paths.extend(paths.iter().filter_map(|p| {
            let fill = p.get_fill();
            if fill.count < 3 {
                None
            } else {
                fill_vertex_count += fill.count;
                Some(GpuPath {
                    fill,
                    stroke: p.get_stroke(),
                })
            }
        }));

        self.resources
            .mesh
            .update_indices(&self.device, &self.queue, fill_vertex_count as u64);

        let call = Call {
            call_type: if bounds_offset.is_some() {
                crate::call::CallType::Fill(fill_type)
            } else {
                crate::call::CallType::ConvexFill
            },
            image: paint.image,
            path_range: path_offset..self.resources.paths.len(),
            triangle: if let Some(offset) = bounds_offset {
                VertexSlice { offset, count: 4 }
            } else {
                Default::default()
            },
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
            vertex_buffer,
            instances,
        };

        if let CallType::Fill(_) = call.call_type {
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
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        scissor: &nvgx::Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[nvgx::PathSlice],
    ) -> anyhow::Result<()> {
        let path_offset = self.resources.paths.len();

        self.resources.paths.extend(paths.iter().filter_map(|p| {
            let stroke = p.get_stroke();
            Some(GpuPath {
                stroke: stroke,
                ..Default::default()
            })
        }));

        let call = Call {
            call_type: CallType::Stroke,
            image: paint.image,
            path_range: path_offset..self.resources.paths.len(),
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
            vertex_buffer,
            triangle: VertexSlice::default(),
            instances,
        };

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
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        scissor: &nvgx::Scissor,
        slice: VertexSlice,
    ) -> anyhow::Result<()> {
        let call = Call {
            call_type: CallType::Triangles,
            image: paint.image,
            triangle: slice,
            path_range: 0..0,
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
            vertex_buffer,
            instances,
        };

        self.resources.calls.push(call);

        self.resources.render_unifrom.value.push(
            RenderCommand::new(&self, paint, scissor, 1.0, 1.0, -1.0).set_type(ShaderType::Image),
        );
        Ok(())
    }

    fn clear(&mut self, color: nvgx::Color) -> anyhow::Result<()> {
        self.cancel()?;
        self.clear_cmd = Some(wgpu::Color {
            r: color.r as f64,
            g: color.g as f64,
            b: color.b as f64,
            a: color.a as f64,
        });
        Ok(())
    }

    #[cfg(feature = "wirelines")]
    fn wirelines(
        &mut self,
        vertex_buffer: Option<Self::VertexBuffer>,
        instances: Option<(Self::VertexBuffer, Range<u32>)>,
        paint: &nvgx::PaintPattern,
        composite_operation: nvgx::CompositeOperationState,
        scissor: &nvgx::Scissor,
        paths: &[nvgx::PathSlice],
    ) -> anyhow::Result<()> {
        let path_offset = self.resources.paths.len();

        self.resources.paths.extend(paths.iter().filter_map(|p| {
            let stroke = p.get_stroke();
            Some(GpuPath {
                stroke: stroke,
                ..Default::default()
            })
        }));

        let call = Call {
            call_type: CallType::Lines,
            image: paint.image,
            path_range: path_offset..self.resources.paths.len(),
            uniform_offset: self.resources.render_unifrom.offset(),
            blend_func: composite_operation,
            vertex_buffer,
            triangle: VertexSlice::default(),
            instances,
        };

        self.resources.calls.push(call);

        self.resources
            .render_unifrom
            .value
            .push(RenderCommand::new(&self, paint, scissor, 1.0, 1.0, -1.0));
        Ok(())
    }
}
