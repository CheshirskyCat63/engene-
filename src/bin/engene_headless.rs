//! ENGENE Headless Binary — pure simulation, no renderer, no window.
//! Used for CI testing, balance testing, and long-run simulation validation.
//!
//! Usage: cargo run --bin engene_headless -- --months 12

use engene::app::runtime_assembly::RuntimeAssembly;
use engene::core::build_manifest::BuildManifest;
use engene::core::crash_telemetry;
use engene::economy::resource_flow;
use engene::tools::doctor;
use engene::world::heightmap::Heightmap;
use engene::world::world::WorldGrid;

struct SoakMetrics {
    initial_entity_count: usize,
    peak_entity_count: usize,
    min_entity_count: usize,
    peak_npc_count: usize,
    peak_monster_count: usize,
    tick_count: u64,
    economy_snapshots: Vec<(u64, f64, f64, u32)>,
}

impl SoakMetrics {
    fn new(initial: usize) -> Self {
        Self {
            initial_entity_count: initial,
            peak_entity_count: initial,
            min_entity_count: initial,
            peak_npc_count: 0,
            peak_monster_count: 0,
            tick_count: 0,
            economy_snapshots: Vec::new(),
        }
    }

    fn record(&mut self, tick: u64, entities: usize, npcs: usize, monsters: usize, money: f64, desperation: f64, bandits: u32) {
        self.peak_entity_count = self.peak_entity_count.max(entities);
        self.min_entity_count = self.min_entity_count.min(entities);
        self.peak_npc_count = self.peak_npc_count.max(npcs);
        self.peak_monster_count = self.peak_monster_count.max(monsters);
        self.tick_count = tick;
        self.economy_snapshots.push((tick, money, desperation, bandits));
    }

    fn entity_stability(&self) -> f64 {
        if self.initial_entity_count == 0 { return 1.0; }
        let drift = (self.peak_entity_count as f64 - self.min_entity_count as f64)
            / self.initial_entity_count as f64;
        (1.0 - drift).max(0.0)
    }

    fn print_report(&self) {
        println!("\n=== SOAK TEST REPORT ===");
        println!("  Total ticks: {}", self.tick_count);
        println!("  Entity count: initial={}, peak={}, min={}",
            self.initial_entity_count, self.peak_entity_count, self.min_entity_count);
        println!("  Entity stability: {:.1}%", self.entity_stability() * 100.0);
        println!("  Peak NPCs: {}, Peak Monsters: {}", self.peak_npc_count, self.peak_monster_count);
        if let Some(last) = self.economy_snapshots.last() {
            println!("  Final economy: ${:.0} total money, {:.2} avg desperation, {} bandits",
                last.1, last.2, last.3);
        }
    }
}

fn main() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    crash_telemetry::install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let manifest = BuildManifest::current();
    println!("=== ENGENE HEADLESS ===");
    manifest.print_full();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let months: u32 = std::env::args()
        .skip_while(|a| a != "--months")
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let emit_truth: bool = std::env::args().any(|a| a == "--truth");

    println!("[headless] simulating {} months...\n", months);

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let _heightmap = Heightmap::generate(&biomes);

    let mut engine = RuntimeAssembly::headless(&biomes);

    let doctor_report = doctor::run_doctor(&engine, doctor::DoctorMode::Strict);
    println!(
        "[doctor] startup: {} errors, {} warnings",
        doctor_report.error_count(),
        doctor_report.warning_count()
    );

    let target_month = engine.time.month + months;
    let sim_dt = 1.0 / 20.0_f32;
    let mut tick_count: u64 = 0;
    let mut soak = SoakMetrics::new(engine.ecs.alive.len());

    while engine.time.month < target_month {
        engine.tick(sim_dt);
        tick_count += 1;

        if tick_count % 1000 == 0 {
            let snap = resource_flow::snapshot(&engine.ecs);
            let npcs = engine.ecs.npcs().len();
            let monsters = engine.ecs.monsters().len();
            let entities = engine.ecs.alive.len();

            soak.record(
                tick_count, entities, npcs, monsters,
                snap.total_npc_money as f64,
                snap.average_desperation as f64,
                snap.bandit_count,
            );

            println!(
                "[tick {}] month {} day {} | npcs:{} monsters:{} total:{} | ${:.0} desp:{:.2}",
                tick_count,
                engine.time.month,
                engine.time.day,
                npcs,
                monsters,
                entities,
                snap.total_npc_money,
                snap.average_desperation,
            );
        }
    }

    println!("\n=== SIMULATION COMPLETE ===");
    println!("  Total ticks: {}", tick_count);
    println!("  Final month: {}", engine.time.month);
    println!("  Entities alive: {}", engine.ecs.alive.len());
    println!("  NPCs: {}", engine.ecs.npcs().len());
    println!("  Monsters: {}", engine.ecs.monsters().len());

    let snap = resource_flow::snapshot(&engine.ecs);
    println!("  Total NPC money: ${:.0}", snap.total_npc_money);
    println!("  Average desperation: {:.2}", snap.average_desperation);

    soak.print_report();

    if emit_truth {
        let truth_json = doctor::generate_runtime_truth_json(&engine);
        let truth_path = "RUNTIME_TRUTH.json";
        match std::fs::write(truth_path, &truth_json) {
            Ok(()) => println!("\n[truth] wrote {}", truth_path),
            Err(e) => eprintln!("[truth] failed to write {}: {}", truth_path, e),
        }
    }

    let final_doctor = doctor::run_doctor(&engine, doctor::DoctorMode::Strict);
    println!(
        "\n[doctor] final: {} errors, {} warnings",
        final_doctor.error_count(),
        final_doctor.warning_count()
    );
    final_doctor.print();

    if soak.entity_stability() < 0.5 {
        eprintln!("[SOAK FAIL] Entity stability below 50%");
        std::process::exit(2);
    }

    if final_doctor.error_count() > 0 {
        std::process::exit(1);
    }
}
