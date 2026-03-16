use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::ecs::{Ecs, Entity};
use crate::core::persistent_id::{EntityRef, PersistentEntityId, RelinkContext};
use crate::memory::save_chunks::{snapshot_entity, EntitySnapshot};
use crate::world::streaming::{ChunkCoord, CHUNK_SIZE};

/// Extended snapshot that includes persistent identity and additional state
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PersistentEntitySnapshot {
    pub persistent_id: u64, // PersistentEntityId.0
    pub snapshot: EntitySnapshot,
    pub social_ties: Vec<u64>, // PersistentEntityIds of known entities
    pub group_leader: Option<u64>, // PersistentEntityId of group leader
    pub group_members: Vec<u64>, // PersistentEntityIds of group members
    pub destruction_state: Option<ChunkDestructionState>,
}

/// Destruction state for objects in a chunk
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkDestructionState {
    pub damaged_nodes: Vec<(u32, f32)>, // (node_id, remaining_integrity)
    pub broken_links: Vec<(u32, u32)>,  // (node_a, node_b)
}

/// Surface state saved per chunk
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChunkSurfaceState {
    pub blood_marks: Vec<SurfaceMark>,
    pub burn_marks: Vec<SurfaceMark>,
    pub mud_marks: Vec<SurfaceMark>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SurfaceMark {
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub intensity: f32,
    pub age_seconds: f32,
}

/// Complete saved state for a chunk — Contract 4 schema versioning
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChunkSaveData {
    pub schema_version_chunk: u32,
    pub schema_version_entity: u32,
    pub coord: (i32, i32),
    pub entities: Vec<PersistentEntitySnapshot>,
    pub surface_state: ChunkSurfaceState,
    pub save_tick: u64,
    #[serde(default)]
    pub terrain_deformation_patches: Vec<TerrainPatch>,
    #[serde(default)]
    pub carcass_states: Vec<CarcassState>,
}

/// Terrain deformation data saved per chunk
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TerrainPatch {
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub depth: f32,
}

/// Carcass state saved per chunk
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CarcassState {
    pub persistent_id: u64,
    pub x: f32,
    pub z: f32,
    pub species: u8,
    pub age_seconds: f32,
}

/// Report generated after chunk load showing reference resolution status
#[derive(Clone, Debug)]
pub struct RelinkReport {
    pub chunk: ChunkCoord,
    pub entities_restored: usize,
    pub duplicates_skipped: usize,
    pub social_ties_resolved: usize,
    pub social_ties_dangling: usize,  // target is unloaded
    pub social_ties_dead: usize,       // target is dead
    pub details: Vec<String>,
}

impl RelinkReport {
    pub fn is_clean(&self) -> bool {
        self.duplicates_skipped == 0 && self.social_ties_dead == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "chunk ({},{}): {} restored, {} dupes, ties: {} ok / {} dangling / {} dead",
            self.chunk.x, self.chunk.z,
            self.entities_restored, self.duplicates_skipped,
            self.social_ties_resolved, self.social_ties_dangling, self.social_ties_dead
        )
    }
}

/// Service that manages chunk save/load with persistent identity
pub struct ChunkPersistenceService {
    save_dir: PathBuf,
    /// Which chunks have been saved (and when)
    saved_chunks: HashMap<(i32, i32), u64>,
}

impl ChunkPersistenceService {
    pub fn new(save_dir: impl Into<PathBuf>) -> Self {
        let dir = save_dir.into();
        let _ = std::fs::create_dir_all(&dir);
        Self {
            save_dir: dir,
            saved_chunks: HashMap::new(),
        }
    }

    /// Save all entities in a chunk before unloading.
    /// Uses ecs.identity directly -- no separate registry parameter needed.
    pub fn save_and_unload(
        &mut self,
        coord: ChunkCoord,
        ecs: &mut Ecs,
        current_tick: u64,
    ) -> usize {
        let chunk_size = CHUNK_SIZE;
        let chunk_min_x = coord.x as f32 * chunk_size;
        let chunk_max_x = chunk_min_x + chunk_size;
        let chunk_min_z = coord.z as f32 * chunk_size;
        let chunk_max_z = chunk_min_z + chunk_size;

        let mut snapshots = Vec::new();
        let mut entities_to_unload = Vec::new();

        let alive_copy: Vec<Entity> = ecs.alive.clone();
        for entity in alive_copy {
            if let Some(transform) = ecs.transforms.get(&entity) {
                let wx = transform.x;
                let wz = transform.y;
                if wx >= chunk_min_x && wx < chunk_max_x && wz >= chunk_min_z && wz < chunk_max_z {
                    let base_snapshot = snapshot_entity(entity, ecs);
                    let pid = match ecs.identity.persistent_id_of(entity) {
                        Some(pid) => pid,
                        None => ecs.identity.register_new(entity),
                    };

                    let social_ties: Vec<u64> = ecs
                        .memories
                        .get(&entity)
                        .map(|mem| {
                            mem.entities
                                .keys()
                                .map(|pid| pid.0)
                                .collect()
                        })
                        .unwrap_or_default();

                    let persistent_snapshot = PersistentEntitySnapshot {
                        persistent_id: pid.0,
                        snapshot: base_snapshot,
                        social_ties,
                        group_leader: None,
                        group_members: Vec::new(),
                        destruction_state: None,
                    };

                    snapshots.push(persistent_snapshot);
                    entities_to_unload.push(entity);
                }
            }
        }

        let count = snapshots.len();

        let save_data = ChunkSaveData {
            schema_version_chunk: crate::core::build_manifest::SCHEMA_VERSION_CHUNK,
            schema_version_entity: crate::core::build_manifest::SCHEMA_VERSION_ENTITY,
            coord: (coord.x, coord.z),
            entities: snapshots,
            surface_state: ChunkSurfaceState::default(),
            save_tick: current_tick,
            terrain_deformation_patches: Vec::new(),
            carcass_states: Vec::new(),
        };

        let file = self
            .save_dir
            .join(format!("chunk_{}_{}.bin", coord.x, coord.z));
        if let Ok(encoded) = bincode::serialize(&save_data) {
            // Use atomic save for crash safety
            use crate::memory::atomic_saved::atomic_save_fast;
            if let Err(e) = atomic_save_fast(&file, &encoded) {
                tracing::error!("atomic write chunk failed: {}", e);
            } else {
                self.saved_chunks.insert((coord.x, coord.z), current_tick);
                tracing::info!("chunk ({},{}) saved: {} entities", coord.x, coord.z, count);
            }
        }

        for entity in &entities_to_unload {
            ecs.unload_entity(*entity);
        }

        count
    }

    /// Load entities from a saved chunk back into ECS.
    /// Uses ecs.spawn_restored() -- no separate registry parameter needed.
    pub fn load_chunk_entities(
        &mut self,
        coord: ChunkCoord,
        ecs: &mut Ecs,
    ) -> usize {
        let file = self
            .save_dir
            .join(format!("chunk_{}_{}.bin", coord.x, coord.z));
        if !file.exists() {
            return 0;
        }

        let data = match std::fs::read(&file) {
            Ok(d) => d,
            Err(_) => return 0,
        };

        let save_data: ChunkSaveData = match bincode::deserialize(&data) {
            Ok(d) => d,
            Err(_) => return 0,
        };

        let mut count = 0;
        for ps in &save_data.entities {
            let pid = PersistentEntityId(ps.persistent_id);
            let entity = match ecs.spawn_restored(pid) {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("chunk load: {e} — skipping duplicate entity");
                    continue;
                }
            };

            restore_entity_from_snapshot(entity, &ps.snapshot, ecs);
            count += 1;
        }

        ecs.rebuild_spatial();

        tracing::info!(
            "chunk ({},{}) loaded: {} entities",
            coord.x, coord.z, count
        );

        count
    }

    /// Load chunk and produce a detailed relink report
    pub fn load_chunk_with_report(
        &mut self,
        coord: ChunkCoord,
        ecs: &mut Ecs,
    ) -> RelinkReport {
        let mut report = RelinkReport {
            chunk: coord,
            entities_restored: 0,
            duplicates_skipped: 0,
            social_ties_resolved: 0,
            social_ties_dangling: 0,
            social_ties_dead: 0,
            details: Vec::new(),
        };

        let file = self
            .save_dir
            .join(format!("chunk_{}_{}.bin", coord.x, coord.z));
        if !file.exists() {
            report.details.push("no save file found".to_string());
            return report;
        }

        let data = match std::fs::read(&file) {
            Ok(d) => d,
            Err(e) => {
                report.details.push(format!("read error: {e}"));
                return report;
            }
        };

        let save_data: ChunkSaveData = match bincode::deserialize(&data) {
            Ok(d) => d,
            Err(e) => {
                report.details.push(format!("deserialize error: {e}"));
                return report;
            }
        };

        for ps in &save_data.entities {
            let pid = PersistentEntityId(ps.persistent_id);
            let entity = match ecs.spawn_restored(pid) {
                Ok(e) => e,
                Err(e) => {
                    report.duplicates_skipped += 1;
                    report.details.push(format!("duplicate: {e}"));
                    continue;
                }
            };

            restore_entity_from_snapshot(entity, &ps.snapshot, ecs);
            report.entities_restored += 1;
        }

        // Use RelinkContext to resolve cross-entity references after all entities are spawned
        let relink = RelinkContext { registry: &ecs.identity };
        for ps in &save_data.entities {
            for &tie_pid_raw in &ps.social_ties {
                let entity_ref = EntityRef::new(PersistentEntityId(tie_pid_raw));
                if entity_ref.resolve(&ecs.identity).is_some() {
                    report.social_ties_resolved += 1;
                } else if entity_ref.is_unloaded(&ecs.identity) {
                    report.social_ties_dangling += 1;
                } else if entity_ref.is_dead(&ecs.identity) {
                    report.social_ties_dead += 1;
                }
            }

            if let Some(leader_pid) = ps.group_leader {
                let _resolved = relink.resolve_persistent(PersistentEntityId(leader_pid));
            }
            for &member_pid in &ps.group_members {
                let _resolved = relink.resolve_persistent(PersistentEntityId(member_pid));
            }
        }

        ecs.rebuild_spatial();
        report
    }

    /// Check if a chunk has saved data
    pub fn has_save(&self, coord: ChunkCoord) -> bool {
        let file = self
            .save_dir
            .join(format!("chunk_{}_{}.bin", coord.x, coord.z));
        file.exists()
    }

    pub fn saved_chunk_count(&self) -> usize {
        self.saved_chunks.len()
    }
}

/// Restore an entity's components from a snapshot
fn restore_entity_from_snapshot(entity: Entity, snap: &EntitySnapshot, ecs: &mut Ecs) {
    use crate::world::components::*;

    if let Some(ref t) = snap.transform {
        ecs.transforms.insert(
            entity,
            Transform {
                x: t.x,
                y: t.y,
                cell_x: t.cell_x,
                cell_y: t.cell_y,
            },
        );
    }

    if let Some(ref k) = snap.kind {
        let kind = if k.is_npc {
            EntityKind::Npc
        } else {
            match k.species {
                Some(0) => EntityKind::Monster(MonsterSpecies::Wolf),
                Some(1) => EntityKind::Monster(MonsterSpecies::Boar),
                Some(2) => EntityKind::Monster(MonsterSpecies::Bloodsucker),
                _ => EntityKind::Monster(MonsterSpecies::Wolf),
            }
        };
        ecs.kinds.insert(entity, kind);
    }

    if let Some(ref pn) = snap.personal_needs {
        ecs.personal_needs.insert(
            entity,
            PersonalNeeds {
                hunger: pn.hunger,
                thirst: pn.thirst,
                sleep: pn.sleep,
                health: pn.health,
                energy: pn.energy,
                fear: pn.fear,
                curiosity: 0.3,
                ambitions: 0.4,
                discomfort: 0.1,
            },
        );
    }

    if let Some(ref sn) = snap.social_needs {
        ecs.social_needs.insert(
            entity,
            SocialNeeds {
                family: sn.family,
                money: sn.money,
                reputation: sn.reputation,
                friendship: sn.friendship,
                ..Default::default()
            },
        );
    }

    if let Some(ref en) = snap.ecosystem_needs {
        ecs.ecosystem_needs.insert(
            entity,
            EcosystemNeeds {
                hunting: en.hunting,
                territory_control: en.territory_control,
                pack_following: en.pack_following,
                migration_urge: en.migration_urge,
                predator_avoidance: 0.3,
                food_chain_position: 0.5,
                resource_competition: 0.4,
                shelter_seeking: 0.3,
                world_event_reaction: 0.3,
                prey_selection: 0.5,
            },
        );
    }

    if let Some(ref nt) = snap.npc_traits {
        ecs.npc_traits.insert(
            entity,
            NpcTraits {
                bravery: nt.bravery,
                aggressiveness: nt.aggressiveness,
                work_ethic: nt.work_ethic,
                curiosity: nt.curiosity,
                honesty: nt.honesty,
                sociality: nt.sociality,
                autonomy: 0.5,
                materialism: 0.4,
                risk_tolerance: 0.4,
                stress_resistance: 0.5,
            },
        );
    }

    if let Some(ref mt) = snap.monster_traits {
        ecs.monster_traits.insert(
            entity,
            MonsterTraits {
                aggressiveness: mt.aggressiveness,
                caution: mt.caution,
                territoriality: mt.territoriality,
                bravery: mt.bravery,
                pack_mentality: mt.pack_mentality,
                energy_level: 0.6,
                hoarding: 0.3,
                curiosity: 0.3,
                adaptability: 0.5,
                stress_tolerance: 0.5,
            },
        );
    }

    if let Some(ref e) = snap.npc_economy {
        ecs.npc_economies.insert(
            entity,
            NpcEconomy {
                money: e.money,
                monthly_required: e.monthly_required,
                job: match e.job {
                    0 => Job::ArtifactHunter,
                    1 => Job::Guard,
                    2 => Job::Trader,
                    3 => Job::Bandit,
                    5 => Job::Hunter,
                    6 => Job::Scavenger,
                    7 => Job::Courier,
                    8 => Job::Resident,
                    _ => Job::Unemployed,
                },
                desperation: e.desperation,
            },
        );
    }

    if let Some(ref li) = snap.life_info {
        ecs.life_info.insert(
            entity,
            LifeInfo {
                age: li.age,
                max_age: li.max_age,
                last_mate_day: li.last_mate_day,
                mate_cooldown_days: li.mate_cooldown_days,
            },
        );
    }

    if let Some(ref em) = snap.emotions {
        ecs.emotions.insert(
            entity,
            crate::ai::emotions::Emotions {
                anger: em.anger,
                grief: em.grief,
                joy: em.joy,
                fear: em.fear,
                disgust: 0.0,
                surprise: 0.0,
                longing: 0.0,
            },
        );
    }

    if let Some(ref name) = snap.name {
        ecs.names.insert(entity, Name(name.clone()));
    }

    if let Some(ref inv) = snap.inventory {
        ecs.inventories.insert(
            entity,
            Inventory {
                items: inv
                    .items
                    .iter()
                    .map(|it| Item {
                        name: it.name.clone(),
                        value: it.value,
                    })
                    .collect(),
            },
        );
    }

    if let Some(ref eq) = snap.equipment {
        ecs.equipment.insert(
            entity,
            EquipmentSlots {
                weapon_condition: eq.weapon_condition,
                armor_condition: eq.armor_condition,
                medkits: eq.medkits,
                food_rations: eq.food_rations,
                ammo: eq.ammo,
                total_weight: eq.total_weight,
            },
        );
    }

    if let Some(ref fm) = snap.faction_membership {
        use crate::gameplay::factions::Faction;
        let faction = match fm.faction {
            0 => Faction::Loners,
            1 => Faction::Duty,
            2 => Faction::Freedom,
            3 => Faction::Bandits,
            4 => Faction::Military,
            5 => Faction::Scientists,
            6 => Faction::Traders,
            _ => Faction::Loners,
        };
        ecs.faction_memberships.insert(
            entity,
            FactionMembership {
                faction,
                standing: fm.standing,
            },
        );
    }
}
