use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::strategies::core::{
    Strategy, StrategyMetadata, StrategyMode, StrategyContext, StrategySignal,
    StrategyCategory, RiskLevel, LiveExecutableStrategy, ControllableStrategy, QuantityType,
};
use crate::strategies::indicators::{self};
use crate::utils::errors::AppError;

use super::config::{RSIStrategyConfig, PositionSizingMethod};
use super::types::*;

/// RSI Strategy Implementation
pub struct RSIStrategy {
    /// Strategy configuration
    config: Option<RSIStrategyConfig>,
    /// Current execution state
    state: RSIStrategyState,
    /// Execution history
    execution_history: Vec<RSIExecution>,
    /// Is strategy currently paused
    is_paused: bool,
    /// Is strategy currently running (for live execution)
    is_running: bool,
    /// Last signal reason
    last_signal_reason: String,
    /// Strategy metadata
    metadata: StrategyMetadata,
}

impl RSIStrategy {
    /// Create a new RSI strategy instance
    pub fn new() -> Self {
        Self {
            config: None,
            state: RSIStrategyState::default(),
            execution_history: Vec::new(),
            is_paused: false,
            is_running: false,
            last_signal_reason: String::new(),
            metadata: Self::create_metadata(),
        }
    }

    /// Create strategy metadata
    pub fn create_metadata() -> StrategyMetadata {
        StrategyMetadata {
            id: "rsi_v1".to_string(),
            name: "RSI Strategy".to_string(),
            description: "Relative Strength Index momentum strategy with overbought/oversold signals and divergence detection".to_string(),
            version: "1.0.0".to_string(),
            author: "E-Squared Trading Bot".to_string(),
            category: StrategyCategory::TechnicalAnalysis,
            risk_level: RiskLevel::Moderate,
            supported_modes: vec![
                StrategyMode::Backtest,
                StrategyMode::Paper,
                StrategyMode::Live,
            ],
            min_balance: Some(Decimal::from(100)),
            max_positions: Some(1),
            supported_intervals: vec![
                "1m".to_string(), "5m".to_string(), "15m".to_string(),
                "30m".to_string(), "1h".to_string(), "4h".to_string(),
                "1d".to_string()
            ],
            tags: vec![
                "rsi".to_string(),
                "momentum".to_string(),
                "technical".to_string(),
                "oscillator".to_string(),
            ],
        }
    }

    /// Analyze RSI signal and market conditions
    fn analyze_rsi(&self, context: &StrategyContext, rsi_value: Decimal) -> Result<RSIAnalysis, AppError> {
        let config = self.config.as_ref().unwrap();
        let price = context.current_price;

        // Determine signal type based on RSI levels
        let signal = if rsi_value <= config.oversold_level {
            RSISignal::Oversold
        } else if rsi_value >= config.overbought_level {
            RSISignal::Overbought
        } else {
            RSISignal::None
        };

        // Calculate signal strength based on how extreme the RSI is
        let strength = match signal {
            RSISignal::Oversold => {
                let distance = config.oversold_level - rsi_value;
                if distance >= Decimal::from(20) {
                    RSISignalStrength::VeryStrong
                } else if distance >= Decimal::from(10) {
                    RSISignalStrength::Strong
                } else if distance >= Decimal::from(5) {
                    RSISignalStrength::Medium
                } else {
                    RSISignalStrength::Weak
                }
            }
            RSISignal::Overbought => {
                let distance = rsi_value - config.overbought_level;
                if distance >= Decimal::from(20) {
                    RSISignalStrength::VeryStrong
                } else if distance >= Decimal::from(10) {
                    RSISignalStrength::Strong
                } else if distance >= Decimal::from(5) {
                    RSISignalStrength::Medium
                } else {
                    RSISignalStrength::Weak
                }
            }
            _ => RSISignalStrength::Weak,
        };

        // Calculate confidence based on signal strength and market conditions
        let base_confidence = match strength {
            RSISignalStrength::VeryStrong => Decimal::new(95, 2), // 0.95
            RSISignalStrength::Strong => Decimal::new(8, 1),     // 0.8
            RSISignalStrength::Medium => Decimal::new(6, 1),     // 0.6
            RSISignalStrength::Weak => Decimal::new(4, 1),       // 0.4
        };

        // Adjust confidence based on volume and volatility if available
        let mut confidence = base_confidence;
        if let Some(volume) = context.market_data.volume_24h {
            if volume > Decimal::from(1000) { // Arbitrary threshold, should be configurable
                confidence += Decimal::new(1, 1); // Add 0.1
            }
        }

        // Cap confidence at 1.0
        confidence = confidence.min(Decimal::ONE);

        // Calculate RSI change
        let rsi_change = if let Some(prev_rsi) = self.state.previous_rsi {
            rsi_value - prev_rsi
        } else {
            Decimal::ZERO
        };

        Ok(RSIAnalysis {
            signal,
            strength,
            confidence,
            rsi_value,
            rsi_change,
            price,
            market_conditions: self.capture_market_conditions(context),
            divergence_info: None, // TODO: Implement divergence detection
        })
    }

    /// Capture current market conditions
    fn capture_market_conditions(&self, context: &StrategyContext) -> MarketConditions {
        MarketConditions {
            price: context.current_price,
            price_change_24h: context.market_data.price_change_24h,
            volume: context.market_data.volume_24h,
            volatility: None, // TODO: Calculate volatility
            support_level: None, // TODO: Calculate support/resistance
            resistance_level: None,
            trend: None, // TODO: Determine trend using SMA
        }
    }

    /// Apply signal filters
    fn passes_filters(&self, analysis: &RSIAnalysis, context: &StrategyContext) -> bool {
        let config = self.config.as_ref().unwrap();
        let filters = &config.signal_filters;

        // Volume filter
        if let Some(min_volume) = filters.min_volume {
            if let Some(volume) = context.market_data.volume_24h {
                if volume < min_volume {
                    debug!("Signal filtered out due to low volume: {} < {}", volume, min_volume);
                    return false;
                }
            }
        }

        // Spread filter
        if let Some(max_spread) = filters.max_spread_pct {
            if let Some(spread) = context.market_data.spread {
                let spread_pct = (spread / context.current_price) * Decimal::from(100);
                if spread_pct > max_spread {
                    debug!("Signal filtered out due to high spread: {}% > {}%", spread_pct, max_spread);
                    return false;
                }
            }
        }

        // RSI change filter
        if let Some(min_change) = filters.min_rsi_change {
            if analysis.rsi_change.abs() < min_change {
                debug!("Signal filtered out due to insufficient RSI change: {} < {}",
                       analysis.rsi_change.abs(), min_change);
                return false;
            }
        }

        true
    }

    /// Check risk management conditions
    fn check_risk_management(&self, context: &StrategyContext) -> Option<TradeSide> {
        let config = self.config.as_ref().unwrap();
        let risk_mgmt = &config.risk_management;

        // Check position and calculate P&L if in position
        if self.state.position != 0 {
            if let Some(entry_price) = self.state.entry_price {
                let current_price = context.current_price;

                // Calculate unrealized P&L percentage
                let pnl_pct = if self.state.position > 0 {
                    // Long position
                    (current_price - entry_price) / entry_price * Decimal::from(100)
                } else {
                    // Short position
                    (entry_price - current_price) / entry_price * Decimal::from(100)
                };

                // Check stop loss
                if let Some(stop_loss) = risk_mgmt.stop_loss_pct {
                    if pnl_pct <= -stop_loss {
                        return Some(if self.state.position > 0 { TradeSide::Sell } else { TradeSide::Buy });
                    }
                }

                // Check take profit
                if let Some(take_profit) = risk_mgmt.take_profit_pct {
                    if pnl_pct >= take_profit {
                        return Some(if self.state.position > 0 { TradeSide::Sell } else { TradeSide::Buy });
                    }
                }
            }
        }

        // Check consecutive losses
        if self.state.current_streak < 0 &&
           (-self.state.current_streak) >= risk_mgmt.max_consecutive_losses as i32 {
            warn!("Maximum consecutive losses reached: {}", -self.state.current_streak);
            return None; // Don't take new positions
        }

        None
    }

    /// Calculate position size based on configuration
    fn calculate_position_size(&self, context: &StrategyContext, analysis: &RSIAnalysis) -> Decimal {
        let config = self.config.as_ref().unwrap();
        let sizing = &config.position_sizing;
        let available_balance = context.available_balance;

        let base_size = match sizing.sizing_method {
            PositionSizingMethod::Fixed => {
                sizing.fixed_size.unwrap_or(Decimal::from(100))
            }
            PositionSizingMethod::PortfolioPercentage => {
                available_balance * sizing.portfolio_percentage / Decimal::from(100)
            }
            PositionSizingMethod::RiskBased => {
                // Calculate position size based on risk per trade
                let risk_amount = available_balance * sizing.risk_per_trade / Decimal::from(100);
                let stop_loss_pct = config.risk_management.stop_loss_pct.unwrap_or(Decimal::new(5, 2));
                risk_amount / (stop_loss_pct / Decimal::from(100))
            }
            PositionSizingMethod::RSIStrengthBased => {
                // Adjust size based on signal strength
                let base_size = available_balance * sizing.portfolio_percentage / Decimal::from(100);
                let strength_multiplier = match analysis.strength {
                    RSISignalStrength::VeryStrong => Decimal::new(15, 1), // 1.5x
                    RSISignalStrength::Strong => Decimal::new(12, 1),     // 1.2x
                    RSISignalStrength::Medium => Decimal::ONE,            // 1.0x
                    RSISignalStrength::Weak => Decimal::new(7, 1),       // 0.7x
                };
                base_size * strength_multiplier
            }
        };

        // Apply min/max limits
        base_size
            .max(sizing.min_position_size)
            .min(sizing.max_position_size)
            .min(available_balance) // Can't exceed available balance
    }

    /// Record strategy execution
    fn record_execution(&mut self, context: &StrategyContext, analysis: RSIAnalysis, side: TradeSide, quantity: Decimal) {
        let execution = RSIExecution {
            timestamp: context.current_time,
            price: context.current_price,
            quantity,
            side,
            rsi_value: analysis.rsi_value,
            signal: analysis.signal.clone(),
            strength: analysis.strength,
            position_before: self.state.position,
            position_after: match side {
                TradeSide::Buy => self.state.position + 1,
                TradeSide::Sell => self.state.position - 1,
            },
            realized_pnl: self.state.realized_pnl,
            market_conditions: analysis.market_conditions,
            reason: self.last_signal_reason.clone(),
        };

        // Update state based on execution
        match side {
            TradeSide::Buy => {
                if self.state.position <= 0 {
                    // Opening long or closing short
                    self.state.entry_price = Some(context.current_price);
                }
                self.state.position = 1;
            }
            TradeSide::Sell => {
                if self.state.position >= 0 {
                    // Opening short or closing long
                    self.state.entry_price = Some(context.current_price);
                }
                self.state.position = -1;
            }
        }

        // Update RSI state
        self.state.previous_rsi = self.state.current_rsi;
        self.state.current_rsi = Some(analysis.rsi_value);
        self.state.last_signal = analysis.signal;
        self.state.last_signal_time = Some(context.current_time);

        self.execution_history.push(execution);

        info!("RSI execution recorded: {:?} {} at {} (RSI: {:.2})",
              side, quantity, context.current_price, analysis.rsi_value);
    }
}

#[async_trait]
impl Strategy for RSIStrategy {
    fn metadata(&self) -> StrategyMetadata {
        self.metadata.clone()
    }

    async fn initialize(
        &mut self,
        parameters: &Value,
        _mode: StrategyMode,
        _context: &StrategyContext,
    ) -> Result<(), AppError> {
        let rsi_config: RSIStrategyConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid RSI strategy config: {}", e)))?;

        self.config = Some(rsi_config);
        self.state = RSIStrategyState::default();

        info!("RSI strategy initialized with config: {:?}", self.config);
        Ok(())
    }

    async fn analyze(
        &mut self,
        context: &StrategyContext,
    ) -> Result<Option<StrategySignal>, AppError> {
        let config = self.config.as_ref()
            .ok_or_else(|| AppError::BadRequest("Strategy not initialized".to_string()))?;

        if self.is_paused {
            return Ok(None);
        }

        // Check risk management first
        if let Some(risk_side) = self.check_risk_management(context) {
            self.last_signal_reason = "Risk management trigger".to_string();

            let quantity = self.calculate_position_size(context, &RSIAnalysis {
                signal: RSISignal::None,
                strength: RSISignalStrength::Strong,
                confidence: Decimal::ONE,
                rsi_value: Decimal::ZERO,
                rsi_change: Decimal::ZERO,
                price: context.current_price,
                market_conditions: MarketConditions::default(),
                divergence_info: None,
            });

            let signal = match risk_side {
                TradeSide::Buy => StrategySignal::buy(
                    context.symbol.clone(),
                    QuantityType::Fixed(quantity),
                    self.last_signal_reason.clone(),
                    None,
                ),
                TradeSide::Sell => StrategySignal::sell(
                    context.symbol.clone(),
                    QuantityType::Fixed(quantity),
                    self.last_signal_reason.clone(),
                    None,
                ),
            };

            return Ok(Some(signal));
        }

        // Extract config values to avoid borrowing conflicts
        let enable_long = config.enable_long;
        let enable_short = config.enable_short;
        let rsi_period = config.rsi_period;

        // Check if we have enough data for RSI calculation
        if context.historical_data.len() < rsi_period + 1 {
            return Ok(None);
        }

        // Calculate RSI
        let rsi_value = indicators::rsi(&context.historical_data, rsi_period)
            .ok_or_else(|| AppError::BadRequest("Failed to calculate RSI".to_string()))?;

        // Analyze RSI signal
        let analysis = self.analyze_rsi(context, rsi_value)?;

        if analysis.signal == RSISignal::None {
            return Ok(None);
        }

        // Apply filters
        if !self.passes_filters(&analysis, context) {
            return Ok(None);
        }

        // Determine trade action based on signal and current position
        let (signal_type, side) = match analysis.signal {
            RSISignal::Oversold => {
                if enable_long && self.state.position != 1 {
                    (Some("buy"), Some(TradeSide::Buy))
                } else {
                    (None, None)
                }
            }
            RSISignal::Overbought => {
                if enable_short && self.state.position != -1 {
                    if self.state.position == 1 {
                        // Close long position
                        (Some("sell"), Some(TradeSide::Sell))
                    } else {
                        // Open short position
                        (Some("sell"), Some(TradeSide::Sell))
                    }
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };

        if let (Some(_signal_type), Some(trade_side)) = (signal_type, side) {
            let quantity = self.calculate_position_size(context, &analysis);

            self.last_signal_reason = format!(
                "{:?} signal (RSI: {:.2}, Strength: {:?}, Confidence: {:.2})",
                analysis.signal, analysis.rsi_value, analysis.strength, analysis.confidence
            );

            // Record the execution
            self.record_execution(context, analysis.clone(), trade_side, quantity);

            let signal = match trade_side {
                TradeSide::Buy => StrategySignal::buy(
                    context.symbol.clone(),
                    QuantityType::Fixed(quantity),
                    self.last_signal_reason.clone(),
                    Some(analysis.confidence),
                ),
                TradeSide::Sell => StrategySignal::sell(
                    context.symbol.clone(),
                    QuantityType::Fixed(quantity),
                    self.last_signal_reason.clone(),
                    Some(analysis.confidence),
                ),
            };

            debug!("RSI strategy signal generated: {:?}", signal);
            return Ok(Some(signal));
        }

        Ok(None)
    }

    fn validate_parameters(&self, parameters: &Value) -> Result<(), AppError> {
        let _config: RSIStrategyConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid RSI strategy config: {}", e)))?;
        Ok(())
    }

    fn parameter_schema(&self) -> Value {
        serde_json::json!({
            "rsi_period": { "type": "integer", "default": 14, "min": 2, "max": 200 },
            "overbought_level": { "type": "number", "default": 70, "min": 50, "max": 90 },
            "oversold_level": { "type": "number", "default": 30, "min": 10, "max": 50 },
            "enable_long": { "type": "boolean", "default": true },
            "enable_short": { "type": "boolean", "default": true }
        })
    }

    fn get_state(&self) -> Result<Value, AppError> {
        let mut state_with_metadata = serde_json::to_value(&self.state)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize state: {}", e)))?;

        // Add execution history summary
        if let Some(state_obj) = state_with_metadata.as_object_mut() {
            state_obj.insert("execution_count".to_string(), serde_json::Value::Number(
                serde_json::Number::from(self.execution_history.len())
            ));

            if let Some(last_execution) = self.execution_history.last() {
                state_obj.insert("last_execution_reason".to_string(),
                    serde_json::Value::String(last_execution.reason.clone()));
                state_obj.insert("last_rsi_value".to_string(),
                    serde_json::Value::String(last_execution.rsi_value.to_string()));
            }
        }

        Ok(state_with_metadata)
    }

    fn restore_state(&mut self, state: &Value) -> Result<(), AppError> {
        self.state = serde_json::from_value(state.clone())
            .map_err(|e| AppError::BadRequest(format!("Failed to deserialize state: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl LiveExecutableStrategy for RSIStrategy {
    async fn start_live_execution(&mut self, _context: &StrategyContext) -> Result<(), AppError> {
        self.is_running = true;
        info!("RSI strategy started for live execution");
        Ok(())
    }

    async fn stop_live_execution(&mut self) -> Result<(), AppError> {
        self.is_running = false;
        info!("RSI strategy stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn next_execution_time(&self) -> Option<DateTime<Utc>> {
        // RSI strategy executes on every market data update
        // Return None to indicate continuous execution
        None
    }
}

#[async_trait]
impl ControllableStrategy for RSIStrategy {
    async fn pause(&mut self) -> Result<(), AppError> {
        self.is_paused = true;
        info!("RSI strategy paused");
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), AppError> {
        self.is_paused = false;
        info!("RSI strategy resumed");
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }
}