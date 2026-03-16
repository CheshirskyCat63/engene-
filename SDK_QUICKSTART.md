# ENGENE SDK — Quick Start

## Launch

```powershell
.\run_sdk.ps1
# or
ENGENE_SDK.exe
# or
cargo run --bin engene_sdk --features sdk_tools
```

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

## Core Workflows

1. **Open world** — world loads automatically at startup
2. **Select entity** — click in SceneHierarchy or world view
3. **Inspect** — selected entity details appear in Inspector
4. **Edit component** — modify fields in Inspector panel
5. **Place entity** — use Console: `spawn npc 100 50`
6. **Save chunk** — PersistenceDashboard or Console: `save_chunk 0 0`
7. **Reload chunk** — Console: `reload_chunk 0 0`

## Doctor

Doctor runs in Strict mode at SDK startup. Check Doctor panel for:
- Identity/authority contract violations
- Orphan references
- Missing low-spec policies
- System registration issues

Zero errors expected for a clean build.
