# ENGENE_SDK_ARCHITECTURE

Status: IMPLEMENTED

SDK ownership surface: `src/sdk.rs`.

SDK-owned code:
- `src/tools/*`: doctor, dashboards, editor shell, debug tools
- `src/app/sdk_runner.rs`: SDK runtime/editor orchestration and event-loop integration
- `src/testsupport/*`: harnesses and stress tooling

## Integration model
- SDK depends on ENGINE runtime.
- SDK may use GAME integration points for tooling/runtime visibility.
- `src/bin/engene_sdk.rs` remains a minimal bootstrap entrypoint (argument parse + startup mode selection + handoff).

## Entrypoint split
- Binary layer: `src/bin/engene_sdk.rs`
- Runtime runner layer: `src/app/sdk_runner.rs`

This preserves platform executable requirements while keeping non-bootstrap orchestration out of the binary.
