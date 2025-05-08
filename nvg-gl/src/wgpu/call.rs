use std::ops::Range;

use nvg::{BlendFactor, CompositeOperationState, PathFillType, VertexSlice};

use super::unifroms::RenderCommand;

#[derive(PartialEq, Debug)]
pub(crate) enum CallType {
    Fill(PathFillType),
    ConvexFill,
    Stroke,
    Triangles,
    Lines,
}

pub(crate) struct Call {
    pub call_type: CallType,
    pub image: Option<usize>,
    pub path_range: Range<usize>,
    pub triangle: VertexSlice,
    pub uniform_offset: usize,
    pub blend_func: CompositeOperationState,
    pub vertex_buffer: wgpu::Buffer,
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

pub(crate) trait ToBlendState: AsRef<CompositeOperationState> {
    fn to_wgpu_blend_state(&self) -> wgpu::BlendState {
        let value = self.as_ref();
        return wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: convert_blend_factor(value.src_rgb),
                dst_factor: convert_blend_factor(value.dst_rgb),
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: convert_blend_factor(value.src_alpha),
                dst_factor: convert_blend_factor(value.dst_alpha),
                operation: wgpu::BlendOperation::Add,
            },
        };
    }
}

fn convert_blend_factor(factor: BlendFactor) -> wgpu::BlendFactor {
    match factor {
        BlendFactor::Zero => wgpu::BlendFactor::Zero,
        BlendFactor::One => wgpu::BlendFactor::One,
        BlendFactor::SrcColor => wgpu::BlendFactor::Src,
        BlendFactor::OneMinusSrcColor => wgpu::BlendFactor::OneMinusSrc,
        BlendFactor::DstColor => wgpu::BlendFactor::Dst,
        BlendFactor::OneMinusDstColor => wgpu::BlendFactor::OneMinusDst,
        BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
        BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
        BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
        BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
        BlendFactor::SrcAlphaSaturate => wgpu::BlendFactor::SrcAlphaSaturated,
    }
}

impl ToBlendState for &CompositeOperationState {}

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
