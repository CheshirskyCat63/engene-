//! ENGENE TEST Binary — Destruction Sandbox 50x50.
//! Double-click TEST.exe to launch the sandbox proving ground immediately.
//! No arguments required. Uses RuntimeAssembly::sandbox_50x50().

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
use engene::graphics::camera::FlyCamera;
use engene::graphics::lod::{LodConfig, LodLevel};
use engene::graphics::mesh::EntityInstance;
use engene::graphics::renderer::{RenderCamera, Renderer};
use engene::graphics::visibility::Frustum;
use engene::input::input::InputState;
use engene::memory::asset_manager::AssetManager;
use engene::physics::ballistics::BallisticsSystem;
use engene::tools::doctor;
use engene::game::hud::{HudState, NotificationKind};
use engene::game::player::PlayerController;
use engene::game::player_save::{PlayerInventory, PlayerSave};
use engene::world::chunk_persistence::ChunkPersistenceService;
use engene::world::components::*;
use engene::world::heightmap::Heightmap;
use engene::world::hierarchical_spatial::HierarchicalSpatialIndex;
use engene::world::streaming::WorldStreamer;
use engene::core::build_manifest::BuildManifest;
use engene::core::crash_telemetry;

fn main() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    crash_telemetry::install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    puffin::set_scopes_on(true);

    let manifest = BuildManifest::current();
    println!("=== ENGENE TEST — Destruction Sandbox 50x50 ===");
    manifest.print_full();
    println!();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let engine = RuntimeAssembly::sandbox_50x50();
    let heightmap = Arc::new(Heightmap::flat(50.0));

    let doctor_report = doctor::run_doctor(&engine, doctor::DoctorMode::Advisory);
    println!(
        "[doctor] startup: {} errors, {} warnings",
        doctor_report.error_count(),
        doctor_report.warning_count()
    );
    doctor_report.print();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let spawn_pos = [25.0_f32, 2.0, 25.0];
    let mut app = TestApp {
        window: None,
        renderer: None,
        engine,
        camera: FlyCamera::new(),
        input: InputState::new(),
        last_frame: None,
        frame: 0,
        sim_accum: 0.0,
        heightmap: heightmap.clone(),
        player: PlayerController::new(spawn_pos),
        player_inventory: PlayerInventory::new(),
        hud: HudState::new(),
        play_time: 0.0,
        fire_seed: 0,
        day_progress: 0.35,
        day_cycle_enabled: true,
        weather_cycle_index: 0,
    };
    let terrain_y = heightmap.sample(spawn_pos[0], spawn_pos[2]);
    app.camera.position = glam::Vec3::new(spawn_pos[0], terrain_y + 1.7, spawn_pos[2]);
    app.camera.speed = 5.0;
    app.camera.pitch = 0.0;
    app.camera.far = 150.0;

    let has_bal = app.engine.resources.contains::<BallisticsSystem>();
    println!("[sandbox] BallisticsSystem in resources: {}", has_bal);
    if !has_bal {
        println!("[sandbox] CRITICAL: BallisticsSystem missing! Shooting will NOT work.");
        println!("[sandbox] Inserting BallisticsSystem manually...");
        app.engine.resources.insert_runtime(BallisticsSystem::new());
    }
    let entity_count = app.engine.ecs.alive.len();
    println!("[sandbox] entities alive at start: {}", entity_count);
    println!("[sandbox] Destruction Sandbox 50x50");
    println!("  Controls:");
    println!("    Click     - capture mouse");
    println!("    WASD      - walk (Shift = sprint)");
    println!("    Mouse     - look around");
    println!("    1/2/3     - Makarov / AK74 / Shotgun");
    println!("    G         - throw grenade");
    println!("    LMB       - fire weapon");
    println!("    F5/F9     - save / load");
    println!("    R         - respawn (when dead)");
    println!("    T         - cycle weather (Clear/Cloudy/Storm/Rain/Clearing)");
    println!("    N         - toggle day/night cycle");
    println!("    ESC       - release mouse / exit");
    println!();
    let _ = event_loop.run_app(&mut app);
    println!("\n=== TEST SESSION ENDED ===");
}

struct TestApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    engine: Engine,
    camera: FlyCamera,
    input: InputState,
    last_frame: Option<Instant>,
    frame: u64,
    sim_accum: f32,
    heightmap: Arc<Heightmap>,
    player: PlayerController,
    player_inventory: PlayerInventory,
    hud: HudState,
    play_time: f64,
    fire_seed: u32,
    // Weather controls
    day_progress: f32,
    day_cycle_enabled: bool,
    weather_cycle_index: u32,
}

impl TestApp {
    fn weather_controller_coverage(&self) -> f32 {
        self.renderer.as_ref().map(|r| r.weather_cloud_coverage()).unwrap_or(0.0)
    }
}

impl ApplicationHandler for TestApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("ENGENE TEST — Destruction Sandbox 50x50")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let mut renderer = Renderer::new(window.clone());
        let biomes = vec![engene::world::biome::Biome::Settlement; 1600];
        renderer.upload_terrain(&self.heightmap, &biomes);
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
                            self.hud.push_notification("[1] Makarov", NotificationKind::Info);
                            println!("[weapon] switched to Makarov");
                        }
                        KeyCode::Digit2 => {
                            self.player.switch_weapon(1);
                            self.hud.push_notification("[2] AK74", NotificationKind::Info);
                            println!("[weapon] switched to AK74");
                        }
                        KeyCode::Digit3 => {
                            self.player.switch_weapon(2);
                            self.hud.push_notification("[3] Shotgun", NotificationKind::Info);
                            println!("[weapon] switched to Shotgun");
                        }
                        KeyCode::KeyG => {
                            println!("[grenade] thrown at ({:.1}, {:.1})",
                                self.camera.position.x, self.camera.position.z);
                            self.hud.push_notification(">> Grenade thrown! <<", NotificationKind::Warning);
                        }
                        KeyCode::KeyT => {
                            self.weather_cycle_index = (self.weather_cycle_index + 1) % 5;
                            let weather_name = match self.weather_cycle_index {
                                0 => "Clear",
                                1 => "Cloudy",
                                2 => "Storm",
                                3 => "Rain",
                                4 => "Clearing",
                                _ => "Clear",
                            };
                            println!("[weather] cycled to: {}", weather_name);
                            self.hud.push_notification(
                                &format!("Weather: {}", weather_name),
                                NotificationKind::Info,
                            );
                            if let Some(r) = self.renderer.as_mut() {
                                r.force_weather_state(self.weather_cycle_index);
                            }
                        }
                        KeyCode::KeyN => {
                            self.day_cycle_enabled = !self.day_cycle_enabled;
                            let status = if self.day_cycle_enabled { "ON" } else { "OFF" };
                            println!("[weather] day/night cycle: {}", status);
                            self.hud.push_notification(
                                &format!("Day cycle: {}", status),
                                NotificationKind::Info,
                            );
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
                } else if self.player.can_fire() {
                    let origin = self.camera.position;
                    let dir = self.camera.forward();
                    let weapon_name = self.player.current_weapon_name()
                        .unwrap_or("Makarov").to_string();
                    let (speed, mass, drag, fire_rate, wid, pellets) = match weapon_name.as_str() {
                        "Makarov" => (315.0_f32, 0.006, 0.0001, 4.0_f32, 0_u16, 1_u32),
                        "AK74"    => (900.0, 0.0034, 0.00008, 10.0, 1, 1),
                        "Shotgun" => (400.0, 0.032, 0.0003, 1.0, 2, 8),
                        _         => (400.0, 0.01, 0.0001, 2.0, 3, 1),
                    };
                    self.player.fire_cooldown = 1.0 / fire_rate;
                    let has_ballistics = self.engine.resources.contains::<BallisticsSystem>();
                    println!("[FIRE] weapon={} speed={} pellets={} ballistics_exists={}", weapon_name, speed, pellets, has_ballistics);
                    if let Some(ballistics) = self.engine.resources.get_mut::<BallisticsSystem>() {
                        for p in 0..pellets {
                            let spread = if pellets > 1 {
                                let angle = (p as f32 / pellets as f32) * std::f32::consts::TAU;
                                glam::Vec3::new(angle.cos() * 0.04, angle.sin() * 0.04, 0.0)
                            } else {
                                glam::Vec3::ZERO
                            };
                            let shot_dir = (dir + spread).normalize();
                            self.fire_seed += 1;
                            ballistics.fire(origin, shot_dir, speed, mass, drag, 0, wid, self.fire_seed);
                        }
                        println!("[FIRE] projectiles_after_fire={}", ballistics.projectiles.len());
                    } else {
                        println!("[FIRE] ERROR: BallisticsSystem not in resources!");
                    }
                    self.hud.push_notification(
                        &format!("[{}] FIRED", weapon_name),
                        NotificationKind::Warning,
                    );
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = self
                    .last_frame
                    .map(|lf| now.duration_since(lf).as_secs_f32())
                    .unwrap_or(1.0 / 60.0);
                self.last_frame = Some(now);

                // Mouse look
                if self.input.mouse_captured {
                    self.camera.yaw += self.input.mouse_dx as f32 * self.camera.sensitivity;
                    self.camera.pitch -= self.input.mouse_dy as f32 * self.camera.sensitivity;
                    self.camera.pitch = self.camera.pitch.clamp(-1.4, 1.4);
                }

                // Grounded movement (horizontal only)
                let fwd = self.camera.forward();
                let flat_fwd = glam::Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero();
                let flat_right = flat_fwd.cross(glam::Vec3::Y).normalize_or_zero();
                let sprinting = self.input.is_key_down(KeyCode::ShiftLeft);
                let speed = if sprinting { 8.0_f32 } else { 4.0 };
                let step = speed * dt;

                if self.input.is_key_down(KeyCode::KeyW) { self.camera.position += flat_fwd * step; }
                if self.input.is_key_down(KeyCode::KeyS) { self.camera.position -= flat_fwd * step; }
                if self.input.is_key_down(KeyCode::KeyD) { self.camera.position += flat_right * step; }
                if self.input.is_key_down(KeyCode::KeyA) { self.camera.position -= flat_right * step; }

                // Player-object collision (push-back)
                for &e in &self.engine.ecs.alive {
                    if let Some(t) = self.engine.ecs.transforms.get(&e) {
                        let dx = self.camera.position.x - t.x;
                        let dz = self.camera.position.z - t.y;
                        let dist_sq = dx * dx + dz * dz;
                        let radius = 1.2_f32;
                        if dist_sq < radius * radius && dist_sq > 0.0001 {
                            let dist = dist_sq.sqrt();
                            let push = radius - dist;
                            self.camera.position.x += (dx / dist) * push;
                            self.camera.position.z += (dz / dist) * push;
                        }
                    }
                }

                // Clamp to sandbox and snap to terrain
                self.camera.position.x = self.camera.position.x.clamp(0.5, 49.5);
                self.camera.position.z = self.camera.position.z.clamp(0.5, 49.5);
                let terrain_y = self.heightmap.sample(self.camera.position.x, self.camera.position.z);
                self.camera.position.y = terrain_y + 1.7;

                self.player.tick(dt, sprinting);
                self.player.position = [
                    self.camera.position.x,
                    self.camera.position.y,
                    self.camera.position.z,
                ];

                if self.player.state == engene::game::player::PlayerState::Dead {
                    if self.input.was_key_pressed(KeyCode::KeyR) {
                        let ry = self.heightmap.sample(25.0, 25.0) + 1.7;
                        self.player.respawn([25.0, ry, 25.0]);
                        self.camera.position = glam::Vec3::new(25.0, ry, 25.0);
                        self.hud.push_notification("Respawned in sandbox", NotificationKind::Info);
                    }
                }

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
                    match save.save_to_file("game/saves/sandbox_quicksave.json") {
                        Ok(()) => self.hud.push_notification("Sandbox saved", NotificationKind::Info),
                        Err(e) => self.hud.push_notification(
                            &format!("Save failed: {}", e),
                            NotificationKind::Warning,
                        ),
                    }
                }
                if self.input.was_key_pressed(KeyCode::F9) {
                    match PlayerSave::load_from_file("game/saves/sandbox_quicksave.json") {
                        Ok(save) => {
                            self.player = save.restore_controller();
                            self.player_inventory = save.restore_inventory();
                            self.camera.position = glam::Vec3::new(
                                self.player.position[0],
                                self.player.position[1],
                                self.player.position[2],
                            );
                            self.hud.push_notification("Sandbox loaded", NotificationKind::Info);
                        }
                        Err(e) => self.hud.push_notification(
                            &format!("Load failed: {}", e),
                            NotificationKind::Warning,
                        ),
                    }
                }

                self.play_time += dt as f64;
                self.hud.tick(dt);
                self.input.end_frame();

                const SIM_DT: f32 = 1.0 / 20.0;
                self.sim_accum += dt.min(0.25);
                while self.sim_accum >= SIM_DT {
                    self.sim_accum -= SIM_DT;
                    self.engine.tick(SIM_DT);
                }
                self.frame += 1;

                // Projectile-entity hit detection
                {
                    let mut hits: Vec<(usize, u64, [f32; 3], String)> = Vec::new();
                    if let Some(ballistics) = self.engine.resources.get::<BallisticsSystem>() {
                        for (pi, proj) in ballistics.projectiles.iter().enumerate() {
                            for &e in &self.engine.ecs.alive {
                                if let Some(t) = self.engine.ecs.transforms.get(&e) {
                                    let ey = self.heightmap.sample(t.x, t.y) + 1.0;
                                    let dx = proj.pos.x - t.x;
                                    let dy = proj.pos.y - ey;
                                    let dz = proj.pos.z - t.y;
                                    let dist_sq = dx * dx + dy * dy + dz * dz;
                                    if dist_sq < 1.5 * 1.5 {
                                        let name = self.engine.ecs.names.get(&e)
                                            .map(|n| n.0.clone())
                                            .unwrap_or_default();
                                        hits.push((pi, e, [t.x, ey, t.y], name));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if !hits.is_empty() {
                        let mut proj_indices: Vec<usize> = Vec::new();
                        for (pi, entity, pos, name) in &hits {
                            proj_indices.push(*pi);
                            self.engine.ecs.despawn(*entity);
                            for i in 0..4u32 {
                                let angle = i as f32 * std::f32::consts::FRAC_PI_2;
                                let debris = self.engine.ecs.spawn();
                                self.engine.ecs.transforms.insert(debris, Transform {
                                    x: pos[0] + angle.cos() * 0.6,
                                    y: pos[2] + angle.sin() * 0.6,
                                    cell_x: 0,
                                    cell_y: 0,
                                });
                                self.engine.ecs.names.insert(debris, Name(format!("debris_{}", name)));
                            }
                            self.hud.push_notification(
                                &format!("DESTROYED: {}", name),
                                NotificationKind::Warning,
                            );
                            println!("[hit] destroyed '{}' at ({:.1},{:.1},{:.1})", name, pos[0], pos[1], pos[2]);
                        }
                        if let Some(ballistics) = self.engine.resources.get_mut::<BallisticsSystem>() {
                            proj_indices.sort_unstable();
                            proj_indices.dedup();
                            for &pi in proj_indices.iter().rev() {
                                if pi < ballistics.projectiles.len() {
                                    ballistics.projectiles.swap_remove(pi);
                                }
                            }
                        }
                    }
                }

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
                            persistence.save_and_unload(*coord, &mut self.engine.ecs, tick);
                        }
                        for coord in &to_load {
                            persistence.load_chunk_entities(*coord, &mut self.engine.ecs);
                        }
                        self.engine.resources.insert_runtime(persistence);
                    }
                }

                if let Some(spatial) =
                    self.engine.resources.get_mut::<HierarchicalSpatialIndex>()
                {
                    spatial.clear();
                    for &e in &self.engine.ecs.alive {
                        if let Some(t) = self.engine.ecs.transforms.get(&e) {
                            spatial.insert(e, t.x, t.y);
                        }
                    }
                }

                if let Some(audio) = self.engine.resources.get_mut::<AudioEngine>() {
                    audio.set_listener(cam_pos, cam_fwd);
                    audio.update(dt);
                }

                {
                    let proj_count = self.engine.resources.get::<BallisticsSystem>()
                        .map(|b| b.projectiles.len()).unwrap_or(0);
                    if self.frame % 300 == 0 {
                        let wname = self.player.current_weapon_name().unwrap_or("none");
                        println!(
                            "[sandbox] frame {} | entities:{} | projectiles:{} | pos:({:.1},{:.1},{:.1}) | weapon:{} | day:{:.2} | clouds:{:.0}%",
                            self.frame,
                            self.engine.ecs.alive.len(),
                            proj_count,
                            self.player.position[0],
                            self.player.position[1],
                            self.player.position[2],
                            wname,
                            self.day_progress,
                            self.weather_controller_coverage() * 100.0,
                        );
                    }
                }

                if let Some(r) = self.renderer.as_mut() {
                    let vp = self.camera.view_projection();
                    let frustum = Frustum::from_view_projection(&vp);
                    let bal_ref = self.engine.resources.get::<BallisticsSystem>();
                    let instances = collect_sandbox_instances(
                        &self.engine.ecs,
                        &self.heightmap,
                        [cam_pos.x, cam_pos.y, cam_pos.z],
                        &frustum,
                        bal_ref,
                    );
                    r.update_entities(&instances);

                    if self.day_cycle_enabled {
                        self.day_progress += dt * 0.05;
                        if self.day_progress > 1.0 {
                            self.day_progress -= 1.0;
                        }
                    }

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
                        day_progress: self.day_progress,
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

fn collect_sandbox_instances(
    ecs: &engene::core::ecs::Ecs,
    heightmap: &Heightmap,
    camera_pos: [f32; 3],
    frustum: &Frustum,
    ballistics: Option<&BallisticsSystem>,
) -> Vec<EntityInstance> {
    let lod_config = LodConfig::default();
    let cam = glam::Vec3::from(camera_pos);
    let mut out = Vec::with_capacity(ecs.alive.len());
    for &e in &ecs.alive {
        let t = match ecs.transforms.get(&e) {
            Some(t) => t,
            None => continue,
        };
        let y = heightmap.sample(t.x, t.y) + 1.0;
        let pos = glam::Vec3::new(t.x, y, t.y);
        let dist = (pos - cam).length();
        if lod_config.compute_lod(dist) == LodLevel::Culled {
            continue;
        }
        if !frustum.test_sphere(pos, 2.0) {
            continue;
        }
        let color = if let Some(EntityKind::Npc) = ecs.kinds.get(&e) {
            [0.16, 0.47, 1.0]
        } else if let Some(EntityKind::Monster(_)) = ecs.kinds.get(&e) {
            [0.83, 0.0, 0.0]
        } else {
            prefab_color(ecs.names.get(&e).map(|n| n.0.as_str()))
        };
        out.push(EntityInstance {
            position: [t.x, y, t.y],
            color,
        });
    }

    // Render active projectiles as bright yellow instances
    if let Some(bal) = ballistics {
        for proj in &bal.projectiles {
            out.push(EntityInstance {
                position: [proj.pos.x, proj.pos.y, proj.pos.z],
                color: [1.0, 0.9, 0.2],
            });
        }
    }

    out
}

fn prefab_color(name: Option<&str>) -> [f32; 3] {
    let n = match name {
        Some(s) => s.to_lowercase(),
        None => return [0.5, 0.5, 0.5],
    };
    if n.contains("wall") || n.contains("tile") || n.contains("brick") {
        [0.85, 0.82, 0.75]
    } else if n.contains("concrete") {
        [0.65, 0.65, 0.65]
    } else if n.contains("glass") {
        [0.3, 0.7, 0.9]
    } else if n.contains("steel") || n.contains("metal") || n.contains("cabinet") {
        [0.55, 0.56, 0.58]
    } else if n.contains("table") || n.contains("chair") || n.contains("crate") || n.contains("wood") {
        [0.55, 0.35, 0.15]
    } else if n.contains("barrel") {
        [0.35, 0.40, 0.30]
    } else if n.contains("sandbag") || n.contains("sand") {
        [0.76, 0.70, 0.50]
    } else if n.contains("light") || n.contains("lamp") {
        [1.0, 0.95, 0.6]
    } else if n.contains("soil") || n.contains("earth") || n.contains("gravel") {
        [0.45, 0.35, 0.25]
    } else if n.contains("cloth") {
        [0.4, 0.35, 0.45]
    } else {
        [0.6, 0.6, 0.6]
    }
}
