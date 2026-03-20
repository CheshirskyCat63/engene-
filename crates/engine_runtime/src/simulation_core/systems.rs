//! Runtime Systems Module.
//! Phase 3: Ownership transfer from root runtime.
//! 
//! Contains runtime orchestration systems that belong to engine_runtime.
//! 
//! Structure:
//! - `engine_system.rs` - Core EngineSystem trait (owned by engine_runtime)
//! - `scheduler.rs` - Time-based scheduling (owned by engine_runtime)
//! - `world_tick.rs` - Concrete WorldTickSystem (owned by engine_runtime)

pub mod engine_system;
pub mod scheduler;
pub mod world_tick;

pub use engine_system::{EngineSystem, FixedTickContext, RenderTickContext, SystemTickContext};
pub use scheduler::Scheduler;
pub use world_tick::WorldTickSystem;
