use betfair_rs::connection_state::{ConnectionManager, ConnectionState};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_connection_state_manager_new() {
    let manager = ConnectionManager::new();
    let state = manager.get_state().await;
    assert!(matches!(state, ConnectionState::Disconnected));
}

#[tokio::test]
async fn test_set_and_get_state() {
    let manager = ConnectionManager::new();

    manager.set_state(ConnectionState::Connecting).await;
    let state = manager.get_state().await;
    assert!(matches!(state, ConnectionState::Connecting));

    manager.set_state(ConnectionState::Connected).await;
    let state = manager.get_state().await;
    assert!(matches!(state, ConnectionState::Connected));

    manager.set_state(ConnectionState::Disconnected).await;
    let state = manager.get_state().await;
    assert!(matches!(state, ConnectionState::Disconnected));
}

#[tokio::test]
async fn test_reconnect_attempts() {
    let manager = ConnectionManager::new();

    let initial_attempts = manager.get_reconnect_attempts().await;
    assert_eq!(initial_attempts, 0);

    manager.set_state(ConnectionState::Reconnecting).await;
    let attempts = manager.get_reconnect_attempts().await;
    assert_eq!(attempts, 1);

    manager.set_state(ConnectionState::Reconnecting).await;
    let attempts = manager.get_reconnect_attempts().await;
    assert_eq!(attempts, 2);

    manager.set_state(ConnectionState::Connected).await;
    let attempts = manager.get_reconnect_attempts().await;
    assert_eq!(attempts, 0);
}

#[tokio::test]
async fn test_is_connected() {
    let manager = ConnectionManager::new();

    assert!(!manager.is_connected().await);

    manager.set_state(ConnectionState::Connecting).await;
    assert!(!manager.is_connected().await);

    manager.set_state(ConnectionState::Connected).await;
    assert!(manager.is_connected().await);

    manager.set_state(ConnectionState::Reconnecting).await;
    assert!(!manager.is_connected().await);

    manager.set_state(ConnectionState::Disconnected).await;
    assert!(!manager.is_connected().await);
}

#[tokio::test]
async fn test_last_connected_duration() {
    let manager = ConnectionManager::new();

    assert!(manager.last_connected_duration().await.is_none());

    manager.set_state(ConnectionState::Connected).await;
    sleep(Duration::from_millis(100)).await;

    let duration = manager.last_connected_duration().await;
    assert!(duration.is_some());
    assert!(duration.unwrap() >= Duration::from_millis(100));

    manager.set_state(ConnectionState::Disconnected).await;
    let duration = manager.last_connected_duration().await;
    assert!(duration.is_some());
    assert!(duration.unwrap() >= Duration::from_millis(100));
}

#[tokio::test]
async fn test_state_transitions() {
    let manager = ConnectionManager::new();

    manager.set_state(ConnectionState::Disconnected).await;
    assert!(matches!(
        manager.get_state().await,
        ConnectionState::Disconnected
    ));

    manager.set_state(ConnectionState::Connecting).await;
    assert!(matches!(
        manager.get_state().await,
        ConnectionState::Connecting
    ));

    manager.set_state(ConnectionState::Connected).await;
    assert!(matches!(
        manager.get_state().await,
        ConnectionState::Connected
    ));

    manager.set_state(ConnectionState::Reconnecting).await;
    assert!(matches!(
        manager.get_state().await,
        ConnectionState::Reconnecting
    ));

    manager.set_state(ConnectionState::Connected).await;
    assert!(matches!(
        manager.get_state().await,
        ConnectionState::Connected
    ));

    manager.set_state(ConnectionState::Disconnected).await;
    assert!(matches!(
        manager.get_state().await,
        ConnectionState::Disconnected
    ));
}

#[tokio::test]
async fn test_concurrent_state_access() {
    use std::sync::Arc;
    use tokio::task;

    let manager = Arc::new(ConnectionManager::new());

    let mut handles = vec![];

    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = task::spawn(async move {
            for _ in 0..10 {
                let state = if i % 2 == 0 {
                    ConnectionState::Connected
                } else {
                    ConnectionState::Disconnected
                };
                manager_clone.set_state(state).await;
                let _ = manager_clone.get_state().await;
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = manager.get_state().await;
    assert!(matches!(
        final_state,
        ConnectionState::Connected | ConnectionState::Disconnected
    ));
}

#[tokio::test]
async fn test_reconnecting_increments_attempts() {
    let manager = ConnectionManager::new();

    for expected_attempts in 1..5 {
        manager.set_state(ConnectionState::Reconnecting).await;
        assert_eq!(manager.get_reconnect_attempts().await, expected_attempts);
        assert!(!manager.is_connected().await);
    }
}
