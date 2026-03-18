//! Phase 7.5.4: Interior weather volume definitions.

use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShelterClass {
    OpenSky,
    Canopy,
    SemiEnclosed,
    EnclosedBroken,
    FullyEnclosed,
}

pub struct WeatherOpening {
    pub position: Vec3,
    pub normal: Vec3,
    pub area: f32,
    pub has_glass: bool,
    pub exposure_cone_angle: f32,
}

pub struct InteriorWeatherVolume {
    pub min: Vec3,
    pub max: Vec3,
    pub shelter_class: ShelterClass,
    pub shelter_coefficient: f32,
    pub openings: Vec<WeatherOpening>,
}

impl InteriorWeatherVolume {
    pub fn contains(&self, pos: Vec3) -> bool {
        pos.x >= self.min.x
            && pos.x <= self.max.x
            && pos.y >= self.min.y
            && pos.y <= self.max.y
            && pos.z >= self.min.z
            && pos.z <= self.max.z
    }

    pub fn rain_factor(&self, rain_dir: Vec3) -> f32 {
        match self.shelter_class {
            ShelterClass::OpenSky => 1.0,
            ShelterClass::Canopy => {
                let lateral = (rain_dir.x * rain_dir.x + rain_dir.z * rain_dir.z).sqrt();
                lateral * 0.5
            }
            ShelterClass::SemiEnclosed => 0.3,
            ShelterClass::EnclosedBroken => {
                let mut ingress = 0.0;
                for opening in &self.openings {
                    if opening.has_glass {
                        continue;
                    }
                    let alignment = (-rain_dir).dot(opening.normal).max(0.0);
                    ingress += alignment * opening.area * 0.01;
                }
                ingress.min(0.5)
            }
            ShelterClass::FullyEnclosed => 0.0,
        }
    }
}
