use glam::Vec3;

use crate::core::ecs::Entity;
use crate::world::damage_profiles::{DebrisProfileId, FracturePatternId};
use crate::world::surface_db::MaterialId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecalType {
    BulletHole,
    Crack,
    Scorch,
    BloodSplat,
    WaterStain,
    Dent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyZone {
    Head,
    Neck,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    Pelvis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundClass {
    ConcreteBullet,
    WoodBullet,
    MetalBullet,
    GlassBreak,
    FleshHit,
    BoneSnap,
    Explosion,
    StructuralCreak,
    StructuralCollapse,
    WaterSplash,
    ClothTear,
    DirtImpact,
    ClothScrape,
    StoneScrape,
}

#[derive(Clone, Debug)]
pub enum DamageResponse {
    SurfaceMarked {
        position: Vec3,
        material: MaterialId,
        decal_type: DecalType,
        intensity: f32,
    },
    LayerFractured {
        entity: Entity,
        layer_idx: u8,
        pattern: FracturePatternId,
        residual_energy: f32,
    },
    LayerDetached {
        entity: Entity,
        layer_idx: u8,
        fragment_count: u8,
    },
    StructuralDamage {
        entity: Entity,
        section_id: u32,
        energy: f32,
    },
    ObjectFragmented {
        entity: Entity,
        section_id: u32,
        cluster_count: u32,
    },
    CollapseTriggered {
        entity: Entity,
        section_id: u32,
    },
    BodyZoneDamaged {
        entity: Entity,
        zone: BodyZone,
        damage: f32,
        penetrated_layers: u8,
    },
    JointBroken {
        entity: Entity,
        joint_id: u8,
    },
    BleedStarted {
        entity: Entity,
        zone: BodyZone,
        rate: f32,
    },
    Ricochet {
        position: Vec3,
        direction: Vec3,
        energy: f32,
    },
    Penetrated {
        exit_pos: Vec3,
        exit_dir: Vec3,
        remaining_energy: f32,
    },
    DebrisSpawned {
        position: Vec3,
        profile: DebrisProfileId,
        count: u16,
    },
    AudioTrigger {
        position: Vec3,
        sound_class: SoundClass,
        intensity: f32,
    },
    TerrainDeformed {
        center: Vec3,
        radius: f32,
        depth: f32,
    },
    SurfaceStateUpdated {
        cell_x: u32,
        cell_z: u32,
        mask_type: SurfaceMaskType,
        delta: f32,
    },
    WorldTopologyChanged {
        position: Vec3,
        radius: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceMaskType {
    Dirt,
    Wetness,
    Scorch,
    Wear,
    BloodStain,
    ImpactDensity,
    CrackPersistence,
}

pub struct ResponseAggregator;

impl ResponseAggregator {
    pub fn collect(responses: &mut Vec<DamageResponse>, new: DamageResponse) {
        responses.push(new);
    }
}
