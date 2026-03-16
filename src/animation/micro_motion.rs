use glam::Vec3;

const MAX_CPU_OSCILLATORS: usize = 256;

#[derive(Clone, Debug)]
pub enum MicroMotionType {
    WindDriven {
        amplitude: f32,
        frequency: f32,
        phase: f32,
    },
    Pendulum {
        length: f32,
        damping: f32,
    },
    Oscillation {
        axis: Vec3,
        amplitude: f32,
        frequency: f32,
    },
    ProximityReact {
        react_radius: f32,
        displacement: f32,
        recovery_speed: f32,
    },
}

#[derive(Clone, Debug)]
pub struct MicroMotionComponent {
    pub motion_type: MicroMotionType,
    pub current_offset: Vec3,
    pub current_rotation: f32,
    pub motion_class: ObjectMotionClass,
}

pub struct MicroMotionLimits {
    pub max_wind_amplitude: f32,
    pub max_pendulum_angle: f32,
    pub max_oscillation_amplitude: f32,
    pub max_proximity_displacement: f32,
    pub frequency_range: (f32, f32),
}

impl Default for MicroMotionLimits {
    fn default() -> Self {
        Self {
            max_wind_amplitude: 0.05,
            max_pendulum_angle: 5.0,
            max_oscillation_amplitude: 0.03,
            max_proximity_displacement: 0.15,
            frequency_range: (0.3, 2.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectMotionClass {
    Vegetation,
    LightClutter,
    HangingObject,
    FlexiblePanel,
    Character,
}

pub fn validate_motion_gating(motion: &MicroMotionType, class: ObjectMotionClass) -> bool {
    match motion {
        MicroMotionType::WindDriven { .. } => matches!(
            class,
            ObjectMotionClass::Vegetation
                | ObjectMotionClass::LightClutter
                | ObjectMotionClass::HangingObject
        ),
        MicroMotionType::Pendulum { .. } => class == ObjectMotionClass::HangingObject,
        MicroMotionType::Oscillation { .. } => class == ObjectMotionClass::FlexiblePanel,
        MicroMotionType::ProximityReact { .. } => matches!(
            class,
            ObjectMotionClass::Vegetation | ObjectMotionClass::LightClutter
        ),
    }
}

pub struct MicroMotionSystem {
    wind_time: f32,
    limits: MicroMotionLimits,
}

impl MicroMotionSystem {
    pub fn new() -> Self {
        Self {
            wind_time: 0.0,
            limits: MicroMotionLimits::default(),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        wind_direction: Vec3,
        wind_strength: f32,
        components: &mut [MicroMotionComponent],
    ) {
        self.wind_time += dt;

        let count = components.len().min(MAX_CPU_OSCILLATORS);
        for comp in components[..count].iter_mut() {
            match &comp.motion_type {
                MicroMotionType::WindDriven {
                    amplitude,
                    frequency,
                    phase,
                } => {
                    let a = amplitude.min(self.limits.max_wind_amplitude);
                    let f = frequency.clamp(self.limits.frequency_range.0, self.limits.frequency_range.1);
                    let t = (self.wind_time * f + phase).sin();
                    comp.current_offset = wind_direction * t * a * wind_strength;
                }
                MicroMotionType::Pendulum { length, damping } => {
                    let angle_rad = self.limits.max_pendulum_angle.to_radians();
                    let t = (self.wind_time * (9.81 / length.max(0.1)).sqrt()).sin();
                    let damped = t * (-self.wind_time * damping).exp().max(0.3);
                    comp.current_rotation = damped * angle_rad;
                }
                MicroMotionType::Oscillation {
                    axis,
                    amplitude,
                    frequency,
                } => {
                    let a = amplitude.min(self.limits.max_oscillation_amplitude);
                    let f = frequency.clamp(self.limits.frequency_range.0, self.limits.frequency_range.1);
                    let t = (self.wind_time * f * std::f32::consts::TAU).sin();
                    comp.current_offset = *axis * t * a;
                }
                MicroMotionType::ProximityReact {
                    displacement,
                    recovery_speed,
                    ..
                } => {
                    let _d = displacement.min(self.limits.max_proximity_displacement);
                    let current_len = comp.current_offset.length();
                    if current_len > 0.001 {
                        let recovery = recovery_speed * dt;
                        let new_len = (current_len - recovery).max(0.0);
                        comp.current_offset = comp.current_offset.normalize_or_zero() * new_len;
                    }
                }
            }
        }
    }
}
