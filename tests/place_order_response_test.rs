use betfair_rs::dto::order::PlaceOrdersResponse;

#[test]
fn test_deserialize_place_orders_response() {
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
            println!("Success! Parsed response: {:?}", response);
            assert_eq!(response.status, "SUCCESS");
            assert_eq!(response.market_id, "1.248324306");
        }
        Err(e) => {
            panic!("Failed to deserialize: {}", e);
        }
    }
}
