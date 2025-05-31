use cgmath::{InnerSpace, Matrix4, Rotation3, Zero};
use wgpu::util::DeviceExt;
use rand::Rng;

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    rotation_speed: f32,
    rotation_axis: cgmath::Vector3<f32>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
        }
    }
}

pub struct InstanceState {
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
}

impl InstanceState {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut instances = Vec::new();
        let mut rng = rand::rng();
        
        for z in 0..NUM_INSTANCES_PER_ROW {
            for x in 0..NUM_INSTANCES_PER_ROW {
                let position = cgmath::Vector3 {
                    x: x as f32 * 2.0,
                    y: 0.0,
                    z: z as f32 * 2.0,
                } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    )
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                // Generate random rotation axis for each instance
                let rotation_axis = cgmath::Vector3::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                ).normalize();

                instances.push(Instance { 
                    position, 
                    rotation,
                    rotation_speed: 20.0, // 20 degrees per frame
                    rotation_axis,
                });
            }
        }

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            instances,
            instance_buffer,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        // Update rotation for each instance
        for instance in &mut self.instances {
            let rotation_delta = cgmath::Quaternion::from_axis_angle(
                instance.rotation_axis,
                cgmath::Deg(instance.rotation_speed)
            );
            instance.rotation = rotation_delta * instance.rotation;
        }

        // Update the buffer with new instance data
        let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
    }

    pub fn num_instances(&self) -> u32 {
        self.instances.len() as u32
    }
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 2.0 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 2.0 * 0.5,
);
