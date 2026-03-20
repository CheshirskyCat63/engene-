---
name: self-maintain-context
description: Refresh only the compact local context after meaningful repo changes. Use when runtime paths, canon routing, or durable project decisions changed. Do not use for trivial local fixes.
---

# Self Maintain Context

Use this skill only after meaningful changes.

## Trigger Conditions
- runtime ownership changed
- path layout changed
- canonical meaning changed
- durable project decision changed
- build/run entrypoint meaning changed

## Update Order
1. inspect what changed
2. update only affected compact files:
   - .cline/memory-bank/pathIndex.md
   - .cline/memory-bank/runtimeMap.md
   - .cline/memory-bank/canonIndex.md
   - .cline/memory-bank/decisionLog.md
   - .cline/memory-bank/progress.md
   - .cline/fast/paths.fast.txt
   - .cline/fast/runtime.fast.md
   - .cline/fast/canon.fast.md
3. append sync queues only if needed
4. keep everything short and diff-friendly

## Never
- rewrite all compact files for a local bugfix
- mirror full docs into compact files
- expand compact files into essays
