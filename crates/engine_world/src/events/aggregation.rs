//! Event aggregation: spatial bucketing of impact events by cell.

use std::collections::HashMap;

use glam::Vec3;

use crate::core::events::canonical::ImpactEvent;
use crate::world::surface_db::MaterialId;

#[derive(Clone, Debug)]
pub struct AggregatedImpact {
    pub position: Vec3,
    pub total_energy: f32,
    pub count: u32,
    pub material: MaterialId,
}

pub struct EventAggregator {
    impact_cells: HashMap<(i32, i32), AggregatedImpact>,
    cell_size: f32,
}

impl EventAggregator {
    pub fn new(cell_size: f32) -> Self {
        Self {
            impact_cells: HashMap::new(),
            cell_size: cell_size.max(0.001),
        }
    }

    fn cell_key(&self, pos: Vec3) -> (i32, i32) {
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cz = (pos.z / self.cell_size).floor() as i32;
        (cx, cz)
    }

    /// Aggregates an impact event into its spatial cell.
    pub fn submit(&mut self, impact: &ImpactEvent) {
        let key = self.cell_key(impact.position);

        if let Some(agg) = self.impact_cells.get_mut(&key) {
            let total = agg.total_energy + impact.energy;
            if total > 0.0 {
                let weight = impact.energy / total;
                agg.position = agg.position * (1.0 - weight) + impact.position * weight;
            }
            agg.total_energy = total;
            agg.count += 1;
            agg.material = impact.material_hit;
        } else {
            self.impact_cells.insert(
                key,
                AggregatedImpact {
                    position: impact.position,
                    total_energy: impact.energy,
                    count: 1,
                    material: impact.material_hit,
                },
            );
        }
    }

    /// Returns and clears all aggregated impacts.
    pub fn drain(&mut self) -> Vec<AggregatedImpact> {
        self.impact_cells.drain().map(|(_, v)| v).collect()
    }
}
