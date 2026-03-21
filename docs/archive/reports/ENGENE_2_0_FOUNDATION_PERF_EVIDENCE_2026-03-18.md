# ENGENE 2.0 Foundation Perf Evidence (Generated)

Date (UTC): 2026-03-18T22:27:46Z
Commit: working tree at evidence generation (see git history for exact SHA)
Host: Linux 6.12.47 x86_64 GNU/Linux
Logical CPUs (`nproc`): 3

## Command set executed
1. `cargo fmt`
2. `cargo check --workspace`
3. `cargo test --tests --no-run -q`
4. `cargo test --test certification_kernel_throughput -q`
5. `cargo test --test certification_boundary_overhead -q`
6. `cargo test --test certification_tick_budget -q`
7. `cargo bench --no-run`
8. `bash scripts/check_dependency_direction.sh`
9. `bash scripts/check_ecs_direct_access.sh`
10. `cargo test --test certification_perf_snapshot -- --nocapture`

## Cargo profile / features note
- Certification tests: `test` profile (unoptimized + debuginfo).
- Benches compile check: `bench` profile (`cargo bench --no-run`).
- Default feature set was used (no feature overrides).

## Final measured numbers
### Pure kernel throughput (producer -> core bus -> consumer)
| Threads | total_ops | ops_per_sec | ns_per_op | p50 ns | p95 ns | p99 ns | dropped_events | backlog_peak | speedup_vs_1t |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 5000 | 877039.42 | 1140.20 | 1898431 | 3354436 | 3417341 | 0 | 3883 | 1.000 |
| 2 | 10000 | 1186862.20 | 842.56 | 2243287 | 4653340 | 4766036 | 0 | 5938 | 1.353 |
| 4 | 20000 | 1067534.53 | 936.74 | 7632819 | 12684992 | 12908718 | 0 | 18244 | 1.217 |
| 8 | 40000 | 1159577.59 | 862.38 | 12513926 | 22315343 | 22840963 | 0 | 31529 | 1.322 |
| 16 | 80000 | 1119199.00 | 893.50 | 27741499 | 47206931 | 48204202 | 0 | 67338 | 1.276 |

### Boundary overhead
- baseline ns/event: **10.3584**
- kernel -> runtime envelope ns/event: **96.2422**
- runtime routing ns/event: **29.4794**
- runtime -> observer ns/event: **9.7034**
- total boundary ns/event: **135.4251**
- relative slowdown vs baseline: **13.0739x**

### Tick pressure
- mean_tick_ns: **1000191.42**
- max_tick_ns: **1000909**
- deadline_miss_ticks: **24**
- max_backlog: **135773**
- drain_efficiency: **1.0000**
- max_sustainable_ops_per_tick: **7001**

## PASS/FAIL by scenario
- Kernel throughput certification: PASS
- Boundary overhead certification: PASS
- Tick budget certification: PASS
- Bench compile surface (`cargo bench --no-run`): PASS

## Caveats
- Host has 3 logical CPUs, so 8/16-thread scaling is oversubscription behavior by design.
- Thresholds in certification are split into binding integrity checks and explicitly temporary performance-floor checks.
