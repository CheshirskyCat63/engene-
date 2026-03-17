use crate::world::components::{NpcTraits, MonsterTraits};

pub fn npc_trait_weight(traits: &NpcTraits, index: usize) -> f32 {
    match index {
        0 => traits.bravery,
        1 => traits.aggressiveness,
        2 => traits.work_ethic,
        3 => traits.curiosity,
        4 => traits.honesty,
        5 => traits.sociality,
        6 => traits.autonomy,
        7 => traits.materialism,
        8 => traits.risk_tolerance,
        9 => traits.stress_resistance,
        _ => 0.0,
    }
}

pub fn monster_trait_weight(traits: &MonsterTraits, index: usize) -> f32 {
    match index {
        0 => traits.aggressiveness,
        1 => traits.caution,
        2 => traits.territoriality,
        3 => traits.bravery,
        4 => traits.pack_mentality,
        5 => traits.energy_level,
        6 => traits.hoarding,
        7 => traits.curiosity,
        8 => traits.adaptability,
        9 => traits.stress_tolerance,
        _ => 0.0,
    }
}
