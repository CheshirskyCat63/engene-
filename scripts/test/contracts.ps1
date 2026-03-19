#!/usr/bin/env pwsh
# ENGENE Contract Lane — runtime boundary contracts and phase ordering
# Targets: runtime_phase_contracts, physics_core_boundary_contracts, physics_bootstrap_contracts

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
      --test physics_core_boundary_contracts `
      --test physics_bootstrap_contracts `
      --test runtime_phase_contracts
} else {
    cargo test --test physics_core_boundary_contracts
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo test --test physics_bootstrap_contracts
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo test --test runtime_phase_contracts
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
