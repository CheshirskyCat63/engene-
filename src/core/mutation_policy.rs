use engine_core::commands::CommandBuffer;
use engine_ecs::Ecs;
use engine_core::events::EventBus;
use engine_core::registry::Resources;
use engine_core::time::GameTime;

pub struct StartupContext<'a> {
    pub ecs: &'a mut Ecs,
    pub events: &'a mut EventBus,
    pub resources: &'a mut Resources,
}

pub struct PreTickContext<'a> {
    pub ecs: &'a Ecs,
    pub events: &'a EventBus,
    pub time: &'a GameTime,
    pub resources: &'a Resources,
    pub commands: &'a mut CommandBuffer,
}

pub struct FixedTickContext<'a> {
    pub ecs: &'a mut Ecs,
    pub events: &'a mut EventBus,
    pub time: &'a GameTime,
    pub resources: &'a mut Resources,
    pub commands: &'a mut CommandBuffer,
}

pub struct PostTickContext<'a> {
    pub ecs: &'a Ecs,
    pub events: &'a EventBus,
    pub time: &'a GameTime,
    pub resources: &'a Resources,
    pub commands: &'a mut CommandBuffer,
}

pub struct ExtractContext<'a> {
    pub ecs: &'a Ecs,
    pub resources: &'a Resources,
}

pub struct ShutdownContext<'a> {
    pub ecs: &'a mut Ecs,
    pub resources: &'a mut Resources,
}
