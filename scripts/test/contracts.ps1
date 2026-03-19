#!/usr/bin/env pwsh
# ENGENE Contract Lane — runtime ownership and boundary validation
# Targets: engine_contracts, determinism_and_sdk, runtime_systems

cargo nextest run --profile default --test engine_contracts --test determinism_and_sdk --test runtime_systems
