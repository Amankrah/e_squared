use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use rust_decimal::Decimal;

use crate::backtesting::engine::BacktestEngine;
// Note: ExecutionEngine integration requires engine structure modifications
use crate::exchange_connectors::Kline;
use crate::utils::errors::AppError;
use super::service::{IndicatorService, IndicatorServiceConfig};
use super::core::{IndicatorValue, IndicatorResult};

/// Enhanced context for strategies with indicator access
#[derive(Clone)]
pub struct IndicatorContext {
    pub symbol: String,
    pub interval: String,
    pub indicator_service: Arc<IndicatorService>,
}

impl IndicatorContext {
    /// Create a new indicator context
    pub fn new(symbol: String, interval: String, indicator_service: Arc<IndicatorService>) -> Self {
        Self {
            symbol,
            interval,
            indicator_service,
        }
    }

    /// Get indicator value with automatic caching
    pub async fn get_indicator(
        &self,
        name: &str,
        params: &serde_json::Value,
    ) -> Result<Option<IndicatorValue>, AppError> {
        self.indicator_service
            .get_value(&self.symbol, &self.interval, name, params)
            .await
    }

    /// Get SMA value
    pub async fn sma(&self, period: usize) -> Result<Option<Decimal>, AppError> {
        let params = serde_json::json!({"period": period});
        if let Some(value) = self.get_indicator("SMA", &params).await? {
            match value.value {
                IndicatorResult::Single(val) => Ok(Some(val)),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get EMA value
    pub async fn ema(&self, period: usize) -> Result<Option<Decimal>, AppError> {
        let params = serde_json::json!({"period": period});
        if let Some(value) = self.get_indicator("EMA", &params).await? {
            match value.value {
                IndicatorResult::Single(val) => Ok(Some(val)),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get RSI value
    pub async fn rsi(&self, period: usize) -> Result<Option<Decimal>, AppError> {
        let params = serde_json::json!({"period": period});
        if let Some(value) = self.get_indicator("RSI", &params).await? {
            match value.value {
                IndicatorResult::Single(val) => Ok(Some(val)),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get MACD values
    pub async fn macd(
        &self,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Result<Option<(Decimal, Decimal, Decimal)>, AppError> {
        let params = serde_json::json!({
            "fast_period": fast_period,
            "slow_period": slow_period,
            "signal_period": signal_period
        });

        if let Some(value) = self.get_indicator("MACD", &params).await? {
            match value.value {
                IndicatorResult::MACD { macd_line, signal_line, histogram } => {
                    Ok(Some((macd_line, signal_line, histogram)))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get Bollinger Bands
    pub async fn bollinger_bands(
        &self,
        period: usize,
        std_dev: Decimal,
    ) -> Result<Option<(Decimal, Decimal, Decimal)>, AppError> {
        let params = serde_json::json!({
            "period": period,
            "std_dev": std_dev
        });

        if let Some(value) = self.get_indicator("BollingerBands", &params).await? {
            match value.value {
                IndicatorResult::Bands { upper, middle, lower } => {
                    Ok(Some((upper, middle, lower)))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get Stochastic values
    pub async fn stochastic(
        &self,
        k_period: usize,
        d_period: usize,
    ) -> Result<Option<(Decimal, Decimal)>, AppError> {
        let params = serde_json::json!({
            "k_period": k_period,
            "d_period": d_period
        });

        if let Some(value) = self.get_indicator("Stochastic", &params).await? {
            match value.value {
                IndicatorResult::Stochastic { k_percent, d_percent } => {
                    Ok(Some((k_percent, d_percent)))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Preload indicators for backtesting
    pub async fn preload_indicators(
        &self,
        klines: &[Kline],
        indicators: &[(&str, serde_json::Value)],
    ) -> Result<(), AppError> {
        info!("Preloading {} indicators with {} klines", indicators.len(), klines.len());

        // Create indicator instances
        for (name, params) in indicators {
            let _indicator = self.indicator_service
                .get_indicator(&self.symbol, &self.interval, name, params)
                .await?;
        }

        // Batch update with historical data
        self.indicator_service
            .update_batch(&self.symbol, &self.interval, klines)
            .await?;

        info!("Preloading complete for {}", self.symbol);
        Ok(())
    }
}

/// Extension trait for BacktestEngine
pub trait BacktestEngineIndicatorExt {
    /// Run backtest with indicator service
    fn with_indicators(self, indicator_service: Arc<IndicatorService>) -> BacktestEngineWithIndicators;
}

impl BacktestEngineIndicatorExt for BacktestEngine {
    fn with_indicators(self, indicator_service: Arc<IndicatorService>) -> BacktestEngineWithIndicators {
        BacktestEngineWithIndicators {
            engine: self,
            indicator_service,
        }
    }
}

/// Backtesting engine enhanced with indicator support
pub struct BacktestEngineWithIndicators {
    engine: BacktestEngine,
    indicator_service: Arc<IndicatorService>,
}

impl BacktestEngineWithIndicators {
    // Note: This function requires modifications to BacktestConfig to include IndicatorContext
    // pub async fn run_backtest_with_indicators(
    //     &self,
    //     config: crate::backtesting::types::BacktestConfig,
    //     indicator_specs: Vec<(&str, serde_json::Value)>,
    // ) -> Result<crate::backtesting::types::BacktestResult, AppError> {
    //     // Implementation requires BacktestConfig modifications
    //     todo!("Implement after BacktestConfig includes IndicatorContext")
    // }
}

/// Extension trait for ExecutionEngine
pub trait ExecutionEngineIndicatorExt {
    /// Add indicator service to execution engine
    fn with_indicators(
        &mut self,
        indicator_service: Arc<IndicatorService>,
    ) -> Result<(), AppError>;
}

// Note: You'll need to modify ExecutionEngine to support this
// impl ExecutionEngineIndicatorExt for ExecutionEngine {
//     fn with_indicators(
//         &mut self,
//         indicator_service: Arc<IndicatorService>,
//     ) -> Result<(), AppError> {
//         // This would require modifying the ExecutionEngine structure
//         // to include indicator service integration
//         Ok(())
//     }
// }

/// Live market data processor with indicator updates
pub struct LiveIndicatorProcessor {
    indicator_service: Arc<IndicatorService>,
    active_symbols: Arc<RwLock<Vec<(String, String)>>>, // (symbol, interval) pairs
}

impl LiveIndicatorProcessor {
    /// Create a new live processor
    pub fn new(indicator_service: Arc<IndicatorService>) -> Self {
        Self {
            indicator_service,
            active_symbols: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start monitoring a symbol/interval pair
    pub async fn start_monitoring(&self, symbol: String, interval: String) -> Result<(), AppError> {
        let mut symbols = self.active_symbols.write().await;
        let pair = (symbol, interval);

        if !symbols.contains(&pair) {
            symbols.push(pair);
            info!("Started monitoring indicators for {}:{}", symbols.last().unwrap().0, symbols.last().unwrap().1);
        }

        Ok(())
    }

    /// Stop monitoring a symbol/interval pair
    pub async fn stop_monitoring(&self, symbol: &str, interval: &str) -> Result<(), AppError> {
        let mut symbols = self.active_symbols.write().await;
        symbols.retain(|(s, i)| s != symbol || i != interval);
        info!("Stopped monitoring indicators for {}:{}", symbol, interval);
        Ok(())
    }

    /// Process incoming kline data
    pub async fn process_kline(&self, symbol: &str, interval: &str, kline: &Kline) -> Result<(), AppError> {
        let symbols = self.active_symbols.read().await;
        let pair = (symbol.to_string(), interval.to_string());

        if symbols.contains(&pair) {
            self.indicator_service
                .update_with_kline(symbol, interval, kline)
                .await?;

            debug!("Processed kline for indicators: {}:{}", symbol, interval);
        }

        Ok(())
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> LiveProcessorStats {
        let symbols = self.active_symbols.read().await;
        let service_stats = self.indicator_service.get_stats().await;

        LiveProcessorStats {
            monitored_symbols: symbols.len(),
            active_indicators: service_stats.active_indicators,
            cached_values: service_stats.cached_values,
        }
    }
}

#[derive(Debug)]
pub struct LiveProcessorStats {
    pub monitored_symbols: usize,
    pub active_indicators: usize,
    pub cached_values: usize,
}

/// Utility functions for indicator integration
pub mod utils {
    use super::*;

    /// Create a default indicator service
    pub fn create_indicator_service() -> Arc<IndicatorService> {
        let config = IndicatorServiceConfig::default();
        Arc::new(IndicatorService::new(config))
    }

    /// Create indicator service with custom config
    pub fn create_indicator_service_with_config(config: IndicatorServiceConfig) -> Arc<IndicatorService> {
        Arc::new(IndicatorService::new(config))
    }

    /// Setup indicators for a strategy
    pub async fn setup_strategy_indicators(
        service: &IndicatorService,
        symbol: &str,
        interval: &str,
        indicators: &[(&str, serde_json::Value)],
    ) -> Result<Vec<String>, AppError> {
        let mut created = Vec::new();

        for (name, params) in indicators {
            let _indicator = service
                .get_indicator(symbol, interval, name, params)
                .await?;
            created.push(name.to_string());
        }

        Ok(created)
    }
}