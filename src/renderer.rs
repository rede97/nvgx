pub use crate::context::{CompositeOperationState, ImageId};
pub use crate::paint::PaintPattern;
pub use crate::path::cache::{PathInfo, Vertex, VertexSlice};
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

pub trait RendererDevice {
    type VertexBuffer;
    fn edge_antialias(&self) -> bool;

    fn resize(&mut self, _width: u32, _height: u32) -> anyhow::Result<()> {
        Ok(())
    }

    fn create_vertex_buffer(
        &mut self,
        init_num_vertex: usize,
    ) -> anyhow::Result<Self::VertexBuffer>;

    fn update_vertex_buffer(
        &mut self,
        buffer: Option<&Self::VertexBuffer>,
        vertexes: &[Vertex],
    ) -> anyhow::Result<()>;

    fn create_texture(
        &mut self,
        texture_type: TextureType,
        width: u32,
        height: u32,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<ImageId>;

    fn delete_texture(&mut self, img: ImageId) -> anyhow::Result<()>;

    fn update_texture(
        &mut self,
        img: ImageId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> anyhow::Result<()>;

    fn texture_size(&self, img: ImageId) -> anyhow::Result<(u32, u32)>;

    fn viewport(&mut self, extent: Extent, device_pixel_ratio: f32) -> anyhow::Result<()>;

    fn cancel(&mut self) -> anyhow::Result<()>;

    fn flush(&mut self) -> anyhow::Result<()>;

    fn fill(
        &mut self,
        vertex_buffer: Option<&Self::VertexBuffer>,
        paint: &PaintPattern,
        composite_operation: CompositeOperationState,
        fill_type: PathFillType,
        scissor: &Scissor,
        fringe: f32,
        bounds_offset: Option<usize>,
        paths: &[PathInfo],
    ) -> anyhow::Result<()>;

    fn stroke(
        &mut self,
        vertex_buffer: Option<&Self::VertexBuffer>,
        paint: &PaintPattern,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[PathInfo],
    ) -> anyhow::Result<()>;

    fn triangles(
        &mut self,
        vertex_buffer: Option<&Self::VertexBuffer>,
        paint: &PaintPattern,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        slice: VertexSlice,
    ) -> anyhow::Result<()>;

    #[cfg(feature = "wirelines")]
    fn wirelines(
        &mut self,
        vertex_buffer: Option<&Self::VertexBuffer>,
        paint: &PaintPattern,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        paths: &[PathInfo],
    ) -> anyhow::Result<()>;

    fn clear(&mut self, color: Color) -> anyhow::Result<()>;
}

pub trait FrameBufferDevice {
    fn size(&self) -> Extent;
    fn image(&self) -> ImageId;
}

pub trait RenderFrameBufferDevice: RendererDevice {
    type FB: FrameBufferDevice;
    fn create_fb(&mut self, width: u32, height: u32, image: ImageId) -> anyhow::Result<Self::FB>;
    fn delete_fb(&mut self, fb: Self::FB) -> anyhow::Result<()>;
    fn bind(&mut self, fb: &Self::FB) -> anyhow::Result<()>;
    fn unbind(&mut self) -> anyhow::Result<()>;
}
