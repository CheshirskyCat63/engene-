//! Graphics subsystem - render boundary and submission
//!
//! # Status: NARROWED to core render surface
//! # Integration: enabled  
//! # Tests: integration only
//!
//! ## CANONICAL PUBLIC API (production)
//! - `renderer` - main renderer
//! - `render_system` - render system
//! - `camera` - camera
//! - `mesh` - mesh handling
//! - `lighting` - lighting
//! - `pbr` - PBR shading
//! - `shadow` - shadows
//! - `shader_loader` - shader loading
//! - `sky` - sky system
//!
//! ## QUARANTINE (not exported - checking for consumers)
//! - art_direction, atmosphere, contact_shadows, debris_instancing
//! - decal_system, decals, deformation_shader, destruction_occlusion
//! - gore_mesh, gpu_culling, gpu_jobs, ibl, lod, particles, postprocess
//! - quality, reflection_probes, render_validation, skinning, skybox
//! - ssr, surface_state_render, taa, terrain, vegetation, vegetation_semantics
//! - visibility, volumetric

pub mod api {
    /// Stable crate identifier.
    pub const CRATE: &str = "engine_render";
}

// ================================================================================
// CANONICAL PUBLIC API - CORE RENDER ONLY
// ================================================================================
pub mod camera;
pub mod lighting;
pub mod mesh;
pub mod pbr;
pub mod render_system;
pub mod renderer;
pub mod shader_loader;
pub mod shadow;
pub mod sky;

// ================================================================================
// QUARANTINE - NOT EXPORTED
// These modules exist but are NOT public API.
// If no consumers found, they will be deleted in next cleanup batch.
// ================================================================================
// pub mod art_direction;              // QUARANTINE - experimental
// pub mod atmosphere;                 // QUARANTINE - experimental
// pub mod contact_shadows;            // QUARANTINE - niche effect
// pub mod debris_instancing;          // QUARANTINE - niche effect
// pub mod decal_system;               // QUARANTINE - niche effect
// pub mod decals;                     // QUARANTINE - niche effect
// pub mod deformation_shader;         // QUARANTINE - experimental
// pub mod destruction_occlusion;      // QUARANTINE - experimental
// pub mod gore_mesh;                  // QUARANTINE - experimental
// pub mod gpu_culling;                // QUARANTINE - experimental
// pub mod gpu_jobs;                   // QUARANTINE - experimental
// pub mod ibl;                        // QUARANTINE - experimental
// pub mod lod;                        // QUARANTINE - not integrated
// pub mod particles;                  // QUARANTINE - not integrated
// pub mod postprocess;                // QUARANTINE - not integrated
// pub mod quality;                    // QUARANTINE - config
// pub mod reflection_probes;          // QUARANTINE - experimental
// pub mod render_validation;          // QUARANTINE - debug only
// pub mod skinning;                   // QUARANTINE - not integrated
// pub mod skybox;                     // QUARANTINE - duplicate of sky
// pub mod ssr;                        // QUARANTINE - experimental
// pub mod surface_state_render;       // QUARANTINE - not integrated
// pub mod taa;                        // QUARANTINE - experimental
// pub mod terrain;                    // QUARANTINE - not integrated
// pub mod vegetation;                 // QUARANTINE - not integrated
// pub mod vegetation_semantics;       // QUARANTINE - not integrated
// pub mod visibility;                 // QUARANTINE - not integrated
// pub mod volumetric;                 // QUARANTINE - experimental

// Re-export main types
pub use mesh::EntityInstance;
pub use renderer::Renderer;
