//! Spatial Dirty Contracts
//!
//! Real production checks for spatial dirty policy and index behavior.

use engene::core::ecs::Entity;
use engene::app::spatial_dirty_journal::SpatialDirtyJournal;
use engene::world::hierarchical_spatial::{
    select_spatial_update_path, HierarchicalSpatialIndex, SpatialDirtyInput, SpatialUpdatePath,
};
use std::fs;

fn entity(n: u64) -> Entity {
    n
}

#[test]
fn incremental_matches_full_rebuild_for_single_move() {
    let mut index = HierarchicalSpatialIndex::new();
    let e1 = entity(1);
    let e2 = entity(2);

    index.insert_new(e1, 5.0, 5.0);
    index.insert_new(e2, 30.0, 30.0);
    index.update(e1, 5.0, 5.0, 12.0, 12.0);

    let from_incremental = index.query_physics(12.0, 12.0, 30.0);

    index.rebuild(&[(e1, 12.0, 12.0), (e2, 30.0, 30.0)]);
    let from_rebuild = index.query_physics(12.0, 12.0, 30.0);

    assert_eq!(from_incremental.len(), from_rebuild.len());
    for e in from_incremental {
        assert!(from_rebuild.contains(&e));
    }
}

#[test]
fn incremental_matches_full_rebuild_for_insert_remove_mix() {
    let mut index = HierarchicalSpatialIndex::new();
    let e1 = entity(1);
    let e2 = entity(2);
    let e3 = entity(3);
    let e4 = entity(4);

    index.insert_new(e1, 0.0, 0.0);
    index.insert_new(e2, 10.0, 10.0);
    index.insert_new(e3, 20.0, 20.0);
    index.remove(e2);
    index.insert_new(e4, 8.0, 8.0);
    index.update(e1, 0.0, 0.0, 15.0, 15.0);

    let incremental = index.query_physics(10.0, 10.0, 25.0);

    index.rebuild(&[(e1, 15.0, 15.0), (e3, 20.0, 20.0), (e4, 8.0, 8.0)]);
    let rebuilt = index.query_physics(10.0, 10.0, 25.0);

    assert_eq!(incremental.len(), rebuilt.len());
}

#[test]
fn editor_mutation_marks_spatial_dirty() {
    let mut dirty = SpatialDirtyInput::default();
    dirty.mark_editor_mutation();
    assert_eq!(
        select_spatial_update_path(&dirty),
        SpatialUpdatePath::Incremental
    );
}

#[test]
fn chunk_unload_marks_spatial_invalidation() {
    let mut dirty = SpatialDirtyInput::default();
    dirty.mark_chunk_structural_change();
    assert_eq!(
        select_spatial_update_path(&dirty),
        SpatialUpdatePath::FullRebuild
    );
}

#[test]
fn origin_shift_forces_rebuild_trigger() {
    let mut dirty = SpatialDirtyInput::default();
    dirty.mark_origin_shift();
    assert_eq!(
        select_spatial_update_path(&dirty),
        SpatialUpdatePath::FullRebuild
    );
}

#[test]
fn no_dirty_input_means_no_spatial_work() {
    let dirty = SpatialDirtyInput::default();
    assert_eq!(select_spatial_update_path(&dirty), SpatialUpdatePath::NoWork);
}

#[test]
fn journal_driven_path_does_not_require_global_dirty_scan_source_check() {
    let source = fs::read_to_string("src/app/sdk_runner/sdk_runner_phases/spatial.rs")
        .expect("src/app/sdk_runner/sdk_runner_phases/spatial.rs must exist");

    assert!(!source.contains("collect_dirty_from_ecs("));
    assert!(!source.contains("spatial_prev_alive"));
    assert!(!source.contains("spatial_prev_positions"));
}

#[test]
fn journal_to_input_reflects_written_dirty_flags() {
    let mut journal = SpatialDirtyJournal::default();
    journal.mark_inserted(entity(1));
    journal.mark_moved(entity(2));
    journal.mark_removed(entity(3));
    journal.mark_editor_mutation();
    journal.mark_chunk_structural_change();

    let input = journal.to_input();
    assert_eq!(input.inserted, vec![entity(1)]);
    assert_eq!(input.moved, vec![entity(2)]);
    assert_eq!(input.removed, vec![entity(3)]);
    assert!(input.editor_mutation);
    assert!(input.chunk_structural_change);
}
