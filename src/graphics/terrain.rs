use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::world::biome::Biome;
use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use crate::world::heightmap::Heightmap;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

impl TerrainVertex {
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
                format: wgpu::VertexFormat::Float32x3,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS,
        }
    }
}

pub struct TerrainMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl TerrainMesh {
    pub fn build(device: &wgpu::Device, heightmap: &Heightmap, biomes: &[Biome]) -> Self {
        let res = heightmap.resolution;
        let world_size = heightmap.world_size;
        let n = (res + 1) as usize;

        let mut vertices = Vec::with_capacity(n * n);

        for iz in 0..=res {
            for ix in 0..=res {
                let wx = ix as f32 / res as f32 * world_size;
                let wz = iz as f32 / res as f32 * world_size;
                let wy = heightmap.sample(wx, wz);
                let normal = heightmap.normal_at(wx, wz);

                let cx = ((wx / CELL_SIZE) as u32).min(GRID_SIZE - 1);
                let cz = ((wz / CELL_SIZE) as u32).min(GRID_SIZE - 1);
                let bi = (cz * GRID_SIZE + cx) as usize;
                let biome = if bi < biomes.len() { biomes[bi] } else { Biome::Plains };
                let color = biome_color(biome, wy);

                vertices.push(TerrainVertex {
                    position: [wx, wy, wz],
                    normal,
                    color,
                });
            }
        }

        let mut indices: Vec<u32> = Vec::with_capacity((res * res * 6) as usize);
        let w = res + 1;
        for iz in 0..res {
            for ix in 0..res {
                let tl = iz * w + ix;
                let tr = iz * w + ix + 1;
                let bl = (iz + 1) * w + ix;
                let br = (iz + 1) * w + ix + 1;

                indices.push(tl);
                indices.push(bl);
                indices.push(tr);

                indices.push(tr);
                indices.push(bl);
                indices.push(br);
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_vb"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_ib"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = indices.len() as u32;

        Self { vertex_buffer, index_buffer, index_count }
    }
}

fn biome_color(biome: Biome, height: f32) -> [f32; 3] {
    let h_factor = (height / 100.0).clamp(0.0, 1.0);
    match biome {
        Biome::Forest => [0.10 + h_factor * 0.05, 0.35 + h_factor * 0.05, 0.08],
        Biome::Plains => [0.35 + h_factor * 0.1, 0.55 + h_factor * 0.08, 0.15],
        Biome::Swamp => [0.20, 0.22 + h_factor * 0.05, 0.25],
        Biome::Hills => [0.40 + h_factor * 0.15, 0.32 + h_factor * 0.1, 0.18],
        Biome::Settlement => [0.55, 0.52, 0.48],
    }
}
