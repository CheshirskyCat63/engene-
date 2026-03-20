use std::collections::HashMap;
use std::sync::Arc;

use engine_core::config::{load_config, ConfigEnvelope};
use engine_core::plugin::EngineBuilder;
use crate::game::ai::combat_tactics::tactics::TacticProfile;
use crate::game::ai_config::AiConfigPlugin;
use crate::game::combat_plugin::CombatPlugin;
use crate::game::economy_plugin::EconomyPlugin;
use crate::game::population_plugin::PopulationPlugin;
use crate::game::stalker_plugin::StalkerPlugin;
use crate::game::weapons_plugin::WeaponsPlugin;
use crate::navigation::cover_map::CoverMap;
use crate::world::heightmap::Heightmap;

use crate::runtime::bootstrap::common::{insert_explicit_game_config, GAME_CONFIG_DIR};

fn load_tactics_or_panic(config_dir: &str) -> HashMap<String, TacticProfile> {
    load_config::<ConfigEnvelope<HashMap<String, TacticProfile>>>(&format!(
        "{}/tactics.ron",
        config_dir
    ))
    .map(|e| e.data)
    .unwrap_or_else(|err| {
        panic!(
            "failed to load required tactics config at {}/tactics.ron: {}",
            config_dir, err
        )
    })
}

pub(super) fn insert_game_content_and_plugins(
    builder: &mut EngineBuilder,
    heightmap: &Arc<Heightmap>,
) {
    insert_explicit_game_config(builder, GAME_CONFIG_DIR);
    builder.add_plugin(StalkerPlugin::from_config(GAME_CONFIG_DIR));
    builder.add_plugin(WeaponsPlugin::new(GAME_CONFIG_DIR));
    builder.insert_resource(load_tactics_or_panic(GAME_CONFIG_DIR));

    let hm_ref = heightmap.clone();
    let cover_map = CoverMap::precompute(crate::world::cell::WORLD_SIZE, 20.0, &|x, z| {
        hm_ref.sample(x, z)
    });
    builder.insert_resource(cover_map);
}

pub(super) fn register_game_plugins(builder: &mut EngineBuilder) {
    builder.add_plugin(CombatPlugin);
    builder.add_plugin(AiConfigPlugin);
    builder.add_plugin(EconomyPlugin);
    builder.add_plugin(PopulationPlugin);
}
