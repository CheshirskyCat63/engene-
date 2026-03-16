//! Phase 3.6: Cloud shadow map projected onto world geometry.
//! Orthographic top-down projection of cloud density for shadowing.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;

pub const CLOUD_SHADOW_SIZE: u32 = 2048;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CloudShadowParams {
    pub light_view_proj: [[f32; 4]; 4],
    pub inv_light_view_proj: [[f32; 4]; 4],
    pub cloud_layer_bottom: f32,
    pub cloud_layer_top: f32,
    pub world_scale: f32,
    pub _pad: f32,
}

const CLOUD_SHADOW_SHADER: &str = r#"
struct CloudShadowParams {
    light_view_proj: mat4x4<f32>,
    inv_light_view_proj: mat4x4<f32>,
    cloud_layer_bottom: f32,
    cloud_layer_top: f32,
    world_scale: f32,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> params: CloudShadowParams;
@group(0) @binding(1) var perlin_worley_3d: texture_3d<f32>;
@group(0) @binding(2) var worley_3d: texture_3d<f32>;
@group(0) @binding(3) var cloud_sampler: sampler;
@group(0) @binding(4) var output_tex: texture_storage_2d<r32float, write>;

const PLANET_RADIUS: f32 = 6371000.0;
const SAMPLE_STEPS: u32 = 32;

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

    let scale = 0.00008 * params.world_scale;
    let sample_pos = pos * scale;
    let pw = textureSampleLevel(perlin_worley_3d, cloud_sampler, sample_pos, 0.0);
    let worley = textureSampleLevel(worley_3d, cloud_sampler, sample_pos * 4.0, 0.0).r;
    let base = (1.0 - worley) * (pw.r * 0.6 + pw.g * 0.3 + pw.b * 0.1);
    return base * layer_factor;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if id.x >= dims.x || id.y >= dims.y { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims);
    let ndc = uv * 2.0 - 1.0;
    let clip = vec4<f32>(ndc.x, ndc.y, 0.0, 1.0);
    let inv = params.inv_light_view_proj;
    let world_4 = inv * clip;
    let world_xy = world_4.xy / world_4.w;
    let layer_center = (params.cloud_layer_bottom + params.cloud_layer_top) * 0.5;
    let layer_height = params.cloud_layer_top - params.cloud_layer_bottom;

    var density_acc = 0.0;
    let step_size = layer_height / f32(SAMPLE_STEPS);
    for (var i = 0u; i < SAMPLE_STEPS; i++) {
        let h = params.cloud_layer_bottom + (f32(i) + 0.5) * step_size;
        let pos_world = vec3<f32>(world_xy.x * params.world_scale, h, world_xy.y * params.world_scale);
        density_acc += sample_cloud_density(pos_world) * step_size;
    }
    let shadow = 1.0 - min(density_acc * 0.001, 1.0);
    textureStore(output_tex, id.xy, vec4<f32>(shadow, 0.0, 0.0, 1.0));
}
"#;

/// Phase 3.6: Cloud shadow map (2048x2048) via orthographic top-down projection.
pub struct CloudShadowPass {
    pub pipeline: wgpu::ComputePipeline,
    pub output_texture: wgpu::Texture,
    pub output_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl CloudShadowPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let params = CloudShadowParams {
            light_view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_light_view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            cloud_layer_bottom: 1500.0,
            cloud_layer_top: 12000.0,
            world_scale: 100000.0,
            _pad: 0.0,
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cloud_shadow_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_shadow_map"),
            size: wgpu::Extent3d {
                width: CLOUD_SHADOW_SIZE,
                height: CLOUD_SHADOW_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let output_view = output_texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cloud_shadow_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cloud_shadow_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cloud_shadow_cs"),
            source: wgpu::ShaderSource::Wgsl(CLOUD_SHADOW_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cloud_shadow_pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cloud_shadow_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let placeholder_3d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_shadow_placeholder_3d"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let placeholder_3d_view = placeholder_3d.create_view(&Default::default());

        let placeholder_r8 = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_shadow_placeholder_r8"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let placeholder_r8_view = placeholder_r8.create_view(&Default::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud_shadow_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&placeholder_3d_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&placeholder_r8_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&output_view) },
            ],
        });

        Self {
            pipeline,
            output_texture,
            output_view,
            sampler,
            params_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Create bind group with noise textures; call after cloud noise textures exist.
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        perlin_worley_view: &wgpu::TextureView,
        worley_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud_shadow_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(perlin_worley_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(worley_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&self.output_view) },
            ],
        })
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &CloudShadowParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("cloud_shadow_compute"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(
            (CLOUD_SHADOW_SIZE + 7) / 8,
            (CLOUD_SHADOW_SIZE + 7) / 8,
            1,
        );
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.output_view
    }
}
