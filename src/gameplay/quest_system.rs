// Phase 11: Quest system — generates quests, assigns, tracks progress.

use crate::core::mutation_policy::*;
use crate::core::system::EngineSystem;
use rand::Rng;
use crate::core::system_descriptor::{DeterminismTier, SystemDescriptor};
use crate::gameplay::quests::{QuestRegistry, QuestStatus, QuestType};
use crate::simulation::world_tick::NewMonth;
use crate::world::components::{EntityDied, EntityKind, Job};

const TRADER_QUEST_RADIUS: f32 = 80.0;
const TRADER_QUEST_RADIUS_SQ: f32 = TRADER_QUEST_RADIUS * TRADER_QUEST_RADIUS;

pub struct QuestSystem;

impl QuestSystem {
    pub fn new() -> Self {
        Self
    }
}

impl EngineSystem for QuestSystem {
    fn name(&self) -> &str {
        "QuestSystem"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("QuestSystem")
            .with_determinism(DeterminismTier::Hard)
            .after("Economy")
    }

    fn startup(&mut self, ctx: &mut StartupContext) {
        ctx.resources.insert_runtime(QuestRegistry::new());
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let registry = match ctx.resources.get_mut::<QuestRegistry>() {
            Some(r) => r,
            None => return,
        };

        let current_day = ctx.time.day + ctx.time.month * 30;
        registry.expire_quests(current_day);

        // On NewMonth: traders generate 2-4 quests each
        if ctx.events.has::<NewMonth>() {
            let mut rng = rand::thread_rng();

            let traders: Vec<_> = ctx.ecs.npcs()
                .into_iter()
                .filter(|&e| ctx.ecs.get_npc_economy(e).map_or(false, |econ| econ.job == Job::Trader))
                .filter_map(|e| ctx.ecs.identity.persistent_id_of(e))
                .collect();

            for giver in traders {
                let count = rng.gen_range(2..=4);
                for _ in 0..count {
                    registry.generate_quest(giver, current_day, &mut rng);
                }
            }
        }

        // NPC checks available quests when near trader
        for npc_entity in ctx.ecs.npcs() {
            let npc_pid = match ctx.ecs.identity.persistent_id_of(npc_entity) {
                Some(p) => p,
                None => continue,
            };

            let npc_pos = match ctx.ecs.get_transform(npc_entity) {
                Some(t) => (t.x, t.y),
                None => continue,
            };

            // Find traders within range
            for trader_entity in ctx.ecs.npcs() {
                if ctx.ecs.get_npc_economy(trader_entity).map_or(true, |e| e.job != Job::Trader) {
                    continue;
                }
                let trader_pid = match ctx.ecs.identity.persistent_id_of(trader_entity) {
                    Some(p) => p,
                    None => continue,
                };
                let trader_pos = match ctx.ecs.get_transform(trader_entity) {
                    Some(t) => (t.x, t.y),
                    None => continue,
                };

                let dx = npc_pos.0 - trader_pos.0;
                let dy = npc_pos.1 - trader_pos.1;
                if dx * dx + dy * dy > TRADER_QUEST_RADIUS_SQ {
                    continue;
                }

                // NPC is near trader — try to take an available quest if they have none active
                let has_active = registry.active_quests_for(npc_pid).next().is_some();
                if !has_active {
                    let quest_id = registry.available_quests()
                        .find(|q| q.giver == trader_pid)
                        .map(|q| q.id);
                    if let Some(qid) = quest_id {
                        registry.assign_quest(qid, npc_pid);
                    }
                }
            }
        }

        // Quest progress tracked per tick
        // 1. Process EntityDied for KillMonsters quests
        let deaths = ctx.events.read::<EntityDied>().to_vec();
        for ev in deaths {
            let killer_pid = ev.killer.and_then(|e| ctx.ecs.identity.persistent_id_of(e));
            let victim_kind = ctx.ecs.get_kind(ev.entity).cloned();

            if let (Some(killer_pid), Some(EntityKind::Monster(species))) = (killer_pid, victim_kind) {
                let to_process: Option<(u32, f32, _)> = registry.active_quests_for(killer_pid)
                    .find(|q| q.quest_type == QuestType::KillMonsters && q.target_species == Some(species))
                    .map(|q| (q.id, q.reward_money, q.assignee));
                if let Some((quest_id, reward, assignee_pid)) = to_process {
                    registry.update_progress(quest_id, 1);
                    if let Some(q) = registry.get_quest(quest_id) {
                        if q.progress >= q.target_count {
                            if let Some(pid) = assignee_pid {
                                if let Some(entity) = ctx.ecs.identity.resolve(pid) {
                                    if let Some(econ) = ctx.ecs.get_npc_economy_mut(entity) {
                                        econ.money += reward;
                                    }
                                }
                            }
                            registry.complete_quest(quest_id);
                        }
                    }
                }
            }
        }

        // 2. Check ScoutArea / FetchArtifact — NPC enters target cell
        for npc_entity in ctx.ecs.npcs() {
            let npc_pid = match ctx.ecs.identity.persistent_id_of(npc_entity) {
                Some(p) => p,
                None => continue,
            };
            let (cell_x, cell_y) = match ctx.ecs.get_transform(npc_entity) {
                Some(t) => (t.cell_x, t.cell_y),
                None => continue,
            };

            let to_complete: Option<(u32, f32, _)> = registry.active_quests_for(npc_pid)
                .find(|q| {
                    q.status == QuestStatus::Active
                        && q.target_cell.map_or(false, |t| t.0 == cell_x && t.1 == cell_y)
                        && (q.quest_type == QuestType::ScoutArea || q.quest_type == QuestType::FetchArtifact)
                })
                .map(|q| (q.id, q.reward_money, q.assignee));
            if let Some((quest_id, reward, assignee_pid)) = to_complete {
                if let Some(pid) = assignee_pid {
                    if let Some(entity) = ctx.ecs.identity.resolve(pid) {
                        if let Some(econ) = ctx.ecs.get_npc_economy_mut(entity) {
                            econ.money += reward;
                        }
                    }
                }
                registry.update_progress(quest_id, 1);
                registry.complete_quest(quest_id);
            }
        }
    }
}

impl Default for QuestSystem {
    fn default() -> Self {
        Self::new()
    }
}
