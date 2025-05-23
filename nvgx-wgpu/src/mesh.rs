use std::iter::FromIterator;

use nvgx::Vertex;
use wgpu::vertex_attr_array;

const VERTEX_ATTRIBS: [wgpu::VertexAttribute; 2] = vertex_attr_array![
    0 => Float32x2,
    1 => Float32x2,
];

pub const VERTEX_DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as _,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &VERTEX_ATTRIBS,
};

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices: Vec<u32>,
}

impl Mesh {
    const INDEX_SIZE: u64 = std::mem::size_of::<u32>() as u64 * 3;

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, init_num_vertex: usize) -> Self {
        let indices =
            Vec::from_iter((0..(init_num_vertex * 3) as u32).map(Self::triangle_fan_indices));
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("NVG New Index Buffer"),
            size: (indices.len() * size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

        Self {
            vertex_buffer: Mesh::create_buffer(device, init_num_vertex),
            index_buffer,
            indices,
        }
    }

    fn triangle_fan_indices(i: u32) -> u32 {
        let v = i / 3;
        return match i % 3 {
            0 => 0,
            1 => v + 1,
            _ => v + 2,
        };
    }

    #[inline]
    pub fn update_indices(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertex_count: u64,
    ) {
        let indices_len = self.indices.len() as u32;
        let new_indices_len = (vertex_count * 3) as u32;
        if indices_len < new_indices_len {
            // update indices buffer: 0-1-2, 0-2-3, 0-3-4, 0-4-5, 0-5-6, ...
            self.indices
                .extend((indices_len..new_indices_len).map(Self::triangle_fan_indices));

            self.index_buffer.destroy();
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("NVG Expand Index Buffer"),
                size: vertex_count * Self::INDEX_SIZE,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }
    }

    #[inline]
    pub fn create_buffer(device: &wgpu::Device, buffer_size: usize) -> wgpu::Buffer {
        let buffer_size = align_size::<1024>(buffer_size);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("NVG Create New Vertex Buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        return buffer;
    }

    #[inline]
    pub fn update_buffer(
        &mut self,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        data: &[u8],
    ) -> anyhow::Result<()> {
        if buffer.size() < data.len() as u64 {
            Err(anyhow!("Vertex buffer out of memory"))
        } else {
            queue.write_buffer(&buffer, 0, data);
            Ok(())
        }
    }

    #[inline]
    pub fn update_inner_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        if self.vertex_buffer.size() < data.len() as u64 {
            self.vertex_buffer.destroy();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("NVG Expand Vertex Buffer"),
                size: data.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.vertex_buffer, 0, data);
    }
}

fn align_size<const S: usize>(offset: usize) -> usize {
    assert!(S.is_power_of_two());
    (offset + S - 1) & !(S - 1)
}
