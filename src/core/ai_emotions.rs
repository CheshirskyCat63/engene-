use serde::{Deserialize, Serialize};

use crate::world::components::{MonsterTraits, NpcTraits};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emotions {
    pub anger: f32,
    pub grief: f32,
    pub joy: f32,
    pub fear: f32,
    pub disgust: f32,
    pub surprise: f32,
    pub longing: f32,
}

impl Emotions {
    pub fn new() -> Self {
        Self {
            anger: 0.0,
            grief: 0.0,
            joy: 0.0,
            fear: 0.0,
            disgust: 0.0,
            surprise: 0.0,
            longing: 0.0,
        }
    }

    pub fn decay(&mut self, delta: f32) {
        let r = delta * 0.003;
        self.anger = (self.anger - r * 1.2).max(0.0);
        self.grief = (self.grief - r * 0.5).max(0.0);
        self.joy = (self.joy - r * 1.5).max(0.0);
        self.fear = (self.fear - r * 1.0).max(0.0);
        self.disgust = (self.disgust - r * 0.8).max(0.0);
        self.surprise = (self.surprise - r * 3.0).max(0.0);
        self.longing = (self.longing - r * 0.4).max(0.0);
    }

    pub fn dominant(&self) -> DominantEmotion {
        let vals = [
            (DominantEmotion::Anger, self.anger),
            (DominantEmotion::Grief, self.grief),
            (DominantEmotion::Joy, self.joy),
            (DominantEmotion::Fear, self.fear),
            (DominantEmotion::Disgust, self.disgust),
            (DominantEmotion::Surprise, self.surprise),
            (DominantEmotion::Longing, self.longing),
        ];
        vals.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .filter(|v| v.1 > 0.1)
            .map(|v| v.0)
            .unwrap_or(DominantEmotion::Calm)
    }

    pub fn mood(&self) -> f32 {
        self.joy * 1.0 - self.grief * 0.8 - self.anger * 0.3 - self.fear * 0.5 + 0.5
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DominantEmotion {
    Calm,
    Anger,
    Grief,
    Joy,
    Fear,
    Disgust,
    Surprise,
    Longing,
}

impl std::fmt::Display for DominantEmotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Calm => write!(f, "calm"),
            Self::Anger => write!(f, "ANGRY"),
            Self::Grief => write!(f, "grieving"),
            Self::Joy => write!(f, "happy"),
            Self::Fear => write!(f, "AFRAID"),
            Self::Disgust => write!(f, "disgusted"),
            Self::Surprise => write!(f, "surprised"),
            Self::Longing => write!(f, "longing"),
        }
    }
}

pub fn apply_npc_personality(
    emo: &mut Emotions,
    traits: &NpcTraits,
    raw_anger: f32,
    raw_fear: f32,
    raw_grief: f32,
    raw_joy: f32,
) {
    emo.anger = (emo.anger
        + raw_anger * traits.aggressiveness * (1.0 - traits.stress_resistance * 0.5))
        .min(1.0);
    emo.fear = (emo.fear
        + raw_fear * (1.0 - traits.bravery) * (1.0 - traits.stress_resistance * 0.3))
        .min(1.0);
    emo.grief = (emo.grief + raw_grief * traits.sociality).min(1.0);
    emo.joy = (emo.joy + raw_joy * (0.5 + traits.sociality * 0.5)).min(1.0);
}

pub fn apply_monster_personality(
    emo: &mut Emotions,
    traits: &MonsterTraits,
    raw_anger: f32,
    raw_fear: f32,
    raw_grief: f32,
    raw_joy: f32,
) {
    emo.anger = (emo.anger
        + raw_anger * traits.aggressiveness * (1.0 - traits.stress_tolerance * 0.3))
        .min(1.0);
    emo.fear = (emo.fear + raw_fear * traits.caution * (1.0 - traits.bravery * 0.5)).min(1.0);
    emo.grief = (emo.grief + raw_grief * traits.pack_mentality).min(1.0);
    emo.joy = (emo.joy + raw_joy * 0.5).min(1.0);
}
