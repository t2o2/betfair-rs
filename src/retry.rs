use anyhow::Result;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

pub struct RetryPolicy {
    config: RetryConfig,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(RetryConfig::default())
    }
}

impl RetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub async fn retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded after {} attempts", attempt);
                    }
                    return Ok(result);
                }
                Err(err) if attempt >= self.config.max_attempts => {
                    warn!("Operation failed after {} attempts: {}", attempt, err);
                    return Err(err);
                }
                Err(err) => {
                    warn!(
                        "Attempt {} failed: {}. Retrying in {:?}...",
                        attempt, err, delay
                    );
                    sleep(delay).await;

                    // Calculate next delay with exponential backoff
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * self.config.multiplier)
                            .min(self.config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }

    pub async fn retry_with_jitter<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded after {} attempts", attempt);
                    }
                    return Ok(result);
                }
                Err(err) if attempt >= self.config.max_attempts => {
                    warn!("Operation failed after {} attempts: {}", attempt, err);
                    return Err(err);
                }
                Err(err) => {
                    // Add jitter to prevent thundering herd
                    let jitter = Duration::from_millis((rand::random::<f64>() * 1000.0) as u64);
                    let actual_delay = delay + jitter;

                    warn!(
                        "Attempt {} failed: {}. Retrying in {:?}...",
                        attempt, err, actual_delay
                    );
                    sleep(actual_delay).await;

                    // Calculate next delay with exponential backoff
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * self.config.multiplier)
                            .min(self.config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let policy = RetryPolicy::default();
        let result = policy.retry(|| async { Ok::<_, anyhow::Error>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::new(RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
        });

        let result = policy
            .retry(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(anyhow::anyhow!("Simulated failure"))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::new(RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
        });

        let result = policy
            .retry(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>(anyhow::anyhow!("Simulated failure"))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
