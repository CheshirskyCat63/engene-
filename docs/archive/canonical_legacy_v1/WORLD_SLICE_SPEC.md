# WORLD_SLICE_SPEC.md — Single Source of Truth

## Map Overview

- Size: 2x2 km (2000m x 2000m)
- Grid: GRID_SIZE cells x CELL_SIZE meters per cell
- Chunk streaming radius: 3000m load / 4000m unload

## Camps

| Name | Center | Radius | Faction | Pop | Trader | Mechanic |
|------|--------|--------|---------|-----|--------|----------|
| Rookie Camp | (500, 500) | 80m | Loners | 8 | Yes | No |
| Duty Outpost | (1200, 800) | 60m | Duty | 6 | Yes | Yes |
| Freedom Base | (600, 1600) | 70m | Freedom | 7 | Yes | No |

## Trader Hub

| Name | Position | Faction | Specialty | Price Mod |
|------|----------|---------|-----------|-----------|
| Sidorovich | (510, 490) | Traders | General | 1.0x |

## Roads

| Name | Points | Width | Safety |
|------|--------|-------|--------|
| Main Road | (200,500) -> (500,500) -> (900,600) -> (1200,800) | 10m | 0.8 |
| Forest Path | (500,500) -> (600,900) -> (600,1600) | 5m | 0.5 |

## Dangerous Corridors

- Forest Path (safety 0.5): connects Rookie Camp to Freedom Base through wolf/boar territory
- Approach to Old Laboratory (1500,1500): bloodsucker nocturnal territory

## Anomaly Clusters

| Name | Center | Radius | Type | Intensity | Artifact Chance |
|------|--------|--------|------|-----------|-----------------|
| Vortex Field | (1400, 600) | 50m | Vortex | 0.7 | 15% |
| Electro Cluster | (800, 1400) | 40m | Electro | 0.5 | 10% |
| Abandoned Factory anomaly | (900, 1100) | in-ruin | Mixed | varies | varies |

## Monster Habitats

| Name | Center | Radius | Species | Pack Size | Aggression | Nocturnal |
|------|--------|--------|---------|-----------|------------|-----------|
| Wolf Den | (700, 1500) | 200m | Wolf | 5 | 0.6 | No |
| Boar Grazing | (300, 1200) | 300m | Boar | 4 | 0.3 | No |
| Bloodsucker Lair | (1500, 1500) | 100m | Bloodsucker | 2 | 0.9 | Yes |

## Ruins / Loot Zones

| Name | Center | Radius | Loot Tier | Structural Integrity | Anomaly |
|------|--------|--------|-----------|----------------------|---------|
| Abandoned Factory | (900, 1100) | 120m | 2 | 40% | Yes |
| Old Laboratory | (1500, 1500) | 80m | 3 | 30% | Yes |

## Destruction Zones

- Abandoned Factory: low structural integrity, destructible props
- Old Laboratory: combined with bloodsucker territory for high-risk/high-reward

## Audio Zones

| Name | Center | Radius | Sound | Day Vol | Night Vol |
|------|--------|--------|-------|---------|-----------|
| Camp Ambience | (500, 500) | 100m | camp_ambience | 0.6 | 0.3 |
| Forest Ambience | (600, 1200) | 400m | forest_ambience | 0.4 | 0.6 |

## Weather / Fog Zones

| Name | Center | Radius | Rain | Fog | Temp Offset |
|------|--------|--------|------|-----|-------------|
| Swamp Zone | (300, 1400) | 250m | 0.3 | 0.5 | -2.0C |

## Patrol Graph

- Loners patrol: Rookie Camp perimeter, Main Road to Duty Outpost
- Duty patrol: Outpost perimeter, Main Road east segment
- Freedom patrol: Base perimeter, Forest Path south segment
- Wolf patrol: Wolf Den to Boar Grazing corridor
- Bloodsucker nocturnal: Old Laboratory ruins radius

## Chunk Grid

Chunks are loaded/unloaded by the WorldStreamer based on camera position.
Streaming thresholds: load=3000m, unload=4000m.

## Biomes

Biomes are assigned per cell based on heightmap and position.
Biome types defined in `game/data/biomes.ron`.
