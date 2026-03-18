# ENGENE — First Run Guide

ENGENE is a three-product platform:

| Product | Canonical command | Purpose |
|---------|-------------------|---------|
| **ENGENE Test/Sandbox** | `cargo run --bin engene_test` | Destruction Sandbox 50x50 — proving ground |
| **ENGENE Game** | `cargo run --bin engene_game` | Playable world runtime |
| **ENGENE SDK** | `cargo run --bin engene_sdk` | Editor/workstation for authoring/debugging |
| **ENGENE Headless** | `cargo run --bin engene_headless -- --months 6` | CI/balance simulation without window |

## Quick Start

### Canonical launch paths

```bash
cargo run --bin engene_test
cargo run --bin engene_game
cargo run --bin engene_sdk
cargo run --bin engene_headless -- --months 12
```

### Optional packaged executables

If you build/package for Windows, executables may exist in `dist/release/`:

```powershell
.\build_release.ps1
.\package_release.ps1
```

These packaged outputs are convenience artifacts; canonical development entrypoints remain the `cargo run --bin ...` commands.

### Optional PowerShell launchers

```powershell
.\run_game.ps1            # launch game (uses .exe if available, else cargo run)
.\run_sdk.ps1             # launch SDK
.\run_game.ps1 -Build     # build then launch
```

## Project Structure

```
/ENGENE_ROOT
  README_FIRST_RUN.md / SDK_QUICKSTART.md / GAME_QUICKSTART.md

  /src              — engine, SDK, and game source code
  /tests            — integration test suite
  /benches          — performance benchmarks
  /docs/canonical   — current authoritative documents
  /docs/archive     — historical reports and notes
  /game             — game assets, world data, saves
  /dist/release     — optional packaged release executables
  /legacy/quarantine — quarantined legacy artifacts (non-canonical)
```

## Version Info

Every executable supports `--version`:

```bash
cargo run --bin engene_game -- --version
```

This prints engine version, git hash, build profile, feature flags, and schema versions.

## Further Reading

- [SDK_QUICKSTART.md](SDK_QUICKSTART.md) — how to use the editor
- [GAME_QUICKSTART.md](GAME_QUICKSTART.md) — how to play and test
- [docs/canonical/ENGENE_2_0_ENTRYPOINTS.md](docs/canonical/ENGENE_2_0_ENTRYPOINTS.md) — canonical launch law
