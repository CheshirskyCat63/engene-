# DOC_STATUS_BOARD

## Tier 0 — truth blockers

| Doc | Status | Why it matters |
|---|---|---|
| `ACTIVE_REPO_STATE.md` | required now | prevents docs from certifying fantasy |
| `ENTRYPOINT_TRUTH.md` | required now | fixes launch ambiguity |
| `ROOT_CRATE_POLICY.md` | required now | stops root shell from silently becoming permanent |
| `RUNTIME_ROLE_MATRIX.md` | required now | separates role from capability/tooling |
| `TEST_LANE_MAP.md` | required now | makes tests operable |

## Tier 1 — control docs

| Doc | Status | Why it matters |
|---|---|---|
| `WORKSPACE_OWNERSHIP_MAP.md` | required now | defines ownership before more split work |
| `DEPENDENCY_LAW.md` | required now | prevents reverse coupling |
| `PHASE_ORDER_CONTRACT.md` | required now | de-monolith driver work needs order law |
| `SPATIAL_DIRTY_CONTRACT.md` | required now | turns perf wish into runtime contract |
| `RUNTIME_INVARIANTS.md` | required now | gives testing/CI hard targets |

## Tier 2 — migration and debt

| Doc | Status | Why it matters |
|---|---|---|
| `MIGRATION_LEDGER.md` | required now | prevents temporary forever |
| `REMOVAL_PLAN.md` | required now | split must delete old sinks |
| `STALE_DOCS_TO_ARCHIVE.md` | required now | removes conflicting guidance |

## Rule

No new canonical doc enters the set unless it changes rules, removes ambiguity, improves operator execution, or records migration/debt truth.
