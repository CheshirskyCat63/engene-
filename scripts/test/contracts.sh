#!/usr/bin/env bash
# ENGENE Contract Lane — runtime boundary contracts and phase ordering
# Targets: physics_core_boundary_contracts, physics_bootstrap_contracts, runtime_phase_contracts, spatial_dirty_contracts

set -euo pipefail

if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --profile default \
    --test physics_core_boundary_contracts \
    --test physics_bootstrap_contracts \
    --test runtime_phase_contracts \
    --test spatial_dirty_contracts
else
  cargo test --test physics_core_boundary_contracts
  cargo test --test physics_bootstrap_contracts
  cargo test --test runtime_phase_contracts
  cargo test --test spatial_dirty_contracts
fi
