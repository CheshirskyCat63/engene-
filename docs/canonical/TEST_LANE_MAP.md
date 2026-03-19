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

### 2. Contract lane
Use after:
- runtime ownership changes,
- determinism / SDK boundary work,
- orchestration split work,
- system contract work,
- physics boundary work.

Targets:
- `engine_contracts`
- `determinism_and_sdk`
- `runtime_systems`
- `runtime_phase_contracts`
- `spatial_dirty_contracts`
- `wiring_boundary_contracts`
- `physics_core_boundary_contracts`
- `physics_bootstrap_contracts`

Commands:
```bash
just contracts
cargo contracts
scripts/test/contracts.sh
scripts/test/contracts.ps1
```

### 3. Certification lane
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

### 4. Perf lane
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
