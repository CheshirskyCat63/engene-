use glam::{Mat4, Vec3};
use winit::keyboard::KeyCode;

use crate::input::input::InputState;

pub struct FlyCamera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl FlyCamera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(
                crate::world::cell::WORLD_SIZE * 0.5,
                200.0,
                crate::world::cell::WORLD_SIZE * 0.5 + 100.0,
            ),
            yaw: 0.0,
            pitch: -0.15,
            speed: 200.0,
            sensitivity: 0.003,
            fov: 70.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 5000.0,
        }
    }

    pub fn update(&mut self, input: &InputState, dt: f32) {
        self.yaw += input.mouse_dx as f32 * self.sensitivity;
        self.pitch -= input.mouse_dy as f32 * self.sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);

        let fwd = self.forward();
        let right = self.right();
        let speed = if input.is_key_down(KeyCode::ShiftLeft) {
            self.speed * 3.0
        } else {
            self.speed
        };
        let step = speed * dt;

        if input.is_key_down(KeyCode::KeyW) {
            self.position += fwd * step;
        }
        if input.is_key_down(KeyCode::KeyS) {
            self.position -= fwd * step;
        }
        if input.is_key_down(KeyCode::KeyD) {
            self.position += right * step;
        }
        if input.is_key_down(KeyCode::KeyA) {
            self.position -= right * step;
        }
        if input.is_key_down(KeyCode::KeyE) || input.is_key_down(KeyCode::Space) {
            self.position.y += step;
        }
        if input.is_key_down(KeyCode::KeyQ) || input.is_key_down(KeyCode::ControlLeft) {
            self.position.y -= step;
        }
    }

    pub fn view_projection(&self) -> Mat4 {
        let view = Mat4::look_to_rh(self.position, self.forward(), Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
        proj * view
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            -(self.pitch.cos() * self.yaw.sin()),
            self.pitch.sin(),
            -(self.pitch.cos() * self.yaw.cos()),
        )
        .normalize()
    }

    fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }
}
