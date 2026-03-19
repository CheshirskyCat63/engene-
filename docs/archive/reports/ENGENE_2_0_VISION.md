# ENGENE 2.0 Vision

## Product Core

### 1) Engine 2.0 product definition
ENGENE 2.0 is a **scalable simulation runtime** for destructible, systemic combat worlds where fidelity is concentrated in a local active bubble and approximated elsewhere. It is not a literal full-physics planet simulator.

Engine 2.0 ships:
- Deterministic-enough simulation layers (L0-L3) with persistence hooks.
- Material-aware ballistics and destruction pipeline.
- Coupled fire/water/weather state model.
- Population-scale AI simulation via abstraction tiers.
- Audio perception and propagation APIs integrated with AI and gameplay.
- Metrics-first runtime observability and budget enforcement.

### 2) SDK 2.0 product definition
SDK 2.0 is a **simulation workstation** for authoring, debugging, validating, and profiling systemic content:
- Material/layer authoring.
- Damage/body profiles.
- Fire/wetness/weather overlays.
- Audio propagation debug.
- Scenario authoring + one-click showcase launch.
- Content validation and performance budget dashboards.

### 3) Game 1.0 Tech Demo definition
Game 1.0 Tech Demo is an executable technology-demonstration game product built on Engine 2.0 and supported by SDK 2.0.
Its purpose is to prove integrated runtime behavior, authoring pipeline viability, and showcase-ready gameplay, not to define engine ownership.

### 4) Showcase-scene definition
One executable scenario with manual controls and scripted triggers demonstrating:
- Layered enemies and body part damage transitions.
- Dismemberment/gore states with bounded runtime cost.
- Ballistic penetration through layered materials.
- Tile shatter exposing concrete and support response.
- Explosive structural failure and nav updates.
- Wind-driven fire spread plus rain/wetness interactions.
- Puncturable containers leaking and pooling behavior.
- Distant weather fronts and local precipitation impact.
- 10k enemies + 5k allies represented with multi-level simulation.
- Audio occlusion/propagation changes visible in debug tooling.

## Performance-first law
ENGENE 2.0 is not only a systems-rich runtime; it is a data-oriented, CPU-efficient, multithreaded simulation runtime.
No subsystem is architecturally complete unless its hot-path data layout, batching model, multithread execution model, and budget behavior are defined and observable.

## Hot-path law
If a system cannot explain:
- how its hot data is packed,
- how it batches work,
- how it parallelizes safely,
- how it bounds synchronization,
- how it avoids heap churn,
- how it degrades under stress,
then it is not production-ready even if features are functionally correct.

## Scope framing for 2.0

### 5) In scope
- Engine workspace split into clear crates and dependency direction gates.
- Hybrid destruction (sim + authored states + budgeted debris).
- Hybrid body damage (hit zones + tissue/material classes + authored gore variants).
- Fire/water/weather simulation with local high-fidelity and far-field abstraction.
- Audio runtime with obstruction, material response, reverb zones, and AI hearing.
- SDK workstation tooling for all flagship systems.
- One production-grade showcase scene.

### 6) Explicitly out of scope
- Fully physically exact fluid dynamics.
- Fully finite-element structural simulation for whole city.
- Exact global weather simulation at meteorological resolution.
- Full campaign/game content completeness.
- Massively multiplayer netcode parity for all systems.

### 7) Must be physically simulated (local/high-fidelity)
- Projectile trajectory and per-layer energy loss.
- Explosive impulse within active bubble.
- Contact/fragment events near player camera bubble.
- Fire ignition/spread decisions in active sectors.
- Rain/wetness accumulation in loaded cells.

### 8) Must be approximated (bounded models)
- Far-field structural stress.
- Far-field fire spread beyond loaded sectors.
- Global weather fronts and cloud transport.
- Off-bubble combat outcomes for populations.
- Audio beyond local relevance radius.

### 9) Must be authored by content/tools
- Material stacks and response curves.
- Break-state graphs and collapse archetypes.
- Enemy body topology, gore assets, dismember thresholds.
- Fire behavior coefficients per material class.
- Weather presets/front templates.
- Audio surfaces, reverb volumes, propagation tags.
- Showcase triggers, spawn waves, scripted validation tracks.

## Shipping vs experimental
- **Shipping in 2.0**: layered penetration, hybrid destruction, fire/wetness/weather coupling, multi-level population sim, audio propagation + AI hearing, SDK visualization.
- **Experimental in 2.0**: advanced volumetric smoke interaction, high-order structural FEM, full CFD fluids, machine-learned crowd behavior.
