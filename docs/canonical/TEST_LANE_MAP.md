# TEST_LANE_MAP

## Why this exists

A large test surface without operator lanes is just a labyrinth with good intentions.

## Canonical lanes

### 1. Smoke lane
Use after:
- doc fixes,
- Cargo wiring edits,
- launch-path fixes,
- small boundary changes.

Targets:
- `engine_contracts`
- `production_candidate`
- `entrypoint_and_operator_truth`
- `ci_surface_contracts`

Commands:
```bash
just smoke
cargo smoke
scripts/test/smoke.sh
scripts/test/smoke.ps1
```

### 2. Contracts lane
Use after:
- platform contract work (phase order, runtime boundary, physics boundary)

Targets:
- `physics_core_boundary_contracts`
- `physics_bootstrap_contracts`
- `runtime_phase_contracts`
- `spatial_dirty_contracts`

Commands:
```bash
just contracts
cargo contracts
scripts/test/contracts.sh
scripts/test/contracts.ps1
```

### 3. Legacy recovery lane
Use only for recovering broken legacy integration tests after API drift.

Targets:
- `world_streaming` (moved to tests_legacy/)
- `physics_body_combat` (moved to tests_legacy/)
- `content_pipeline` (moved to tests_legacy/)
- `persistence_full` (moved to tests_legacy/)
- `runtime_systems` (moved to tests_legacy/)
- `gameplay_and_ai` (moved to tests_legacy/)

Commands:
```bash
just legacy-recovery
scripts/test/legacy_recovery.sh
scripts/test/legacy_recovery.ps1
```

### 4. Certification lane
Use after:
- performance boundary work,
- measured acceptance changes,
- fixed-tick pressure changes.

Targets:
- `certification_boundary_overhead`
- `certification_kernel_throughput`
- `certification_tick_budget`
- `certification_perf_snapshot`

Commands:
```bash
just certification
cargo cert
scripts/test/certification.sh
scripts/test/certification.ps1
```

### 5. Perf lane
Use only when touching hot paths or perf contracts.

Targets:
- `certification_perf_snapshot`
- benches:
  - `engine_benchmarks`
  - `hot_paths`
  - `simulation_transition_core`
  - `kernel_throughput`
  - `boundary_cost`
  - `tick_pressure`

Commands:
```bash
just perf
scripts/test/perf.sh
scripts/test/perf.ps1
```

## Rule

Nobody should have to remember raw test names during normal work.
The command surface must do that remembering for them.

## Current gate boundary

Current green platform gate is narrow by design:
- Smoke: `engine_contracts`, `production_candidate`, `entrypoint_and_operator_truth`, `ci_surface_contracts`
- Contracts: `physics_core_boundary_contracts`, `physics_bootstrap_contracts`, `runtime_phase_contracts`, `spatial_dirty_contracts`

### NEW: Expanded Lane Structure for 520+ Tests

### 6. Architecture Lane
Use for:
- Ownership enforcement
- Dependency direction validation
- Root module contracts
- Bootstrap purity

Targets:
- `architectural_gates`
- `root_thin_shell_invariants`
- `compat_hub_purity`
- `forbidden_dependency_directions`
- `bootstrap_contract`

Commands:
```bash
just architecture
cargo architecture
scripts/test/architecture.sh
scripts/test/architecture.ps1
```

### 7. Core Lane
Use for:
- Runtime profiles
- Quality policies
- Authority matrix
- Performance contracts

Targets:
- `runtime_profile_and_quality_contracts`
- `identity_and_authority_contracts`
- `core_command_and_access_contracts`

Commands:
```bash
just core
cargo core
scripts/test/core.sh
scripts/test/core.ps1
```

### 8. ECS Lane
Use for:
- Entity lifecycle
- Component operations
- Access descriptors
- Query contracts

Targets:
- `entity_lifecycle_contracts`
- `persistent_identity_contracts`
- `command_buffer_contracts`
- `access_descriptor_contracts`
- `query_contracts`

Commands:
```bash
just ecs
cargo ecs
scripts/test/ecs.sh
scripts/test/ecs.ps1
```

### 9. Events Lane
Use for:
- Event bus contracts
- Sticky semantics
- Frame lifecycle
- Event aggregation

Targets:
- `event_bus_contracts`
- `event_aggregation_contracts`
- `event_tracing_contracts`

Commands:
```bash
just events
cargo events
scripts/test/events.sh
scripts/test/events.ps1
```

### 10. World Lane
Use for:
- Chunk persistence
- Streaming contracts
- Spatial contracts
- World fields

Targets:
- `world_persistence_contracts`
- `spatial_dirty_contracts`
- `world_fields_contracts`
- `streaming_contracts`

Commands:
```bash
just world
cargo world
scripts/test/world.sh
scripts/test/world.ps1
```

### 11. Tools Lane
Use for:
- Editor safe mode
- Debug surface isolation
- Profiler contracts
- Dashboard isolation

Targets:
- `editor_safe_mode_contracts`
- `debug_surface_contracts`
- `profiler_contracts`
- `dashboard_contracts`

Commands:
```bash
just tools
cargo tools
scripts/test/tools.sh
scripts/test/tools.ps1
```

Excluded from default gate and isolated in `tests_legacy/`:
- `world_streaming`
- `physics_body_combat`
- `content_pipeline`
- `persistence_full`
- `runtime_systems`
- `gameplay_and_ai`
