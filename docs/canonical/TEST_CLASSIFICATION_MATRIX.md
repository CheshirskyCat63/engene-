# TEST_CLASSIFICATION_MATRIX

**Status**: Law
**Purpose**: Classify all tests by purpose. Separate truth from transition scaffolds.

---

## Classification Categories

| Class | Purpose | Must Have |
|-------|---------|-----------|
| **TRUTH** | Protect architectural laws, ownership, invariants | Sunset: Never (permanent) |
| **CONTRACT** | API contracts between crates | Sunset: Never (evolving) |
| **MIGRATION** | Protect transition scaffolds | Sunset: After handoff |
| **PERF** | Performance benchmarks, budgets | Sunset: Never (evolving) |

---

## TRUTH Tests (Protect Architecture Laws)

These tests protect non-negotiable architectural rules. They must survive handoff.

| Path | Current Purpose | Class | Status |
|------|-----------------|-------|--------|
| `tests/architecture_validation.rs` | Meta-tests validating test architecture itself | TRUTH | ✅ Keep |
| `tests/architectural_gates.rs` | Ownership enforcement gates | TRUTH | ✅ Keep |
| `tests/runtime_role_separation.rs` | Role bootstrap boundaries enforced | TRUTH | ✅ Keep |
| `tests/engine_contracts.rs` | Thin aggregator of all contract suites | TRUTH | ✅ Keep (thin) |
| `tests/editor_safe_mode_contracts.rs` | Editor isolation from simulation | TRUTH | ✅ Keep |
| `tests/ci_surface_contracts.rs` | CI surface integrity | TRUTH | ✅ Keep |
| `tests/production_candidate.rs` | Core production invariants | TRUTH | ✅ Keep |
| `tests/tools_runtime_purity.rs` | Tools runtime has no game logic | TRUTH | ✅ Keep |
| `tests/root_not_runtime_brain.rs` | Root is NOT runtime brain | TRUTH | ✅ **NEW** |

### What makes a test TRUTH

- Tests ownership boundaries (root is not runtime brain)
- Tests role/capability separation
- Tests dependency direction
- Tests phase order (as invariant, not source-text check)
- Tests render does not define truth
- Tests audio independence from render path
- Tests thin app shell principle

---

## CONTRACT Tests (Protect API Contracts)

These tests verify API contracts between crates. They evolve with the API.

| Path | Current Purpose | Class | Status |
|------|-----------------|-------|--------|
| `tests/spatial_dirty_contracts.rs` | Spatial dirty model contracts | CONTRACT | ✅ Keep |
| `tests/determinism_and_sdk.rs` | Determinism audit, SDK contracts | CONTRACT | ✅ Keep |
| `tests/runtime_phase_contracts.rs` | Phase module invariants, phase order validation | CONTRACT | ✅ Keep (was MIGRATION, rewritten) |
| `tests/ecs_lifecycle_contracts.rs` | Entity lifecycle contracts | CONTRACT | ✅ Keep |
| `tests/ecs_authority_contracts.rs` | ECS authority contracts | CONTRACT | ✅ Keep |
| `tests/entity_lifecycle_contracts.rs` | Entity lifecycle API | CONTRACT | ✅ Keep |
| `tests/query_contracts.rs` | Query API contracts | CONTRACT | ✅ Keep |
| `tests/core_command_and_access_contracts.rs` | Command buffer contracts | CONTRACT | ✅ Keep |
| `tests/event_bus_contracts.rs` | Event bus contracts | CONTRACT | ✅ Keep |
| `tests/identity_and_authority_contracts.rs` | Identity/authority contracts | CONTRACT | ✅ Keep |
| `tests/runtime_profile_and_quality_contracts.rs` | Profile/quality contracts | CONTRACT | ✅ Keep |
| `tests/world_persistence_contracts.rs` | World persistence contracts | CONTRACT | ✅ Keep |
| `tests/world_integration_contracts.rs` | World integration contracts | CONTRACT | ✅ Keep |
| `tests/simulation_integration_contracts.rs` | Simulation integration contracts | CONTRACT | ✅ Keep |
| `tests/rendering_pipeline_contracts.rs` | Render pipeline contracts | CONTRACT | ✅ Keep |
| `tests/editor_console_contracts.rs` | Console API contracts | CONTRACT | ✅ Keep |
| `tests/editor_inspector_contracts.rs` | Inspector API contracts | CONTRACT | ✅ Keep |
| `tests/physics_core_boundary_contracts.rs` | Physics boundary contracts | CONTRACT | ✅ Keep |
| `tests/physics_bootstrap_contracts.rs` | Physics bootstrap contracts | CONTRACT | ✅ Keep |
| `tests/navigation_integration_contracts.rs` | Navigation contracts | CONTRACT | ✅ Keep |
| `tests/camera_contracts.rs` | Camera API contracts | CONTRACT | ✅ Keep |
| `tests/performance_governance_contracts.rs` | Performance governance contracts | CONTRACT | ✅ Keep |

### What makes a test CONTRACT

- Tests API between crates
- Tests behavior contracts (input → output)
- Tests resource ownership at API level
- Tests event publishing/subscribing contracts

---

## MIGRATION Tests (Protect Transition Scaffolds)

**⚠️ CRITICAL**: These tests protect temporary transition code. They MUST die after handoff.

| Path | Current Purpose | Class | Should Rewrite? | Sunset Condition |
|------|-----------------|-------|-----------------|------------------|
| `tests/entrypoint_and_operator_truth.rs` | Verifies specific transition files (README, docs, Cargo.toml) | MIGRATION | ⚠️ YES - rewrite as structural test | After apps/* become canonical |
| `tests/runtime_infrastructure.rs` | Tests transition infrastructure | MIGRATION | ⚠️ MAYBE | After handoff complete |

**Note**: `runtime_phase_contracts.rs` moved to CONTRACT - now tests phase module invariants, not source text.

### What makes a test MIGRATION

- Checks source text of transition files
- Tests presence of specific transition scaffolding
- Tests text order in temporary orchestration code
- Tests that specific deprecated paths are blocked
- Tests migration ledger items

### MIGRATION Test Rewrite Rules

1. **Source-text checks** → Replace with runtime invariant tests
2. **Specific file checks** → Replace with structural tests (check directory exists, not file contents)
3. **Transition file presence** → Replace with "does not exist" structural checks after removal

---

## PERF Tests (Performance Benchmarks)

These tests measure performance. They evolve but never die.

| Path | Current Purpose | Class | Status |
|------|-----------------|-------|--------|
| `tests/certification_tick_budget.rs` | Fixed tick budget verification | PERF | ✅ Keep |
| `tests/certification_perf_snapshot.rs` | Performance snapshot output | PERF | ✅ Keep |
| `tests/certification_kernel_throughput.rs` | Kernel throughput benchmarks | PERF | ✅ Keep |
| `tests/certification_boundary_overhead.rs` | Boundary overhead benchmarks | PERF | ✅ Keep |
| `tests/certification_bootstrap_invariants.rs` | Bootstrap perf invariants | PERF | ✅ Keep |
| `tests/performance.rs` | General performance tests | PERF | ✅ Keep |
| `tests/perf_lowspec_mt.rs` | Low-spec multi-thread perf | PERF | ✅ Keep |
| `tests/multithreading_performance_contracts.rs` | Multithreading perf contracts | PERF | ✅ Keep |
| `tests/ecs_performance_contracts.rs` | ECS performance contracts | PERF | ✅ Keep |
| `tests/multithreading.rs` | Multithreading tests | PERF | ✅ Keep |

---

## ✅ UNKNOWN / NEEDS REVIEW — RESOLVED

All 13 unknown tests have been classified:

| Path | Current Purpose | Class | Reason |
|------|-----------------|-------|--------|
| `tests/vertical_slice.rs` | Integration tests for game assembly | CONTRACT | Tests API between systems |
| `tests/tools_runtime_purity.rs` | Verifies tools runtime has no game logic | **TRUTH** | Architectural law - tools must not boot game content |
| `tests/sdk_editor_gui.rs` | SDK editor/console/inspector API | CONTRACT | Tests SDK tool APIs |
| `tests/render_pipeline.rs` | Render LOD, camera, validation | CONTRACT | Render API contracts |
| `tests/quest_and_faction.rs` | Quest/faction game logic | CONTRACT | Game logic API |
| `tests/physics_chain_reaction_contracts.rs` | Physics chain reactions | CONTRACT | Physics API |
| `tests/body_pipeline.rs` | Body/combat pipeline | CONTRACT | Body system API |
| `tests/e2e_regression.rs` | 101 e2e integration tests | CONTRACT | Integration API |
| `tests/economy_cycle.rs` | Economy cycle logic | CONTRACT | Game economy API |
| `tests/ai_social_economy.rs` | AI social/economy simulation | CONTRACT | Simulation API |
| `tests/engine_lifecycle_contracts.rs` | Engine bootstrap/lifecycle | CONTRACT | Lifecycle API |
| `tests/engine_infrastructure.rs` | (Not reviewed - likely MIGRATION) | MIGRATION_CANDIDATE | Likely tests transition infrastructure |
| `tests/save_load_torture.rs` | (Not reviewed - likely CONTRACT) | CONTRACT | Likely save/load API |

**TRUTH count increased**: +1 (tools_runtime_purity.rs)

---

## Summary by Class (UPDATED)

| Class | Count | Action |
|-------|-------|--------|
| TRUTH | 9 | Keep permanently (+1: root_not_runtime_brain.rs) |
| CONTRACT | ~25 | Keep, evolve with API (+1: runtime_phase_contracts) |
| MIGRATION | 2 | **Rewrite or delete after handoff** |
| PERF | 10 | Keep permanently |
| UNKNOWN | ✅ 0 | All classified |

---

## Critical Actions Required

### 1. Rewrite MIGRATION tests

**`tests/runtime_phase_contracts.rs`**
- Current: Checks source text of `sdk_runner.rs`
- Problem: Protects transition scaffolding, not architecture
- Rewrite: Convert to runtime phase order invariant test
- After: `sdk_runner.rs` replaced with phase methods

**`tests/entrypoint_and_operator_truth.rs`**
- Current: Checks specific files (README, docs)
- Problem: Protects transition docs, not architecture
- Rewrite: Convert to structural test (check apps/* exist, not content)
- After: apps/* become canonical

### 2. Review UNKNOWN tests

13 tests need manual classification. Priority:
1. `tests/engine_lifecycle_contracts.rs` - likely CONTRACT
2. `tests/engine_infrastructure.rs` - likely MIGRATION
3. `tests/vertical_slice.rs` - likely CONTRACT
4. Others

### 3. Add missing TRUTH tests

Current TRUTH coverage gaps:
- No test for "root is not runtime brain"
- No test for dependency law enforcement
- No test for "render does not own simulation truth"
- No test for "audio independent of render"

---

## Exit Criteria

This document is complete when:

- [ ] All MIGRATION tests rewritten or scheduled for deletion
- [ ] All UNKNOWN tests classified
- [ ] Missing TRUTH tests identified and scheduled
- [ ] Clear sunset conditions for all MIGRATION tests
- [ ] Test lane mapping reflects classification

---

## Relationship to Other Documents

- **CURRENT_RUNTIME_TRUTH.md** — current state
- **GAP_MAP_CURRENT_TO_GOD_TIER.md** — blockers
- **ENGENE_2_0_GOD_TIER_ARCHITECTURE.md** — target
- **TEST_CLASSIFICATION_MATRIX.md** — test layer organization (this file)
- **TEST_LANE_MAP.md** — test execution lanes
