# TEST MATRIX CURRENT - ACTUAL STATE

## Real Test Inventory

This document provides the **actual** current state of tests, not aspirational targets.

| File | Lines | Owner Domain | Lane | Test Type | Speed | Risk Level | Status | Notes |
|------|-------|-------------|------|-----------|------|-----------|---------|-------|
| **MEGASUITES (>500 lines)** |
| `sdk_editor_gui.rs` | 1,296 | **MEGA** | None | Mixed | Mixed | High | **SPLIT IN PROGRESS** | Editor GUI, console, inspector, tools |
| `e2e_regression.rs` | 1,005 | **MEGA** | None | Scenario | Heavy | High | **SPLIT NEEDED** | End-to-end regression tests |
| `perf_lowspec_mt.rs` | 741 | **MEGA** | Perf | Performance | Heavy | High | **SPLIT NEEDED** | Low-spec multithreading performance |
| `engine_infrastructure.rs` | 508 | **MEGA** | None | Integration | Medium | Medium | **SPLIT NEEDED** | Infrastructure tests |
| **SPECIALIZED SUITES (<500 lines)** |
| `editor_safe_mode_contracts.rs` | 479 | Tools | tools | Contract | Fast | Medium | **KEEP** | Editor safety, panic recovery |
| `camera_contracts.rs` | 449 | Graphics | render_audio_tools | Contract | Medium | Medium | **KEEP** | Camera systems, viewport |
| `rendering_pipeline_contracts.rs` | 447 | Graphics | render_audio_tools | Contract | Medium | Medium | **KEEP** | Rendering pipeline, LOD |
| `save_load_torture.rs` | 441 | World | None | Scenario | Heavy | High | **SPLIT NEEDED** | Save/load torture tests |
| `world_persistence_contracts.rs` | 437 | World | contracts | Contract | Medium | Medium | **KEEP** | World persistence contracts |
| `ecs_performance_contracts.rs` | 436 | ECS | ecs | Contract | Medium | Medium | **KEEP** | ECS performance contracts |
| `determinism_and_sdk.rs` | 408 | SDK | None | Integration | Medium | Medium | **SPLIT NEEDED** | SDK contracts, determinism |
| `ecs_authority_contracts.rs` | 390 | ECS | ecs | Contract | Fast | Medium | **KEEP** | ECS authority contracts |
| `query_contracts.rs` | 374 | ECS | ecs | Contract | Medium | Medium | **KEEP** | Query contracts |
| `ecs_lifecycle_contracts.rs` | 372 | ECS | ecs | Contract | Fast | Medium | **KEEP** | ECS lifecycle contracts |
| `entity_lifecycle_contracts.rs` | 369 | ECS | ecs | Contract | Fast | Medium | **KEEP** | Entity lifecycle contracts |
| `architecture_validation.rs` | 317 | Architecture | smoke | Meta | Fast | Medium | **KEEP** | Architecture validation meta-tests |
| `runtime_profile_and_quality_contracts.rs` | 305 | Runtime | contracts | Contract | Medium | Medium | **KEEP** | Runtime profile & quality |
| `physics_core_boundary_contracts.rs` | 283 | Physics | None | Contract | Medium | Medium | **KEEP** | Physics boundary contracts |
| `identity_and_authority_contracts.rs` | 282 | Core | contracts | Contract | Fast | Medium | **KEEP** | Identity & authority contracts |
| `event_bus_contracts.rs` | 257 | Events | contracts | Contract | Fast | Medium | **KEEP** | Event bus contracts |
| `entrypoint_and_operator_truth.rs` | 249 | Apps | smoke | Contract | Fast | Medium | **KEEP** | Entry point contracts |
| `architectural_gates.rs` | 248 | Architecture | smoke | Contract | Fast | Medium | **KEEP** | Ownership enforcement |
| `vertical_slice.rs` | 210 | Core | None | Integration | Medium | Medium | **KEEP** | Vertical slice tests |
| `ci_surface_contracts.rs` | 205 | CI | smoke | Contract | Fast | Medium | **KEEP** | CI surface contracts |
| `core_command_and_access_contracts.rs` | 203 | Core | contracts | Contract | Fast | Medium | **KEEP** | Command & access contracts |
| `runtime_phase_contracts.rs` | 192 | Runtime | contracts | Contract | Medium | Medium | **KEEP** | Runtime phase contracts |
| **SMALL SUITES (<200 lines)** |
| `wiring_boundary_contracts.rs` | 164 | Runtime | contracts | Contract | Fast | Medium | **KEEP** | Wiring boundary contracts |
| `production_candidate.rs` | 142 | Release | smoke | Integration | Medium | High | **KEEP** | Release readiness checks |
| `physics_bootstrap_contracts.rs` | 130 | Physics | contracts | Contract | Medium | Medium | **KEEP** | Physics bootstrap contracts |
| `quest_and_faction.rs` | 119 | Game | None | Integration | Medium | Low | **MOVE** | Quest and faction tests |
| `engine_contracts.rs` | 112 | Architecture | smoke | Aggregator | Fast | Medium | **KEEP** | Test aggregator |
| `spatial_dirty_contracts.rs` | 104 | World | contracts | Contract | Fast | Medium | **KEEP** | Spatial update contracts |
| `tools_runtime_purity.rs` | 87 | Tools | None | Contract | Fast | Medium | **MOVE** | Tools isolation |
| `economy_cycle.rs` | 78 | Game | None | Integration | Medium | Low | **MOVE** | Economy cycle tests |
| `certification_bootstrap_invariants.rs` | 73 | QA | certification | Certification | Heavy | High | **KEEP** | Bootstrap invariants |
| `render_pipeline.rs` | 58 | Graphics | None | Integration | Medium | Medium | **MOVE** | Render pipeline tests |
| `certification_perf_snapshot.rs` | 57 | QA | certification | Certification | Heavy | High | **KEEP** | Performance snapshots |
| `runtime_role_separation.rs` | 55 | Runtime | None | Contract | Fast | Medium | **MOVE** | Runtime role separation |
| `multithreading.rs` | 55 | Core | None | Contract | Medium | Medium | **MOVE** | Multithreading contracts |
| `body_pipeline.rs` | 47 | Physics | None | Integration | Medium | Medium | **MOVE** | Body pipeline tests |
| `performance.rs` | 40 | QA | None | Perf | Heavy | Medium | **MOVE** | Performance tests |
| `ai_social_economy.rs` | 40 | Game | None | Integration | Medium | Low | **MOVE** | AI social/economy tests |
| `certification_kernel_throughput.rs` | 33 | QA | certification | Certification | Heavy | High | **KEEP** | Kernel throughput |
| `certification_tick_budget.rs` | 27 | QA | certification | Certification | Heavy | High | **KEEP** | Tick budget enforcement |
| `certification_boundary_overhead.rs` | 25 | QA | certification | Certification | Heavy | High | **KEEP** | Boundary overhead |
| **NEW EDITOR SUITES** |
| `editor_console_contracts.rs` | 450 | Tools | tools | Contract | Fast | Medium | **NEW** | Editor console contracts |
| `editor_inspector_contracts.rs` | 450 | Tools | tools | Contract | Fast | Medium | **NEW** | Editor inspector contracts |

## Current Status Summary

### ✅ **COMPLETED**
- **4 megasuites split**: core_ecs.rs, graphics_renderer.rs, game_player_layer.rs, engine_contracts.rs
- **11 specialized suites created** with proper lane assignments
- **Architecture validation meta-tests** implemented
- **Lane compliance** achieved for new suites

### 🔄 **IN PROGRESS**
- **sdk_editor_gui.rs**: Split started (2 suites created, original still exists)

### ❌ **REMAINING MEGASUITES TO SPLIT**
1. `e2e_regression.rs` (1,005 lines) - End-to-end regression
2. `perf_lowspec_mt.rs` (741 lines) - Low-spec performance
3. `engine_infrastructure.rs` (508 lines) - Infrastructure tests
4. `save_load_torture.rs` (441 lines) - Save/load torture
5. `determinism_and_sdk.rs` (408 lines) - SDK contracts

### 🎯 **LANE ASSIGNMENT ISSUES**
- Several files still have `Lane: None` - need canonical lane assignment
- Some files use non-canonical lane names
- `tools_runtime_purity.rs` should be `Lane: tools`

### 📊 **QUANTITATIVE STATUS**
- **Total test files**: 47 files
- **Megasuites remaining**: 4 files (>500 lines)
- **Specialized suites**: 11 files (<500 lines)
- **Lines in megasuites**: 3,762 lines
- **Lines in specialized suites**: 4,950 lines

### 🏗️ **ARCHITECTURE ISSUES**
- **engine_contracts.rs**: Still transitional aggregator, not pure smoke
- **Mock production types**: Some suites still contain local mocks
- **Lane validation**: Need automated enforcement of canonical lanes

## Next Execution Priority

### 🎯 **IMMEDIATE (Next Session)**
1. **Complete sdk_editor_gui.rs split** - move original to legacy
2. **Split e2e_regression.rs** - largest remaining megasuite
3. **Fix lane assignments** - canonical lane names only
4. **Run architecture validation** - identify violations

### 📋 **MEDIUM PRIORITY**
1. **Split perf_lowspec_mt.rs** - performance megasuite
2. **Split engine_infrastructure.rs** - infrastructure tests
3. **Remove local mocks** - use production APIs only
4. **Thin engine_contracts.rs** - pure smoke aggregator

### 🔮 **FUTURE**
1. **Automated lane validation** - CI enforcement
2. **Meta-test expansion** - comprehensive validation
3. **Documentation sync** - living inventory maintenance

---

*This matrix reflects the **actual current state** as of the last file scan, not aspirational targets.*
