# PHYSICS_CORE_BOUNDARY

## What is core

Core is the set of crates that provide fundamental engine infrastructure:

- `engine_core` — events, scheduling, system descriptors, ECS primitives
- `engine_ecs` — entity lifecycle, component storage
- `engine_world` — spatial indexing, persistence, streaming, world state
- `engine_runtime` — runtime assembly, wiring orchestration

Core **does not** include:
- physics simulation
- rendering
- audio
- AI decision-making
- content pipeline

## What is physics runtime

Physics runtime is the set of capabilities provided by `engine_physics`:

- collision detection
- ballistics / trajectory simulation
- rigid body dynamics
- destruction topology
- impact event generation

Physics is a **capability**, not a runtime role. It can be enabled in any runtime mode (game, SDK, headless, tools).

## Where physics must live

Physics must live in `crates/engine_physics/`.

This is the **only** crate allowed to contain physics implementation.

## What physics can read from core

Physics may read from core:

- `engine_core::events::EventBus` — to emit and consume events
- `engine_core::system_descriptor::SystemDescriptor` — for system registration
- `engine_core::ecs::Entity` — for entity references
- `engine_core::resources::Resources` — for configuration
- `engine_world` — spatial queries (read-only, via public API)

Physics may **not** read internal implementation of other engine crates.

## What physics must NOT pull back to core

Physics must NOT create dependencies that require:

- Core depending on physics types
- Core depending on physics events
- Core depending on physics systems
- Any engine crate importing `engine_physics` transitively

**Forbidden direction:** `core -> physics`

**Allowed direction:** `physics -> core`

This is a one-way contract.

## Boundary surface

The physics-to-core boundary consists of:

### Events (outbound from physics)
- `ImpactEvent` — collision/ballistic impact
- `WorldTopologyChanged` — terrain deformation
- `SoundTrigger` — physics-triggered audio

### Resources (configuration)
- Physics configuration from `RuntimeConfig`
- Spatial index access via `engine_world` public API

### Systems (registration)
- Physics systems registered via `SystemDescriptor`
- Tick phase: `Phase::Physics`

### Events (inbound to physics)
- `TerrainChanged` — world topology modification
- `EntityMoved` — entity position updates

## Forbidden dependency patterns

1. **No core-to-physics imports** — Core may not `use engine_physics::` anything
2. **No physics re-exports in core** — Core may not re-export physics types
3. **No physics feature in core** — `engine_core` may not enable `physics` feature
4. **No physics events in core event enum** — Core may not define physics-specific events
5. **No physics system descriptors in core** — Core provides registration, not physics systems

## Verification

To verify boundary integrity:

1. Check that `engine_physics/Cargo.toml` depends on `engine_core`, not vice versa
2. Check that no `use engine_physics::` exists in `engine_core`, `engine_ecs`, `engine_world`, `engine_runtime`
3. Check that physics tests in `tests/physics_core_boundary_contracts.rs` pass

## Current state

As of this writing:

- `engine_physics` exists as a **stub** with only `pub mod api { pub const CRATE: &str = "engine_physics"; }`
- No real physics implementation exists
- Physics boundary contract is **not yet enforced** — it is declared
- No physics tests exist in the test suite

## What must exist before physics is "real"

1. `PHYSICS_CORE_BOUNDARY.md` — **this document**
2. `PHYSICS_BOOTSTRAP_CONTRACT.md` — where physics connects to platform
3. `tests/physics_core_boundary_contracts.rs` — boundary tests
4. Real physics implementation in `crates/engine_physics/src/`
5. At least one physics system registered in runtime wiring

## Rule

This is a literal contract, not philosophy.
If a dependency violates this boundary, it is a bug, not a feature.
