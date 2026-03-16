use glam::Vec3;

#[derive(Clone, Debug)]
pub struct Breach {
    pub position: Vec3,
    pub width: f32,
    pub height: f32,
    pub passable: bool,
}

pub fn analyze_breach(
    section_integrity: f32,
    _wall_thickness: f32,
    opening_position: Vec3,
) -> Option<Breach> {
    if section_integrity > 0.3 {
        return None;
    }

    let width = (1.0 - section_integrity) * 2.0;
    let height = (1.0 - section_integrity) * 2.5;
    let passable = width > 0.6 && height > 1.5;

    Some(Breach {
        position: opening_position,
        width,
        height,
        passable,
    })
}
