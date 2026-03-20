---
name: rust-implementation
description: Implement or refactor Rust code safely in a workspace with multiple crates. Use when adding features, reshaping module boundaries, wiring functionality, or making production-quality code changes in Rust.
---

# Rust Implementation

## Default Process
1. inspect owning crate and module
2. inspect direct callers and direct dependencies
3. inspect relevant tests
4. make smallest safe implementation change
5. run narrow validation
6. widen only if needed

## Code Quality Rules
- prefer explicit ownership
- prefer readable APIs
- avoid accidental coupling
- preserve deterministic behavior
- keep diffs reviewable
- do not add dependencies casually

## Repo-Specific Rules
- keep engine / sdk / game separation
- use .cline/fast and .cline/memory-bank first
- consult canon only if ownership, contracts, or runtime boundaries are involved

## Escalation
Use official-rust-docs if:
- std behavior is uncertain
- cargo behavior is uncertain
- external crate behavior is uncertain
- a language/tooling rule needs verification
