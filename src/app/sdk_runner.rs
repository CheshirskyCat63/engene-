//! SDK runtime runner for the ENGENE editor application.

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

use crate::audio::audio::AudioEngine;
use crate::core::build_manifest::BuildManifest;
use crate::core::crash_telemetry;
use crate::core::engine::Engine;
use crate::graphics::camera::FlyCamera;
use crate::graphics::lod::{LodConfig, LodLevel};
use crate::graphics::mesh::EntityInstance;
use crate::graphics::renderer::{RenderCamera, Renderer};
use crate::graphics::visibility::Frustum;
use crate::input::input::InputState;
use crate::memory::asset_manager::AssetManager;
use crate::runtime::bootstrap::ToolsRuntimeAssembly;
use crate::tools::doctor;
use crate::tools::editor_shell::EditorShell;
use crate::world::chunk_persistence::ChunkPersistenceService;
use crate::world::components::*;
use crate::world::heightmap::Heightmap;
use crate::world::hierarchical_spatial::HierarchicalSpatialIndex;
use crate::world::streaming::WorldStreamer;

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

    let heightmap: ArcHeightmap = Arc::new(Heightmap::flat(crate::world::cell::WORLD_SIZE));
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

                if !self.sim_paused {
                    const SIM_DT: f32 = 1.0 / 20.0;
                    self.sim_accum += (dt * self.sim_speed).min(0.25);
                    while self.sim_accum >= SIM_DT {
                        self.sim_accum -= SIM_DT;
                        self.engine.tick(SIM_DT);
                    }
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

                // World streaming
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

                // Rebuild spatial index
                if let Some(spatial) = self.engine.resources.get_mut::<HierarchicalSpatialIndex>() {
                    spatial.clear();
                    for &e in &self.engine.ecs.alive {
                        if let Some(t) = self.engine.ecs.get_transform(e) {
                            spatial.insert(e, t.x, t.y);
                        }
                    }
                }

                if let Some(audio) = self.engine.resources.get_mut::<AudioEngine>() {
                    audio.set_listener(cam_pos, cam_fwd);
                    audio.update(dt);
                }

                // Feed live data into dashboard panels
                self.editor_shell.update_dashboards(&self.engine);

                // Apply any pending inspector edits
                self.editor_shell.apply_inspector_edits(&mut self.engine);

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

                    let telemetry = crate::core::perf::telemetry::Telemetry::new();
                    let ecs_ref = &self.engine.ecs;
                    let events_ref = &self.engine.events;
                    let shell = &mut self.editor_shell;

                    match r.render_with_egui(&render_cam, |ctx| {
                        shell.draw_with_event_bus(ctx, ecs_ref, &telemetry, events_ref);
                    }) {
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

fn collect_entity_instances(
    ecs: &crate::core::ecs::Ecs,
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
