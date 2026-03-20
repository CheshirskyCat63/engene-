# TEST REFACTOR RULES

## Core Principles

This document defines the mandatory rules for the test architecture refactoring. All new tests must follow these rules without exception.

## 1. File Organization Rules

### **1.1 No Megasuites**
- **FORBIDDEN**: Files > 500 lines with mixed ownership domains
- **REQUIRED**: Split by ownership domain, not by functionality
- **EXCEPTION**: Legacy quarantine files in `tests_legacy/` only

### **1.2 Ownership-Based Naming**
```
tests/{domain}_{purpose}_contracts.rs
tests/{domain}_{purpose}_integration.rs
tests/{domain}_{purpose}_scenario.rs
tests/{domain}_{purpose}_perf.rs
tests/{domain}_{purpose}_certification.rs
```

**Valid Examples:**
- `tests/core_command_and_access_contracts.rs`
- `tests/event_bus_contracts.rs`
- `tests/world_persistence_integration.rs`

**Invalid Examples:**
- `tests/engine_contracts.rs` (megsuite)
- `tests/mixed_domain_tests.rs` (unclear ownership)
- `tests/random_tests.rs` (no domain)

### **1.3 Domain Boundaries**
Each file must belong to exactly one ownership domain:

| Domain | Owner | Purpose |
|--------|-------|---------|
| architecture | Core Architecture Team | Ownership enforcement, dependency directions |
| core | Core Runtime Team | Runtime profiles, quality policies |
| ecs | ECS Team | Entity lifecycle, command buffer, access |
| events | Events Team | Event bus, sticky semantics, frame lifecycle |
| runtime | Runtime Team | Phase ordering, bootstrap, orchestration |
| world | World Team | Chunk persistence, streaming, spatial |
| physics | Physics Team | Physics contracts, damage, destruction |
| render_audio_tools | Tools Team | Boundary purity, editor safe mode |
| apps_sdk | Apps Team | Entrypoints, SDK workflows, operators |
| certification | QA Team | Determinism, replay, certification |
| legacy_recovery | Maintenance Team | Legacy suite wiring, migration |
| meta | Test Infrastructure Team | Lane consistency, naming hygiene |

## 2. Test Structure Rules

### **2.1 Mandatory Header**
Every test file must start with:

```rust
//! {Domain} {Purpose} Contracts
//! 
//! {Brief description of what this file tests}
//! Ownership: {Team Name}
//! Lane: {Lane Name}
//! Type: {Test Types}
//! Speed: {Speed Category}

#[cfg(test)]
mod {module_name} {
    // imports
    // tests
}
```

### **2.2 Test Naming Convention**
```rust
#[test]
fn {invariant_being_tested}_{specific_condition}_{expected_behavior}() {
    // test implementation
}
```

**Examples:**
- `spawn_new_assigns_unique_persistent_ids`
- `event_bus_sticky_survives_clear_frame`
- `runtime_profile_headless_has_zero_render_budget`

### **2.3 Single Invariant Per Test**
- **REQUIRED**: Each test protects exactly one invariant
- **FORBIDDEN**: Multiple unrelated assertions in one test
- **FORBIDDEN**: "Smoke test" without specific invariant

### **2.4 Test Documentation**
Each test must have a comment explaining:
```rust
/// Tests that [specific invariant] is maintained when [condition].
/// 
/// Invariant: [Clear statement of what must never happen]
/// Owner: [Domain team]
/// Lane: [Lane assignment]
/// Type: [Test type]
/// Speed: [Speed category]
```

## 3. Lane Assignment Rules

### **3.1 Lane Definitions**
| Lane | Purpose | Speed | Typical Test Types |
|------|---------|-------|-------------------|
| architecture | Ownership enforcement | Fast | Contract, Unit |
| core | Core policies | Fast | Contract, Unit |
| ecs | ECS contracts | Fast | Contract, Unit |
| events | Event contracts | Fast | Contract, Unit |
| runtime | Runtime orchestration | Medium | Contract, Integration |
| world | World persistence | Medium | Contract, Integration |
| physics | Physics boundaries | Medium | Contract, Integration |
| render_audio_tools | Tools purity | Fast | Contract, Unit |
| apps_sdk | App/Sdk workflows | Medium | Integration, Scenario |
| certification | QA certification | Heavy | Scenario, Certification |
| legacy_recovery | Legacy maintenance | Medium | Integration |
| meta | Test infrastructure | Fast | Meta |

### **3.2 Lane Assignment Rules**
- **REQUIRED**: Every test must have a lane assignment
- **FORBIDDEN**: Tests without lane assignment
- **REQUIRED**: Lane must match domain team's primary lane
- **EXCEPTION**: Cross-domain tests use the lane of the primary owner

### **3.3 Lane Performance Targets**
| Lane | Max Runtime | Max Memory | Parallel Safe |
|------|-------------|------------|---------------|
| architecture | 5s | 100MB | Yes |
| core | 5s | 100MB | Yes |
| ecs | 5s | 100MB | Yes |
| events | 5s | 100MB | Yes |
| runtime | 30s | 500MB | Yes |
| world | 30s | 500MB | Yes |
| physics | 30s | 500MB | Yes |
| render_audio_tools | 5s | 100MB | Yes |
| apps_sdk | 30s | 500MB | Yes |
| certification | 300s | 2GB | No |
| legacy_recovery | 30s | 500MB | Yes |
| meta | 5s | 100MB | Yes |

## 4. Test Type Rules

### **4.1 Test Type Classification**
| Type | Purpose | Scope | Speed |
|------|---------|-------|-------|
| Unit | Single function/component | Isolated | Fast |
| Contract | API contract enforcement | Module | Fast |
| Integration | Component interaction | System | Medium |
| Scenario | End-to-end workflow | Full | Heavy |
| Perf | Performance validation | System | Heavy |
| Certification | Release certification | Full | Heavy |
| Meta | Test system validation | Meta | Fast |

### **4.2 Test Type Assignment Rules**
- **Unit**: Single function, no external dependencies
- **Contract**: API boundaries, error conditions, edge cases
- **Integration**: Multiple components working together
- **Scenario**: User workflows, system-level behavior
- **Perf**: Performance benchmarks, memory usage
- **Certification**: Release readiness, compliance
- **Meta**: Test infrastructure validation

### **4.3 Test Type Implementation Rules**

#### Unit Tests
```rust
#[test]
fn specific_function_under_specific_condition_returns_expected_result() {
    // Arrange
    let input = create_test_input();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected_result);
}
```

#### Contract Tests
```rust
#[test]
fn api_contract_precondition_violation_returns_error() {
    // Test contract violation handling
    let result = api_call(invalid_input);
    assert!(result.is_err());
}
```

#### Integration Tests
```rust
#[test]
fn component_a_and_component_b_integration_produces_expected_output() {
    // Test real component interaction
    let system = setup_integration_test();
    let result = system.process(test_input);
    assert_eq!(result, expected_output);
}
```

## 5. Speed Classification Rules

### **5.1 Speed Categories**
| Speed | Max Runtime | Max Memory | Test Types |
|-------|-------------|------------|------------|
| Fast | < 5s | < 100MB | Unit, Contract, Meta |
| Medium | < 30s | < 500MB | Integration |
| Heavy | < 300s | < 2GB | Scenario, Perf, Certification |

### **5.2 Speed Assignment Rules**
- **Fast**: Pure computation, no I/O, no external dependencies
- **Medium**: File I/O, network calls, database access
- **Heavy**: Large datasets, long-running processes, full system

### **5.3 Speed Enforcement**
- **REQUIRED**: Test must complete within speed category limit
- **FORBIDDEN**: Slow tests in fast lanes
- **REQUIRED**: Memory usage within category limits

## 6. Invariant Protection Rules

### **6.1 Invariant Definition**
Every test must protect a specific invariant:

```rust
/// Invariant: Entity IDs are unique and persistent across world cycles
/// Violation: Duplicate IDs or ID reuse after despawn
```

### **6.2 Invariant Categories**
| Category | Examples | Protection Method |
|----------|----------|------------------|
| Identity | Unique IDs, persistent state | Generation tracking, validation |
| Safety | Memory safety, thread safety | Type system, runtime checks |
| Performance | Budget limits, memory bounds | Monitoring, assertions |
| Consistency | State consistency, data integrity | Validation, checksums |
| Isolation | Component isolation, lane separation | Access control, boundaries |

### **6.3 Invariant Testing Rules**
- **REQUIRED**: Each test must state the invariant it protects
- **REQUIRED**: Test must verify the invariant holds
- **FORBIDDEN**: Tests without clear invariant protection
- **REQUIRED**: Invariant violations must be detectable

## 7. Ownership Rules

### **7.1 Domain Ownership**
Each test file has a single domain owner:

```rust
//! Ownership: Core Architecture Team
```

### **7.2 Ownership Responsibilities**
- **Domain Team**: Writes and maintains tests in their domain
- **Cross-Domain Tests**: Must have primary owner and secondary reviewers
- **Ownership Changes**: Must be documented and approved

### **7.3 Ownership Enforcement**
- **REQUIRED**: Every test file must have ownership declaration
- **FORBIDDEN**: Tests without clear ownership
- **REQUIRED**: Ownership must match domain boundaries

## 8. Quality Rules

### **8.1 Code Quality**
- **REQUIRED**: Tests must compile without warnings
- **REQUIRED**: Tests must follow Rust style guidelines
- **FORBIDDEN**: `unwrap()` without error handling
- **REQUIRED**: Proper error handling and assertions

### **8.2 Test Quality**
- **REQUIRED**: Tests must be deterministic
- **REQUIRED**: Tests must be isolated (no shared state)
- **FORBIDDEN**: Tests that depend on execution order
- **REQUIRED**: Tests must clean up after themselves

### **8.3 Documentation Quality**
- **REQUIRED**: Clear test names describing what is tested
- **REQUIRED**: Comments explaining complex test logic
- **REQUIRED**: Invariant documentation
- **FORBIDDEN**: Tests without explanatory comments

## 9. CI Integration Rules

### **9.1 Lane Commands**
Each lane must have executable commands:

```bash
# justfile
architecture:
    cargo test --test architectural_gates --test core_command_and_access_contracts

core:
    cargo test --test runtime_profile_and_quality_contracts

ecs:
    cargo test --test ecs_lifecycle_contracts --test ecs_command_contracts
```

### **9.2 CI Pipeline Rules**
- **REQUIRED**: Each lane runs in separate CI job
- **REQUIRED**: Fast lanes run in parallel
- **REQUIRED**: Heavy lanes run sequentially
- **REQUIRED**: All lanes must pass for merge

### **9.3 Performance Rules**
- **REQUIRED**: Lane execution time within targets
- **REQUIRED**: Memory usage within limits
- **FORBIDDEN**: Lanes that exceed performance budgets

## 10. Migration Rules

### **10.1 Legacy Test Handling**
- **KEEP**: Well-structured tests with clear ownership
- **SPLIT**: Megasuites with mixed domains
- **MOVE**: Tests in wrong domain or lane
- **QUARANTINE**: Tests that cannot be fixed immediately

### **10.2 Migration Process**
1. **Inventory**: Classify existing tests
2. **Split**: Break down megasuites by domain
3. **Move**: Reassign to correct domains
4. **Refactor**: Apply all rules to new tests
5. **Validate**: Ensure all tests pass

### **10.3 Validation Rules**
- **REQUIRED**: All new tests follow these rules
- **REQUIRED**: All migrated tests follow these rules
- **FORBIDDEN**: Exceptions to these rules
- **REQUIRED**: Regular compliance audits

## 11. Enforcement Rules

### **11.1 Automated Enforcement**
- **Lint Rules**: Enforce naming conventions
- **CI Checks**: Validate lane assignments
- **Tests**: Verify rule compliance
- **Scripts**: Automated rule validation

### **11.2 Manual Enforcement**
- **Code Review**: Ensure rule compliance
- **Architecture Review**: Validate domain boundaries
- **Test Review**: Verify test quality
- **Documentation Review**: Check completeness

### **11.3 Violation Handling**
- **Block**: Prevent merge of rule violations
- **Fix**: Require immediate correction
- **Document**: Track violations and fixes
- **Learn**: Update rules to prevent future violations

## 12. Success Metrics

### **12.1 Quantitative Metrics**
- **520+ total tests** across all domains
- **Zero megasuites** > 500 lines
- **100% lane coverage** for all tests
- **100% ownership assignment** for all tests
- **100% invariant documentation** for all tests

### **12.2 Qualitative Metrics**
- **Clear domain boundaries** with no overlap
- **Fast feedback** through lane commands
- **High test quality** with good coverage
- **Maintainable architecture** with clear ownership
- **Effective invariant protection** with comprehensive testing

---

*These rules are mandatory for the test architecture refactoring. No exceptions will be permitted.*
