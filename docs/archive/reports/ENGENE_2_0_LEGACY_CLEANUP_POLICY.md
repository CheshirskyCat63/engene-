# ENGENE 2.0 Legacy Cleanup Policy

## Purpose
Define how legacy code/assets/tools are removed, quarantined, or migrated before and during workspace split.

## Classification model
1. **Delete now** — dead and unused.
2. **Quarantine** — maybe useful, not active.
3. **Refactor then move** — active but structurally wrong.
4. **Canonical keep** — active and correctly owned.

## Required metadata per legacy item
- owner
- current path
- target path (if migrating)
- status/classification
- rationale
- removal phase/date if deprecated

## Rules
- No unclassified legacy artifact may remain in primary shipping paths.
- Quarantined artifacts are non-shipping and clearly marked.
- Legacy launchers/scripts must be removed from canonical entrypoint docs.
- Migration cannot copy legacy ambiguity into new crate boundaries.

## Enforcement
- Phase 0 report is mandatory baseline artifact.
- CI/check tooling must fail when deprecated paths are referenced by canonical apps.
- Stale quarantine without owner/date is non-compliant.
