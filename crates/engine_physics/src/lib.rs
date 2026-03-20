//! Engine Physics - Physics runtime capability for ENGENE.
//!
//! This crate provides the physics capability layer. It is a capability, not a runtime role.
//! Physics can be enabled in any runtime mode (game, SDK, headless, tools).

pub mod api;
pub mod bootstrap;
pub mod contracts;
pub mod resources;
pub mod systems;

// Animation - self-contained
pub mod animation;

// Physics modules - re-export for root compatibility
// TEMPORARY: physics module disabled - depends on types (Ecs, Entity, WorldFields, etc.)
// that exist in root but not in crate structure (engine_ecs doesn't have Ecs, engine_world is placeholder)
// See MIGRATION_LEDGER.md for tracking
// pub mod physics;

// TEMPORARY: physics module disabled - depends on types (Ecs, Entity, WorldFields, etc.)
// that exist in root but not in crate structure (engine_ecs doesn't have Ecs, engine_world is placeholder)
// See MIGRATION_LEDGER.md for tracking
// pub mod physics;
