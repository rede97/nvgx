use super::Renderer;

impl nvg::Renderer for Renderer {
    fn edge_antialias(&self) -> bool {
        return true;
    }

    fn create_texture(
        &mut self,
        texture_type: nvg::TextureType,
        width: usize,
        height: usize,
        flags: nvg::ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<nvg::ImageId> {
        todo!()
    }

    fn delete_texture(&mut self, img: nvg::ImageId) -> anyhow::Result<()> {
        todo!()
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
        todo!()
    }

    fn texture_size(&self, img: nvg::ImageId) -> anyhow::Result<(usize, usize)> {
        todo!()
    }

    fn viewport(&mut self, extent: nvg::Extent, device_pixel_ratio: f32) -> anyhow::Result<()> {
        todo!()
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
