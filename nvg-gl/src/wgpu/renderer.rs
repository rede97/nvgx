use anyhow::Ok;
use wgpu::{Extent3d, Origin2d};

use crate::wgpu::texture;

use super::Renderer;

impl nvg::Renderer for Renderer {
    fn edge_antialias(&self) -> bool {
        return true;
    }

    fn resize(&mut self, _width: u32, _height: u32) -> anyhow::Result<()> {
        self.surface_config.width = _width;
        self.surface_config.height = _height;
        self.surface.configure(&self.device, &self.surface_config);
        Ok(())
    }

    fn create_texture(
        &mut self,
        texture_type: nvg::TextureType,
        width: usize,
        height: usize,
        flags: nvg::ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<nvg::ImageId> {
        let size = wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
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
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let texture = self
            .textures
            .get_mut(img as usize)
            .ok_or_else(|| anyhow::anyhow!("Texture not found"))?;
        texture.update(
            &self.queue,
            data,
            Origin2d {
                x: x as u32,
                y: y as u32,
            },
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );
        Ok(())
    }

    fn texture_size(&self, img: nvg::ImageId) -> anyhow::Result<(usize, usize)> {
        let texture = self
            .textures
            .get(img as usize)
            .ok_or_else(|| anyhow::anyhow!("Texture not found"))?;
        let size = texture.size();
        Ok((size.width as usize, size.height as usize))
    }

    fn viewport(&mut self, extent: nvg::Extent, device_pixel_ratio: f32) -> anyhow::Result<()> {
        self.viewsize = extent;
        Ok(())
    }

    fn cancel(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        todo!()
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
        todo!()
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
        todo!()
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
