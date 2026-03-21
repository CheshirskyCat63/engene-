use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

pub const SHADOW_MAP_SIZE: u32 = 2048;
pub const CASCADE_COUNT: usize = 4;

const CASCADE_SPLITS: [f32; CASCADE_COUNT] = [0.05, 0.15, 0.4, 1.0];

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowUniforms {
    pub light_view_proj: [[[f32; 4]; 4]; CASCADE_COUNT],
    pub cascade_splits: [f32; 4],
}

pub struct CascadedShadowMap {
    pub texture: wgpu::Texture,
    pub views: Vec<wgpu::TextureView>,
    pub sampler: wgpu::Sampler,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub shadow_pipeline: wgpu::RenderPipeline,
}

impl CascadedShadowMap {
    pub fn new(device: &wgpu::Device, camera_bgl: &wgpu::BindGroupLayout) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_map"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: CASCADE_COUNT as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let views: Vec<wgpu::TextureView> = (0..CASCADE_COUNT as u32)
            .map(|i| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("shadow_cascade_{i}")),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i,
                    array_layer_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let initial_uniforms = ShadowUniforms {
            light_view_proj: [Mat4::IDENTITY.to_cols_array_2d(); CASCADE_COUNT],
            cascade_splits: CASCADE_SPLITS,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow_uniform_buf"),
            contents: bytemuck::bytes_of(&initial_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let array_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow_array_view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&array_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADOW_DEPTH_SHADER.into()),
        });

        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shadow_pipeline_layout"),
                bind_group_layouts: &[camera_bgl],
                push_constant_ranges: &[],
            });

        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow_pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (3 * 4 + 3 * 4 + 3 * 4) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
                compilation_options: Default::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview: None,
        });

        Self {
            texture,
            views,
            sampler,
            uniform_buffer,
            bind_group_layout,
            bind_group,
            shadow_pipeline,
        }
    }

    pub fn compute_cascade_matrices(
        &self,
        queue: &wgpu::Queue,
        camera_pos: Vec3,
        camera_forward: Vec3,
        sun_direction: Vec3,
        near: f32,
        far: f32,
    ) {
        let mut matrices = [Mat4::IDENTITY.to_cols_array_2d(); CASCADE_COUNT];
        let mut prev_split = near;

        for (i, &split_pct) in CASCADE_SPLITS.iter().enumerate() {
            let split_far = near + (far - near) * split_pct;
            let center = camera_pos + camera_forward * ((prev_split + split_far) * 0.5);
            let radius = (split_far - prev_split) * 0.5 * 1.5;

            let light_dir = sun_direction.normalize();
            let light_pos = center - light_dir * radius * 2.0;
            let light_view = Mat4::look_at_rh(light_pos, center, Vec3::Y);
            let light_proj =
                Mat4::orthographic_rh(-radius, radius, -radius, radius, 0.1, radius * 4.0);

            matrices[i] = (light_proj * light_view).to_cols_array_2d();
            prev_split = split_far;
        }

        let uniforms = ShadowUniforms {
            light_view_proj: matrices,
            cascade_splits: CASCADE_SPLITS,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }
}

const SHADOW_DEPTH_SHADER: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct VsIn {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> @builtin(position) vec4<f32> {
    return camera.view_proj * vec4<f32>(in.position, 1.0);
}
"#;

pub const ENTITY_SHADOW_SHADER: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) instance_pos: vec3<f32>,
    @location(3) instance_color: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> @builtin(position) vec4<f32> {
    let world_pos = in.position + in.instance_pos;
    return camera.view_proj * vec4<f32>(world_pos, 1.0);
}
"#;

impl CascadedShadowMap {
    pub fn create_entity_shadow_pipeline(
        device: &wgpu::Device,
        camera_bgl: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("entity_shadow_shader"),
            source: wgpu::ShaderSource::Wgsl(ENTITY_SHADOW_SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("entity_shadow_pipeline_layout"),
            bind_group_layouts: &[camera_bgl],
            push_constant_ranges: &[],
        });

        let vertex_attrs = [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
        ];
        let instance_attrs = [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x3,
            },
        ];

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("entity_shadow_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attrs,
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &instance_attrs,
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview: None,
        })
    }
}
