# RUNTIME_ROLE_MATRIX

## Why this exists

The project cannot keep mixing:

- runtime roles,
- capabilities,
- tooling overlays,
- deployment profiles.

This matrix defines runtime identity first.

## Runtime roles

| Role | Purpose | Must own | Must not own |
|---|---|---|---|
| Engine | reusable runtime substrate | ECS, world truth, runtime contracts, event infrastructure, persistence laws, spatial contracts | editor UX, game-specific narrative/content policy, app-shell decisions |
| SDK | authoring / inspection / debugging workstation | editor shell, inspectors, dashboards, debug overlays, safe tooling mutation paths | permanent authority over simulation truth |
| Game | playable product runtime | game rules, authored content integration, game-facing bootstrap choices | editor-only tooling behavior |
| Headless | non-visual simulation / CI runtime | deterministic-compatible tick surfaces, batch/simulation entrypoints, non-render execution | renderer-first assumptions, editor mutation flows |
| Tools | diagnostics / maintenance / doctor surfaces | audits, reports, migration diagnostics, repair/inspection tools | gameplay ownership |

## Capabilities vs roles

Capabilities are **not** runtime identities.

Examples of capabilities:
- render
- physics
- ai
- audio
- networking

These may appear inside multiple runtime roles, but they do not define the role itself.

## Tooling overlays

Examples:
- debug UI
- editor panels
- instrumentation
- replay inspection

These are overlays, not products.

## Deployment / quality profiles

Examples:
- low_spec
- dev
- ci
- release

These are execution profiles, not capabilities and not roles.

## Current branch implication

Current features and docs must be rewritten so that:
- role comes first,
- capability comes second,
- overlay/tooling comes third,
- profile comes fourth.
