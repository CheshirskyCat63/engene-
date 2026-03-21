#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Biome {
    Forest,
    Plains,
    Swamp,
    Hills,
    Settlement,
}

impl Biome {
    pub fn food_density(&self) -> f32 {
        match self {
            Self::Forest => 0.7,
            Self::Plains => 0.5,
            Self::Swamp => 0.3,
            Self::Hills => 0.2,
            Self::Settlement => 0.1,
        }
    }

    pub fn danger_level(&self) -> f32 {
        match self {
            Self::Forest => 0.4,
            Self::Plains => 0.2,
            Self::Swamp => 0.7,
            Self::Hills => 0.3,
            Self::Settlement => 0.05,
        }
    }

    pub fn water_density(&self) -> f32 {
        match self {
            Self::Swamp => 0.9,
            Self::Forest => 0.5,
            Self::Plains => 0.3,
            Self::Hills => 0.2,
            Self::Settlement => 0.6,
        }
    }

    pub fn night_danger_mult(&self) -> f32 {
        match self {
            Self::Forest => 1.8,
            Self::Swamp => 2.0,
            Self::Hills => 1.5,
            Self::Plains => 1.3,
            Self::Settlement => 1.1,
        }
    }
}

impl std::fmt::Display for Biome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forest => write!(f, "Forest"),
            Self::Plains => write!(f, "Plains"),
            Self::Swamp => write!(f, "Swamp"),
            Self::Hills => write!(f, "Hills"),
            Self::Settlement => write!(f, "Settlement"),
        }
    }
}
