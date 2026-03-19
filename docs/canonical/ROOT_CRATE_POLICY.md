# ROOT_CRATE_POLICY

## Status

The root crate `engene` is a **migration shell**.

It is not allowed to silently become the permanent architecture center.

## What the root crate may do

The root crate may temporarily act as:

- workspace-visible compatibility façade,
- temporary home for active bins while `apps/*` ownership is unfinished,
- migration adapter between legacy module layout and workspace crate layout,
- execution shell during staged split.

## What the root crate may not do

The root crate may **not** become the place where new architecture is born.

Forbidden:

- new domain ownership,
- new permanent runtime subsystems,
- new cross-boundary orchestration growth,
- new catch-all exports,
- new feature bundles that blur roles further.

## Mandatory behavior while it exists

1. Every new temporary root responsibility must be logged in `MIGRATION_LEDGER.md`.
2. Every compatibility export must have a removal target.
3. Every new bin or entrypoint decision must update `ENTRYPOINT_TRUTH.md`.
4. Root shell logic must move toward thin façade status, not toward renewed centrality.

## Exit criteria

The root crate can stop being a migration shell only in one of two ways:

### Option A — thin permanent shell
Allowed only if:
- it owns no domain logic,
- it owns no runtime orchestration,
- it only forwards into package-owned apps/crates,
- dependency law remains one-way.

### Option B — removal
Preferred if:
- package-owned apps fully replace current bins,
- façade exports are no longer needed,
- tooling and docs are updated,
- CI and operator commands no longer depend on root ownership.

## Architectural smell rule

If someone says “let’s just put it in root for now”,
that counts as migration debt immediately and must be written down.
Temporary code is immortal until proven otherwise.
