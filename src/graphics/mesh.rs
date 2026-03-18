use std::f32::consts::{FRAC_PI_2, PI};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct EntityInstance {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl MeshVertex {
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
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS,
        }
    }
}

impl EntityInstance {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        static ATTRS: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x3,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<EntityInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

pub struct CapsuleMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl CapsuleMesh {
    pub fn new(
        device: &wgpu::Device,
        radius: f32,
        half_body: f32,
        slices: u32,
        cap_stacks: u32,
    ) -> Self {
        let (verts, indices) = generate_capsule(radius, half_body, slices, cap_stacks);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("capsule_vb"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("capsule_ib"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}

fn generate_capsule(
    radius: f32,
    half_body: f32,
    slices: u32,
    cap_stacks: u32,
) -> (Vec<MeshVertex>, Vec<u32>) {
    let mut verts = Vec::new();
    let mut idxs = Vec::new();

    // Bottom pole
    verts.push(MeshVertex {
        position: [0.0, -(half_body + radius), 0.0],
        normal: [0.0, -1.0, 0.0],
    });

    let total_rings = 2 * cap_stacks + 1;

    for ring in 0..total_rings {
        let (y, ring_r, ny) = if ring < cap_stacks {
            let t = (ring + 1) as f32 / cap_stacks as f32;
            let theta = -FRAC_PI_2 * (1.0 - t);
            (
                -half_body + radius * theta.sin(),
                radius * theta.cos(),
                theta.sin(),
            )
        } else if ring == cap_stacks {
            (half_body, radius, 0.0)
        } else {
            let top_i = ring - cap_stacks;
            let t = top_i as f32 / cap_stacks as f32;
            let theta = FRAC_PI_2 * t;
            (
                half_body + radius * theta.sin(),
                radius * theta.cos(),
                theta.sin(),
            )
        };

        for s in 0..slices {
            let phi = 2.0 * PI * s as f32 / slices as f32;
            let (cp, sp) = (phi.cos(), phi.sin());
            let x = ring_r * cp;
            let z = ring_r * sp;
            let len = (cp * cp + ny * ny + sp * sp).sqrt().max(0.001);
            verts.push(MeshVertex {
                position: [x, y, z],
                normal: [cp / len, ny / len, sp / len],
            });
        }
    }

    // Top pole
    verts.push(MeshVertex {
        position: [0.0, half_body + radius, 0.0],
        normal: [0.0, 1.0, 0.0],
    });

    // Bottom pole fan
    for i in 0..slices {
        idxs.push(0);
        idxs.push(1 + (i + 1) % slices);
        idxs.push(1 + i);
    }

    // Ring strips
    for ring in 0..(total_rings - 1) {
        let base = 1 + ring * slices;
        let next = base + slices;
        for i in 0..slices {
            let ni = (i + 1) % slices;
            idxs.push(base + i);
            idxs.push(next + i);
            idxs.push(base + ni);

            idxs.push(base + ni);
            idxs.push(next + i);
            idxs.push(next + ni);
        }
    }

    // Top pole fan
    let top = verts.len() as u32 - 1;
    let last_ring = top - slices;
    for i in 0..slices {
        idxs.push(top);
        idxs.push(last_ring + i);
        idxs.push(last_ring + (i + 1) % slices);
    }

    (verts, idxs)
}
