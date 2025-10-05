use betfair_rs::dto::order::CancelOrdersResponse;

#[test]
fn test_cancel_orders_response_with_size_cancelled() {
    // This is the actual response from Betfair with sizeCancelled
    let json = r#"{"customerRef":"rustler_1759653044","status":"SUCCESS","marketId":"1.248324306","instructionReports":[{"status":"SUCCESS","instruction":{"betId":"404282827660"},"sizeCancelled":2.0,"cancelledDate":"2025-10-05T08:30:44.000Z"}]}"#;

    let result: Result<CancelOrdersResponse, _> = serde_json::from_str(json);
    match result {
        Ok(response) => {
            println!("✅ Successfully parsed cancel response with sizeCancelled");
            println!("Response: {:#?}", response);
            assert_eq!(response.status, "SUCCESS");
        }
        Err(e) => {
            println!("❌ Failed to deserialize cancel response");
            println!("Error: {}", e);
            panic!("Deserialization failed: {}", e);
        }
    }
}
