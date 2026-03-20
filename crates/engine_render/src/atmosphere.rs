use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct AtmosphereParams {
    pub camera_pos: [f32; 4],
    pub sun_direction: [f32; 4],
    pub sun_color: [f32; 4],
    pub fog_color: [f32; 4],
    pub fog_density: f32,
    pub fog_height_falloff: f32,
    pub fog_max_opacity: f32,
    pub aerial_perspective_density: f32,
    pub god_ray_intensity: f32,
    pub god_ray_decay: f32,
    pub _pad: [f32; 2],
}

impl Default for AtmosphereParams {
    fn default() -> Self {
        Self {
            camera_pos: [0.0; 4],
            sun_direction: [0.3, 0.8, 0.5, 0.0],
            sun_color: [1.0, 0.95, 0.85, 1.0],
            fog_color: [0.6, 0.65, 0.75, 1.0],
            fog_density: 0.003,
            fog_height_falloff: 0.02,
            fog_max_opacity: 0.85,
            aerial_perspective_density: 0.001,
            god_ray_intensity: 0.4,
            god_ray_decay: 0.96,
            _pad: [0.0; 2],
        }
    }
}

pub struct AtmospherePass {
    pub pipeline: wgpu::RenderPipeline,
    pub params_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl AtmospherePass {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let params = AtmosphereParams::default();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("atmosphere_params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atmosphere_bgl"),
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
            label: Some("atmosphere_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("atmosphere_shader"),
            source: wgpu::ShaderSource::Wgsl(ATMOSPHERE_SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("atmosphere_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("atmosphere_pipeline"),
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
            params_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, params: &AtmosphereParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }
}

const ATMOSPHERE_SHADER: &str = r#"
struct Params {
    camera_pos: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    fog_color: vec4<f32>,
    fog_density: f32,
    fog_height_falloff: f32,
    fog_max_opacity: f32,
    aerial_density: f32,
    god_ray_intensity: f32,
    god_ray_decay: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> params: Params;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

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
    // Volumetric fog: distance-based with height falloff
    let uv_centered = in.uv * 2.0 - vec2<f32>(1.0);
    let fake_depth = length(uv_centered) * 2000.0;

    let fog_amount = 1.0 - exp(-fake_depth * params.fog_density);
    let fog_opacity = min(fog_amount, params.fog_max_opacity);

    // God rays: radial blur from sun position
    let sun_screen = params.sun_direction.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    let delta_to_sun = sun_screen - in.uv;
    let dist_to_sun = length(delta_to_sun);
    let ray = params.god_ray_intensity * max(0.0, 1.0 - dist_to_sun * 2.0);

    var fog_color = params.fog_color.rgb + params.sun_color.rgb * ray * 0.3;
    return vec4<f32>(fog_color, fog_opacity * 0.15);
}
"#;
