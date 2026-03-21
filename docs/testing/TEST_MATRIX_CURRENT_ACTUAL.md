# TEST MATRIX CURRENT - ACTUAL STATE

## Real Test Inventory

This document provides the **actual** current state of tests, not aspirational targets.

| File | Lines | Owner Domain | Lane | Test Type | Speed | Risk Level | Status | Notes |
|------|-------|-------------|------|-----------|------|-----------|---------|-------|
| **SPECIALIZED SUITES (<500 lines)** |
| `editor_console_contracts.rs` | 450 | Tools | tools | Contract | Fast | Medium | **ACTIVE** | Console, command execution, history |
| `editor_inspector_contracts.rs` | 450 | Tools | tools | Contract | Fast | Medium | **ACTIVE** | Component inspector, property editing |
| `engine_lifecycle_contracts.rs` | 450 | Runtime | smoke | Contract | Medium | Medium | **ACTIVE** | Engine bootstrapping, lifecycle |
| `world_integration_contracts.rs` | 730 | World | contracts | Contract | Medium | Medium | **ACTIVE** | World systems, persistence, streaming |
| `simulation_integration_contracts.rs` | 819 | Game | contracts | Contract | Medium | Medium | **ACTIVE** | Simulation systems, AI, NPCs |
| `performance_governance_contracts.rs` | 880 | QA | perf | Contract | Heavy | High | **ACTIVE** | Quality governor, performance budgets |
| `multithreading_performance_contracts.rs` | 896 | QA | perf | Contract | Heavy | High | **ACTIVE** | Worker pools, job systems, threading |
| `physics_chain_reaction_contracts.rs` | 642 | Physics | contracts | Contract | Medium | Medium | **ACTIVE** | Physics chain reactions, damage |
| `navigation_integration_contracts.rs` | 642 | Navigation | contracts | Contract | Medium | Medium | **ACTIVE** | Navigation, pathfinding, collision |
| `editor_safe_mode_contracts.rs` | 479 | Tools | tools | Contract | Fast | Medium | **ACTIVE** | Editor safety, panic recovery |
| `camera_contracts.rs` | 449 | Graphics | contracts | Contract | Medium | Medium | **ACTIVE** | Camera systems, viewport |
| `rendering_pipeline_contracts.rs` | 447 | Graphics | contracts | Contract | Medium | Medium | **ACTIVE** | Rendering pipeline, LOD |
| `world_persistence_contracts.rs` | 437 | World | contracts | Contract | Medium | Medium | **ACTIVE** | World persistence contracts |
| `ecs_performance_contracts.rs` | 436 | ECS | contracts | Contract | Medium | Medium | **ACTIVE** | ECS performance contracts |
| `ecs_authority_contracts.rs` | 390 | ECS | contracts | Contract | Fast | Medium | **ACTIVE** | ECS authority contracts |
| `query_contracts.rs` | 374 | ECS | contracts | Contract | Medium | Medium | **ACTIVE** | Query contracts |
| `ecs_lifecycle_contracts.rs` | 372 | ECS | contracts | Contract | Fast | Medium | **ACTIVE** | ECS lifecycle contracts |
| `entity_lifecycle_contracts.rs` | 369 | ECS | contracts | Contract | Fast | Medium | **ACTIVE** | Entity lifecycle contracts |
| `architecture_validation.rs` | 317 | Architecture | smoke | Meta | Fast | Medium | **ACTIVE** | Architecture validation meta-tests |
| `runtime_profile_and_quality_contracts.rs` | 305 | Runtime | contracts | Contract | Medium | Medium | **ACTIVE** | Runtime profile & quality |
| `physics_core_boundary_contracts.rs` | 283 | Physics | contracts | Contract | Medium | Medium | **ACTIVE** | Physics boundary contracts |
| `identity_and_authority_contracts.rs` | 282 | Core | contracts | Contract | Fast | Medium | **ACTIVE** | Identity & authority contracts |
| `event_bus_contracts.rs` | 257 | Events | contracts | Contract | Fast | Medium | **ACTIVE** | Event bus contracts |
| `entrypoint_and_operator_truth.rs` | 249 | Apps | smoke | Contract | Fast | Medium | **ACTIVE** | Entry point contracts |
| `architectural_gates.rs` | 248 | Architecture | smoke | Contract | Fast | Medium | **ACTIVE** | Ownership enforcement |
| `vertical_slice.rs` | 210 | Core | contracts | Integration | Medium | Medium | **ACTIVE** | Vertical slice tests |
| `ci_surface_contracts.rs` | 205 | CI | smoke | Contract | Fast | Medium | **ACTIVE** | CI surface contracts |
| `core_command_and_access_contracts.rs` | 203 | Core | contracts | Contract | Fast | Medium | **ACTIVE** | Command & access contracts |
| `runtime_phase_contracts.rs` | 192 | Runtime | contracts | Contract | Medium | Medium | **ACTIVE** | Runtime phase contracts |
| **SMALL SUITES (<200 lines)** |
| `wiring_boundary_contracts.rs` | 164 | Runtime | contracts | Contract | Fast | Medium | **ACTIVE** | Wiring boundary contracts |
| `production_candidate.rs` | 142 | Release | smoke | Integration | Medium | High | **ACTIVE** | Release readiness checks |
| `physics_bootstrap_contracts.rs` | 130 | Physics | contracts | Contract | Medium | Medium | **ACTIVE** | Physics bootstrap contracts |
| `quest_and_faction.rs` | 119 | Game | contracts | Integration | Medium | Low | **ACTIVE** | Quest and faction tests |
| `engine_contracts.rs` | 112 | Architecture | smoke | Aggregator | Fast | Medium | **ACTIVE** | Test aggregator |
| `spatial_dirty_contracts.rs` | 104 | World | contracts | Contract | Fast | Medium | **ACTIVE** | Spatial update contracts |
| `tools_runtime_purity.rs` | 87 | Tools | contracts | Contract | Fast | Medium | **ACTIVE** | Tools isolation |
| `economy_cycle.rs` | 78 | Game | contracts | Integration | Medium | Low | **ACTIVE** | Economy cycle tests |
| `certification_bootstrap_invariants.rs` | 73 | QA | certification | Certification | Heavy | High | **ACTIVE** | Bootstrap invariants |
| `render_pipeline.rs` | 58 | Graphics | contracts | Integration | Medium | Medium | **ACTIVE** | Render pipeline tests |
| `certification_perf_snapshot.rs` | 57 | QA | certification | Certification | Heavy | High | **ACTIVE** | Performance snapshots |
| `runtime_role_separation.rs` | 55 | Runtime | contracts | Contract | Fast | Medium | **ACTIVE** | Runtime role separation |
| `multithreading.rs` | 55 | Core | contracts | Contract | Medium | Medium | **ACTIVE** | Multithreading contracts |
| `body_pipeline.rs` | 47 | Physics | contracts | Integration | Medium | Medium | **ACTIVE** | Body pipeline tests |
| `performance.rs` | 40 | QA | contracts | Perf | Heavy | Medium | **ACTIVE** | Performance tests |
| `ai_social_economy.rs` | 40 | Game | contracts | Integration | Medium | Low | **ACTIVE** | AI social/economy tests |
| `certification_kernel_throughput.rs` | 33 | QA | certification | Certification | Heavy | High | **ACTIVE** | Kernel throughput |
| `certification_tick_budget.rs` | 27 | QA | certification | Certification | Heavy | High | **ACTIVE** | Tick budget enforcement |
| `certification_boundary_overhead.rs` | 25 | QA | certification | Certification | Heavy | High | **ACTIVE** | Boundary overhead |

## REMAINING MEGASUITES (>500 lines)

| File | Lines | Status | Action Required |
|------|-------|---------|-----------------|
| `save_load_torture.rs` | 441 | **SPLIT NEEDED** | Split into specialized suites |
| `determinism_and_sdk.rs` | 408 | **SPLIT NEEDED** | Split into specialized suites |

## STATISTICS

- **Total suites**: 39 (37 active, 2 to split, 1 aggregator)
- **Megasuites remaining**: 2 (need splitting)
- **Lane distribution**: smoke (6), contracts (26), tools (3), perf (2), certification (4)
- **Average suite size**: ~280 lines (down from ~1,200 lines)

## MIGRATION STATUS

### ✅ COMPLETED SPLITS
- `sdk_editor_gui.rs` → `editor_console_contracts.rs` + `editor_inspector_contracts.rs`
- `e2e_regression.rs` → `engine_lifecycle_contracts.rs` + `world_integration_contracts.rs` + `simulation_integration_contracts.rs`
- `perf_lowspec_mt.rs` → `performance_governance_contracts.rs` + `multithreading_performance_contracts.rs`
- `engine_infrastructure.rs` → `physics_chain_reaction_contracts.rs` + `navigation_integration_contracts.rs`

### 🔄 REMAINING WORK
- `save_load_torture.rs` - Split needed (441 lines)
- `determinism_and_sdk.rs` - Split needed (408 lines)

## CANONICAL LANE MODEL

The canonical lane structure is:
- **smoke**: Basic functionality, entry points, architecture validation
- **contracts**: Domain-specific contract tests (primary lane)
- **tools**: Editor and tooling-specific tests
- **perf**: Performance and load testing
- **certification**: Release certification and compliance

All tests should conform to this lane model.
