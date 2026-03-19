# ENGENE 2.0 Simulation Strategy

## Heavy simulation capability model

| Capability | Owner layer | Data model | Runtime algorithm | Fidelity mode | Perf strategy | Tooling/debug | Persistence |
|---|---|---|---|---|---|---|---|
| Layered materials | `engine_content` + `engine_physics` | Material stack, density, hardness, fracture energy | Layer traversal + energy attenuation | L0/L1 full, L2/L3 abstract | cache-friendly SOA tables | material inspector + penetration traces | save layer damage states |
| Ballistic penetration | `engine_physics` | Projectile profile + impact packet | swept ray + per-layer solve | L0 full, L1 reduced, L2 probabilistic | batch rays + SIMD + time slicing | trajectory overlay, impact log | persist projectile events optional |
| Fracture/destruction | `engine_physics` + `engine_world` | Break graph + support links + debris budget tags | hybrid break-state + stress thresholds | L0 local detailed, L1 stateful, L2 baked outcome | debris cap + sleep + pooling | fracture heatmap, support graph view | persist break-state node IDs |
| Body damage/dismember | `engine_physics` + `game_framework` | Body zones, limb graph, gore variants | hit resolution + threshold state machine | L0 full, L1 simplified | cap gore actors + pooled decals | body damage debugger | persist injury state |
| Fire + wind | `engine_world` | Cell burn state, fuel, moisture, wind vector | cellular spread + stochastic ignition | L0/L1 active cells, L2 summary fronts | active-cell budget + tick decimation | fire front overlay | persist burn progress per cell |
| Rain/wetness/leaks | `engine_world` | Wetness scalar maps + container fill + puncture edges | source/sink flow approximation | L0 local dynamic, L2 coarse reservoir | low-res grids + event-driven updates | wetness/leak debug panel | persist wetness + container volume |
| Weather fronts | `engine_world` + `engine_render` | Weather cells + front metadata | advection + noise-driven blending | Local weather full; far-field visual model | coarse global grid + async updates | weather map + timeline | persist weather seed/phase |
| Population 10k/5k | `engine_runtime` + `game_framework` | Agent archetypes + squads + region stats | L0 behavior tree; L1 squad sim; L2 region combat model; L3 statistical | mixed-level by distance/relevance | level-of-sim scheduler + capped promotions | sim-level overlay, agent counters | persist aggregate + promoted entities |
| Audio propagation | `engine_audio` | emitters, obstruction volumes, material acoustic tags | ray-cone approximation + zone routing | L0 near-field; L1 simplified; L2 ambience only | voice budget + priority mixer | audio rays + occlusion monitor | persist ambience state optional |

## What is too expensive to simulate literally
- Global rigid-body collapse chain reactions across entire world.
- True fluid volumetrics for every leak/puddle.
- Exact atmospheric simulation for planet-scale weather.
- Full-agent decision simulation for all 15k actors every frame.

## Required approximations
- Far-field destruction reduced to integrity states and authored collapse outcomes.
- Water as height/volume fields with event-driven leaks.
- Weather as moving macro-cells with local refinement only near camera/active gameplay.
- Population as multi-level abstraction with promotion into high-fidelity bubble.

---

## WORLD-SCALE SIMULATION STRATEGY

### L0-L3 model
- **L0 (Immediate bubble, ~0-80m):** full interactions, projectiles, destruction, detailed AI, local audio.
- **L1 (Near field, ~80-300m):** reduced physics, grouped AI, deterministic event summaries.
- **L2 (Regional, loaded but distant):** probabilistic combat/economy/social ticks per region.
- **L3 (Global, unloaded):** coarse world-state deltas (population, weather, fire risk, structural state tags).

### Near-field vs far-field
- Near-field prioritizes player-observable causality.
- Far-field prioritizes continuity and believable outcomes, not exact micro-events.

### Promotion/demotion rules
- Promote to higher fidelity when player proximity, interaction priority, or tactical relevance crosses threshold.
- Demote after cooldown when interaction entropy is low.
- Promotion budget is capped per frame to avoid spikes.

### Reconciliation policy (L0↔L1↔L2↔L3)
- Transition uses authoritative snapshots plus deterministic seed reconstruction.
- Conflicts resolved by authority precedence: local active truth > regional aggregate > global aggregate.
- Reconciliation emits explicit summary events for observability.

### Combat abstraction outside active bubble
- Squad-vs-squad model using doctrinal stats, terrain modifiers, morale, supply.
- Event summaries feed promotion system when player approaches.

### Economy/social abstraction outside active bubble
- Region graph updates resources, migration pressure, faction control, unrest.
- Periodic stochastic events (raids, shortages, diplomacy shifts).

### Weather abstraction
- World grid (macro fronts) updated at coarse timestep.
- Local bubble refines precipitation/wind to gameplay resolution.

### Fire/wetness/destruction persistence
- Persist only authoritative state vectors: burn %, wetness, structural integrity tags, container volume.
- Reconstruct transient visuals (smoke/debris particles) on load.

### Save/load semantics by layer
- L0/L1: save explicit entity/component state.
- L2/L3: save aggregate counters + deterministic seeds.
- Load path reconstructs lower layers from nearest authoritative snapshot.

### Streaming + persistence + simulation interaction
- Streaming enters region -> promote L2/L3 summaries into L1/L0 entities.
- Streaming exits region -> demote entities into aggregates with reconciliation pass.
- Persistence snapshots at region granularity with deterministic seeds for replayable reconstruction.
