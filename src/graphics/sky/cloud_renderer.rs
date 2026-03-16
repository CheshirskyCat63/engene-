//! Phase 3.4: Volumetric cloud raymarching renderer.
//! Fragment shader raymarches through cloud layer 1500m-12000m with Beer's law and Henyey-Greenstein.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;

const CLOUD_LAYER_BOTTOM: f32 = 1500.0;
const CLOUD_LAYER_TOP: f32 = 12000.0;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CloudRenderParams {
    pub inv_view_proj: [[f32; 4]; 4],
    pub sun_direction: [f32; 4],
    pub camera_pos: [f32; 4],
    pub time: f32,
    pub cloud_coverage: f32,
    pub cloud_layer_bottom: f32,
    pub cloud_layer_top: f32,
    pub density_multiplier: f32,
    pub light_absorption: f32,
    pub henyey_g: f32,
    pub ray_steps: u32,
    pub _pad: [u32; 4],
}

impl Default for CloudRenderParams {
    fn default() -> Self {
        Self {
            inv_view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            sun_direction: [0.3, 0.8, 0.5, 0.0],
            camera_pos: [0.0; 4],
            time: 0.0,
            cloud_coverage: 0.5,
            cloud_layer_bottom: CLOUD_LAYER_BOTTOM,
            cloud_layer_top: CLOUD_LAYER_TOP,
            density_multiplier: 0.5,
            light_absorption: 0.35,
            henyey_g: 0.76,
            ray_steps: 64,
            _pad: [0; 4],
        }
    }
}

/// Phase 3.4: Volumetric cloud raymarching renderer.
pub struct CloudRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub params_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

const CLOUD_RENDER_SHADER: &str = r#"
struct CloudRenderParams {
    inv_view_proj: mat4x4<f32>,
    sun_direction: vec4<f32>,
    camera_pos: vec4<f32>,
    time: f32,
    cloud_coverage: f32,
    cloud_layer_bottom: f32,
    cloud_layer_top: f32,
    density_multiplier: f32,
    light_absorption: f32,
    henyey_g: f32,
    ray_steps: u32,
    _pad: vec4<u32>,
}

@group(0) @binding(0) var<uniform> params: CloudRenderParams;
@group(0) @binding(1) var cloud_coverage_tex: texture_2d<f32>;
@group(0) @binding(2) var perlin_worley_3d: texture_3d<f32>;
@group(0) @binding(3) var worley_3d: texture_3d<f32>;
@group(0) @binding(4) var cloud_sampler: sampler;

const PLANET_RADIUS: f32 = 6371000.0;

fn ray_sphere_intersect(ro: vec3<f32>, rd: vec3<f32>, radius: f32) -> vec2<f32> {
    let oc = ro;
    let b = dot(oc, rd);
    let c = dot(oc, oc) - radius * radius;
    let d = b * b - c;
    if d < 0.0 { return vec2<f32>(-1.0, -1.0); }
    let sd = sqrt(d);
    return vec2<f32>(-b - sd, -b + sd);
}

fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g * cos_theta;
    return (1.0 - g2) / (4.0 * 3.14159265 * pow(max(denom, 0.0001), 1.5));
}

fn sample_cloud_density(pos: vec3<f32>) -> f32 {
    let planet_center = vec3<f32>(0.0, -PLANET_RADIUS, 0.0);
    let rp = pos - planet_center;
    let height = length(rp) - PLANET_RADIUS;
    if height < params.cloud_layer_bottom || height > params.cloud_layer_top {
        return 0.0;
    }
    let height_norm = (height - params.cloud_layer_bottom) / (params.cloud_layer_top - params.cloud_layer_bottom);
    let lf_raw = sin(height_norm * 3.14159265);
    let layer_factor = lf_raw * lf_raw;

    let scale = 0.00008;
    let wind_offset = params.time * 0.05;
    let sample_pos = pos * scale + vec3<f32>(wind_offset, 0.0, wind_offset * 0.5);

    let pw = textureSample(perlin_worley_3d, cloud_sampler, sample_pos);
    let worley = textureSample(worley_3d, cloud_sampler, sample_pos * 4.0).r;
    let base = (1.0 - worley) * (pw.r * 0.6 + pw.g * 0.3 + pw.b * 0.1);
    let coverage = params.cloud_coverage;
    let density = clamp((base - (1.0 - coverage)) / max(coverage, 0.001), 0.0, 1.0);
    return density * layer_factor * params.density_multiplier;
}

fn sample_coverage(uv: vec2<f32>) -> f32 {
    return textureSample(cloud_coverage_tex, cloud_sampler, uv).r;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 1.0, 1.0);
}

struct RayMarchOut {
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> RayMarchOut {
    let cam = params.camera_pos.xyz;
    let sun_dir = normalize(params.sun_direction.xyz);

    let ndc = vec2<f32>(frag_coord.xy) / vec2<f32>(frag_coord.w * 0.5, frag_coord.w * 0.5) - 1.0;
    let clip = vec4<f32>(ndc.x, ndc.y, 1.0, 1.0);
    let world_pos = params.inv_view_proj * clip;
    let rd = normalize((world_pos.xyz / world_pos.w) - cam);

    let bottom = PLANET_RADIUS + params.cloud_layer_bottom;
    let top = PLANET_RADIUS + params.cloud_layer_top;
    let t_bottom = ray_sphere_intersect(cam, rd, bottom);
    let t_top = ray_sphere_intersect(cam, rd, top);

    var t_start = max(t_bottom.x, 0.0);
    var t_end = t_top.y;
    if t_bottom.x < 0.0 && t_bottom.y < 0.0 {
        t_start = t_top.x;
    }
    if t_end <= t_start || t_start < 0.0 {
        return RayMarchOut(vec4<f32>(0.0, 0.0, 0.0, 0.0));
    }

    let step_size = (t_end - t_start) / f32(params.ray_steps);
    var transmittance = 1.0;
    var luminance = vec3<f32>(0.0);
    let light_absorption_f = params.light_absorption;

    for (var i = 0u; i < params.ray_steps; i++) {
        let t = t_start + (f32(i) + 0.5) * step_size;
        let pos = cam + rd * t;
        let density = sample_cloud_density(pos) * step_size * 50.0;

        if density > 0.0001 {
            let sun_trans = exp(-density * light_absorption_f);
            let cos_theta = dot(rd, sun_dir);
            let phase = henyey_greenstein(cos_theta, params.henyey_g);
            let light_contrib = vec3<f32>(1.2, 1.15, 1.0) * phase * sun_trans * density;
            luminance += transmittance * light_contrib;
        }
        transmittance *= exp(-density);
        if transmittance < 0.01 { break; }
    }

    return RayMarchOut(vec4<f32>(luminance, 1.0 - transmittance));
}
"#;

impl CloudRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, target_format: wgpu::TextureFormat) -> Self {
        let params = CloudRenderParams::default();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cloud_render_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cloud_render_bgl"),
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
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cloud_render_shader"),
            source: wgpu::ShaderSource::Wgsl(CLOUD_RENDER_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cloud_render_pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cloud_render_pipeline"),
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
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
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

        let placeholder_2d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_placeholder_2d"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &placeholder_2d, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &[128u8],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(1), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let placeholder_2d_view = placeholder_2d.create_view(&Default::default());

        let placeholder_3d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_placeholder_3d"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &placeholder_3d, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &[128u8, 128, 128, 255],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let placeholder_3d_view = placeholder_3d.create_view(&Default::default());

        let placeholder_r8_3d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_placeholder_r8_3d"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &placeholder_r8_3d, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &[128u8],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(1), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let placeholder_r8_3d_view = placeholder_r8_3d.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cloud_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud_render_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&placeholder_2d_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&placeholder_3d_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&placeholder_r8_3d_view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        Self {
            pipeline,
            params_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Create bind group with textures; call after cloud coverage and noise textures exist.
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        cloud_coverage_view: &wgpu::TextureView,
        perlin_worley_view: &wgpu::TextureView,
        worley_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud_render_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(cloud_coverage_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(perlin_worley_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(worley_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    pub fn update(&self, queue: &wgpu::Queue, params: &CloudRenderParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
