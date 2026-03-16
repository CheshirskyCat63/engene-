# Engine → SDK → Game Execution Plan

## Phase 1: Sandbox Foundation (DONE)

- DESTRUCTION_SANDBOX_SPEC.md, authored scene 50x50, tile wall, ground zone, basement, destructible clutter, lights/audio zones
- Canonical test scene of the engine

## Phase 2: Sandbox Physics Certification (DONE)

- Tile wall fracture (3-layer), debris, crater deformation, terrain update, save/load truth, material-specific reactions

## Phase 3: Sandbox Render/Audio Certification (DONE)

- Dynamic shadows (cascaded, contact shadows dormant), explosion particles, debris readability, material decals, sound by material, interior/exterior reverb

## Phase 4: SDK Authoring Hardening (DONE)

- Prefab Placer tool, save/reload workflow, 9 prefab types, group placement

## Phase 5: Game Loop on Sandbox (DONE)

- Grounded player, 4 weapons (Makarov/AK/Shotgun/Grenade), weapon switching (1/2/3/G), save/load, respawn

## Phase 6: Expand to 2x2km World (PENDING — only after Sandbox Perf Gate)

- Camp, trader, safe roads, danger corridors, anomalies, monster ecology, loot zones

## Phase 7: SDK to Unreal-like Tooling (PENDING)

- Better editor ergonomics, visual gizmos, data-driven authoring, integrated validation

## Phase 8: Final Productization (PENDING)

- Root exe packaging (TEST.exe + Game + SDK + Headless), docs canonicalization, soak test, low-spec certification

---

## Gates

| Gate | Description | Status |
|------|-------------|--------|
| **Gate A** | 0 build errors, all exe standalone | PASSED |
| **Gate B** | Acceptance checklist | defined |
| **Gate C** | Authoring without code | PASSED (Prefab Placer) |
| **G0** | 4 weapons verified | PASSED (weapon switching wired) |
| **Perf Gate** | Stable fps, no memory runaway | PENDING runtime test |
| **Milestone S0** | TEST.exe in root, double-click playable | PASSED (binary created) |
