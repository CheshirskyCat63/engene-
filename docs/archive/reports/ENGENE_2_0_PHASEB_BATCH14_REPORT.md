# ENGENE 2.0 — Phase B Batch 14 Report

## Scope
- Narrow transition-core pass only.
- Changed files:
  - `benches/simulation_transition_core.rs`
  - `scripts/check_transition_speed_law.sh` (no new Batch 14 logic changes)
  - `docs/canonical/ENGENE_2_0_PHASEB_BATCH14_REPORT.md`
  - `docs/generated/PHASE_B_TRANSITION_CORE_BENCH_BASELINE.md`
  - `docs/generated/PHASE_A_API_SURFACE_REPORT.md`
  - `docs/generated/PHASE_A_DEPENDENCY_GRAPH.md`

## What changed in code
- `classify_only_multi_thread_x8` persistent worker synchronization was changed from per-iteration `mpsc` command/ack signaling to persistent fixed-worker barriers (`start_barrier`/`done_barrier`) with fixed shard ownership.
- No workload changes (`CLASSIFY_COUNT` unchanged).
- No threshold changes.
- No semantics changes in transition-core contracts/orchestrator.
- `classify_materialize` path was not changed in Batch 14.

## Required command outcomes (container run)
- `cargo fmt --all`: PASS
- `cargo check --bench simulation_transition_core -q`: PASS
- `cargo bench --bench simulation_transition_core --no-run`: PASS
- `bash scripts/check_transition_speed_law.sh --proof-warm-build`: PASS
- `bash scripts/check_transition_speed_law.sh`: FAIL
  - per-surface:
    - `deterministic_merge_prep`: NEAR_MISS
    - all other surfaces: PASS
  - scaling evaluation blocked by environment guard (`available_cpus=3`, `required>=8`)
- `bash scripts/check_transition_speed_law.sh --from-existing`: FAIL (same reasons as full gate run)

## Closure status
- Phase B is **not closed** in this container run.
- Honest x8 scaling proof still requires rerun on host with `>=8` logical CPUs.
