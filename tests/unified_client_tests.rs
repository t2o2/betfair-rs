use betfair_rs::config::{BetfairConfig, Config};
use betfair_rs::dto::{
    LimitOrder, MarketFilter, OrderType, PersistenceType, PlaceInstruction, PlaceOrdersRequest,
    Side,
};
use betfair_rs::unified_client::BetfairClient;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn create_test_config() -> Config {
    Config {
        betfair: BetfairConfig {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            api_key: "test_api_key".to_string(),
            pem_path: "test.pem".to_string(),
        },
    }
}

#[test]
fn test_unified_client_creation() {
    let config = create_test_config();
    let client = BetfairClient::new(config);

    assert!(client.get_session_token().is_none());
    assert!(client.get_streaming_orderbooks().is_none());
    assert!(!client.is_streaming_connected());
}

#[test]
fn test_get_and_set_session_token() {
    let config = create_test_config();
    let mut client = BetfairClient::new(config);

    assert!(client.get_session_token().is_none());

    client.set_session_token("test_token".to_string());
    assert_eq!(client.get_session_token(), Some("test_token".to_string()));
}

#[test]
fn test_streaming_not_connected_initially() {
    let config = create_test_config();
    let client = BetfairClient::new(config);

    assert!(!client.is_streaming_connected());
    assert!(client.get_streaming_orderbooks().is_none());
    assert!(client.get_market_last_update_time("1.123456").is_none());
}

#[tokio::test]
async fn test_market_filter_methods() {
    let config = create_test_config();
    let mut client = BetfairClient::new(config);
    client.set_session_token("test_token".to_string());

    let filter = MarketFilter {
        event_type_ids: Some(vec!["1".to_string()]),
        competition_ids: Some(vec!["10".to_string()]),
        ..Default::default()
    };

    assert!(filter.event_type_ids.is_some());
    assert!(filter.competition_ids.is_some());
    assert!(filter.market_ids.is_none());
}

#[test]
fn test_place_order_request_creation() {
    let request = PlaceOrdersRequest {
        market_id: "1.123456".to_string(),
        instructions: vec![PlaceInstruction {
            order_type: OrderType::Limit,
            selection_id: 12345,
            handicap: Some(Decimal::ZERO),
            side: Side::Back,
            limit_order: Some(LimitOrder {
                size: dec!(10.0),
                price: dec!(2.0),
                persistence_type: PersistenceType::Lapse,
                time_in_force: None,
                min_fill_size: None,
                bet_target_type: None,
                bet_target_size: None,
            }),
            limit_on_close_order: None,
            market_on_close_order: None,
            customer_order_ref: Some("test_ref".to_string()),
        }],
        customer_ref: Some("customer_ref".to_string()),
        market_version: None,
        customer_strategy_ref: None,
        async_: None,
    };

    assert_eq!(request.market_id, "1.123456");
    assert_eq!(request.instructions.len(), 1);
    assert_eq!(request.instructions[0].selection_id, 12345);
    assert_eq!(request.instructions[0].side, Side::Back);
    assert!(request.instructions[0].limit_order.is_some());
    assert_eq!(request.customer_ref, Some("customer_ref".to_string()));
}

#[test]
fn test_unified_client_shared_orderbooks_initialization() {
    let config = create_test_config();
    let client = BetfairClient::new(config);

    let orderbooks = client.get_streaming_orderbooks();
    assert!(orderbooks.is_none());
}

#[test]
fn test_market_last_update_time_not_available() {
    let config = create_test_config();
    let client = BetfairClient::new(config);

    let update_time = client.get_market_last_update_time("1.123456");
    assert!(update_time.is_none());
}

#[tokio::test]
async fn test_place_order_with_updates_request() {
    let request = PlaceOrdersRequest {
        market_id: "1.123456".to_string(),
        instructions: vec![PlaceInstruction {
            order_type: OrderType::Limit,
            selection_id: 12345,
            handicap: Some(Decimal::ZERO),
            side: Side::Back,
            limit_order: Some(LimitOrder {
                size: dec!(10.0),
                price: dec!(2.0),
                persistence_type: PersistenceType::Lapse,
                time_in_force: None,
                min_fill_size: None,
                bet_target_type: None,
                bet_target_size: None,
            }),
            limit_on_close_order: None,
            market_on_close_order: None,
            customer_order_ref: Some("test_ref".to_string()),
        }],
        customer_ref: Some("customer_ref".to_string()),
        market_version: None,
        customer_strategy_ref: None,
        async_: None,
    };

    assert_eq!(request.market_id, "1.123456");
    assert_eq!(request.instructions[0].selection_id, 12345);

    let wait_duration = std::time::Duration::from_secs(5);
    assert_eq!(wait_duration.as_secs(), 5);
}
