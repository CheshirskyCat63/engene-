#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskGroup {
    Simulation,
    Physics,
    AI,
    Navigation,
    Audio,
    Render,
    Streaming,
    Background,
}
