# How to Validate and Cook (Phase 0 baseline)

## Architecture/boundary gates
```bash
bash scripts/check_dependency_direction.sh
bash scripts/check_ecs_direct_access.sh
```

## Build validation
```bash
cargo check --workspace
```

## Content/cooking note
Current repo remains pre-Phase A monolith; content pipeline law is defined in:
- `docs/canonical/ENGENE_2_0_CONTENT_PIPELINE.md`

Phase A+ will formalize toolchain commands and artifacts as workspace crates split proceeds.
