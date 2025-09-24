use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::exchange_connectors::{Kline, KlineInterval};
use crate::utils::errors::AppError;

/// Cache key for kline data
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct CacheKey {
    symbol: String,
    interval: String,
    start_time: i64,
    end_time: i64,
}

/// Cached data entry with TTL tracking
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Arc<Vec<Kline>>,
    created_at: Instant,
    access_count: u32,
    last_accessed: Instant,
}

/// Smart caching system for historical kline data
pub struct DataCache {
    /// Main cache storage
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    /// Rate limit tracker for Binance API
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Cache configuration
    config: CacheConfig,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size in MB
    pub max_size_mb: usize,
    /// TTL for cache entries
    pub ttl_seconds: u64,
    /// TTL for frequently accessed entries (extended)
    pub hot_ttl_seconds: u64,
    /// Number of accesses to be considered "hot"
    pub hot_threshold: u32,
    /// Enable cache compression
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 500, // 500MB cache
            ttl_seconds: 300, // 5 minutes for cold data
            hot_ttl_seconds: 900, // 15 minutes for hot data
            hot_threshold: 3, // 3+ accesses = hot
            enable_compression: false, // Disabled for now
        }
    }
}

/// Rate limiter for Binance API
#[derive(Debug)]
struct RateLimiter {
    /// Weight used in current minute
    weight_used: u32,
    /// Maximum weight per minute (Binance default: 1200)
    max_weight_per_minute: u32,
    /// Last reset time
    last_reset: Instant,
    /// Pending requests queue
    pending_requests: Vec<Instant>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            weight_used: 0,
            max_weight_per_minute: 1200,
            last_reset: Instant::now(),
            pending_requests: Vec::new(),
        }
    }

    /// Check if we can make a request with given weight
    fn can_request(&mut self, weight: u32) -> bool {
        self.reset_if_needed();
        self.weight_used + weight <= self.max_weight_per_minute
    }

    /// Record a request
    fn record_request(&mut self, weight: u32) {
        self.reset_if_needed();
        self.weight_used += weight;
        self.pending_requests.push(Instant::now());
    }

    /// Reset counter if minute has passed
    fn reset_if_needed(&mut self) {
        if self.last_reset.elapsed() >= Duration::from_secs(60) {
            self.weight_used = 0;
            self.last_reset = Instant::now();
            self.pending_requests.clear();
        }
    }

    /// Get time to wait before next request
    fn time_to_wait(&self) -> Duration {
        if self.weight_used >= self.max_weight_per_minute {
            let elapsed = self.last_reset.elapsed();
            if elapsed < Duration::from_secs(60) {
                Duration::from_secs(60) - elapsed
            } else {
                Duration::ZERO
            }
        } else {
            Duration::ZERO
        }
    }
}

impl DataCache {
    /// Create a new data cache instance
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            config,
        }
    }

    /// Get cached data or None if not found/expired
    pub async fn get(
        &self,
        symbol: &str,
        interval: &KlineInterval,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Option<Arc<Vec<Kline>>> {
        let key = CacheKey {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            start_time: start_time.timestamp_millis(),
            end_time: end_time.timestamp_millis(),
        };

        // Try to get from cache
        if let Some(mut entry) = self.cache.get_mut(&key) {
            let now = Instant::now();

            // Check if entry is expired
            let ttl = if entry.access_count >= self.config.hot_threshold {
                Duration::from_secs(self.config.hot_ttl_seconds)
            } else {
                Duration::from_secs(self.config.ttl_seconds)
            };

            if now.duration_since(entry.created_at) > ttl {
                // Entry expired
                drop(entry);
                self.cache.remove(&key);
                debug!("Cache miss: expired entry for {}:{}", symbol, interval);
                return None;
            }

            // Update access stats
            entry.access_count += 1;
            entry.last_accessed = now;

            let data = entry.data.clone();
            debug!(
                "Cache hit: {}:{} (accesses: {})",
                symbol, interval, entry.access_count
            );

            Some(data)
        } else {
            debug!("Cache miss: no entry for {}:{}", symbol, interval);
            None
        }
    }

    /// Store data in cache
    pub async fn store(
        &self,
        symbol: &str,
        interval: &KlineInterval,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        data: Vec<Kline>,
    ) {
        let key = CacheKey {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            start_time: start_time.timestamp_millis(),
            end_time: end_time.timestamp_millis(),
        };

        let entry = CacheEntry {
            data: Arc::new(data),
            created_at: Instant::now(),
            access_count: 0,
            last_accessed: Instant::now(),
        };

        // Check cache size before inserting
        if self.should_evict().await {
            self.evict_lru().await;
        }

        self.cache.insert(key, entry);
        info!(
            "Cached data for {}:{} ({} to {})",
            symbol, interval, start_time, end_time
        );
    }

    /// Check if we need to evict entries
    async fn should_evict(&self) -> bool {
        // Estimate cache size (simplified)
        let entry_count = self.cache.len();
        let estimated_size_mb = entry_count * 2; // Rough estimate: 2MB per entry

        estimated_size_mb > self.config.max_size_mb
    }

    /// Evict least recently used entries
    async fn evict_lru(&self) {
        let mut entries: Vec<_> = self.cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let last_accessed = entry.last_accessed;
                (key, last_accessed)
            })
            .collect();

        // Sort by last accessed time
        entries.sort_by_key(|&(_, last_accessed)| last_accessed);

        // Remove 20% of oldest entries
        let to_remove = entries.len() / 5;
        for (key, _) in entries.iter().take(to_remove) {
            self.cache.remove(key);
        }

        info!("Evicted {} cache entries", to_remove);
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        self.cache.clear();
        info!("Cache cleared");
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let total_entries = self.cache.len();
        let mut hot_entries = 0;
        let mut total_accesses = 0;

        for entry in self.cache.iter() {
            total_accesses += entry.access_count as usize;
            if entry.access_count >= self.config.hot_threshold {
                hot_entries += 1;
            }
        }

        CacheStats {
            total_entries,
            hot_entries,
            total_accesses,
            estimated_size_mb: total_entries * 2,
        }
    }

    /// Check rate limit status
    pub async fn can_make_request(&self, weight: u32) -> bool {
        let mut limiter = self.rate_limiter.write().await;
        limiter.can_request(weight)
    }

    /// Record API request
    pub async fn record_request(&self, weight: u32) {
        let mut limiter = self.rate_limiter.write().await;
        limiter.record_request(weight);
    }

    /// Get time to wait before next request
    pub async fn time_to_wait(&self) -> Duration {
        let limiter = self.rate_limiter.read().await;
        limiter.time_to_wait()
    }

    /// Wait if rate limited
    pub async fn wait_if_needed(&self) {
        let wait_time = self.time_to_wait().await;
        if wait_time > Duration::ZERO {
            warn!("Rate limited, waiting {:?}", wait_time);
            tokio::time::sleep(wait_time).await;
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hot_entries: usize,
    pub total_accesses: usize,
    pub estimated_size_mb: usize,
}

/// Global cache instance (singleton)
static CACHE_INSTANCE: once_cell::sync::OnceCell<Arc<DataCache>> = once_cell::sync::OnceCell::new();

/// Get or create the global cache instance
pub fn get_cache() -> Arc<DataCache> {
    CACHE_INSTANCE
        .get_or_init(|| Arc::new(DataCache::new(CacheConfig::default())))
        .clone()
}

/// Initialize cache with custom config
pub fn init_cache(config: CacheConfig) -> Arc<DataCache> {
    let cache = Arc::new(DataCache::new(config));
    CACHE_INSTANCE.set(cache.clone()).ok();
    cache
}