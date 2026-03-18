# Sandbox Manual Test Plan

ENGENE Destruction Sandbox 50×50m — Manual QA checklist.

**Execution:** Run `TEST.exe` or `ENGENE_Game.exe --layout destruction_sandbox_50x50`

---

## 1. Launch & Basics

| Step | Action | Expected Result | ✓ |
|------|--------|-----------------|---|
| 1.1 | Launch TEST.exe (or ENGENE_Game.exe --layout destruction_sandbox_50x50) | Application starts without crash | |
| 1.2 | Verify window title | Window shows correct title (e.g. ENGENE or Destruction Sandbox) | |
| 1.3 | Verify 50×50 world loads | World terrain/structure visible, no missing geometry | |
| 1.4 | Verify player spawns at (25, 2, 25) | Player at center of map, elevated appropriately | |
| 1.5 | Press W/A/S/D | Player moves forward/left/backward/right | |
| 1.6 | Move mouse | Camera/look direction changes (mouse look) | |

---

## 2. Zone A — Tile Wall

**Zone coordinates:** X 0–15, Z 0–50

| Step | Action | Expected Result | ✓ |
|------|--------|-----------------|---|
| 2.1 | Walk to Zone A | Reach tile wall area | |
| 2.2 | Fire pistol 3× at tile wall | Local chips visible on tiles | |
| 2.3 | Fire shotgun at tile wall | Multi-tile fracture (multiple tiles affected) | |
| 2.4 | Fire rifle at tile wall | Crack forms; possible tile detach | |
| 2.5 | Throw grenade near wall | Cluster detach; debris shower | |
| 2.6 | Press F5 to save | Save confirmation (if applicable) | |
| 2.7 | Press F9 to load | Load completes | |
| 2.8 | Verify wall damage preserved | Same chips, cracks, missing tiles as before save | |

---

## 3. Zone B — Ground/Grenades

**Zone coordinates:** X 15–30, Z 0–25

| Step | Action | Expected Result | ✓ |
|------|--------|-----------------|---|
| 3.1 | Walk to Zone B | Reach ground/terrain area | |
| 3.2 | Throw grenade | Crater forms in ground | |
| 3.3 | Verify terrain mesh deformation | Crater geometry visible, mesh deformed | |
| 3.4 | Verify particles | Dust, dirt, and/or rock particles visible | |
| 3.5 | Press F5 to save | Save completes | |
| 3.6 | Press F9 to load | Load completes | |
| 3.7 | Verify crater persists | Crater still visible after load | |

---

## 4. Zone C — Basement

**Zone coordinates:** X 30–50, Z 0–25

| Step | Action | Expected Result | ✓ |
|------|--------|-----------------|---|
| 4.1 | Enter basement area | Indoor basement area reached | |
| 4.2 | Verify dynamic shadows | Shadows from ceiling lights visible | |
| 4.3 | Shoot furniture | Destruction occurs; debris spawns | |
| 4.4 | Verify audio difference | Reverb/occlusion different from outdoor | |
| 4.5 | Verify contact shadows | Shadows under objects visible | |
| 4.6 | Press F5 to save | Save completes | |
| 4.7 | Press F9 to load | Load completes | |
| 4.8 | Verify basement state preserved | Furniture destruction, debris state preserved | |

---

## 5. Zone D — Material Targets

**Zone coordinates:** X 15–30, Z 25–50

| Step | Action | Expected Result | ✓ |
|------|--------|-----------------|---|
| 5.1 | Walk to material targets area | Target area reached | |
| 5.2 | Shoot wood target | Wood response: penetrate, splinter, thud | |
| 5.3 | Shoot cloth target | Cloth response: penetrate easily, tears | |
| 5.4 | Shoot stone target | Stone response: ricochet/chip, sparks | |
| 5.5 | Shoot metal target | Metal response: ricochet/dent, metallic ring | |
| 5.6 | Shoot glass target | Glass response: shatter, glass shards | |
| 5.7 | Shoot each remaining material target | Distinct visual + audio + ballistic response per material | |

---

## 6. Zone E — Stress Zone

**Zone coordinates:** X 30–50, Z 25–50

| Step | Action | Expected Result | ✓ |
|------|--------|-----------------|---|
| 6.1 | Enter stress area | Stress zone reached | |
| 6.2 | Trigger multiple explosions | Several explosions in quick succession | |
| 6.3 | Verify stable framerate | No severe framerate drop; remains playable | |
| 6.4 | Verify no physics instability | No jittering debris; no flying/warping objects | |

---

## 7. Cross-Cutting

| Step | Action | Expected Result | ✓ |
|------|--------|-----------------|---|
| 7.1 | Press F5 then F9 in any zone | Save/load works globally; state preserved | |
| 7.2 | Press ESC | Mouse cursor released (if applicable) | |
| 7.3 | Resize window | Window resizes; rendering adapts | |

---

## Notes

- **TEST.exe** and **ENGENE_Game.exe** are interchangeable; use whichever is available for the build.
- If the layout flag differs in your build, adjust the launch command accordingly.
- Checkmarks (✓) are for manual QA sign-off.
