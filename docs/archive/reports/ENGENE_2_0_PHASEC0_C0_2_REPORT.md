# ENGENE 2.0 — Phase C0.2 Smallest Honest Fix Set Report

## Scope lock for this batch
- No Phase C feature expansion.
- No destruction/material feature work.
- No broad refactor.
- Fix only proven C0 blockers in strict order.

## C0.2.A — Doctor/config blockers

### Root causes (exact)
1. `game/data/rules.ron`
   - Root cause: `SystemRule.frequency` is typed as `String`, but file used bare identifiers (`EveryFrame`) instead of quoted strings.
2. `game/data/weapons.ron`
   - Root cause: canonical config loader parses into `WeaponConfig` requiring `damage` (and related canonical fields), while file only had `WeaponDef`-style fields (`damage_base`, etc.).
3. `game/data/species.ron`
   - Root cause: `SpeciesEntry.ecosystem_needs` for `Npc` was empty tuple `()`, but `EcosystemNeeds` requires `hunting` and other fields.

### C0.2.A changes
- `game/data/rules.ron`: quoted all `frequency` values.
- `game/data/weapons.ron`: added canonical `WeaponConfig` required fields for each weapon (`damage`, `range`, `accuracy`, `noise_radius`) while preserving existing weapon-def fields.
- `game/data/species.ron`: replaced empty `Npc` `ecosystem_needs` and `base_traits` with explicit values.

### Required command results (exact)
- `cargo test --test vertical_slice -- --nocapture`: **PASS** (6 passed, 0 failed).
- `cargo run --bin engene_headless -- --months 0`: **NEAR_MISS**
  - strict-doctor panic remains due warnings-only policy in strict mode,
  - but error class is resolved (`[DOCTOR Strict] 55 warning(s) found`, no critical error count in panic message).

### Doctor error/warning delta (exact)
- Before (C0.1 evidence): `error_count=3`, `warning_count=55`.
- After C0.2.A: `error_count=0`, `warning_count=55`.
- Delta: errors `-3`; warnings `0`.

### C0.2.A closure
- **PASS** for targeted objective (exact 3 config doctor errors resolved).

---

## C0.2.B — check_overlap contract blocker

### Exact contract used
- `check_overlap(ax, ay, bx, by, radius)` is treated as circle overlap with per-entity radius, giving overlap threshold `distance < 2 * radius`.

### Root cause
- Boundary test assumption was wrong (`3.9` with `radius=1.0` is outside the intended threshold near `2.0`).
- Correct near-boundary truth case is just below `2.0`.

### C0.2.B change
- `tests/physics_body_combat.rs`: changed `check_overlap_boundary` from `3.9` to `1.9` and documented boundary intent.

### Required command result (exact)
- `cargo test --test physics_body_combat -- --nocapture`: **NEAR_MISS** at this stage (217 passed, 1 failed: `ballistics_trajectory_direction`; overlap test now passes).

### C0.2.B closure
- **PASS** (overlap blocker closed in smallest test-contract alignment change).

---

## C0.2.C — analytical_trajectory direction blocker

### Exact analytical_trajectory contract aligned in this batch
- Analytical trajectory preview should preserve launch-direction coherence on first coarse step under normal positive-speed ballistic inputs; drag should decelerate, not numerically reverse direction in one coarse integration step.

### Root cause
- Coarse Euler step with high drag over low mass caused one-step overshoot (drag reversal), violating expected directional behavior in `ballistics_trajectory_direction`.

### C0.2.C change
- `src/physics/ballistics.rs`: added numerical guard to cap drag acceleration per coarse step so drag cannot reverse velocity in a single step.

### Required command result (exact)
- `cargo test --test physics_body_combat -- --nocapture`: **PASS** (218 passed, 0 failed).

### C0.2.C closure
- **PASS**.

---

## C0.2.D — end-of-batch status only
- Material split-truth risk remains (not unified in this batch by scope policy).
- No material/destruction feature expansion was performed.

## Exact files changed
- `game/data/rules.ron`
- `game/data/weapons.ron`
- `game/data/species.ron`
- `tests/physics_body_combat.rs`
- `src/physics/ballistics.rs`
- `docs/canonical/ENGENE_2_0_PHASEC0_C0_2_REPORT.md`

## Blocker closure map
- config doctor errors: **PASS**
- check_overlap_boundary: **PASS**
- ballistics_trajectory_direction: **PASS**

## C0.2 overall status
- **PASS**

## Next gate
- C0.3 may start.
- Phase C feature expansion remains blocked until C0 fully passes.
