use rand::Rng;

use crate::core::ai_emotions::Emotions;
use crate::core::ai_memory::Memory;
use crate::core::ecs::{Ecs, Entity};
use crate::world::cell::CELL_SIZE;
use crate::world::components::*;

const MAX_NPCS: usize = 30;
const MAX_WOLVES: usize = 40;
const MAX_BOARS: usize = 40;
const MAX_BLOODSUCKERS: usize = 25;

/// Attempt reproduction between two entities.
/// 
/// Uses helper methods instead of direct storage access.
pub fn try_reproduce(ecs: &mut Ecs, parent: Entity, mate: Entity) -> Option<Entity> {
    let kind = ecs.get_kind(parent)?.clone();
    let species_count = count_species(ecs, &kind);
    let cap = species_cap(&kind);
    if species_count >= cap { return None; }

    let pt = ecs.get_transform(parent)?.clone();
    let mut rng = rand::thread_rng();

    let (child, child_pid) = ecs.spawn_new();
    let offset_x: f32 = rng.gen_range(-CELL_SIZE * 0.3..CELL_SIZE * 0.3);
    let offset_y: f32 = rng.gen_range(-CELL_SIZE * 0.3..CELL_SIZE * 0.3);
    let ws = crate::world::cell::WORLD_SIZE;
    let cx = (pt.x + offset_x).clamp(0.0, ws);
    let cy = (pt.y + offset_y).clamp(0.0, ws);
    let (cell_x, cell_y) = crate::world::cell::pos_to_cell(cx, cy);

    ecs.transforms.insert(child, Transform { x: cx, y: cy, cell_x, cell_y });
    ecs.kinds.insert(child, kind.clone());
    ecs.ai_states.insert(child, AiState::Idle);
    ecs.inventories.insert(child, Inventory { items: Vec::new() });
    ecs.memories.insert(child, Memory::new());
    ecs.emotions.insert(child, Emotions::new());
    ecs.sim_levels.insert(child, SimLevel { level: SimulationLevel::L1 });

    match &kind {
        EntityKind::Npc => {
            let p_traits = ecs.get_npc_traits(parent);
            let m_traits = ecs.get_npc_traits(mate);
            let blended = blend_npc_traits(p_traits, m_traits, &mut rng);
            let name = generate_child_name(ecs, parent);
            ecs.names.insert(child, Name(name));
            ecs.npc_traits.insert(child, blended);
            ecs.personal_needs.insert(child, PersonalNeeds::default_npc());
            ecs.social_needs.insert(child, SocialNeeds::default());
            ecs.npc_economies.insert(child, NpcEconomy {
                money: rng.gen_range(5.0..15.0),
                monthly_required: 50.0,
                job: Job::Unemployed,
                desperation: 0.0,
            });
            ecs.life_info.insert(child, LifeInfo { age: 0.0, max_age: rng.gen_range(350.0..450.0), last_mate_day: 0, mate_cooldown_days: 60 });
        }
        EntityKind::Monster(species) => {
            ecs.names.insert(child, Name(format!("{}", species)));
            ecs.monster_traits.insert(child, blend_monster_traits(ecs, parent, mate, *species, &mut rng));
            ecs.personal_needs.insert(child, PersonalNeeds::default_monster());
            ecs.ecosystem_needs.insert(child, EcosystemNeeds::for_species(*species));
            ecs.life_info.insert(child, LifeInfo::new_monster(*species).with_age(0.0));
        }
    }

    // Mark mate cooldown on both parents
    let current_day = ecs.tick as u32 / 120;
    if let Some(li) = ecs.get_life_info_mut(parent) { li.last_mate_day = current_day; }
    if let Some(li) = ecs.get_life_info_mut(mate) { li.last_mate_day = current_day; }

    if let Some(mem) = ecs.get_memory_mut(parent) {
        mem.adjust_opinion(child_pid, |op| { op.trust = 0.8; op.familiarity = 0.5; });
    }
    if let Some(mem) = ecs.get_memory_mut(mate) {
        mem.adjust_opinion(child_pid, |op| { op.trust = 0.7; op.familiarity = 0.4; });
    }

    Some(child)
}

fn count_species(ecs: &Ecs, kind: &EntityKind) -> usize {
    match kind {
        EntityKind::Npc => ecs.count_npcs(),
        EntityKind::Monster(sp) => ecs.count_species(*sp),
    }
}

fn species_cap(kind: &EntityKind) -> usize {
    match kind {
        EntityKind::Npc => MAX_NPCS,
        EntityKind::Monster(MonsterSpecies::Wolf) => MAX_WOLVES,
        EntityKind::Monster(MonsterSpecies::Boar) => MAX_BOARS,
        EntityKind::Monster(MonsterSpecies::Bloodsucker) => MAX_BLOODSUCKERS,
    }
}

fn blend_npc_traits(a: Option<&NpcTraits>, b: Option<&NpcTraits>, rng: &mut impl Rng) -> NpcTraits {
    let mut blend = |va: f32, vb: f32| -> f32 {
        let mid = (va + vb) * 0.5;
        (mid + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0)
    };
    let da = NpcTraits { bravery: 0.5, aggressiveness: 0.4, work_ethic: 0.5, curiosity: 0.5, honesty: 0.5, sociality: 0.5, autonomy: 0.5, materialism: 0.4, risk_tolerance: 0.4, stress_resistance: 0.5 };
    let pa = a.unwrap_or(&da);
    let pb = b.unwrap_or(&da);

    NpcTraits {
        bravery: blend(pa.bravery, pb.bravery),
        aggressiveness: blend(pa.aggressiveness, pb.aggressiveness),
        work_ethic: blend(pa.work_ethic, pb.work_ethic),
        curiosity: blend(pa.curiosity, pb.curiosity),
        honesty: blend(pa.honesty, pb.honesty),
        sociality: blend(pa.sociality, pb.sociality),
        autonomy: blend(pa.autonomy, pb.autonomy),
        materialism: blend(pa.materialism, pb.materialism),
        risk_tolerance: blend(pa.risk_tolerance, pb.risk_tolerance),
        stress_resistance: blend(pa.stress_resistance, pb.stress_resistance),
    }
}

fn blend_monster_traits(ecs: &Ecs, parent: Entity, mate: Entity, species: MonsterSpecies, rng: &mut impl Rng) -> MonsterTraits {
    let mut blend = |va: f32, vb: f32| -> f32 {
        let mid = (va + vb) * 0.5;
        (mid + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0)
    };
    let pa = ecs.get_monster_traits(parent);
    let pb = ecs.get_monster_traits(mate);
    let base = match species {
        MonsterSpecies::Wolf => MonsterTraits { aggressiveness: 0.6, caution: 0.4, territoriality: 0.5, bravery: 0.5, pack_mentality: 0.8, energy_level: 0.7, hoarding: 0.2, curiosity: 0.3, adaptability: 0.5, stress_tolerance: 0.5 },
        MonsterSpecies::Boar => MonsterTraits { aggressiveness: 0.3, caution: 0.6, territoriality: 0.3, bravery: 0.3, pack_mentality: 0.4, energy_level: 0.6, hoarding: 0.3, curiosity: 0.2, adaptability: 0.5, stress_tolerance: 0.5 },
        MonsterSpecies::Bloodsucker => MonsterTraits { aggressiveness: 0.9, caution: 0.2, territoriality: 0.8, bravery: 0.7, pack_mentality: 0.1, energy_level: 0.8, hoarding: 0.3, curiosity: 0.3, adaptability: 0.5, stress_tolerance: 0.5 },
    };
    let a = pa.unwrap_or(&base);
    let b = pb.unwrap_or(&base);

    MonsterTraits {
        aggressiveness: blend(a.aggressiveness, b.aggressiveness),
        caution: blend(a.caution, b.caution),
        territoriality: blend(a.territoriality, b.territoriality),
        bravery: blend(a.bravery, b.bravery),
        pack_mentality: blend(a.pack_mentality, b.pack_mentality),
        energy_level: blend(a.energy_level, b.energy_level),
        hoarding: blend(a.hoarding, b.hoarding),
        curiosity: blend(a.curiosity, b.curiosity),
        adaptability: blend(a.adaptability, b.adaptability),
        stress_tolerance: blend(a.stress_tolerance, b.stress_tolerance),
    }
}

fn generate_child_name(ecs: &Ecs, parent: Entity) -> String {
    static CHILD_NAMES: &[&str] = &[
        "Alyosha", "Misha", "Katya", "Dasha", "Pasha",
        "Kolya", "Vanya", "Sveta", "Zhenya", "Borya",
        "Lena", "Grisha", "Tonya", "Nikita", "Oleg",
    ];
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..CHILD_NAMES.len());
    let base = CHILD_NAMES[idx];
    let parent_name = ecs.get_name(parent).map(|n| n.0.as_str()).unwrap_or("?");
    format!("{} (child of {})", base, parent_name)
}
