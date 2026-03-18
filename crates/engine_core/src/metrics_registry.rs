//! Unified Observability Layer (Phase C.7)
//!
//! Provides per-subsystem metrics collection, aggregation, and export.
//! Thread-safe, lock-free counters where possible.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Metric types
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Monotonically increasing counter
    Counter(u64),
    /// Point-in-time gauge value
    Gauge(f64),
    /// Histogram bucket (for distribution tracking)
    Histogram {
        sum: f64,
        count: u64,
        min: f64,
        max: f64,
    },
}

impl Default for MetricValue {
    fn default() -> Self {
        MetricValue::Counter(0)
    }
}

/// A single metric entry
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub subsystem: String,
    pub value: MetricValue,
    pub description: String,
}

/// Thread-safe counter
#[derive(Debug)]
pub struct AtomicCounter {
    value: AtomicU64,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn increment(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe gauge
#[derive(Debug)]
pub struct AtomicGauge {
    value: RwLock<f64>,
}

impl AtomicGauge {
    pub fn new(initial: f64) -> Self {
        Self {
            value: RwLock::new(initial),
        }
    }

    pub fn set(&self, value: f64) {
        if let Ok(mut v) = self.value.write() {
            *v = value;
        }
    }

    pub fn get(&self) -> f64 {
        self.value.read().map(|v| *v).unwrap_or(0.0)
    }
}

impl Default for AtomicGauge {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Per-subsystem metric group
#[derive(Debug, Default)]
pub struct SubsystemMetrics {
    pub counters: HashMap<String, AtomicCounter>,
    pub gauges: HashMap<String, AtomicGauge>,
}

/// Global metrics registry
pub struct MetricsRegistry {
    subsystems: RwLock<HashMap<String, SubsystemMetrics>>,
    descriptions: RwLock<HashMap<String, String>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            subsystems: RwLock::new(HashMap::new()),
            descriptions: RwLock::new(HashMap::new()),
        }
    }

    /// Register a counter metric
    pub fn register_counter(&self, subsystem: &str, name: &str, description: &str) {
        if let Ok(mut subs) = self.subsystems.write() {
            let sub = subs.entry(subsystem.to_string()).or_default();
            sub.counters
                .entry(name.to_string())
                .or_insert_with(AtomicCounter::new);
        }
        if let Ok(mut descs) = self.descriptions.write() {
            descs.insert(format!("{}.{}", subsystem, name), description.to_string());
        }
    }

    /// Register a gauge metric
    pub fn register_gauge(&self, subsystem: &str, name: &str, description: &str) {
        if let Ok(mut subs) = self.subsystems.write() {
            let sub = subs.entry(subsystem.to_string()).or_default();
            sub.gauges
                .entry(name.to_string())
                .or_insert_with(AtomicGauge::default);
        }
        if let Ok(mut descs) = self.descriptions.write() {
            descs.insert(format!("{}.{}", subsystem, name), description.to_string());
        }
    }

    /// Increment a counter
    pub fn increment(&self, subsystem: &str, name: &str, delta: u64) {
        if let Ok(subs) = self.subsystems.read() {
            if let Some(sub) = subs.get(subsystem) {
                if let Some(counter) = sub.counters.get(name) {
                    counter.increment(delta);
                }
            }
        }
    }

    /// Set a gauge value
    pub fn set_gauge(&self, subsystem: &str, name: &str, value: f64) {
        if let Ok(subs) = self.subsystems.read() {
            if let Some(sub) = subs.get(subsystem) {
                if let Some(gauge) = sub.gauges.get(name) {
                    gauge.set(value);
                }
            }
        }
    }

    /// Get counter value
    pub fn get_counter(&self, subsystem: &str, name: &str) -> u64 {
        if let Ok(subs) = self.subsystems.read() {
            if let Some(sub) = subs.get(subsystem) {
                if let Some(counter) = sub.counters.get(name) {
                    return counter.get();
                }
            }
        }
        0
    }

    /// Get gauge value
    pub fn get_gauge(&self, subsystem: &str, name: &str) -> f64 {
        if let Ok(subs) = self.subsystems.read() {
            if let Some(sub) = subs.get(subsystem) {
                if let Some(gauge) = sub.gauges.get(name) {
                    return gauge.get();
                }
            }
        }
        0.0
    }

    /// Get all metrics as a flat list
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        let mut result = Vec::new();

        if let Ok(subs) = self.subsystems.read() {
            for (subsystem_name, sub) in subs.iter() {
                for (name, counter) in &sub.counters {
                    result.push(Metric {
                        name: name.clone(),
                        subsystem: subsystem_name.clone(),
                        value: MetricValue::Counter(counter.get()),
                        description: self.get_description(subsystem_name, name),
                    });
                }
                for (name, gauge) in sub.gauges.iter() {
                    result.push(Metric {
                        name: name.clone(),
                        subsystem: subsystem_name.clone(),
                        value: MetricValue::Gauge(gauge.get()),
                        description: self.get_description(subsystem_name, name),
                    });
                }
            }
        }

        result
    }

    fn get_description(&self, subsystem: &str, name: &str) -> String {
        let key = format!("{}.{}", subsystem, name);
        if let Ok(descs) = self.descriptions.read() {
            descs.get(&key).cloned().unwrap_or_default()
        } else {
            String::new()
        }
    }

    /// Reset all counters (for testing)
    pub fn reset_all_counters(&self) {
        if let Ok(subs) = self.subsystems.read() {
            for sub in subs.values() {
                for counter in sub.counters.values() {
                    counter.reset();
                }
            }
        }
    }

    /// Export to CSV format
    pub fn export_csv(&self) -> String {
        let mut csv = String::from("subsystem,name,value,type,description\n");

        for metric in self.get_all_metrics() {
            let (value_str, type_str) = match metric.value {
                MetricValue::Counter(v) => (v.to_string(), "counter"),
                MetricValue::Gauge(v) => (v.to_string(), "gauge"),
                MetricValue::Histogram { sum, .. } => (sum.to_string(), "histogram"),
            };

            csv.push_str(&format!(
                "{},{},{},{},\"{}\"\n",
                metric.subsystem, metric.name, value_str, type_str, metric.description
            ));
        }

        csv
    }

    /// Export to JSON format
    pub fn export_json(&self) -> String {
        let metrics = self.get_all_metrics();
        let mut json = String::from("{\n  \"metrics\": [\n");

        for (i, metric) in metrics.iter().enumerate() {
            let value_json = match &metric.value {
                MetricValue::Counter(v) => format!("{{\"type\": \"counter\", \"value\": {}}}", v),
                MetricValue::Gauge(v) => format!("{{\"type\": \"gauge\", \"value\": {}}}", v),
                MetricValue::Histogram {
                    sum,
                    count,
                    min,
                    max,
                } => {
                    format!(
                        "{{\"type\": \"histogram\", \"sum\": {}, \"count\": {}, \"min\": {}, \"max\": {}}}",
                        sum, count, min, max
                    )
                }
            };

            json.push_str(&format!(
                "    {{\"subsystem\": \"{}\", \"name\": \"{}\", \"value\": {}, \"description\": \"{}\"}}",
                metric.subsystem,
                metric.name,
                value_json,
                metric.description
            ));

            if i < metrics.len() - 1 {
                json.push_str(",\n");
            } else {
                json.push_str("\n");
            }
        }

        json.push_str("  ]\n}\n");
        json
    }

    /// Write metrics to file
    pub fn export_to_file(&self, path: &str, format: MetricsExportFormat) -> std::io::Result<()> {
        let content = match format {
            MetricsExportFormat::Csv => self.export_csv(),
            MetricsExportFormat::Json => self.export_json(),
        };
        std::fs::write(path, content)
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MetricsExportFormat {
    Csv,
    Json,
}

/// Create default metrics registry with all required subsystem metrics
pub fn create_default_metrics_registry() -> MetricsRegistry {
    let registry = MetricsRegistry::new();

    // Engine metrics
    registry.register_counter(
        "engine",
        "frame_time_us",
        "Total frame time in microseconds",
    );
    registry.register_counter("engine", "tick_time_us", "Fixed tick time in microseconds");
    registry.register_counter(
        "engine",
        "systems_serialized_count",
        "Number of systems that had to run serially",
    );
    registry.register_counter(
        "engine",
        "parallel_groups_count",
        "Number of parallel groups detected",
    );
    registry.register_gauge("engine", "fps", "Current frames per second");

    // AI metrics
    registry.register_counter(
        "ai",
        "entities_ticked",
        "Number of AI entities ticked this frame",
    );
    registry.register_counter("ai", "budget_misses", "Times AI budget was exceeded");
    registry.register_counter("ai", "replan_count", "Number of AI plan recalculations");
    registry.register_counter("ai", "stuck_entities", "Entities stuck without valid goals");
    registry.register_gauge("ai", "budget_usage_ratio", "Ratio of AI budget used");

    // Physics metrics
    registry.register_counter(
        "physics",
        "rapier_step_us",
        "Time spent in Rapier physics step",
    );
    registry.register_counter(
        "physics",
        "fire_cells_active",
        "Active fire simulation cells",
    );
    registry.register_counter(
        "physics",
        "water_cells_active",
        "Active water simulation cells",
    );
    registry.register_counter("physics", "cloth_particles", "Active cloth particles");
    registry.register_counter(
        "physics",
        "impacts_resolved",
        "Damage impacts resolved this frame",
    );

    // Graphics metrics
    registry.register_counter("graphics", "draw_calls", "Number of draw calls this frame");
    registry.register_counter("graphics", "triangles", "Total triangles rendered");
    registry.register_counter("graphics", "gpu_time_us", "GPU frame time if available");
    registry.register_counter("graphics", "shader_reloads", "Number of shader hot reloads");
    registry.register_gauge(
        "graphics",
        "degradation_level",
        "Current quality degradation level",
    );

    // World metrics
    registry.register_counter("world", "chunks_loaded", "Total chunks loaded");
    registry.register_counter("world", "chunks_unloaded", "Total chunks unloaded");
    registry.register_counter(
        "world",
        "entities_streamed_in",
        "Entities streamed in this frame",
    );
    registry.register_counter(
        "world",
        "entities_streamed_out",
        "Entities streamed out this frame",
    );

    // Memory metrics
    registry.register_counter("memory", "total_bytes", "Total memory allocated");
    registry.register_counter("memory", "cache_hits", "Cache hit count");
    registry.register_counter("memory", "cache_misses", "Cache miss count");
    registry.register_counter("memory", "leak_suspects", "Potential memory leaks detected");

    // Event metrics
    registry.register_counter("events", "events_emitted", "Total events emitted");
    registry.register_counter("events", "events_dropped", "Events dropped due to overflow");
    registry.register_gauge("events", "bus_pressure_ratio", "Event bus fill ratio");

    // Streaming metrics
    registry.register_counter(
        "streaming",
        "read_bytes_frame",
        "Bytes read from disk this frame",
    );
    registry.register_counter(
        "streaming",
        "decompress_bytes_frame",
        "Bytes decompressed this frame",
    );
    registry.register_counter(
        "streaming",
        "upload_bytes_frame",
        "Bytes uploaded to GPU this frame",
    );

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_increment() {
        let registry = MetricsRegistry::new();
        registry.register_counter("test", "counter1", "Test counter");

        registry.increment("test", "counter1", 5);
        assert_eq!(registry.get_counter("test", "counter1"), 5);

        registry.increment("test", "counter1", 3);
        assert_eq!(registry.get_counter("test", "counter1"), 8);
    }

    #[test]
    fn test_gauge_set() {
        let registry = MetricsRegistry::new();
        registry.register_gauge("test", "gauge1", "Test gauge");

        registry.set_gauge("test", "gauge1", 42.5);
        assert!((registry.get_gauge("test", "gauge1") - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_export_csv() {
        let registry = MetricsRegistry::new();
        registry.register_counter("engine", "frames", "Frame count");
        registry.increment("engine", "frames", 100);

        let csv = registry.export_csv();
        assert!(csv.contains("engine,frames,100,counter"));
    }

    #[test]
    fn test_export_json() {
        let registry = MetricsRegistry::new();
        registry.register_gauge("engine", "fps", "FPS");
        registry.set_gauge("engine", "fps", 60.0);

        let json = registry.export_json();
        assert!(json.contains("\"fps\""));
        assert!(json.contains("60"));
    }

    #[test]
    fn test_default_registry() {
        let registry = create_default_metrics_registry();

        // Check a few key metrics exist
        assert!(registry.get_counter("engine", "frame_time_us") == 0);
        assert!(registry.get_counter("ai", "entities_ticked") == 0);
        assert!(registry.get_counter("physics", "impacts_resolved") == 0);
    }

    #[test]
    fn test_reset_counters() {
        let registry = MetricsRegistry::new();
        registry.register_counter("test", "counter", "Test");
        registry.increment("test", "counter", 100);

        registry.reset_all_counters();
        assert_eq!(registry.get_counter("test", "counter"), 0);
    }
}
