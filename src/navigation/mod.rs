//! Navigation subsystem - pathfinding, cover, and movement.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `navigation`, `navmesh`, `hpa_star` - production, pathfinding
//! - `cover_map`, `avoidance` - production, tactical movement
//! - `dynamic_nav_update` - production, runtime updates

pub mod avoidance;
pub mod breach_analysis;
pub mod cover_map;
pub mod dynamic_nav_update;
pub mod hpa_star;
pub mod local_cover_refresh;
pub mod navigation;
pub mod navmesh;
pub mod path_cache;
pub mod world_graph;
