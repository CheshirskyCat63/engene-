use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
    Free,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerState {
    Alive,
    Dead,
    InMenu,
    InDialogue,
    InTrade,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerController {
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub bleeding: f32,
    pub camera_mode: CameraMode,
    pub state: PlayerState,
    pub move_speed: f32,
    pub sprint_speed: f32,
    pub interaction_range: f32,
    pub weight_carried: f32,
    pub max_weight: f32,
    pub weapon_slots: [Option<String>; 4],
    pub current_weapon_slot: usize,
    pub fire_cooldown: f32,
}

impl PlayerController {
    pub fn new(spawn: [f32; 3]) -> Self {
        Self {
            position: spawn,
            rotation: [0.0, 0.0],
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            bleeding: 0.0,
            camera_mode: CameraMode::FirstPerson,
            state: PlayerState::Alive,
            move_speed: 4.0,
            sprint_speed: 7.0,
            interaction_range: 3.0,
            weight_carried: 0.0,
            max_weight: 50.0,
            weapon_slots: [
                Some("Makarov".to_string()),
                Some("AK74".to_string()),
                Some("Shotgun".to_string()),
                Some("Grenade".to_string()),
            ],
            current_weapon_slot: 0,
            fire_cooldown: 0.0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.state == PlayerState::Alive
    }

    pub fn can_move(&self) -> bool {
        self.state == PlayerState::Alive && self.stamina > 0.0
    }

    pub fn can_sprint(&self) -> bool {
        self.can_move() && self.stamina > 10.0 && self.weight_carried < self.max_weight * 0.8
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        if self.health <= 0.0 {
            self.state = PlayerState::Dead;
        }
    }

    pub fn heal(&mut self, amount: f32) {
        if self.state == PlayerState::Alive {
            self.health = (self.health + amount).min(self.max_health);
        }
    }

    pub fn tick(&mut self, dt: f32, sprinting: bool) {
        if self.state != PlayerState::Alive {
            return;
        }

        self.tick_fire_cooldown(dt);

        if self.bleeding > 0.0 {
            self.health = (self.health - self.bleeding * dt).max(0.0);
            self.bleeding = (self.bleeding - 0.01 * dt).max(0.0);
            if self.health <= 0.0 {
                self.state = PlayerState::Dead;
            }
        }

        if sprinting && self.can_sprint() {
            self.stamina = (self.stamina - 15.0 * dt).max(0.0);
        } else {
            self.stamina = (self.stamina + 8.0 * dt).min(self.max_stamina);
        }
    }

    pub fn health_fraction(&self) -> f32 {
        self.health / self.max_health
    }

    pub fn stamina_fraction(&self) -> f32 {
        self.stamina / self.max_stamina
    }

    pub fn weight_fraction(&self) -> f32 {
        if self.max_weight <= 0.0 {
            return 0.0;
        }
        self.weight_carried / self.max_weight
    }

    pub fn enter_dialogue(&mut self) {
        self.state = PlayerState::InDialogue;
    }
    pub fn enter_trade(&mut self) {
        self.state = PlayerState::InTrade;
    }
    pub fn exit_interaction(&mut self) {
        self.state = PlayerState::Alive;
    }
    pub fn respawn(&mut self, pos: [f32; 3]) {
        self.position = pos;
        self.health = self.max_health;
        self.stamina = self.max_stamina;
        self.bleeding = 0.0;
        self.state = PlayerState::Alive;
        self.fire_cooldown = 0.0;
    }

    pub fn switch_weapon(&mut self, slot: usize) {
        if slot < self.weapon_slots.len() && self.weapon_slots[slot].is_some() {
            self.current_weapon_slot = slot;
            self.fire_cooldown = 0.3;
        }
    }

    pub fn current_weapon_name(&self) -> Option<&str> {
        self.weapon_slots[self.current_weapon_slot].as_deref()
    }

    pub fn can_fire(&self) -> bool {
        self.state == PlayerState::Alive && self.fire_cooldown <= 0.0
    }

    pub fn tick_fire_cooldown(&mut self, dt: f32) {
        if self.fire_cooldown > 0.0 {
            self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        }
    }
}
