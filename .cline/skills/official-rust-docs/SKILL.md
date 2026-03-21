---
name: official-rust-docs
description: Research Rust, Cargo, rustc, Clippy, async, and crate behavior from official documentation. Use when library behavior is uncertain, APIs may have changed, a standard/tooling question needs verification, or a senior-level implementation decision needs doc-backed confirmation.
---

# Official Rust Docs

Use this skill when correctness depends on up-to-date or authoritative documentation.

## Objectives
- prefer official sources first
- minimize speculation
- return compact findings
- extract only decision-relevant facts

## Source Order
1. official Rust docs
2. official crate docs or project docs
3. docs.rs when it reflects the crate API surface
4. issue trackers only if docs are insufficient

## Research Process
1. identify the exact uncertainty
2. search official docs first
3. read only the relevant section
4. extract:
   - behavior
   - constraints
   - version-sensitive caveats
   - code-impact summary
5. report:
   - what is confirmed
   - what remains uncertain
   - what code change is justified

## For This Repo
- keep results short
- add durable conclusions to .cline/memory-bank only if they affect repeated work
- do not dump large doc excerpts into context

See [sources.md](docs/sources.md)
