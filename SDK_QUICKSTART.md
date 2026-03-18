# ENGENE SDK — Quick Start

## Canonical launch

```bash
cargo run --bin engene_sdk
```

Optional helper:

```powershell
.\run_sdk.ps1
```

For canonical entrypoint law see:
- `docs/canonical/ENGENE_2_0_ENTRYPOINTS.md`
- `README_FIRST_RUN.md`

## Controls

| Key | Action |
|-----|--------|
| **Click** | Capture mouse for camera |
| **ESC** | Release mouse / quit |
| **WASD** | Fly camera |
| **Mouse** | Look around (when captured) |
| **P** | Pause / resume simulation |
| **]** | Speed up simulation (2x) |
| **[** | Slow down simulation (0.5x) |

## Panels

The SDK opens with 17+ editor panels visible via egui:

- **SceneHierarchy** — entity tree, selection, filtering
- **Inspector** — component viewer/editor for selected entity
- **Doctor** — diagnostic checks (strict mode by default)
- **RuntimeTruth** — live system status dashboard
- **Profiler** — frame budget and system timing
- **EventMonitor** — event bus activity
- **PersistenceDashboard** — chunk save/load status
- **CrashViewer** — crash log browser
- **WorldMap** — top-down world overview
- **Console** — command execution (spawn, teleport, kill, etc.)
- **TimeControls** — day/night, speed, pause
- **QuestBoard** — active quest tracking
- **EconomyDashboard** — trade metrics, wealth distribution
- **SimMetricsDashboard** — population, entity counts
- **ReplayBrowser** — replay recording and playback
- **AssetBrowser** — content filtering and preview
- **EditorSafeMode** — panel crash isolation
