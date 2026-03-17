# V1_SCOPE_LOCK

Scope lock status: PARTIAL.

## Locked
- Engine runtime core/ECS/world/persistence foundations.
- Game layer in `src/game/*` as semantic ownership center.
- Simulation/animation runtime ownership split completed (no mixed zones).
- SDK tools ownership via `src/tools/*`, `src/app/*` and `src/sdk.rs` surface.

## Not fully locked
- Full physical redistribution into nested `src/engine/*` and `src/sdk/*` tree is not completed.
- ECS direct-access allowlist is still wider than target minimum.
- Thin binary entrypoint extraction is incomplete.
