# ENGINE-ONLY RECOVERY SCOPE - FINAL

## 🚨 ENGINE-ONLY RECOVERY SCOPE - IMMEDIATE

**All consumer/tooling crates are excluded from current engine recovery scope.**

### ❌ OUT OF CURRENT SCOPE (DO NOT TOUCH)
- `apps/engene_sdk` - SDK consumer layer
- `crates/sdk_app` - SDK application layer  
- `apps/engene_game` - Game consumer layer
- `crates/game_framework` - Game framework layer
- `engine_tools` - Tooling layer
- All editor/render/input/windowing code
- All doctor reports, spatial editor, asset polling
- All UI/GUI/inspection code
- All app bootstrap luxury

### ✅ ENGINE-ONLY SCOPE (SACRED)
**Only these crates participate in engine recovery:**
- `engine_core` - Tiny kernel
- `engine_ecs` - Entity mechanics
- `engine_runtime` - Scheduling/execution
- `engine_world` - Minimal substrate (stub)

**Optional later (NOT NOW):**
- `engine_physics` - Physics boundary layer

## 🎯 NEW OBJECTIVE: PURE DETERMINISTIC COMPUTE MACHINE

**Engine = compute machine, NOT application platform.**

### Input Requirements:
- Math types
- Deterministic rules
- ECS mechanics
- Fixed timestep
- Minimal world substrate
- Scheduling contracts

### Output Requirements:
- ticks/second performance
- Deterministic replay capability
- Bounded allocations
- Measurable hot paths
- No consumer dependencies

## 🚫 BLOCKERS vs NON-BLOCKERS

### ❌ ENGINE BLOCKERS (MUST FIX)
- Core compilation failures
- ECS contract violations  
- Runtime scheduler bugs
- Deterministic contract breaks

### ✅ NOT BLOCKERS (IGNORE DURING RECOVERY)
- sdk_app compilation errors
- engene_sdk bootstrap failures
- game_framework integration issues
- editor tool problems
- UI rendering issues
- Asset loading failures
- Doctor report failures

## 📋 VALIDATION SCOPE

**Engine-only validation:**
```bash
# Core compilation
cargo check -p engine_core
cargo check -p engine_ecs  
cargo check -p engine_runtime
cargo check -p engine_world

# Engine-only tests
cargo test -p engine_core
cargo test -p engine_ecs
cargo test -p engine_runtime
cargo test -p engine_world
```

**Out-of-scope validation:**
```bash
# DO NOT RUN THESE DURING RECOVERY
# cargo check -p sdk_app          # ❌ OUT OF CURRENT SCOPE
# cargo check -p game_framework    # ❌ OUT OF CURRENT SCOPE  
# cargo check -p engine_tools      # ❌ OUT OF CURRENT SCOPE
# cargo test -p engene_sdk       # ❌ OUT OF CURRENT SCOPE
```

## 🏗️ NEW ARCHITECTURAL MODEL

### Tier 1 - Sacred (Engine Recovery)
```
engine_core     -> Tiny kernel (time, determinism, failure taxonomy)
engine_ecs      -> Entity mechanics (components, systems, queries)
engine_runtime   -> Scheduling (fixed tick, deterministic execution)
engine_world     -> Minimal substrate (stub for compilation)
```

### Tier 2 - Consumer Layer (Out of Current Scope)
```
sdk_app         -> SDK applications (IGNORED)
engene_sdk      -> SDK bootstrap (IGNORED)  
game_framework  -> Game framework (IGNORED)
engine_tools     -> Tooling (IGNORED)
apps/*          -> All applications (IGNORED)
```

## 🚨 IMMEDIATE ACTIONS

### 1. NO FIXES FOR OUT-OF-SCOPE CRATES IN THIS PASS
- Do not fix sdk_app compilation errors
- Do not fix engene_sdk bootstrap issues
- Do not fix game_framework integration problems
- Do not spend time on editor/tool issues

### 2. ENGINE-ONLY WORK ONLY
- Fix engine_core compilation
- Fix engine_ecs contracts  
- Fix engine_runtime scheduler
- Verify engine_world stub works
- Test deterministic execution

### 3. NO CONSIDERATION OF CONSUMER NEEDS
- Do not consider SDK requirements
- Do not consider editor usability
- Do not consider game framework integration
- Do not consider tooling completeness

## 🎯 SUCCESS CRITERIA

**Engine recovery complete when:**
1. ✅ All engine-only crates compile clean
2. ✅ All engine-only tests pass
3. ✅ Deterministic runtime verified
4. ✅ Performance baseline established
5. ✅ Core contracts documented

**Consumer crates status: IRRELEVANT during recovery**

## 📞 INSTRUCTIONS TO CLINE

**Stop working on:**
- sdk_app
- engene_sdk  
- game_framework
- engine_tools
- apps/*
- All editor/UI/render code

**Only work on:**
- engine_core
- engine_ecs
- engine_runtime
- engine_world

**If asked about consumer crates:**
"Excluded during engine-only recovery. Not part of current scope."

---

**STATUS: ENGINE-ONLY RECOVERY MODE ACTIVATED**
