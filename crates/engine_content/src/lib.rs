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

// TEMPORARY: model_loader intentionally disabled
// - WHY: depends on crate::animation and crate::graphics paths that reference root layout
// - TYPES NEEDED: Skeleton, AnimationClip, Joint, Keyframe, Channel, SkinVertex
// - THESE TYPES SHOULD LIVE IN: dedicated engine_animation and engine_graphics crates
// - RETURN CONDITION: re-enable when engine_animation + engine_graphics crates exist with these types
// - DEBT TRACKED IN: docs/canonical/MIGRATION_LEDGER.md (Temporary Disabled Features section)
// pub mod model_loader;
