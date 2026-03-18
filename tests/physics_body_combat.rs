//! Integration tests for Physics, Body simulation, and Combat systems.
//! Split into domain sections to reduce giant-file cognitive load.

use engene::body::anatomy::{BleedPoint, BodyState as AnatomyBodyState, JointInfo, ZoneState};
use engene::body::blood::{compute_blood_lod, BloodLod};
use engene::body::body_damage::apply_zone_damage;
use engene::body::body_response::{BodyPhysicalResponseCache, PhysicalResponseTier};
use engene::body::body_store::{BodyHandle, BodyStateStore};
use engene::body::death_pipeline::{CorpseManager, CorpseState, DeathState};
use engene::body::dismemberment::{check_dismemberment, severed_zones};
use engene::core::ecs::Ecs;
use engene::core::events::EventBus;
use engene::game::ai::body::{is_night, time_of_day_mult, BodyState as AiBodyState};
use engene::game::ai::combat::{resolve_combat, HitLocation, StaggerState};
use engene::navigation::path_cache::PathCache;
use engene::navigation::world_graph::{LocationId, WorldGraph};
use engene::physics::ballistics::{
    BallisticEvent, BallisticsSystem, ImpactResult, MaterialProps, MaterialTable, Projectile,
};
use engene::physics::building::{
    BuildingDescriptor, SectionDescriptor, SectionNeighbor, SectionType, StructuralSection,
};
use engene::physics::chain_reactions::{ChainEvent, ChainReactionQueue};
use engene::physics::collapse_solver::{
    evaluate_failure, evaluate_hanging, CollapseResult, FailureMode,
};
use engene::physics::collision::{check_overlap, resolve_collisions};
use engene::physics::damage_pipeline::orchestrator::DamageOrchestrator;
use engene::physics::damage_pipeline::response_aggregator::{
    BodyZone, DamageResponse, DecalType, ResponseAggregator, SoundClass,
};
use engene::physics::damage_taxonomy::{DamageCapability, DamageClass};
use engene::physics::destruction::{
    DestructibleObject, DestructionEvent, DestructionLink, DestructionLod, DestructionNode,
    DestructionSystem,
};
use engene::physics::fire::{FireCell, FireGrid, FireState};
use engene::physics::impact_event::{ImpactEvent, ProjectileInfo, StressEvent};
use engene::physics::layered_damage::{DamageLayer, DamageableObject, DamageableStore};
use engene::physics::material_fracture::{compute_fracture, FractureResult};
use engene::physics::movement::{apply_velocity, move_toward, Velocity};
use engene::physics::physics::PhysicsSystem;
use engene::physics::secondary_impacts::{fragment_to_impact, generate_fragments, DebrisFragment};
use engene::physics::sim_lod::PhysicsLod;
use engene::physics::soft_state::{ObjectCondition, SoftDamageState};
use engene::physics::structural_load::{cascade_collapse, redistribute_loads};
use engene::physics::water::{WaterCell, WaterGrid};
use engene::world::components::*;
use engene::world::fields::WorldFields;
use engene::world::heightmap::Heightmap;
use engene::world::surface_db::{ResponseClass, SurfaceMaterial};
use glam::Vec3;
use std::sync::Arc;

#[path = "physics_body_combat/section_1.rs"]
mod section_1;
#[path = "physics_body_combat/section_2.rs"]
mod section_2;
#[path = "physics_body_combat/section_3.rs"]
mod section_3;
#[path = "physics_body_combat/section_4.rs"]
mod section_4;
#[path = "physics_body_combat/section_5.rs"]
mod section_5;

pub(crate) fn make_section(id: u32, integrity: f32, neighbor_count: usize) -> StructuralSection {
    StructuralSection {
        id,
        node_ids: vec![id],
        section_type: SectionType::Wall,
        integrity,
        neighbors: (0..neighbor_count)
            .map(|i| SectionNeighbor {
                section_id: i as u32 + 100,
                load_transfer: if i == 0 { 0.5 } else { 0.0 },
                collapse_priority: 0,
            })
            .collect(),
    }
}
