//! Tile Wall Fracture — 3-layer destruction model for tiled walls.
//! Layers: tile shell → adhesive/plaster → support wall.
//! See TILE_WALL_FRACTURE_SPEC.md for the full contract.

use glam::Vec3;
use crate::core::ecs::Entity;
use crate::physics::ballistics::MaterialId;
use crate::physics::destruction::{DestructibleObject, DestructionNode, DestructionLink};
use crate::physics::layered_damage::{DamageLayer, DamageableObject};
use crate::physics::damage_taxonomy::DamageCapability;

pub const LAYER_TILE: u8 = 0;
pub const LAYER_ADHESIVE: u8 = 1;
pub const LAYER_SUPPORT: u8 = 2;

pub const MAT_TILE: MaterialId = 7;
pub const MAT_CONCRETE: MaterialId = 8;
pub const MAT_BRICK: MaterialId = 9;

#[derive(Clone, Debug)]
pub struct TileWallSpec {
    pub tile_cols: u32,
    pub tile_rows: u32,
    pub tile_size: f32,
    pub tile_material: MaterialId,
    pub adhesive_material: MaterialId,
    pub support_material: MaterialId,
    pub tile_strength: f32,
    pub adhesive_strength: f32,
    pub support_strength: f32,
}

impl Default for TileWallSpec {
    fn default() -> Self {
        Self {
            tile_cols: 8,
            tile_rows: 4,
            tile_size: 0.3,
            tile_material: MAT_TILE,
            adhesive_material: MAT_CONCRETE,
            support_material: MAT_BRICK,
            tile_strength: 50.0,
            adhesive_strength: 120.0,
            support_strength: 300.0,
        }
    }
}

pub fn build_tile_wall(
    entity: Entity,
    origin: Vec3,
    spec: &TileWallSpec,
) -> (DestructibleObject, DamageableObject) {
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    let mut node_id = 0u32;

    let tile_count = spec.tile_cols * spec.tile_rows;
    let mut tile_node_ids: Vec<u32> = Vec::with_capacity(tile_count as usize);
    let mut adhesive_node_ids: Vec<u32> = Vec::with_capacity(tile_count as usize);
    let mut support_node_ids: Vec<u32> = Vec::with_capacity(tile_count as usize);

    for row in 0..spec.tile_rows {
        for col in 0..spec.tile_cols {
            let x = origin.x + col as f32 * spec.tile_size;
            let y = origin.y + row as f32 * spec.tile_size;
            let z = origin.z;

            // Tile shell node
            let tile_nid = node_id;
            nodes.push(DestructionNode {
                id: tile_nid,
                position: Vec3::new(x, y, z),
                mass: 0.8,
                material: spec.tile_material,
                accumulated_stress: 0.0,
            });
            tile_node_ids.push(tile_nid);
            node_id += 1;

            // Adhesive node (behind tile)
            let adh_nid = node_id;
            nodes.push(DestructionNode {
                id: adh_nid,
                position: Vec3::new(x, y, z - 0.02),
                mass: 0.3,
                material: spec.adhesive_material,
                accumulated_stress: 0.0,
            });
            adhesive_node_ids.push(adh_nid);
            node_id += 1;

            // Support wall node
            let sup_nid = node_id;
            nodes.push(DestructionNode {
                id: sup_nid,
                position: Vec3::new(x, y, z - 0.15),
                mass: 5.0,
                material: spec.support_material,
                accumulated_stress: 0.0,
            });
            support_node_ids.push(sup_nid);
            node_id += 1;

            // Tile → adhesive link (breakable: tile detaches)
            links.push(DestructionLink {
                a: tile_nid,
                b: adh_nid,
                strength: spec.tile_strength,
                fatigue: 0.0,
                broken: false,
            });

            // Adhesive → support link (breakable: plaster detaches)
            links.push(DestructionLink {
                a: adh_nid,
                b: sup_nid,
                strength: spec.adhesive_strength,
                fatigue: 0.0,
                broken: false,
            });
        }
    }

    // Lateral links between adjacent tiles (crack propagation)
    for row in 0..spec.tile_rows {
        for col in 0..spec.tile_cols {
            let idx = (row * spec.tile_cols + col) as usize;
            let tile_nid = tile_node_ids[idx];

            if col + 1 < spec.tile_cols {
                let right = tile_node_ids[idx + 1];
                links.push(DestructionLink {
                    a: tile_nid,
                    b: right,
                    strength: spec.tile_strength * 0.6,
                    fatigue: 0.0,
                    broken: false,
                });
            }
            if row + 1 < spec.tile_rows {
                let above = tile_node_ids[idx + spec.tile_cols as usize];
                links.push(DestructionLink {
                    a: tile_nid,
                    b: above,
                    strength: spec.tile_strength * 0.6,
                    fatigue: 0.0,
                    broken: false,
                });
            }
        }
    }

    // Lateral links between support nodes (structural continuity)
    for row in 0..spec.tile_rows {
        for col in 0..spec.tile_cols {
            let idx = (row * spec.tile_cols + col) as usize;
            let sup_nid = support_node_ids[idx];

            if col + 1 < spec.tile_cols {
                let right = support_node_ids[idx + 1];
                links.push(DestructionLink {
                    a: sup_nid,
                    b: right,
                    strength: spec.support_strength,
                    fatigue: 0.0,
                    broken: false,
                });
            }
            if row + 1 < spec.tile_rows {
                let above = support_node_ids[idx + spec.tile_cols as usize];
                links.push(DestructionLink {
                    a: sup_nid,
                    b: above,
                    strength: spec.support_strength,
                    fatigue: 0.0,
                    broken: false,
                });
            }
        }
    }

    let dest_obj = DestructibleObject::new(entity, nodes, links);

    let damage_obj = DamageableObject {
        entity,
        capability: DamageCapability::all(),
        layers: vec![
            DamageLayer {
                material: spec.tile_material,
                thickness: 0.01,
                integrity: 1.0,
                adhesion: 0.6,
                accumulated_stress: 0.0,
                thermal_damage: 0.0,
                moisture_damage: 0.0,
            },
            DamageLayer {
                material: spec.adhesive_material,
                thickness: 0.02,
                integrity: 1.0,
                adhesion: 0.8,
                accumulated_stress: 0.0,
                thermal_damage: 0.0,
                moisture_damage: 0.0,
            },
            DamageLayer {
                material: spec.support_material,
                thickness: 0.15,
                integrity: 1.0,
                adhesion: 1.0,
                accumulated_stress: 0.0,
                thermal_damage: 0.0,
                moisture_damage: 0.0,
            },
        ],
        structural_section: None,
    };

    (dest_obj, damage_obj)
}

#[derive(Clone, Copy, Debug)]
pub enum WeaponImpactType {
    Pistol,
    Rifle,
    Shotgun,
    GrenadeBlast,
}

impl WeaponImpactType {
    pub fn energy(&self) -> f32 {
        match self {
            Self::Pistol => 30.0,
            Self::Rifle => 80.0,
            Self::Shotgun => 120.0,
            Self::GrenadeBlast => 400.0,
        }
    }

    pub fn contact_area(&self) -> f32 {
        match self {
            Self::Pistol => 0.0001,
            Self::Rifle => 0.00005,
            Self::Shotgun => 0.02,
            Self::GrenadeBlast => 0.5,
        }
    }

    pub fn spread_radius(&self) -> f32 {
        match self {
            Self::Pistol => 0.05,
            Self::Rifle => 0.03,
            Self::Shotgun => 0.4,
            Self::GrenadeBlast => 2.0,
        }
    }
}

/// Per-weapon tile wall impact behavior as specified in TILE_WALL_FRACTURE_SPEC.md.
pub fn apply_weapon_to_tile_wall(
    wall: &mut DestructibleObject,
    hit_position: Vec3,
    weapon: WeaponImpactType,
) -> TileWallImpactResult {
    let events = wall.apply_impulse(hit_position, weapon.energy());

    let broken_links: usize = wall.links.iter().filter(|l| l.broken).count();
    let total_links = wall.links.len();
    let clusters = wall.find_clusters_public();
    let detached_count = if clusters.len() > 1 { clusters.len() - 1 } else { 0 };

    TileWallImpactResult {
        weapon,
        hit_position,
        broken_link_count: broken_links as u32,
        total_link_count: total_links as u32,
        fragment_clusters: clusters.len() as u32,
        detached_tile_count: detached_count as u32,
        events,
    }
}

#[derive(Clone, Debug)]
pub struct TileWallImpactResult {
    pub weapon: WeaponImpactType,
    pub hit_position: Vec3,
    pub broken_link_count: u32,
    pub total_link_count: u32,
    pub fragment_clusters: u32,
    pub detached_tile_count: u32,
    pub events: Vec<crate::physics::destruction::DestructionEvent>,
}
