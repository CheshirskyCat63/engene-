use crate::core::ecs::{Ecs, Entity};

/// Result of a trade attempt.
#[derive(Debug, Clone)]
pub enum TradeResult {
    Success { player_paid: f32, npc_received: f32 },
    Failed(String),
}

pub fn attempt_trade(ecs: &mut Ecs, buyer: Entity, seller: Entity, price: f32) -> bool {
    let buyer_money = ecs.get_npc_economy(buyer).map_or(0.0, |e| e.money);
    if buyer_money < price {
        return false;
    }

    if let Some(econ) = ecs.get_npc_economy_mut(buyer) {
        econ.money -= price;
    }
    if let Some(econ) = ecs.get_npc_economy_mut(seller) {
        econ.money += price;
    }

    true
}
