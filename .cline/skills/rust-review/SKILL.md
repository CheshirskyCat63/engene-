---
name: rust-review
description: Review Rust changes like a senior engineer. Use when reviewing diffs, dependency changes, API changes, test quality, boundary violations, or code that may be correct but risky.
---

# Rust Review

## Review Axes
- correctness
- ownership and boundaries
- dependency impact
- test coverage
- determinism and runtime safety
- maintainability
- unnecessary scope growth

## Review Process
1. identify changed files
2. identify owning crate/module
3. inspect direct impact surface
4. call out risks
5. separate must-fix from nice-to-have
6. suggest smallest safe improvement

## Repo Rules
- engine/sdk/game split is mandatory
- canonical docs matter for boundary and contract changes
- prefer precise, reviewable fixes over grand rewrites
