/// ENGENE Integration Matrix
///
/// Formal mapping of world-state transitions between systems.
/// Each row = event type. Columns = subsystem roles.
///
/// Legend:
///   Class A = Authoritative (changes world truth, replay-relevant)
///   Class B = Consequence (presentation/reaction, can batch/drop)
///
/// Delivery Semantics:
///   MD = must_deliver
///   CB = can_batch
///   CD = can_drop_under_pressure
///   ST = sticky
///   RR = replay_relevant
///
/// State Transition Ownership:
///   Each authoritative event has exactly ONE legal producer (no shortcuts).
///
/// ┌──────────────────────┬───────┬──────────────────────────┬───────────────────────────────────────────────┬─────────┬─────────────┐
/// │ Event                │ Class │ Publisher (exclusive)     │ Consumers                                     │ Sync    │ Delivery    │
/// ├──────────────────────┼───────┼──────────────────────────┼───────────────────────────────────────────────┼─────────┼─────────────┤
/// │ ImpactEvent          │ A     │ BallisticsTickSystem      │ DamageDispatch, DestructionTick               │ deferred│ MD, RR      │
/// │ StructuralCollapse   │ A     │ DestructionTickSystem     │ OcclusionWire, Audio, NavDirty                │ deferred│ MD, RR      │
/// │ TerrainDeformed      │ A     │ DamageDispatchSystem      │ TerrainDeformationTick                        │ deferred│ MD, RR      │
/// │ TerrainChanged       │ A     │ TerrainDeformationTick    │ NavDirtyTick                                  │ deferred│ MD, RR      │
/// │ NavUpdated           │ A     │ NavDirtyTickSystem        │ AI (path replanning)                          │ deferred│ MD          │
/// │ WorldTopologyChanged │ A     │ DestructionTickSystem     │ NavDirty, CoverRefresh, AI, Occlusion         │ deferred│ MD, RR      │
/// │ EntityDied           │ A     │ AiSystem (combat)         │ Economy, Ecosystem, AI(witness)               │ deferred│ MD, RR, ST  │
/// │ BodyZoneDamaged      │ A     │ DamageDispatchSystem      │ GoreWire, AI(witness), Audio                  │ deferred│ MD, RR      │
/// │ FireSpreadEvent      │ A     │ FireGrid (PhysicsSystem)  │ TerrainDeformation, SurfaceState              │ deferred│ MD, RR      │
/// │ CoverChanged         │ A     │ NavDirtyTickSystem        │ AI (tactics)                                  │ deferred│ MD          │
/// │ SurfaceDamaged       │ B     │ DamageDispatchSystem      │ SurfaceStateStore, Audio, Render              │ batched │ CB, CD      │
/// │ DecalSpawn           │ B     │ DamageDispatchSystem      │ Render                                        │ batched │ CB, CD      │
/// │ ParticleBurst        │ B     │ DamageDispatch/Resolvers  │ Render                                        │ batched │ CB, CD      │
/// │ GoreMeshSpawn        │ B     │ GoreWireSystem            │ Render                                        │ batched │ CB, CD      │
/// │ SoundTrigger         │ B     │ BallisticsTick + Dispatch │ AudioIntegration                              │ batched │ CB, CD      │
/// └──────────────────────┴───────┴──────────────────────────┴───────────────────────────────────────────────┴─────────┴─────────────┘
///
/// CONSEQUENCE CHAIN (verified connected):
///
/// Shot fired
///   → BallisticsTick emits ImpactEvent
///     → DamageDispatch emits TerrainDeformed / BodyZoneDamaged / SurfaceDamaged / SoundTrigger
///       → DestructionTick emits WorldTopologyChanged / StructuralCollapse
///         → NavDirtyTick emits NavUpdated / CoverChanged
///         → OcclusionWire registers breach (AI/audio can query)
///       → TerrainDeformationTick emits TerrainChanged
///         → NavDirtyTick emits NavUpdated / CoverChanged
///       → GoreWire emits GoreMeshSpawn
///       → AudioIntegration plays spatial sound with occlusion
///
/// NO CONSUMER MAY INFER MISSING UPSTREAM TRUTH.
/// All downstream consumers receive explicit events.

pub struct IntegrationEntry {
    pub event_name: &'static str,
    pub class: EventClass,
    pub publisher: &'static str,
    pub consumers: &'static [&'static str],
    pub sync_mode: SyncMode,
    pub must_deliver: bool,
    pub can_batch: bool,
    pub can_drop: bool,
    pub sticky: bool,
    pub replay_relevant: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum EventClass { Authoritative, Consequence }

#[derive(Debug, Clone, Copy)]
pub enum SyncMode { Deferred, Batched }

pub fn integration_matrix() -> Vec<IntegrationEntry> {
    vec![
        IntegrationEntry {
            event_name: "ImpactEvent",
            class: EventClass::Authoritative,
            publisher: "BallisticsTickSystem",
            consumers: &["DamageDispatchSystem", "DestructionTickSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "StructuralCollapse",
            class: EventClass::Authoritative,
            publisher: "DestructionTickSystem",
            consumers: &["OcclusionWireSystem", "AudioIntegrationSystem", "NavDirtyTickSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "TerrainDeformed",
            class: EventClass::Authoritative,
            publisher: "DamageDispatchSystem",
            consumers: &["TerrainDeformationTickSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "TerrainChanged",
            class: EventClass::Authoritative,
            publisher: "TerrainDeformationTickSystem",
            consumers: &["NavDirtyTickSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "NavUpdated",
            class: EventClass::Authoritative,
            publisher: "NavDirtyTickSystem",
            consumers: &["AiSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: false,
        },
        IntegrationEntry {
            event_name: "WorldTopologyChanged",
            class: EventClass::Authoritative,
            publisher: "DestructionTickSystem",
            consumers: &["NavDirtyTickSystem", "CoverRefresh", "AiSystem", "OcclusionWireSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "EntityDied",
            class: EventClass::Authoritative,
            publisher: "AiSystem",
            consumers: &["EconomySystem", "EcosystemSystem", "AiSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: true, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "BodyZoneDamaged",
            class: EventClass::Authoritative,
            publisher: "DamageDispatchSystem",
            consumers: &["GoreWireSystem", "AiSystem", "AudioIntegrationSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "FireSpreadEvent",
            class: EventClass::Authoritative,
            publisher: "PhysicsSystem",
            consumers: &["TerrainDeformationTickSystem", "SurfaceState"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: true,
        },
        IntegrationEntry {
            event_name: "CoverChanged",
            class: EventClass::Authoritative,
            publisher: "NavDirtyTickSystem",
            consumers: &["AiSystem"],
            sync_mode: SyncMode::Deferred,
            must_deliver: true, can_batch: false, can_drop: false, sticky: false, replay_relevant: false,
        },
        IntegrationEntry {
            event_name: "SurfaceDamaged",
            class: EventClass::Consequence,
            publisher: "DamageDispatchSystem",
            consumers: &["SurfaceStateStore", "AudioIntegrationSystem", "Renderer"],
            sync_mode: SyncMode::Batched,
            must_deliver: false, can_batch: true, can_drop: true, sticky: false, replay_relevant: false,
        },
        IntegrationEntry {
            event_name: "GoreMeshSpawn",
            class: EventClass::Consequence,
            publisher: "GoreWireSystem",
            consumers: &["Renderer"],
            sync_mode: SyncMode::Batched,
            must_deliver: false, can_batch: true, can_drop: true, sticky: false, replay_relevant: false,
        },
        IntegrationEntry {
            event_name: "SoundTrigger",
            class: EventClass::Consequence,
            publisher: "BallisticsTickSystem/DamageDispatchSystem",
            consumers: &["AudioIntegrationSystem"],
            sync_mode: SyncMode::Batched,
            must_deliver: false, can_batch: true, can_drop: true, sticky: false, replay_relevant: false,
        },
    ]
}
