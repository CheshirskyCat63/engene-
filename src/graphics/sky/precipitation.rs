//! Phase 7: GPU precipitation particle system (rain/snow/hail).

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrecipitationType {
    Rain,
    Snow,
    Hail,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PrecipitationParams {
    pub view_proj: [[f32; 4]; 4],
    pub rain_direction: [f32; 4],
    pub wind_vector: [f32; 4],
    pub camera_pos: [f32; 4],
    pub intensity: f32,
    pub time: f32,
    pub precip_type: u32,
    pub _pad: u32,
}

pub struct PrecipitationConfig {
    pub precipitation_type: PrecipitationType,
    pub intensity: f32,
    pub wind_influence: f32,
    pub particle_count: u32,
}

impl Default for PrecipitationConfig {
    fn default() -> Self {
        Self {
            precipitation_type: PrecipitationType::Rain,
            intensity: 0.0,
            wind_influence: 0.5,
            particle_count: 2000,
        }
    }
}

pub struct PrecipitationSystem {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub params_buffer: wgpu::Buffer,
    pub particle_count: u32,
}

impl PrecipitationSystem {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let params = PrecipitationParams {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            rain_direction: [0.0, -1.0, 0.0, 0.0],
            wind_vector: [0.0, 0.0, 0.0, 0.0],
            camera_pos: [0.0, 0.0, 0.0, 1.0],
            intensity: 0.0,
            time: 0.0,
            precip_type: 0,
            _pad: 0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("precipitation_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("precipitation_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("precipitation_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("precipitation_shader"),
            source: wgpu::ShaderSource::Wgsl(PRECIPITATION_SHADER_WGSL.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("precipitation_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("precipitation_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
            params_buffer,
            particle_count: 2000,
        }
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        params: PrecipitationParams,
    ) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..self.particle_count * 6, 0..1);
    }
}

/// Build PrecipitationParams from common inputs.
/// rain_dir = normalize(gravity + wind_vector)
pub fn build_precipitation_params(
    view_proj: Mat4,
    precipitation_type: PrecipitationType,
    intensity: f32,
    wind_vector: Vec3,
    time: f32,
    camera_pos: Vec3,
) -> PrecipitationParams {
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    let rain_dir = (gravity + wind_vector).normalize_or_zero();

    let precip_type_u32 = match precipitation_type {
        PrecipitationType::Rain => 0,
        PrecipitationType::Snow => 1,
        PrecipitationType::Hail => 2,
    };

    PrecipitationParams {
        view_proj: view_proj.to_cols_array_2d(),
        rain_direction: [rain_dir.x, rain_dir.y, rain_dir.z, 0.0],
        wind_vector: [wind_vector.x, wind_vector.y, wind_vector.z, 0.0],
        camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
        intensity,
        time,
        precip_type: precip_type_u32,
        _pad: 0,
    }
}

const PRECIPITATION_SHADER_WGSL: &str = r#"
struct Params {
    view_proj: mat4x4<f32>,
    rain_direction: vec4<f32>,
    wind_vector: vec4<f32>,
    camera_pos: vec4<f32>,
    intensity: f32,
    time: f32,
    precip_type: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> params: Params;

const GRAVITY: vec3<f32> = vec3<f32>(0.0, -9.81, 0.0);
const GRID_SIZE: u32 = 16u;
const LAYER_STRIDE: u32 = GRID_SIZE * GRID_SIZE;
const PARTICLES_PER_QUAD: u32 = 6u;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) fade: f32,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    let particle_id = vi / PARTICLES_PER_QUAD;
    let corner = vi % PARTICLES_PER_QUAD;

    let layer = particle_id / LAYER_STRIDE;
    let cell = particle_id % LAYER_STRIDE;
    let ix = cell % GRID_SIZE;
    let iz = cell / GRID_SIZE;

    let spacing = 8.0;
    let offset = vec3<f32>(
        f32(ix) * spacing - f32(GRID_SIZE) * spacing * 0.5,
        f32(layer) * 3.0,
        f32(iz) * spacing - f32(GRID_SIZE) * spacing * 0.5,
    );

    let phase = params.time * 2.0 + f32(particle_id) * 0.01;
    let drift = params.rain_direction.xyz * phase * 15.0;
    let world_pos = params.camera_pos.xyz + offset + drift;

    var quad_pos: vec2<f32>;
    if corner == 0u { quad_pos = vec2<f32>(-0.5, -0.5); }
    else if corner == 1u { quad_pos = vec2<f32>(0.5, -0.5); }
    else if corner == 2u { quad_pos = vec2<f32>(-0.5, 0.5); }
    else if corner == 3u { quad_pos = vec2<f32>(-0.5, 0.5); }
    else if corner == 4u { quad_pos = vec2<f32>(0.5, -0.5); }
    else { quad_pos = vec2<f32>(0.5, 0.5); }

    let billboard_size = 0.15;
    let to_cam = params.camera_pos.xyz - world_pos;
    let dist = length(to_cam);
    let right = normalize(cross(params.rain_direction.xyz, vec3<f32>(0.0, 1.0, 0.0)));
    let up = normalize(cross(right, params.rain_direction.xyz));
    let vertex_offset = right * quad_pos.x * billboard_size + up * quad_pos.y * billboard_size;
    let final_pos = world_pos + vertex_offset;

    let clip_pos = params.view_proj * vec4<f32>(final_pos, 1.0);

    let fade = 1.0 - smoothstep(30.0, 80.0, dist);

    var out: VsOut;
    out.clip_pos = clip_pos;
    out.world_pos = world_pos;
    out.uv = quad_pos + vec2<f32>(0.5);
    out.fade = fade * params.intensity;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dist = length(in.world_pos - params.camera_pos.xyz);
    let beer_lambert = exp(-dist * 0.002);
    let alpha = in.fade * beer_lambert;

    var color: vec3<f32>;
    if params.precip_type == 0u {
        color = vec3<f32>(0.7, 0.8, 0.95);
    } else if params.precip_type == 1u {
        color = vec3<f32>(0.95, 0.97, 1.0);
    } else {
        color = vec3<f32>(0.9, 0.92, 0.95);
    }

    return vec4<f32>(color, alpha);
}
"#;
