//! Game app loop and frame/input/render orchestration.

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

use super::diagnostics::{collect_entity_instances, print_brief, print_economy};
use engine_audio::audio::AudioEngine;
// LEGACY IMPORT - Replace with canonical crate
use engine_core::engine::Engine;
use engine_game::hud::{HudState, NotificationKind};
use engine_game::player::PlayerController;
use engine_game::player_save::{PlayerInventory, PlayerSave};
use engine_render::camera::FlyCamera;
use engine_render::renderer::{RenderCamera, Renderer};
use engine_render::visibility::Frustum;
use engine_input::input::InputState;
use engine_memory::asset_manager::AssetManager;
use engine_world::chunk_persistence::ChunkPersistenceService;
use engine_world::heightmap::Heightmap;
use engine_world::hierarchical_spatial::HierarchicalSpatialIndex;
use engine_world::streaming::WorldStreamer;

type ArcHeightmap = Arc<Heightmap>;

pub struct GameApp {
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
    biomes: Vec<engine_world::biome::Biome>,
    player: PlayerController,
    player_inventory: PlayerInventory,
    hud: HudState,
    play_time: f64,
    interaction_target: Option<u64>,
}

impl GameApp {
    pub fn new(
        engine: Engine,
        heightmap: ArcHeightmap,
        biomes: Vec<engine_world::biome::Biome>,
    ) -> Self {
        let spawn_pos = [500.0_f32, 50.0, 500.0];
        let mut app = Self {
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
        app
    }
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let title = "ENGENE Game";
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
                            self.hud
                                .push_notification("Weapon: Makarov", NotificationKind::Info);
                        }
                        KeyCode::Digit2 => {
                            self.player.switch_weapon(1);
                            self.hud
                                .push_notification("Weapon: AK74", NotificationKind::Info);
                        }
                        KeyCode::Digit3 => {
                            self.player.switch_weapon(2);
                            self.hud
                                .push_notification("Weapon: Shotgun", NotificationKind::Info);
                        }
                        KeyCode::KeyG => {
                            self.player.switch_weapon(3);
                            self.hud
                                .push_notification("Weapon: Grenade", NotificationKind::Info);
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
                if self.player.state == engine_game::player::PlayerState::Dead {
                    if self.input.was_key_pressed(KeyCode::KeyR) {
                        let respawn = [500.0, 50.0, 500.0];
                        self.player.respawn(respawn);
                        self.camera.position = glam::Vec3::new(respawn[0], respawn[1], respawn[2]);
                        self.hud
                            .push_notification("Respawned", NotificationKind::Info);
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
                        Ok(()) => self
                            .hud
                            .push_notification("Game saved", NotificationKind::Info),
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
                            self.hud
                                .push_notification("Game loaded", NotificationKind::Info);
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
                                    coord.x,
                                    coord.z,
                                    saved
                                );
                            }
                        }
                        for coord in &to_load {
                            let loaded =
                                persistence.load_chunk_entities(*coord, &mut self.engine.ecs);
                            if loaded > 0 {
                                tracing::info!(
                                    "streamer: loaded chunk ({},{}) — {} entities restored",
                                    coord.x,
                                    coord.z,
                                    loaded
                                );
                            }
                        }
                        self.engine.resources.insert_runtime(persistence);
                    }
                }

                // Rebuild hierarchical spatial index (C.2: use rebuild() instead of clear()+loop)
                // Uses query API instead of direct ECS access (A.2)
                if let Some(spatial) = self.engine.resources.get_mut::<HierarchicalSpatialIndex>() {
                    // Collect all entities once, then rebuild in bulk
                    let entities: Vec<_> = self
                        .engine
                        .ecs
                        .entities_with_transform()
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
