# STALE_DOCS_TO_ARCHIVE

## Correct or archive immediately

### 1. `README_FIRST_RUN.md` (old version)
Reason:
- referenced `engene_test` as a canonical entrypoint even though current Cargo bins do not include it.

Action:
- replace with the version in this gold drop.

### 2. `docs/canonical/ENGENE_2_0_ENTRYPOINTS.md` (old version)
Reason:
- transitional wording around ownership no longer works as the primary truth doc.
- must redirect to current launch truth instead of narrating future package ownership as present reality.

Action:
- replace with the compatibility redirect version in this gold drop.

### 3. Any doc that says or implies “split complete”
Reason:
- current branch still uses root-package bins and broad root exports.
- that means the split is declared, but not fully enforced.

Action:
- demote to archive, or rewrite against `ACTIVE_REPO_STATE.md`.

## Practical audit question

For every canonical doc ask:

> Does this file describe what the repo does today, or what we hope it does after three more cleanup passes?

If the answer is “hope,” it is not canonical yet.
