use super::super::SdkApp;

pub fn run(app: &mut SdkApp, sim_dt: f32) {
    if !app.sim_paused {
        app.engine.tick(sim_dt);
    }
}
