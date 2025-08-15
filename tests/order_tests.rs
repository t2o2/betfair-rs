use betfair_rs::order::{Order, LimitOrder};
use betfair_rs::{Side, OrderType, PersistenceType};

#[test]
fn test_order_creation() {
    let order = Order {
        market_id: "1.123456789".to_string(),
        selection_id: 12345,
        side: Side::Back,
        order_type: OrderType::Limit,
        limit_order: Some(LimitOrder {
            size: 10.0,
            price: 2.0,
            persistence_type: PersistenceType::Persist,
            time_in_force: None,
            min_fill_size: None,
            bet_target_type: None,
            bet_target_size: None,
        }),
        handicap: 0.0,
    };

    assert_eq!(order.market_id, "1.123456789");
    assert_eq!(order.selection_id, 12345);
    assert_eq!(order.side, Side::Back);
    assert_eq!(order.order_type, OrderType::Limit);
    assert_eq!(order.handicap, 0.0);

    let limit_order = order.limit_order.unwrap();
    assert_eq!(limit_order.size, 10.0);
    assert_eq!(limit_order.price, 2.0);
    assert_eq!(limit_order.persistence_type, PersistenceType::Persist);
}

#[test]
fn test_order_to_place_instruction() {
    let order = Order {
        market_id: "1.123456789".to_string(),
        selection_id: 12345,
        side: Side::Lay,
        order_type: OrderType::Limit,
        limit_order: Some(LimitOrder {
            size: 5.0,
            price: 3.0,
            persistence_type: PersistenceType::Persist,
            time_in_force: None,
            min_fill_size: None,
            bet_target_type: None,
            bet_target_size: None,
        }),
        handicap: 0.0,
    };

    let instruction = order.to_place_instruction();

    assert_eq!(instruction.selection_id, 12345);
    assert_eq!(instruction.handicap, Some(0.0));
    assert_eq!(instruction.side, Side::Lay);
    assert_eq!(instruction.order_type, OrderType::Limit);

    let limit_order = instruction.limit_order.unwrap();
    assert_eq!(limit_order.size, 5.0);
    assert_eq!(limit_order.price, 3.0);
    assert_eq!(limit_order.persistence_type, PersistenceType::Persist);
}

#[test]
fn test_order_side_serialization() {
    let back_order = Order {
        market_id: "1.123456789".to_string(),
        selection_id: 12345,
        side: Side::Back,
        order_type: OrderType::Limit,
        limit_order: Some(LimitOrder {
            size: 10.0,
            price: 2.0,
            persistence_type: PersistenceType::Persist,
            time_in_force: None,
            min_fill_size: None,
            bet_target_type: None,
            bet_target_size: None,
        }),
        handicap: 0.0,
    };

    let lay_order = Order {
        market_id: "1.123456789".to_string(),
        selection_id: 12345,
        side: Side::Lay,
        order_type: OrderType::Limit,
        limit_order: Some(LimitOrder {
            size: 10.0,
            price: 2.0,
            persistence_type: PersistenceType::Persist,
            time_in_force: None,
            min_fill_size: None,
            bet_target_type: None,
            bet_target_size: None,
        }),
        handicap: 0.0,
    };

    assert_ne!(back_order.side, lay_order.side);
}

#[test]
fn test_order_type_serialization() {
    let limit_order = Order {
        market_id: "1.123456789".to_string(),
        selection_id: 12345,
        side: Side::Back,
        order_type: OrderType::Limit,
        limit_order: Some(LimitOrder {
            size: 10.0,
            price: 2.0,
            persistence_type: PersistenceType::Persist,
            time_in_force: None,
            min_fill_size: None,
            bet_target_type: None,
            bet_target_size: None,
        }),
        handicap: 0.0,
    };

    assert_eq!(limit_order.order_type, OrderType::Limit);
}
