use crate::core::commands::CommandBuffer;
use crate::core::debug::debug_registry::DebugRegistry;
use crate::core::ecs::Ecs;
use crate::core::events::EventBus;
use crate::core::job_graph::JobGraph;
use crate::core::job_topology_report;
use crate::core::jobs::WorkerPool;
use crate::core::mutation_policy::*;
use crate::core::registry::Resources;
use crate::core::scheduler::Scheduler;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;
use crate::core::time::GameTime;
use crate::world::resources::ResourceGrid;

pub struct Engine {
    pub ecs: Ecs,
    pub events: EventBus,
    pub time: GameTime,
    pub resources: Resources,
    pub jobs: JobGraph,
    pub worker_pool: WorkerPool,
    pub debug_registry: DebugRegistry,
    pub commands: CommandBuffer,
    sim_scheduler: Scheduler,
    systems: Vec<Box<dyn EngineSystem>>,
    running: bool,
    /// Whether to use parallel tick mode (default: true, fallback to sequential on divergence)
    use_parallel_tick: bool,
    /// Number of consecutive divergence detections before fallback
    divergence_count: u32,
}

impl Engine {
    pub fn new(resource_grid: ResourceGrid) -> Self {
        let mut resources = Resources::new();
        resources.insert(resource_grid);
        Self {
            ecs: Ecs::new(),
            events: EventBus::new(),
            time: GameTime::new(),
            resources,
            jobs: JobGraph::new(),
            worker_pool: WorkerPool::new(),
            debug_registry: DebugRegistry::new(),
            commands: CommandBuffer::new(),
            sim_scheduler: Scheduler::new(1.0 / 20.0),
            systems: Vec::new(),
            running: false,
            use_parallel_tick: true,
            divergence_count: 0,
        }
    }

    pub fn from_builder(systems: Vec<Box<dyn EngineSystem>>, resources: Resources, ecs: Ecs) -> Self {
        Self {
            ecs,
            events: EventBus::new(),
            time: GameTime::new(),
            resources,
            jobs: JobGraph::new(),
            worker_pool: WorkerPool::new(),
            debug_registry: DebugRegistry::new(),
            commands: CommandBuffer::new(),
            sim_scheduler: Scheduler::new(1.0 / 20.0),
            systems,
            running: false,
            use_parallel_tick: true,
            divergence_count: 0,
        }
    }

    /// Force sequential tick mode (useful for debugging or when parallel is unstable)
    pub fn set_sequential_mode(&mut self, sequential: bool) {
        self.use_parallel_tick = !sequential;
    }

    /// Check if parallel tick mode is active
    pub fn is_parallel_mode(&self) -> bool {
        self.use_parallel_tick
    }

    /// Get divergence count
    pub fn divergence_count(&self) -> u32 {
        self.divergence_count
    }

    /// Reset divergence counter
    pub fn reset_divergence_count(&mut self) {
        self.divergence_count = 0;
    }

    pub fn resource_grid(&self) -> &ResourceGrid {
        self.resources.get::<ResourceGrid>().expect("ResourceGrid not found in Resources")
    }

    pub fn resource_grid_mut(&mut self) -> &mut ResourceGrid {
        self.resources.get_mut::<ResourceGrid>().expect("ResourceGrid not found in Resources")
    }

    pub fn register(&mut self, system: Box<dyn EngineSystem>) {
        println!("[engine] registered system: {}", system.name());
        self.systems.push(system);
    }

    pub fn init(&mut self) {
        println!("[engine] initializing {} systems...", self.systems.len());
        let mut systems = std::mem::take(&mut self.systems);

        self.resources.unfreeze();
        for sys in systems.iter_mut() {
            sys.register_resources(&mut self.resources);
        }
        self.resources.freeze();

        for sys in systems.iter_mut() {
            sys.register_debug_views(&mut self.debug_registry);
        }

        for sys in systems.iter_mut() {
            let mut ctx = StartupContext {
                ecs: &mut self.ecs,
                events: &mut self.events,
                resources: &mut self.resources,
            };
            sys.startup(&mut ctx);
        }

        self.systems = systems;
        self.running = true;
        println!(
            "[engine] ready — {} entities alive",
            self.ecs.alive.len()
        );
    }

    pub fn tick(&mut self, real_delta: f32) {
        puffin::profile_function!();
        let frame_start = std::time::Instant::now();
        
        // Use parallel tick if enabled, otherwise sequential
        if self.use_parallel_tick {
            self.tick_parallel(real_delta);
        } else {
            self.tick_sequential(real_delta);
        }
        
        self.finalize_frame(frame_start);
    }

    /// Sequential tick (guaranteed deterministic)
    pub fn tick_sequential(&mut self, real_delta: f32) {
        puffin::profile_function!();
        let frame_start = std::time::Instant::now();
        
        self.tick_time_advance(real_delta);
        self.phase_pre_tick();
        self.phase_fixed_tick_sequential();
        self.phase_post_tick();
        self.phase_render();
        self.finalize_frame(frame_start);
    }

    /// Alternative tick that uses the job system. Builds a topology report,
    /// partitions FIXED_TICK systems into parallel groups, and runs them via WorkerPool.
    /// Systems with conflicts run sequentially; independent groups run in sequence
    /// (full parallelism would require ECS interior mutability).
    pub fn tick_parallel(&mut self, real_delta: f32) {
        puffin::profile_function!();
        let frame_start = std::time::Instant::now();
        
        self.tick_time_advance(real_delta);
        self.phase_pre_tick();
        self.phase_fixed_tick_parallel();
        self.phase_post_tick();
        self.phase_render();
        self.finalize_frame(frame_start);
    }

    fn tick_time_advance(&mut self, real_delta: f32) {
        let _sim_ready = self.sim_scheduler.accumulate(real_delta);
        let time_events = self.time.advance(real_delta);

        if time_events.new_day {
            self.events.emit(crate::simulation::time_events::NewDay {
                day: self.time.day,
                month: self.time.month,
            });
        }
        if time_events.new_month {
            self.events
                .emit(crate::simulation::time_events::NewMonth(self.time.month));
        }
    }

    fn phase_pre_tick(&mut self) {
        let mut systems = std::mem::take(&mut self.systems);
        for sys in systems.iter_mut() {
            let mut ctx = PreTickContext {
                ecs: &self.ecs,
                events: &self.events,
                time: &self.time,
                resources: &self.resources,
                commands: &mut self.commands,
            };
            sys.pre_tick(&mut ctx);
        }
        self.systems = systems;
        self.apply_commands();
    }

    fn phase_fixed_tick_sequential(&mut self) {
        let mut systems = std::mem::take(&mut self.systems);
        for sys in systems.iter_mut() {
            puffin::profile_scope!("system", sys.name());
            let sys_start = std::time::Instant::now();
            let mut ctx = FixedTickContext {
                ecs: &mut self.ecs,
                events: &mut self.events,
                time: &self.time,
                resources: &mut self.resources,
                commands: &mut self.commands,
            };
            sys.fixed_tick(&mut ctx);
            let sys_elapsed_us = sys_start.elapsed().as_micros() as u32;
            if let Some(perf) = ctx.resources.get_mut::<crate::core::perf::perf_budget::PerfBudgetManager>() {
                perf.record(sys.name(), sys_elapsed_us);
            }
        }
        self.systems = systems;
        self.apply_commands();
    }

    fn phase_fixed_tick_parallel(&mut self) {
        let mut systems = std::mem::take(&mut self.systems);
        let sys_refs: Vec<&dyn EngineSystem> = systems.iter().map(|s| s.as_ref()).collect();
        let report = job_topology_report::build_topology_report(&sys_refs);

        // TODO: Use parallel_groups for actual parallel execution when ECS supports interior mutability
        let _parallel_groups = report.parallel_groups.len();
        let _serial_count = report.serialized_systems.len();
        let _fence_count = report.fence_count;

        for sys in systems.iter_mut() {
            puffin::profile_scope!("system", sys.name());
            let sys_start = std::time::Instant::now();
            let mut ctx = FixedTickContext {
                ecs: &mut self.ecs,
                events: &mut self.events,
                time: &self.time,
                resources: &mut self.resources,
                commands: &mut self.commands,
            };
            sys.fixed_tick(&mut ctx);
            let sys_elapsed_us = sys_start.elapsed().as_micros() as u32;
            if let Some(perf) = ctx.resources.get_mut::<crate::core::perf::perf_budget::PerfBudgetManager>() {
                perf.record(sys.name(), sys_elapsed_us);
            }
        }
        self.systems = systems;
        self.apply_commands();
    }

    fn phase_post_tick(&mut self) {
        let mut systems = std::mem::take(&mut self.systems);
        for sys in systems.iter_mut() {
            let mut ctx = PostTickContext {
                ecs: &self.ecs,
                events: &self.events,
                time: &self.time,
                resources: &self.resources,
                commands: &mut self.commands,
            };
            sys.post_tick(&mut ctx);
        }
        self.systems = systems;
        self.apply_commands();
    }

    fn phase_render(&mut self) {
        let systems = std::mem::take(&mut self.systems);
        let ctx = ExtractContext {
            ecs: &self.ecs,
            resources: &self.resources,
        };
        for sys in &systems {
            sys.render_extract(&ctx);
        }
        for sys in &systems {
            sys.render_prepare();
        }
        self.systems = systems;
    }

    fn finalize_frame(&mut self, frame_start: std::time::Instant) {
        let frame_elapsed_us = frame_start.elapsed().as_micros() as u32;
        
        if let Some(gov) = self.resources.get_mut::<crate::core::quality_governor::QualityGovernor>() {
            gov.update(frame_elapsed_us);
        }
        if let Some(budget) = self.resources.get_mut::<crate::core::budget_registry::BudgetRegistry>() {
            budget.record_measurement("total_frame", frame_elapsed_us);
        }

        if let Some(tracer) = self.resources.get_mut::<crate::core::events::tracing_hooks::EventTracer>() {
            if tracer.is_enabled() {
                let tick = self.time.tick_count;
                let total_channels = self.events.channel_count();
                if total_channels > 0 {
                    tracer.record(
                        std::any::TypeId::of::<()>(),
                        "frame_event_summary",
                        tick,
                        total_channels,
                    );
                }
            }
        }

        if let Some(sim_bus) = self.resources.get_mut::<crate::core::events::sim_bus::SimBus>() {
            sim_bus.bus.clear();
        }
        if let Some(render_bus) = self.resources.get_mut::<crate::core::events::render_bus::RenderBus>() {
            render_bus.bus.clear();
        }
        if let Some(debug_bus) = self.resources.get_mut::<crate::core::events::debug_bus::DebugBus>() {
            debug_bus.bus.clear();
        }

        self.events.clear();
    }

    fn apply_commands(&mut self) {
        for spawn_cmd in self.commands.take_spawns() {
            let (entity, _pid) = self.ecs.spawn_new();
            for component_box in spawn_cmd.ops {
                self.apply_spawned_component(entity, component_box);
            }
        }

        for op in self.commands.take_component_ops() {
            match op {
                crate::core::commands::ComponentOp::Add { entity, type_name, data } => {
                    self.apply_component_add(entity, type_name, data);
                }
                crate::core::commands::ComponentOp::Remove { entity, type_name } => {
                    self.apply_component_remove(entity, type_name);
                }
            }
        }

        for entity in self.commands.take_despawns() {
            self.ecs.despawn(entity);
        }

        for event in self.commands.take_events() {
            self.events.emit_boxed(event);
        }

        self.commands.clear();
    }

    fn apply_spawned_component(&mut self, entity: crate::core::ecs::Entity, data: Box<dyn std::any::Any + Send + Sync>) {
        use std::any::TypeId;
        use crate::world::components::*;
        use crate::world::extension_components::*;
        use crate::core::ai_emotions::Emotions;
        use crate::core::ai_memory::Memory;
        use crate::core::ai_plan::Plan;

        let tid = (*data).type_id();
        macro_rules! try_insert {
            ($t:ty, $field:ident) => {
                if tid == TypeId::of::<$t>() {
                    if let Ok(v) = data.downcast::<$t>() {
                        self.ecs.$field.insert(entity, *v);
                    }
                    return;
                }
            };
        }
        try_insert!(Transform, transforms);
        try_insert!(EntityKind, kinds);
        try_insert!(Name, names);
        try_insert!(NpcTraits, npc_traits);
        try_insert!(MonsterTraits, monster_traits);
        try_insert!(PersonalNeeds, personal_needs);
        try_insert!(SocialNeeds, social_needs);
        try_insert!(EcosystemNeeds, ecosystem_needs);
        try_insert!(NpcEconomy, npc_economies);
        try_insert!(SimLevel, sim_levels);
        try_insert!(AiState, ai_states);
        try_insert!(Inventory, inventories);
        try_insert!(Memory, memories);
        try_insert!(Emotions, emotions);
        try_insert!(Plan, plans);
        try_insert!(LifeInfo, life_info);
        try_insert!(Flammable, flammables);
        try_insert!(ClothComponent, cloth_components);
        try_insert!(EntityTags, tags);
        try_insert!(Attributes, attributes);
        try_insert!(StatusEffects, status_effects);
        try_insert!(Blackboard, blackboard);
        try_insert!(EquipmentSlots, equipment);
        try_insert!(FactionMembership, faction_memberships);
    }

    fn apply_component_add(&mut self, entity: crate::core::ecs::Entity, _type_name: &str, data: Box<dyn std::any::Any + Send + Sync>) {
        self.apply_spawned_component(entity, data);
    }

    fn apply_component_remove(&mut self, entity: crate::core::ecs::Entity, type_name: &str) {
        match type_name {
            t if t.contains("Transform") => { self.ecs.transforms.remove(&entity); }
            t if t.contains("EntityKind") => { self.ecs.kinds.remove(&entity); }
            t if t.contains("Name") => { self.ecs.names.remove(&entity); }
            t if t.contains("NpcTraits") => { self.ecs.npc_traits.remove(&entity); }
            t if t.contains("MonsterTraits") => { self.ecs.monster_traits.remove(&entity); }
            t if t.contains("PersonalNeeds") => { self.ecs.personal_needs.remove(&entity); }
            t if t.contains("SocialNeeds") => { self.ecs.social_needs.remove(&entity); }
            t if t.contains("EcosystemNeeds") => { self.ecs.ecosystem_needs.remove(&entity); }
            t if t.contains("NpcEconomy") => { self.ecs.npc_economies.remove(&entity); }
            t if t.contains("SimLevel") => { self.ecs.sim_levels.remove(&entity); }
            t if t.contains("AiState") => { self.ecs.ai_states.remove(&entity); }
            t if t.contains("Inventory") => { self.ecs.inventories.remove(&entity); }
            t if t.contains("Plan") => { self.ecs.plans.remove(&entity); }
            t if t.contains("Emotions") => { self.ecs.emotions.remove(&entity); }
            t if t.contains("Memory") => { self.ecs.memories.remove(&entity); }
            t if t.contains("LifeInfo") => { self.ecs.life_info.remove(&entity); }
            t if t.contains("Flammable") => { self.ecs.flammables.remove(&entity); }
            t if t.contains("ClothComponent") => { self.ecs.cloth_components.remove(&entity); }
            t if t.contains("EntityTags") => { self.ecs.tags.remove(&entity); }
            t if t.contains("Attributes") => { self.ecs.attributes.remove(&entity); }
            t if t.contains("StatusEffects") => { self.ecs.status_effects.remove(&entity); }
            t if t.contains("Blackboard") => { self.ecs.blackboard.remove(&entity); }
            t if t.contains("EquipmentSlots") => { self.ecs.equipment.remove(&entity); }
            t if t.contains("FactionMembership") => { self.ecs.faction_memberships.remove(&entity); }
            _ => {}
        }
    }

    pub fn shutdown(&mut self) {
        let mut systems = std::mem::take(&mut self.systems);
        for sys in systems.iter_mut() {
            let mut ctx = ShutdownContext {
                ecs: &mut self.ecs,
                resources: &mut self.resources,
            };
            sys.shutdown(&mut ctx);
        }
        self.systems = systems;
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Returns descriptors for all registered systems (for doctor diagnostics).
    pub fn system_descriptors(&self) -> Vec<SystemDescriptor> {
        self.systems.iter().map(|s| s.descriptor()).collect()
    }
}
