pub fn init() {
    puffin::set_scopes_on(true);
}

pub fn frame_start() {
    puffin::GlobalProfiler::lock().new_frame();
}
