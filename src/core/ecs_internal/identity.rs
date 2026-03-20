//! ECS Identity Layer - Entity lifecycle and persistence.
//! Phase 2a.2: Now re-exports from engine_ecs (ownership transferred).
//! This module handles entity ID allocation and persistent ID mapping.

pub use engine_ecs::ecs_mechanics::EcsMechanics;
pub use engine_ecs::Entity;
pub use engine_ecs::GenEntity;
pub use engine_ecs::persistent_id::{DuplicateIdError, IdentityRegistry, PersistentEntityId};

// Transitional alias for backward compatibility
pub type IdentitySubsystem = engine_ecs::persistent_id::IdentityRegistry;
