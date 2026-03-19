#!/usr/bin/env pwsh
# ENGENE Smoke Lane — quick validation after minor changes
# Targets: engine_contracts, production_candidate

cargo nextest run --profile default --test engine_contracts --test production_candidate
