# PHASE_ORDER_CONTRACT

## Canonical phase order for the SDK/game-style frame driver

1. `tick`
2. `streaming`
3. `persistence`
4. `spatial`
5. `audio`
6. `editor_update`
7. `render`

## Why this order

### tick
Simulation truth changes first.

### streaming
World residency decisions happen against updated runtime state.

### persistence
Loaded/unloaded chunk transitions become durable and explicit.

### spatial
Derived lookup structure catches up to truth changes after residency changes.

### audio
Listener/runtime sound update consumes stable post-spatial state.

### editor_update
Editor mutation and dashboard sync happen against a known, post-sim frame state.

### render
Rendering is last and consumes prepared state; it does not define it.

## Contract rules

- No silent reordering.
- Render must not own simulation truth.
- Audio must not depend on render success.
- Spatial must be fed from explicit dirty causes or explicit fallback path.
- Editor mutation must either:
  - mark dirty sources for next spatial pass, or
  - force a documented controlled rebuild path.

## Required tests

- phase order invariant test,
- editor mutation propagation test,
- audio independent-of-render test,
- spatial equivalence test.
