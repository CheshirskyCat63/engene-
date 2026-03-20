---
name: rust-performance
description: Investigate and improve Rust performance in hot paths, data movement, allocations, determinism-sensitive loops, and runtime-critical systems. Use when profiling, optimizing, or reviewing perf-sensitive code.
---

# Rust Performance

## Perf Workflow
1. identify the hot path
2. define what is slow
3. inspect allocation patterns, cloning, iteration shape, data layout, and synchronization
4. propose minimal perf-safe changes
5. validate behavior first
6. measure again if measurement is available

## Optimization Rules
- prefer evidence over folklore
- avoid speculative micro-optimizations
- keep correctness and determinism first
- avoid large refactors unless perf gain justifies it

## Repo Rules
- if change touches runtime law, ordering, event flow, or concurrency, inspect relevant canon group
- keep perf notes compact
