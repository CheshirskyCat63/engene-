# TEST LANE OWNERSHIP MAP

## Lane Ownership Matrix

This document defines the complete mapping between lanes, ownership domains, and teams.

## Lane Structure

### **Primary Lanes (12)**

| Lane | Owner Team | Purpose | Test Types | Speed | Command |
|------|------------|---------|-------------|-------|---------|
| **architecture** | Core Architecture Team | Ownership enforcement, dependency directions | Contract, Unit | Fast | `just architecture` |
| **core** | Core Runtime Team | Runtime profiles, quality policies | Contract, Unit | Fast | `just core` |
| **ecs** | ECS Team | Entity lifecycle, command buffer, access | Contract, Unit | Fast | `just ecs` |
| **events** | Events Team | Event bus, sticky semantics, frame lifecycle | Contract, Unit | Fast | `just events` |
| **runtime** | Runtime Team | Phase ordering, bootstrap, orchestration | Contract, Integration | Medium | `just runtime` |
| **world** | World Team | Chunk persistence, streaming, spatial | Contract, Integration | Medium | `just world` |
| **physics** | Physics Team | Physics contracts, damage, destruction | Contract, Integration | Medium | `just physics` |
| **render_audio_tools** | Tools Team | Boundary purity, editor safe mode | Contract, Unit | Fast | `just tools` |
| **apps_sdk** | Apps Team | Entrypoints, SDK workflows, operators | Integration, Scenario | Medium | `just apps` |
| **certification** | QA Team | Determinism, replay, certification | Scenario, Certification | Heavy | `just certification` |
| **legacy_recovery** | Maintenance Team | Legacy suite wiring, migration | Integration | Medium | `just legacy` |
| **meta** | Test Infrastructure Team | Lane consistency, naming hygiene | Meta | Fast | `just meta` |

### **Legacy Lanes (3)**

| Lane | Owner Team | Purpose | Status | Command |
|------|------------|---------|--------|---------|
| **smoke** | CI Team | Quick health checks | Keep | `just smoke` |
| **contracts** | Architecture Team | Critical contracts | Keep | `just contracts` |
| **perf** | QA Team | Performance gates | Keep | `just perf` |

## Domain-to-Lane Mapping

### **Architecture & Ownership Domain**
- **Primary Lane**: `architecture`
- **Owner**: Core Architecture Team
- **Test Files**:
  - `architectural_gates.rs` (existing)
  - `root_thin_shell_invariants.rs` (new)
  - `forbidden_dependency_directions.rs` (new)
  - `bootstrap_contract.rs` (new)
  - `compat_hub_purity.rs` (new)

### **Core Policies Domain**
- **Primary Lane**: `core`
- **Owner**: Core Runtime Team
- **Test Files**:
  - `runtime_profile_and_quality_contracts.rs` (new)
  - `quality_governor_contracts.rs` (new)
  - `performance_contracts.rs` (new)
  - `budget_registry_contracts.rs` (new)

### **ECS, Commands, Access Domain**
- **Primary Lane**: `ecs`
- **Owner**: ECS Team
- **Test Files**:
  - `core_command_and_access_contracts.rs` (new)
  - `entity_lifecycle_contracts.rs` (new)
  - `command_buffer_contracts.rs` (new)
  - `access_descriptor_contracts.rs` (new)
  - `query_contracts.rs` (new)

### **Events Domain**
- **Primary Lane**: `events`
- **Owner**: Events Team
- **Test Files**:
  - `event_bus_contracts.rs` (new)
  - `event_aggregation_contracts.rs` (new)
  - `event_tracing_contracts.rs` (new)
  - `sticky_events_contracts.rs` (new)

### **Runtime Phases & Orchestration Domain**
- **Primary Lane**: `runtime`
- **Owner**: Runtime Team
- **Test Files**:
  - `runtime_phase_contracts.rs` (existing)
  - `bootstrap_assembly_contracts.rs` (new)
  - `simulation_orchestration_contracts.rs` (new)
  - `jobs_and_fences_contracts.rs` (new)

### **World, Spatial, Persistence Domain**
- **Primary Lane**: `world`
- **Owner**: World Team
- **Test Files**:
  - `world_persistence_contracts.rs` (new)
  - `spatial_dirty_contracts.rs` (existing)
  - `world_fields_contracts.rs` (new)
  - `streaming_contracts.rs` (new)

### **Physics Boundary Domain**
- **Primary Lane**: `physics`
- **Owner**: Physics Team
- **Test Files**:
  - `physics_core_boundary_contracts.rs` (existing)
  - `physics_bootstrap_contracts.rs` (existing)
  - `damage_pipeline_contracts.rs` (new)
  - `destruction_topology_contracts.rs` (new)

### **Render, Audio, Tools Purity Domain**
- **Primary Lane**: `render_audio_tools`
- **Owner**: Tools Team
- **Test Files**:
  - `editor_safe_mode_contracts.rs` (new)
  - `render_boundary_contracts.rs` (new)
  - `audio_boundary_contracts.rs` (new)
  - `debug_surface_contracts.rs` (new)
  - `profiler_contracts.rs` (new)

### **Apps, SDK, Operator Truth Domain**
- **Primary Lane**: `apps_sdk`
- **Owner**: Apps Team
- **Test Files**:
  - `entrypoint_and_operator_truth.rs` (existing)
  - `sdk_workflow_contracts.rs` (new)
  - `bootstrap_launcher_contracts.rs` (new)
  - `docs_entrypoint_contracts.rs` (new)

### **Determinism, Replay, Certification Domain**
- **Primary Lane**: `certification`
- **Owner**: QA Team
- **Test Files**:
  - `determinism_contracts.rs` (new)
  - `replay_checkpoint_contracts.rs` (new)
  - `certification_boundary_overhead.rs` (existing)
  - `certification_kernel_throughput.rs` (existing)
  - `certification_tick_budget.rs` (existing)
  - `certification_perf_snapshot.rs` (existing)

### **Legacy Recovery Domain**
- **Primary Lane**: `legacy_recovery`
- **Owner**: Maintenance Team
- **Test Files**:
  - `legacy_suite_wiring.rs` (new)
  - `migration_compatibility_guards.rs` (new)
  - `known_failing_scenarios.rs` (new)

### **Meta-Tests Domain**
- **Primary Lane**: `meta`
- **Owner**: Test Infrastructure Team
- **Test Files**:
  - `lane_membership_consistency.rs` (new)
  - `file_naming_suite_hygiene.rs` (new)
  - `test_system_validation.rs` (new)

## Lane Command Definitions

### **Architecture Lane**
```bash
# justfile
architecture:
    cargo test --test architectural_gates \
               --test root_thin_shell_invariants \
               --test forbidden_dependency_directions \
               --test bootstrap_contract \
               --test compat_hub_purity

# Shell script
scripts/test/architecture.sh
#!/bin/bash
cargo test --test architectural_gates \
          --test root_thin_shell_invariants \
          --test forbidden_dependency_directions \
          --test bootstrap_contract \
          --test compat_hub_purity
```

### **Core Lane**
```bash
# justfile
core:
    cargo test --test runtime_profile_and_quality_contracts \
               --test quality_governor_contracts \
               --test performance_contracts \
               --test budget_registry_contracts

# Shell script
scripts/test/core.sh
#!/bin/bash
cargo test --test runtime_profile_and_quality_contracts \
          --test quality_governor_contracts \
          --test performance_contracts \
          --test budget_registry_contracts
```

### **ECS Lane**
```bash
# justfile
ecs:
    cargo test --test core_command_and_access_contracts \
               --test entity_lifecycle_contracts \
               --test command_buffer_contracts \
               --test access_descriptor_contracts \
               --test query_contracts

# Shell script
scripts/test/ecs.sh
#!/bin/bash
cargo test --test core_command_and_access_contracts \
          --test entity_lifecycle_contracts \
          --test command_buffer_contracts \
          --test access_descriptor_contracts \
          --test query_contracts
```

### **Events Lane**
```bash
# justfile
events:
    cargo test --test event_bus_contracts \
               --test event_aggregation_contracts \
               --test event_tracing_contracts \
               --test sticky_events_contracts

# Shell script
scripts/test/events.sh
#!/bin/bash
cargo test --test event_bus_contracts \
          --test event_aggregation_contracts \
          --test event_tracing_contracts \
          --test sticky_events_contracts
```

### **Runtime Lane**
```bash
# justfile
runtime:
    cargo test --test runtime_phase_contracts \
               --test bootstrap_assembly_contracts \
               --test simulation_orchestration_contracts \
               --test jobs_and_fences_contracts

# Shell script
scripts/test/runtime.sh
#!/bin/bash
cargo test --test runtime_phase_contracts \
          --test bootstrap_assembly_contracts \
          --test simulation_orchestration_contracts \
          --test jobs_and_fences_contracts
```

### **World Lane**
```bash
# justfile
world:
    cargo test --test world_persistence_contracts \
               --test spatial_dirty_contracts \
               --test world_fields_contracts \
               --test streaming_contracts

# Shell script
scripts/test/world.sh
#!/bin/bash
cargo test --test world_persistence_contracts \
          --test spatial_dirty_contracts \
          --test world_fields_contracts \
          --test streaming_contracts
```

### **Physics Lane**
```bash
# justfile
physics:
    cargo test --test physics_core_boundary_contracts \
               --test physics_bootstrap_contracts \
               --test damage_pipeline_contracts \
               --test destruction_topology_contracts

# Shell script
scripts/test/physics.sh
#!/bin/bash
cargo test --test physics_core_boundary_contracts \
          --test physics_bootstrap_contracts \
          --test damage_pipeline_contracts \
          --test destruction_topology_contracts
```

### **Tools Lane**
```bash
# justfile
tools:
    cargo test --test editor_safe_mode_contracts \
               --test render_boundary_contracts \
               --test audio_boundary_contracts \
               --test debug_surface_contracts \
               --test profiler_contracts

# Shell script
scripts/test/tools.sh
#!/bin/bash
cargo test --test editor_safe_mode_contracts \
          --test render_boundary_contracts \
          --test audio_boundary_contracts \
          --test debug_surface_contracts \
          --test profiler_contracts
```

### **Apps SDK Lane**
```bash
# justfile
apps:
    cargo test --test entrypoint_and_operator_truth \
               --test sdk_workflow_contracts \
               --test bootstrap_launcher_contracts \
               --test docs_entrypoint_contracts

# Shell script
scripts/test/apps.sh
#!/bin/bash
cargo test --test entrypoint_and_operator_truth \
          --test sdk_workflow_contracts \
          --test bootstrap_launcher_contracts \
          --test docs_entrypoint_contracts
```

### **Certification Lane**
```bash
# justfile
certification:
    cargo test --test determinism_contracts \
               --test replay_checkpoint_contracts \
               --test certification_boundary_overhead \
               --test certification_kernel_throughput \
               --test certification_tick_budget \
               --test certification_perf_snapshot

# Shell script
scripts/test/certification.sh
#!/bin/bash
cargo test --test determinism_contracts \
          --test replay_checkpoint_contracts \
          --test certification_boundary_overhead \
          --test certification_kernel_throughput \
          --test certification_tick_budget \
          --test certification_perf_snapshot
```

### **Legacy Recovery Lane**
```bash
# justfile
legacy:
    cargo test --test legacy_suite_wiring \
               --test migration_compatibility_guards \
               --test known_failing_scenarios

# Shell script
scripts/test/legacy.sh
#!/bin/bash
cargo test --test legacy_suite_wiring \
          --test migration_compatibility_guards \
          --test known_failing_scenarios
```

### **Meta Lane**
```bash
# justfile
meta:
    cargo test --test lane_membership_consistency \
               --test file_naming_suite_hygiene \
               --test test_system_validation

# Shell script
scripts/test/meta.sh
#!/bin/bash
cargo test --test lane_membership_consistency \
          --test file_naming_suite_hygiene \
          --test test_system_validation
```

## Lane Performance Targets

### **Fast Lanes** (< 5s, < 100MB)
- **architecture**: Ownership enforcement tests
- **core**: Runtime profile and quality tests
- **ecs**: Entity and command tests
- **events**: Event bus and aggregation tests
- **render_audio_tools**: Tools purity tests
- **meta**: Test infrastructure tests

### **Medium Lanes** (< 30s, < 500MB)
- **runtime**: Phase ordering and orchestration
- **world**: Persistence and streaming tests
- **physics**: Physics boundary tests
- **apps_sdk**: SDK and workflow tests
- **legacy_recovery**: Legacy maintenance tests

### **Heavy Lanes** (< 300s, < 2GB)
- **certification**: Determinism and replay tests

## Lane Execution Strategy

### **Parallel Execution**
Fast lanes can run in parallel:
```bash
# Parallel fast lanes
just architecture & just core & just ecs & just events & just tools & just meta
```

### **Sequential Execution**
Medium lanes should run sequentially:
```bash
# Sequential medium lanes
just runtime && just world && just physics && just apps && just legacy
```

### **Isolated Execution**
Heavy lane runs in isolation:
```bash
# Isolated heavy lane
just certification
```

## Lane Ownership Responsibilities

### **Primary Owner Responsibilities**
- Write and maintain tests in their lane
- Ensure lane performance targets are met
- Review cross-lane test dependencies
- Approve lane membership changes

### **Secondary Owner Responsibilities**
- Review tests that affect their domain
- Provide domain expertise for cross-lane tests
- Ensure domain invariants are protected
- Participate in lane architecture decisions

### **Cross-Lane Coordination**
- **Cross-Domain Tests**: Must have primary lane owner and secondary domain owner review
- **Shared Dependencies**: Must be documented and approved by all affected lane owners
- **Performance Impact**: Heavy tests must not affect fast lane performance
- **Resource Usage**: Memory and CPU usage must stay within lane targets

## Lane Governance

### **Lane Addition Rules**
- **New lanes** require architecture team approval
- **Lane splits** must maintain clear ownership
- **Lane mergers** must preserve test coverage
- **Lane deprecation** must have migration plan

### **Lane Modification Rules**
- **Test file moves** require both lane owners approval
- **Ownership changes** require team lead approval
- **Performance target changes** require architecture review
- **Command changes** must update all execution methods

### **Lane Compliance Rules**
- **All tests** must belong to a lane
- **Lane assignments** must be documented
- **Performance targets** must be enforced
- **Ownership declarations** must be accurate

## Lane Monitoring

### **Performance Monitoring**
- **Execution time** tracked per lane
- **Memory usage** monitored per lane
- **Parallel efficiency** measured
- **Resource contention** detected

### **Quality Monitoring**
- **Test coverage** per lane
- **Failure rates** per lane
- **Flaky test** detection
- **Maintenance burden** tracking

### **Usage Monitoring**
- **Lane execution frequency**
- **Developer adoption** metrics
- **CI pipeline integration** status
- **Documentation completeness**

---

*This lane ownership map is the authoritative source for test organization and execution.*
