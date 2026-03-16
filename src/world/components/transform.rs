//! Spatial components for entity positioning.

use serde::{Deserialize, Serialize};

/// Entity position in world coordinates.
/// x, y are world-space coordinates (y is used as z in 3D for terrain sampling).
/// cell_x, cell_y are chunk coordinates for streaming.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub cell_x: u32,
    pub cell_y: u32,
}

/// Entity display name.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Name(pub String);
