# LOW_SPEC_EQUIVALENCE.md — Acceptable vs Unacceptable Degradation

## Acceptable Degradation (low_spec feature)

| System | Degradation | Impact |
|--------|-------------|--------|
| Particles | Fewer particles, simpler emitters | Visual only |
| Vegetation | Reduced density, no wind animation | Visual only |
| Shadows | Fewer cascades, lower resolution | Visual only |
| Ragdoll | Simplified death animation, fewer bones | Visual only |
| Procedural animation | No foot IK, no micro-motion at distance | Visual only |
| Audio reverb | Reduced reverb zones, fewer channels | Audio fidelity |
| AI cadence | Lower update frequency at L2+ distance | Behavioral detail |
| LOD | More aggressive culling, earlier impostor switch | Visual only |
| Post-processing | Reduced bloom, no SSR, simplified TAA | Visual only |
| Skinning | Fewer bone updates for distant entities | Visual only |

## Unacceptable Degradation (NEVER allowed)

| Invariant | Reason |
|-----------|--------|
| Persistence truth lost | Save/load must be identical regardless of spec |
| Quest state broken | Player progress must not change with graphics settings |
| Combat resolution different | Hit/miss, damage, death must be deterministic |
| Critical entity refs lost | Entity relationships must survive spec changes |
| Save/load integrity broken | Saves from low-spec must load on high-spec and vice versa |
| Economy outcomes different | Trading, pricing, NPC wealth must not depend on visual fidelity |
| NPC goal decisions different | AI decisions at L0 must be identical regardless of spec |
| Faction reputation drift | Reputation changes must be spec-independent |
| Chunk persistence format different | Same format, same schema version |
| Entity despawn on spec change | Lowering spec must not kill or remove entities |

## Simulation Level Guarantees

| Level | Spec-independent? | Notes |
|-------|-------------------|-------|
| L0 (full sim) | Yes | All logic runs identically |
| L1 (simplified) | Yes | Reduced visual detail only |
| L2 (background) | Yes | Same state transitions, no rendering |
| L3 (frozen truth) | Yes | State preserved, no updates |

## Verification

- Headless sim produces identical results regardless of `low_spec` feature
- Save file from `low_spec` build loads in `full` build without errors
- Deterministic replay passes on both spec levels
