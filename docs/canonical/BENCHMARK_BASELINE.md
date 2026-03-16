# ENGENE Benchmark Baseline

Established: Stage 2 (SDK Unlock). Updated every 2 stages.

## Build Times (dev profile)

| Target | Time | Stage |
|--------|------|-------|
| lib only | ~2 min | 14 |
| lib + 3 bins | ~4 min | 14 |
| lib + tests + benches | ~3 min | 14 |
| full clean | ~8 min | 14 |

## Runtime Baselines (TBD — measure at first GPU-capable run)

| Metric | Value | Stage |
|--------|-------|-------|
| Frame time (render + sim) | TBD | 2 |
| Chunk save I/O (ms) | TBD | 4 |
| Chunk load I/O (ms) | TBD | 4 |
| AI tick cost (us/entity) | TBD | 5 |
| Nav rebuild spike (ms) | TBD | 5 |
| Memory peak (MB) | TBD | 2 |
| egui panel draw cost (us) | TBD | 2 |

## Benchmark Suite Results (cargo bench)

Run: `cargo bench --bench engine_benchmarks`

| Benchmark | Ops/sec | Latency | Stage |
|-----------|---------|---------|-------|
| CommandBuffer throughput | TBD | TBD | 2 |
| EventBus throughput | TBD | TBD | 2 |
| WorkerPool fanout | TBD | TBD | 2 |
| ContentDependencyGraph | TBD | TBD | 2 |
| PrefabRegistry lookup | TBD | TBD | 2 |
| BudgetedStreamer | TBD | TBD | 2 |
| World tick (100 entities) | TBD | TBD | 2 |
| AI decision batch (50 NPCs) | TBD | TBD | 2 |
| Chunk save/load roundtrip | TBD | TBD | 2 |
| Frustum culling (1000) | TBD | TBD | 2 |

## Stage 12 Optimization Infrastructure

| Component | Status | Notes |
|-----------|--------|-------|
| Per-system PerfBudget recording | ACTIVE | Timing wired into tick() and tick_parallel() |
| QualityGovernor | ACTIVE | Pressure-based degradation, wired into both tick paths |
| BudgetRegistry | ACTIVE | Phase-level CPU/GPU/memory budgets |
| DeterministicMerger | READY | BTreeMap-based merge for parallel outputs |
| DirtySet (Chunk/Nav/Entity) | REGISTERED | Resources in vertical_slice and headless assemblies |
| LowSpecCertifier | REGISTERED | 30 FPS target, 2GB memory limit, report generator |
| WorkerPool (rayon) | ACTIVE | Thread count = available_parallelism, par_join available |
| JobTopologyReport | ACTIVE | Partition systems into parallel groups, fence tracking |
| DeterminismPolicyMatrix | ACTIVE | Required/Preferred/Acceptable classification |
| DegradationOrder | ACTIVE | 16 entries, NeverCut for collision/persistence/save/nav |

## Trend

| Stage | Status | Notes |
|-------|--------|-------|
| 1 | OK | 0 errors, 0 warnings, all targets build |
| 2 | OK | egui wired, panels interactive |
| 3-5 | OK | Content-first, persistence, simulation all wired |
| 6-8 | OK | Item/economy, body/physics, animation integrated |
| 9-11 | OK | Visuals, audio, player layer complete |
| 12 | OK | Per-system timing, dirty-sets, deterministic merge, low-spec cert, perf budgets |
