//! Runtime systems tests for ENGENE.
//! Split by responsibility buckets to keep ownership and navigation clear.

use engene::core::ecs::Ecs;
use engene::core::engine::Engine;
use engene::core::system::EngineSystem;
use engene::game::economy::trader_economy::TraderState;
use engene::runtime::bootstrap::{GameRuntimeAssembly, ToolsRuntimeAssembly};
use engene::simulation::camp_simulation::CampState;
use engene::simulation::role_simulation::{NpcRole, RoleBehavior};
use engene::simulation::simulation_level::{
    level_for_distance, should_tick, L0_RADIUS, L1_RADIUS, L1_TICK_INTERVAL, L2_RADIUS,
    L2_TICK_INTERVAL,
};
use engene::simulation::world_milestones::WorldMilestoneTracker;
use engene::world::biome::Biome;
use engene::world::components::*;
use engene::world::heightmap::Heightmap;
use engene::world::world::WorldGrid;
use std::sync::Arc;

#[path = "runtime_systems/category_1_startup.rs"]
mod category_1_startup;
#[path = "runtime_systems/category_2_fixed_tick.rs"]
mod category_2_fixed_tick;
#[path = "runtime_systems/category_3_resources.rs"]
mod category_3_resources;
#[path = "runtime_systems/category_4_low_spec.rs"]
mod category_4_low_spec;
#[path = "runtime_systems/category_5_producer_consumer.rs"]
mod category_5_producer_consumer;
#[path = "runtime_systems/category_6_camp_sim.rs"]
mod category_6_camp_sim;
#[path = "runtime_systems/category_7_role_sim.rs"]
mod category_7_role_sim;
