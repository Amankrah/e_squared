use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use rust_decimal::{Decimal, prelude::FromPrimitive};

use super::types::*;

/// Complete DCA strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCAConfig {
    /// Base amount to invest each interval (in quote currency)
    pub base_amount: Decimal,

    /// DCA execution frequency
    pub frequency: DCAFrequency,

    /// Type of DCA strategy
    pub strategy_type: DCAType,

    /// RSI configuration (for RSI-based and Dynamic strategies)
    pub rsi_config: Option<RSIConfig>,

    /// Volatility configuration (for Volatility-based and Dynamic strategies)
    pub volatility_config: Option<VolatilityConfig>,

    /// Sentiment configuration (for Sentiment-based and Dynamic strategies)
    pub sentiment_config: Option<SentimentConfig>,

    /// Dynamic factors configuration (for Dynamic strategy)
    pub dynamic_factors: Option<DynamicFactors>,

    /// Dip buying levels (for DipBuying strategy)
    pub dip_levels: Option<Vec<DipBuyingLevel>>,

    /// Reference price for dip calculations
    pub reference_price: Option<Decimal>,

    /// Reference period for calculating reference price (in days)
    pub reference_period_days: Option<u32>,

    /// Maximum amount per single purchase
    pub max_single_amount: Option<Decimal>,

    /// Minimum amount per single purchase
    pub min_single_amount: Option<Decimal>,

    /// Maximum total position size (stop DCA when reached)
    pub max_position_size: Option<Decimal>,

    /// Enable/disable DCA during high volatility
    pub pause_on_high_volatility: bool,

    /// Volatility threshold to pause DCA
    pub volatility_pause_threshold: Option<Decimal>,

    /// Enable/disable DCA during bear market
    pub pause_on_bear_market: bool,

    /// Price drop threshold to detect bear market (from recent high)
    pub bear_market_threshold: Option<Decimal>,

    /// Additional filters and conditions
    pub filters: DCAFilters,
}

/// Additional filters for DCA execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DCAFilters {
    /// Only execute during specific hours (UTC)
    pub allowed_hours: Option<Vec<u8>>,

    /// Only execute on specific days of week (0=Sunday, 6=Saturday)
    pub allowed_weekdays: Option<Vec<u8>>,

    /// Minimum time between executions (in minutes)
    pub min_interval_minutes: Option<u32>,

    /// Maximum executions per day
    pub max_executions_per_day: Option<u32>,

    /// Require minimum volume before execution
    pub min_volume_threshold: Option<Decimal>,

    /// Skip execution if spread is too high
    pub max_spread_percentage: Option<Decimal>,

    /// Skip execution if price moved too much since last check
    pub max_price_deviation_percentage: Option<Decimal>,
}

impl DCAConfig {
    /// Create a simple DCA configuration
    pub fn simple(base_amount: Decimal, frequency: DCAFrequency) -> Self {
        Self {
            base_amount,
            frequency,
            strategy_type: DCAType::Simple,
            rsi_config: None,
            volatility_config: None,
            sentiment_config: None,
            dynamic_factors: None,
            dip_levels: None,
            reference_price: None,
            reference_period_days: None,
            max_single_amount: None,
            min_single_amount: None,
            max_position_size: None,
            pause_on_high_volatility: false,
            volatility_pause_threshold: None,
            pause_on_bear_market: false,
            bear_market_threshold: None,
            filters: DCAFilters::default(),
        }
    }

    /// Create RSI-based DCA configuration
    pub fn rsi_based(
        base_amount: Decimal,
        frequency: DCAFrequency,
        rsi_config: RSIConfig,
    ) -> Self {
        Self {
            base_amount,
            frequency,
            strategy_type: DCAType::RSIBased,
            rsi_config: Some(rsi_config),
            volatility_config: None,
            sentiment_config: None,
            dynamic_factors: None,
            dip_levels: None,
            reference_price: None,
            reference_period_days: None,
            max_single_amount: None,
            min_single_amount: None,
            max_position_size: None,
            pause_on_high_volatility: false,
            volatility_pause_threshold: None,
            pause_on_bear_market: false,
            bear_market_threshold: None,
            filters: DCAFilters::default(),
        }
    }

    /// Create volatility-based DCA configuration
    pub fn volatility_based(
        base_amount: Decimal,
        frequency: DCAFrequency,
        volatility_config: VolatilityConfig,
    ) -> Self {
        Self {
            base_amount,
            frequency,
            strategy_type: DCAType::VolatilityBased,
            rsi_config: None,
            volatility_config: Some(volatility_config),
            sentiment_config: None,
            dynamic_factors: None,
            dip_levels: None,
            reference_price: None,
            reference_period_days: None,
            max_single_amount: None,
            min_single_amount: None,
            max_position_size: None,
            pause_on_high_volatility: false,
            volatility_pause_threshold: None,
            pause_on_bear_market: false,
            bear_market_threshold: None,
            filters: DCAFilters::default(),
        }
    }

    /// Create dip-buying DCA configuration
    pub fn dip_buying(
        base_amount: Decimal,
        frequency: DCAFrequency,
        dip_levels: Vec<DipBuyingLevel>,
        reference_price: Option<Decimal>,
    ) -> Self {
        Self {
            base_amount,
            frequency,
            strategy_type: DCAType::DipBuying,
            rsi_config: None,
            volatility_config: None,
            sentiment_config: None,
            dynamic_factors: None,
            dip_levels: Some(dip_levels),
            reference_price,
            reference_period_days: Some(30), // Default to 30 days
            max_single_amount: None,
            min_single_amount: None,
            max_position_size: None,
            pause_on_high_volatility: false,
            volatility_pause_threshold: None,
            pause_on_bear_market: false,
            bear_market_threshold: None,
            filters: DCAFilters::default(),
        }
    }

    /// Create dynamic DCA configuration
    pub fn dynamic(
        base_amount: Decimal,
        frequency: DCAFrequency,
        rsi_config: RSIConfig,
        volatility_config: VolatilityConfig,
        dynamic_factors: DynamicFactors,
    ) -> Self {
        Self {
            base_amount,
            frequency,
            strategy_type: DCAType::Dynamic,
            rsi_config: Some(rsi_config),
            volatility_config: Some(volatility_config),
            sentiment_config: None,
            dynamic_factors: Some(dynamic_factors),
            dip_levels: None,
            reference_price: None,
            reference_period_days: None,
            max_single_amount: None,
            min_single_amount: None,
            max_position_size: None,
            pause_on_high_volatility: false,
            volatility_pause_threshold: None,
            pause_on_bear_market: false,
            bear_market_threshold: None,
            filters: DCAFilters::default(),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate base amount
        if self.base_amount <= Decimal::ZERO {
            return Err("Base amount must be positive".to_string());
        }

        // Validate min/max amounts
        if let (Some(min), Some(max)) = (self.min_single_amount, self.max_single_amount) {
            if min > max {
                return Err("Minimum single amount cannot be greater than maximum".to_string());
            }
            if self.base_amount < min || self.base_amount > max {
                return Err("Base amount must be within min/max single amount range".to_string());
            }
        }

        // Validate strategy-specific configurations
        match self.strategy_type {
            DCAType::RSIBased | DCAType::Dynamic => {
                if self.rsi_config.is_none() {
                    return Err("RSI configuration is required for RSI-based strategies".to_string());
                }
            }
            DCAType::VolatilityBased => {
                if self.volatility_config.is_none() {
                    return Err("Volatility configuration is required for volatility-based strategies".to_string());
                }
            }
            DCAType::DipBuying => {
                if self.dip_levels.is_none() || self.dip_levels.as_ref().unwrap().is_empty() {
                    return Err("Dip levels are required for dip-buying strategy".to_string());
                }
            }
            DCAType::SentimentBased => {
                if self.sentiment_config.is_none() {
                    return Err("Sentiment configuration is required for sentiment-based strategies".to_string());
                }
            }
            _ => {}
        }

        // Validate dynamic factors
        if let Some(ref factors) = self.dynamic_factors {
            let total_weight = factors.rsi_weight + factors.volatility_weight +
                            factors.sentiment_weight + factors.trend_weight;
            if total_weight > Decimal::from_f32(1.0).unwrap() + Decimal::from_f32(0.01).unwrap() {
                return Err("Total weight of dynamic factors cannot exceed 1.0".to_string());
            }
        }

        Ok(())
    }

    /// Get JSON schema for this configuration
    pub fn json_schema() -> Value {
        json!({
            "type": "object",
            "required": ["base_amount", "frequency", "strategy_type"],
            "properties": {
                "base_amount": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Base amount to invest each interval (in quote currency)"
                },
                "frequency": {
                    "type": "object",
                    "description": "DCA execution frequency",
                    "oneOf": [
                        {
                            "properties": {
                                "Hourly": {"type": "number", "minimum": 1}
                            }
                        },
                        {
                            "properties": {
                                "Daily": {"type": "number", "minimum": 1}
                            }
                        },
                        {
                            "properties": {
                                "Weekly": {"type": "number", "minimum": 1}
                            }
                        },
                        {
                            "properties": {
                                "Monthly": {"type": "number", "minimum": 1}
                            }
                        },
                        {
                            "properties": {
                                "Custom": {"type": "number", "minimum": 1, "description": "Interval in minutes"}
                            }
                        }
                    ]
                },
                "strategy_type": {
                    "type": "string",
                    "enum": ["Simple", "RSIBased", "VolatilityBased", "Dynamic", "DipBuying", "SentimentBased"],
                    "description": "Type of DCA strategy to use"
                },
                "rsi_config": {
                    "type": "object",
                    "description": "RSI configuration (required for RSI-based strategies)",
                    "properties": {
                        "period": {"type": "integer", "minimum": 2, "maximum": 100},
                        "oversold_threshold": {"type": "number", "minimum": 0, "maximum": 100},
                        "overbought_threshold": {"type": "number", "minimum": 0, "maximum": 100},
                        "oversold_multiplier": {"type": "number", "minimum": 0},
                        "overbought_multiplier": {"type": "number", "minimum": 0},
                        "normal_multiplier": {"type": "number", "minimum": 0}
                    }
                },
                "volatility_config": {
                    "type": "object",
                    "description": "Volatility configuration (required for volatility-based strategies)",
                    "properties": {
                        "period": {"type": "integer", "minimum": 2, "maximum": 100},
                        "low_threshold": {"type": "number", "minimum": 0},
                        "high_threshold": {"type": "number", "minimum": 0},
                        "low_volatility_multiplier": {"type": "number", "minimum": 0},
                        "high_volatility_multiplier": {"type": "number", "minimum": 0},
                        "normal_multiplier": {"type": "number", "minimum": 0}
                    }
                },
                "max_single_amount": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Maximum amount per single purchase"
                },
                "min_single_amount": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Minimum amount per single purchase"
                },
                "max_position_size": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Maximum total position size (stop DCA when reached)"
                }
            }
        })
    }
}