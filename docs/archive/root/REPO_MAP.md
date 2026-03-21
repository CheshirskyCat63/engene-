# ENGENE Repository Map (Phase 0)

## Top-level ownership map
- `src/` — runtime/source code (engine+sdk+game modules in monolith pre-Phase A).
- `src/runtime/` — runtime role bootstrap and runtime-wiring glue (non-game domain ownership).
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
- `cargo run --bin engene_tools`
- `cargo run --bin engene_headless -- --ticks 1200`
- No root `src/main.rs` alias dispatcher remains; binary ownership is explicit in `src/bin/*`.

See `docs/canonical/ENGENE_2_0_ENTRYPOINTS.md` for entrypoint law.
See `docs/canonical/ENGENE_2_0_FOUNDATION_STRUCTURE_MAP.md` for ownership-first layout map.
See `docs/canonical/ENGENE_2_0_PUBLIC_API_SURFACE_MAP.md` for API exposure classification.
See `docs/canonical/ENGENE_2_0_DEPENDENCY_OWNERSHIP_MAP.md` for direct dependency ownership.
