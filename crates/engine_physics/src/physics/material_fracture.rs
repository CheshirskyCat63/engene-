use crate::world::surface_db::{ResponseClass, SurfaceMaterial};

pub struct FractureResult {
    pub fractured: bool,
    pub integrity_loss: f32,
    pub fragment_count: u8,
    pub residual_energy: f32,
}

pub fn compute_fracture(
    material: &SurfaceMaterial,
    energy: f32,
    contact_area: f32,
) -> FractureResult {
    let stress = energy / contact_area.max(0.001);
    let threshold = material.compressive_strength * 0.3;

    if stress < threshold {
        return FractureResult {
            fractured: false,
            integrity_loss: 0.0,
            fragment_count: 0,
            residual_energy: 0.0,
        };
    }

    match material.response_class {
        ResponseClass::BrittleCeramic => brittle_clustered(material, stress, energy),
        ResponseClass::BrittleGlassRadial => brittle_radial(material, stress, energy),
        ResponseClass::AnisotropicWood => anisotropic_split(material, stress, energy),
        ResponseClass::DuctileMetal => ductile_dent(material, stress, energy),
        ResponseClass::BiologicalSoft => biological_rupture(material, stress, energy),
        ResponseClass::BiologicalHard => bone_fracture(material, stress, energy),
        ResponseClass::LayeredMasonry => masonry_fracture(material, stress, energy),
        ResponseClass::Ite => aggregate_fracture(material, stress, energy),
        ResponseClass::Composite => composite_fracture(material, stress, energy),
    }
}

fn brittle_clustered(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.compressive_strength.max(1.0);
    FractureResult {
        fractured: true,
        integrity_loss: (ratio * mat.brittleness).min(1.0),
        fragment_count: (mat.fragmentation_coeff * 12.0).min(15.0) as u8,
        residual_energy: energy * (1.0 - mat.brittleness * 0.8),
    }
}

fn brittle_radial(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.compressive_strength.max(1.0);
    FractureResult {
        fractured: true,
        integrity_loss: ratio.min(1.0),
        fragment_count: (mat.fragmentation_coeff * 16.0).min(15.0) as u8,
        residual_energy: energy * 0.1,
    }
}

fn anisotropic_split(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.tensile_strength.max(1.0);
    FractureResult {
        fractured: ratio > 0.5,
        integrity_loss: (ratio * 0.4).min(1.0),
        fragment_count: (mat.fragmentation_coeff * 4.0).min(15.0) as u8,
        residual_energy: energy * 0.4,
    }
}

fn ductile_dent(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.tensile_strength.max(1.0);
    FractureResult {
        fractured: ratio > 1.5,
        integrity_loss: (ratio * 0.2).min(1.0),
        fragment_count: 0,
        residual_energy: energy * (1.0 - mat.elasticity),
    }
}

fn biological_rupture(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.tensile_strength.max(0.1);
    FractureResult {
        fractured: ratio > 0.3,
        integrity_loss: (ratio * 0.6).min(1.0),
        fragment_count: 0,
        residual_energy: energy * 0.7,
    }
}

fn bone_fracture(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.compressive_strength.max(1.0);
    FractureResult {
        fractured: ratio > 0.6,
        integrity_loss: (ratio * 0.5).min(1.0),
        fragment_count: (mat.fragmentation_coeff * 6.0).min(15.0) as u8,
        residual_energy: energy * 0.3,
    }
}

fn masonry_fracture(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.compressive_strength.max(1.0);
    FractureResult {
        fractured: ratio > 0.4,
        integrity_loss: (ratio * mat.brittleness).min(1.0),
        fragment_count: (mat.fragmentation_coeff * 10.0).min(15.0) as u8,
        residual_energy: energy * 0.3,
    }
}

fn aggregate_fracture(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.compressive_strength.max(1.0);
    FractureResult {
        fractured: ratio > 0.5,
        integrity_loss: (ratio * 0.5).min(1.0),
        fragment_count: (mat.fragmentation_coeff * 8.0).min(15.0) as u8,
        residual_energy: energy * 0.4,
    }
}

fn composite_fracture(mat: &SurfaceMaterial, stress: f32, energy: f32) -> FractureResult {
    let ratio = stress / mat.compressive_strength.max(1.0);
    FractureResult {
        fractured: ratio > 0.8,
        integrity_loss: (ratio * 0.3).min(1.0),
        fragment_count: (mat.fragmentation_coeff * 3.0).min(15.0) as u8,
        residual_energy: energy * (1.0 - mat.elasticity * 0.5),
    }
}
