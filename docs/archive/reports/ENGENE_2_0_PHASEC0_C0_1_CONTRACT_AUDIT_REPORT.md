# ENGENE 2.0 — Phase C0.1 Contract Audit Report

## Phase C0 definition
Phase C0 is a mandatory pre-Phase-C entry gate: **Physics / Content Truth Stabilization**. No Phase C destruction/material feature expansion is allowed until C0 passes.

## C0 acceptance contract (from task)
- `cargo test --test physics_body_combat -- --nocapture` = PASS
- `cargo test --test vertical_slice -- --nocapture` = PASS
- collision contract documented and aligned
- trajectory contract documented and aligned
- doctor boot errors resolved or contractually justified with same-PR doc updates
- material truth singular and auditable
- no new Phase C feature scope added

## Scope for C0.1
- Audit-only truth pass (contracts + evidence + blocker ordering)
- No feature additions
- No refactor

## Required command evidence (executed)
- `cargo test --test physics_body_combat -- --nocapture` -> FAIL (216 pass / 2 fail)
- `cargo test --test vertical_slice -- --nocapture` -> FAIL (5 pass / 1 fail)
- `rg "check_overlap\(" src tests crates`
- `rg "analytical_trajectory|ballistics_trajectory_direction" src tests crates`
- `rg "materials.ron|surfaces.ron|schema_version" game crates src tests docs`
- `rg "run_doctor|DoctorMode::Advisory|error_count" src tests crates`
- `rg "pub mod " src/lib.rs crates/*/src/lib.rs`
- supplemental extraction: `cargo run --bin engene_game` (used to print advisory doctor diagnostics)

## Mandatory blocker order audit

### 1) True contract of `check_overlap()`
- Implementation contract is circle-circle overlap predicate with one radius argument:
  - computes squared distance between centers
  - compares against `(radius * 2.0)^2`
- File evidence: `src/physics/collision.rs`.

### 2) `check_overlap_boundary`: test wrong or implementation wrong?
- Current failing assertion: `check_overlap(0.0, 0.0, 3.9, 0.0, 1.0)` expected `true`.
- Under declared function contract above, expected result should be `true` because `3.9 < 4.0` threshold.
- But runtime evidence shows it returns `false` in integration test run.
- C0.1 finding: **contract/behavior misalignment exists**; this blocker is real and unresolved in audit-only pass.
- C0.2 target: instrument/verify compiled path and patch the smallest honest fix in code or test (whichever proof supports).

### 3) True contract of `analytical_trajectory()`
- Contract from implementation:
  - starts with normalized `dir * speed`
  - advances with coarse Euler integration (`coarse_dt=0.05`) under gravity + drag
  - appends points until max distance or low speed
- File evidence: `src/physics/ballistics.rs`.

### 4) `ballistics_trajectory_direction`: test wrong or implementation wrong?
- Failing assertion expects second point to have non-decreasing x for +X launch direction.
- Given present integration formula + parameters (`mass=0.01`, `drag=0.3`), first update can reverse x velocity immediately due large drag acceleration.
- C0.1 finding: **implementation likely violates intuitive forward-progress contract for this API/test use** (coarse-step + drag term dominates instantly).
- C0.2 target: smallest fix should align trajectory semantics and test expectations without softening thresholds.

### 5) Exact 3 doctor errors behind `vertical_slice_engine_boots`
Extracted from advisory doctor output via `cargo run --bin engene_game`:
1. `game/data/rules.ron`: parse error (`Expected string`).
2. `game/data/weapons.ron`: missing field `damage` in `WeaponConfig`.
3. `game/data/species.ron`: missing field `hunting` in `EcosystemNeeds`.

### 6) Material truth split audit (`materials.ron` / `surfaces.ron`)
- `game/data/materials.ron` uses `schema_version: 1` and simple material map.
- `game/data/surfaces.ron` uses `schema_version: 2` and expanded material model with broader physical/destruction attributes.
- Runtime load logs show both are consumed (`materials: 16`, `surfaces: 16`), indicating dual material truths in active config surface.
- C0.1 finding: **split material truth risk is real** and blocks schema-law singularity.

### 7) Root-crate/module exposure controllability audit
- Root crate exports broad subsystem surface (`src/lib.rs` many `pub mod` entries).
- Workspace crates exist, but root remains a large exposure hub.
- C0.1 finding: controllability risk present; no further coupling increase introduced in this batch.

## Per-stage closure status
- **C0.1: PASS (audit-only completed with explicit blocker truth map).**
- **C0.2: FAIL (not executed in this batch by sequence rule).**
- **C0.3: FAIL (not executed in this batch by sequence rule).**

## Phase C expansion decision
- **Phase C remains blocked.**
- Expansion is not permitted until C0.2 and C0.3 close and C0 acceptance criteria are met.
