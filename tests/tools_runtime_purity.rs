use engene::core::runtime_config::{RuntimeConfig, RuntimeProfile};
use engene::runtime::bootstrap::ToolsRuntimeAssembly;
use engene::tools::doctor::{run_doctor, DoctorMode};

#[test]
fn tools_runtime_is_tools_profile_and_strict_doctor_clean() {
    let engine = ToolsRuntimeAssembly::minimal();
    let config = engine
        .resources
        .get::<RuntimeConfig>()
        .expect("tools runtime must insert RuntimeConfig");
    assert_eq!(config.profile, RuntimeProfile::Tools);

    let report = run_doctor(&engine, DoctorMode::Strict);
    assert_eq!(
        report.error_count(),
        0,
        "strict doctor should have 0 errors"
    );
    assert_eq!(
        report.warning_count(),
        0,
        "strict doctor should have 0 warnings for tools runtime"
    );
}

#[test]
fn tools_runtime_does_not_boot_world_or_gameplay_content() {
    let engine = ToolsRuntimeAssembly::minimal();

    assert!(
        engine
            .resources
            .get::<engene::world::resources::ResourceGrid>()
            .is_none(),
        "tools runtime must not bootstrap world resource grid"
    );
    assert!(
        engine
            .resources
            .get::<engene::world::heightmap::Heightmap>()
            .is_none(),
        "tools runtime must not bootstrap heightmap/world content"
    );
    assert!(
        engine
            .resources
            .get::<engene::core::game_config::GameConfig>()
            .is_none(),
        "tools runtime must not load game config"
    );
    assert!(
        engine
            .resources
            .get::<engene::core::material_truth::MaterialTruthService>()
            .is_none(),
        "tools runtime must not bootstrap material truth/game authored routing"
    );
    assert!(
        engine
            .resources
            .get::<engene::world::fields::WorldFields>()
            .is_none(),
        "tools runtime must not wire gameplay world fields"
    );
    assert!(
        engine
            .resources
            .get::<engene::physics::ballistics::BallisticsSystem>()
            .is_none(),
        "tools runtime must not wire ballistics"
    );
    assert!(
        engine
            .resources
            .get::<engene::physics::destruction::DestructionSystem>()
            .is_none(),
        "tools runtime must not wire destruction"
    );
    assert!(
        engine
            .resources
            .get::<engene::physics::water::WaterGrid>()
            .is_none(),
        "tools runtime must not wire weather/water gameplay state"
    );

    assert!(
        engine.system_descriptors().is_empty(),
        "tools runtime must not register gameplay systems"
    );
}
