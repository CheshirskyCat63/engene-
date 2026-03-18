# ENGENE Repository Map (Phase 0)

## Top-level ownership map
- `src/` — runtime/source code (engine+sdk+game modules in monolith pre-Phase A).
- `game/` — authored runtime data/content/saves.
- `tests/` — integration and systems test suites.
- `benches/` — benchmark harnesses.
- `docs/canonical/` — authoritative ENGENE 2.0 law/program docs (`ENGENE_2_0_*.md`).
- `docs/archive/` — historical and archived reports/specs.
- `legacy/quarantine/` — quarantined non-canonical legacy artifacts.
- `scripts/` — architecture boundary and ECS discipline gates.

## Canonical launch entrypoints
- `cargo run --bin engene_game`
- `cargo run --bin engene_sdk`
- `cargo run --bin engene_test`
- `cargo run --bin engene_headless`

See `docs/canonical/ENGENE_2_0_ENTRYPOINTS.md` for entrypoint law.
