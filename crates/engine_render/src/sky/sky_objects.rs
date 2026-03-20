//! Phase 4.2: Procedural star field (no HDRI/EXR dependency).
//! ~5000 stars on unit sphere, fade with sun elevation.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use wgpu::util::DeviceExt;

const STAR_COUNT: usize = 5000;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StarVertex {
    pub position: [f32; 3],
    pub brightness: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StarUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub sun_height: f32,
    pub sky_radius: f32,
    pub _pad: [f32; 2],
}

/// Star field: procedural stars on unit sphere, rendered as points.
pub struct StarField {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl StarField {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let stars = Self::generate_stars(12345);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("star_vb"),
            contents: bytemuck::cast_slice(&stars),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniforms = StarUniforms {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            sun_height: -0.5,
            sky_radius: 10000.0,
            _pad: [0.0; 2],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("star_uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("star_bgl"),
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
            label: Some("star_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("star_shader"),
            source: wgpu::ShaderSource::Wgsl(STAR_SHADER_WGSL.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("star_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("star_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<StarVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                }],
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
                topology: wgpu::PrimitiveTopology::PointList,
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
            vertex_buffer,
            uniform_buffer,
            bind_group,
        }
    }

    fn generate_stars(seed: u64) -> Vec<StarVertex> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut stars = Vec::with_capacity(STAR_COUNT);
        for _ in 0..STAR_COUNT {
            let theta = rng.gen::<f32>() * std::f32::consts::TAU;
            let phi = (rng.gen::<f32>() * 2.0 - 1.0).acos();
            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();
            let brightness = 0.3 + rng.gen::<f32>() * 0.7;
            stars.push(StarVertex {
                position: [x, y, z],
                brightness,
            });
        }
        stars
    }

    /// Update uniforms. Call before render each frame.
    pub fn update(&self, queue: &wgpu::Queue, view_proj: Mat4, sky_radius: f32, sun_height: f32) {
        let uniforms = StarUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            sun_height,
            sky_radius,
            _pad: [0.0; 2],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Render the star field. Ensure update() was called this frame with current view_proj, sky_radius, sun_height.
    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, _sun_height: f32) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..STAR_COUNT as u32, 0..1);
    }
}

const STAR_SHADER_WGSL: &str = r#"
struct StarUniforms {
    view_proj: mat4x4<f32>,
    sun_height: f32,
    sky_radius: f32,
}

@group(0) @binding(0) var<uniform> uniforms: StarUniforms;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) brightness: f32,
}

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) brightness: f32,
}

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let world_pos = uniforms.sky_radius * in.position;
    out.clip_pos = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    out.brightness = in.brightness;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let sun_h = uniforms.sun_height;
    let night = 1.0 - max(sun_h, 0.0);
    let fade = smoothstep(0.0, 0.15, night);
    let alpha = in.brightness * fade;
    return vec4<f32>(1.0, 1.0, 1.05, alpha);
}
"#;
