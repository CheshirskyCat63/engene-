use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::plugin::{EngineBuilder, Plugin};
use crate::physics::ballistics::{BallisticsSystem, MaterialProps};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDef {
    pub muzzle_velocity: f32,
    pub bullet_mass: f32,
    pub drag_coefficient: f32,
    pub damage_base: f32,
    pub fire_rate: f32,
    pub magazine_size: u32,
    pub reload_time: f32,
    pub accuracy_spread: f32,
}

#[derive(Debug, Clone)]
pub struct WeaponRegistry {
    pub weapons: HashMap<String, WeaponDef>,
    pub name_to_id: HashMap<String, u16>,
}

impl WeaponRegistry {
    pub fn get_by_name(&self, name: &str) -> Option<&WeaponDef> {
        self.weapons.get(name)
    }

    pub fn get_id(&self, name: &str) -> Option<u16> {
        self.name_to_id.get(name).copied()
    }
}

pub struct WeaponsPlugin {
    data_dir: String,
}

impl WeaponsPlugin {
    pub fn new(data_dir: &str) -> Self {
        Self {
            data_dir: data_dir.to_string(),
        }
    }
}

impl Plugin for WeaponsPlugin {
    fn name(&self) -> &str {
        "WeaponsPlugin"
    }

    fn build(&self, builder: &mut EngineBuilder) {
        let mut ballistics = BallisticsSystem::new();

        if let Some(config) = builder
            .resources
            .get::<crate::core::game_config::GameConfig>()
        {
            let mut next_id: u16 = 0;
            for (name, mat_cfg) in &config.materials {
                let props = MaterialProps {
                    hardness: mat_cfg.hardness,
                    penetration_resistance: mat_cfg.penetration_resistance,
                    density: mat_cfg.density,
                };
                ballistics.material_table.register(name, next_id, props);
                next_id += 1;
            }
            println!(
                "[weapons] loaded {} materials into ballistics table",
                next_id
            );
        }

        let mut weapon_registry = WeaponRegistry {
            weapons: HashMap::new(),
            name_to_id: HashMap::new(),
        };

        let weapons_path = format!("{}/weapons.ron", self.data_dir);
        if let Ok(envelope) = crate::core::config::load_config::<
            crate::core::config::ConfigEnvelope<HashMap<String, WeaponDef>>,
        >(&weapons_path)
        {
            let mut next_id: u16 = 0;
            for (name, def) in envelope.data {
                weapon_registry.name_to_id.insert(name.clone(), next_id);
                weapon_registry.weapons.insert(name, def);
                next_id += 1;
            }
            println!(
                "[weapons] loaded {} weapon definitions",
                weapon_registry.weapons.len()
            );
        }

        builder.insert_resource(ballistics);
        builder.insert_resource(weapon_registry);
    }
}
