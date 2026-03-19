pub mod api {
    /// Phase A facade placeholder for engine_content.
    pub const CRATE: &str = "engine_content";
}

pub mod asset_budget;
pub mod cooking;
pub mod import;
pub mod schema_governance;

pub mod pipeline;
pub mod prefabs;
pub mod validation;

pub mod content_hash;

// moved from root
pub mod content;
pub mod model_loader;
