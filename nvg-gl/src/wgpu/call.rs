use std::ops::Range;

use nvg::{BlendFactor, CompositeOperationState, PathFillType, Vertex};

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
    pub path_offset: usize,
    pub path_count: usize,
    pub triangle_offset: usize,
    pub triangle_count: usize,
    pub uniform_offset: usize,
    pub blend_func: CompositeOperationState,
    #[cfg(feature = "wireframe")]
    pub wireframe: bool,
}

impl Default for Call {
    fn default() -> Self {
        Self {
            call_type: CallType::ConvexFill,
            image: None,
            path_offset: 0,
            path_count: 0,
            triangle_offset: 0,
            triangle_count: 0,
            uniform_offset: 0,
            blend_func: CompositeOperationState::default(),
            #[cfg(feature = "wireframe")]
            wireframe: false,
        }
    }
}

impl Call {
    #[inline]
    pub fn triangle_slice(&self) -> Range<u64> {
        let start = (self.triangle_offset * size_of::<Vertex>()) as u64;
        let end = ((self.triangle_offset + self.triangle_count) * size_of::<Vertex>()) as u64;
        return start..end;
    }

    #[inline]
    pub fn triangle_count(&self) -> u32 {
        self.triangle_count as u32
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
    pub fill_offset: usize,
    pub fill_count: usize,
    pub stroke_offset: usize,
    pub stroke_count: usize,
}

impl GpuPath {
    #[inline]
    pub fn triangle_fan_slice(&self) -> Range<u64> {
        let start = (self.fill_offset * size_of::<Vertex>()) as u64;
        let end = ((self.fill_offset + self.fill_count) * size_of::<Vertex>()) as u64;
        return start..end;
    }

    #[inline]
    pub fn triangle_fan_count(&self) -> u32 {
        (self.fill_count - 2) as u32
    }

    #[inline]
    pub fn stroke_slice(&self) -> Range<u64> {
        let start = (self.stroke_offset * size_of::<Vertex>()) as u64;
        let end = ((self.stroke_offset + self.stroke_count) * size_of::<Vertex>()) as u64;
        return start..end;
    }

    #[inline]
    pub fn stroke_count(&self) -> u32 {
        self.stroke_count as u32
    }
}
