use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub looping: bool,
    pub blend_in: f32,
    pub blend_out: f32,
    pub layer: AnimationLayer,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationLayer {
    Base,
    Upper,
    Additive,
    Override,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationState {
    Idle,
    Walk,
    Run,
    Sprint,
    Crouch,
    CrouchWalk,
    AttackMelee,
    AttackRanged,
    Reload,
    HitReactLight,
    HitReactHeavy,
    DeathFall,
    DeathRagdoll,
    InjuryLimp,
    InjuryCrawl,
    Interact,
    Eat,
    Sleep,
}

pub struct ClipMap {
    clips: HashMap<String, AnimationClip>,
    state_bindings: HashMap<AnimationState, String>,
}

impl ClipMap {
    pub fn new() -> Self {
        let mut map = Self {
            clips: HashMap::new(),
            state_bindings: HashMap::new(),
        };
        map.populate_defaults();
        map
    }

    fn populate_defaults(&mut self) {
        let defaults = vec![
            ("idle", AnimationState::Idle, 2.0, true),
            ("walk", AnimationState::Walk, 1.0, true),
            ("run", AnimationState::Run, 0.8, true),
            ("sprint", AnimationState::Sprint, 0.6, true),
            ("crouch", AnimationState::Crouch, 1.5, true),
            ("crouch_walk", AnimationState::CrouchWalk, 1.2, true),
            ("attack_melee", AnimationState::AttackMelee, 0.5, false),
            ("attack_ranged", AnimationState::AttackRanged, 0.3, false),
            ("reload", AnimationState::Reload, 2.0, false),
            ("hit_react_light", AnimationState::HitReactLight, 0.3, false),
            ("hit_react_heavy", AnimationState::HitReactHeavy, 0.6, false),
            ("death_fall", AnimationState::DeathFall, 1.5, false),
            ("death_ragdoll", AnimationState::DeathRagdoll, 0.1, false),
            ("injury_limp", AnimationState::InjuryLimp, 1.2, true),
            ("injury_crawl", AnimationState::InjuryCrawl, 2.0, true),
            ("interact", AnimationState::Interact, 1.0, false),
            ("eat", AnimationState::Eat, 3.0, false),
            ("sleep", AnimationState::Sleep, 5.0, true),
        ];

        for (name, state, duration, looping) in defaults {
            let clip = AnimationClip {
                name: name.to_string(),
                duration,
                looping,
                blend_in: 0.15,
                blend_out: 0.15,
                layer: AnimationLayer::Base,
            };
            self.clips.insert(name.to_string(), clip);
            self.state_bindings.insert(state, name.to_string());
        }
    }

    pub fn get_clip(&self, name: &str) -> Option<&AnimationClip> {
        self.clips.get(name)
    }

    pub fn clip_for_state(&self, state: &AnimationState) -> Option<&AnimationClip> {
        self.state_bindings.get(state)
            .and_then(|name| self.clips.get(name))
    }

    pub fn register_clip(&mut self, clip: AnimationClip) {
        self.clips.insert(clip.name.clone(), clip);
    }

    pub fn bind_state(&mut self, state: AnimationState, clip_name: &str) {
        self.state_bindings.insert(state, clip_name.to_string());
    }

    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }
}

impl Default for ClipMap {
    fn default() -> Self { Self::new() }
}
