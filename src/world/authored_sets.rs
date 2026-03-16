use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampDefinition {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub faction: String,
    pub initial_population: u32,
    pub has_trader: bool,
    pub has_mechanic: bool,
    pub danger_memory: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuinDefinition {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub loot_tier: u8,
    pub structural_integrity: f32,
    pub has_anomaly: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnomalyDefinition {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub anomaly_type: String,
    pub intensity: f32,
    pub artifact_chance: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraderPost {
    pub name: String,
    pub position: [f32; 2],
    pub faction: String,
    pub specialty: String,
    pub price_modifier: f32,
    pub restock_interval_hours: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterHabitat {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub species: String,
    pub pack_size: u32,
    pub aggression: f32,
    pub nocturnal: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoadSegment {
    pub name: String,
    pub points: Vec<[f32; 2]>,
    pub width: f32,
    pub safety_level: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioZoneDefinition {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub ambient_sound: String,
    pub day_volume: f32,
    pub night_volume: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherZoneDefinition {
    pub name: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub base_rain: f32,
    pub base_fog: f32,
    pub temperature_offset: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldLayout {
    pub camps: Vec<CampDefinition>,
    pub ruins: Vec<RuinDefinition>,
    pub anomalies: Vec<AnomalyDefinition>,
    pub traders: Vec<TraderPost>,
    pub habitats: Vec<MonsterHabitat>,
    pub roads: Vec<RoadSegment>,
    pub audio_zones: Vec<AudioZoneDefinition>,
    pub weather_zones: Vec<WeatherZoneDefinition>,
}

impl WorldLayout {
    pub fn empty() -> Self {
        Self {
            camps: Vec::new(),
            ruins: Vec::new(),
            anomalies: Vec::new(),
            traders: Vec::new(),
            habitats: Vec::new(),
            roads: Vec::new(),
            audio_zones: Vec::new(),
            weather_zones: Vec::new(),
        }
    }

    pub fn vertical_slice_skeleton() -> Self {
        Self {
            camps: vec![
                CampDefinition {
                    name: "Rookie Camp".into(),
                    center: [500.0, 500.0],
                    radius: 80.0,
                    faction: "Loners".into(),
                    initial_population: 8,
                    has_trader: true,
                    has_mechanic: false,
                    danger_memory: 0.0,
                },
                CampDefinition {
                    name: "Duty Outpost".into(),
                    center: [1200.0, 800.0],
                    radius: 60.0,
                    faction: "Duty".into(),
                    initial_population: 6,
                    has_trader: true,
                    has_mechanic: true,
                    danger_memory: 0.2,
                },
            ],
            ruins: vec![RuinDefinition {
                name: "Abandoned Factory".into(),
                center: [900.0, 1100.0],
                radius: 120.0,
                loot_tier: 2,
                structural_integrity: 0.4,
                has_anomaly: true,
            }],
            anomalies: vec![AnomalyDefinition {
                name: "Vortex Field".into(),
                center: [1400.0, 600.0],
                radius: 50.0,
                anomaly_type: "Vortex".into(),
                intensity: 0.7,
                artifact_chance: 0.15,
            }],
            traders: vec![TraderPost {
                name: "Sidorovich".into(),
                position: [510.0, 490.0],
                faction: "Traders".into(),
                specialty: "General".into(),
                price_modifier: 1.0,
                restock_interval_hours: 48.0,
            }],
            habitats: vec![
                MonsterHabitat {
                    name: "Wolf Den".into(),
                    center: [700.0, 1500.0],
                    radius: 200.0,
                    species: "Wolf".into(),
                    pack_size: 5,
                    aggression: 0.6,
                    nocturnal: false,
                },
                MonsterHabitat {
                    name: "Boar Grazing".into(),
                    center: [300.0, 1200.0],
                    radius: 300.0,
                    species: "Boar".into(),
                    pack_size: 4,
                    aggression: 0.3,
                    nocturnal: false,
                },
            ],
            roads: vec![RoadSegment {
                name: "Main Road".into(),
                points: vec![
                    [200.0, 500.0],
                    [500.0, 500.0],
                    [900.0, 600.0],
                    [1200.0, 800.0],
                ],
                width: 10.0,
                safety_level: 0.8,
            }],
            audio_zones: vec![AudioZoneDefinition {
                name: "Camp Ambience".into(),
                center: [500.0, 500.0],
                radius: 100.0,
                ambient_sound: "camp_ambience".into(),
                day_volume: 0.6,
                night_volume: 0.3,
            }],
            weather_zones: vec![WeatherZoneDefinition {
                name: "Swamp".into(),
                center: [300.0, 1400.0],
                radius: 250.0,
                base_rain: 0.3,
                base_fog: 0.5,
                temperature_offset: -2.0,
            }],
        }
    }
}
