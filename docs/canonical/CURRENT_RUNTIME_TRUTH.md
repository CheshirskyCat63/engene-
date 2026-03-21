# CURRENT_RUNTIME_TRUTH

**Status**: Transition-shaped repo. Split declared; split not fully enforced.
**Last verified**: 2026-03-20
**Purpose**: Freeze current reality. No aspiration. No "we will". Only "is".

---

## 1. Purpose

This document records **what actually runs today**, not what should run after handoff.
It is the anti-fantasy layer: any doc that describes aspirational state must reference this file as the baseline.

If a claim in any other doc contradicts this file, **this file wins**.

---

## 2. Current Entrypoint Truth

### Canonical Operator Commands (package-based)

| Runtime role | Canonical command | First called crate | Runner location | Status |
|---|---|---|---|---|
| Game | `cargo run -p app_engene_game` | `game_framework` | `crates/game_framework/src/lib.rs` | transitional |
| SDK | `cargo run -p app_engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | transitional |
| Headless | `cargo run -p app_engene_headless -- --ticks 1200` | `game_framework` | `crates/game_framework/src/lib.rs` | transitional |
| Bootstrap | `cargo run -p engene_bootstrap` | `engene_bootstrap` | `apps/engene_bootstrap/src/main.rs` | app shell |

### Root Bins (compatibility only)

| Command | Status |
|---------|--------|
| `cargo run --bin engene_game` | deprecated, compatibility only |
| `cargo run --bin engene_sdk` | deprecated, compatibility only |
| `cargo run --bin engene_headless` | deprecated, compatibility only |
| `cargo run --bin engene_tools` | deprecated, compatibility only |

### Active Runtime Implementation

- **SDK runtime**: `crates/sdk_app/src/lib.rs` — stub, pending phase extraction
- **Game runtime**: `crates/game_framework/src/lib.rs` — transitional
- Actual runtime logic is still distributed across:
  - `sdk_app::lib_complex` (draft, depends on non-existent engene crate)
  - Root's sdk_runner (still contains orchestration)

### Transition Status

- Phase extraction in progress (engine_runtime::phase)
- Editor slice extracted to sdk_app::editor
- Full handoff pending

---

## 3. Current Crate Ownership Truth

### Real owners (already functioning)

| Crate | Owns | Status |
|---|---|---|
| `engine_core` | contracts, events, scheduling primitives | functional |
| `engine_ecs` | entity lifecycle, storage, component model | functional |
| `engine_world` | world truth, spatial, persistence, streaming | functional |
| `engine_render` | render extraction + implementation | functional |
| `engine_physics` | physics runtime | functional |
| `engine_audio` | audio runtime | functional |
| `engine_content` | assets, content pipeline | functional |
| `engine_tools` | doctor, diagnostics, audits | functional |
| `game_framework` | playable runtime composition | transitional — depends on root |
| `sdk_app` | editor shell, inspectors, dashboards | transitional — depends on root |

### App shells (thin launchers only)

| App | Purpose | Status |
|---|---|---|
| `apps/engene_game` | game launch wrapper | transitional shell |
| `apps/engene_sdk` | SDK launch wrapper | transitional shell |
| `apps/engene_headless` | headless launch wrapper | transitional shell |
| `apps/engene_bootstrap` | bootstrap entrypoint | transitional shell |

### Root crate

| Responsibility | Status |
|---|---|
| Root package `.` | migration shell — still hosts active bins |
| Root re-exports | broad monolith module tree — deprecated |
| Root bins | `engene_game`, `engene_sdk`, `engene_headless`, `engene_tools` — still active for compatibility |

---

## 4. Current Root Limitations

Root crate **may not**:
- own new domain logic
- define new permanent runtime subsystems
- grow cross-boundary orchestration
- create new catch-all exports
- define new feature bundles that blur roles

Root crate **still does** (transition debt):
- hosts active binaries for compatibility
- re-exports broad monolith module tree
- serves as temporary adapter between legacy and workspace layout
- role crates depend on root for:
  - `engene::core::build_manifest::BuildManifest`
  - `engene::core::crash_telemetry`
  - `engene::runtime::bootstrap::*`
  - `engene::world::heightmap::Heightmap`
  - `engene::world::world::WorldGrid`
  - `engene::app::game_runner::GameApp`
  - `engene::app::spatial_dirty_journal::SpatialDirtyJournal`
  - `engene::world::components::*`
  - `engene::world::hierarchical_spatial::SpatialUpdatePath`

---

## 5. Transitional Scaffolds Still Allowed

The following are **tolerated temporarily** but must have removal conditions:

| Scaffold | Purpose | Removal condition |
|---|---|---|
| Root re-exports | compatibility façade | role crates stop depending on root |
| Root active bins | operator continuity | apps/* become primary launch surface |
| Role crate root dependencies | migration adapters | direct crate-to-crate ownership |
| `sdk_runner.rs` orchestration | world-kitchen phase driver | explicit phase methods replace redraw handling |
| `integration.rs` catch-all | domain wiring hub | boundary modules replace mixed-domain file |

---

## 6. Current Dependency Direction

```
apps/*  ->  sdk_app / game_framework  ->  engine_* crates
root (temporary adapters only)
```

**Forbidden directions** (already enforced):
- `engine_*` may NOT depend on `sdk_app`, `game_framework`, `apps/*`
- `sdk_app` may NOT depend on `apps/*` or game-only authored logic
- `game_framework` may NOT depend on SDK/editor-only overlays

---

## 7. Forbidden Regressions (must never happen)

| Regression | Why forbidden |
|---|---|
| Adding new domain ownership to root | Root is migration shell, not architecture center |
| Creating new catch-all modules in root | Dependency law violation |
| Blurring role/capability/profile in Cargo features | Feature taxonomy must stay clean |
| Adding new root bins without deprecation path | Entrypoint truth must stay single-source |
| Removing CI coverage of transition branch | Active branch must be CI-protected |
| Adding editor logic to engine_core | engine_core must stay minimal |
| Adding game logic to engine_core | engine_core must stay minimal |
| Allowing spatial full-rebuild without observability | Performance contract violation |
| Allowing render to own simulation truth | Phase order violation |
| Allowing audio to depend on render success | Phase order violation |

---

## 8. Current Phase Order (enforced)

```
tick -> streaming -> persistence -> spatial -> audio -> editor_update -> render
```

**No silent reordering allowed.**

---

## 9. Current Test Lanes

| Lane | Purpose | Targets |
|---|---|---|
| smoke | wiring, docs, launch, Cargo | `engine_contracts`, `production_candidate` |
| contract | ownership boundaries, runtime laws | `engine_contracts`, `determinism_and_sdk`, `runtime_systems` |
| certification | perf acceptance, throughput | `certification_boundary_overhead`, `certification_kernel_throughput`, `certification_tick_budget`, `certification_perf_snapshot` |
| perf | hot paths, benches | `certification_perf_snapshot`, benches |

---

## 10. Exit Conditions for This Document

This document becomes obsolete when ALL of:

- [ ] `apps/*` become primary launch surface (no root bins as canonical)
- [ ] role crates (`game_framework`, `sdk_app`) have zero root dependencies
- [ ] root crate is either thin shell or removed
- [ ] `sdk_runner.rs` replaced with explicit phase methods
- [ ] `integration.rs` replaced with boundary modules
- [ ] CI covers transition branch as first-class

Until then, **this file is the source of truth for current runtime state**.

---

## 11. Mandatory Reference

Any document that describes runtime behavior must include:

```
Current runtime truth: see docs/canonical/CURRENT_RUNTIME_TRUTH.md
```

No doc may claim "this is how it works" without linking to this file.
