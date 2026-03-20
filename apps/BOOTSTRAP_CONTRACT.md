# Bootstrap Application Architecture Contract

## Purpose
The `engene_bootstrap` application serves as a launcher for other engine applications.

## Architectural Rules

### ✅ ALLOWED
- **Process launching**: Use `std::process::Command::new()` to spawn other executables
- **Argument parsing**: Parse command-line arguments to determine target application
- **Error handling**: Provide clear error messages for failed launches

### ❌ FORBIDDEN
- **Direct engine crate usage**: Never import or call engine crates directly
  - `game_framework::run*`
  - `sdk_app::run*`
  - `engine_runtime::engine::Engine`
  - Any other engine crate APIs
- **Root module imports**: No `use engene::` imports
- **Engine initialization**: Bootstrap must NOT initialize engine systems

## Implementation Pattern
```rust
// ✅ CORRECT: Use Command::new for launching
let status = Command::new(target_exe())
    .args(child_args)
    .status();

// ❌ WRONG: Direct engine usage
// game_framework::run_from_env_args(); // FORBIDDEN
```

## Verification
This contract is enforced by:
1. `gate_3_apps_launch_from_canonical_crates` test in `tests/architectural_gates.rs`
2. CI pipeline legacy path detection

## Rationale
Bootstrap is a system-level launcher, not an engine application. It must remain lightweight and independent of engine internals to avoid circular dependencies and maintain clean separation of concerns.
