use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use engine_core::events::EventBus;

use super::metrics::{throughput, to_latency_stats, ScalingPoint, ThroughputMetrics};

#[derive(Clone, Debug)]
pub struct KernelThroughputConfig {
    pub events_per_thread: usize,
    pub thread_levels: Vec<usize>,
    pub channel_capacity: usize,
}

#[derive(Clone, Debug)]
pub struct KernelThroughputReport {
    pub points: Vec<ScalingPoint>,
}

#[derive(Clone)]
struct KernelEvent {
    produced_at: Instant,
    id: u64,
}

pub fn run_kernel_throughput(config: &KernelThroughputConfig) -> KernelThroughputReport {
    let mut points = Vec::with_capacity(config.thread_levels.len());
    let mut single_thread_ops_per_sec = 0.0;

    for &threads in &config.thread_levels {
        let metrics = run_single_level(threads, config.events_per_thread, config.channel_capacity);
        if threads == 1 {
            single_thread_ops_per_sec = metrics.ops_per_sec;
        }
        let speedup = if single_thread_ops_per_sec > 0.0 {
            metrics.ops_per_sec / single_thread_ops_per_sec
        } else {
            0.0
        };
        points.push(ScalingPoint {
            threads,
            metrics,
            speedup_vs_single_thread: speedup,
        });
    }

    KernelThroughputReport { points }
}

fn run_single_level(
    threads: usize,
    events_per_thread: usize,
    channel_capacity: usize,
) -> ThroughputMetrics {
    let (tx, rx) = mpsc::channel::<KernelEvent>();
    let backlog = Arc::new(AtomicUsize::new(0));
    let backlog_peak = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();
    let mut joins = Vec::with_capacity(threads);
    for thread_idx in 0..threads {
        let tx = tx.clone();
        let backlog = Arc::clone(&backlog);
        let backlog_peak = Arc::clone(&backlog_peak);
        joins.push(thread::spawn(move || {
            for i in 0..events_per_thread {
                let current = backlog.fetch_add(1, Ordering::Relaxed) + 1;
                let _ = backlog_peak.fetch_max(current, Ordering::Relaxed);
                let event = KernelEvent {
                    produced_at: Instant::now(),
                    id: ((thread_idx * events_per_thread) + i) as u64,
                };
                if tx.send(event).is_err() {
                    break;
                }
            }
        }));
    }
    drop(tx);

    let mut bus = EventBus::with_capacity(channel_capacity);
    bus.set_channel_capacity::<u64>(channel_capacity);

    let expected = threads * events_per_thread;
    let mut consumed = 0u64;
    let mut latencies = Vec::with_capacity(expected);

    while consumed < expected as u64 {
        match rx.recv() {
            Ok(event) => {
                backlog.fetch_sub(1, Ordering::Relaxed);
                latencies.push(event.produced_at.elapsed().as_nanos() as u64);
                bus.emit(event.id);
                consumed += 1;
            }
            Err(_) => break,
        }
    }

    for handle in joins {
        let _ = handle.join();
    }

    let elapsed = start.elapsed();
    let (ops_per_sec, ns_per_op) = throughput(consumed, elapsed);

    ThroughputMetrics {
        total_ops: consumed,
        elapsed,
        ops_per_sec,
        ns_per_op,
        dropped_events: bus.total_dropped(),
        backlog_peak: backlog_peak.load(Ordering::Relaxed),
        latency: to_latency_stats(&latencies),
    }
}
