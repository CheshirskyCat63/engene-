use crate::core::debug::debug_registry::DebugRegistry;
use crate::core::mutation_policy::*;
use engine_core::registry::Resources;
use crate::core::system_descriptor::SystemDescriptor;

pub trait EngineSystem {
    fn name(&self) -> &str;

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("unnamed")
    }

    fn startup(&mut self, _ctx: &mut StartupContext) {}
    fn register_resources(&mut self, _res: &mut Resources) {}
    fn register_debug_views(&mut self, _debug: &mut DebugRegistry) {}
    fn pre_tick(&mut self, _ctx: &mut PreTickContext) {}
    fn fixed_tick(&mut self, ctx: &mut FixedTickContext);
    fn post_tick(&mut self, _ctx: &mut PostTickContext) {}
    fn render_extract(&self, _ctx: &ExtractContext) {}
    fn render_prepare(&self) {}
    fn shutdown(&mut self, _ctx: &mut ShutdownContext) {}
}
