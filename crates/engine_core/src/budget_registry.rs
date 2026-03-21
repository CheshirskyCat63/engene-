pub struct BudgetEntry {
    pub phase_name: &'static str,
    pub owner_system: &'static str,
    pub cpu_budget_us: u32,
    pub gpu_budget_us: u32,
    pub memory_budget_bytes: usize,
    pub profiler_category: &'static str,
    pub overrun_count: u32,
    pub last_measured_us: u32,
}

pub struct BudgetRegistry {
    entries: Vec<BudgetEntry>,
}

impl BudgetRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, entry: BudgetEntry) {
        self.entries.push(entry);
    }

    pub fn record_measurement(&mut self, phase_name: &str, measured_us: u32) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.phase_name == phase_name) {
            entry.last_measured_us = measured_us;
            if measured_us > entry.cpu_budget_us {
                entry.overrun_count += 1;
            }
        }
    }

    pub fn total_budget_us(&self) -> u32 {
        self.entries.iter().map(|e| e.cpu_budget_us).sum()
    }

    pub fn total_overruns(&self) -> u32 {
        self.entries.iter().map(|e| e.overrun_count).sum()
    }

    pub fn entries(&self) -> &[BudgetEntry] {
        &self.entries
    }
}

pub fn create_default_registry() -> BudgetRegistry {
    let mut reg = BudgetRegistry::new();

    reg.register(BudgetEntry {
        phase_name: "damage_pipeline",
        owner_system: "DamageOrchestrator",
        cpu_budget_us: 500,
        gpu_budget_us: 0,
        memory_budget_bytes: 8 * 1024 * 1024,
        profiler_category: "damage",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "body_damage",
        owner_system: "BodyStateStore",
        cpu_budget_us: 200,
        gpu_budget_us: 0,
        memory_budget_bytes: 4 * 1024 * 1024,
        profiler_category: "damage",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "structural_load",
        owner_system: "StructuralLoadSystem",
        cpu_budget_us: 300,
        gpu_budget_us: 0,
        memory_budget_bytes: 2 * 1024 * 1024,
        profiler_category: "damage",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "terrain_deformation",
        owner_system: "TerrainDeformationSystem",
        cpu_budget_us: 200,
        gpu_budget_us: 100,
        memory_budget_bytes: 16 * 1024 * 1024,
        profiler_category: "terrain",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "dynamic_nav",
        owner_system: "NavDirtyTracker",
        cpu_budget_us: 300,
        gpu_budget_us: 0,
        memory_budget_bytes: 2 * 1024 * 1024,
        profiler_category: "navigation",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "chain_reactions",
        owner_system: "ChainReactionQueue",
        cpu_budget_us: 100,
        gpu_budget_us: 0,
        memory_budget_bytes: 1 * 1024 * 1024,
        profiler_category: "damage",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "persistent_stress",
        owner_system: "PersistentStressSystem",
        cpu_budget_us: 100,
        gpu_budget_us: 0,
        memory_budget_bytes: 1 * 1024 * 1024,
        profiler_category: "damage",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "surface_state",
        owner_system: "SurfaceStateStore",
        cpu_budget_us: 200,
        gpu_budget_us: 100,
        memory_budget_bytes: 32 * 1024 * 1024,
        profiler_category: "surface",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "micro_motion",
        owner_system: "MicroMotionSystem",
        cpu_budget_us: 100,
        gpu_budget_us: 50,
        memory_budget_bytes: 1 * 1024 * 1024,
        profiler_category: "animation",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.register(BudgetEntry {
        phase_name: "camera_intelligence",
        owner_system: "CameraSimulation",
        cpu_budget_us: 50,
        gpu_budget_us: 200,
        memory_budget_bytes: 512 * 1024,
        profiler_category: "postprocess",
        overrun_count: 0,
        last_measured_us: 0,
    });

    reg
}
