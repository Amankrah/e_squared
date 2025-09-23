use chrono::{DateTime, Utc, Duration};
use rust_decimal::{Decimal, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::strategies::{Strategy, TradeSignal};
use crate::strategies::indicators;
use crate::exchange_connectors::types::Kline;
use crate::utils::errors::AppError;

/// Dollar Cost Averaging Strategy
///
/// This strategy implements various DCA approaches:
/// - Simple DCA: Buy fixed amount at regular intervals
/// - Dynamic DCA: Adjust buy amount based on market conditions
/// - RSI-based DCA: Increase buying when RSI is low (oversold)
/// - Volatility-based DCA: Adjust buying based on market volatility
pub struct DCAStrategy {
    config: Option<DCAConfig>,
    last_buy_time: Option<DateTime<Utc>>,
    last_signal_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCAConfig {
    /// Base amount to invest each interval (in quote currency)
    pub base_amount: Decimal,

    /// Interval between purchases in hours
    pub interval_hours: i64,

    /// DCA strategy type
    pub strategy_type: DCAType,

    /// RSI settings (for RSI-based DCA)
    pub rsi_period: Option<usize>,
    pub rsi_oversold_threshold: Option<Decimal>,
    pub rsi_multiplier: Option<Decimal>,

    /// Volatility settings (for volatility-based DCA)
    pub volatility_period: Option<usize>,
    pub volatility_threshold: Option<Decimal>,
    pub volatility_multiplier: Option<Decimal>,

    /// Market condition settings
    pub use_market_sentiment: bool,
    pub fear_greed_threshold: Option<i32>,

    /// Stop loss settings
    pub enable_stop_loss: bool,
    pub stop_loss_percentage: Option<Decimal>,

    /// Take profit settings
    pub enable_take_profit: bool,
    pub take_profit_percentage: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DCAType {
    Simple,           // Fixed amount at regular intervals
    RSIBased,         // Adjust amount based on RSI
    VolatilityBased,  // Adjust amount based on volatility
    Dynamic,          // Combine multiple factors
}

impl DCAStrategy {
    pub fn new() -> Self {
        Self {
            config: None,
            last_buy_time: None,
            last_signal_reason: String::new(),
        }
    }

    fn should_buy(&self, data: &[Kline], current_index: usize) -> bool {
        let config = match &self.config {
            Some(c) => c,
            None => return false,
        };

        let current_time = data[current_index].close_time;

        // Check if enough time has passed since last buy
        if let Some(last_buy) = self.last_buy_time {
            let time_diff = current_time - last_buy;
            if time_diff < Duration::hours(config.interval_hours) {
                return false;
            }
        }

        true
    }

    fn calculate_buy_amount(&mut self, data: &[Kline], current_index: usize) -> Decimal {
        let config = match &self.config {
            Some(c) => c,
            None => return Decimal::ZERO,
        };

        let mut amount = config.base_amount;
        let mut reasons = Vec::new();

        match config.strategy_type {
            DCAType::Simple => {
                reasons.push("Simple DCA".to_string());
            }
            DCAType::RSIBased => {
                if let Some(rsi_period) = config.rsi_period {
                    if current_index >= rsi_period {
                        let data_slice = &data[..=current_index];
                        if let Some(rsi) = indicators::rsi(data_slice, rsi_period) {
                            if let Some(oversold_threshold) = config.rsi_oversold_threshold {
                                if rsi < oversold_threshold {
                                    let multiplier = config.rsi_multiplier.unwrap_or(Decimal::from_f32(1.5).unwrap());
                                    amount *= multiplier;
                                    reasons.push(format!("RSI oversold ({:.2}), multiplier: {}", rsi, multiplier));
                                } else {
                                    reasons.push(format!("RSI normal ({:.2})", rsi));
                                }
                            }
                        }
                    }
                }
            }
            DCAType::VolatilityBased => {
                if let Some(volatility_period) = config.volatility_period {
                    if current_index >= volatility_period {
                        let data_slice = &data[current_index.saturating_sub(volatility_period)..=current_index];
                        let volatility = self.calculate_volatility(data_slice);

                        if let Some(vol_threshold) = config.volatility_threshold {
                            if volatility > vol_threshold {
                                let multiplier = config.volatility_multiplier.unwrap_or(Decimal::from_f32(1.3).unwrap());
                                amount *= multiplier;
                                reasons.push(format!("High volatility ({:.2}%), multiplier: {}", volatility, multiplier));
                            } else {
                                reasons.push(format!("Normal volatility ({:.2}%)", volatility));
                            }
                        }
                    }
                }
            }
            DCAType::Dynamic => {
                // Combine RSI and volatility factors
                let mut total_multiplier = Decimal::ONE;

                // RSI factor
                if let Some(rsi_period) = config.rsi_period {
                    if current_index >= rsi_period {
                        let data_slice = &data[..=current_index];
                        if let Some(rsi) = indicators::rsi(data_slice, rsi_period) {
                            let rsi_factor = self.calculate_rsi_factor(rsi, &config);
                            total_multiplier *= rsi_factor;
                            reasons.push(format!("RSI factor: {:.2} (RSI: {:.2})", rsi_factor, rsi));
                        }
                    }
                }

                // Volatility factor
                if let Some(volatility_period) = config.volatility_period {
                    if current_index >= volatility_period {
                        let data_slice = &data[current_index.saturating_sub(volatility_period)..=current_index];
                        let volatility = self.calculate_volatility(data_slice);
                        let vol_factor = self.calculate_volatility_factor(volatility, &config);
                        total_multiplier *= vol_factor;
                        reasons.push(format!("Volatility factor: {:.2} (Vol: {:.2}%)", vol_factor, volatility));
                    }
                }

                amount *= total_multiplier;
                reasons.push(format!("Dynamic DCA, total multiplier: {:.2}", total_multiplier));
            }
        }

        self.last_signal_reason = reasons.join(", ");
        amount
    }

    fn calculate_rsi_factor(&self, rsi: Decimal, config: &DCAConfig) -> Decimal {
        let oversold_threshold = config.rsi_oversold_threshold.unwrap_or(Decimal::from(30));
        let overbought_threshold = Decimal::from(70);

        if rsi < oversold_threshold {
            // Heavily oversold - increase buying
            Decimal::from_f32(2.0).unwrap()
        } else if rsi < Decimal::from(40) {
            // Moderately oversold - increase buying slightly
            Decimal::from_f32(1.5).unwrap()
        } else if rsi > overbought_threshold {
            // Overbought - reduce buying
            Decimal::from_f32(0.5).unwrap()
        } else {
            // Normal range
            Decimal::ONE
        }
    }

    fn calculate_volatility_factor(&self, volatility: Decimal, config: &DCAConfig) -> Decimal {
        let threshold = config.volatility_threshold.unwrap_or(Decimal::from(20));

        if volatility > threshold * Decimal::from_f32(1.5).unwrap() {
            // Very high volatility - increase buying (opportunity)
            Decimal::from_f32(1.8).unwrap()
        } else if volatility > threshold {
            // High volatility - increase buying slightly
            Decimal::from_f32(1.3).unwrap()
        } else if volatility < threshold / Decimal::from(2) {
            // Low volatility - reduce buying
            Decimal::from_f32(0.8).unwrap()
        } else {
            // Normal volatility
            Decimal::ONE
        }
    }

    fn calculate_volatility(&self, data: &[Kline]) -> Decimal {
        if data.len() < 2 {
            return Decimal::ZERO;
        }

        let returns: Vec<Decimal> = data
            .windows(2)
            .map(|window| {
                let prev_price = window[0].close;
                let curr_price = window[1].close;
                ((curr_price - prev_price) / prev_price).abs()
            })
            .collect();

        let mean_return = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

        let variance = returns
            .iter()
            .map(|r| {
                let diff = *r - mean_return;
                diff * diff // Use multiplication instead of powi
            })
            .sum::<Decimal>() / Decimal::from(returns.len());

        // Convert to percentage (simplified square root)
        variance * Decimal::from(100)
    }
}

impl Strategy for DCAStrategy {
    fn initialize(&mut self, parameters: &Value) -> Result<(), AppError> {
        let config: DCAConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid DCA parameters: {}", e)))?;

        // Validate config
        if config.base_amount <= Decimal::ZERO {
            return Err(AppError::BadRequest("Base amount must be positive".to_string()));
        }

        if config.interval_hours <= 0 {
            return Err(AppError::BadRequest("Interval hours must be positive".to_string()));
        }

        self.config = Some(config);
        self.last_buy_time = None;
        self.last_signal_reason = "Strategy initialized".to_string();

        Ok(())
    }

    fn analyze(&mut self, data: &[Kline], current_index: usize) -> Option<TradeSignal> {
        if current_index >= data.len() {
            return None;
        }

        if !self.should_buy(data, current_index) {
            return None;
        }

        let amount = self.calculate_buy_amount(data, current_index);

        if amount > Decimal::ZERO {
            self.last_buy_time = Some(data[current_index].close_time);
            Some(TradeSignal::Buy(amount))
        } else {
            None
        }
    }

    fn get_last_signal_reason(&self) -> String {
        self.last_signal_reason.clone()
    }

    fn name(&self) -> &'static str {
        "DCA (Dollar Cost Averaging)"
    }

    fn description(&self) -> &'static str {
        "Dollar Cost Averaging strategy with multiple variants including RSI-based and volatility-based adjustments"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["base_amount", "interval_hours", "strategy_type"],
            "properties": {
                "base_amount": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Base amount to invest each interval (in quote currency)"
                },
                "interval_hours": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Hours between each purchase"
                },
                "strategy_type": {
                    "type": "string",
                    "enum": ["Simple", "RSIBased", "VolatilityBased", "Dynamic"],
                    "description": "Type of DCA strategy"
                },
                "rsi_period": {
                    "type": "integer",
                    "minimum": 2,
                    "maximum": 100,
                    "description": "Period for RSI calculation (required for RSI-based strategies)"
                },
                "rsi_oversold_threshold": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "RSI threshold for oversold condition"
                },
                "rsi_multiplier": {
                    "type": "number",
                    "minimum": 1,
                    "description": "Multiplier for buy amount when RSI is oversold"
                },
                "volatility_period": {
                    "type": "integer",
                    "minimum": 2,
                    "maximum": 100,
                    "description": "Period for volatility calculation"
                },
                "volatility_threshold": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Volatility threshold percentage"
                },
                "volatility_multiplier": {
                    "type": "number",
                    "minimum": 1,
                    "description": "Multiplier for buy amount during high volatility"
                },
                "use_market_sentiment": {
                    "type": "boolean",
                    "description": "Whether to consider market sentiment"
                },
                "fear_greed_threshold": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "Fear & Greed index threshold"
                },
                "enable_stop_loss": {
                    "type": "boolean",
                    "description": "Enable stop loss functionality"
                },
                "stop_loss_percentage": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "Stop loss percentage"
                },
                "enable_take_profit": {
                    "type": "boolean",
                    "description": "Enable take profit functionality"
                },
                "take_profit_percentage": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Take profit percentage"
                }
            }
        })
    }

    fn validate_parameters(&self, parameters: &Value) -> Result<(), AppError> {
        let config: DCAConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;

        // Additional validation
        match config.strategy_type {
            DCAType::RSIBased | DCAType::Dynamic => {
                if config.rsi_period.is_none() {
                    return Err(AppError::BadRequest("RSI period is required for RSI-based strategies".to_string()));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Default for DCAStrategy {
    fn default() -> Self {
        Self::new()
    }
}