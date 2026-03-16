//! Phase 13.6: Weather state serialization and save/load.

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WeatherSaveState {
    pub weather_state_name: String,
    pub transition_progress: f32,
    pub cloud_coverage_seed: u64,
    pub wind_offset: [f32; 3],
    pub storm_cells: Vec<StormCellSave>,
    pub master_seed: u64,
    pub game_time_elapsed: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StormCellSave {
    pub position: [f32; 3],
    pub radius: f32,
    pub intensity: f32,
    pub humidity: f32,
    pub stage: u8,
    pub stage_timer: f32,
    pub seed: u64,
}

impl Default for WeatherSaveState {
    fn default() -> Self {
        Self {
            weather_state_name: "Clear".into(),
            transition_progress: 0.0,
            cloud_coverage_seed: 42,
            wind_offset: [0.0; 3],
            storm_cells: Vec::new(),
            master_seed: 42,
            game_time_elapsed: 0.0,
        }
    }
}
