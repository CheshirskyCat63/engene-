use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionConfig {
    pub hunt_radius: f32,
    pub fear_radius: f32,
    pub ally_radius: f32,
    pub social_radius: f32,
    pub nearby_count_radius: f32,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            hunt_radius: 120.0,
            fear_radius: 150.0,
            ally_radius: 80.0,
            social_radius: 100.0,
            nearby_count_radius: 80.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub max_npcs: usize,
    pub max_wolves: usize,
    pub max_boars: usize,
    pub max_bloodsuckers: usize,
    pub min_wolves: usize,
    pub min_boars: usize,
    pub min_bloodsuckers: usize,
    pub respawn_interval: f32,
    pub npc_names: Vec<String>,
    pub child_names: Vec<String>,
    pub npc_age_min: f32,
    pub npc_age_max: f32,
    pub npc_max_age_min: f32,
    pub npc_max_age_max: f32,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            max_npcs: 30,
            max_wolves: 40,
            max_boars: 40,
            max_bloodsuckers: 25,
            min_wolves: 5,
            min_boars: 5,
            min_bloodsuckers: 2,
            respawn_interval: 300.0,
            npc_names: vec![
                "Viktor", "Elena", "Sasha", "Dmitri", "Irina",
                "Andrei", "Natasha", "Boris", "Yuri", "Olga",
                "Maxim", "Tatiana", "Sergei", "Anya", "Pavel",
                "Ilya", "Marina", "Roman", "Vera", "Artem",
            ].into_iter().map(String::from).collect(),
            child_names: vec![
                "Alyosha", "Misha", "Katya", "Dasha", "Pasha",
                "Kolya", "Vanya", "Sveta", "Zhenya", "Borya",
                "Lena", "Grisha", "Tonya", "Nikita", "Oleg",
            ].into_iter().map(String::from).collect(),
            npc_age_min: 80.0,
            npc_age_max: 200.0,
            npc_max_age_min: 350.0,
            npc_max_age_max: 450.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub monthly_required: f32,
    pub desperation_bandit_threshold: f32,
    pub desperation_force_bandit: f32,
    pub desperation_increase_rate: f32,
    pub desperation_decrease_on_pay: f32,
    pub initial_money_min: f32,
    pub initial_money_max: f32,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            monthly_required: 50.0,
            desperation_bandit_threshold: 0.7,
            desperation_force_bandit: 0.9,
            desperation_increase_rate: 0.3,
            desperation_decrease_on_pay: 0.1,
            initial_money_min: 20.0,
            initial_money_max: 60.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    pub daily_income: f32,
    pub danger: f32,
    pub energy_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalConfig {
    pub max_duration: f32,
    pub target_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConfig {
    pub food_density: f32,
    pub danger_level: f32,
    pub water_density: f32,
    pub night_danger_mult: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonConfig {
    pub food_regen_mult: f32,
    pub hunger_drain_mult: f32,
    pub breeding_mult: f32,
    pub danger_mult: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub seconds_per_day: f32,
    pub days_per_month: u32,
    pub sim_tick_rate: f32,
    pub l0_radius: f32,
    pub l1_radius: f32,
    pub l2_radius: f32,
    pub l0_tick_interval: u32,
    pub l1_tick_interval: u32,
    pub l2_tick_interval: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            seconds_per_day: 120.0,
            days_per_month: 30,
            sim_tick_rate: 20.0,
            l0_radius: 300.0,
            l1_radius: 5000.0,
            l2_radius: 50000.0,
            l0_tick_interval: 1,
            l1_tick_interval: 12,
            l2_tick_interval: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialConfig {
    pub flammability: f32,
    pub fuel: f32,
    pub hardness: f32,
    pub penetration_resistance: f32,
    pub density: f32,
}

// === NEW: Food Chain Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodChainConfig {
    pub predator_prey: Vec<(String, String)>,
    pub food_chain_rank: HashMap<String, u32>,
}

impl Default for FoodChainConfig {
    fn default() -> Self {
        Self {
            predator_prey: vec![
                ("Wolf".into(), "Boar".into()),
                ("Bloodsucker".into(), "Wolf".into()),
            ],
            food_chain_rank: [
                ("Boar".into(), 1),
                ("Wolf".into(), 2),
                ("Bloodsucker".into(), 3),
            ].into_iter().collect(),
        }
    }
}

// === NEW: Species Config ===

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EcosystemNeeds {
    pub hunting: f32,
    pub predator_avoidance: f32,
    pub food_chain_position: f32,
    pub territory_control: f32,
    pub migration_urge: f32,
    pub resource_competition: f32,
    pub pack_following: f32,
    pub shelter_seeking: f32,
    pub world_event_reaction: f32,
    pub prey_selection: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaseTraits {
    pub aggressiveness: f32,
    pub caution: f32,
    pub territoriality: f32,
    pub bravery: f32,
    pub pack_mentality: f32,
    pub energy_level: f32,
    pub hoarding: f32,
    pub curiosity: f32,
    pub adaptability: f32,
    pub stress_tolerance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesEntry {
    pub base_power: f32,
    pub food_value: f32,
    pub max_age: f32,
    pub mate_cooldown_days: u32,
    pub ecosystem_needs: EcosystemNeeds,
    pub base_traits: BaseTraits,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeciesConfig {
    pub species: HashMap<String, SpeciesEntry>,
}

// === NEW: Tactics Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticsConfig {
    pub flank_weight: f32,
    pub charge_weight: f32,
    pub ambush_weight: f32,
    pub retreat_weight: f32,
    pub surround_weight: f32,
    pub hit_and_run_weight: f32,
    pub hold_ground_weight: f32,
    pub retreat_health_threshold: f32,
    pub retreat_ally_loss_ratio: f32,
    pub preferred_group_size: u32,
}

impl Default for TacticsConfig {
    fn default() -> Self {
        Self {
            flank_weight: 0.2,
            charge_weight: 0.2,
            ambush_weight: 0.1,
            retreat_weight: 0.1,
            surround_weight: 0.1,
            hit_and_run_weight: 0.1,
            hold_ground_weight: 0.1,
            retreat_health_threshold: 0.25,
            retreat_ally_loss_ratio: 0.5,
            preferred_group_size: 2,
        }
    }
}

// === NEW: Rules Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRule {
    pub name: String,
    pub frequency: String,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRadii {
    pub l0: f32,
    pub l1: f32,
    pub l2: f32,
    pub l0_tick: u32,
    pub l1_tick: u32,
    pub l2_tick: u32,
}

impl Default for SimulationRadii {
    fn default() -> Self {
        Self {
            l0: 300.0,
            l1: 5000.0,
            l2: 50000.0,
            l0_tick: 1,
            l1_tick: 12,
            l2_tick: 60,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesData {
    pub systems: Vec<SystemRule>,
    pub simulation_radii: SimulationRadii,
    pub replicated_events: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesConfig {
    pub data: RulesData,
}

// === NEW: Weapons Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub damage: f32,
    pub range: f32,
    pub fire_rate: f32,
    pub accuracy: f32,
    pub noise_radius: f32,
}

impl Default for WeaponConfig {
    fn default() -> Self {
        Self {
            damage: 10.0,
            range: 100.0,
            fire_rate: 1.0,
            accuracy: 0.8,
            noise_radius: 50.0,
        }
    }
}

// === NEW: Surface Config (surfaces.ron) ===

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResponseClass {
    Ite,
    LayeredMasonry,
    AnisotropicWood,
    BrittleGlassRadial,
    DuctileMetal,
    BrittleCeramic,
    Composite,
    BiologicalSoft,
    BiologicalHard,
}

impl Default for ResponseClass {
    fn default() -> Self { Self::Ite }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceMaterial {
    pub id: u32,
    pub name: String,
    pub response_class: ResponseClass,
    pub compressive_strength: f32,
    pub tensile_strength: f32,
    pub shear_strength: f32,
    pub brittleness: f32,
    pub density: f32,
    pub elasticity: f32,
    pub penetration_resistance: f32,
    pub hardness: f32,
    pub flammability: f32,
    pub fuel_content: f32,
    pub ignition_temp: f32,
    pub thermal_conductivity: f32,
    pub porosity: f32,
    pub erosion_resistance: f32,
    pub fragmentation_coeff: f32,
    pub fracture_pattern: u32,
    pub debris_profile: u32,
    pub decal_profile: u32,
    pub dust_intensity: f32,
    pub is_biological: bool,
    pub gore_response: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurfacesConfig {
    pub materials: Vec<SurfaceMaterial>,
}

// === NEW: Material Bridge Config (material_bridge.ron) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBridge {
    pub material_id: u32,
    pub base_albedo_tint: (f32, f32, f32),
    pub roughness_range: (f32, f32),
    pub metallic: f32,
    pub normal_intensity: f32,
    pub subsurface: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioBridge {
    pub material_id: u32,
    pub impact_sound_class: String,
    pub footstep_sound_class: String,
    pub scrape_sound_class: String,
    pub break_sound_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleBridge {
    pub material_id: u32,
    pub debris_color: (f32, f32, f32),
    pub debris_size_range: (f32, f32),
    pub dust_color: (f32, f32, f32),
    pub dust_density: f32,
    pub spark_on_impact: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialBridgeConfig {
    pub render: Vec<RenderBridge>,
    pub audio: Vec<AudioBridge>,
    pub particle: Vec<ParticleBridge>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameConfig {
    pub perception: PerceptionConfig,
    pub population: PopulationConfig,
    pub economy: EconomyConfig,
    pub simulation: SimulationConfig,
    pub jobs: HashMap<String, JobConfig>,
    pub goals: HashMap<String, GoalConfig>,
    pub biomes: HashMap<String, BiomeConfig>,
    pub seasons: HashMap<String, SeasonConfig>,
    pub materials: HashMap<String, MaterialConfig>,
    // NEW
    pub food_chain: FoodChainConfig,
    pub species: SpeciesConfig,
    pub tactics: HashMap<String, TacticsConfig>,
    pub rules: RulesConfig,
    pub weapons: HashMap<String, WeaponConfig>,
    // NEW: Surfaces & Material Bridge
    pub surfaces: SurfacesConfig,
    pub material_bridge: MaterialBridgeConfig,
}

impl GameConfig {
    pub fn load_from_dir(dir: &str) -> Self {
        let mut config = Self::default();

        // Existing loaders
        if let Ok(p) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<PerceptionConfig>>(
            &format!("{}/perception.ron", dir),
        ) {
            config.perception = p.data;
        }

        if let Ok(p) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<PopulationConfig>>(
            &format!("{}/population.ron", dir),
        ) {
            config.population = p.data;
        }

        if let Ok(e) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<EconomyConfig>>(
            &format!("{}/economy.ron", dir),
        ) {
            config.economy = e.data;
        }

        if let Ok(s) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<SimulationConfig>>(
            &format!("{}/simulation.ron", dir),
        ) {
            config.simulation = s.data;
        }

        if let Ok(j) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<HashMap<String, JobConfig>>>(
            &format!("{}/jobs.ron", dir),
        ) {
            config.jobs = j.data;
        }

        if let Ok(g) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<HashMap<String, GoalConfig>>>(
            &format!("{}/goals.ron", dir),
        ) {
            config.goals = g.data;
        }

        if let Ok(b) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<HashMap<String, BiomeConfig>>>(
            &format!("{}/biomes.ron", dir),
        ) {
            config.biomes = b.data;
        }

        if let Ok(s) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<HashMap<String, SeasonConfig>>>(
            &format!("{}/seasons.ron", dir),
        ) {
            config.seasons = s.data;
        }

        if let Ok(m) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<HashMap<String, MaterialConfig>>>(
            &format!("{}/materials.ron", dir),
        ) {
            config.materials = m.data;
        }

        // NEW loaders
        if let Ok(fc) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<FoodChainConfig>>(
            &format!("{}/food_chain.ron", dir),
        ) {
            config.food_chain = fc.data;
        }

        if let Ok(sp) = load_species_config(&format!("{}/species.ron", dir)) {
            config.species = sp;
        }

        if let Ok(t) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<HashMap<String, TacticsConfig>>>(
            &format!("{}/tactics.ron", dir),
        ) {
            config.tactics = t.data;
        }

        if let Ok(r) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<RulesData>>(
            &format!("{}/rules.ron", dir),
        ) {
            config.rules.data = r.data;
        }

        if let Ok(w) = crate::core::config::load_config::<crate::core::config::ConfigEnvelope<HashMap<String, WeaponConfig>>>(
            &format!("{}/weapons.ron", dir),
        ) {
            config.weapons = w.data;
        }

        // Surfaces & Material Bridge
        if let Ok(s) = load_surfaces_config(&format!("{}/surfaces.ron", dir)) {
            config.surfaces = s;
        }

        if let Ok(m) = load_material_bridge_config(&format!("{}/material_bridge.ron", dir)) {
            config.material_bridge = m;
        }

        println!("[config] loaded GameConfig from '{}'", dir);
        println!(
            "  jobs: {}, goals: {}, biomes: {}, materials: {}, tactics: {}, weapons: {}, surfaces: {}",
            config.jobs.len(),
            config.goals.len(),
            config.biomes.len(),
            config.materials.len(),
            config.tactics.len(),
            config.weapons.len(),
            config.surfaces.materials.len(),
        );

        config
    }
    
    /// Get predator-prey relationship
    pub fn is_predator_of(&self, predator: &str, prey: &str) -> bool {
        self.food_chain.predator_prey.iter()
            .any(|(p, pr)| p == predator && pr == prey)
    }
    
    /// Get food chain rank for a species
    pub fn chain_rank(&self, species: &str) -> u32 {
        self.food_chain.food_chain_rank.get(species).copied().unwrap_or(0)
    }
    
    /// Get tactics for a species
    pub fn get_tactics(&self, species: &str) -> Option<&TacticsConfig> {
        self.tactics.get(species)
    }
    
    /// Get weapon config
    pub fn get_weapon(&self, name: &str) -> Option<&WeaponConfig> {
        self.weapons.get(name)
    }
}

/// Load species config with special handling for the nested structure
fn load_species_config(path: &str) -> Result<SpeciesConfig, Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Read;
    
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    // Parse the outer envelope
    #[derive(Deserialize)]
    struct SpeciesEnvelope {
        #[allow(dead_code)]
        schema_version: u32,
        data: HashMap<String, SpeciesEntry>,
    }
    
    let envelope: SpeciesEnvelope = ron::from_str(&content)?;
    Ok(SpeciesConfig { species: envelope.data })
}

/// Load surfaces config with special handling for the nested structure
fn load_surfaces_config(path: &str) -> Result<SurfacesConfig, Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Read;
    
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    // Parse the outer envelope
    #[derive(Deserialize)]
    struct SurfacesEnvelope {
        #[allow(dead_code)]
        schema_version: u32,
        materials: Vec<SurfaceMaterial>,
    }
    
    let envelope: SurfacesEnvelope = ron::from_str(&content)?;
    Ok(SurfacesConfig { materials: envelope.materials })
}

/// Load material bridge config
fn load_material_bridge_config(path: &str) -> Result<MaterialBridgeConfig, Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Read;
    
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    let config: MaterialBridgeConfig = ron::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_config_default() {
        let config = PerceptionConfig::default();
        assert!(config.hunt_radius > 0.0);
        assert!(config.fear_radius > config.hunt_radius);
        assert!(config.ally_radius > 0.0);
    }

    #[test]
    fn test_population_config_default() {
        let config = PopulationConfig::default();
        assert!(config.max_npcs > 0);
        assert!(config.max_wolves > 0);
        assert!(config.respawn_interval > 0.0);
        assert!(!config.npc_names.is_empty());
        assert!(!config.child_names.is_empty());
    }

    #[test]
    fn test_economy_config_default() {
        let config = EconomyConfig::default();
        assert!(config.monthly_required > 0.0);
        assert!(config.desperation_bandit_threshold > 0.0 && config.desperation_bandit_threshold < 1.0);
        assert!(config.desperation_force_bandit > config.desperation_bandit_threshold);
    }

    #[test]
    fn test_simulation_config_default() {
        let config = SimulationConfig::default();
        assert!(config.seconds_per_day > 0.0);
        assert!(config.days_per_month > 0);
        assert!(config.l0_radius < config.l1_radius);
        assert!(config.l1_radius < config.l2_radius);
    }

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert!(config.perception.hunt_radius > 0.0);
        assert!(config.population.max_npcs > 0);
        assert!(config.economy.monthly_required > 0.0);
    }

    #[test]
    fn test_food_chain_predator_prey() {
        let config = GameConfig::default();
        assert!(config.is_predator_of("Wolf", "Boar"));
        assert!(!config.is_predator_of("Boar", "Wolf"));
    }

    #[test]
    fn test_food_chain_rank() {
        let config = GameConfig::default();
        assert_eq!(config.chain_rank("Bloodsucker"), 3);
        assert_eq!(config.chain_rank("Wolf"), 2);
        assert_eq!(config.chain_rank("Boar"), 1);
        assert_eq!(config.chain_rank("Unknown"), 0);
    }

    #[test]
    fn test_tactics_config_default() {
        let config = TacticsConfig::default();
        assert!(config.retreat_health_threshold > 0.0 && config.retreat_health_threshold < 1.0);
        assert!(config.preferred_group_size > 0);
    }

    #[test]
    fn test_weapon_config_default() {
        let config = WeaponConfig::default();
        assert!(config.damage > 0.0);
        assert!(config.range > 0.0);
        assert!(config.accuracy > 0.0 && config.accuracy <= 1.0);
    }

    #[test]
    fn test_response_class_default() {
        let class = ResponseClass::default();
        assert_eq!(class, ResponseClass::Ite);
    }

    #[test]
    fn test_surfaces_config_default() {
        let config = SurfacesConfig::default();
        assert!(config.materials.is_empty());
    }

    #[test]
    fn test_material_bridge_config_default() {
        let config = MaterialBridgeConfig::default();
        assert!(config.render.is_empty());
        assert!(config.audio.is_empty());
        assert!(config.particle.is_empty());
    }
}
