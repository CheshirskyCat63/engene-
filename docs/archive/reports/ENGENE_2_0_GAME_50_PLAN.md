# ENGENE 2.0 Game 1.0 Tech Demo Plan

## Tech Demo framing
Game 1.0 Tech Demo means one complete, stable, evidence-backed vertical slice proving flagship systems in real gameplay conditions without claiming full game product completeness.

## What is deliberately missing in Tech Demo 1.0
- Full campaign progression.
- Final narrative and mission breadth.
- Final balancing across all factions/biomes.
- Full content quantity expected for a complete shipped game.

## Target: one executable showcase scene
Scenario: **Industrial District Breach**
- Multi-material compound with tile facades, support frames, concrete cores.
- Enemy squads in layered armor/body profiles.
- Ally squads operating under abstracted command tiers.
- Scripted weather shift from dry wind to storm front.

## Demonstration slices

### G1: Combat and body response
- Player weapons with penetrative ammo classes.
- Layered enemy damage and dismemberment states.
- Material-aware impact audio.

### G2: Destruction and structures
- Tile-to-concrete reveal on sustained ballistic fire.
- Explosive breaching of structural supports.
- Building partial collapse and nav reroute.

### G3: Environment dynamics
- Ignition sources and wind-biased spread.
- Rain onset causing wetness accumulation and suppression.
- Puncturable water/fuel containers with leak trails.

### G4: Population and world scale
- 10k enemies + 5k allies represented across L0-L3.
- Nearby units promoted to active simulation bubble.
- Distant fronts and conflict updates visible via SDK overlays.

### G5: Audio proof
- Obstruction-sensitive gunfire and explosions.
- Reverb contrast between interior halls and open streets.
- AI hearing behavior demonstrably altered by occlusion/weather.

## Exact showcase proof checklist (pass/fail)
1. Layered enemy penetration visible and logged.
2. Dismember/gore state transitions triggered by runtime damage, not scripts.
3. Tile fracture exposes concrete and changes subsequent ballistic response.
4. Explosive event causes structural integrity drop and nav reroute.
5. Fire spreads only over burnable links and is wind-biased.
6. Rain/wetness measurably reduces fire spread rate.
7. Punctured container leaks and updates local wetness map.
8. Distant weather front visible before local weather transition.
9. Population counters show 10k/5k represented via L0-L3 layers.
10. Audio occlusion/reverb/hearing overlays show causal changes.

## 50% readiness acceptance
- Scene launches from `engene_game` executable.
- All flagship systems visibly demonstrable without dev-only hacks.
- Crash-free 20-minute scripted demo run.
- Metrics capture proves subsystem budgets remain in target bands.
