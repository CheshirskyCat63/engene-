use engine_core::runtime_config::QualityTier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressureLevel {
    Normal,
    Moderate,
    High,
    Critical,
}

pub struct QualityGovernor {
    pub frame_budget_us: u32,
    pub pressure_level: PressureLevel,
    pub smoothed_frame_time_us: u32,
    pub hysteresis_frames: u8,
    consecutive_overruns: u8,
    consecutive_underruns: u8,
}

impl QualityGovernor {
    pub fn new(target_fps: u32) -> Self {
        let frame_budget_us = 1_000_000 / target_fps;
        Self {
            frame_budget_us,
            pressure_level: PressureLevel::Normal,
            smoothed_frame_time_us: frame_budget_us / 2,
            hysteresis_frames: 8,
            consecutive_overruns: 0,
            consecutive_underruns: 0,
        }
    }

    pub fn update(&mut self, actual_frame_time_us: u32) {
        let alpha = 0.1;
        self.smoothed_frame_time_us = (self.smoothed_frame_time_us as f32 * (1.0 - alpha)
            + actual_frame_time_us as f32 * alpha) as u32;

        if self.smoothed_frame_time_us > self.frame_budget_us {
            self.consecutive_overruns += 1;
            self.consecutive_underruns = 0;
        } else {
            self.consecutive_underruns += 1;
            self.consecutive_overruns = 0;
        }

        if self.consecutive_overruns >= self.hysteresis_frames {
            self.pressure_level = match self.pressure_level {
                PressureLevel::Normal => PressureLevel::Moderate,
                PressureLevel::Moderate => PressureLevel::High,
                PressureLevel::High | PressureLevel::Critical => PressureLevel::Critical,
            };
            self.consecutive_overruns = 0;
        }

        if self.consecutive_underruns >= self.hysteresis_frames * 2 {
            self.pressure_level = match self.pressure_level {
                PressureLevel::Critical => PressureLevel::High,
                PressureLevel::High => PressureLevel::Moderate,
                PressureLevel::Moderate | PressureLevel::Normal => PressureLevel::Normal,
            };
            self.consecutive_underruns = 0;
        }
    }

    pub fn max_dirty_surface_uploads(&self) -> usize {
        match self.pressure_level {
            PressureLevel::Normal => 16,
            PressureLevel::Moderate => 8,
            PressureLevel::High => 4,
            PressureLevel::Critical => 0,
        }
    }

    pub fn max_micro_motion_oscillators(&self) -> usize {
        match self.pressure_level {
            PressureLevel::Normal => 256,
            PressureLevel::Moderate => 128,
            PressureLevel::High => 64,
            PressureLevel::Critical => 0,
        }
    }

    pub fn debris_density_factor(&self) -> f32 {
        match self.pressure_level {
            PressureLevel::Normal => 1.0,
            PressureLevel::Moderate => 0.5,
            PressureLevel::High => 0.25,
            PressureLevel::Critical => 0.0,
        }
    }

    pub fn histogram_frequency(&self) -> u32 {
        match self.pressure_level {
            PressureLevel::Normal => 1,
            PressureLevel::Moderate => 2,
            PressureLevel::High => 4,
            PressureLevel::Critical => 8,
        }
    }

    pub fn max_chain_reaction_depth(&self) -> u32 {
        match self.pressure_level {
            PressureLevel::Normal => 4,
            PressureLevel::Moderate => 2,
            PressureLevel::High => 1,
            PressureLevel::Critical => 0,
        }
    }

    pub fn max_terrain_rebuilds_per_frame(&self) -> usize {
        match self.pressure_level {
            PressureLevel::Normal => 2,
            PressureLevel::Moderate => 1,
            PressureLevel::High | PressureLevel::Critical => 0,
        }
    }

    pub fn max_nav_dirty_cells(&self) -> usize {
        match self.pressure_level {
            PressureLevel::Normal => 64,
            PressureLevel::Moderate => 32,
            PressureLevel::High => 16,
            PressureLevel::Critical => 8,
        }
    }
}

// ── Degradation Order ────────────────────────────────────────

/// Priority at which a subsystem gets degraded.
/// Lower number = cut first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DegradationPriority {
    CutFirst = 0,
    CutSecond = 1,
    CutThird = 2,
    NeverCut = 3,
}

/// Describes what happens to a subsystem at each quality tier
#[derive(Clone, Debug)]
pub struct DegradationEntry {
    pub subsystem: &'static str,
    pub priority: DegradationPriority,
    pub at_low: &'static str,
    pub at_medium: &'static str,
    pub at_high: &'static str,
}

pub fn degradation_order() -> Vec<DegradationEntry> {
    vec![
        // CUT FIRST — cosmetic, non-essential
        DegradationEntry {
            subsystem: "Debug UI / Overlays",
            priority: DegradationPriority::CutFirst,
            at_low: "disabled",
            at_medium: "reduced refresh rate",
            at_high: "full",
        },
        DegradationEntry {
            subsystem: "SSAO",
            priority: DegradationPriority::CutFirst,
            at_low: "disabled",
            at_medium: "half-res",
            at_high: "full",
        },
        DegradationEntry {
            subsystem: "Volumetric Lighting",
            priority: DegradationPriority::CutFirst,
            at_low: "disabled",
            at_medium: "disabled",
            at_high: "full",
        },
        DegradationEntry {
            subsystem: "Detailed Decals",
            priority: DegradationPriority::CutFirst,
            at_low: "disabled",
            at_medium: "reduced count",
            at_high: "full",
        },
        DegradationEntry {
            subsystem: "Particle Count",
            priority: DegradationPriority::CutFirst,
            at_low: "2000 max",
            at_medium: "5000 max",
            at_high: "10000 max",
        },
        // CUT SECOND — quality reduction but still functional
        DegradationEntry {
            subsystem: "Far Animation",
            priority: DegradationPriority::CutSecond,
            at_low: "disabled (T-pose beyond 50m)",
            at_medium: "reduced bones beyond 100m",
            at_high: "full skeletal",
        },
        DegradationEntry {
            subsystem: "Social AI Frequency",
            priority: DegradationPriority::CutSecond,
            at_low: "every 10th tick",
            at_medium: "every 5th tick",
            at_high: "every tick",
        },
        DegradationEntry {
            subsystem: "Shadow Cascades",
            priority: DegradationPriority::CutSecond,
            at_low: "2 cascades, 512px",
            at_medium: "3 cascades, 1024px",
            at_high: "4 cascades, 2048px",
        },
        DegradationEntry {
            subsystem: "AI Think Budget",
            priority: DegradationPriority::CutSecond,
            at_low: "2ms per frame",
            at_medium: "4ms per frame",
            at_high: "6ms per frame",
        },
        // CUT THIRD — significant impact but tolerable
        DegradationEntry {
            subsystem: "Streaming Radius",
            priority: DegradationPriority::CutThird,
            at_low: "2000m",
            at_medium: "3000m",
            at_high: "4000m+",
        },
        DegradationEntry {
            subsystem: "Visible NPC Cap",
            priority: DegradationPriority::CutThird,
            at_low: "50",
            at_medium: "100",
            at_high: "200+",
        },
        // NEVER CUT — correctness-critical
        DegradationEntry {
            subsystem: "Collision Detection",
            priority: DegradationPriority::NeverCut,
            at_low: "always on",
            at_medium: "always on",
            at_high: "always on",
        },
        DegradationEntry {
            subsystem: "Persistent Identity",
            priority: DegradationPriority::NeverCut,
            at_low: "always on",
            at_medium: "always on",
            at_high: "always on",
        },
        DegradationEntry {
            subsystem: "Save Correctness",
            priority: DegradationPriority::NeverCut,
            at_low: "always on",
            at_medium: "always on",
            at_high: "always on",
        },
        DegradationEntry {
            subsystem: "Navigation",
            priority: DegradationPriority::NeverCut,
            at_low: "always on",
            at_medium: "always on",
            at_high: "always on",
        },
    ]
}

/// Given a quality tier, return list of systems that should be disabled
pub fn systems_to_disable(tier: QualityTier) -> Vec<&'static str> {
    let order = degradation_order();
    let mut disabled = Vec::new();
    for entry in &order {
        match tier {
            QualityTier::Low => {
                if entry.at_low == "disabled" {
                    disabled.push(entry.subsystem);
                }
            }
            QualityTier::Medium => {
                if entry.at_medium == "disabled" {
                    disabled.push(entry.subsystem);
                }
            }
            _ => {}
        }
    }
    disabled
}

pub fn degradation_report() -> String {
    let order = degradation_order();
    let mut report = String::new();
    report.push_str("=== Degradation Order ===\n");
    for entry in &order {
        report.push_str(&format!(
            "[{:?}] {} — low: {} | med: {} | high: {}\n",
            entry.priority, entry.subsystem, entry.at_low, entry.at_medium, entry.at_high
        ));
    }
    report
}
