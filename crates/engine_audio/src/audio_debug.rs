#[derive(Clone, Debug, Default)]
pub struct AudioDebugState {
    pub active_channels: u32,
    pub max_channels: u32,
    pub active_sources: Vec<AudioSourceInfo>,
    pub reverb_zone: Option<String>,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ambient_volume: f32,
}

#[derive(Clone, Debug)]
pub struct AudioSourceInfo {
    pub name: String,
    pub position: [f32; 3],
    pub volume: f32,
    pub distance: f32,
    pub occluded: bool,
    pub looping: bool,
}

impl AudioDebugState {
    pub fn new() -> Self {
        Self {
            max_channels: 32,
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,
            ambient_volume: 0.5,
            ..Default::default()
        }
    }

    pub fn channel_usage(&self) -> f32 {
        if self.max_channels == 0 {
            return 0.0;
        }
        self.active_channels as f32 / self.max_channels as f32
    }
}
