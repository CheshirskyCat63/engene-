use crate::core::plugin::{EngineBuilder, Plugin};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn name(&self) -> &str {
        "CombatPlugin"
    }

    fn build(&self, _builder: &mut EngineBuilder) {
        println!("[combat] CombatPlugin registered (rules will be data-driven)");
    }
}
