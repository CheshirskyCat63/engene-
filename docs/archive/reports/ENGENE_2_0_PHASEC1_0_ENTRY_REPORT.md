# ENGENE 2.0 — Phase C1.0 Material Penetration Truth Entry Batch

## C1.0.A — Penetration contract audit

### 1) Exact penetration contract
- Penetration decision is energy-threshold based:
  - threshold = `penetration_resistance * 1000.0`
  - `proj.energy > threshold` => `Penetrated`
  - otherwise => `Stopped`
- On penetration:
  - `energy_lost = threshold`
  - `exit_vel = proj.vel * sqrt((energy - threshold) / energy)`

### 2) Exact ricochet contract
- Ricochet occurs when both are true:
  - shallow-impact proxy: `cos_angle < 0.3`
  - hardness gate: `hardness > 0.5`
- Ricochet response:
  - reflected direction from hit normal
  - `new_vel = reflected * speed * 0.6`
  - `energy_lost = energy * (1 - cos_angle) * hardness`

### 3) Authoritative fields for C1 penetration
- Authoritative ballistic fields: `hardness`, `penetration_resistance`, `density` (from canonical surfaces runtime projection).
- `response_class` is authoritative for fracture/material-response pipelines, not currently used in ballistic `resolve_impact` branch predicates.

### 4) Exact runtime consumers
- `WeaponsPlugin` populates `BallisticsSystem.material_table` from `config.surfaces.materials`.
- `BallisticsSystem::resolve_impact` consumes `MaterialProps` from that table.
- penetration/ricochet tests in `tests/physics_body_combat.rs` assert contract behavior.

### 5) Allowed files touched in C1.0 batch
- `tests/vertical_slice.rs`
- `tests/physics_body_combat.rs`
- `docs/canonical/ENGENE_2_0_PHASEC1_0_ENTRY_REPORT.md`

### 6) Forbidden files for this batch
- collapse/debris/nav integration systems
- broad destruction expansion
- gameplay feature additions

### 7) C1.0 acceptance criteria
- Runtime ballistic material physics remains anchored to canonical surfaces truth.
- Penetration/ricochet contracts are explicit and regression-guarded by tests.
- Required C1.0 commands pass.

---

## C1.0.B — Regression guard added
- Added integration guard test proving ballistic material physics comes from `surfaces.ron` canonical values and not silent fallback to legacy `materials.ron` values.
- Test: `vertical_slice_ballistics_material_truth_comes_from_surfaces`.

## C1.0.C — MaterialTruthService authored-gap verdict
Verdict: **ACCEPTABLE FOR NOW**.

Reason:
- `MaterialTruthService::empty()` currently affects presentation bridge queries (render/audio/particle mapping fallback), not the ballistic penetration truth path enforced in C1.0.
- Ballistic penetration truth is now guarded against fallback to legacy material source.
- Smallest next batch if this changes: wire authored material bridge into runtime assembly without touching penetration mechanics.

## C1.0.D — Narrow penetration evidence improvements
- Added focused penetration threshold contract test:
  - `impact_penetration_threshold_contract`
  - asserts stop-below-threshold and penetrate-above-threshold behavior.

## C1.0 status
- **PASS**
- Phase C1 may continue into deeper penetration/material response work, while keeping current scope constraints (no collapse/debris/nav yet).
