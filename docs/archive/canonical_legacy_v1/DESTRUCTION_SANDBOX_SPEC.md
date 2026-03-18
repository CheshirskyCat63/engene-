# Destruction Sandbox — Master Specification

## Overview

The Destruction Sandbox is a **50×50 meter technical proving ground** for the ENGENE game engine. It validates destruction systems, physics simulation, rendering pipeline, and audio integration under controlled, repeatable conditions. All engine subsystems must meet requirements defined in this spec.

---

## Coordinate System

- **Origin:** `(0, 0)` at ground level (corner of the sandbox)
- **Extents:** `(0, 0)` → `(50, 50)` meters
- **Axes:** Right-handed; X and Z (or equivalent) span the horizontal plane; Y (or equivalent) is up

---

## Zones (Exact Coordinates)

### Zone A — Tile Wall
- **Bounds:** `(0, 0)`–`(15, 50)`
- **Purpose:** Wall destruction, layered materials, ballistic response
- **Contents:**
  - Sections of tiled wall (ceramic tile surface)
  - Plaster
  - Concrete backing
  - Metal fixtures
  - Glass panels
- **Focus:** Fracture mechanics, debris physics, material-specific visuals and audio

### Zone B — Ground / Grenades
- **Bounds:** `(15, 0)`–`(30, 25)`
- **Purpose:** Terrain destruction, crater formation, grenade effects
- **Terrain types:**
  - Dry soil
  - Mud
  - Sand/gravel
  - Grass
- **Focus:** Deformable terrain mesh, crater persistence, material-specific particles, nav/cover updates

### Zone C — Basement
- **Bounds:** `(30, 0)`–`(50, 25)`
- **Purpose:** Indoor environment, dynamic shadows, confined-space destruction
- **Contents:**
  - Low ceiling
  - Lights
  - Furniture
  - Pipes
- **Focus:** Shadow stability, contact shadows, muzzle flash lighting, interior audio (reverb), debris in confined space

### Zone D — Material Targets
- **Bounds:** `(15, 25)`–`(30, 50)`
- **Purpose:** Per-material ballistic and destruction testing
- **Test targets:** Wood, metal, concrete, tile, glass, soil, cloth, crate
- **Focus:** Distinct response per material for visuals, physics, and audio

### Zone E — Stress Zone
- **Bounds:** `(30, 25)`–`(50, 50)`
- **Purpose:** Worst-case load for engine stability
- **Conditions:**
  - 20–40 active debris objects
  - Multiple explosions
  - Particles + shadows + audio under stress
- **Focus:** FPS stability, no memory runaway, nav rebuild within budget, chunk reload preserving truth state

---

## Materials Required

16 materials must be supported:

| Material | Notes |
|----------|-------|
| Wood | Planks, furniture, crates |
| Cloth | Targets, drapes |
| Thatch | Roofing, cover |
| Stone | Walls, rubble |
| Metal | Fixtures, targets |
| Flesh | (Reserved) |
| Earth | Dry soil, mud |
| Tile | Ceramic wall tiles |
| Concrete | Walls, floors |
| Brick | Walls |
| Glass | Panels, windows |
| Steel | Reinforced structures |
| Sand | Ground, piles |
| Gravel | Ground, rubble |
| Rubber | Belts, seals |
| Plastic | Fixtures |

---

## Weapons

- **Makarov pistol** — Low damage, localized chip/crack
- **AK-type rifle** — Medium-high damage, crack + possible detach
- **Shotgun** — Spread, multi-tile fracture cone
- **Grenade** — Blast radius, cluster detach, crater formation

---

## Rendering

- **Shadows:** Dynamic only — no baked lightmaps, no baked shadows
- **Shadow techniques:**
  - Cascaded shadow maps
  - Contact shadows for close-range detail
- **Requirements:** Per-material visual distinction; no baked shadow artifacts

---

## Audio

- **Backend:** `rodio` mandatory
- **Material-specific impact sounds:** Each material layer must have distinct impact/destruct sounds
- **Interior reverb:** Zone C basement requires reverb for indoor acoustic

---

## Persistence

- **Requirement:** All destruction state survives save/load
- **Scope:** Missing geometry, cracks, craters, debris placement, fracture topology — all preserved
- **Policy:** Large debris persistent; small shards may be transient (policy-defined)

---

## Layout File

```
game/world/layouts/destruction_sandbox_50x50.ron
```

This RON file defines the world layout, zone boundaries, and initial placement for the sandbox.

---

## Low-Spec Rules

- **Truth state preserved:** Destruction topology and physics state remain correct
- **Presentation reduced:** Lower shadow resolution, fewer particles, reduced post-process quality
- **No truth reduction:** No simplification of destruction, fracture, or debris physics on low-spec
