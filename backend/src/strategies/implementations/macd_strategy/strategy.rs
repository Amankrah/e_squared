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

use super::config::{MACDStrategyConfig, PositionSizingMethod};
use super::types::*;

/// MACD Strategy Implementation
pub struct MACDStrategy {
    /// Strategy configuration
    config: Option<MACDStrategyConfig>,
    /// Current execution state
    state: MACDStrategyState,
    /// Execution history
    execution_history: Vec<MACDExecution>,
    /// Is strategy currently paused
    is_paused: bool,
    /// Is strategy currently running (for live execution)
    is_running: bool,
    /// Last signal reason
    last_signal_reason: String,
    /// Strategy metadata
    metadata: StrategyMetadata,
}

impl MACDStrategy {
    /// Create a new MACD strategy instance
    pub fn new() -> Self {
        Self {
            config: None,
            state: MACDStrategyState::default(),
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
            id: "macd_v1".to_string(),
            name: "MACD Strategy".to_string(),
            description: "Moving Average Convergence Divergence strategy with signal line crossovers, zero line crosses, and histogram analysis".to_string(),
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
                "macd".to_string(),
                "momentum".to_string(),
                "technical".to_string(),
                "crossover".to_string(),
            ],
        }
    }

    /// Detect MACD crossovers
    fn detect_crossover(&mut self, macd_line: Decimal, signal_line: Decimal) -> MACDSignal {
        let histogram = macd_line - signal_line;

        // Check for signal line crossovers
        if let (Some(prev_macd), Some(prev_signal)) = (self.state.previous_macd_line, self.state.previous_signal_line) {
            let prev_histogram = prev_macd - prev_signal;

            // Signal line crossover
            if prev_histogram <= Decimal::ZERO && histogram > Decimal::ZERO {
                return MACDSignal::BullishCrossover;
            } else if prev_histogram >= Decimal::ZERO && histogram < Decimal::ZERO {
                return MACDSignal::BearishCrossover;
            }

            // Zero line crossover
            if prev_macd <= Decimal::ZERO && macd_line > Decimal::ZERO {
                return MACDSignal::ZeroCrossBullish;
            } else if prev_macd >= Decimal::ZERO && macd_line < Decimal::ZERO {
                return MACDSignal::ZeroCrossBearish;
            }
        }

        MACDSignal::None
    }

    /// Analyze MACD signal and market conditions
    fn analyze_macd(&self, context: &StrategyContext, macd_line: Decimal, signal_line: Decimal, signal: MACDSignal) -> Result<MACDAnalysis, AppError> {
        let histogram = macd_line - signal_line;
        let price = context.current_price;

        // Calculate histogram change
        let histogram_change = if let Some(prev_hist) = self.state.previous_histogram {
            histogram - prev_hist
        } else {
            Decimal::ZERO
        };

        // Determine signal strength based on MACD values and momentum
        let strength = self.calculate_signal_strength(&signal, macd_line, signal_line, histogram, histogram_change);

        // Calculate confidence based on signal strength and market conditions
        let confidence = self.calculate_signal_confidence(&strength, &signal, histogram.abs(), histogram_change);

        // Analyze trend state
        let trend_analysis = self.analyze_trend(macd_line, signal_line, histogram, histogram_change);

        // Create crossover info if applicable
        let crossover_info = match signal {
            MACDSignal::BullishCrossover | MACDSignal::BearishCrossover |
            MACDSignal::ZeroCrossBullish | MACDSignal::ZeroCrossBearish => {
                Some(self.create_crossover_info(&signal, macd_line, signal_line, histogram_change))
            }
            _ => None,
        };

        Ok(MACDAnalysis {
            signal,
            strength,
            confidence,
            macd_line,
            signal_line,
            histogram,
            histogram_change,
            price,
            market_conditions: self.capture_market_conditions(context),
            crossover_info,
            trend_analysis,
        })
    }

    /// Calculate signal strength based on MACD characteristics
    fn calculate_signal_strength(&self, signal: &MACDSignal, macd_line: Decimal, signal_line: Decimal, histogram: Decimal, histogram_change: Decimal) -> MACDSignalStrength {
        let config = self.config.as_ref().unwrap();
        let strength_config = &config.signal_config.signal_strength;

        let histogram_abs = histogram.abs();
        let crossover_distance = (macd_line - signal_line).abs();

        // Base strength on histogram magnitude and crossover characteristics
        if histogram_abs >= strength_config.min_strong_histogram * Decimal::from(5) &&
           crossover_distance >= strength_config.min_crossover_distance * Decimal::from(3) {
            // Check for acceleration requirement
            if strength_config.require_histogram_acceleration {
                match signal {
                    MACDSignal::BullishCrossover | MACDSignal::ZeroCrossBullish => {
                        if histogram_change > Decimal::ZERO {
                            MACDSignalStrength::VeryStrong
                        } else {
                            MACDSignalStrength::Strong
                        }
                    }
                    MACDSignal::BearishCrossover | MACDSignal::ZeroCrossBearish => {
                        if histogram_change < Decimal::ZERO {
                            MACDSignalStrength::VeryStrong
                        } else {
                            MACDSignalStrength::Strong
                        }
                    }
                    _ => MACDSignalStrength::Medium,
                }
            } else {
                MACDSignalStrength::VeryStrong
            }
        } else if histogram_abs >= strength_config.min_strong_histogram &&
                  crossover_distance >= strength_config.min_crossover_distance {
            MACDSignalStrength::Strong
        } else if crossover_distance >= strength_config.min_crossover_distance / Decimal::from(2) {
            MACDSignalStrength::Medium
        } else {
            MACDSignalStrength::Weak
        }
    }

    /// Calculate signal confidence
    fn calculate_signal_confidence(&self, strength: &MACDSignalStrength, signal: &MACDSignal, histogram_abs: Decimal, histogram_change: Decimal) -> Decimal {
        let mut confidence = match strength {
            MACDSignalStrength::VeryStrong => Decimal::new(9, 1),  // 0.9
            MACDSignalStrength::Strong => Decimal::new(75, 2),     // 0.75
            MACDSignalStrength::Medium => Decimal::new(6, 1),      // 0.6
            MACDSignalStrength::Weak => Decimal::new(4, 1),        // 0.4
        };

        // Adjust based on signal type
        match signal {
            MACDSignal::BullishCrossover | MACDSignal::BearishCrossover => {
                // Signal line crossovers are generally reliable
                confidence += Decimal::new(5, 2); // +0.05
            }
            MACDSignal::ZeroCrossBullish | MACDSignal::ZeroCrossBearish => {
                // Zero line crossovers indicate trend changes
                confidence += Decimal::new(1, 1); // +0.1
            }
            _ => {}
        }

        // Adjust based on momentum (histogram change)
        if histogram_change.abs() > Decimal::new(1, 4) { // 0.0001
            confidence += Decimal::new(5, 2); // +0.05
        }

        // Cap confidence at 1.0
        confidence.min(Decimal::ONE)
    }

    /// Analyze trend state based on MACD components
    fn analyze_trend(&self, macd_line: Decimal, signal_line: Decimal, histogram: Decimal, histogram_change: Decimal) -> TrendAnalysis {
        let trend_state = if macd_line > Decimal::ZERO && histogram > Decimal::ZERO && histogram_change > Decimal::ZERO {
            TrendState::StrongBullish
        } else if histogram > Decimal::ZERO && histogram_change >= Decimal::ZERO {
            TrendState::WeakBullish
        } else if macd_line < Decimal::ZERO && histogram < Decimal::ZERO && histogram_change < Decimal::ZERO {
            TrendState::StrongBearish
        } else if histogram < Decimal::ZERO && histogram_change <= Decimal::ZERO {
            TrendState::WeakBearish
        } else {
            TrendState::Neutral
        };

        let trend_strength = match trend_state {
            TrendState::StrongBullish | TrendState::StrongBearish => Decimal::new(8, 1), // 0.8
            TrendState::WeakBullish | TrendState::WeakBearish => Decimal::new(5, 1),     // 0.5
            TrendState::Neutral => Decimal::new(2, 1),                                  // 0.2
        };

        let momentum_direction = if histogram_change > Decimal::new(1, 4) {
            MomentumDirection::Accelerating
        } else if histogram_change < -Decimal::new(1, 4) {
            MomentumDirection::Decelerating
        } else {
            MomentumDirection::Stable
        };

        let momentum_acceleration = histogram_change.abs() * Decimal::from(10000); // Scale for visibility

        TrendAnalysis {
            trend_state,
            trend_strength,
            momentum_direction,
            momentum_acceleration,
            trend_duration: 1, // TODO: Track actual trend duration
        }
    }

    /// Create crossover information
    fn create_crossover_info(&self, signal: &MACDSignal, macd_line: Decimal, signal_line: Decimal, histogram_change: Decimal) -> CrossoverInfo {
        let crossover_type = match signal {
            MACDSignal::BullishCrossover => CrossoverType::BullishSignalCross,
            MACDSignal::BearishCrossover => CrossoverType::BearishSignalCross,
            MACDSignal::ZeroCrossBullish => CrossoverType::BullishZeroCross,
            MACDSignal::ZeroCrossBearish => CrossoverType::BearishZeroCross,
            _ => CrossoverType::BullishSignalCross, // Default fallback
        };

        CrossoverInfo {
            crossover_type,
            crossover_speed: histogram_change.abs() * Decimal::from(1000), // Scale for visibility
            crossover_distance: (macd_line - signal_line).abs(),
            histogram_momentum: histogram_change,
        }
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
            long_term_trend: None, // TODO: Determine trend using SMA
        }
    }

    /// Apply signal filters
    fn passes_filters(&self, analysis: &MACDAnalysis, context: &StrategyContext) -> bool {
        let config = self.config.as_ref().unwrap();
        let filters = &config.signal_filters;

        // Volume filter
        if let Some(min_volume) = filters.min_volume {
            if let Some(volume) = context.market_data.volume_24h {
                if volume < min_volume {
                    debug!("MACD signal filtered out due to low volume: {} < {}", volume, min_volume);
                    return false;
                }
            }
        }

        // Spread filter
        if let Some(max_spread) = filters.max_spread_pct {
            if let Some(spread) = context.market_data.spread {
                let spread_pct = (spread / context.current_price) * Decimal::from(100);
                if spread_pct > max_spread {
                    debug!("MACD signal filtered out due to high spread: {}% > {}%", spread_pct, max_spread);
                    return false;
                }
            }
        }

        // Histogram change filter
        if let Some(min_change) = filters.min_histogram_change {
            if analysis.histogram_change.abs() < min_change {
                debug!("MACD signal filtered out due to insufficient histogram change: {} < {}",
                       analysis.histogram_change.abs(), min_change);
                return false;
            }
        }

        // Crossover distance filter
        if let Some(min_distance) = filters.min_crossover_distance {
            if let Some(crossover_info) = &analysis.crossover_info {
                if crossover_info.crossover_distance < min_distance {
                    debug!("MACD signal filtered out due to insufficient crossover distance: {} < {}",
                           crossover_info.crossover_distance, min_distance);
                    return false;
                }
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

                // Check MACD reversal stop
                if risk_mgmt.macd_reversal_stop {
                    if let (Some(current_hist), Some(prev_hist)) = (self.state.current_histogram, self.state.previous_histogram) {
                        // Exit long if histogram starts declining significantly
                        if self.state.position > 0 && current_hist < prev_hist {
                            if let Some(threshold) = risk_mgmt.histogram_stop_threshold {
                                if (prev_hist - current_hist) > threshold {
                                    return Some(TradeSide::Sell);
                                }
                            }
                        }
                        // Exit short if histogram starts inclining significantly
                        if self.state.position < 0 && current_hist > prev_hist {
                            if let Some(threshold) = risk_mgmt.histogram_stop_threshold {
                                if (current_hist - prev_hist) > threshold {
                                    return Some(TradeSide::Buy);
                                }
                            }
                        }
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
    fn calculate_position_size(&self, context: &StrategyContext, analysis: &MACDAnalysis) -> Decimal {
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
                let stop_loss_pct = config.risk_management.stop_loss_pct.unwrap_or(Decimal::new(4, 2));
                risk_amount / (stop_loss_pct / Decimal::from(100))
            }
            PositionSizingMethod::MACDMomentumBased => {
                // Adjust size based on MACD momentum
                let base_size = available_balance * sizing.portfolio_percentage / Decimal::from(100);
                let momentum_multiplier = match analysis.trend_analysis.momentum_direction {
                    MomentumDirection::Accelerating => Decimal::new(13, 1), // 1.3x
                    MomentumDirection::Stable => Decimal::ONE,              // 1.0x
                    MomentumDirection::Decelerating => Decimal::new(8, 1),  // 0.8x
                };
                base_size * momentum_multiplier
            }
        };

        // Scale by MACD strength if enabled
        let scaled_size = if sizing.scale_by_macd_strength {
            let strength_multiplier = match analysis.strength {
                MACDSignalStrength::VeryStrong => Decimal::new(14, 1), // 1.4x
                MACDSignalStrength::Strong => Decimal::new(12, 1),     // 1.2x
                MACDSignalStrength::Medium => Decimal::ONE,            // 1.0x
                MACDSignalStrength::Weak => Decimal::new(7, 1),       // 0.7x
            };
            base_size * strength_multiplier
        } else {
            base_size
        };

        // Apply min/max limits
        scaled_size
            .max(sizing.min_position_size)
            .min(sizing.max_position_size)
            .min(available_balance) // Can't exceed available balance
    }

    /// Record strategy execution
    fn record_execution(&mut self, context: &StrategyContext, analysis: MACDAnalysis, side: TradeSide, quantity: Decimal) {
        let execution = MACDExecution {
            timestamp: context.current_time,
            price: context.current_price,
            quantity,
            side,
            macd_line: analysis.macd_line,
            signal_line: analysis.signal_line,
            histogram: analysis.histogram,
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

        // Update MACD state
        self.state.previous_macd_line = self.state.current_macd_line;
        self.state.previous_signal_line = self.state.current_signal_line;
        self.state.previous_histogram = self.state.current_histogram;
        self.state.current_macd_line = Some(analysis.macd_line);
        self.state.current_signal_line = Some(analysis.signal_line);
        self.state.current_histogram = Some(analysis.histogram);
        self.state.last_signal = analysis.signal;
        self.state.last_signal_time = Some(context.current_time);
        self.state.trend_state = analysis.trend_analysis.trend_state;

        self.execution_history.push(execution);

        info!("MACD execution recorded: {:?} {} at {} (MACD: {:.6}, Signal: {:.6}, Histogram: {:.6})",
              side, quantity, context.current_price, analysis.macd_line, analysis.signal_line, analysis.histogram);
    }
}

#[async_trait]
impl Strategy for MACDStrategy {
    fn metadata(&self) -> StrategyMetadata {
        self.metadata.clone()
    }

    async fn initialize(
        &mut self,
        parameters: &Value,
        _mode: StrategyMode,
        _context: &StrategyContext,
    ) -> Result<(), AppError> {
        let macd_config: MACDStrategyConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid MACD strategy config: {}", e)))?;

        self.config = Some(macd_config);
        self.state = MACDStrategyState::default();

        info!("MACD strategy initialized with config: {:?}", self.config);
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

            let quantity = self.calculate_position_size(context, &MACDAnalysis {
                signal: MACDSignal::None,
                strength: MACDSignalStrength::Strong,
                confidence: Decimal::ONE,
                macd_line: Decimal::ZERO,
                signal_line: Decimal::ZERO,
                histogram: Decimal::ZERO,
                histogram_change: Decimal::ZERO,
                price: context.current_price,
                market_conditions: MarketConditions::default(),
                crossover_info: None,
                trend_analysis: TrendAnalysis {
                    trend_state: TrendState::Neutral,
                    trend_strength: Decimal::ZERO,
                    momentum_direction: MomentumDirection::Stable,
                    momentum_acceleration: Decimal::ZERO,
                    trend_duration: 0,
                },
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
        let fast_period = config.fast_period;
        let slow_period = config.slow_period;
        let signal_period = config.signal_period;
        let signal_crossover_enabled = config.signal_config.enable_signal_crossover;
        let zero_crossover_enabled = config.signal_config.enable_zero_crossover;

        // Check if we have enough data for MACD calculation
        let min_data_points = slow_period + signal_period;
        if context.historical_data.len() < min_data_points {
            return Ok(None);
        }

        // Calculate MACD
        let macd = indicators::macd(&context.historical_data, fast_period, slow_period, signal_period)
            .ok_or_else(|| AppError::BadRequest("Failed to calculate MACD".to_string()))?;

        let macd_line = macd.macd_line;
        let signal_line = macd.signal_line;

        // Detect crossover signals
        let signal = self.detect_crossover(macd_line, signal_line);

        // Only proceed if we have a signal and it's enabled
        let proceed = match signal {
            MACDSignal::BullishCrossover | MACDSignal::ZeroCrossBullish => signal_crossover_enabled || zero_crossover_enabled,
            MACDSignal::BearishCrossover | MACDSignal::ZeroCrossBearish => signal_crossover_enabled || zero_crossover_enabled,
            _ => false,
        };

        if !proceed || signal == MACDSignal::None {
            // Update state for next iteration
            self.state.previous_macd_line = self.state.current_macd_line;
            self.state.previous_signal_line = self.state.current_signal_line;
            self.state.previous_histogram = self.state.current_histogram;
            self.state.current_macd_line = Some(macd_line);
            self.state.current_signal_line = Some(signal_line);
            self.state.current_histogram = Some(macd_line - signal_line);
            return Ok(None);
        }

        // Analyze MACD signal
        let analysis = self.analyze_macd(context, macd_line, signal_line, signal)?;

        // Apply filters
        if !self.passes_filters(&analysis, context) {
            return Ok(None);
        }

        // Determine trade action based on signal and current position
        let (signal_type, side) = match analysis.signal {
            MACDSignal::BullishCrossover | MACDSignal::ZeroCrossBullish => {
                if enable_long && self.state.position != 1 {
                    (Some("buy"), Some(TradeSide::Buy))
                } else {
                    (None, None)
                }
            }
            MACDSignal::BearishCrossover | MACDSignal::ZeroCrossBearish => {
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
                "{:?} signal (MACD: {:.6}, Signal: {:.6}, Histogram: {:.6}, Strength: {:?})",
                analysis.signal, analysis.macd_line, analysis.signal_line,
                analysis.histogram, analysis.strength
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

            debug!("MACD strategy signal generated: {:?}", signal);
            return Ok(Some(signal));
        }

        Ok(None)
    }

    fn validate_parameters(&self, parameters: &Value) -> Result<(), AppError> {
        let _config: MACDStrategyConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid MACD strategy config: {}", e)))?;
        Ok(())
    }

    fn parameter_schema(&self) -> Value {
        serde_json::json!({
            "fast_period": { "type": "integer", "default": 12, "min": 2, "max": 200 },
            "slow_period": { "type": "integer", "default": 26, "min": 2, "max": 200 },
            "signal_period": { "type": "integer", "default": 9, "min": 2, "max": 50 },
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
                state_obj.insert("last_macd_value".to_string(),
                    serde_json::Value::String(last_execution.macd_line.to_string()));
                state_obj.insert("last_signal_value".to_string(),
                    serde_json::Value::String(last_execution.signal_line.to_string()));
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
impl LiveExecutableStrategy for MACDStrategy {
    async fn start_live_execution(&mut self, _context: &StrategyContext) -> Result<(), AppError> {
        self.is_running = true;
        info!("MACD strategy started for live execution");
        Ok(())
    }

    async fn stop_live_execution(&mut self) -> Result<(), AppError> {
        self.is_running = false;
        info!("MACD strategy stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn next_execution_time(&self) -> Option<DateTime<Utc>> {
        // MACD strategy executes on every market data update
        // Return None to indicate continuous execution
        None
    }
}

#[async_trait]
impl ControllableStrategy for MACDStrategy {
    async fn pause(&mut self) -> Result<(), AppError> {
        self.is_paused = true;
        info!("MACD strategy paused");
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), AppError> {
        self.is_paused = false;
        info!("MACD strategy resumed");
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }
}

impl Default for MACDStrategy {
    fn default() -> Self {
        Self::new()
    }
}