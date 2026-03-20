use crate::core::ecs::Entity;
use engine_physics::ballistics::MaterialId;
use glam::Vec3;

// ---------------------------------------------------------------------------
// Event Classification: A = Authoritative, B = Consequence
// Delivery Semantics per event type documented inline
// ---------------------------------------------------------------------------

// ===== CLASS A: AUTHORITATIVE WORLD EVENTS =====
// These change world truth. Replay-relevant. must_deliver.

#[derive(Clone, Debug)]
pub struct ImpactEvent {
    pub position: Vec3,
    pub direction: Vec3,
    pub energy: f32,
    pub material_hit: MaterialId,
    pub instigator: Option<Entity>,
    pub target_entity: Option<Entity>,
}
// Delivery: must_deliver, replay_relevant

#[derive(Clone, Debug)]
pub struct StructuralCollapse {
    pub entity: Entity,
    pub position: Vec3,
    pub cluster_count: u32,
}
// Delivery: must_deliver, replay_relevant

#[derive(Clone, Debug)]
pub struct TerrainDeformed {
    pub position: Vec3,
    pub radius: f32,
    pub depth: f32,
}
// Delivery: must_deliver, replay_relevant

#[derive(Clone, Debug)]
pub struct TerrainChanged {
    pub patches: Vec<(i32, i32)>,
}
// Delivery: must_deliver, replay_relevant

#[derive(Clone, Debug)]
pub struct NavUpdated {
    pub dirty_cells_processed: usize,
}
// Delivery: must_deliver

#[derive(Clone, Debug)]
pub struct WorldTopologyChanged {
    pub position: Vec3,
    pub radius: f32,
    pub cause: TopologyChangeCause,
}
// Delivery: must_deliver, replay_relevant

#[derive(Clone, Debug)]
pub enum TopologyChangeCause {
    StructuralCollapse,
    TerrainDeformation,
    Explosion,
}

#[derive(Clone, Debug)]
pub struct EntityDied {
    pub entity: Entity,
    pub position: Vec3,
    pub killer: Option<Entity>,
}
// Delivery: must_deliver, replay_relevant, sticky (until decay)

#[derive(Clone, Debug)]
pub struct BodyZoneDamaged {
    pub entity: Entity,
    pub zone: u8,
    pub damage: f32,
    pub position: Vec3,
}
// Delivery: must_deliver, replay_relevant

/// Combat hit with HitLocation for BodySystem zone integrity reduction.
/// hit_zone: 0=Head, 1=Torso, 2=Arms, 3=Legs
#[derive(Clone, Debug)]
pub struct CombatHit {
    pub entity: Entity,
    pub hit_zone: u8,
    pub damage: f32,
}
// Delivery: must_deliver, replay_relevant

#[derive(Clone, Debug)]
pub struct FireSpreadEvent {
    pub cell_x: i32,
    pub cell_z: i32,
    pub intensity: f32,
}
// Delivery: must_deliver, replay_relevant

#[derive(Clone, Debug)]
pub struct CoverChanged {
    pub cells_updated: usize,
}
// Delivery: must_deliver

// ===== CLASS B: CONSEQUENCE / PRESENTATION EVENTS =====
// Reactions to already-changed truth. can_batch, can_drop_under_pressure.

#[derive(Clone, Debug)]
pub struct SurfaceDamaged {
    pub position: Vec3,
    pub material: MaterialId,
    pub intensity: f32,
}
// Delivery: can_batch, can_drop_under_pressure, NOT replay_relevant

#[derive(Clone, Debug)]
pub struct SoundTrigger {
    pub position: Vec3,
    pub kind: SoundTriggerKind,
    pub volume: f32,
}
// Delivery: can_batch, can_drop_under_pressure

#[derive(Clone, Debug)]
pub enum SoundTriggerKind {
    Impact { material: MaterialId },
    Collapse,
    Fire,
    Pain,
    Footstep { material: MaterialId },
    GunShot,
}

#[derive(Clone, Debug)]
pub struct DecalSpawn {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: MaterialId,
    pub size: f32,
}
// Delivery: can_batch, can_drop_under_pressure

#[derive(Clone, Debug)]
pub struct ParticleBurst {
    pub position: Vec3,
    pub kind: ParticleKind,
    pub count: u32,
}
// Delivery: can_batch, can_drop_under_pressure

#[derive(Clone, Debug)]
pub enum ParticleKind {
    Debris { material: MaterialId },
    Dust,
    Blood,
    Spark,
    Smoke,
}

#[derive(Clone, Debug)]
pub struct GoreMeshSpawn {
    pub entity: Entity,
    pub zone: u8,
    pub position: Vec3,
    pub intensity: f32,
}
// Delivery: can_batch, can_drop_under_pressure

// ===== STATE TRANSITION OWNERSHIP =====
// Each authoritative event has exactly ONE legal producer:
//
// ImpactEvent            -> BallisticsSystem (exclusive)
// StructuralCollapse     -> DestructionSystem (exclusive)
// TerrainDeformed        -> DamageOrchestrator (exclusive)
// TerrainChanged         -> TerrainDeformationSystem (exclusive)
// NavUpdated             -> NavDirtyTracker (exclusive)
// WorldTopologyChanged   -> DestructionSystem (exclusive)
// EntityDied             -> AiSystem/CombatSystem (exclusive)
// BodyZoneDamaged        -> DamageOrchestrator/BodyResolver (exclusive)
// FireSpreadEvent        -> FireGrid (exclusive)
// CoverChanged           -> CoverRefreshQueue (exclusive)
//
// NO SHORTCUT PRODUCERS ALLOWED.
