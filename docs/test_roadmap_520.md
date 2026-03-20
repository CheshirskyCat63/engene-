# Test Architecture Analysis & 520-Test Roadmap

## Current State Analysis

### Existing Test Structure
The current test layer shows **mixed ownership** and **eclectic organization**:

**Current Test Files:**
- `engine_contracts.rs` (1,205 lines) - **MEGASUITE** - mixes command buffer, event bus, runtime manifests, parallel validation, persistent identity, authority matrix, chunk persistence, editor safe mode
- `architectural_gates.rs` (281 lines) - ownership enforcement, file-scan contracts
- `core_ecs.rs` (50,531 lines) - **MEGASUITE** - entity lifecycle, command buffer, component ops, access descriptors
- `runtime_phase_contracts.rs` (8,297 lines) - focused on runtime orchestration
- `physics_*_contracts.rs` - focused physics boundary tests
- `spatial_dirty_contracts.rs` - focused spatial tests
- `certification_*.rs` - performance and boundary overhead tests
- `determinism_and_sdk.rs` - SDK and determinism tests
- `tools_runtime_purity.rs` - tools/editor isolation

### Current Lane Coverage
Based on `TEST_LANE_MAP.md`:

**Smoke Lane (4 targets):**
- `engine_contracts` ⚠️ **MEGASUITE**
- `production_candidate`
- `entrypoint_and_operator_truth`
- `ci_surface_contracts`

**Contracts Lane (4 targets):**
- `physics_core_boundary_contracts`
- `physics_bootstrap_contracts`
- `runtime_phase_contracts`
- `spatial_dirty_contracts`

**Certification Lane (4 targets):**
- `certification_boundary_overhead`
- `certification_kernel_throughput`
- `certification_tick_budget`
- `certification_perf_snapshot`

**Legacy Recovery Lane (6 targets):**
- Isolated in `tests_legacy/` - quarantine layer

## Problem Analysis

### Issues with Current Structure
1. **Megasuite Problem**: `engine_contracts.rs` and `core_ecs.rs` contain multiple ownership domains
2. **Lane Coverage Gap**: Only covers ~18 test targets vs 520 needed
3. **Missing P0 Groups**: Architecture, core policies, ECS access, events, world persistence not properly separated
4. **Mixed Test Types**: Unit, integration, contract, scenario tests mixed in same files

## 520-Test Roadmap Structure

### Phase 1: Split Megasuites (Immediate)

**Split `engine_contracts.rs` into:**
- `core_command_and_access_contracts.rs` (15 tests)
- `event_bus_contracts.rs` (20 tests) 
- `identity_and_authority_contracts.rs` (15 tests)
- `runtime_profile_and_quality_contracts.rs` (15 tests)
- `world_persistence_contracts.rs` (20 tests)
- `editor_safe_mode_contracts.rs` (10 tests)

**Split `core_ecs.rs` into:**
- `entity_lifecycle_contracts.rs` (20 tests)
- `persistent_identity_contracts.rs` (20 tests)
- `command_buffer_contracts.rs` (25 tests)
- `access_descriptor_contracts.rs` (15 tests)
- `query_contracts.rs` (20 tests)

### Phase 2: P0 Test Groups (350 tests total)

#### 1. Architecture & Ownership (60 tests)
```
tests/architecture/
├── root_thin_shell_invariants.rs (10 tests)
├── compat_hub_purity.rs (10 tests)
├── forbidden_dependency_directions.rs (15 tests)
├── app_entrypoint_law.rs (10 tests)
├── bootstrap_contract.rs (5 tests)
└── architectural_gates_self_validation.rs (10 tests)
```

#### 2. Engine Core Policies (50 tests)
```
tests/core_policies/
├── runtime_config_profile.rs (15 tests)
├── quality_degradation_policy.rs (10 tests)
├── ownership_map_authority.rs (10 tests)
├── determinism_policy.rs (10 tests)
└── build_runtime_manifest.rs (5 tests)
```

#### 3. ECS, Access, Commands, Descriptors (70 tests)
```
tests/ecs_contracts/
├── entity_lifecycle.rs (15 tests)
├── persistent_identity.rs (15 tests)
├── command_buffer_ops.rs (15 tests)
├── access_descriptors.rs (10 tests)
└── query_contracts.rs (15 tests)
```

#### 4. Event System and Buses (60 tests)
```
tests/event_contracts/
├── event_bus_boundedness.rs (10 tests)
├── sticky_semantics.rs (10 tests)
├── clear_frame_lifecycle.rs (10 tests)
├── canonical_event_delivery.rs (10 tests)
├── aggregation_spatial_bucketing.rs (10 tests)
└── tracing_debug_hooks.rs (10 tests)
```

#### 5. Runtime Orchestration and Phase Laws (55 tests)
```
tests/runtime_contracts/
├── scheduler_world_tick.rs (10 tests)
├── phase_ordering.rs (15 tests)
├── runtime_bootstrap_assembly.rs (10 tests)
├── simulation_transition_orchestration.rs (10 tests)
└── jobs_fences_frame_graph.rs (10 tests)
```

#### 6. World, Spatial, Persistence, Streaming (55 tests)
```
tests/world_contracts/
├── chunk_persistence_cycles.rs (15 tests)
├── relink_report_integrity.rs (10 tests)
├── world_fields_material_surface_truth.rs (10 tests)
├── spatial_dirty_update_path.rs (10 tests)
└── streaming_load_unload_contracts.rs (10 tests)
```

### Phase 3: Extended Groups (170 tests)

#### 7. Physics Boundary and Damage Contracts (45 tests)
```
tests/physics_contracts/
├── physics_bootstrap_contracts.rs (10 tests)
├── damage_pipeline_ordering.rs (10 tests)
├── destruction_topology_signals.rs (10 tests)
├── fire_water_destruction_separation.rs (5 tests)
└── body_ballistics_integration_boundary.rs (10 tests)
```

#### 8. Render/Audio/Tools Purity (40 tests)
```
tests/boundary_purity/
├── render_boundary_purity.rs (10 tests)
├── audio_boundary_purity.rs (10 tests)
├── tools_editor_safe_mode.rs (10 tests)
└── debug_profiler_dashboard_isolation.rs (10 tests)
```

#### 9. Apps/SDK/Operator Truth (35 tests)
```
tests/sdk_contracts/
├── game_sdk_headless_entrypoint.rs (10 tests)
├── bootstrap_launcher_semantics.rs (10 tests)
├── sdk_workflow_operator_commands.rs (10 tests)
└── docs_entrypoint_consistency.rs (5 tests)
```

#### 10. Determinism/Replay/Certification (25 tests)
```
tests/determinism_contracts/
├── determinism_snapshot_equality.rs (10 tests)
├── replay_checkpoints_divergence.rs (10 tests)
└── perf_certification_invariants.rs (5 tests)
```

#### 11. Legacy Recovery Isolation (15 tests)
```
tests/legacy_recovery/
├── legacy_suite_wiring.rs (5 tests)
├── migration_compatibility_guards.rs (5 tests)
└── known_failing_scenario_markers.rs (5 tests)
```

#### 12. Meta-Tests for Test System (10 tests)
```
tests/meta_tests/
├── lane_membership_consistency.rs (5 tests)
└── file_naming_suite_hygiene.rs (5 tests)
```

## Implementation Strategy

### Step 1: Test Matrix Creation
Create comprehensive matrix tracking:
- File ownership
- Test type (unit/integration/contract/scenario/certification)
- Lane assignment
- Speed classification (fast/heavy)
- Invariant protection

### Step 2: Megasuite Splitting
- Refactor `engine_contracts.rs` → 6 focused suites
- Refactor `core_ecs.rs` → 5 focused suites
- Maintain existing test logic, improve organization

### Step 3: P0 Implementation
- Start with 20 critical tests from your list
- Focus on architecture, core, ECS, events, runtime, world persistence
- Each test answers: what invariant, whose ownership, which lane, fast/heavy

### Step 4: Lane Expansion
- Update `TEST_LANE_MAP.md` for 520-test coverage
- Add new lane commands: `just architecture`, `just core`, `just ecs`, etc.
- Ensure each test belongs to exactly one primary lane

## Critical Tests for Immediate Implementation

From your 20-example list, prioritize these:

1. `root_thin_shell_contains_only_allowed_modules`
2. `core_compat_hub_exports_only_canonical_surfaces`
3. `bootstrap_never_uses_engine_crates_directly`
4. `command_buffer_take_spawns_is_single_consumer`
5. `event_bus_sticky_survives_clear_frame`
6. `event_bus_capacity_drops_last_overflow_only`
7. `parallel_validation_detects_write_write_conflict`
8. `parallel_validation_allows_disjoint_parallel_systems`
9. `spawn_new_assigns_unique_persistent_ids`
10. `despawn_marks_presence_dead`
11. `authority_matrix_contains_transform_needs_destruction_spatial`
12. `chunk_save_unload_marks_entities_unloaded_not_dead`
13. `chunk_load_relinks_same_pid_after_cycle`
14. `runtime_profile_headless_has_zero_render_budget`
15. `degradation_order_contains_collision_navigation_identity_as_never_cut`
16. `transition_orchestrator_merge_is_deterministic_across_shards`
17. `editor_safe_mode_disables_panel_after_three_panics`
18. `engine_render_does_not_depend_on_sdk_app`
19. `engine_audio_does_not_require_gameplay_types_for_core_playback`
20. `engene_game_main_calls_only_game_framework`

## Success Metrics

- **Zero megasuites** > 500 lines
- **Each test** protects specific invariant
- **Clear ownership** boundaries
- **Comprehensive lane coverage** (520 tests)
- **Fast feedback** for developers (appropriate lane commands)
- **Release-gate proof** for critical invariants

This roadmap transforms the eclectic test layer into a structured, ownership-aware test architecture that can scale to 520+ tests while maintaining clarity and purpose.
