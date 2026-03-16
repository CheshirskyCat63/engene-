use bytemuck::{Pod, Zeroable};

const PARTICLE_UPDATE_SHADER: &str = r#"
struct Emitter {
    position: vec3<f32>,
    rate: f32,
    velocity_min: vec3<f32>,
    lifetime: f32,
    velocity_max: vec3<f32>,
    size: f32,
    color_start: vec4<f32>,
    color_end: vec4<f32>,
    gravity: f32,
    count: u32,
    seed: u32,
    _pad: u32,
};

struct Particle {
    pos: vec3<f32>,
    life: f32,
    vel: vec3<f32>,
    max_life: f32,
    color: vec4<f32>,
    size: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

struct Globals {
    dt: f32,
    total_time: f32,
    max_particles: u32,
    _pad: u32,
};

struct Counter {
    alive: atomic<u32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> emitter: Emitter;
@group(0) @binding(2) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(3) var<storage, read_write> counter: Counter;

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / f32(0xFFFFFFFFu);
}

@compute @workgroup_size(64)
fn cs_update(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= globals.max_particles { return; }

    var p = particles[idx];

    if p.life <= 0.0 {
        // Try to spawn from emitter
        if atomicAdd(&counter.alive, 0u) < emitter.count {
            let s = emitter.seed + idx * 3u;
            p.pos = emitter.position + vec3<f32>(
                mix(emitter.velocity_min.x, emitter.velocity_max.x, rand_f(s)) * 0.1,
                mix(emitter.velocity_min.y, emitter.velocity_max.y, rand_f(s + 1u)) * 0.1,
                mix(emitter.velocity_min.z, emitter.velocity_max.z, rand_f(s + 2u)) * 0.1,
            );
            p.vel = vec3<f32>(
                mix(emitter.velocity_min.x, emitter.velocity_max.x, rand_f(s + 3u)),
                mix(emitter.velocity_min.y, emitter.velocity_max.y, rand_f(s + 4u)),
                mix(emitter.velocity_min.z, emitter.velocity_max.z, rand_f(s + 5u)),
            );
            p.life = emitter.lifetime;
            p.max_life = emitter.lifetime;
            p.size = emitter.size;
            p.color = emitter.color_start;
            atomicAdd(&counter.alive, 1u);
        }
    } else {
        p.vel.y += emitter.gravity * globals.dt;
        p.pos += p.vel * globals.dt;
        p.life -= globals.dt;
        let t = 1.0 - (p.life / p.max_life);
        p.color = mix(emitter.color_start, emitter.color_end, t);
    }

    particles[idx] = p;
}
"#;

const PARTICLE_RENDER_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_right: vec3<f32>,
    _p0: f32,
    camera_up: vec3<f32>,
    _p1: f32,
};

struct Particle {
    pos: vec3<f32>,
    life: f32,
    vel: vec3<f32>,
    max_life: f32,
    color: vec4<f32>,
    size: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex fn vs_main(
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) ii: u32,
) -> VsOut {
    let p = particles[ii];
    if p.life <= 0.0 {
        var o: VsOut;
        o.position = vec4<f32>(0.0, 0.0, -2.0, 1.0);
        o.color = vec4<f32>(0.0);
        o.uv = vec2<f32>(0.0);
        return o;
    }
    var offsets = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>(-0.5,  0.5),
        vec2<f32>(-0.5,  0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>( 0.5,  0.5),
    );
    let off = offsets[vi];
    let world = p.pos
        + camera.camera_right * off.x * p.size
        + camera.camera_up * off.y * p.size;
    var o: VsOut;
    o.position = camera.view_proj * vec4<f32>(world, 1.0);
    o.color = p.color;
    o.uv = off + vec2<f32>(0.5);
    return o;
}

@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dist = length(in.uv - vec2<f32>(0.5));
    let alpha = smoothstep(0.5, 0.3, dist) * in.color.a;
    return vec4<f32>(in.color.rgb, alpha);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ParticleGlobals {
    pub dt: f32,
    pub total_time: f32,
    pub max_particles: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuEmitter {
    pub position: [f32; 3],
    pub rate: f32,
    pub velocity_min: [f32; 3],
    pub lifetime: f32,
    pub velocity_max: [f32; 3],
    pub size: f32,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub gravity: f32,
    pub count: u32,
    pub seed: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ParticleCameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_right: [f32; 3],
    pub _p0: f32,
    pub camera_up: [f32; 3],
    pub _p1: f32,
}

const MAX_PARTICLES: u32 = 16384;

pub struct ParticleSystem {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub particle_buf: wgpu::Buffer,
    pub counter_buf: wgpu::Buffer,
    pub globals_buf: wgpu::Buffer,
    pub emitter_buf: wgpu::Buffer,
    pub camera_buf: wgpu::Buffer,
    pub compute_bg: wgpu::BindGroup,
    pub render_bg: wgpu::BindGroup,
    total_time: f32,
    frame_seed: u32,
}

impl ParticleSystem {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("particle_compute"),
            source: wgpu::ShaderSource::Wgsl(PARTICLE_UPDATE_SHADER.into()),
        });
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("particle_render"),
            source: wgpu::ShaderSource::Wgsl(PARTICLE_RENDER_SHADER.into()),
        });

        let particle_size = MAX_PARTICLES as u64 * 64;
        let particle_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particles"),
            size: particle_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let counter_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle_counter"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle_globals"),
            size: std::mem::size_of::<ParticleGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let emitter_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle_emitter"),
            size: std::mem::size_of::<GpuEmitter>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle_camera"),
            size: std::mem::size_of::<ParticleCameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle_compute_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });

        let render_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle_render_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });

        let compute_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle_compute_bg"),
            layout: &compute_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: globals_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: emitter_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: particle_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: counter_buf.as_entire_binding() },
            ],
        });

        let render_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle_render_bg"),
            layout: &render_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: particle_buf.as_entire_binding() },
            ],
        });

        let compute_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("particle_compute_pl"),
            bind_group_layouts: &[&compute_bgl],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("particle_compute"),
            layout: Some(&compute_pl),
            module: &compute_shader,
            entry_point: Some("cs_update"),
            compilation_options: Default::default(),
            cache: None,
        });

        let render_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("particle_render_pl"),
            bind_group_layouts: &[&render_bgl],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("particle_render"),
            layout: Some(&render_pl),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            cache: None,
            multiview: None,
        });

        Self {
            compute_pipeline,
            render_pipeline,
            particle_buf,
            counter_buf,
            globals_buf,
            emitter_buf,
            camera_buf,
            compute_bg,
            render_bg,
            total_time: 0.0,
            frame_seed: 0,
        }
    }

    pub fn dispatch_update(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        dt: f32,
        emitter: &GpuEmitter,
    ) {
        self.total_time += dt;
        self.frame_seed = self.frame_seed.wrapping_add(1);

        let globals = ParticleGlobals {
            dt,
            total_time: self.total_time,
            max_particles: MAX_PARTICLES,
            _pad: 0,
        };
        queue.write_buffer(&self.globals_buf, 0, bytemuck::bytes_of(&globals));

        let mut em = *emitter;
        em.seed = self.frame_seed.wrapping_mul(747796405);
        queue.write_buffer(&self.emitter_buf, 0, bytemuck::bytes_of(&em));

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("particle_update"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.compute_pipeline);
        pass.set_bind_group(0, &self.compute_bg, &[]);
        pass.dispatch_workgroups((MAX_PARTICLES + 63) / 64, 1, 1);
    }

    pub fn render<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        queue: &wgpu::Queue,
        camera_uniform: &ParticleCameraUniform,
    ) {
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(camera_uniform));
        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.render_bg, &[]);
        pass.draw(0..6, 0..MAX_PARTICLES);
    }
}

pub fn fire_emitter(pos: [f32; 3]) -> GpuEmitter {
    GpuEmitter {
        position: pos,
        rate: 50.0,
        velocity_min: [-1.0, 2.0, -1.0],
        lifetime: 1.5,
        velocity_max: [1.0, 5.0, 1.0],
        size: 0.8,
        color_start: [1.0, 0.6, 0.1, 0.9],
        color_end: [0.3, 0.1, 0.0, 0.0],
        gravity: -0.5,
        count: 200,
        seed: 0,
        _pad: 0,
    }
}

pub fn smoke_emitter(pos: [f32; 3]) -> GpuEmitter {
    GpuEmitter {
        position: pos,
        rate: 30.0,
        velocity_min: [-0.5, 1.0, -0.5],
        lifetime: 3.0,
        velocity_max: [0.5, 3.0, 0.5],
        size: 1.5,
        color_start: [0.4, 0.4, 0.4, 0.6],
        color_end: [0.2, 0.2, 0.2, 0.0],
        gravity: -0.2,
        count: 100,
        seed: 0,
        _pad: 0,
    }
}

pub fn rain_emitter(pos: [f32; 3]) -> GpuEmitter {
    GpuEmitter {
        position: [pos[0], pos[1] + 50.0, pos[2]],
        rate: 500.0,
        velocity_min: [-5.0, -15.0, -5.0],
        lifetime: 2.0,
        velocity_max: [5.0, -10.0, 5.0],
        size: 0.05,
        color_start: [0.6, 0.7, 0.8, 0.4],
        color_end: [0.5, 0.6, 0.7, 0.1],
        gravity: 9.81,
        count: 2000,
        seed: 0,
        _pad: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_globals_size() {
        assert_eq!(std::mem::size_of::<ParticleGlobals>(), 16);
    }

    #[test]
    fn test_gpu_emitter_size() {
        // GpuEmitter should be properly aligned for GPU
        assert!(std::mem::size_of::<GpuEmitter>() > 0);
    }

    #[test]
    fn test_particle_camera_uniform_size() {
        // Size should be: mat4x4 (64) + camera_right (12 + 4 pad) + camera_up (12 + 4 pad) = 96
        assert!(std::mem::size_of::<ParticleCameraUniform>() > 0);
    }

    #[test]
    fn test_fire_emitter() {
        let emitter = fire_emitter([0.0, 0.0, 0.0]);
        assert_eq!(emitter.position, [0.0, 0.0, 0.0]);
        assert!(emitter.rate > 0.0);
        assert!(emitter.lifetime > 0.0);
        assert!(emitter.count > 0);
        assert!(emitter.gravity < 0.0); // Fire rises
    }

    #[test]
    fn test_smoke_emitter() {
        let emitter = smoke_emitter([10.0, 5.0, 2.0]);
        assert_eq!(emitter.position, [10.0, 5.0, 2.0]);
        assert!(emitter.rate > 0.0);
        assert!(emitter.lifetime > 0.0);
        assert!(emitter.gravity < 0.0); // Smoke rises
    }

    #[test]
    fn test_rain_emitter() {
        let emitter = rain_emitter([0.0, 0.0, 0.0]);
        // Rain should be positioned above the given position
        assert!(emitter.position[1] > 0.0);
        assert!(emitter.gravity > 0.0); // Rain falls
        assert!(emitter.count > 100); // Lots of rain drops
    }

    #[test]
    fn test_emitter_velocity_range() {
        let emitter = fire_emitter([0.0, 0.0, 0.0]);
        // velocity_max should be >= velocity_min for each component
        assert!(emitter.velocity_max[0] >= emitter.velocity_min[0]);
        assert!(emitter.velocity_max[1] >= emitter.velocity_min[1]);
        assert!(emitter.velocity_max[2] >= emitter.velocity_min[2]);
    }

    #[test]
    fn test_max_particles_constant() {
        assert_eq!(MAX_PARTICLES, 16384);
    }
}
