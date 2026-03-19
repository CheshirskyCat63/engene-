# ENGENE 2.0 — Phase C0.4 Material Truth Singularity Closure

## C0.4.A — Exact material truth map

### 1) Exact semantic role of `materials.ron`
- Legacy/simple material projection payload (`flammability`, `fuel`, `hardness`, `penetration_resistance`, `density`).
- In C0.4 it is treated as compatibility artifact, not canonical runtime source of truth.

### 2) Exact semantic role of `surfaces.ron`
- Canonical material semantics for runtime-facing systems:
  - response class,
  - strength/brittleness/density,
  - ballistic resistance/hardness,
  - fracture/debris/decal semantics,
  - fire/wetness-related physical parameters.

### 3) Exact semantic role of `material_bridge.ron`
- Projection mapping layer from canonical material IDs to render/audio/particle mappings.
- It should not redefine physical material semantics; it maps them to presentation domains.

### 4) Exact overlap / split-meaning before C0.4
- `materials.ron` and `surfaces.ron` both carried ballistic-critical fields (`hardness`, `penetration_resistance`, `density`).
- Runtime ballistic table previously consumed `config.materials` while destruction/fracture paths depend on surface semantics.
- This created parallel material meaning.

### 5) Runtime consumers before C0.4
- `GameConfig::load_from_dir` loaded both `materials.ron` and `surfaces.ron` into independent structures.
- `WeaponsPlugin` built ballistics material table from `config.materials` (simple map path).
- Surface/destruction systems use surface-domain types (`SurfaceMaterial`/`ResponseClass`).

### 6) Canonical owner that should exist
- Canonical owner: `surfaces.ron` material table (`SurfacesConfig.materials`) as singular runtime material truth.
- `materials.ron` becomes strict projection/compatibility data only.

### 7) Files allowed to touch for C0.4.B
- `src/core/game_config.rs`
- `src/game/weapons_plugin.rs`
- `docs/canonical/ENGENE_2_0_PHASEC0_C0_4_REPORT.md`

### 8) Files forbidden to touch for C0.4.B
- destruction/gameplay feature systems
- broad ECS/runtime refactors
- doctor policy paths unrelated to material truth

---

## C0.4.B — Ownership fix executed (smallest honest)

### Root cause
- Ballistic runtime consumed `config.materials` while broader material semantics lived in `config.surfaces.materials`; overlapping semantic truth existed.

### Ownership decision
- Singular runtime truth for material physical semantics is now `SurfacesConfig.materials`.

### Changes
1. `GameConfig::load_from_dir`
   - stopped assigning runtime truth from `materials.ron` directly.
   - now projects `config.materials` from `config.surfaces.materials` after surfaces load.
2. `WeaponsPlugin`
   - now builds ballistic material table directly from `config.surfaces.materials`.

### Runtime consumers before vs after
- Before:
  - ballistic runtime source = `config.materials` (simple file path semantics).
- After:
  - ballistic runtime source = `config.surfaces.materials` (canonical surface semantics).
  - `config.materials` is a derived projection from canonical surfaces.

---

## C0.4.C — Truth checks
- `cargo test --test physics_body_combat -- --nocapture`: PASS
- `cargo test --test vertical_slice -- --nocapture`: PASS
- `cargo run --bin engene_headless -- --months 0`: PASS
- required `rg` material-truth scan executed.

## Exact files changed
- `src/core/game_config.rs`
- `src/game/weapons_plugin.rs`
- `docs/canonical/ENGENE_2_0_PHASEC0_C0_4_REPORT.md`

## Closure status
- material truth singularity: **PASS**
- hidden runtime fallback risk: **PASS** (ballistic runtime no longer reads parallel simple-material truth)
- test/runtime alignment: **PASS**
- full C0: **PASS**

## Phase gate decision
- Phase C feature expansion may begin now from C0-gate perspective.
- This report does not start C1 work.
