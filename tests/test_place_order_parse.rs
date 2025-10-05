use betfair_rs::dto::PlaceOrdersResponse;

#[test]
fn test_deserialize_place_orders_response() {
    let json = r#"{"customerRef":"rustler_1759619942","status":"SUCCESS","marketId":"1.248324306","instructionReports":[{"status":"SUCCESS","instruction":{"selectionId":56343,"limitOrder":{"size":2.0,"price":1000.0,"persistenceType":"PERSIST"},"orderType":"LIMIT","side":"BACK"},"betId":"404246128956","placedDate":"2025-10-04T23:19:02.000Z","averagePriceMatched":0.0,"sizeMatched":0.0,"orderStatus":"EXECUTABLE"}]}"#;
    
    let result: Result<PlaceOrdersResponse, _> = serde_json::from_str(json);
    
    match &result {
        Ok(resp) => println!("Success: {:#?}", resp),
        Err(e) => {
            println!("Error: {}", e);
            println!("\nDetailed error:");
            println!("  Line: {}", e.line());
            println!("  Column: {}", e.column());
        }
    }
    
    result.expect("Failed to parse PlaceOrdersResponse");
}
