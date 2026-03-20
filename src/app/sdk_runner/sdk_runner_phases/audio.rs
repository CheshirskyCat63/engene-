use super::super::SdkApp;

pub fn run(app: &mut SdkApp, cam_pos: glam::Vec3, cam_fwd: glam::Vec3, dt: f32) {
    if let Some(audio) = app
        .engine
        .resources
        .get_mut::<engine_audio::audio::AudioEngine>()
    {
        audio.set_listener(cam_pos, cam_fwd);
        audio.update(dt);
    }
}
