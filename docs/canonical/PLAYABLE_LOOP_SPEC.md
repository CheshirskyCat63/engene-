# PLAYABLE_LOOP_SPEC.md — Golden Scenario

## Player Start

- Location: Rookie Camp (500, 500)
- Health: 100%, Stamina: 100%
- Equipment: basic pistol, 2 medkits, 30 rounds
- Money: $500

## Step 1: Talk to Trader (Sidorovich)

- Walk to Sidorovich at (510, 490)
- Interaction: approach + press E
- Opens trade/quest dialogue

## Step 2: Accept First Quest

- Quest: "Clear the Wolf Den" or "Retrieve artifact from Vortex Field"
- Quest type: kill/fetch
- Reward: $300-500 + reputation
- Marker appears on HUD compass/minimap

## Step 3: Travel to Danger Zone

- Route: Main Road east or Forest Path south
- Expected travel time: 2-3 minutes real-time
- Encounters along the way: boars on Forest Path, random stalker patrols on Main Road

## Step 4: Combat

- Wolf Den (700, 1500): 3-5 wolves, daylight encounter
- Or Vortex Field (1400, 600): anomaly navigation + possible mutant
- Expected pattern: 2-4 enemies, player uses cover, fires weapon, takes damage
- Body system: hits cause pain, bleeding if limb damage

## Step 5: Injury

- Player takes damage during combat
- Bleeding status if hit in limb zone
- Use medkit to heal (inventory -> use item)

## Step 6: Loot

- Loot wolf carcasses or retrieve artifact
- Items go to player inventory
- Corpse persistence: bodies remain on map

## Step 7: Return to Camp

- Travel back to Rookie Camp
- Encounter risk: monsters may have shifted during travel time

## Step 8: Trade

- Sell loot/artifact to Sidorovich
- Buy ammo/medkits for next run
- Trader prices reflect supply/demand

## Step 9: Complete Quest

- Deliver quest objective to Sidorovich
- Receive reward (money + faction reputation)
- New quest available

## Step 10: Save

- Player saves game
- All state persisted: position, inventory, quest progress, faction rep
- World state persisted: NPC positions, camp state, corpses, destruction

## Step 11: Load

- Exit and reload save
- All state restored accurately
- Diff-verify: no critical state lost

## Step 12: Continue

- Player continues from saved state
- World has advanced (if time passed): NPCs moved, economy shifted
- New quests available

## Fail State

- Player dies: respawn at last camp, lose some inventory
- Death screen with option to reload last save
- Corpse remains at death location (lootable by NPCs)

## Success Criteria

- Full loop takes 10-15 minutes real-time
- No crashes or state loss
- Save/load roundtrip clean
- Economy observably changes over multiple loops
