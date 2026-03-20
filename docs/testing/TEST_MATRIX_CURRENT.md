# TEST MATRIX CURRENT

## Current Test Inventory

This document provides a complete inventory of all existing tests with their classification and disposition.

| File | Lines | Owner Domain | Lane | Test Type | Speed | Risk Level | Status | Notes |
|------|-------|-------------|------|-----------|------|-----------|---------|-------|
| `engine_contracts.rs` | 1,205 | **MEGA** | Smoke | Mixed | Mixed | High | **SPLIT** | Contains command buffer, event bus, runtime manifests, parallel validation, identity, authority, persistence, editor safe mode |
| `core_ecs.rs` | 50,531 | **MEGA** | None | Mixed | Heavy | High | **SPLIT** | Entity lifecycle, command buffer, component ops, access descriptors, queries |
| `architectural_gates.rs` | 281 | Architecture | Smoke | Contract | Fast | Medium | **KEEP** | Ownership enforcement, file-scan contracts |
| `runtime_phase_contracts.rs` | 8,297 | Runtime | Contracts | Contract | Medium | Medium | **KEEP** | Runtime orchestration, phase ordering |
| `physics_core_boundary_contracts.rs` | 12,304 | Physics | Contracts | Contract | Medium | Medium | **KEEP** | Physics boundary contracts |
| `physics_bootstrap_contracts.rs` | 5,480 | Physics | Contracts | Contract | Medium | Medium | **KEEP** | Physics bootstrap contracts |
| `spatial_dirty_contracts.rs` | 3,623 | World | Contracts | Contract | Fast | Medium | **KEEP** | Spatial update contracts |
| `certification_boundary_overhead.rs` | 1,246 | QA | Certification | Certification | Heavy | High | **KEEP** | Performance boundary limits |
| `certification_kernel_throughput.rs` | 1,665 | QA | Certification | Certification | Heavy | High | **KEEP** | Kernel performance contracts |
| `certification_tick_budget.rs` | 1,208 | QA | Certification | Certification | Heavy | High | **KEEP** | Tick budget enforcement |
| `certification_perf_snapshot.rs` | 2,213 | QA | Certification | Certification | Heavy | High | **KEEP** | Performance snapshots |
| `determinism_and_sdk.rs` | 14,002 | SDK | None | Integration | Medium | Medium | **SPLIT** | SDK contracts, determinism tests |
| `tools_runtime_purity.rs` | 2,774 | Tools | None | Contract | Fast | Medium | **MOVE** | Tools isolation, editor purity |
| `entrypoint_and_operator_truth.rs` | 8,441 | Apps | Smoke | Contract | Fast | Medium | **KEEP** | Entry point contracts |
| `ci_surface_contracts.rs` | 7,517 | CI | Smoke | Contract | Fast | Medium | **KEEP** | CI surface contracts |
| `production_candidate.rs` | 5,792 | Release | Smoke | Integration | Medium | High | **KEEP** | Release readiness checks |
| `ai_social_economy.rs` | 1,855 | Game | None | Integration | Medium | Low | **MOVE** | AI social/economy tests |
| `body_pipeline.rs` | 1,716 | Physics | None | Integration | Medium | Medium | **MOVE** | Body pipeline tests |
| `e2e_regression.rs` | 33,001 | Game | None | Scenario | Heavy | High | **SPLIT** | End-to-end regression tests |
| `economy_cycle.rs` | 2,752 | Game | None | Integration | Medium | Low | **MOVE** | Economy cycle tests |
| `engine_infrastructure.rs` | 18,344 | Core | None | Integration | Medium | Medium | **SPLIT** | Infrastructure tests |
| `game_player_layer.rs` | 33,733 | Game | None | Integration | Heavy | High | **SPLIT** | Game player layer tests |
| `graphics_renderer.rs` | 41,993 | Render | None | Integration | Heavy | Medium | **SPLIT** | Graphics renderer tests |
| `multithreading.rs` | 1,911 | Core | None | Contract | Medium | Medium | **MOVE** | Multithreading contracts |
| `performance.rs` | 1,401 | QA | None | Perf | Heavy | Medium | **MOVE** | Performance tests |
| `perf_lowspec_mt.rs` | 27,640 | QA | Perf | Perf | Heavy | High | **SPLIT** | Low-spec multithreading perf |
| `physics_bootstrap_contracts.rs` | 5,480 | Physics | Contracts | Contract | Medium | Medium | **KEEP** | Physics bootstrap contracts |
| `quest_and_faction.rs` | 4,236 | Game | None | Integration | Medium | Low | **MOVE** | Quest and faction tests |
| `render_pipeline.rs` | 2,164 | Render | None | Integration | Medium | Medium | **MOVE** | Render pipeline tests |
| `runtime_role_separation.rs` | 2,067 | Runtime | None | Contract | Fast | Medium | **MOVE** | Runtime role separation |
| `save_load_torture.rs` | 17,461 | World | None | Scenario | Heavy | High | **SPLIT** | Save/load torture tests |
| `sdk_editor_gui.rs` | 39,278 | SDK | None | Integration | Heavy | Medium | **SPLIT** | SDK editor GUI tests |
| `spatial_dirty_contracts.rs` | 3,623 | World | Contracts | Contract | Fast | Medium | **KEEP** | Spatial dirty contracts |
| `vertical_slice.rs` | 8,399 | Game | None | Integration | Medium | Medium | **MOVE** | Vertical slice tests |
| `wiring_boundary_contracts.rs` | 6,758 | Runtime | None | Contract | Medium | Medium | **MOVE** | Wiring boundary contracts |

## Disposition Summary

### **KEEP** (9 files, 42,777 lines)
- Well-structured, single-domain tests
- Clear ownership and purpose
- Appropriate lane assignment
- **Action**: No changes needed

### **SPLIT** (8 files, 233,847 lines)
- Megasuites with mixed ownership
- Multiple domains in single file
- Need to be broken down by domain
- **Action**: Create focused suites

### **MOVE** (15 files, 88,582 lines)
- Good tests but wrong lane/location
- Need domain reassignment
- **Action**: Move to appropriate domain

### **DELETE** (0 files)
- No tests marked for deletion
- All tests have value

## Risk Assessment

### **High Risk** (5 files)
- `engine_contracts.rs` - Critical contracts mixed
- `core_ecs.rs` - Core ECS functionality at risk
- `e2e_regression.rs` - End-to-end coverage
- `game_player_layer.rs` - Game layer coverage
- `perf_lowspec_mt.rs` - Performance contracts

### **Medium Risk** (15 files)
- Most integration and contract tests
- Domain boundary tests
- Performance-related tests

### **Low Risk** (12 files)
- Unit and contract tests
- Well-isolated functionality
- Clear ownership

## Lane Assignment Gaps

### **Missing Lane Assignments**
- `determinism_and_sdk.rs` - Needs lane
- `tools_runtime_purity.rs` - Needs lane  
- `ai_social_economy.rs` - Needs lane
- `body_pipeline.rs` - Needs lane
- `e2e_regression.rs` - Needs lane
- `economy_cycle.rs` - Needs lane
- `engine_infrastructure.rs` - Needs lane
- `game_player_layer.rs` - Needs lane
- `graphics_renderer.rs` - Needs lane
- `multithreading.rs` - Needs lane
- `performance.rs` - Needs lane
- `perf_lowspec_mt.rs` - Needs lane
- `quest_and_faction.rs` - Needs lane
- `render_pipeline.rs` - Needs lane
- `runtime_role_separation.rs` - Needs lane
- `save_load_torture.rs` - Needs lane
- `sdk_editor_gui.rs` - Needs lane
- `vertical_slice.rs` - Needs lane
- `wiring_boundary_contracts.rs` - Needs lane

## Next Steps

1. **Phase 1**: Split megasuites (8 files)
2. **Phase 2**: Assign lanes to unassigned tests (19 files)
3. **Phase 3**: Move tests to appropriate domains (15 files)
4. **Phase 4**: Validate all test assignments

---

*This matrix will be updated as the refactoring progresses.*
