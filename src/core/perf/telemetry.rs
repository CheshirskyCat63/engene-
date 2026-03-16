use std::collections::HashMap;

pub struct SystemTiming {
    pub system_name: String,
    pub last_us: u32,
    pub avg_us: f32,
    pub max_us: u32,
    pub sample_count: u64,
}

pub struct Telemetry {
    system_timings: HashMap<String, SystemTiming>,
    frame_time_us: u32,
    frame_count: u64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            system_timings: HashMap::new(),
            frame_time_us: 0,
            frame_count: 0,
        }
    }

    pub fn record_system(&mut self, name: &str, elapsed_us: u32) {
        let timing = self.system_timings.entry(name.to_string()).or_insert(SystemTiming {
            system_name: name.to_string(),
            last_us: 0,
            avg_us: 0.0,
            max_us: 0,
            sample_count: 0,
        });
        timing.last_us = elapsed_us;
        timing.max_us = timing.max_us.max(elapsed_us);
        timing.sample_count += 1;
        let alpha = 0.05;
        timing.avg_us = timing.avg_us * (1.0 - alpha) + elapsed_us as f32 * alpha;
    }

    pub fn record_frame(&mut self, frame_time_us: u32) {
        self.frame_time_us = frame_time_us;
        self.frame_count += 1;
    }

    pub fn system_timings(&self) -> &HashMap<String, SystemTiming> {
        &self.system_timings
    }

    pub fn frame_time_us(&self) -> u32 { self.frame_time_us }
    pub fn frame_count(&self) -> u64 { self.frame_count }
}

impl Default for Telemetry {
    fn default() -> Self { Self::new() }
}
