use serde::{Deserialize, Serialize};

use crate::world::biome::Biome;
use crate::world::streaming::ChunkCoord;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub coord: ChunkCoord,
    pub biome: Biome,
    pub danger_level: f32,
    pub has_camp: bool,
    pub has_trader: bool,
    pub resource_density: f32,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityPlacement {
    pub prefab_name: String,
    pub position: [f32; 2],
    pub rotation: f32,
    pub spawn_on_load: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnZone {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub entity_kind: String,
    pub max_entities: u32,
    pub respawn_time_hours: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatrolRoute {
    pub name: String,
    pub waypoints: Vec<[f32; 2]>,
    pub looping: bool,
    pub faction: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioZone {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub sound_event: String,
    pub volume: f32,
    pub is_ambient: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherOverride {
    pub center: [f32; 2],
    pub radius: f32,
    pub rain_intensity: f32,
    pub fog_density: f32,
    pub wind_strength: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthoredChunk {
    pub metadata: ChunkMetadata,
    pub entity_placements: Vec<EntityPlacement>,
    pub spawn_zones: Vec<SpawnZone>,
    pub patrol_routes: Vec<PatrolRoute>,
    pub audio_zones: Vec<AudioZone>,
    pub weather_overrides: Vec<WeatherOverride>,
}

impl AuthoredChunk {
    pub fn empty(coord: ChunkCoord, biome: Biome) -> Self {
        Self {
            metadata: ChunkMetadata {
                coord,
                biome,
                danger_level: 0.0,
                has_camp: false,
                has_trader: false,
                resource_density: 0.5,
                description: String::new(),
            },
            entity_placements: Vec::new(),
            spawn_zones: Vec::new(),
            patrol_routes: Vec::new(),
            audio_zones: Vec::new(),
            weather_overrides: Vec::new(),
        }
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let data = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, data)
    }

    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(path)?;
        ron::from_str(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
