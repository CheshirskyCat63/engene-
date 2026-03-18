use crate::game::ai::goals::{pick_best, ScoredGoal};
use crate::world::components::*;

pub fn decide_npc(
    traits: &NpcTraits,
    personal: &PersonalNeeds,
    social: &SocialNeeds,
    economy: &NpcEconomy,
) -> Goal {
    let mut c = Vec::with_capacity(12);

    c.push(ScoredGoal {
        goal: Goal::SeekFood,
        score: personal.hunger * 1.8 + personal.discomfort * 0.3,
    });
    c.push(ScoredGoal {
        goal: Goal::SeekWater,
        score: personal.thirst * 2.2,
    });
    c.push(ScoredGoal {
        goal: Goal::Rest,
        score: personal.sleep * 1.8
            + (1.0 - personal.energy) * 0.9
            + (1.0 - personal.health) * 1.5
            + personal.discomfort * 0.5 * (1.0 - traits.stress_resistance),
    });
    c.push(ScoredGoal {
        goal: Goal::Work,
        score: social.money * traits.work_ethic * 1.5
            + traits.materialism * 0.5
            + economy.desperation * 1.2
            + personal.ambitions * traits.work_ethic * 0.8
            + social.competition * 0.3,
    });
    c.push(ScoredGoal {
        goal: Goal::Hunt,
        score: traits.bravery * 0.5
            + traits.aggressiveness * 0.6
            + personal.hunger * 2.5
            + economy.desperation * 0.8
            + personal.ambitions * 0.3
            + social.reputation * 0.2
            + (1.0 - traits.autonomy) * 0.3,
    });
    c.push(ScoredGoal {
        goal: Goal::Trade,
        score: social.money * 0.8
            + traits.materialism * 0.7
            + social.reputation * 0.3
            + social.competition * 0.2,
    });
    c.push(ScoredGoal {
        goal: Goal::Explore,
        score: personal.curiosity * traits.curiosity * 1.6
            + personal.ambitions * 0.3
            + traits.autonomy * 0.3
            + personal.discomfort * 0.2,
    });
    c.push(ScoredGoal {
        goal: Goal::Socialize,
        score: social.loneliness * traits.sociality * 1.6
            + social.entertainment * 0.4
            + social.family * 0.5
            + social.friendship * 0.3
            + social.faction_loyalty * 0.2
            + (1.0 - traits.autonomy) * 0.3,
    });
    c.push(ScoredGoal {
        goal: Goal::Flee,
        score: personal.fear * 2.5 * (1.0 - traits.bravery)
            + (1.0 - personal.health) * personal.fear * 1.5
            + (1.0 - traits.stress_resistance) * personal.fear * 0.5,
    });

    if economy.desperation > 0.5 {
        c.push(ScoredGoal {
            goal: Goal::StealOrRob,
            score: economy.desperation * (1.0 - traits.honesty) * 2.0
                + traits.risk_tolerance * 0.6
                + traits.aggressiveness * 0.4
                - social.fear_of_punishment * 1.0
                - social.reputation * 0.3,
        });
    }

    pick_best(&c)
}

pub fn decide_monster(
    traits: &MonsterTraits,
    personal: &PersonalNeeds,
    eco: &EcosystemNeeds,
) -> Goal {
    let mut c = Vec::with_capacity(10);

    c.push(ScoredGoal {
        goal: Goal::Hunt,
        score: personal.hunger * 3.0
            + eco.hunting * traits.aggressiveness * 1.5
            + traits.bravery * 0.3
            + eco.prey_selection * 0.3
            + personal.ambitions * 0.2,
    });
    c.push(ScoredGoal {
        goal: Goal::Flee,
        score: personal.fear * 3.0 * traits.caution
            + eco.predator_avoidance * 2.0
            + (1.0 - personal.health) * traits.caution * 2.0
            + (1.0 - traits.bravery) * personal.fear * 1.0,
    });
    c.push(ScoredGoal {
        goal: Goal::Rest,
        score: personal.sleep * 1.8
            + (1.0 - personal.energy) * (2.0 - traits.energy_level)
            + (1.0 - personal.health) * 1.5
            + personal.discomfort * 0.5
            + eco.shelter_seeking * 0.5,
    });
    c.push(ScoredGoal {
        goal: Goal::DefendTerritory,
        score: eco.territory_control * traits.territoriality * 1.8
            + traits.aggressiveness * 0.4
            + eco.resource_competition * 0.5
            + eco.world_event_reaction * 0.3,
    });
    c.push(ScoredGoal {
        goal: Goal::Migrate,
        score: eco.migration_urge * 1.5
            + personal.hunger * 0.4 * (1.0 - eco.hunting)
            + eco.resource_competition * 0.6
            + traits.adaptability * 0.3
            + personal.discomfort * 0.3,
    });
    c.push(ScoredGoal {
        goal: Goal::FollowPack,
        score: traits.pack_mentality * eco.pack_following * 1.5
            + personal.fear * traits.pack_mentality * 0.5
            + (1.0 - traits.bravery) * 0.2,
    });
    c.push(ScoredGoal {
        goal: Goal::Explore,
        score: traits.curiosity * personal.curiosity * 1.3 + traits.adaptability * 0.2,
    });
    c.push(ScoredGoal {
        goal: Goal::SeekFood,
        score: personal.hunger * 1.5 * (1.0 - traits.aggressiveness)
            + traits.hoarding * 0.5
            + eco.food_chain_position * 0.2,
    });

    pick_best(&c)
}
