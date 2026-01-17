//! Rate limiting module for protecting against abuse.
//!
//! Implements a token bucket algorithm for rate limiting API requests.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Configuration for the rate limiter.
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Time window for rate limiting
    pub window: Duration,
    /// Whether to enable rate limiting
    pub enabled: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            enabled: true,
        }
    }
}

/// Token bucket rate limiter for controlling request throughput.
///
/// This implementation uses a sliding window approach to track
/// request rates and prevent abuse.
///
/// # Example
/// ```
/// use ecash_sdk_core::rate_limiter::{RateLimiter, RateLimiterConfig};
/// use std::time::Duration;
///
/// let config = RateLimiterConfig {
///     max_requests: 10,
///     window: Duration::from_secs(1),
///     enabled: true,
/// };
/// let limiter = RateLimiter::new(config);
///
/// // Check if request is allowed
/// // limiter.check().await.expect("rate limit exceeded");
/// ```
pub struct RateLimiter {
    config: RateLimiterConfig,
    request_count: AtomicU64,
    window_start: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Creates a new rate limiter with the given configuration.
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            request_count: AtomicU64::new(0),
            window_start: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Creates a disabled rate limiter (always allows requests).
    pub fn disabled() -> Self {
        Self::new(RateLimiterConfig {
            enabled: false,
            ..Default::default()
        })
    }

    /// Checks if a request is allowed under the current rate limit.
    ///
    /// Returns `Ok(())` if the request is allowed, or an error message
    /// if the rate limit has been exceeded.
    ///
    /// This method is safe to call concurrently from multiple tasks.
    pub async fn check(&self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut window_start = self.window_start.lock().await;
        let now = Instant::now();

        // Check if we need to reset the window
        if now.duration_since(*window_start) >= self.config.window {
            *window_start = now;
            self.request_count.store(0, Ordering::SeqCst);
        }

        // Check current request count
        let current = self.request_count.fetch_add(1, Ordering::SeqCst);
        if current >= self.config.max_requests as u64 {
            self.request_count.fetch_sub(1, Ordering::SeqCst);
            return Err(format!(
                "rate limit exceeded: {} requests per {:?}",
                self.config.max_requests, self.config.window
            ));
        }

        Ok(())
    }

    /// Returns the current request count in the window.
    pub fn current_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }

    /// Returns the remaining requests allowed in the current window.
    pub fn remaining(&self) -> u64 {
        let current = self.request_count.load(Ordering::Relaxed);
        let max = self.config.max_requests as u64;
        if current >= max {
            0
        } else {
            max - current
        }
    }

    /// Resets the rate limiter, clearing the request count.
    pub async fn reset(&self) {
        let mut window_start = self.window_start.lock().await;
        *window_start = Instant::now();
        self.request_count.store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let config = RateLimiterConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        for _ in 0..5 {
            assert!(limiter.check().await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let config = RateLimiterConfig {
            max_requests: 3,
            window: Duration::from_secs(60),
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        // Use up the limit
        for _ in 0..3 {
            limiter.check().await.unwrap();
        }

        // Next request should fail
        let result = limiter.check().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("rate limit exceeded"));
    }

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let limiter = RateLimiter::disabled();

        // Should always succeed when disabled
        for _ in 0..100 {
            assert!(limiter.check().await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_remaining() {
        let config = RateLimiterConfig {
            max_requests: 10,
            window: Duration::from_secs(60),
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.remaining(), 10);

        limiter.check().await.unwrap();
        assert_eq!(limiter.remaining(), 9);

        limiter.check().await.unwrap();
        assert_eq!(limiter.remaining(), 8);
    }

    #[tokio::test]
    async fn test_rate_limiter_reset() {
        let config = RateLimiterConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        // Use up all requests
        for _ in 0..5 {
            limiter.check().await.unwrap();
        }
        assert_eq!(limiter.remaining(), 0);

        // Reset and verify
        limiter.reset().await;
        assert_eq!(limiter.remaining(), 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_reset() {
        let config = RateLimiterConfig {
            max_requests: 2,
            window: Duration::from_millis(50),
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        // Use up limit
        limiter.check().await.unwrap();
        limiter.check().await.unwrap();
        assert!(limiter.check().await.is_err());

        // Wait for window to pass
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should be allowed again
        assert!(limiter.check().await.is_ok());
    }
}
