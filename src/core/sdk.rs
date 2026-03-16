use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PluginContract {
    pub name: String,
    pub version: String,
    pub provides_systems: Vec<String>,
    pub requires_systems: Vec<String>,
    pub reads_resources: Vec<String>,
    pub writes_resources: Vec<String>,
    pub emits_events: Vec<String>,
    pub reads_events: Vec<String>,
    pub ordering_constraints: Vec<OrderingConstraint>,
}

#[derive(Clone, Debug)]
pub struct OrderingConstraint {
    pub system: String,
    pub kind: OrderingKind,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderingKind {
    Before,
    After,
    Requires,
    ConflictsWith,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundaryLayer {
    Engine,
    Game,
    Scripting,
}

#[derive(Clone, Debug)]
pub struct SystemContract {
    pub name: String,
    pub layer: BoundaryLayer,
    pub deterministic: bool,
    pub parallel_safe: bool,
    pub headless_compatible: bool,
    pub public_api: Vec<String>,
}

pub struct EngineSDK {
    plugins: Vec<PluginContract>,
    systems: HashMap<String, SystemContract>,
    engine_boundary: Vec<String>,
    game_boundary: Vec<String>,
    scripting_api: Vec<ScriptingEndpoint>,
}

#[derive(Clone, Debug)]
pub struct ScriptingEndpoint {
    pub name: String,
    pub description: String,
    pub parameters: Vec<(String, String)>,
    pub return_type: String,
    pub side_effects: bool,
}

impl EngineSDK {
    pub fn new() -> Self {
        let mut sdk = Self {
            plugins: Vec::new(),
            systems: HashMap::new(),
            engine_boundary: Vec::new(),
            game_boundary: Vec::new(),
            scripting_api: Vec::new(),
        };
        sdk.populate_default_contracts();
        sdk
    }

    fn populate_default_contracts(&mut self) {
        let engine_systems = vec![
            ("SimulationSystem", true, true, true),
            ("WorldTickSystem", true, true, true),
            ("AiSystem", true, true, true),
            ("PhysicsSystem", true, false, true),
            ("BallisticsSystem", true, false, true),
            ("DamageDispatchSystem", true, false, true),
            ("DestructionSystem", true, false, true),
            ("TerrainDeformationSystem", true, false, true),
            ("NavDirtyTickSystem", true, true, true),
            ("AnimationIntegrationSystem", false, false, false),
            ("AudioIntegrationSystem", false, false, false),
            ("QuestSystem", true, true, true),
            ("BodySystem", true, false, true),
            ("RenderSystem", false, true, false),
            ("AudioPlaybackBridge", false, false, false),
            ("InputActionSystem", false, true, true),
            ("NetworkSystem", false, false, true),
            ("AiDecisionWireSystem", true, false, true),
        ];

        for (name, deterministic, parallel_safe, headless_compat) in &engine_systems {
            self.register_system(SystemContract {
                name: name.to_string(),
                layer: BoundaryLayer::Engine,
                deterministic: *deterministic,
                parallel_safe: *parallel_safe,
                headless_compatible: *headless_compat,
                public_api: vec![format!("{name}::tick")],
            });
        }

        let game_systems = vec![
            "EconomySystem",
            "OcclusionWireSystem",
            "GoreWireSystem",
        ];

        for name in &game_systems {
            self.register_system(SystemContract {
                name: name.to_string(),
                layer: BoundaryLayer::Game,
                deterministic: true,
                parallel_safe: false,
                headless_compatible: true,
                public_api: vec![format!("{name}::tick")],
            });
        }

        self.define_engine_boundary(
            engine_systems.iter().map(|(n, ..)| n.to_string()).collect(),
        );
        self.define_game_boundary(
            game_systems.iter().map(|n| n.to_string()).collect(),
        );

        self.populate_scripting_endpoints();
    }

    fn populate_scripting_endpoints(&mut self) {
        let endpoints = vec![
            ("ecs.spawn", "Spawn a new entity", vec![("kind", "EntityKind")], "Entity", true),
            ("ecs.despawn", "Remove entity from world", vec![("entity", "Entity")], "bool", true),
            ("ecs.get_transform", "Get entity position", vec![("entity", "Entity")], "Transform?", false),
            ("ecs.set_transform", "Set entity position", vec![("entity", "Entity"), ("x", "f32"), ("y", "f32")], "()", true),
            ("ecs.alive_count", "Count alive entities", vec![], "usize", false),
            ("time.day", "Get current day", vec![], "u32", false),
            ("time.month", "Get current month", vec![], "u32", false),
            ("time.day_progress", "Get time of day 0..1", vec![], "f32", false),
            ("quest.create", "Create a new quest", vec![("quest_type", "QuestType"), ("reward", "f32")], "u32", true),
            ("quest.complete", "Mark quest completed", vec![("quest_id", "u32")], "bool", true),
            ("audio.play_3d", "Play 3D sound", vec![("kind", "SoundKind"), ("pos", "Vec3"), ("vol", "f32")], "SoundHandle", true),
            ("world.save_chunk", "Save chunk to disk", vec![("x", "i32"), ("z", "i32")], "usize", true),
            ("world.load_chunk", "Load chunk from disk", vec![("x", "i32"), ("z", "i32")], "usize", true),
            ("sim.set_speed", "Set simulation speed", vec![("speed", "f32")], "()", true),
            ("sim.pause", "Pause simulation", vec![], "()", true),
            ("sim.resume", "Resume simulation", vec![], "()", true),
        ];

        for (name, desc, params, ret, side_effects) in endpoints {
            self.register_scripting_endpoint(ScriptingEndpoint {
                name: name.to_string(),
                description: desc.to_string(),
                parameters: params.into_iter().map(|(n, t)| (n.to_string(), t.to_string())).collect(),
                return_type: ret.to_string(),
                side_effects,
            });
        }
    }

    pub fn plugins(&self) -> &[PluginContract] {
        &self.plugins
    }

    pub fn systems(&self) -> &HashMap<String, SystemContract> {
        &self.systems
    }

    pub fn register_plugin(&mut self, contract: PluginContract) {
        self.plugins.push(contract);
    }

    pub fn register_system(&mut self, contract: SystemContract) {
        self.systems.insert(contract.name.clone(), contract);
    }

    pub fn define_engine_boundary(&mut self, exported: Vec<String>) {
        self.engine_boundary = exported;
    }

    pub fn define_game_boundary(&mut self, exported: Vec<String>) {
        self.game_boundary = exported;
    }

    pub fn register_scripting_endpoint(&mut self, endpoint: ScriptingEndpoint) {
        self.scripting_api.push(endpoint);
    }

    pub fn validate_contracts(&self) -> Vec<String> {
        let mut issues = Vec::new();

        for plugin in &self.plugins {
            for req in &plugin.requires_systems {
                let provided = self.plugins.iter()
                    .any(|p| p.provides_systems.contains(req));
                let is_system = self.systems.contains_key(req);
                if !provided && !is_system {
                    issues.push(format!(
                        "Plugin '{}' requires system '{}' which is not provided by any plugin",
                        plugin.name, req
                    ));
                }
            }

            for constraint in &plugin.ordering_constraints {
                if constraint.kind == OrderingKind::Requires {
                    let exists = self.systems.contains_key(&constraint.target)
                        || self.plugins.iter().any(|p| p.provides_systems.contains(&constraint.target));
                    if !exists {
                        issues.push(format!(
                            "Plugin '{}' system '{}' requires '{}' which doesn't exist",
                            plugin.name, constraint.system, constraint.target
                        ));
                    }
                }
            }
        }

        for (name, sys) in &self.systems {
            if sys.layer == BoundaryLayer::Game && !self.game_boundary.contains(name) {
                issues.push(format!(
                    "Game system '{}' is not listed in game boundary exports",
                    name
                ));
            }
        }

        issues
    }

    pub fn engine_systems(&self) -> Vec<&SystemContract> {
        self.systems.values()
            .filter(|s| s.layer == BoundaryLayer::Engine)
            .collect()
    }

    pub fn game_systems(&self) -> Vec<&SystemContract> {
        self.systems.values()
            .filter(|s| s.layer == BoundaryLayer::Game)
            .collect()
    }

    pub fn scripting_endpoints(&self) -> &[ScriptingEndpoint] {
        &self.scripting_api
    }

    pub fn plugin_count(&self) -> usize { self.plugins.len() }
    pub fn system_count(&self) -> usize { self.systems.len() }

    pub fn generate_api_reference(&self) -> String {
        let mut out = String::from("# ENGENE Scripting API Reference\n\n");
        out.push_str(&format!("Generated from {} endpoints.\n\n", self.scripting_api.len()));

        for ep in &self.scripting_api {
            out.push_str(&format!("## `{}`\n\n", ep.name));
            out.push_str(&format!("{}\n\n", ep.description));
            if !ep.parameters.is_empty() {
                out.push_str("**Parameters:**\n\n");
                for (name, ty) in &ep.parameters {
                    out.push_str(&format!("- `{}`: `{}`\n", name, ty));
                }
                out.push('\n');
            }
            out.push_str(&format!("**Returns:** `{}`\n\n", ep.return_type));
            if ep.side_effects {
                out.push_str("*Has side effects.*\n\n");
            }
            out.push_str("---\n\n");
        }

        out
    }
}

impl Default for EngineSDK {
    fn default() -> Self { Self::new() }
}
