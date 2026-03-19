# CONFIG_WIRING_MATRIX

Status: IMPLEMENTED

Canonical required config set is defined in `src/core/game_config.rs` as `CANONICAL_CONFIG_FILES` (16 files under `game/data`).

## Runtime required
All listed canonical files are required for strict validation via `validate_required_configs*`.

## Optional
- Shader files are treated as optional runtime assets in doctor checks (embedded fallback assumption).

## Tooling-only
- No separate tooling-only RON set is declared in canonical config API in this pass.

## Validation wiring
- Canonical parser/required checks: `core::game_config`.
- Doctor `check_ron_configs` now delegates to canonical validation and reports parse/presence errors in strict mode diagnostics.

Verdict: IMPLEMENTED.
