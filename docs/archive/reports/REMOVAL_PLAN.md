# REMOVAL_PLAN

## Why this exists

A split plan that only adds files and never defines deletions produces fossil layers.

## Planned removals / demotions

| Old artifact / behavior | Replacement | Removal gate |
|---|---|---|
| root-wide broad export surface | package/crate ownership + thinner façade | all active callers stop relying on broad root re-export |
| root-package canonical entrypoint docs | package-owned app truth docs | `apps/*` become real operator launch surface |
| `sdk_runner.rs` giant redraw orchestration | explicit phase methods | phase-order tests + behavior parity |
| `integration.rs` mixed-domain wiring | boundary modules + re-export shell | modules compile, tests pass, file becomes thin hub |
| unconditional full spatial rebuild in editor redraw | dirty-input incremental path + explicit fallback | equivalence tests, chunk invalidation tests, origin-shift tests |

## Removal principle

A thing is removable only when all three are true:

1. replacement exists,
2. replacement is validated,
3. operators no longer rely on the old path.

If any of these are false, deletion is theatre.
