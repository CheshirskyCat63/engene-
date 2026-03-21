# Documentation Consistency Matrix

**Last Updated:** 2026-03-21 (docs fully synchronized - terminology unified)

## Status Legend
- **REMOVED** - Physically deleted from repository
- **KEPT** - Physically exists, actively used
- **KEPT TEMP** - Physically exists, temporary status

## Engine Core Modules

| Module | Final Status | Physical Reality | Reflected in WORKSPACE_STATUS | Reflected in ARCHITECTURE_TRUTH | Notes |
|--------|-------------|------------------|------------------------------|--------------------------------|-------|
| data_policy, determinism_policy, deterministic_merge, failure_taxonomy, time | **KEPT** | ✅ Physically exist | ✅ | ✅ | Core modules - only 5 remaining |
| All other pre-cleanup modules (budget_registry, game_config, etc.) | **REMOVED** | ✅ Physically deleted | ✅ | ✅ | 21 files deleted in cleanup |

## Engine Runtime Modules

| Module | Final Status | Physical Reality | Reflected in WORKSPACE_STATUS | Reflected in ARCHITECTURE_TRUTH | Notes |
|--------|-------------|------------------|------------------------------|--------------------------------|-------|
| async_services, job_graph, phase_runner, jobs/, streaming/ | **REMOVED** | ✅ Physically deleted | ✅ | ✅ | Runtime assembly residue removed |
| perf/, wiring/ | **KEPT** | ✅ Physically exist | ✅ | ✅ | Test consumers |
| assembly/ | **KEPT** | ✅ Physically exist | ✅ | ✅ | Internal bootstrap glue |

## Engine World Modules

| Module | Final Status | Physical Reality | Reflected in WORKSPACE_STATUS | Reflected in ARCHITECTURE_TRUTH | Notes |
|--------|-------------|------------------|------------------------------|--------------------------------|-------|
| population, material_bridge, terrain_truth, spatial_index, hierarchical_spatial | **REMOVED** | ✅ Physically deleted | ✅ | ✅ | World data residue removed |
| heightmap, material_truth | **KEPT TEMP** | ✅ Physically exist | ✅ | ✅ | Test/engine_core consumers |
| events/, fields.rs, resources.rs | **KEPT** | ✅ Physically exist | ✅ | ✅ | Support systems |

## Engine Render Modules

| Module | Final Status | Physical Reality | Reflected in WORKSPACE_STATUS | Reflected in ARCHITECTURE_TRUTH | Notes |
|--------|-------------|------------------|------------------------------|--------------------------------|-------|
| decal_system, decals, model_loader | **REMOVED** | ✅ Physically deleted | ✅ | ✅ | Render residue removed |
| terrain, vegetation | **KEPT** | ✅ Physically exist | ✅ | ✅ | Potentially used |
| destruction_occlusion, gore_mesh | **KEPT TEMP** | ✅ Physically exist | ✅ | ✅ | Active consumers |

## Engine ECS Modules

| Module | Final Status | Physical Reality | Reflected in WORKSPACE_STATUS | Reflected in ARCHITECTURE_TRUTH | Notes |
|--------|-------------|------------------|------------------------------|--------------------------------|-------|
| component_registry | **REMOVED** | ✅ Physically deleted | ✅ | ✅ | ECS residue removed |

## Consistency Status: ✅ FULLY SYNCHRONIZED

**Achievements:**
1. **Terminology Unified** - Only REMOVED/KEPT/KEPT TEMP used across all docs
2. **QUARANTINE terminology eliminated** - Replaced with accurate physical reality
3. **All statuses reflect actual file system state**
4. **Both canonical documents use identical terminology**
5. **Complete matrix created for verification**

**No contradictions remain between WORKSPACE_STATUS.md and ARCHITECTURE_TRUTH.md**

---

**Verification:** All entries in this matrix reflect the current physical state of the repository as of the cleanup/core-physical-delete branch.
