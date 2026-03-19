# ENGENE 2.0 Entity Lifecycle

## Purpose
Define a single lifecycle law for entity identity, authority, and state transitions across ECS, streaming, persistence, and L0-L3 simulation.

## Entity states
1. **Transient entity**: runtime-only helper entity, no long-term identity guarantee.
2. **Persistent entity**: durable identity with save/load continuity requirements.
3. **Promoted entity**: reconstructed or activated into higher-fidelity layer (L2/L3 -> L1/L0).
4. **Demoted aggregate**: folded into summary representation (L0/L1 -> L2/L3).
5. **Unloaded entity**: not instantiated in local ECS; represented by persisted/aggregate state.
6. **Dead entity**: terminal gameplay state; may persist as corpse record, marker, or removed identity.
7. **Restored entity**: re-instantiated from save/load snapshot.
8. **Replicated-from-summary entity**: materialized from aggregate summary + deterministic seed.

## Authority owner by stage
- L0/L1 active simulation owns fine-grained tactical truth.
- L2/L3 owns aggregate strategic truth.
- Persistence owns durable snapshots between sessions.
- Reconstruction derives missing high-fidelity detail but cannot violate authoritative aggregates.

## Identity guarantees
- Persistent entities must keep stable IDs across promote/demote/save/load transitions.
- Transient entities are explicitly non-stable and must never be referenced as durable truth.
- Summary-derived entities must carry origin/correlation metadata.

## Reference validity rules
- Cross-system references must use stable handles/IDs, never raw pointers.
- References to demoted or unloaded entities resolve to aggregate handles or invalid state explicitly.
- Dangling references are P0 if they can alter authoritative gameplay outcomes.

## Truth-loss policy on demotion
Allowed to lose:
- Fine animation state.
- Local solver/transient caches.
- Cosmetic-only runtime noise.
Must retain:
- Combat-relevant outcomes.
- Inventory/loadout truth.
- Structural participation and faction ownership.
- Persistence-critical flags.

## Save/load survival law
Must survive save/load:
- Persistent ID.
- Simulation layer ownership.
- Gameplay-critical stats and status.
- Required causality anchors for reconstruction.
May be reconstructed:
- Secondary visuals.
- Non-authoritative helper caches.
