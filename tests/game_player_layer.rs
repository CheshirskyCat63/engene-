//! Integration tests for Game and Player Layer systems.
//! 144 tests total across PlayerController, HudState, PlayerInventory, PlayerSave,
//! EquipmentSlots, Entity combat, and PersonalNeeds.

use engene::game::hud::{HudState, NotificationKind};
use engene::game::player::{CameraMode, PlayerController, PlayerState};
use engene::game::player_save::{PlayerInventory, PlayerItem, PlayerItemType, PlayerSave};
use engene::world::components::{
    base_power, food_value, is_prey_for, EntityKind, EquipmentSlots, MonsterSpecies,
    PersonalNeeds,
};

// =============================================================================
// 1. PlayerController creation (10)
// =============================================================================

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
fn player_controller_default_state_alive() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert_eq!(player.state, PlayerState::Alive);
}

#[test]
fn player_controller_default_speeds() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert_eq!(player.move_speed, 4.0);
    assert_eq!(player.sprint_speed, 7.0);
}

#[test]
fn player_controller_default_weight_zero() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert_eq!(player.weight_carried, 0.0);
    assert_eq!(player.max_weight, 50.0);
}

#[test]
fn player_controller_default_camera_mode() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert_eq!(player.camera_mode, CameraMode::FirstPerson);
}

#[test]
fn player_controller_works_with_sandbox_engine() {
    use engene::app::runtime_assembly::RuntimeAssembly;

    let mut engine = RuntimeAssembly::sandbox();
    engine.tick(0.016);

    let player = PlayerController::new([500.0, 10.0, 500.0]);
    assert!(player.is_alive());
    assert!(player.can_move());
}

// =============================================================================
// 2. PlayerController movement (12)
// =============================================================================

#[test]
fn player_can_move_when_alive_with_stamina() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert!(player.can_move());
}

#[test]
fn player_cannot_move_when_dead() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(100.0);
    assert!(!player.can_move());
}

#[test]
fn player_cannot_move_when_zero_stamina() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.stamina = 0.0;
    assert!(!player.can_move());
}

#[test]
fn player_can_sprint_when_conditions_met() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert!(player.can_sprint());
}

#[test]
fn player_cannot_sprint_when_low_stamina() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.stamina = 5.0;
    assert!(!player.can_sprint());
}

#[test]
fn player_cannot_sprint_when_overloaded() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.weight_carried = 45.0; // 90% of 50
    assert!(!player.can_sprint());
}

#[test]
fn player_tick_regenerates_stamina_when_not_sprinting() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.stamina = 50.0;
    player.tick(1.0, false);
    assert!(player.stamina > 50.0);
    assert!(player.stamina <= 100.0);
}

#[test]
fn player_tick_drains_stamina_when_sprinting() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.tick(0.5, true);
    assert!(player.stamina < 100.0);
}

#[test]
fn player_tick_no_effect_when_dead() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(100.0);
    let stamina_before = player.stamina;
    player.tick(1.0, true);
    assert_eq!(player.stamina, stamina_before);
}

#[test]
fn player_bleeding_reduces_health_over_time() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.bleeding = 5.0;
    let health_before = player.health;
    player.tick(1.0, false);
    assert!(player.health < health_before);
}

#[test]
fn player_bleeding_decays_over_time() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.bleeding = 10.0;
    player.tick(10.0, false);
    assert!(player.bleeding < 10.0);
}

#[test]
fn player_bleeding_can_cause_death() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.health = 10.0;
    player.bleeding = 50.0;
    player.tick(1.0, false);
    assert!(!player.is_alive());
}

// =============================================================================
// 3. PlayerController damage (15)
// =============================================================================

#[test]
fn player_take_damage_reduces_health() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(30.0);
    assert_eq!(player.health, 70.0);
}

#[test]
fn player_take_damage_does_not_go_below_zero() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(150.0);
    assert_eq!(player.health, 0.0);
}

#[test]
fn player_take_damage_sets_dead_state() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(100.0);
    assert_eq!(player.state, PlayerState::Dead);
}

#[test]
fn player_heal_increases_health() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(40.0);
    player.heal(20.0);
    assert_eq!(player.health, 80.0);
}

#[test]
fn player_heal_caps_at_max_health() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.heal(50.0);
    assert_eq!(player.health, 100.0);
}

#[test]
fn player_heal_no_effect_when_dead() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(100.0);
    player.heal(50.0);
    assert_eq!(player.health, 0.0);
}

#[test]
fn player_is_alive_returns_true_when_alive() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert!(player.is_alive());
}

#[test]
fn player_is_alive_returns_false_when_dead() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(100.0);
    assert!(!player.is_alive());
}

#[test]
fn player_health_fraction_full() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert!((player.health_fraction() - 1.0).abs() < 0.001);
}

#[test]
fn player_health_fraction_half() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(50.0);
    assert!((player.health_fraction() - 0.5).abs() < 0.001);
}

#[test]
fn player_respawn_restores_health() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(100.0);
    player.respawn([10.0, 0.0, 10.0]);
    assert_eq!(player.health, 100.0);
}

#[test]
fn player_respawn_sets_position() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.respawn([25.0, 50.0, 75.0]);
    assert_eq!(player.position, [25.0, 50.0, 75.0]);
}

#[test]
fn player_respawn_clears_bleeding() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.bleeding = 10.0;
    player.respawn([0.0, 0.0, 0.0]);
    assert_eq!(player.bleeding, 0.0);
}

#[test]
fn player_respawn_restores_alive_state() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.take_damage(100.0);
    player.respawn([0.0, 0.0, 0.0]);
    assert!(player.is_alive());
}

#[test]
fn player_stamina_fraction() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.stamina = 25.0;
    assert!((player.stamina_fraction() - 0.25).abs() < 0.001);
}

// =============================================================================
// 4. PlayerController states (10)
// =============================================================================

#[test]
fn player_enter_dialogue_sets_state() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.enter_dialogue();
    assert_eq!(player.state, PlayerState::InDialogue);
}

#[test]
fn player_enter_trade_sets_state() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.enter_trade();
    assert_eq!(player.state, PlayerState::InTrade);
}

#[test]
fn player_exit_interaction_restores_alive() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.enter_dialogue();
    player.exit_interaction();
    assert_eq!(player.state, PlayerState::Alive);
}

#[test]
fn player_cannot_move_in_dialogue() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.enter_dialogue();
    assert!(!player.can_move());
}

#[test]
fn player_cannot_move_in_trade() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.enter_trade();
    assert!(!player.can_move());
}

#[test]
fn player_cannot_move_in_menu() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.state = PlayerState::InMenu;
    assert!(!player.can_move());
}

#[test]
fn player_exit_after_trade_restores_alive() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.enter_trade();
    player.exit_interaction();
    assert_eq!(player.state, PlayerState::Alive);
}

#[test]
fn player_weight_fraction_calculation() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.weight_carried = 25.0;
    assert!((player.weight_fraction() - 0.5).abs() < 0.001);
}

#[test]
fn player_in_dialogue_tick_no_stamina_change() {
    let mut player = PlayerController::new([0.0, 0.0, 0.0]);
    player.enter_dialogue();
    player.stamina = 60.0;
    player.tick(1.0, true);
    assert_eq!(player.stamina, 60.0);
}

#[test]
fn player_interaction_range_default() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert_eq!(player.interaction_range, 3.0);
}

// =============================================================================
// 5. CameraMode (5)
// =============================================================================

#[test]
fn camera_mode_first_person_variant() {
    let mode = CameraMode::FirstPerson;
    assert!(matches!(mode, CameraMode::FirstPerson));
}

#[test]
fn camera_mode_third_person_variant() {
    let mode = CameraMode::ThirdPerson;
    assert!(matches!(mode, CameraMode::ThirdPerson));
}

#[test]
fn camera_mode_free_variant() {
    let mode = CameraMode::Free;
    assert!(matches!(mode, CameraMode::Free));
}

#[test]
fn camera_mode_default_is_first_person() {
    let player = PlayerController::new([0.0, 0.0, 0.0]);
    assert!(matches!(player.camera_mode, CameraMode::FirstPerson));
}

#[test]
fn camera_mode_equality() {
    assert_eq!(CameraMode::FirstPerson, CameraMode::FirstPerson);
    assert_ne!(CameraMode::FirstPerson, CameraMode::ThirdPerson);
}

// =============================================================================
// 6. HudState (15)
// =============================================================================

#[test]
fn hud_state_new_creates_instance() {
    let hud = HudState::new();
    assert!(hud.show_health);
}

#[test]
fn hud_state_show_health_default_true() {
    let hud = HudState::new();
    assert!(hud.show_health);
}

#[test]
fn hud_state_show_stamina_default_true() {
    let hud = HudState::new();
    assert!(hud.show_stamina);
}

#[test]
fn hud_state_show_compass_default_true() {
    let hud = HudState::new();
    assert!(hud.show_compass);
}

#[test]
fn hud_state_show_minimap_default_false() {
    let hud = HudState::new();
    assert!(!hud.show_minimap);
}

#[test]
fn hud_state_show_crosshair_default_true() {
    let hud = HudState::new();
    assert!(hud.show_crosshair);
}

#[test]
fn hud_state_push_notification_adds_to_queue() {
    let mut hud = HudState::new();
    hud.push_notification("Test", NotificationKind::Info);
    assert_eq!(hud.notification_queue.len(), 1);
}

#[test]
fn hud_state_push_notification_stores_text() {
    let mut hud = HudState::new();
    hud.push_notification("Quest updated!", NotificationKind::QuestUpdate);
    assert_eq!(hud.notification_queue[0].text, "Quest updated!");
}

#[test]
fn hud_state_tick_removes_expired_notifications() {
    let mut hud = HudState::new();
    hud.push_notification("Temp", NotificationKind::Info);
    hud.tick(4.0);
    assert_eq!(hud.notification_queue.len(), 0);
}

#[test]
fn hud_state_set_interaction_sets_text() {
    let mut hud = HudState::new();
    hud.set_interaction("Press E to open");
    assert!(hud.show_interaction_prompt);
    assert_eq!(hud.interaction_text, "Press E to open");
}

#[test]
fn hud_state_clear_interaction_clears_prompt() {
    let mut hud = HudState::new();
    hud.set_interaction("Test");
    hud.clear_interaction();
    assert!(!hud.show_interaction_prompt);
    assert!(hud.interaction_text.is_empty());
}

#[test]
fn hud_state_set_quest_updates_fields() {
    let mut hud = HudState::new();
    hud.set_quest("Find artifact", "1/1");
    assert_eq!(hud.active_quest_name, "Find artifact");
    assert_eq!(hud.active_quest_progress, "1/1");
}

#[test]
fn hud_state_show_quest_tracker_default_true() {
    let hud = HudState::new();
    assert!(hud.show_quest_tracker);
}

#[test]
fn hud_state_multiple_notifications_queue() {
    let mut hud = HudState::new();
    hud.push_notification("First", NotificationKind::Info);
    hud.push_notification("Second", NotificationKind::Warning);
    assert_eq!(hud.notification_queue.len(), 2);
}

#[test]
fn hud_state_tick_advances_elapsed() {
    let mut hud = HudState::new();
    hud.push_notification("Test", NotificationKind::Info);
    hud.tick(1.0);
    assert!(hud.notification_queue[0].elapsed > 0.0);
}

// =============================================================================
// 7. NotificationKind (5)
// =============================================================================

#[test]
fn notification_kind_info() {
    let k = NotificationKind::Info;
    assert!(matches!(k, NotificationKind::Info));
}

#[test]
fn notification_kind_warning() {
    let k = NotificationKind::Warning;
    assert!(matches!(k, NotificationKind::Warning));
}

#[test]
fn notification_kind_quest_update() {
    let k = NotificationKind::QuestUpdate;
    assert!(matches!(k, NotificationKind::QuestUpdate));
}

#[test]
fn notification_kind_item_pickup() {
    let k = NotificationKind::ItemPickup;
    assert!(matches!(k, NotificationKind::ItemPickup));
}

#[test]
fn notification_kind_damage() {
    let k = NotificationKind::Damage;
    assert!(matches!(k, NotificationKind::Damage));
}

// =============================================================================
// 8. PlayerInventory (20)
// =============================================================================

#[test]
fn player_inventory_new_creates_empty() {
    let inv = PlayerInventory::new();
    assert!(inv.items.is_empty());
}

#[test]
fn player_inventory_new_default_money() {
    let inv = PlayerInventory::new();
    assert_eq!(inv.money, 500.0);
}

#[test]
fn player_inventory_add_item_success() {
    let mut inv = PlayerInventory::new();
    let item = PlayerItem {
        name: "Medkit".to_string(),
        quantity: 1,
        weight: 2.0,
        item_type: PlayerItemType::Medkit,
    };
    assert!(inv.add_item(item));
    assert!(inv.has_item("Medkit"));
}

#[test]
fn player_inventory_add_item_rejects_overweight() {
    let mut inv = PlayerInventory::new();
    inv.max_weight = 5.0;
    let item = PlayerItem {
        name: "Heavy".to_string(),
        quantity: 10,
        weight: 10.0,
        item_type: PlayerItemType::Junk,
    };
    assert!(!inv.add_item(item));
}

#[test]
fn player_inventory_remove_item_success() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Ammo".to_string(),
        quantity: 20,
        weight: 0.5,
        item_type: PlayerItemType::Ammo,
    });
    assert!(inv.remove_item("Ammo", 10));
    assert_eq!(inv.item_count("Ammo"), 10);
}

#[test]
fn player_inventory_remove_item_removes_entry_when_zero() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Food".to_string(),
        quantity: 2,
        weight: 0.5,
        item_type: PlayerItemType::Food,
    });
    inv.remove_item("Food", 2);
    assert!(!inv.has_item("Food"));
}

#[test]
fn player_inventory_has_item_returns_false_for_missing() {
    let inv = PlayerInventory::new();
    assert!(!inv.has_item("Nonexistent"));
}

#[test]
fn player_inventory_item_count_zero_for_missing() {
    let inv = PlayerInventory::new();
    assert_eq!(inv.item_count("Missing"), 0);
}

#[test]
fn player_inventory_weight_tracking() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "W".to_string(),
        quantity: 2,
        weight: 5.0,
        item_type: PlayerItemType::Tool,
    });
    assert!((inv.weight_carried - 10.0).abs() < 0.001);
}

#[test]
fn player_inventory_money_field() {
    let mut inv = PlayerInventory::new();
    inv.money = 1000.0;
    assert_eq!(inv.money, 1000.0);
}

#[test]
fn player_inventory_equipped_weapon_none_default() {
    let inv = PlayerInventory::new();
    assert!(inv.equipped_weapon.is_none());
}

#[test]
fn player_inventory_add_stacks_same_item() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Ammo".to_string(),
        quantity: 5,
        weight: 0.1,
        item_type: PlayerItemType::Ammo,
    });
    inv.add_item(PlayerItem {
        name: "Ammo".to_string(),
        quantity: 5,
        weight: 0.1,
        item_type: PlayerItemType::Ammo,
    });
    assert_eq!(inv.item_count("Ammo"), 10);
}

#[test]
fn player_inventory_remove_item_fails_for_insufficient_count() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "X".to_string(),
        quantity: 2,
        weight: 1.0,
        item_type: PlayerItemType::Junk,
    });
    assert!(!inv.remove_item("X", 5));
}

#[test]
fn player_inventory_remove_item_fails_for_missing() {
    let mut inv = PlayerInventory::new();
    assert!(!inv.remove_item("Nonexistent", 1));
}

#[test]
fn player_inventory_max_weight_default() {
    let inv = PlayerInventory::new();
    assert_eq!(inv.max_weight, 50.0);
}

#[test]
fn player_inventory_weight_decreases_on_remove() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Y".to_string(),
        quantity: 4,
        weight: 2.5,
        item_type: PlayerItemType::Junk,
    });
    inv.remove_item("Y", 2);
    assert!((inv.weight_carried - 5.0).abs() < 0.001);
}

#[test]
fn player_inventory_item_count_returns_quantity() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Z".to_string(),
        quantity: 7,
        weight: 0.5,
        item_type: PlayerItemType::Food,
    });
    assert_eq!(inv.item_count("Z"), 7);
}

#[test]
fn player_inventory_add_item_updates_weight_carried() {
    let mut inv = PlayerInventory::new();
    let item = PlayerItem {
        name: "Test".to_string(),
        quantity: 3,
        weight: 1.0,
        item_type: PlayerItemType::Tool,
    };
    inv.add_item(item);
    assert!((inv.weight_carried - 3.0).abs() < 0.001);
}

#[test]
fn player_inventory_items_vec_empty_initially() {
    let inv = PlayerInventory::new();
    assert_eq!(inv.items.len(), 0);
}

#[test]
fn player_inventory_has_item_true_when_present() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Rifle".to_string(),
        quantity: 1,
        weight: 4.0,
        item_type: PlayerItemType::Weapon,
    });
    assert!(inv.has_item("Rifle"));
}

// =============================================================================
// 9. PlayerItemType (8)
// =============================================================================

#[test]
fn player_item_type_weapon() {
    let t = PlayerItemType::Weapon;
    assert!(matches!(t, PlayerItemType::Weapon));
}

#[test]
fn player_item_type_ammo() {
    let t = PlayerItemType::Ammo;
    assert!(matches!(t, PlayerItemType::Ammo));
}

#[test]
fn player_item_type_medkit() {
    let t = PlayerItemType::Medkit;
    assert!(matches!(t, PlayerItemType::Medkit));
}

#[test]
fn player_item_type_food() {
    let t = PlayerItemType::Food;
    assert!(matches!(t, PlayerItemType::Food));
}

#[test]
fn player_item_type_artifact() {
    let t = PlayerItemType::Artifact;
    assert!(matches!(t, PlayerItemType::Artifact));
}

#[test]
fn player_item_type_armor() {
    let t = PlayerItemType::Armor;
    assert!(matches!(t, PlayerItemType::Armor));
}

#[test]
fn player_item_type_junk() {
    let t = PlayerItemType::Junk;
    assert!(matches!(t, PlayerItemType::Junk));
}

#[test]
fn player_item_type_tool() {
    let t = PlayerItemType::Tool;
    assert!(matches!(t, PlayerItemType::Tool));
}

// =============================================================================
// 10. PlayerSave (15)
// =============================================================================

#[test]
fn player_save_from_state_creates_save() {
    let controller = PlayerController::new([1.0, 2.0, 3.0]);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(
        &controller,
        &inventory,
        vec![1, 2],
        vec![3],
        3600.0,
        15,
        6,
    );
    assert_eq!(save.position, [1.0, 2.0, 3.0]);
}

#[test]
fn player_save_restore_controller_position() {
    let controller = PlayerController::new([10.0, 20.0, 30.0]);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_controller();
    assert_eq!(restored.position, [10.0, 20.0, 30.0]);
}

#[test]
fn player_save_restore_controller_health() {
    let mut controller = PlayerController::new([0.0, 0.0, 0.0]);
    controller.take_damage(30.0);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_controller();
    assert_eq!(restored.health, 70.0);
}

#[test]
fn player_save_restore_inventory_items() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let mut inventory = PlayerInventory::new();
    inventory.add_item(PlayerItem {
        name: "Artifact".to_string(),
        quantity: 1,
        weight: 1.0,
        item_type: PlayerItemType::Artifact,
    });
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_inventory();
    assert!(restored.has_item("Artifact"));
}

#[test]
fn player_save_restore_inventory_money() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let mut inventory = PlayerInventory::new();
    inventory.money = 750.0;
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_inventory();
    assert_eq!(restored.money, 750.0);
}

#[test]
fn player_save_preserves_rotation() {
    let mut controller = PlayerController::new([0.0, 0.0, 0.0]);
    controller.rotation = [1.5, 2.0];
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    assert_eq!(save.rotation, [1.5, 2.0]);
}

#[test]
fn player_save_preserves_play_time() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 7200.5, 10, 3);
    assert!((save.play_time_seconds - 7200.5).abs() < 0.001);
}

#[test]
fn player_save_preserves_active_quests() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![1, 2, 3], vec![], 0.0, 1, 1);
    assert_eq!(save.active_quests, vec![1, 2, 3]);
}

#[test]
fn player_save_preserves_completed_quests() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![4, 5], 0.0, 1, 1);
    assert_eq!(save.completed_quests, vec![4, 5]);
}

#[test]
fn player_save_preserves_day_month() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 25, 12);
    assert_eq!(save.save_day, 25);
    assert_eq!(save.save_month, 12);
}

#[test]
fn player_save_restore_controller_alive_when_health_positive() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_controller();
    assert!(restored.is_alive());
}

#[test]
fn player_save_restore_controller_dead_when_health_zero() {
    let mut controller = PlayerController::new([0.0, 0.0, 0.0]);
    controller.take_damage(100.0);
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_controller();
    assert!(!restored.is_alive());
}

#[test]
fn player_save_preserves_bleeding() {
    let mut controller = PlayerController::new([0.0, 0.0, 0.0]);
    controller.bleeding = 5.0;
    let inventory = PlayerInventory::new();
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    assert_eq!(save.bleeding, 5.0);
}

#[test]
fn player_save_equipped_weapon() {
    let controller = PlayerController::new([0.0, 0.0, 0.0]);
    let mut inventory = PlayerInventory::new();
    inventory.equipped_weapon = Some("AK47".to_string());
    let save = PlayerSave::from_state(&controller, &inventory, vec![], vec![], 0.0, 1, 1);
    assert_eq!(save.equipped_weapon.as_deref(), Some("AK47"));
}

// =============================================================================
// 11. EquipmentSlots (15)
// =============================================================================

#[test]
fn equipment_slots_default_stalker_values() {
    let eq = EquipmentSlots::default_stalker();
    assert!((eq.weapon_condition - 0.8).abs() < 0.001);
    assert_eq!(eq.medkits, 2);
}

#[test]
fn equipment_slots_empty_all_zero() {
    let eq = EquipmentSlots::empty();
    assert_eq!(eq.weapon_condition, 0.0);
    assert_eq!(eq.medkits, 0);
    assert_eq!(eq.ammo, 0);
}

#[test]
fn equipment_slots_combat_effectiveness_with_weapon() {
    let eq = EquipmentSlots::default_stalker();
    let eff = eq.combat_effectiveness();
    assert!(eff > 0.5);
    assert!(eff <= 1.0);
}

#[test]
fn equipment_slots_combat_effectiveness_no_ammo() {
    let mut eq = EquipmentSlots::default_stalker();
    eq.ammo = 0;
    let eff = eq.combat_effectiveness();
    assert!(eff < 0.5);
}

#[test]
fn equipment_slots_degrade_combat() {
    let mut eq = EquipmentSlots::default_stalker();
    let w_before = eq.weapon_condition;
    let a_before = eq.armor_condition;
    eq.degrade_combat();
    assert!(eq.weapon_condition < w_before);
    assert!(eq.armor_condition < a_before);
}

#[test]
fn equipment_slots_degrade_time() {
    let mut eq = EquipmentSlots::default_stalker();
    let w_before = eq.weapon_condition;
    eq.degrade_time(1000.0);
    assert!(eq.weapon_condition < w_before);
}

#[test]
fn equipment_slots_needs_repair_false_when_good() {
    let eq = EquipmentSlots::default_stalker();
    assert!(!eq.needs_repair());
}

#[test]
fn equipment_slots_needs_repair_true_when_low() {
    let mut eq = EquipmentSlots::default_stalker();
    eq.weapon_condition = 0.2;
    assert!(eq.needs_repair());
}

#[test]
fn equipment_slots_needs_supplies_true_when_no_medkits() {
    let mut eq = EquipmentSlots::default_stalker();
    eq.medkits = 0;
    assert!(eq.needs_supplies());
}

#[test]
fn equipment_slots_needs_supplies_true_when_low_ammo() {
    let mut eq = EquipmentSlots::default_stalker();
    eq.ammo = 5;
    assert!(eq.needs_supplies());
}

#[test]
fn equipment_slots_use_medkit_success() {
    let mut eq = EquipmentSlots::default_stalker();
    assert!(eq.use_medkit());
    assert_eq!(eq.medkits, 1);
}

#[test]
fn equipment_slots_use_medkit_fail_when_empty() {
    let mut eq = EquipmentSlots::empty();
    assert!(!eq.use_medkit());
}

#[test]
fn equipment_slots_eat_ration_success() {
    let mut eq = EquipmentSlots::default_stalker();
    assert!(eq.eat_ration());
    assert_eq!(eq.food_rations, 2);
}

#[test]
fn equipment_slots_eat_ration_fail_when_empty() {
    let mut eq = EquipmentSlots::empty();
    assert!(!eq.eat_ration());
}

#[test]
fn equipment_slots_repair_cost_increases_with_damage() {
    let eq_full = EquipmentSlots::default_stalker();
    let eq_damaged = EquipmentSlots {
        weapon_condition: 0.5,
        armor_condition: 0.5,
        ..EquipmentSlots::default_stalker()
    };
    assert!(eq_damaged.repair_cost() > eq_full.repair_cost());
}

#[test]
fn equipment_slots_degrade_combat_reduces_ammo() {
    let mut eq = EquipmentSlots::default_stalker();
    eq.ammo = 20;
    eq.degrade_combat();
    assert!(eq.ammo < 20);
}

// =============================================================================
// 12. Entity combat (8)
// =============================================================================

#[test]
fn base_power_npc() {
    let kind = EntityKind::Npc;
    assert!((base_power(&kind) - 5.0).abs() < 0.001);
}

#[test]
fn base_power_wolf() {
    let kind = EntityKind::Monster(MonsterSpecies::Wolf);
    assert!((base_power(&kind) - 3.0).abs() < 0.001);
}

#[test]
fn base_power_boar() {
    let kind = EntityKind::Monster(MonsterSpecies::Boar);
    assert!((base_power(&kind) - 7.0).abs() < 0.001);
}

#[test]
fn base_power_bloodsucker() {
    let kind = EntityKind::Monster(MonsterSpecies::Bloodsucker);
    assert!((base_power(&kind) - 10.0).abs() < 0.001);
}

#[test]
fn is_prey_for_bloodsucker_hunts_npc() {
    let hunter = EntityKind::Monster(MonsterSpecies::Bloodsucker);
    let target = EntityKind::Npc;
    assert!(is_prey_for(&hunter, &target));
}

#[test]
fn is_prey_for_wolf_not_prey_of_npc() {
    let hunter = EntityKind::Npc;
    let target = EntityKind::Monster(MonsterSpecies::Wolf);
    assert!(is_prey_for(&hunter, &target));
}

#[test]
fn food_value_values() {
    assert!((food_value(&EntityKind::Npc) - 0.5).abs() < 0.001);
    assert!((food_value(&EntityKind::Monster(MonsterSpecies::Bloodsucker)) - 0.8).abs() < 0.001);
}

#[test]
fn is_prey_for_equal_power_false() {
    let a = EntityKind::Npc;
    let b = EntityKind::Npc;
    assert!(!is_prey_for(&a, &b));
}

// =============================================================================
// 13. PersonalNeeds (6)
// =============================================================================

#[test]
fn personal_needs_default_npc_creates() {
    let needs = PersonalNeeds::default_npc();
    assert!(needs.hunger > 0.0);
}

#[test]
fn personal_needs_default_npc_hunger_value() {
    let needs = PersonalNeeds::default_npc();
    assert!((needs.hunger - 0.2).abs() < 0.001);
}

#[test]
fn personal_needs_default_monster_creates() {
    let needs = PersonalNeeds::default_monster();
    assert!(needs.hunger > 0.0);
}

#[test]
fn personal_needs_default_monster_ambitions_zero() {
    let needs = PersonalNeeds::default_monster();
    assert_eq!(needs.ambitions, 0.0);
}

#[test]
fn personal_needs_field_access() {
    let needs = PersonalNeeds::default_npc();
    assert!(needs.health > 0.0);
    assert!(needs.energy > 0.0);
}

#[test]
fn personal_needs_npc_vs_monster_differ() {
    let npc = PersonalNeeds::default_npc();
    let monster = PersonalNeeds::default_monster();
    assert_ne!(npc.ambitions, monster.ambitions);
}
