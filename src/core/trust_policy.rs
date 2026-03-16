//! Trust Boundaries & Graceful Degradation - Production 1.0 Requirement
//!
//! Defines what happens when input data is invalid, corrupted, or missing.
//! For a production engine this is not a luxury — it's a requirement.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit + integration (trust_boundaries.rs)
//!
//! ## Failure Scenarios & Behaviors
//!
//! | Situation                      | Behavior |
//! |--------------------------------|----------|
//! | .ron config parse error        | Log error, fall back to hardcoded defaults, Doctor warning. No crash. |
//! | Shader file missing/broken     | Fall back to include_str!() embedded shader. Log error. |
//! | Cooked ass