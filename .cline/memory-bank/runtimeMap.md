# Runtime Map

high-level runtime map:
- engine/shared core lives under crates/
- app/bootstrap entrypoints live under apps/
- game-facing runtime lives under game/

runtime law:
- do not silently move ownership between these layers
- decide owner before editing
- localize changes to owning layer when possible

fast routing:
- bootstrap/start/run issues -> apps/, run_*.ps1, quickstarts
- engine/core logic -> crates/
- game-facing behavior -> game/

