use super::super::SdkApp;

pub fn run(
    app: &mut SdkApp,
    to_load: &[engine_world::streaming::ChunkCoord],
    to_unload: &[engine_world::streaming::ChunkCoord],
) {
    if !to_unload.is_empty() || !to_load.is_empty() {
        if let Some(mut persistence) = app
            .engine
            .resources
            .take::<engine_world::chunk_persistence::ChunkPersistenceService>()
        {
            let tick = app.engine.ecs.tick;
            for coord in to_unload {
                persistence.save_and_unload(*coord, &mut app.engine.ecs, tick);
            }
            for coord in to_load {
                persistence.load_chunk_entities(*coord, &mut app.engine.ecs);
            }
            app.engine.resources.insert_runtime(persistence);
        }
    }
}
