use super::super::SdkApp;
use crate::world::hierarchical_spatial::{select_spatial_update_path, SpatialUpdatePath};

fn collect_dirty_from_ecs(app: &mut SdkApp) {
    let mut current_alive = std::collections::HashSet::with_capacity(app.engine.ecs.alive.len());
    for &e in &app.engine.ecs.alive {
        current_alive.insert(e);
    }

    for e in current_alive.difference(&app.spatial_prev_alive) {
        app.spatial_dirty_input.inserted.push(*e);
    }
    for e in app.spatial_prev_alive.difference(&current_alive) {
        app.spatial_dirty_input.removed.push(*e);
        app.spatial_prev_positions.remove(e);
    }

    for &e in &current_alive {
        if let Some(t) = app.engine.ecs.get_transform(e) {
            let new_pos = (t.x, t.y);
            if let Some(prev) = app.spatial_prev_positions.get(&e).copied() {
                if prev != new_pos {
                    app.spatial_dirty_input.moved.push(e);
                }
            }
            app.spatial_prev_positions.insert(e, new_pos);
        }
    }

    app.spatial_prev_alive = current_alive;
}

fn rebuild_spatial_from_ecs(app: &mut SdkApp) {
    if let Some(spatial) = app
        .engine
        .resources
        .get_mut::<crate::world::hierarchical_spatial::HierarchicalSpatialIndex>()
    {
        let mut entities = Vec::with_capacity(app.engine.ecs.alive.len());
        for &e in &app.engine.ecs.alive {
            if let Some(t) = app.engine.ecs.get_transform(e) {
                entities.push((e, t.x, t.y));
            }
        }
        spatial.rebuild(&entities);
    }
}

fn apply_incremental_update(app: &mut SdkApp) {
    if let Some(spatial) = app
        .engine
        .resources
        .get_mut::<crate::world::hierarchical_spatial::HierarchicalSpatialIndex>()
    {
        for &e in &app.spatial_dirty_input.removed {
            spatial.remove(e);
        }

        for &e in &app.spatial_dirty_input.inserted {
            if let Some((x, y)) = app.spatial_prev_positions.get(&e).copied() {
                spatial.insert_new(e, x, y);
            }
        }

        for &e in &app.spatial_dirty_input.moved {
            if let Some((new_x, new_y)) = app.spatial_prev_positions.get(&e).copied() {
                if let Some((old_x, old_y)) = app.spatial_last_applied_positions.get(&e).copied() {
                    spatial.update(e, old_x, old_y, new_x, new_y);
                } else {
                    spatial.insert_new(e, new_x, new_y);
                }
            }
        }
    }
}

fn refresh_applied_positions(app: &mut SdkApp) {
    let mut next = std::collections::HashMap::with_capacity(app.engine.ecs.alive.len());
    for &e in &app.engine.ecs.alive {
        if let Some(t) = app.engine.ecs.get_transform(e) {
            next.insert(e, (t.x, t.y));
        }
    }
    app.spatial_last_applied_positions = next;
}

pub fn run(app: &mut SdkApp) {
    if let Some(origin) = app.engine.resources.get::<crate::world::origin_shift::OriginShift>() {
        if origin.shift_count != app.spatial_last_origin_shift_count {
            app.spatial_dirty_input.mark_origin_shift();
            app.spatial_last_origin_shift_count = origin.shift_count;
        }
    }

    collect_dirty_from_ecs(app);

    let path = select_spatial_update_path(&app.spatial_dirty_input);
    match path {
        SpatialUpdatePath::NoWork => {}
        SpatialUpdatePath::Incremental => apply_incremental_update(app),
        SpatialUpdatePath::FullRebuild => rebuild_spatial_from_ecs(app),
    }

    refresh_applied_positions(app);
    app.spatial_last_update_path = path;
    app.spatial_dirty_input = crate::world::hierarchical_spatial::SpatialDirtyInput::default();
}
