# ENGENE 2.0 Schema Law

## Purpose
Prevent schema drift between subsystems by enforcing a canonical data model for shared simulation content.

## Canonical ownership
- Canonical schemas are owned by `engine_content`.
- Runtime consumers (`engine_physics`, `engine_audio`, `engine_world`, `engine_render`, `game_framework`) are read-only at schema definition level.
- SDK editors write authoring sources that compile into canonical cooked schema artifacts.

## Schema projection rules
Allowed projections:
1. Canonical schema projection.
2. Cooked schema projection.
3. Runtime cache projection.
4. SDK editor projection.

Rule: no projection may silently introduce semantics absent in canonical schema.

## Hot/cold data split policy
- Canonical schema may include both hot runtime semantics and cold authoring/debug semantics.
- Cooking must split hot data from cold metadata.
- Hot runtime projections must be compact, index-addressable, batch-friendly.
- Cold metadata must never be required in gameplay-critical hot loops.

## Material schema law
There must be one canonical material definition that includes (inline or by strict references):
1. Ballistic parameters (density, penetration resistance, ricochet behavior).
2. Fracture/destruction parameters (fracture energy, support contribution, debris class).
3. Fire parameters (ignition temp proxy, burn rate, fuel class, ember susceptibility).
4. Wetness parameters (absorption rate, saturation cap, runoff profile).
5. Acoustic parameters (occlusion attenuation, reflection/absorption bands).
6. Visual/render parameters (surface class, decal response, wet look controls).
7. Collapse/debris parameters (break archetype references, debris lifetime class).

## Body/damage schema law
Canonical body schema includes:
- Zone topology.
- Armor/tissue layer mapping.
- Dismember thresholds.
- Gore variant references.
- Persistence tags for save/load.

## Weather/environment schema law
Canonical weather schema includes:
- Front class parameters.
- Wind/precipitation ranges.
- Coupling coefficients to fire/wetness systems.
- Visual far-field representation hints.

## Forbidden patterns
- Subsystem-private “material_v2”, “physics_material”, “audio_surface” duplicates with overlapping meaning.
- Runtime-only hidden defaults that bypass canonical schema values.
- SDK-only schema branches that are not cook-compatible.

## Versioning and migrations
- Every schema has semantic version and migration steps.
- Runtime loader rejects unknown major versions.
- SDK auto-suggests migration, but explicit user approval required for destructive transforms.

## Enforcement
- CI schema uniqueness checks.
- Validator ensures all required parameter groups exist.
- Cooking fails on missing cross-domain references.
