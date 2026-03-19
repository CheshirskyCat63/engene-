# ENGENE 2.0 Determinism Policy

## Purpose
Define determinism classes, acceptable divergence, and evidence requirements so teams do not debate implicit assumptions.

## Determinism classes
1. **Strict deterministic**: identical input + seed + build must produce identical authoritative outcome.
2. **Bounded deterministic**: small variance allowed within declared acceptance bands.
3. **Non-deterministic non-authoritative**: cosmetic variability allowed; cannot change gameplay truth.
4. **Replay-captured**: behavior may be non-deterministic at runtime but must be reproducible when capture stream is replayed.

## System classification
- Promotion/demotion reconciliation: strict or tightly bounded deterministic.
- Break-state selection logic: strict deterministic.
- Macro weather evolution: bounded deterministic.
- Debris spray visuals: non-deterministic non-authoritative.
- Ambience voice stealing: bounded/non-authoritative within class.

## Seed ownership and RNG scoping
- Seeds are owned per simulation domain (world, region, entity group).
- RNG streams must be scoped; no global shared mutable RNG for cross-domain logic.
- Cross-system deterministic events must include seed context in telemetry.

## Replay guarantees
- Canonical validation scenarios require replay-capable capture bundles.
- Replay includes seed map, event correlation IDs, and content/build hashes.
- Missing replay metadata in canonical scenario is a gate failure.

## Acceptable divergence bands
- Define numeric tolerances per subsystem (position drift, timing drift, aggregate outcome drift).
- Drift beyond band is classified as mismatch and triaged.

## Mismatch classification
- **P0**: divergence changes authoritative gameplay outcomes or persistence truth.
- **P1**: divergence breaks phase acceptance criteria but not release truth yet.
- **Debug-only**: divergence only in cosmetic projections.

## Bug capture requirements
Determinism bug reports must include:
- Build hash.
- Content/schema hash.
- Seed map.
- Capture/replay artifact.
- Divergence point and subsystem tag.
