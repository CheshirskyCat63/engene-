# ENGENE 2.0 Phase B Batch 6 Report

## Scope
Phase B Batch 6 closes remaining ambiguity around transition-core speed proof by adding cold-build truth, executable multithread scaling proof, and concrete bench isolation checks.

## What changed
- Bench isolation truth path:
  - Added executable cold/warm isolation proof modes in `scripts/check_transition_speed_law.sh`.
  - Proof uses `cargo rustc --bench simulation_transition_core --profile bench -vv` and fails if root runtime bins are compiled.
- Multithread proof surfaces:
  - Bench now includes `classify_single_thread` and `classify_multi_thread_x8` on shared deterministic workload.
  - Deterministic shard partition + deterministic merge prep path is benchmarked.
  - Scaling ratio (`single_thread_mean / multi_thread_x8_mean`) is now enforced in gate.
- Regression gate upgrade:
  - Added pass/fail thresholds for all required surfaces.
  - Added explicit scaling minimum enforcement (`>= 3.0x`) with target visibility (`4.5x`).
- Hot-path sanity assertions:
  - Debug-only queue and reusable batch capacity invariants in runtime/compatibility path.

## Cold-build truth
- `cargo bench --bench simulation_transition_core --no-run` can still compile root bins under Cargo bench behavior.
- Isolated proof path for transition-core compile now uses `cargo rustc --bench simulation_transition_core --profile bench` and validates no root bin compile hits.
- Batch 6 treats this as executable truth and keeps both paths explicit:
  - required compatibility check: `cargo bench --bench simulation_transition_core --no-run`
  - isolated proof check: `bash scripts/check_transition_speed_law.sh --proof-cold-build`

## Law-compliance notes
- No ownership migration outside transition-core orchestration/proof surfaces.
- No gameplay changes or subsystem expansions.
- Determinism and bounded queue law remain unchanged.

## Validation
- `bash scripts/check_dependency_direction.sh`
- `bash scripts/check_ecs_direct_access.sh`
- `cargo check --workspace -q`
- `cargo bench --bench simulation_transition_core --no-run`
- `bash scripts/check_transition_speed_law.sh --proof-cold-build`
- `bash scripts/check_transition_speed_law.sh --proof-warm-build`
