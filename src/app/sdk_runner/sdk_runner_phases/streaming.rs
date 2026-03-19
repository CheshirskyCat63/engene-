use super::super::SdkApp;

pub fn run(
    app: &mut SdkApp,
    cam_pos: glam::Vec3,
) -> (
    Vec<crate::world::streaming::ChunkCoord>,
    Vec<crate::world::streaming::ChunkCoord>,
) {
    if let Some(streamer) = app
        .engine
        .resources
        .get_mut::<crate::world::streaming::WorldStreamer>()
    {
        let result = streamer.update(cam_pos.x, cam_pos.z);
        for coord in &result.0 {
            streamer.mark_loaded(*coord);
        }
        if !result.0.is_empty() || !result.1.is_empty() {
            app.spatial_dirty_journal.mark_chunk_structural_change();
        }
        result
    } else {
        (vec![], vec![])
    }
}
