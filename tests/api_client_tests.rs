#[cfg(test)]
mod tests {
    use betfair_rs::dto::{
        LimitOrder, MarketFilter, OrderType, PersistenceType, PlaceInstruction, PlaceOrdersRequest,
        Side,
    };
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_market_filter_creation() {
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
    }
}
