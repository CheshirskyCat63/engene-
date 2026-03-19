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
    powershell -ExecutionPolicy Bypass -File .\scripts\test\smoke.ps1

contracts:
    powershell -ExecutionPolicy Bypass -File .\scripts\test\contracts.ps1

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

legacy-recovery:
    powershell -ExecutionPolicy Bypass -File .\scripts\test\legacy_recovery.ps1

sdk:
    cargo run --bin engene_sdk

game:
    cargo run --bin engene_game

headless:
    cargo run --bin engene_headless -- --ticks 1200
