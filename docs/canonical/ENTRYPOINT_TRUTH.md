# ENTRYPOINT_TRUTH

## Rule

There must be **one** canonical description of how ENGENE starts in the current branch.

That description lives here.

## Current canonical entrypoints

| Runtime role | Current command | Current owner |
|---|---|---|
| Game | `cargo run --bin engene_game` | root package bin |
| SDK | `cargo run --bin engene_sdk` | root package bin |
| Headless | `cargo run --bin engene_headless -- --ticks 1200` | root package bin |
| Tools | `cargo run --bin engene_tools` | root package bin |

## Not current truth

The following are **not** current canonical launch truths in this branch:

- `cargo run --bin engene_test`
- package-level execution through `apps/*` as the primary documented start path

## Ownership statement

The workspace may already declare `apps/engene_game`, `apps/engene_sdk`, and `apps/engene_headless`,
but until package-level execution replaces the root-package bins in actual operator use,
the documentation must describe the root-package bins as canonical.

## Transition policy

When package-level entrypoints become the active truth, this file changes in one step:

1. package commands become canonical,
2. root-package bin paths become compatibility-only,
3. old commands are marked deprecated with removal criteria.

No document may advertise a future entrypoint as current truth.

## Validation checklist

Any doc that describes launch paths must pass all of these:

- names only currently declared bins or packages,
- matches Cargo reality,
- does not invent `test/sandbox` binaries,
- explicitly distinguishes current path vs. target path,
- points back to this file.
