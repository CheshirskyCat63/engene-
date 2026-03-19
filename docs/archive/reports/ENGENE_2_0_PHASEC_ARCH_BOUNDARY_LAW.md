# ENGENE 2.0 — Phase C Architecture Boundary Law (Canonical)

Status: **BINDING**  
Scope: Phase C simulation foundation ownership, truth, wiring, LOD, persistence, and gating.  
This document is a normative law, not an implementation plan.

---

## 1) Hard NO to monolithic physics

ENGENE 2.0 explicitly forbids a monolithic "physics owns everything" model.

Destruction, fire, water, cloth, weather/fields, smoke, materials, and render projections **must not** be merged into one giant physics owner.

Reason this is forbidden:
1. It destroys canonical truth boundaries and enables split-truth regressions.
2. It blocks scalable LOD budgeting by coupling unrelated solver costs.
3. It hides integration defects behind broad subsystem ownership.
4. It makes persistence and reconstruction policy ambiguous.
5. It permits render-facing data to leak into simulation truth.

Binding consequence: physics core remains a domain owner, not a dumping ground.

### 1.1) Physics Core Definition

Physics core / physical engine kernel includes:
1. rigid body motion integration
2. collision detection and contact generation
3. constraint solving
4. shared stepping infrastructure
5. shared physical query infrastructure (raycasts/sweeps/overlaps and equivalent canonical query interfaces)
6. deterministic physical field sampling interfaces used by solvers that require physical environment inputs

Physics core does **not** own canonical truth for:
- fire
- water
- destruction
- weather/world fields
- smoke
- authored material truth
- surface-state/wetness truth
- render particles/reflections/fog/sky truth
- gameplay orchestration

Binding statement:
- Physics core is shared physical solving infrastructure, not the universal owner of every physically-looking phenomenon.

### 1.2) Temporary Co-Location Does Not Change Ownership

Implementation storage location does not redefine canonical ownership.

Binding rules:
1. A domain temporarily hosted inside `PhysicsSystem` is not therefore canonically owned by `PhysicsSystem`.
2. Temporary runtime co-location must never be used as architectural justification for future ownership expansion.
3. Ownership is defined by canonical responsibility and law, not by field placement in a struct/resource.
4. Transitional hosting is allowed only if canonical ownership remains explicit and enforced by contracts/guards.

---

## 2) Canonical ownership matrix

| Domain | Canonical truth owner | Solver owner | Projection / presentation owner | Persistence owner | Budget / degradation owner | Main consumers |
|---|---|---|---|---|---|---|
| physics core | Physics core runtime state (kinematics/collision) | PhysicsSystem/Rapier path | Render/Audio consume outputs | Entity/chunk persistence pipeline | QualityGovernor + sim LOD policy | AI, gameplay, ballistics, nav |
| ballistics | BallisticsSystem projectile/impact truth | Ballistics solver + BallisticsTick integration | Impact FX/audio projections | Ballistics persistence policy owner (defined in Persistence Law) | QualityGovernor + sim LOD | damage, destruction, AI |
| materials | Canonical authored surfaces/material truth | domain solvers query material truth | Material bridge projections (render/audio/particle mappings) | canonical config + runtime material persistence policy | validator + test gates + quality policy | ballistics, destruction, fire, water, render/audio |
| destruction | DestructionSystem structural truth | Destruction solver + DestructionTick integration | Occlusion/gore/render projections | Chunk/world persistence owner | budget registry + QualityGovernor | terrain deformation, nav, audio/occlusion, gameplay |
| terrain deformation | TerrainDeformationSystem patch truth | TerrainDeformationTick | Terrain mesh/update projection | Chunk persistence owner | budget registry + QualityGovernor | nav dirty, render, gameplay |
| fire | Fire domain truth (fire grid/state) | Fire solver | fire/smoke visual projection | session/chunk persistence owner | sim LOD + QualityGovernor | AI danger, damage, smoke |
| water | Water domain truth (levels/flow/container/leak) | Water solver | water visual/reflection/ripple projection | session/chunk persistence owner | sim LOD + QualityGovernor | wetness, buoyancy, gameplay, render |
| world fields / weather | canonical world field truth (wind/rain/air/anomaly) | weather/field solver | sky/fog/precip projection | world/chunk weather persistence owner | field LOD + QualityGovernor | ballistics, fire, water, smoke, render |
| smoke | smoke field truth (density/advection/decay) | smoke solver | particle/fog projection | session/chunk persistence owner | smoke LOD + particle budget policy | AI visibility, gameplay, audio masking, render |
| surface state / wetness | SurfaceStateStore truth | surface-state update/decay owner | surface render upload/projection | chunk/session persistence owner | store caps + QualityGovernor | render, gameplay traces |
| cloth | cloth particle/constraint truth | cloth solver | cloth render projection | entity/chunk persistence owner | cloth LOD iterations + QualityGovernor | gameplay collision (if used), render |
| render projections | renderer transient GPU/view truth only | renderer | renderer | never saved (transient only) | render budget + QualityGovernor | player presentation only |

---

## 3) Canonical truth definitions

### physics core
Canonical truth includes entity physical state and collision-resolved motion state used by gameplay and simulation.

### ballistics
Canonical truth includes projectile position/velocity/energy/mass/TTL and impact outcomes (stopped/ricochet/penetrated) and emitted canonical impact semantics.

### materials
Canonical truth includes material physical response constants and semantic response class.
Material bridge entries are canonical **projection mappings**, not canonical physical truth.

### destruction
Canonical truth includes structural integrity graph/topology, section/node/link integrity, failure mode/collapse state, and destruction consequence events.

### terrain deformation
Canonical truth includes deformation patches/stamps and current modified terrain state used by nav and gameplay.

### fire
Canonical truth includes per-cell/state burn status, fuel state, ignition state, temperature/spread state.

### water
Canonical truth includes per-cell terrain height coupling, water level, flow state, container state, and leak constraints.

### world fields / weather
Canonical truth includes wind field, rain/precipitation field, air-density field, anomaly field, and weather control parameters used by simulation.

### smoke
Canonical truth includes smoke density/advection/decay field used for gameplay/sensing; render particles are projections.

### surface state / wetness
Canonical truth includes wetness/scorch/dirt/blood/impact/crack persistence masks and decay state.

### cloth
Canonical truth includes cloth particle positions/previous positions, constraints, wind/gravity inputs, and solved constraint state.

### render projections
GPU buffers, render instance lists, particles, reflection textures, fog layers, and post-processing state are **not** canonical simulation truth.

### 3.1) Weather/Render Truth Clarification
Renderer may interpolate or project weather presentation, but may not originate canonical weather truth.

### 3.2) Smoke Truth Clarification
Smoke particles without canonical smoke field are presentation-only and must not be treated as gameplay smoke truth.

---

## 4) Interaction law

Cross-domain interaction rule:
- Read access is allowed to canonical truth where declared.
- Writes to another domain’s canonical truth are forbidden except through that domain owner’s API or canonical events consumed by that owner.
- Direct cross-domain mutation is forbidden.

### 4.1) Game-Facing Contract Law

Game layer MAY:
1. query stable public contracts
2. send intents/requests/events through canonical interfaces
3. consume canonical outcomes
4. depend on stable semantic APIs

Game layer MAY NOT:
1. mutate solver internals directly
2. depend on temporary storage layout
3. write canonical state by arbitrary mutable resource grabs
4. use render-only transient state as gameplay truth

Binding statement:
- game code must never couple directly to solver internals.

### 4.2) Narrow API Law

Cross-domain API rules are binding:
1. Cross-domain APIs must be narrow and stable.
2. No raw internal mutable container exposure across domains.
3. No semantic ownership leakage through convenience getters.
4. Event payloads must be semantic contracts, not internal-state dumps.
5. Owner domains define mutation semantics and admissible transitions.

### Mandatory interaction pairs

1. **world fields -> ballistics**
   - Read: ballistics may read canonical world fields.
   - Write: ballistics may not write world fields.
   - Write path requirement: N/A (read-only dependency).
   - Forbidden: constructing local default fields when canonical world fields are declared.

2. **world fields -> fire**
   - Read: fire solver may read canonical wind/rain fields.
   - Write: fire may not write world fields.
   - Forbidden: render-local weather replacing fire weather input.

3. **world fields -> water**
   - Read: water solver may read canonical rain/weather fields.
   - Write: water may not write world fields.
   - Forbidden: standalone local rain truth disconnected from canonical world fields.

4. **world fields -> smoke**
   - Read: smoke solver may read canonical wind/weather fields.
   - Write: smoke may not write world fields.
   - Forbidden: particle-only smoke pretending to be gameplay smoke truth.

5. **materials -> ballistics**
   - Read: ballistics may read canonical material physical truth.
   - Write: ballistics may not mutate material truth.
   - Forbidden: projection bridge redefining penetration/ricochet physics constants.

6. **materials -> destruction**
   - Read: destruction may read canonical material response truth.
   - Write: destruction may not mutate material truth.
   - Forbidden: destruction-local hardcoded material truth overriding canonical source.

7. **destruction -> terrain deformation**
   - Read: terrain deformation may consume destruction outputs/events.
   - Write: terrain deformation owner writes deformation truth.
   - Write path requirement: via canonical event chain or owner API only.
   - Forbidden: direct mutation of terrain truth by non-owner systems.

8. **terrain deformation -> nav dirty**
   - Read: nav dirty owner may read terrain-changed events.
   - Write: nav dirty owner writes nav dirty truth.
   - Write path requirement: owner API/event-driven only.
   - Forbidden: terrain deformation directly mutating nav internals.

9. **fire -> smoke**
   - Read: smoke solver may read fire outputs/events.
   - Write: smoke owner writes smoke truth.
   - Write path requirement: owner API/event chain.
   - Forbidden: fire directly writing smoke field internals.

10. **rain -> wetness**
    - Read: surface-state owner may read canonical rain truth.
    - Write: surface-state owner writes wetness truth.
    - Forbidden: renderer wetness driving canonical wetness state.

11. **water -> wetness**
    - Read: surface-state owner may read water truth.
    - Write: surface-state owner writes wetness truth.
    - Forbidden: water solver directly mutating surface-state internals outside owner API/event.

12. **weather -> render sky/fog/precipitation**
    - Read: renderer may read canonical weather/world fields.
    - Write: renderer may not own canonical weather truth.
    - Forbidden: renderer-local weather becoming canonical simulation weather or replacing canonical weather authority.

13. **simulation truth -> reflections / particles / audio**
    - Read: projection systems may read canonical simulation truth.
    - Write: projection systems may not mutate canonical simulation truth.
    - Forbidden: projection layers redefining simulation outcomes.

---

## 5) Anti-fake-wiring law

The following are binding architectural rules:

1. If a system declares a canonical resource dependency, it must consume that canonical resource at runtime.
   - Silent local-default substitution is forbidden.

2. If authored runtime truth exists, an empty fallback may not be used as primary runtime path.

3. Fallback is legal only for missing data and must never outrank canonical data.

4. Render-local simulation/state may not replace canonical gameplay/simulation truth.

5. Doctor severity class does not reduce architectural seriousness.
   - INFO findings may still represent structural/correctness blockers.

6. Any new producer/consumer runtime route must carry a regression guard proving it is wired.

---

## 6) LOD law

L0/L1/L2/L3 policy is mandatory per domain.

### Global invariants (never degraded)
- correctness-critical truth
- identity correctness
- save correctness
- nav correctness
- collision correctness

### ballistics
- L0: full projectile/impact simulation in active zone.
- L1: reduced cadence/aggregation where safe, preserving impact correctness.
- L2: statistical/far approximation for non-critical distant trajectories.
- L3: background coarse progression or inactive when out of relevance window.

### destruction
- L0: full local structural solve.
- L1: reduced update cadence with bounded chain depth.
- L2: statistical structural progression.
- L3: background chunk-level aggregate only.

### fire
- L0: full cellular update.
- L1: decimated cadence.
- L2: statistical spread/burnout.
- L3: background/no active local solve.

### water
- L0: full local flow/container/leak simulation.
- L1: decimated cadence.
- L2: statistical accumulation/evaporation/flow approximation.
- L3: background aggregate only.

### weather/fields
- L0: full local sampling of canonical fields.
- L1: reduced temporal resolution.
- L2: coarse regional fields.
- L3: background coarse weather progression only.

### smoke
- L0: local advection/diffusion field + projection.
- L1: coarser field resolution.
- L2: scalar/aggregated smoke field only.
- L3: background dissipation model only.

### cloth
- L0: full or near-full iterations.
- L1: reduced iterations.
- L2: minimal/statistical/no-iteration fallback preserving gameplay invariants.
- L3: suspend or reconstruct-on-demand.

### surface state
- L0: full local updates + bounded dirty uploads.
- L1: reduced update frequency and upload budget.
- L2: coarse decay/aggregation only.
- L3: background decay only.

---

## 7) Persistence law

Each domain must declare one of: permanent, session, derived/transient, reconstruct-on-load, never saved.

Default persistence policy in this law is normative unless a stricter domain amendment supersedes it.

### destruction
- classification: **permanent**
- policy: persisted as chunk/world structural truth.

### terrain deformation
- classification: **session (default), permanent only if gameplay persistence explicitly requires it**
- policy: persist patch deltas; reconstruct geometry from deltas on load.

### fire
- classification: **session**
- policy: persist fire state for loaded/session chunks; not permanent world canon unless explicitly promoted by gameplay contract.

### water
- classification: **session by default**
- policy: persist water levels/containers/leak states in active/session scope only.
- explicit rule: persistent water is opt-in and container/entity scoped by gameplay need, not default persistence mode for all world water.

### surface state
- classification: **session**
- policy: persist wetness/scorch/blood/impact persistence in chunk session data.

### cloth
- classification: **derived/transient** by default
- policy: reconstruct-on-load from anchors/initial conditions unless explicit permanent gameplay cloth state is required.

### weather fields
- classification: **derived/transient + reconstruct-on-load**
- policy: store compact weather seeds/timeline only when required for continuity; rebuild runtime fields on load.

### smoke
- classification: **derived/transient**
- policy: reconstruct-on-load; no permanent save of simulation smoke by default.

### render projections
- classification: **never saved**
- policy: always rebuilt per frame/runtime start.

### 7.1) Ownership Migration Rule

When a domain moves from temporary host placement to final dedicated ownership, all of the following are mandatory in the same batch:
1. authority matrix update
2. persistence policy confirmation/update
3. regression guard update/addition
4. runtime assembly parity check update when affected
5. canonical documentation update

Binding statement:
- ownership migration without updated guards and authority statements is incomplete.

---

## 8) Test discipline law

Binding rule:

**“If it works and connects something, it needs a regression guard.”**

Required guard categories for Phase C:
1. canonical source wiring tests
2. fallback precedence tests
3. runtime assembly parity tests (vertical/headless/tools)
4. no silent default substitution tests/invariants
5. major cross-domain event-chain tests

Additional binding conditions:
- Runtime consumer of authored data requires test proving canonical source consumption.
- New producer/consumer relationship requires wiring test or doctor/validator invariant.
- Fallback path requires canonical-precedence test.

No regression guard = batch incomplete.

---

## 9) Immediate roadmap gating

### Work allowed now
- docs-only law/report batches
- narrow wiring-closure batches that enforce canonical truth and add regression guards
- memory/perf discipline batches that preserve behavior and add guards

### Work blocked now
- deeper feature growth in destruction/fire/water/weather/smoke/cloth until wiring-closure batches pass
- any batch introducing new cross-domain coupling without guard coverage

Binding gate:
- deeper fire/water/destruction/weather growth is blocked until truth/wiring closure batches pass acceptance criteria.
- this gate is mandatory and may not be relaxed by implementation convenience, temporary co-location, or diagnostic severity labeling.

---

## 10) Batch ordering after this law

Only the next smallest honest batches are authorized, in this order.

### C-WIRE-1 — material truth runtime honesty
- goal: canonical authored material truth is runtime primary path; fallback only for missing entries.
- allowed files: `src/core/material_truth.rs`, `src/game/runtime_assembly.rs`, `tests/vertical_slice.rs`, optional canonical doc update.
- forbidden files: unrelated simulation/feature files.
- acceptance criteria:
  - runtime no longer uses empty truth service as primary path
  - canonical mapping consumed in all runtime assemblies
  - fallback precedence guard present

### C-WIRE-2 — world fields canonical consumption
- goal: remove silent local default substitution in world-field consumers.
- allowed files: `src/game/integration_systems.rs`, `tests/runtime_systems.rs`, optional canonical doc update.
- forbidden files: renderer feature expansion, unrelated systems.
- acceptance criteria:
  - declared world-fields dependency consumed from canonical runtime resource
  - no local default substitution in canonical path
  - regression guard present

### C-WIRE-3 — weather -> fire/water/wetness minimum chain
- goal: minimal canonical weather coupling for fire/water/wetness truth.
- allowed files: minimal relevant simulation/integration files + tests + optional canonical doc update.
- forbidden files: deeper feature expansion, broad refactors.
- acceptance criteria:
  - canonical rain/wind truth reaches required consumers
  - rain/water to wetness chain wired through owner path
  - regression guards for each new route

### C-WIRE-4 — assembly parity + doctor invariant hardening
- goal: enforce parity across assembly modes and add invariants for fake-wiring detection.
- allowed files: `tests/runtime_systems.rs`, `src/tools/doctor.rs`, optional canonical docs.
- forbidden files: solver semantics/features.
- acceptance criteria:
  - parity tests for critical truth resources/routes across runtime assembly modes
  - doctor/validator invariant catches declared-resource-but-defaulted patterns

### C-PERF-1 — allocation discipline
- goal: remove avoidable per-frame allocation churn in hot paths while preserving truth behavior.
- allowed files: targeted hot-path runtime files + tests + optional canonical doc update.
- forbidden files: feature growth or architecture drift changes.
- acceptance criteria:
  - identified hot paths avoid repeated heap allocation where reusable buffers suffice
  - behavior-equivalence guards pass
  - no truth ownership regressions

---

End of law.
