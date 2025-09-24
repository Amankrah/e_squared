use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration, Datelike, Timelike};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::strategies::core::{
    Strategy, StrategyMetadata, StrategyMode, StrategyContext, StrategySignal,
    StrategyCategory, RiskLevel, BacktestableStrategy, LiveExecutableStrategy,
    ControllableStrategy, QuantityType, IndicatorValue,
};
use crate::strategies::indicators;
use crate::utils::errors::AppError;

use super::config::DCAConfig;
use super::types::*;

/// Dollar Cost Averaging Strategy Implementation
pub struct DCAStrategy {
    /// Strategy configuration
    config: Option<DCAConfig>,
    /// Current execution state
    state: DCAState,
    /// Execution history
    execution_history: Vec<DCAExecution>,
    /// Is strategy currently paused
    is_paused: bool,
    /// Is strategy currently running (for live execution)
    is_running: bool,
    /// Last signal reason
    last_signal_reason: String,
    /// Strategy metadata
    metadata: StrategyMetadata,
}

impl DCAStrategy {
    /// Create a new DCA strategy instance
    pub fn new() -> Self {
        Self {
            config: None,
            state: DCAState::default(),
            execution_history: Vec::new(),
            is_paused: false,
            is_running: false,
            last_signal_reason: String::new(),
            metadata: Self::create_metadata(),
        }
    }

    /// Create strategy metadata
    fn create_metadata() -> StrategyMetadata {
        StrategyMetadata {
            id: "dca_v2".to_string(),
            name: "Dollar Cost Averaging v2".to_string(),
            description: "Advanced Dollar Cost Averaging strategy with multiple variants including RSI-based, volatility-based, and dip-buying approaches".to_string(),
            version: "2.0.0".to_string(),
            author: "E-Squared Trading Bot".to_string(),
            category: StrategyCategory::DCA,
            risk_level: RiskLevel::Conservative,
            supported_modes: vec![
                StrategyMode::Backtest,
                StrategyMode::Paper,
                StrategyMode::Live,
            ],
            min_balance: Some(Decimal::from(100)),
            max_positions: Some(1), // DCA typically focuses on one asset
            supported_intervals: vec![
                "1m".to_string(), "5m".to_string(), "15m".to_string(),
                "30m".to_string(), "1h".to_string(), "4h".to_string(),
                "1d".to_string()
            ],
            tags: vec![
                "dca".to_string(),
                "conservative".to_string(),
                "systematic".to_string(),
                "long-term".to_string(),
                "automated".to_string(),
            ],
        }
    }

    /// Check if it's time to execute DCA
    fn should_execute(&self, context: &StrategyContext) -> bool {
        let config = match &self.config {
            Some(c) => c,
            None => return false,
        };

        // Check if strategy is paused
        if self.is_paused {
            return false;
        }

        // Check time-based execution
        if let Some(last_execution) = self.state.last_execution {
            let time_diff = context.current_time - last_execution;
            let required_interval = Duration::minutes(config.frequency.to_minutes() as i64);

            if time_diff < required_interval {
                return false;
            }
        }

        // Check filters
        if !self.passes_filters(context, config) {
            return false;
        }

        // Check position size limits
        if let Some(max_position) = config.max_position_size {
            let current_position_value = context.position_value();
            if current_position_value >= max_position {
                return false;
            }
        }

        true
    }

    /// Check if execution passes all filters
    fn passes_filters(&self, context: &StrategyContext, config: &DCAConfig) -> bool {
        let filters = &config.filters;

        // Check allowed hours
        if let Some(ref allowed_hours) = filters.allowed_hours {
            let current_hour = context.current_time.hour() as u8;
            if !allowed_hours.contains(&current_hour) {
                return false;
            }
        }

        // Check allowed weekdays
        if let Some(ref allowed_weekdays) = filters.allowed_weekdays {
            let current_weekday = context.current_time.weekday().num_days_from_sunday() as u8;
            if !allowed_weekdays.contains(&current_weekday) {
                return false;
            }
        }

        // Check minimum volume
        if let Some(min_volume) = filters.min_volume_threshold {
            if let Some(volume_24h) = context.market_data.volume_24h {
                if volume_24h < min_volume {
                    return false;
                }
            }
        }

        // Check maximum spread
        if let Some(max_spread_pct) = filters.max_spread_percentage {
            if let Some(spread_pct) = context.spread_percentage() {
                if spread_pct > max_spread_pct {
                    return false;
                }
            }
        }

        true
    }

    /// Calculate the amount to invest based on strategy type
    fn calculate_investment_amount(&mut self, context: &StrategyContext) -> Result<Decimal, AppError> {
        let config = self.config.as_ref().ok_or_else(|| {
            AppError::BadRequest("Strategy not initialized".to_string())
        })?;

        let base_amount = config.base_amount;
        let mut multiplier = Decimal::from(1);
        let mut reasons = Vec::new();
        let mut market_conditions = MarketConditions::default();

        match config.strategy_type {
            DCAType::Simple => {
                reasons.push("Simple DCA".to_string());
            }

            DCAType::RSIBased => {
                if let Some(ref rsi_config) = config.rsi_config {
                    multiplier = self.calculate_rsi_multiplier(context, rsi_config, &mut market_conditions, &mut reasons)?;
                }
            }

            DCAType::VolatilityBased => {
                if let Some(ref vol_config) = config.volatility_config {
                    multiplier = self.calculate_volatility_multiplier(context, vol_config, &mut market_conditions, &mut reasons)?;
                }
            }

            DCAType::Dynamic => {
                multiplier = self.calculate_dynamic_multiplier(context, config, &mut market_conditions, &mut reasons)?;
            }

            DCAType::DipBuying => {
                multiplier = self.calculate_dip_multiplier(context, config, &mut market_conditions, &mut reasons)?;
            }

            DCAType::SentimentBased => {
                if let Some(ref sentiment_config) = config.sentiment_config {
                    multiplier = self.calculate_sentiment_multiplier(sentiment_config, &mut market_conditions, &mut reasons)?;
                }
            }
        }

        // Apply limits
        let final_amount = base_amount * multiplier;
        let clamped_amount = self.apply_amount_limits(final_amount, config);

        self.last_signal_reason = format!(
            "{} (multiplier: {:.2}, amount: {})",
            reasons.join(", "),
            multiplier,
            clamped_amount
        );

        Ok(clamped_amount)
    }

    /// Calculate RSI-based multiplier
    fn calculate_rsi_multiplier(
        &self,
        context: &StrategyContext,
        rsi_config: &RSIConfig,
        market_conditions: &mut MarketConditions,
        reasons: &mut Vec<String>,
    ) -> Result<Decimal, AppError> {
        if context.historical_data.len() < rsi_config.period {
            return Ok(rsi_config.normal_multiplier);
        }

        let rsi = indicators::rsi(&context.historical_data, rsi_config.period)
            .ok_or_else(|| AppError::BadRequest("Failed to calculate RSI".to_string()))?;

        market_conditions.rsi = Some(rsi);

        let multiplier = if rsi <= rsi_config.oversold_threshold {
            reasons.push(format!("RSI oversold ({:.2})", rsi));
            rsi_config.oversold_multiplier
        } else if rsi >= rsi_config.overbought_threshold {
            reasons.push(format!("RSI overbought ({:.2})", rsi));
            rsi_config.overbought_multiplier
        } else {
            reasons.push(format!("RSI normal ({:.2})", rsi));
            rsi_config.normal_multiplier
        };

        Ok(multiplier)
    }

    /// Calculate volatility-based multiplier
    fn calculate_volatility_multiplier(
        &self,
        context: &StrategyContext,
        vol_config: &VolatilityConfig,
        market_conditions: &mut MarketConditions,
        reasons: &mut Vec<String>,
    ) -> Result<Decimal, AppError> {
        if context.historical_data.len() < vol_config.period {
            return Ok(vol_config.normal_multiplier);
        }

        let volatility = self.calculate_volatility(&context.historical_data, vol_config.period);
        market_conditions.volatility = Some(volatility);

        let multiplier = if volatility <= vol_config.low_threshold {
            reasons.push(format!("Low volatility ({:.2}%)", volatility));
            vol_config.low_volatility_multiplier
        } else if volatility >= vol_config.high_threshold {
            reasons.push(format!("High volatility ({:.2}%)", volatility));
            vol_config.high_volatility_multiplier
        } else {
            reasons.push(format!("Normal volatility ({:.2}%)", volatility));
            vol_config.normal_multiplier
        };

        Ok(multiplier)
    }

    /// Calculate dynamic multiplier combining multiple factors
    fn calculate_dynamic_multiplier(
        &self,
        context: &StrategyContext,
        config: &DCAConfig,
        market_conditions: &mut MarketConditions,
        reasons: &mut Vec<String>,
    ) -> Result<Decimal, AppError> {
        let factors = config.dynamic_factors.as_ref()
            .ok_or_else(|| AppError::BadRequest("Dynamic factors not configured".to_string()))?;

        let mut total_multiplier = Decimal::from(1);

        // RSI factor
        if factors.rsi_weight > Decimal::ZERO {
            if let Some(ref rsi_config) = config.rsi_config {
                let rsi_multiplier = self.calculate_rsi_multiplier(context, rsi_config, market_conditions, reasons)?;
                total_multiplier += (rsi_multiplier - Decimal::from(1)) * factors.rsi_weight;
            }
        }

        // Volatility factor
        if factors.volatility_weight > Decimal::ZERO {
            if let Some(ref vol_config) = config.volatility_config {
                let vol_multiplier = self.calculate_volatility_multiplier(context, vol_config, market_conditions, reasons)?;
                total_multiplier += (vol_multiplier - Decimal::from(1)) * factors.volatility_weight;
            }
        }

        // Apply limits
        total_multiplier = total_multiplier.max(factors.min_multiplier).min(factors.max_multiplier);

        reasons.push(format!("Dynamic multiplier: {:.2}", total_multiplier));
        Ok(total_multiplier)
    }

    /// Calculate dip-buying multiplier
    fn calculate_dip_multiplier(
        &self,
        context: &StrategyContext,
        config: &DCAConfig,
        market_conditions: &mut MarketConditions,
        reasons: &mut Vec<String>,
    ) -> Result<Decimal, AppError> {
        let dip_levels = config.dip_levels.as_ref()
            .ok_or_else(|| AppError::BadRequest("Dip levels not configured".to_string()))?;

        // Get reference price
        let reference_price = match config.reference_price {
            Some(price) => price,
            None => {
                // Calculate reference price from recent high
                let period_days = config.reference_period_days.unwrap_or(30);
                let lookback_candles = period_days * 24; // Assuming hourly data
                let start_idx = context.historical_data.len().saturating_sub(lookback_candles as usize);
                let recent_high = context.historical_data[start_idx..]
                    .iter()
                    .map(|k| k.high)
                    .max()
                    .unwrap_or(context.current_price);
                recent_high
            }
        };

        let current_price = context.current_price;
        let price_drop_pct = ((reference_price - current_price) / reference_price) * Decimal::from(100);

        market_conditions.price_change_percentage = Some(-price_drop_pct);

        // Find applicable dip level
        let mut best_multiplier = Decimal::from(1);
        for (i, dip_level) in dip_levels.iter().enumerate() {
            if price_drop_pct >= dip_level.price_drop_percentage {
                // Check if this level hasn't exceeded max triggers
                let level_key = format!("level_{}", i);
                let current_triggers = self.state.dip_level_executions.get(&level_key).unwrap_or(&0);

                if let Some(max_triggers) = dip_level.max_triggers {
                    if *current_triggers >= max_triggers {
                        continue;
                    }
                }

                best_multiplier = dip_level.amount_multiplier;
                reasons.push(format!(
                    "Dip level {} triggered (price drop: {:.2}%)",
                    i + 1,
                    price_drop_pct
                ));
                break;
            }
        }

        if best_multiplier == Decimal::from(1) {
            reasons.push("No dip level triggered".to_string());
        }

        Ok(best_multiplier)
    }

    /// Calculate sentiment-based multiplier
    fn calculate_sentiment_multiplier(
        &self,
        _sentiment_config: &SentimentConfig,
        market_conditions: &mut MarketConditions,
        reasons: &mut Vec<String>,
    ) -> Result<Decimal, AppError> {
        // This would integrate with external sentiment APIs
        // For now, return neutral multiplier
        market_conditions.sentiment_score = Some(Decimal::ZERO);
        reasons.push("Sentiment analysis not implemented".to_string());
        Ok(Decimal::from(1))
    }

    /// Apply amount limits to the calculated investment amount
    fn apply_amount_limits(&self, amount: Decimal, config: &DCAConfig) -> Decimal {
        let mut final_amount = amount;

        if let Some(min_amount) = config.min_single_amount {
            final_amount = final_amount.max(min_amount);
        }

        if let Some(max_amount) = config.max_single_amount {
            final_amount = final_amount.min(max_amount);
        }

        final_amount
    }

    /// Calculate volatility using standard deviation of returns
    fn calculate_volatility(&self, data: &[crate::exchange_connectors::Kline], period: usize) -> Decimal {
        if data.len() < period {
            return Decimal::ZERO;
        }

        let recent_data = &data[data.len() - period..];

        // Calculate returns
        let returns: Vec<Decimal> = recent_data
            .windows(2)
            .map(|window| {
                let prev_price = window[0].close;
                let curr_price = window[1].close;
                if prev_price > Decimal::ZERO {
                    ((curr_price - prev_price) / prev_price).abs()
                } else {
                    Decimal::ZERO
                }
            })
            .collect();

        if returns.is_empty() {
            return Decimal::ZERO;
        }

        // Calculate mean
        let mean = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

        // Calculate variance
        let variance = returns
            .iter()
            .map(|r| {
                let diff = *r - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(returns.len());

        // Convert to annualized percentage (approximate)
        variance * Decimal::from(100)
    }

    /// Record execution in state and history
    fn record_execution(&mut self, context: &StrategyContext, amount: Decimal, market_conditions: MarketConditions) {
        let quantity = amount / context.current_price;

        // Update state
        self.state.last_execution = Some(context.current_time);
        self.state.total_invested += amount;
        self.state.total_quantity += quantity;
        self.state.purchase_count += 1;

        // Recalculate average price
        if self.state.total_quantity > Decimal::ZERO {
            self.state.average_price = self.state.total_invested / self.state.total_quantity;
        }

        // Add to execution history
        let execution = DCAExecution {
            timestamp: context.current_time,
            amount,
            quantity,
            price: context.current_price,
            strategy_type: self.config.as_ref().unwrap().strategy_type.clone(),
            reason: self.last_signal_reason.clone(),
            multiplier: amount / self.config.as_ref().unwrap().base_amount,
            market_conditions,
        };

        self.execution_history.push(execution);

        // Keep only last 1000 executions to prevent memory bloat
        if self.execution_history.len() > 1000 {
            self.execution_history.remove(0);
        }
    }
}

#[async_trait]
impl Strategy for DCAStrategy {
    fn metadata(&self) -> StrategyMetadata {
        self.metadata.clone()
    }

    async fn initialize(
        &mut self,
        parameters: &Value,
        _mode: StrategyMode,
        _context: &StrategyContext,
    ) -> Result<(), AppError> {
        let config: DCAConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid DCA parameters: {}", e)))?;

        // Validate configuration
        config.validate()
            .map_err(|e| AppError::BadRequest(e))?;

        self.config = Some(config);
        self.state = DCAState::default();
        self.execution_history.clear();
        self.is_paused = false;
        self.last_signal_reason = "Strategy initialized".to_string();

        info!("DCA strategy initialized successfully");
        Ok(())
    }

    async fn analyze(
        &mut self,
        context: &StrategyContext,
    ) -> Result<Option<StrategySignal>, AppError> {
        if !self.should_execute(context) {
            return Ok(None);
        }

        let amount = self.calculate_investment_amount(context)?;

        if amount <= Decimal::ZERO {
            return Ok(None);
        }

        // Create signal
        let signal = StrategySignal::dca_buy(
            context.symbol.clone(),
            amount,
            self.last_signal_reason.clone(),
        );

        Ok(Some(signal))
    }

    fn validate_parameters(&self, parameters: &Value) -> Result<(), AppError> {
        let config: DCAConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;

        config.validate()
            .map_err(|e| AppError::BadRequest(e))?;

        Ok(())
    }

    fn parameter_schema(&self) -> Value {
        DCAConfig::json_schema()
    }

    fn get_state(&self) -> Result<Value, AppError> {
        serde_json::to_value(&self.state)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize state: {}", e)))
    }

    fn restore_state(&mut self, state: &Value) -> Result<(), AppError> {
        self.state = serde_json::from_value(state.clone())
            .map_err(|e| AppError::BadRequest(format!("Failed to deserialize state: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl LiveExecutableStrategy for DCAStrategy {
    async fn start_live_execution(&mut self, _context: &StrategyContext) -> Result<(), AppError> {
        self.is_running = true;
        info!("DCA strategy started for live execution");
        Ok(())
    }

    async fn stop_live_execution(&mut self) -> Result<(), AppError> {
        self.is_running = false;
        info!("DCA strategy stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn next_execution_time(&self) -> Option<DateTime<Utc>> {
        if let (Some(config), Some(last_execution)) = (&self.config, self.state.last_execution) {
            let interval_minutes = config.frequency.to_minutes() as i64;
            Some(last_execution + Duration::minutes(interval_minutes))
        } else {
            None
        }
    }
}

#[async_trait]
impl ControllableStrategy for DCAStrategy {
    async fn pause(&mut self) -> Result<(), AppError> {
        self.is_paused = true;
        info!("DCA strategy paused");
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), AppError> {
        self.is_paused = false;
        info!("DCA strategy resumed");
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }
}

impl Default for DCAStrategy {
    fn default() -> Self {
        Self::new()
    }
}