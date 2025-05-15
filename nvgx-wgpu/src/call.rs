use std::{ops::Range, sync::Arc};

use nvgx::{CompositeOperationState, PathFillType, VertexSlice};

use super::unifroms::RenderCommand;

#[derive(PartialEq, Debug)]
pub(crate) enum CallType {
    Fill(PathFillType),
    ConvexFill,
    Stroke,
    Triangles,
    #[cfg(feature = "wirelines")]
    Lines,
}

pub(crate) struct Call {
    pub call_type: CallType,
    pub image: Option<usize>,
    pub path_range: Range<usize>,
    pub triangle: VertexSlice,
    pub uniform_offset: usize,
    pub blend_func: CompositeOperationState,
    pub vertex_buffer: Option<Arc<wgpu::Buffer>>,
    pub instances: Option<(Arc<wgpu::Buffer>, Range<u32>)>,
}

impl Call {
    #[inline]
    pub fn triangle_vert(&self) -> Range<u32> {
        let start = self.triangle.offset as u32;
        let end = (self.triangle.offset + self.triangle.count) as u32;
        return start..end;
    }

    #[inline]
    pub fn uniform_offset(&self, offset: usize) -> u32 {
        ((self.uniform_offset + offset) * size_of::<RenderCommand>()) as u32
    }
}

#[derive(Default)]
pub(crate) struct GpuPath {
    pub fill: VertexSlice,
    pub stroke: VertexSlice,
}

impl GpuPath {
    #[inline]
    pub fn triangle_fan_offset(&self) -> i32 {
        return self.fill.offset as i32;
    }

    #[inline]
    pub fn triangle_fan_count(&self) -> u32 {
        assert!(self.fill.count > 2);
        return (self.fill.count - 2) as u32;
    }

    #[inline]
    pub fn stroke_vert(&self) -> Range<u32> {
        let start = self.stroke.offset as u32;
        let end = (self.stroke.offset + self.stroke.count) as u32;
        return start..end;
    }
}
