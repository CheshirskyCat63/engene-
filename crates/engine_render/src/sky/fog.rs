//! Phase 9: Terrain-aware volumetric fog system.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

pub struct FogConfig {
    pub base_density: f32,
    pub height_falloff: f32,
    pub terrain_basin_strength: f32,
    pub wind_dispersion: f32,
    pub dawn_boost: f32,
    pub climate_multiplier: f32,
    pub interior_factor: f32,
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            base_density: 0.003,
            height_falloff: 0.02,
            terrain_basin_strength: 1.5,
            wind_dispersion: 0.5,
            dawn_boost: 2.0,
            climate_multiplier: 1.0,
            interior_factor: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct FogParams {
    pub camera_pos: [f32; 4],
    pub sun_direction: [f32; 4],
    pub fog_color: [f32; 4],
    pub base_density: f32,
    pub height_falloff: f32,
    pub terrain_basin_strength: f32,
    pub wind_dispersion: f32,
    pub dawn_boost: f32,
    pub climate_multiplier: f32,
    pub interior_factor: f32,
    pub day_progress: f32,
    pub wind_speed: f32,
    pub fog_max_opacity: f32,
    pub _align_pad: [f32; 2],
    pub inv_view_proj: [[f32; 4]; 4],
    pub _pad: [f32; 4],
}

pub struct FogSystem {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub params_buffer: wgpu::Buffer,
}

impl FogSystem {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let params = FogParams {
            camera_pos: [0.0, 0.0, 0.0, 1.0],
            sun_direction: [0.0, 1.0, 0.0, 0.0],
            fog_color: [0.6, 0.65, 0.75, 1.0],
            base_density: 0.003,
            height_falloff: 0.02,
            terrain_basin_strength: 1.5,
            wind_dispersion: 0.5,
            dawn_boost: 2.0,
            climate_multiplier: 1.0,
            interior_factor: 1.0,
            day_progress: 0.5,
            wind_speed: 0.0,
            fog_max_opacity: 0.85,
            _align_pad: [0.0; 2],
            inv_view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            _pad: [0.0; 4],
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fog_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fog_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fog_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fog_shader"),
            source: wgpu::ShaderSource::Wgsl(FOG_SHADER_WGSL.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fog_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fog_pipeline"),
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
        }
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        config: &FogConfig,
        camera_pos: Vec3,
        sun_dir: Vec3,
        day_progress: f32,
        wind_speed: f32,
        inv_view_proj: Mat4,
    ) {
        let params = FogParams {
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
            sun_direction: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0],
            fog_color: [0.6, 0.65, 0.75, 1.0],
            base_density: config.base_density,
            height_falloff: config.height_falloff,
            terrain_basin_strength: config.terrain_basin_strength,
            wind_dispersion: config.wind_dispersion,
            dawn_boost: config.dawn_boost,
            climate_multiplier: config.climate_multiplier,
            interior_factor: config.interior_factor,
            day_progress,
            wind_speed,
            fog_max_opacity: 0.85,
            _align_pad: [0.0; 2],
            inv_view_proj: inv_view_proj.to_cols_array_2d(),
            _pad: [0.0; 4],
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

const FOG_SHADER_WGSL: &str = r#"
struct FogParams {
    camera_pos: vec4<f32>,
    sun_direction: vec4<f32>,
    fog_color: vec4<f32>,
    base_density: f32,
    height_falloff: f32,
    terrain_basin_strength: f32,
    wind_dispersion: f32,
    dawn_boost: f32,
    climate_multiplier: f32,
    interior_factor: f32,
    day_progress: f32,
    wind_speed: f32,
    fog_max_opacity: f32,
    _align_pad: vec2<f32>,
    inv_view_proj: mat4x4<f32>,
    _pad: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: FogParams;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let pos = positions[vi];
    var out: VsOut;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let ndc = vec4<f32>(in.uv * 2.0 - vec2<f32>(1.0), 0.99, 1.0);
    let world_h = params.inv_view_proj * ndc;
    let world_pos = world_h.xyz / world_h.w;
    let dist = length(world_pos - params.camera_pos.xyz);

    let camera_height = params.camera_pos.y;

    let terrain_basin_factor = 1.0 + params.terrain_basin_strength * max(0.0, 1.0 - camera_height * 0.01);
    let temperature_inversion_factor = 1.0 + 0.5 * (1.0 - abs(params.day_progress - 0.5) * 2.0);
    let wind_dispersion_factor = 1.0 / (1.0 + params.wind_speed * params.wind_dispersion);
    let dawn = smoothstep(0.2, 0.3, params.day_progress) * (1.0 - smoothstep(0.3, 0.4, params.day_progress));
    let dusk = smoothstep(0.6, 0.7, params.day_progress) * (1.0 - smoothstep(0.7, 0.8, params.day_progress));
    let time_of_day_factor = 1.0 + params.dawn_boost * (dawn + dusk);

    let fog_density = params.base_density
        * terrain_basin_factor
        * temperature_inversion_factor
        * wind_dispersion_factor
        * time_of_day_factor
        * params.climate_multiplier
        * params.interior_factor;

    let height_factor = exp(-camera_height * params.height_falloff);
    let density = fog_density * height_factor;

    let aerial = 1.0 - exp(-dist * density);
    let fog_amount = min(aerial, params.fog_max_opacity);

    let sun_dir = normalize(params.sun_direction.xyz);
    let view_dir = normalize(world_pos - params.camera_pos.xyz);
    let sun_contrib = max(0.0, dot(view_dir, sun_dir)) * 0.2;
    var fog_color = params.fog_color.rgb + vec3<f32>(sun_contrib, sun_contrib * 0.95, sun_contrib * 0.9);

    return vec4<f32>(fog_color, fog_amount);
}
"#;
