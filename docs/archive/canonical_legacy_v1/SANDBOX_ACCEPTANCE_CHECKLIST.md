# Sandbox Acceptance Checklist

Pass/fail checklist for the Destruction Sandbox 50×50 proving ground. Each item is evaluated as `PASS` or `FAIL`.

---

## Zone A — Tile Wall

| # | Status | Requirement |
|---|--------|-------------|
| A1 | `[ ]` PASS / FAIL | Pistol: chip or localized crack in tile layer only |
| A2 | `[ ]` PASS / FAIL | Rifle: crack + possible tile detach, adhesive layer damage |
| A3 | `[ ]` PASS / FAIL | Shotgun: multi-tile fracture cone, multiple detachments |
| A4 | `[ ]` PASS / FAIL | Grenade near wall: cluster detach + debris shower, support wall damage |
| A5 | `[ ]` PASS / FAIL | Debris physics: detached tiles behave as physics bodies |
| A6 | `[ ]` PASS / FAIL | Surface marks differ by material (tile, plaster, concrete, metal, glass) |
| A7 | `[ ]` PASS / FAIL | Save/load preserves wall state (missing tiles, cracks, debris) |

---

## Zone B — Ground

| # | Status | Requirement |
|---|--------|-------------|
| B1 | `[ ]` PASS / FAIL | Grenade crater forms in terrain |
| B2 | `[ ]` PASS / FAIL | Terrain mesh updates to reflect crater |
| B3 | `[ ]` PASS / FAIL | Particles differ by soil type (dry soil, mud, sand/gravel, grass) |
| B4 | `[ ]` PASS / FAIL | Nav mesh and cover points update after crater |
| B5 | `[ ]` PASS / FAIL | Crater persists after save/load |
| B6 | `[ ]` PASS / FAIL | Low-spec acceptable fallback (reduced particles, preserved crater truth) |

---

## Zone C — Basement

| # | Status | Requirement |
|---|--------|-------------|
| C1 | `[ ]` PASS / FAIL | Dynamic shadows stable in interior |
| C2 | `[ ]` PASS / FAIL | Contact shadows present under furniture (chairs, tables, crates) |
| C3 | `[ ]` PASS / FAIL | Muzzle flash briefly illuminates scene |
| C4 | `[ ]` PASS / FAIL | Furniture destruction works (breakable props) |
| C5 | `[ ]` PASS / FAIL | Debris behaves correctly in confined space |
| C6 | `[ ]` PASS / FAIL | Interior audio reverb applies |
| C7 | `[ ]` PASS / FAIL | Save/load preserves basement destruction state |

---

## Zone D — Material Targets

| # | Status | Requirement |
|---|--------|-------------|
| D1 | `[ ]` PASS / FAIL | Each material has distinct ballistic response |
| D2 | `[ ]` PASS / FAIL | Each material has distinct visual feedback |
| D3 | `[ ]` PASS / FAIL | Each material has distinct audio feedback |
| D4 | `[ ]` PASS / FAIL | Persistence per material (wood, metal, concrete, tile, glass, soil, cloth, crate) |

---

## Zone E — Stress Zone

| # | Status | Requirement |
|---|--------|-------------|
| E1 | `[ ]` PASS / FAIL | Stable FPS under 20–40 debris + explosions |
| E2 | `[ ]` PASS / FAIL | No memory runaway during stress test |
| E3 | `[ ]` PASS / FAIL | Nav mesh rebuild completes within budget |
| E4 | `[ ]` PASS / FAIL | Chunk reload preserves truth state |

---

## Cross-Cutting

| # | Status | Requirement |
|---|--------|-------------|
| X1 | `[ ]` PASS / FAIL | `TEST.exe` launches sandbox |
| X2 | `[ ]` PASS / FAIL | `ENGENE_Game.exe --layout destruction_sandbox_50x50` works |
| X3 | `[ ]` PASS / FAIL | SDK opens sandbox |
| X4 | `[ ]` PASS / FAIL | All executables run without requiring Cargo (standalone binaries) |
