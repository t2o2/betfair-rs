use betfair_rs::dto::*;
use serde_json::{from_value, json, to_value};

#[test]
fn test_market_filter_serialization() {
    let filter = MarketFilter {
        event_type_ids: Some(vec!["1".to_string(), "2".to_string()]),
        competition_ids: Some(vec!["10".to_string()]),
        market_ids: Some(vec!["1.123456".to_string()]),
        in_play_only: Some(true),
        turn_in_play_enabled: Some(false),
        market_betting_types: Some(vec!["ODDS".to_string()]),
        venues: Some(vec!["Stadium".to_string()]),
        bsp_only: Some(false),
        text_query: Some("test".to_string()),
        ..Default::default()
    };

    let json = to_value(&filter).unwrap();
    assert_eq!(json["eventTypeIds"], json!(["1", "2"]));
    assert_eq!(json["competitionIds"], json!(["10"]));
    assert_eq!(json["marketIds"], json!(["1.123456"]));
    assert_eq!(json["inPlayOnly"], json!(true));
    assert_eq!(json["turnInPlayEnabled"], json!(false));
    assert_eq!(json["marketBettingTypes"], json!(["ODDS"]));
    assert_eq!(json["venues"], json!(["Stadium"]));
    assert_eq!(json["bspOnly"], json!(false));
    assert_eq!(json["textQuery"], json!("test"));
}

#[test]
fn test_market_filter_deserialization() {
    let json = json!({
        "eventTypeIds": ["1", "2"],
        "competitionIds": ["10"],
        "marketIds": ["1.123456"],
        "inPlayOnly": true,
        "turnInPlayEnabled": false,
        "marketBettingTypes": ["ODDS"],
        "venues": ["Stadium"],
        "bspOnly": false,
        "textQuery": "test"
    });

    let filter: MarketFilter = from_value(json).unwrap();
    assert_eq!(
        filter.event_type_ids,
        Some(vec!["1".to_string(), "2".to_string()])
    );
    assert_eq!(filter.competition_ids, Some(vec!["10".to_string()]));
    assert_eq!(filter.market_ids, Some(vec!["1.123456".to_string()]));
    assert_eq!(filter.in_play_only, Some(true));
    assert_eq!(filter.turn_in_play_enabled, Some(false));
    assert_eq!(filter.market_betting_types, Some(vec!["ODDS".to_string()]));
    assert_eq!(filter.venues, Some(vec!["Stadium".to_string()]));
    assert_eq!(filter.bsp_only, Some(false));
    assert_eq!(filter.text_query, Some("test".to_string()));
}

#[test]
fn test_place_instruction_serialization() {
    let instruction = PlaceInstruction {
        order_type: OrderType::Limit,
        selection_id: 12345,
        handicap: Some(0.0),
        side: Side::Back,
        limit_order: Some(LimitOrder {
            size: 10.0,
            price: 2.0,
            persistence_type: PersistenceType::Lapse,
            time_in_force: None,
            min_fill_size: None,
            bet_target_type: None,
            bet_target_size: None,
        }),
        limit_on_close_order: None,
        market_on_close_order: None,
        customer_order_ref: Some("ref123".to_string()),
    };

    let json = to_value(&instruction).unwrap();
    assert_eq!(json["orderType"], json!("LIMIT"));
    assert_eq!(json["selectionId"], json!(12345));
    assert_eq!(json["handicap"], json!(0.0));
    assert_eq!(json["side"], json!("BACK"));
    assert_eq!(json["limitOrder"]["size"], json!(10.0));
    assert_eq!(json["limitOrder"]["price"], json!(2.0));
    assert_eq!(json["limitOrder"]["persistenceType"], json!("LAPSE"));
    assert_eq!(json["customerOrderRef"], json!("ref123"));
}

#[test]
fn test_place_instruction_deserialization() {
    let json = json!({
        "orderType": "LIMIT",
        "selectionId": 12345,
        "handicap": 0.0,
        "side": "BACK",
        "limitOrder": {
            "size": 10.0,
            "price": 2.0,
            "persistenceType": "LAPSE"
        },
        "customerOrderRef": "ref123"
    });

    let instruction: PlaceInstruction = from_value(json).unwrap();
    assert_eq!(instruction.order_type, OrderType::Limit);
    assert_eq!(instruction.selection_id, 12345);
    assert_eq!(instruction.handicap, Some(0.0));
    assert_eq!(instruction.side, Side::Back);
    assert!(instruction.limit_order.is_some());
    let limit_order = instruction.limit_order.unwrap();
    assert_eq!(limit_order.size, 10.0);
    assert_eq!(limit_order.price, 2.0);
    assert_eq!(limit_order.persistence_type, PersistenceType::Lapse);
    assert_eq!(instruction.customer_order_ref, Some("ref123".to_string()));
}

#[test]
fn test_runner_serialization() {
    let runner = Runner {
        selection_id: 12345,
        handicap: 0.0,
        status: RunnerStatus::Active,
        adjustment_factor: Some(1.0),
        last_price_traded: Some(2.5),
        total_matched: Some(1000.0),
        removal_date: None,
        sp: None,
        ex: Some(ExchangePrices {
            available_to_back: Some(vec![
                PriceSize {
                    price: 2.4,
                    size: 100.0,
                },
                PriceSize {
                    price: 2.3,
                    size: 200.0,
                },
            ]),
            available_to_lay: Some(vec![
                PriceSize {
                    price: 2.5,
                    size: 150.0,
                },
                PriceSize {
                    price: 2.6,
                    size: 250.0,
                },
            ]),
            traded_volume: None,
        }),
        orders: None,
        matches: None,
        matched_preplay: None,
    };

    let json = to_value(&runner).unwrap();
    assert_eq!(json["selectionId"], json!(12345));
    assert_eq!(json["handicap"], json!(0.0));
    assert_eq!(json["status"], json!("ACTIVE"));
    assert_eq!(json["adjustmentFactor"], json!(1.0));
    assert_eq!(json["lastPriceTraded"], json!(2.5));
    assert_eq!(json["totalMatched"], json!(1000.0));
    assert!(json["ex"]["availableToBack"].is_array());
    assert!(json["ex"]["availableToLay"].is_array());
}

#[test]
fn test_runner_deserialization() {
    let json = json!({
        "selectionId": 12345,
        "handicap": 0.0,
        "status": "ACTIVE",
        "adjustmentFactor": 1.0,
        "lastPriceTraded": 2.5,
        "totalMatched": 1000.0,
        "ex": {
            "availableToBack": [
                {"price": 2.4, "size": 100.0},
                {"price": 2.3, "size": 200.0}
            ],
            "availableToLay": [
                {"price": 2.5, "size": 150.0},
                {"price": 2.6, "size": 250.0}
            ]
        }
    });

    let runner: Runner = from_value(json).unwrap();
    assert_eq!(runner.selection_id, 12345);
    assert_eq!(runner.handicap, 0.0);
    assert!(matches!(runner.status, RunnerStatus::Active));
    assert_eq!(runner.adjustment_factor, Some(1.0));
    assert_eq!(runner.last_price_traded, Some(2.5));
    assert_eq!(runner.total_matched, Some(1000.0));
    assert!(runner.ex.is_some());
    let ex = runner.ex.unwrap();
    assert_eq!(ex.available_to_back.unwrap().len(), 2);
    assert_eq!(ex.available_to_lay.unwrap().len(), 2);
}

#[test]
fn test_market_book_serialization() {
    let market_book = MarketBook {
        market_id: "1.123456".to_string(),
        is_market_data_delayed: false,
        status: Some(betfair_rs::dto::MarketStatus::Open),
        bet_delay: Some(0),
        bsp_reconciled: Some(false),
        complete: Some(true),
        inplay: Some(false),
        number_of_winners: Some(1),
        number_of_runners: Some(3),
        number_of_active_runners: Some(3),
        last_match_time: None,
        total_matched: Some(1000.0),
        total_available: Some(5000.0),
        cross_matching: Some(true),
        runners_voidable: Some(false),
        version: Some(123456789),
        runners: Some(vec![]),
        key_line_description: None,
    };

    let json = to_value(&market_book).unwrap();
    assert_eq!(json["marketId"], json!("1.123456"));
    assert_eq!(json["isMarketDataDelayed"], json!(false));
    assert_eq!(json["status"], json!("OPEN"));
    assert_eq!(json["betDelay"], json!(0));
    assert_eq!(json["bspReconciled"], json!(false));
    assert_eq!(json["complete"], json!(true));
    assert_eq!(json["inplay"], json!(false));
    assert_eq!(json["numberOfWinners"], json!(1));
    assert_eq!(json["numberOfRunners"], json!(3));
    assert_eq!(json["numberOfActiveRunners"], json!(3));
    assert_eq!(json["totalMatched"], json!(1000.0));
    assert_eq!(json["totalAvailable"], json!(5000.0));
    assert_eq!(json["crossMatching"], json!(true));
    assert_eq!(json["runnersVoidable"], json!(false));
    assert_eq!(json["version"], json!(123456789));
}

#[test]
fn test_market_book_deserialization() {
    let json = json!({
        "marketId": "1.123456",
        "isMarketDataDelayed": false,
        "status": "OPEN",
        "betDelay": 0,
        "bspReconciled": false,
        "complete": true,
        "inplay": false,
        "numberOfWinners": 1,
        "numberOfRunners": 3,
        "numberOfActiveRunners": 3,
        "totalMatched": 1000.0,
        "totalAvailable": 5000.0,
        "crossMatching": true,
        "runnersVoidable": false,
        "version": 123456789,
        "runners": []
    });

    let market_book: MarketBook = from_value(json).unwrap();
    assert_eq!(market_book.market_id, "1.123456");
    assert_eq!(market_book.is_market_data_delayed, false);
    assert_eq!(
        market_book.status,
        Some(betfair_rs::dto::MarketStatus::Open)
    );
    assert_eq!(market_book.bet_delay, Some(0));
    assert_eq!(market_book.bsp_reconciled, Some(false));
    assert_eq!(market_book.complete, Some(true));
    assert_eq!(market_book.inplay, Some(false));
    assert_eq!(market_book.number_of_winners, Some(1));
    assert_eq!(market_book.number_of_runners, Some(3));
    assert_eq!(market_book.number_of_active_runners, Some(3));
    assert_eq!(market_book.total_matched, Some(1000.0));
    assert_eq!(market_book.total_available, Some(5000.0));
    assert_eq!(market_book.cross_matching, Some(true));
    assert_eq!(market_book.runners_voidable, Some(false));
    assert_eq!(market_book.version, Some(123456789));
    assert_eq!(market_book.runners.as_ref().unwrap().len(), 0);
}

#[test]
fn test_current_order_summary_serialization() {
    let order = CurrentOrderSummary {
        bet_id: "123456789".to_string(),
        market_id: "1.123456".to_string(),
        selection_id: 12345,
        handicap: Some(0.0),
        price_size: PriceSize {
            price: 2.0,
            size: 10.0,
        },
        bsp_liability: Some(0.0),
        side: Side::Back,
        status: OrderStatus::Executable,
        persistence_type: PersistenceType::Lapse,
        order_type: OrderType::Limit,
        placed_date: Some("2024-01-01T00:00:00.000Z".to_string()),
        matched_date: Some("2024-01-01T00:01:00.000Z".to_string()),
        average_price_matched: Some(2.0),
        size_matched: Some(10.0),
        size_remaining: Some(0.0),
        size_lapsed: Some(0.0),
        size_cancelled: Some(0.0),
        size_voided: Some(0.0),
        regulator_auth_code: None,
        regulator_code: None,
        customer_order_ref: Some("ref123".to_string()),
        customer_strategy_ref: None,
    };

    let json = to_value(&order).unwrap();
    assert_eq!(json["betId"], json!("123456789"));
    assert_eq!(json["marketId"], json!("1.123456"));
    assert_eq!(json["selectionId"], json!(12345));
    assert_eq!(json["handicap"], json!(0.0));
    assert_eq!(json["priceSize"]["price"], json!(2.0));
    assert_eq!(json["priceSize"]["size"], json!(10.0));
    assert_eq!(json["side"], json!("BACK"));
    assert_eq!(json["status"], json!("EXECUTABLE"));
    assert_eq!(json["persistenceType"], json!("LAPSE"));
    assert_eq!(json["orderType"], json!("LIMIT"));
    assert_eq!(json["customerOrderRef"], json!("ref123"));
}

#[test]
fn test_cancel_instruction_serialization() {
    let instruction = CancelInstruction {
        bet_id: "123456789".to_string(),
        size_reduction: Some(5.0),
    };

    let json = to_value(&instruction).unwrap();
    assert_eq!(json["betId"], json!("123456789"));
    assert_eq!(json["sizeReduction"], json!(5.0));
}

#[test]
fn test_cancel_instruction_deserialization() {
    let json = json!({
        "betId": "123456789",
        "sizeReduction": 5.0
    });

    let instruction: CancelInstruction = from_value(json).unwrap();
    assert_eq!(instruction.bet_id, "123456789");
    assert_eq!(instruction.size_reduction, Some(5.0));
}

#[test]
fn test_account_funds_response_serialization() {
    let funds = AccountFundsResponse {
        available_to_bet_balance: 1000.0,
        exposure: -50.0,
        retained_commission: 5.0,
        exposure_limit: -10000.0,
        discount_rate: 2.0,
        points_balance: 100.0,
        wallet: "UK".to_string(),
    };

    let json = to_value(&funds).unwrap();
    assert_eq!(json["availableToBetBalance"], json!(1000.0));
    assert_eq!(json["exposure"], json!(-50.0));
    assert_eq!(json["retainedCommission"], json!(5.0));
    assert_eq!(json["exposureLimit"], json!(-10000.0));
    assert_eq!(json["discountRate"], json!(2.0));
    assert_eq!(json["pointsBalance"], json!(100.0));
}

#[test]
fn test_event_type_result_deserialization() {
    let json = json!({
        "eventType": {
            "id": "1",
            "name": "Soccer"
        },
        "marketCount": 100
    });

    let result: EventTypeResult = from_value(json).unwrap();
    assert_eq!(result.event_type.id, "1");
    assert_eq!(result.event_type.name, "Soccer");
    assert_eq!(result.market_count, 100);
}
