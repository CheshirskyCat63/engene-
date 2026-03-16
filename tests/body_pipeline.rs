#[test]
fn body_state_default() {
    use engene::body::anatomy::BodyState;

    let body = BodyState::new_humanoid(0);
    assert_eq!(body.zones.len(), 8);
    for zone in &body.zones {
        assert_eq!(zone.integrity, 100.0);
    }
    assert!((body.aggregate_health() - 100.0).abs() < 1.0);
}

#[test]
fn body_state_take_damage() {
    use engene::body::anatomy::BodyState;
    use engene::body::body_damage;
    use engene::body::body_store::BodyStateStore;
    use engene::physics::damage_pipeline::response_aggregator::BodyZone;

    let mut store = BodyStateStore::new();
    let body = BodyState::new_humanoid(0);
    let handle = store.allocate(body);
    let health_before = store.get(handle.store_id).unwrap().aggregate_health();

    body_damage::apply_zone_damage(&mut store, handle.store_id, BodyZone::Torso, 30.0);

    let health_after = store.get(handle.store_id).unwrap().aggregate_health();
    assert!(health_after < health_before);
}

#[test]
fn body_system_exists() {
    use engene::body::body_system::BodySystem;
    use engene::core::system::EngineSystem;

    let system = BodySystem;
    assert_eq!(system.name(), "BodySystem");
}

#[test]
fn body_zone_mapping() {
    use engene::physics::damage_pipeline::response_aggregator::BodyZone;

    let _head = BodyZone::Head;
    let _torso = BodyZone::Torso;
    let _left_arm = BodyZone::LeftArm;
    let _right_arm = BodyZone::RightArm;
    let _left_leg = BodyZone::LeftLeg;
    let _right_leg = BodyZone::RightLeg;
}

#[test]
fn body_anatomy_regions() {
    use engene::body::anatomy::BodyState;

    let body = BodyState::new_humanoid(0);
    assert_eq!(body.zones.len(), 8);
}
