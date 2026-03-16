use rand::Rng;

use crate::ai::emotions::Emotions;
use crate::ai::memory::Memory;
use crate::core::ecs::Ecs;
use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use crate::world::components::*;

static NPC_NAMES: &[&str] = &[
    "Viktor", "Elena", "Sasha", "Dmitri", "Irina",
    "Andrei", "Natasha", "Boris", "Yuri", "Olga",
    "Maxim", "Tatiana", "Sergei", "Anya", "Pavel",
    "Ilya", "Marina", "Roman", "Vera", "Artem",
];

pub fn spawn_npcs(ecs: &mut Ecs) {
    let mut rng = rand::thread_rng();
    let jobs = [
        Job::Guard, Job::Trader, Job::ArtifactHunter, Job::Guard, Job::Trader,
        Job::ArtifactHunter, Job::Guard, Job::Trader, Job::Guard, Job::ArtifactHunter,
        Job::Trader, Job::Guard, Job::ArtifactHunter, Job::Trader, Job::Guard,
        Job::ArtifactHunter, Job::Guard, Job::Trader, Job::Guard, Job::Trader,
    ];

    for (i, &name) in NPC_NAMES.iter().enumerate() {
        let (e, _pid) = ecs.spawn_new();
        let center = GRID_SIZE / 2;
        let cx = rng.gen_range(center.saturating_sub(2)..center + 2);
        let cy = rng.gen_range(center.saturating_sub(2)..center + 2);

        ecs.transforms.insert(e, Transform {
            x: cx as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
            y: cy as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
            cell_x: cx,
            cell_y: cy,
        });
        ecs.kinds.insert(e, EntityKind::Npc);
        ecs.names.insert(e, Name(name.to_string()));
        ecs.npc_traits.insert(e, random_npc_traits(&mut rng));
        ecs.personal_needs.insert(e, PersonalNeeds::default_npc());
        ecs.social_needs.insert(e, SocialNeeds::default());
        ecs.npc_economies.insert(e, NpcEconomy {
            money: rng.gen_range(20.0..60.0),
            monthly_required: 50.0,
            job: jobs[i % jobs.len()],
            desperation: 0.0,
        });
        ecs.sim_levels.insert(e, SimLevel { level: SimulationLevel::L1 });
        ecs.ai_states.insert(e, AiState::Idle);
        ecs.inventories.insert(e, Inventory { items: Vec::new() });
        ecs.memories.insert(e, Memory::new());
        ecs.emotions.insert(e, Emotions::new());
        ecs.life_info.insert(e, LifeInfo {
            age: rng.gen_range(80.0..200.0),
            max_age: rng.gen_range(350.0..450.0),
            last_mate_day: 0,
            mate_cooldown_days: 60,
        });
        ecs.equipment.insert(e, EquipmentSlots::default_stalker());
        ecs.faction_memberships.insert(e, FactionMembership::default());
    }
}

pub fn spawn_monsters(ecs: &mut Ecs) {
    let mut rng = rand::thread_rng();

    let g = GRID_SIZE;
    let packs: Vec<(MonsterSpecies, u32, u32, u32)> = vec![
        (MonsterSpecies::Wolf, 6, g / 5, g / 5),
        (MonsterSpecies::Wolf, 5, 3 * g / 4, g / 3),
        (MonsterSpecies::Wolf, 4, g / 10, 4 * g / 5),
        (MonsterSpecies::Wolf, 5, 3 * g / 5, g / 10),
        (MonsterSpecies::Wolf, 5, 17 * g / 20, 13 * g / 20),
        (MonsterSpecies::Wolf, 5, 2 * g / 5, 9 * g / 10),
        (MonsterSpecies::Boar, 5, 7 * g / 20, 3 * g / 5),
        (MonsterSpecies::Boar, 5, 7 * g / 10, 7 * g / 10),
        (MonsterSpecies::Boar, 5, 3 * g / 20, 2 * g / 5),
        (MonsterSpecies::Boar, 5, 9 * g / 10, g / 4),
        (MonsterSpecies::Boar, 5, 3 * g / 10, g / 20),
        (MonsterSpecies::Boar, 5, 4 * g / 5, 9 * g / 10),
        (MonsterSpecies::Bloodsucker, 4, 3 * g / 20, 17 * g / 20),
        (MonsterSpecies::Bloodsucker, 4, 17 * g / 20, 3 * g / 20),
        (MonsterSpecies::Bloodsucker, 4, g / 20, g / 20),
        (MonsterSpecies::Bloodsucker, 4, 9 * g / 10, 9 * g / 10),
        (MonsterSpecies::Bloodsucker, 4, g / 2, 3 * g / 4),
    ];

    for &(species, count, cx, cy) in &packs {
        for _ in 0..count {
            let (e, _pid) = ecs.spawn_new();
            ecs.transforms.insert(e, Transform {
                x: cx as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
                y: cy as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
                cell_x: cx,
                cell_y: cy,
            });
            ecs.kinds.insert(e, EntityKind::Monster(species));
            ecs.names.insert(e, Name(format!("{}", species)));
            ecs.monster_traits.insert(e, random_monster_traits(&mut rng, species));
            ecs.personal_needs.insert(e, PersonalNeeds::default_monster());
            ecs.ecosystem_needs.insert(e, EcosystemNeeds::for_species(species));
            ecs.sim_levels.insert(e, SimLevel { level: SimulationLevel::L1 });
            ecs.ai_states.insert(e, AiState::Idle);
            ecs.inventories.insert(e, Inventory { items: Vec::new() });
            ecs.memories.insert(e, Memory::new());
            ecs.emotions.insert(e, Emotions::new());
            let mut li = LifeInfo::new_monster(species);
            li.age = rng.gen_range(10.0..li.max_age * 0.5);
            ecs.life_info.insert(e, li);
        }
    }
}

pub fn spawn_single_monster(ecs: &mut Ecs, species: MonsterSpecies, rng: &mut impl Rng) {
    let (e, _pid) = ecs.spawn_new();
    let cx: u32 = rng.gen_range(0..GRID_SIZE);
    let cy: u32 = rng.gen_range(0..GRID_SIZE);
    ecs.transforms.insert(
        e,
        Transform {
            x: cx as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
            y: cy as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
            cell_x: cx,
            cell_y: cy,
        },
    );
    ecs.kinds.insert(e, EntityKind::Monster(species));
    ecs.names.insert(e, Name(format!("{}", species)));
    ecs.monster_traits
        .insert(e, random_monster_traits(rng, species));
    ecs.personal_needs
        .insert(e, PersonalNeeds::default_monster());
    ecs.ecosystem_needs
        .insert(e, EcosystemNeeds::for_species(species));
    ecs.sim_levels.insert(
        e,
        SimLevel {
            level: SimulationLevel::L1,
        },
    );
    ecs.ai_states.insert(e, AiState::Idle);
    ecs.inventories.insert(e, Inventory { items: Vec::new() });
    ecs.memories.insert(e, Memory::new());
    ecs.emotions.insert(e, Emotions::new());
    ecs.life_info.insert(e, LifeInfo::new_monster(species));
}

fn random_npc_traits(rng: &mut impl Rng) -> NpcTraits {
    NpcTraits {
        bravery: rng.gen_range(0.2..0.9),
        aggressiveness: rng.gen_range(0.1..0.7),
        work_ethic: rng.gen_range(0.3..1.0),
        curiosity: rng.gen_range(0.2..0.8),
        honesty: rng.gen_range(0.2..0.9),
        sociality: rng.gen_range(0.2..0.8),
        autonomy: rng.gen_range(0.3..0.8),
        materialism: rng.gen_range(0.2..0.8),
        risk_tolerance: rng.gen_range(0.2..0.8),
        stress_resistance: rng.gen_range(0.3..0.9),
    }
}

fn random_monster_traits(rng: &mut impl Rng, species: MonsterSpecies) -> MonsterTraits {
    let base = match species {
        MonsterSpecies::Wolf => (0.6, 0.4, 0.5, 0.5, 0.8),
        MonsterSpecies::Boar => (0.3, 0.6, 0.3, 0.3, 0.4),
        MonsterSpecies::Bloodsucker => (0.9, 0.2, 0.8, 0.7, 0.1),
    };
    let mut jitter = |v: f32| (v + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);

    MonsterTraits {
        aggressiveness: jitter(base.0),
        caution: jitter(base.1),
        territoriality: jitter(base.2),
        bravery: jitter(base.3),
        pack_mentality: jitter(base.4),
        energy_level: rng.gen_range(0.5..0.9),
        hoarding: rng.gen_range(0.1..0.5),
        curiosity: rng.gen_range(0.1..0.5),
        adaptability: rng.gen_range(0.3..0.7),
        stress_tolerance: rng.gen_range(0.3..0.8),
    }
}
