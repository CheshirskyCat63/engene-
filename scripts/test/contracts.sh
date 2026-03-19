#!/bin/bash
# ENGENE Contract Lane — runtime ownership and boundary validation
# Targets: engine_contracts, determinism_and_sdk, runtime_systems, runtime_phase_contracts, spatial_dirty_contracts, wiring_boundary_contracts

set -e
cargo nextest run --profile default --test engine_contracts --test determinism_and_sdk --test runtime_systems --test runtime_phase_contracts --test spatial_dirty_contracts --test wiring_boundary_contracts
