use betfair_rs::msg_model::{LoginResponse, MarketChangeMessage, HeartbeatMessage};
use serde_json::json;

#[test]
fn test_login_response_deserialization() {
    let json = json!({
        "sessionToken": "abc123",
        "loginStatus": "SUCCESS"
    });

    let response: LoginResponse = serde_json::from_value(json).unwrap();
    assert_eq!(response.session_token.unwrap(), "abc123");
    assert_eq!(response.login_status, "SUCCESS");
}

#[test]
fn test_login_response_display() {
    let response = LoginResponse {
        session_token: Some("abc123".to_string()),
        login_status: "SUCCESS".to_string(),
    };

    assert_eq!(response.to_string(), "LoginResponse { status: SUCCESS }");
}

#[test]
fn test_market_change_message_deserialization() {
    let json = json!({
        "op": "mcm",
        "id": 1,
        "clk": "AJctAKk5AJMu",
        "pt": 1742747423927i64,
        "mc": [{
            "id": "1.241200277",
            "rc": [{
                "id": 58805,
                "batb": [[0, 4.3, 943.24]],
                "batl": [[0, 4.4, 1000.0]]
            }]
        }]
    });

    let message: MarketChangeMessage = serde_json::from_value(json).unwrap();
    assert_eq!(message.op, "mcm");
    assert_eq!(message.id, 1);
    assert_eq!(message.clock, "AJctAKk5AJMu");
    assert_eq!(message.pt, 1742747423927i64);
    assert_eq!(message.market_changes.len(), 1);
    
    let market_change = &message.market_changes[0];
    assert_eq!(market_change.id, "1.241200277");
    assert_eq!(market_change.runner_changes.len(), 1);
    
    let runner_change = &market_change.runner_changes[0];
    assert_eq!(runner_change.id, 58805);
    
    let batb = runner_change.available_to_back.as_ref().unwrap();
    assert_eq!(batb[0][0], 0.0);
    assert_eq!(batb[0][1], 4.3);
    assert_eq!(batb[0][2], 943.24);
    
    let batl = runner_change.available_to_lay.as_ref().unwrap();
    assert_eq!(batl[0][0], 0.0);
    assert_eq!(batl[0][1], 4.4);
    assert_eq!(batl[0][2], 1000.0);
}

#[test]
fn test_heartbeat_message_serialization() {
    let message = HeartbeatMessage {
        clk: "AJctAKk5AJMu".to_string(),
        ct: "HEARTBEAT".to_string(),
        id: 1,
        op: "mcm".to_string(),
        pt: 1742747423927i64,
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["clk"], "AJctAKk5AJMu");
    assert_eq!(json["ct"], "HEARTBEAT");
    assert_eq!(json["id"], 1);
    assert_eq!(json["op"], "mcm");
    assert_eq!(json["pt"], 1742747423927i64);
}

#[test]
fn test_heartbeat_message_deserialization() {
    let json = json!({
        "clk": "AJctAKk5AJMu",
        "ct": "HEARTBEAT",
        "id": 1,
        "op": "mcm",
        "pt": 1742747423927i64
    });

    let message: HeartbeatMessage = serde_json::from_value(json).unwrap();
    assert_eq!(message.clk, "AJctAKk5AJMu");
    assert_eq!(message.ct, "HEARTBEAT");
    assert_eq!(message.id, 1);
    assert_eq!(message.op, "mcm");
    assert_eq!(message.pt, 1742747423927i64);
} 