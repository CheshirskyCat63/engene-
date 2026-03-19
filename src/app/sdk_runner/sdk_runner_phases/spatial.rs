use super::super::SdkApp;

pub fn run(app: &mut SdkApp) {
    if let Some(spatial) = app
        .engine
        .resources
        .get_mut::<crate::world::hierarchical_spatial::HierarchicalSpatialIndex>()
    {
        spatial.clear();
        for &e in &app.engine.ecs.alive {
            if let Some(t) = app.engine.ecs.get_transform(e) {
                spatial.insert(e, t.x, t.y);
            }
        }
    }
}
