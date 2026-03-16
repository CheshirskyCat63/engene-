//! Content subsystem - asset import, cooking, and validation.
//!
//! # Status: partial
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `import`, `cooking` - partial, content pipeline
//! - `prefabs`, `validation` - partial, content management

pub mod asset_budget;
pub mod import;
pub mod cooking;
pub mod prefabs;
pub mod validation;
pub mod schema_governance;
pub mod pipeline;
