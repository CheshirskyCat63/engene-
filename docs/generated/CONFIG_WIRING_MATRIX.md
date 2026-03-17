# ENGENE Config Wiring Matrix

> **Дата создания:** 2025-01-21
> **Последнее обновление:** 2025-01-21
> **Canonical config count:** 16 files

---

## Summary

| Metric | Value |
|--------|-------|
| Total config files | 16 |
| Loaded in GameConfig | 16 |
| With hardcoded duplicates | ~8 |
| Required configs | 16 |

---

## Config Files Matrix

| # | File | Exists | Parsed | Bound in GameConfig | Runtime Consumer | Required | Fallback Allowed | Hardcoded Duplicate |
|---|------|--------|--------|---------------------|------------------|----------|------------------|---------------------|
| 1 | `biomes.ron` | ✅ | ✅ | ✅ | `world/biome.rs` | Yes | No | Partial |
| 2 | `economy.ron` | ✅ | ✅ | ✅ | `economy/*.rs` | Yes | No | Yes |
| 3 | `food_chain.ron` | ✅ | ✅ | ✅ | `ecosystem/*.rs` | Yes | No | Yes |
| 4 | `goals.ron` | ✅ | ✅ | ✅ | `ai/goal.rs` | Yes | No | No |
| 5 | `jobs.ron` | ✅ | ✅ | ✅ | `economy/jobs.rs` | Yes | No | Yes |
| 6 | `materials.ron` | ✅ | ✅ | ✅ | `physics/material.rs` | Yes | No | Partial |
| 7 | `material_bridge.ron` | ✅ | ✅ | ✅ | `graphics/material.rs` | Yes | No | No |
| 8 | `perception.ron` | ✅ | ✅ | ✅ | `ai/perception.rs` | Yes | No | Yes |
| 9 | `population.ron` | ✅ | ✅ | ✅ | `ai/reproduction.rs` | Yes | No | **Yes** |
| 10 | `rules.ron` | ✅ | ✅ | ✅ | `core/rules.rs` | Yes | No | No |
| 11 | `seasons.ron` | ✅ | ✅ | ✅ | `world/season.rs` | Yes | No | No |
| 12 | `simulation.ron` | ✅ | ✅ | ✅ | `simulation/*.rs` | Yes | No | **Yes** |
| 13 | `species.ron` | ✅ | ✅ | ✅ | `ecosystem/species.rs` | Yes | No | No |
| 14 | `surfaces.ron` | ✅ | ✅ | ✅ | `world/surface.rs` | Yes | No | No |
| 15 | `tactics.ron` | ✅ | ✅ | ✅ | `ai/combat_tactics/*.rs` | Yes | No | No |
| 16 | `weapons.ron` | ✅ | ✅ | ✅ | `physics/ballistics.rs` | Yes | No | No |

---

## Hardcoded Duplicates (Critical)

### population.ron vs Code

| Parameter | Config Value | Code Location | Code Value |
|-----------|--------------|---------------|------------|
| `max_npcs` | 30 | `ai/reproduction.rs` | 30 |
| `max_wolves` | 40 | `ai/reproduction.rs` | 40 |
| `max_boars` | 40 | `ai/reproduction.rs` | 40 |
| `max_bloodsuckers` | 25 | `ai/reproduction.rs` | 25 |

**Status:** Values match, but code should read from config!

---

### simulation.ron vs Code

| Parameter | Config Value | Code Location | Code Value |
|-----------|--------------|---------------|------------|
| `l0_radius` | 300.0 | `simulation/simulation_level.rs` | 300.0 |
| `l1_radius` | 5000.0 | `simulation/simulation_level.rs` | 5000.0 |
| `l2_radius` | 50000.0 | `simulation/simulation_level.rs` | 50000.0 |
| `l0_tick_interval` | 1 | `simulation/simulation_level.rs` | 1 |
| `l1_tick_interval` | 12 | `simulation/simulation_level.rs` | 12 |
| `l2_tick_interval` | 60 | `simulation/simulation_level.rs` | 60 |

**Status:** Values match, but code should read from config!

---

### economy.ron vs Code

| Parameter | Config File | Code Location | Status |
|-----------|-------------|---------------|--------|
| `monthly_required` | economy.ron | `economy/monthly.rs` | Duplicate |
| `desperation_threshold` | economy.ron | `ai/desire.rs` | Duplicate |

---

### perception.ron vs Code

| Parameter | Config File | Code Location | Status |
|-----------|-------------|---------------|--------|
| `hunt_radius` | perception.ron | `ai/perception.rs` | Duplicate |
| `fear_radius` | perception.ron | `ai/perception.rs` | Duplicate |
| `ally_radius` | perception.ron | `ai/perception.rs` | Duplicate |

---

## Config Loading Flow

```
GameConfig::load_from_dir("game/data")
│
├── perception.ron  → PerceptionConfig
├── population.ron  → PopulationConfig
├── economy.ron     → EconomyConfig
├── simulation.ron  → SimulationConfig
├── jobs.ron        → HashMap<String, JobConfig>
├── goals.ron       → HashMap<String, GoalConfig>
├── biomes.ron      → HashMap<String, BiomeConfig>
├── seasons.ron     → HashMap<String, SeasonConfig>
├── materials.ron   → HashMap<String, MaterialConfig>
├── food_chain.ron  → FoodChainConfig
├── species.ron     → SpeciesConfig
├── tactics.ron     → HashMap<String, TacticsConfig>
├── rules.ron       → RulesData
├── weapons.ron     → HashMap<String, WeaponConfig>
├── surfaces.ron    → SurfacesConfig
└── material_bridge.ron → MaterialBridgeConfig
```

---

## Action Items

### P0 - Remove Hardcoded Duplicates

1. **population.ron** → `ai/reproduction.rs` must use `GameConfig::population`
2. **simulation.ron** → `simulation/simulation_level.rs` must use `GameConfig::simulation`
3. **economy.ron** → All economy values from config
4. **perception.ron** → All perception radii from config

### P1 - Verify Config Usage

1. Audit each system to confirm it reads from GameConfig
2. Remove any remaining hardcoded defaults
3. Add config validation in doctor

---

## Doctor Integration

Doctor should verify:
- [ ] All 16 config files exist
- [ ] All 16 config files parse successfully
- [ ] No hardcoded values in runtime code
- [ ] GameConfig is accessible to all systems

---

## История изменений

| Дата | Изменение |
|------|-----------|
| 2025-01-21 | Создана матрица, идентифицированы hardcoded duplicates |
