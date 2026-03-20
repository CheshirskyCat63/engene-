use crate::world::biome::Biome;
use engine_audio::audio::{AudioEngine, SoundHandle, SoundKind};

pub struct BiomeAmbience {
    current_biome: Option<Biome>,
    ambient_handle: Option<SoundHandle>,
    wind_handle: Option<SoundHandle>,
}

impl BiomeAmbience {
    pub fn new() -> Self {
        Self {
            current_biome: None,
            ambient_handle: None,
            wind_handle: None,
        }
    }

    pub fn update(&mut self, engine: &mut AudioEngine, biome: Biome) {
        if self.current_biome == Some(biome) {
            return;
        }

        if let Some(h) = self.ambient_handle.take() {
            engine.stop(&h);
        }
        if let Some(h) = self.wind_handle.take() {
            engine.stop(&h);
        }

        let ambient_vol = match biome {
            Biome::Forest => 0.4,
            Biome::Swamp => 0.3,
            Biome::Plains => 0.2,
            Biome::Hills => 0.15,
            Biome::Settlement => 0.1,
        };

        self.ambient_handle = Some(engine.play_ambient(SoundKind::Ambient, ambient_vol));

        let wind_vol = match biome {
            Biome::Hills => 0.5,
            Biome::Plains => 0.3,
            Biome::Settlement => 0.1,
            _ => 0.15,
        };
        self.wind_handle = Some(engine.play_ambient(SoundKind::Wind, wind_vol));

        self.current_biome = Some(biome);
    }
}
