# ENGENE — First Run Guide

ENGENE is a three-product platform:

| Product | Executable | Purpose |
|---------|-----------|---------|
| **TEST** | `TEST.exe` | Destruction Sandbox 50x50 — double-click to launch sandbox proving ground |
| **ENGENE Game** | `ENGENE_Game.exe` | Standalone playable world — player, quests, combat, trading, save/load |
| **ENGENE SDK** | `ENGENE_SDK.exe` | World editor — chunk authoring, inspector, doctor, profiler, runtime truth |
| **ENGENE Headless** | `ENGENE_Headless.exe` | CI/balance simulation — no window, pure sim for testing and validation |

## Quick Start

### Option A: Pre-built executables

If `.exe` files exist in the project root or `dist/release/`:

```
TEST.exe                 # launch Destruction Sandbox 50x50 (proving ground)
ENGENE_Game.exe          # launch the game (full world)
ENGENE_Game.exe --layout destruction_sandbox_50x50   # game in sandbox mode
ENGENE_SDK.exe           # launch the editor
ENGENE_Headless.exe --months 6   # run 6-month headless sim
```

### Option B: Build from source

Prerequisites: Rust toolchain (stable), Git.

```powershell
# Build all three
.\build_release.ps1

# Package into dist/release/ and project root
.\package_release.ps1

# Or build and run directly
cargo run --bin engene_game
cargo run --bin engene_sdk --features sdk_tools
cargo run --bin engene_headless -- --months 12
```

### Option C: PowerShell launchers

```powershell
.\run_game.ps1            # launch game (uses .exe if available, else cargo run)
.\run_sdk.ps1             # launch SDK
.\run_game.ps1 -Build     # build then launch
```

## Project Structure

```
/ENGENE_ROOT
  TEST.exe / ENGENE_SDK.exe / ENGENE_Game.exe / ENGENE_Headless.exe
  README_FIRST_RUN.md / SDK_QUICKSTART.md / GAME_QUICKSTART.md

  /src              — engine, SDK, and game source code
  /tests            — integration test suite
  /benches          — performance benchmarks
  /docs/canonical   — current authoritative documents
  /docs/archive     — historical reports and notes
  /game             — game assets, world data, saves
  /dist/release     — packaged release executables
```

## Version Info

Every executable supports `--version`:

```
ENGENE_Game.exe --version
```

This prints engine version, git hash, build profile, feature flags, and schema versions.

## Further Reading

- [SDK_QUICKSTART.md](SDK_QUICKSTART.md) — how to use the editor
- [GAME_QUICKSTART.md](GAME_QUICKSTART.md) — how to play and test
