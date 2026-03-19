#!/usr/bin/env pwsh
# ENGENE Smoke Lane — quick validation after minor changes
# Targets: engine_contracts, production_candidate, entrypoint_and_operator_truth, ci_surface_contracts

$nextest = $false
try {
    cargo nextest --version | Out-Null
    if ($LASTEXITCODE -eq 0) {
        $nextest = $true
    } else {
        $nextest = $false
    }
} catch {
    $nextest = $false
}

if ($nextest) {
    cargo nextest run --profile default `
      --test engine_contracts `
      --test production_candidate `
      --test entrypoint_and_operator_truth `
      --test ci_surface_contracts
} else {
    cargo test --test engine_contracts
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo test --test production_candidate
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo test --test entrypoint_and_operator_truth
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo test --test ci_surface_contracts
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
