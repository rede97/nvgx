use nvgx::Transform;

const INSTANCE_ATTRIBS: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32x2, 4 => Float32x2];

pub const INSTANCE_DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Transform>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &INSTANCE_ATTRIBS,
};
