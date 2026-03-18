use bytemuck::{Pod, Zeroable};

const CULL_SHADER: &str = r#"
struct CullParams {
    view_proj: mat4x4<f32>,
    frustum_planes: array<vec4<f32>, 6>,
    entity_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct InstanceIn {
    world_pos: vec3<f32>,
    _pad: f32,
    color: vec3<f32>,
    radius: f32,
};

struct InstanceOut {
    world_pos: vec3<f32>,
    _pad: f32,
    color: vec3<f32>,
    _pad2: f32,
};

struct DrawIndirect {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
};

@group(0) @binding(0) var<uniform> params: CullParams;
@group(0) @binding(1) var<storage, read> instances_in: array<InstanceIn>;
@group(0) @binding(2) var<storage, read_write> instances_out: array<InstanceOut>;
@group(0) @binding(3) var<storage, read_write> draw_args: DrawIndirect;

fn test_sphere_frustum(center: vec3<f32>, radius: f32) -> bool {
    for (var i = 0u; i < 6u; i = i + 1u) {
        let plane = params.frustum_planes[i];
        let dist = dot(plane.xyz, center) + plane.w;
        if dist < -radius {
            return false;
        }
    }
    return true;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= params.entity_count { return; }

    let inst = instances_in[idx];
    if !test_sphere_frustum(inst.world_pos, inst.radius) { return; }

    let out_idx = atomicAdd(&draw_args.instance_count, 1u);
    instances_out[out_idx].world_pos = inst.world_pos;
    instances_out[out_idx].color = inst.color;
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CullParams {
    pub view_proj: [[f32; 4]; 4],
    pub frustum_planes: [[f32; 4]; 6],
    pub entity_count: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CullInstanceIn {
    pub world_pos: [f32; 3],
    pub _pad: f32,
    pub color: [f32; 3],
    pub radius: f32,
}

pub struct GpuCullPass {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    params_buf: wgpu::Buffer,
    input_buf: wgpu::Buffer,
    output_buf: wgpu::Buffer,
    draw_indirect_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: u32,
}

impl GpuCullPass {
    pub fn new(device: &wgpu::Device, max_entities: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu_cull_shader"),
            source: wgpu::ShaderSource::Wgsl(CULL_SHADER.into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cull_bgl"),
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
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cull_pipeline_layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cull_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cull_params"),
            size: std::mem::size_of::<CullParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let input_size = (max_entities as usize * std::mem::size_of::<CullInstanceIn>()) as u64;
        let output_size = input_size;

        let input_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cull_input"),
            size: input_size.max(32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cull_output"),
            size: output_size.max(32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let draw_indirect_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cull_draw_indirect"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cull_bg"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: draw_indirect_buf.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group_layout: bgl,
            params_buf,
            input_buf,
            output_buf,
            draw_indirect_buf,
            bind_group,
            capacity: max_entities,
        }
    }

    pub fn output_buffer(&self) -> &wgpu::Buffer {
        &self.output_buf
    }

    pub fn draw_indirect_buffer(&self) -> &wgpu::Buffer {
        &self.draw_indirect_buf
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        params: &CullParams,
        instances: &[CullInstanceIn],
    ) {
        queue.write_buffer(&self.params_buf, 0, bytemuck::bytes_of(params));

        let count = instances.len().min(self.capacity as usize);
        if count > 0 {
            queue.write_buffer(
                &self.input_buf,
                0,
                bytemuck::cast_slice(&instances[..count]),
            );
        }

        queue.write_buffer(&self.draw_indirect_buf, 0, &[0u8; 16]);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("cull_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        let workgroups = (count as u32 + 63) / 64;
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
}
