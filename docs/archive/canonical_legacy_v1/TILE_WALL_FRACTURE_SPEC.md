# Tile Wall Fracture — Mini-Contract

Specification for tile wall destruction behavior in Zone A of the Destruction Sandbox.

---

## Layer Model

The tile wall is modeled as **3 layers**:

1. **Tile shell** — Outer ceramic tile surface; chips and fractures first
2. **Adhesive/plaster** — Middle layer; damaged when tile detaches
3. **Support wall** — Backing structure (concrete/masonry); damaged by high-energy impacts

---

## Per-Weapon Behavior

| Weapon | Expected Behavior |
|--------|-------------------|
| **Pistol** | Chip or localized crack in tile layer only; no detach |
| **Rifle** | Crack + possible tile detach; adhesive damage |
| **Shotgun** | Multi-tile fracture cone; multiple detachments in spread pattern |
| **Grenade near wall** | Cluster detach + debris shower; support wall damage |

---

## Fracture Mechanics

- **DestructionNode per tile segment:** Each tile (or tile subdivision) has a DestructionNode for damage tracking
- **Crack propagation:** Cracks propagate between adjacent tiles; adjacency graph governs propagation
- **Angle-of-impact:** Impact angle affects fracture pattern (oblique vs normal)
- **Energy thresholds:** Different energy levels for chip → crack → detach → support damage

---

## Debris

- **Physics:** Detached tiles become physics bodies (Rapier3D)
- **Large tiles:** Persistent debris — remain after save/load
- **Small shards:** Transient debris — may be culled for performance but policy must be defined
- **Collision:** Debris collides with world and other debris

---

## Visual

- **Decals:** Per-material-layer decals (tile, plaster, concrete)
- **Dust particles:** Spawn on impact and detach
- **Debris readability:** Flying debris must be visually trackable; distinct from background

---

## Audio

All events must have distinct sounds:

- **Tile crack** — Crack propagation in tile layer
- **Tile detach** — Tile separation from wall
- **Debris landing** — Debris impact on ground/surfaces
- Material layer influences sound selection

---

## Persistence

After save/load:

- Missing tiles remain missing
- Cracked state preserved
- Debris policy applied: large = persistent, small = transient
- Fracture topology (which tiles are damaged, detached, or intact) must match pre-save state

---

## Low-Spec

- **Reduce:** Particle count, secondary debris sprites
- **Preserve:** Truth of detached tiles, fracture topology, destruction state
- **Never reduce:** Which tiles are missing, crack connectivity, debris placement for large pieces
