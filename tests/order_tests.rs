use betfair_rs::order::{Order, OrderSide, OrderType, PersistenceType, TimeInForceType};

#[test]
fn test_order_creation() {
    let order = Order::new(
        "1.123456789".to_string(),
        12345,
        OrderSide::Back,
        2.0,
        10.0,
        None,
    );

    assert_eq!(order.market_id, "1.123456789");
    assert_eq!(order.selection_id, 12345);
    assert_eq!(order.side, OrderSide::Back);
    assert_eq!(order.order_type, OrderType::Limit);
    assert_eq!(order.handicap, 0.0);

    let limit_order = order.limit_order.unwrap();
    assert_eq!(limit_order.size, 10.0);
    assert_eq!(limit_order.price, 2.0);
    assert_eq!(limit_order.persistence_type, Some(PersistenceType::Persist));
}

#[test]
fn test_order_to_place_instruction() {
    let order = Order::new(
        "1.123456789".to_string(),
        12345,
        OrderSide::Lay,
        3.0,
        5.0,
        None,
    );

    let instruction = order.to_place_instruction();

    assert_eq!(instruction.selection_id, 12345);
    assert_eq!(instruction.handicap, 0.0);
    assert_eq!(instruction.side, OrderSide::Lay);
    assert_eq!(instruction.order_type, Some(OrderType::Limit));

    let limit_order = instruction.limit_order.unwrap();
    assert_eq!(limit_order.size, 5.0);
    assert_eq!(limit_order.price, 3.0);
    assert_eq!(limit_order.persistence_type, Some(PersistenceType::Persist));
}

#[test]
fn test_order_side_serialization() {
    let back_order = Order::new(
        "1.123456789".to_string(),
        12345,
        OrderSide::Back,
        2.0,
        10.0,
        None,
    );

    let lay_order = Order::new(
        "1.123456789".to_string(),
        12345,
        OrderSide::Lay,
        2.0,
        10.0,
        None,
    );

    assert_ne!(back_order.side, lay_order.side);
}

#[test]
fn test_order_type_serialization() {
    let limit_order = Order::new(
        "1.123456789".to_string(),
        12345,
        OrderSide::Back,
        2.0,
        10.0,
        None,
    );

    assert_eq!(limit_order.order_type, OrderType::Limit);
}
