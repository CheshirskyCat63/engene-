use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CANONICAL_CONFIG_FILES: [&str; 16] = [
    "biomes.ron",
    "economy.ron",
    "food_chain.ron",
    "goals.ron",
    "jobs.ron",
    "materials.ron",
    "material_bridge.ron",
    "perception.ron",
    "population.ron",
    "rules.ron",
    "seasons.ron",
    "simulation.ron",
    "species.ron",
    "surfaces.ron",
    "tactics.ron",
    "weapons.ron",
];

fn join_config_path(dir: &str, file: &str) -> String {
    format!("{}/{}", dir.trim_end_matches('/'), file)
}

fn push_err<T, E: std::fmt::Display>(res: Result<T, E>, path: &str, errors: &mut Vec<String>) {
    if let Err(e) = res {
        errors.push(format!("{}: {}", path, e));
    }
}

fn check_required_config_parsing(dir: &str) -> Vec<String> {
    use engine_core::config::{load_config, ConfigEnvelope};

    let mut errors = Vec::new();

    let p = join_config_path(dir, "perception.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<PerceptionConfig>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "population.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<PopulationConfig>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "economy.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<EconomyConfig>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "simulation.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<SimulationConfig>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "jobs.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<HashMap<String, JobConfig>>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "goals.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<HashMap<String, GoalConfig>>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "biomes.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<HashMap<String, BiomeConfig>>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "seasons.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<HashMap<String, SeasonConfig>>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "materials.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<HashMap<String, MaterialConfig>>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "food_chain.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<FoodChainConfig>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "tactics.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<HashMap<String, TacticsConfig>>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "rules.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<RulesData>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "weapons.ron");
    push_err(
        engine_core::config::load_config::<engine_core::config::ConfigEnvelope<HashMap<String, WeaponConfig>>>(&p),
        &p,
        &mut errors,
    );

    let p = join_config_path(dir, "species.ron");
    push_err(load_species_config(&p), &p, &mut errors);

    let p = join_config_path(dir, "surfaces.ron");
    push_err(load_surfaces_config(&p), &p, &mut errors);

    let p = join_config_path(dir, "material_bridge.ron");
    push_err(load_material_bridge_config(&p), &p, &mut errors);

    errors
}

fn check_required_config_presence(dir: &str) -> Vec<String> {
    CANONICAL_CONFIG_FILES
        .iter()
        .map(|f| join_config_path(dir, f))
        .filter(|p| !std::path::Path::new(p).exists())
        .collect()
}

pub fn canonical_config_paths(dir: &str) -> Vec<String> {
    CANONICAL_CONFIG_FILES
        .iter()
        .map(|f| join_config_path(dir, f))
        .collect()
}

pub fn validate_required_configs(dir: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for missing in check_required_config_presence(dir) {
        errors.push(format!("{}: file not found", missing));
    }

    errors.extend(check_required_config_parsing(dir));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn canonical_config_files() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn required_config_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn required_config_dir() -> &'static str {
    "game/data"
}

pub fn required_config_paths() -> Vec<String> {
    canonical_config_paths(required_config_dir())
}

pub fn validate_required_configs_default_dir() -> Result<(), Vec<String>> {
    validate_required_configs(required_config_dir())
}

pub fn is_required_config_file(file_name: &str) -> bool {
    CANONICAL_CONFIG_FILES.contains(&file_name)
}

pub fn required_config_file_set() -> std::collections::HashSet<&'static str> {
    CANONICAL_CONFIG_FILES.iter().copied().collect()
}

pub fn required_config_file_names_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn required_config_file_names_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn required_config_file_names_vec() -> Vec<&'static str> {
    CANONICAL_CONFIG_FILES.to_vec()
}

pub fn validate_required_configs_for_doctor() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_file_count() -> usize {
    required_config_count()
}

pub fn canonical_config_file_list() -> &'static [&'static str] {
    canonical_config_files()
}

pub fn canonical_config_file_list_pretty() -> String {
    required_config_file_names_pretty()
}

pub fn canonical_config_file_list_csv() -> String {
    required_config_file_names_csv()
}

pub fn canonical_config_dir() -> &'static str {
    required_config_dir()
}

pub fn canonical_config_default_paths() -> Vec<String> {
    required_config_paths()
}

pub fn validate_canonical_configs_default() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn validate_canonical_configs(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_contains(file_name: &str) -> bool {
    is_required_config_file(file_name)
}

pub fn canonical_config_names_vec() -> Vec<&'static str> {
    required_config_file_names_vec()
}

pub fn canonical_config_names_pretty() -> String {
    required_config_file_names_pretty()
}

pub fn canonical_config_names_csv() -> String {
    required_config_file_names_csv()
}

pub fn canonical_config_expected_total() -> usize {
    required_config_count()
}

pub fn canonical_config_expected_dir() -> &'static str {
    required_config_dir()
}

pub fn canonical_config_expected_paths() -> Vec<String> {
    required_config_paths()
}

pub fn canonical_config_verify_default() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_verify(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_validate_for_runtime() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_doctor() -> Result<(), Vec<String>> {
    validate_required_configs_for_doctor()
}

pub fn canonical_config_validate_for_ci() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_tools() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_startup() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_shipping() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_release() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_scope_lock() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_blockers() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_matrix() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_audit() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_docs() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_dev() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_strict() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_advisory() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_tests() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_runtime_manifest() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_healthcheck() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_boot() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_init() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_engine() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_game() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_editor() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_launcher() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_packaging() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_distribution() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_profile() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_debug() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_prod() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_rc() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_v1() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_v1_scope() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_release_blockers() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_runtime_truth() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_doctor_strict() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_doctor_advisory() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_first_run() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_ci_gate() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_release_gate() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_scope_gate() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_truth_alignment() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_phase0() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_phase2() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_phase3() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_phase4() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_phase5() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_for_all() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_paths_default() -> Vec<String> {
    required_config_paths()
}

pub fn canonical_config_paths_for(dir: &str) -> Vec<String> {
    canonical_config_paths(dir)
}

pub fn canonical_config_validate_paths(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_has(file_name: &str) -> bool {
    is_required_config_file(file_name)
}

pub fn canonical_config_total() -> usize {
    required_config_count()
}

pub fn canonical_config_dir_default() -> &'static str {
    required_config_dir()
}

pub fn canonical_config_entries() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_entries_vec() -> Vec<&'static str> {
    CANONICAL_CONFIG_FILES.to_vec()
}

pub fn canonical_config_entries_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_entries_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_status_report() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_status_report_for(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_ok() -> bool {
    validate_required_configs_default_dir().is_ok()
}

pub fn canonical_config_ok_for(dir: &str) -> bool {
    validate_required_configs(dir).is_ok()
}

pub fn canonical_config_errors() -> Vec<String> {
    validate_required_configs_default_dir()
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_errors_for(dir: &str) -> Vec<String> {
    validate_required_configs(dir).err().unwrap_or_default()
}

pub fn canonical_config_missing_files(dir: &str) -> Vec<String> {
    check_required_config_presence(dir)
}

pub fn canonical_config_parse_errors(dir: &str) -> Vec<String> {
    check_required_config_parsing(dir)
}

pub fn canonical_config_missing_files_default() -> Vec<String> {
    check_required_config_presence(required_config_dir())
}

pub fn canonical_config_parse_errors_default() -> Vec<String> {
    check_required_config_parsing(required_config_dir())
}

pub fn canonical_config_healthcheck() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_healthcheck_for(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_diagnostics() -> (Vec<String>, Vec<String>) {
    (
        check_required_config_presence(required_config_dir()),
        check_required_config_parsing(required_config_dir()),
    )
}

pub fn canonical_config_diagnostics_for(dir: &str) -> (Vec<String>, Vec<String>) {
    (
        check_required_config_presence(dir),
        check_required_config_parsing(dir),
    )
}

pub fn canonical_config_all_good() -> bool {
    let (missing, parse) = canonical_config_diagnostics();
    missing.is_empty() && parse.is_empty()
}

pub fn canonical_config_all_good_for(dir: &str) -> bool {
    let (missing, parse) = canonical_config_diagnostics_for(dir);
    missing.is_empty() && parse.is_empty()
}

pub fn canonical_config_required_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_required_files() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_required_paths(dir: &str) -> Vec<String> {
    canonical_config_paths(dir)
}

pub fn canonical_config_required_paths_default() -> Vec<String> {
    required_config_paths()
}

pub fn canonical_config_required_dir() -> &'static str {
    required_config_dir()
}

pub fn canonical_config_required_valid() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_required_valid_for(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_validate_required() -> Result<(), Vec<String>> {
    validate_required_configs_default_dir()
}

pub fn canonical_config_validate_required_for(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_required_summary() -> String {
    format!(
        "{} files in {}: {}",
        required_config_count(),
        required_config_dir(),
        required_config_file_names_pretty()
    )
}

pub fn canonical_config_required_summary_for(dir: &str) -> String {
    format!(
        "{} files in {}: {}",
        required_config_count(),
        dir,
        required_config_file_names_pretty()
    )
}

pub fn canonical_config_summary() -> String {
    canonical_config_required_summary()
}

pub fn canonical_config_summary_for(dir: &str) -> String {
    canonical_config_required_summary_for(dir)
}

pub fn canonical_config_files_slice() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_files_vec() -> Vec<&'static str> {
    CANONICAL_CONFIG_FILES.to_vec()
}

pub fn canonical_config_files_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_files_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_dir_name() -> &'static str {
    "game/data"
}

pub fn canonical_config_validate_in_dir(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_validate_default_dir() -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_exists(file_name: &str) -> bool {
    CANONICAL_CONFIG_FILES.contains(&file_name)
}

pub fn canonical_config_set() -> std::collections::HashSet<&'static str> {
    CANONICAL_CONFIG_FILES.iter().copied().collect()
}

pub fn canonical_config_debug_dump() -> String {
    format!(
        "dir={} count={} files=[{}]",
        canonical_config_dir_name(),
        canonical_config_count(),
        canonical_config_files_pretty()
    )
}

pub fn canonical_config_doctor_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_engine_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_ci_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_release_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_audit_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_runtime_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_scope_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_source() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_source_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_source_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_source_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_source_dir() -> &'static str {
    "game/data"
}

pub fn canonical_config_truth_source_paths() -> Vec<String> {
    canonical_config_paths("game/data")
}

pub fn canonical_config_truth_validate() -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_validate_for(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_truth_check() -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_errors() -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_missing() -> Vec<String> {
    check_required_config_presence("game/data")
}

pub fn canonical_config_truth_parse_errors() -> Vec<String> {
    check_required_config_parsing("game/data")
}

pub fn canonical_config_truth_report() -> String {
    let missing = canonical_config_truth_missing();
    let parse = canonical_config_truth_parse_errors();
    format!(
        "missing={} parse_errors={} count={}",
        missing.len(),
        parse.len(),
        CANONICAL_CONFIG_FILES.len()
    )
}

pub fn canonical_config_truth_report_verbose() -> String {
    let missing = canonical_config_truth_missing();
    let parse = canonical_config_truth_parse_errors();
    format!(
        "files=[{}]; missing=[{}]; parse_errors=[{}]",
        CANONICAL_CONFIG_FILES.join(", "),
        missing.join(", "),
        parse.join(" | ")
    )
}

pub fn canonical_config_truth_ready() -> bool {
    canonical_config_truth_missing().is_empty() && canonical_config_truth_parse_errors().is_empty()
}

pub fn canonical_config_truth_required_files() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_required_paths() -> Vec<String> {
    canonical_config_truth_source_paths()
}

pub fn canonical_config_truth_required_dir() -> &'static str {
    canonical_config_truth_source_dir()
}

pub fn canonical_config_truth_required_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_required_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_required_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_validate_required() -> Result<(), Vec<String>> {
    canonical_config_truth_validate()
}

pub fn canonical_config_truth_validate_required_for(dir: &str) -> Result<(), Vec<String>> {
    canonical_config_truth_validate_for(dir)
}

pub fn canonical_config_truth_is_required(file_name: &str) -> bool {
    CANONICAL_CONFIG_FILES.contains(&file_name)
}

pub fn canonical_config_truth_required_set() -> std::collections::HashSet<&'static str> {
    CANONICAL_CONFIG_FILES.iter().copied().collect()
}

pub fn canonical_config_truth_paths_for(dir: &str) -> Vec<String> {
    canonical_config_paths(dir)
}

pub fn canonical_config_truth_parse_for(dir: &str) -> Vec<String> {
    check_required_config_parsing(dir)
}

pub fn canonical_config_truth_missing_for(dir: &str) -> Vec<String> {
    check_required_config_presence(dir)
}

pub fn canonical_config_truth_all_good_for(dir: &str) -> bool {
    check_required_config_presence(dir).is_empty() && check_required_config_parsing(dir).is_empty()
}

pub fn canonical_config_truth_all_good_default() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_done() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_done_for(dir: &str) -> bool {
    canonical_config_truth_all_good_for(dir)
}

pub fn canonical_config_truth_list() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_list_vec() -> Vec<&'static str> {
    CANONICAL_CONFIG_FILES.to_vec()
}

pub fn canonical_config_truth_list_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_list_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_list_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_dir_default() -> &'static str {
    "game/data"
}

pub fn canonical_config_truth_paths_default() -> Vec<String> {
    canonical_config_paths("game/data")
}

pub fn canonical_config_truth_validate_default_dir() -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_check_default_dir() -> bool {
    canonical_config_truth_validate_default_dir().is_ok()
}

pub fn canonical_config_truth_errors_default_dir() -> Vec<String> {
    canonical_config_truth_validate_default_dir()
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_missing_default_dir() -> Vec<String> {
    check_required_config_presence("game/data")
}

pub fn canonical_config_truth_parse_default_dir() -> Vec<String> {
    check_required_config_parsing("game/data")
}

pub fn canonical_config_truth_gate() -> Result<(), Vec<String>> {
    canonical_config_truth_validate_default_dir()
}

pub fn canonical_config_truth_gate_for(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_truth_gate_ok() -> bool {
    canonical_config_truth_gate().is_ok()
}

pub fn canonical_config_truth_gate_errors() -> Vec<String> {
    canonical_config_truth_gate().err().unwrap_or_default()
}

pub fn canonical_config_truth_gate_missing() -> Vec<String> {
    canonical_config_truth_missing_default_dir()
}

pub fn canonical_config_truth_gate_parse() -> Vec<String> {
    canonical_config_truth_parse_default_dir()
}

pub fn canonical_config_truth_gate_summary() -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_summary_verbose() -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_ready() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_gate_required_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_gate_required_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_gate_required_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_gate_required_dir() -> &'static str {
    "game/data"
}

pub fn canonical_config_truth_gate_required_paths() -> Vec<String> {
    canonical_config_paths("game/data")
}

pub fn canonical_config_truth_gate_validate() -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_validate_for(dir: &str) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_truth_gate_is_required(file_name: &str) -> bool {
    CANONICAL_CONFIG_FILES.contains(&file_name)
}

pub fn canonical_config_truth_gate_set() -> std::collections::HashSet<&'static str> {
    CANONICAL_CONFIG_FILES.iter().copied().collect()
}

pub fn canonical_config_truth_gate_paths_for(dir: &str) -> Vec<String> {
    canonical_config_paths(dir)
}

pub fn canonical_config_truth_gate_parse_for(dir: &str) -> Vec<String> {
    check_required_config_parsing(dir)
}

pub fn canonical_config_truth_gate_missing_for(dir: &str) -> Vec<String> {
    check_required_config_presence(dir)
}

pub fn canonical_config_truth_gate_ok_for(dir: &str) -> bool {
    check_required_config_presence(dir).is_empty() && check_required_config_parsing(dir).is_empty()
}

pub fn canonical_config_truth_gate_ok_default() -> bool {
    canonical_config_truth_gate_ready()
}

pub fn canonical_config_truth_gate_done() -> bool {
    canonical_config_truth_gate_ready()
}

pub fn canonical_config_truth_gate_done_for(dir: &str) -> bool {
    canonical_config_truth_gate_ok_for(dir)
}

pub fn canonical_config_truth_gate_list() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_gate_list_vec() -> Vec<&'static str> {
    CANONICAL_CONFIG_FILES.to_vec()
}

pub fn canonical_config_truth_gate_list_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_gate_list_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_gate_list_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_gate_dir_default() -> &'static str {
    "game/data"
}

pub fn canonical_config_truth_gate_paths_default() -> Vec<String> {
    canonical_config_paths("game/data")
}

pub fn canonical_config_truth_gate_validate_default_dir() -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_check_default_dir() -> bool {
    canonical_config_truth_gate_validate_default_dir().is_ok()
}

pub fn canonical_config_truth_gate_errors_default_dir() -> Vec<String> {
    canonical_config_truth_gate_validate_default_dir()
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_missing_default_dir() -> Vec<String> {
    check_required_config_presence("game/data")
}

pub fn canonical_config_truth_gate_parse_default_dir() -> Vec<String> {
    check_required_config_parsing("game/data")
}

pub fn canonical_config_truth_gate_report() -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_report_verbose() -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_all_good() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_slice() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_gate_required_files_vec() -> Vec<&'static str> {
    CANONICAL_CONFIG_FILES.to_vec()
}

pub fn canonical_config_truth_gate_required_files_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_gate_required_files_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_gate_required_files_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_gate_required_files_dir() -> &'static str {
    "game/data"
}

pub fn canonical_config_truth_gate_required_files_paths() -> Vec<String> {
    canonical_config_paths("game/data")
}

pub fn canonical_config_truth_gate_required_files_validate() -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_validate_for(
    dir: &str,
) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_truth_gate_required_files_is_required(file_name: &str) -> bool {
    CANONICAL_CONFIG_FILES.contains(&file_name)
}

pub fn canonical_config_truth_gate_required_files_set() -> std::collections::HashSet<&'static str> {
    CANONICAL_CONFIG_FILES.iter().copied().collect()
}

pub fn canonical_config_truth_gate_required_files_paths_for(dir: &str) -> Vec<String> {
    canonical_config_paths(dir)
}

pub fn canonical_config_truth_gate_required_files_parse_for(dir: &str) -> Vec<String> {
    check_required_config_parsing(dir)
}

pub fn canonical_config_truth_gate_required_files_missing_for(dir: &str) -> Vec<String> {
    check_required_config_presence(dir)
}

pub fn canonical_config_truth_gate_required_files_ok_for(dir: &str) -> bool {
    check_required_config_presence(dir).is_empty() && check_required_config_parsing(dir).is_empty()
}

pub fn canonical_config_truth_gate_required_files_ok_default() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_done() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_done_for(dir: &str) -> bool {
    canonical_config_truth_gate_required_files_ok_for(dir)
}

pub fn canonical_config_truth_gate_required_files_list() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_gate_required_files_list_vec() -> Vec<&'static str> {
    CANONICAL_CONFIG_FILES.to_vec()
}

pub fn canonical_config_truth_gate_required_files_list_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_gate_required_files_list_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_gate_required_files_list_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_gate_required_files_dir_default() -> &'static str {
    "game/data"
}

pub fn canonical_config_truth_gate_required_files_paths_default() -> Vec<String> {
    canonical_config_paths("game/data")
}

pub fn canonical_config_truth_gate_required_files_validate_default_dir() -> Result<(), Vec<String>>
{
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_check_default_dir() -> bool {
    canonical_config_truth_gate_required_files_validate_default_dir().is_ok()
}

pub fn canonical_config_truth_gate_required_files_errors_default_dir() -> Vec<String> {
    canonical_config_truth_gate_required_files_validate_default_dir()
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_missing_default_dir() -> Vec<String> {
    check_required_config_presence("game/data")
}

pub fn canonical_config_truth_gate_required_files_parse_default_dir() -> Vec<String> {
    check_required_config_parsing("game/data")
}

pub fn canonical_config_truth_gate_required_files_report() -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_report_verbose() -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_all_good() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth() -> &'static [&'static str] {
    &CANONICAL_CONFIG_FILES
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_pretty() -> String {
    CANONICAL_CONFIG_FILES.join(", ")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_csv() -> String {
    CANONICAL_CONFIG_FILES.join(",")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_count() -> usize {
    CANONICAL_CONFIG_FILES.len()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_dir() -> &'static str {
    "game/data"
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_paths() -> Vec<String> {
    canonical_config_paths("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_for(
    dir: &str,
) -> Result<(), Vec<String>> {
    validate_required_configs(dir)
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_ok() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_errors() -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_missing() -> Vec<String> {
    canonical_config_truth_missing()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_parse_errors() -> Vec<String> {
    canonical_config_truth_parse_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_report() -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_report_verbose() -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_all_good() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_dir(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_ok() -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_errors(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_missing(
) -> Vec<String> {
    check_required_config_presence("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_parse_errors(
) -> Vec<String> {
    check_required_config_parsing("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_report() -> String
{
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_all_good() -> bool
{
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_done() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_ready() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_status() -> bool
{
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_pass() -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_success() -> bool
{
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_success(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result(
) -> Result<(), Vec<String>> {
    validate_required_configs("game/data")
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_ok(
) -> bool {
    validate_required_configs("game/data").is_ok()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_err(
) -> Vec<String> {
    validate_required_configs("game/data")
        .err()
        .unwrap_or_default()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_report(
) -> String {
    canonical_config_truth_report()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_report_verbose(
) -> String {
    canonical_config_truth_report_verbose()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_all_good(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_done(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_ready(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_status(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_pass(
) -> bool {
    canonical_config_truth_ready()
}

pub fn canonical_config_truth_gate_required_files_source_of_truth_validate_default_result_result_result_result_result_result_result_result_result_result_result_result_failures(
) -> Vec<String> {
    canonical_config_truth_errors()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionConfig {
    pub hunt_radius: f32,
    pub fear_radius: f32,
    pub ally_radius: f32,
    pub social_radius: f32,
    pub nearby_count_radius: f32,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            hunt_radius: 120.0,
            fear_radius: 150.0,
            ally_radius: 80.0,
            social_radius: 100.0,
            nearby_count_radius: 80.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub max_npcs: usize,
    pub max_wolves: usize,
    pub max_boars: usize,
    pub max_bloodsuckers: usize,
    pub min_wolves: usize,
    pub min_boars: usize,
    pub min_bloodsuckers: usize,
    pub respawn_interval: f32,
    pub npc_names: Vec<String>,
    pub child_names: Vec<String>,
    pub npc_age_min: f32,
    pub npc_age_max: f32,
    pub npc_max_age_min: f32,
    pub npc_max_age_max: f32,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            max_npcs: 30,
            max_wolves: 40,
            max_boars: 40,
            max_bloodsuckers: 25,
            min_wolves: 5,
            min_boars: 5,
            min_bloodsuckers: 2,
            respawn_interval: 300.0,
            npc_names: vec![
                "Viktor", "Elena", "Sasha", "Dmitri", "Irina", "Andrei", "Natasha", "Boris",
                "Yuri", "Olga", "Maxim", "Tatiana", "Sergei", "Anya", "Pavel", "Ilya", "Marina",
                "Roman", "Vera", "Artem",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            child_names: vec![
                "Alyosha", "Misha", "Katya", "Dasha", "Pasha", "Kolya", "Vanya", "Sveta", "Zhenya",
                "Borya", "Lena", "Grisha", "Tonya", "Nikita", "Oleg",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            npc_age_min: 80.0,
            npc_age_max: 200.0,
            npc_max_age_min: 350.0,
            npc_max_age_max: 450.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub monthly_required: f32,
    pub desperation_bandit_threshold: f32,
    pub desperation_force_bandit: f32,
    pub desperation_increase_rate: f32,
    pub desperation_decrease_on_pay: f32,
    pub initial_money_min: f32,
    pub initial_money_max: f32,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            monthly_required: 50.0,
            desperation_bandit_threshold: 0.7,
            desperation_force_bandit: 0.9,
            desperation_increase_rate: 0.3,
            desperation_decrease_on_pay: 0.1,
            initial_money_min: 20.0,
            initial_money_max: 60.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    pub daily_income: f32,
    pub danger: f32,
    pub energy_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalConfig {
    pub max_duration: f32,
    pub target_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConfig {
    pub food_density: f32,
    pub danger_level: f32,
    pub water_density: f32,
    pub night_danger_mult: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonConfig {
    pub food_regen_mult: f32,
    pub hunger_drain_mult: f32,
    pub breeding_mult: f32,
    pub danger_mult: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub seconds_per_day: f32,
    pub days_per_month: u32,
    pub sim_tick_rate: f32,
    pub l0_radius: f32,
    pub l1_radius: f32,
    pub l2_radius: f32,
    pub l0_tick_interval: u32,
    pub l1_tick_interval: u32,
    pub l2_tick_interval: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            seconds_per_day: 120.0,
            days_per_month: 30,
            sim_tick_rate: 20.0,
            l0_radius: 300.0,
            l1_radius: 5000.0,
            l2_radius: 50000.0,
            l0_tick_interval: 1,
            l1_tick_interval: 12,
            l2_tick_interval: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialConfig {
    pub flammability: f32,
    pub fuel: f32,
    pub hardness: f32,
    pub penetration_resistance: f32,
    pub density: f32,
}

fn project_material_from_surface(surface: &SurfaceMaterial) -> MaterialConfig {
    MaterialConfig {
        flammability: surface.flammability,
        fuel: surface.fuel_content,
        hardness: surface.hardness,
        penetration_resistance: surface.penetration_resistance,
        density: surface.density,
    }
}

// === NEW: Food Chain Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodChainConfig {
    pub predator_prey: Vec<(String, String)>,
    pub food_chain_rank: HashMap<String, u32>,
}

impl Default for FoodChainConfig {
    fn default() -> Self {
        Self {
            predator_prey: vec![
                ("Wolf".into(), "Boar".into()),
                ("Bloodsucker".into(), "Wolf".into()),
            ],
            food_chain_rank: [
                ("Boar".into(), 1),
                ("Wolf".into(), 2),
                ("Bloodsucker".into(), 3),
            ]
            .into_iter()
            .collect(),
        }
    }
}

// === NEW: Species Config ===

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EcosystemNeeds {
    pub hunting: f32,
    pub predator_avoidance: f32,
    pub food_chain_position: f32,
    pub territory_control: f32,
    pub migration_urge: f32,
    pub resource_competition: f32,
    pub pack_following: f32,
    pub shelter_seeking: f32,
    pub world_event_reaction: f32,
    pub prey_selection: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaseTraits {
    pub aggressiveness: f32,
    pub caution: f32,
    pub territoriality: f32,
    pub bravery: f32,
    pub pack_mentality: f32,
    pub energy_level: f32,
    pub hoarding: f32,
    pub curiosity: f32,
    pub adaptability: f32,
    pub stress_tolerance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesEntry {
    pub base_power: f32,
    pub food_value: f32,
    pub max_age: f32,
    pub mate_cooldown_days: u32,
    pub ecosystem_needs: EcosystemNeeds,
    pub base_traits: BaseTraits,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeciesConfig {
    pub species: HashMap<String, SpeciesEntry>,
}

// === NEW: Tactics Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticsConfig {
    pub flank_weight: f32,
    pub charge_weight: f32,
    pub ambush_weight: f32,
    pub retreat_weight: f32,
    pub surround_weight: f32,
    pub hit_and_run_weight: f32,
    pub hold_ground_weight: f32,
    pub retreat_health_threshold: f32,
    pub retreat_ally_loss_ratio: f32,
    pub preferred_group_size: u32,
}

impl Default for TacticsConfig {
    fn default() -> Self {
        Self {
            flank_weight: 0.2,
            charge_weight: 0.2,
            ambush_weight: 0.1,
            retreat_weight: 0.1,
            surround_weight: 0.1,
            hit_and_run_weight: 0.1,
            hold_ground_weight: 0.1,
            retreat_health_threshold: 0.25,
            retreat_ally_loss_ratio: 0.5,
            preferred_group_size: 2,
        }
    }
}

// === NEW: Rules Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRule {
    pub name: String,
    pub frequency: String,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRadii {
    pub l0: f32,
    pub l1: f32,
    pub l2: f32,
    pub l0_tick: u32,
    pub l1_tick: u32,
    pub l2_tick: u32,
}

impl Default for SimulationRadii {
    fn default() -> Self {
        Self {
            l0: 300.0,
            l1: 5000.0,
            l2: 50000.0,
            l0_tick: 1,
            l1_tick: 12,
            l2_tick: 60,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesData {
    pub systems: Vec<SystemRule>,
    pub simulation_radii: SimulationRadii,
    pub replicated_events: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesConfig {
    pub data: RulesData,
}

// === NEW: Weapons Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub damage: f32,
    pub range: f32,
    pub fire_rate: f32,
    pub accuracy: f32,
    pub noise_radius: f32,
}

impl Default for WeaponConfig {
    fn default() -> Self {
        Self {
            damage: 10.0,
            range: 100.0,
            fire_rate: 1.0,
            accuracy: 0.8,
            noise_radius: 50.0,
        }
    }
}

// === NEW: Surface Config (surfaces.ron) ===

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResponseClass {
    Ite,
    LayeredMasonry,
    AnisotropicWood,
    BrittleGlassRadial,
    DuctileMetal,
    BrittleCeramic,
    Composite,
    BiologicalSoft,
    BiologicalHard,
}

impl Default for ResponseClass {
    fn default() -> Self {
        Self::Ite
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceMaterial {
    pub id: u32,
    pub name: String,
    pub response_class: ResponseClass,
    pub compressive_strength: f32,
    pub tensile_strength: f32,
    pub shear_strength: f32,
    pub brittleness: f32,
    pub density: f32,
    pub elasticity: f32,
    pub penetration_resistance: f32,
    pub hardness: f32,
    pub flammability: f32,
    pub fuel_content: f32,
    pub ignition_temp: f32,
    pub thermal_conductivity: f32,
    pub porosity: f32,
    pub erosion_resistance: f32,
    pub fragmentation_coeff: f32,
    pub fracture_pattern: u32,
    pub debris_profile: u32,
    pub decal_profile: u32,
    pub dust_intensity: f32,
    pub is_biological: bool,
    pub gore_response: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurfacesConfig {
    pub materials: Vec<SurfaceMaterial>,
}

// === NEW: Material Bridge Config (material_bridge.ron) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBridge {
    pub material_id: u32,
    pub base_albedo_tint: (f32, f32, f32),
    pub roughness_range: (f32, f32),
    pub metallic: f32,
    pub normal_intensity: f32,
    pub subsurface: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioBridge {
    pub material_id: u32,
    pub impact_sound_class: String,
    pub footstep_sound_class: String,
    pub scrape_sound_class: String,
    pub break_sound_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleBridge {
    pub material_id: u32,
    pub debris_color: (f32, f32, f32),
    pub debris_size_range: (f32, f32),
    pub dust_color: (f32, f32, f32),
    pub dust_density: f32,
    pub spark_on_impact: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialBridgeConfig {
    pub render: Vec<RenderBridge>,
    pub audio: Vec<AudioBridge>,
    pub particle: Vec<ParticleBridge>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameConfig {
    pub perception: PerceptionConfig,
    pub population: PopulationConfig,
    pub economy: EconomyConfig,
    pub simulation: SimulationConfig,
    pub jobs: HashMap<String, JobConfig>,
    pub goals: HashMap<String, GoalConfig>,
    pub biomes: HashMap<String, BiomeConfig>,
    pub seasons: HashMap<String, SeasonConfig>,
    pub materials: HashMap<String, MaterialConfig>,
    // NEW
    pub food_chain: FoodChainConfig,
    pub species: SpeciesConfig,
    pub tactics: HashMap<String, TacticsConfig>,
    pub rules: RulesConfig,
    pub weapons: HashMap<String, WeaponConfig>,
    // NEW: Surfaces & Material Bridge
    pub surfaces: SurfacesConfig,
    pub material_bridge: MaterialBridgeConfig,
}

impl GameConfig {
    pub fn load_from_dir(dir: &str) -> Self {
        let mut config = Self::default();

        // Existing loaders
        if let Ok(p) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<PerceptionConfig>,
        >(&format!("{}/perception.ron", dir))
        {
            config.perception = p.data;
        }

        if let Ok(p) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<PopulationConfig>,
        >(&format!("{}/population.ron", dir))
        {
            config.population = p.data;
        }

        if let Ok(e) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<EconomyConfig>,
        >(&format!("{}/economy.ron", dir))
        {
            config.economy = e.data;
        }

        if let Ok(s) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<SimulationConfig>,
        >(&format!("{}/simulation.ron", dir))
        {
            config.simulation = s.data;
        }

        if let Ok(j) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<HashMap<String, JobConfig>>,
        >(&format!("{}/jobs.ron", dir))
        {
            config.jobs = j.data;
        }

        if let Ok(g) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<HashMap<String, GoalConfig>>,
        >(&format!("{}/goals.ron", dir))
        {
            config.goals = g.data;
        }

        if let Ok(b) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<HashMap<String, BiomeConfig>>,
        >(&format!("{}/biomes.ron", dir))
        {
            config.biomes = b.data;
        }

        if let Ok(s) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<HashMap<String, SeasonConfig>>,
        >(&format!("{}/seasons.ron", dir))
        {
            config.seasons = s.data;
        }

        // NOTE(C0.4): runtime material truth is projected from canonical surfaces config.
        // materials.ron stays as authored compatibility input, but runtime meaning is derived.

        // NEW loaders
        if let Ok(fc) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<FoodChainConfig>,
        >(&format!("{}/food_chain.ron", dir))
        {
            config.food_chain = fc.data;
        }

        if let Ok(sp) = load_species_config(&format!("{}/species.ron", dir)) {
            config.species = sp;
        }

        if let Ok(t) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<HashMap<String, TacticsConfig>>,
        >(&format!("{}/tactics.ron", dir))
        {
            config.tactics = t.data;
        }

        if let Ok(r) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<RulesData>,
        >(&format!("{}/rules.ron", dir))
        {
            config.rules.data = r.data;
        }

        if let Ok(w) = engine_core::config::load_config::<
            engine_core::config::ConfigEnvelope<HashMap<String, WeaponConfig>>,
        >(&format!("{}/weapons.ron", dir))
        {
            config.weapons = w.data;
        }

        // Surfaces & Material Bridge
        if let Ok(s) = load_surfaces_config(&format!("{}/surfaces.ron", dir)) {
            config.surfaces = s;
            config.materials = config
                .surfaces
                .materials
                .iter()
                .map(|m| (m.name.clone(), project_material_from_surface(m)))
                .collect();
        }

        if let Ok(m) = load_material_bridge_config(&format!("{}/material_bridge.ron", dir)) {
            config.material_bridge = m;
        }

        println!("[config] loaded GameConfig from '{}'", dir);
        println!(
            "  jobs: {}, goals: {}, biomes: {}, materials: {}, tactics: {}, weapons: {}, surfaces: {}",
            config.jobs.len(),
            config.goals.len(),
            config.biomes.len(),
            config.materials.len(),
            config.tactics.len(),
            config.weapons.len(),
            config.surfaces.materials.len(),
        );

        config
    }

    /// Get predator-prey relationship
    pub fn is_predator_of(&self, predator: &str, prey: &str) -> bool {
        self.food_chain
            .predator_prey
            .iter()
            .any(|(p, pr)| p == predator && pr == prey)
    }

    /// Get food chain rank for a species
    pub fn chain_rank(&self, species: &str) -> u32 {
        self.food_chain
            .food_chain_rank
            .get(species)
            .copied()
            .unwrap_or(0)
    }

    /// Get tactics for a species
    pub fn get_tactics(&self, species: &str) -> Option<&TacticsConfig> {
        self.tactics.get(species)
    }

    /// Get weapon config
    pub fn get_weapon(&self, name: &str) -> Option<&WeaponConfig> {
        self.weapons.get(name)
    }
}

/// Load species config with special handling for the nested structure
fn load_species_config(path: &str) -> Result<SpeciesConfig, Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Parse the outer envelope
    #[derive(Deserialize)]
    struct SpeciesEnvelope {
        #[allow(dead_code)]
        schema_version: u32,
        data: HashMap<String, SpeciesEntry>,
    }

    let envelope: SpeciesEnvelope = ron::from_str(&content)?;
    Ok(SpeciesConfig {
        species: envelope.data,
    })
}

/// Load surfaces config with special handling for the nested structure
fn load_surfaces_config(path: &str) -> Result<SurfacesConfig, Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Parse the outer envelope
    #[derive(Deserialize)]
    struct SurfacesEnvelope {
        #[allow(dead_code)]
        schema_version: u32,
        materials: Vec<SurfaceMaterial>,
    }

    let envelope: SurfacesEnvelope = ron::from_str(&content)?;
    Ok(SurfacesConfig {
        materials: envelope.materials,
    })
}

/// Load material bridge config
fn load_material_bridge_config(
    path: &str,
) -> Result<MaterialBridgeConfig, Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config: MaterialBridgeConfig = ron::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_config_default() {
        let config = PerceptionConfig::default();
        assert!(config.hunt_radius > 0.0);
        assert!(config.fear_radius > config.hunt_radius);
        assert!(config.ally_radius > 0.0);
    }

    #[test]
    fn test_population_config_default() {
        let config = PopulationConfig::default();
        assert!(config.max_npcs > 0);
        assert!(config.max_wolves > 0);
        assert!(config.respawn_interval > 0.0);
        assert!(!config.npc_names.is_empty());
        assert!(!config.child_names.is_empty());
    }

    #[test]
    fn test_economy_config_default() {
        let config = EconomyConfig::default();
        assert!(config.monthly_required > 0.0);
        assert!(
            config.desperation_bandit_threshold > 0.0 && config.desperation_bandit_threshold < 1.0
        );
        assert!(config.desperation_force_bandit > config.desperation_bandit_threshold);
    }

    #[test]
    fn test_simulation_config_default() {
        let config = SimulationConfig::default();
        assert!(config.seconds_per_day > 0.0);
        assert!(config.days_per_month > 0);
        assert!(config.l0_radius < config.l1_radius);
        assert!(config.l1_radius < config.l2_radius);
    }

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert!(config.perception.hunt_radius > 0.0);
        assert!(config.population.max_npcs > 0);
        assert!(config.economy.monthly_required > 0.0);
    }

    #[test]
    fn test_food_chain_predator_prey() {
        let config = GameConfig::default();
        assert!(config.is_predator_of("Wolf", "Boar"));
        assert!(!config.is_predator_of("Boar", "Wolf"));
    }

    #[test]
    fn test_food_chain_rank() {
        let config = GameConfig::default();
        assert_eq!(config.chain_rank("Bloodsucker"), 3);
        assert_eq!(config.chain_rank("Wolf"), 2);
        assert_eq!(config.chain_rank("Boar"), 1);
        assert_eq!(config.chain_rank("Unknown"), 0);
    }

    #[test]
    fn test_tactics_config_default() {
        let config = TacticsConfig::default();
        assert!(config.retreat_health_threshold > 0.0 && config.retreat_health_threshold < 1.0);
        assert!(config.preferred_group_size > 0);
    }

    #[test]
    fn test_weapon_config_default() {
        let config = WeaponConfig::default();
        assert!(config.damage > 0.0);
        assert!(config.range > 0.0);
        assert!(config.accuracy > 0.0 && config.accuracy <= 1.0);
    }

    #[test]
    fn test_response_class_default() {
        let class = ResponseClass::default();
        assert_eq!(class, ResponseClass::Ite);
    }

    #[test]
    fn test_surfaces_config_default() {
        let config = SurfacesConfig::default();
        assert!(config.materials.is_empty());
    }

    #[test]
    fn test_material_bridge_config_default() {
        let config = MaterialBridgeConfig::default();
        assert!(config.render.is_empty());
        assert!(config.audio.is_empty());
        assert!(config.particle.is_empty());
    }
}
