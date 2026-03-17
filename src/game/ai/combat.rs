use rand::Rng;

use crate::game::ai::witness;
use crate::core::ecs::{Ecs, Entity};
use crate::core::events::EventBus;
use crate::core::events::canonical::CombatHit;
use crate::game::ecosystem::food_chain;
use crate::world::components::*;

#[derive(Clone, Copy, Debug)]
pub enum HitLocation {
    Head,
    Torso,
    Arms,
    Legs,
}

impl HitLocation {
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0u8..10) {
            0 => Self::Head,
            1..=4 => Self::Torso,
            5..=7 => Self::Arms,
            _ => Self::Legs,
        }
    }

    pub fn damage_multiplier(&self) -> f32 {
        match self {
            Self::Head => 2.0,
            Self::Torso => 1.0,
            Self::Arms => 0.7,
            Self::Legs => 0.8,
        }
    }

    pub fn stagger_chance(&self) -> f32 {
        match self {
            Self::Head => 0.8,
            Self::Torso => 0.3,
            Self::Arms => 0.1,
            Self::Legs => 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StaggerState {
    None,
    Stagger { remaining: f32 },
    Knockdown { remaining: f32 },
}

impl StaggerState {
    pub fn is_incapacitated(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn tick(&mut self, dt: f32) {
        match self {
            Self::Stagger { remaining } => {
                *remaining -= dt;
                if *remaining <= 0.0 { *self = Self::None; }
            }
            Self::Knockdown { remaining } => {
                *remaining -= dt;
                if *remaining <= 0.0 { *self = Self::None; }
            }
            Self::None => {}
        }
    }
}

/// Resolve combat between attacker and defender.
/// 
/// Uses helper methods instead of direct storage access.
pub fn resolve_combat(ecs: &mut Ecs, events: &mut EventBus, attacker: Entity, defender: Entity) {
    resolve_group_combat(ecs, events, &[attacker], defender);
}

pub fn resolve_group_combat(ecs: &mut Ecs, events: &mut EventBus, attackers: &[Entity], defender: Entity) {
    let mut rng = rand::thread_rng();
    let tick = ecs.tick;

    for &a in attackers {
        witness::on_attacked(ecs, defender, a, tick);
    }

    let atk_power: f32 = attackers.iter().map(|&a| {
        let bp = ecs.get_kind(a).map_or(1.0, base_power);
        let hp = ecs.get_needs(a).map_or(1.0, |p| p.health);
        let energy = ecs.get_needs(a).map_or(0.5, |p| p.energy);
        bp * hp * (0.7 + energy * 0.3)
    }).sum();

    let def_kind = ecs.get_kind(defender).cloned();
    let def_bp = def_kind.as_ref().map_or(1.0, base_power);
    let def_hp = ecs.get_needs(defender).map_or(1.0, |p| p.health);
    let def_energy = ecs.get_needs(defender).map_or(0.5, |p| p.energy);
    let def_total = def_bp * def_hp * (0.7 + def_energy * 0.3) * rng.gen_range(0.7..1.3);
    let atk_total = atk_power * rng.gen_range(0.7..1.3);

    let group_size = attackers.len() as f32;
    let group_bonus = 1.0 + (group_size - 1.0) * 0.3;
    // Predator bonus: species higher on food chain deal more damage
    let predator_bonus = match (attackers.first().and_then(|&a| ecs.get_kind(a)), &def_kind) {
        (Some(EntityKind::Monster(atk_sp)), Some(EntityKind::Monster(def_sp)))
            if food_chain::is_predator_of(*atk_sp, *def_sp) => 1.25,
        _ => 1.0,
    };
    let effective_atk = atk_total * group_bonus * predator_bonus;

    let hit_loc = HitLocation::random(&mut rng);
    let loc_mult = hit_loc.damage_multiplier();
    let dmg_to_defender = (effective_atk * 0.12 * loc_mult).min(0.8);
    let dmg_per_attacker = (def_total * 0.08 / group_size).min(0.25);

    if let Some(pn) = ecs.get_needs_mut(defender) {
        pn.health = (pn.health - dmg_to_defender).max(0.0);
        pn.fear = (pn.fear + 0.2 * group_size.min(3.0)).min(1.0);
        pn.energy = (pn.energy - 0.03).max(0.0);
    }

    events.emit(CombatHit {
        entity: defender,
        hit_zone: match hit_loc {
            HitLocation::Head => 0,
            HitLocation::Torso => 1,
            HitLocation::Arms => 2,
            HitLocation::Legs => 3,
        },
        damage: dmg_to_defender,
    });

    if hit_loc.stagger_chance() > rng.gen_range(0.0..1.0) {
        let stagger = if matches!(hit_loc, HitLocation::Head) && dmg_to_defender > 0.3 {
            StaggerState::Knockdown { remaining: 2.0 }
        } else {
            StaggerState::Stagger { remaining: 0.5 }
        };
        let _ = stagger;
    }
    for &a in attackers {
        if let Some(pn) = ecs.get_needs_mut(a) {
            pn.health = (pn.health - dmg_per_attacker).max(0.0);
            pn.energy = (pn.energy - 0.04).max(0.0);
        }
        events.emit(CombatHit {
            entity: a,
            hit_zone: 1, // Torso for counter-damage
            damage: dmg_per_attacker,
        });
    }

    let target_dead = ecs.get_needs(defender).map_or(false, |p| p.health <= 0.0);

    if target_dead {
        let food = def_kind.as_ref().map_or(0.3, food_value);
        let loot = def_bp * 3.0;
        let share_food = food / group_size;
        let share_loot = loot / group_size;

        for &a in attackers {
            if let Some(pn) = ecs.get_needs_mut(a) {
                pn.hunger = (pn.hunger - share_food).max(0.0);
                pn.energy = (pn.energy + share_food * 0.3).min(1.0);
                pn.ambitions = (pn.ambitions + 0.05).min(1.0);
            }
            if let Some(econ) = ecs.get_npc_economy_mut(a) { econ.money += share_loot; }
            if let Some(sn) = ecs.get_social_needs_mut(a) {
                sn.reputation = (sn.reputation + 0.1).min(1.0);
            }
            if let Some(eco) = ecs.get_ecosystem_needs_mut(a) {
                eco.food_chain_position = (eco.food_chain_position + 0.05).min(1.0);
                eco.hunting = (eco.hunting - 0.3).max(0.0);
            }
        }

        let def_name = ecs.get_name(defender).map(|n| n.0.clone()).unwrap_or_default();
        if attackers.len() > 1 {
            let names: Vec<String> = attackers.iter()
                .filter_map(|&a| ecs.get_name(a).map(|n| n.0.clone())).collect();
            println!("    [GANG KILL] {} ({}) killed {}", names.join("+"), attackers.len(), def_name);
            witness::on_group_kill(ecs, attackers, defender, tick);
        } else {
            let atk_name = attackers.first().and_then(|&a| ecs.get_name(a)).map(|n| n.0.clone()).unwrap_or_default();
            println!("    [KILL] {} killed {}", atk_name, def_name);
            if let Some(&killer) = attackers.first() {
                witness::on_kill(ecs, killer, defender, tick);
            }
        }
        events.emit(EntityDied { entity: defender, killer: attackers.first().copied() });
    } else {
        for &a in attackers {
            witness::on_hunt_failed(ecs, a, def_kind.as_ref(), attackers.len() > 1);
        }
    }

    for &a in attackers {
        let attacker_dead = ecs.get_needs(a).map_or(false, |p| p.health <= 0.0);
        if attacker_dead {
            let atk_kind = ecs.get_kind(a).cloned();
            let food = atk_kind.as_ref().map_or(0.3, food_value);
            if let Some(pn) = ecs.get_needs_mut(defender) { pn.hunger = (pn.hunger - food).max(0.0); }
            let atk_name = ecs.get_name(a).map(|n| n.0.clone()).unwrap_or_default();
            let def_name = ecs.get_name(defender).map(|n| n.0.clone()).unwrap_or_default();
            println!("    [KILL] {} died attacking {}", atk_name, def_name);
            witness::on_kill(ecs, defender, a, tick);
            events.emit(EntityDied { entity: a, killer: Some(defender) });
        }
    }

    if let Some(loc) = ecs.get_transform(defender).map(|t| (t.cell_x, t.cell_y)) {
        if let Some(&a) = attackers.first() {
            witness::communicate_danger(ecs, defender, 200.0, loc);
            let _ = a;
        }
    }
}
