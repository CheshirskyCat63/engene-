# ENGENE_2_0_GOD_TIER_ARCHITECTURE

**Status**: Target architecture. Not aspiration — the destination after handoff.
**Authority**: This file defines the final form. All code must converge here.

---

## 1. Core Principle

**Role-first architecture.**

ENGENE does not think: "I am engine with physics/render/audio/debug_ui/full".
ENGENE thinks: "I am role_game", "I need cap_render, cap_physics, cap_audio", "surface tool_debug_ui", "run profile_release".

This is the only path to "installs in 2 seconds, runs like a beast".

---

## 2. Final Runtime Topology

```
Layer 1: App Shells (apps/*)
    │
    ├── parse args
    ├── load config
    ├── select role + profile + capabilities
    └── call product owner crate
    │
    ▼
Layer 2: Product Owners (sdk_app, game_framework)
    │
    ├── sdk_app: editor shell, inspectors, dashboards, overlays
    ├── game_framework: playable runtime, authored integration
    └── NO engine law, NO world truth ownership
    │
    ▼
Layer 3: Engine Crates
    │
    ├── engine_core: minimal contracts, events, scheduling, shared types
    ├── engine_runtime: phase graph, task scheduling, determinism surfaces
    ├── engine_ecs: storage, lifecycle, queries
    ├── engine_world: truth, spatial, persistence, streaming
    ├── engine_render: extraction + implementation (consumes truth)
    ├── engine_audio: runtime (independent of render)
    ├── engine_physics: physics runtime
    ├── engine_content: assets, pipeline
    └── engine_tools: doctor, audits, diagnostics
    │
    ▼
Layer 4: Root (if survives)
    │
    └── ONLY compatibility shell, NO new architecture
```

---

## 3. Final Crate Ownership

| Crate | Owns | Does NOT own |
|---|---|---|
| `engine_core` | contracts, events, scheduling primitives, shared types | editor logic, game logic, domain orchestration |
| `engine_runtime` | phase graph, task scheduling, deterministic tick surfaces | world truth, editor UX, game rules |
| `engine_ecs` | entity lifecycle, storage, component model, queries | simulation logic, rendering |
| `engine_world` | spatial truth, persistence, streaming, residency, transform/state truth | rendering, game rules |
| `engine_render` | extraction, render implementation | simulation truth |
| `engine_audio` | audio runtime | render success |
| `engine_physics` | physics runtime | game rules |
| `engine_content` | assets, pipeline, import/export | game logic |
| `engine_tools` | doctor, audits, diagnostics, migration | gameplay, SDK UX |
| `sdk_app` | editor shell, inspectors, dashboards, overlays, tooling-safe commands | simulation authority |
| `game_framework` | game bootstrap, authored integration, game-facing choices | editor-only panels |
| `apps/*` | parse args, load config, select role, call owner | domain logic |

---

## 4. Final Dependency Law

```
apps/* -> sdk_app / game_framework -> engine_* crates
root -> temporary adapters only
```

### Forbidden directions

| Layer | May NOT depend on |
|---|---|
| `engine_*` | `sdk_app`, `game_framework`, `apps/*` |
| `sdk_app` | `apps/*`, game-only authored runtime logic |
| `game_framework` | SDK/editor-only panels, overlays |
| `apps/*` | domain truth (they launch, configure, compose) |
| `engine_core` | editor logic, game logic, domain-heavy orchestration |
| `root` | new architecture (migration shell only) |

### Review rule

Every dependency change must answer:
1. Why does this dependency belong to this layer?
2. Why is the reverse direction forbidden?
3. Is this permanent or transitional?
4. What file proves the decision?

---

## 5. Final Launch Model

| Role | Canonical command |
|---|---|
| Game | `cargo run -p app_engene_game` |
| SDK | `cargo run -p app_engene_sdk` |
| Headless | `cargo run -p app_engene_headless -- --ticks 1200` |
| Tools | `cargo run -p app_engene_tools` |

### Transition path

1. `apps/*` become primary launch surface
2. Root bins become deprecated with removal timeline
3. Root crate becomes thin shell OR removed
4. Direct crate-to-crate ownership replaces root adapters

---

## 6. Final Execution Law (Phase Order)

**This is non-negotiable. No silent reordering.**

```
tick -> streaming -> persistence -> spatial -> audio -> editor_update -> render
```

### Why this order

| Phase | Reason |
|---|---|
| tick | Simulation truth changes first |
| streaming | World residency decisions against updated state |
| persistence | Loaded/unloaded transitions become durable |
| spatial | Derived structure catches up to truth changes |
| audio | Listener updates consume stable post-spatial state |
| editor_update | Editor mutation against known post-sim state |
| render | Consumes prepared state; does not define it |

### Phase rules

- Render must NOT own simulation truth
- Audio must NOT depend on render success
- Spatial must be fed from explicit dirty causes or explicit fallback
- Editor mutation must either mark dirty sources or force documented rebuild

---

## 7. Final Spatial Law

### Dirty input model

Explicit dirty causes:
- entity inserted
- entity moved
- entity removed
- chunk loaded
- chunk unloaded
- editor mutation changed transform
- origin shift / rebuild trigger
- recovery / safety rebuild request

### Update paths

**Incremental path** — use when dirty input is explicit and bounded:
- insert new entries
- update moved entries
- remove unloaded/despawned
- patch affected chunks/cells only

**Full rebuild fallback** — use when:
- dirty source is incomplete
- origin shift invalidates assumptions
- recovery mode requested
- verification path asks for rebuild

### Law

- Fallback is allowed
- Fallback without observability is NOT allowed

### Required telemetry

Every spatial update must be classifiable as:
- `incremental`
- `full_rebuild`
- `no_op`

Telemetry must include:
- dirty entity count
- dirty chunk count
- reason
- duration
- fallback reason (if any)

### Equivalence requirement

```
incremental(world, dirty_input) == rebuild(world_after_changes)
```

---

## 8. Final Storage/Data Law

### Data categories

| Category | Mutability | Ownership |
|---|---|---|
| truth data | mutable in tick | `engine_world` |
| derived/presentation data | derived from truth | respective domain crates |
| hot-path data | read-heavy, write-light | flat/SoA layout required |
| cold data | rarely accessed | heap-allocated acceptable |

### Forbidden patterns

| Pattern | Why forbidden |
|---|---|
| Hidden allocations in hot paths | destroys perf budget |
| Pointer chasing in tight loops | cache miss tax |
| Random cross-crate coupling | ownership violation |
| Mutable shared state without clear owner | architectural decay |

### Storage layout rules

- Hot data: flat/SoA, cache-line aligned
- Cold data: heap OK
- No hidden box/rc/arc in hot paths
- Explicit storage contract per crate

---

## 9. Final Test Architecture

| Lane | Tests | Gate |
|---|---|---|
| smoke | wiring, docs, launch, Cargo | `engine_contracts`, `production_candidate` |
| contract | ownership boundaries, runtime laws, determinism | `engine_contracts`, `determinism_and_sdk`, `runtime_systems` |
| certification | perf acceptance, throughput, tick budget | `certification_*` suite |
| perf | hot paths, benches, scaling | benches |

### Required invariant tests

- phase order invariant
- editor mutation propagation
- audio independent of render
- spatial incremental == rebuild equivalence

### Migration tests

Each must have:
- `reason`: why this test exists
- `owner`: who maintains it
- `sunset_condition`: when it can be removed
- `remove_after`: date or milestone

---

## 10. Final CI Gates

| Gate | What it checks | Failure action |
|---|---|---|
| build | compiles, no warnings | block merge |
| unit/integration | smoke + contract lanes pass | block merge |
| truth | ownership boundaries enforced | block merge |
| contract | runtime laws not violated | block merge |
| perf | within budget (cold build, startup, alloc/frame, tick) | block merge |
| doc consistency | no fantasy docs | block merge |
| forbidden dependency | no reverse dependency violations | block merge |

### Required metrics

- cold build budget
- startup budget
- alloc budget per frame
- scheduler overhead
- single-thread baseline
- 2/4/8-thread scaling
- deterministic replay capability
- hidden root dependency regression check

---

## 11. Final Removal Law

A thing is removable only when ALL three are true:

1. **Replacement exists** — documented in GAP_MAP or transition notes
2. **Replacement is validated** — tests pass, CI green
3. **Operators no longer rely on old path** — no active dependencies

If any are false, deletion is theatre.

### Current removal candidates

| Artifact | Replacement | Gate |
|---|---|---|
| root broad export surface | crate ownership + thin façade | all callers stop relying |
| root-package canonical entrypoint docs | package-owned app truth | apps/* become primary |
| `sdk_runner.rs` giant redraw | explicit phase methods | phase-order tests |
| `integration.rs` catch-all | boundary modules | modules compile, tests pass |
| full spatial rebuild in editor | dirty-input incremental | equivalence tests |

---

## 12. Forbidden Moves (永远不)

| Move | Why forbidden |
|---|---|
| Add new domain ownership to root | Root is migration shell |
| Create catch-all modules in root | Dependency law violation |
| Blur role/capability/profile in features | Feature taxonomy violation |
| Add new root bins without deprecation path | Entrypoint truth violation |
| Remove CI coverage of transition branch | Active branch must be protected |
| Add editor logic to engine_core | engine_core must stay minimal |
| Add game logic to engine_core | engine_core must stay minimal |
| Allow spatial full-rebuild without observability | Performance contract violation |
| Allow render to own simulation truth | Phase order violation |
| Allow audio to depend on render success | Phase order violation |
| Allow cross-domain catch-all hubs | Ownership violation |
| Describe split as complete when it isn't | Fantasy doc violation |

---

## 13. Relationship to Other Documents

- **CURRENT_RUNTIME_TRUTH.md** — where we are now
- **GAP_MAP_CURRENT_TO_GOD_TIER.md** — what blocks us
- **ENGENE_2_0_GOD_TIER_ARCHITECTURE.md** — where we're going (this file)

---

## 14. Exit Criteria (Handoff Complete)

This architecture is achieved when:

- [ ] `apps/*` are primary launch surface
- [ ] Root crate is thin shell OR removed
- [ ] Zero root dependencies in role crates
- [ ] `sdk_runner.rs` replaced with phase methods
- [ ] `integration.rs` split into boundary modules
- [ ] Feature taxonomy is role-first
- [ ] CI covers transition branch
- [ ] Spatial dirty model enforced with telemetry
- [ ] Phase order tests exist and pass
- [ ] All forbidden moves are compile-checked or CI-checked
- [ ] No fantasy docs remain (all docs reference CURRENT_RUNTIME_TRUTH)

---

## 15. Enforcement

This document is **law**. Violations must be caught by:
- compile-time boundaries
- contract tests
- CI gates
- doc consistency checks
- forbidden dependency checks

No doc may contradict this file. No code may violate this architecture.
This is not aspirational. This is the destination.
