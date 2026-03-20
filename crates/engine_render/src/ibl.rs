use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct EnvLightUniforms {
    pub sky_color_top: [f32; 4],
    pub sky_color_horizon: [f32; 4],
    pub ground_color: [f32; 4],
    pub intensity: f32,
    pub _pad: [f32; 3],
}

impl EnvLightUniforms {
    pub fn from_day_progress(day_progress: f32) -> Self {
        let sun_angle = (day_progress - 0.25) * std::f32::consts::TAU;
        let sun_height = sun_angle.sin().max(0.0);

        let day_sky_top = [0.2, 0.4, 0.9, 1.0];
        let day_sky_horizon = [0.6, 0.75, 0.95, 1.0];
        let night_sky_top = [0.01, 0.01, 0.03, 1.0];
        let night_sky_horizon = [0.02, 0.03, 0.06, 1.0];

        let t = sun_height;
        let lerp = |a: [f32; 4], b: [f32; 4]| -> [f32; 4] {
            [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
                1.0,
            ]
        };

        Self {
            sky_color_top: lerp(night_sky_top, day_sky_top),
            sky_color_horizon: lerp(night_sky_horizon, day_sky_horizon),
            ground_color: [0.05 + 0.1 * t, 0.04 + 0.08 * t, 0.03 + 0.05 * t, 1.0],
            intensity: 0.05 + 0.35 * t,
            _pad: [0.0; 3],
        }
    }
}

pub struct IblBindings {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
}

impl IblBindings {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ibl_uniform"),
            size: std::mem::size_of::<EnvLightUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ibl_bgl"),
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
            label: Some("ibl_bg"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group,
            layout,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, day_progress: f32) {
        let uniforms = EnvLightUniforms::from_day_progress(day_progress);
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniforms));
    }
}
