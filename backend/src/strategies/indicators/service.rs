use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use chrono::{Utc, Duration};
use tracing::{debug, warn, error, info};
use uuid::Uuid;

use crate::exchange_connectors::Kline;
use crate::utils::errors::AppError;
use super::core::*;
use super::moving_averages::*;
use super::momentum::*;

/// Service for managing indicators as shared resources
pub struct IndicatorService {
    /// Cache of indicator instances by symbol/interval/name
    indicators: Arc<RwLock<HashMap<IndicatorCacheKey, Box<dyn Indicator>>>>,
    /// Cache of calculated values with TTL
    value_cache: Arc<RwLock<HashMap<IndicatorCacheKey, CachedIndicatorValue>>>,
    /// Registry of available indicator factories
    registry: Arc<RwLock<HashMap<String, Box<dyn IndicatorFactory>>>>,
    /// Active subscriptions for real-time updates
    subscriptions: Arc<RwLock<HashMap<String, Vec<IndicatorSubscription>>>>,
    /// Configuration
    config: IndicatorServiceConfig,
}

#[derive(Debug, Clone)]
pub struct IndicatorServiceConfig {
    /// Maximum cache size per symbol
    pub max_cache_per_symbol: usize,
    /// Cache TTL for values
    pub value_ttl_minutes: i64,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Batch update size
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct IndicatorSubscription {
    pub subscriber_id: Uuid,
    pub indicator_key: IndicatorCacheKey,
    pub callback_url: Option<String>,
}

impl Default for IndicatorServiceConfig {
    fn default() -> Self {
        Self {
            max_cache_per_symbol: 1000,
            value_ttl_minutes: 60,
            enable_profiling: false,
            batch_size: 100,
        }
    }
}

impl IndicatorService {
    /// Create a new indicator service
    pub fn new(config: IndicatorServiceConfig) -> Self {
        let service = Self {
            indicators: Arc::new(RwLock::new(HashMap::new())),
            value_cache: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // Initialize with default indicators
        tokio::spawn({
            let service = service.clone();
            async move {
                if let Err(e) = service.register_default_indicators().await {
                    error!("Failed to register default indicators: {}", e);
                }
            }
        });

        service
    }

    /// Register default indicator factories
    async fn register_default_indicators(&self) -> Result<(), AppError> {
        let mut registry = self.registry.write().await;

        // Register moving averages
        registry.insert("SMA".to_string(), Box::new(SMAFactory));
        registry.insert("EMA".to_string(), Box::new(EMAFactory));
        registry.insert("WMA".to_string(), Box::new(WMAFactory));

        // Register momentum indicators
        registry.insert("RSI".to_string(), Box::new(RSIFactory));
        registry.insert("MACD".to_string(), Box::new(MACDFactory));
        registry.insert("Stochastic".to_string(), Box::new(StochasticFactory));

        info!("Registered {} indicator factories", registry.len());
        Ok(())
    }

    /// Get or create an indicator
    pub async fn get_indicator(
        &self,
        symbol: &str,
        interval: &str,
        indicator_name: &str,
        params: &serde_json::Value,
    ) -> Result<Arc<Mutex<Box<dyn Indicator>>>, AppError> {
        let cache_key = IndicatorCacheKey {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            indicator_name: indicator_name.to_string(),
            params_hash: self.hash_params(params),
        };

        // Check cache first
        {
            let indicators = self.indicators.read().await;
            if let Some(indicator) = indicators.get(&cache_key) {
                return Ok(Arc::new(Mutex::new(indicator.clone_box())));
            }
        }

        // Create new indicator
        let indicator = self.create_indicator(indicator_name, params).await?;

        // Cache the indicator
        {
            let mut indicators = self.indicators.write().await;
            indicators.insert(cache_key.clone(), indicator.clone_box());
        }

        Ok(Arc::new(Mutex::new(indicator)))
    }

    /// Create a new indicator instance
    async fn create_indicator(
        &self,
        name: &str,
        params: &serde_json::Value,
    ) -> Result<Box<dyn Indicator>, AppError> {
        let registry = self.registry.read().await;
        let factory = registry.get(name)
            .ok_or_else(|| AppError::BadRequest(format!("Unknown indicator: {}", name)))?;

        factory.create(params)
    }

    /// Get cached indicator value
    pub async fn get_value(
        &self,
        symbol: &str,
        interval: &str,
        indicator_name: &str,
        params: &serde_json::Value,
    ) -> Result<Option<IndicatorValue>, AppError> {
        let cache_key = IndicatorCacheKey {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            indicator_name: indicator_name.to_string(),
            params_hash: self.hash_params(params),
        };

        let value_cache = self.value_cache.read().await;
        if let Some(cached) = value_cache.get(&cache_key) {
            // Check TTL
            let ttl = Duration::minutes(self.config.value_ttl_minutes);
            if Utc::now().signed_duration_since(cached.last_update) < ttl {
                return Ok(Some(cached.value.clone()));
            }
        }

        Ok(None)
    }

    /// Update indicators with new kline data
    pub async fn update_with_kline(
        &self,
        symbol: &str,
        interval: &str,
        kline: &Kline,
    ) -> Result<(), AppError> {
        let prefix = format!("{}:{}", symbol, interval);
        let indicator_keys: Vec<IndicatorCacheKey> = {
            let indicators = self.indicators.read().await;
            indicators.keys()
                .filter(|key| key.symbol == symbol && key.interval == interval)
                .cloned()
                .collect()
        };

        let num_indicators = indicator_keys.len();

        // Update all matching indicators
        for key in indicator_keys {
            if let Err(e) = self.update_single_indicator(&key, kline).await {
                warn!("Failed to update indicator {}: {}", key.indicator_name, e);
            }
        }

        debug!("Updated {} indicators for {}", num_indicators, prefix);
        Ok(())
    }

    /// Update multiple indicators with batch data
    pub async fn update_batch(
        &self,
        symbol: &str,
        interval: &str,
        klines: &[Kline],
    ) -> Result<(), AppError> {
        let chunk_size = self.config.batch_size;

        for chunk in klines.chunks(chunk_size) {
            for kline in chunk {
                self.update_with_kline(symbol, interval, kline).await?;
            }
        }

        info!("Batch updated {} klines for {}:{}", klines.len(), symbol, interval);
        Ok(())
    }

    /// Update a single indicator
    async fn update_single_indicator(
        &self,
        key: &IndicatorCacheKey,
        kline: &Kline,
    ) -> Result<(), AppError> {
        // Get indicator
        let indicator = {
            let indicators = self.indicators.read().await;
            indicators.get(key)
                .ok_or_else(|| AppError::BadRequest("Indicator not found".to_string()))?
                .clone_box()
        };

        // Update indicator in a separate task to avoid holding locks
        let mut indicator = indicator;
        indicator.update(kline).await?;

        // Cache the updated value
        if let Some(value) = indicator.value() {
            let cached_value = CachedIndicatorValue {
                value,
                last_update: Utc::now(),
                kline_count: 1,
            };

            let mut value_cache = self.value_cache.write().await;
            value_cache.insert(key.clone(), cached_value);
        }

        // Update the indicator in cache
        {
            let mut indicators = self.indicators.write().await;
            indicators.insert(key.clone(), indicator);
        }

        Ok(())
    }

    /// Subscribe to indicator updates
    pub async fn subscribe(
        &self,
        subscriber_id: Uuid,
        symbol: &str,
        interval: &str,
        indicator_name: &str,
        params: &serde_json::Value,
        callback_url: Option<String>,
    ) -> Result<(), AppError> {
        let cache_key = IndicatorCacheKey {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            indicator_name: indicator_name.to_string(),
            params_hash: self.hash_params(params),
        };

        let subscription = IndicatorSubscription {
            subscriber_id,
            indicator_key: cache_key.clone(),
            callback_url,
        };

        let mut subscriptions = self.subscriptions.write().await;
        let key = format!("{}:{}", symbol, interval);
        subscriptions.entry(key).or_insert_with(Vec::new).push(subscription);

        Ok(())
    }

    /// Unsubscribe from indicator updates
    pub async fn unsubscribe(&self, subscriber_id: Uuid) -> Result<(), AppError> {
        let mut subscriptions = self.subscriptions.write().await;

        for (_, subs) in subscriptions.iter_mut() {
            subs.retain(|sub| sub.subscriber_id != subscriber_id);
        }

        // Remove empty entries
        subscriptions.retain(|_, subs| !subs.is_empty());

        Ok(())
    }

    /// Clean up expired cache entries
    pub async fn cleanup_cache(&self) -> Result<(), AppError> {
        let ttl = Duration::minutes(self.config.value_ttl_minutes);
        let now = Utc::now();

        let mut value_cache = self.value_cache.write().await;
        let initial_size = value_cache.len();

        value_cache.retain(|_, cached| {
            now.signed_duration_since(cached.last_update) < ttl
        });

        let cleaned = initial_size - value_cache.len();
        if cleaned > 0 {
            info!("Cleaned up {} expired cache entries", cleaned);
        }

        Ok(())
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> IndicatorServiceStats {
        let indicators = self.indicators.read().await;
        let value_cache = self.value_cache.read().await;
        let subscriptions = self.subscriptions.read().await;

        IndicatorServiceStats {
            active_indicators: indicators.len(),
            cached_values: value_cache.len(),
            active_subscriptions: subscriptions.values().map(|v| v.len()).sum(),
            memory_usage_mb: 0, // TODO: Calculate actual memory usage
        }
    }

    /// List available indicators
    pub async fn list_indicators(&self) -> Result<Vec<IndicatorMetadata>, AppError> {
        let registry = self.registry.read().await;
        Ok(registry.values().map(|factory| factory.metadata()).collect())
    }

    fn hash_params(&self, params: &serde_json::Value) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        params.to_string().hash(&mut hasher);
        hasher.finish()
    }
}

impl Clone for IndicatorService {
    fn clone(&self) -> Self {
        Self {
            indicators: Arc::clone(&self.indicators),
            value_cache: Arc::clone(&self.value_cache),
            registry: Arc::clone(&self.registry),
            subscriptions: Arc::clone(&self.subscriptions),
            config: self.config.clone(),
        }
    }
}

#[derive(Debug)]
pub struct IndicatorServiceStats {
    pub active_indicators: usize,
    pub cached_values: usize,
    pub active_subscriptions: usize,
    pub memory_usage_mb: usize,
}

// Factory implementations for default indicators
struct SMAFactory;
struct EMAFactory;
struct WMAFactory;
struct RSIFactory;
struct MACDFactory;
struct StochasticFactory;

impl IndicatorFactory for SMAFactory {
    fn create(&self, params: &serde_json::Value) -> Result<Box<dyn Indicator>, AppError> {
        let period = params.get("period")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;
        Ok(Box::new(SMA::new(period)))
    }

    fn metadata(&self) -> IndicatorMetadata {
        IndicatorMetadata {
            name: "SMA".to_string(),
            category: IndicatorCategory::Trend,
            description: "Simple Moving Average".to_string(),
            default_params: serde_json::json!({"period": 20}),
            min_periods: 1,
            output_type: "Single".to_string(),
        }
    }
}

impl IndicatorFactory for EMAFactory {
    fn create(&self, params: &serde_json::Value) -> Result<Box<dyn Indicator>, AppError> {
        let period = params.get("period")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;
        Ok(Box::new(EMA::new(period)))
    }

    fn metadata(&self) -> IndicatorMetadata {
        IndicatorMetadata {
            name: "EMA".to_string(),
            category: IndicatorCategory::Trend,
            description: "Exponential Moving Average".to_string(),
            default_params: serde_json::json!({"period": 20}),
            min_periods: 1,
            output_type: "Single".to_string(),
        }
    }
}

impl IndicatorFactory for WMAFactory {
    fn create(&self, params: &serde_json::Value) -> Result<Box<dyn Indicator>, AppError> {
        let period = params.get("period")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;
        Ok(Box::new(WMA::new(period)))
    }

    fn metadata(&self) -> IndicatorMetadata {
        IndicatorMetadata {
            name: "WMA".to_string(),
            category: IndicatorCategory::Trend,
            description: "Weighted Moving Average".to_string(),
            default_params: serde_json::json!({"period": 20}),
            min_periods: 1,
            output_type: "Single".to_string(),
        }
    }
}

impl IndicatorFactory for RSIFactory {
    fn create(&self, params: &serde_json::Value) -> Result<Box<dyn Indicator>, AppError> {
        let period = params.get("period")
            .and_then(|v| v.as_u64())
            .unwrap_or(14) as usize;
        Ok(Box::new(RSI::new(period)))
    }

    fn metadata(&self) -> IndicatorMetadata {
        IndicatorMetadata {
            name: "RSI".to_string(),
            category: IndicatorCategory::Momentum,
            description: "Relative Strength Index".to_string(),
            default_params: serde_json::json!({"period": 14}),
            min_periods: 15,
            output_type: "Single".to_string(),
        }
    }
}

impl IndicatorFactory for MACDFactory {
    fn create(&self, params: &serde_json::Value) -> Result<Box<dyn Indicator>, AppError> {
        let fast = params.get("fast_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(12) as usize;
        let slow = params.get("slow_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(26) as usize;
        let signal = params.get("signal_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(9) as usize;
        Ok(Box::new(MACD::new(fast, slow, signal)))
    }

    fn metadata(&self) -> IndicatorMetadata {
        IndicatorMetadata {
            name: "MACD".to_string(),
            category: IndicatorCategory::Momentum,
            description: "Moving Average Convergence Divergence".to_string(),
            default_params: serde_json::json!({
                "fast_period": 12,
                "slow_period": 26,
                "signal_period": 9
            }),
            min_periods: 35,
            output_type: "MACD".to_string(),
        }
    }
}

impl IndicatorFactory for StochasticFactory {
    fn create(&self, params: &serde_json::Value) -> Result<Box<dyn Indicator>, AppError> {
        let k_period = params.get("k_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(14) as usize;
        let d_period = params.get("d_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;
        Ok(Box::new(Stochastic::new(k_period, d_period)))
    }

    fn metadata(&self) -> IndicatorMetadata {
        IndicatorMetadata {
            name: "Stochastic".to_string(),
            category: IndicatorCategory::Momentum,
            description: "Stochastic Oscillator".to_string(),
            default_params: serde_json::json!({
                "k_period": 14,
                "d_period": 3
            }),
            min_periods: 16,
            output_type: "Stochastic".to_string(),
        }
    }
}