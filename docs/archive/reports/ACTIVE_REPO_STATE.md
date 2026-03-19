# ACTIVE_REPO_STATE

## Status label

**Transition-shaped repo. Split declared; split not fully enforced.**

## Verified current state

### Cargo / package truth
- Workspace is declared and includes root package, engine crates, SDK/game crates, and `apps/*`.
- Root package `engene` still exists and still hosts the active bins:
  - `engene_game`
  - `engene_sdk`
  - `engene_headless`
  - `engene_tools`
- Root feature model is still role-blurry:
  - domain-ish: `physics`, `render`, `ai`, `audio`, `networking`
  - mode-ish: `headless`, `low_spec`
  - tooling-ish: `debug_ui`, `sdk_tools`
  - bundle-ish: `full`

### Root export truth
`src/lib.rs` still re-exports the broad monolith module tree:

- `animation`
- `app`
- `audio`
- `body`
- `content`
- `core`
- `engine`
- `game`
- `graphics`
- `input`
- `memory`
- `navigation`
- `network`
- `physics`
- `runtime`
- `simulation`
- `testsupport`
- `tools`
- `world`

### Entrypoint truth
Current first-run truth is the root-package bins, not `apps/*` package execution.

### SDK orchestration truth
`src/app/sdk_runner.rs` still performs multiple responsibilities inside redraw handling:

- camera/input update
- simulation stepping
- asset polling
- world streaming
- chunk persistence handoff
- full spatial rebuild
- audio listener update
- dashboard updates
- inspector mutation
- render preparation
- render error handling

### Wiring truth
`src/runtime/wiring/integration.rs` still mixes multiple boundary domains in one file:

- ballistics / damage
- destruction / terrain
- nav / occlusion
- gore
- AI wiring
- animation wiring

### Test surface truth
The transition branch already has real named suites suitable for lane-based execution:

- smoke candidates:
  - `engine_contracts`
  - `production_candidate`
- contract candidates:
  - `engine_contracts`
  - `determinism_and_sdk`
  - `runtime_systems`
- certification candidates:
  - `certification_boundary_overhead`
  - `certification_kernel_throughput`
  - `certification_tick_budget`
  - `certification_perf_snapshot`

### Documentation truth
- `README_FIRST_RUN.md` still mentions `engene_test`, which is not present in current Cargo bin declarations.
- `docs/canonical/ENGENE_2_0_ENTRYPOINTS.md` is transitional and must be rewritten around current launch truth.
- Docs must stop certifying future package ownership as if it were already active runtime truth.

### CI truth
`.github/workflows/rust.yml` still watches only `main`.

## Immediate implications

1. The repo is not allowed to describe the split as completed.
2. The root crate must be treated as a migration shell, not as a permanent architecture truth.
3. Operator docs must use current bins and current test targets.
4. The next big wins are not shiny features; they are:
   - truth lockdown,
   - runtime role law,
   - orchestration split,
   - spatial dirty-path enforcement,
   - test operator ergonomics.
