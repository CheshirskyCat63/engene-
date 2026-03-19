# ENGENE 2.0 SDK Program

## SDK 2.0 tooling roadmap

### S1: Foundation workstation
- Entity inspector/editor with archetype-aware panels.
- Live world state browser (L0-L3 simulation view).
- Scenario launcher and deterministic replay controls.

### S2: Authoring tools
- Layered material editor (penetration, fracture, acoustic coefficients).
- Body/damage authoring (hit zones, limb sever thresholds, gore variant binding).
- Break-state graph editor for structures.

### S3: Simulation debugging
- Destruction visualization (stress/support/debris budgets).
- Fire/water/wetness overlays and event playback.
- Weather front map with timeline scrubbing.
- Sound propagation and AI hearing overlays.

### S4: Validation and performance
- Content validation pipelines (schema + semantic constraints).
- Perf dashboards per subsystem + per simulation level.
- Budget violation alerts with capture-to-file.
- Showcase setup wizard and acceptance checklist runner.

## Required SDK non-negotiables
- Every flagship system must expose editable data and debug visualization.
- Every runtime budget must be visible and recordable in SDK.
- One-click export of scenario evidence package (config, metrics, replay hash).
