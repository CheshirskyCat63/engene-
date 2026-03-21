//! Phase 14.5: Debug visualization toolkit for sky/weather systems.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DebugViewMode {
    Off,
    CloudCoverage,
    CloudDensitySlice,
    SkyIrradiance,
    CloudShadowMap,
    PrecipitationOcclusion,
    HydrologyHeatmap,
    HydrologyFlow,
    WetnessMask,
    IndoorOutdoor,
    ClimateZones,
    FogBasins,
    MaterialWeather,
    PerformanceTimings,
    StormCells,
    WindField,
}

pub struct WeatherDebugState {
    pub active_mode: DebugViewMode,
    pub subsystem_timings_ms: [f32; 12],
}

impl Default for WeatherDebugState {
    fn default() -> Self {
        Self {
            active_mode: DebugViewMode::Off,
            subsystem_timings_ms: [0.0; 12],
        }
    }
}

impl WeatherDebugState {
    pub fn cycle_mode(&mut self) {
        self.active_mode = match self.active_mode {
            DebugViewMode::Off => DebugViewMode::CloudCoverage,
            DebugViewMode::CloudCoverage => DebugViewMode::CloudDensitySlice,
            DebugViewMode::CloudDensitySlice => DebugViewMode::SkyIrradiance,
            DebugViewMode::SkyIrradiance => DebugViewMode::CloudShadowMap,
            DebugViewMode::CloudShadowMap => DebugViewMode::PrecipitationOcclusion,
            DebugViewMode::PrecipitationOcclusion => DebugViewMode::HydrologyHeatmap,
            DebugViewMode::HydrologyHeatmap => DebugViewMode::HydrologyFlow,
            DebugViewMode::HydrologyFlow => DebugViewMode::WetnessMask,
            DebugViewMode::WetnessMask => DebugViewMode::IndoorOutdoor,
            DebugViewMode::IndoorOutdoor => DebugViewMode::ClimateZones,
            DebugViewMode::ClimateZones => DebugViewMode::FogBasins,
            DebugViewMode::FogBasins => DebugViewMode::MaterialWeather,
            DebugViewMode::MaterialWeather => DebugViewMode::PerformanceTimings,
            DebugViewMode::PerformanceTimings => DebugViewMode::StormCells,
            DebugViewMode::StormCells => DebugViewMode::WindField,
            DebugViewMode::WindField => DebugViewMode::Off,
        };
    }
}
