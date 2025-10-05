use betfair_rs::dto::order::PlaceOrdersResponse;

#[test]
fn test_exact_production_response() {
    // This is the exact response body from production logs
    let json = r#"{"customerRef":"rustler_1759652261","status":"SUCCESS","marketId":"1.248324306","instructionReports":[{"status":"SUCCESS","instruction":{"selectionId":56343,"limitOrder":{"size":2.0,"price":1000.0,"persistenceType":"PERSIST"},"orderType":"LIMIT","side":"BACK"},"betId":"404282003422","placedDate":"2025-10-05T08:17:41.000Z","averagePriceMatched":0.0,"sizeMatched":0.0,"orderStatus":"EXECUTABLE"}]}"#;

    let result: Result<PlaceOrdersResponse, _> = serde_json::from_str(json);
    match result {
        Ok(response) => {
            println!("✅ Successfully parsed production response");
            println!("Response: {:#?}", response);
            assert_eq!(response.status, "SUCCESS");
            assert_eq!(response.market_id, "1.248324306");
        }
        Err(e) => {
            println!("❌ Failed to deserialize production response");
            println!("Error: {}", e);
            println!("Full error: {:?}", e);
            panic!("Deserialization failed: {}", e);
        }
    }
}
