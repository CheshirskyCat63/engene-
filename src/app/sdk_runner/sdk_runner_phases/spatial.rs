use super::super::SdkApp;
use crate::world::hierarchical_spatial::{select_spatial_update_path, SpatialUpdatePath};

fn collect_dirty_from_ecs_journal(app: &mut SdkApp) {
    for e in app.engine.ecs.drain_spatial_inserted_journal() {
        app.spatial_dirty_journal.mark_inserted(e);
    }
    for e in app.engine.ecs.drain_spatial_moved_journal() {
        app.spatial_dirty_journal.mark_moved(e);
    }
    for e in app.engine.ecs.drain_spatial_removed_journal() {
        app.spatial_dirty_journal.mark_removed(e);
    }
}

fn rebuild_spatial_from_ecs(app: &mut SdkApp) {
    if let Some(spatial) = app
        .engine
        .resources
        .get_mut::<crate::world::hierarchical_spatial::HierarchicalSpatialIndex>()
    {
        let mut entities = Vec::with_capacity(app.engine.ecs.alive.len());
        let mut applied_positions = std::collections::HashMap::with_capacity(app.engine.ecs.alive.len());
        for &e in &app.engine.ecs.alive {
            if let Some(t) = app.engine.ecs.get_transform(e) {
                entities.push((e, t.x, t.y));
                applied_positions.insert(e, (t.x, t.y));
            }
        }
        spatial.rebuild(&entities);
        app.spatial_last_applied_positions = applied_positions;
    }
}

fn apply_incremental_update(app: &mut SdkApp) {
    if let Some(spatial) = app
        .engine
        .resources
        .get_mut::<crate::world::hierarchical_spatial::HierarchicalSpatialIndex>()
    {
        for &e in &app.spatial_dirty_journal.removed {
            spatial.remove(e);
            app.spatial_last_applied_positions.remove(&e);
        }

        for &e in &app.spatial_dirty_journal.inserted {
            if let Some(t) = app.engine.ecs.get_transform(e) {
                let (x, y) = (t.x, t.y);
                spatial.insert_new(e, x, y);
                app.spatial_last_applied_positions.insert(e, (x, y));
            }
        }

        for &e in &app.spatial_dirty_journal.moved {
            if let Some(t) = app.engine.ecs.get_transform(e) {
                let (new_x, new_y) = (t.x, t.y);
                if let Some((old_x, old_y)) = app.spatial_last_applied_positions.get(&e).copied()
                {
                    spatial.update(e, old_x, old_y, new_x, new_y);
                } else {
                    spatial.insert_new(e, new_x, new_y);
                }
                app.spatial_last_applied_positions.insert(e, (new_x, new_y));
            }
        }
    }
}

pub fn run(app: &mut SdkApp) {
    if let Some(origin) = app.engine.resources.get::<crate::world::origin_shift::OriginShift>() {
        if origin.shift_count != app.spatial_last_origin_shift_count {
            app.spatial_dirty_journal.mark_force_rebuild();
            app.spatial_last_origin_shift_count = origin.shift_count;
        }
    }

    collect_dirty_from_ecs_journal(app);

    let dirty_input = app.spatial_dirty_journal.to_input();
    let path = select_spatial_update_path(&dirty_input);
    match path {
        SpatialUpdatePath::NoWork => {}
        SpatialUpdatePath::Incremental => apply_incremental_update(app),
        SpatialUpdatePath::FullRebuild => rebuild_spatial_from_ecs(app),
    }

    app.spatial_last_update_path = path;
    app.spatial_dirty_journal.clear();
}
