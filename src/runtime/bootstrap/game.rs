//! Game runtime assembly orchestration.

use std::sync::Arc;

use crate::core::plugin::EngineBuilder;
use crate::core::world_state_authority;
use crate::runtime::bootstrap::common::finalize_builder;
use crate::runtime::bootstrap::game_plugins::{
    insert_game_content_and_plugins, register_game_plugins,
};
use crate::runtime::bootstrap::game_resources::{
    insert_game_tail_resources, insert_headless_runtime_resources,
    insert_vertical_runtime_resources, insert_world_and_damage_resources,
};
use crate::runtime::bootstrap::game_systems::{
    register_headless_systems, register_vertical_systems,
};
use crate::runtime::bootstrap::GameRuntimeAssembly;
use crate::world::heightmap::Heightmap;
use crate::world::resources::ResourceGrid;
use crate::world::world::WorldGrid;

impl GameRuntimeAssembly {
    /// Canonical game runtime: full game + projection stack.
    pub fn vertical_slice(
        heightmap: Arc<Heightmap>,
        biomes: &[crate::world::biome::Biome],
    ) -> engine_runtime::engine::Engine {
        let grid = WorldGrid::generate();
        let authority_entries = world_state_authority::authority_matrix();

        let mut builder = EngineBuilder::new();
        builder.insert_resource(ResourceGrid::new(biomes));

        insert_world_and_damage_resources(&mut builder);
        insert_game_content_and_plugins(&mut builder, &heightmap);
        insert_vertical_runtime_resources(&mut builder, &heightmap, authority_entries);
        register_vertical_systems(&mut builder, grid, heightmap);
        register_game_plugins(&mut builder);
        insert_game_tail_resources(&mut builder, true);

        finalize_builder(builder)
    }

    /// Game simulation runtime without projection stack.
    pub fn headless(biomes: &[crate::world::biome::Biome]) -> engine_runtime::engine::Engine {
        let grid = WorldGrid::generate();
        let authority_entries = world_state_authority::authority_matrix();
        let heightmap = Arc::new(Heightmap::generate(biomes));

        let mut builder = EngineBuilder::new();
        builder.insert_resource(ResourceGrid::new(biomes));

        insert_world_and_damage_resources(&mut builder);
        insert_game_content_and_plugins(&mut builder, &heightmap);
        insert_headless_runtime_resources(&mut builder, &heightmap, authority_entries);
        register_headless_systems(&mut builder, grid, heightmap);
        register_game_plugins(&mut builder);
        insert_game_tail_resources(&mut builder, false);

        finalize_builder(builder)
    }
}
