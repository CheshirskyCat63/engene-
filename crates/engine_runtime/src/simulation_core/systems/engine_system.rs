//! Engine System Trait - Core trait for runtime systems.
//! Phase 3: Extracted from world_tick.rs for proper separation.
//!
//! This trait defines the contract for all runtime systems in the engine.
//! It is owned by engine_runtime and used by both engine_runtime and root transitional code.

/// Core trait for all engine systems.
///
/// Systems are the fundamental unit of runtime logic in the engine.
/// Each system operates on ECS data during tick cycles.
pub trait EngineSystem {
    /// Returns the unique name of this system.
    fn name(&self) -> &str;

    /// Called on each tick (default: no-op).
    fn tick(&mut self, _ctx: &mut SystemTickContext) {}

    /// Called on fixed tick intervals (default: no-op).
    fn fixed_tick(&mut self, _ctx: &mut FixedTickContext) {}

    /// Called on render tick (default: no-op).
    fn render_tick(&mut self, _ctx: &mut RenderTickContext) {}
}

/// Context for regular system ticks.
pub struct SystemTickContext {
    pub delta: f32,
    pub ecs: (),
}

impl SystemTickContext {
    pub fn new(delta: f32) -> Self {
        Self { delta, ecs: () }
    }
}

/// Context for fixed-interval system ticks.
///
/// Uses generic time type to avoid coupling to engine_core.
/// Root transitional code will provide concrete time implementation.
pub struct FixedTickContext<T = ()> {
    pub delta: f32,
    pub time: T,
    pub ecs: (),
}

impl<T> FixedTickContext<T> {
    pub fn new(delta: f32, time: T) -> Self {
        Self {
            delta,
            time,
            ecs: (),
        }
    }
}

/// Context for render ticks.
pub struct RenderTickContext {
    pub delta: f32,
    pub ecs: (),
}

impl RenderTickContext {
    pub fn new(delta: f32) -> Self {
        Self { delta, ecs: () }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_system_trait_object_safe() {
        // Verify trait can be used with dyn
        fn check_name(system: &dyn EngineSystem) -> String {
            system.name().to_string()
        }

        struct DummySystem;
        impl EngineSystem for DummySystem {
            fn name(&self) -> &str {
                "Dummy"
            }
        }

        assert_eq!(check_name(&DummySystem), "Dummy");
    }

    #[test]
    fn test_context_creation() {
        let tick_ctx = SystemTickContext::new(0.016);
        assert_eq!(tick_ctx.delta, 0.016);

        // Using () as placeholder time type for tests
        let fixed_ctx = FixedTickContext::new(0.016, ());
        assert_eq!(fixed_ctx.delta, 0.016);

        let render_ctx = RenderTickContext::new(0.016);
        assert_eq!(render_ctx.delta, 0.016);
    }
}
