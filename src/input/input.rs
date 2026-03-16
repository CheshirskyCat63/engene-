use std::collections::HashSet;
use winit::keyboard::KeyCode;

pub struct InputState {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    pub mouse_dx: f64,
    pub mouse_dy: f64,
    pub mouse_captured: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            mouse_captured: false,
        }
    }

    pub fn key_pressed(&mut self, key: KeyCode) {
        if self.pressed.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    pub fn key_released(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn was_key_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn accumulate_mouse(&mut self, dx: f64, dy: f64) {
        if self.mouse_captured {
            self.mouse_dx += dx;
            self.mouse_dy += dy;
        }
    }

    pub fn end_frame(&mut self) {
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
        self.just_pressed.clear();
    }
}
