use serde::{Deserialize, Serialize};

// Retry configuration DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfigDto {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
    pub jitter: bool,
}

impl Default for RetryConfigDto {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            exponential_base: 2.0,
            jitter: true,
        }
    }
}
