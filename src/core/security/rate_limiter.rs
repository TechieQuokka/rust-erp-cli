use crate::utils::error::ErpResult;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

#[async_trait::async_trait]
pub trait RateLimiterTrait: Send + Sync {
    async fn allow_request(&self, identifier: &str) -> ErpResult<bool>;
    async fn check_rate_limit(&self, identifier: &str, limit: u64, window_seconds: u64) -> ErpResult<bool>;
    async fn reset_limit(&self, identifier: &str) -> ErpResult<()>;
    async fn get_remaining_requests(&self, identifier: &str) -> ErpResult<RemainingRequests>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u64,
    pub requests_per_hour: u64,
    pub requests_per_day: u64,
    pub burst_size: u64,
    pub cleanup_interval_minutes: u64,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_size: 10,
            cleanup_interval_minutes: 30,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
struct RequestWindow {
    requests: Vec<DateTime<Utc>>,
    last_request: DateTime<Utc>,
    total_requests: u64,
}

impl RequestWindow {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            last_request: Utc::now(),
            total_requests: 0,
        }
    }

    fn add_request(&mut self, timestamp: DateTime<Utc>) {
        self.requests.push(timestamp);
        self.last_request = timestamp;
        self.total_requests += 1;
    }

    fn cleanup_old_requests(&mut self, cutoff: DateTime<Utc>) {
        let initial_len = self.requests.len();
        self.requests.retain(|&timestamp| timestamp > cutoff);
        let removed = initial_len - self.requests.len();

        if removed > 0 {
            debug!("Cleaned up {} old requests", removed);
        }
    }

    fn count_requests_in_window(&self, cutoff: DateTime<Utc>) -> usize {
        self.requests.iter().filter(|&&timestamp| timestamp > cutoff).count()
    }
}

pub struct RateLimiter {
    config: RateLimitConfig,
    windows: Arc<Mutex<HashMap<String, RequestWindow>>>,
    last_cleanup: Arc<Mutex<DateTime<Utc>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            windows: Arc::new(Mutex::new(HashMap::new())),
            last_cleanup: Arc::new(Mutex::new(Utc::now())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    pub async fn allow_request(&self, identifier: &str) -> ErpResult<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        let now = Utc::now();
        let mut windows = self.windows.lock().await;

        // Get or create window for this identifier
        let window = windows.entry(identifier.to_string()).or_insert_with(RequestWindow::new);

        // Clean up old requests
        let minute_cutoff = now - Duration::minutes(1);
        let hour_cutoff = now - Duration::hours(1);
        let day_cutoff = now - Duration::days(1);

        window.cleanup_old_requests(day_cutoff);

        // Check rate limits
        let requests_last_minute = window.count_requests_in_window(minute_cutoff);
        let requests_last_hour = window.count_requests_in_window(hour_cutoff);
        let requests_last_day = window.count_requests_in_window(day_cutoff);

        // Check burst limit (based on last few seconds)
        let burst_cutoff = now - Duration::seconds(10);
        let burst_requests = window.count_requests_in_window(burst_cutoff);

        debug!("Rate limit check for {}: minute={}, hour={}, day={}, burst={}",
               identifier, requests_last_minute, requests_last_hour, requests_last_day, burst_requests);

        // Apply limits
        if burst_requests >= self.config.burst_size as usize {
            warn!("Burst limit exceeded for {}: {} requests in last 10 seconds", identifier, burst_requests);
            return Ok(false);
        }

        if requests_last_minute >= self.config.requests_per_minute as usize {
            warn!("Minute limit exceeded for {}: {} requests", identifier, requests_last_minute);
            return Ok(false);
        }

        if requests_last_hour >= self.config.requests_per_hour as usize {
            warn!("Hour limit exceeded for {}: {} requests", identifier, requests_last_hour);
            return Ok(false);
        }

        if requests_last_day >= self.config.requests_per_day as usize {
            warn!("Day limit exceeded for {}: {} requests", identifier, requests_last_day);
            return Ok(false);
        }

        // Allow request and record it
        window.add_request(now);
        debug!("Request allowed for {}", identifier);

        // Drop the mutex guard before async operations
        drop(windows);

        // Periodic cleanup
        self.periodic_cleanup().await?;

        Ok(true)
    }

    pub async fn check_rate_limit(&self, identifier: &str, limit: u64, window_seconds: u64) -> ErpResult<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        let now = Utc::now();
        let windows = self.windows.lock().await;

        if let Some(window) = windows.get(identifier) {
            let cutoff = now - Duration::seconds(window_seconds as i64);
            let requests_in_window = window.count_requests_in_window(cutoff);

            if requests_in_window >= limit as usize {
                warn!("Custom rate limit exceeded for {}: {} requests in {} seconds",
                      identifier, requests_in_window, window_seconds);
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn reset_limit(&self, identifier: &str) -> ErpResult<()> {
        let mut windows = self.windows.lock().await;
        windows.remove(identifier);
        debug!("Rate limit reset for {}", identifier);
        Ok(())
    }

    pub async fn get_remaining_requests(&self, identifier: &str) -> ErpResult<RemainingRequests> {
        let now = Utc::now();
        let windows = self.windows.lock().await;

        if let Some(window) = windows.get(identifier) {
            let minute_cutoff = now - Duration::minutes(1);
            let hour_cutoff = now - Duration::hours(1);
            let day_cutoff = now - Duration::days(1);

            let requests_last_minute = window.count_requests_in_window(minute_cutoff);
            let requests_last_hour = window.count_requests_in_window(hour_cutoff);
            let requests_last_day = window.count_requests_in_window(day_cutoff);

            Ok(RemainingRequests {
                per_minute: self.config.requests_per_minute.saturating_sub(requests_last_minute as u64),
                per_hour: self.config.requests_per_hour.saturating_sub(requests_last_hour as u64),
                per_day: self.config.requests_per_day.saturating_sub(requests_last_day as u64),
                reset_time_minute: now + Duration::minutes(1),
                reset_time_hour: now + Duration::hours(1),
                reset_time_day: now + Duration::days(1),
            })
        } else {
            Ok(RemainingRequests {
                per_minute: self.config.requests_per_minute,
                per_hour: self.config.requests_per_hour,
                per_day: self.config.requests_per_day,
                reset_time_minute: now + Duration::minutes(1),
                reset_time_hour: now + Duration::hours(1),
                reset_time_day: now + Duration::days(1),
            })
        }
    }

    pub async fn get_statistics(&self) -> ErpResult<RateLimitStatistics> {
        let windows = self.windows.lock().await;

        let total_clients = windows.len();
        let total_requests: u64 = windows.values().map(|w| w.total_requests).sum();

        let now = Utc::now();
        let hour_cutoff = now - Duration::hours(1);
        let active_clients = windows.values()
            .filter(|w| w.last_request > hour_cutoff)
            .count();

        Ok(RateLimitStatistics {
            total_clients,
            active_clients,
            total_requests,
            config: self.config.clone(),
        })
    }

    async fn periodic_cleanup(&self) -> ErpResult<()> {
        let mut last_cleanup = self.last_cleanup.lock().await;
        let now = Utc::now();

        if now.signed_duration_since(*last_cleanup).num_minutes() >= self.config.cleanup_interval_minutes as i64 {
            *last_cleanup = now;
            drop(last_cleanup); // Release the lock before the cleanup

            self.cleanup_old_entries().await?;
        }

        Ok(())
    }

    pub async fn cleanup_old_entries(&self) -> ErpResult<usize> {
        let mut windows = self.windows.lock().await;
        let now = Utc::now();
        let cleanup_cutoff = now - Duration::days(1);

        let initial_size = windows.len();

        // Remove windows that have no recent activity
        windows.retain(|_, window| window.last_request > cleanup_cutoff);

        // Clean up old requests in remaining windows
        for window in windows.values_mut() {
            window.cleanup_old_requests(cleanup_cutoff);
        }

        let removed = initial_size - windows.len();
        if removed > 0 {
            debug!("Cleaned up {} old rate limit entries", removed);
        }

        Ok(removed)
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn get_config(&self) -> &RateLimitConfig {
        &self.config
    }
}

#[async_trait::async_trait]
impl RateLimiterTrait for RateLimiter {
    async fn allow_request(&self, identifier: &str) -> ErpResult<bool> {
        self.allow_request(identifier).await
    }

    async fn check_rate_limit(&self, identifier: &str, limit: u64, window_seconds: u64) -> ErpResult<bool> {
        self.check_rate_limit(identifier, limit, window_seconds).await
    }

    async fn reset_limit(&self, identifier: &str) -> ErpResult<()> {
        self.reset_limit(identifier).await
    }

    async fn get_remaining_requests(&self, identifier: &str) -> ErpResult<RemainingRequests> {
        self.get_remaining_requests(identifier).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemainingRequests {
    pub per_minute: u64,
    pub per_hour: u64,
    pub per_day: u64,
    pub reset_time_minute: DateTime<Utc>,
    pub reset_time_hour: DateTime<Utc>,
    pub reset_time_day: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatistics {
    pub total_clients: usize,
    pub active_clients: usize,
    pub total_requests: u64,
    pub config: RateLimitConfig,
}

// Mock implementation for testing
pub struct MockRateLimiter {
    allow_requests: bool,
}

impl MockRateLimiter {
    pub fn new() -> Self {
        Self {
            allow_requests: true,
        }
    }

    pub fn set_allow_requests(&mut self, allow: bool) {
        self.allow_requests = allow;
    }

    pub async fn allow_request(&self, _identifier: &str) -> ErpResult<bool> {
        Ok(self.allow_requests)
    }

    pub async fn check_rate_limit(&self, _identifier: &str, _limit: u64, _window_seconds: u64) -> ErpResult<bool> {
        Ok(self.allow_requests)
    }

    pub async fn reset_limit(&self, _identifier: &str) -> ErpResult<()> {
        Ok(())
    }

    pub async fn get_remaining_requests(&self, _identifier: &str) -> ErpResult<RemainingRequests> {
        Ok(RemainingRequests {
            per_minute: u64::MAX,
            per_hour: u64::MAX,
            per_day: u64::MAX,
            reset_time_minute: Utc::now(),
            reset_time_hour: Utc::now(),
            reset_time_day: Utc::now(),
        })
    }
}

#[async_trait::async_trait]
impl RateLimiterTrait for MockRateLimiter {
    async fn allow_request(&self, identifier: &str) -> ErpResult<bool> {
        self.allow_request(identifier).await
    }

    async fn check_rate_limit(&self, identifier: &str, limit: u64, window_seconds: u64) -> ErpResult<bool> {
        self.check_rate_limit(identifier, limit, window_seconds).await
    }

    async fn reset_limit(&self, identifier: &str) -> ErpResult<()> {
        self.reset_limit(identifier).await
    }

    async fn get_remaining_requests(&self, identifier: &str) -> ErpResult<RemainingRequests> {
        self.get_remaining_requests(identifier).await
    }
}

// Factory function for different rate limiting strategies
pub fn create_rate_limiter(strategy: RateLimitStrategy) -> RateLimiter {
    let config = match strategy {
        RateLimitStrategy::Strict => RateLimitConfig {
            requests_per_minute: 30,
            requests_per_hour: 500,
            requests_per_day: 5000,
            burst_size: 5,
            cleanup_interval_minutes: 15,
            enabled: true,
        },
        RateLimitStrategy::Moderate => RateLimitConfig::default(),
        RateLimitStrategy::Lenient => RateLimitConfig {
            requests_per_minute: 120,
            requests_per_hour: 5000,
            requests_per_day: 50000,
            burst_size: 20,
            cleanup_interval_minutes: 60,
            enabled: true,
        },
        RateLimitStrategy::Disabled => RateLimitConfig {
            enabled: false,
            ..RateLimitConfig::default()
        },
    };

    RateLimiter::new(config)
}

#[derive(Debug, Clone, Copy)]
pub enum RateLimitStrategy {
    Strict,
    Moderate,
    Lenient,
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limits() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            burst_size: 5,
            cleanup_interval_minutes: 30,
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // Should allow first few requests
        for i in 0..5 {
            let allowed = limiter.allow_request("test_user").await.unwrap();
            assert!(allowed, "Request {} should be allowed", i);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_burst() {
        let config = RateLimitConfig {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_size: 3,
            cleanup_interval_minutes: 30,
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // Allow up to burst limit
        for i in 0..3 {
            let allowed = limiter.allow_request("burst_test").await.unwrap();
            assert!(allowed, "Request {} should be allowed", i);
        }

        // Should block the next request (exceeds burst)
        let allowed = limiter.allow_request("burst_test").await.unwrap();
        assert!(!allowed, "Request should be blocked due to burst limit");
    }

    #[tokio::test]
    async fn test_rate_limiter_minute_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_hour: 100,
            requests_per_day: 1000,
            burst_size: 10,
            cleanup_interval_minutes: 30,
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // Allow first 2 requests
        assert!(limiter.allow_request("minute_test").await.unwrap());
        assert!(limiter.allow_request("minute_test").await.unwrap());

        // Should block the third request
        let allowed = limiter.allow_request("minute_test").await.unwrap();
        assert!(!allowed, "Third request should be blocked by minute limit");
    }

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            enabled: false,
            ..RateLimitConfig::default()
        };

        let limiter = RateLimiter::new(config);

        // Should allow unlimited requests when disabled
        for _i in 0..100 {
            let allowed = limiter.allow_request("disabled_test").await.unwrap();
            assert!(allowed);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_reset() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            requests_per_hour: 10,
            requests_per_day: 100,
            burst_size: 1,
            cleanup_interval_minutes: 30,
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // Use up the limit
        assert!(limiter.allow_request("reset_test").await.unwrap());
        assert!(!limiter.allow_request("reset_test").await.unwrap());

        // Reset and try again
        limiter.reset_limit("reset_test").await.unwrap();
        assert!(limiter.allow_request("reset_test").await.unwrap());
    }

    #[tokio::test]
    async fn test_remaining_requests() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            burst_size: 5,
            cleanup_interval_minutes: 30,
            enabled: true,
        };

        let limiter = RateLimiter::new(config);

        // Initial remaining requests
        let remaining = limiter.get_remaining_requests("remaining_test").await.unwrap();
        assert_eq!(remaining.per_minute, 10);
        assert_eq!(remaining.per_hour, 100);
        assert_eq!(remaining.per_day, 1000);

        // Make some requests
        for _i in 0..3 {
            limiter.allow_request("remaining_test").await.unwrap();
        }

        let remaining = limiter.get_remaining_requests("remaining_test").await.unwrap();
        assert_eq!(remaining.per_minute, 7);
        assert_eq!(remaining.per_hour, 97);
        assert_eq!(remaining.per_day, 997);
    }

    #[tokio::test]
    async fn test_statistics() {
        let limiter = RateLimiter::with_default_config();

        // Make requests from different identifiers
        limiter.allow_request("user1").await.unwrap();
        limiter.allow_request("user2").await.unwrap();
        limiter.allow_request("user1").await.unwrap();

        let stats = limiter.get_statistics().await.unwrap();
        assert_eq!(stats.total_clients, 2);
        assert_eq!(stats.active_clients, 2);
        assert_eq!(stats.total_requests, 3);
    }

    #[tokio::test]
    async fn test_cleanup_old_entries() {
        let mut config = RateLimitConfig::default();
        config.cleanup_interval_minutes = 0; // Force cleanup every time

        let limiter = RateLimiter::new(config);

        // Add some requests
        limiter.allow_request("cleanup_test").await.unwrap();

        let stats_before = limiter.get_statistics().await.unwrap();
        assert_eq!(stats_before.total_clients, 1);

        // Run cleanup (this won't remove anything since requests are recent)
        let removed = limiter.cleanup_old_entries().await.unwrap();
        assert_eq!(removed, 0);

        let stats_after = limiter.get_statistics().await.unwrap();
        assert_eq!(stats_after.total_clients, 1);
    }

    #[tokio::test]
    async fn test_custom_rate_limit_check() {
        let limiter = RateLimiter::with_default_config();

        // Make some requests
        for _i in 0..5 {
            limiter.allow_request("custom_test").await.unwrap();
        }

        // Check custom limit (should pass - 5 requests in 60 seconds is under limit of 10)
        let allowed = limiter.check_rate_limit("custom_test", 10, 60).await.unwrap();
        assert!(allowed);

        // Check stricter custom limit (should fail - 5 requests exceeds limit of 3)
        let allowed = limiter.check_rate_limit("custom_test", 3, 60).await.unwrap();
        assert!(!allowed);
    }

    #[test]
    fn test_rate_limit_strategies() {
        let strict = create_rate_limiter(RateLimitStrategy::Strict);
        let moderate = create_rate_limiter(RateLimitStrategy::Moderate);
        let lenient = create_rate_limiter(RateLimitStrategy::Lenient);
        let disabled = create_rate_limiter(RateLimitStrategy::Disabled);

        assert!(strict.get_config().requests_per_minute < moderate.get_config().requests_per_minute);
        assert!(moderate.get_config().requests_per_minute < lenient.get_config().requests_per_minute);
        assert!(!disabled.is_enabled());
        assert!(strict.is_enabled());
    }

    #[tokio::test]
    async fn test_mock_rate_limiter() {
        let mut mock = MockRateLimiter::new();

        assert!(mock.allow_request("test").await.unwrap());

        mock.set_allow_requests(false);
        assert!(!mock.allow_request("test").await.unwrap());

        assert!(mock.check_rate_limit("test", 10, 60).await.unwrap() == false);
        assert!(mock.reset_limit("test").await.is_ok());
    }
}