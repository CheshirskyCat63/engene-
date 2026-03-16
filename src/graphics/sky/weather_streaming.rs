//! Phase 13.7: World streaming integration for weather data.

use std::collections::HashMap;

/// Per-chunk weather data for streaming.
#[derive(Clone, Debug)]
pub struct ChunkWeatherData {
    pub wetness_grid: Option<Vec<u8>>,
    pub hydrology_summary: f32,
    pub has_interior_volumes: bool,
}

impl Default for ChunkWeatherData {
    fn default() -> Self {
        Self {
            wetness_grid: None,
            hydrology_summary: 0.0,
            has_interior_volumes: false,
        }
    }
}

/// Manages weather data per loaded chunk for world streaming.
pub struct WeatherStreamingManager {
    pub chunks: HashMap<(i32, i32), ChunkWeatherData>,
}

impl Default for WeatherStreamingManager {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
}

impl WeatherStreamingManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_chunk_load(
        &mut self,
        coord: (i32, i32),
        _current_weather: (),
    ) {
        self.chunks.insert(
            coord,
            ChunkWeatherData {
                wetness_grid: None,
                hydrology_summary: 0.0,
                has_interior_volumes: false,
            },
        );
    }

    pub fn on_chunk_unload(&mut self, coord: (i32, i32)) -> ChunkWeatherData {
        self.chunks
            .remove(&coord)
            .unwrap_or_default()
    }

    pub fn get_chunk_data(&self, coord: (i32, i32)) -> Option<&ChunkWeatherData> {
        self.chunks.get(&coord)
    }
}
