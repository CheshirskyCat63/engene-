use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PostProcessParams {
    pub exposure: f32,
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
    pub ssao_radius: f32,
    pub ssao_bias: f32,
    pub ssao_intensity: f32,
    pub _pad: [f32; 2],
}

impl Default for PostProcessParams {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            bloom_threshold: 1.0,
            bloom_intensity: 0.3,
            ssao_radius: 0.5,
            ssao_bias: 0.025,
            ssao_intensity: 1.0,
            _pad: [0.0; 2],
        }
    }
}

pub struct HdrTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

impl HdrTarget {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let format = wgpu::TextureFormat::Rgba16Float;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr_target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            format,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        *self = Self::new(device, width, height);
    }
}

pub struct BloomPass {
    pub downsample_views: Vec<wgpu::TextureView>,
    pub downsample_texture: wgpu::Texture,
    pub mip_count: u32,
    pub pipeline: wgpu::RenderPipeline,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub params_buffer: wgpu::Buffer,
}

impl BloomPass {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        _surface_format: wgpu::TextureFormat,
    ) -> Self {
        let mip_count = ((width.min(height) as f32).log2() as u32).min(6).max(1);

        let downsample_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bloom_downsample"),
            size: wgpu::Extent3d {
                width: (width / 2).max(1),
                height: (height / 2).max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let downsample_views: Vec<wgpu::TextureView> = (0..mip_count)
            .map(|mip| {
                downsample_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("bloom_mip_{mip}")),
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let params = PostProcessParams::default();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bloom_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_shader"),
            source: wgpu::ShaderSource::Wgsl(BLOOM_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom_pipeline"),
            layout: Some(&pipeline_layout),
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
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            downsample_views,
            downsample_texture,
            mip_count,
            pipeline,
            sampler,
            bind_group_layout,
            params_buffer,
        }
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &PostProcessParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
}

pub struct ToneMapPass {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub params_buffer: wgpu::Buffer,
}

impl ToneMapPass {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("tonemap_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tonemap_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let params = PostProcessParams::default();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tonemap_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tonemap_shader"),
            source: wgpu::ShaderSource::Wgsl(TONEMAP_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tonemap_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tonemap_pipeline"),
            layout: Some(&pipeline_layout),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
        }
    }

    pub fn make_bind_group(
        &self,
        device: &wgpu::Device,
        hdr_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tonemap_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params_buffer.as_entire_binding(),
                },
            ],
        })
    }
}

pub struct SsaoPass {
    pub noise_texture: wgpu::Texture,
    pub noise_view: wgpu::TextureView,
    pub kernel_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl SsaoPass {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let kernel = generate_ssao_kernel(64);
        let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ssao_kernel"),
            contents: bytemuck::cast_slice(&kernel),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let noise_data = generate_ssao_noise(4);
        let noise_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ssao_noise"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &noise_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&noise_data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * 4),
                rows_per_image: Some(4),
            },
            wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
        );

        let noise_view = noise_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssao_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        Self {
            noise_texture,
            noise_view,
            kernel_buffer,
            bind_group_layout,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SsaoSample {
    position: [f32; 4],
}

fn generate_ssao_kernel(count: usize) -> Vec<SsaoSample> {
    let mut kernel = Vec::with_capacity(count);
    for i in 0..count {
        let scale = i as f32 / count as f32;
        let scale = 0.1 + 0.9 * scale * scale;
        let hash = ((i * 73856093) ^ (i * 19349663)) as f32 / u32::MAX as f32;
        let hash2 = ((i * 83492791) ^ (i * 45678543)) as f32 / u32::MAX as f32;
        let hash3 = ((i * 15731) ^ (i * 789221)) as f32 / u32::MAX as f32;

        let x = hash * 2.0 - 1.0;
        let y = hash2 * 2.0 - 1.0;
        let z = hash3;
        let len = (x * x + y * y + z * z).sqrt().max(0.001);
        kernel.push(SsaoSample {
            position: [x / len * scale, y / len * scale, z / len * scale, 0.0],
        });
    }
    kernel
}

fn generate_ssao_noise(size: usize) -> Vec<[f32; 4]> {
    let mut noise = Vec::with_capacity(size * size);
    for i in 0..(size * size) {
        let hash = ((i * 73856093) ^ (i * 19349663)) as f32 / u32::MAX as f32;
        let hash2 = ((i * 83492791) ^ (i * 45678543)) as f32 / u32::MAX as f32;
        noise.push([hash * 2.0 - 1.0, hash2 * 2.0 - 1.0, 0.0, 0.0]);
    }
    noise
}

// ---------------------------------------------------------------------------
// Camera Intelligence (Phase 18)
// ---------------------------------------------------------------------------

pub struct CameraSimulation {
    pub auto_exposure: bool,
    pub adaptation_speed_up: f32,
    pub adaptation_speed_down: f32,
    pub min_exposure: f32,
    pub max_exposure: f32,
    pub target_luminance: f32,
    pub current_luminance: f32,
    pub local_exposure_blend: f32,
    pub bloom_threshold: f32,
}

impl Default for CameraSimulation {
    fn default() -> Self {
        Self {
            auto_exposure: true,
            adaptation_speed_up: 1.2,
            adaptation_speed_down: 0.6,
            min_exposure: 0.1,
            max_exposure: 8.0,
            target_luminance: 0.18,
            current_luminance: 0.18,
            local_exposure_blend: 0.3,
            bloom_threshold: 1.5,
        }
    }
}

impl CameraSimulation {
    pub fn update(&mut self, measured_luminance: f32, dt: f32, limits: &ExposureLimits) {
        if !self.auto_exposure {
            return;
        }

        let speed = if measured_luminance > self.current_luminance {
            self.adaptation_speed_up.min(limits.adaptation_speed_up_max)
        } else {
            self.adaptation_speed_down
                .min(limits.adaptation_speed_down_max)
        };

        let alpha = 1.0 - (-speed * dt).exp();
        self.current_luminance += (measured_luminance - self.current_luminance) * alpha;
        self.current_luminance = self.current_luminance.max(0.001);
    }

    pub fn exposure(&self) -> f32 {
        let e = self.target_luminance / self.current_luminance;
        e.clamp(self.min_exposure, self.max_exposure)
    }
}

pub struct WorldLightingResponse {
    pub fog_density: f32,
    pub fog_color: [f32; 3],
    pub color_temperature: f32,
    pub ambient_intensity: f32,
}

impl Default for WorldLightingResponse {
    fn default() -> Self {
        Self {
            fog_density: 0.003,
            fog_color: [0.7, 0.75, 0.85],
            color_temperature: 6500.0,
            ambient_intensity: 0.3,
        }
    }
}

impl WorldLightingResponse {
    pub fn update_from_world(
        &mut self,
        humidity: f32,
        rain: bool,
        sun_angle: f32,
        day_progress: f32,
    ) {
        self.fog_density = (humidity * 0.015 + if rain { 0.005 } else { 0.0 }).clamp(0.001, 0.02);

        let sunset_factor = (1.0 - (sun_angle / 90.0).abs()).max(0.0);
        self.fog_color = [
            0.7 + sunset_factor * 0.2,
            0.75 - sunset_factor * 0.1,
            0.85 - sunset_factor * 0.25,
        ];

        self.color_temperature = if day_progress < 0.25 {
            3500.0 + (day_progress / 0.25) * 3000.0
        } else if day_progress < 0.75 {
            6500.0
        } else if day_progress < 0.85 {
            6500.0 - ((day_progress - 0.75) / 0.10) * 3500.0
        } else {
            8000.0
        };
    }
}

pub struct ArtisticGrading {
    pub saturation: f32,
    pub contrast: f32,
    pub shadow_color: [f32; 3],
    pub highlight_color: [f32; 3],
    pub vignette_intensity: f32,
    pub tint: f32,
}

impl Default for ArtisticGrading {
    fn default() -> Self {
        Self {
            saturation: 1.0,
            contrast: 1.0,
            shadow_color: [0.0, 0.0, 0.05],
            highlight_color: [1.0, 0.98, 0.95],
            vignette_intensity: 0.15,
            tint: 0.0,
        }
    }
}

pub struct ExposureLimits {
    pub adaptation_speed_up_max: f32,
    pub adaptation_speed_down_max: f32,
    pub color_temp_range: (f32, f32),
    pub color_temp_shift_speed: f32,
    pub bloom_intensity_max: f32,
    pub fog_density_range: (f32, f32),
}

impl Default for ExposureLimits {
    fn default() -> Self {
        Self {
            adaptation_speed_up_max: 1.5,
            adaptation_speed_down_max: 0.8,
            color_temp_range: (3000.0, 8000.0),
            color_temp_shift_speed: 200.0,
            bloom_intensity_max: 0.4,
            fog_density_range: (0.001, 0.02),
        }
    }
}

const BLOOM_SHADER: &str = r#"
struct Params {
    exposure: f32,
    bloom_threshold: f32,
    bloom_intensity: f32,
    ssao_radius: f32,
    ssao_bias: f32,
    ssao_intensity: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let color = textureSample(src_texture, src_sampler, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if (brightness > params.bloom_threshold) {
        return vec4<f32>(color.rgb * params.bloom_intensity, 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
"#;

const TONEMAP_SHADER: &str = r#"
struct Params {
    exposure: f32,
    bloom_threshold: f32,
    bloom_intensity: f32,
    ssao_radius: f32,
    ssao_bias: f32,
    ssao_intensity: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return out;
}

fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var color = textureSample(hdr_texture, hdr_sampler, in.uv).rgb;

    // Exposure
    color = color * params.exposure;

    // ACES tone mapping
    color = aces_tonemap(color);

    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
"#;
