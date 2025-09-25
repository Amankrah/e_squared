use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::exchange_connectors::Kline;
use crate::utils::errors::AppError;

/// Trait for all technical indicators
#[async_trait]
pub trait Indicator: Send + Sync {
    /// Get the name of the indicator
    fn name(&self) -> String;

    /// Get the current value of the indicator
    fn value(&self) -> Option<IndicatorValue>;

    /// Update the indicator with new data
    async fn update(&mut self, kline: &Kline) -> Result<(), AppError>;

    /// Update with batch of historical data
    async fn update_batch(&mut self, klines: &[Kline]) -> Result<(), AppError>;

    /// Reset the indicator state
    fn reset(&mut self);

    /// Check if the indicator has enough data to produce a value
    fn is_ready(&self) -> bool;

    /// Get the minimum number of periods required
    fn min_periods(&self) -> usize;

    /// Clone the indicator
    fn clone_box(&self) -> Box<dyn Indicator>;

    /// Get configuration as JSON
    fn config(&self) -> serde_json::Value;
}

/// Indicator value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorValue {
    pub value: IndicatorResult,
    pub timestamp: DateTime<Utc>,
    pub confidence: Decimal,
}

/// Different types of indicator results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorResult {
    Single(Decimal),
    Double(Decimal, Decimal),
    Triple(Decimal, Decimal, Decimal),
    Bands {
        upper: Decimal,
        middle: Decimal,
        lower: Decimal,
    },
    MACD {
        macd_line: Decimal,
        signal_line: Decimal,
        histogram: Decimal,
    },
    Stochastic {
        k_percent: Decimal,
        d_percent: Decimal,
    },
    Custom(HashMap<String, Decimal>),
}

/// Base trait for stateful indicators
pub trait StatefulIndicator {
    /// Update internal state with new value
    fn update_state(&mut self, value: Decimal);

    /// Get historical buffer
    fn buffer(&self) -> &[Decimal];

    /// Clear historical buffer
    fn clear_buffer(&mut self);
}

/// Indicator factory for creating indicators
pub trait IndicatorFactory: Send + Sync {
    /// Create an indicator instance
    fn create(&self, params: &serde_json::Value) -> Result<Box<dyn Indicator>, AppError>;

    /// Get indicator metadata
    fn metadata(&self) -> IndicatorMetadata;
}

/// Metadata about an indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorMetadata {
    pub name: String,
    pub category: IndicatorCategory,
    pub description: String,
    pub default_params: serde_json::Value,
    pub min_periods: usize,
    pub output_type: String,
}

/// Indicator categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorCategory {
    Trend,
    Momentum,
    Volatility,
    Volume,
    Support,
    Custom,
}

/// Indicator cache key
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IndicatorCacheKey {
    pub symbol: String,
    pub interval: String,
    pub indicator_name: String,
    pub params_hash: u64,
}

/// Cached indicator value
#[derive(Debug, Clone)]
pub struct CachedIndicatorValue {
    pub value: IndicatorValue,
    pub last_update: DateTime<Utc>,
    pub kline_count: usize,
}