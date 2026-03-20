use super::super::SdkApp;
use engine_render::renderer::RenderCamera;
use engine_render::visibility::Frustum;

pub fn run(app: &mut SdkApp, vp: glam::Mat4, cam_pos: glam::Vec3, _dt: f32) {
    if let Some(r) = app.renderer.as_mut() {
        let frustum = Frustum::from_view_projection(&vp);
        let instances = super::super::collect_entity_instances(
            &app.engine.ecs,
            &app.heightmap,
            [cam_pos.x, cam_pos.y, cam_pos.z],
            &frustum,
        );
        r.update_entities(&instances);

        let render_cam = RenderCamera {
            view_proj: vp.to_cols_array_2d(),
            inv_view_proj: vp.inverse().to_cols_array_2d(),
            position: [cam_pos.x, cam_pos.y, cam_pos.z],
            forward: {
                let f = app.camera.forward();
                [f.x, f.y, f.z]
            },
            near: app.camera.near,
            far: app.camera.far,
            day_progress: app.engine.time.day_progress(),
        };

        let telemetry = engine_core::perf::telemetry::Telemetry::new();
        let ecs_ref = &app.engine.ecs;
        let events_ref = &app.engine.events;
        let shell = &mut app.editor_shell;

        match r.render_with_egui(&render_cam, |ctx| {
            shell.draw_with_event_bus(ctx, ecs_ref, &telemetry, events_ref);
        }) {
            Ok(()) => {}
            Err(wgpu::SurfaceError::Lost) => {
                let (w, h) = r.size();
                r.resize(w, h);
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("Out of memory during render");
            }
            Err(e) => eprintln!("render error: {e}"),
        }
    }
}
