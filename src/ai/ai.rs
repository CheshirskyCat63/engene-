use crate::ai::groups;
use crate::ai::monster_ai;
use crate::ai::npc_ai;
use crate::ai::social;
use crate::core::mutation_policy::*;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::{DeterminismTier, SystemDescriptor};
use crate::ecosystem::territory;
use crate::simulation::simulation_level;
use crate::world::components::*;
use crate::world::population;
use crate::world::resources::ResourceGrid;

pub struct AiSystem {
    respawn_timer: f32,
    territory_cooldown: u64,
}

impl AiSystem {
    pub fn new() -> Self {
        Self { respawn_timer: 0.0, territory_cooldown: 0 }
    }
}

impl EngineSystem for AiSystem {
    fn name(&self) -> &str {
        "AI"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AI")
            .with_determinism(DeterminismTier::Hard)
            .after("WorldTick")
            .before("Economy")
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let resources = ctx.resources.get_mut::<ResourceGrid>().expect("ResourceGrid missing");
        let resources_ptr = resources as *mut ResourceGrid;

        ctx.ecs.tick += 1;
        let frame = ctx.ecs.tick;

        ctx.ecs.rebuild_spatial();
        if frame >= self.territory_cooldown {
            self.territory_cooldown = frame + 60;
            ctx.ecs.territory = territory::compute_territory(ctx.ecs);
        }

        let delta = ctx.time.delta;
        let day_progress = ctx.time.day_progress();
        let season = ctx.time.season();
        let entity_count = ctx.ecs.alive.len();

        for idx in 0..entity_count {
            let entity = match ctx.ecs.alive.get(idx) {
                Some(&e) => e,
                None => break,
            };
            if !ctx.ecs.is_alive(entity) { continue; }

            let sim_level = ctx.ecs
                .sim_levels
                .get(&entity)
                .map(|s| s.level)
                .unwrap_or(SimulationLevel::L0);

            if !simulation_level::should_tick(sim_level, frame) {
                continue;
            }

            let tick_delta = match sim_level {
                SimulationLevel::L0 => delta,
                SimulationLevel::L1 => delta * simulation_level::L1_TICK_INTERVAL as f32,
                SimulationLevel::L2 => delta * simulation_level::L2_TICK_INTERVAL as f32,
                SimulationLevel::L3 => continue,
            };

            // SAFETY: ResourceGrid is independent of ECS iteration
            let resources = unsafe { &mut *resources_ptr };
            match ctx.ecs.kinds.get(&entity) {
                Some(EntityKind::Npc) => {
                    npc_ai::tick_npc(ctx.ecs, ctx.events, resources, entity, tick_delta, day_progress);
                }
                Some(EntityKind::Monster(_)) => {
                    monster_ai::tick_monster(ctx.ecs, ctx.events, resources, entity, tick_delta, day_progress);
                }
                None => {}
            }

            if let Some(pn) = ctx.ecs.personal_needs.get_mut(&entity) {
                let extra_drain = (season.hunger_drain_mult() - 1.0) * tick_delta * 0.003;
                pn.hunger = (pn.hunger + extra_drain).min(1.0);
            }
        }

        // Wire social interactions: group formation and trade for L0 NPCs
        if frame % 100 == 0 {
            let npc_entities: Vec<_> = ctx.ecs.alive.iter().copied()
                .filter(|e| matches!(ctx.ecs.kinds.get(e), Some(EntityKind::Npc)))
                .collect();
            for &npc in &npc_entities {
                let _ = groups::find_or_form_group(ctx.ecs, npc);
            }
            for i in 0..npc_entities.len().saturating_sub(1) {
                let a = npc_entities[i];
                let b = npc_entities[i + 1];
                social::try_trade(ctx.ecs, a, b);
                social::share_knowledge(ctx.ecs, a, b);
            }
        }

        let resources = ctx.resources.get_mut::<ResourceGrid>().expect("ResourceGrid missing");
        collect_dead(ctx.ecs, resources);
        age_entities(ctx.ecs, delta);

        self.respawn_timer += delta;
        if self.respawn_timer > 300.0 {
            self.respawn_timer = 0.0;
            respawn_monsters_if_needed(ctx.ecs);
        }
    }
}

fn collect_dead(ecs: &mut crate::core::ecs::Ecs, resources: &mut ResourceGrid) {
    let dead: Vec<_> = ecs.alive.iter().copied()
        .filter(|e| ecs.personal_needs.get(e).map_or(false, |pn| pn.health <= 0.0))
        .collect();

    for e in dead {
        let food_val = ecs.kinds.get(&e).map_or(0.3, food_value);
        let name = ecs.names.get(&e).map(|n| n.0.clone()).unwrap_or_default();
        if let Some(t) = ecs.transforms.get(&e) {
            resources.add_carcass(t.x, t.y, t.cell_x, t.cell_y, food_val, name);
        }
        ecs.despawn(e);
    }
}

fn age_entities(ecs: &mut crate::core::ecs::Ecs, delta: f32) {
    let age_increment = delta / 120.0;
    let mut died_of_age: Vec<u64> = Vec::new();

    for &e in &ecs.alive {
        if let Some(li) = ecs.life_info.get_mut(&e) {
            li.age += age_increment;
            if li.age >= li.max_age {
                died_of_age.push(e);
            }
        }
    }

    for e in died_of_age {
        if let Some(pn) = ecs.personal_needs.get_mut(&e) {
            pn.health = 0.0;
        }
    }
}

const MIN_WOLVES: usize = 5;
const MIN_BOARS: usize = 5;
const MIN_BLOODSUCKERS: usize = 2;

fn respawn_monsters_if_needed(ecs: &mut crate::core::ecs::Ecs) {
    let wolves = ecs.count_species(MonsterSpecies::Wolf);
    let boars = ecs.count_species(MonsterSpecies::Boar);
    let bloods = ecs.count_species(MonsterSpecies::Bloodsucker);

    let mut rng = rand::thread_rng();

    if wolves < MIN_WOLVES {
        let n = MIN_WOLVES - wolves;
        for _ in 0..n { population::spawn_single_monster(ecs, MonsterSpecies::Wolf, &mut rng); }
        println!("    [spawn] +{} wolves migrated into the zone", n);
    }
    if boars < MIN_BOARS {
        let n = MIN_BOARS - boars;
        for _ in 0..n { population::spawn_single_monster(ecs, MonsterSpecies::Boar, &mut rng); }
        println!("    [spawn] +{} boars migrated into the zone", n);
    }
    if bloods < MIN_BLOODSUCKERS {
        let n = MIN_BLOODSUCKERS - bloods;
        for _ in 0..n { population::spawn_single_monster(ecs, MonsterSpecies::Bloodsucker, &mut rng); }
        println!("    [spawn] +{} bloodsuckers appeared from the depths", n);
    }
}
