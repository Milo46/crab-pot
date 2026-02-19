use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    // key_hash -> (request_count, window_start, last_cleanup)
    buckets: Arc<DashMap<String, RateLimitBucket>>,
}

#[derive(Debug, Clone)]
struct RateLimitBucket {
    count: u32,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new() -> Self {
        let limiter = Self {
            buckets: Arc::new(DashMap::new()),
        };

        let buckets = limiter.buckets.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                Self::cleanup_old_buckets(&buckets);
            }
        });

        limiter
    }

    pub fn check_rate_limit(
        &self,
        key_hash: &str,
        max_per_second: u32,
        burst_size: u32,
    ) -> Result<(), RateLimitError> {
        let now = Instant::now();
        let window = Duration::from_secs(1);

        let mut entry =
            self.buckets
                .entry(key_hash.to_string())
                .or_insert_with(|| RateLimitBucket {
                    count: 0,
                    window_start: now,
                });

        if now.duration_since(entry.window_start) > window {
            entry.count = 0;
            entry.window_start = now;
        }

        if entry.count >= burst_size {
            let retry_after = window
                .saturating_sub(now.duration_since(entry.window_start))
                .as_secs();
            return Err(RateLimitError {
                retry_after,
                limit: max_per_second,
                remaining: 0,
            });
        }

        entry.count += 1;

        Ok(())
    }

    pub fn get_status(
        &self,
        key_hash: &str,
        max_per_second: u32,
        burst_size: u32,
    ) -> RateLimitStatus {
        let now = Instant::now();
        let window = Duration::from_secs(1);

        if let Some(entry) = self.buckets.get(key_hash) {
            if now.duration_since(entry.window_start) > window {
                return RateLimitStatus {
                    limit: burst_size,
                    remaining: burst_size,
                    reset_in_secs: 0,
                };
            }

            let remaining = burst_size.saturating_sub(entry.count);
            let reset_in_secs = window
                .saturating_sub(now.duration_since(entry.window_start))
                .as_secs();

            RateLimitStatus {
                limit: burst_size,
                remaining,
                reset_in_secs,
            }
        } else {
            RateLimitStatus {
                limit: burst_size,
                remaining: burst_size,
                reset_in_secs: 0,
            }
        }
    }

    fn cleanup_old_buckets(buckets: &DashMap<String, RateLimitBucket>) {
        let now = Instant::now();
        let ttl = Duration::from_secs(60 * 5);

        buckets.retain(|_, bucket| now.duration_since(bucket.window_start) < ttl);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitError {
    pub retry_after: u64,
    pub limit: u32,
    pub remaining: u32,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rate limit exceeded. Limit: {}/s, Retry after: {}s",
            self.limit, self.retry_after
        )
    }
}

impl std::error::Error for RateLimitError {}

#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub reset_in_secs: u64,
}
