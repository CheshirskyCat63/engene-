use bytemuck::{Pod, Zeroable};

const VOLUMETRIC_SHADER: &str = r#"
struct VolParams {
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    num_steps: u32,
    light_dir: vec3<f32>,
    density: f32,
    light_color: vec3<f32>,
    scattering: f32,
};

@group(0) @binding(0) var depth_tex: texture_2d<f32>;
@group(0) @binding(1) var shadow_tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> params: VolParams;

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

@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let fog = params.density * 0.01;
    return vec4<f32>(params.light_color * fog, fog);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct VolParams {
    pub inv_view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub num_steps: u32,
    pub light_dir: [f32; 3],
    pub density: f32,
    pub light_color: [f32; 3],
    pub scattering: f32,
}

pub struct VolumetricPass {
    pub pipeline: wgpu::RenderPipeline,
    pub params_buf: wgpu::Buffer,
    pub bgl: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl VolumetricPass {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("volumetric_shader"),
            source: wgpu::ShaderSource::Wgsl(VOLUMETRIC_SHADER.into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("vol_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vol_pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("volumetric_pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None,
            multisample: Default::default(),
            cache: None,
            multiview: None,
        });

        let sampler = device.create_sampler(&Default::default());
        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vol_params"),
            size: std::mem::size_of::<VolParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { pipeline, params_buf, bgl, sampler }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.bgl
    }

    pub fn params_buffer(&self) -> &wgpu::Buffer {
        &self.params_buf
    }
}
