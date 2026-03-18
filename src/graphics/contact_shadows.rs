use bytemuck::{Pod, Zeroable};

const CONTACT_SHADOW_SHADER: &str = r#"
struct ContactShadowParams {
    light_dir: vec3<f32>,
    max_steps: u32,
    inv_view_proj: mat4x4<f32>,
    resolution: vec2<f32>,
    step_size: f32,
    max_distance: f32,
};

@group(0) @binding(0) var depth_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> params: ContactShadowParams;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var o: VsOut;
    o.position = vec4<f32>(pos[vi], 0.0, 1.0);
    o.uv = pos[vi] * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return o;
}

fn reconstruct_world_pos(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let world = params.inv_view_proj * ndc;
    return world.xyz / world.w;
}

@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let depth = textureSample(depth_tex, tex_sampler, in.uv).r;
    if depth >= 1.0 { return vec4<f32>(1.0); }

    let world_pos = reconstruct_world_pos(in.uv, depth);
    let ray_dir = normalize(params.light_dir);
    let step = ray_dir * params.step_size;

    var shadow = 1.0;
    var pos = world_pos + step;

    for (var i = 0u; i < params.max_steps; i = i + 1u) {
        let projected = params.inv_view_proj * vec4<f32>(pos, 1.0);
        let screen_uv = (projected.xy / projected.w) * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);

        if screen_uv.x < 0.0 || screen_uv.x > 1.0 || screen_uv.y < 0.0 || screen_uv.y > 1.0 {
            break;
        }

        let sample_depth = textureSample(depth_tex, tex_sampler, screen_uv).r;
        let sample_pos = reconstruct_world_pos(screen_uv, sample_depth);
        let dist = length(pos - world_pos);

        if dist > params.max_distance { break; }

        let delta = length(pos - sample_pos);
        if delta < params.step_size * 0.5 {
            shadow = 0.0;
            break;
        }

        pos += step;
    }

    return vec4<f32>(shadow);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ContactShadowParams {
    pub light_dir: [f32; 3],
    pub max_steps: u32,
    pub inv_view_proj: [[f32; 4]; 4],
    pub resolution: [f32; 2],
    pub step_size: f32,
    pub max_distance: f32,
}

pub struct ContactShadowPass {
    pub pipeline: wgpu::RenderPipeline,
    pub params_buf: wgpu::Buffer,
    pub bgl: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl ContactShadowPass {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("contact_shadow_shader"),
            source: wgpu::ShaderSource::Wgsl(CONTACT_SHADOW_SHADER.into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("contact_shadow_bgl"),
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

        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("contact_shadow_pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("contact_shadow_pipeline"),
            layout: Some(&pl),
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
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::Src,
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
            multisample: Default::default(),
            cache: None,
            multiview: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("contact_shadow_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("contact_shadow_params"),
            size: std::mem::size_of::<ContactShadowParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            params_buf,
            bgl,
            sampler,
        }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.bgl
    }

    pub fn params_buffer(&self) -> &wgpu::Buffer {
        &self.params_buf
    }
}
