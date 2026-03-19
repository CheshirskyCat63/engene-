# ENGENE 2.0 Event Model

## Purpose
Define runtime signal contracts so heavy simulation systems communicate without uncontrolled event chaos.

## Event classes
1. **Gameplay/runtime events**: authoritative interactions (impact, damage, collapse trigger).
2. **Persistence-worthy events**: events that mutate durable state and must survive save/load.
3. **Debug-only events**: telemetry/visualization signals, never authoritative.
4. **SDK-observable events**: stable event stream for tooling panels.
5. **Rate-limited events**: high-frequency events with aggregation windows.
6. **Aggregated summary events**: L2/L3 periodic summaries.

## Core taxonomy (minimum)
- `BallisticImpact`
- `MaterialFractured`
- `SupportWeakened`
- `CollapseTriggered`
- `FireIgnited`
- `FireCellAdvanced`
- `CellWetted`
- `RainFrontAdvanced`
- `ContainerPunctured`
- `LeakFlowUpdated`
- `AudioEventEmitted`
- `AIHeardSound`
- `RegionSimulationSummarized`
- `PopulationPromoted`
- `PopulationDemoted`


## Weather event policy
Minimum weather edge taxonomy:
- `WeatherStateChanged`
- `RainStarted`
- `RainStopped`
- `StormSpawned`
- `StormDissipated`
- `FogEnteredThreshold`
- `FogExitedThreshold`

Policy:
- Weather events are edge notifications, not durable weather truth.
- Durable weather truth is `engine_world` weather state + published frame snapshot.
- Consumers must not infer long-lived weather truth from a single event.
- If weather events are delayed/dropped by policy, snapshot truth remains authoritative.

## Ordering guarantees
- Per-domain ordering is guaranteed by tick/frame ownership.
- Cross-domain global total ordering is not guaranteed unless explicitly declared for a pipeline.
- Every event includes tick/frame timestamp contract fields.

## Delivery semantics
- Default: at-most-once delivery.
- Exactly-once required only for explicit authoritative channels.
- Aggregated replacement semantics must preserve counters/severity metadata.
- Dropped events must be observable via telemetry counters.

## Event identity contract
Each event carries:
- Event ID.
- Correlation ID.
- Source subsystem.
- Authority tag.
- Replay eligibility flag.

## Event payload discipline
- Payloads are immutable DTOs.
- No direct pointers to private runtime internals.
- No transient references without stable handles/IDs.
- No large heap-owned payloads in high-frequency burst channels.
- String payloads forbidden in burst gameplay channels.
- Heavy debug context only in sampled/optional channels.

## Threading and throughput rules
- Hot-path publication must be allocation-bounded.
- Burst domains use preallocated ring buffers or pooled packets.
- Cross-thread transfer must avoid blocking producer threads.
- Declared authoritative channels require deterministic merge ordering.
- Debug/telemetry channels must never block gameplay-critical simulation.

## Local vs cross-system signals
- Local-only: transient subsystem internals (e.g., per-iteration solver debug).
- Cross-system authoritative: affects other systems or persistence.
- Cross-system observational: SDK/perf visibility only.

## Serialization policy
- Persist state, not raw event history by default.
- Persist event history only for replay/debug captures explicitly enabled.
- Debug-only events are non-serializable truth.

## Truth rules
- Events are triggers/notifications, not long-term authoritative storage.
- Durable truth resides in world/component state.
- On load, systems reconstruct transient event flows from durable state.

## Rate limits and backpressure policy
- Burst channels (impacts, debris, audio) require fixed max events per frame.
- First action: aggregate low-priority burst events.
- Second action: drop non-authoritative cosmetic events with counter increments.
- Never drop critical authoritative events; escalate to hard error if queue safety is at risk.
- SDK must display raw, aggregated, and dropped counts.

## Event bus contract
- One runtime event bus facade in `engine_runtime`.
- Typed channels by domain; no stringly-typed event routing.
- Backpressure behavior is explicit and observable.
