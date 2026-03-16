use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoundEvent {
    FootstepDirt,
    FootstepGrass,
    FootstepConcrete,
    FootstepMetal,
    FootstepWater,
    WeaponFirePistol,
    WeaponFireRifle,
    WeaponFireShotgun,
    WeaponReload,
    WeaponEmpty,
    BulletImpactFlesh,
    BulletImpactMetal,
    BulletImpactDirt,
    BulletImpactWood,
    MonsterGrowlWolf,
    MonsterGrowlBoar,
    MonsterGrowlBloodsucker,
    MonsterAttack,
    MonsterDeath,
    NpcHurt,
    NpcDeath,
    NpcGreeting,
    NpcAlarm,
    AnomalyHum,
    AnomalyDischarge,
    ArtifactPickup,
    ItemPickup,
    ItemDrop,
    TradeComplete,
    QuestAccepted,
    QuestCompleted,
    AmbientWind,
    AmbientRain,
    AmbientNight,
    AmbientCamp,
    AmbientForest,
    DoorOpen,
    DoorClose,
    ExplosionSmall,
    ExplosionLarge,
    MedkitUse,
    FoodEat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundDef {
    pub event: SoundEvent,
    pub file_path: String,
    pub volume: f32,
    pub pitch_variance: f32,
    pub max_distance: f32,
    pub priority: u8,
    pub looping: bool,
}

pub struct SoundBank {
    sounds: HashMap<SoundEvent, Vec<SoundDef>>,
}

impl SoundBank {
    pub fn new() -> Self {
        let mut bank = Self {
            sounds: HashMap::new(),
        };
        bank.register_placeholders();
        bank
    }

    fn register_placeholders(&mut self) {
        let placeholders = vec![
            (
                SoundEvent::FootstepDirt,
                "audio/footstep_dirt.wav",
                0.5,
                false,
            ),
            (
                SoundEvent::FootstepGrass,
                "audio/footstep_grass.wav",
                0.4,
                false,
            ),
            (
                SoundEvent::FootstepConcrete,
                "audio/footstep_concrete.wav",
                0.6,
                false,
            ),
            (
                SoundEvent::WeaponFirePistol,
                "audio/pistol_fire.wav",
                0.8,
                false,
            ),
            (
                SoundEvent::WeaponFireRifle,
                "audio/rifle_fire.wav",
                0.9,
                false,
            ),
            (
                SoundEvent::WeaponReload,
                "audio/weapon_reload.wav",
                0.5,
                false,
            ),
            (
                SoundEvent::BulletImpactFlesh,
                "audio/impact_flesh.wav",
                0.6,
                false,
            ),
            (
                SoundEvent::MonsterGrowlWolf,
                "audio/wolf_growl.wav",
                0.7,
                false,
            ),
            (
                SoundEvent::MonsterGrowlBoar,
                "audio/boar_growl.wav",
                0.6,
                false,
            ),
            (
                SoundEvent::NpcHurt,
                "audio/npc_hurt.wav",
                0.7,
                false,
            ),
            (
                SoundEvent::NpcDeath,
                "audio/npc_death.wav",
                0.8,
                false,
            ),
            (
                SoundEvent::AnomalyHum,
                "audio/anomaly_hum.wav",
                0.5,
                true,
            ),
            (
                SoundEvent::AmbientWind,
                "audio/ambient_wind.wav",
                0.3,
                true,
            ),
            (
                SoundEvent::AmbientRain,
                "audio/ambient_rain.wav",
                0.4,
                true,
            ),
            (
                SoundEvent::AmbientNight,
                "audio/ambient_night.wav",
                0.3,
                true,
            ),
            (
                SoundEvent::AmbientCamp,
                "audio/ambient_camp.wav",
                0.4,
                true,
            ),
            (
                SoundEvent::AmbientForest,
                "audio/ambient_forest.wav",
                0.3,
                true,
            ),
            (
                SoundEvent::ItemPickup,
                "audio/item_pickup.wav",
                0.5,
                false,
            ),
            (
                SoundEvent::TradeComplete,
                "audio/trade_complete.wav",
                0.5,
                false,
            ),
            (
                SoundEvent::QuestAccepted,
                "audio/quest_accept.wav",
                0.5,
                false,
            ),
            (
                SoundEvent::QuestCompleted,
                "audio/quest_complete.wav",
                0.6,
                false,
            ),
            (SoundEvent::MedkitUse, "audio/medkit_use.wav", 0.5, false),
            (
                SoundEvent::ExplosionSmall,
                "audio/explosion_small.wav",
                0.9,
                false,
            ),
            (
                SoundEvent::ExplosionLarge,
                "audio/explosion_large.wav",
                1.0,
                false,
            ),
        ];

        for (event, path, volume, looping) in placeholders {
            self.register(SoundDef {
                event: event.clone(),
                file_path: path.to_string(),
                volume,
                pitch_variance: 0.1,
                max_distance: 100.0,
                priority: 5,
                looping,
            });
        }
    }

    pub fn register(&mut self, def: SoundDef) {
        self.sounds.entry(def.event.clone()).or_default().push(def);
    }

    pub fn get(&self, event: &SoundEvent) -> Option<&SoundDef> {
        self.sounds.get(event).and_then(|v| v.first())
    }

    pub fn get_random(&self, event: &SoundEvent) -> Option<&SoundDef> {
        self.sounds.get(event).and_then(|v| {
            if v.is_empty() {
                None
            } else {
                Some(&v[0])
            }
        })
    }

    pub fn event_count(&self) -> usize {
        self.sounds.len()
    }
}

impl Default for SoundBank {
    fn default() -> Self {
        Self::new()
    }
}
