//! Transitional facade - redirects to internal module split.
//! Phase 2a.1: Ecs has been internally split into subsystems.
//! This file remains for backward compatibility during transition.
//!
//! DEPRECATED: Use `crate::core::ecs::Ecs` instead.
//! This facade will be removed once all consumers migrate to the new structure.

pub use crate::core::ecs_internal::Ecs;
pub use crate::core::ecs_internal::Entity;
pub use crate::core::ecs_internal::GenEntity;

// Legacy re-exports for backward compatibility
pub use crate::core::persistent_id::{DuplicateIdError, PersistentEntityId};
