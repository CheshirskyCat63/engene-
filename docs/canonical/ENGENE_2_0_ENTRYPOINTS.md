# ENGENE 2.0 Entrypoints

## Purpose
Single sanity-check reference for canonical runtime entrypoints to avoid multiple conflicting launch paths.

## Canonical commands

### Game executable path
```bash
cargo run --bin engene_game
```

### SDK workstation path
```bash
cargo run --bin engene_sdk
```

### Tools diagnostics path
```bash
cargo run --bin engene_tools
```

### Headless runtime path
```bash
cargo run --bin engene_headless -- --ticks 1200
```

## Entrypoint ownership map (code-level)
| Role | Binary file | Runner owner | Bootstrap owner |
|---|---|---|---|
| Game | `src/bin/engene_game.rs` | `src/app/game_runner/mod.rs` | `src/runtime/bootstrap/game.rs::GameRuntimeAssembly::vertical_slice` |
| Tools/SDK | `src/bin/engene_sdk.rs` | `src/app/sdk_runner.rs` | `src/runtime/bootstrap/tools.rs::ToolsRuntimeAssembly::minimal` |
| Tools diagnostics | `src/bin/engene_tools.rs` | `src/app/tools_runner.rs` | `src/runtime/bootstrap/tools.rs::ToolsRuntimeAssembly::minimal` |
| Headless/Kernel | `src/bin/engene_headless.rs` | `src/app/headless_runner.rs` | `src/runtime/bootstrap/headless.rs::EngineRuntimeAssembly::kernel_headless` |

Root-level `src/main.rs` dispatcher is intentionally removed to keep entrypoint ownership unambiguous.

## Canonical rule
- These commands are the default canonical launch entrypoints.
- Any alternate launch method must be documented and mapped to one of these runtime products.
- Deprecated launch scripts/binaries must not be presented as primary entrypoints.
- Dependency direction and bootstrap boundaries are defined in `ENGENE_2_0_RUNTIME_DEPENDENCY_BASELINE.md`.

## Validation reminder
When workspace split to `apps/*` is complete, this file must be updated to crate/package-level canonical commands while preserving single-source-of-truth semantics.
