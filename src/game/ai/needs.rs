use crate::world::components::PersonalNeeds;

const HUNGER_RATE: f32 = 0.015;
const THIRST_RATE: f32 = 0.020;
const SLEEP_RATE: f32 = 0.008;
const ENERGY_DRAIN: f32 = 0.010;
const CURIOSITY_RATE: f32 = 0.005;
const DISCOMFORT_RATE: f32 = 0.003;

pub fn decay_personal_needs(needs: &mut PersonalNeeds, delta: f32) {
    needs.hunger = (needs.hunger + delta * HUNGER_RATE).min(1.0);
    needs.thirst = (needs.thirst + delta * THIRST_RATE).min(1.0);
    needs.sleep = (needs.sleep + delta * SLEEP_RATE).min(1.0);
    needs.energy = (needs.energy - delta * ENERGY_DRAIN).max(0.0);
    needs.curiosity = (needs.curiosity + delta * CURIOSITY_RATE).min(1.0);
    needs.discomfort = (needs.discomfort + delta * DISCOMFORT_RATE).min(1.0);
}

pub fn satisfy_hunger(needs: &mut PersonalNeeds, amount: f32) {
    needs.hunger = (needs.hunger - amount).max(0.0);
    needs.energy = (needs.energy + amount * 0.3).min(1.0);
}

pub fn satisfy_thirst(needs: &mut PersonalNeeds, amount: f32) {
    needs.thirst = (needs.thirst - amount).max(0.0);
}

pub fn satisfy_sleep(needs: &mut PersonalNeeds, amount: f32) {
    needs.sleep = (needs.sleep - amount).max(0.0);
    needs.energy = (needs.energy + amount * 0.5).min(1.0);
    needs.discomfort = (needs.discomfort - amount * 0.2).max(0.0);
}

pub fn take_damage(needs: &mut PersonalNeeds, amount: f32) {
    needs.health = (needs.health - amount).max(0.0);
    needs.fear = (needs.fear + amount * 0.5).min(1.0);
}

pub fn heal(needs: &mut PersonalNeeds, amount: f32) {
    needs.health = (needs.health + amount).min(1.0);
    needs.fear = (needs.fear - amount * 0.1).max(0.0);
}

pub fn urgency(needs: &PersonalNeeds) -> f32 {
    let critical = needs.hunger.max(needs.thirst).max(needs.sleep);
    let low_health = 1.0 - needs.health;
    critical.max(low_health)
}
