# ENGENE 2.0 Repo Hygiene and Handoff

## Purpose
Make repository readability, cleanup discipline, and engineer handoff explicit release-quality requirements.

## Folder readability law
- Top-level directories must have single clear purpose.
- Ambiguous buckets (`misc`, `temp`, `new2`, `final_final`) are forbidden.
- Overlapping parallel trees with unclear ownership are forbidden.
- Each major directory has owner and short README/index.

## Module readability law
- Public entrypoints are obvious and documented.
- Internal/private modules hidden by default.
- No deep implicit import chains for normal use.
- Consistent naming across crates/modules/commands/artifacts/docs.

## Legacy quarantine law
- Deprecated code is never mixed into active runtime paths.
- Legacy is kept only in explicit quarantine (`legacy/`, `_archive/`) with status markers.
- Archived assets/tools must include reason, owner, and date.

## Handoff law
A fresh engineer must be able to identify quickly:
- where engine starts,
- where sdk starts,
- where game starts,
- where authored content lives,
- what is canonical/generated/deprecated,
- how to run core apps,
- how to validate/cook content,
- how to inspect performance and showcase proofs.

## Mandatory handoff docs
- `README.md` (root quick orientation).
- `docs/REPO_MAP.md`.
- `docs/HOW_TO_RUN.md`.
- `docs/HOW_TO_VALIDATE_AND_COOK.md`.
- `docs/HOW_TO_DEBUG_SHOWCASE.md`.

## Cleanup status model
Every legacy candidate is classified as:
1. **Delete now** — dead and unused.
2. **Quarantine** — maybe useful, not active.
3. **Refactor then move** — active but structurally wrong.
4. **Canonical keep** — active and correctly owned.

Each item tracks:
- owner,
- current path,
- target path,
- status,
- removal phase/date if deprecated.
