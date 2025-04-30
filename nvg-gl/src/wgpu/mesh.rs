use std::iter::FromIterator;

use nvg::Vertex;
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
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    const VERTEX_SIZE: u64 = std::mem::size_of::<Vertex>() as u64;
    const INDEX_SIZE: u64 = std::mem::size_of::<u32>() as u64 * 3;

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, init_vertex_size: u64) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: init_vertex_size * size_of::<Vertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: init_vertex_size * 3 * size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices =
            Vec::from_iter((0..init_vertex_size as u32 * 3).map(Self::triangle_fan_indices));
        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

        Self {
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(init_vertex_size as usize),
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
    fn vertex_free_space(&self) -> u64 {
        return self.vertex_buffer.size();
    }

    #[allow(unused)]
    pub fn update_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let vertex_count = self.vertices.len() as u64;
        if self.vertex_free_space() < vertex_count * Self::VERTEX_SIZE {
            self.vertex_buffer.destroy();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: vertex_count * Self::VERTEX_SIZE,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.index_buffer.destroy();
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                size: vertex_count * Self::INDEX_SIZE,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // update indices buffer: 0-1-2, 0-2-3, 0-3-4, 0-4-5, 0-5-6, ...
            let indices_len = self.indices.len() as u32;
            let new_indices_len = (vertex_count * 3) as u32;
            self.indices
                .extend((indices_len..new_indices_len).map(Self::triangle_fan_indices));
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }

    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
    }
}
