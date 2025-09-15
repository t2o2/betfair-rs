use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

#[derive(Clone)]
pub struct ConnectionManager {
    state: Arc<RwLock<ConnectionState>>,
    last_connected: Arc<RwLock<Option<Instant>>>,
    reconnect_attempts: Arc<RwLock<u32>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            last_connected: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    pub async fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.write().await;

        match &new_state {
            ConnectionState::Connected => {
                *self.last_connected.write().await = Some(Instant::now());
                *self.reconnect_attempts.write().await = 0;
            }
            ConnectionState::Reconnecting => {
                *self.reconnect_attempts.write().await += 1;
            }
            _ => {}
        }

        *state = new_state;
    }

    pub async fn get_reconnect_attempts(&self) -> u32 {
        *self.reconnect_attempts.read().await
    }

    pub async fn is_connected(&self) -> bool {
        matches!(*self.state.read().await, ConnectionState::Connected)
    }

    pub async fn last_connected_duration(&self) -> Option<std::time::Duration> {
        self.last_connected
            .read()
            .await
            .map(|instant| instant.elapsed())
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_connection_manager_new() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.get_state().await, ConnectionState::Disconnected);
        assert_eq!(manager.get_reconnect_attempts().await, 0);
        assert!(manager.last_connected_duration().await.is_none());
    }

    #[tokio::test]
    async fn test_connection_manager_default() {
        let manager = ConnectionManager::default();
        assert_eq!(manager.get_state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_set_state_connected() {
        let manager = ConnectionManager::new();

        manager.set_state(ConnectionState::Connected).await;
        assert_eq!(manager.get_state().await, ConnectionState::Connected);
        assert!(manager.is_connected().await);
        assert_eq!(manager.get_reconnect_attempts().await, 0);

        sleep(Duration::from_millis(10)).await;
        let duration = manager.last_connected_duration().await;
        assert!(duration.is_some());
        assert!(duration.unwrap() >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_set_state_reconnecting() {
        let manager = ConnectionManager::new();

        manager.set_state(ConnectionState::Reconnecting).await;
        assert_eq!(manager.get_state().await, ConnectionState::Reconnecting);
        assert_eq!(manager.get_reconnect_attempts().await, 1);

        manager.set_state(ConnectionState::Reconnecting).await;
        assert_eq!(manager.get_reconnect_attempts().await, 2);

        manager.set_state(ConnectionState::Reconnecting).await;
        assert_eq!(manager.get_reconnect_attempts().await, 3);
    }

    #[tokio::test]
    async fn test_reconnect_attempts_reset_on_connected() {
        let manager = ConnectionManager::new();

        manager.set_state(ConnectionState::Reconnecting).await;
        manager.set_state(ConnectionState::Reconnecting).await;
        assert_eq!(manager.get_reconnect_attempts().await, 2);

        manager.set_state(ConnectionState::Connected).await;
        assert_eq!(manager.get_reconnect_attempts().await, 0);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let manager = ConnectionManager::new();

        manager.set_state(ConnectionState::Connecting).await;
        assert_eq!(manager.get_state().await, ConnectionState::Connecting);
        assert!(!manager.is_connected().await);

        manager.set_state(ConnectionState::Connected).await;
        assert!(manager.is_connected().await);

        manager
            .set_state(ConnectionState::Failed("Error".to_string()))
            .await;
        assert_eq!(
            manager.get_state().await,
            ConnectionState::Failed("Error".to_string())
        );
        assert!(!manager.is_connected().await);
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

        manager
            .set_state(ConnectionState::Failed("Test".to_string()))
            .await;
        assert!(!manager.is_connected().await);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let manager = ConnectionManager::new();
        let manager2 = manager.clone();
        let manager3 = manager.clone();

        let handle1 = tokio::spawn(async move {
            for _ in 0..10 {
                manager.set_state(ConnectionState::Connecting).await;
                sleep(Duration::from_millis(1)).await;
            }
        });

        let handle2 = tokio::spawn(async move {
            for _ in 0..10 {
                manager2.set_state(ConnectionState::Reconnecting).await;
                sleep(Duration::from_millis(1)).await;
            }
        });

        let handle3 = tokio::spawn(async move {
            for _ in 0..10 {
                let _ = manager3.get_state().await;
                sleep(Duration::from_millis(1)).await;
            }
        });

        let _ = tokio::join!(handle1, handle2, handle3);
    }

    #[tokio::test]
    async fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
        assert_eq!(ConnectionState::Reconnecting, ConnectionState::Reconnecting);
        assert_eq!(
            ConnectionState::Failed("test".to_string()),
            ConnectionState::Failed("test".to_string())
        );
        assert_ne!(
            ConnectionState::Failed("test1".to_string()),
            ConnectionState::Failed("test2".to_string())
        );
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_clone_manager() {
        let manager = ConnectionManager::new();
        manager.set_state(ConnectionState::Connected).await;

        let cloned = manager.clone();
        assert_eq!(cloned.get_state().await, ConnectionState::Connected);

        cloned.set_state(ConnectionState::Disconnected).await;
        assert_eq!(manager.get_state().await, ConnectionState::Disconnected);
    }
}
