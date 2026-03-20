# Path Index

root areas:
- crates/ = engine and shared core
- apps/ = entrypoints / bootstrap
- game/ = game-facing runtime
- docs/ = docs source
- docs/canonical/ = canonical truth when present
- scripts/ = helper automation
- tests/ = active tests
- tests_legacy/ = legacy tests
- legacy/quarantine/ = read-only default
- .cline/ = compact agent cache
- .cline/memory-bank/ = compact persistent memory

routing:
- runtime question -> apps/, crates/, game/, runtimeMap.md, .cline/fast/runtime.fast.md
- docs question -> .cline/fast/docs.fast.txt, canonIndex.md, docs/canonical/*
- run/build question -> .cline/fast/commands.fast.txt, ROOT_DOCS_INDEX.txt, root quickstarts
- test question -> .cline/fast/tests.fast.txt, tests/, crate-local tests

