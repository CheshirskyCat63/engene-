use crate::core::plugin::{EngineBuilder, Plugin};

pub struct AiConfigPlugin;

impl Plugin for AiConfigPlugin {
    fn name(&self) -> &str {
        "AiConfigPlugin"
    }

    fn build(&self, _builder: &mut EngineBuilder) {
        println!("[ai_config] AiConfigPlugin registered (desire weights will be data-driven)");
    }
}
