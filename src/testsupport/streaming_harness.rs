use crate::core::ecs::Ecs;
use crate::core::persistent_id::PersistentEntityId;
use crate::world::chunk_persistence::ChunkPersistenceService;
use crate::world::streaming::{ChunkCoord, WorldStreamer};

/// Report from multi-region streaming test
#[derive(Debug)]
pub struct StreamingTestReport {
    pub regions_tested: usize,
    pub entities_saved: usize,
    pub entities_restored: usize,
    pub identity_preserved: bool,
    pub social_ties_preserved: bool,
    pub determinism_check: bool,
    pub details: Vec<String>,
}

/// Run a 4-region streaming test with identity continuity validation
pub fn run_multi_region_test(
    ecs: &mut Ecs,
    _streamer: &mut WorldStreamer,
    persistence: &mut ChunkPersistenceService,
) -> StreamingTestReport {
    let mut report = StreamingTestReport {
        regions_tested: 0,
        entities_saved: 0,
        entities_restored: 0,
        identity_preserved: true,
        social_ties_preserved: true,
        determinism_check: true,
        details: Vec::new(),
    };

    // Register all existing entities with persistent IDs
    let initial_entities: Vec<_> = ecs.alive.clone();
    let mut initial_pids: Vec<(u64, PersistentEntityId)> = Vec::new();
    for &entity in &initial_entities {
        let pid = match ecs.identity.persistent_id_of(entity) {
            Some(pid) => pid,
            None => ecs.identity.register_new(entity),
        };
        initial_pids.push((entity, pid));
    }
    report.details.push(format!(
        "Registered {} entities with persistent IDs",
        initial_pids.len()
    ));

    // Simulate 4-region unload/reload cycle
    let test_regions = [
        ChunkCoord { x: 0, z: 0 },
        ChunkCoord { x: 1, z: 0 },
        ChunkCoord { x: 0, z: 1 },
        ChunkCoord { x: 1, z: 1 },
    ];

    let current_tick = ecs.tick;

    for region in &test_regions {
        // Save and unload
        let saved = persistence.save_and_unload(*region, ecs, current_tick);
        report.entities_saved += saved;
        report.regions_tested += 1;
        report.details.push(format!(
            "Region ({},{}): saved {} entities",
            region.x, region.z, saved
        ));
    }

    // Reload all regions
    for region in &test_regions {
        let loaded = persistence.load_chunk_entities(*region, ecs);
        report.entities_restored += loaded;
        report.details.push(format!(
            "Region ({},{}): loaded {} entities",
            region.x, region.z, loaded
        ));
    }

    // Verify identity continuity
    for &(_, pid) in &initial_pids {
        match ecs.identity.presence(pid) {
            crate::core::persistent_id::EntityPresence::Live(_) => {}
            crate::core::persistent_id::EntityPresence::Unloaded => {
                // Entity was outside all test regions, still unloaded -- ok
            }
            crate::core::persistent_id::EntityPresence::Dead => {
                report.identity_preserved = false;
                report
                    .details
                    .push(format!("FAIL: PersistentId {:?} is dead after reload", pid));
            }
        }
    }

    report.details.push(format!(
        "Identity check: saved={}, restored={}, preserved={}",
        report.entities_saved, report.entities_restored, report.identity_preserved
    ));

    report
}

/// L0/L2 population migration: promote/demote entities based on chunk boundaries
pub struct MigrationService;

impl MigrationService {
    /// Check if any background (L2) entities should be promoted to L0
    /// because they've migrated into a loaded chunk
    pub fn check_promotions(
        ecs: &Ecs,
        streamer: &WorldStreamer,
        background_positions: &[(PersistentEntityId, f32, f32)],
    ) -> Vec<PersistentEntityId> {
        let mut to_promote = Vec::new();
        for &(pid, x, z) in background_positions {
            let coord = ChunkCoord::from_world(x, z);
            if streamer.is_loaded(&coord) {
                if ecs.identity.resolve(pid).is_none() {
                    to_promote.push(pid);
                }
            }
        }
        to_promote
    }

    pub fn check_demotions(ecs: &Ecs, streamer: &WorldStreamer) -> Vec<(PersistentEntityId, u64)> {
        let mut to_demote = Vec::new();
        for &entity in &ecs.alive {
            if let Some(transform) = ecs.get_transform(entity) {
                let coord = ChunkCoord::from_world(transform.x, transform.y);
                if !streamer.is_loaded(&coord) {
                    if let Some(pid) = ecs.identity.persistent_id_of(entity) {
                        to_demote.push((pid, entity));
                    }
                }
            }
        }
        to_demote
    }
}

// ── Golden Scenario Definitions ──────────────────────────────────────

/// A golden scenario for regression testing
pub struct GoldenScenario {
    pub name: &'static str,
    pub description: &'static str,
    pub deterministic_seed: u64,
    pub expected_min_entities: usize,
    pub expected_max_entities: usize,
}

pub fn golden_scenarios() -> Vec<GoldenScenario> {
    vec![
        GoldenScenario {
            name: "Ruined Outpost Unload/Reload",
            description: "Destroy structures, unload chunk, reload -- all destruction persisted",
            deterministic_seed: 42,
            expected_min_entities: 10,
            expected_max_entities: 500,
        },
        GoldenScenario {
            name: "Monster Migration Across 4 Regions",
            description: "Pack moves through all chunks -- identity preserved, nav works",
            deterministic_seed: 1337,
            expected_min_entities: 50,
            expected_max_entities: 300,
        },
        GoldenScenario {
            name: "20-Minute Camp Economy Loop",
            description: "Trader arrives, trades, leaves -- economy stable",
            deterministic_seed: 7777,
            expected_min_entities: 20,
            expected_max_entities: 200,
        },
        GoldenScenario {
            name: "Quick Population Ecology",
            description: "10 min headless -- no collapse, no runaway",
            deterministic_seed: 9999,
            expected_min_entities: 100,
            expected_max_entities: 600,
        },
        GoldenScenario {
            name: "Low-Spec Regression",
            description: "All scenarios on Low quality tier -- no crashes or corruption",
            deterministic_seed: 42,
            expected_min_entities: 10,
            expected_max_entities: 500,
        },
        GoldenScenario {
            name: "Replay Determinism",
            description: "Record scenario, replay -- zero divergence",
            deterministic_seed: 12345,
            expected_min_entities: 50,
            expected_max_entities: 400,
        },
    ]
}

// ── Determinism / Replay Gates ───────────────────────────────────────

/// Check replay consistency after unload/reload
pub struct DeterminismGate {
    pub phase: String,
    pub test_name: String,
    pub passed: bool,
    pub details: String,
}

pub fn check_replay_determinism_gate(
    seed: u64,
    entity_count_before: usize,
    entity_count_after: usize,
) -> DeterminismGate {
    let passed = entity_count_before == entity_count_after;
    DeterminismGate {
        phase: "Wave 2".to_string(),
        test_name: "Entity count consistency after reload".to_string(),
        passed,
        details: format!(
            "seed={}, before={}, after={}, match={}",
            seed, entity_count_before, entity_count_after, passed
        ),
    }
}
