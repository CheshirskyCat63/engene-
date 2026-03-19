//! Content subsystem - asset import, cooking, and validation.
//!
//! # Status: partial
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `import`, `cooking` - partial, content pipeline
//! - `prefabs`, `validation` - partial, content management

pub use engine_content::asset_budget;
pub use engine_content::cooking;
pub use engine_content::import;
pub use engine_content::pipeline;
pub use engine_content::prefabs;
pub use engine_content::schema_governance;
pub use engine_content::validation;
