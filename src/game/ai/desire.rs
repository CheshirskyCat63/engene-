use crate::game::ai::body;
use crate::game::ai::emotions::DominantEmotion;
use crate::game::ai::memory::{LessonAction, LessonContext};
use crate::game::ai::thresholds::Thresholds;
use crate::core::ecs::{Ecs, Entity};
use crate::world::components::*;

/// Determine NPC goal based on needs, traits, and context.
/// 
/// Uses helper methods instead of direct storage access.
pub fn desire_npc(entity: Entity, ecs: &Ecs, day_progress: f32) -> Goal {
    let pn = match ecs.get_needs(entity) { Some(p) => p, None => return Goal::Rest };
    let traits = match ecs.get_npc_traits(entity) { Some(t) => t, None => return Goal::Rest };
    let sn = match ecs.get_social_needs(entity) { Some(s) => s, None => return Goal::Rest };
    let econ = match ecs.get_npc_economy(entity) { Some(e) => e, None => return Goal::Rest };
    let emo = match ecs.get_emotions(entity) { Some(e) => e, None => return Goal::Rest };
    let mem = ecs.get_memory(entity);
    let th = Thresholds::from_npc(traits, mem);

    if pn.health < th.health_panic {
        return if emo.fear > 0.3 || traits.bravery < 0.4 { Goal::Flee } else { Goal::Rest };
    }
    if emo.fear > th.fear_flee {
        return Goal::Flee;
    }

    // Territory avoidance: if in rival monster territory, increase urgency to flee
    if let Some(t) = ecs.get_transform(entity) {
        if ecs.territory.contains_key(&(t.cell_x, t.cell_y)) && traits.bravery < 0.6 {
            return Goal::Flee;
        }
    }

    if pn.hunger > th.hunger_critical {
        let conf = hunt_confidence_npc(mem, traits);
        return if conf > 0.4 && traits.aggressiveness > 0.3 { Goal::Hunt } else { Goal::SeekFood };
    }
    if pn.thirst > th.thirst_critical { return Goal::SeekWater; }

    // Shelter & sleep: at night, non-nocturnal entities seek shelter
    let is_night = body::is_night(day_progress);
    if is_night && pn.sleep > th.sleep_critical {
        let at_shelter = mem.map_or(false, |m| {
            if let Some(t) = ecs.get_transform(entity) {
                m.spatial.get(&(t.cell_x, t.cell_y)).map_or(false, |k| k.shelter > 0.3)
            } else { false }
        });
        return if at_shelter { Goal::Sleep } else { Goal::SeekShelter };
    }

    if pn.sleep > th.sleep_critical || pn.energy < th.energy_low { return Goal::Rest; }

    // Reproduction: if healthy, fed, energetic, and cooldown passed
    if let Some(li) = ecs.get_life_info(entity) {
        if li.life_stage() == LifeStage::Adult && pn.health > 0.7 && pn.hunger < 0.3 && pn.energy > 0.5 {
            if li.can_mate(ecs.tick as u32 / 120) {
                if traits.sociality > 0.3 { return Goal::Mate; }
            }
        }
    }

    match emo.dominant() {
        DominantEmotion::Anger => {
            if traits.aggressiveness > 0.4 {
                return if econ.desperation > th.desperation_crime && traits.honesty < 0.4 { Goal::StealOrRob } else { Goal::Hunt };
            }
        }
        DominantEmotion::Grief | DominantEmotion::Longing => {
            if traits.sociality > 0.4 { return Goal::Socialize; }
        }
        DominantEmotion::Fear => {
            if traits.bravery < 0.5 { return Goal::Flee; }
        }
        _ => {}
    }

    if econ.desperation > 0.6 {
        if traits.honesty < 0.4 && traits.risk_tolerance > 0.5 {
            let steal_score = mem.map_or(0.5, |m| m.lesson_score(LessonAction::Steal, LessonContext::General));
            if steal_score > 0.3 { return Goal::StealOrRob; }
        }
        return Goal::Work;
    }

    if sn.loneliness > th.loneliness_seek && traits.sociality > 0.4 {
        return Goal::Socialize;
    }

    if pn.hunger > th.hunger_proactive {
        let conf = hunt_confidence_npc(mem, traits);
        if conf > 0.5 { return Goal::Hunt; }
        return Goal::SeekFood;
    }

    if sn.money > 0.5 && traits.work_ethic > 0.5 { return Goal::Work; }
    if sn.money > 0.4 && traits.materialism > 0.5 { return Goal::Trade; }

    if pn.curiosity > 0.5 && traits.curiosity > 0.4 { return Goal::Explore; }
    if pn.ambitions > 0.5 {
        return if traits.work_ethic > traits.aggressiveness { Goal::Work } else { Goal::Hunt };
    }
    if sn.entertainment > 0.5 { return Goal::Socialize; }

    Goal::Explore
}

pub fn desire_monster(entity: Entity, ecs: &Ecs, day_progress: f32) -> Goal {
    let pn = match ecs.get_needs(entity) { Some(p) => p, None => return Goal::Rest };
    let traits = match ecs.get_monster_traits(entity) { Some(t) => t, None => return Goal::Rest };
    let eco = match ecs.get_ecosystem_needs(entity) { Some(e) => e, None => return Goal::Rest };
    let emo = match ecs.get_emotions(entity) { Some(e) => e, None => return Goal::Rest };
    let mem = ecs.get_memory(entity);
    let th = Thresholds::from_monster(traits, mem);
    let is_nocturnal = matches!(ecs.get_kind(entity), Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)));

    if pn.health < th.health_panic {
        return if traits.bravery > 0.7 { Goal::DefendTerritory } else { Goal::Flee };
    }
    if emo.fear > th.fear_flee && traits.caution > 0.3 {
        return if traits.pack_mentality > 0.5 { Goal::FollowPack } else { Goal::Flee };
    }

    // Territory avoidance: if in rival species territory, flee or avoid
    if let Some(t) = ecs.get_transform(entity) {
        if let Some(dom) = ecs.territory_owner(t.cell_x, t.cell_y) {
            if let Some(EntityKind::Monster(my_sp)) = ecs.get_kind(entity) {
                if dom != *my_sp && traits.bravery < 0.6 {
                    return Goal::Flee;
                }
            }
        }
    }

    if pn.hunger > th.hunger_critical {
        let conf = hunt_confidence_monster(mem, traits);
        return if conf > 0.3 && traits.aggressiveness > 0.2 { Goal::Hunt } else { Goal::SeekFood };
    }

    // Shelter & sleep for non-nocturnal at night, or nocturnal during day
    let is_night = body::is_night(day_progress);
    let wants_sleep = if is_nocturnal { !is_night } else { is_night };
    if wants_sleep && pn.sleep > th.sleep_critical {
        let at_shelter = mem.map_or(false, |m| {
            if let Some(t) = ecs.get_transform(entity) {
                m.spatial.get(&(t.cell_x, t.cell_y)).map_or(false, |k| k.shelter > 0.3)
            } else { false }
        });
        return if at_shelter { Goal::Sleep } else { Goal::SeekShelter };
    }

    if pn.sleep > th.sleep_critical || pn.energy < th.energy_low { return Goal::Rest; }

    // Reproduction for adult monsters
    if let Some(li) = ecs.get_life_info(entity) {
        if li.life_stage() == LifeStage::Adult && pn.health > 0.7 && pn.hunger < 0.3 && pn.energy > 0.5 {
            if li.can_mate(ecs.tick as u32 / 120) {
                return Goal::Mate;
            }
        }
    }

    match emo.dominant() {
        DominantEmotion::Anger => {
            return if eco.territory_control > 0.4 { Goal::DefendTerritory } else { Goal::Hunt };
        }
        DominantEmotion::Grief | DominantEmotion::Longing => {
            if traits.pack_mentality > 0.4 { return Goal::FollowPack; }
        }
        DominantEmotion::Fear => { return Goal::Flee; }
        _ => {}
    }

    if eco.territory_control > 0.5 && traits.territoriality > 0.5 { return Goal::DefendTerritory; }
    if eco.pack_following > 0.6 && traits.pack_mentality > 0.5 { return Goal::FollowPack; }

    if pn.hunger > th.hunger_proactive && traits.aggressiveness > 0.3 { return Goal::Hunt; }
    if eco.migration_urge > 0.5 || eco.resource_competition > 0.6 { return Goal::Migrate; }
    if pn.hunger > 0.3 { return Goal::SeekFood; }
    if traits.curiosity > 0.3 && pn.curiosity > 0.3 { return Goal::Explore; }

    Goal::Rest
}

fn hunt_confidence_npc(mem: Option<&crate::core::ai_memory::Memory>, traits: &NpcTraits) -> f32 {
    let base = traits.bravery * 0.3 + traits.aggressiveness * 0.3;
    let learned = mem.map_or(0.5, |m| {
        let solo = m.lesson_score(LessonAction::SoloHunt, LessonContext::VsWolf);
        let group = m.lesson_score(LessonAction::GroupHunt, LessonContext::VsBoar);
        solo * 0.5 + group * 0.5
    });
    (base + learned) * 0.5
}

fn hunt_confidence_monster(mem: Option<&crate::core::ai_memory::Memory>, traits: &MonsterTraits) -> f32 {
    let base = traits.aggressiveness * 0.4 + traits.bravery * 0.2;
    let learned = mem.map_or(0.5, |m| {
        let solo = m.lesson_score(LessonAction::SoloHunt, LessonContext::General);
        let group = m.lesson_score(LessonAction::GroupHunt, LessonContext::General);
        (solo + group) * 0.5
    });
    (base + learned) * 0.5
}
