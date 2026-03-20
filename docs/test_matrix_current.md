# Test Matrix - Current State Analysis

## Current Test Files Analysis

| File | Lines | Owner | Type | Lane | Speed | Invariant | Issues |
|------|-------|-------|------|------|-------|-----------|---------|
| `engine_contracts.rs` | 1,205 | **MEGA** | Mixed | Smoke | Mixed | Multiple domains | **SPLIT NEEDED** |
| `core_ecs.rs` | 50,531 | **MEGA** | Mixed | None | Heavy | Entity lifecycle | **SPLIT NEEDED** |
| `architectural_gates.rs` | 281 | Architecture | Contract | Smoke | Fast | Ownership rules | Good |
| `runtime_phase_contracts.rs` | 8,297 | Runtime | Contract | Contracts | Medium | Phase ordering | Good |
| `physics_core_boundary_contracts.rs` | 12,304 | Physics | Contract | Contracts | Medium | Physics boundaries | Good |
| `physics_bootstrap_contracts.rs` | 5,480 | Physics | Contract | Contracts | Medium | Bootstrap contracts | Good |
| `spatial_dirty_contracts.rs` | 3,623 | World | Contract | Contracts | Fast | Spatial updates | Good |
| `certification_boundary_overhead.rs` | 1,246 | Performance | Certification | Certification | Heavy | Performance limits | Good |
| `certification_kernel_throughput.rs` | 1,665 | Performance | Certification | Certification | Heavy | Kernel performance | Good |
| `certification_tick_budget.rs` | 1,208 | Performance | Certification | Certification | Heavy | Tick budgets | Good |
| `certification_perf_snapshot.rs` | 2,213 | Performance | Certification | Certification | Heavy | Performance snapshots | Good |
| `determinism_and_sdk.rs` | 14,002 | SDK | Integration | None | Medium | SDK determinism | Needs lane |
| `tools_runtime_purity.rs` | 2,774 | Tools | Contract | None | Fast | Tools isolation | Needs lane |
| `entrypoint_and_operator_truth.rs` | 8,441 | Apps | Contract | Smoke | Fast | Entry points | Good |
| `ci_surface_contracts.rs` | 7,517 | CI | Contract | Smoke | Fast | CI contracts | Good |
| `production_candidate.rs` | 5,792 | Release | Integration | Smoke | Medium | Release readiness | Good |

## Current Test Coverage by Lane

### Smoke Lane (4 targets) ✅
- `engine_contracts` - **MEGA** - needs split
- `production_candidate` ✅
- `entrypoint_and_operator_truth` ✅  
- `ci_surface_contracts` ✅

### Contracts Lane (4 targets) ✅
- `physics_core_boundary_contracts` ✅
- `physics_bootstrap_contracts` ✅
- `runtime_phase_contracts` ✅
- `spatial_dirty_contracts` ✅

### Certification Lane (4 targets) ✅
- `certification_boundary_overhead` ✅
- `certification_kernel_throughput` ✅
- `certification_tick_budget` ✅
- `certification_perf_snapshot` ✅

### Missing Lane Assignments ❌
- `core_ecs.rs` - **MEGA** - needs split + lane
- `determinism_and_sdk.rs` - needs lane
- `tools_runtime_purity.rs` - needs lane
- Multiple integration files without lane assignment

## Test Type Distribution

| Type | Current Count | Target Count | Gap |
|------|---------------|---------------|-----|
| Unit | ~15 | 150 | -135 |
| Integration | ~8 | 120 | -112 |
| Contract | ~12 | 100 | -88 |
| Scenario/Golden | ~3 | 80 | -77 |
| Certification/Perf | ~4 | 70 | -66 |

## Ownership Domain Analysis

### Current Domains
- **Architecture**: `architectural_gates.rs`
- **Core**: Mixed in `engine_contracts.rs`
- **ECS**: Mixed in `core_ecs.rs` and `engine_contracts.rs`
- **Events**: Mixed in `engine_contracts.rs`
- **Runtime**: `runtime_phase_contracts.rs`
- **Physics**: `physics_*_contracts.rs`
- **World**: `spatial_dirty_contracts.rs`
- **Performance**: `certification_*.rs`
- **SDK**: `determinism_and_sdk.rs`
- **Tools**: `tools_runtime_purity.rs`

### Missing Domains ❌
- **Quality/Degradation Policy**: No dedicated tests
- **Ownership Map/Resource Authority**: Scattered
- **Determinism Policy**: Minimal coverage
- **Build/Runtime Manifest**: Minimal coverage
- **Event Aggregation/Spatial Bucketing**: Missing
- **Tracing/Debug Hooks**: Missing
- **Bootstrap Assembly**: Minimal
- **Jobs/Fences/Frame Graph**: Missing
- **Chunk Persistence**: Missing
- **Relink/Report Integrity**: Missing
- **World Fields/Material/Surface Truth**: Missing
- **Streaming/Load-Unload**: Missing
- **Damage Pipeline**: Partial
- **Destruction Topology**: Missing
- **Fire/Water/Destruction Separation**: Missing
- **Body/Ballistics Integration**: Missing
- **Render/Audio Boundary Purity**: Missing
- **Editor Safe Mode**: Minimal
- **Debug/Profiler/Dashboard Isolation**: Missing
- **Game/SDK/Headless Entrypoints**: Partial
- **Bootstrap Launcher**: Missing
- **SDK Workflow/Operator Commands**: Missing
- **Docs/Entrypoint Consistency**: Missing
- **Replay Checkpoints/Divergence**: Missing
- **Legacy Recovery**: Minimal
- **Meta-Tests**: Missing

## Critical Issues Summary

### 🚨 Immediate Actions Required
1. **Split Megasuites**: `engine_contracts.rs` and `core_ecs.rs`
2. **Assign Lanes**: 8+ files without lane assignment
3. **Missing Domains**: 20+ ownership domains without tests
4. **Test Type Balance**: Heavy on contracts, light on unit/integration

### 📊 Current vs Target Coverage
- **Current**: ~18 test files, mixed ownership
- **Target**: 520 tests across 12 domains
- **Progress**: ~3.5% complete

### 🎯 Next Steps
1. Create focused test suites from megasuites
2. Implement P0 domains (Architecture, Core, ECS, Events, Runtime, World)
3. Expand lane assignments for comprehensive coverage
4. Add missing ownership domains systematically
