use crate::core::game_config::GameConfig;
use crate::core::plugin::{EngineBuilder, Plugin};
use crate::game::rules::GameRules;

pub struct StalkerPlugin {
    data_dir: String,
}

impl StalkerPlugin {
    pub fn from_config(data_dir: &str) -> Self {
        Self {
            data_dir: data_dir.to_string(),
        }
    }
}

impl Plugin for StalkerPlugin {
    fn name(&self) -> &str {
        "StalkerPlugin"
    }

    fn build(&self, builder: &mut EngineBuilder) {
        let config = GameConfig::load_from_dir(&self.data_dir);
        let rules = GameRules::default();

        println!("[stalker] loaded {} system entries from rules", rules.systems.len());
        println!("[stalker] L0 radius: {}, L1: {}, L2: {}",
            rules.simulation_radii.l0,
            rules.simulation_radii.l1,
            rules.simulation_radii.l2,
        );

        builder.insert_resource(config);
        builder.insert_resource(rules);
    }
}
