---
name: rust-debugging
description: Diagnose Rust compile errors, failing tests, bad imports, path issues, and local runtime problems. Use when the code does not compile, tests fail, symbols are unresolved, or behavior regressed.
---

# Rust Debugging

## Debug Order
1. reproduce the failure
2. inspect exact error site
3. inspect nearest owner module
4. inspect recent local changes if available
5. patch only what the evidence supports
6. re-run narrow validation

## Default Commands
- cargo check -p <crate>
- cargo test -p <crate> <target>
- cargo test <specific_test_name>

## Rules
- do not widen scope early
- do not rewrite unrelated code
- fix the root cause, not just the symptom
- if docs/tooling behavior is unclear, activate official-rust-docs
