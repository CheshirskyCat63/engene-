#!/usr/bin/env pwsh
# ENGENE Perf Lane — performance benchmarks and snapshot
# Targets: certification_perf_snapshot + all benches

cargo test --test certification_perf_snapshot -- --nocapture
cargo bench --bench engine_benchmarks
cargo bench --bench hot_paths
cargo bench --bench simulation_transition_core
cargo bench --bench kernel_throughput
cargo bench --bench boundary_cost
cargo bench --bench tick_pressure
