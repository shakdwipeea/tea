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

// Changed
const VERTICES: &[VertexData] = &[
    VertexData {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.99240386],
    }, // A
    VertexData {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.56958647],
    }, // B
    VertexData {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.05060294],
    }, // C
    VertexData {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.1526709],
    }, // D
    VertexData {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.7347359],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

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
