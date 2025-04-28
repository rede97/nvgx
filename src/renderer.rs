pub use crate::context::{CompositeOperationState, ImageId};
pub use crate::paint::PaintPattern;
pub use crate::path::cache::PathInfo;
pub use crate::path::cache::Vertex;
pub use crate::*;

#[derive(Debug, Copy, Clone)]
pub enum TextureType {
    RGBA,
    Alpha,
}

#[derive(Debug, Copy, Clone)]
pub struct Scissor {
    pub xform: Transform,
    pub extent: Extent,
}

pub trait Renderer {
    fn edge_antialias(&self) -> bool;

    fn create_texture(
        &mut self,
        texture_type: TextureType,
        width: usize,
        height: usize,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<ImageId>;

    fn delete_texture(&mut self, img: ImageId) -> anyhow::Result<()>;

    fn update_texture(
        &mut self,
        img: ImageId,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        data: &[u8],
    ) -> anyhow::Result<()>;

    fn texture_size(&self, img: ImageId) -> anyhow::Result<(usize, usize)>;

    fn viewport(&mut self, extent: Extent, device_pixel_ratio: f32) -> anyhow::Result<()>;

    fn cancel(&mut self) -> anyhow::Result<()>;

    fn flush(&mut self) -> anyhow::Result<()>;

    fn fill(
        &mut self,
        paint: &PaintPattern,
        composite_operation: CompositeOperationState,
        fill_type: PathFillType,
        scissor: &Scissor,
        fringe: f32,
        bounds: Bounds,
        paths: &[PathInfo],
    ) -> anyhow::Result<()>;

    fn stroke(
        &mut self,
        paint: &PaintPattern,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[PathInfo],
    ) -> anyhow::Result<()>;

    fn triangles(
        &mut self,
        paint: &PaintPattern,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        vertexes: &[Vertex],
    ) -> anyhow::Result<()>;

    fn clear(&mut self, color: Color) -> anyhow::Result<()>;

    #[cfg(feature = "wireframe")]
    fn wireframe(&mut self, _enable: bool) -> anyhow::Result<()>;

    #[cfg(feature = "wirelines")]
    fn wirelines(
        &mut self,
        _paint: &PaintPattern,
        _composite_operation: CompositeOperationState,
        _scissor: &Scissor,
        _path: &[PathInfo],
    ) -> anyhow::Result<()>;
}
