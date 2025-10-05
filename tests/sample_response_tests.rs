use betfair_rs::dto::order::PlaceOrdersResponse;

#[test]
fn test_place_order_success_limit() {
    let json = r#"{
    "customerRef": "rustler_1759625902",
    "status": "SUCCESS",
    "marketId": "1.248324306",
    "instructionReports": [
        {
            "status": "SUCCESS",
            "instruction": {
                "selectionId": 56343,
                "limitOrder": {
                    "size": 2.0,
                    "price": 1000.0,
                    "persistenceType": "PERSIST"
                },
                "orderType": "LIMIT",
                "side": "BACK"
            },
            "betId": "404254044217",
            "placedDate": "2025-10-05T00:58:23.000Z",
            "averagePriceMatched": 0.0,
            "sizeMatched": 0.0,
            "orderStatus": "EXECUTABLE"
        }
    ]
}"#;

    let result: Result<PlaceOrdersResponse, _> = serde_json::from_str(json);
    match result {
        Ok(response) => {
            println!("✅ Successfully parsed limit order response");
            assert_eq!(response.status, "SUCCESS");
            assert_eq!(response.market_id, "1.248324306");
            let reports = response.instruction_reports.expect("Should have instruction reports");
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].bet_id, Some("404254044217".to_string()));
        }
        Err(e) => {
            panic!("❌ Failed to deserialize: {}", e);
        }
    }
}

#[test]
fn test_place_order_failure_duplicate() {
    let json = r#"{
    "customerRef": "rustler_1759625903",
    "status": "FAILURE",
    "errorCode": "DUPLICATE_TRANSACTION",
    "marketId": "1.248324306",
    "instructionReports": [
        {
            "status": "FAILURE",
            "errorCode": "ERROR_IN_ORDER",
            "instruction": {
                "selectionId": 56343,
                "limitOrder": {
                    "size": 2.0,
                    "price": 1000.0,
                    "persistenceType": "PERSIST"
                },
                "orderType": "LIMIT",
                "side": "BACK"
            }
        }
    ]
}"#;

    let result: Result<PlaceOrdersResponse, _> = serde_json::from_str(json);
    match result {
        Ok(response) => {
            println!("✅ Successfully parsed duplicate order failure response");
            assert_eq!(response.status, "FAILURE");
            assert_eq!(response.error_code, Some("DUPLICATE_TRANSACTION".to_string()));
            let reports = response.instruction_reports.expect("Should have instruction reports");
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].status, "FAILURE");
            assert_eq!(reports[0].error_code, Some("ERROR_IN_ORDER".to_string()));
        }
        Err(e) => {
            panic!("❌ Failed to deserialize: {}", e);
        }
    }
}
