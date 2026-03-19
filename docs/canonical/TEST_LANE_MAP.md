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
- Contracts: `physics_core_boundary_contracts`, `physics_bootstrap_contracts`, `runtime_phase_contracts`

Excluded from default gate and isolated in `tests_legacy/`:
- `world_streaming`
- `physics_body_combat`
- `content_pipeline`
- `persistence_full`
- `runtime_systems`
- `gameplay_and_ai`
