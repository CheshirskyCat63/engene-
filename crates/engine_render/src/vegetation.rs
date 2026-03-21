use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::world::biome::Biome;
use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use crate::world::heightmap::Heightmap;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GrassInstance {
    pub position: [f32; 3],
    pub rotation_scale: [f32; 2],
}

impl GrassInstance {
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
                format: wgpu::VertexFormat::Float32x2,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GrassInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GrassVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl GrassVertex {
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
                format: wgpu::VertexFormat::Float32x2,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GrassVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS,
        }
    }
}

pub struct VegetationSystem {
    pub grass_quad_vb: wgpu::Buffer,
    pub grass_quad_ib: wgpu::Buffer,
    pub grass_instance_buf: wgpu::Buffer,
    pub grass_count: u32,
    pub tree_instance_buf: wgpu::Buffer,
    pub tree_count: u32,
    pub pipeline: wgpu::RenderPipeline,
}

impl VegetationSystem {
    pub fn new(
        device: &wgpu::Device,
        camera_bgl: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
        heightmap: &Heightmap,
        biomes: &[Biome],
    ) -> Self {
        let (quad_verts, quad_indices) = grass_quad();
        let grass_quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grass_quad_vb"),
            contents: bytemuck::cast_slice(&quad_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let grass_quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grass_quad_ib"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let (grass_instances, tree_instances) = scatter_vegetation(heightmap, biomes);

        let grass_count = grass_instances.len() as u32;
        let grass_instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grass_instances"),
            contents: bytemuck::cast_slice(&grass_instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let tree_count = tree_instances.len() as u32;
        let tree_buf_data = if tree_instances.is_empty() {
            vec![GrassInstance {
                position: [0.0; 3],
                rotation_scale: [0.0; 2],
            }]
        } else {
            tree_instances
        };
        let tree_instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tree_instances"),
            contents: bytemuck::cast_slice(&tree_buf_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vegetation_shader"),
            source: wgpu::ShaderSource::Wgsl(VEGETATION_SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vegetation_layout"),
            bind_group_layouts: &[camera_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vegetation_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[GrassVertex::layout(), GrassInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview: None,
        });

        Self {
            grass_quad_vb,
            grass_quad_ib,
            grass_instance_buf,
            grass_count,
            tree_instance_buf,
            tree_count,
            pipeline,
        }
    }
}

fn grass_quad() -> (Vec<GrassVertex>, Vec<u32>) {
    let w = 0.3;
    let h = 0.6;
    let verts = vec![
        GrassVertex {
            position: [-w, 0.0, 0.0],
            uv: [0.0, 1.0],
        },
        GrassVertex {
            position: [w, 0.0, 0.0],
            uv: [1.0, 1.0],
        },
        GrassVertex {
            position: [w, h, 0.0],
            uv: [1.0, 0.0],
        },
        GrassVertex {
            position: [-w, h, 0.0],
            uv: [0.0, 0.0],
        },
        GrassVertex {
            position: [0.0, 0.0, -w],
            uv: [0.0, 1.0],
        },
        GrassVertex {
            position: [0.0, 0.0, w],
            uv: [1.0, 1.0],
        },
        GrassVertex {
            position: [0.0, h, w],
            uv: [1.0, 0.0],
        },
        GrassVertex {
            position: [0.0, h, -w],
            uv: [0.0, 0.0],
        },
    ];
    let indices = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];
    (verts, indices)
}

fn scatter_vegetation(
    heightmap: &Heightmap,
    biomes: &[Biome],
) -> (Vec<GrassInstance>, Vec<GrassInstance>) {
    let ws = GRID_SIZE as f32 * CELL_SIZE;
    let mut grass = Vec::new();
    let mut trees = Vec::new();

    let grass_spacing = 8.0;
    let tree_spacing = 40.0;

    let mut x = 0.0f32;
    while x < ws {
        let mut z = 0.0f32;
        while z < ws {
            let bx = ((x / ws) * GRID_SIZE as f32) as usize;
            let bz = ((z / ws) * GRID_SIZE as f32) as usize;
            let bi = (bz * GRID_SIZE as usize + bx).min(biomes.len().saturating_sub(1));
            let biome = if bi < biomes.len() {
                biomes[bi]
            } else {
                Biome::Plains
            };

            let density = match biome {
                Biome::Forest => 1.0,
                Biome::Plains => 0.7,
                Biome::Swamp => 0.3,
                Biome::Hills => 0.2,
                Biome::Settlement => 0.05,
            };

            let hash = pseudo_hash(x as u32, z as u32);
            if (hash as f32 / u32::MAX as f32) < density {
                let y = heightmap.sample(x, z);
                let rot = (hash as f32 / u32::MAX as f32) * std::f32::consts::TAU;
                let scale = 0.8 + (hash as f32 / u32::MAX as f32) * 0.4;
                grass.push(GrassInstance {
                    position: [x, y, z],
                    rotation_scale: [rot, scale],
                });
            }
            z += grass_spacing;
        }
        x += grass_spacing;
    }

    x = 0.0;
    while x < ws {
        let mut z = 0.0f32;
        while z < ws {
            let bx = ((x / ws) * GRID_SIZE as f32) as usize;
            let bz = ((z / ws) * GRID_SIZE as f32) as usize;
            let bi = (bz * GRID_SIZE as usize + bx).min(biomes.len().saturating_sub(1));
            let biome = if bi < biomes.len() {
                biomes[bi]
            } else {
                Biome::Plains
            };

            let tree_density = match biome {
                Biome::Forest => 0.6,
                Biome::Swamp => 0.2,
                Biome::Plains => 0.1,
                _ => 0.0,
            };

            let hash = pseudo_hash(x as u32 + 7919, z as u32 + 6271);
            if (hash as f32 / u32::MAX as f32) < tree_density {
                let y = heightmap.sample(x, z);
                let rot = (hash as f32 / u32::MAX as f32) * std::f32::consts::TAU;
                let scale = 3.0 + (hash as f32 / u32::MAX as f32) * 4.0;
                trees.push(GrassInstance {
                    position: [x, y, z],
                    rotation_scale: [rot, scale],
                });
            }
            z += tree_spacing;
        }
        x += tree_spacing;
    }

    (grass, trees)
}

fn pseudo_hash(a: u32, b: u32) -> u32 {
    let mut h = a.wrapping_mul(73856093) ^ b.wrapping_mul(19349663);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

const VEGETATION_SHADER: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) inst_pos: vec3<f32>,
    @location(3) rot_scale: vec2<f32>,
};

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) height_factor: f32,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let rot = in.rot_scale.x;
    let scale = in.rot_scale.y;
    let c = cos(rot);
    let s = sin(rot);
    var rotated = vec3<f32>(
        in.position.x * c - in.position.z * s,
        in.position.y,
        in.position.x * s + in.position.z * c,
    );
    // Wind animation: top vertices sway based on world position + implicit time
    let height_blend = 1.0 - in.uv.y; // 1 at top, 0 at base
    let phase = in.inst_pos.x * 0.07 + in.inst_pos.z * 0.09;
    let wind_str = height_blend * height_blend * 0.4 * scale;
    rotated.x += sin(phase + in.inst_pos.x * 0.3) * wind_str;
    rotated.z += cos(phase + in.inst_pos.z * 0.2) * wind_str * 0.7;
    let world_pos = rotated * scale + in.inst_pos;
    var out: VsOut;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = in.uv;
    out.height_factor = in.uv.y;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let base_green = vec3<f32>(0.15, 0.45, 0.1);
    let tip_green = vec3<f32>(0.3, 0.7, 0.15);
    let color = mix(base_green, tip_green, 1.0 - in.height_factor);
    let alpha = 1.0 - in.height_factor * 0.3;
    return vec4<f32>(color, alpha);
}
"#;
