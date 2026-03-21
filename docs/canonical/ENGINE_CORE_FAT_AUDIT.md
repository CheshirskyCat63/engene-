# ENGINE_CORE AUDIT - FAT vs TINY

## CURRENT MODULES (OVERLOADED)

### ❌ SUSPECTED FAT/UNNECESSARY
- `budget_registry` - Budget policy, not core
- `build_manifest` - Build system, not core
- `config` - Game config, not core
- `determinism_audit` - Auditing, not core
- `dirty_set` - Utility, not core
- `game_config` - Game config, not core
- `integration_matrix` - Integration testing, not core
- `metrics_registry` - Metrics, not core
- `mutation_policy` - Policy, not core
- `ownership_map` - Ownership tracking, not core
- `plugin` - Plugin system, not core
- `profiler` - Profiling, not core
- `quality_governor` - Quality policy, not core
- `registry` - Registry system, not core
- `replay` - Replay system, not core
- `runtime_config` - Runtime config, not core
- `runtime_manifest` - Runtime manifest, not core
- `serialization` - Serialization, not core

### ✅ REAL CORE (KEEP)
- `data_policy` - Data ownership contracts
- `determinism_policy` - Determinism guarantees
- `deterministic_merge` - Deterministic state merge
- `failure_taxonomy` - Failure classification
- `time` - Time management

## VERDICT: ENGINE_CORE IS 80% FAT

**Real core modules: 5/20 (25%)**
**Fat/suspect modules: 15/20 (75%)**

## RECOMMENDATION: TRIM TO TINY CORE

**KEEP in engine_core:**
- time
- data_policy  
- determinism_policy
- deterministic_merge
- failure_taxonomy

**MOVE OUT of engine_core:**
- build_manifest → engine_build (new crate)
- game_config → engine_config (new crate)  
- registry, plugin → engine_framework (new crate)
- profiler, metrics → engine_diagnostics (new crate)
- All other policy/utility modules

## CURRENT STATUS: ARCHITECTURAL COMPROMISE

engine_core currently violates "Law A - Core is tiny"
It's more like "engine_framework" than "engine_core"

This explains why engine feels "fat" - core is doing framework work.
