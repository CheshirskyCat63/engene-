use engine_core::budget_registry::create_default_registry;
use engine_core::events::aggregation::EventAggregator;
use engine_core::events::debug_bus::DebugBus;
use engine_core::events::render_bus::RenderBus;
use engine_core::events::sim_bus::SimBus;
use engine_core::events::sticky::StickyEvents;
use engine_core::events::tracing_hooks::EventTracer;
use engine_core::plugin::EngineBuilder;
use engine_core::quality_governor::QualityGovernor;
use engine_core::runtime_config::RuntimeConfig;
use engine_core::sdk::EngineSDK;
use engine_ecs::Ecs;
use engine_runtime::engine::Engine;

pub(crate) const GAME_CONFIG_DIR: &str = "game/data";

pub(crate) fn insert_explicit_game_config(builder: &mut EngineBuilder, dir: &str) {
    builder.insert_resource(engine_core::game_config::GameConfig::load_from_dir(dir));
}

pub(crate) fn insert_runtime_core(builder: &mut EngineBuilder, runtime_config: RuntimeConfig) {
    builder.insert_resource(runtime_config);
    builder.insert_resource(QualityGovernor::new(60));
    builder.insert_resource(create_default_registry());
    builder.insert_resource(SimBus::new());
    builder.insert_resource(RenderBus::new());
    builder.insert_resource(DebugBus::new());
    builder.insert_resource(StickyEvents::new());
    builder.insert_resource(EventAggregator::new(10.0));
    let mut tracer = EventTracer::new();
    tracer.enable();
    builder.insert_resource(tracer);
    builder.insert_resource(EngineSDK::new());
}

pub(crate) fn finalize_builder(builder: EngineBuilder) -> Engine {
    let mut ecs = Ecs::new();
    let (systems, resources) = builder.build(&mut ecs);
    let mut engine = Engine::from_builder(systems, resources, ecs);
    engine.init();
    engine
}
