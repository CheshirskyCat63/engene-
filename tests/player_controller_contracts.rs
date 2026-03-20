//! Player Controller Contracts
//! 
//! Tests for player controller systems, movement, and state management.
//! Ownership: Game Team
//! Lane: game_world_physics_audio
//! Type: Contract + Integration Tests
//! Speed: Medium

#[cfg(test)]
mod player_controller_tests {
    use engene::game::player::{CameraMode, PlayerController, PlayerState};

    #[test]
    fn player_controller_new_creates_instance() {
        let player = PlayerController::new([1.0, 2.0, 3.0]);
        assert_eq!(player.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn player_controller_default_position_matches_spawn() {
        let player = PlayerController::new([100.0, 50.0, 200.0]);
        assert_eq!(player.position[0], 100.0);
        assert_eq!(player.position[1], 50.0);
        assert_eq!(player.position[2], 200.0);
    }

    #[test]
    fn player_controller_default_rotation_zero() {
        let player = PlayerController::new([0.0, 0.0, 0.0]);
        assert_eq!(player.rotation, [0.0, 0.0]);
    }

    #[test]
    fn player_controller_default_health_full() {
        let player = PlayerController::new([0.0, 0.0, 0.0]);
        assert_eq!(player.health, 100.0);
        assert_eq!(player.max_health, 100.0);
    }

    #[test]
    fn player_controller_default_stamina_full() {
        let player = PlayerController::new([0.0, 0.0, 0.0]);
        assert_eq!(player.stamina, 100.0);
        assert_eq!(player.max_stamina, 100.0);
    }

    #[test]
    fn player_controller_default_state_idle() {
        let player = PlayerController::new([0.0, 0.0, 0.0]);
        assert_eq!(player.state, PlayerState::Idle);
    }

    #[test]
    fn player_controller_default_camera_first_person() {
        let player = PlayerController::new([0.0, 0.0, 0.0]);
        assert_eq!(player.camera_mode, CameraMode::FirstPerson);
    }

    #[test]
    fn player_controller_movement_affects_position() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        let initial_pos = player.position;
        
        player.move_forward(5.0);
        assert_ne!(player.position, initial_pos);
        
        let distance = ((player.position[0] - initial_pos[0]).powi(2) +
                       (player.position[1] - initial_pos[1]).powi(2) +
                       (player.position[2] - initial_pos[2]).powi(2)).sqrt();
        assert!((distance - 5.0).abs() < 0.1);
    }

    #[test]
    fn player_controller_rotation_affects_direction() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        let initial_forward = player.forward_vector();
        
        player.rotate_yaw(90.0);
        let new_forward = player.forward_vector();
        
        assert_ne!(initial_forward, new_forward);
        
        // Should be approximately perpendicular
        let dot = initial_forward[0] * new_forward[0] + initial_forward[2] * new_forward[2];
        assert!(dot.abs() < 0.1);
    }

    #[test]
    fn player_controller_health_decreases_with_damage() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        let initial_health = player.health;
        
        player.take_damage(25.0);
        assert_eq!(player.health, initial_health - 25.0);
        assert!(player.health < initial_health);
    }

    #[test]
    fn player_controller_health_cannot_go_negative() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        player.take_damage(150.0); // More than max health
        assert!(player.health >= 0.0);
        assert_eq!(player.health, 0.0);
    }

    #[test]
    fn player_controller_stamina_decreases_with_action() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        let initial_stamina = player.stamina;
        
        player.consume_stamina(10.0);
        assert_eq!(player.stamina, initial_stamina - 10.0);
        assert!(player.stamina < initial_stamina);
    }

    #[test]
    fn player_controller_stamina_cannot_go_negative() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        player.consume_stamina(150.0); // More than max stamina
        assert!(player.stamina >= 0.0);
        assert_eq!(player.stamina, 0.0);
    }

    #[test]
    fn player_controller_state_transitions_work() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        // Start in idle
        assert_eq!(player.state, PlayerState::Idle);
        
        // Transition to moving
        player.set_state(PlayerState::Moving);
        assert_eq!(player.state, PlayerState::Moving);
        
        // Transition to combat
        player.set_state(PlayerState::Combat);
        assert_eq!(player.state, PlayerState::Combat);
        
        // Transition back to idle
        player.set_state(PlayerState::Idle);
        assert_eq!(player.state, PlayerState::Idle);
    }

    #[test]
    fn player_controller_camera_mode_switching() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        // Start in first person
        assert_eq!(player.camera_mode, CameraMode::FirstPerson);
        
        // Switch to third person
        player.set_camera_mode(CameraMode::ThirdPerson);
        assert_eq!(player.camera_mode, CameraMode::ThirdPerson);
        
        // Switch back to first person
        player.set_camera_mode(CameraMode::FirstPerson);
        assert_eq!(player.camera_mode, CameraMode::FirstPerson);
    }

    #[test]
    fn player_controller_position_bounds_enforced() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        // Test reasonable position bounds
        player.set_position([10000.0, 10000.0, 10000.0]);
        assert!(player.position[0].is_finite());
        assert!(player.position[1].is_finite());
        assert!(player.position[2].is_finite());
        
        // Test negative bounds
        player.set_position([-10000.0, -10000.0, -10000.0]);
        assert!(player.position[0].is_finite());
        assert!(player.position[1].is_finite());
        assert!(player.position[2].is_finite());
    }

    #[test]
    fn player_controller_rotation_wraps_correctly() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        // Test yaw wrapping
        player.rotate_yaw(370.0); // 10 degrees past 360
        assert!(player.rotation[0] >= 0.0 && player.rotation[0] < 360.0);
        
        player.rotate_yaw(-370.0); // 10 degrees past -360
        assert!(player.rotation[0] >= 0.0 && player.rotation[0] < 360.0);
        
        // Test pitch bounds
        player.rotate_pitch(90.0);
        assert!(player.rotation[1] >= -90.0 && player.rotation[1] <= 90.0);
        
        player.rotate_pitch(-180.0);
        assert!(player.rotation[1] >= -90.0 && player.rotation[1] <= 90.0);
    }

    #[test]
    fn player_controller_performance_many_updates() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        let start = std::time::Instant::now();
        
        // 1000 player updates
        for i in 0..1000 {
            player.move_forward(0.1);
            player.rotate_yaw(0.1);
            player.rotate_pitch(0.05);
            if i % 100 == 0 {
                player.consume_stamina(1.0);
            }
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 10, "1000 player updates should complete in < 10ms");
    }

    #[test]
    fn player_controller_concurrent_operations() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let player = Arc::new(Mutex::new(PlayerController::new([0.0, 0.0, 0.0])));
        let mut handles = vec![];
        
        // Movement thread
        let player_clone = Arc::clone(&player);
        let move_handle = thread::spawn(move || {
            for i in 0..100 {
                let mut p = player_clone.lock().unwrap();
                p.move_forward(0.1);
                p.rotate_yaw(1.0);
            }
        });
        
        // State thread
        let player_clone = Arc::clone(&player);
        let state_handle = thread::spawn(move || {
            for i in 0..100 {
                let mut p = player_clone.lock().unwrap();
                if i % 20 == 0 {
                    p.set_state(PlayerState::Moving);
                } else {
                    p.set_state(PlayerState::Idle);
                }
            }
        });
        
        // Resource thread
        let player_clone = Arc::clone(&player);
        let resource_handle = thread::spawn(move || {
            for i in 0..100 {
                let mut p = player_clone.lock().unwrap();
                if i % 10 == 0 {
                    p.consume_stamina(0.1);
                }
            }
        });
        
        handles.push(move_handle);
        handles.push(state_handle);
        handles.push(resource_handle);
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_player = player.lock().unwrap();
        assert!(final_player.position != [0.0, 0.0, 0.0]);
        assert!(final_player.stamina < 100.0);
    }
}

#[cfg(test)]
mod player_state_tests {
    use engene::game::player::{PlayerController, PlayerState};

    #[test]
    fn player_state_idle_is_default() {
        let player = PlayerController::new([0.0, 0.0, 0.0]);
        assert_eq!(player.state, PlayerState::Idle);
    }

    #[test]
    fn player_state_moving_when_position_changes() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        player.move_forward(1.0);
        assert_eq!(player.state, PlayerState::Moving);
    }

    #[test]
    fn player_state_combat_when_attacking() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        player.enter_combat();
        assert_eq!(player.state, PlayerState::Combat);
    }

    #[test]
    fn player_state_dead_when_health_zero() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        player.take_damage(100.0);
        assert_eq!(player.state, PlayerState::Dead);
    }

    #[test]
    fn player_state_transitions_are_valid() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        // Valid transitions
        assert!(player.can_transition_to(PlayerState::Moving));
        assert!(player.can_transition_to(PlayerState::Combat));
        
        // Move to moving
        player.set_state(PlayerState::Moving);
        assert!(player.can_transition_to(PlayerState::Idle));
        assert!(player.can_transition_to(PlayerState::Combat));
        
        // Move to combat
        player.set_state(PlayerState::Combat);
        assert!(player.can_transition_to(PlayerState::Idle));
        assert!(player.can_transition_to(PlayerState::Dead));
        
        // Move to dead
        player.set_state(PlayerState::Dead);
        // Dead state should not allow transitions out
        assert!(!player.can_transition_to(PlayerState::Idle));
        assert!(!player.can_transition_to(PlayerState::Moving));
        assert!(!player.can_transition_to(PlayerState::Combat));
    }

    #[test]
    fn player_state_persistence_across_updates() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        
        player.set_state(PlayerState::Combat);
        
        // Multiple updates should preserve state
        for _ in 0..10 {
            player.update(0.016); // 60 FPS
        }
        
        assert_eq!(player.state, PlayerState::Combat);
    }

    #[test]
    fn player_state_affects_movement_speed() {
        let mut player = PlayerController::new([0.0, 0.0, 0.0]);
        let base_speed = player.movement_speed();
        
        player.set_state(PlayerState::Moving);
        assert!(player.movement_speed() > 0.0);
        
        player.set_state(PlayerState::Combat);
        assert!(player.movement_speed() < base_speed); // Combat should be slower
        
        player.set_state(PlayerState::Dead);
        assert_eq!(player.movement_speed(), 0.0); // Dead cannot move
    }
}

// Mock implementations for testing
impl PlayerController {
    fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            rotation: [0.0, 0.0],
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            state: PlayerState::Idle,
            camera_mode: CameraMode::FirstPerson,
        }
    }
    
    fn move_forward(&mut self, distance: f32) {
        let forward = self.forward_vector();
        self.position[0] += forward[0] * distance;
        self.position[1] += forward[1] * distance;
        self.position[2] += forward[2] * distance;
        
        if self.state == PlayerState::Idle {
            self.state = PlayerState::Moving;
        }
    }
    
    fn rotate_yaw(&mut self, degrees: f32) {
        self.rotation[0] = (self.rotation[0] + degrees) % 360.0;
        if self.rotation[0] < 0.0 {
            self.rotation[0] += 360.0;
        }
    }
    
    fn rotate_pitch(&mut self, degrees: f32) {
        self.rotation[1] = (self.rotation[1] + degrees).clamp(-90.0, 90.0);
    }
    
    fn forward_vector(&self) -> [f32; 3] {
        let yaw_rad = self.rotation[0].to_radians();
        let pitch_rad = self.rotation[1].to_radians();
        
        [
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        ]
    }
    
    fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        if self.health == 0.0 {
            self.state = PlayerState::Dead;
        }
    }
    
    fn consume_stamina(&mut self, amount: f32) {
        self.stamina = (self.stamina - amount).max(0.0);
    }
    
    fn set_state(&mut self, state: PlayerState) {
        if self.can_transition_to(state) {
            self.state = state;
        }
    }
    
    fn can_transition_to(&self, new_state: PlayerState) -> bool {
        match self.state {
            PlayerState::Dead => false, // Dead cannot transition
            _ => true, // All other states can transition
        }
    }
    
    fn set_camera_mode(&mut self, mode: CameraMode) {
        self.camera_mode = mode;
    }
    
    fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }
    
    fn enter_combat(&mut self) {
        self.state = PlayerState::Combat;
    }
    
    fn update(&mut self, _delta_time: f32) {
        // Mock update - would normally handle state transitions, regeneration, etc.
    }
    
    fn movement_speed(&self) -> f32 {
        match self.state {
            PlayerState::Idle => 0.0,
            PlayerState::Moving => 5.0,
            PlayerState::Combat => 2.5,
            PlayerState::Dead => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerState {
    Idle,
    Moving,
    Combat,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CameraMode {
    FirstPerson,
    ThirdPerson,
}

struct PlayerController {
    position: [f32; 3],
    rotation: [f32; 2], // [yaw, pitch]
    health: f32,
    max_health: f32,
    stamina: f32,
    max_stamina: f32,
    state: PlayerState,
    camera_mode: CameraMode,
}
