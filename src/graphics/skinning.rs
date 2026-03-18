use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub const MAX_JOINTS: usize = 64;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SkinVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub joints: [u32; 4],
    pub weights: [f32; 4],
}

impl SkinVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        static ATTRS: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 24,
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32x4,
            },
            wgpu::VertexAttribute {
                offset: 40,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkinVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct JointMatricesUniform {
    pub matrices: [[[f32; 4]; 4]; MAX_JOINTS],
}

impl JointMatricesUniform {
    pub fn identity() -> Self {
        let id = glam::Mat4::IDENTITY.to_cols_array_2d();
        Self {
            matrices: [id; MAX_JOINTS],
        }
    }

    pub fn from_mats(mats: &[glam::Mat4]) -> Self {
        let mut u = Self::identity();
        for (i, m) in mats.iter().enumerate().take(MAX_JOINTS) {
            u.matrices[i] = m.to_cols_array_2d();
        }
        u
    }
}

pub struct SkinBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl SkinBuffer {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("joint_matrices"),
            contents: bytemuck::bytes_of(&JointMatricesUniform::identity()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("skin_bg"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self { buffer, bind_group }
    }

    pub fn update(&self, queue: &wgpu::Queue, mats: &[glam::Mat4]) {
        let uniform = JointMatricesUniform::from_mats(mats);
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }
}
