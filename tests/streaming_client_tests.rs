use betfair_rs::config::{BetfairConfig, Config};
use betfair_rs::streaming_client::StreamingClient;
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_config() -> Config {
    Config {
        betfair: BetfairConfig {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            api_key: "test_api_key".to_string(),
            pfx_path: "test.pfx".to_string(),
            pfx_password: "test_pfx_pass".to_string(),
            proxy_url: None,
        },
    }
}

#[test]
fn test_streaming_client_new() {
    let client = StreamingClient::new("test_api_key".to_string());

    assert!(!client.is_connected());
    let orderbooks = client.get_orderbooks();
    let books = orderbooks.read().unwrap();
    assert!(books.is_empty());
}

#[test]
fn test_streaming_client_with_session_token() {
    let client = StreamingClient::with_session_token(
        "test_api_key".to_string(),
        "test_session_token".to_string(),
    );

    assert!(!client.is_connected());
    let orderbooks = client.get_orderbooks();
    let books = orderbooks.read().unwrap();
    assert!(books.is_empty());
}

#[test]
fn test_streaming_client_from_config() {
    let config = create_test_config();
    let client = StreamingClient::from_config(config);

    assert!(!client.is_connected());
    let orderbooks = client.get_orderbooks();
    let books = orderbooks.read().unwrap();
    assert!(books.is_empty());
}

#[test]
fn test_set_session_token() {
    let mut client = StreamingClient::new("test_api_key".to_string());

    client.set_session_token("new_session_token".to_string());

    assert!(!client.is_connected());
}

#[test]
fn test_get_orderbooks() {
    let client = StreamingClient::new("test_api_key".to_string());

    let orderbooks = client.get_orderbooks();
    assert!(Arc::strong_count(&orderbooks) > 0);

    let books = orderbooks.read().unwrap();
    assert!(books.is_empty());
}

#[test]
fn test_get_last_update_time_not_exists() {
    let client = StreamingClient::new("test_api_key".to_string());

    let update_time = client.get_last_update_time("1.123456");
    assert!(update_time.is_none());
}

#[test]
fn test_is_connected_initially_false() {
    let client = StreamingClient::new("test_api_key".to_string());

    assert!(!client.is_connected());
}

#[tokio::test]
async fn test_subscribe_without_start() {
    let client = StreamingClient::new("test_api_key".to_string());

    let result = client.subscribe_to_market("1.123456".to_string(), 5).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Streaming client not started"));
}

#[tokio::test]
async fn test_unsubscribe_without_start() {
    let client = StreamingClient::new("test_api_key".to_string());

    let result = client.unsubscribe_from_market("1.123456".to_string()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Streaming client not started"));
}

#[test]
fn test_multiple_clients_independence() {
    let client1 = StreamingClient::new("api_key_1".to_string());
    let client2 = StreamingClient::new("api_key_2".to_string());

    let orderbooks1 = client1.get_orderbooks();
    let orderbooks2 = client2.get_orderbooks();

    assert!(!Arc::ptr_eq(&orderbooks1, &orderbooks2));
}

#[test]
fn test_shared_orderbooks_thread_safety() {
    use std::thread;

    let client = Arc::new(StreamingClient::new("test_api_key".to_string()));
    let orderbooks = client.get_orderbooks();

    let mut handles = vec![];

    for i in 0..5 {
        let orderbooks_clone = Arc::clone(&orderbooks);
        let handle = thread::spawn(move || {
            let mut books = orderbooks_clone.write().unwrap();
            books.insert(format!("market_{}", i), HashMap::new());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let books = orderbooks.read().unwrap();
    assert_eq!(books.len(), 5);
}

#[test]
fn test_streaming_client_with_empty_api_key() {
    let client = StreamingClient::new("".to_string());

    assert!(!client.is_connected());
    let orderbooks = client.get_orderbooks();
    let books = orderbooks.read().unwrap();
    assert!(books.is_empty());
}

#[test]
fn test_streaming_client_session_token_update() {
    let mut client = StreamingClient::new("test_api_key".to_string());

    client.set_session_token("token1".to_string());
    assert!(!client.is_connected());

    client.set_session_token("token2".to_string());
    assert!(!client.is_connected());

    client.set_session_token("token3".to_string());
    assert!(!client.is_connected());
}
