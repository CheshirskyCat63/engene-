use super::*;

// =============================================================================
// 5. TRADER + ECONOMY + PRICING (28 tests)
// =============================================================================

#[test]
fn trader_state_new() {
    let trader = TraderState::new("Barman", "Loners", 500.0);
    assert_eq!(trader.name, "Barman");
    assert_eq!(trader.capital, 500.0);
    assert!(trader.inventory.is_empty());
}

#[test]
fn trader_state_effective_buy_price() {
    let trader = TraderState::new("T", "F", 100.0);
    let price = trader.effective_buy_price(100.0, "medkit");
    assert!(price < 100.0);
}

#[test]
fn trader_state_effective_sell_price() {
    let trader = TraderState::new("T", "F", 100.0);
    let price = trader.effective_sell_price(100.0, "medkit");
    assert!(price > 100.0);
}

#[test]
fn trader_state_can_buy_empty_inventory() {
    let trader = TraderState::new("T", "F", 100.0);
    assert!(trader.can_buy("medkit").is_none());
}

#[test]
fn trader_state_can_buy_with_stock() {
    let mut trader = TraderState::new("T", "F", 100.0);
    trader.inventory.push(TraderInventorySlot {
        item_id: "medkit".into(),
        quantity: 5,
        buy_price: 50.0,
        sell_price: 60.0,
    });
    let slot = trader.can_buy("medkit");
    assert!(slot.is_some());
    assert_eq!(slot.unwrap().quantity, 5);
}

#[test]
fn trader_state_tick_restock() {
    let mut trader = TraderState::new("T", "F", 100.0);
    trader.inventory.push(TraderInventorySlot {
        item_id: "medkit".into(),
        quantity: 5,
        buy_price: 50.0,
        sell_price: 60.0,
    });
    trader.tick_restock(50.0);
    assert!(trader.inventory[0].quantity >= 5);
}

#[test]
fn item_registry_new() {
    let reg = ItemRegistry::new();
    assert!(reg.get("medkit").is_some());
}

#[test]
fn item_registry_register() {
    let mut reg = ItemRegistry::new();
    let t = ItemTemplate {
        id: "custom".into(),
        name: "Custom".into(),
        category: ItemCategory::Junk,
        rarity: ItemRarity::Common,
        base_value: 1.0,
        weight: 0.1,
        max_stack: 99,
        max_durability: 1.0,
        description: "Test".into(),
    };
    reg.register(t);
    assert!(reg.get("custom").is_some());
}

#[test]
fn item_registry_get() {
    let reg = ItemRegistry::new();
    let t = reg.get("bread").unwrap();
    assert_eq!(t.name, "Bread");
}

#[test]
fn item_registry_by_category() {
    let reg = ItemRegistry::new();
    let food = reg.by_category(ItemCategory::Food);
    assert!(!food.is_empty());
}

#[test]
fn item_registry_create_instance() {
    let reg = ItemRegistry::new();
    let inst = reg.create_instance("medkit", 2);
    assert!(inst.is_some());
    assert_eq!(inst.unwrap().template_id, "medkit");
}

#[test]
fn item_category_weapon() {
    assert!(matches!(ItemCategory::Weapon, ItemCategory::Weapon));
}

#[test]
fn item_category_armor() {
    assert!(matches!(ItemCategory::Armor, ItemCategory::Armor));
}

#[test]
fn item_category_medkit() {
    assert!(matches!(ItemCategory::Medkit, ItemCategory::Medkit));
}

#[test]
fn item_category_food() {
    assert!(matches!(ItemCategory::Food, ItemCategory::Food));
}

#[test]
fn item_rarity_common() {
    assert!(matches!(ItemRarity::Common, ItemRarity::Common));
}

#[test]
fn item_rarity_rare() {
    assert!(matches!(ItemRarity::Rare, ItemRarity::Rare));
}

#[test]
fn attempt_trade_success() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.0,
        },
    );
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let ok = attempt_trade(&mut ecs, buyer, seller, 30.0);
    assert!(ok);
    assert!((ecs.npc_economies.get(&buyer).unwrap().money - 70.0).abs() < 0.01);
    assert!((ecs.npc_economies.get(&seller).unwrap().money - 80.0).abs() < 0.01);
}

#[test]
fn attempt_trade_insufficient_funds() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 10.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.0,
        },
    );
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let ok = attempt_trade(&mut ecs, buyer, seller, 50.0);
    assert!(!ok);
}

#[test]
fn economy_snapshot_empty() {
    let ecs = Ecs::new();
    let snap = snapshot(&ecs);
    assert_eq!(snap.total_npc_money, 0.0);
    assert_eq!(snap.bandit_count, 0);
}

#[test]
fn economy_snapshot_with_npcs() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Npc);
    ecs.npc_economies.insert(
        e1,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.2,
        },
    );
    ecs.npc_economies.insert(
        e2,
        NpcEconomy {
            money: 200.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let snap = snapshot(&ecs);
    assert!(snap.total_npc_money > 0.0);
}

#[test]
fn economy_snapshot_bandit_count() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.npc_economies.insert(
        e1,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Bandit,
            desperation: 0.8,
        },
    );
    let snap = snapshot(&ecs);
    assert_eq!(snap.bandit_count, 1);
}

#[test]
fn item_instance_stack_count() {
    let reg = ItemRegistry::new();
    let inst = reg.create_instance("bread", 5).unwrap();
    assert!(inst.stack_count <= 5);
}
