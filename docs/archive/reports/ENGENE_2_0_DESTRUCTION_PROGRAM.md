# ENGENE 2.0 Destruction Program

## Model
- **True simulation:** projectile impact solve, local support damage accumulation, impulse propagation in active bubble.
- **Authored states:** pre-baked fracture maps, collapse archetypes, tile/concrete reveal transitions.
- **Hybrid runtime:** simulated triggers choose among authored break states with physically-plausible timing.

## Tile -> adhesive -> support -> concrete example
1. Projectile enters tile layer, loses energy by brittleness + thickness.
2. Adhesive layer may delaminate; chance scales with prior moisture/wetness.
3. Support frame receives residual impulse and structural HP reduction.
4. Concrete backing chips/spalls when impulse threshold reached.
5. Exposed concrete state modifies subsequent penetration and audio response.

## Phase plan

### D1: Material penetration core
- Deliverables: per-layer ballistic solver, material database integration, debug traces.
- Acceptance: repeatable penetration outcomes against test stacks.

### D2: Break graph + structural integrity
- Deliverables: support graph, localized stress accumulation, authored break-state triggers.
- Acceptance: walls/floors fail only when support logic threshold reached.

### D3: Explosive structural damage
- Deliverables: blast pressure falloff, impulse application, collapse archetypes.
- Acceptance: buildings can partially or fully collapse with deterministic bounds.

### D4: Debris and nav integration
- Deliverables: debris pooling/lifetime budget, navmesh patch updates, hazard tagging.
- Acceptance: debris count remains budgeted; AI pathing adapts after collapse.

## Budgets
- Debris rigid bodies capped per zone.
- Distant debris replaced by impostor decals or merged rubble entities.
- Collapse events rate-limited by frame budget scheduler.

## Verification commands
- `cargo test --test physics_body_combat -- --nocapture`
- `cargo test --test vertical_slice -- --nocapture`
- `cargo bench --profile dev --bench hot_paths -- --sample-size 10`
