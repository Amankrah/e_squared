use std::sync::Arc;
use rust_decimal::Decimal;
use crate::utils::errors::AppError;
use crate::exchange_connectors::Kline;
use super::service::{IndicatorService, IndicatorServiceConfig};
use super::integration::{IndicatorContext, utils};

/// Example: Enhanced DCA Strategy with Indicator Service
/// This shows how strategies should use the new indicator system
pub struct EnhancedDCAStrategy {
    symbol: String,
    interval: String,
    indicator_context: IndicatorContext,
    config: DCAStrategyConfig,
}

#[derive(Debug, Clone)]
pub struct DCAStrategyConfig {
    pub base_amount: Decimal,
    pub rsi_period: usize,
    pub rsi_oversold: Decimal,
    pub rsi_overbought: Decimal,
    pub ema_fast: usize,
    pub ema_slow: usize,
}

impl EnhancedDCAStrategy {
    /// Create a new enhanced DCA strategy
    pub async fn new(
        symbol: String,
        interval: String,
        config: DCAStrategyConfig,
        indicator_service: Arc<IndicatorService>,
    ) -> Result<Self, AppError> {
        let indicator_context = IndicatorContext::new(
            symbol.clone(),
            interval.clone(),
            indicator_service,
        );

        // Setup required indicators
        let indicators = vec![
            ("RSI", serde_json::json!({"period": config.rsi_period})),
            ("EMA", serde_json::json!({"period": config.ema_fast})),
            ("EMA", serde_json::json!({"period": config.ema_slow})),
        ];

        utils::setup_strategy_indicators(
            &indicator_context.indicator_service,
            &symbol,
            &interval,
            &indicators,
        ).await?;

        Ok(Self {
            symbol,
            interval,
            indicator_context,
            config,
        })
    }

    /// Analyze market conditions using indicators
    pub async fn should_buy(&self) -> Result<bool, AppError> {
        // Get RSI value
        let rsi = self.indicator_context
            .rsi(self.config.rsi_period)
            .await?;

        // Get EMA values
        let ema_fast = self.indicator_context
            .ema(self.config.ema_fast)
            .await?;

        let ema_slow = self.indicator_context
            .ema(self.config.ema_slow)
            .await?;

        // Apply strategy logic
        match (rsi, ema_fast, ema_slow) {
            (Some(rsi_val), Some(fast), Some(slow)) => {
                // Buy conditions:
                // 1. RSI indicates oversold
                // 2. Fast EMA > Slow EMA (uptrend)
                let oversold = rsi_val <= self.config.rsi_oversold;
                let uptrend = fast > slow;

                Ok(oversold && uptrend)
            }
            _ => {
                // Not enough data yet
                Ok(false)
            }
        }
    }

    /// Calculate dynamic buy amount based on RSI
    pub async fn calculate_amount(&self) -> Result<Decimal, AppError> {
        let rsi = self.indicator_context
            .rsi(self.config.rsi_period)
            .await?;

        match rsi {
            Some(rsi_val) => {
                // More oversold = higher multiplier
                let multiplier = if rsi_val <= Decimal::from(20) {
                    Decimal::from(2) // 2x for extremely oversold
                } else if rsi_val <= Decimal::from(30) {
                    Decimal::from(15) / Decimal::from(10) // 1.5x for oversold
                } else {
                    Decimal::from(1) // Normal amount
                };

                Ok(self.config.base_amount * multiplier)
            }
            None => Ok(self.config.base_amount)
        }
    }

    /// Update strategy with new kline data
    pub async fn update(&self, kline: &Kline) -> Result<(), AppError> {
        // Update indicators through the service
        self.indicator_context.indicator_service
            .update_with_kline(&self.symbol, &self.interval, kline)
            .await
    }
}

/// Example: Multi-indicator trend strategy
pub struct TrendFollowStrategy {
    indicator_context: IndicatorContext,
    config: TrendStrategyConfig,
}

#[derive(Debug, Clone)]
pub struct TrendStrategyConfig {
    pub ema_periods: Vec<usize>,
    pub rsi_period: usize,
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
}

impl TrendFollowStrategy {
    pub async fn new(
        symbol: String,
        interval: String,
        config: TrendStrategyConfig,
        indicator_service: Arc<IndicatorService>,
    ) -> Result<Self, AppError> {
        let indicator_context = IndicatorContext::new(symbol.clone(), interval.clone(), indicator_service);

        // Setup multiple indicators
        let mut indicators = Vec::new();

        // Multiple EMAs
        for period in &config.ema_periods {
            indicators.push(("EMA", serde_json::json!({"period": period})));
        }

        // RSI
        indicators.push(("RSI", serde_json::json!({"period": config.rsi_period})));

        // MACD
        indicators.push(("MACD", serde_json::json!({
            "fast_period": config.macd_fast,
            "slow_period": config.macd_slow,
            "signal_period": config.macd_signal
        })));

        utils::setup_strategy_indicators(
            &indicator_context.indicator_service,
            &symbol,
            &interval,
            &indicators,
        ).await?;

        Ok(Self {
            indicator_context,
            config,
        })
    }

    /// Comprehensive trend analysis
    pub async fn analyze_trend(&self) -> Result<TrendSignal, AppError> {
        // Get all EMA values
        let mut emas = Vec::new();
        for period in &self.config.ema_periods {
            if let Some(ema_val) = self.indicator_context.ema(*period).await? {
                emas.push(ema_val);
            }
        }

        // Get RSI
        let rsi = self.indicator_context.rsi(self.config.rsi_period).await?;

        // Get MACD
        let macd = self.indicator_context.macd(
            self.config.macd_fast,
            self.config.macd_slow,
            self.config.macd_signal,
        ).await?;

        // Analyze trend strength
        let ema_trend = self.analyze_ema_alignment(&emas);
        let momentum = self.analyze_momentum(rsi, macd);

        Ok(TrendSignal {
            direction: ema_trend,
            strength: momentum,
            confidence: self.calculate_confidence(&emas, rsi, macd),
        })
    }

    fn analyze_ema_alignment(&self, emas: &[Decimal]) -> TrendDirection {
        if emas.len() < 2 {
            return TrendDirection::Sideways;
        }

        // Check if EMAs are in ascending order (bullish) or descending (bearish)
        let ascending = emas.windows(2).all(|w| w[0] < w[1]);
        let descending = emas.windows(2).all(|w| w[0] > w[1]);

        if ascending {
            TrendDirection::Bullish
        } else if descending {
            TrendDirection::Bearish
        } else {
            TrendDirection::Sideways
        }
    }

    fn analyze_momentum(&self, rsi: Option<Decimal>, macd: Option<(Decimal, Decimal, Decimal)>) -> MomentumStrength {
        let mut score = 0;

        // RSI contribution
        if let Some(rsi_val) = rsi {
            if rsi_val > Decimal::from(60) {
                score += 1; // Bullish momentum
            } else if rsi_val < Decimal::from(40) {
                score -= 1; // Bearish momentum
            }
        }

        // MACD contribution
        if let Some((macd_line, signal_line, histogram)) = macd {
            if macd_line > signal_line {
                score += 1; // Bullish
            } else {
                score -= 1; // Bearish
            }

            if histogram > Decimal::ZERO {
                score += 1; // Increasing momentum
            } else {
                score -= 1; // Decreasing momentum
            }
        }

        match score {
            2..=4 => MomentumStrength::Strong,
            1 => MomentumStrength::Weak,
            0 => MomentumStrength::Neutral,
            -1 => MomentumStrength::WeakNegative,
            -4..=-2 => MomentumStrength::StrongNegative,
            _ => MomentumStrength::Neutral,
        }
    }

    fn calculate_confidence(&self, emas: &[Decimal], rsi: Option<Decimal>, macd: Option<(Decimal, Decimal, Decimal)>) -> Decimal {
        let mut confidence = Decimal::ZERO;
        let mut factors = 0;

        // EMA alignment confidence
        if !emas.is_empty() {
            confidence += Decimal::from(3) / Decimal::from(10); // 0.3
            factors += 1;
        }

        // RSI confidence
        if rsi.is_some() {
            confidence += Decimal::from(3) / Decimal::from(10); // 0.3
            factors += 1;
        }

        // MACD confidence
        if macd.is_some() {
            confidence += Decimal::from(4) / Decimal::from(10); // 0.4
            factors += 1;
        }

        if factors > 0 {
            confidence
        } else {
            Decimal::ZERO
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrendSignal {
    pub direction: TrendDirection,
    pub strength: MomentumStrength,
    pub confidence: Decimal,
}

#[derive(Debug, Clone)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Sideways,
}

#[derive(Debug, Clone)]
pub enum MomentumStrength {
    Strong,
    Weak,
    Neutral,
    WeakNegative,
    StrongNegative,
}

/// Example usage and testing
pub mod examples {
    use super::*;

    /// Setup example for backtesting
    pub async fn setup_backtest_with_indicators() -> Result<(), AppError> {
        // Create indicator service
        let config = IndicatorServiceConfig {
            max_cache_per_symbol: 10000,
            value_ttl_minutes: 120,
            enable_profiling: true,
            batch_size: 500,
        };

        let indicator_service = Arc::new(IndicatorService::new(config));

        // Create DCA strategy
        let dca_config = DCAStrategyConfig {
            base_amount: Decimal::from(100),
            rsi_period: 14,
            rsi_oversold: Decimal::from(30),
            rsi_overbought: Decimal::from(70),
            ema_fast: 12,
            ema_slow: 26,
        };

        let _dca_strategy = EnhancedDCAStrategy::new(
            "BTCUSDT".to_string(),
            "1h".to_string(),
            dca_config,
            Arc::clone(&indicator_service),
        ).await?;

        println!("✅ DCA Strategy initialized with indicator service");

        // Create trend strategy
        let trend_config = TrendStrategyConfig {
            ema_periods: vec![9, 21, 55],
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
        };

        let _trend_strategy = TrendFollowStrategy::new(
            "ETHUSDT".to_string(),
            "4h".to_string(),
            trend_config,
            indicator_service,
        ).await?;

        println!("✅ Trend Strategy initialized with indicator service");

        Ok(())
    }
}