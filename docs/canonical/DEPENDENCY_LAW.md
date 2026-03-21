# DEPENDENCY_LAW

## Purpose

Physical split without dependency law is just distributed spaghetti in formal wear.

## Allowed high-level direction

```text
apps/*  ->  sdk_app / game_framework  ->  engine_* crates
root migration shell  ->  temporary adapters only
```

## Forbidden direction

### Engine layer may not depend on:
- `sdk_app`
- `game_framework`
- `apps/*`

### SDK app may not depend on:
- `apps/*`
- game-only authored runtime logic that does not belong to editor tooling

### Game framework may not depend on:
- SDK/editor-only panels or overlays

### App shells may not define domain truth
They launch, configure, and compose.
They do not own engine law.

## Root shell exception policy

The root package may temporarily depend across boundaries only when:

1. it is explicitly acting as a migration adapter,
2. the exception is recorded in `MIGRATION_LEDGER.md`,
3. there is a removal condition,
4. no new domain truth is born there.

## Review questions

Every dependency change must answer:

- why does this dependency belong to this layer?
- why is the reverse direction forbidden?
- is this permanent or transitional?
- what file proves the decision?

If those answers are weak, the dependency is weak.
