//! Physics subsystem - simulation, destruction, and material response.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit + integration
//!
//! ## Modules
//! - `physics`, `collision`, `movement` - production, core physics
//! - `damage_pipeline`, `ballistics`, `fire` - production, damage system
//! - `destruction`, `collapse_solver` - production, structural simulation
//! - `cloth` - experimental, Rapier cloth
//! - `sim_lod` - production, LOD enforcement

pub mod ballistics;
pub mod building;
pub mod chain_reactions;
pub mod cloth;
pub mod collapse_solver;
pub mod collision;
pub mod damage_pipeline;
pub mod damage_taxonomy;
pub mod destruction;
pub mod fire;
pub mod impact_event;
pub mod layered_damage;
pub mod mass_properties;
pub mod material_fracture;
pub mod movement;
pub mod persistent_stress;
pub mod physics;
pub mod physics_bubble;
pub mod rapier_world;
pub mod secondary_impacts;
pub mod sim_lod;
pub mod soft_state;
pub mod structural_load;
pub mod tile_wall_fracture;
pub mod water;
