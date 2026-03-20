//! World components module.
//!
//! This module provides all component types used by the ECS.
//! Components are organized into submodules by domain for maintainability.

mod ai;
mod economy;
mod entity_kind;
mod equipment;
mod events;
mod faction;
mod life;
mod needs;
mod physical;
mod traits_mod;
mod transform;

// Re-export all public types at module level for backward compatibility
pub use ai::*;
pub use economy::*;
pub use entity_kind::*;
pub use equipment::*;
pub use events::*;
pub use faction::*;
pub use life::*;
pub use needs::*;
pub use physical::*;
pub use traits_mod::*;
pub use transform::*;
