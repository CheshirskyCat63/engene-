# Performance and Low-Spec Validation Report

Generated: 2026-03-10

---

## 1. Performance Infrastructure

### QualityGovernor (`src/core/quality_governor.rs`)

- **Status**: **ACTIVE** — updated every tick with frame time
- **Target**: 60 FPS (16,667 us per frame)
- **Pressure levels**: tracks frame time, computes pressure score
- **Degradation order**: 16 entries

### BudgetRegistry (`src/core/budget_registry.rs`)

- **Status**: **ACTIVE** — records `total_frame` measurement each tick

### PerfBudgetManager (`src/core/perf/perf_budget.rs`)

- **Status**: **ACTIVE** — wired with per-system timing in both `tick()` and `tick_parallel()`
- **Budget allocations defined**: AI 25%, Physics 15%, Simulation 10%, Economy 5%, Animation 5%, Render 25%, Audio 3%, Navigation 5%, Body 2%, Streaming 3%, Overhead 2%
- **Per-system timing**: Systems record measurements into PerfBudgetManager

### puffin Profiling

- **Status**: **ACTIVE** — `puffin::set_scopes_on(true)` called in both binaries
- **Instrumentation**: `puffin::profile_function!()` in Engine::tick, `puffin::profile_scope!("system", name)` per system
- **Viewer**: puffin data collected but no puffin viewer UI wired (would need `puffin_egui` or `puffin_http`)

### Additional Infrastructure

- **DirtySet** (Chunk/Nav/Entity): Registered as resources
- **DeterministicMerger**: Infrastructure ready
- **LowSpecCertifier**: Registered (30 FPS target, 2GB memory limit)

---

## 2. Benchmark Suite

### Benchmarks (`benches/engine_benchmarks.rs`)

| Benchmark | What It Measures | Status |
|-----------|-----------------|--------|
| `bench_command_buffer_throughput` | CommandBuffer spawn/despawn speed | **ACTIVE** |
| `bench_event_bus_throughput` | EventBus emit/read speed | **ACTIVE** |
| `bench_worker_pool_fanout` | WorkerPool task distribution | **ACTIVE** |
| `bench_dependency_graph_invalidation` | DependencyGraph invalidation speed | **ACTIVE** |
| `bench_prefab_registry_lookup` | PrefabRegistry lookup speed | **ACTIVE** |
| `bench_io_budget_streamer` | I/O budget streaming throughput | **ACTIVE** |
| `bench_world_tick_100_entities` | Full world tick with 100 entities | **ACTIVE** |
| `bench_ai_batch_50_npcs` | AI batch processing for 50 NPCs | **ACTIVE** |
| `bench_chunk_save_load_roundtrip` | Chunk save/load round-trip time | **ACTIVE** |
| `bench_frustum_cull_1000_instances` | Frustum culling 1000 instances | **ACTIVE** |

**Note**: Benchmarks compile and run but `BENCHMARK_BASELINE.md` with locked baseline numbers has not been generated.

---

## 3. Shipping Profile Assessment

### Entity Counts (at startup)

- Total entities: ~60-100 (varies by spawn rules)
- NPCs: ~25-35
- Monsters: ~25-30 (wolves + boars + bloodsuckers)
- World size: 2x2 km

### Frame Pacing

- Simulation rate: 20Hz fixed tick
- Render rate: Uncapped (vsync depends on driver)
- Frame accumulator: Clamped at 0.25s max to prevent spiral-of-death

### Memory

- No explicit memory tracking in runtime
- No memory peak measurement system
- `AssetBudget` exists as data type but not enforced

### Render Performance

- Shadow: 4-cascade CSM (terrain + entity pass per cascade = 8 draw calls)
- Geometry: 1 terrain draw + 1 instanced entity draw + vegetation draw
- Skybox: 1 fullscreen triangle
- Tonemap: 1 fullscreen triangle
- Total draw calls per frame: ~12-15 (efficient instancing)
- No GPU profiling instrumented

### Known Performance Characteristics

- **Stable frame pacing**: YES — fixed-step simulation prevents timing jitter
- **No runaway allocations**: YES — event channels have capacity limits, dropped events counted
- **Entity count stable**: YES — verified in soak test (200 ticks, <50% drift)

---

## 4. Low-Spec Profile Assessment

### LowSpecPolicy System

Every `SystemDescriptor` has a `low_spec_policy` field:

| Policy | Meaning | Systems Using |
|--------|---------|---------------|
| `Off` | Completely disabled in low-spec | None declared |
| `ReducedCadence` | Runs every N ticks instead of every tick | None wired |
| `SimplifiedPath` | Uses simplified algorithm | Default for all systems |
| `NeverCut` | Must always run at full quality | None declared |

### Doctor Validation

`check_low_spec_policies()` in `doctor.rs` validates that all systems have a declared policy. Default is `SimplifiedPath` (set in `SystemDescriptor::new()`).

### Actual Low-Spec Behavior

| System | Low-Spec Degradation | Actually Implemented |
|--------|---------------------|---------------------|
| AI | Reduce cadence, simplify decisions | **NO** — same code path always |
| Physics | Reduce iterations | **NO** — same code path |
| Animation | Reduce quality (AnimationLadder) | **NO** — AnimationLadder not wired |
| Particles | Reduce density | **NO** — particles not wired |
| Vegetation | Reduce density | **NO** — same density always |
| Audio | Reduce channels | **NO** — same behavior |
| Body | Simplify response | **NO** — same code path |
| Render | Reduce shadow cascades, disable bloom | **NO** — same render path |

**Verdict**: The low-spec policy infrastructure exists (every system has a `LowSpecPolicy` field), but no system actually reads its policy at runtime or changes behavior based on it. The QualityGovernor tracks frame pressure but no system responds to it.

---

## 5. Performance Tests

### From `tests/production_candidate.rs`:

| Test | What It Checks | Status |
|------|---------------|--------|
| `production_perf_budget` | PerfBudgetManager records and reports | **PASS** (data type test) |

### From headless soak:

- Entity stability verified over extended simulation
- No crash over 10-minute headless run
- Economy metrics remain sensible

---

## 6. Honest Assessment

### What IS Real:
- Fixed-step simulation prevents frame time spikes from causing physics explosions
- Instanced rendering keeps draw calls low (~12-15 per frame)
- Event channel capacity limits prevent unbounded memory growth
- puffin instrumentation is present (data is collected)
- QualityGovernor tracks frame time (pressure levels, degradation order with 16 entries)
- PerfBudgetManager records per-system timing in tick() and tick_parallel()
- DirtySet, DeterministicMerger, LowSpecCertifier infrastructure in place

### What is NOT Real:
- Per-system CPU budget tracking (PerfBudgetManager not wired)
- Low-spec mode (infrastructure exists, zero behavioral changes)
- GPU profiling
- Memory peak tracking
- Benchmark baseline lock (no BENCHMARK_BASELINE.md)
- Performance regression detection between builds
- Adaptive AI cadence, vegetation/particle density reduction, audio channel reduction

### Summary Table

| Metric | Shipping | Low-Spec | Gap |
|--------|----------|----------|-----|
| Frame budget tracking | Total + per-system (PerfBudgetManager) | Same | BudgetRegistry + PerfBudgetManager active |
| Simulation stability | Good | Same | No degradation available |
| Draw call count | ~12-15 | Same | No reduction path |
| Entity budget | ~100 | Same | No scaling |
| Memory tracking | LowSpecCertifier (2GB limit) | Same | Target defined |
| Low-spec behavior | N/A | Infrastructure (LowSpecCertifier 30 FPS) | Behavioral changes not wired |
