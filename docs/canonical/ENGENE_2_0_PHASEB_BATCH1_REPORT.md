# ENGENE 2.0 Phase B Batch 1 Report

## Scope
Phase B Batch 1 starts simulation-core transition work by establishing runtime-owned contracts and a minimal L0/L1/L2/L3 orchestration skeleton. Work is intentionally restricted to scheduling/orchestration contracts and compatibility bridges.

## What was added
- `engine_runtime::simulation_core::contracts` now owns:
  - simulation level contract (`SimulationLevel`)
  - simulation policy profile (`SimulationPolicyProfile`)
  - promotion/demotion request contracts (`PromotionRequest`, `DemotionRequest`, `TransitionRequest`)
  - transition reason/result contracts (`TransitionReason`, `TransitionResult`, `TransitionHandoff`)
  - authority/reconciliation markers (`TransitionAuthority`, `ReconciliationMarker`)
  - observability contracts (`TransitionTraceRecord`, `SimulationLevelDebugVisibility`)
- `engine_runtime::simulation_core::orchestrator` now owns:
  - minimal `TransitionOrchestrator`
  - transition request resolution skeleton
  - transition tracer callback contract (`TransitionTracer`)
  - debug visibility aggregation helper

## Compatibility strategy
- Monolith simulation-level helpers remain active.
- `src/simulation/simulation_level.rs` now bridges world-level components to runtime-owned simulation contracts.
- `src/simulation/activation.rs` now uses the runtime orchestrator to classify and resolve level transitions before writing back to ECS world components.

## Law-compliance notes
- Domain truth was not moved out of `engine_world`/feature systems.
- `engine_runtime` additions are orchestration contracts and transition scheduling only.
- No gameplay feature work and no deep subsystem migrations were performed.

## Validation
- `bash scripts/check_dependency_direction.sh`
- `bash scripts/check_ecs_direct_access.sh`
- `cargo check --workspace -q`
