# PLATFORM_AUDIT

## Current package truth

| Crate | Status | Notes |
|---|---|---|
| `engene` (root) | **Active** | migration shell, hosts 4 canonical bins |
| `engine_core` | Active | core contracts, events, scheduling |
| `engine_ecs` | Active | ECS and entity lifecycle |
| `engine_world` | Active | world truth, spatial, persistence |
| `engine_runtime` | Active | runtime orchestration |
| `engine_render` | Active | rendering |
| `engine_physics` | **Active seam** | real minimal bootstrap seam (`enabled_minimal`, `disabled`, `validate`) |
| `engine_audio` | Active | audio runtime |
| `engine_content` | Active | content pipeline |
| `engine_tools` | Active | maintenance tooling |
| `sdk_app` | Active | SDK/editor |
| `game_framework` | Active | game runtime composition |
| `apps/*` | Declared | not current canonical launch |

## Current launch truth

| Role | Command | Owner |
|---|---|---|
| Game | `cargo run --bin engene_game` | root bin |
| SDK | `cargo run --bin engene_sdk` | root bin |
| Headless | `cargo run --bin engene_headless -- --ticks 1200` | root bin |
| Tools | `cargo run --bin engene_tools` | root bin |

## Current fast verification truth

### Smoke lane
- `just smoke`
- `cargo smoke`
- `scripts/test/smoke.sh`
- `scripts/test/smoke.ps1`

Targets:
- `engine_contracts`
- `production_candidate`
- `entrypoint_and_operator_truth`
- `ci_surface_contracts`

### Contract lane
- `just contracts`
- `cargo contracts`
- `scripts/test/contracts.sh`
- `scripts/test/contracts.ps1`

Targets:
- `physics_core_boundary_contracts`
- `physics_bootstrap_contracts`
- `runtime_phase_contracts`
- `spatial_dirty_contracts`

### Legacy recovery lane
- `just legacy-recovery`
- `scripts/test/legacy_recovery.sh`
- `scripts/test/legacy_recovery.ps1`

Targets:
- `world_streaming` (isolated in tests_legacy/)
- `physics_body_combat` (isolated in tests_legacy/)
- `content_pipeline` (isolated in tests_legacy/)
- `persistence_full` (isolated in tests_legacy/)
- `runtime_systems` (isolated in tests_legacy/)
- `gameplay_and_ai` (isolated in tests_legacy/)

### Certification lane
- `just certification`
- `cargo cert`
- `scripts/test/certification.sh`
- `scripts/test/certification.ps1`

Targets:
- `certification_boundary_overhead`
- `certification_kernel_throughput`
- `certification_tick_budget`
- `certification_perf_snapshot`

### Perf lane
- `just perf`
- `scripts/test/perf.sh`
- `scripts/test/perf.ps1`

Targets: benches only.

## Current doc truth

| Document | Status |
|---|---|
| `CURRENT_BRANCH_STATE.md` | ✅ accurate |
| `ENTRYPOINT_TRUTH.md` | ✅ accurate |
| `TEST_LANE_MAP.md` | ✅ accurate |
| `WORKSPACE_OWNERSHIP_MAP.md` | ✅ accurate |
| `ROOT_CRATE_POLICY.md` | ✅ accurate |
| `FEATURE_ROLE_POLICY.md` | ✅ accurate |
| `RUNTIME_INVARIANTS.md` | ✅ accurate |
| `PLATFORM_AUDIT.md` | ✅ accurate |
| `PHYSICS_CORE_BOUNDARY.md` | ✅ exists |
| `PHYSICS_BOOTSTRAP_CONTRACT.md` | ✅ exists |

## Current test truth

| Test file | Status |
|---|---|
| `engine_contracts.rs` | ✅ Core contracts, production objects |
| `production_candidate.rs` | ✅ Production candidate gate |
| `entrypoint_and_operator_truth.rs` | ✅ Exact match to docs |
| `ci_surface_contracts.rs` | ✅ CI structure verification |
| `runtime_phase_contracts.rs` | ✅ Source text checks + real production calls |
| `spatial_dirty_contracts.rs` | ✅ Real spatial dirty policy contracts |
| `physics_core_boundary_contracts.rs` | ✅ Boundary tests exist |
| `physics_bootstrap_contracts.rs` | ✅ Bootstrap tests exist |
| `world_streaming.rs` | 🔄 Isolated in tests_legacy/ (broken legacy) |
| `physics_body_combat.rs` | 🔄 Isolated in tests_legacy/ (broken legacy) |
| `content_pipeline.rs` | 🔄 Isolated in tests_legacy/ (broken legacy) |
| `persistence_full.rs` | 🔄 Isolated in tests_legacy/ (broken legacy) |
| `runtime_systems.rs` | 🔄 Isolated in tests_legacy/ (broken legacy) |
| `gameplay_and_ai.rs` | 🔄 Isolated in tests_legacy/ (broken legacy) |

## Current CI truth

Jobs in PR CI:
- `fmt` — `cargo fmt --all -- --check`
- `clippy` — `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `build` — `cargo build --verbose`
- `smoke` — nextest, engine_contracts + production_candidate + entrypoint + ci_surface
- `contracts` — nextest, runtime_phase + physics_boundary + physics_bootstrap

Jobs NOT in PR CI:
- `certification`
- `perf`

Branches watched: `main`, `engene-2.0-transition`

## Exact blockers

1. ~~physics-to-core boundary not documented~~ — ✅ Now exists: `PHYSICS_CORE_BOUNDARY.md`
2. ~~physics bootstrap contract not documented~~ — ✅ Now exists: `PHYSICS_BOOTSTRAP_CONTRACT.md`
3. ~~physics tests not present~~ — ✅ Now exists: `physics_core_boundary_contracts.rs`, `physics_bootstrap_contracts.rs`
4. ~~runtime_phase_contracts uses text scan~~ — ✅ Includes real production calls (`ToolsRuntimeAssembly::minimal()`, `doctor::run_doctor()`)

## Exact "not yet true" statements

1. `apps/*` packages are declared but **not** current canonical launch truth
2. `engine_physics` is a minimal seam, not a full physics stack migration endpoint
3. Split is declared but **not** fully enforced
4. Root package still hosts active execution and broad exports
5. ~~No physics boundary tests exist~~ — ✅ Now exist
6. ~~No physics bootstrap contract tests exist~~ — ✅ Now exist
7. ~~runtime_phase_contracts.rs does not call production runtime assembly path~~ — ✅ Now does

## Fast verification assessment

**Cold path width:** Still wide because:
- Root shell (`engene`) pulls all engine dependencies
- Heavy transitive dependencies from root to physics seam
- No fast-engine-only crate to bypass root

**Smoke lane:** Contains only fast tests, correct.
**Contract lane:** Contains behavioral tests, correct.

**Status:** Fast verification path is truthful and currently green, but not maximally small due to root-shell architecture.

Migration is not finished while root shell remains active.
Next step after this stabilization pass is structural movement, not another audit pass.

Spatial update policy is explicit in SDK runtime:
- `no_op` when no dirty input,
- incremental update when dirty entities exist,
- controlled full rebuild on structural invalidation (chunk/origin-shift/recovery).

## Rule

This file is a snapshot. Update when platform truth changes.
Do not document intentions as current state.
