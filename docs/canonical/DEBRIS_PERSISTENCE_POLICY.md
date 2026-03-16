# Debris Persistence Policy

ENGENE Destruction Sandbox — Canonical specification for debris lifecycle and persistence.

---

## 1. Debris Categories

### 1.1 Persistent Debris

Debris that must survive until explicit cleanup or zone reset. Includes:

- **Large detached tiles** — Floor/wall tiles > size threshold that have broken free
- **Large wall chunks** — Wall sections that have detached from structure
- **Large crate/furniture pieces** — Destroyed props above size threshold
- **Objects affecting navigation/cover** — Debris that changes nav mesh, cover map, or AI routing

### 1.2 Transient Debris

Debris that despawns after a short lifetime. Includes:

- **Tiny dust particles** — Material dust (wood, stone, brick, etc.)
- **Small shards** — Glass, ceramic, plastic fragments below size threshold
- **Short-lived particle chunks** — Cosmetic-only fragments
- **Cosmetic-only fragments** — No gameplay relevance

### 1.3 Gameplay-Relevant Debris

Subset of persistent debris that has explicit gameplay impact:

- Objects that **change nav mesh** — Blocks pathfinding
- Objects that **change cover map** — Affects AI cover usage
- Objects that **affect AI routing** — Obstacles for pathfinding

---

## 2. Size Thresholds

| Threshold | Dimension | Classification |
|-----------|-----------|----------------|
| **> 0.3 m** | Any dimension | Persistent candidate — may be saved and kept until cleanup |
| **0.1–0.3 m** | Any dimension | Context-dependent — gameplay-relevant → persistent; cosmetic-only → transient |
| **< 0.1 m** | Any dimension | Always transient — never saved, despawns after time policy |

---

## 3. Time Policy

| Category | Lifespan | Notes |
|----------|----------|-------|
| **Transient debris** | 30 seconds | Despawns automatically after spawn |
| **Persistent debris** | Until explicit cleanup or zone reset | No automatic despawn |

---

## 4. Save/Load Rules

After save/load, the following **must be preserved**:

| Preserved | Examples |
|-----------|----------|
| **Large detached segments** | Wall chunks, floor slabs, structural pieces |
| **Large destroyed props** | Crates, furniture, large objects |
| **Crater truth** | Terrain deformation, crater geometry |
| **Cover/nav relevant rubble** | Debris affecting nav mesh or cover map |

The following **must NOT be saved**:

| Not Saved | Examples |
|-----------|----------|
| **Transient debris** | Dust, small shards, cosmetic particles |

---

## 5. Memory Budget

| Limit | Value | Behavior |
|-------|-------|----------|
| **Max persistent debris entities per chunk** | 256 | Per spatial chunk |
| **Excess handling** | Oldest-first cleanup | When limit exceeded, remove oldest persistent debris first |

---

## 6. Rapier3D Stability

| Requirement | Value | Behavior |
|-------------|-------|----------|
| **Sleep state deadline** | 5 seconds | Persistent debris must reach sleep state within 5 seconds |
| **Force-stabilization** | If not sleeping | Debris still moving/oscillating after 5s is force-stabilized |

---

## Summary Table

| Aspect | Rule |
|--------|------|
| Persistent candidate | Any dimension > 0.3 m |
| Always transient | Any dimension < 0.1 m |
| Transient lifespan | 30 seconds |
| Persistent lifespan | Until cleanup or zone reset |
| Save/load | Large segments, craters, nav/cover debris saved; transient not saved |
| Memory budget | 256 persistent entities per chunk, oldest-first eviction |
| Physics stability | Sleep within 5 s or force-stabilize |
