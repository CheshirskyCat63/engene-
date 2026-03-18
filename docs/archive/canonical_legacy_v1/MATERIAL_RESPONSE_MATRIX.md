# Material Response Matrix

ENGENE Destruction Sandbox — Canonical specification for all 16 materials.

---

## Quick Reference Table

| ID | Material | Ballistic | Fracture | Decal | Debris | Dust | Impact | Resonance | Persistence | Low-Spec |
|----|----------|-----------|----------|-------|--------|------|--------|-----------|------------|----------|
| 0 | Wood | penetrate | splinter | gouge | splinters | wood dust | thud | hollow | splinters, holes | reduce debris count |
| 1 | Cloth | penetrate easily | tears | tear | fabric particles | minimal | soft impact | muted | tears, holes | reduce particles |
| 2 | Thatch | penetrate | crumbles | hole | straw particles | straw | rustling | dry rustle | holes, debris | reduce straw particles |
| 3 | Stone | ricochet/chip | chip | chip/scuff | stone chips | stone dust | sharp crack | brittle | chips, sparks | reduce sparks |
| 4 | Metal | ricochet/dent | dent | dent | none | none | metallic ring | metallic | dents | cannot reduce |
| 5 | Flesh | penetrate | blood splatter | blood decal | blood droplets | mist | wet impact | organic | blood decals | reduce particle density |
| 6 | Earth | stop | crater | crater | none | dirt spray | dull thud | dense | crater geometry | reduce spray particles |
| 7 | Tile | shatter | shatters | crack | ceramic shards | fine dust | brittle crack | ceramic | shards, damage | reduce shard count |
| 8 | Concrete | chip/crack | chip/crack | crack | rubble | concrete dust | heavy impact | heavy | cracks, rubble | reduce rubble |
| 9 | Brick | chip/crack | chip/crack | chip | brick fragments | red dust | heavy thud | masonry | chips, fragments | reduce dust |
| 10 | Glass | shatter | shatters | none | glass shards | none | sharp shatter | crystalline | shards | reduce shard count |
| 11 | Steel | ricochet | dent | dent | none | none | loud ring | metallic | dents | cannot reduce |
| 12 | Sand | stop | spray | crater | none | sand particles | soft thud | granular | crater | reduce particles |
| 13 | Gravel | stop/ricochet | spray | scuff | pebble spray | dust | rattling | loose | crater, pebbles | reduce spray |
| 14 | Rubber | stop/absorb | deformation | deformation | none | none | dull thump | elastic | deformation | cannot reduce |
| 15 | Plastic | penetrate/crack | crack | crack | plastic shards | minimal | sharp crack | brittle | cracks, shards | reduce shards |

---

## Detailed Sections

### 1. Wood (id:0)
- **Ballistic outcome:** Penetrate
- **Fracture mode:** Splinter
- **Visual effect:** Gouge decal, splinter debris, wood dust (medium)
- **Audio effect:** Impact class: thud. Resonance class: hollow
- **Persistence:** Splinters, penetration holes, visible damage to geometry
- **Low-spec downgrade:** Can reduce: debris particle count, dust density. Cannot reduce: penetration logic, base decal

### 2. Cloth (id:1)
- **Ballistic outcome:** Penetrate easily
- **Fracture mode:** Tears
- **Visual effect:** Tear decal, fabric particles, minimal dust
- **Audio effect:** Impact class: soft impact. Resonance class: muted
- **Persistence:** Tears, holes in cloth geometry
- **Low-spec downgrade:** Can reduce: fabric particle count. Cannot reduce: tear geometry, penetration

### 3. Thatch (id:2)
- **Ballistic outcome:** Penetrate
- **Fracture mode:** Crumbles
- **Visual effect:** Hole decal, straw particles, straw dust (medium)
- **Audio effect:** Impact class: rustling. Resonance class: dry rustle
- **Persistence:** Holes, small debris scatter
- **Low-spec downgrade:** Can reduce: straw particle count. Cannot reduce: crumble behavior, hole formation

### 4. Stone (id:3)
- **Ballistic outcome:** Ricochet / chip
- **Fracture mode:** Chip
- **Visual effect:** Chip/scuff decal, stone chips, sparks, stone dust (medium)
- **Audio effect:** Impact class: sharp crack. Resonance class: brittle
- **Persistence:** Chips, scuffs, sparks (ephemeral)
- **Low-spec downgrade:** Can reduce: sparks, dust particles. Cannot reduce: chip geometry, ricochet

### 5. Metal (id:4)
- **Ballistic outcome:** Ricochet / dent
- **Fracture mode:** Dent
- **Visual effect:** Dent decal, sparks, no dust
- **Audio effect:** Impact class: metallic ring. Resonance class: metallic
- **Persistence:** Dents, no debris
- **Low-spec downgrade:** Cannot reduce: ballistic response, audio, dent geometry (core gameplay)

### 6. Flesh (id:5)
- **Ballistic outcome:** Penetrate
- **Fracture mode:** Blood splatter
- **Visual effect:** Blood decal, blood droplets debris, mist
- **Audio effect:** Impact class: wet impact. Resonance class: organic
- **Persistence:** Blood decals, splatter geometry
- **Low-spec downgrade:** Can reduce: particle density, mist. Cannot reduce: decal, splatter logic

### 7. Earth (id:6)
- **Ballistic outcome:** Stop
- **Fracture mode:** Crater
- **Visual effect:** Crater decal, dirt spray debris, no separate dust type
- **Audio effect:** Impact class: dull thud. Resonance class: dense
- **Persistence:** Crater geometry, terrain mesh deformation
- **Low-spec downgrade:** Can reduce: dirt spray particle count. Cannot reduce: crater formation

### 8. Tile (id:7)
- **Ballistic outcome:** Shatter
- **Fracture mode:** Shatters
- **Visual effect:** Crack decal, ceramic shards, fine dust
- **Audio effect:** Impact class: brittle crack. Resonance class: ceramic
- **Persistence:** Ceramic shards, crack geometry, detached tiles
- **Low-spec downgrade:** Can reduce: shard count, dust. Cannot reduce: shatter behavior

### 9. Concrete (id:8)
- **Ballistic outcome:** Chip / crack
- **Fracture mode:** Chip / crack
- **Visual effect:** Crack decal, rubble debris, concrete dust (heavy)
- **Audio effect:** Impact class: heavy impact. Resonance class: heavy
- **Persistence:** Cracks, rubble, concrete dust accumulation
- **Low-spec downgrade:** Can reduce: rubble count, dust density. Cannot reduce: crack propagation

### 10. Brick (id:9)
- **Ballistic outcome:** Chip / crack
- **Fracture mode:** Chip / crack
- **Visual effect:** Chip decal, brick fragments, red dust (medium)
- **Audio effect:** Impact class: heavy thud. Resonance class: masonry
- **Persistence:** Chips, brick fragments, crack lines
- **Low-spec downgrade:** Can reduce: red dust, fragment count. Cannot reduce: chip/crack logic

### 11. Glass (id:10)
- **Ballistic outcome:** Shatter
- **Fracture mode:** Shatters
- **Visual effect:** None (shatters immediately), glass shards, no dust
- **Audio effect:** Impact class: sharp shatter. Resonance class: crystalline
- **Persistence:** Glass shards, broken pane state
- **Low-spec downgrade:** Can reduce: shard count. Cannot reduce: shatter behavior, audio

### 12. Steel (id:11)
- **Ballistic outcome:** Ricochet
- **Fracture mode:** Dent (no fracture)
- **Visual effect:** Dent decal, sparks, no dust
- **Audio effect:** Impact class: loud ring. Resonance class: metallic
- **Persistence:** Dents only
- **Low-spec downgrade:** Cannot reduce: ballistic response, audio, dent (core gameplay)

### 13. Sand (id:12)
- **Ballistic outcome:** Stop
- **Fracture mode:** Spray
- **Visual effect:** Crater decal, sand particles
- **Audio effect:** Impact class: soft thud. Resonance class: granular
- **Persistence:** Crater, scattered sand
- **Low-spec downgrade:** Can reduce: sand particle count. Cannot reduce: stop behavior, crater

### 14. Gravel (id:13)
- **Ballistic outcome:** Stop / ricochet
- **Fracture mode:** Pebble spray
- **Visual effect:** Scuff decal, pebble spray, dust
- **Audio effect:** Impact class: rattling. Resonance class: loose
- **Persistence:** Crater, pebble scatter
- **Low-spec downgrade:** Can reduce: pebble spray, dust. Cannot reduce: stop/ricochet logic

### 15. Rubber (id:14)
- **Ballistic outcome:** Stop / absorb
- **Fracture mode:** Deformation
- **Visual effect:** Deformation decal, no debris, no dust
- **Audio effect:** Impact class: dull thump. Resonance class: elastic
- **Persistence:** Deformation geometry
- **Low-spec downgrade:** Cannot reduce: deformation, ballistic absorption (physics-critical)

### 16. Plastic (id:15)
- **Ballistic outcome:** Penetrate / crack
- **Fracture mode:** Crack (can detach)
- **Visual effect:** Crack decal, plastic shards, minimal dust
- **Audio effect:** Impact class: sharp crack. Resonance class: brittle
- **Persistence:** Cracks, plastic shards
- **Low-spec downgrade:** Can reduce: shard count, dust. Cannot reduce: crack/penetration logic

---

## Summary: Cannot Reduce (Low-Spec)

| Material | Cannot Reduce |
|----------|---------------|
| Wood, Cloth, Thatch | Penetration logic |
| Stone | Chip geometry, ricochet |
| Metal, Steel | Ballistic response, audio, dent geometry |
| Flesh | Decal, splatter logic |
| Earth | Crater formation |
| Tile, Glass | Shatter behavior |
| Concrete, Brick | Crack propagation |
| Sand | Stop behavior, crater |
| Gravel | Stop/ricochet logic |
| Rubber | Deformation, ballistic absorption |
| Plastic | Crack/penetration logic |
