use crate::world::components::MonsterSpecies;

pub fn is_predator_of(predator: MonsterSpecies, prey: MonsterSpecies) -> bool {
    matches!(
        (predator, prey),
        (MonsterSpecies::Wolf, MonsterSpecies::Boar)
            | (MonsterSpecies::Bloodsucker, MonsterSpecies::Wolf)
    )
}

pub fn is_prey_of(species: MonsterSpecies) -> Option<MonsterSpecies> {
    match species {
        MonsterSpecies::Boar => Some(MonsterSpecies::Wolf),
        MonsterSpecies::Wolf => Some(MonsterSpecies::Bloodsucker),
        MonsterSpecies::Bloodsucker => None,
    }
}

pub fn food_chain_rank(species: MonsterSpecies) -> u8 {
    match species {
        MonsterSpecies::Boar => 1,
        MonsterSpecies::Wolf => 2,
        MonsterSpecies::Bloodsucker => 3,
    }
}
