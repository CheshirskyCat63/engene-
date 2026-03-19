# ENGENE 2.0 Product Boundaries

## Purpose
Lock product-level separation so Engine, SDK, and Game remain distinct products with auditable ownership.

## Product separation law
- **Engine 2.0**: reusable simulation/runtime capability + canonical contracts.
- **SDK 2.0**: authoring/validation/debug/profiling workstation.
- **Game 1.0 Tech Demo**: executable showcase product proving integration.

No product may absorb another product's ownership domain for convenience.

## Engine 2.0 ownership
Engine owns:
- General systemic runtime capabilities.
- Canonical simulation contracts and data interfaces.
- Performance/concurrency/runtime truth machinery.

Engine must not own:
- Scenario-specific gameplay hacks.
- SDK UI/editor ownership.
- Demo-only special-case branches.

## SDK 2.0 ownership
SDK owns:
- Authoring tools and workflows.
- Validation/cooking/debug/perf visualization surfaces.
- Replay/inspection/evidence tooling.

SDK must not own:
- Runtime authoritative truth.
- Shipping gameplay correctness path requirements.

## Game 1.0 Tech Demo ownership
Game owns:
- Scenario rules/content and showcase assembly.
- Proof of Engine+SDK integration in executable form.

Game must not own:
- Canonical engine contract definitions.
- Engine runtime truth semantics.

## Hard boundary rules
1. Engine remains valuable without this specific game.
2. SDK remains usable without game-specific engine internals.
3. Game may depend on engine and optional SDK debug adapters, never inverse ownership.
4. “Only for demo” is never a justification for boundary violations.
5. No subsystem may leak product-specific assumptions into shared core contracts.
