# ENGENE 2.0 Entrypoints

## Purpose
Single sanity-check reference for canonical runtime entrypoints to avoid multiple conflicting launch paths.

## Canonical commands

### Game executable path
```bash
cargo run --bin engene_game
```

### SDK workstation path
```bash
cargo run --bin engene_sdk
```

### Test/sandbox executable path
```bash
cargo run --bin engene_test
```

### Headless runtime path
```bash
cargo run --bin engene_headless
```

## Canonical rule
- These commands are the default canonical launch entrypoints.
- Any alternate launch method must be documented and mapped to one of these runtime products.
- Deprecated launch scripts/binaries must not be presented as primary entrypoints.

## Validation reminder
When workspace split to `apps/*` is complete, this file must be updated to crate/package-level canonical commands while preserving single-source-of-truth semantics.
