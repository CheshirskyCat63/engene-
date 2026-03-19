# ENGENE 2.0 Entrypoints

This file is kept for compatibility with existing references.

**Canonical launch truth now lives in `docs/canonical/ENTRYPOINT_TRUTH.md`.**

## Current launch commands

```bash
cargo run --bin engene_game
cargo run --bin engene_sdk
cargo run --bin engene_headless -- --ticks 1200
cargo run --bin engene_tools
```

## Important rule

Do not present `engene_test` as canonical in the transition branch unless it exists again in Cargo.
