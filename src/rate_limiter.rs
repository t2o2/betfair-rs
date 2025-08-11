use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Token bucket implementation for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<BucketState>>,
    capacity: u32,
    refill_rate: f64, // tokens per second
}

#[derive(Debug)]
struct BucketState {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter with specified capacity and refill rate
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            state: Arc::new(Mutex::new(BucketState {
                tokens: capacity as f64,
                last_refill: Instant::now(),
            })),
            capacity,
            refill_rate,
        }
    }

    /// Create rate limiter for Betfair data requests (60 per minute)
    pub fn for_data_requests() -> Self {
        Self::new(60, 1.0) // 60 tokens, 1 token per second
    }

    /// Create rate limiter for Betfair navigation requests (1000 per minute)
    pub fn for_navigation_requests() -> Self {
        Self::new(1000, 16.67) // 1000 tokens, ~16.67 tokens per second
    }

    /// Create rate limiter for Betfair transaction requests (60 per minute)
    pub fn for_transaction_requests() -> Self {
        Self::new(60, 1.0) // 60 tokens, 1 token per second
    }

    /// Acquire a token, waiting if necessary
    pub async fn acquire(&self) -> Result<()> {
        self.acquire_tokens(1.0).await
    }

    /// Acquire multiple tokens, waiting if necessary
    pub async fn acquire_tokens(&self, tokens: f64) -> Result<()> {
        if tokens > self.capacity as f64 {
            return Err(anyhow::anyhow!(
                "Requested {} tokens exceeds capacity of {}",
                tokens,
                self.capacity
            ));
        }

        loop {
            let mut state = self.state.lock().await;

            // Refill tokens based on elapsed time
            let now = Instant::now();
            let elapsed = now.duration_since(state.last_refill).as_secs_f64();
            let tokens_to_add = elapsed * self.refill_rate;

            state.tokens = (state.tokens + tokens_to_add).min(self.capacity as f64);
            state.last_refill = now;

            if state.tokens >= tokens {
                // We have enough tokens
                state.tokens -= tokens;
                debug!(
                    "Rate limiter: acquired {} tokens, {} remaining",
                    tokens, state.tokens
                );
                return Ok(());
            }

            // Calculate wait time
            let tokens_needed = tokens - state.tokens;
            let wait_seconds = tokens_needed / self.refill_rate;
            let wait_duration = Duration::from_secs_f64(wait_seconds);

            warn!(
                "Rate limit: waiting {:?} for {} tokens (current: {:.1}, capacity: {})",
                wait_duration, tokens, state.tokens, self.capacity
            );

            // Release lock while waiting
            drop(state);
            sleep(wait_duration).await;
        }
    }

    /// Try to acquire a token without waiting
    pub async fn try_acquire(&self) -> Result<bool> {
        self.try_acquire_tokens(1.0).await
    }

    /// Try to acquire multiple tokens without waiting
    pub async fn try_acquire_tokens(&self, tokens: f64) -> Result<bool> {
        if tokens > self.capacity as f64 {
            return Err(anyhow::anyhow!(
                "Requested {} tokens exceeds capacity of {}",
                tokens,
                self.capacity
            ));
        }

        let mut state = self.state.lock().await;

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;

        state.tokens = (state.tokens + tokens_to_add).min(self.capacity as f64);
        state.last_refill = now;

        if state.tokens >= tokens {
            state.tokens -= tokens;
            debug!(
                "Rate limiter: acquired {} tokens, {} remaining",
                tokens, state.tokens
            );
            Ok(true)
        } else {
            debug!(
                "Rate limiter: insufficient tokens ({} < {})",
                state.tokens, tokens
            );
            Ok(false)
        }
    }

    /// Get current number of available tokens
    pub async fn available_tokens(&self) -> f64 {
        let mut state = self.state.lock().await;

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;

        state.tokens = (state.tokens + tokens_to_add).min(self.capacity as f64);
        state.last_refill = now;

        state.tokens
    }
}

/// Composite rate limiter for different API endpoint types
#[derive(Clone)]
pub struct BetfairRateLimiter {
    data_limiter: RateLimiter,
    navigation_limiter: RateLimiter,
    transaction_limiter: RateLimiter,
}

impl BetfairRateLimiter {
    pub fn new() -> Self {
        Self {
            data_limiter: RateLimiter::for_data_requests(),
            navigation_limiter: RateLimiter::for_navigation_requests(),
            transaction_limiter: RateLimiter::for_transaction_requests(),
        }
    }

    pub async fn acquire_for_data(&self) -> Result<()> {
        self.data_limiter.acquire().await
    }

    pub async fn acquire_for_navigation(&self) -> Result<()> {
        self.navigation_limiter.acquire().await
    }

    pub async fn acquire_for_transaction(&self) -> Result<()> {
        self.transaction_limiter.acquire().await
    }
}

impl Default for BetfairRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(5, 1.0); // 5 tokens, 1 token/sec

        // Should be able to acquire 5 tokens immediately
        for _ in 0..5 {
            assert!(limiter.acquire().await.is_ok());
        }

        // 6th request should wait
        let start = Instant::now();
        assert!(limiter.acquire().await.is_ok());
        let elapsed = start.elapsed();

        // Should have waited approximately 1 second
        assert!(elapsed >= Duration::from_millis(900));
        assert!(elapsed < Duration::from_millis(1200));
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::new(2, 1.0);

        // Should succeed for first 2
        assert!(limiter.try_acquire().await.unwrap());
        assert!(limiter.try_acquire().await.unwrap());

        // Should fail for 3rd
        assert!(!limiter.try_acquire().await.unwrap());

        // Wait for refill
        sleep(Duration::from_secs(1)).await;

        // Should succeed again
        assert!(limiter.try_acquire().await.unwrap());
    }

    #[tokio::test]
    async fn test_betfair_rate_limiter() {
        let limiter = BetfairRateLimiter::new();

        // Should be able to acquire for different types
        assert!(limiter.acquire_for_data().await.is_ok());
        assert!(limiter.acquire_for_navigation().await.is_ok());
        assert!(limiter.acquire_for_transaction().await.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let limiter = Arc::new(RateLimiter::new(10, 10.0)); // 10 tokens, 10/sec

        let mut handles = vec![];

        // Spawn 20 tasks trying to acquire tokens
        for _ in 0..20 {
            let limiter_clone = limiter.clone();
            handles.push(tokio::spawn(async move { limiter_clone.acquire().await }));
        }

        // Wait for all with timeout
        let results = timeout(Duration::from_secs(3), async {
            let mut results = vec![];
            for handle in handles {
                results.push(handle.await.unwrap());
            }
            results
        })
        .await;

        assert!(results.is_ok());
        let results = results.unwrap();

        // All should succeed
        for result in results {
            assert!(result.is_ok());
        }
    }
}
