pub mod api {
    /// Phase A facade placeholder for engine_render.
    pub const CRATE: &str = "engine_render";
}

pub mod mesh;
pub mod renderer;
pub mod destruction_occlusion;
pub mod gore_mesh;
pub mod model_loader;

// Public facade exports
pub use mesh::EntityInstance;
pub use renderer::Renderer;
pub use destruction_occlusion::{DestructionOcclusionSystem, OcclusionBreach};
pub use gore_mesh::{GoreMeshInstance, GoreMeshSystem};
pub use model_loader::LoadedModel;
