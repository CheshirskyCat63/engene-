# ENGENE 2.0 Runtime Truth Model

## Purpose
Define what is authoritative truth versus derived/cache/projection state across runtime, SDK, content, persistence, and replay.

## Truth categories
1. **Authoritative truth**: canonical state that drives gameplay/persistence outcomes.
2. **Derived state**: deterministic transforms from authoritative truth.
3. **Cache state**: performance acceleration state; rebuildable.
4. **Projection state**: visualization/audio representation of truth.
5. **Debug representation**: observability overlays and telemetry views.
6. **Persisted state**: saved authoritative snapshot and required reconstruction anchors.
7. **Reconstructed approximation**: generated state on promotion/load based on authoritative seeds/summaries.

## Examples
- Material schema: canonical content truth.
- Cooked lookup tables: derived state.
- Fire active cell map (L0/L1): authoritative local truth.
- Smoke particle cloud: projection state.
- AI hearing event stream: derived signal.
- Region battle summary (L2/L3): authoritative aggregate truth.
- Promoted individual from summary + seed: reconstructed approximation.

## Law
- Projection/caches/debug data must never silently redefine authoritative truth.
- Persisted state must encode enough anchors to reconstruct valid local state.
- If mismatch occurs, authoritative truth wins; derived/projection must be regenerated.


## Sky/Weather truth separation contract
- Authoritative sky/weather truth lives in `engine_world` state.
- Runtime builds one per-frame `FrameSkyWeatherSnapshot` in `engine_runtime`.
- Published snapshot is immutable for the frame/tick.
- Render/audio/AI/debug/game readers consume snapshot view, not mutable world internals.
- Snapshot mismatch across subsystems in same frame is a truth violation.
- Weather events are edge notifications and may not redefine durable weather truth.

## Cross-system conflict resolution
- Content truth conflict -> schema law and validation pipeline decide.
- Runtime truth conflict -> layer authority precedence from simulation strategy.
- Persistence conflict -> latest valid authoritative snapshot with deterministic reconciliation.
