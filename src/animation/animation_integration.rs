//! Phase 4: Animation wiring - connects ECS state to LocomotionMachine per entity.

use std::collections::{HashMap, HashSet};

use crate::ai::body::BodyState;
use crate::animation::locomotion::LocomotionMachine;
use crate::core::ecs::Entity;
use crate::core::events::canonical::BodyZoneDamaged;
use crate::core::mutation_policy::FixedTickContext;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;
use crate::world::components::{AiState, Goal, LifeStage};

const SIM_DT: f32 = 1.0 / 20.0;

fn goal_to_speed(goal: Goal) -> f32 {
    match goal {
        Goal::Flee => 4.0,
        Goal::Hunt | Goal::DefendTerritory => 3.0,
        Goal::SeekFood | Goal::SeekWater | Goal::Explore | Goal::Migrate
        | Goal::FollowPack | Goal::StealOrRob | Goal::SeekShelter => 1.5,
        Goal::Socialize | Goal::Work | Goal::Trade | Goal::Mate
        | Goal::RepairEquipment | Goal::BuySupplies | Goal::StayAtPost => 1.0,
        Goal::DoQuest => 1.5,
        Goal::Rest | Goal::Sleep => 0.3,
    }
}

pub struct AnimationIntegrationSystem {
    locomotion: HashMap<Entity, LocomotionMachine>,
    damaged_entities: HashSet<Entity>,
}

impl AnimationIntegrationSystem {
    pub fn new() -> Self {
        Self {
            locomotion: HashMap::new(),
            damaged_entities: HashSet::new(),
        }
    }
}

impl EngineSystem for AnimationIntegrationSystem {
    fn name(&self) -> &str {
        "AnimationIntegration"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AnimationIntegration")
            .reads_event::<BodyZoneDamaged>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        // Collect BodyZoneDamaged entities for hit reaction
        self.damaged_entities.clear();
        for ev in ctx.events.read::<BodyZoneDamaged>() {
            self.damaged_entities.insert(ev.entity);
        }

        let dt = SIM_DT;

        for &entity in &ctx.ecs.alive {
            if ctx.ecs.get_transform(entity).is_none() {
                continue;
            }
            let ai_state = ctx.ecs.get_ai_state(entity);

            // Determine speed from goal/state
            let speed = match ai_state {
                Some(AiState::Executing(goal)) => {
                    let stage = ctx.ecs.get_life_info(entity)
                        .map_or(LifeStage::Adult, |li| li.life_stage());
                    let body = ctx.ecs.get_needs(entity)
                        .map(|pn| BodyState::compute_with_stage(pn, stage))
                        .unwrap_or(BodyState {
                            move_speed_mult: 1.0,
                            perception_radius_mult: 1.0,
                            combat_power_mult: 1.0,
                            work_efficiency_mult: 1.0,
                        });
                    goal_to_speed(*goal) * body.move_speed_mult
                }
                _ => 0.0,
            };

            let is_dead = ctx.ecs.get_needs(entity)
                .map_or(false, |pn| pn.health <= 0.0);

            let machine = self.locomotion.entry(entity).or_insert_with(LocomotionMachine::new);
            machine.update(speed, is_dead, dt);
        }

        // Remove locomotion state for despawned entities (cleanup on next tick)
        self.locomotion.retain(|e, _| ctx.ecs.is_alive(*e));
    }
}
