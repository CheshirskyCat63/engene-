# ENGENE — First Run Guide

ENGENE currently exposes three real cargo entrypoints from the root package.

| Product | Canonical command | Purpose |
|---|---|---|
| **ENGENE Game** | `cargo run -p engene_game` | canonical runtime (apps/engene_game) |
| **ENGENE SDK** | `cargo run -p engene_sdk` | editor / workstation / debugging shell (apps/engene_sdk) |
| **ENGENE Headless** | `cargo run -p engene_run -- --ticks 1200` | simulation / CI / non-visual validation (apps/engene_run) |

## Reality rule

`engene_test` is **not** a current canonical bin in the transition branch.
Do not present it as a first-run entrypoint until it exists in Cargo again.

## Quick start

### Canonical launch paths

```bash
cargo run -p engene_game
cargo run -p engene_sdk
cargo run -p engene_run -- --ticks 1200
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

## Legacy recovery

For isolated broken legacy integration tests:

**Canonical order:**
```bash
just legacy-recovery
scripts/test/legacy_recovery.sh
scripts/test/legacy_recovery.ps1
```

The legacy forest is isolated in `tests_legacy/` and is not part of default platform gate.
It is a manual recovery surface after API drift repair, not a migration-readiness signal.

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

1. `just smoke`
2. `cargo smoke`
3. `scripts/test/smoke.sh`
4. `scripts/test/smoke.ps1`

Smoke targets:
- `engine_contracts`
- `production_candidate`
- `entrypoint_and_operator_truth`
- `ci_surface_contracts`

## Platform contracts

1. `just contracts`
2. `cargo contracts`
3. `scripts/test/contracts.sh`
4. `scripts/test/contracts.ps1`

Contracts targets:
- `physics_core_boundary_contracts`
- `physics_bootstrap_contracts`
- `runtime_phase_contracts`
- `spatial_dirty_contracts`

## Legacy recovery

1. `just legacy-recovery`
2. `scripts/test/legacy_recovery.sh`
3. `scripts/test/legacy_recovery.ps1`

## Narrow platform gate

Current green gate is intentionally narrow and excludes legacy domain suites from default path.

Smoke lane and contracts lane together are the current platform signal:
```bash
cargo test --test entrypoint_and_operator_truth
cargo test --test ci_surface_contracts
cargo test --test physics_core_boundary_contracts
cargo test --test physics_bootstrap_contracts
cargo test --test runtime_phase_contracts
cargo test --test spatial_dirty_contracts
```

Migration is not finished while root shell ownership is still active as canonical launch path.
Current state is finish-ready for migration handoff, with root constrained to thin compatibility shell duties.

## Version info

```bash
cargo run --bin engene_game -- --version
```

## Warning

The workspace already declares `apps/*`, but the active cargo entrypoints are still the root-package bins.
Until package-level launch ownership is complete, documentation must describe **current execution truth**, not intended end state.
