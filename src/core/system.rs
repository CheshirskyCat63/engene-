// DEPRECATED - Use engine_runtime::simulation_core::systems::engine_system::EngineSystem
// This trait is replaced by the canonical EngineSystem in engine_runtime
// Left temporarily for compatibility during migration

use engine_ecs::system_descriptor::SystemDescriptor;
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem as CanonicalEngineSystem, SystemTickContext, FixedTickContext, RenderTickContext};

pub trait EngineSystem: CanonicalEngineSystem {
    // Legacy methods - migrate to new lifecycle or remove
    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("UnnamedSystem")
    }
    fn name(&self) -> &str {
        "UnnamedSystem"
    }
    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {}
    fn startup(&mut self, _ctx: &mut LegacyStartupContext) {}
    fn register_resources(&mut self, _res: &mut engine_core::registry::Resources) {}
    fn register_debug_views(&mut self, _debug: &mut crate::core::debug::debug_registry::DebugRegistry) {}
    fn pre_tick(&mut self, _ctx: &mut LegacyPreTickContext) {}
    fn post_tick(&mut self, _ctx: &mut LegacyPostTickContext) {}
    fn render_extract(&self, _ctx: &mut LegacyExtractContext) {}
    fn render_prepare(&self) {}
    fn shutdown(&mut self, _ctx: &mut LegacyShutdownContext) {}
}

// Legacy context types - will be removed
pub struct LegacyStartupContext;
pub struct LegacyPreTickContext; 
pub struct LegacyPostTickContext;
pub struct LegacyExtractContext;
pub struct LegacyShutdownContext;
