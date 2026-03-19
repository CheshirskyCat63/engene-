# ENGENE 2.0 Runtime & Dependency Baseline

## Scope
This document is the base closure for runtime ownership and dependency direction.
It defines the current frozen runtime model and what is forbidden.
Detailed minimal-stack and dependency-admission laws are in:
- `ENGENE_2_0_MINIMAL_ENGINE_STACK.md`
- `ENGENE_2_0_DEPENDENCY_ADMISSION_LAW.md`

## Frozen runtime roles
Only three runtime roles are canonical:
1. **Game**
2. **Headless (Kernel)**
3. **Tools/SDK**

No other runtime identity is canonical.
No sandbox/demo runtime role is allowed.

## Entrypoints and ownership
- `src/bin/engene_game.rs` -> **Game** (`app::game_runner`).
- `src/bin/engene_headless.rs` -> **Headless** (`app::headless_runner`).
- `src/bin/engene_sdk.rs` -> **Tools/SDK** (`app::sdk_runner`).
- `src/bin/engene_tools.rs` -> **Tools/SDK** diagnostics bootstrap.

## Dependency direction (enforced policy)
### A. Engine core/kernel
Allowed:
- `core::*`
- ECS/scheduler/events/runtime-config/runtime-manifest
- engine-level simulation loop and kernel systems

Forbidden:
- game content bootstrap (`game/data`, `GameConfig` loading)
- gameplay plugins/systems bootstrap
- tools UI/doctor bootstrap

### B. Engine + tools/sdk
Allowed:
- engine core + tooling UI and diagnostics (`tools::*`)
- tools runtime bootstrap (`ToolsRuntimeAssembly::minimal`)

Forbidden:
- gameplay/world content bootstrap
- authored game data loading for tools baseline startup

### C. Game + engine
Allowed:
- engine core + game systems/plugins/content bootstrap
- `GameRuntimeAssembly` paths

Forbidden:
- hidden fallback to deprecated roles

## Bootstrap policy
- `runtime::bootstrap::EngineRuntimeAssembly::kernel_headless()` is the canonical headless bootstrap (`src/runtime/bootstrap/headless.rs`).
- `runtime::bootstrap::ToolsRuntimeAssembly::minimal()` is tools-only bootstrap (`src/runtime/bootstrap/tools.rs`).
- `runtime::bootstrap::GameRuntimeAssembly::vertical_slice()` is canonical game bootstrap (`src/runtime/bootstrap/game.rs`).

## Test policy
- Tests must import explicit assemblies (`EngineRuntimeAssembly`, `ToolsRuntimeAssembly`, `GameRuntimeAssembly`).
- `RuntimeAssembly` legacy facade is forbidden.
- Legacy runtime naming in test identities should be migrated when touched.

## Legacy-tail policy
- Do not add new references to removed runtime concepts:
  - `RuntimeAssembly` facade
  - sandbox/demo runtime role
  - removed runtime profile variants (`VerticalSlice`, `DebugTools`, `LowSpec`, `Shipping`, `Sandbox`)
- Prefer explicit role language in code and docs.

## Audit command
Use this grep to continuously audit legacy tails:

```bash
rg -n "RuntimeAssembly|Sandbox|sandbox|VerticalSlice|DebugTools|LowSpec|Shipping|engene_test|destruction_sandbox_50x50" src tests docs Cargo.toml -S
```

## Foundation validation matrix (closure baseline)
The following command matrix is required for foundation closure and must pass together:

| Gate | Command | Purpose |
|---|---|---|
| Dependency direction | `bash scripts/check_dependency_direction.sh` | Enforce one-way dependency posture (`engine -> game` reverse imports forbidden). |
| ECS direct access | `bash scripts/check_ecs_direct_access.sh` | Prevent unauthorized direct ECS access paths outside approved boundaries. |
| Workspace health | `cargo check --workspace` | Confirm all workspace crates compile under current dependency and ownership wiring. |

## Canonical dependency and ownership coverage table
| Concern | Canonical owner | Enforcement / evidence |
|---|---|---|
| Crate dependency direction | Architecture law docs (`ENGENE_2_0_DEPENDENCY_LAW.md`, `ENGENE_2_0_ARCHITECTURE.md`) | `scripts/check_dependency_direction.sh` gate. |
| Runtime role boundaries | Runtime baseline (`ENGENE_2_0_RUNTIME_DEPENDENCY_BASELINE.md`) | Assembly policy + import discipline in `src/bin/*` and tests. |
| Product ownership boundaries | Product law (`ENGENE_2_0_PRODUCT_BOUNDARIES.md`) | Canonical role split (Game / Headless / Tools), no reverse product absorption. |
| ECS authority boundaries | ECS boundary policy + allowlist | `scripts/check_ecs_direct_access.sh` + `scripts/ecs_direct_access_allowlist.txt`. |
| Entrypoint ownership | Entrypoints law (`ENGENE_2_0_ENTRYPOINTS.md`) | Canonical launch commands mapped to role-specific bootstrap assemblies. |

## Game module ownership classification (current)
| Module | Classification | Rationale |
|---|---|---|
| `game::ai`, `game::ecosystem`, `game::gameplay`, `game::player`, `game::economy`, `game::*_plugin` | KEEP in game | Game-authored simulation/content logic and game plugin contracts. |
| `game::runtime_world_tick`, `game::integration_systems`, `game::animation_integration`, `game::runtime_background` | MOVED to runtime wiring | Runtime orchestration glue, not game-owned domain API. |
| `game::debug_output` | MOVED to app | App-facing HUD/debug print utility, not canonical game domain contract. |

## Runtime bootstrap file ownership (current)
| File | Role ownership |
|---|---|
| `src/runtime/bootstrap/headless.rs` | kernel/headless bootstrap only |
| `src/runtime/bootstrap/tools.rs` | tools/sdk bootstrap only |
| `src/runtime/bootstrap/game.rs` | game bootstrap only |
| `src/runtime/bootstrap/common.rs` | shared bootstrap primitives only |
| `src/runtime/wiring/*` | runtime glue systems (non-game domain API surface) |

## Major module/dependency buckets (current classification)
| Bucket | Classification |
|---|---|
| `core::*`, scheduler/events/runtime config | KEEP in kernel/engine core |
| `runtime/bootstrap/headless.rs` | KEEP in kernel bootstrap |
| `runtime/bootstrap/tools.rs` + `tools::*` | KEEP in tools-only |
| `runtime/bootstrap/game.rs` + `game::*` domain plugins/content | KEEP in game-only |
| `runtime/wiring/*` | KEEP in engine simulation layer (runtime glue), split further later |
| legacy runtime facade aliases | DELETE legacy |

## Remaining monolith map / next split targets
| Zone | Why still monolithic | Acceptable now | Blocker | Next split target |
|---|---|---|---|---|
| `runtime/bootstrap/game_resources.rs` | large game resource registration map | yes (short-term) | broad ownership of world/audio/physics resources in one constructor | split by resource domains (`world`, `physics`, `audio`) |
| `app/game_runner/app_loop.rs` | input/render/runtime orchestration in one file | yes (short-term) | loop state + render/input logic co-located | split into `loop_state.rs` + `frame_tick.rs` |
| `app/sdk_runner.rs` | editor shell + sim tick + render event loop | yes (short-term) | UI/runtime orchestration coupled | split into `sdk_loop.rs` + `sdk_startup.rs` |
| `tests/runtime_systems/category_*.rs` | category files still broad inside bucket | partially | historical accumulation and macro-scale integration assertions | tighten further by runtime role + subsystem contracts |
