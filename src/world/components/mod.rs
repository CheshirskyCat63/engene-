//! World components module.
//!
//! This module provides all component types used by the ECS.
//! Components are organized into submodules by domain for maintainability.

mod transform;
mod entity_kind;
mod needs;
mod traits_mod;
mod ai;
mod economy;
mod life;
mod physical;
mod equipment;
mod faction;
mod events;

// Re-export all public types at module level for backward compatibility
pub use transform::*;
pub use entity_kind::*;
pub use needs::*;
pub use traits_mod::*;
pub use ai::*;
pub use economy::*;
pub use life::*;
pub use physical::*;
pub use equipment::*;
pub use faction::*;
pub use events::*;
