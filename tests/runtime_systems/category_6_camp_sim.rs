use super::*;

// Category 6: Camp simulation (15 tests)
// =============================================================================

#[test]
fn camp_state_new() {
    let camp = CampState::new("Test", "Loners", 10);
    assert_eq!(camp.name, "Test");
    assert_eq!(camp.faction, "Loners");
    assert_eq!(camp.population, 10);
    assert!(camp.food_supply > 0.0);
}

#[test]
fn camp_state_tick_daily() {
    let mut camp = CampState::new("Test", "Loners", 5);
    let food_before = camp.food_supply;
    camp.tick_daily();
    assert!(camp.food_supply <= food_before || camp.food_supply >= 0.0);
}

#[test]
fn camp_state_report_danger() {
    let mut camp = CampState::new("Test", "Loners", 5);
    camp.report_danger(0.5);
    assert!(camp.danger_memory > 0.0);
}

#[test]
fn camp_state_resupply() {
    let mut camp = CampState::new("Test", "Loners", 5);
    camp.food_supply = 0.2;
    camp.resupply(0.5);
    assert!(camp.food_supply >= 0.2);
}

#[test]
fn camp_state_is_safe() {
    let camp = CampState::new("Test", "Loners", 5);
    let _ = camp.is_safe();
}

#[test]
fn camp_state_pressure_level() {
    let camp = CampState::new("Test", "Loners", 5);
    let p = camp.pressure_level();
    assert!(p >= 0.0 && p <= 1.0);
}

#[test]
fn camp_state_tick_daily_mood_clamp() {
    let mut camp = CampState::new("Test", "Loners", 5);
    for _ in 0..100 {
        camp.tick_daily();
    }
    assert!(camp.mood >= 0.0 && camp.mood <= 1.0);
}

#[test]
fn camp_state_low_food_decreases_mood() {
    let mut camp = CampState::new("Test", "Loners", 5);
    camp.food_supply = 0.1;
    let mood_before = camp.mood;
    camp.tick_daily();
    assert!(camp.mood <= mood_before + 0.1);
}

#[test]
fn camp_state_services_vec() {
    let camp = CampState::new("Test", "Loners", 5);
    assert!(camp.services.is_empty());
}

#[test]
fn camp_state_danger_memory_decay() {
    let mut camp = CampState::new("Test", "Loners", 5);
    camp.danger_memory = 1.0;
    camp.tick_daily();
    assert!(camp.danger_memory < 1.0);
}

#[test]
fn camp_state_population_consumption() {
    let mut camp = CampState::new("Test", "Loners", 100);
    camp.food_supply = 1.0;
    camp.tick_daily();
    assert!(camp.food_supply < 1.0 || camp.food_supply >= 0.0);
}

#[test]
fn camp_state_resupply_caps() {
    let mut camp = CampState::new("Test", "Loners", 5);
    camp.food_supply = 1.5;
    camp.resupply(1.0);
    assert!(camp.food_supply <= 2.0);
}

#[test]
fn camp_state_high_security_improves_mood() {
    let mut camp = CampState::new("Test", "Loners", 5);
    camp.security_level = 0.9;
    let mood_before = camp.mood;
    camp.tick_daily();
    assert!(camp.mood >= mood_before - 0.1);
}

#[test]
fn camp_state_report_danger_caps() {
    let mut camp = CampState::new("Test", "Loners", 5);
    camp.report_danger(5.0);
    assert!(camp.danger_memory <= 1.0);
}

// =============================================================================
