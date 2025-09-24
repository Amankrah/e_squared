use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use rust_decimal::Decimal;

use super::types::{RiskSettings, SignalFilters};

/// Complete SMA Crossover strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMACrossoverConfig {
    /// Fast SMA period
    pub fast_period: usize,
    /// Slow SMA period
    pub slow_period: usize,
    /// Position size as percentage of available balance
    pub position_size_pct: Decimal,
    /// Risk management settings
    pub risk_settings: RiskSettings,
    /// Signal filters
    pub filters: SignalFilters,
    /// Enable long positions
    pub enable_long: bool,
    /// Enable short positions
    pub enable_short: bool,
    /// Use market orders instead of limit orders
    pub use_market_orders: bool,
    /// Additional confirmation indicators
    pub confirmation_indicators: ConfirmationSettings,
}

/// Settings for additional confirmation indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationSettings {
    /// Use RSI for confirmation
    pub use_rsi: bool,
    /// RSI period
    pub rsi_period: usize,
    /// Use MACD for confirmation
    pub use_macd: bool,
    /// MACD fast period
    pub macd_fast: usize,
    /// MACD slow period
    pub macd_slow: usize,
    /// MACD signal period
    pub macd_signal: usize,
    /// Use volume confirmation
    pub use_volume: bool,
    /// Volume period for average calculation
    pub volume_period: usize,
    /// Minimum volume multiplier (e.g., 1.5x average volume)
    pub min_volume_multiplier: Decimal,
}

impl Default for ConfirmationSettings {
    fn default() -> Self {
        Self {
            use_rsi: false,
            rsi_period: 14,
            use_macd: false,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            use_volume: false,
            volume_period: 20,
            min_volume_multiplier: Decimal::new(15, 1), // 1.5
        }
    }
}

impl Default for SMACrossoverConfig {
    fn default() -> Self {
        Self {
            fast_period: 10,
            slow_period: 30,
            position_size_pct: Decimal::new(10, 2), // 10%
            risk_settings: RiskSettings::default(),
            filters: SignalFilters::default(),
            enable_long: true,
            enable_short: true,
            use_market_orders: false,
            confirmation_indicators: ConfirmationSettings::default(),
        }
    }
}

impl SMACrossoverConfig {
    /// Create a simple SMA crossover configuration
    pub fn simple(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            ..Default::default()
        }
    }

    /// Create a conservative configuration with tight risk management
    pub fn conservative(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            position_size_pct: Decimal::new(5, 2), // 5%
            risk_settings: RiskSettings {
                stop_loss_pct: Decimal::new(15, 3), // 1.5%
                take_profit_pct: Decimal::new(3, 2), // 3%
                max_position_pct: Decimal::new(5, 2), // 5%
                min_signal_interval: 60, // 1 hour
                trailing_stop: true,
                trailing_stop_pct: Some(Decimal::new(1, 2)), // 1%
            },
            filters: SignalFilters {
                min_sma_spread_pct: Some(Decimal::new(2, 3)), // 0.2%
                macd_confirmation: true,
                ..Default::default()
            },
            confirmation_indicators: ConfirmationSettings {
                use_rsi: true,
                use_macd: true,
                use_volume: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create an aggressive configuration for trending markets
    pub fn aggressive(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            position_size_pct: Decimal::new(20, 2), // 20%
            risk_settings: RiskSettings {
                stop_loss_pct: Decimal::new(3, 2), // 3%
                take_profit_pct: Decimal::new(6, 2), // 6%
                max_position_pct: Decimal::new(25, 2), // 25%
                min_signal_interval: 15, // 15 minutes
                trailing_stop: false,
                trailing_stop_pct: None,
            },
            filters: SignalFilters {
                min_sma_spread_pct: Some(Decimal::new(5, 4)), // 0.05%
                macd_confirmation: false,
                ..Default::default()
            },
            use_market_orders: true,
            ..Default::default()
        }
    }

    /// Create a scalping configuration for short-term trades
    pub fn scalping(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            position_size_pct: Decimal::new(15, 2), // 15%
            risk_settings: RiskSettings {
                stop_loss_pct: Decimal::new(5, 3), // 0.5%
                take_profit_pct: Decimal::new(1, 2), // 1%
                max_position_pct: Decimal::new(30, 2), // 30%
                min_signal_interval: 5, // 5 minutes
                trailing_stop: true,
                trailing_stop_pct: Some(Decimal::new(3, 3)), // 0.3%
            },
            filters: SignalFilters {
                min_volume: Some(Decimal::from(1000000)), // 1M volume
                max_spread_pct: Some(Decimal::new(1, 3)), // 0.1%
                min_sma_spread_pct: Some(Decimal::new(2, 4)), // 0.02%
                ..Default::default()
            },
            confirmation_indicators: ConfirmationSettings {
                use_volume: true,
                min_volume_multiplier: Decimal::from(2), // 2x volume
                ..Default::default()
            },
            use_market_orders: true,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate SMA periods
        if self.fast_period >= self.slow_period {
            return Err("Fast period must be less than slow period".to_string());
        }

        if self.fast_period < 2 {
            return Err("Fast period must be at least 2".to_string());
        }

        if self.slow_period < 3 {
            return Err("Slow period must be at least 3".to_string());
        }

        // Validate position size
        if self.position_size_pct <= Decimal::ZERO || self.position_size_pct > Decimal::from(100) {
            return Err("Position size percentage must be between 0 and 100".to_string());
        }

        // Validate risk settings
        if self.risk_settings.stop_loss_pct <= Decimal::ZERO || self.risk_settings.stop_loss_pct > Decimal::from(50) {
            return Err("Stop loss percentage must be between 0 and 50".to_string());
        }

        if self.risk_settings.take_profit_pct <= Decimal::ZERO {
            return Err("Take profit percentage must be positive".to_string());
        }

        if self.risk_settings.max_position_pct <= Decimal::ZERO || self.risk_settings.max_position_pct > Decimal::from(100) {
            return Err("Max position percentage must be between 0 and 100".to_string());
        }

        // Validate confirmation settings
        if self.confirmation_indicators.use_rsi && self.confirmation_indicators.rsi_period < 2 {
            return Err("RSI period must be at least 2".to_string());
        }

        if self.confirmation_indicators.use_macd {
            if self.confirmation_indicators.macd_fast >= self.confirmation_indicators.macd_slow {
                return Err("MACD fast period must be less than slow period".to_string());
            }
        }

        // Validate filters
        if let Some(min_spread) = self.filters.min_sma_spread_pct {
            if min_spread < Decimal::ZERO {
                return Err("Minimum SMA spread percentage cannot be negative".to_string());
            }
        }

        if !self.enable_long && !self.enable_short {
            return Err("At least one of long or short positions must be enabled".to_string());
        }

        Ok(())
    }

    /// Get JSON schema for this configuration
    pub fn json_schema() -> Value {
        json!({
            "type": "object",
            "required": ["fast_period", "slow_period"],
            "properties": {
                "fast_period": {
                    "type": "integer",
                    "minimum": 2,
                    "maximum": 100,
                    "description": "Fast SMA period"
                },
                "slow_period": {
                    "type": "integer",
                    "minimum": 3,
                    "maximum": 200,
                    "description": "Slow SMA period"
                },
                "position_size_pct": {
                    "type": "number",
                    "minimum": 0.1,
                    "maximum": 100,
                    "description": "Position size as percentage of available balance"
                },
                "risk_settings": {
                    "type": "object",
                    "properties": {
                        "stop_loss_pct": {
                            "type": "number",
                            "minimum": 0.1,
                            "maximum": 50,
                            "description": "Stop loss percentage"
                        },
                        "take_profit_pct": {
                            "type": "number",
                            "minimum": 0.1,
                            "description": "Take profit percentage"
                        },
                        "max_position_pct": {
                            "type": "number",
                            "minimum": 0.1,
                            "maximum": 100,
                            "description": "Maximum position size percentage"
                        },
                        "min_signal_interval": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Minimum minutes between signals"
                        },
                        "trailing_stop": {
                            "type": "boolean",
                            "description": "Enable trailing stop loss"
                        },
                        "trailing_stop_pct": {
                            "type": "number",
                            "minimum": 0.1,
                            "description": "Trailing stop distance percentage"
                        }
                    }
                },
                "enable_long": {
                    "type": "boolean",
                    "description": "Enable long positions"
                },
                "enable_short": {
                    "type": "boolean",
                    "description": "Enable short positions"
                },
                "use_market_orders": {
                    "type": "boolean",
                    "description": "Use market orders instead of limit orders"
                },
                "confirmation_indicators": {
                    "type": "object",
                    "properties": {
                        "use_rsi": {
                            "type": "boolean",
                            "description": "Use RSI for confirmation"
                        },
                        "rsi_period": {
                            "type": "integer",
                            "minimum": 2,
                            "maximum": 100,
                            "description": "RSI period"
                        },
                        "use_macd": {
                            "type": "boolean",
                            "description": "Use MACD for confirmation"
                        },
                        "use_volume": {
                            "type": "boolean",
                            "description": "Use volume confirmation"
                        }
                    }
                }
            }
        })
    }
}