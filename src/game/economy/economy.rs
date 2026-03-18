use crate::core::ecs::Entity;
use crate::core::mutation_policy::*;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::{DeterminismTier, SystemDescriptor};
use crate::simulation::time_events::NewMonth;
use crate::world::components::*;

pub struct EconomySystem;

impl EngineSystem for EconomySystem {
    fn name(&self) -> &str {
        "Economy"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("Economy")
            .with_determinism(DeterminismTier::Hard)
            .after("AI")
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        if !ctx.events.has::<NewMonth>() {
            return;
        }

        let npcs = ctx.ecs.npcs();
        for entity in npcs {
            process_monthly_payment(ctx.ecs, ctx.events, entity);
        }
    }
}

fn process_monthly_payment(
    ecs: &mut crate::core::ecs::Ecs,
    events: &mut crate::core::events::EventBus,
    entity: Entity,
) {
    let (paid, remaining_desperation) = {
        let econ = match ecs.get_npc_economy(entity) {
            Some(e) => e,
            None => return,
        };

        let name = ecs.get_name(entity).map(|n| n.0.as_str()).unwrap_or("?");

        if econ.money >= econ.monthly_required {
            println!(
                "  [econ] {} paid monthly debt ({:.0}/{:.0})",
                name, econ.money, econ.monthly_required
            );
            (true, 0.0_f32)
        } else {
            let deficit = 1.0 - econ.money / econ.monthly_required;
            let new_desp = (econ.desperation + deficit * 0.3).min(1.0);
            println!(
                "  [econ] {} FAILED to pay ({:.0}/{:.0}) — desperation {:.2}",
                name, econ.money, econ.monthly_required, new_desp
            );
            (false, new_desp)
        }
    };

    if let Some(econ) = ecs.get_npc_economy_mut(entity) {
        if paid {
            econ.money -= econ.monthly_required;
            econ.desperation = (econ.desperation - 0.1).max(0.0);
        } else {
            econ.desperation = remaining_desperation;
            econ.money = 0.0;
        }
    }

    escalate_desperation(ecs, events, entity);
}

fn escalate_desperation(
    ecs: &mut crate::core::ecs::Ecs,
    events: &mut crate::core::events::EventBus,
    entity: Entity,
) {
    let (desperation, current_job) = {
        let econ = match ecs.get_npc_economy(entity) {
            Some(e) => e,
            None => return,
        };
        (econ.desperation, econ.job)
    };

    if desperation > 0.7 && current_job != Job::Bandit {
        let honesty = ecs.get_npc_traits(entity).map_or(0.5, |t| t.honesty);

        if honesty < 0.5 || desperation > 0.9 {
            let name = ecs
                .get_name(entity)
                .map(|n| n.0.clone())
                .unwrap_or_default();
            println!(
                "  [econ] {} turned BANDIT (desperation {:.2})",
                name, desperation
            );

            if let Some(econ) = ecs.get_npc_economy_mut(entity) {
                econ.job = Job::Bandit;
            }
            events.emit(NpcWentBankrupt(entity));
        }
    }
}
