//! Game application state and event handling.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: none

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

use crate::audio::audio::AudioEngine;
use crate::core::engine::Engine;
use crate::graphics::camera::FlyCamera;
use crate::graphics::renderer::{RenderCamera, Renderer};
use crate::graphics::visibility::Frustum;
use crate::input::input::InputState;
use crate::memory::asset_manager::AssetManager;
use crate::world::chunk_persistence::ChunkPersistenceService;
use crate::world::hierarchical_spatial::HierarchicalSpatialIndex;
use crate::world::streaming::WorldStreamer;

type ArcHeightmap = Arc<crate::world::heightmap::Heightmap>;

/// Main game application state.
pub struct GameApp {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer>,
    pub engine: Engine,
    pub camera: FlyCamera,
    pub input: InputState,
    pub last_frame: Option<Instant>,
    pub frame: u64,
    pub sim_accum: f32,
    pub last_report_month: u32,
    pub heightmap: ArcHeightmap,
    pub biomes: Vec<crate::world::biome::Biome>,
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("ENGENE 3D")
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
                self.tick_frame(event_loop);
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

impl GameApp {
    pub fn tick_frame(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let now = std::time::Instant::now();
        let dt = self
            .last_frame
            .map(|lf| now.duration_since(lf).as_secs_f32())
            .unwrap_or(1.0 / 60.0);
        self.last_frame = Some(now);

        self.camera.update(&self.input, dt);
        self.input.end_frame();

        const SIM_DT: f32 = 1.0 / 20.0;
        self.sim_accum += dt.min(0.25);
        while self.sim_accum >= SIM_DT {
            self.sim_accum -= SIM_DT;
            self.engine.tick(SIM_DT);
        }
        self.frame += 1;

        self.update_streaming();
        self.update_spatial_index();
        self.update_audio(dt);

        let month = self.engine.time.month;
        if month != self.last_report_month {
            self.last_report_month = month;
            println!(
                "\n========== MONTH {} ({}) ==========",
                month,
                self.engine.time.season()
            );
            super::debug_output::print_economy(&self.engine);
        }

        if self.frame % 120 == 0 {
            super::debug_output::print_brief(&self.engine, self.frame);
        }

        self.render_frame(event_loop);
    }

    fn update_streaming(&mut self) {
        let cam_pos = self.camera.position;
        let (to_load, to_unload) = {
            if let Some(streamer) = self.engine.resources.get_mut::<WorldStreamer>() {
                let result = streamer.update(cam_pos.x, cam_pos.z);
                for coord in &result.0 {
                    streamer.mark_loaded(*coord);
                }
                result
            } else {
                return;
            }
        };

        if to_unload.is_empty() && to_load.is_empty() {
            return;
        }

        if let Some(mut persistence) = self.engine.resources.take::<ChunkPersistenceService>() {
            let tick = self.engine.ecs.tick;
            for coord in &to_unload {
                let saved = persistence.save_and_unload(*coord, &mut self.engine.ecs, tick);
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
                let loaded = persistence.load_chunk_entities(*coord, &mut self.engine.ecs);
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

    fn update_spatial_index(&mut self) {
        if let Some(spatial) = self.engine.resources.get_mut::<HierarchicalSpatialIndex>() {
            spatial.clear();
            for &e in &self.engine.ecs.alive {
                if let Some(t) = self.engine.ecs.get_transform(e) {
                    spatial.insert(e, t.x, t.y);
                }
            }
        }
    }

    fn update_audio(&mut self, dt: f32) {
        if let Some(audio) = self.engine.resources.get_mut::<AudioEngine>() {
            audio.set_listener(self.camera.position, self.camera.forward());
            audio.update(dt);
        }
    }

    fn render_frame(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(r) = self.renderer.as_mut() else {
            return;
        };

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
        let vp = self.camera.view_projection();
        let frustum = Frustum::from_view_projection(&vp);
        let instances = super::debug_output::collect_entity_instances(
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
