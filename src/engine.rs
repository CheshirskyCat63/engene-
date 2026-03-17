//! Engine ownership surface.
//! This module provides a stable namespace for reusable runtime infrastructure.

pub mod core {
    pub use crate::core::*;
}

pub mod world {
    pub use crate::world::*;
}

pub mod memory {
    pub use crate::memory::*;
}

pub mod physics {
    pub use crate::physics::*;
}

pub mod audio {
    pub use crate::audio::*;
}

pub mod graphics {
    pub use crate::graphics::*;
}

pub mod navigation {
    pub use crate::navigation::*;
}

pub mod content {
    pub use crate::content::*;
}

pub mod animation {
    pub use crate::animation::*;
}

pub mod simulation {
    pub use crate::simulation::*;
}

pub mod input {
    pub use crate::input::*;
}

pub mod body {
    pub use crate::body::*;
}

pub mod network {
    pub use crate::network::*;
}
