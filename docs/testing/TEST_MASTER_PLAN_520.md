# TEST MASTER PLAN 520

## Executive Summary

This document defines the comprehensive test architecture refactoring from the current eclectic structure to a systematic 520+ test suite organized by ownership domains and lanes.

## Current State Problems

1. **Megasuites**: `engine_contracts.rs` (1,205 lines) and `core_ecs.rs` (50,531 lines) mix multiple ownership domains
2. **Lane Coverage Gap**: Only 18 test targets vs 520+ needed
3. **Missing Ownership**: No clear domain ownership boundaries
4. **Test Type Confusion**: Unit, integration, contract tests mixed without purpose

## Target Architecture: 12 Ownership Domains

### 1. Architecture & Ownership (40+ tests)
**Owner**: Core Architecture Team
**Purpose**: Enforce crate ownership rules, dependency directions, bootstrap purity
**Lane**: architecture
**Test Types**: Contract + Unit

### 2. Core Policies (40+ tests)  
**Owner**: Core Runtime Team
**Purpose**: Runtime profiles, quality policies, performance contracts
**Lane**: core
**Test Types**: Contract + Unit

### 3. ECS, Commands, Access (60+ tests)
**Owner**: ECS Team
**Purpose**: Entity lifecycle, command buffer operations, access descriptors
**Lane**: ecs
**Test Types**: Contract + Unit

### 4. Events (50+ tests)
**Owner**: Events Team  
**Purpose**: Event bus contracts, sticky semantics, frame lifecycle
**Lane**: events
**Test Types**: Contract + Unit

### 5. Runtime Phases & Orchestration (50+ tests)
**Owner**: Runtime Team
**Purpose**: Phase ordering, bootstrap assembly, simulation orchestration
**Lane**: runtime
**Test Types**: Contract + Integration

### 6. World, Spatial, Persistence (60+ tests)
**Owner**: World Team
**Purpose**: Chunk persistence, streaming contracts, spatial updates
**Lane**: world
**Test Types**: Contract + Integration

### 7. Physics Boundary (40+ tests)
**Owner**: Physics Team
**Purpose**: Physics contracts, damage pipeline, destruction topology
**Lane**: physics
**Test Types**: Contract + Integration

### 8. Render, Audio, Tools Purity (35+ tests)
**Owner**: Tools Team
**Purpose**: Boundary purity, editor safe mode, debug isolation
**Lane**: render_audio_tools
**Test Types**: Contract + Unit

### 9. Apps, SDK, Operator Truth (25+ tests)
**Owner**: Apps Team
**Purpose**: Entrypoint contracts, SDK workflows, operator commands
**Lane**: apps_sdk
**Test Types**: Integration + Scenario

### 10. Determinism, Replay, Certification (20+ tests)
**Owner**: QA Team
**Purpose**: Determinism contracts, replay checkpoints, certification gates
**Lane**: certification
**Test Types**: Scenario + Certification

### 11. Legacy Recovery (20+ tests)
**Owner**: Maintenance Team
**Purpose**: Legacy suite wiring, migration guards, known failing markers
**Lane**: legacy_recovery
**Test Types**: Integration

### 12. Meta-Tests (20+ tests)
**Owner**: Test Infrastructure Team
**Purpose**: Lane consistency, naming hygiene, test system validation
**Lane**: meta
**Test Types**: Meta

## Implementation Strategy

### Phase 1: Infrastructure (Week 1)
1. Create all documentation files
2. Split existing megasuites
3. Establish lane commands
4. Create test inventory matrix

### Phase 2: P0 Domains (Weeks 2-3)
1. Architecture & Ownership (40 tests)
2. Core Policies (40 tests)
3. ECS, Commands, Access (60 tests)
4. Events (50 tests)
5. Runtime Phases (50 tests)
6. World, Spatial, Persistence (60 tests)

### Phase 3: Extended Domains (Weeks 4-5)
1. Physics Boundary (40 tests)
2. Render, Audio, Tools (35 tests)
3. Apps, SDK, Operator (25 tests)
4. Determinism, Certification (20 tests)
5. Legacy Recovery (20 tests)
6. Meta-Tests (20 tests)

### Phase 4: Validation (Week 6)
1. Complete test inventory
2. Lane command validation
3. Performance validation
4. Documentation finalization

## Success Metrics

- **Zero megasuites** > 500 lines
- **520+ tests** across 12 domains
- **Clear ownership** for every test
- **Lane assignment** for every test
- **Invariant protection** for every test
- **Fast feedback** through lane commands

## Risk Mitigation

1. **Legacy Test Quarantine**: Move problematic tests to `tests_legacy/`
2. **Gradual Migration**: Keep existing tests working during transition
3. **Lane Validation**: Ensure each lane runs in reasonable time
4. **Ownership Clarity**: Prevent future megasuite formation

## Acceptance Criteria

See individual sections for specific acceptance criteria per domain.

---

*This document is living and will be updated as the refactoring progresses.*
