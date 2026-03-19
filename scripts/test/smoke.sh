#!/bin/bash
# ENGENE Smoke Lane — quick validation after minor changes
# Targets: engine_contracts, production_candidate

set -e
cargo nextest run --profile default --test engine_contracts --test production_candidate
