# ENGENE 2.0 Data Layout Policy

## Purpose
Define data-oriented runtime layout rules for cache locality, SIMD friendliness, and low-churn hot paths.

## Performance-first layout law
Hot runtime loops must be:
- contiguous,
- predictable,
- batchable,
- SIMD-friendly,
- allocation-light,
- cache-first.

## Core layout rules
1. Prefer SoA over AoS for hot simulation data.
2. Separate hot and cold data explicitly.
3. Use dense contiguous storages for frequently iterated components.
4. Sparse structures on hot loops require profiling proof.
5. String/hash lookups are pre-resolved into compact IDs for runtime hot paths.

## Hot/cold split policy
- Hot: transforms, velocities, health, threat, flags, active simulation coefficients.
- Cold: names, editor labels, thumbnails, notes, verbose provenance.
- Cold metadata must never be required inside gameplay-critical hot loops.

## Packed storage rules
- Favor fixed-size POD-friendly structs for hot records.
- Avoid pointer-rich object graphs in hot update paths.
- Align packed arrays to support vectorized loads where practical.

## False sharing prevention
- Partition worker-write domains by chunk or archetype stripe.
- Avoid multiple worker threads writing adjacent cache-line fields in same frame phase.
- Use padding/striping where cross-thread write adjacency is unavoidable.

## Allocation policy for hot paths
- Zero or near-zero steady-state allocations during frame-critical updates.
- Burst systems must use pools/arenas/slabs with bounded growth.
- Per-entity heap allocations on update path are forbidden unless profiled and waived.

## Stable handle vs packed data
- Public/runtime APIs should pass stable handles/indices.
- Internal execution should resolve handles to packed slices before iteration.
- Rich object wrappers are allowed in tooling/editor layers, not hot simulation loops.

## Branch minimization and batching
- Replace branch-heavy per-entity logic with table-driven or mask-driven batched steps where practical.
- Evaluate neighbors/queries in chunked batches, not random scattered probes.

## Domain-specific packing requirements
- Materials cooked into compact indexable tables.
- L2/L3 AI aggregate state in flat arrays/packed vectors.
- Fire/wetness cells in grid-packed fields with active/dirty masks.
- Audio propagation inputs in compact emitter/listener batches.

## Evidence requirements
- Hot-loop cache miss profiles.
- Allocation telemetry for frame-critical paths.
- Locality benchmark snapshots under worst-case showcase triggers.
