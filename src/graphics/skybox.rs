use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SkyParams {
    pub inv_view_proj: [[f32; 4]; 4],
    pub sun_direction: [f32; 4],
    pub camera_pos: [f32; 4],
}

pub struct SkyboxPass {
    pub pipeline: wgpu::RenderPipeline,
    pub params_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    /// LUT-based pipeline (created after atmosphere is ready)
    pub lut_pipeline: Option<wgpu::RenderPipeline>,
    pub lut_bind_group: Option<wgpu::BindGroup>,
    pub use_lut: bool,
}

impl SkyboxPass {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let params = SkyParams {
            inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            sun_direction: [0.3, 0.8, 0.5, 0.0],
            camera_pos: [0.0; 4],
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sky_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky_bgl"),
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
            label: Some("sky_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("skybox_shader"),
            source: wgpu::ShaderSource::Wgsl(SKYBOX_SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skybox_pipeline"),
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview: None,
        });

        Self {
            pipeline,
            params_buffer,
            bind_group_layout,
            bind_group,
            lut_pipeline: None,
            lut_bind_group: None,
            use_lut: false,
        }
    }

    /// Set up the Bruneton LUT-based skybox pipeline.
    pub fn init_lut_pipeline(
        &mut self,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        atmo: &super::sky::atmosphere::BrunetonAtmosphere,
    ) {
        let lut_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky_lut_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let lut_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky_lut_bg"),
            layout: &lut_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&atmo.sky_view_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&atmo.transmittance_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&atmo.lut_sampler) },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("skybox_lut_shader"),
            source: wgpu::ShaderSource::Wgsl(SKYBOX_LUT_SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_lut_layout"),
            bind_group_layouts: &[&lut_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skybox_lut_pipeline"),
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview: None,
        });

        self.lut_pipeline = Some(pipeline);
        self.lut_bind_group = Some(lut_bg);
        self.use_lut = true;
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        inv_view_proj: glam::Mat4,
        sun_direction: [f32; 3],
        camera_pos: [f32; 3],
    ) {
        let params = SkyParams {
            inv_view_proj: inv_view_proj.to_cols_array_2d(),
            sun_direction: [sun_direction[0], sun_direction[1], sun_direction[2], 0.0],
            camera_pos: [camera_pos[0], camera_pos[1], camera_pos[2], 0.0],
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    pub fn active_pipeline(&self) -> &wgpu::RenderPipeline {
        if self.use_lut {
            self.lut_pipeline.as_ref().unwrap_or(&self.pipeline)
        } else {
            &self.pipeline
        }
    }

    pub fn active_bind_group(&self) -> &wgpu::BindGroup {
        if self.use_lut {
            self.lut_bind_group.as_ref().unwrap_or(&self.bind_group)
        } else {
            &self.bind_group
        }
    }
}

/// Fallback analytical skybox shader (no LUT dependency).
const SKYBOX_SHADER: &str = r#"
struct SkyParams {
    inv_view_proj: mat4x4<f32>,
    sun_direction: vec4<f32>,
    camera_pos: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: SkyParams;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) ray_dir: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let pos = positions[vertex_index];
    var out: VsOut;
    out.position = vec4<f32>(pos, 1.0, 1.0);

    let clip = vec4<f32>(pos, 1.0, 1.0);
    let world_pos = params.inv_view_proj * clip;
    out.ray_dir = normalize(world_pos.xyz / world_pos.w - params.camera_pos.xyz);
    return out;
}

const PI: f32 = 3.14159265359;

fn rayleigh_phase(cos_theta: f32) -> f32 {
    return 0.75 * (1.0 + cos_theta * cos_theta);
}

fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let num = 3.0 * (1.0 - g2) * (1.0 + cos_theta * cos_theta);
    let denom = 8.0 * PI * (2.0 + g2) * pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return num / (denom + 0.0001);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dir = normalize(in.ray_dir);
    let sun_dir = normalize(params.sun_direction.xyz);

    let cos_theta = dot(dir, sun_dir);
    let sun_height = max(sun_dir.y, 0.0);

    let rayleigh_coeff = vec3<f32>(5.8e-6, 13.5e-6, 33.1e-6);
    let safe_y = max(dir.y, 0.0) + 0.1;
    let optical_depth = 1.0 / safe_y;
    let rayleigh = rayleigh_coeff * rayleigh_phase(cos_theta) * optical_depth;

    let mie_coeff = vec3<f32>(21e-6);
    let mie = mie_coeff * mie_phase(cos_theta, 0.76) * optical_depth * 0.01;

    let sun_intensity = 20.0 * max(sun_height, 0.05);
    var sky_color = (rayleigh + mie) * sun_intensity;

    let horizon_factor = 1.0 - abs(dir.y);
    let sunset_color = vec3<f32>(1.0, 0.4, 0.1) * horizon_factor * horizon_factor * (1.0 - sun_height) * 2.0;
    sky_color = sky_color + sunset_color * sun_intensity * 0.1;

    // Ground: warm lit surface below horizon
    if dir.y < 0.0 {
        let ground_color = vec3<f32>(0.15, 0.12, 0.10) * max(sun_height, 0.02) * 5.0;
        let blend = clamp(-dir.y * 8.0, 0.0, 1.0);
        sky_color = mix(sky_color, ground_color, blend);
    }

    let sun_size = 0.9995;
    if (cos_theta > sun_size) {
        let sun_edge = (cos_theta - sun_size) / (1.0 - sun_size);
        sky_color = sky_color + vec3<f32>(10.0) * sun_edge * sun_edge;
    }

    return vec4<f32>(sky_color, 1.0);
}
"#;

/// Bruneton LUT-based skybox shader: samples precomputed SkyView LUT.
const SKYBOX_LUT_SHADER: &str = r#"
struct SkyParams {
    inv_view_proj: mat4x4<f32>,
    sun_direction: vec4<f32>,
    camera_pos: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: SkyParams;
@group(0) @binding(1) var sky_view_lut: texture_2d<f32>;
@group(0) @binding(2) var transmittance_lut: texture_2d<f32>;
@group(0) @binding(3) var lut_sampler: sampler;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) ray_dir: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let pos = positions[vertex_index];
    var out: VsOut;
    out.position = vec4<f32>(pos, 1.0, 1.0);

    let clip = vec4<f32>(pos, 1.0, 1.0);
    let world_pos = params.inv_view_proj * clip;
    out.ray_dir = normalize(world_pos.xyz / world_pos.w - params.camera_pos.xyz);
    return out;
}

const PI: f32 = 3.14159265359;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dir = normalize(in.ray_dir);
    let sun_dir = normalize(params.sun_direction.xyz);

    let azimuth = atan2(dir.x, dir.z);
    let elevation = asin(clamp(dir.y, -1.0, 1.0));

    let u = azimuth / (2.0 * PI) + 0.5;
    let v_raw = elevation / PI + 0.5;
    let v = clamp(v_raw, 0.01, 0.99);

    var sky_color = textureSample(sky_view_lut, lut_sampler, vec2<f32>(u, v)).rgb;

    // Below horizon: blend toward warm ground color to avoid black band
    if dir.y < 0.0 {
        let horizon_transmittance = textureSample(transmittance_lut, lut_sampler, vec2<f32>(0.5, 0.5)).rgb;
        let ground_albedo = vec3<f32>(0.15, 0.12, 0.10);
        let ground_lit = ground_albedo * max(sun_dir.y, 0.0) * 5.0 * horizon_transmittance;
        let blend = clamp(-dir.y * 10.0, 0.0, 1.0);
        sky_color = mix(sky_color, ground_lit, blend);
    }

    let cos_theta = dot(dir, sun_dir);
    let sun_size = 0.9997;
    var sun_disk = vec3<f32>(0.0);
    if cos_theta > sun_size {
        let sun_edge = (cos_theta - sun_size) / (1.0 - sun_size);
        let transmittance = textureSample(transmittance_lut, lut_sampler,
            vec2<f32>(0.0, sun_dir.y * 0.5 + 0.5)).rgb;
        sun_disk = transmittance * 20.0 * sun_edge * sun_edge;
    }

    return vec4<f32>(sky_color + sun_disk, 1.0);
}
"#;
