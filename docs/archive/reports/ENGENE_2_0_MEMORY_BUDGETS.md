# ENGENE 2.0 Memory Budgets

## Purpose
Define enforceable memory footprint and bandwidth budgets across simulation layers and subsystems.

## Memory classes
- Persistent world state.
- Active simulation state (L0/L1).
- Aggregate simulation state (L2/L3).
- Transient scratch buffers.
- Streaming/cache buffers.
- Telemetry/replay/debug buffers.

## Budget contract model
Each subsystem must define:
- hard memory ceiling,
- soft target,
- burst allowance,
- recovery policy,
- degradation actions.

## Layer representation budgets
- L0: richest representation, strict active-region cap.
- L1: reduced-fidelity but still entity-addressable.
- L2: aggregate vectors/tables, no full per-agent payload.
- L3: strategic summary counters and seeds only.

## Subsystem memory policy
- Physics/destruction: debris pools, solver scratch arenas, bounded history.
- AI: capped promoted actor state, pooled decision buffers.
- Environment: chunked grid fields with active/dirty masks.
- Audio: bounded emitter/listener working sets and query caches.
- Runtime/events: pooled event packets and bounded queues.

## Bandwidth and locality law
- Memory bandwidth-sensitive loops require dedicated profiling evidence.
- Random-access-heavy loops must provide locality mitigation plan.
- Fragmented storage on hot loops is prohibited without explicit waiver.

## Fragmentation and pool policy
- Hot-path allocations must prefer pools/arenas/slabs.
- Pool sizing is scenario-driven and periodically validated.
- Fragmentation drift beyond threshold is a performance gate signal.

## Streaming memory ceiling
- Streaming caches have explicit ceiling and eviction policy.
- Prefetch cannot violate active simulation memory hard cap.

## Content footprint law
- Cooked runtime blobs must be locality-optimized and compact.
- Optional editor/debug metadata separated from shipping hot blobs.

## Emergency memory degradation
If nearing hard ceilings, degrade in controlled order:
1. Reduce active cell precision in far/low-priority regions.
2. Shrink debris history and proxy non-critical remnants.
3. Trim far-field caches and prefetch windows.
4. Lower replay/debug capture detail.
5. Merge aggregate populations sooner.

## Evidence requirements
- Memory footprint by L0/L1/L2/L3.
- Bandwidth-sensitive hotspot counters.
- Pool usage and overflow statistics.
- Fragmentation and allocator churn reports.
