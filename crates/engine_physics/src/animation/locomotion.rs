#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocomotionState {
    Idle,
    Walk,
    Run,
    Death,
}

pub struct LocomotionMachine {
    pub state: LocomotionState,
    pub transition_timer: f32,
    clip_map: [Option<usize>; 4],
}

impl LocomotionMachine {
    pub fn new() -> Self {
        Self {
            state: LocomotionState::Idle,
            transition_timer: 0.0,
            clip_map: [None; 4],
        }
    }

    pub fn set_clip(&mut self, state: LocomotionState, clip_index: usize) {
        self.clip_map[state as usize] = Some(clip_index);
    }

    pub fn update(&mut self, speed: f32, is_dead: bool, dt: f32) {
        let target = if is_dead {
            LocomotionState::Death
        } else if speed < 0.5 {
            LocomotionState::Idle
        } else if speed < 3.0 {
            LocomotionState::Walk
        } else {
            LocomotionState::Run
        };

        if target != self.state {
            self.state = target;
            self.transition_timer = 0.0;
        }
        self.transition_timer = (self.transition_timer + dt).min(1.0);
    }

    pub fn current_clip(&self) -> Option<usize> {
        self.clip_map[self.state as usize]
    }
}
