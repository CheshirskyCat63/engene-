# Phase A API Surface Report (Batch 8 closure) + Phase B Batch 12 update

## New crate facades
Each new crate exposes a minimal `api` module to establish stable public entrypoints:
- `engine_core::api`
- `engine_ecs::api`
- `engine_world::api`
- `engine_runtime::api`
- `engine_render::api`
- `engine_physics::api`
- `engine_audio::api`
- `engine_content::api`
- `engine_tools::api`
- `sdk_app::api`
- `game_framework::api`

## `engine_core` exported modules (Phase A closure state)
- `engine_core::time`
- `engine_core::failure_taxonomy`
- `engine_core::deterministic_merge`
- `engine_core::runtime_config`
- `engine_core::build_manifest`
- `engine_core::runtime_manifest`
- `engine_core::determinism_policy`
- `engine_core::data_policy`
- `engine_core::registry`
- `engine_core::metrics_registry`
- `engine_core::ownership_map`
- `engine_core::integration_matrix`
- `engine_core::profiler`

## `engine_content` exported modules (Phase A closure state)
- `engine_content::asset_budget`
- `engine_content::schema_governance`
- `engine_content::pipeline`
- `engine_content::import::{asset_pipeline, hashes}`
- `engine_content::cooking::{cook_pipeline, dependency_graph}`
- `engine_content::prefabs::{prefab, prefab_registry}`
- `engine_content::validation::content_validator`
- `engine_content::content_hash`

## `engine_ecs` exported modules (Phase A closure state)
- `engine_ecs::sparse_set`
- `engine_ecs::access::{ecs_view, resource_view, queries}`
- `engine_ecs::persistent_id`
- `engine_ecs::system_descriptor`
- `engine_ecs::commands`
- `engine_ecs::parallel_validation`
- `engine_ecs::query_contract` including:
  - `EntityMatcher<Ctx>`
  - `QueryFilter<Ctx>`
  - `AndMatcher<A, B>` and alias `And<A, B>`
  - `FilteredEntityIter<'a, Ctx, M>` and alias `QueryIter<'a, Ctx, F>`
  - `ReadComponent<'a, T>` / `WriteComponent<'a, T>`
  - `collect_matching(...)` / `count_matching(...)`

## `engine_runtime` exported modules (Phase B Batch 12)
- `engine_runtime::simulation_core::contracts` including:
  - `SimulationLevel`, `SimulationPolicyProfile`, `TransitionSpeedLawTargets`
  - `PromotionRequest`, `DemotionRequest`, `TransitionRequest`
  - `TransitionReason`, `TransitionResult`, `TransitionHandoff`
  - `TransitionAuthority`, `ReconciliationMarker`, `TransitionDisposition`
  - `TransitionExecutionContext`, `TransitionMetricsSnapshot`
  - `TransitionOrderingPolicy`, `DeferredTransitionQueue`, `DeferredTransitionEntry`, `DeferredTransitionQueueSnapshot`, `DeferredTransitionPolicy`
  - `TransitionShardOutput`, `TransitionShardMergePolicy`
  - `TransitionTraceRecord`, `SimulationLevelDebugVisibility`
- `engine_runtime::simulation_core::orchestrator` including:
  - `TransitionOrchestrator` (including deterministic `order_batch`, `merge_shard_outputs`, and ordered batch resolution helpers)
  - `TransitionTracer`
  - `TransitionHandoffSink`

## Compatibility bridge (Phase B Batch 4 state)
- Monolith `engene` remains active as assembly/compatibility shell.
- `src/simulation/simulation_level.rs` bridges world component level values to runtime-owned simulation-core contracts.
- `src/simulation/activation.rs` supplies explicit frame/tick context, consumes orchestrator metrics/handoff hooks, and preserves world-owned ECS truth.

## Internal wiring policy
- Runtime owns scheduling/orchestration contracts for simulation transitions.
- Domain truth (world/render/audio/physics/game) remains outside `engine_runtime`.


## Bench/regression tooling (Phase B Batch 12)
- `benches/simulation_transition_core.rs` retains the same fair ST/MT benchmark pairs (Batch 7 truth) while Batch 11 focuses on structural transition-core closure work and severity-tagged runtime gate truth output:
  - `classify_only_single_thread` vs `classify_only_multi_thread_x8`
  - `classify_materialize_single_thread` vs `classify_materialize_multi_thread_x8`
- `scripts/check_transition_speed_law.sh` enforces per-surface latency/throughput thresholds for all transition-core surfaces.
- Scaling gates are pair-specific and workload-equivalent:
  - `classify_only_x8 >= 3.0x` (target 4.5x)
  - `classify_materialize_x8 >= 3.0x` (target 4.5x)
- Compile truth split is explicit:
  - compatibility check: `cargo bench --bench simulation_transition_core --no-run`
  - isolation truth: `cargo rustc --bench simulation_transition_core --profile bench -vv -- -C debuginfo=0` via `--proof-cold-build/--proof-warm-build`.
