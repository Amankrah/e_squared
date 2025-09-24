use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::strategies::core::{
    Strategy, StrategyMetadata, StrategyMode, StrategyContext, StrategySignal,
    StrategyCategory, RiskLevel, LiveExecutableStrategy, ControllableStrategy, IndicatorValue, QuantityType,
};
use crate::strategies::indicators::{self, IndicatorService, IndicatorContext};
use crate::utils::errors::AppError;

use super::config::SMACrossoverConfig;
use super::types::*;

/// SMA Crossover Strategy Implementation
pub struct SMACrossoverStrategy {
    /// Strategy configuration
    config: Option<SMACrossoverConfig>,
    /// Current execution state
    state: SMACrossoverState,
    /// Execution history
    execution_history: Vec<SMACrossoverExecution>,
    /// Is strategy currently paused
    is_paused: bool,
    /// Is strategy currently running (for live execution)
    is_running: bool,
    /// Last signal reason
    last_signal_reason: String,
    /// Strategy metadata
    metadata: StrategyMetadata,
}

impl SMACrossoverStrategy {
    /// Create a new SMA Crossover strategy instance
    pub fn new() -> Self {
        Self {
            config: None,
            state: SMACrossoverState::default(),
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
            id: "sma_crossover_v2".to_string(),
            name: "SMA Crossover v2".to_string(),
            description: "Advanced Simple Moving Average crossover strategy with multiple confirmation indicators and comprehensive risk management".to_string(),
            version: "2.0.0".to_string(),
            author: "E-Squared Trading Bot".to_string(),
            category: StrategyCategory::TechnicalAnalysis,
            risk_level: RiskLevel::Moderate,
            supported_modes: vec![
                StrategyMode::Backtest,
                StrategyMode::Paper,
                StrategyMode::Live,
            ],
            min_balance: Some(Decimal::from(500)),
            max_positions: Some(3),
            supported_intervals: vec![
                "1m".to_string(), "5m".to_string(), "15m".to_string(),
                "30m".to_string(), "1h".to_string(), "4h".to_string(),
                "1d".to_string()
            ],
            tags: vec![
                "sma".to_string(),
                "crossover".to_string(),
                "trend".to_string(),
                "technical".to_string(),
                "momentum".to_string(),
            ],
        }
    }

    /// Detect crossover between fast and slow SMA
    fn detect_crossover(&mut self, fast_sma: Decimal, slow_sma: Decimal) -> CrossoverSignal {
        let signal = match (self.state.prev_fast_sma, self.state.prev_slow_sma) {
            (Some(prev_fast), Some(prev_slow)) => {
                // Check for bullish crossover (fast crosses above slow)
                if prev_fast <= prev_slow && fast_sma > slow_sma {
                    CrossoverSignal::BullishCrossover
                }
                // Check for bearish crossover (fast crosses below slow)
                else if prev_fast >= prev_slow && fast_sma < slow_sma {
                    CrossoverSignal::BearishCrossover
                } else {
                    CrossoverSignal::None
                }
            }
            _ => CrossoverSignal::None, // Not enough data
        };

        // Update state with current values
        self.state.prev_fast_sma = self.state.last_fast_sma;
        self.state.prev_slow_sma = self.state.last_slow_sma;
        self.state.last_fast_sma = Some(fast_sma);
        self.state.last_slow_sma = Some(slow_sma);

        signal
    }

    /// Analyze market conditions and generate crossover analysis
    fn analyze_crossover(&self, context: &StrategyContext, fast_sma: Decimal, slow_sma: Decimal, signal: CrossoverSignal) -> Result<CrossoverAnalysis, AppError> {
        let config = self.config.as_ref().unwrap();
        let sma_spread = fast_sma - slow_sma;
        let sma_spread_pct = (sma_spread / slow_sma) * Decimal::from(100);

        // Calculate signal strength based on SMA spread
        let strength = if sma_spread_pct.abs() > Decimal::new(2, 2) { // 2%
            SignalStrength::VeryStrong
        } else if sma_spread_pct.abs() > Decimal::new(1, 2) { // 1%
            SignalStrength::Strong
        } else if sma_spread_pct.abs() > Decimal::new(5, 3) { // 0.5%
            SignalStrength::Moderate
        } else {
            SignalStrength::Weak
        };

        // Calculate confidence based on multiple factors
        let mut confidence = Decimal::new(5, 1); // Base confidence 0.5

        // Increase confidence based on signal strength
        confidence += match strength {
            SignalStrength::VeryStrong => Decimal::new(3, 1), // +0.3
            SignalStrength::Strong => Decimal::new(2, 1), // +0.2
            SignalStrength::Moderate => Decimal::new(1, 1), // +0.1
            SignalStrength::Weak => Decimal::ZERO,
        };

        // Capture market conditions
        let market_conditions = self.capture_market_conditions(context);

        // Adjust confidence based on market conditions
        if let Some(rsi) = market_conditions.rsi {
            match signal {
                CrossoverSignal::BullishCrossover => {
                    if rsi < Decimal::from(50) {
                        confidence += Decimal::new(1, 1); // RSI supports bullish signal
                    }
                }
                CrossoverSignal::BearishCrossover => {
                    if rsi > Decimal::from(50) {
                        confidence += Decimal::new(1, 1); // RSI supports bearish signal
                    }
                }
                CrossoverSignal::None => {}
            }
        }

        // Cap confidence at 1.0
        confidence = confidence.min(Decimal::ONE);

        Ok(CrossoverAnalysis {
            signal,
            strength,
            confidence,
            fast_sma,
            slow_sma,
            sma_spread,
            market_conditions,
        })
    }

    /// Check if signal passes all filters
    fn passes_filters(&self, analysis: &CrossoverAnalysis, context: &StrategyContext) -> bool {
        let config = self.config.as_ref().unwrap();
        let filters = &config.filters;

        // Check minimum SMA spread
        if let Some(min_spread_pct) = filters.min_sma_spread_pct {
            let spread_pct = (analysis.sma_spread.abs() / analysis.slow_sma) * Decimal::from(100);
            if spread_pct < min_spread_pct {
                debug!("Signal filtered: SMA spread too small ({:.3}% < {:.3}%)", spread_pct, min_spread_pct);
                return false;
            }
        }

        // Check minimum volume
        if let Some(min_volume) = filters.min_volume {
            if let Some(current_volume) = analysis.market_conditions.volume {
                if current_volume < min_volume {
                    debug!("Signal filtered: Volume too low");
                    return false;
                }
            }
        }

        // Check RSI filters
        match analysis.signal {
            CrossoverSignal::BullishCrossover => {
                if let (Some(rsi), Some(overbought)) = (analysis.market_conditions.rsi, filters.rsi_overbought) {
                    if rsi > overbought {
                        debug!("Bullish signal filtered: RSI overbought ({:.1} > {:.1})", rsi, overbought);
                        return false;
                    }
                }
            }
            CrossoverSignal::BearishCrossover => {
                if let (Some(rsi), Some(oversold)) = (analysis.market_conditions.rsi, filters.rsi_oversold) {
                    if rsi < oversold {
                        debug!("Bearish signal filtered: RSI oversold ({:.1} < {:.1})", rsi, oversold);
                        return false;
                    }
                }
            }
            CrossoverSignal::None => {}
        }

        // Check MACD confirmation
        if filters.macd_confirmation {
            if let Some(macd_histogram) = analysis.market_conditions.macd_histogram {
                match analysis.signal {
                    CrossoverSignal::BullishCrossover => {
                        if macd_histogram <= Decimal::ZERO {
                            debug!("Bullish signal filtered: MACD not confirming");
                            return false;
                        }
                    }
                    CrossoverSignal::BearishCrossover => {
                        if macd_histogram >= Decimal::ZERO {
                            debug!("Bearish signal filtered: MACD not confirming");
                            return false;
                        }
                    }
                    CrossoverSignal::None => {}
                }
            }
        }

        // Check minimum time between signals
        if let Some(last_signal_time) = self.state.last_signal_time {
            let time_diff = context.current_time.signed_duration_since(last_signal_time);
            let min_interval = Duration::minutes(config.risk_settings.min_signal_interval as i64);

            if time_diff < min_interval {
                debug!("Signal filtered: Too soon since last signal ({} < {} minutes)",
                       time_diff.num_minutes(), min_interval.num_minutes());
                return false;
            }
        }

        true
    }

    /// Calculate position size based on configuration and risk management
    fn calculate_position_size(&self, context: &StrategyContext, analysis: &CrossoverAnalysis) -> Decimal {
        let config = self.config.as_ref().unwrap();
        let available_balance = context.available_balance;

        // Base position size from configuration
        let base_size = available_balance * config.position_size_pct / Decimal::from(100);

        // Adjust based on signal strength
        let strength_multiplier = match analysis.strength {
            SignalStrength::VeryStrong => Decimal::new(12, 1), // 1.2x
            SignalStrength::Strong => Decimal::new(11, 1), // 1.1x
            SignalStrength::Moderate => Decimal::ONE, // 1.0x
            SignalStrength::Weak => Decimal::new(8, 1), // 0.8x
        };

        // Adjust based on confidence
        let confidence_multiplier = Decimal::new(5, 1) + (analysis.confidence * Decimal::new(5, 1)); // 0.5 to 1.0

        let adjusted_size = base_size * strength_multiplier * confidence_multiplier;

        // Apply maximum position size limit
        let max_size = available_balance * config.risk_settings.max_position_pct / Decimal::from(100);
        adjusted_size.min(max_size)
    }

    /// Capture current market conditions
    fn capture_market_conditions(&self, context: &StrategyContext) -> MarketConditions {
        let mut conditions = MarketConditions {
            price: context.current_price,
            price_change_24h: context.market_data.price_change_24h,
            volume: context.market_data.volume_24h,
            ..Default::default()
        };

        let config = self.config.as_ref().unwrap();

        // Calculate RSI if enabled
        if config.confirmation_indicators.use_rsi && context.historical_data.len() >= config.confirmation_indicators.rsi_period {
            if let Some(rsi) = indicators::rsi(&context.historical_data, config.confirmation_indicators.rsi_period) {
                conditions.rsi = Some(rsi);
            }
        }

        // Calculate MACD if enabled
        if config.confirmation_indicators.use_macd {
            if let Some(macd_result) = indicators::macd(
                &context.historical_data,
                config.confirmation_indicators.macd_fast,
                config.confirmation_indicators.macd_slow,
                config.confirmation_indicators.macd_signal,
            ) {
                conditions.macd_histogram = Some(macd_result.histogram);
            }
        }

        conditions
    }

    /// Record execution in state and history
    fn record_execution(&mut self, context: &StrategyContext, analysis: CrossoverAnalysis, side: TradeSide, quantity: Decimal) {
        let price = context.current_price;

        // Update state
        match side {
            TradeSide::Buy => {
                self.state.position = 1; // Long position
                self.state.entry_price = Some(price);
            }
            TradeSide::Sell => {
                // Close position and calculate PnL
                if let Some(entry_price) = self.state.entry_price {
                    let pnl = if self.state.position == 1 {
                        (price - entry_price) * quantity // Long position profit
                    } else {
                        (entry_price - price) * quantity // Short position profit
                    };

                    self.state.total_pnl += pnl;
                    if pnl > Decimal::ZERO {
                        self.state.winning_trades += 1;
                    }
                }

                self.state.position = 0; // No position
                self.state.entry_price = None;
            }
        }

        self.state.trade_count += 1;
        self.state.last_signal = Some(analysis.signal.clone());
        self.state.last_signal_time = Some(context.current_time);

        // Create execution record
        let execution = SMACrossoverExecution {
            timestamp: context.current_time,
            signal: analysis.signal,
            price,
            quantity,
            side,
            fast_sma: analysis.fast_sma,
            slow_sma: analysis.slow_sma,
            market_conditions: analysis.market_conditions,
            reason: self.last_signal_reason.clone(),
        };

        self.execution_history.push(execution);

        // Keep only last 1000 executions to prevent memory bloat
        if self.execution_history.len() > 1000 {
            self.execution_history.remove(0);
        }

        info!("SMA Crossover execution recorded: {:?} {} at {} (PnL: {})",
              side, quantity, price, self.state.total_pnl);
    }

    /// Check risk management conditions
    fn check_risk_management(&self, context: &StrategyContext) -> Option<TradeSide> {
        if self.state.position == 0 || self.state.entry_price.is_none() {
            return None;
        }

        let config = self.config.as_ref().unwrap();
        let entry_price = self.state.entry_price.unwrap();
        let current_price = context.current_price;

        if self.state.position == 1 { // Long position
            // Check stop loss
            let stop_loss_price = entry_price * (Decimal::ONE - config.risk_settings.stop_loss_pct / Decimal::from(100));
            if current_price <= stop_loss_price {
                return Some(TradeSide::Sell);
            }

            // Check take profit
            let take_profit_price = entry_price * (Decimal::ONE + config.risk_settings.take_profit_pct / Decimal::from(100));
            if current_price >= take_profit_price {
                return Some(TradeSide::Sell);
            }
        } else if self.state.position == -1 { // Short position
            // Check stop loss
            let stop_loss_price = entry_price * (Decimal::ONE + config.risk_settings.stop_loss_pct / Decimal::from(100));
            if current_price >= stop_loss_price {
                return Some(TradeSide::Buy);
            }

            // Check take profit
            let take_profit_price = entry_price * (Decimal::ONE - config.risk_settings.take_profit_pct / Decimal::from(100));
            if current_price <= take_profit_price {
                return Some(TradeSide::Buy);
            }
        }

        None
    }
}

#[async_trait]
impl Strategy for SMACrossoverStrategy {
    fn metadata(&self) -> StrategyMetadata {
        self.metadata.clone()
    }

    async fn initialize(
        &mut self,
        parameters: &Value,
        _mode: StrategyMode,
        _context: &StrategyContext,
    ) -> Result<(), AppError> {
        let config: SMACrossoverConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid SMA Crossover parameters: {}", e)))?;

        // Validate configuration
        config.validate()
            .map_err(|e| AppError::BadRequest(e))?;

        self.config = Some(config);
        self.state = SMACrossoverState::default();
        self.execution_history.clear();
        self.is_paused = false;
        self.last_signal_reason = "Strategy initialized".to_string();

        info!("SMA Crossover strategy initialized successfully");
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

            let signal = match risk_side {
                TradeSide::Buy => StrategySignal::buy(
                    context.symbol.clone(),
                    QuantityType::Fixed(self.calculate_position_size(context, &CrossoverAnalysis {
                        signal: CrossoverSignal::None,
                        strength: SignalStrength::Strong,
                        confidence: Decimal::ONE,
                        fast_sma: Decimal::ZERO,
                        slow_sma: Decimal::ZERO,
                        sma_spread: Decimal::ZERO,
                        market_conditions: MarketConditions::default(),
                    })),
                    self.last_signal_reason.clone(),
                    None,
                ),
                TradeSide::Sell => StrategySignal::sell(
                    context.symbol.clone(),
                    QuantityType::Fixed(self.calculate_position_size(context, &CrossoverAnalysis {
                        signal: CrossoverSignal::None,
                        strength: SignalStrength::Strong,
                        confidence: Decimal::ONE,
                        fast_sma: Decimal::ZERO,
                        slow_sma: Decimal::ZERO,
                        sma_spread: Decimal::ZERO,
                        market_conditions: MarketConditions::default(),
                    })),
                    self.last_signal_reason.clone(),
                    None,
                ),
            };

            return Ok(Some(signal));
        }

        // Extract config values to avoid borrowing conflicts
        let enable_long = config.enable_long;
        let enable_short = config.enable_short;
        let fast_period = config.fast_period;
        let slow_period = config.slow_period;

        // Check if we have enough data for SMA calculation
        if context.historical_data.len() < slow_period {
            return Ok(None);
        }

        // Calculate SMAs
        let fast_sma = indicators::sma(&context.historical_data, fast_period)
            .ok_or_else(|| AppError::BadRequest("Failed to calculate fast SMA".to_string()))?;

        let slow_sma = indicators::sma(&context.historical_data, slow_period)
            .ok_or_else(|| AppError::BadRequest("Failed to calculate slow SMA".to_string()))?;

        // Detect crossover
        let crossover_signal = self.detect_crossover(fast_sma, slow_sma);

        if crossover_signal == CrossoverSignal::None {
            return Ok(None);
        }

        // Analyze the crossover
        let analysis = self.analyze_crossover(context, fast_sma, slow_sma, crossover_signal)?;

        // Apply filters
        if !self.passes_filters(&analysis, context) {
            return Ok(None);
        }

        // Determine trade action
        let (signal_type, side) = match analysis.signal {
            CrossoverSignal::BullishCrossover => {
                if enable_long && self.state.position != 1 {
                    (Some("buy"), Some(TradeSide::Buy))
                } else {
                    (None, None)
                }
            }
            CrossoverSignal::BearishCrossover => {
                if enable_short && self.state.position != -1 {
                    if self.state.position == 1 {
                        // Close long position
                        (Some("sell"), Some(TradeSide::Sell))
                    } else {
                        // Open short position (if enabled)
                        (Some("sell"), Some(TradeSide::Sell))
                    }
                } else {
                    (None, None)
                }
            }
            CrossoverSignal::None => (None, None),
        };

        if let (Some(_signal_type), Some(trade_side)) = (signal_type, side) {
            let quantity = self.calculate_position_size(context, &analysis);

            self.last_signal_reason = format!(
                "{:?} crossover (Fast SMA: {:.4}, Slow SMA: {:.4}, Confidence: {:.2})",
                analysis.signal, analysis.fast_sma, analysis.slow_sma, analysis.confidence
            );

            // Record the execution
            self.record_execution(context, analysis.clone(), trade_side.clone(), quantity);

            let signal = match trade_side {
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

            // Add indicators to signal metadata
            let indicators = vec![
                IndicatorValue {
                    name: "Fast SMA".to_string(),
                    value: analysis.fast_sma,
                    signal: "trend".to_string(),
                },
                IndicatorValue {
                    name: "Slow SMA".to_string(),
                    value: analysis.slow_sma,
                    signal: "trend".to_string(),
                },
            ];

            let enhanced_signal = signal
                .with_indicators(indicators)
                .with_confidence(analysis.confidence);

            Ok(Some(enhanced_signal))
        } else {
            Ok(None)
        }
    }

    fn validate_parameters(&self, parameters: &Value) -> Result<(), AppError> {
        let config: SMACrossoverConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;

        config.validate()
            .map_err(|e| AppError::BadRequest(e))?;

        Ok(())
    }

    fn parameter_schema(&self) -> Value {
        SMACrossoverConfig::json_schema()
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
                state_obj.insert("last_signal_type".to_string(),
                    serde_json::Value::String(format!("{:?}", last_execution.signal)));
            }

            // Calculate performance metrics
            if self.state.trade_count > 0 {
                let win_rate = Decimal::from(self.state.winning_trades) / Decimal::from(self.state.trade_count);
                state_obj.insert("win_rate".to_string(),
                    serde_json::Value::String(win_rate.to_string()));
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
impl LiveExecutableStrategy for SMACrossoverStrategy {
    async fn start_live_execution(&mut self, _context: &StrategyContext) -> Result<(), AppError> {
        self.is_running = true;
        info!("SMA Crossover strategy started for live execution");
        Ok(())
    }

    async fn stop_live_execution(&mut self) -> Result<(), AppError> {
        self.is_running = false;
        info!("SMA Crossover strategy stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn next_execution_time(&self) -> Option<DateTime<Utc>> {
        // SMA crossover is event-driven, no scheduled executions
        None
    }
}

#[async_trait]
impl ControllableStrategy for SMACrossoverStrategy {
    async fn pause(&mut self) -> Result<(), AppError> {
        self.is_paused = true;
        info!("SMA Crossover strategy paused");
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), AppError> {
        self.is_paused = false;
        info!("SMA Crossover strategy resumed");
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }
}

impl Default for SMACrossoverStrategy {
    fn default() -> Self {
        Self::new()
    }
}