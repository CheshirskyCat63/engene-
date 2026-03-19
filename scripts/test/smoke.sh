#!/bin/bash
# ENGENE Smoke Lane — quick validation after minor changes
# Targets: engine_contracts, production_candidate, entrypoint_and_operator_truth, ci_surface_contracts

set -e
cargo nextest run --profile default --test engine_contracts --test production_candidate --test entrypoint_and_operator_truth --test ci_surface_contracts
