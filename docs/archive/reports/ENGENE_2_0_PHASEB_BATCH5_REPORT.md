# ENGENE 2.0 Phase B Batch 5 Report

## Scope
Phase B Batch 5 turns transition-core speed law into executable proof with isolated bench compilation, repeatable regression gating, and bounded hot-path sanity assertions.

## What changed
- Benchmark isolation:
  - package sets `autobins = false` and explicitly marks runtime binaries `bench = false` to avoid unrelated heavy bin bench compilation.
  - dedicated transition-core bench target remains explicit (`simulation_transition_core`).
- Regression gate:
  - `scripts/check_transition_speed_law.sh` adds pass/fail evaluation for each required benchmark surface using Criterion estimates.
  - gate validates both max-mean-latency and minimum-throughput thresholds per surface.
- Proof hygiene:
  - benchmark baseline document updated with explicit per-surface gate semantics and script usage.
  - Batch 4 speed-law defaults remain synchronized in code and docs.
- Hot-path enforcement:
  - debug-only assertions verify reusable transition batch and queue capacity invariants.
  - no new shipping-path allocations introduced.

## Executable proof status
- Bench compile proof target:
  - `cargo bench --bench simulation_transition_core --no-run`.
- Regression proof target:
  - `bash scripts/check_transition_speed_law.sh --from-existing` for deterministic re-check from existing Criterion outputs.
  - `bash scripts/check_transition_speed_law.sh` for full run + gate.

## Law-compliance notes
- Work is limited to transition-core benchmarking/enforcement surfaces.
- No gameplay features, no domain-truth migration, and no new runtime subsystem expansion.

## Validation
- `bash scripts/check_dependency_direction.sh`
- `bash scripts/check_ecs_direct_access.sh`
- `cargo check --workspace -q`
- `cargo bench --bench simulation_transition_core --no-run`
