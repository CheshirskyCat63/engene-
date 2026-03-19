# DEPENDENCY_DIRECTION_REPORT

Date: 2026-03-17

## Rule set
- ENGINE -> GAME: forbidden
- ENGINE -> SDK: forbidden
- GAME -> ENGINE: allowed
- SDK -> GAME: integration-only

## Gate
- Script: `scripts/check_dependency_direction.sh`
- Strict ENGINE scan scope:
  - `src/core`
  - `src/world`
  - `src/memory`
  - `src/physics`
  - `src/audio`
  - `src/graphics`
  - `src/navigation`
  - `src/content`
  - `src/simulation`
  - `src/animation`
  - `src/input`
  - `src/body`
  - `src/network`

## Result
- Strict engine->game imports: NOT FOUND

Verdict: IMPLEMENTED.
