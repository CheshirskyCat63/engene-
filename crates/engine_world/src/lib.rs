pub mod cell;
pub mod fields;
pub mod api {
/// Stable crate identifier.
pub const CRATE: &str = "engine_world";
pub use crate::cell::{
ACTIVE_LOADED_RADIUS_CHUNKS,
CELL_SIZE,
CHUNK_CELLS,
CHUNK_SIZE_M,
GRID_SIZE,
MAX_DRAW_CALLS,
MAX_TRIANGLES,
MAX_TRIANGLES_LOW_SPEC,
PHYSICS_BUBBLE_RADIUS,
WORLD_SIZE,
};
pub use crate::fields::{
AirDensityField,
AnomalyField,
AnomalyForceType,
AnomalyForces,
AnomalyZone,
RainField,
StormCellWind,
WindField,
WorldFields,
};
}

