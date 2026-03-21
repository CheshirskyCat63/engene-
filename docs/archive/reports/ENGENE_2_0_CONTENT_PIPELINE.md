# ENGENE 2.0 Content Pipeline

## Purpose
Define mandatory authored-data lifecycle so runtime, SDK, and persistence use the same truth chain.

## Mandatory pipeline
`authoring source -> schema version -> validator -> cooker -> cooked artifact -> runtime loader -> SDK inspector -> persistence`

## Stages
1. **Authoring source**
   - Human-editable assets/data (materials, body profiles, break graphs, weather presets, audio tags, scenarios).
2. **Schema version bind**
   - Each source asset declares canonical schema version.
3. **Validation**
   - Structural + semantic checks (cross-reference integrity, budget sanity, forbidden defaults).
4. **Cooking**
   - Deterministic transformation to runtime-ready compact artifacts.
5. **Runtime loading**
   - Runtime consumes only cooked artifacts for shipping paths.
6. **SDK inspection**
   - SDK can inspect both source and cooked forms; discrepancies are errors.
7. **Persistence interaction**
   - Save files store runtime state deltas referencing schema IDs/versions.

## Reproducibility contract
- Identical source + schema version + cooker tool version must yield identical cooked hash.
- Cook reports include toolchain version, schema version, input hash, output hash.
- Non-deterministic cooking is forbidden for canonical assets.

## Runtime packing requirements
- Cooked artifacts must be optimized for runtime locality and direct indexed access.
- Shipping runtime must not parse high-level authoring graphs on hot paths.
- String references must resolve to compact IDs during cooking.
- Cross-domain references should be flattened into stable indices/handles where practical.
- Optional debug/editor metadata is stored separately from hot runtime blobs.

## Invalidation rules
- Source change invalidates dependent cooked artifacts transitively.
- Schema major bump invalidates all old cooked artifacts without migration.
- Reference change (e.g., material -> acoustic profile) invalidates consumers.

## Migration rules
- Migrations are explicit scripts, version-to-version.
- CI rejects data with skipped migration chain.
- Destructive migration requires backup artifact generation.

## Persistence/content compatibility
- Saves carry content/schema version references.
- Load mismatch behavior must be explicit: migrate or reject with diagnostics.
- Silent fallback to different semantics is forbidden.

## Canonical scenarios
- Designated canonical validation scenarios are build blockers if validation/cooking fails.
- Canonical scenario cook reports must be archived with CI evidence.

## Runtime truth constraints
- No hidden runtime constants that override cooked values silently.
- Validation failures are build blockers for canonical scenarios.
- “Editor-only behavior” diverging from runtime is forbidden.

## Required tooling
- Content doctor command for full validation.
- Cook report with artifact hashes.
- Diff viewer for source vs cooked semantics.
