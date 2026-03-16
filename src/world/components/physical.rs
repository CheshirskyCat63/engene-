//! Physical property components.

use serde::{Deserialize, Serialize};

/// Flammable material types.
#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlammableMaterial {
    Wood,
    Cloth,
    Thatch,
    Stone,
}

impl FlammableMaterial {
    pub fn flammability(self) -> f32 {
        match self {
            Self::Wood => 0.6,
            Self::Cloth => 0.9,
            Self::Thatch => 0.8,
            Self::Stone => 0.0,
        }
    }

    pub fn fuel(self) -> f32 {
        match self {
            Self::Wood => 5.0,
            Self::Cloth => 2.0,
            Self::Thatch => 3.0,
            Self::Stone => 0.0,
        }
    }
}

/// Flammable entity component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flammable {
    pub material: FlammableMaterial,
}

/// Cloth soft body component (XPBD).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClothComponent {
    pub cloth_id: u32,
    pub width: u32,
    pub height: u32,
}
