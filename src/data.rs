use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexData {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl VertexData {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Cube vertices
const VERTICES: &[VertexData] = &[
    // Front face
    VertexData {
        position: [-0.5, -0.5,  0.5],
        tex_coords: [0.0, 0.0],
    }, // 0: front bottom left
    VertexData {
        position: [ 0.5, -0.5,  0.5],
        tex_coords: [1.0, 0.0],
    }, // 1: front bottom right
    VertexData {
        position: [ 0.5,  0.5,  0.5],
        tex_coords: [1.0, 1.0],
    }, // 2: front top right
    VertexData {
        position: [-0.5,  0.5,  0.5],
        tex_coords: [0.0, 1.0],
    }, // 3: front top left

    // Back face
    VertexData {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // 4: back bottom left
    VertexData {
        position: [ 0.5, -0.5, -0.5],
        tex_coords: [0.0, 0.0],
    }, // 5: back bottom right
    VertexData {
        position: [ 0.5,  0.5, -0.5],
        tex_coords: [0.0, 1.0],
    }, // 6: back top right
    VertexData {
        position: [-0.5,  0.5, -0.5],
        tex_coords: [1.0, 1.0],
    }, // 7: back top left
];

const INDICES: &[u16] = &[
    // Front face
    0, 1, 2,  2, 3, 0,
    // Back face
    4, 5, 6,  6, 7, 4,
    // Left face
    7, 3, 0,  0, 4, 7,
    // Right face
    1, 5, 6,  6, 2, 1,
    // Bottom face
    4, 0, 1,  1, 5, 4,
    // Top face
    3, 7, 6,  6, 2, 3,
];

pub struct VertexState {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
}

impl VertexState {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: "vertex_buffer".into(),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: "index_buffer".into(),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }),
            num_vertices: VERTICES.len() as u32,
            num_indices: INDICES.len() as u32,
        }
    }
}
