//! Phase 12.1: Performance Lockdown
//! Measures performance baseline for headless simulation.

use std::collections::HashMap;
use std::time::Instant;

use crate::bootstrap::GameRuntimeAssembly;

const SIM_DT: f32 = 1.0 / 20.0;

/// Performance baseline metrics.
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub avg_frame_time_ms: f64,
    pub p99_frame_time_ms: f64,
    pub worst_spike_ms: f64,
    pub destruction_spike_ms: f64,
    pub ai_crowd_spike_ms: f64,
    pub replay_overhead_ms: f64,
    pub debug_overhead_ms: f64,
    pub streaming_io_stalls: u32,
    pub surface_state_memory_mb: f64,
    pub active_body_count: usize,
    pub nav_dirty_count: usize,
    pub event_drop_count: u64,
    pub queue_pressure: HashMap<String, f64>,
}

/// Run headless for N ticks and measure timing and resource usage.
pub fn measure_baseline(ticks: u64) -> PerformanceBaseline {
    let grid = crate::world::world::WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);

    let mut frame_times: Vec<f64> = Vec::with_capacity(ticks as usize);
    let _destruction_times: Vec<f64> = Vec::new();
    let _ai_times: Vec<f64> = Vec::new();

    for _ in 0..ticks {
        let t0 = Instant::now();
        engine.tick(SIM_DT);
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        frame_times.push(elapsed_ms);
    }

    frame_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = frame_times.len();
    let avg_frame_time_ms = if n > 0 {
        frame_times.iter().sum::<f64>() / n as f64
    } else {
        0.0
    };
    let p99_idx = (n as f64 * 0.99) as usize;
    let p99_frame_time_ms = frame_times
        .get(p99_idx.min(n.saturating_sub(1)))
        .copied()
        .unwrap_or(0.0);
    let worst_spike_ms = frame_times.last().copied().unwrap_or(0.0);

    let active_body_count = engine.ecs.alive.len();
    let nav_dirty_count = engine
        .resources
        .get::<crate::navigation::dynamic_nav_update::NavDirtyTracker>()
        .map(|t| t.pending_count())
        .unwrap_or(0);
    let event_drop_count = engine.events.total_dropped();

    let mut queue_pressure = HashMap::new();
    queue_pressure.insert(
        "event_drop_ratio".to_string(),
        if frame_times.is_empty() {
            0.0
        } else {
            event_drop_count as f64 / (ticks * 20) as f64
        },
    );

    PerformanceBaseline {
        avg_frame_time_ms,
        p99_frame_time_ms,
        worst_spike_ms,
        destruction_spike_ms: 0.0,
        ai_crowd_spike_ms: 0.0,
        replay_overhead_ms: 0.0,
        debug_overhead_ms: 0.0,
        streaming_io_stalls: 0,
        surface_state_memory_mb: 0.0,
        active_body_count,
        nav_dirty_count,
        event_drop_count,
        queue_pressure,
    }
}
