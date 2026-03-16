use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PbrMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub _pad: f32,
}

impl PbrMaterial {
    pub fn default_terrain() -> Self {
        Self {
            base_color: [0.3, 0.5, 0.2, 1.0],
            metallic: 0.0,
            roughness: 0.85,
            ao: 1.0,
            _pad: 0.0,
        }
    }

    pub fn default_entity() -> Self {
        Self {
            base_color: [0.75, 0.6, 0.5, 1.0],
            metallic: 0.0,
            roughness: 0.6,
            ao: 1.0,
            _pad: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SunLight {
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub ambient: [f32; 4],
    pub camera_pos: [f32; 4],
}

impl SunLight {
    pub fn from_day_progress(day_progress: f32, camera_pos: [f32; 3]) -> Self {
        let angle = day_progress * std::f32::consts::PI;
        let sun_y = angle.sin().max(0.05);
        let sun_x = angle.cos() * 0.5;

        let sunset_factor = (1.0 - sun_y).clamp(0.0, 1.0);
        let r = 1.0;
        let g = 0.95 - sunset_factor * 0.3;
        let b = 0.85 - sunset_factor * 0.5;
        let intensity = sun_y.clamp(0.1, 1.0) * 3.0;

        let ambient_intensity = 0.05 + sun_y * 0.15;

        Self {
            direction: [sun_x, sun_y, 0.3, 0.0],
            color: [r * intensity, g * intensity, b * intensity, 1.0],
            ambient: [
                ambient_intensity * 0.6,
                ambient_intensity * 0.7,
                ambient_intensity * 1.0,
                1.0,
            ],
            camera_pos: [camera_pos[0], camera_pos[1], camera_pos[2], 0.0],
        }
    }
}

pub const PBR_TERRAIN_SHADER: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
};

struct Light {
    direction: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,
    camera_pos: vec4<f32>,
};

struct Material {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> light: Light;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.normal = in.normal;
    out.color = in.color;
    return out;
}

const PI: f32 = 3.14159265359;

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom + 0.0001);
}

fn geometry_schlick(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k + 0.0001);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    return geometry_schlick(n_dot_v, roughness) * geometry_schlick(n_dot_l, roughness);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let albedo = in.color;
    let metallic = 0.0;
    let roughness = 0.85;
    let ao = 1.0;

    let n = normalize(in.normal);
    let v = normalize(light.camera_pos.xyz - in.world_pos);
    let l = normalize(light.direction.xyz);
    let h = normalize(v + l);

    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_v = max(dot(n, v), 0.001);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);

    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    let ndf = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, roughness);
    let f = fresnel_schlick(h_dot_v, f0);

    let numerator = ndf * g * f;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular = numerator / denominator;

    let k_s = f;
    let k_d = (vec3<f32>(1.0) - k_s) * (1.0 - metallic);

    let lo = (k_d * albedo / PI + specular) * light.color.xyz * n_dot_l;
    let ambient_term = light.ambient.xyz * albedo * ao;
    var color = ambient_term + lo;

    // Reinhard tone mapping
    color = color / (color + vec3<f32>(1.0));
    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
"#;

pub struct PbrBindings {
    pub light_buffer: wgpu::Buffer,
    pub light_bgl: wgpu::BindGroupLayout,
    pub light_bg: wgpu::BindGroup,
}

impl PbrBindings {
    pub fn new(device: &wgpu::Device) -> Self {
        let initial = SunLight::from_day_progress(0.25, [0.0, 0.0, 0.0]);
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("pbr_light_buf"),
            contents: bytemuck::bytes_of(&initial),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pbr_light_bgl"),
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

        let light_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pbr_light_bg"),
            layout: &light_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        Self {
            light_buffer,
            light_bgl,
            light_bg,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, light: &SunLight) {
        queue.write_buffer(&self.light_buffer, 0, bytemuck::bytes_of(light));
    }
}
