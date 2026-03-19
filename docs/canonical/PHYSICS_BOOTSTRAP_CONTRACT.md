# PHYSICS_BOOTSTRAP_CONTRACT

## Where physics is allowed to connect

Physics may connect to the platform at these points:

### 1. Bootstrap (engine startup)

Physics must be initialized via runtime assembly. This means:
- Physics systems are registered in `engine_runtime` wiring
- Physics resources are inserted into `Resources` during startup
- Physics uses the same phase ordering as other subsystems

**Allowed:** Physics initialization in `RuntimeAssembly::build()` or equivalent.
**Not allowed:** Physics manually spawning threads or creating independent execution contexts.

### 2. Runtime assembly

Physics systems must be added via the runtime assembly pattern:
- Register via `SystemDescriptor::new("PhysicsTick")`
- Specify phase: `Phase::Physics`
- Declare resource dependencies (read/write)
- Declare event dependencies (read/emits)

**Allowed:** `system.add_to_wiring(wiring)`
**Not allowed:** Manual `std::thread::spawn` from physics code

### 3. System registration

Physics systems must be registered as engine systems:
- Must have a `SystemDescriptor`
- Must implement `EngineSystem` trait or equivalent
- Must participate in the tick loop via phase ordering

**Allowed:** Physics as a first-class system in the engine tick
**Not allowed:** Physics as an out-of-band background task

### 4. Resource ownership

Physics may own resources:
- `PhysicsWorld` — the physics simulation state
- `PhysicsConfig` — simulation parameters
- `BallisticsState` — projectile state

These resources are owned by physics and read by other systems (e.g., `BallisticsTickSystem` reads `BallisticsState`).

**Allowed:** Physics resource ownership within `engine_physics`
**Not allowed:** Physics creating global singletons outside `Resources`

## Where physics must NOT connect

### 1. Root shell as permanent center

Physics must not treat `engene` root package as its permanent home. Physics must live in `crates/engine_physics`.

**Not allowed:** New physics code in `src/physics/` root directory
**Not allowed:** Physics binaries in root `[[bin]]` section

### 2. Editor-only paths

Physics should be available in all runtime modes, not just editor.

**Not allowed:** Physics gated behind `feature = "sdk_tools"` only
**Allowed:** Physics in `feature = "physics"` (capability, not mode)

### 3. Random feature growth in root

Physics features must not be added to root `Cargo.toml` as miscellaneous flags.

**Not allowed:** `physics_debug = []` in root `[features]`
**Allowed:** `physics = ["engine_core"]` as a capability feature in root

## Preconditions before physics is "normally connected"

Physics can be considered "normally connected" (not a stub, not an experiment) when:

1. ✅ **Boundary documented** — `PHYSICS_CORE_BOUNDARY.md` exists and is accurate
2. ✅ **Bootstrap documented** — This file exists and is accurate
3. ✅ **Tests exist** — `physics_core_boundary_contracts.rs` passes
4. ✅ **Bootstrap tests exist** — `physics_bootstrap_contracts.rs` passes
5. ✅ **Implementation exists** — Real physics code in `crates/engine_physics/src/`
6. ✅ **At least one system registered** — Physics tick system in runtime wiring
7. ✅ **No root shell dependency** — Physics does not require root package for core functionality

## Current state

As of this writing:

- `PHYSICS_CORE_BOUNDARY.md` — **exists** (this boundary contract)
- `PHYSICS_BOOTSTRAP_CONTRACT.md` — **exists** (this document)
- `engine_physics` crate — provides a minimal bootstrap seam (validation + registration state)
- Physics systems registration is minimal but present (bootstrapped by runtime helper)
- Physics bootstrap tests — **exist and validate real code paths**

**Status:** Physics is partially connected: bootstrap seam exists, but full physics system integration is not yet complete.

## What "not future fanfic" means

This document describes the **current** bootstrap contract, not a future ideal.

- If physics is not connected, this document describes what **would** be true when it is connected
- This document does **not** claim physics is already connected
- This document does **not** create fictional bootstrap paths

If physics implementation does not match this contract, the implementation is wrong, not the contract.

## Fast verification impact

Current fast verification (`just smoke`) does **not** include physics tests because physics is not yet connected.

When physics becomes normally connected:
- `smoke` lane should **not** change (physics is not a fast-path gate)
- `contracts` lane may optionally add physics boundary tests
- `certification` lane may add physics performance tests

## Rule

This is a literal contract. If physics implementation violates bootstrap points, it is a bug.
