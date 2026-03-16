/// Classification of entity fields for L0 <-> L2 transition
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldPreservation {
    /// Must be preserved exactly across transition (bit-identical)
    PreserveExact,
    /// Can be aggregated/summarized in lower-fidelity sim
    Aggregate,
    /// Recomputed when promoted back to L0
    Recompute,
    /// Forbidden to change in background sim
    Frozen,
}

/// Describes how a specific entity field behaves across simulation level transitions
#[derive(Clone, Debug)]
pub struct FieldInvariant {
    pub field_name: &'static str,
    pub component: &'static str,
    pub preservation: FieldPreservation,
    pub description: &'static str,
}

/// Complete invariant table for L0 <-> L2 transitions
pub fn field_invariant_table() -> Vec<FieldInvariant> {
    vec![
        // PRESERVE EXACT -- core identity and state
        FieldInvariant {
            field_name: "persistent_id",
            component: "PersistentEntityId",
            preservation: FieldPreservation::PreserveExact,
            description: "Stable identity. Never changes.",
        },
        FieldInvariant {
            field_name: "position (x, y, cell)",
            component: "Transform",
            preservation: FieldPreservation::PreserveExact,
            description: "World position. Background sim updates this for migration.",
        },
        FieldInvariant {
            field_name: "kind",
            component: "EntityKind",
            preservation: FieldPreservation::PreserveExact,
            description: "NPC or Monster type. Immutable.",
        },
        FieldInvariant {
            field_name: "health",
            component: "PersonalNeeds",
            preservation: FieldPreservation::PreserveExact,
            description: "Core health value preserved across transitions.",
        },
        FieldInvariant {
            field_name: "hunger, thirst, energy",
            component: "PersonalNeeds",
            preservation: FieldPreservation::PreserveExact,
            description: "Survival needs. Background sim applies simplified decay.",
        },
        FieldInvariant {
            field_name: "money",
            component: "NpcEconomy",
            preservation: FieldPreservation::PreserveExact,
            description: "Currency. Background sim may add/subtract via offline_npc_work.",
        },
        FieldInvariant {
            field_name: "job",
            component: "NpcEconomy",
            preservation: FieldPreservation::PreserveExact,
            description: "Current job assignment. Does not change in background sim.",
        },
        FieldInvariant {
            field_name: "npc_traits / monster_traits",
            component: "NpcTraits / MonsterTraits",
            preservation: FieldPreservation::PreserveExact,
            description: "Personality traits. Immutable after spawn.",
        },
        FieldInvariant {
            field_name: "social_needs (family, reputation, friendship)",
            component: "SocialNeeds",
            preservation: FieldPreservation::PreserveExact,
            description: "Social state preserved. No social events in background sim.",
        },
        FieldInvariant {
            field_name: "life_info (age, max_age)",
            component: "LifeInfo",
            preservation: FieldPreservation::PreserveExact,
            description: "Age advances in background sim. Max age immutable.",
        },
        // AGGREGATE -- simplified in background
        FieldInvariant {
            field_name: "ai_state (goal, sub_state)",
            component: "AiState",
            preservation: FieldPreservation::Aggregate,
            description: "Full AI state summarized to goal + sub-goal in L2.",
        },
        FieldInvariant {
            field_name: "memory.entities (opinions)",
            component: "Memory",
            preservation: FieldPreservation::Aggregate,
            description: "Entity opinions preserved but not updated in L2.",
        },
        FieldInvariant {
            field_name: "emotions",
            component: "Emotions",
            preservation: FieldPreservation::Aggregate,
            description: "Emotion values decay toward baseline in background sim.",
        },
        FieldInvariant {
            field_name: "plan",
            component: "Plan",
            preservation: FieldPreservation::Aggregate,
            description: "Current plan discarded on promotion. AI recalculates at L0.",
        },
        // RECOMPUTE -- rebuilt on promotion to L0
        FieldInvariant {
            field_name: "spatial_index entry",
            component: "SpatialIndex",
            preservation: FieldPreservation::Recompute,
            description: "Rebuilt from position when entity enters L0.",
        },
        FieldInvariant {
            field_name: "sim_level",
            component: "SimLevel",
            preservation: FieldPreservation::Recompute,
            description: "Recalculated from camera distance every tick.",
        },
        FieldInvariant {
            field_name: "perception_cache",
            component: "AiSystem",
            preservation: FieldPreservation::Recompute,
            description: "Perception results recomputed at L0 from scratch.",
        },
        FieldInvariant {
            field_name: "render_instance",
            component: "Renderer",
            preservation: FieldPreservation::Recompute,
            description: "GPU instance data created fresh when entity is visible.",
        },
        // FROZEN -- forbidden to change in background sim
        FieldInvariant {
            field_name: "inventory items",
            component: "Inventory",
            preservation: FieldPreservation::Frozen,
            description: "No item transactions in background sim. Only at L0.",
        },
        FieldInvariant {
            field_name: "body_state (zone HP, joints)",
            component: "BodyState",
            preservation: FieldPreservation::Frozen,
            description: "No detailed body damage in background sim.",
        },
        FieldInvariant {
            field_name: "group_membership",
            component: "Groups",
            preservation: FieldPreservation::Frozen,
            description: "No group formation/dissolution in background sim.",
        },
    ]
}

/// Actions explicitly forbidden in background simulation (L2/L3)
pub fn forbidden_background_actions() -> Vec<&'static str> {
    vec![
        "Reproduction (creating new entities)",
        "Forming new social ties",
        "Joining or leaving groups",
        "Detailed combat resolution",
        "Inventory transactions",
        "Body zone damage",
        "Creating new memories about specific entities",
        "Changing job assignment",
        "Death from detailed simulation (only from simplified rules)",
    ]
}

/// Event summary contract: when entity returns from L2 to L0
#[derive(Clone, Debug)]
pub struct BackgroundEventSummary {
    pub money_delta: f32,
    pub hunger_delta: f32,
    pub energy_delta: f32,
    pub distance_traveled: f32,
    pub health_delta: f32,
    pub ticks_in_background: u64,
}

impl BackgroundEventSummary {
    pub fn empty() -> Self {
        Self {
            money_delta: 0.0,
            hunger_delta: 0.0,
            energy_delta: 0.0,
            distance_traveled: 0.0,
            health_delta: 0.0,
            ticks_in_background: 0,
        }
    }
}

/// Validate that invariants hold after a promotion/demotion cycle
pub fn validate_invariants_preserved(
    _field: &str,
    preservation: FieldPreservation,
    before: f64,
    after: f64,
) -> bool {
    match preservation {
        FieldPreservation::PreserveExact => (before - after).abs() < f64::EPSILON,
        FieldPreservation::Aggregate => true,
        FieldPreservation::Recompute => true,
        FieldPreservation::Frozen => (before - after).abs() < f64::EPSILON,
    }
}
