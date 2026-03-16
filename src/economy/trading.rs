use crate::core::ecs::{Ecs, Entity};

/// Result of a trade attempt.
#[derive(Debug, Clone)]
pub enum TradeResult {
    Success { player_paid: f32, npc_received: f32 },
    Failed(String),
}

pub fn attempt_trade(ecs: &mut Ecs, buyer: Entity, seller: Entity, price: f32) -> bool {
    let buyer_money = ecs.npc_economies.get(&buyer).map_or(0.0, |e| e.money);
    if buyer_money < price {
        return false;
    }

    if let Some(econ) = ecs.npc_economies.get_mut(&buyer) {
        econ.money -= price;
    }
    if let Some(econ) = ecs.npc_economies.get_mut(&seller) {
        econ.money += price;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_result_success() {
        let result = TradeResult::Success { player_paid: 10.0, npc_received: 10.0 };
        match result {
            TradeResult::Success { player_paid, npc_received } => {
                assert_eq!(player_paid, 10.0);
                assert_eq!(npc_received, 10.0);
            }
            TradeResult::Failed(_) => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_trade_result_failed() {
        let result = TradeResult::Failed("Not enough money".to_string());
        match result {
            TradeResult::Failed(msg) => assert_eq!(msg, "Not enough money"),
            TradeResult::Success { .. } => panic!("Expected Failed"),
        }
    }

    #[test]
    fn test_trade_result_clone() {
        let result = TradeResult::Success { player_paid: 5.0, npc_received: 5.0 };
        let cloned = result.clone();
        match cloned {
            TradeResult::Success { player_paid, .. } => assert_eq!(player_paid, 5.0),
            TradeResult::Failed(_) => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_trade_result_debug() {
        let result = TradeResult::Failed("test".to_string());
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("Failed"));
    }
}
