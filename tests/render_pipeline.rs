#[test]
fn lod_config_levels() {
    use engene::graphics::lod::{LodConfig, LodLevel};

    let config = LodConfig::default();
    assert_eq!(config.compute_lod(0.0), LodLevel::Full);
    assert_eq!(config.compute_lod(100.0), LodLevel::Medium);
    assert_eq!(config.compute_lod(200.0), LodLevel::Low);
    assert_eq!(config.compute_lod(500.0), LodLevel::Billboard);
    assert_eq!(config.compute_lod(2000.0), LodLevel::Culled);
}

#[test]
fn frustum_containment() {
    use engene::graphics::visibility::Frustum;
    use engene::graphics::camera::FlyCamera;
    

    let camera = FlyCamera::new();
    let vp = camera.view_projection();
    let frustum = Frustum::from_view_projection(&vp);

    let center = camera.position + camera.forward() * 100.0;
    assert!(frustum.test_sphere(center, 10.0));
}

#[test]
fn camera_simulation_default() {
    use engene::graphics::camera::FlyCamera;

    let camera = FlyCamera::new();
    assert!(camera.speed > 0.0);
    assert!(camera.fov > 0.0);
    assert!(camera.near < camera.far);
}

#[test]
fn render_validation_clean() {
    use engene::graphics::render_validation::PassValidationResult;

    let r = PassValidationResult {
        pass_name: "test".to_string(),
        visual_correct: true,
        gpu_time_ms: 0.0,
        budget_ceiling_ms: 1.0,
        within_budget: true,
        temporal_stable: true,
        exposure_correct: true,
        tier_fallback_ok: true,
        notes: vec![],
    };
    assert!(r.all_ok());
}

#[test]
fn vegetation_semantics() {
    use engene::graphics::vegetation_semantics::{VegetationSemantics, OcclusionClass};

    let short = VegetationSemantics::short_grass();
    assert_eq!(short.occlusion, OcclusionClass::None);

    let tall = VegetationSemantics::tall_grass();
    assert_eq!(tall.occlusion, OcclusionClass::Partial);

    let bush = VegetationSemantics::dense_bush();
    assert_eq!(bush.occlusion, OcclusionClass::Full);

    let tree = VegetationSemantics::tree();
    assert!(!tree.destructible);

    let dead = VegetationSemantics::dead_tree();
    assert!(dead.flammability > 1.0);
}
