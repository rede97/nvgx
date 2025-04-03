#[derive(Debug, Copy, Clone)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturate,
}

#[derive(Debug, Copy, Clone)]
pub enum BasicCompositeOperation {
    SrcOver,
    SrcIn,
    SrcOut,
    Atop,
    DstOver,
    DstIn,
    DstOut,
    DstAtop,
    Lighter,
    Copy,
    Xor,
}

#[derive(Debug, Copy, Clone)]
pub enum CompositeOperation {
    Basic(BasicCompositeOperation),
    BlendFunc {
        src: BlendFactor,
        dst: BlendFactor,
    },
    BlendFuncSeparate {
        src_rgb: BlendFactor,
        dst_rgb: BlendFactor,
        src_alpha: BlendFactor,
        dst_alpha: BlendFactor,
    },
}

impl Into<CompositeOperationState> for CompositeOperation {
    fn into(self) -> CompositeOperationState {
        match self {
            CompositeOperation::Basic(op) => {
                let (src_factor, dst_factor) = match op {
                    BasicCompositeOperation::SrcOver => {
                        (BlendFactor::One, BlendFactor::OneMinusSrcAlpha)
                    }
                    BasicCompositeOperation::SrcIn => (BlendFactor::DstAlpha, BlendFactor::Zero),
                    BasicCompositeOperation::SrcOut => {
                        (BlendFactor::OneMinusDstAlpha, BlendFactor::Zero)
                    }
                    BasicCompositeOperation::Atop => {
                        (BlendFactor::DstAlpha, BlendFactor::OneMinusSrcAlpha)
                    }
                    BasicCompositeOperation::DstOver => {
                        (BlendFactor::OneMinusDstAlpha, BlendFactor::One)
                    }
                    BasicCompositeOperation::DstIn => (BlendFactor::Zero, BlendFactor::SrcAlpha),
                    BasicCompositeOperation::DstOut => {
                        (BlendFactor::Zero, BlendFactor::OneMinusSrcAlpha)
                    }
                    BasicCompositeOperation::DstAtop => {
                        (BlendFactor::OneMinusDstAlpha, BlendFactor::SrcAlpha)
                    }
                    BasicCompositeOperation::Lighter => (BlendFactor::One, BlendFactor::One),
                    BasicCompositeOperation::Copy => (BlendFactor::One, BlendFactor::Zero),
                    BasicCompositeOperation::Xor => {
                        (BlendFactor::OneMinusDstAlpha, BlendFactor::OneMinusSrcAlpha)
                    }
                };

                CompositeOperationState {
                    src_rgb: src_factor,
                    dst_rgb: dst_factor,
                    src_alpha: src_factor,
                    dst_alpha: dst_factor,
                }
            }
            CompositeOperation::BlendFunc { src, dst } => CompositeOperationState {
                src_rgb: src,
                dst_rgb: dst,
                src_alpha: src,
                dst_alpha: dst,
            },
            CompositeOperation::BlendFuncSeparate {
                src_rgb,
                dst_rgb,
                src_alpha,
                dst_alpha,
            } => CompositeOperationState {
                src_rgb,
                dst_rgb,
                src_alpha,
                dst_alpha,
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CompositeOperationState {
    pub src_rgb: BlendFactor,
    pub dst_rgb: BlendFactor,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
}
