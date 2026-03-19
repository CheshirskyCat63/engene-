# ENGENE 2.0 Concurrency Model

## Purpose
Define thread ownership, job system behavior, synchronization constraints, and determinism-safe parallel execution rules.

## Core principle
Concurrency is a production contract, not an optimization afterthought. A subsystem is incomplete unless it defines:
- what thread(s) own authoritative writes,
- how jobs are partitioned,
- where merge/barrier points exist,
- how determinism classes are preserved.

## Thread ownership model

### Main thread responsibilities
- Frame orchestration, input ingest, present/swap coordination.
- Final phase commits for authoritative world state transitions.
- Determinism-critical ordering checkpoints.

### Worker thread responsibilities
- Batch processing of simulation jobs (AI, physics slices, environment chunks, audio probes).
- Parallel precompute, query, and reduction workloads.
- Never directly finalize cross-domain authoritative truth outside merge phase.

### Ownership law
- Authoritative storage has explicit phase owner.
- Cross-thread writes into same authoritative storage in same phase are forbidden.

## Job system contract
- Jobs must declare: input handles, output buffers, determinism class, and merge target.
- Hot-path jobs must be allocation-bounded and pool-backed.
- Job stealing is allowed only for subsystems whose determinism class allows nondeterministic completion ordering.
- Deterministic channels require deterministic merge ordering regardless of worker completion order.

## Read/write phase rules
1. Read phase: parallel read across immutable snapshots.
2. Compute phase: produce commands/deltas in thread-local or partition-local buffers.
3. Merge phase: apply deltas in deterministic order to authoritative storage.
4. Publish phase: emit events/telemetry snapshots.

ECS structural mutation must occur through deferred command application only.

## Synchronization policy
- Coarse phase barriers are preferred over frequent fine-grained locks.
- No fine-grained mutex acquisition in top hot gameplay loops.
- Lock usage in hot paths requires explicit perf evidence and documented waiver.
- Blocking waits in gameplay-critical phases are budgeted and telemetry-visible.

## Forbidden lock usage
- Per-entity lock contention on L0 gameplay hot loops.
- Nested cross-subsystem locks with unbounded wait order.
- Lock-protected queues where lock-free/ring alternatives are required by burst profile.

## Queue/channel/event threading rules
- Burst channels must use preallocated ring buffers or pooled packet slabs.
- Producer threads must not block on debug/telemetry consumers.
- Backpressure policy must preserve authoritative events and degrade cosmetic/debug channels first.

## Cross-subsystem barrier points
Mandatory declared barrier windows:
- After command generation, before authoritative merge.
- After authoritative merge, before event publication.
- Before persistence snapshot extraction.

Barrier count per frame class is budgeted in CPU performance law.

## Deterministic multithreading policy
- Strict deterministic subsystems: deterministic partitioning + deterministic merge.
- Bounded deterministic subsystems: bounded variance with stable aggregate outcomes.
- Non-authoritative cosmetic paths may use nondeterministic scheduling.

## Conflict rules
- Write/write conflicts resolved only in merge phase with explicit precedence policy.
- Read/write races are treated as P0 correctness/perf hazard on authoritative paths.

## Evidence requirements
- Per-phase thread utilization.
- Barrier timeline and wait durations.
- Queue depth/backpressure counters.
- Contention reports for lock sites.
