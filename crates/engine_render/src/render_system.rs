use std::sync::Mutex;

// LEGACY IMPORTS - Use canonical crates instead
use crate::core::system::EngineSystem as LegacyEngineSystem;
use crate::world::components::EntityKind;
use engine_ecs::system_descriptor::SystemDescriptor;
use engine_render::destruction_occlusion::DestructionOcclusionSystem;
use engine_render::mesh::EntityInstance;
use engine_render::renderer::Renderer;
use engine_runtime::simulation_core::systems::engine_system::{
    EngineSystem, ExtractContext, FixedTickContext,
};

pub struct RenderExtractData {
    pub instances: Vec<EntityInstance>,
    pub day_progress: f32,
    pub entity_count: usize,
}

impl RenderExtractData {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            day_progress: 0.0,
            entity_count: 0,
        }
    }
}

pub struct RenderSystem;

impl LegacyEngineSystem for RenderSystem {
    fn name(&self) -> &str {
        "RenderSystem"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("RenderSystem")
            .reads_component::<crate::world::components::Transform>()
            .reads_component::<EntityKind>()
            .with_parallel(true)
            .with_headless(false)
    }

    fn register_resources(&mut self, res: &mut crate::core::registry::Resources) {
        res.insert(Mutex::new(RenderExtractData::new()));
    }

    fn fixed_tick(&mut self, _ctx: &mut FixedTickContext) {}

    fn render_extract(&self, ctx: &ExtractContext) {
        let Some(mtx) = ctx.resources.get::<Mutex<RenderExtractData>>() else {
            return;
        };
        let Ok(mut data) = mtx.lock() else {
            return;
        };

        data.instances.clear();
        data.entity_count = ctx.ecs.alive.len();
        data.day_progress = 0.0;

        for &e in &ctx.ecs.alive {
            let Some(t) = ctx.ecs.get_transform(e) else {
                continue;
            };
            let color = match ctx.ecs.get_kind(e) {
                Some(EntityKind::Npc) => [0.16, 0.47, 1.0],
                Some(EntityKind::Monster(crate::world::components::MonsterSpecies::Wolf)) => {
                    [0.9, 0.9, 0.9]
                }
                Some(EntityKind::Monster(crate::world::components::MonsterSpecies::Boar)) => {
                    [0.55, 0.43, 0.39]
                }
                Some(EntityKind::Monster(
                    crate::world::components::MonsterSpecies::Bloodsucker,
                )) => [0.83, 0.0, 0.0],
                None => [0.5, 0.5, 0.5],
            };
            data.instances.push(EntityInstance {
                position: [t.x, 0.0, t.y],
                color,
            });
        }
    }
}
