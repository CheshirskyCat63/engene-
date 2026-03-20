pub mod access;
pub mod commands;
pub mod ecs_mechanics;
pub mod entity;
pub mod parallel_validation;
pub mod persistent_id;
pub mod query_contract;
pub mod sparse_set;
pub mod system_descriptor;

pub use ecs_mechanics::EcsMechanics;
pub use ecs_mechanics::GenEntity;
pub use entity::Entity;

pub mod api {
/// Stable crate identifier.
pub const CRATE: &str = "engine_ecs";
pub use crate::access;
pub use crate::commands;
pub use crate::entity::Entity;
pub use crate::parallel_validation;
pub use crate::persistent_id;
pub use crate::query_contract;
pub use crate::sparse_set;
pub use crate::system_descriptor;
}
