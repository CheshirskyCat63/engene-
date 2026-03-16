use crate::core::plugin::{EngineBuilder, Plugin};
use crate::economy::economy::EconomySystem;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn name(&self) -> &str {
        "EconomyPlugin"
    }

    fn build(&self, builder: &mut EngineBuilder) {
        builder.add_system_default(Box::new(EconomySystem));
    }
}
