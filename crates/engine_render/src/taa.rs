use bytemuck::{Pod, Zeroable};

const TAA_RESOLVE_SHADER: &str = r#"
struct TaaParams {
    jitter_offset: vec2<f32>,
    feedback: f32,
    _pad: f32,
};

@group(0) @binding(0) var current_tex: texture_2d<f32>;
@group(0) @binding(1) var history_tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> params: TaaParams;

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
    let current = textureSample(current_tex, tex_sampler, in.uv);
    let history = textureSample(history_tex, tex_sampler, in.uv - params.jitter_offset);

    let min_c = current - vec4<f32>(0.1);
    let max_c = current + vec4<f32>(0.1);
    let clamped_history = clamp(history, min_c, max_c);

    return mix(current, clamped_history, params.feedback);
}
"#;

const HALTON_SEQUENCE: [[f32; 2]; 8] = [
    [0.5, 0.333333],
    [0.25, 0.666666],
    [0.75, 0.111111],
    [0.125, 0.444444],
    [0.625, 0.777777],
    [0.375, 0.222222],
    [0.875, 0.555555],
    [0.0625, 0.888888],
];

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TaaParams {
    jitter_offset: [f32; 2],
    feedback: f32,
    _pad: f32,
}

pub struct TaaPass {
    pub pipeline: wgpu::RenderPipeline,
    pub sampler: wgpu::Sampler,
    pub params_buf: wgpu::Buffer,
    pub bgl: wgpu::BindGroupLayout,
    history_tex: wgpu::Texture,
    history_view: wgpu::TextureView,
    bind_group: Option<wgpu::BindGroup>,
    frame_index: u32,
    width: u32,
    height: u32,
}

impl TaaPass {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("taa_shader"),
            source: wgpu::ShaderSource::Wgsl(TAA_RESOLVE_SHADER.into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("taa_bgl"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
            label: Some("taa_pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("taa_pipeline"),
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
                    blend: None,
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
            label: Some("taa_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("taa_params"),
            size: std::mem::size_of::<TaaParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (history_tex, history_view) =
            Self::create_history(device, width, height, target_format);

        Self {
            pipeline,
            sampler,
            params_buf,
            bgl,
            history_tex,
            history_view,
            bind_group: None,
            frame_index: 0,
            width,
            height,
        }
    }

    fn create_history(
        device: &wgpu::Device,
        w: u32,
        h: u32,
        format: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("taa_history"),
            size: wgpu::Extent3d {
                width: w.max(1),
                height: h.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&Default::default());
        (tex, view)
    }

    pub fn jitter(&self, width: u32, height: u32) -> [f32; 2] {
        let idx = (self.frame_index as usize) % HALTON_SEQUENCE.len();
        let h = HALTON_SEQUENCE[idx];
        [(h[0] - 0.5) / width as f32, (h[1] - 0.5) / height as f32]
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        self.width = width;
        self.height = height;
        let (tex, view) = Self::create_history(device, width, height, format);
        self.history_tex = tex;
        self.history_view = view;
        self.bind_group = None;
    }

    pub fn advance_frame(&mut self) {
        self.frame_index += 1;
    }
}
