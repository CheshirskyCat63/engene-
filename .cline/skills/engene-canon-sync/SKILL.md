---
name: engene-canon-sync
description: Refresh compact project memory after changes to canonical docs, runtime ownership, paths, or architecture. Use when docs/canonical changes, runtime boundaries move, or compact indexes need updating.
---

# ENGENE Canon Sync

## Use Cases
- canonical docs changed
- ownership map changed
- runtime map changed
- path structure changed
- compact context is stale

## Sync Order
1. identify what changed
2. decide which compact files are affected
3. update only affected files:
   - .cline/memory-bank/canonIndex.md
   - .cline/memory-bank/pathIndex.md
   - .cline/memory-bank/runtimeMap.md
   - .cline/fast/canon.fast.md
   - .cline/fast/paths.fast.txt
   - .cline/fast/runtime.fast.md
4. append sync queues if needed
5. keep entries dense and short

## Never
- rewrite all compact files for a local fix
- mirror full canonical docs into compact memory
