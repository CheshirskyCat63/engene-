# ENGINE_RUNTIME AUDIT - REAL SPINE vs FAKE THEATER

## CURRENT DEPENDENCIES ANALYSIS

### ❌ SUSPICIOUS DEPENDENCIES
```toml
[dependencies]
engine_core = { path = "../engine_core" }
engine_ecs = { path = "../engine_ecs" }
engine_world = { path = "../engine_world" }
rapier3d = "0.17"        # ⚠️ PHYSICS IN MINIMAL RUNTIME?
```

### 🚨 CRITICAL QUESTION: WHY RAPIER3D?

**For "minimal deterministic headless runtime":**
- ❌ Physics is NOT required for core runtime proof
- ❌ Physics adds heavy dependency (rapier3d = 3D physics engine)
- ❌ Physics violates "minimal" principle
- ❌ Physics suggests architecture compromise

## CURRENT IMPLEMENTATION ANALYSIS

### ✅ REAL RUNTIME COMPONENTS
```rust
// simulation_core/systems/scheduler.rs
pub struct Scheduler {
    accumulated: f32,
    interval: f32,
}
// ✅ Real fixed-interval scheduling

// simulation_core/systems/engine_system.rs  
pub trait EngineSystem {
    fn name(&self) -> &str;
    fn tick(&mut self, _ctx: &mut SystemTickContext) {}
    fn fixed_tick(&mut self, _ctx: &mut FixedTickContext) {}
}
// ✅ Real system trait

// lib.rs
pub struct EngineRuntime {
    systems: Vec<Box<dyn EngineSystem>>,
}
// ✅ Real runtime container
```

### ❓ MINIMAL RUNTIME TEST VALIDATION
```rust
// minimal_runtime_test.rs
fn engine_minimal_runtime_path() {
    let config = GameConfig::default();     // ✅ Config layer
    let mut runtime = EngineRuntime::new(&config);  // ✅ Runtime assembly
    let mut scheduler = Scheduler::new(1.0 / 60.0);   // ✅ Scheduler
    let mut world = WorldGrid::generate();   // ❌ STUB WORLD
    // ... 1000 ticks
}
```

## VERDICT: MIXED REAL + FAKE

### ✅ REAL COMPONENTS (70%)
- Fixed-interval scheduler implementation
- EngineSystem trait and execution
- Runtime assembly pattern
- Deterministic tick progression

### ❌ FAKE/SUSPICIOUS COMPONENTS (30%)
- **rapier3d dependency** - Physics in minimal runtime?
- **stub world integration** - Uses fake WorldGrid
- **no system ordering contracts** - Missing execution guarantees
- **no scheduler invariants** - Missing order stability tests

## CRITICAL ARCHITECTURAL ISSUE

**engine_runtime currently pulls physics (rapier3d) into minimal engine path**

This violates the "minimal runtime" principle and suggests:
1. Architecture compromise for physics integration
2. Runtime not truly minimal
3. Physics may be driving runtime design

## RECOMMENDATION: PHYSICS DECOUPLING

### Step 1: Remove rapier3d from minimal runtime
```toml
[dependencies]
engine_core = { path = "../engine_core" }
engine_ecs = { path = "../engine_ecs" }
engine_world = { path = "../engine_world" }
# rapier3d = "0.17"  # ❌ REMOVE FROM MINIMAL PATH
```

### Step 2: Test runtime without physics
- Verify 1000-tick test works without physics
- Verify deterministic replay works without physics
- Prove runtime spine is independent

### Step 3: Add physics as optional layer
```rust
// Later: physics as runtime subsystem
#[cfg(feature = "physics")]
pub mod physics_integration {
    // Physics subsystem on top of runtime spine
}
```

## CURRENT STATUS: COMPROMISED MINIMALITY

**Claim:** "minimal deterministic headless runtime"  
**Reality:** "runtime with physics dependency"

**The runtime spine is real, but not truly minimal due to physics coupling.**
