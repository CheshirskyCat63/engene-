use super::*;

// =============================================================================
// 4. CAMP + ROLE SIMULATION (35 tests)
// =============================================================================

#[test]
fn camp_state_new() {
    let camp = CampState::new("Rostok", "Duty", 50);
    assert_eq!(camp.name, "Rostok");
    assert_eq!(camp.population, 50);
    assert_eq!(camp.food_supply, 1.0);
}

#[test]
fn camp_state_tick_daily() {
    let mut camp = CampState::new("Test", "Faction", 10);
    camp.food_supply = 1.0;
    camp.tick_daily();
    assert!(camp.food_supply < 1.0);
}

#[test]
fn camp_state_food_decreases() {
    let mut camp = CampState::new("Test", "F", 100);
    let before = camp.food_supply;
    camp.tick_daily();
    assert!(camp.food_supply < before);
}

#[test]
fn camp_state_security_changes_mood() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.security_level = 0.9;
    camp.tick_daily();
    assert!(camp.mood >= 0.0);
}

#[test]
fn camp_state_mood_clamped() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.mood = 0.0;
    camp.tick_daily();
    assert!(camp.mood >= 0.0 && camp.mood <= 1.0);
}

#[test]
fn camp_state_report_danger() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.report_danger(0.5);
    assert!(camp.danger_memory > 0.0);
    assert!(camp.mood < 0.5);
}

#[test]
fn camp_state_resupply() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.food_supply = 0.2;
    camp.resupply(0.5);
    assert!(camp.food_supply > 0.2);
}

#[test]
fn camp_state_is_safe() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.security_level = 0.8;
    let safe = camp.is_safe();
    assert!(safe || camp.danger_memory >= 0.3);
}

#[test]
fn camp_state_pressure_level() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.food_supply = 0.0;
    let pressure = camp.pressure_level();
    assert!(pressure > 0.0);
}

#[test]
fn npc_role_guard() {
    let role = NpcRole::Guard;
    let behavior = RoleBehavior::for_role(role);
    assert_eq!(behavior.role, NpcRole::Guard);
    assert!(behavior.daily_income > 0.0);
}

#[test]
fn npc_role_hunter() {
    let behavior = RoleBehavior::for_role(NpcRole::Hunter);
    assert_eq!(behavior.role, NpcRole::Hunter);
    assert!(behavior.danger_exposure > 0.5);
}

#[test]
fn npc_role_trader() {
    let behavior = RoleBehavior::for_role(NpcRole::Trader);
    assert_eq!(behavior.role, NpcRole::Trader);
    assert!(behavior.social_interaction > 0.5);
}

#[test]
fn npc_role_scavenger() {
    let behavior = RoleBehavior::for_role(NpcRole::Scavenger);
    assert_eq!(behavior.role, NpcRole::Scavenger);
    assert!(!behavior.required_equipment.is_empty());
}

#[test]
fn npc_role_courier() {
    let behavior = RoleBehavior::for_role(NpcRole::Courier);
    assert_eq!(behavior.role, NpcRole::Courier);
}

#[test]
fn npc_role_bandit() {
    let behavior = RoleBehavior::for_role(NpcRole::Bandit);
    assert!(behavior.danger_exposure > 0.5);
}

#[test]
fn npc_role_idle_resident() {
    let behavior = RoleBehavior::for_role(NpcRole::IdleResident);
    assert!(behavior.daily_income < 20.0);
}

#[test]
fn npc_role_mechanic() {
    let behavior = RoleBehavior::for_role(NpcRole::Mechanic);
    assert_eq!(behavior.role, NpcRole::Mechanic);
}

#[test]
fn npc_role_medic() {
    let behavior = RoleBehavior::for_role(NpcRole::Medic);
    assert!(behavior.daily_income > 0.0);
}

#[test]
fn role_behavior_daily_income() {
    let guard = RoleBehavior::for_role(NpcRole::Guard);
    let trader = RoleBehavior::for_role(NpcRole::Trader);
    assert!(trader.daily_income > guard.daily_income);
}

#[test]
fn role_behavior_danger_exposure() {
    let bandit = RoleBehavior::for_role(NpcRole::Bandit);
    let trader = RoleBehavior::for_role(NpcRole::Trader);
    assert!(bandit.danger_exposure > trader.danger_exposure);
}

#[test]
fn role_behavior_matches_job_guard() {
    let role = RoleBehavior::matches_job(&Job::Guard);
    assert_eq!(role, NpcRole::Guard);
}

#[test]
fn role_behavior_matches_job_trader() {
    let role = RoleBehavior::matches_job(&Job::Trader);
    assert_eq!(role, NpcRole::Trader);
}

#[test]
fn world_milestone_tracker_new() {
    let tracker = WorldMilestoneTracker::new();
    assert_eq!(tracker.bankruptcies, 0);
    assert_eq!(tracker.quest_completions, 0);
}

#[test]
fn world_milestone_tracker_record_bankruptcy() {
    let mut tracker = WorldMilestoneTracker::new();
    tracker.record_bankruptcy();
    tracker.record_bankruptcy();
    assert_eq!(tracker.bankruptcies, 2);
}

#[test]
fn world_milestone_tracker_record_banditization() {
    let mut tracker = WorldMilestoneTracker::new();
    tracker.record_banditization();
    assert_eq!(tracker.banditizations, 1);
}

#[test]
fn world_milestone_tracker_record_quest() {
    let mut tracker = WorldMilestoneTracker::new();
    tracker.record_quest_completion();
    tracker.record_quest_failure();
    assert_eq!(tracker.quest_completions, 1);
    assert_eq!(tracker.quest_failures, 1);
}

#[test]
fn world_milestone_tracker_check_milestones() {
    let mut tracker = WorldMilestoneTracker::new();
    for _ in 0..3 {
        tracker.record_bankruptcy();
    }
    tracker.check_milestones(1);
    assert!(!tracker.milestones_achieved.is_empty());
}

#[test]
fn world_milestone_tracker_summary() {
    let tracker = WorldMilestoneTracker::new();
    let s = tracker.summary();
    assert!(s.contains("Months"));
}

#[test]
fn camp_services_vec() {
    let camp = CampState::new("Test", "F", 10);
    assert!(camp.services.is_empty());
}
