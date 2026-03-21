// Entity type - migrated from root src/core/ecs.rs
// This is the fundamental entity identifier used throughout the ECS system.

/// Entity identifier - a simple u64 wrapper for ECS entity references.
/// This type is owned by engine_ecs as the canonical entity definition.
pub type Entity = u64;
