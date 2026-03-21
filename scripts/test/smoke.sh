#!/usr/bin/env bash
# ENGENE Smoke Lane — quick validation after minor changes
# Targets: engine_contracts, production_candidate, entrypoint_and_operator_truth, ci_surface_contracts

set -euo pipefail

if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --profile default \
    --test engine_contracts \
    --test production_candidate \
    --test entrypoint_and_operator_truth \
    --test ci_surface_contracts
else
  cargo test --test engine_contracts
  cargo test --test production_candidate
  cargo test --test entrypoint_and_operator_truth
  cargo test --test ci_surface_contracts
fi
