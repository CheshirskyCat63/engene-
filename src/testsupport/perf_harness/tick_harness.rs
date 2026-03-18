use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use super::metrics::TickMetrics;

#[derive(Clone, Debug)]
pub struct TickPressureConfig {
    pub ticks: usize,
    pub producer_threads: usize,
    pub events_per_thread_per_tick: usize,
    pub tick_budget_ns: u64,
}

pub fn run_tick_pressure(config: &TickPressureConfig) -> TickMetrics {
    let (tx, rx) = mpsc::channel::<u64>();
    let backlog = Arc::new(AtomicUsize::new(0));
    let max_backlog = Arc::new(AtomicUsize::new(0));

    let mut produced_events = 0u64;
    let mut drained_events = 0u64;
    let mut deadline_miss_ticks = 0usize;
    let mut max_tick_ns = 0u128;
    let mut total_tick_ns = 0u128;
    let mut max_sustainable_ops_per_tick = 0u64;

    for tick in 0..config.ticks {
        let mut joins = Vec::with_capacity(config.producer_threads);
        for thread_idx in 0..config.producer_threads {
            let tx = tx.clone();
            let backlog = Arc::clone(&backlog);
            let max_backlog = Arc::clone(&max_backlog);
            let count = config.events_per_thread_per_tick;
            joins.push(thread::spawn(move || {
                for i in 0..count {
                    let event_id = ((tick * 1_000_000) + (thread_idx * count) + i) as u64;
                    let current = backlog.fetch_add(1, Ordering::Relaxed) + 1;
                    let _ = max_backlog.fetch_max(current, Ordering::Relaxed);
                    if tx.send(event_id).is_err() {
                        break;
                    }
                }
            }));
        }

        for handle in joins {
            let _ = handle.join();
        }

        produced_events += (config.producer_threads * config.events_per_thread_per_tick) as u64;

        let tick_start = Instant::now();
        let mut drained_this_tick = 0u64;
        loop {
            if tick_start.elapsed().as_nanos() >= config.tick_budget_ns as u128 {
                break;
            }
            match rx.try_recv() {
                Ok(_event) => {
                    backlog.fetch_sub(1, Ordering::Relaxed);
                    drained_this_tick += 1;
                }
                Err(_) => break,
            }
        }

        let tick_ns = tick_start.elapsed().as_nanos();
        total_tick_ns += tick_ns;
        max_tick_ns = max_tick_ns.max(tick_ns);
        if tick_ns > config.tick_budget_ns as u128 {
            deadline_miss_ticks += 1;
        }
        max_sustainable_ops_per_tick = max_sustainable_ops_per_tick.max(drained_this_tick);

        drained_events += drained_this_tick;
    }

    drop(tx);
    while rx.try_recv().is_ok() {
        backlog.fetch_sub(1, Ordering::Relaxed);
        drained_events += 1;
    }

    let ticks_f = config.ticks.max(1) as f64;
    let mean_tick_ns = total_tick_ns as f64 / ticks_f;
    let drain_efficiency = if produced_events > 0 {
        drained_events as f64 / produced_events as f64
    } else {
        1.0
    };

    TickMetrics {
        ticks: config.ticks,
        produced_events,
        drained_events,
        max_backlog: max_backlog.load(Ordering::Relaxed),
        deadline_miss_ticks,
        mean_tick_ns,
        max_tick_ns,
        max_sustainable_ops_per_tick,
        drain_efficiency,
    }
}
