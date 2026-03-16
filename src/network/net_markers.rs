/// Network-awareness markers for ECS state categories.
/// These are declarative annotations -- they don't implement networking,
/// but define the contract for future replication/prediction systems.

/// Who owns and is allowed to mutate a piece of state in a networked context
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetAuthority {
    /// Server is the single source of truth; clients receive updates
    ServerAuthoritative,
    /// Client that owns this entity can predict; server corrects
    OwnerPredicted,
    /// Any client can write (cosmetic-only, no gameplay impact)
    ClientLocal,
    /// Not yet classified for networking
    Unassigned,
}

/// How frequently this state needs to be replicated
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetPriority {
    /// Every server tick (position, health)
    EveryTick,
    /// When changed (inventory, job, group membership)
    OnChange,
    /// Rarely (traits, kind -- only on spawn/major event)
    Rare,
    /// Never sent over network (render state, local debug)
    Never,
}

/// How large the serialized payload typically is
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetPayloadSize {
    Tiny,   // < 16 bytes (position, single scalar)
    Small,  // 16-128 bytes (needs struct, inventory)
    Medium, // 128-1024 bytes (memory, complex state)
    Large,  // > 1KB (full snapshot, terrain patch)
}

/// Complete network annotation for a state category
#[derive(Clone, Debug)]
pub struct NetStateMarker {
    pub state_name: &'static str,
    pub authority: NetAuthority,
    pub priority: NetPriority,
    pub payload_size: NetPayloadSize,
    pub interpolatable: bool,
    pub description: &'static str,
}

pub fn net_state_markers() -> Vec<NetStateMarker> {
    vec![
        NetStateMarker {
            state_name: "Transform",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::EveryTick,
            payload_size: NetPayloadSize::Tiny,
            interpolatable: true,
            description: "Position/rotation. Server-auth, client interpolates.",
        },
        NetStateMarker {
            state_name: "PersonalNeeds",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Small,
            interpolatable: false,
            description: "Health/hunger/thirst. Server-auth, sent on change.",
        },
        NetStateMarker {
            state_name: "AiState",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Tiny,
            interpolatable: false,
            description: "Current AI behavior state. Server only.",
        },
        NetStateMarker {
            state_name: "Inventory",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Small,
            interpolatable: false,
            description: "Items. Server-auth, sent to owner + nearby.",
        },
        NetStateMarker {
            state_name: "NpcEconomy",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Small,
            interpolatable: false,
            description: "Money/job. Server-auth.",
        },
        NetStateMarker {
            state_name: "EntityKind",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::Rare,
            payload_size: NetPayloadSize::Tiny,
            interpolatable: false,
            description: "NPC/Monster type. Sent once on spawn.",
        },
        NetStateMarker {
            state_name: "BodyState",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Medium,
            interpolatable: false,
            description: "Damage zones, bleed. Server-auth, cosmetic on client.",
        },
        NetStateMarker {
            state_name: "GroupMembership",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Small,
            interpolatable: false,
            description: "Group leader/members. Server-auth.",
        },
        NetStateMarker {
            state_name: "Emotions",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Small,
            interpolatable: false,
            description: "Emotional state. Server-only, visual hints to client.",
        },
        NetStateMarker {
            state_name: "Memory",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::Never,
            payload_size: NetPayloadSize::Large,
            interpolatable: false,
            description: "AI memory. Server-only, never replicated.",
        },
        NetStateMarker {
            state_name: "SimLevel",
            authority: NetAuthority::ClientLocal,
            priority: NetPriority::Never,
            payload_size: NetPayloadSize::Tiny,
            interpolatable: false,
            description: "LOD level. Per-client, never replicated.",
        },
        NetStateMarker {
            state_name: "SpatialIndex",
            authority: NetAuthority::ClientLocal,
            priority: NetPriority::Never,
            payload_size: NetPayloadSize::Large,
            interpolatable: false,
            description: "Derived spatial data. Never replicated.",
        },
        NetStateMarker {
            state_name: "RenderInstances",
            authority: NetAuthority::ClientLocal,
            priority: NetPriority::Never,
            payload_size: NetPayloadSize::Large,
            interpolatable: false,
            description: "GPU data. Client-local.",
        },
        NetStateMarker {
            state_name: "DestructionTopology",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::OnChange,
            payload_size: NetPayloadSize::Medium,
            interpolatable: false,
            description: "Structural state. Server-auth, chunk-scoped.",
        },
        NetStateMarker {
            state_name: "SurfaceState",
            authority: NetAuthority::ClientLocal,
            priority: NetPriority::Never,
            payload_size: NetPayloadSize::Medium,
            interpolatable: false,
            description: "Blood/burn marks. Cosmetic, client-local.",
        },
        NetStateMarker {
            state_name: "PersistentEntityId",
            authority: NetAuthority::ServerAuthoritative,
            priority: NetPriority::Rare,
            payload_size: NetPayloadSize::Tiny,
            interpolatable: false,
            description: "Stable identity. Assigned by server, sent on spawn.",
        },
    ]
}

pub fn net_report() -> String {
    let markers = net_state_markers();
    let mut report = String::new();
    report.push_str("=== Network State Markers Report ===\n");
    report.push_str(&format!("Total annotated states: {}\n", markers.len()));

    let server_auth = markers.iter().filter(|m| m.authority == NetAuthority::ServerAuthoritative).count();
    let client_local = markers.iter().filter(|m| m.authority == NetAuthority::ClientLocal).count();
    let every_tick = markers.iter().filter(|m| m.priority == NetPriority::EveryTick).count();
    let never = markers.iter().filter(|m| m.priority == NetPriority::Never).count();

    report.push_str(&format!("  Server-authoritative: {}\n", server_auth));
    report.push_str(&format!("  Client-local: {}\n", client_local));
    report.push_str(&format!("  Replicated every tick: {}\n", every_tick));
    report.push_str(&format!("  Never replicated: {}\n", never));

    for m in &markers {
        report.push_str(&format!(
            "\n  [{}] {} — {:?} / {:?} / {:?}{}",
            m.state_name,
            m.description,
            m.authority,
            m.priority,
            m.payload_size,
            if m.interpolatable { " [INTERP]" } else { "" }
        ));
    }
    report
}
