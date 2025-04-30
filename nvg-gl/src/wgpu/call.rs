use nvg::{BlendFactor, CompositeOperationState, PathFillType};
use wgpu::BlendComponent;

#[derive(PartialEq, Eq)]
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
    pub blend_func: wgpu::BlendState,
    #[cfg(feature = "wireframe")]
    pub wireframe: bool,
}

pub(crate) trait ToBlendState: AsRef<CompositeOperationState> {
    fn to_wgpu_blend_state(&self) -> wgpu::BlendState {
        let value = self.as_ref();
        return wgpu::BlendState {
            color: BlendComponent {
                src_factor: convert_blend_factor(value.src_rgb),
                dst_factor: convert_blend_factor(value.dst_rgb),
                operation: wgpu::BlendOperation::Add,
            },
            alpha: BlendComponent {
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
