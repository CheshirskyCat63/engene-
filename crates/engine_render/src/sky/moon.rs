//! Phase 4.1: Moon renderer with 8k texture and phase calculation.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::path::Path;
use wgpu::util::DeviceExt;

const MOON_QUAD_SIZE: f32 = 0.008;
const LUNAR_CYCLE_DAYS: f32 = 29.53;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MoonUniforms {
    pub model: [[f32; 4]; 4],
    pub view_proj: [[f32; 4]; 4],
    pub phase: f32,
    pub brightness: f32,
    pub _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MoonVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

/// Moon pass: billboard quad rendered in skybox pass with moon texture and phase.
pub struct MoonPass {
    pub pipeline: wgpu::RenderPipeline,
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub uniforms_buffer: wgpu::Buffer,
    pub current_direction: Vec3,
    pub phase: f32,
}

impl MoonPass {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let texture = Self::load_moon_texture(device, queue);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("moon_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("moon_bgl"),
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
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let uniforms = MoonUniforms {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            phase: 0.0,
            brightness: 0.3,
            _pad: [0.0; 2],
        };
        let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("moon_uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&Default::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("moon_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let vertices: [MoonVertex; 6] = [
            MoonVertex {
                position: [-1.0, -1.0, 0.0],
                uv: [0.0, 1.0],
            },
            MoonVertex {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 1.0],
            },
            MoonVertex {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
            },
            MoonVertex {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
            },
            MoonVertex {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 1.0],
            },
            MoonVertex {
                position: [1.0, 1.0, 0.0],
                uv: [1.0, 0.0],
            },
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("moon_vb"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("moon_shader"),
            source: wgpu::ShaderSource::Wgsl(MOON_SHADER_WGSL.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("moon_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("moon_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<MoonVertex>() as u64,
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
                            format: wgpu::VertexFormat::Float32x2,
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
            texture,
            texture_view,
            sampler,
            bind_group,
            vertex_buffer,
            uniforms_buffer,
            current_direction: Vec3::NEG_Y,
            phase: 0.0,
        }
    }

    fn load_moon_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let path = Path::new("game/assets/sky/moon/moon_albedo_8k.jpg");
        let (width, height, data) = match std::fs::read(path) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => {
                    let rgb = img.into_rgb8();
                    let (w, h) = (rgb.width(), rgb.height());
                    let raw_rgb = rgb.into_raw();
                    let mut raw = Vec::with_capacity(raw_rgb.len() / 3 * 4);
                    for chunk in raw_rgb.chunks(3) {
                        raw.extend_from_slice(chunk);
                        raw.push(255);
                    }
                    (w, h, raw)
                }
                Err(e) => {
                    tracing::warn!("Failed to decode moon texture: {}", e);
                    Self::fallback_texture_data()
                }
            },
            Err(e) => {
                tracing::warn!("Moon texture not found at {}: {}", path.display(), e);
                Self::fallback_texture_data()
            }
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("moon_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    fn fallback_texture_data() -> (u32, u32, Vec<u8>) {
        const SIZE: u32 = 256;
        let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let dx = (x as f32 / SIZE as f32 - 0.5) * 2.0;
                let dy = (y as f32 / SIZE as f32 - 0.5) * 2.0;
                let r = (dx * dx + dy * dy).sqrt();
                let gray = if r <= 1.0 {
                    (0.7 * (1.0 - r * 0.3)) as u8
                } else {
                    0
                };
                data.push(gray);
                data.push(gray);
                data.push(gray);
                data.push(255);
            }
        }
        (SIZE, SIZE, data)
    }

    /// Compute moon direction from sun (opposite + orbital offset).
    /// day_progress: 0-1 (fraction of day), day_of_year: 0-365 for lunar phase.
    fn moon_direction(sun_dir: Vec3, _day_progress: f32, day_of_year: f32) -> Vec3 {
        let sun = sun_dir.normalize();
        let opposite = -sun;
        let orbital_offset = (day_of_year / LUNAR_CYCLE_DAYS) * std::f32::consts::TAU;
        let axis = if sun.y.abs() > 0.99 { Vec3::X } else { Vec3::Y };
        let rot = Mat4::from_axis_angle(axis, orbital_offset * 0.1);
        let dir = (rot * opposite.extend(0.0)).truncate().normalize();
        dir
    }

    /// Moon phase 0=new, 0.5=full, 1=new. Based on day_of_year.
    fn moon_phase(day_of_year: f32) -> f32 {
        (day_of_year / LUNAR_CYCLE_DAYS) % 1.0
    }

    /// Brightness based on sun elevation (visible at night).
    fn moon_brightness(sun_height: f32) -> f32 {
        let night = (1.0 - sun_height).max(0.0);
        (night * 2.0).min(1.0) * 0.4
    }

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        sun_dir: Vec3,
        day_progress: f32,
        day_of_year: f32,
        view_proj: Mat4,
        camera_pos: Vec3,
    ) {
        let moon_dir = Self::moon_direction(sun_dir, day_progress, day_of_year);
        self.current_direction = moon_dir;
        self.phase = Self::moon_phase(day_of_year);
        let sun_height = sun_dir.normalize().y;
        let brightness = Self::moon_brightness(sun_height);

        let right = if moon_dir.y.abs() > 0.99 {
            Vec3::X
        } else {
            moon_dir.cross(Vec3::Y).normalize()
        };
        let up = right.cross(moon_dir).normalize();

        let sky_distance = 5000.0;
        let center = camera_pos + moon_dir * sky_distance;
        let half = MOON_QUAD_SIZE * sky_distance;
        let v0 = (center - right * half - up * half).to_array();
        let v1 = (center + right * half - up * half).to_array();
        let v2 = (center - right * half + up * half).to_array();
        let v3 = (center + right * half + up * half).to_array();

        let uniforms = MoonUniforms {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view_proj: view_proj.to_cols_array_2d(),
            phase: self.phase,
            brightness,
            _pad: [0.0; 2],
        };

        queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));

        let vertices: [MoonVertex; 6] = [
            MoonVertex {
                position: v0,
                uv: [0.0, 1.0],
            },
            MoonVertex {
                position: v1,
                uv: [1.0, 1.0],
            },
            MoonVertex {
                position: v2,
                uv: [0.0, 0.0],
            },
            MoonVertex {
                position: v2,
                uv: [0.0, 0.0],
            },
            MoonVertex {
                position: v1,
                uv: [1.0, 1.0],
            },
            MoonVertex {
                position: v3,
                uv: [1.0, 0.0],
            },
        ];

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..6, 0..1);
    }
}

const MOON_SHADER_WGSL: &str = r#"
struct MoonUniforms {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    phase: f32,
    brightness: f32,
}

@group(0) @binding(0) var<uniform> uniforms: MoonUniforms;
@group(0) @binding(1) var moon_tex: texture_2d<f32>;
@group(0) @binding(2) var moon_sampler: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let world_pos = vec4<f32>(in.position, 1.0);
    out.clip_pos = uniforms.view_proj * world_pos;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var color = textureSample(moon_tex, moon_sampler, in.uv).rgb;
    let phase = uniforms.phase;
    if phase > 0.01 && phase < 0.99 {
        let shadow = smoothstep(0.45, 0.55, phase);
        color = mix(color, vec3<f32>(0.02, 0.02, 0.03), shadow * 0.9);
    }
    return vec4<f32>(color * uniforms.brightness, 1.0);
}
"#;
