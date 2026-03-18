# Sandbox Runtime Proof — Destruction Sandbox 50x50

## Active Systems in Sandbox Runtime

Systems registered in `RuntimeAssembly::sandbox_50x50()`:

- SimulationSystem
- PhysicsSystem
- BallisticsTickSystem
- DamageDispatchSystem
- DestructionTickSystem
- TerrainDeformationTickSystem
- NavDirtyTickSystem
- OcclusionWireSystem
- GoreWireSystem
- AnimationIntegrationSystem
- AnimationWireSystem
- AudioIntegrationSystem
- BodySystem
- RenderSystem
- AudioPlaybackBridge
- InputActionSystem

**Plugins:** CombatPlugin

**Not active (sandbox-specific):** QuestSystem, NetworkSystem, AiSystem, EconomyPlugin, PopulationPlugin, WorldTickSystem

---

## Event Chains Verified

### Ballistics
`fire()` → ShotFired → BallisticsTickSystem → DamageDispatch → material-specific response

### Destruction
DamageDispatch → DestructionTickSystem → LinkBroken/ObjectFragmented → WorldTopologyChanged → NavDirtyTick

### Terrain
DamageDispatch → TerrainDeformed → TerrainDeformationTick → TerrainChanged → NavDirtyTick → CoverChanged

### Audio
SoundTrigger → AudioIntegrationSystem (with destruction occlusion + reverb zones) → AudioEngine.play_3d

---

## Nav/Render/Audio/Persistence Impact

| Domain      | Impact |
|------------|--------|
| **Nav**    | NavDirtyTracker marks dirty cells on topology and terrain changes |
| **Render** | RenderSystem with cascaded shadows (4 cascades, dynamic-only), ContactShadowPass (wired dormant) |
| **Audio**  | OcclusionSystem reverb zones wired to AudioIntegrationSystem, destruction occlusion attenuates sounds |
| **Persistence** | ChunkPersistenceService handles save/load with terrain patches, entity snapshots |

---

## Low-spec Reductions

- **Reduced:** particle count, shadow resolution, post-process quality
- **Preserved:** destruction truth, terrain deformation patches, entity persistence state, nav/cover dirty propagation

---

## Budget Summary (template)

| Budget      | Target |
|------------|--------|
| Frame budget target | 16.6ms (60 fps) |
| Physics budget      | X ms |
| Render budget       | X ms |
| Audio budget        | X ms |
| Nav rebuild budget  | X ms |
| Memory              | X MB |
