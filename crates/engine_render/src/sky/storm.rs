//! Phase 5.1-5.3: Storm cell system with full lifecycle and cumulonimbus.

use crate::world::fields::StormCellWind;
use glam::Vec3;

/// Storm lifecycle stage with realistic timing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StormStage {
    Cumulus,
    ToweringCumulus,
    Cumulonimbus,
    Dissipating,
}

impl StormStage {
    pub fn as_u8(&self) -> u8 {
        match self {
            StormStage::Cumulus => 0,
            StormStage::ToweringCumulus => 1,
            StormStage::Cumulonimbus => 2,
            StormStage::Dissipating => 3,
        }
    }
}

/// Storm cell with full lifecycle: cumulus -> towering cumulus -> cumulonimbus -> dissipating.
#[derive(Clone, Debug)]
pub struct StormCell {
    pub position: Vec3,
    pub radius: f32,
    pub intensity: f32,
    pub humidity: f32,
    pub temperature: f32,
    pub pressure: f32,
    pub updraft: f32,
    pub wind_vector: Vec3,
    pub lifecycle_stage: StormStage,
    pub stage_timer: f32,
    pub cloud_base: f32,
    pub cloud_top: f32,
    pub seed: u64,
    pub anvil_spread: f32,
}

/// Cumulus: 5-15 min, cloud top grows to 3000m
const CUMULUS_DURATION_MIN: f32 = 300.0; // 5 min
const CUMULUS_DURATION_MAX: f32 = 900.0; // 15 min
const CUMULUS_TOP_TARGET: f32 = 3000.0;

/// Towering cumulus: 15-30 min, cloud top grows to 8000m
const TOWERING_DURATION_MIN: f32 = 900.0; // 15 min
const TOWERING_DURATION_MAX: f32 = 1800.0; // 30 min
const TOWERING_TOP_TARGET: f32 = 8000.0;

/// Cumulonimbus: 30-60 min, cloud top 10000-12000m, anvil spread
const CUMULONIMBUS_TOP_MIN: f32 = 10000.0;
const CUMULONIMBUS_TOP_MAX: f32 = 12000.0;
const CUMULONIMBUS_DURATION_MIN: f32 = 1800.0; // 30 min
const CUMULONIMBUS_DURATION_MAX: f32 = 3600.0; // 60 min

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn seeded_f32(seed: &mut u64, min: f32, max: f32) -> f32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let t = ((*seed >> 32) as u32 as f32) / (u32::MAX as f32);
    lerp(min, max, t)
}

impl StormCell {
    pub fn new(position: Vec3, seed: u64) -> Self {
        let mut s = seed;
        Self {
            position,
            radius: seeded_f32(&mut s, 2000.0, 6000.0),
            intensity: 0.2,
            humidity: 0.8,
            temperature: 15.0,
            pressure: 1010.0,
            updraft: 3.0,
            wind_vector: Vec3::new(
                seeded_f32(&mut s, -5.0, 5.0),
                0.0,
                seeded_f32(&mut s, -5.0, 5.0),
            ),
            lifecycle_stage: StormStage::Cumulus,
            stage_timer: 0.0,
            cloud_base: 1500.0,
            cloud_top: 1800.0,
            seed,
            anvil_spread: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.position += self.wind_vector * dt;
        self.stage_timer += dt;

        match self.lifecycle_stage {
            StormStage::Cumulus => {
                let duration = lerp(
                    CUMULUS_DURATION_MIN,
                    CUMULUS_DURATION_MAX,
                    (self.seed % 100) as f32 / 100.0,
                );
                self.radius = (self.radius + dt * 50.0).min(8000.0);
                self.cloud_top = (self.cloud_top + dt * 2.0).min(CUMULUS_TOP_TARGET);
                self.intensity = (self.intensity + dt * 0.001).min(0.5);
                self.updraft = (self.updraft + dt * 0.01).min(8.0);

                if self.stage_timer >= duration || (self.updraft > 5.0 && self.cloud_top >= 2800.0)
                {
                    self.lifecycle_stage = StormStage::ToweringCumulus;
                    self.stage_timer = 0.0;
                }
            }
            StormStage::ToweringCumulus => {
                let duration = lerp(
                    TOWERING_DURATION_MIN,
                    TOWERING_DURATION_MAX,
                    (self.seed % 100) as f32 / 100.0,
                );
                self.radius = (self.radius + dt * 100.0).min(25000.0);
                self.cloud_top = (self.cloud_top + dt * 8.0).min(TOWERING_TOP_TARGET);
                self.intensity = (self.intensity + dt * 0.002).min(0.8);
                self.updraft = (self.updraft + dt * 0.02).min(15.0);

                if self.stage_timer >= duration || self.cloud_top >= 7500.0 {
                    self.lifecycle_stage = StormStage::Cumulonimbus;
                    self.stage_timer = 0.0;
                    let target_top = lerp(
                        CUMULONIMBUS_TOP_MIN,
                        CUMULONIMBUS_TOP_MAX,
                        ((self.seed >> 8) % 100) as f32 / 100.0,
                    );
                    self.cloud_top = self.cloud_top.min(target_top);
                }
            }
            StormStage::Cumulonimbus => {
                let duration = lerp(
                    CUMULONIMBUS_DURATION_MIN,
                    CUMULONIMBUS_DURATION_MAX,
                    (self.seed % 100) as f32 / 100.0,
                );
                let target_top = lerp(
                    CUMULONIMBUS_TOP_MIN,
                    CUMULONIMBUS_TOP_MAX,
                    ((self.seed >> 8) % 100) as f32 / 100.0,
                );
                self.radius = (self.radius + dt * 30.0).min(50000.0);
                self.cloud_top = (self.cloud_top + dt * 3.0).min(target_top);
                self.cloud_base = 1000.0;
                self.anvil_spread = (self.anvil_spread + dt * 0.0005).min(1.0);
                self.intensity = (self.intensity + dt * 0.0005).min(1.0);

                if self.stage_timer >= duration || self.humidity < 0.35 {
                    self.lifecycle_stage = StormStage::Dissipating;
                    self.stage_timer = 0.0;
                }
            }
            StormStage::Dissipating => {
                self.intensity -= dt * 0.002;
                self.updraft = (self.updraft - dt * 0.05).max(0.0);
                self.humidity = (self.humidity - dt * 0.005).max(0.0);
                self.cloud_top = (self.cloud_top - dt * 20.0).max(2000.0);
                self.anvil_spread = (self.anvil_spread - dt * 0.0002).max(0.0);
            }
        }
    }

    /// Anvil factor at given height (0-1+). Cumulonimbus anvil spreads above ~9000m.
    pub fn anvil_factor(&self, height: f32) -> f32 {
        if self.lifecycle_stage != StormStage::Cumulonimbus {
            return 0.0;
        }
        if height < 9000.0 {
            return 0.0;
        }
        let t = ((height - 9000.0) / (self.cloud_top - 9000.0).max(1.0)).clamp(0.0, 1.0);
        t * self.anvil_spread
    }

    /// Influence of this storm at world position (0-1).
    pub fn influence_at(&self, world_pos: Vec3) -> f32 {
        let d = (world_pos - self.position).length();
        (1.0 - (d / self.radius)).max(0.0)
    }

    /// Precipitation intensity 0-1 based on stage and humidity.
    pub fn precipitation_intensity(&self) -> f32 {
        match self.lifecycle_stage {
            StormStage::Cumulus => 0.0,
            StormStage::ToweringCumulus => self.intensity * self.humidity * 0.3,
            StormStage::Cumulonimbus => self.intensity * self.humidity,
            StormStage::Dissipating => self.intensity * self.humidity * 0.5,
        }
    }

    /// Lightning probability per second (0-1). Highest in mature cumulonimbus.
    pub fn lightning_probability(&self) -> f32 {
        match self.lifecycle_stage {
            StormStage::Cumulus => 0.0,
            StormStage::ToweringCumulus => 0.001,
            StormStage::Cumulonimbus => 0.01 * self.intensity,
            StormStage::Dissipating => 0.002,
        }
    }
}

impl From<&StormCell> for StormCellWind {
    fn from(c: &StormCell) -> Self {
        Self {
            position: c.position,
            radius: c.radius,
            wind_vector: c.wind_vector,
        }
    }
}
