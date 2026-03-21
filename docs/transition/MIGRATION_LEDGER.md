# MIGRATION_LEDGER

## Purpose

Temporary structure without a ledger becomes permanent folklore.

## Active items

| Item | Current state | Target state | Exit condition |
|---|---|---|---|
| Root package broad façade | active legacy/migration shell | thin shell or removed | root exports reduced to compatibility-only or removed entirely |
| Root-package bins | active execution truth | package-owned app entrypoints | `apps/*` become actual canonical launch commands |
| Role-blurry features | active | role/capability/tooling/profile separation | Cargo + docs stop mixing categories |
| `sdk_runner.rs` world-kitchen | active | phase driver with explicit phase methods | redraw path split and ordered contract tested |
| `integration.rs` catch-all | active | re-export hub + boundary modules | big domains moved out into dedicated modules |
| Full spatial rebuild in SDK redraw | active | dirty-input incremental path + explicit fallback | runtime path uses dirty contract and equivalence tests |
| Branch-blind CI | active | branch-aware CI | transition branch covered on push/PR |
| Doc drift around entrypoints | active | current-truth docs only | `README_FIRST_RUN.md` and entrypoint docs aligned with Cargo |

## Rule

Anything marked temporary but not written here is not temporary.
It is stealth architecture.
