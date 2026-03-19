#!/usr/bin/env bash
# ENGENE Contract Lane — runtime boundary contracts and phase ordering
# Targets: runtime_phase_contracts, physics_core_boundary_contracts, physics_bootstrap_contracts

set -euo pipefail

if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --profile default \
    --test physics_core_boundary_contracts \
    --test physics_bootstrap_contracts \
    --test runtime_phase_contracts
else
  cargo test --test physics_core_boundary_contracts
  cargo test --test physics_bootstrap_contracts
  cargo test --test runtime_phase_contracts
fi
