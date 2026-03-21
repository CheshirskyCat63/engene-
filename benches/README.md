# Benches Status

## Active Performance Law Benches

### ✅ simulation_transition_core.rs
- **Status**: ACTIVE_PERF_LAW
- **Purpose**: Core simulation transition performance validation
- **Engine Integration**: Uses `engine_runtime::simulation_core`
- **Last Validated**: March 2026

### Legacy Benches (Status Unknown)

- `boundary_cost.rs` - Needs audit for current engine relevance
- `engine_benchmarks.rs` - Needs audit for current engine relevance  
- `hot_paths.rs` - Needs audit for current engine relevance
- `kernel_throughput.rs` - Needs audit for current engine relevance
- `tick_pressure.rs` - Needs audit for current engine relevance

## Performance Law Requirements

Active benches must validate:
- Minimum TPS thresholds for single/multi-threaded scenarios
- Latency budgets under load
- Scaling efficiency ratios
- Memory usage boundaries

## Audit Required

Each legacy bench needs assessment:
1. Does it compile with current engine crates?
2. Are its metrics still relevant to current architecture?
3. Should it be migrated to new engine structure?

**Decision**: Keep only actively maintained benches that validate current engine performance laws.
