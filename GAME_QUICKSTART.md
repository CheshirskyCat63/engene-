# ENGENE Game — Quick Start

## Canonical launch

```bash
cargo run --bin engene_game
```

Optional helper:

```powershell
.\run_game.ps1
```

For canonical entrypoint law see:
- `docs/canonical/ENTRYPOINT_TRUTH.md`
- `README_FIRST_RUN.md`

## Controls

| Key | Action |
|-----|--------|
| **Click** | Capture mouse |
| **ESC** | Release mouse / quit |
| **WASD** | Move |
| **Shift** | Sprint (drains stamina) |
| **Mouse** | Look around |
| **E** | Interact with nearby NPC |
| **R** | Respawn (when dead) |
| **F5** | Quick save |
| **F9** | Quick load |

## Gameplay

- You spawn at the **Rookie Camp** (500, 50, 500)
- NPCs with names appear as blue entities — approach and press **E** to interact
- Monsters roam the world: wolves (white), boars (brown), bloodsuckers (red)
- The HUD shows health, stamina, bleed status, and interaction prompts
- Day/night cycle and seasons affect NPC behavior and monster activity

## Save System

- **F5** saves to `game/saves/quicksave.json`
- **F9** loads from the same file
- Player position, health, stamina, inventory, and reputation are preserved
- On death, press **R** to respawn at Rookie Camp

## World

The game loads a 2x2 km world slice with:
- Multiple biomes (plains, forest, swamp, hills)
- NPC camps with traders, guards, hunters
- Monster habitats with wolves, boars, bloodsuckers
- Chunk streaming based on camera position
- Persistent entity state across chunk load/unload cycles
