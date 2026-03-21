//! Scheduler - Time-based tick scheduling for systems.
//! Phase 3: Extracted from world_tick.rs for proper separation.
//!
//! The scheduler provides fixed-interval tick logic for systems that need
//! to run at specific time intervals rather than every frame.

/// Scheduler for fixed-interval system execution.
///
/// Accumulates delta time and returns due ticks when interval threshold is reached.
pub struct Scheduler {
    accumulated: f32,
    interval: f32,
    due_ticks: u32,
}

impl Scheduler {
    /// Creates a new scheduler with specified interval in seconds.
    pub fn new(interval_seconds: f32) -> Self {
        Self {
            accumulated: 0.0,
            interval: interval_seconds,
            due_ticks: 0,
        }
    }

    /// Returns interval in seconds.
    pub fn interval(&self) -> f32 {
        self.interval
    }

    /// Returns current accumulated time.
    pub fn accumulated(&self) -> f32 {
        self.accumulated
    }

    /// Returns number of due ticks.
    pub fn due_ticks(&self) -> u32 {
        self.due_ticks
    }

    /// Accumulates delta time. Returns due ticks when interval threshold is reached.
    ///
    /// When due ticks are returned, accumulator is reduced by one interval,
    /// allowing for consistent tick timing.
    pub fn accumulate(&mut self, delta: f32) -> u32 {
        self.accumulated += delta;
        let mut due = 0;
        while self.accumulated >= self.interval {
            self.accumulated -= self.interval;
            due += 1;
            self.due_ticks += 1;
        }
        due
    }

    /// Resets accumulator to zero.
    pub fn reset(&mut self) {
        self.accumulated = 0.0;
        self.due_ticks = 0;
    }

    /// Sets a new interval.
    pub fn set_interval(&mut self, interval_seconds: f32) {
        self.interval = interval_seconds;
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_accumulates() {
        let mut scheduler = Scheduler::new(1.0);

        // First tick should not trigger
        assert_eq!(scheduler.accumulate(0.5), 0);
        assert_eq!(scheduler.accumulated(), 0.5);

        // Second tick should trigger
        assert_eq!(scheduler.accumulate(0.6), 1);
        assert_eq!(scheduler.due_ticks(), 1);
    }

    #[test]
    fn test_scheduler_multiple_accumulates() {
        let mut scheduler = Scheduler::new(1.0);

        // Should trigger twice
        assert!(scheduler.accumulate(2.5));
        assert!(scheduler.accumulate(0.5)); // 0.0 + 0.5 = 0.5, not enough
        assert!(scheduler.accumulate(0.6)); // 0.5 + 0.6 = 1.1 >= 1.0, triggers
    }

    #[test]
    fn test_scheduler_reset() {
        let mut scheduler = Scheduler::new(1.0);
        scheduler.accumulate(0.9);
        scheduler.reset();

        assert_eq!(scheduler.accumulated(), 0.0);
        assert!(!scheduler.accumulate(0.5));
    }

    #[test]
    fn test_scheduler_set_interval() {
        let mut scheduler = Scheduler::new(1.0);
        scheduler.set_interval(2.0);

        assert_eq!(scheduler.interval(), 2.0);
        assert!(!scheduler.accumulate(1.5));
        assert!(scheduler.accumulate(0.6));
    }

    #[test]
    fn test_scheduler_default() {
        let scheduler = Scheduler::default();
        assert_eq!(scheduler.interval(), 1.0);
        assert_eq!(scheduler.accumulated(), 0.0);
    }
}
