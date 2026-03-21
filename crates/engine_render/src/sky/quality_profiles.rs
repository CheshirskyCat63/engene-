//! Phase 13.5: Per-subsystem quality fallback profiles.

#[derive(Clone, Debug)]
pub struct SkyQualityProfile {
    pub transmittance_lut_size: (u32, u32),
    pub sky_view_lut_size: (u32, u32),
    pub aerial_perspective_enabled: bool,
    pub cloud_ray_steps: u32,
    pub cloud_shadow_ray_steps: u32,
    pub cloud_resolution_scale: f32,
    pub cloud_temporal_frames: u32,
    pub cloud_shadow_enabled: bool,
    pub cloud_shadow_resolution: u32,
    pub fog_froxel_resolution: (u32, u32, u32),
    pub fog_terrain_aware: bool,
    pub precipitation_particle_density: f32,
    pub precipitation_occlusion_enabled: bool,
    pub precipitation_oblique_ingress: bool,
    pub hydrology_enabled: bool,
    pub hydrology_grid_resolution: f32,
    pub hydrology_update_hz: f32,
    pub wetness_update_frequency: u32,
    pub material_weather_enabled: bool,
    pub puddle_reflections: bool,
    pub sky_irradiance_enabled: bool,
    pub irradiance_cubemap_size: u32,
    pub interior_volumes_enabled: bool,
    pub interior_fog_propagation: bool,
}

impl SkyQualityProfile {
    pub fn ultra() -> Self {
        Self {
            transmittance_lut_size: (256, 64),
            sky_view_lut_size: (192, 108),
            aerial_perspective_enabled: true,
            cloud_ray_steps: 128,
            cloud_shadow_ray_steps: 32,
            cloud_resolution_scale: 0.5,
            cloud_temporal_frames: 16,
            cloud_shadow_enabled: true,
            cloud_shadow_resolution: 2048,
            fog_froxel_resolution: (64, 64, 32),
            fog_terrain_aware: true,
            precipitation_particle_density: 1.0,
            precipitation_occlusion_enabled: true,
            precipitation_oblique_ingress: true,
            hydrology_enabled: true,
            hydrology_grid_resolution: 2.0,
            hydrology_update_hz: 10.0,
            wetness_update_frequency: 1,
            material_weather_enabled: true,
            puddle_reflections: true,
            sky_irradiance_enabled: true,
            irradiance_cubemap_size: 32,
            interior_volumes_enabled: true,
            interior_fog_propagation: true,
        }
    }

    pub fn high() -> Self {
        Self {
            cloud_ray_steps: 64,
            cloud_resolution_scale: 0.5,
            cloud_shadow_resolution: 1024,
            fog_froxel_resolution: (32, 32, 16),
            precipitation_particle_density: 0.75,
            irradiance_cubemap_size: 16,
            ..Self::ultra()
        }
    }

    pub fn medium() -> Self {
        Self {
            cloud_ray_steps: 32,
            cloud_resolution_scale: 0.25,
            cloud_shadow_resolution: 512,
            fog_froxel_resolution: (16, 16, 8),
            fog_terrain_aware: true,
            precipitation_particle_density: 0.5,
            precipitation_oblique_ingress: false,
            hydrology_update_hz: 4.0,
            wetness_update_frequency: 4,
            interior_fog_propagation: false,
            irradiance_cubemap_size: 8,
            ..Self::ultra()
        }
    }

    pub fn low() -> Self {
        Self {
            transmittance_lut_size: (128, 32),
            sky_view_lut_size: (64, 36),
            aerial_perspective_enabled: false,
            cloud_ray_steps: 16,
            cloud_shadow_ray_steps: 8,
            cloud_resolution_scale: 0.25,
            cloud_temporal_frames: 8,
            cloud_shadow_enabled: false,
            cloud_shadow_resolution: 512,
            fog_froxel_resolution: (16, 16, 8),
            fog_terrain_aware: false,
            precipitation_particle_density: 0.25,
            precipitation_occlusion_enabled: false,
            precipitation_oblique_ingress: false,
            hydrology_enabled: false,
            hydrology_grid_resolution: 0.5,
            hydrology_update_hz: 2.0,
            wetness_update_frequency: 8,
            material_weather_enabled: false,
            puddle_reflections: false,
            sky_irradiance_enabled: false,
            irradiance_cubemap_size: 8,
            interior_volumes_enabled: false,
            interior_fog_propagation: false,
            ..Self::ultra()
        }
    }

    pub fn potato() -> Self {
        Self {
            transmittance_lut_size: (64, 16),
            sky_view_lut_size: (32, 18),
            aerial_perspective_enabled: false,
            cloud_ray_steps: 0,
            cloud_shadow_ray_steps: 0,
            cloud_resolution_scale: 0.0,
            cloud_temporal_frames: 0,
            cloud_shadow_enabled: false,
            cloud_shadow_resolution: 0,
            fog_froxel_resolution: (8, 8, 4),
            fog_terrain_aware: false,
            precipitation_particle_density: 0.1,
            precipitation_occlusion_enabled: false,
            precipitation_oblique_ingress: false,
            hydrology_enabled: false,
            hydrology_grid_resolution: 0.0,
            hydrology_update_hz: 0.0,
            wetness_update_frequency: 0,
            material_weather_enabled: false,
            puddle_reflections: false,
            sky_irradiance_enabled: false,
            irradiance_cubemap_size: 0,
            interior_volumes_enabled: false,
            interior_fog_propagation: false,
            ..Self::ultra()
        }
    }
}
