# ENGENE gold drop — apply order

This pack is not a 50-file markdown theme park.
It is a small set of **enforcement docs**, **truth maps**, and **operator files**
that reduce ambiguity and make the transition branch auditable.

## What this pack is for

Use this pack to close the dangerous gap between:

- declared workspace split
- real runtime ownership
- docs that still overstate future state
- CI/test execution ergonomics

## Apply order

1. Replace `README_FIRST_RUN.md`.
2. Replace `.github/workflows/rust.yml`.
3. Add `.cargo/config.toml`, `.config/nextest.toml`, `Justfile`, `scripts/test/*`.
4. Add the new files under `docs/canonical/`.
5. Update any old docs to point to:
   - `docs/canonical/ACTIVE_REPO_STATE.md`
   - `docs/canonical/ENTRYPOINT_TRUTH.md`
   - `docs/canonical/RUNTIME_ROLE_MATRIX.md`
   - `docs/canonical/TEST_LANE_MAP.md`

## Replace now

- `README_FIRST_RUN.md`
- `.github/workflows/rust.yml`
- `docs/canonical/ENGENE_2_0_ENTRYPOINTS.md`

## Add now

- `docs/canonical/ACTIVE_REPO_STATE.md`
- `docs/canonical/ENTRYPOINT_TRUTH.md`
- `docs/canonical/ROOT_CRATE_POLICY.md`
- `docs/canonical/RUNTIME_ROLE_MATRIX.md`
- `docs/canonical/FEATURE_ROLE_POLICY.md`
- `docs/canonical/WORKSPACE_OWNERSHIP_MAP.md`
- `docs/canonical/DEPENDENCY_LAW.md`
- `docs/canonical/MIGRATION_LEDGER.md`
- `docs/canonical/REMOVAL_PLAN.md`
- `docs/canonical/PHASE_ORDER_CONTRACT.md`
- `docs/canonical/RUNTIME_INVARIANTS.md`
- `docs/canonical/SPATIAL_DIRTY_CONTRACT.md`
- `docs/canonical/TEST_LANE_MAP.md`
- `docs/canonical/DOC_STATUS_BOARD.md`
- `docs/canonical/STALE_DOCS_TO_ARCHIVE.md`

## Why this pack is intentionally small

A document is only allowed here if it does one of four jobs:

1. changes code rules,
2. removes ambiguity,
3. tells operators exactly what to run,
4. records debt/risk/migration truth.

If a file does none of these, it is decoration.
