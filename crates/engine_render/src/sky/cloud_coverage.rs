//! Phase 3.1: Procedural cloud coverage generator using FBM noise.
//! Compute shader generates 512x512 2D coverage map with inline procedural FBM (no noise.rs import).

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub const COVERAGE_SIZE: u32 = 512;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CloudCoverageParams {
    pub wind_offset: [f32; 2],
    pub coverage_factor: f32,
    pub time: f32,
}

/// Inline procedural FBM noise in WGSL (not imported from noise.rs).
const CLOUD_COVERAGE_SHADER: &str = r#"
fn mod289_f(x: f32) -> f32 { return x - floor(x / 289.0) * 289.0; }
fn mod289_v2(x: vec2<f32>) -> vec2<f32> { return x - floor(x / 289.0) * 289.0; }
fn mod289_v4(x: vec4<f32>) -> vec4<f32> { return x - floor(x / 289.0) * 289.0; }
fn permute_v4(x: vec4<f32>) -> vec4<f32> { return mod289_v4((x * 34.0 + 10.0) * x); }
fn fade_v2(t: vec2<f32>) -> vec2<f32> { return t * t * t * (t * (t * 6.0 - 15.0) + 10.0); }

fn classic_noise_2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let pi0 = mod289_v2(i);
    let pi1 = mod289_v2(i + 1.0);
    let ix = vec4<f32>(pi0.x, pi1.x, pi0.x, pi1.x);
    let iy = vec4<f32>(pi0.y, pi0.y, pi1.y, pi1.y);
    let fx = vec4<f32>(f.x, f.x - 1.0, f.x, f.x - 1.0);
    let fy = vec4<f32>(f.y, f.y, f.y - 1.0, f.y - 1.0);
    let ip = permute_v4(permute_v4(ix) + iy);
    let phi = ip / 41.0 * 6.28318530718;
    let g00 = vec2<f32>(cos(phi.x), sin(phi.x));
    let g10 = vec2<f32>(cos(phi.y), sin(phi.y));
    let g01 = vec2<f32>(cos(phi.z), sin(phi.z));
    let g11 = vec2<f32>(cos(phi.w), sin(phi.w));
    let n00 = dot(g00, vec2<f32>(fx.x, fy.x));
    let n10 = dot(g10, vec2<f32>(fx.y, fy.y));
    let n01 = dot(g01, vec2<f32>(fx.z, fy.z));
    let n11 = dot(g11, vec2<f32>(fx.w, fy.w));
    let fade_xy = fade_v2(f);
    let n_x = mix(vec2<f32>(n00, n01), vec2<f32>(n10, n11), fade_xy.x);
    return 1.41421356 * mix(n_x.x, n_x.y, fade_xy.y);
}

fn fbm_classic_2d(p: vec2<f32>, octaves: i32) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    for (var i = 0; i < octaves; i++) {
        val += amp * classic_noise_2d(p * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return val;
}

struct CloudCoverageParams {
    wind_offset: vec2<f32>,
    coverage_factor: f32,
    time: f32,
}

@group(0) @binding(0) var<uniform> params: CloudCoverageParams;
@group(0) @binding(1) var output_tex: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if id.x >= dims.x || id.y >= dims.y { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims);
    let scale = 4.0;
    let world_uv = uv * scale + params.wind_offset;
    let noise_val = fbm_classic_2d(world_uv, 5);
    let remapped = noise_val * 0.5 + 0.5;
    let coverage = clamp(remapped - (1.0 - params.coverage_factor), 0.0, 1.0);
    textureStore(output_tex, id.xy, vec4<f32>(coverage, 0.0, 0.0, 1.0));
}
"#;

/// Phase 3.1: Cloud coverage generator (512x512 2D coverage map).
pub struct CloudCoveragePass {
    pub pipeline: wgpu::ComputePipeline,
    pub output_texture: wgpu::Texture,
    pub output_view: wgpu::TextureView,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
}

impl CloudCoveragePass {
    pub fn new(device: &wgpu::Device) -> Self {
        let params = CloudCoverageParams {
            wind_offset: [0.0, 0.0],
            coverage_factor: 0.5,
            time: 0.0,
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cloud_coverage_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_coverage_map"),
            size: wgpu::Extent3d {
                width: COVERAGE_SIZE,
                height: COVERAGE_SIZE,
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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cloud_coverage_bgl"),
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
            label: Some("cloud_coverage_cs"),
            source: wgpu::ShaderSource::Wgsl(CLOUD_COVERAGE_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cloud_coverage_pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cloud_coverage_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud_coverage_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&output_view),
                },
            ],
        });

        Self {
            pipeline,
            output_texture,
            output_view,
            bind_group_layout,
            bind_group,
            params_buffer,
        }
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &CloudCoverageParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("cloud_coverage_compute"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups((COVERAGE_SIZE + 7) / 8, (COVERAGE_SIZE + 7) / 8, 1);
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.output_view
    }

    pub fn output_texture(&self) -> &wgpu::Texture {
        &self.output_texture
    }
}
