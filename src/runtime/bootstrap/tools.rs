pub use crate::runtime::bootstrap::ToolsRuntimeAssembly;

use engine_core::plugin::EngineBuilder;
use crate::runtime::bootstrap::common::{finalize_builder, insert_runtime_core};

impl ToolsRuntimeAssembly {
    /// Minimal runtime for tools/test harnesses.
    pub fn minimal() -> engine_runtime::engine::Engine {
        let mut builder = EngineBuilder::new();
        insert_runtime_core(
            &mut builder,
            engine_core::runtime_config::RuntimeConfig::tools(),
        );

        // Intentionally tools-only:
        // - no world bootstrap
        // - no game config/material bootstrap
        // - no gameplay/physics stack systems
        finalize_builder(builder)
    }
}
