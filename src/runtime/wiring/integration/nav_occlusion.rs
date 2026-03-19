use crate::core::events::canonical::*;
use crate::core::mutation_policy::FixedTickContext;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;
use crate::graphics::destruction_occlusion::{DestructionOcclusionSystem, OcclusionBreach};
use crate::navigation::dynamic_nav_update::NavDirtyTracker;

const NAV_CELL_SIZE: f32 = 8.0;

// ---------------------------------------------------------------------------
// 5. NavDirtyTickSystem
// ---------------------------------------------------------------------------

pub struct NavDirtyTickSystem;

impl EngineSystem for NavDirtyTickSystem {
    fn name(&self) -> &str {
        "NavDirtyTick"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("NavDirtyTick")
            .reads_resource::<NavDirtyTracker>()
            .reads_event::<WorldTopologyChanged>()
            .reads_event::<TerrainChanged>()
            .emits_event::<NavUpdated>()
            .emits_event::<CoverChanged>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(nav) = ctx.resources.get_mut::<NavDirtyTracker>() else {
            return;
        };

        let topo: Vec<WorldTopologyChanged> = ctx
            .events
            .read::<WorldTopologyChanged>()
            .iter()
            .map(|r| (*r).clone())
            .collect();
        let terrain: Vec<TerrainChanged> = ctx
            .events
            .read::<TerrainChanged>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &topo {
            nav.mark_area_dirty(ev.position, ev.radius, NAV_CELL_SIZE);
        }

        for ev in &terrain {
            for &(cx, cz) in &ev.patches {
                nav.mark_dirty(cx, cz);
            }
        }

        let batch = nav.drain_dirty_batch();
        let count = batch.len();

        if count > 0 {
            ctx.events.emit(NavUpdated {
                dirty_cells_processed: count,
            });
            ctx.events.emit(CoverChanged {
                cells_updated: count,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// 6. OcclusionWireSystem
// ---------------------------------------------------------------------------

pub struct OcclusionWireSystem;

impl EngineSystem for OcclusionWireSystem {
    fn name(&self) -> &str {
        "OcclusionWire"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("OcclusionWire")
            .reads_resource::<DestructionOcclusionSystem>()
            .reads_event::<StructuralCollapse>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(occlusion) = ctx.resources.get_mut::<DestructionOcclusionSystem>() else {
            return;
        };

        let collapses: Vec<StructuralCollapse> = ctx
            .events
            .read::<StructuralCollapse>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &collapses {
            let radius = 2.0 + (ev.cluster_count as f32).sqrt();
            occlusion.register_breach(OcclusionBreach {
                position: ev.position,
                radius,
                sound_passthrough: 0.6,
                light_passthrough: 0.5,
                vision_passthrough: 0.7,
            });
        }
    }
}