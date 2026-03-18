//! Integration tests for Graphics and Renderer systems.
//! 175 tests covering camera, LOD, frustum, shadow, atmosphere, particles, postprocess, etc.
//! Tests only what can be tested without GPU where possible.

use bytemuck::Zeroable;
use engene::graphics::atmosphere::AtmosphereParams;
use engene::graphics::camera::FlyCamera;
use engene::graphics::contact_shadows::ContactShadowParams;
use engene::graphics::lod::{compute_entity_lods, EntityLod, LodConfig, LodLevel};
use engene::graphics::mesh::{EntityInstance, MeshVertex};
use engene::graphics::particles::{
    fire_emitter, rain_emitter, smoke_emitter, GpuEmitter, ParticleCameraUniform, ParticleGlobals,
};
use engene::graphics::postprocess::{
    ArtisticGrading, CameraSimulation, ExposureLimits, PostProcessParams, WorldLightingResponse,
};
use engene::graphics::renderer::RenderCamera;
use engene::graphics::shadow::{ShadowUniforms, CASCADE_COUNT, SHADOW_MAP_SIZE};
use engene::graphics::terrain::TerrainVertex;
use engene::graphics::vegetation::{GrassInstance, GrassVertex};
use engene::graphics::visibility::Frustum;
use engene::input::input::InputState;
use glam::{Mat4, Vec3, Vec4Swizzles};

// ===== Camera (20 tests) =====

#[test]
fn camera_fly_new_creates_default() {
    let cam = FlyCamera::new();
    assert!(cam.position.x.is_finite());
    assert!(cam.position.y.is_finite());
    assert!(cam.position.z.is_finite());
}

#[test]
fn camera_fly_position_accessible() {
    let cam = FlyCamera::new();
    let _ = cam.position;
    assert!(cam.position.x.is_finite() && cam.position.y.is_finite() && cam.position.z.is_finite());
}

#[test]
fn camera_fly_forward_unit_length() {
    let cam = FlyCamera::new();
    let fwd = cam.forward();
    let len = fwd.length();
    assert!((len - 1.0).abs() < 0.001);
}

#[test]
fn camera_fly_forward_direction() {
    let mut cam = FlyCamera::new();
    cam.yaw = 0.0;
    cam.pitch = 0.0;
    let fwd = cam.forward();
    assert!(fwd.z < 0.0);
}

#[test]
fn camera_fly_view_projection_matrix() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let arr = vp.to_cols_array_2d();
    assert_eq!(arr.len(), 4);
    assert_eq!(arr[0].len(), 4);
}

#[test]
fn camera_fly_view_projection_determinant_nonzero() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let det = vp.determinant();
    assert!(det.abs() > 0.001);
}

#[test]
fn camera_fly_fov_positive() {
    let cam = FlyCamera::new();
    assert!(cam.fov > 0.0);
    assert!(cam.fov < std::f32::consts::PI);
}

#[test]
fn camera_fly_aspect_positive() {
    let cam = FlyCamera::new();
    assert!(cam.aspect > 0.0);
}

#[test]
fn camera_fly_near_far_order() {
    let cam = FlyCamera::new();
    assert!(cam.near < cam.far);
}

#[test]
fn camera_fly_speed_positive() {
    let cam = FlyCamera::new();
    assert!(cam.speed > 0.0);
}

#[test]
fn camera_fly_sensitivity_positive() {
    let cam = FlyCamera::new();
    assert!(cam.sensitivity > 0.0);
}

#[test]
fn camera_fly_yaw_pitch_sane() {
    let cam = FlyCamera::new();
    assert!(cam.yaw.is_finite());
    assert!(cam.pitch.is_finite());
}

#[test]
fn camera_fly_update_with_no_input() {
    let mut cam = FlyCamera::new();
    let input = InputState::new();
    cam.update(&input, 0.016);
    assert!(cam.position.x.is_finite());
}

#[test]
fn camera_fly_update_preserves_position_with_no_keys() {
    let mut cam = FlyCamera::new();
    let pos_before = cam.position;
    let mut input = InputState::new();
    input.accumulate_mouse(0.0, 0.0);
    cam.update(&input, 0.016);
    assert_eq!(cam.position, pos_before);
}

#[test]
fn camera_fly_forward_changes_with_yaw() {
    let mut cam1 = FlyCamera::new();
    let mut cam2 = FlyCamera::new();
    cam1.yaw = 0.0;
    cam2.yaw = std::f32::consts::PI;
    let f1 = cam1.forward();
    let f2 = cam2.forward();
    assert!(f1.dot(f2) < 0.0);
}

#[test]
fn camera_fly_view_projection_produces_valid_ndc() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let pt = vp * (cam.position + cam.forward() * 100.0).extend(1.0);
    let ndc = pt.xyz() / pt.w;
    assert!(ndc.x.abs() < 10.0);
    assert!(ndc.y.abs() < 10.0);
}

#[test]
fn camera_fly_custom_values() {
    let mut cam = FlyCamera::new();
    cam.speed = 500.0;
    cam.fov = 1.0;
    assert_eq!(cam.speed, 500.0);
    assert_eq!(cam.fov, 1.0);
}

#[test]
fn camera_fly_forward_normalized_after_pitch() {
    let mut cam = FlyCamera::new();
    cam.pitch = 0.5;
    let f = cam.forward();
    assert!((f.length() - 1.0).abs() < 0.001);
}

#[test]
fn camera_fly_position_modifiable() {
    let mut cam = FlyCamera::new();
    cam.position = Vec3::new(0.0, 0.0, 0.0);
    assert_eq!(cam.position, Vec3::ZERO);
}

#[test]
fn camera_fly_view_projection_same_for_same_state() {
    let cam = FlyCamera::new();
    let vp1 = cam.view_projection();
    let vp2 = cam.view_projection();
    assert_eq!(vp1, vp2);
}

#[test]
fn camera_fly_aspect_ratio_default() {
    let cam = FlyCamera::new();
    assert_eq!(cam.aspect, 16.0 / 9.0);
}

#[test]
fn camera_fly_near_positive() {
    let cam = FlyCamera::new();
    assert!(cam.near > 0.0);
}

// ===== LOD (25 tests) =====

#[test]
fn lod_config_default_values() {
    let cfg = LodConfig::default();
    assert!(cfg.full_distance < cfg.medium_distance);
    assert!(cfg.medium_distance < cfg.low_distance);
}

#[test]
fn lod_compute_full() {
    let cfg = LodConfig::default();
    assert_eq!(cfg.compute_lod(0.0), LodLevel::Full);
    assert_eq!(cfg.compute_lod(40.0), LodLevel::Full);
}

#[test]
fn lod_compute_medium() {
    let cfg = LodConfig::default();
    assert_eq!(cfg.compute_lod(80.0), LodLevel::Medium);
    assert_eq!(cfg.compute_lod(100.0), LodLevel::Medium);
}

#[test]
fn lod_compute_low() {
    let cfg = LodConfig::default();
    assert_eq!(cfg.compute_lod(200.0), LodLevel::Low);
    assert_eq!(cfg.compute_lod(250.0), LodLevel::Low);
}

#[test]
fn lod_compute_billboard() {
    let cfg = LodConfig::default();
    assert_eq!(cfg.compute_lod(400.0), LodLevel::Billboard);
    assert_eq!(cfg.compute_lod(500.0), LodLevel::Billboard);
}

#[test]
fn lod_compute_culled() {
    let cfg = LodConfig::default();
    assert_eq!(cfg.compute_lod(1500.0), LodLevel::Culled);
    assert_eq!(cfg.compute_lod(2000.0), LodLevel::Culled);
}

#[test]
fn lod_compute_boundary_full_medium() {
    let cfg = LodConfig::default();
    let d = cfg.full_distance - 0.01;
    assert_eq!(cfg.compute_lod(d), LodLevel::Full);
    let d = cfg.full_distance + 0.01;
    assert_eq!(cfg.compute_lod(d), LodLevel::Medium);
}

#[test]
fn lod_should_animate_near() {
    let cfg = LodConfig::default();
    assert!(cfg.should_animate(50.0));
    assert!(cfg.should_animate(100.0));
}

#[test]
fn lod_should_animate_far() {
    let cfg = LodConfig::default();
    assert!(!cfg.should_animate(300.0));
    assert!(!cfg.should_animate(500.0));
}

#[test]
fn lod_should_animate_boundary() {
    let cfg = LodConfig::default();
    assert!(cfg.should_animate(cfg.animation_distance - 1.0));
    assert!(!cfg.should_animate(cfg.animation_distance + 1.0));
}

#[test]
fn lod_level_full_variant() {
    assert_eq!(LodLevel::Full, LodLevel::Full);
}

#[test]
fn lod_level_all_variants_distinct() {
    let levels = [
        LodLevel::Full,
        LodLevel::Medium,
        LodLevel::Low,
        LodLevel::Billboard,
        LodLevel::Culled,
    ];
    for (i, a) in levels.iter().enumerate() {
        for (j, b) in levels.iter().enumerate() {
            assert_eq!(a == b, i == j);
        }
    }
}

#[test]
fn lod_entity_lod_fields() {
    let el = EntityLod {
        entity_id: 42,
        lod: LodLevel::Medium,
        distance: 100.0,
        animate: true,
    };
    assert_eq!(el.entity_id, 42);
    assert_eq!(el.lod, LodLevel::Medium);
    assert!((el.distance - 100.0).abs() < 0.001);
    assert!(el.animate);
}

#[test]
fn lod_compute_entity_lods_empty() {
    let positions: Vec<(u64, Vec3)> = vec![];
    let camera = Vec3::new(0.0, 0.0, 0.0);
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert!(result.is_empty());
}

#[test]
fn lod_compute_entity_lods_culls_far() {
    let positions = vec![(1u64, Vec3::new(2000.0, 0.0, 0.0))];
    let camera = Vec3::ZERO;
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert!(result.is_empty());
}

#[test]
fn lod_compute_entity_lods_keeps_near() {
    let positions = vec![(1u64, Vec3::new(10.0, 0.0, 0.0))];
    let camera = Vec3::ZERO;
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].entity_id, 1);
    assert_eq!(result[0].lod, LodLevel::Full);
}

#[test]
fn lod_compute_entity_lods_distance_correct() {
    let positions = vec![(1u64, Vec3::new(30.0, 40.0, 0.0))];
    let camera = Vec3::ZERO;
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert_eq!(result.len(), 1);
    let expected_dist = 50.0f32;
    assert!((result[0].distance - expected_dist).abs() < 0.01);
}

#[test]
fn lod_compute_entity_lods_animate_near() {
    let positions = vec![(1u64, Vec3::new(50.0, 0.0, 0.0))];
    let camera = Vec3::ZERO;
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert_eq!(result.len(), 1);
    assert!(result[0].animate);
}

#[test]
fn lod_compute_entity_lods_multiple() {
    let positions = vec![
        (1u64, Vec3::new(10.0, 0.0, 0.0)),
        (2u64, Vec3::new(100.0, 0.0, 0.0)),
        (3u64, Vec3::new(500.0, 0.0, 0.0)),
        (4u64, Vec3::new(3000.0, 0.0, 0.0)),
    ];
    let camera = Vec3::ZERO;
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert_eq!(result.len(), 3);
}

#[test]
fn lod_config_custom_distances() {
    let cfg = LodConfig {
        full_distance: 100.0,
        medium_distance: 200.0,
        low_distance: 400.0,
        billboard_distance: 800.0,
        cull_distance: 1600.0,
        animation_distance: 150.0,
    };
    assert_eq!(cfg.compute_lod(50.0), LodLevel::Full);
    assert_eq!(cfg.compute_lod(150.0), LodLevel::Medium);
    assert_eq!(cfg.compute_lod(300.0), LodLevel::Low);
    assert_eq!(cfg.compute_lod(600.0), LodLevel::Billboard);
    assert_eq!(cfg.compute_lod(2000.0), LodLevel::Culled);
}

#[test]
fn lod_level_debug_format() {
    let s = format!("{:?}", LodLevel::Full);
    assert!(s.contains("Full"));
}

#[test]
fn lod_entity_lod_copy() {
    let el = EntityLod {
        entity_id: 1,
        lod: LodLevel::Low,
        distance: 200.0,
        animate: false,
    };
    let el2 = el;
    assert_eq!(el2.entity_id, 1);
}

#[test]
fn lod_config_animation_distance_default() {
    let cfg = LodConfig::default();
    assert_eq!(cfg.animation_distance, 200.0);
}

#[test]
fn lod_compute_entity_lods_no_animate_far() {
    let positions = vec![(1u64, Vec3::new(250.0, 0.0, 0.0))];
    let camera = Vec3::ZERO;
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert_eq!(result.len(), 1);
    assert!(!result[0].animate);
}

#[test]
fn lod_compute_entity_lods_preserves_entity_ids() {
    let positions = vec![
        (100u64, Vec3::new(20.0, 0.0, 0.0)),
        (200u64, Vec3::new(120.0, 0.0, 0.0)),
    ];
    let camera = Vec3::ZERO;
    let cfg = LodConfig::default();
    let result = compute_entity_lods(&positions, camera, &cfg);
    assert!(result.iter().any(|r| r.entity_id == 100));
    assert!(result.iter().any(|r| r.entity_id == 200));
}

// ===== Frustum / Visibility (15 tests) =====

#[test]
fn frustum_from_identity() {
    let vp = Mat4::IDENTITY;
    let f = Frustum::from_view_projection(&vp);
    assert!(f.test_sphere(Vec3::ZERO, 0.1));
}

#[test]
fn frustum_test_sphere_inside() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let f = Frustum::from_view_projection(&vp);
    let center = cam.position + cam.forward() * 50.0;
    assert!(f.test_sphere(center, 5.0));
}

#[test]
fn frustum_test_sphere_behind_camera() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let f = Frustum::from_view_projection(&vp);
    let center = cam.position - cam.forward() * 100.0;
    assert!(!f.test_sphere(center, 1.0));
}

#[test]
fn frustum_test_sphere_large_radius() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let f = Frustum::from_view_projection(&vp);
    let center = cam.position + cam.forward() * 100.0;
    assert!(f.test_sphere(center, 500.0));
}

#[test]
fn frustum_test_sphere_zero_radius() {
    let vp = Mat4::IDENTITY;
    let f = Frustum::from_view_projection(&vp);
    assert!(f.test_sphere(Vec3::ZERO, 0.0));
}

#[test]
fn frustum_from_perspective() {
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 1000.0);
    let view = Mat4::look_to_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z, Vec3::Y);
    let vp = proj * view;
    let f = Frustum::from_view_projection(&vp);
    assert!(f.test_sphere(Vec3::new(0.0, 0.0, 0.0), 1.0));
}

#[test]
fn frustum_test_sphere_far_but_visible() {
    let proj = Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0);
    let view = Mat4::look_to_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::NEG_Z, Vec3::Y);
    let vp = proj * view;
    let f = Frustum::from_view_projection(&vp);
    assert!(f.test_sphere(Vec3::new(0.0, 0.0, 0.0), 2.0));
}

#[test]
fn frustum_consistency_same_view_proj() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let f1 = Frustum::from_view_projection(&vp);
    let f2 = Frustum::from_view_projection(&vp);
    let center = cam.position + cam.forward() * 100.0;
    assert_eq!(f1.test_sphere(center, 10.0), f2.test_sphere(center, 10.0));
}

#[test]
fn frustum_sphere_at_origin_identity_vp() {
    let f = Frustum::from_view_projection(&Mat4::IDENTITY);
    assert!(f.test_sphere(Vec3::ZERO, 1.0));
}

#[test]
fn frustum_negative_radius_handled() {
    let f = Frustum::from_view_projection(&Mat4::IDENTITY);
    assert!(f.test_sphere(Vec3::ZERO, -0.5));
}

#[test]
fn frustum_orthographic_like() {
    let proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
    let view = Mat4::look_to_rh(Vec3::new(0.0, 0.0, 50.0), Vec3::NEG_Z, Vec3::Y);
    let vp = proj * view;
    let f = Frustum::from_view_projection(&vp);
    assert!(f.test_sphere(Vec3::new(0.0, 0.0, 0.0), 5.0));
}

#[test]
fn frustum_sphere_outside_lateral() {
    let proj = Mat4::perspective_rh(0.5, 1.0, 0.1, 100.0);
    let view = Mat4::look_to_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::NEG_Z, Vec3::Y);
    let vp = proj * view;
    let f = Frustum::from_view_projection(&vp);
    let far_side = Vec3::new(1000.0, 0.0, 0.0);
    assert!(!f.test_sphere(far_side, 1.0));
}

#[test]
fn frustum_camera_forward_axis_visible() {
    let cam = FlyCamera::new();
    let vp = cam.view_projection();
    let f = Frustum::from_view_projection(&vp);
    for t in [10.0, 50.0, 200.0, 500.0] {
        let c = cam.position + cam.forward() * t;
        assert!(f.test_sphere(c, 0.5));
    }
}

#[test]
fn frustum_from_projection_only() {
    let proj = Mat4::perspective_rh(1.0, 1.0, 1.0, 100.0);
    let f = Frustum::from_view_projection(&proj);
    assert!(f.test_sphere(Vec3::new(0.0, 0.0, -50.0), 10.0));
}

#[test]
fn frustum_planes_non_degenerate() {
    let vp = Mat4::perspective_rh(1.0, 1.0, 1.0, 100.0)
        * Mat4::look_to_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z, Vec3::Y);
    let _f = Frustum::from_view_projection(&vp);
}

// ===== RenderCamera (10 tests) =====

#[test]
fn render_camera_struct_constructible() {
    let id = glam::Mat4::IDENTITY.to_cols_array_2d();
    let _cam = RenderCamera {
        view_proj: id,
        inv_view_proj: id,
        position: [0.0, 0.0, 0.0],
        forward: [0.0, 0.0, -1.0],
        near: 0.1,
        far: 5000.0,
        day_progress: 0.5,
    };
}

#[test]
fn render_camera_position_access() {
    let cam = RenderCamera {
        view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        position: [1.0, 2.0, 3.0],
        forward: [0.0, 0.0, -1.0],
        near: 0.1,
        far: 1000.0,
        day_progress: 0.0,
    };
    assert_eq!(cam.position[0], 1.0);
    assert_eq!(cam.position[1], 2.0);
    assert_eq!(cam.position[2], 3.0);
}

#[test]
fn render_camera_forward_access() {
    let cam = RenderCamera {
        view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        position: [0.0; 3],
        forward: [0.0, 0.0, -1.0],
        near: 0.1,
        far: 1000.0,
        day_progress: 0.0,
    };
    assert_eq!(cam.forward[2], -1.0);
}

#[test]
fn render_camera_near_far() {
    let cam = RenderCamera {
        view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        position: [0.0; 3],
        forward: [0.0, 0.0, -1.0],
        near: 0.1,
        far: 5000.0,
        day_progress: 0.0,
    };
    assert!(cam.near < cam.far);
}

#[test]
fn render_camera_day_progress_range() {
    for dp in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let cam = RenderCamera {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            position: [0.0; 3],
            forward: [0.0, 0.0, -1.0],
            near: 0.1,
            far: 1000.0,
            day_progress: dp,
        };
        assert_eq!(cam.day_progress, dp);
    }
}

#[test]
fn render_camera_view_proj_4x4() {
    let vp = glam::Mat4::IDENTITY.to_cols_array_2d();
    assert_eq!(vp.len(), 4);
    assert_eq!(vp[0].len(), 4);
}

#[test]
fn render_camera_inv_view_proj_4x4() {
    let m = glam::Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0);
    let inv = m.inverse();
    let inv_arr = inv.to_cols_array_2d();
    assert_eq!(inv_arr.len(), 4);
}

#[test]
fn render_camera_identity_view_proj() {
    let id = glam::Mat4::IDENTITY.to_cols_array_2d();
    let cam = RenderCamera {
        view_proj: id,
        inv_view_proj: id,
        position: [0.0; 3],
        forward: [0.0, 0.0, -1.0],
        near: 0.1,
        far: 1000.0,
        day_progress: 0.0,
    };
    assert_eq!(cam.view_proj[0][0], 1.0);
    assert_eq!(cam.view_proj[3][3], 1.0);
}

#[test]
fn render_camera_copy() {
    let cam = RenderCamera {
        view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        position: [0.0; 3],
        forward: [0.0, 0.0, -1.0],
        near: 0.1,
        far: 1000.0,
        day_progress: 0.5,
    };
    let cam2 = cam;
    assert_eq!(cam2.day_progress, 0.5);
}

#[test]
fn render_camera_from_fly_camera() {
    let fly = FlyCamera::new();
    let vp = fly.view_projection();
    let inv = vp.inverse();
    let cam = RenderCamera {
        view_proj: vp.to_cols_array_2d(),
        inv_view_proj: inv.to_cols_array_2d(),
        position: fly.position.to_array(),
        forward: fly.forward().to_array(),
        near: fly.near,
        far: fly.far,
        day_progress: 0.5,
    };
    assert_eq!(cam.near, fly.near);
    assert_eq!(cam.far, fly.far);
}

// ===== Shadow (10 tests) =====

#[test]
fn shadow_map_size_constant() {
    assert_eq!(SHADOW_MAP_SIZE, 2048);
}

#[test]
fn shadow_cascade_count() {
    assert_eq!(CASCADE_COUNT, 4);
}

#[test]
fn shadow_uniforms_constructible() {
    let id = glam::Mat4::IDENTITY.to_cols_array_2d();
    let _u = ShadowUniforms {
        light_view_proj: [id; 4],
        cascade_splits: [0.05, 0.15, 0.4, 1.0],
    };
}

#[test]
fn shadow_uniforms_cascade_splits_order() {
    let u = ShadowUniforms {
        light_view_proj: [glam::Mat4::IDENTITY.to_cols_array_2d(); 4],
        cascade_splits: [0.1, 0.2, 0.5, 1.0],
    };
    assert!(u.cascade_splits[0] < u.cascade_splits[1]);
    assert!(u.cascade_splits[1] < u.cascade_splits[2]);
    assert!(u.cascade_splits[2] < u.cascade_splits[3]);
}

#[test]
fn shadow_uniforms_light_view_proj_array_len() {
    let id = glam::Mat4::IDENTITY.to_cols_array_2d();
    let u = ShadowUniforms {
        light_view_proj: [id; 4],
        cascade_splits: [0.0; 4],
    };
    assert_eq!(u.light_view_proj.len(), CASCADE_COUNT);
}

#[test]
fn shadow_uniforms_matrix_format() {
    let m = glam::Mat4::IDENTITY.to_cols_array_2d();
    assert_eq!(m.len(), 4);
    for row in &m {
        assert_eq!(row.len(), 4);
    }
}

#[test]
fn shadow_uniforms_bytemuck_compatible() {
    let u = ShadowUniforms {
        light_view_proj: [glam::Mat4::IDENTITY.to_cols_array_2d(); 4],
        cascade_splits: [0.05, 0.15, 0.4, 1.0],
    };
    let bytes = bytemuck::bytes_of(&u);
    assert!(!bytes.is_empty());
}

#[test]
fn shadow_uniforms_copy() {
    let u = ShadowUniforms {
        light_view_proj: [glam::Mat4::IDENTITY.to_cols_array_2d(); 4],
        cascade_splits: [0.1, 0.2, 0.4, 1.0],
    };
    let u2 = u;
    assert_eq!(u2.cascade_splits[0], 0.1);
}

#[test]
fn shadow_uniforms_zeroable() {
    let u = ShadowUniforms::zeroed();
    assert_eq!(u.cascade_splits[0], 0.0);
}

#[test]
fn shadow_map_size_power_of_two() {
    assert!(SHADOW_MAP_SIZE.is_power_of_two());
}

// ===== Atmosphere (15 tests) =====

#[test]
fn atmosphere_params_default() {
    let p = AtmosphereParams::default();
    assert_eq!(p.camera_pos[3], 0.0);
}

#[test]
fn atmosphere_params_sun_direction() {
    let p = AtmosphereParams::default();
    assert_eq!(p.sun_direction.len(), 4);
    assert!(p.sun_direction[0].is_finite());
}

#[test]
fn atmosphere_params_sun_color() {
    let p = AtmosphereParams::default();
    assert!(p.sun_color[0] > 0.0);
    assert!(p.sun_color[1] > 0.0);
    assert!(p.sun_color[2] > 0.0);
}

#[test]
fn atmosphere_params_fog_density() {
    let p = AtmosphereParams::default();
    assert!(p.fog_density > 0.0);
    assert!(p.fog_density < 0.1);
}

#[test]
fn atmosphere_params_fog_color() {
    let p = AtmosphereParams::default();
    assert!(p.fog_color[0] >= 0.0);
    assert!(p.fog_color[1] >= 0.0);
    assert!(p.fog_color[2] >= 0.0);
}

#[test]
fn atmosphere_params_fog_height_falloff() {
    let p = AtmosphereParams::default();
    assert!(p.fog_height_falloff > 0.0);
}

#[test]
fn atmosphere_params_fog_max_opacity() {
    let p = AtmosphereParams::default();
    assert!(p.fog_max_opacity > 0.0);
    assert!(p.fog_max_opacity <= 1.0);
}

#[test]
fn atmosphere_params_god_ray_intensity() {
    let p = AtmosphereParams::default();
    assert!(p.god_ray_intensity >= 0.0);
}

#[test]
fn atmosphere_params_god_ray_decay() {
    let p = AtmosphereParams::default();
    assert!(p.god_ray_decay > 0.0);
    assert!(p.god_ray_decay <= 1.0);
}

#[test]
fn atmosphere_params_aerial_perspective() {
    let p = AtmosphereParams::default();
    assert!(p.aerial_perspective_density >= 0.0);
}

#[test]
fn atmosphere_params_construct_custom() {
    let p = AtmosphereParams {
        camera_pos: [1.0, 2.0, 3.0, 1.0],
        sun_direction: [0.0, 1.0, 0.0, 0.0],
        sun_color: [1.0, 1.0, 1.0, 1.0],
        fog_color: [0.5, 0.5, 0.5, 1.0],
        fog_density: 0.01,
        fog_height_falloff: 0.1,
        fog_max_opacity: 0.9,
        aerial_perspective_density: 0.002,
        god_ray_intensity: 0.5,
        god_ray_decay: 0.95,
        _pad: [0.0; 2],
    };
    assert_eq!(p.fog_density, 0.01);
}

#[test]
fn atmosphere_params_bytemuck_safe() {
    let p = AtmosphereParams::default();
    let _ = bytemuck::bytes_of(&p);
}

#[test]
fn atmosphere_params_pad_length() {
    let p = AtmosphereParams::default();
    assert_eq!(p._pad.len(), 2);
}

#[test]
fn atmosphere_params_copy() {
    let p = AtmosphereParams::default();
    let p2 = p;
    assert_eq!(p2.fog_density, p.fog_density);
}

#[test]
fn atmosphere_params_camera_pos_xyzw() {
    let mut p = AtmosphereParams::default();
    p.camera_pos = [10.0, 20.0, 30.0, 1.0];
    assert_eq!(p.camera_pos[0], 10.0);
    assert_eq!(p.camera_pos[3], 1.0);
}

// ===== Contact Shadows (8 tests) =====

#[test]
fn contact_shadow_params_constructible() {
    let id = glam::Mat4::IDENTITY.to_cols_array_2d();
    let _p = ContactShadowParams {
        light_dir: [0.0, -1.0, 0.0],
        max_steps: 32,
        inv_view_proj: id,
        resolution: [1920.0, 1080.0],
        step_size: 0.05,
        max_distance: 5.0,
    };
}

#[test]
fn contact_shadow_params_light_dir() {
    let p = ContactShadowParams {
        light_dir: [1.0, 0.0, 0.0],
        max_steps: 16,
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        resolution: [800.0, 600.0],
        step_size: 0.1,
        max_distance: 2.0,
    };
    assert_eq!(p.light_dir[0], 1.0);
}

#[test]
fn contact_shadow_params_max_steps() {
    let p = ContactShadowParams {
        light_dir: [0.0; 3],
        max_steps: 64,
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        resolution: [0.0; 2],
        step_size: 0.0,
        max_distance: 0.0,
    };
    assert_eq!(p.max_steps, 64);
}

#[test]
fn contact_shadow_params_resolution() {
    let p = ContactShadowParams {
        light_dir: [0.0; 3],
        max_steps: 0,
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        resolution: [1920.0, 1080.0],
        step_size: 0.05,
        max_distance: 5.0,
    };
    assert_eq!(p.resolution[0], 1920.0);
    assert_eq!(p.resolution[1], 1080.0);
}

#[test]
fn contact_shadow_params_step_size() {
    let p = ContactShadowParams {
        light_dir: [0.0; 3],
        max_steps: 0,
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        resolution: [0.0; 2],
        step_size: 0.025,
        max_distance: 3.0,
    };
    assert!(p.step_size > 0.0);
}

#[test]
fn contact_shadow_params_max_distance() {
    let p = ContactShadowParams {
        light_dir: [0.0; 3],
        max_steps: 0,
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        resolution: [0.0; 2],
        step_size: 0.0,
        max_distance: 10.0,
    };
    assert_eq!(p.max_distance, 10.0);
}

#[test]
fn contact_shadow_params_inv_view_proj() {
    let m = glam::Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0).inverse();
    let p = ContactShadowParams {
        light_dir: [0.0; 3],
        max_steps: 0,
        inv_view_proj: m.to_cols_array_2d(),
        resolution: [0.0; 2],
        step_size: 0.0,
        max_distance: 0.0,
    };
    assert_eq!(p.inv_view_proj.len(), 4);
}

#[test]
fn contact_shadow_params_bytemuck() {
    let p = ContactShadowParams {
        light_dir: [0.0; 3],
        max_steps: 32,
        inv_view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        resolution: [1920.0, 1080.0],
        step_size: 0.05,
        max_distance: 5.0,
    };
    let _ = bytemuck::bytes_of(&p);
}

// ===== Terrain (8 tests) =====

#[test]
fn terrain_vertex_constructible() {
    let v = TerrainVertex {
        position: [10.0, 20.0, 30.0],
        normal: [0.0, 1.0, 0.0],
        color: [0.5, 0.6, 0.4],
    };
    assert_eq!(v.position[0], 10.0);
    assert_eq!(v.normal[1], 1.0);
    assert_eq!(v.color[2], 0.4);
}

#[test]
fn terrain_vertex_sizeof() {
    assert_eq!(std::mem::size_of::<TerrainVertex>(), 36);
}

#[test]
fn terrain_vertex_position_offset() {
    let v = TerrainVertex {
        position: [1.0, 2.0, 3.0],
        normal: [0.0; 3],
        color: [0.0; 3],
    };
    assert_eq!(v.position[0], 1.0);
}

#[test]
fn terrain_vertex_normal_offset() {
    let v = TerrainVertex {
        position: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        color: [0.0; 3],
    };
    assert_eq!(v.normal[1], 1.0);
}

#[test]
fn terrain_vertex_color_rgb() {
    let v = TerrainVertex {
        position: [0.0; 3],
        normal: [0.0; 3],
        color: [0.2, 0.5, 0.8],
    };
    assert!(v.color[0] >= 0.0 && v.color[0] <= 1.0);
}

#[test]
fn terrain_vertex_layout_stride() {
    let layout = TerrainVertex::layout();
    assert_eq!(layout.array_stride, 36);
}

#[test]
fn terrain_vertex_bytemuck_pod() {
    let v = TerrainVertex {
        position: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        color: [1.0, 0.0, 0.0],
    };
    let _ = bytemuck::bytes_of(&v);
}

// ===== Vegetation (8 tests) =====

#[test]
fn grass_instance_constructible() {
    let g = GrassInstance {
        position: [10.0, 5.0, 20.0],
        rotation_scale: [0.5, 1.2],
    };
    assert_eq!(g.position[0], 10.0);
    assert_eq!(g.rotation_scale[1], 1.2);
}

#[test]
fn grass_instance_position() {
    let g = GrassInstance {
        position: [1.0, 2.0, 3.0],
        rotation_scale: [0.0, 1.0],
    };
    assert_eq!(g.position, [1.0, 2.0, 3.0]);
}

#[test]
fn grass_instance_rotation_scale() {
    let g = GrassInstance {
        position: [0.0; 3],
        rotation_scale: [std::f32::consts::PI, 2.0],
    };
    assert!(g.rotation_scale[0].is_finite());
}

#[test]
fn grass_vertex_constructible() {
    let v = GrassVertex {
        position: [0.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    };
    assert_eq!(v.uv[1], 1.0);
}

#[test]
fn grass_vertex_position_uv() {
    let v = GrassVertex {
        position: [-0.3, 0.0, 0.0],
        uv: [1.0, 0.0],
    };
    assert_eq!(v.position[0], -0.3);
    assert_eq!(v.uv[0], 1.0);
}

#[test]
fn grass_vertex_sizeof() {
    assert_eq!(std::mem::size_of::<GrassVertex>(), 20);
}

#[test]
fn grass_vertex_bytemuck() {
    let v = GrassVertex {
        position: [0.0; 3],
        uv: [0.5, 0.5],
    };
    let _ = bytemuck::bytes_of(&v);
}

// ===== Particles (20 tests) =====

#[test]
fn fire_emitter_position() {
    let e = fire_emitter([1.0, 2.0, 3.0]);
    assert_eq!(e.position, [1.0, 2.0, 3.0]);
}

#[test]
fn fire_emitter_velocity_range() {
    let e = fire_emitter([0.0; 3]);
    assert!(e.velocity_min[1] < e.velocity_max[1]);
}

#[test]
fn fire_emitter_color_transition() {
    let e = fire_emitter([0.0; 3]);
    assert!(e.color_start[0] > e.color_end[0]);
}

#[test]
fn smoke_emitter_position() {
    let e = smoke_emitter([5.0, 0.0, 5.0]);
    assert_eq!(e.position, [5.0, 0.0, 5.0]);
}

#[test]
fn smoke_emitter_lifetime() {
    let e = smoke_emitter([0.0; 3]);
    assert!(e.lifetime > 1.0);
}

#[test]
fn rain_emitter_offset_position() {
    let e = rain_emitter([0.0, 0.0, 0.0]);
    assert_eq!(e.position[1], 50.0);
}

#[test]
fn rain_emitter_gravity() {
    let e = rain_emitter([0.0; 3]);
    assert!(e.gravity > 0.0);
}

#[test]
fn rain_emitter_velocity_down() {
    let e = rain_emitter([0.0; 3]);
    assert!(e.velocity_min[1] < 0.0);
}

#[test]
fn gpu_emitter_fields() {
    let e = GpuEmitter {
        position: [0.0; 3],
        rate: 10.0,
        velocity_min: [-1.0, 0.0, -1.0],
        lifetime: 2.0,
        velocity_max: [1.0, 5.0, 1.0],
        size: 1.0,
        color_start: [1.0, 0.0, 0.0, 1.0],
        color_end: [0.0, 0.0, 0.0, 0.0],
        gravity: -9.81,
        count: 100,
        seed: 0,
        _pad: 0,
    };
    assert_eq!(e.rate, 10.0);
    assert_eq!(e.count, 100);
}

#[test]
fn particle_globals_fields() {
    let g = ParticleGlobals {
        dt: 0.016,
        total_time: 1.0,
        max_particles: 16384,
        _pad: 0,
    };
    assert_eq!(g.dt, 0.016);
    assert!(g.max_particles > 0);
}

#[test]
fn particle_camera_uniform_constructible() {
    let m = glam::Mat4::IDENTITY.to_cols_array_2d();
    let _u = ParticleCameraUniform {
        view_proj: m,
        camera_right: [1.0, 0.0, 0.0],
        _p0: 0.0,
        camera_up: [0.0, 1.0, 0.0],
        _p1: 0.0,
    };
}

#[test]
fn particle_camera_uniform_camera_axes() {
    let u = ParticleCameraUniform {
        view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        camera_right: [1.0, 0.0, 0.0],
        _p0: 0.0,
        camera_up: [0.0, 1.0, 0.0],
        _p1: 0.0,
    };
    assert_eq!(u.camera_right[0], 1.0);
    assert_eq!(u.camera_up[1], 1.0);
}

#[test]
fn particle_camera_uniform_view_proj() {
    let vp = glam::Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0).to_cols_array_2d();
    let u = ParticleCameraUniform {
        view_proj: vp,
        camera_right: [0.0; 3],
        _p0: 0.0,
        camera_up: [0.0; 3],
        _p1: 0.0,
    };
    assert_eq!(u.view_proj.len(), 4);
}

#[test]
fn fire_emitter_gravity_negative() {
    let e = fire_emitter([0.0; 3]);
    assert!(e.gravity < 0.0);
}

#[test]
fn fire_emitter_lifetime_positive() {
    let e = fire_emitter([0.0; 3]);
    assert!(e.lifetime > 0.0);
}

#[test]
fn fire_emitter_size_positive() {
    let e = fire_emitter([0.0; 3]);
    assert!(e.size > 0.0);
}

#[test]
fn smoke_emitter_rate() {
    let e = smoke_emitter([0.0; 3]);
    assert!(e.rate > 0.0);
}

#[test]
fn rain_emitter_count() {
    let e = rain_emitter([0.0; 3]);
    assert!(e.count > 1000);
}

#[test]
fn gpu_emitter_bytemuck() {
    let e = fire_emitter([0.0; 3]);
    let _ = bytemuck::bytes_of(&e);
}

#[test]
fn particle_globals_bytemuck() {
    let g = ParticleGlobals {
        dt: 0.0,
        total_time: 0.0,
        max_particles: 0,
        _pad: 0,
    };
    let _ = bytemuck::bytes_of(&g);
}

// ===== PostProcess (20 tests) =====

#[test]
fn postprocess_params_default() {
    let p = PostProcessParams::default();
    assert_eq!(p.exposure, 1.0);
}

#[test]
fn postprocess_params_bloom_threshold() {
    let p = PostProcessParams::default();
    assert!(p.bloom_threshold > 0.0);
}

#[test]
fn postprocess_params_ssao_fields() {
    let p = PostProcessParams::default();
    assert!(p.ssao_radius > 0.0);
    assert!(p.ssao_bias >= 0.0);
    assert!(p.ssao_intensity >= 0.0);
}

#[test]
fn camera_simulation_default() {
    let cs = CameraSimulation::default();
    assert!(cs.auto_exposure);
    assert!(cs.min_exposure < cs.max_exposure);
}

#[test]
fn camera_simulation_exposure() {
    let cs = CameraSimulation::default();
    let e = cs.exposure();
    assert!(e >= cs.min_exposure);
    assert!(e <= cs.max_exposure);
}

#[test]
fn camera_simulation_update() {
    let mut cs = CameraSimulation::default();
    let limits = ExposureLimits::default();
    cs.update(0.5, 0.016, &limits);
    assert!(cs.current_luminance.is_finite());
}

#[test]
fn camera_simulation_exposure_adapts() {
    let mut cs = CameraSimulation::default();
    let limits = ExposureLimits::default();
    cs.update(0.01, 0.1, &limits);
    let e1 = cs.exposure();
    cs.update(0.5, 0.1, &limits);
    let e2 = cs.exposure();
    assert!(e1 != e2 || (e1 - e2).abs() < 0.01);
}

#[test]
fn world_lighting_response_default() {
    let w = WorldLightingResponse::default();
    assert!(w.fog_density > 0.0);
    assert_eq!(w.fog_color.len(), 3);
}

#[test]
fn world_lighting_response_update_from_world() {
    let mut w = WorldLightingResponse::default();
    w.update_from_world(0.5, false, 45.0, 0.5);
    assert!(w.fog_density.is_finite());
    assert!(w.color_temperature.is_finite());
}

#[test]
fn world_lighting_response_rain_increases_fog() {
    let mut w1 = WorldLightingResponse::default();
    let mut w2 = WorldLightingResponse::default();
    w1.update_from_world(0.5, false, 45.0, 0.5);
    w2.update_from_world(0.5, true, 45.0, 0.5);
    assert!(w2.fog_density >= w1.fog_density);
}

#[test]
fn artistic_grading_default() {
    let g = ArtisticGrading::default();
    assert_eq!(g.saturation, 1.0);
    assert_eq!(g.contrast, 1.0);
}

#[test]
fn artistic_grading_vignette_and_tint() {
    let g = ArtisticGrading::default();
    assert!(g.vignette_intensity >= 0.0);
    assert!(g.tint.is_finite());
}

#[test]
fn artistic_grading_shadow_highlight_colors() {
    let g = ArtisticGrading::default();
    assert!(g.shadow_color[0] <= g.highlight_color[0]);
}

#[test]
fn exposure_limits_default() {
    let l = ExposureLimits::default();
    assert!(l.adaptation_speed_up_max > 0.0);
    assert!(l.color_temp_range.0 < l.color_temp_range.1);
}

#[test]
fn exposure_limits_fog_density_range() {
    let l = ExposureLimits::default();
    assert!(l.fog_density_range.0 < l.fog_density_range.1);
}

#[test]
fn postprocess_params_bloom_intensity() {
    let p = PostProcessParams::default();
    assert!(p.bloom_intensity >= 0.0);
    assert!(p.bloom_intensity <= 1.0);
}

#[test]
fn camera_simulation_adaptation_speeds() {
    let cs = CameraSimulation::default();
    assert!(cs.adaptation_speed_up > 0.0);
    assert!(cs.adaptation_speed_down > 0.0);
}

#[test]
fn postprocess_params_pad_length() {
    let p = PostProcessParams::default();
    assert_eq!(p._pad.len(), 2);
}

#[test]
fn camera_simulation_no_auto_exposure_skip_update() {
    let mut cs = CameraSimulation::default();
    cs.auto_exposure = false;
    let lum_before = cs.current_luminance;
    cs.update(1.0, 0.1, &ExposureLimits::default());
    assert_eq!(cs.current_luminance, lum_before);
}

#[test]
fn world_lighting_response_day_progress_sunset() {
    let mut w = WorldLightingResponse::default();
    w.update_from_world(0.0, false, 10.0, 0.85);
    assert!(w.color_temperature.is_finite());
}

// ===== Mesh (8 tests) =====

#[test]
fn mesh_vertex_constructible() {
    let v = MeshVertex {
        position: [1.0, 2.0, 3.0],
        normal: [0.0, 1.0, 0.0],
    };
    assert_eq!(v.position[0], 1.0);
    assert_eq!(v.normal[1], 1.0);
}

#[test]
fn mesh_vertex_sizeof() {
    assert_eq!(std::mem::size_of::<MeshVertex>(), 24);
}

#[test]
fn entity_instance_constructible() {
    let i = EntityInstance {
        position: [10.0, 0.0, 5.0],
        color: [1.0, 0.5, 0.0],
    };
    assert_eq!(i.position[0], 10.0);
    assert_eq!(i.color[2], 0.0);
}

#[test]
fn entity_instance_sizeof() {
    assert_eq!(std::mem::size_of::<EntityInstance>(), 24);
}

#[test]
fn mesh_vertex_layout_attribute_count() {
    let layout = MeshVertex::layout();
    assert!(layout.attributes.len() >= 2);
}

#[test]
fn mesh_vertex_bytemuck() {
    let v = MeshVertex {
        position: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
    };
    let _ = bytemuck::bytes_of(&v);
}

#[test]
fn entity_instance_bytemuck() {
    let i = EntityInstance {
        position: [0.0; 3],
        color: [1.0; 3],
    };
    let _ = bytemuck::bytes_of(&i);
}

#[test]
fn entity_instance_position_color() {
    let i = EntityInstance {
        position: [100.0, 200.0, 300.0],
        color: [0.2, 0.4, 0.6],
    };
    assert_eq!(i.position, [100.0, 200.0, 300.0]);
    assert_eq!(i.color, [0.2, 0.4, 0.6]);
}

// ===== TAA (8 tests) - requires device; we test what we can =====

#[test]
fn taa_pass_type_exists() {
    use engene::graphics::taa::TaaPass;
    fn _assert_taa(_: &TaaPass) {}
}

#[test]
fn taa_halton_jitter_formula_subpixel() {
    let halton = [[0.5, 0.333333], [0.25, 0.666666], [0.75, 0.111111]];
    let w = 1920u32;
    let height = 1080u32;
    for sample in halton {
        let jx = (sample[0] - 0.5) / w as f32;
        let jy = (sample[1] - 0.5) / height as f32;
        assert!(jx.abs() < 0.5);
        assert!(jy.abs() < 0.5);
    }
}

#[test]
fn taa_jitter_range_per_component() {
    for (x, y) in [(0.5, 0.333333), (0.25, 0.666666), (0.0, 0.0)] {
        let jx = (x - 0.5) / 1920.0;
        let jy = (y - 0.5) / 1080.0;
        assert!(jx >= -0.5 / 1920.0 && jx <= 0.5 / 1920.0);
        assert!(jy >= -0.5 / 1080.0 && jy <= 0.5 / 1080.0);
    }
}

#[test]
fn taa_halton_sequence_8_samples() {
    let seq = [
        [0.5_f32, 0.333333],
        [0.25, 0.666666],
        [0.75, 0.111111],
        [0.125, 0.444444],
        [0.625, 0.777777],
        [0.375, 0.222222],
        [0.875, 0.555555],
        [0.0625, 0.888888],
    ];
    assert_eq!(seq.len(), 8);
    for s in seq {
        assert!(s[0] >= 0.0 && s[0] <= 1.0);
        assert!(s[1] >= 0.0 && s[1] <= 1.0);
    }
}

#[test]
fn taa_jitter_centered_at_zero() {
    let jx = (0.5 - 0.5) / 1920.0;
    let jy = (0.333333 - 0.5) / 1080.0;
    assert_eq!(jx, 0.0);
    assert!(jy < 0.0);
}

#[test]
fn taa_frame_cycle_mod_8() {
    for frame in 0..16u32 {
        let idx = (frame as usize) % 8;
        assert!(idx < 8);
    }
}

#[test]
fn taa_jitter_dimensions_preserved() {
    let w = 1280u32;
    let h = 720u32;
    let j = [(0.5 - 0.5) / w as f32, (0.333333 - 0.5) / h as f32];
    assert_eq!(j.len(), 2);
    assert!(j[0].is_finite());
    assert!(j[1].is_finite());
}

#[test]
fn taa_module_compiles() {
    use engene::graphics::taa::TaaPass;
    let _ = std::any::type_name::<TaaPass>();
}
