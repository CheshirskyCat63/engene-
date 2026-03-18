# ENGENE 2.0 Minimal Engine Stack (Frozen Baseline)

## Purpose
Freeze the minimal engine stack so future repo split work can proceed without role ambiguity.

## Runtime roles (already frozen)
- **Kernel/Headless**
- **Tools/SDK**
- **Game**

No additional runtime role is valid.
No sandbox/demo runtime identity is valid.

## Minimal engine kernel definition
### MUST exist in kernel
1. Math primitives and deterministic numeric helpers.
2. ECS identity/storage/query contracts.
3. Resource storage and typed runtime registry.
4. Scheduler/system-order contract layer.
5. Event/message buses and deterministic dispatch contracts.
6. Fixed-tick timing and runtime manifest/config identity.
7. Deterministic invariant/diagnostic base contracts.

### MAY exist only in engine simulation layer (not kernel)
- Simulation state transition modules.
- Physics-state progression modules.
- Derived topology maintenance (e.g. nav dirty propagation).
- World-state mutation systems that are content-agnostic.

### Tools/SDK only
- Editor UI shell/panels.
- Runtime doctor command/UI and tooling dashboards.
- Tooling convenience bootstraps.

### Game only
- Authored content bootstrap (`game/data`, `GameConfig` loading from authored paths).
- Game plugins/rules/content coupling.
- Presentation/gameplay orchestration specific to product scenario.

### Explicitly forbidden in kernel
- Authored game bootstrap.
- Gameplay plugin/rule ownership.
- Demo/sandbox role identity.
- Renderer-owned truth.
- Editor/tool UI ownership.
- Hidden content loading.

## Dependency direction
- Kernel must not require game bootstrap.
- Kernel must not require tools bootstrap.
- Tools may depend on kernel + tools modules, but not game bootstrap.
- Game may depend on kernel + game modules and optional projections.

## Dependency/module classification inventory
| Item | Why it exists | Owner role | Kernel runnable without it? | Tools runnable without it? | Game needs it? | Classification |
|---|---|---:|---:|---:|---:|---|
| `core::ecs`, `core::engine`, `core::events`, `core::scheduler`, query contracts | State compute + orchestration core | Kernel | No | No | Yes | KEEP in engine kernel |
| `core::runtime_config`, `core::runtime_manifest`, fixed-tick runtime identity | Role/tick config contract | Kernel | No | No | Yes | KEEP in engine kernel |
| `core::determinism_*`, invariant contracts | Deterministic safety baseline | Kernel | No | No | Yes | KEEP in engine kernel |
| `simulation::*` activation/demotion orchestration | Runtime progression policy | Engine sim layer | Yes | Yes | Yes | KEEP in engine simulation layer |
| `physics::physics` core rigid-body stepping | Simulation progression | Engine sim layer | Yes | Yes | Yes | KEEP in engine simulation layer |
| `physics::ballistics` | Projectile simulation | Engine sim layer | Yes | Yes | Yes (current product) | KEEP in engine simulation layer |
| `physics::destruction` | Destructible-state simulation | Engine sim layer | Yes | Yes | Yes (current product) | KEEP in engine simulation layer |
| `physics::fire` / `physics::water` | Environmental simulation state | Engine sim layer | Yes | Yes | Yes (current product) | KEEP in engine simulation layer |
| `world::fields`, `surface_state` | Cross-system world/environment state | Engine sim layer | Yes | Yes | Yes | KEEP in engine simulation layer |
| `navigation::*` + nav dirty derived topology | Runtime pathing topology updates | Engine sim layer | Yes | Yes | Yes | KEEP in engine simulation layer |
| `graphics::renderer` + sky/weather stack | Projection/presentation | Projection layer | Yes | Yes | Yes | MOVE later (projection split target) |
| `audio::*` spatial playback | Projection/presentation | Projection layer | Yes | Yes | Yes | MOVE later (projection split target) |
| `tools::doctor`, `tools::editor_shell`, dashboards | Tooling diagnostics and UI | Tools | Yes | No | No | KEEP in tools/sdk only |
| `game::*` plugins, economy/content glue | Product game ownership | Game | Yes | Yes | No (kernel/tools) | KEEP in game only |
| Authored data loading via `game/data` | Product authored bootstrap | Game | Yes | Yes | No (kernel/tools) | KEEP in game only |
| Legacy archive docs (`docs/archive/*`) | Historical tail with contradictory runtime model | None | Yes | Yes | Yes | DELETE as obsolete |
| Legacy term traces in tests/docs strings | Migration residue in names/text | Mixed | Yes | Yes | Yes | MOVE later / text cleanup backlog |

## Big subsystem role classification
| Subsystem | Kernel? | Engine sim layer? | Tools? | Game? | Projection only? | Future split target |
|---|---:|---:|---:|---:|---:|---|
| Math | ✅ | ✅ | ✅ | ✅ | ❌ | Kernel core |
| ECS/query/runtime config/scheduler/events | ✅ | ✅ | ✅ | ✅ | ❌ | Kernel core |
| Physics core | ❌ | ✅ | ⚠️ (indirect test/use) | ✅ | ❌ | Engine sim |
| Ballistics | ❌ | ✅ | ⚠️ (diagnostics/tests) | ✅ | ❌ | Engine sim (or game-policy hooks split later) |
| Destruction | ❌ | ✅ | ⚠️ (diagnostics/tests) | ✅ | ❌ | Engine sim |
| Fire | ❌ | ✅ | ⚠️ | ✅ | ❌ | Engine sim |
| Water | ❌ | ✅ | ⚠️ | ✅ | ❌ | Engine sim |
| Cloth | ❌ | ✅ | ⚠️ | ✅ | ❌ | Engine sim |
| World fields/weather state | ❌ | ✅ | ⚠️ | ✅ | ❌ | Engine sim |
| Surface state/wetness | ❌ | ✅ | ⚠️ | ✅ | ❌ | Engine sim |
| Nav dirty / derived topology | ❌ | ✅ | ⚠️ | ✅ | ❌ | Engine sim |
| Renderer/sky/weather prototype stack | ❌ | ❌ | ⚠️ (tool visualization) | ✅ | ✅ | Projection split |
| Audio/spatial audio | ❌ | ❌ | ⚠️ (tool diagnostics) | ✅ | ✅ | Projection split |
| Tools/doctor/SDK | ❌ | ❌ | ✅ | ❌ | ❌ | Tools package |
| Game config/plugins/content bootstrap | ❌ | ❌ | ❌ | ✅ | ❌ | Game package |
| Bins/runners | ❌ | ❌ | role-owned entry layer | role-owned entry layer | ❌ | App package split |

## Suspicious tails (current snapshot)
1. Some tests still carry legacy naming language in function names/messages (not architecture ownership, but migration residue).
2. Renderer weather controller remains in graphics projection stack; must stay out of kernel/tools ownership boundaries.
3. `LowSpec*` naming remains in perf/certification modules and cook variants; this is not a runtime-role concept, but a naming cleanup candidate.

## Split safety rule
No module may move to kernel if it requires authored game bootstrap, UI toolkit ownership, or renderer/audio projection truth.
