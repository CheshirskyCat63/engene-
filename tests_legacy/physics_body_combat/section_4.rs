use super::*;

// ===== BODY ANATOMY (15 tests) =====

#[test]
fn anatomy_body_state_new_humanoid() {
    let body = AnatomyBodyState::new_humanoid(0);
    assert_eq!(body.zones.len(), 8);
    assert_eq!(body.joints.len(), 6);
}

#[test]
fn anatomy_is_alive() {
    let body = AnatomyBodyState::new_humanoid(1);
    assert!(body.is_alive());
}

#[test]
fn anatomy_aggregate_health() {
    let body = AnatomyBodyState::new_humanoid(2);
    let h = body.aggregate_health();
    assert!(h > 0.0 && h <= 100.0);
}

#[test]
fn zone_state_new() {
    let z = ZoneState::new(BodyZone::Head);
    assert_eq!(z.integrity, 100.0);
    assert!(!z.is_severed);
}

#[test]
fn anatomy_zones_order() {
    let body = AnatomyBodyState::new_humanoid(3);
    assert_eq!(body.zones[0].zone, BodyZone::Head);
    assert_eq!(body.zones[2].zone, BodyZone::Torso);
}

#[test]
fn anatomy_blood_level() {
    let body = AnatomyBodyState::new_humanoid(4);
    assert_eq!(body.blood_level, 1.0);
}

#[test]
fn anatomy_bleed_points_empty() {
    let body = AnatomyBodyState::new_humanoid(5);
    assert!(body.bleed_points.is_empty());
}

#[test]
fn joint_info_dismember_threshold() {
    let j = JointInfo {
        id: 0,
        zone: BodyZone::Neck,
        integrity: 100.0,
        broken: false,
        dismember_threshold: 15.0,
    };
    assert_eq!(j.dismember_threshold, 15.0);
}

#[test]
fn bleed_point_fields() {
    let bp = BleedPoint {
        zone: BodyZone::Torso,
        rate: 0.1,
        time_active: 0.0,
    };
    assert_eq!(bp.rate, 0.1);
}

// ===== BODY STORE (10 tests) =====

#[test]
fn body_state_store_new() {
    let store = BodyStateStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn body_state_store_allocate() {
    let mut store = BodyStateStore::new();
    let body = AnatomyBodyState::new_humanoid(0);
    let handle = store.allocate(body);
    assert_eq!(store.len(), 1);
    assert!(handle.is_alive);
}

#[test]
fn body_state_store_get() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(1));
    let body = store.get(handle.store_id).unwrap();
    assert_eq!(body.entity, 1);
}

#[test]
fn body_state_store_get_mut() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(2));
    let b = store.get_mut(handle.store_id).unwrap();
    b.blood_level = 0.5;
    assert_eq!(store.get(handle.store_id).unwrap().blood_level, 0.5);
}

#[test]
fn body_state_store_free() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(3));
    let len_before = store.len();
    store.free(handle.store_id);
    assert_eq!(store.len(), len_before - 1);
}

#[test]
fn body_state_store_tick() {
    let mut store = BodyStateStore::new();
    store.allocate(AnatomyBodyState::new_humanoid(4));
    store.tick(0.1);
}

#[test]
fn body_handle_fields() {
    let mut store = BodyStateStore::new();
    let h = store.allocate(AnatomyBodyState::new_humanoid(5));
    assert_eq!(h.store_id, 0);
    assert!(!h.has_severed_limb);
}

// ===== BODY RESPONSE (10 tests) =====

#[test]
fn body_physical_response_cache_healthy() {
    let cache = BodyPhysicalResponseCache::healthy();
    assert_eq!(cache.movement_speed_mult, 1.0);
    assert!(cache.can_sprint);
}

#[test]
fn body_physical_response_compute_from_health() {
    let cache = BodyPhysicalResponseCache::compute_from_health(0.9, 1.0, 0.0);
    assert_eq!(cache.response_tier, PhysicalResponseTier::Healthy);
}

#[test]
fn body_physical_response_critical() {
    let cache = BodyPhysicalResponseCache::compute_from_health(0.2, 0.5, 0.8);
    assert_eq!(cache.response_tier, PhysicalResponseTier::Critical);
}

#[test]
fn body_physical_response_is_combat_capable() {
    let cache = BodyPhysicalResponseCache::healthy();
    assert!(cache.is_combat_capable());
}

#[test]
fn body_physical_response_is_mobile() {
    let cache = BodyPhysicalResponseCache::healthy();
    assert!(cache.is_mobile());
}

#[test]
fn physical_response_tier_variants() {
    let _ = PhysicalResponseTier::Healthy;
    let _ = PhysicalResponseTier::MinorInjury;
    let _ = PhysicalResponseTier::MajorInjury;
    let _ = PhysicalResponseTier::Critical;
    let _ = PhysicalResponseTier::Unconscious;
    let _ = PhysicalResponseTier::Dead;
}

#[test]
fn body_physical_response_dead() {
    let cache = BodyPhysicalResponseCache::compute_from_health(0.0, 0.0, 1.0);
    assert!(!cache.is_mobile());
}

// ===== BODY DAMAGE & DISMEMBERMENT (8 tests) =====

#[test]
fn test_apply_zone_damage() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(0));
    let health_before = store.get(handle.store_id).unwrap().aggregate_health();
    apply_zone_damage(&mut store, handle.store_id, BodyZone::Torso, 30.0);
    let health_after = store.get(handle.store_id).unwrap().aggregate_health();
    assert!(health_after < health_before);
}

#[test]
fn check_dismemberment_joint_intact() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(0));
    let result = check_dismemberment(&mut store, handle.store_id, 0);
    assert!(!result);
}

#[test]
fn check_dismemberment_joint_broken() {
    let mut store = BodyStateStore::new();
    let mut body = AnatomyBodyState::new_humanoid(0);
    body.joints[0].broken = true;
    let handle = store.allocate(body);
    let result = check_dismemberment(&mut store, handle.store_id, 0);
    assert!(result);
}

#[test]
fn severed_zones_empty() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(0));
    let zones = severed_zones(&store, handle.store_id);
    assert!(zones.is_empty());
}

#[test]
fn severed_zones_after_dismember() {
    let mut store = BodyStateStore::new();
    let mut body = AnatomyBodyState::new_humanoid(0);
    body.joints[1].broken = true;
    body.zones[3].is_severed = true;
    let handle = store.allocate(body);
    let zones = severed_zones(&store, handle.store_id);
    assert!(!zones.is_empty());
}

// ===== DEATH PIPELINE (12 tests) =====

#[test]
fn corpse_state_new() {
    let c = CorpseState::new(1, [10.0, 20.0], "combat");
    assert_eq!(c.entity, 1);
    assert_eq!(c.position[0], 10.0);
    assert!(c.lootable);
}

#[test]
fn corpse_state_tick() {
    let mut c = CorpseState::new(2, [0.0, 0.0], "fall");
    c.tick(1.0);
    assert!(c.decay_timer > 0.0);
}

#[test]
fn corpse_state_decay_progress() {
    let mut c = CorpseState::new(3, [0.0, 0.0], "test");
    c.tick(150.0);
    assert!(c.decay_progress() > 0.0);
}

#[test]
fn corpse_state_should_remove() {
    let mut c = CorpseState::new(4, [0.0, 0.0], "test");
    for _ in 0..350 {
        c.tick(1.0);
    }
    assert!(c.should_remove());
}

#[test]
fn corpse_state_loot_item() {
    let mut c = CorpseState::new(5, [0.0, 0.0], "test");
    c.items_remaining.push("sword".to_string());
    let item = c.loot_item(0);
    assert_eq!(item, Some("sword".to_string()));
    assert!(c.items_remaining.is_empty());
}

#[test]
fn corpse_manager_new() {
    let mgr = CorpseManager::new();
    assert_eq!(mgr.corpse_count(), 0);
}

#[test]
fn corpse_manager_register_death() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [5.0, 5.0], "combat", vec!["axe".to_string()]);
    assert_eq!(mgr.corpse_count(), 1);
}

#[test]
fn corpse_manager_tick_all() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [0.0, 0.0], "test", vec![]);
    mgr.tick_all(1.0);
}

#[test]
fn corpse_manager_corpses_near() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [10.0, 10.0], "test", vec![]);
    let near = mgr.corpses_near(10.0, 10.0, 5.0);
    assert!(!near.is_empty());
}

#[test]
fn death_state_variants() {
    let _ = DeathState::Alive;
    let _ = DeathState::Dying;
    let _ = DeathState::Dead;
    let _ = DeathState::Corpse { decay_ticks: 0 };
    let _ = DeathState::Skeleton;
    let _ = DeathState::Removed;
}

// ===== BLOOD (5 tests) =====

#[test]
fn compute_blood_lod_full() {
    assert_eq!(compute_blood_lod(100.0 * 100.0), BloodLod::Full);
}

#[test]
fn compute_blood_lod_reduced() {
    assert_eq!(compute_blood_lod(500.0 * 500.0), BloodLod::Reduced);
}

#[test]
fn compute_blood_lod_state_only() {
    assert_eq!(compute_blood_lod(2000.0 * 2000.0), BloodLod::StateOnly);
}

#[test]
fn blood_lod_variants() {
    let _ = BloodLod::Full;
    let _ = BloodLod::Reduced;
    let _ = BloodLod::StateOnly;
}
