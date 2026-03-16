//! ENGENE Game Binary — standalone playable world viewer.
//! Loads the 2x2km world with full simulation: stalkers, monsters, economy, quests,
//! day/night, streaming, save/load, low-spec mode.

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

use engene::app::runtime_assembly::RuntimeAssembly;
use engene::audio::audio::AudioEngine;
use engene::core::engine::Engine;
use engene::economy::resource_flow;
use engene::graphics::camera::FlyCamera;
use engene::graphics::lod::{LodConfig, LodLevel};
use engene::graphics::mesh::EntityInstance;
use engene::graphics::renderer::{RenderCamera, Renderer};
use engene::graphics::visibility::Frustum;
use engene::input::input::InputState;
use engene::memory::asset_manager::AssetManager;
use engene::tools::doctor;
use engene::game::hud::{HudState, NotificationKind};
use engene::game::player::PlayerController;
use engene::game::player_save::{PlayerInventory, PlayerSave};
use engene::world::chunk_persistence::ChunkPersistenceService;
use engene::world::components::*;
use engene::world::hierarchical_spatial::HierarchicalSpatialIndex;
use engene::world::streaming::WorldStreamer;
use engene::world::world::WorldGrid;
use engene::ai::body as ai_body;
use engene::core::build_manifest::BuildManifest;
use engene::core::crash_telemetry;
use engene::core::query::{WithMonster, WithNpc};

type ArcHeightmap = Arc<Heightmap>;

fn parse_layout() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--layout" {
            return args.get(i + 1).cloned();
        }
    }
    None
}

/// Parse CLI flags for engine configuration (C.1)
fn parse_engine_flags() -> EngineFlags {
    let args: Vec<String> = std::env::args().collect();
    EngineFlags {
        sequential: args.iter().any(|a| a == "--sequential"),
    }
}

#[derive(Debug, Clone)]
struct EngineFlags {
    sequential: bool,
}

fn main() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    crash_telemetry::install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    puffin::set_scopes_on(true);

    // Parse CLI flags (C.1)
    let engine_flags = parse_engine_flags();
    
    let manifest = BuildManifest::current();
    let layout = parse_layout();

    let is_sandbox = layout.as_deref() == Some("destruction_sandbox_50x50");

    if is_sandbox {
        println!("=== ENGENE GAME — Destruction Sandbox 50x50 ===");
    } else {
        println!("=== ENGENE GAME ===");
    }
    manifest.print_full();
    println!();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let (engine, heightmap, biomes) = if is_sandbox {
        let engine = RuntimeAssembly::sandbox_50x50();
        let heightmap: ArcHeightmap = Arc::new(Heightmap::flat(50.0));
        let biomes = vec![];
        (engine, heightmap, biomes)
    } else {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let heightmap: ArcHeightmap = Arc::new(Heightmap::generate(&biomes));
        let engine = RuntimeAssembly::vertical_slice(heightmap.clone(), &biomes);
        (engine, heightmap, biomes)
    };

    let doctor_report = doctor::run_doctor(&engine, doctor::DoctorMode::Advisory);
    println!(
        "[doctor] startup: {} errors, {} warnings",
        doctor_report.error_count(),
        doctor_report.warning_count()
    );
    doctor_report.print();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let spawn_pos = if is_sandbox {
        [25.0_f32, 2.0, 25.0]
    } else {
        [500.0_f32, 50.0, 500.0]
    };
    let mut app = GameApp {
        window: None,
        renderer: None,
        engine,
        camera: FlyCamera::new(),
        input: InputState::new(),
        last_frame: None,
        frame: 0,
        sim_accum: 0.0,
        last_report_month: 0,
        heightmap,
        biomes,
        player: PlayerController::new(spawn_pos),
        player_inventory: PlayerInventory::new(),
        hud: HudState::new(),
        play_time: 0.0,
        interaction_target: None,
    };
    app.camera.position = glam::Vec3::new(spawn_pos[0], spawn_pos[1], spawn_pos[2]);

    println!("[render] starting 3D view — click to capture mouse, ESC to release\n");
    let _ = event_loop.run_app(&mut app);
    println!("\n=== SESSION ENDED ===");
}

struct GameApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    engine: Engine,
    camera: FlyCamera,
    input: InputState,
    last_frame: Option<Instant>,
    frame: u64,
    sim_accum: f32,
    last_report_month: u32,
    heightmap: ArcHeightmap,
    biomes: Vec<engene::world::biome::Biome>,
    player: PlayerController,
    player_inventory: PlayerInventory,
    hud: HudState,
    play_time: f64,
    interaction_target: Option<u64>,
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let title = if parse_layout().as_deref() == Some("destruction_sandbox_50x50") {
            "ENGENE Game — Destruction Sandbox 50x50"
        } else {
            "ENGENE Game"
        };
        let attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let mut renderer = Renderer::new(window.clone());
        renderer.upload_terrain(&self.heightmap, &self.biomes);
        renderer.upload_vegetation(&self.heightmap, &self.biomes);
        self.camera.aspect = renderer.aspect();
        self.renderer = Some(renderer);
        self.window = Some(window);
        self.last_frame = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(size.width, size.height);
                    self.camera.aspect = r.aspect();
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    if key == KeyCode::Escape {
                        if self.input.mouse_captured {
                            if let Some(w) = &self.window {
                                let _ = w.set_cursor_grab(CursorGrabMode::None);
                                w.set_cursor_visible(true);
                                self.input.mouse_captured = false;
                            }
                        } else {
                            event_loop.exit();
                        }
                    }

                    match key {
                        KeyCode::Digit1 => {
                            self.player.switch_weapon(0);
                            self.hud.push_notification("Weapon: Makarov", NotificationKind::Info);
                        }
                        KeyCode::Digit2 => {
                            self.player.switch_weapon(1);
                            self.hud.push_notification("Weapon: AK74", NotificationKind::Info);
                        }
                        KeyCode::Digit3 => {
                            self.player.switch_weapon(2);
                            self.hud.push_notification("Weapon: Shotgun", NotificationKind::Info);
                        }
                        KeyCode::KeyG => {
                            self.player.switch_weapon(3);
                            self.hud.push_notification("Weapon: Grenade", NotificationKind::Info);
                        }
                        _ => {}
                    }

                    self.input.key_pressed(key);
                }
                ElementState::Released => self.input.key_released(key),
            },

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if !self.input.mouse_captured {
                    if let Some(w) = &self.window {
                        let _ = w.set_cursor_grab(CursorGrabMode::Confined);
                        w.set_cursor_visible(false);
                        self.input.mouse_captured = true;
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = self
                    .last_frame
                    .map(|lf| now.duration_since(lf).as_secs_f32())
                    .unwrap_or(1.0 / 60.0);
                self.last_frame = Some(now);

                self.camera.update(&self.input, dt);

                // Tick player
                let sprinting = self.input.is_key_down(KeyCode::ShiftLeft);
                self.player.tick(dt, sprinting);

                // Sync player position with camera
                self.player.position = [
                    self.camera.position.x,
                    self.camera.position.y,
                    self.camera.position.z,
                ];

                // Interaction detection: find nearest NPC within range
                // Uses query API instead of direct ECS access (A.2)
                self.hud.clear_interaction();
                self.interaction_target = None;
                if self.player.is_alive() {
                    let px = self.player.position[0];
                    let pz = self.player.position[2];
                    let range = self.player.interaction_range;
                    let mut nearest_dist = f32::MAX;
                    let mut nearest_name = String::new();
                    let mut nearest_entity = None;

                    // Use iter_npcs() query instead of manual iteration
                    for npc in self.engine.ecs.iter_npcs() {
                        let dx = npc.transform.x - px;
                        let dz = npc.transform.y - pz;
                        let dist = (dx * dx + dz * dz).sqrt();
                        if dist < range && dist < nearest_dist {
                            nearest_dist = dist;
                            nearest_entity = Some(npc.entity);
                            nearest_name = npc.name.0.clone();
                        }
                    }

                    if let Some(entity) = nearest_entity {
                        self.interaction_target = Some(entity);
                        self.hud
                            .set_interaction(&format!("[E] Talk to {}", nearest_name));

                        if self.input.was_key_pressed(KeyCode::KeyE) {
                            self.hud.push_notification(
                                &format!("Talking to {}...", nearest_name),
                                NotificationKind::Info,
                            );
                        }
                    }
                }

                // Handle death/respawn at Rookie Camp per WORLD_SLICE_SPEC
                if self.player.state == engene::game::player::PlayerState::Dead {
                    if self.input.was_key_pressed(KeyCode::KeyR) {
                        let respawn = if parse_layout().as_deref() == Some("destruction_sandbox_50x50") {
                            [25.0, 2.0, 25.0]
                        } else {
                            [500.0, 50.0, 500.0]
                        };
                        self.player.respawn(respawn);
                        self.camera.position = glam::Vec3::new(respawn[0], respawn[1], respawn[2]);
                        self.hud.push_notification("Respawned", NotificationKind::Info);
                    }
                }

                // Quick save (F5) / quick load (F9)
                if self.input.was_key_pressed(KeyCode::F5) {
                    let save = PlayerSave::from_state(
                        &self.player,
                        &self.player_inventory,
                        Vec::new(),
                        Vec::new(),
                        self.play_time,
                        self.engine.time.day,
                        self.engine.time.month,
                    );
                    match save.save_to_file("game/saves/quicksave.json") {
                        Ok(()) => self.hud.push_notification("Game saved", NotificationKind::Info),
                        Err(e) => self.hud.push_notification(
                            &format!("Save failed: {}", e),
                            NotificationKind::Warning,
                        ),
                    }
                }
                if self.input.was_key_pressed(KeyCode::F9) {
                    match PlayerSave::load_from_file("game/saves/quicksave.json") {
                        Ok(save) => {
                            self.player = save.restore_controller();
                            self.player_inventory = save.restore_inventory();
                            self.camera.position = glam::Vec3::new(
                                self.player.position[0],
                                self.player.position[1],
                                self.player.position[2],
                            );
                            self.hud.push_notification("Game loaded", NotificationKind::Info);
                        }
                        Err(e) => self.hud.push_notification(
                            &format!("Load failed: {}", e),
                            NotificationKind::Warning,
                        ),
                    }
                }

                self.play_time += dt as f64;

                // Update HUD with player state
                self.hud.tick(dt);

                self.input.end_frame();

                const SIM_DT: f32 = 1.0 / 20.0;
                self.sim_accum += dt.min(0.25);
                while self.sim_accum >= SIM_DT {
                    self.sim_accum -= SIM_DT;
                    self.engine.tick(SIM_DT);
                }
                self.frame += 1;

                if let Some(am) = self
                    .engine
                    .resources
                    .get_mut::<std::sync::Mutex<AssetManager>>()
                {
                    if let Ok(mut am) = am.lock() {
                        am.poll();
                    }
                }

                let cam_pos = self.camera.position;
                let cam_fwd = self.camera.forward();

                // World streaming with persistence
                let (to_load, to_unload) = {
                    if let Some(streamer) = self.engine.resources.get_mut::<WorldStreamer>() {
                        let result = streamer.update(cam_pos.x, cam_pos.z);
                        for coord in &result.0 {
                            streamer.mark_loaded(*coord);
                        }
                        result
                    } else {
                        (vec![], vec![])
                    }
                };

                if !to_unload.is_empty() || !to_load.is_empty() {
                    if let Some(mut persistence) =
                        self.engine.resources.take::<ChunkPersistenceService>()
                    {
                        let tick = self.engine.ecs.tick;
                        for coord in &to_unload {
                            let saved =
                                persistence.save_and_unload(*coord, &mut self.engine.ecs, tick);
                            if saved > 0 {
                                tracing::info!(
                                    "streamer: unloaded chunk ({},{}) — {} entities saved",
                                    coord.x, coord.z, saved
                                );
                            }
                        }
                        for coord in &to_load {
                            let loaded =
                                persistence.load_chunk_entities(*coord, &mut self.engine.ecs);
                            if loaded > 0 {
                                tracing::info!(
                                    "streamer: loaded chunk ({},{}) — {} entities restored",
                                    coord.x, coord.z, loaded
                                );
                            }
                        }
                        self.engine.resources.insert_runtime(persistence);
                    }
                }

                // Rebuild hierarchical spatial index (C.2: use rebuild() instead of clear()+loop)
                // Uses query API instead of direct ECS access (A.2)
                if let Some(spatial) =
                    self.engine.resources.get_mut::<HierarchicalSpatialIndex>()
                {
                    // Collect all entities once, then rebuild in bulk
                    let entities: Vec<_> = self.engine.ecs.entities_with_transform()
                        .map(|(e, t)| (e, t.x, t.y))
                        .collect();
                    spatial.rebuild(&entities);
                }

                // Update audio listener
                if let Some(audio) = self.engine.resources.get_mut::<AudioEngine>() {
                    audio.set_listener(cam_pos, cam_fwd);
                    audio.update(dt);
                }

                let month = self.engine.time.month;
                if month != self.last_report_month {
                    self.last_report_month = month;
                    let season = self.engine.time.season();
                    println!("\n========== MONTH {} ({}) ==========", month, season);
                    print_economy(&self.engine);
                }

                if self.frame % 120 == 0 {
                    print_brief(&self.engine, self.frame);
                    println!(
                        "  [player] hp:{:.0} stamina:{:.0} pos:({:.0},{:.0},{:.0})",
                        self.player.health,
                        self.player.stamina,
                        self.player.position[0],
                        self.player.position[1],
                        self.player.position[2]
                    );
                }

                if let Some(r) = self.renderer.as_mut() {
                    let vp = self.camera.view_projection();
                    let frustum = Frustum::from_view_projection(&vp);
                    let instances = collect_entity_instances(
                        &self.engine.ecs,
                        &self.heightmap,
                        [cam_pos.x, cam_pos.y, cam_pos.z],
                        &frustum,
                    );
                    r.update_entities(&instances);

                    let render_cam = RenderCamera {
                        view_proj: vp.to_cols_array_2d(),
                        inv_view_proj: vp.inverse().to_cols_array_2d(),
                        position: [cam_pos.x, cam_pos.y, cam_pos.z],
                        forward: {
                            let f = self.camera.forward();
                            [f.x, f.y, f.z]
                        },
                        near: self.camera.near,
                        far: self.camera.far,
                        day_progress: self.engine.time.day_progress(),
                    };
                    match r.render(&render_cam) {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            let (w, h) = r.size();
                            r.resize(w, h);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("render error: {e}"),
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input.accumulate_mouse(delta.0, delta.1);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

fn print_brief(engine: &Engine, frame: u64) {
    // Uses query API instead of direct ECS access (A.2)
    let npcs: Vec<_> = engine.ecs.iter_npcs().collect();
    let monsters: Vec<_> = engine.ecs.query_filter(WithMonster).collect();
    let snap = resource_flow::snapshot(&engine.ecs);

    let mut wolves = 0u32;
    let mut boars = 0u32;
    let mut bloods = 0u32;
    for &m in &monsters {
        match engine.ecs.kinds.get(&m) {
            Some(EntityKind::Monster(MonsterSpecies::Wolf)) => wolves += 1,
            Some(EntityKind::Monster(MonsterSpecies::Boar)) => boars += 1,
            Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)) => bloods += 1,
            _ => {}
        }
    }

    let carcasses = engine.resource_grid().carcasses.len();
    let night = if ai_body::is_night(engine.time.day_progress()) {
        "NIGHT"
    } else {
        "day"
    };
    let season = engine.time.season();

    println!(
        "[tick {}] day {} mo {} {} ({}) | NPC:{} W:{} B:{} BS:{} tot:{} carcass:{} | ${:.0} desp:{:.2}",
        frame, engine.time.day, engine.time.month, season, night,
        npcs.len(), wolves, boars, bloods, engine.ecs.alive.len(), carcasses,
        snap.total_npc_money, snap.average_desperation,
    );
}

fn collect_entity_instances(
    ecs: &engene::core::ecs::Ecs,
    heightmap: &Heightmap,
    camera_pos: [f32; 3],
    frustum: &Frustum,
) -> Vec<EntityInstance> {
    // Uses query API instead of direct ECS access (A.2)
    let lod_config = LodConfig::default();
    let cam = glam::Vec3::from(camera_pos);
    let mut out = Vec::with_capacity(ecs.alive.len());
    
    // Use iter_transform_kind() query instead of manual iteration
    for (entity, transform, kind) in ecs.iter_transform_kind() {
        let y = heightmap.sample(transform.x, transform.y) + 1.0;
        let pos = glam::Vec3::new(transform.x, y, transform.y);
        let dist = (pos - cam).length();
        if lod_config.compute_lod(dist) == LodLevel::Culled {
            continue;
        }
        if !frustum.test_sphere(pos, 2.0) {
            continue;
        }
        let color = match kind {
            EntityKind::Npc => [0.16, 0.47, 1.0],
            EntityKind::Monster(MonsterSpecies::Wolf)) => [0.9, 0.9, 0.9],
            EntityKind::Monster(MonsterSpecies::Boar)) => [0.55, 0.43, 0.39],
            EntityKind::Monster(MonsterSpecies::Bloodsucker)) => [0.83, 0.0, 0.0],
        };
        out.push(EntityInstance {
            position: [transform.x, y, transform.y],
            color,
        });
    }
    out
}

fn print_economy(engine: &Engine) {
    // Uses query API instead of direct ECS access (A.2)
    println!("--- NPCs ({}) ---", engine.ecs.count_npcs());
    
    // Use iter_npcs() query instead of manual iteration
    for npc in engine.ecs.iter_npcs() {
        let name = npc.name.0.as_str();
        let goal = match engine.ecs.ai_states.get(&npc.entity) {
            Some(AiState::Executing(g)) => format!("{}", g),
            _ => "Idle".into(),
        };
        let hp = npc.needs.map_or(1.0, |p| p.health);
        let hunger = npc.needs.map_or(0.0, |p| p.hunger);
        let money = npc.economy.map_or(0.0, |e| e.money);

        println!(
            "  {} | {} | hp:{:.0}% hunger:{:.0}% ${:.0}",
            name, goal, hp * 100.0, hunger * 100.0, money,
        );
    }
}
