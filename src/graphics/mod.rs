//! Graphics subsystem - rendering, shaders, and visual effects.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `renderer`, `mesh`, `camera`, `pbr` - production, core rendering
//! - `skybox`, `sky`, `atmosphere` - production, sky/weather
//! - `particles` - partial, needs integration
//! - `taa`, `ssr`, `volumetric` - partial, advanced effects
//! - `gpu_jobs`, `lighting` - partial, forward path only
//! - `deformation_shader`, `gore_mesh` - experimental
//! - `shader_loader` - partial, external shader loading (D.1)

pub mod art_direction;
pub mod atmosphere;
pub mod camera;
pub mod contact_shadows;
pub mod debris_instancing;
pub mod decals;
pub mod decal_system;
pub mod deformation_shader;
pub mod destruction_occlusion;
pub mod gore_mesh;
pub mod gpu_culling;
pub mod gpu_jobs;
pub mod ibl;
pub mod lighting;
pub mod lod;
pub mod mesh;
pub mod model_loader;
pub mod particles;
pub mod pbr;
pub mod postprocess;
pub mod quality;
pub mod reflection_probes;
pub mod render_validation;
pub mod render_system;
pub mod renderer;
pub mod shader_loader;
pub mod shadow;
pub mod skinning;
pub mod skybox;
pub mod ssr;
pub mod surface_state_render;
pub mod taa;
pub mod terrain;
pub mod vegetation;
pub mod vegetation_semantics;
pub mod visibility;
pub mod volumetric;
pub mod sky;
