use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::core::ecs::{Ecs, Entity};
use crate::world::components::*;
use crate::world::streaming::ChunkCoord;

pub const SNAPSHOT_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub version: u32,
    pub day: u32,
    pub month: u32,
    pub year: u32,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: u64,
    pub persistent_id: Option<u64>,
    pub transform: Option<SerTransform>,
    pub kind: Option<SerEntityKind>,
    pub name: Option<String>,
    pub personal_needs: Option<SerPersonalNeeds>,
    pub social_needs: Option<SerSocialNeeds>,
    pub ecosystem_needs: Option<SerEcosystemNeeds>,
    pub npc_traits: Option<SerNpcTraits>,
    pub monster_traits: Option<SerMonsterTraits>,
    pub npc_economy: Option<SerNpcEconomy>,
    pub sim_level: Option<u8>,
    pub ai_state: Option<u8>,
    pub life_info: Option<SerLifeInfo>,
    pub emotions: Option<SerEmotions>,
    pub flammable: Option<u8>,
    #[serde(default)]
    pub inventory: Option<SerInventory>,
    #[serde(default)]
    pub equipment: Option<SerEquipment>,
    #[serde(default)]
    pub faction_membership: Option<SerFactionMembership>,
}

#[derive(Serialize, Deserialize)]
pub struct SerTransform {
    pub x: f32,
    pub y: f32,
    pub cell_x: u32,
    pub cell_y: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SerEntityKind {
    pub is_npc: bool,
    pub species: Option<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct SerPersonalNeeds {
    pub hunger: f32,
    pub thirst: f32,
    pub sleep: f32,
    pub health: f32,
    pub energy: f32,
    pub fear: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerSocialNeeds {
    pub family: f32,
    pub money: f32,
    pub reputation: f32,
    pub friendship: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerEcosystemNeeds {
    pub hunting: f32,
    pub territory_control: f32,
    pub pack_following: f32,
    pub migration_urge: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerNpcTraits {
    pub bravery: f32,
    pub aggressiveness: f32,
    pub work_ethic: f32,
    pub curiosity: f32,
    pub honesty: f32,
    pub sociality: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerMonsterTraits {
    pub aggressiveness: f32,
    pub caution: f32,
    pub territoriality: f32,
    pub bravery: f32,
    pub pack_mentality: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerNpcEconomy {
    pub money: f32,
    pub monthly_required: f32,
    pub job: u8,
    pub desperation: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerLifeInfo {
    pub age: f32,
    pub max_age: f32,
    pub last_mate_day: u32,
    pub mate_cooldown_days: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SerEmotions {
    pub anger: f32,
    pub grief: f32,
    pub joy: f32,
    pub fear: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerInventory {
    pub items: Vec<SerItem>,
}

#[derive(Serialize, Deserialize)]
pub struct SerItem {
    pub name: String,
    pub value: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerEquipment {
    pub weapon_condition: f32,
    pub armor_condition: f32,
    pub medkits: u32,
    pub food_rations: u32,
    pub ammo: u32,
    pub total_weight: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SerFactionMembership {
    pub faction: u8,
    pub standing: f32,
}

pub fn snapshot_entity(id: Entity, ecs: &Ecs) -> EntitySnapshot {
    let persistent_id = ecs.identity.persistent_id_of(id).map(|pid| pid.0);
    let transform = ecs.get_transform(id).map(|t| SerTransform {
        x: t.x,
        y: t.y,
        cell_x: t.cell_x,
        cell_y: t.cell_y,
    });

    let kind = ecs.get_kind(id).map(|k| match k {
        EntityKind::Npc => SerEntityKind {
            is_npc: true,
            species: None,
        },
        EntityKind::Monster(s) => SerEntityKind {
            is_npc: false,
            species: Some(match s {
                MonsterSpecies::Wolf => 0,
                MonsterSpecies::Boar => 1,
                MonsterSpecies::Bloodsucker => 2,
            }),
        },
    });

    let personal_needs = ecs.get_needs(id).map(|pn| SerPersonalNeeds {
        hunger: pn.hunger,
        thirst: pn.thirst,
        sleep: pn.sleep,
        health: pn.health,
        energy: pn.energy,
        fear: pn.fear,
    });

    let social_needs = ecs.get_social_needs(id).map(|sn| SerSocialNeeds {
        family: sn.family,
        money: sn.money,
        reputation: sn.reputation,
        friendship: sn.friendship,
    });

    let ecosystem_needs = ecs.get_ecosystem_needs(id).map(|en| SerEcosystemNeeds {
        hunting: en.hunting,
        territory_control: en.territory_control,
        pack_following: en.pack_following,
        migration_urge: en.migration_urge,
    });

    let npc_traits = ecs.get_npc_traits(id).map(|t| SerNpcTraits {
        bravery: t.bravery,
        aggressiveness: t.aggressiveness,
        work_ethic: t.work_ethic,
        curiosity: t.curiosity,
        honesty: t.honesty,
        sociality: t.sociality,
    });

    let monster_traits = ecs.get_monster_traits(id).map(|t| SerMonsterTraits {
        aggressiveness: t.aggressiveness,
        caution: t.caution,
        territoriality: t.territoriality,
        bravery: t.bravery,
        pack_mentality: t.pack_mentality,
    });

    let npc_economy = ecs.get_npc_economy(id).map(|e| SerNpcEconomy {
        money: e.money,
        monthly_required: e.monthly_required,
        job: match e.job {
            Job::ArtifactHunter => 0,
            Job::Guard => 1,
            Job::Trader => 2,
            Job::Bandit => 3,
            Job::Unemployed => 4,
            Job::Hunter => 5,
            Job::Scavenger => 6,
            Job::Courier => 7,
            Job::Resident => 8,
        },
        desperation: e.desperation,
    });

    let sim_level = ecs.sim_levels.get(&id).map(|s| match s.level {
        SimulationLevel::L0 => 0u8,
        SimulationLevel::L1 => 1,
        SimulationLevel::L2 => 2,
        SimulationLevel::L3 => 3,
    });

    let ai_state = ecs.ai_states.get(&id).map(|_| 0u8);

    let life_info = ecs.get_life_info(id).map(|li| SerLifeInfo {
        age: li.age,
        max_age: li.max_age,
        last_mate_day: li.last_mate_day,
        mate_cooldown_days: li.mate_cooldown_days,
    });

    let emotions = ecs.get_emotions(id).map(|e| SerEmotions {
        anger: e.anger,
        grief: e.grief,
        joy: e.joy,
        fear: e.fear,
    });

    let flammable = ecs.flammables.get(&id).map(|f| match f.material {
        FlammableMaterial::Wood => 0u8,
        FlammableMaterial::Cloth => 1,
        FlammableMaterial::Thatch => 2,
        FlammableMaterial::Stone => 3,
    });

    let name = ecs.get_name(id).map(|n| n.0.clone());

    let inventory = ecs.inventories.get(&id).map(|inv| SerInventory {
        items: inv
            .items
            .iter()
            .map(|it| SerItem {
                name: it.name.clone(),
                value: it.value,
            })
            .collect(),
    });

    let equipment = ecs.equipment.get(&id).map(|eq| SerEquipment {
        weapon_condition: eq.weapon_condition,
        armor_condition: eq.armor_condition,
        medkits: eq.medkits,
        food_rations: eq.food_rations,
        ammo: eq.ammo,
        total_weight: eq.total_weight,
    });

    let faction_membership = ecs.faction_memberships.get(&id).map(|fm| {
        use crate::gameplay::factions::Faction;
        let faction_id = match fm.faction {
            Faction::Loners => 0u8,
            Faction::Duty => 1,
            Faction::Freedom => 2,
            Faction::Bandits => 3,
            Faction::Military => 4,
            Faction::Scientists => 5,
            Faction::Traders => 6,
        };
        SerFactionMembership {
            faction: faction_id,
            standing: fm.standing,
        }
    });

    EntitySnapshot {
        id,
        persistent_id,
        transform,
        kind,
        name,
        personal_needs,
        social_needs,
        ecosystem_needs,
        npc_traits,
        monster_traits,
        npc_economy,
        sim_level,
        ai_state,
        life_info,
        emotions,
        flammable,
        inventory,
        equipment,
        faction_membership,
    }
}

pub fn save_world(path: &Path, snapshot: &WorldSnapshot) -> Result<(), String> {
    use crate::memory::atomic_saved::atomic_save;
    
    let encoded = bincode::serialize(snapshot).map_err(|e| format!("serialize: {e}"))?;
    atomic_save(path, &encoded).map_err(|e| format!("atomic write: {e}"))?;
    tracing::info!(
        "world saved to {} ({} entities)",
        path.display(),
        snapshot.entities.len()
    );
    Ok(())
}

pub fn load_world(path: &Path) -> Result<WorldSnapshot, String> {
    let data = fs::read(path).map_err(|e| format!("read: {e}"))?;
    let snapshot: WorldSnapshot =
        bincode::deserialize(&data).map_err(|e| format!("deserialize: {e}"))?;
    tracing::info!(
        "world loaded from {} ({} entities)",
        path.display(),
        snapshot.entities.len()
    );
    Ok(snapshot)
}

pub fn save_chunk(dir: &Path, coord: ChunkCoord, entities: &[EntitySnapshot]) -> Result<(), String> {
    use crate::memory::atomic_saved::atomic_save_fast;
    
    fs::create_dir_all(dir).map_err(|e| format!("mkdir: {e}"))?;
    let file = dir.join(format!("{}_{}.bin", coord.x, coord.z));
    let encoded = bincode::serialize(entities).map_err(|e| format!("serialize chunk: {e}"))?;
    atomic_save_fast(&file, &encoded).map_err(|e| format!("atomic write chunk: {e}"))?;
    Ok(())
}

pub fn load_chunk(dir: &Path, coord: ChunkCoord) -> Result<Vec<EntitySnapshot>, String> {
    let file = dir.join(format!("{}_{}.bin", coord.x, coord.z));
    if !file.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read(&file).map_err(|e| format!("read chunk: {e}"))?;
    let entities: Vec<EntitySnapshot> =
        bincode::deserialize(&data).map_err(|e| format!("deserialize chunk: {e}"))?;
    Ok(entities)
}
