set shell := ["bash", "-cu"]

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

smoke:
    cargo nextest run --profile default --test engine_contracts --test production_candidate --test entrypoint_and_operator_truth --test ci_surface_contracts

contracts:
    cargo nextest run --profile default --test engine_contracts --test determinism_and_sdk --test runtime_systems --test runtime_phase_contracts --test spatial_dirty_contracts --test wiring_boundary_contracts --test physics_core_boundary_contracts --test physics_bootstrap_contracts

certification:
    cargo nextest run --profile ci --test certification_boundary_overhead --test certification_kernel_throughput --test certification_tick_budget --test certification_perf_snapshot

perf:
    cargo test --test certification_perf_snapshot -- --nocapture
    cargo bench --bench engine_benchmarks
    cargo bench --bench hot_paths
    cargo bench --bench simulation_transition_core
    cargo bench --bench kernel_throughput
    cargo bench --bench boundary_cost
    cargo bench --bench tick_pressure

doctor:
    cargo run --bin engene_tools

sdk:
    cargo run --bin engene_sdk

game:
    cargo run --bin engene_game

headless:
    cargo run --bin engene_headless -- --ticks 1200
