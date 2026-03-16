use std::collections::HashMap;

pub type CommandHandler = Box<dyn Fn(&[&str]) -> String + Send>;

pub struct EngineConsole {
    history: Vec<ConsoleEntry>,
    commands: HashMap<String, CommandInfo>,
    handlers: HashMap<String, CommandHandler>,
    input_buffer: String,
    max_history: usize,
}

#[derive(Clone, Debug)]
pub struct ConsoleEntry {
    pub input: String,
    pub output: String,
    pub is_error: bool,
}

#[derive(Clone, Debug)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
}

impl EngineConsole {
    pub fn new() -> Self {
        let mut console = Self {
            history: Vec::new(),
            commands: HashMap::new(),
            handlers: HashMap::new(),
            input_buffer: String::new(),
            max_history: 500,
        };
        console.register_builtin_commands();
        console
    }

    fn register_builtin_commands(&mut self) {
        let engine_cmds: &[(&str, &str, &str)] = &[
            ("help", "List all commands", "help [command]"),
            ("clear", "Clear console history", "clear"),
            ("scan_entity_refs", "Audit entity references", "scan_entity_refs"),
            ("runtime_truth", "Generate runtime truth JSON", "runtime_truth"),
            ("world_budget", "Display world budget status", "world_budget"),
            ("spawn", "Spawn entity at position", "spawn <npc|wolf|boar|bloodsucker> <x> <z>"),
            ("teleport", "Teleport entity", "teleport <entity_id> <x> <z>"),
            ("kill", "Despawn entity", "kill <entity_id>"),
            ("save_chunk", "Save chunk to disk", "save_chunk <cx> <cz>"),
            ("reload_chunk", "Reload chunk from disk", "reload_chunk <cx> <cz>"),
            ("time_scale", "Set simulation time scale", "time_scale <factor>"),
            ("doctor", "Run doctor diagnostics", "doctor [strict]"),
            ("save_snapshot", "Save all loaded chunks", "save_snapshot"),
            ("entity_count", "Show entity counts", "entity_count"),
        ];
        for &(name, desc, usage) in engine_cmds {
            self.commands.insert(name.to_string(), CommandInfo {
                name: name.to_string(),
                description: desc.to_string(),
                usage: usage.to_string(),
            });
        }
        self.handlers.insert("help".to_string(), Box::new(|_| "Use 'help' to list commands".to_string()));
        self.handlers.insert("clear".to_string(), Box::new(|_| String::new()));
        for &name in &["scan_entity_refs", "runtime_truth", "world_budget", "spawn",
                        "teleport", "kill", "save_chunk", "reload_chunk", "time_scale",
                        "doctor", "save_snapshot", "entity_count"] {
            self.handlers.insert(name.to_string(), Box::new(|_| {
                format!("{}: requires engine context", "command")
            }));
        }
    }

    pub fn execute_with_engine(&mut self, input: &str, engine: &crate::core::engine::Engine) -> String {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }
        let output = match parts[0] {
            "scan_entity_refs" => {
                let ecs = &engine.ecs;
                let total = ecs.alive.len();
                let with_pid = ecs.alive.iter()
                    .filter(|&&e| ecs.identity.persistent_id_of(e).is_some())
                    .count();
                let without_pid = total - with_pid;
                let without_kind = ecs.alive.iter()
                    .filter(|&&e| ecs.kinds.get(&e).is_none())
                    .count();
                let without_transform = ecs.alive.iter()
                    .filter(|&&e| ecs.transforms.get(&e).is_none())
                    .count();
                let tombstones = ecs.identity.tombstone_count();
                format!(
                    "=== Entity Reference Audit ===\n\
                     Alive: {}\n\
                     With PersistentId: {} ({:.0}%)\n\
                     Without PersistentId: {}\n\
                     Without EntityKind: {}\n\
                     Without Transform: {}\n\
                     Tombstones: {}\n\
                     Identity registry total: {}",
                    total,
                    with_pid, if total > 0 { with_pid as f32 / total as f32 * 100.0 } else { 0.0 },
                    without_pid,
                    without_kind,
                    without_transform,
                    tombstones,
                    ecs.identity.total_count(),
                )
            }
            "runtime_truth" => {
                crate::tools::doctor::generate_runtime_truth_json(engine)
            }
            "world_budget" | "entity_count" => {
                let entities = engine.ecs.alive.len();
                let npcs = engine.ecs.count_npcs();
                let monsters = engine.ecs.monsters().len();
                format!(
                    "=== World Budget ===\nEntities: {}\nNPCs: {}\nMonsters: {}",
                    entities, npcs, monsters,
                )
            }
            "doctor" => {
                let mode = if parts.get(1) == Some(&"strict") {
                    crate::tools::doctor::DoctorMode::Strict
                } else {
                    crate::tools::doctor::DoctorMode::Advisory
                };
                let report = crate::tools::doctor::run_doctor(engine, mode);
                format!(
                    "Doctor ({}): {} errors, {} warnings",
                    if matches!(mode, crate::tools::doctor::DoctorMode::Strict) { "strict" } else { "advisory" },
                    report.error_count(),
                    report.warning_count(),
                )
            }
            "spawn" | "teleport" | "kill" | "save_chunk" | "reload_chunk"
            | "time_scale" | "save_snapshot" => {
                format!("{}: accepted (will apply on next frame)", parts[0])
            }
            _ => return self.execute(input),
        };
        self.push_entry(input, &output, false);
        output
    }

    pub fn execute_with_engine_mut(&mut self, input: &str, engine: &mut crate::core::engine::Engine) -> String {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }
        let output = match parts[0] {
            "spawn" => {
                if parts.len() < 4 {
                    "Usage: spawn <npc|wolf|boar|bloodsucker> <x> <z>".to_string()
                } else {
                    let kind = parts[1];
                    let x: f32 = parts[2].parse().unwrap_or(500.0);
                    let z: f32 = parts[3].parse().unwrap_or(500.0);
                    let entity = engine.ecs.spawn();
                    engine.ecs.transforms.insert(entity, crate::world::components::Transform { x, y: z, cell_x: (x / 256.0) as u32, cell_y: (z / 256.0) as u32 });
                    let ek = match kind {
                        "wolf" => crate::world::components::EntityKind::Monster(crate::world::components::MonsterSpecies::Wolf),
                        "boar" => crate::world::components::EntityKind::Monster(crate::world::components::MonsterSpecies::Boar),
                        "bloodsucker" => crate::world::components::EntityKind::Monster(crate::world::components::MonsterSpecies::Bloodsucker),
                        _ => crate::world::components::EntityKind::Npc,
                    };
                    engine.ecs.kinds.insert(entity, ek);
                    engine.ecs.names.insert(entity, crate::world::components::Name(format!("spawned_{}", entity)));
                    format!("Spawned {} entity {} at ({}, {})", kind, entity, x, z)
                }
            }
            "teleport" => {
                if parts.len() < 4 {
                    "Usage: teleport <entity_id> <x> <z>".to_string()
                } else {
                    let eid: u64 = parts[1].parse().unwrap_or(0);
                    let x: f32 = parts[2].parse().unwrap_or(0.0);
                    let z: f32 = parts[3].parse().unwrap_or(0.0);
                    if let Some(t) = engine.ecs.transforms.get_mut(&eid) {
                        t.x = x;
                        t.y = z;
                        format!("Teleported entity {} to ({}, {})", eid, x, z)
                    } else {
                        format!("Entity {} not found or has no transform", eid)
                    }
                }
            }
            "kill" => {
                if parts.len() < 2 {
                    "Usage: kill <entity_id>".to_string()
                } else {
                    let eid: u64 = parts[1].parse().unwrap_or(0);
                    engine.ecs.despawn(eid);
                    format!("Despawned entity {}", eid)
                }
            }
            "save_chunk" => {
                if parts.len() < 3 {
                    "Usage: save_chunk <cx> <cz>".to_string()
                } else {
                    let cx: i32 = parts[1].parse().unwrap_or(0);
                    let cz: i32 = parts[2].parse().unwrap_or(0);
                    if let Some(mut persistence) = engine.resources.take::<crate::world::chunk_persistence::ChunkPersistenceService>() {
                        let coord = crate::world::streaming::ChunkCoord { x: cx, z: cz };
                        let tick = engine.ecs.tick;
                        let saved = persistence.save_and_unload(coord, &mut engine.ecs, tick);
                        engine.resources.insert_runtime(persistence);
                        format!("Saved chunk ({},{}) — {} entities", cx, cz, saved)
                    } else {
                        "ChunkPersistenceService not available".to_string()
                    }
                }
            }
            "reload_chunk" => {
                if parts.len() < 3 {
                    "Usage: reload_chunk <cx> <cz>".to_string()
                } else {
                    let cx: i32 = parts[1].parse().unwrap_or(0);
                    let cz: i32 = parts[2].parse().unwrap_or(0);
                    if let Some(mut persistence) = engine.resources.take::<crate::world::chunk_persistence::ChunkPersistenceService>() {
                        let coord = crate::world::streaming::ChunkCoord { x: cx, z: cz };
                        let loaded = persistence.load_chunk_entities(coord, &mut engine.ecs);
                        engine.resources.insert_runtime(persistence);
                        format!("Reloaded chunk ({},{}) — {} entities", cx, cz, loaded)
                    } else {
                        "ChunkPersistenceService not available".to_string()
                    }
                }
            }
            "save_snapshot" => {
                format!("save_snapshot: saving all loaded chunks is not yet implemented as a single command")
            }
            _ => return self.execute_with_engine(input, engine),
        };
        self.push_entry(input, &output, false);
        output
    }

    pub fn register_command(
        &mut self,
        name: &str,
        description: &str,
        usage: &str,
        handler: CommandHandler,
    ) {
        self.commands.insert(name.to_string(), CommandInfo {
            name: name.to_string(),
            description: description.to_string(),
            usage: usage.to_string(),
        });
        self.handlers.insert(name.to_string(), handler);
    }

    pub fn execute(&mut self, input: &str) -> String {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }

        let cmd = parts[0];
        let args = &parts[1..];

        if cmd == "clear" {
            self.history.clear();
            return String::new();
        }

        if cmd == "help" {
            if args.is_empty() {
                let mut lines = Vec::new();
                let mut cmds: Vec<&str> = self.commands.keys().map(|s| s.as_str()).collect();
                cmds.sort();
                for name in cmds {
                    if let Some(info) = self.commands.get(name) {
                        lines.push(format!("  {} -- {}", info.name, info.description));
                    }
                }
                let output = lines.join("\n");
                self.push_entry(input, &output, false);
                return output;
            }
        }

        let output = if let Some(handler) = self.handlers.get(cmd) {
            handler(args)
        } else {
            format!("Unknown command: '{}'. Type 'help' for available commands.", cmd)
        };

        let is_error = output.starts_with("Error") || output.starts_with("Unknown");
        self.push_entry(input, &output, is_error);
        output
    }

    fn push_entry(&mut self, input: &str, output: &str, is_error: bool) {
        self.history.push(ConsoleEntry {
            input: input.to_string(),
            output: output.to_string(),
            is_error,
        });
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn history(&self) -> &[ConsoleEntry] {
        &self.history
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    #[cfg(feature = "debug_ui")]
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.heading("Console");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in &self.history {
                    ui.label(format!("> {}", entry.input));
                    if !entry.output.is_empty() {
                        if entry.is_error {
                            ui.colored_label(egui::Color32::RED, &entry.output);
                        } else {
                            ui.label(&entry.output);
                        }
                    }
                }
            });

        ui.separator();
        let mut input = self.input_buffer.clone();
        let response = ui.text_edit_singleline(&mut input);
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !input.trim().is_empty() {
                self.execute(&input);
            }
            input.clear();
        }
        self.input_buffer = input;
    }
}

impl Default for EngineConsole {
    fn default() -> Self { Self::new() }
}
