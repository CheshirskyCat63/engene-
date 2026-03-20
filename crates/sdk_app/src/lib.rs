//! SDK role owner crate.
//! This crate is transitioning to own SDK runtime and editor startup.

pub mod api {
    pub const CRATE: &str = "sdk_app";
    pub const STATUS: &str = "transitional_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "sdk_app";
    pub const TARGET_OWNER: &str = "sdk_app";
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

use engene::core::build_manifest::BuildManifest;
use engene::core::crash_telemetry;
use engene::core::engine::Engine;
use engene::graphics::camera::FlyCamera;
use engene::graphics::lod::{LodConfig, LodLevel};
use engene::graphics::mesh::EntityInstance;
use engene::graphics::renderer::Renderer;
use engene::graphics::visibility::Frustum;
use engene::input::input::InputState;
use engene::memory::asset_manager::AssetManager;
use engene::runtime::bootstrap::ToolsRuntimeAssembly;
use engene::tools::doctor;
use engene::tools::editor_shell::EditorShell;
use engene::app::spatial_dirty_journal::SpatialDirtyJournal;
use engene::world::components::*;
use engene::world::heightmap::Heightmap;
use engene::world::hierarchical_spatial::SpatialUpdatePath;

type ArcHeightmap = Arc<Heightmap>;

pub fn run_from_env_args() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    crash_telemetry::install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    puffin::set_scopes_on(true);

    let manifest = BuildManifest::current();
    println!("=== ENGENE SDK ===");
    manifest.print_full();
    println!();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let heightmap: ArcHeightmap = Arc::new(Heightmap::flat(engene::world::cell::WORLD_SIZE));
    let engine = ToolsRuntimeAssembly::minimal();

    let doctor_report = doctor::run_doctor(&engine, doctor::DoctorMode::Strict);
    println!(
        "[doctor] startup (strict): {} errors, {} warnings",
        doctor_report.error_count(),
        doctor_report.warning_count()
    );
    doctor_report.print();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut editor_shell = EditorShell::new();
    editor_shell.last_doctor_report = Some(doctor_report);

    let mut app = SdkApp {
        window: None,
        renderer: None,
        engine,
        camera: FlyCamera::new(),
        input: InputState::new(),
        last_frame: None,
        frame: 0,
        sim_accum: 0.0,
        sim_paused: false,
        sim_speed: 1.0,
        spatial_dirty_journal: SpatialDirtyJournal {
            force_rebuild: true,
            ..SpatialDirtyJournal::default()
        },
        spatial_last_applied_positions: HashMap::new(),
        spatial_last_origin_shift_count: 0,
        spatial_last_update_path: SpatialUpdatePath::FullRebuild,
        heightmap,
        editor_shell,
    };

    println!("[sdk] starting editor — click to capture mouse, ESC to release, P to pause sim\n");
    let _ = event_loop.run_app(&mut app);
    println!("\n=== SDK SESSION ENDED ===");
}

struct SdkApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    engine: Engine,
    camera: FlyCamera,
    input: InputState,
    last_frame: Option<Instant>,
    frame: u64,
    sim_accum: f32,
    sim_paused: bool,
    sim_speed: f32,
    spatial_dirty_journal: SpatialDirtyJournal,
    spatial_last_applied_positions: HashMap<engene::core::ecs::Entity, (f32, f32)>,
    spatial_last_origin_shift_count: u32,
    spatial_last_update_path: SpatialUpdatePath,
    heightmap: ArcHeightmap,
    editor_shell: EditorShell,
}

impl ApplicationHandler for SdkApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("ENGENE SDK — Editor")
            .with_inner_size(winit::dpi::LogicalSize::new(1600, 900));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        // Tools runtime purity:
        // avoid booting renderer weather/sky prototype stack in tools mode.
        self.renderer = None;
        self.window = Some(window);
        self.last_frame = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let egui_consumed = if let Some(r) = self.renderer.as_mut() {
            let resp = r.on_window_event(&event);
            resp.consumed
        } else {
            false
        };

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
            } if !egui_consumed => match state {
                ElementState::Pressed => {
                    match key {
                        KeyCode::Escape => {
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
                        KeyCode::KeyP => {
                            self.sim_paused = !self.sim_paused;
                            println!(
                                "[sdk] simulation {}",
                                if self.sim_paused { "PAUSED" } else { "RESUMED" }
                            );
                        }
                        KeyCode::BracketRight => {
                            self.sim_speed = (self.sim_speed * 2.0).min(16.0);
                            println!("[sdk] sim speed: {:.1}x", self.sim_speed);
                        }
                        KeyCode::BracketLeft => {
                            self.sim_speed = (self.sim_speed * 0.5).max(0.125);
                            println!("[sdk] sim speed: {:.1}x", self.sim_speed);
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
            } if !egui_consumed => {
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
                self.input.end_frame();

                const SIM_DT: f32 = 1.0 / 20.0;
                if !self.sim_paused {
                    self.sim_accum += (dt * self.sim_speed).min(0.25);
                    while self.sim_accum >= SIM_DT {
                        self.sim_accum -= SIM_DT;
                        engene::app::sdk_runner::sdk_runner_phases::tick::run(self, SIM_DT);
                    }
                }
                self.frame += 1;

                if let Some(am) =
                    self.engine
                        .resources
                        .get_mut::<std::sync::Mutex<AssetManager>>()
                {
                    if let Ok(mut am) = am.lock() {
                        am.poll();
                    }
                }

                let cam_pos = self.camera.position;
                let cam_fwd = self.camera.forward();

                // World streaming
                let (to_load, to_unload) = engene::app::sdk_runner::sdk_runner_phases::streaming::run(self, cam_pos);

                // Persistence
                engene::app::sdk_runner::sdk_runner_phases::persistence::run(self, &to_load, &to_unload);

                // Audio
                engene::app::sdk_runner::sdk_runner_phases::audio::run(self, cam_pos, cam_fwd, dt);

                // Editor
                engene::app::sdk_runner::sdk_runner_phases::editor::run(self);

                // Spatial policy (no-op / incremental / controlled full rebuild)
                engene::app::sdk_runner::sdk_runner_phases::spatial::run(self);

                // Render
                let vp = self.camera.view_projection();
                engene::app::sdk_runner::sdk_runner_phases::render::run(self, vp, cam_pos, dt);
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

pub fn collect_entity_instances(
    ecs: &engene::core::ecs::Ecs,
    heightmap: &Heightmap,
    camera_pos: [f32; 3],
    frustum: &Frustum,
) -> Vec<EntityInstance> {
    let lod_config = LodConfig::default();
    let cam = glam::Vec3::from(camera_pos);
    let mut out = Vec::with_capacity(ecs.alive.len());
    for &e in &ecs.alive {
        let t = match ecs.get_transform(e) {
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
        let color = match ecs.get_kind(e) {
            Some(EntityKind::Npc) => [0.16, 0.47, 1.0],
            Some(EntityKind::Monster(MonsterSpecies::Wolf)) => [0.9, 0.9, 0.9],
            Some(EntityKind::Monster(MonsterSpecies::Boar)) => [0.55, 0.43, 0.39],
            Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)) => [0.83, 0.0, 0.0],
            None => [0.5, 0.5, 0.5],
        };
        out.push(EntityInstance {
            position: [t.x, y, t.y],
            color,
        });
    }
    out
}
