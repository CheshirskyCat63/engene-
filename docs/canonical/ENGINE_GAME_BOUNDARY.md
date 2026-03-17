# ENGINE_GAME_BOUNDARY

Status: IMPLEMENTED

## Ownership table (final audit)

| Area | Ownership | Status | Notes |
|---|---|---|---|
| `src/core/*` | ENGINE | IMPLEMENTED | engine runtime, ECS, scheduling, config core |
| `src/world/*` | ENGINE | IMPLEMENTED | world data/runtime infra |
| `src/memory/*` | ENGINE | IMPLEMENTED | persistence and save I/O |
| `src/physics/*` | ENGINE | IMPLEMENTED | reusable simulation subsystems |
| `src/audio/*` | ENGINE | IMPLEMENTED | reusable audio runtime |
| `src/graphics/*` | ENGINE | IMPLEMENTED | renderer/camera/visibility |
| `src/navigation/*` | ENGINE | IMPLEMENTED | nav systems and pathing |
| `src/simulation/*` | ENGINE | IMPLEMENTED | simulation infra + time events, no game imports |
| `src/animation/*` | ENGINE | IMPLEMENTED | pure animation runtime modules |
| `src/tools/*` | SDK | IMPLEMENTED | doctor/editor/dashboards/debug tooling |
| `src/app/*` | SDK | IMPLEMENTED | runtime assembly and app-facing SDK shell |
| `src/testsupport/*` | SDK | IMPLEMENTED | harnesses and stress tooling |
| `src/game/*` | GAME | IMPLEMENTED | AI/economy/ecosystem/gameplay/game runtime hooks |
| `src/bin/*` | APPS | PARTIAL | ownership clear, but heavy logic still in bins |

## Dependency policy

- ENGINE -> GAME: forbidden.
- SDK -> ENGINE: allowed.
- SDK -> GAME: allowed only for editor/runtime integration surfaces.
- GAME -> ENGINE: allowed.

## Current boundary verdict

- Reverse dependency gate (`scripts/check_dependency_direction.sh`) passes across strict ENGINE scope, including `src/simulation` and `src/animation`.

Verdict: IMPLEMENTED.
