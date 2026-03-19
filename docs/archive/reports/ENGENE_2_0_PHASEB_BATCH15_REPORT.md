# ENGENE 2.0 — Phase B Batch 15 Report

## Scope
- Narrow transition-core pass in allowed perimeter only.
- Code changes only in `benches/simulation_transition_core.rs`.

## Root cause targeted
- Remaining x8 under-scaling was dominated by per-iteration MT coordination overhead in both classify-only and classify+materialize bench execution models (`mpsc` command/ack fanout per iteration).
- That coordination tax was large enough to keep valid >=8 CPU scaling below required `>=3.0x` in the latest validated run (`classify_only_x8=2.325x`, `classify_materialize_x8=2.911x`).

## Code-level changes
- Replaced both MT pools’ per-iteration `mpsc` signaling with persistent fixed-worker epoch synchronization:
  - shared atomic epoch (`AtomicU64`)
  - shared atomic completion counter (`AtomicUsize`)
  - shared stop flag (`AtomicBool`)
  - fixed shard ownership for each worker
  - allocation-free per-iteration run path (`run_epoch_and_wait`)
- Worker wait path uses bounded `spin_loop` then `yield_now` fallback to avoid pathological oversubscription stalls on low-core hosts while preserving low coordination overhead on valid x8 hosts.

## Validation in this container (3 CPU)
- `cargo fmt --all`: PASS
- `cargo check --bench simulation_transition_core -q`: PASS
- `cargo bench --bench simulation_transition_core --no-run`: PASS
- `bash scripts/check_transition_speed_law.sh --proof-warm-build`: PASS
- `bash scripts/check_transition_speed_law.sh`: FAIL
  - `deterministic_merge_prep`: NEAR_MISS
  - x8 scaling proof blocked by environment guard (`available_cpus=3`, `required>=8`)
- `bash scripts/check_transition_speed_law.sh --from-existing`: FAIL (same reasons)

## Closure status
- Not closed in this container run (invalid host for x8 proof + deterministic_merge_prep near-miss).
- Final closure requires rerun on the same valid >=8 CPU host used for scaling truth.
