# ENGENE — First Run Guide

ENGENE currently exposes four real cargo entrypoints from the root package.

| Product | Canonical command | Purpose |
|---|---|---|
| **ENGENE Game** | `cargo run --bin engene_game` | playable runtime |
| **ENGENE SDK** | `cargo run --bin engene_sdk` | editor / workstation / debugging shell |
| **ENGENE Headless** | `cargo run --bin engene_headless -- --ticks 1200` | simulation / CI / non-visual validation |
| **ENGENE Tools** | `cargo run --bin engene_tools` | diagnostics / maintenance / non-game tooling |

## Reality rule

`engene_test` is **not** a current canonical bin in the transition branch.
Do not present it as a first-run entrypoint until it exists in Cargo again.

## Quick start

### Canonical launch paths

```bash
cargo run --bin engene_game
cargo run --bin engene_sdk
cargo run --bin engene_headless -- --ticks 1200
cargo run --bin engene_tools
```

### Operator shortcuts

**Bash (Linux/macOS/Git Bash):**
```bash
just smoke
just contracts
just certification
just perf
```

**PowerShell (Windows):**
```powershell
scripts/test/smoke.ps1
scripts/test/contracts.ps1
scripts/test/certification.ps1
scripts/test/perf.ps1
```

**Cargo aliases:**
```bash
cargo smoke
cargo contracts
cargo cert
```

**Bash scripts:**
```bash
scripts/test/smoke.sh
scripts/test/contracts.sh
scripts/test/certification.sh
scripts/test/perf.sh
```

## Current structure truth

```
/ENGENE_ROOT
  /apps                 # future package-owned runtime shells
  /crates               # workspace engine/game/sdk crates
  /src                  # current root package code + migration shell
  /tests                # integration / contract / certification tests
  /benches              # performance benchmarks
  /docs/canonical       # authoritative current-state docs
  /.github/workflows    # CI
  /scripts/test         # ergonomic lane entrypoints
```

## Canonical truth docs

- `docs/canonical/CURRENT_BRANCH_STATE.md`
- `docs/canonical/ENTRYPOINT_TRUTH.md`
- `docs/canonical/RUNTIME_ROLE_MATRIX.md`
- `docs/canonical/TEST_LANE_MAP.md`
- `docs/canonical/PLATFORM_AUDIT.md`
- `docs/canonical/PHYSICS_CORE_BOUNDARY.md`
- `docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md`

## Fast engine verification

For quick validation after minor changes, use the smoke lane:

**Canonical order:**
```bash
just smoke
cargo smoke
scripts/test/smoke.sh
scripts/test/smoke.ps1
```

The smoke lane runs:
- `engine_contracts` — core engine contracts
- `production_candidate` — production readiness gate
- `entrypoint_and_operator_truth` — exact doc/bin match

## Fast platform gate verification

For narrow green gate validation (physics/platform readiness):

**Canonical order:**
```bash
cargo test entrypoint_and_operator_truth
cargo test ci_surface_contracts
cargo test physics_core_boundary_contracts
cargo test physics_bootstrap_contracts
cargo test runtime_phase_contracts
```

These tests validate the real physics seam and platform contracts without legacy interference.
- `ci_surface_contracts` — CI structure verification

**Contract lane** (slower, more thorough):
```bash
just contracts
cargo contracts
scripts/test/contracts.sh
scripts/test/contracts.ps1
```

The contracts lane adds:
- `determinism_and_sdk`
- `runtime_systems`
- `runtime_phase_contracts` — includes real production calls
- `spatial_dirty_contracts` — uses production HierarchicalSpatialIndex
- `wiring_boundary_contracts` — real system descriptors

## Version info

```bash
cargo run --bin engene_game -- --version
```

## Warning

The workspace already declares `apps/*`, but the active cargo entrypoints are still the root-package bins.
Until package-level launch ownership is complete, documentation must describe **current execution truth**, not intended end state.
