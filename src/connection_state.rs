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
