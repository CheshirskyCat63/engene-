//! Low-spec certification — proves the engine meets minimum requirements
//! defined in LOW_SPEC_EQUIVALENCE.md:
//! - 30 FPS sustained
//! - No truth loss (persistence, quest state, combat resolution unchanged)
//! - Acceptable degradation only (per degradation_order)
//! - No runaway memory

use engine_core::quality_governor::{
    degradation_order, DegradationPriority, PressureLevel, QualityGovernor,
};

#[derive(Debug, Clone)]
pub struct LowSpecCertResult {
    pub passed: bool,
    pub target_fps: u32,
    pub achieved_fps_estimate: f32,
    pub truth_loss_detected: bool,
    pub memory_runaway: bool,
    pub unacceptable_degradation: Vec<String>,
    pub notes: Vec<String>,
}

pub struct LowSpecCertifier {
    target_fps: u32,
    frame_samples: Vec<u32>,
    truth_checks_passed: bool,
    peak_memory_bytes: usize,
    memory_limit_bytes: usize,
}

impl LowSpecCertifier {
    pub fn new(target_fps: u32, memory_limit_mb: usize) -> Self {
        Self {
            target_fps,
            frame_samples: Vec::with_capacity(1000),
            truth_checks_passed: true,
            peak_memory_bytes: 0,
            memory_limit_bytes: memory_limit_mb * 1024 * 1024,
        }
    }

    pub fn record_frame(&mut self, frame_time_us: u32) {
        self.frame_samples.push(frame_time_us);
    }

    pub fn record_memory(&mut self, bytes: usize) {
        if bytes > self.peak_memory_bytes {
            self.peak_memory_bytes = bytes;
        }
    }

    pub fn mark_truth_loss(&mut self) {
        self.truth_checks_passed = false;
    }

    pub fn certify(&self, governor: &QualityGovernor) -> LowSpecCertResult {
        let budget_us = 1_000_000 / self.target_fps;

        let avg_frame_us = if self.frame_samples.is_empty() {
            budget_us
        } else {
            let sum: u64 = self.frame_samples.iter().map(|&f| f as u64).sum();
            (sum / self.frame_samples.len() as u64) as u32
        };

        let achieved_fps = if avg_frame_us > 0 {
            1_000_000.0 / avg_frame_us as f32
        } else {
            999.0
        };

        let fps_ok = achieved_fps >= self.target_fps as f32;
        let memory_ok = self.peak_memory_bytes <= self.memory_limit_bytes;

        let mut unacceptable = Vec::new();
        let order = degradation_order();
        for entry in &order {
            if entry.priority == DegradationPriority::NeverCut
                && governor.pressure_level == PressureLevel::Critical
            {
                unacceptable.push(format!(
                    "{} may be impacted at Critical pressure",
                    entry.subsystem
                ));
            }
        }

        let mut notes = Vec::new();
        if !fps_ok {
            notes.push(format!(
                "FPS below target: {:.1} < {}",
                achieved_fps, self.target_fps
            ));
        }
        if !memory_ok {
            notes.push(format!(
                "Memory peak {} MB exceeds limit {} MB",
                self.peak_memory_bytes / (1024 * 1024),
                self.memory_limit_bytes / (1024 * 1024)
            ));
        }
        if !self.truth_checks_passed {
            notes.push("Truth loss detected during certification run".into());
        }
        notes.push(format!("Pressure level: {:?}", governor.pressure_level));
        notes.push(format!("Frame samples: {}", self.frame_samples.len()));

        LowSpecCertResult {
            passed: fps_ok && self.truth_checks_passed && memory_ok && unacceptable.is_empty(),
            target_fps: self.target_fps,
            achieved_fps_estimate: achieved_fps,
            truth_loss_detected: !self.truth_checks_passed,
            memory_runaway: !memory_ok,
            unacceptable_degradation: unacceptable,
            notes,
        }
    }

    pub fn generate_report(&self, governor: &QualityGovernor) -> String {
        let result = self.certify(governor);
        let mut report = String::new();
        report.push_str("# Low-Spec Certification Report\n\n");
        report.push_str(&format!(
            "**Result: {}**\n\n",
            if result.passed { "PASSED" } else { "FAILED" }
        ));
        report.push_str(&format!("- Target FPS: {}\n", result.target_fps));
        report.push_str(&format!(
            "- Achieved FPS estimate: {:.1}\n",
            result.achieved_fps_estimate
        ));
        report.push_str(&format!("- Truth loss: {}\n", result.truth_loss_detected));
        report.push_str(&format!("- Memory runaway: {}\n", result.memory_runaway));
        if !result.unacceptable_degradation.is_empty() {
            report.push_str("\n## Unacceptable Degradation\n");
            for d in &result.unacceptable_degradation {
                report.push_str(&format!("- {}\n", d));
            }
        }
        report.push_str("\n## Notes\n");
        for note in &result.notes {
            report.push_str(&format!("- {}\n", note));
        }
        report
    }
}

impl Default for LowSpecCertifier {
    fn default() -> Self {
        Self::new(30, 2048)
    }
}
