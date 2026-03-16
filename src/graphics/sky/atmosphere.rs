//! Bruneton precomputed atmospheric scattering.
//! Computes 4 LUTs via GPU compute: Transmittance, MultiScattering, SkyView, AerialPerspective.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub const TRANSMITTANCE_W: u32 = 256;
pub const TRANSMITTANCE_H: u32 = 64;
pub const MULTI_SCATTERING_SIZE: u32 = 32;
pub const SKY_VIEW_W: u32 = 192;
pub const SKY_VIEW_H: u32 = 108;
pub const AERIAL_SIZE: u32 = 32;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct AtmosphereConfig {
    pub planet_radius: f32,
    pub atmosphere_height: f32,
    pub rayleigh_scale_height: f32,
    pub mie_scale_height: f32,
    pub rayleigh_coeff: [f32; 4],
    pub mie_coeff: [f32; 4],
    pub ozone_coeff: [f32; 4],
    pub sun_direction: [f32; 4],
    pub sun_intensity: f32,
    pub mie_g: f32,
    pub camera_height: f32,
    pub _pad: f32,
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self {
            planet_radius: 6371000.0,
            atmosphere_height: 100000.0,
            rayleigh_scale_height: 8000.0,
            mie_scale_height: 1200.0,
            rayleigh_coeff: [5.802e-6, 13.558e-6, 33.1e-6, 0.0],
            mie_coeff: [3.996e-6, 3.996e-6, 3.996e-6, 0.0],
            ozone_coeff: [0.650e-6, 1.881e-6, 0.085e-6, 0.0],
            sun_direction: [0.0, 0.7071, 0.7071, 0.0],
            sun_intensity: 20.0,
            mie_g: 0.8,
            camera_height: 1.8,
            _pad: 0.0,
        }
    }
}

pub struct BrunetonAtmosphere {
    pub config: AtmosphereConfig,
    config_buffer: wgpu::Buffer,

    pub transmittance_lut: wgpu::Texture,
    pub transmittance_view: wgpu::TextureView,
    pub multi_scattering_lut: wgpu::Texture,
    pub multi_scattering_view: wgpu::TextureView,
    pub sky_view_lut: wgpu::Texture,
    pub sky_view_view: wgpu::TextureView,
    pub aerial_lut: wgpu::Texture,
    pub aerial_view: wgpu::TextureView,

    pub lut_sampler: wgpu::Sampler,
    pub lut_bind_group_layout: wgpu::BindGroupLayout,
    pub lut_bind_group: wgpu::BindGroup,

    transmittance_pipeline: wgpu::ComputePipeline,
    transmittance_bg: wgpu::BindGroup,
    multi_scatter_pipeline: wgpu::ComputePipeline,
    multi_scatter_bg: wgpu::BindGroup,
    sky_view_pipeline: wgpu::ComputePipeline,
    sky_view_bg: wgpu::BindGroup,
    aerial_pipeline: wgpu::ComputePipeline,
    aerial_bg: wgpu::BindGroup,

    last_sun_y: f32,
}

impl BrunetonAtmosphere {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        let config = AtmosphereConfig::default();
        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("atmo_config"),
            contents: bytemuck::bytes_of(&config),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let transmittance_lut = Self::create_lut_2d(device, "transmittance", TRANSMITTANCE_W, TRANSMITTANCE_H);
        let transmittance_view = transmittance_lut.create_view(&Default::default());

        let multi_scattering_lut = Self::create_lut_2d(device, "multi_scatter", MULTI_SCATTERING_SIZE, MULTI_SCATTERING_SIZE);
        let multi_scattering_view = multi_scattering_lut.create_view(&Default::default());

        let sky_view_lut = Self::create_lut_2d(device, "sky_view", SKY_VIEW_W, SKY_VIEW_H);
        let sky_view_view = sky_view_lut.create_view(&Default::default());

        let aerial_lut = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("aerial_perspective"),
            size: wgpu::Extent3d { width: AERIAL_SIZE, height: AERIAL_SIZE, depth_or_array_layers: AERIAL_SIZE },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let aerial_view = aerial_lut.create_view(&Default::default());

        let lut_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lut_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let config_bgle = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        // --- Transmittance compute ---
        let trans_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("trans_bgl"),
            entries: &[
                config_bgle,
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let trans_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("transmittance_cs"),
            source: wgpu::ShaderSource::Wgsl(TRANSMITTANCE_COMPUTE_WGSL.into()),
        });
        let trans_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&trans_bgl],
            push_constant_ranges: &[],
        });
        let transmittance_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("transmittance_pipeline"),
            layout: Some(&trans_pl),
            module: &trans_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let transmittance_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("trans_bg"),
            layout: &trans_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: config_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&transmittance_view) },
            ],
        });

        // --- Multi-scattering compute ---
        let ms_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ms_bgl"),
            entries: &[
                config_bgle,
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let ms_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("multi_scatter_cs"),
            source: wgpu::ShaderSource::Wgsl(MULTI_SCATTER_COMPUTE_WGSL.into()),
        });
        let ms_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&ms_bgl],
            push_constant_ranges: &[],
        });
        let multi_scatter_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("multi_scatter_pipeline"),
            layout: Some(&ms_pl),
            module: &ms_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let multi_scatter_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ms_bg"),
            layout: &ms_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: config_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&transmittance_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&lut_sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&multi_scattering_view) },
            ],
        });

        // --- SkyView compute ---
        let sv_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sv_bgl"),
            entries: &[
                config_bgle,
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let sv_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky_view_cs"),
            source: wgpu::ShaderSource::Wgsl(SKY_VIEW_COMPUTE_WGSL.into()),
        });
        let sv_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&sv_bgl],
            push_constant_ranges: &[],
        });
        let sky_view_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("sky_view_pipeline"),
            layout: Some(&sv_pl),
            module: &sv_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let sky_view_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sv_bg"),
            layout: &sv_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: config_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&transmittance_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&multi_scattering_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&lut_sampler) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&sky_view_view) },
            ],
        });

        // --- Aerial Perspective compute ---
        let ap_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ap_bgl"),
            entries: &[
                config_bgle,
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
            ],
        });
        let ap_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("aerial_cs"),
            source: wgpu::ShaderSource::Wgsl(AERIAL_COMPUTE_WGSL.into()),
        });
        let ap_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&ap_bgl],
            push_constant_ranges: &[],
        });
        let aerial_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("aerial_pipeline"),
            layout: Some(&ap_pl),
            module: &ap_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let aerial_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ap_bg"),
            layout: &ap_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: config_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&transmittance_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&multi_scattering_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&lut_sampler) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&aerial_view) },
            ],
        });

        // --- Read-only bind group for sampling LUTs in fragment shaders ---
        let lut_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atmo_lut_read_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let lut_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atmo_lut_read_bg"),
            layout: &lut_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: config_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&transmittance_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&multi_scattering_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&sky_view_view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&aerial_view) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&lut_sampler) },
            ],
        });

        Self {
            config,
            config_buffer,
            transmittance_lut,
            transmittance_view,
            multi_scattering_lut,
            multi_scattering_view,
            sky_view_lut,
            sky_view_view,
            aerial_lut,
            aerial_view,
            lut_sampler,
            lut_bind_group_layout,
            lut_bind_group,
            transmittance_pipeline,
            transmittance_bg,
            multi_scatter_pipeline,
            multi_scatter_bg,
            sky_view_pipeline,
            sky_view_bg,
            aerial_pipeline,
            aerial_bg,
            last_sun_y: f32::NAN,
        }
    }

    pub fn update_config(&mut self, queue: &wgpu::Queue, sun_dir: [f32; 3], camera_height: f32) {
        self.config.sun_direction = [sun_dir[0], sun_dir[1], sun_dir[2], 0.0];
        self.config.camera_height = camera_height;
        queue.write_buffer(&self.config_buffer, 0, bytemuck::bytes_of(&self.config));
    }

    /// Returns true if LUTs need recomputation (sun moved enough).
    pub fn needs_recompute(&self, sun_dir_y: f32) -> bool {
        (self.last_sun_y - sun_dir_y).abs() > 0.005 || self.last_sun_y.is_nan()
    }

    /// Dispatch all compute passes: transmittance -> multi-scattering -> sky view -> aerial perspective.
    pub fn compute_luts(&mut self, encoder: &mut wgpu::CommandEncoder, sun_dir_y: f32) {
        self.last_sun_y = sun_dir_y;

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("transmittance_compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.transmittance_pipeline);
            pass.set_bind_group(0, &self.transmittance_bg, &[]);
            pass.dispatch_workgroups(
                (TRANSMITTANCE_W + 7) / 8,
                (TRANSMITTANCE_H + 7) / 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("multi_scatter_compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.multi_scatter_pipeline);
            pass.set_bind_group(0, &self.multi_scatter_bg, &[]);
            pass.dispatch_workgroups(
                (MULTI_SCATTERING_SIZE + 7) / 8,
                (MULTI_SCATTERING_SIZE + 7) / 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("sky_view_compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.sky_view_pipeline);
            pass.set_bind_group(0, &self.sky_view_bg, &[]);
            pass.dispatch_workgroups(
                (SKY_VIEW_W + 7) / 8,
                (SKY_VIEW_H + 7) / 8,
                1,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("aerial_compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.aerial_pipeline);
            pass.set_bind_group(0, &self.aerial_bg, &[]);
            pass.dispatch_workgroups(
                (AERIAL_SIZE + 3) / 4,
                (AERIAL_SIZE + 3) / 4,
                (AERIAL_SIZE + 3) / 4,
            );
        }
    }

    fn create_lut_2d(device: &wgpu::Device, label: &str, w: u32, h: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }
}

/// Shared WGSL utilities for atmosphere shaders (available for external consumers).
pub const ATMO_COMMON_WGSL: &str = r#"
struct AtmoConfig {
    planet_radius: f32,
    atmosphere_height: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,
    rayleigh_coeff: vec4<f32>,
    mie_coeff: vec4<f32>,
    ozone_coeff: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_intensity: f32,
    mie_g: f32,
    camera_height: f32,
    _pad: f32,
};

const PI: f32 = 3.14159265359;

fn safe_sqrt(x: f32) -> f32 { return sqrt(max(x, 0.0)); }

fn ray_sphere_intersect(ro: vec3<f32>, rd: vec3<f32>, radius: f32) -> vec2<f32> {
    let b = dot(ro, rd);
    let c = dot(ro, ro) - radius * radius;
    let d = b * b - c;
    if d < 0.0 { return vec2<f32>(-1.0); }
    let sd = sqrt(d);
    return vec2<f32>(-b - sd, -b + sd);
}

fn get_atmosphere_density(height: f32, cfg: AtmoConfig) -> vec3<f32> {
    let rayleigh = exp(-height / cfg.rayleigh_scale_height);
    let mie = exp(-height / cfg.mie_scale_height);
    let ozone_center = 25000.0;
    let ozone_width = 15000.0;
    let ozone = max(0.0, 1.0 - abs(height - ozone_center) / ozone_width);
    return vec3<f32>(rayleigh, mie, ozone);
}

fn get_extinction(density: vec3<f32>, cfg: AtmoConfig) -> vec3<f32> {
    return cfg.rayleigh_coeff.rgb * density.x +
           cfg.mie_coeff.rgb * density.y * 1.1 +
           cfg.ozone_coeff.rgb * density.z;
}

fn get_scattering(density: vec3<f32>, cfg: AtmoConfig) -> vec3<f32> {
    return cfg.rayleigh_coeff.rgb * density.x + cfg.mie_coeff.rgb * density.y;
}

fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta);
}

fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let num = 3.0 * (1.0 - g2) * (1.0 + cos_theta * cos_theta);
    let denom = (8.0 * PI) * (2.0 + g2) * pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return num / max(denom, 0.0001);
}

fn uv_to_transmittance_params(uv: vec2<f32>, cfg: AtmoConfig) -> vec2<f32> {
    let R = cfg.planet_radius;
    let H = cfg.atmosphere_height;
    let top = R + H;
    let h = mix(R, top, uv.x);
    let cos_angle = uv.y * 2.0 - 1.0;
    return vec2<f32>(h, cos_angle);
}

fn transmittance_params_to_uv(h: f32, cos_angle: f32, cfg: AtmoConfig) -> vec2<f32> {
    let R = cfg.planet_radius;
    let top = R + cfg.atmosphere_height;
    let x = (h - R) / cfg.atmosphere_height;
    let y = cos_angle * 0.5 + 0.5;
    return vec2<f32>(clamp(x, 0.0, 1.0), clamp(y, 0.0, 1.0));
}
"#;

const TRANSMITTANCE_COMPUTE_WGSL: &str = concat!(r#"
@group(0) @binding(0) var<uniform> cfg: AtmoConfig;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba16float, write>;
"#, r#"
const PI: f32 = 3.14159265359;

struct AtmoConfig {
    planet_radius: f32,
    atmosphere_height: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,
    rayleigh_coeff: vec4<f32>,
    mie_coeff: vec4<f32>,
    ozone_coeff: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_intensity: f32,
    mie_g: f32,
    camera_height: f32,
    _pad: f32,
};

fn get_atmosphere_density(height: f32) -> vec3<f32> {
    let rayleigh = exp(-height / cfg.rayleigh_scale_height);
    let mie = exp(-height / cfg.mie_scale_height);
    let ozone_center = 25000.0;
    let ozone_width = 15000.0;
    let ozone = max(0.0, 1.0 - abs(height - ozone_center) / ozone_width);
    return vec3<f32>(rayleigh, mie, ozone);
}

fn get_extinction(density: vec3<f32>) -> vec3<f32> {
    return cfg.rayleigh_coeff.rgb * density.x +
           cfg.mie_coeff.rgb * density.y * 1.1 +
           cfg.ozone_coeff.rgb * density.z;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if id.x >= dims.x || id.y >= dims.y { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims);
    let R = cfg.planet_radius;
    let top = R + cfg.atmosphere_height;
    let h = mix(R, top, uv.x);
    let cos_angle = uv.y * 2.0 - 1.0;

    let origin = vec3<f32>(0.0, h, 0.0);
    let dir = vec3<f32>(sqrt(max(1.0 - cos_angle * cos_angle, 0.0)), cos_angle, 0.0);

    let b = dot(origin, dir);
    let c_top = dot(origin, origin) - top * top;
    let d_top = b * b - c_top;
    var t_max = 0.0;
    if d_top >= 0.0 {
        t_max = -b + sqrt(d_top);
    }

    let c_ground = dot(origin, origin) - R * R;
    let d_ground = b * b - c_ground;
    if d_ground >= 0.0 {
        let t_ground = -b - sqrt(d_ground);
        if t_ground > 0.0 { t_max = min(t_max, t_ground); }
    }

    let STEPS = 40;
    let step_size = t_max / f32(STEPS);
    var optical_depth = vec3<f32>(0.0);

    for (var i = 0; i < STEPS; i++) {
        let t = (f32(i) + 0.5) * step_size;
        let pos = origin + dir * t;
        let altitude = length(pos) - R;
        let density = get_atmosphere_density(altitude);
        optical_depth += get_extinction(density) * step_size;
    }

    let transmittance = exp(-optical_depth);
    textureStore(output_tex, id.xy, vec4<f32>(transmittance, 1.0));
}
"#);

const MULTI_SCATTER_COMPUTE_WGSL: &str = r#"
struct AtmoConfig {
    planet_radius: f32,
    atmosphere_height: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,
    rayleigh_coeff: vec4<f32>,
    mie_coeff: vec4<f32>,
    ozone_coeff: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_intensity: f32,
    mie_g: f32,
    camera_height: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> cfg: AtmoConfig;
@group(0) @binding(1) var transmittance_lut: texture_2d<f32>;
@group(0) @binding(2) var lut_sampler: sampler;
@group(0) @binding(3) var output_tex: texture_storage_2d<rgba16float, write>;

fn sample_transmittance(h: f32, cos_angle: f32) -> vec3<f32> {
    let R = cfg.planet_radius;
    let top = R + cfg.atmosphere_height;
    let x = (h - R) / cfg.atmosphere_height;
    let y = cos_angle * 0.5 + 0.5;
    return textureSampleLevel(transmittance_lut, lut_sampler, vec2<f32>(clamp(x, 0.001, 0.999), clamp(y, 0.001, 0.999)), 0.0).rgb;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if id.x >= dims.x || id.y >= dims.y { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims);
    let R = cfg.planet_radius;
    let top = R + cfg.atmosphere_height;
    let h = mix(R + 1.0, top - 1.0, uv.x);
    let sun_cos = uv.y * 2.0 - 1.0;

    let DIR_SAMPLES = 8;
    let STEP_COUNT = 20;
    var lum_total = vec3<f32>(0.0);
    var fms_total = vec3<f32>(0.0);

    let isotropic_phase = 1.0 / (4.0 * 3.14159265359);

    for (var s = 0; s < DIR_SAMPLES; s++) {
        let theta = 3.14159265359 * (f32(s) + 0.5) / f32(DIR_SAMPLES);
        let cos_theta = cos(theta);
        let sin_theta = sin(theta);
        let dir = vec3<f32>(sin_theta, cos_theta, 0.0);

        let origin = vec3<f32>(0.0, h, 0.0);
        let b = dot(origin, dir);
        let c_top = dot(origin, origin) - top * top;
        var t_max = -b + sqrt(max(b * b - c_top, 0.0));

        let c_gnd = dot(origin, origin) - R * R;
        let d_gnd = b * b - c_gnd;
        var hit_ground = false;
        if d_gnd >= 0.0 {
            let t_gnd = -b - sqrt(d_gnd);
            if t_gnd > 0.0 { t_max = t_gnd; hit_ground = true; }
        }

        let step_size = t_max / f32(STEP_COUNT);
        var throughput = vec3<f32>(1.0);
        var lum = vec3<f32>(0.0);
        var fms = vec3<f32>(0.0);

        for (var i = 0; i < STEP_COUNT; i++) {
            let t = (f32(i) + 0.5) * step_size;
            let pos = origin + dir * t;
            let altitude = length(pos) - R;

            let rayleigh = exp(-altitude / cfg.rayleigh_scale_height);
            let mie = exp(-altitude / cfg.mie_scale_height);
            let scatter = cfg.rayleigh_coeff.rgb * rayleigh + cfg.mie_coeff.rgb * mie;
            let extinct = cfg.rayleigh_coeff.rgb * rayleigh + cfg.mie_coeff.rgb * mie * 1.1;

            let sun_trans = sample_transmittance(length(pos), sun_cos);
            let s_step = scatter * isotropic_phase;
            lum += throughput * s_step * sun_trans * cfg.sun_intensity * step_size;
            fms += throughput * s_step * step_size;
            throughput *= exp(-extinct * step_size);
        }

        lum_total += lum * sin_theta;
        fms_total += fms * sin_theta;
    }

    let weight = 3.14159265359 / f32(DIR_SAMPLES);
    lum_total *= weight;
    fms_total *= weight;

    let psi = lum_total / max(vec3<f32>(1.0) - fms_total, vec3<f32>(0.001));
    textureStore(output_tex, id.xy, vec4<f32>(psi, 1.0));
}
"#;

const SKY_VIEW_COMPUTE_WGSL: &str = r#"
struct AtmoConfig {
    planet_radius: f32,
    atmosphere_height: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,
    rayleigh_coeff: vec4<f32>,
    mie_coeff: vec4<f32>,
    ozone_coeff: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_intensity: f32,
    mie_g: f32,
    camera_height: f32,
    _pad: f32,
};

const PI: f32 = 3.14159265359;

@group(0) @binding(0) var<uniform> cfg: AtmoConfig;
@group(0) @binding(1) var transmittance_lut: texture_2d<f32>;
@group(0) @binding(2) var multi_scatter_lut: texture_2d<f32>;
@group(0) @binding(3) var lut_sampler: sampler;
@group(0) @binding(4) var output_tex: texture_storage_2d<rgba16float, write>;

fn sample_transmittance(h: f32, cos_angle: f32) -> vec3<f32> {
    let R = cfg.planet_radius;
    let x = (h - R) / cfg.atmosphere_height;
    let y = cos_angle * 0.5 + 0.5;
    return textureSampleLevel(transmittance_lut, lut_sampler, vec2<f32>(clamp(x, 0.001, 0.999), clamp(y, 0.001, 0.999)), 0.0).rgb;
}

fn sample_multi_scatter(h: f32, sun_cos: f32) -> vec3<f32> {
    let R = cfg.planet_radius;
    let x = (h - R) / cfg.atmosphere_height;
    let y = sun_cos * 0.5 + 0.5;
    return textureSampleLevel(multi_scatter_lut, lut_sampler, vec2<f32>(clamp(x, 0.001, 0.999), clamp(y, 0.001, 0.999)), 0.0).rgb;
}

fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let num = 3.0 * (1.0 - g2) * (1.0 + cos_theta * cos_theta);
    let denom = (8.0 * PI) * (2.0 + g2) * pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return num / max(denom, 0.0001);
}

fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if id.x >= dims.x || id.y >= dims.y { return; }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims);

    let azimuth = (uv.x - 0.5) * 2.0 * PI;
    let elevation = (uv.y - 0.5) * PI;

    let cos_e = cos(elevation);
    let dir = vec3<f32>(cos_e * sin(azimuth), sin(elevation), cos_e * cos(azimuth));

    let R = cfg.planet_radius;
    let top = R + cfg.atmosphere_height;
    let h = R + max(cfg.camera_height, 1.0);
    let origin = vec3<f32>(0.0, h, 0.0);
    let sun_dir = normalize(cfg.sun_direction.xyz);

    let b = dot(origin, dir);
    let c_top = dot(origin, origin) - top * top;
    var t_max = -b + sqrt(max(b * b - c_top, 0.0));

    let c_gnd = dot(origin, origin) - R * R;
    let d_gnd = b * b - c_gnd;
    if d_gnd >= 0.0 {
        let t_gnd = -b - sqrt(d_gnd);
        if t_gnd > 0.0 { t_max = min(t_max, t_gnd); }
    }

    let STEPS = 32;
    let step_size = t_max / f32(STEPS);
    var throughput = vec3<f32>(1.0);
    var lum = vec3<f32>(0.0);

    let cos_theta = dot(dir, sun_dir);
    let phase_r = rayleigh_phase(cos_theta);
    let phase_m = mie_phase(cos_theta, cfg.mie_g);

    for (var i = 0; i < STEPS; i++) {
        let t = (f32(i) + 0.5) * step_size;
        let pos = origin + dir * t;
        let altitude = length(pos) - R;

        let rayleigh_d = exp(-altitude / cfg.rayleigh_scale_height);
        let mie_d = exp(-altitude / cfg.mie_scale_height);

        let scatter_r = cfg.rayleigh_coeff.rgb * rayleigh_d;
        let scatter_m = cfg.mie_coeff.rgb * mie_d;
        let extinct = scatter_r + scatter_m * 1.1 + cfg.ozone_coeff.rgb *
            max(0.0, 1.0 - abs(altitude - 25000.0) / 15000.0);

        let pos_norm = normalize(pos);
        let sun_cos_at_pos = dot(pos_norm, sun_dir);
        let sun_trans = sample_transmittance(length(pos), sun_cos_at_pos);

        let inscatter = (scatter_r * phase_r + scatter_m * phase_m) * sun_trans * cfg.sun_intensity;
        let ms = sample_multi_scatter(length(pos), sun_cos_at_pos) *
                 (scatter_r + scatter_m);

        lum += throughput * (inscatter + ms) * step_size;
        throughput *= exp(-extinct * step_size);
    }

    textureStore(output_tex, id.xy, vec4<f32>(lum, 1.0));
}
"#;

const AERIAL_COMPUTE_WGSL: &str = r#"
struct AtmoConfig {
    planet_radius: f32,
    atmosphere_height: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,
    rayleigh_coeff: vec4<f32>,
    mie_coeff: vec4<f32>,
    ozone_coeff: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_intensity: f32,
    mie_g: f32,
    camera_height: f32,
    _pad: f32,
};

const PI: f32 = 3.14159265359;

@group(0) @binding(0) var<uniform> cfg: AtmoConfig;
@group(0) @binding(1) var transmittance_lut: texture_2d<f32>;
@group(0) @binding(2) var multi_scatter_lut: texture_2d<f32>;
@group(0) @binding(3) var lut_sampler: sampler;
@group(0) @binding(4) var output_tex: texture_storage_3d<rgba16float, write>;

fn sample_transmittance(h: f32, cos_angle: f32) -> vec3<f32> {
    let R = cfg.planet_radius;
    let x = (h - R) / cfg.atmosphere_height;
    let y = cos_angle * 0.5 + 0.5;
    return textureSampleLevel(transmittance_lut, lut_sampler, vec2<f32>(clamp(x, 0.001, 0.999), clamp(y, 0.001, 0.999)), 0.0).rgb;
}

fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let num = 3.0 * (1.0 - g2) * (1.0 + cos_theta * cos_theta);
    let denom = (8.0 * PI) * (2.0 + g2) * pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return num / max(denom, 0.0001);
}

fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta);
}

@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if id.x >= u32(dims.x) || id.y >= u32(dims.y) || id.z >= u32(dims.z) { return; }

    let uvw = (vec3<f32>(id) + 0.5) / vec3<f32>(dims);
    let max_dist = 32000.0;
    let dist = uvw.z * uvw.z * max_dist;

    let azimuth = (uvw.x - 0.5) * 2.0 * PI;
    let elevation = (uvw.y - 0.5) * PI * 0.5;
    let cos_e = cos(elevation);
    let dir = vec3<f32>(cos_e * sin(azimuth), sin(elevation), cos_e * cos(azimuth));

    let R = cfg.planet_radius;
    let h = R + max(cfg.camera_height, 1.0);
    let origin = vec3<f32>(0.0, h, 0.0);
    let sun_dir = normalize(cfg.sun_direction.xyz);
    let cos_theta = dot(dir, sun_dir);
    let phase_r = rayleigh_phase(cos_theta);
    let phase_m = mie_phase(cos_theta, cfg.mie_g);

    let STEPS = 16;
    let step_size = dist / f32(STEPS);
    var throughput = vec3<f32>(1.0);
    var lum = vec3<f32>(0.0);

    for (var i = 0; i < STEPS; i++) {
        let t = (f32(i) + 0.5) * step_size;
        let pos = origin + dir * t;
        let altitude = length(pos) - R;
        if altitude < 0.0 { break; }

        let rayleigh_d = exp(-altitude / cfg.rayleigh_scale_height);
        let mie_d = exp(-altitude / cfg.mie_scale_height);
        let scatter_r = cfg.rayleigh_coeff.rgb * rayleigh_d;
        let scatter_m = cfg.mie_coeff.rgb * mie_d;
        let extinct = scatter_r + scatter_m * 1.1;

        let sun_cos_at_pos = dot(normalize(pos), sun_dir);
        let sun_trans = sample_transmittance(length(pos), sun_cos_at_pos);
        let inscatter = (scatter_r * phase_r + scatter_m * phase_m) * sun_trans * cfg.sun_intensity;

        lum += throughput * inscatter * step_size;
        throughput *= exp(-extinct * step_size);
    }

    let transmittance_val = throughput.r * 0.33 + throughput.g * 0.34 + throughput.b * 0.33;
    textureStore(output_tex, id, vec4<f32>(lum, transmittance_val));
}
"#;
