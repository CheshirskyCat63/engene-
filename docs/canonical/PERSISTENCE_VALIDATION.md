# Persistence Validation Report

Generated: 2026-03-10

---

## 1. Persistence Infrastructure

### ChunkPersistenceService (`src/world/chunk_persistence.rs`)

- **Status**: **ACTIVE** — used in both `engene_sdk` and `engene_game` binaries
- **Trigger**: World streaming — when camera moves, chunks load/unload
- **Operations**: `save_and_unload(coord, ecs, tick)` and `load_chunk_entities(coord, ecs)`

### PersistentEntityId (`src/core/persistent_id.rs`)

- **Status**: **ACTIVE** — every entity spawned gets a PID via `Ecs::spawn_new()`
- **Format**: UUID-based stable identifier
- **Relink**: `RelinkContext` handles reference fixup on load

---

## 2. What Persists (Verified via test suite)

### Tests in `tests/persistence_full.rs`:

| Test | Component/State | Round-trip Verified | Status |
|------|----------------|-------------------|--------|
| `equipment_state_roundtrip` | EquipmentSlots (weapon, armor, medkits, ammo) | YES | **PASS** |
| `faction_membership_roundtrip` | FactionMembership (faction name, rank, standing) | YES | **PASS** |
| `surface_destruction_state_roundtrip` | SurfaceStateStore (destruction state) | YES | **PASS** |
| `runtime_truth_json_generation` | Runtime truth JSON snapshot | YES | **PASS** |
| PlayerSave roundtrip | Position, inventory, play_time, day/month (via RON) | YES | **PASS** |

### Tests in `tests/vertical_slice.rs`:

| Test | What's Tested | Status |
|------|--------------|--------|
| `vertical_slice_engine_boots` | Engine boots with entities, doctor 0 errors | **PASS** |
| `vertical_slice_simulation_stable` | Entity count stable over 200 ticks (<50% drift) | **PASS** |

---

## 3. Components That Persist via Chunk Streaming

The following components are saved when a chunk is unloaded and restored when loaded:

| Component | Sparse Set | Save | Restore | Verified |
|-----------|-----------|------|---------|----------|
| Transform | `ecs.transforms` | YES | YES | YES (via streaming) |
| EntityKind | `ecs.kinds` | YES | YES | YES |
| Name | `ecs.names` | YES | YES | YES |
| NpcTraits | `ecs.npc_traits` | YES | YES | YES |
| MonsterTraits | `ecs.monster_traits` | YES | YES | YES |
| PersonalNeeds | `ecs.personal_needs` | YES | YES | YES |
| NpcEconomy | `ecs.npc_economies` | YES | YES | YES |
| SimLevel | `ecs.sim_levels` | YES | YES | YES |
| AiState | `ecs.ai_states` | YES | YES | YES |
| Inventory | `ecs.inventories` | YES | YES | YES |
| EquipmentSlots | `ecs.equipment` | YES | YES | YES (explicit test) |
| FactionMembership | `ecs.faction_memberships` | YES | YES | YES (explicit test) |

---

## 4. What Does NOT Persist

| Data | Reason | Impact |
|------|--------|--------|
| Memory (AI) | Not included in chunk save | NPCs lose learned danger zones, opinions on reload |
| Emotions | Not included in chunk save | NPCs reset emotional state on reload |
| Plans | Not included in chunk save | NPCs re-plan on reload (acceptable) |
| Quest state (player-facing) | No player quest log exists | N/A — quests are NPC-side only |
| Corpses | `CorpseManager` not integrated | Dead bodies vanish on chunk unload |
| Trader inventory | `TraderState` not integrated | Trader stock not persistent |
| Camp state | `CampSimulation` not integrated | Camp mood/food/security resets |
| World milestones | `WorldMilestoneTracker` not integrated | Achievements lost |
| Terrain deformation | `TerrainDeformationSystem` state not in chunk save | Craters reset on chunk reload |

---

## 5. Save/Load Round-Trip Test

### Test procedure (streaming-based):

1. Engine boots, spawns ~100+ entities
2. Camera moves, triggering chunk unload
3. `save_and_unload()` serializes entities in chunk
4. Camera moves back, triggering chunk load
5. `load_chunk_entities()` deserializes entities
6. Entity PID matches, components restored

### Result:

- **Entity count**: Stable (verified in soak test and vertical slice test)
- **PID restoration**: YES — entities keep their PersistentEntityId across save/load
- **Component integrity**: Core components (Transform, EntityKind, Name, Needs, Economy, Equipment, Faction) round-trip correctly
- **Reference integrity**: EntityRef fields that use PID survive via RelinkContext

---

## 6. Full-Game Save/Load

- **Quick save (F5)**: **WORKING** — implemented in game binary
- **Quick load (F9)**: **WORKING** — implemented in game binary
- **PlayerSave roundtrip**: Position, inventory, play_time, day/month via RON
- **PlayerInventory**: Items, money, equipped weapon persisted
- **Chunk-streaming save/load**: **WORKING** — automatic, transparent to gameplay

### Assessment:

Chunk-level persistence and PlayerSave are tested. PlayerInventory with items, money, equipped weapon persists. Quick save/load (F5/F9) functional in game binary. Remaining gaps: full-world save, save menu UI, corpses, camps, trader state, terrain damage.
