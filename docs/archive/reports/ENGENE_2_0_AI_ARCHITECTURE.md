# ENGENE 2.0 AI Architecture

## Purpose
Define multi-level AI architecture by both distance and semantic role to support 10k enemies + 5k allies.

## AI layer split

### L0 — Individual AI (high-fidelity)
- Scope: entities in active bubble.
- Model: perception, tactical movement, weapon use, local reactions.
- Persistence: full tactical state for promoted actors.

### L1 — Squad AI (near-field)
- Scope: nearby but not fully simulated individuals.
- Model: squad objectives, formation intent, engagement stance.
- Persistence: squad-level intent + member aggregates.

### L2 — Regional tactical simulation
- Scope: loaded regional cells outside near-field.
- Model: force-vs-force abstractions, attrition, morale, supply influence.
- Persistence: region battle summaries + unit strength vectors.

### L3 — Strategic population simulation
- Scope: far-field/unloaded world.
- Model: faction pressure, migration, reinforcement generation, macro conflict.
- Persistence: strategic counters and event summaries only.

## Semantic role × level matrix

| Role | L0 | L1 | L2 | L3 |
|---|---|---|---|---|
| Combatant | Full combat brain | Squad slot behavior | Regional force vector | Strategic pressure pool |
| Civilian | Panic/pathing + local needs | Crowd intent | Migration stats | Demography pressure |
| Logistics | Carrier/convoy local behavior | Convoy intent | Supply flow node | Macro stock pressure |
| Command | Local tactical commander | Squad doctrine | Regional objective allocator | Strategic directive layer |

## Promotion trigger taxonomy
- Distance trigger.
- Visibility trigger.
- Ballistic interaction trigger.
- Audio/perception trigger.
- Structural hazard trigger.
- Narrative/showcase trigger.
- Persistence boundary trigger.

## Demotion safety rules
- Never demote actor in unresolved close combat.
- Never demote actor owning critical local truth.
- Never demote while actor has pending gameplay-critical event chain.
- Demote only after cooldown + stable entropy window.

## AI performance law
- L0 updates are budget-capped and batched by archetype/role.
- L1 squad updates run at reduced cadence and aggregate local reasoning.
- L2/L3 updates are table-driven/statistical, not per-agent behavior loops.
- Promotions per frame are hard-capped.
- Demotion/reconstruction work is time-sliced.
- Perception queries must use spatial prefilters and batched LOS evaluation.
- Hot-loop AI code avoids heap allocation/dynamic dispatch unless profiled safe.

## AI multithreading policy
- L0 tactical evaluation parallelizes by spatial partition or squad partition.
- Command generation may be parallel.
- Command application/writeback is merge-phase-bound.
- High-contention shared blackboards are forbidden on hot paths.

## AI reconstruction cost control
- Reconstruction budget is capped per frame.
- Expensive reconstruction can be staged over frames if causality remains believable.
- Actors outside immediate interaction radius receive shell-first reconstruction before full tactical richness.
- Reconstruction profiles vary by role (combatant/civilian/logistics/command).

## AI observability contract
Gameplay-affecting decisions must expose:
- Decision reason chain.
- Current goal.
- Threat model snapshot.
- Perception inputs.
- Squad command source.
- Current abstraction layer source.
- Reconstruction source metadata on promotion.

## AI truth-loss policy
Allowed loss when demoting:
- Fine-grained animation posture.
- Micro-navigation intent history.
Must retain:
- Combat outcomes.
- Health/loadout/relation truth.
- Objective progress and command intent.

## Authority and reconstruction
- L2/L3 own far-field truth.
- L0/L1 own near-field behavior truth.
- On promotion, reconstruct individuals from aggregate summaries + deterministic seeds.

## Non-negotiables
- Never simulate all 15k actors at L0.
- No direct dependency from AI logic to rendering internals.
- All AI decisions that affect gameplay must be inspectable in SDK.
