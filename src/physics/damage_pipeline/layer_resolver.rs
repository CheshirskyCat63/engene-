use crate::physics::damage_pipeline::response_aggregator::DamageResponse;
use crate::physics::impact_event::{ImpactEvent, StressEvent};
use crate::physics::layered_damage::DamageableStore;
use crate::world::surface_db::{ResponseClass, SurfaceDB};

pub struct LayerResolver;

impl LayerResolver {
    pub fn resolve_impact(
        event: &ImpactEvent,
        surface_db: &SurfaceDB,
        store: &mut DamageableStore,
        responses: &mut Vec<DamageResponse>,
    ) {
        let entity = match event.target_entity {
            Some(e) => e,
            None => return,
        };

        let obj = match store.get_mut(entity) {
            Some(o) => o,
            None => return,
        };

        let mut remaining_energy = event.energy;

        for (idx, layer) in obj.layers.iter_mut().enumerate() {
            if remaining_energy <= 0.0 {
                break;
            }

            let mat = match surface_db.get(layer.material) {
                Some(m) => m,
                None => continue,
            };

            let layer_resistance = mat.compressive_strength * layer.thickness * layer.integrity;
            let fracture_result = Self::fracture_for_class(
                mat.response_class,
                remaining_energy,
                layer_resistance,
                mat.brittleness,
            );

            if fracture_result.fractured {
                layer.integrity -= fracture_result.integrity_loss;
                if layer.integrity <= 0.0 {
                    layer.integrity = 0.0;
                    responses.push(DamageResponse::LayerDetached {
                        entity,
                        layer_idx: idx as u8,
                        fragment_count: (mat.fragmentation_coeff * 8.0) as u8,
                    });
                } else {
                    responses.push(DamageResponse::LayerFractured {
                        entity,
                        layer_idx: idx as u8,
                        pattern: mat.fracture_pattern,
                        residual_energy: fracture_result.residual,
                    });
                }
            }

            remaining_energy = fracture_result.residual;
        }
    }

    pub fn resolve_stress(
        event: &StressEvent,
        surface_db: &SurfaceDB,
        store: &mut DamageableStore,
        _responses: &mut Vec<DamageResponse>,
    ) {
        let obj = match store.get_mut(event.target_entity) {
            Some(o) => o,
            None => return,
        };

        for layer in obj.layers.iter_mut() {
            let mat = match surface_db.get(layer.material) {
                Some(m) => m,
                None => continue,
            };

            let damage_rate = event.intensity * event.duration;
            match event.damage_class {
                crate::physics::damage_taxonomy::DamageClass::Thermal => {
                    layer.thermal_damage += damage_rate / mat.ignition_temp.max(1.0);
                    if layer.thermal_damage > 1.0 {
                        layer.integrity -= 0.01 * damage_rate;
                    }
                }
                crate::physics::damage_taxonomy::DamageClass::Hydraulic
                | crate::physics::damage_taxonomy::DamageClass::Erosion => {
                    layer.moisture_damage += damage_rate * mat.porosity;
                    layer.integrity -= damage_rate * (1.0 - mat.erosion_resistance) * 0.001;
                }
                crate::physics::damage_taxonomy::DamageClass::Corrosion => {
                    layer.integrity -= damage_rate * 0.01;
                }
                crate::physics::damage_taxonomy::DamageClass::Fatigue => {
                    layer.accumulated_stress += damage_rate;
                    if layer.accumulated_stress > mat.compressive_strength * layer.thickness {
                        layer.integrity -= 0.005;
                    }
                }
                _ => {}
            }
            layer.integrity = layer.integrity.clamp(0.0, 1.0);
        }
    }

    fn fracture_for_class(
        class: ResponseClass,
        energy: f32,
        resistance: f32,
        brittleness: f32,
    ) -> FractureResult {
        if energy < resistance * 0.1 {
            return FractureResult {
                fractured: false,
                integrity_loss: 0.0,
                residual: 0.0,
            };
        }

        let ratio = energy / resistance.max(0.01);
        let absorbed = match class {
            ResponseClass::BrittleCeramic | ResponseClass::BrittleGlassRadial => {
                energy * brittleness.max(0.5)
            }
            ResponseClass::DuctileMetal => energy * 0.8,
            ResponseClass::AnisotropicWood => energy * 0.6,
            ResponseClass::BiologicalSoft => energy * 0.3,
            ResponseClass::BiologicalHard => energy * 0.5,
            _ => energy * 0.5,
        };

        let residual = (energy - absorbed).max(0.0);
        let integrity_loss = (ratio * brittleness).min(1.0);

        FractureResult {
            fractured: ratio > 0.3,
            integrity_loss,
            residual,
        }
    }
}

struct FractureResult {
    fractured: bool,
    integrity_loss: f32,
    residual: f32,
}
