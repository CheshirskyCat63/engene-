//! Phase 5.4: Central weather controller managing storms, transitions, wind, coverage.

use glam::Vec3;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::storm::{StormCell, StormStage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeatherState {
    Clear,
    Cloudy,
    Storm,
    Rain,
    Clearing,
}

pub struct WeatherController {
    pub state: WeatherState,
    pub target_state: WeatherState,
    pub transition_progress: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub cloud_coverage: f32,
    pub storm_cells: Vec<StormCell>,
    pub master_seed: u64,
    pub time_accumulator: f32,
    pub rng: StdRng,
}

impl WeatherController {
    pub fn new(seed: u64) -> Self {
        Self {
            state: WeatherState::Clear,
            target_state: WeatherState::Clear,
            transition_progress: 1.0,
            temperature: 20.0,
            humidity: 0.5,
            pressure: 1013.25,
            cloud_coverage: 0.0,
            storm_cells: Vec::new(),
            master_seed: seed,
            time_accumulator: 0.0,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn force_state(&mut self, state: WeatherState) {
        self.state = state;
        self.target_state = state;
        self.transition_progress = 1.0;
        match state {
            WeatherState::Clear => {
                self.cloud_coverage = 0.1;
                self.humidity = 0.3;
            }
            WeatherState::Cloudy => {
                self.cloud_coverage = 0.6;
                self.humidity = 0.6;
            }
            WeatherState::Storm => {
                self.cloud_coverage = 0.95;
                self.humidity = 0.9;
            }
            WeatherState::Rain => {
                self.cloud_coverage = 0.8;
                self.humidity = 0.85;
            }
            WeatherState::Clearing => {
                self.cloud_coverage = 0.4;
                self.humidity = 0.5;
            }
        }
    }

    pub fn update(&mut self, dt: f32, day_progress: f32) {
        self.time_accumulator += dt;
        self.update_base_conditions(day_progress);
        self.try_spawn_storm(dt, day_progress);
        self.update_storms(dt);
        self.update_transitions(dt);
    }

    fn update_base_conditions(&mut self, day_progress: f32) {
        let sun_height = (day_progress * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2).sin();
        let night = (1.0 - sun_height).max(0.0);
        self.temperature = 20.0 - 10.0 * night;
        let humidity_drift = (self.time_accumulator * 0.0003).sin() * 0.2;
        self.humidity = (self.humidity + humidity_drift * 0.01).clamp(0.1, 1.0);
        self.pressure = 1013.25 - self.cloud_coverage * 20.0;
    }

    fn try_spawn_storm(&mut self, dt: f32, _day_progress: f32) {
        if self.storm_cells.len() >= 3 {
            return;
        }
        let storm_risk =
            (self.humidity - 0.4) * (1.0 - (self.pressure - 990.0) / 40.0).clamp(0.0, 1.0);
        if storm_risk < 0.3 {
            return;
        }
        let roll = self.rng.gen::<f32>();
        if roll < storm_risk * dt * 0.01 {
            let offset_x = (self.rng.gen::<f32>() - 0.5) * 20000.0;
            let offset_z = (self.rng.gen::<f32>() - 0.5) * 20000.0;
            let seed = self
                .master_seed
                .wrapping_add((self.time_accumulator * 1000.0) as u64)
                .wrapping_add((offset_x as u64) << 16)
                .wrapping_add(offset_z as u64);
            self.storm_cells
                .push(StormCell::new(Vec3::new(offset_x, 0.0, offset_z), seed));
        }
    }

    fn update_storms(&mut self, dt: f32) {
        for storm in &mut self.storm_cells {
            storm.update(dt);
        }
        self.storm_cells.retain(|s| s.intensity > 0.01);
    }

    fn update_transitions(&mut self, dt: f32) {
        let max_coverage = self
            .storm_cells
            .iter()
            .map(|s| s.intensity * s.influence_at(Vec3::ZERO))
            .fold(self.humidity * 0.6, |a, b| a.max(b));
        let target_coverage = match self.state {
            WeatherState::Clear => 0.05,
            WeatherState::Cloudy => 0.5 + self.humidity * 0.3,
            WeatherState::Storm => 0.9,
            WeatherState::Rain => 0.7,
            WeatherState::Clearing => 0.1,
        };
        let effective_target = target_coverage.max(max_coverage);
        self.cloud_coverage += (effective_target - self.cloud_coverage) * dt * 0.15;

        let has_active_storm = self.storm_cells.iter().any(|s| {
            s.lifecycle_stage == StormStage::Cumulonimbus
                || s.lifecycle_stage == StormStage::ToweringCumulus
        });
        let avg_precip: f32 = self
            .storm_cells
            .iter()
            .map(|s| s.precipitation_intensity())
            .sum::<f32>()
            / (self.storm_cells.len().max(1) as f32);

        let next = if has_active_storm && self.cloud_coverage > 0.5 {
            WeatherState::Storm
        } else if avg_precip > 0.2 {
            WeatherState::Rain
        } else if self.cloud_coverage > 0.4 {
            WeatherState::Cloudy
        } else if self.cloud_coverage < 0.15 && self.state != WeatherState::Clear {
            WeatherState::Clearing
        } else if self.state == WeatherState::Clearing && self.cloud_coverage < 0.05 {
            WeatherState::Clear
        } else {
            self.state
        };

        if next != self.target_state {
            self.target_state = next;
        }
        if next != self.state {
            self.transition_progress = 0.0;
            self.state = next;
        }
        self.transition_progress = (self.transition_progress + dt * 0.08).min(1.0);
    }

    pub fn cloud_coverage(&self) -> f32 {
        self.cloud_coverage.clamp(0.0, 1.0)
    }

    pub fn rain_intensity(&self) -> f32 {
        self.storm_cells
            .iter()
            .map(|s| s.precipitation_intensity() * s.influence_at(Vec3::ZERO))
            .fold(0.0f32, |a, b| a.max(b))
    }

    pub fn wind_speed(&self) -> f32 {
        match self.state {
            WeatherState::Clear => 2.0,
            WeatherState::Cloudy => 5.0,
            WeatherState::Storm => 15.0 + self.cloud_coverage * 10.0,
            WeatherState::Rain => 8.0,
            WeatherState::Clearing => 4.0,
        }
    }

    pub fn wind_direction(&self) -> Vec3 {
        let base = Vec3::new(1.0, 0.0, 0.3);
        let noise = ((self.time_accumulator * 0.1).sin() * 0.3).abs();
        let dir = base + Vec3::new(noise, 0.0, noise * 0.5);
        dir.normalize()
    }

    pub fn fog_density(&self) -> f32 {
        match self.state {
            WeatherState::Clear => 0.01,
            WeatherState::Cloudy => 0.03 + self.cloud_coverage * 0.05,
            WeatherState::Storm => 0.1,
            WeatherState::Rain => 0.06,
            WeatherState::Clearing => 0.02,
        }
    }

    pub fn lightning_active(&self) -> bool {
        self.storm_cells.iter().any(|s| {
            s.lifecycle_stage == StormStage::Cumulonimbus && s.lightning_probability() > 0.005
        })
    }
}
