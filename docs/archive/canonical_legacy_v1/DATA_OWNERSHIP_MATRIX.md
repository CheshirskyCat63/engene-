# DATA_OWNERSHIP_MATRIX.md — State Ownership

| State | Source of Truth | Persisted? | Rebuilt on Load? | Editable in SDK? | Runtime Owner |
|-------|-----------------|------------|------------------|------------------|---------------|
| Entity Transform | ECS `transforms` | Yes | No | Yes | SimulationSystem |
| Entity Kind | ECS `kinds` | Yes | No | Yes (placement) | ECS init |
| NPC Traits | ECS `npc_traits` | Yes | No | No | StalkerPlugin init |
| Monster Traits | ECS `monster_traits` | Yes | No | No | StalkerPlugin init |
| Personal Needs | ECS `personal_needs` | Yes | No | No | AiSystem / WorldTick |
| Social Needs | ECS `social_needs` | Yes | No | No | AiSystem |
| AI State | ECS `ai_states` | Yes | No | No | AiSystem |
| Emotions | ECS `emotions` | Yes | No | No | AiSystem |
| Memory | ECS `memories` | Yes | No | No | AiSystem |
| Plan | ECS `plans` | No | Yes (re-decide) | No | AiSystem |
| Inventory | ECS `inventories` | Yes | No | Yes | EconomySystem |
| Equipment | ECS `equipment_slots` | Yes | No | Yes | EconomySystem |
| NPC Economy | ECS `npc_economies` | Yes | No | No | EconomySystem |
| Life Info | ECS `life_info` | Yes | No | No | WorldTickSystem |
| Quest State | QuestRegistry (resource) | Yes | No | Yes (quest board) | QuestSystem |
| Faction State | FactionRelations (resource) | Yes | No | Yes | QuestSystem / gameplay |
| Camp Mood | Vec<CampState> (resource) | Yes | No | No | CampSimulation |
| Trader Inventory | Via NpcEconomy + Inventory | Yes | No | Yes | EconomySystem |
| Group Refs | ECS social_needs | Yes | No | No | AiSystem |
| Corpse State | CorpseManager (resource) | Yes | No | No | DeathPipeline |
| Body Zone State | ECS via BodySystem | Yes | No | No | BodySystem |
| Surface State | SurfaceStateStore (resource) | Yes | No | No | WorldTickSystem |
| Terrain Deformation | TerrainDeformationSystem (resource) | Yes | No | No | Impact events |
| Destruction Topology | DestructionSystem (resource) | Yes | No | No | DestructionTickSystem |
| Nav Dirty State | NavDirtyTracker (resource) | No | Yes (rebuild) | No | NavDirtyTickSystem |
| Spatial Index | HierarchicalSpatialIndex | No | Yes (rebuild) | No | Main loop rebuild |
| Resource Grid | ResourceGrid (resource) | No | Yes (from biomes) | Yes (spawn zones) | WorldTickSystem |
| World Fields | WorldFields (resource) | No | Yes (from config) | Yes (anomaly placement) | Physics / AI |

## Rules

1. "Persisted = Yes" means chunk save/load must roundtrip this state
2. "Rebuilt on Load = Yes" means the state is derived and can be reconstructed
3. "Editable in SDK" means the SDK authoring tools can modify this in editor mode
4. "Runtime Owner" is the system that has write authority during simulation
5. No two systems should write the same state without explicit handoff via events/commands
