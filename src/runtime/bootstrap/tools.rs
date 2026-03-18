use crate::core::plugin::EngineBuilder;
use crate::runtime::bootstrap::common::{finalize_builder, insert_runtime_core};
use crate::runtime::bootstrap::ToolsRuntimeAssembly;

impl ToolsRuntimeAssembly {
    /// Minimal runtime for tools/test harnesses.
    pub fn minimal() -> crate::core::engine::Engine {
        let mut builder = EngineBuilder::new();
        insert_runtime_core(
            &mut builder,
            crate::core::runtime_config::RuntimeConfig::tools(),
        );

        // Intentionally tools-only:
        // - no world bootstrap
        // - no game config/material bootstrap
        // - no gameplay/physics stack systems
        finalize_builder(builder)
    }
}
