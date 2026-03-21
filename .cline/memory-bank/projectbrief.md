# Project Brief

project: ENGENE
repo: CheshirskyCat63/engene-
branch baseline: engene-2.0-transition

goal:
build and harden a clean runtime split:
- engine
- sdk
- game

hard constraints:
- no ownership collapse
- no silent engine/sdk/game mixing
- canon before architecture edits
- minimal safe edits
- focused validation first
- legacy/quarantine is read-only by default

startup order:
1. .cline/fast
2. memory-bank
3. direct source files
4. relevant canon only if needed

