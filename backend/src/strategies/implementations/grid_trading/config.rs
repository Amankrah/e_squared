use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use rust_decimal::Decimal;

use super::types::{GridRiskSettings, GridSpacing, GridBounds, GridTradingMode, BoundsType};

/// Complete Grid Trading strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridTradingConfig {
    /// Number of grid levels (buy + sell orders)
    pub grid_levels: usize,
    /// Total investment amount
    pub total_investment: Decimal,
    /// Grid spacing configuration
    pub spacing: GridSpacing,
    /// Grid bounds configuration
    pub bounds: GridBounds,
    /// Risk management settings
    pub risk_settings: GridRiskSettings,
    /// Minimum order size
    pub min_order_size: Decimal,
    /// Maximum order size
    pub max_order_size: Option<Decimal>,
    /// Enable grid rebalancing
    pub enable_rebalancing: bool,
    /// Rebalancing interval (hours)
    pub rebalancing_interval: Option<u32>,
    /// Take profit threshold
    pub take_profit_threshold: Option<Decimal>,
    /// Stop loss threshold
    pub stop_loss_threshold: Option<Decimal>,
    /// Market making settings
    pub market_making: MarketMakingSettings,
}

/// Market making specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMakingSettings {
    /// Enable market making mode
    pub enabled: bool,
    /// Spread percentage for market making
    pub spread_pct: Decimal,
    /// Inventory target (0 = neutral, positive = long bias, negative = short bias)
    pub inventory_target: Decimal,
    /// Maximum deviation from target inventory
    pub max_inventory_deviation: Decimal,
    /// Adjust prices based on inventory
    pub inventory_adjustment: bool,
    /// Skew factor for inventory adjustment
    pub inventory_skew_factor: Decimal,
}

impl Default for MarketMakingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            spread_pct: Decimal::new(2, 3), // 0.2%
            inventory_target: Decimal::ZERO,
            max_inventory_deviation: Decimal::from(500),
            inventory_adjustment: true,
            inventory_skew_factor: Decimal::new(1, 3), // 0.1%
        }
    }
}

impl Default for GridTradingConfig {
    fn default() -> Self {
        Self {
            grid_levels: 10,
            total_investment: Decimal::from(1000),
            spacing: GridSpacing::default(),
            bounds: GridBounds::default(),
            risk_settings: GridRiskSettings::default(),
            min_order_size: Decimal::from(10),
            max_order_size: None,
            enable_rebalancing: true,
            rebalancing_interval: Some(24), // 24 hours
            take_profit_threshold: Some(Decimal::new(5, 2)), // 5%
            stop_loss_threshold: Some(Decimal::new(10, 2)), // 10%
            market_making: MarketMakingSettings::default(),
        }
    }
}

impl GridTradingConfig {
    /// Create a simple grid trading configuration
    pub fn simple(grid_levels: usize, total_investment: Decimal, spacing_pct: Decimal) -> Self {
        Self {
            grid_levels,
            total_investment,
            spacing: GridSpacing {
                mode: GridTradingMode::Standard,
                fixed_spacing: Some(spacing_pct),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create a conservative grid configuration with tight risk management
    pub fn conservative(grid_levels: usize, total_investment: Decimal) -> Self {
        Self {
            grid_levels,
            total_investment,
            spacing: GridSpacing {
                mode: GridTradingMode::Standard,
                fixed_spacing: Some(Decimal::new(5, 3)), // 0.5%
                ..Default::default()
            },
            bounds: GridBounds {
                upper_bound: Decimal::new(5, 2), // 5% above center
                lower_bound: Decimal::new(5, 2), // 5% below center
                bounds_type: BoundsType::PercentageFromCenter,
                auto_adjust: true,
                use_support_resistance: true,
            },
            risk_settings: GridRiskSettings {
                max_inventory: total_investment * Decimal::new(3, 1), // 30% max inventory
                stop_loss_pct: Some(Decimal::new(3, 2)), // 3%
                take_profit_pct: Some(Decimal::new(6, 2)), // 6%
                max_drawdown_pct: Decimal::new(5, 2), // 5%
                max_time_in_position: Some(72), // 72 hours
                dynamic_adjustment: true,
                volatility_pause_threshold: Some(Decimal::from(30)),
            },
            take_profit_threshold: Some(Decimal::new(3, 2)), // 3%
            stop_loss_threshold: Some(Decimal::new(5, 2)), // 5%
            ..Default::default()
        }
    }

    /// Create an aggressive grid configuration for trending markets
    pub fn aggressive(grid_levels: usize, total_investment: Decimal) -> Self {
        Self {
            grid_levels,
            total_investment,
            spacing: GridSpacing {
                mode: GridTradingMode::Standard,
                fixed_spacing: Some(Decimal::new(2, 2)), // 2%
                ..Default::default()
            },
            bounds: GridBounds {
                upper_bound: Decimal::new(20, 2), // 20% above center
                lower_bound: Decimal::new(20, 2), // 20% below center
                bounds_type: BoundsType::PercentageFromCenter,
                auto_adjust: false,
                use_support_resistance: false,
            },
            risk_settings: GridRiskSettings {
                max_inventory: total_investment, // 100% max inventory
                stop_loss_pct: Some(Decimal::new(10, 2)), // 10%
                take_profit_pct: Some(Decimal::new(20, 2)), // 20%
                max_drawdown_pct: Decimal::new(15, 2), // 15%
                max_time_in_position: None,
                dynamic_adjustment: false,
                volatility_pause_threshold: None,
            },
            enable_rebalancing: false,
            take_profit_threshold: Some(Decimal::new(10, 2)), // 10%
            stop_loss_threshold: Some(Decimal::new(15, 2)), // 15%
            ..Default::default()
        }
    }

    /// Create a market making grid configuration
    pub fn market_making(grid_levels: usize, total_investment: Decimal, spread_pct: Decimal) -> Self {
        Self {
            grid_levels,
            total_investment,
            spacing: GridSpacing {
                mode: GridTradingMode::Standard,
                fixed_spacing: Some(spread_pct / Decimal::from(2)), // Half spread per level
                ..Default::default()
            },
            bounds: GridBounds {
                upper_bound: Decimal::new(5, 2), // 5% above center
                lower_bound: Decimal::new(5, 2), // 5% below center
                bounds_type: BoundsType::PercentageFromCenter,
                auto_adjust: true,
                use_support_resistance: false,
            },
            risk_settings: GridRiskSettings {
                max_inventory: total_investment * Decimal::new(5, 1), // 50% max inventory
                stop_loss_pct: Some(Decimal::new(2, 2)), // 2%
                take_profit_pct: Some(Decimal::new(4, 2)), // 4%
                max_drawdown_pct: Decimal::new(3, 2), // 3%
                max_time_in_position: Some(24), // 24 hours
                dynamic_adjustment: true,
                volatility_pause_threshold: Some(Decimal::from(25)),
            },
            enable_rebalancing: true,
            rebalancing_interval: Some(4), // 4 hours
            take_profit_threshold: Some(Decimal::new(2, 2)), // 2%
            stop_loss_threshold: Some(Decimal::new(3, 2)), // 3%
            market_making: MarketMakingSettings {
                enabled: true,
                spread_pct,
                inventory_target: Decimal::ZERO,
                max_inventory_deviation: total_investment * Decimal::new(2, 1), // 20%
                inventory_adjustment: true,
                inventory_skew_factor: Decimal::new(5, 4), // 0.05%
            },
            ..Default::default()
        }
    }

    /// Create a dynamic grid that adjusts to volatility
    pub fn dynamic(grid_levels: usize, total_investment: Decimal) -> Self {
        Self {
            grid_levels,
            total_investment,
            spacing: GridSpacing {
                mode: GridTradingMode::Dynamic,
                dynamic_base_pct: Some(Decimal::new(5, 3)), // 0.5% base spacing
                volatility_factor: Some(Decimal::new(15, 1)), // 1.5x volatility adjustment
                ..Default::default()
            },
            bounds: GridBounds {
                upper_bound: Decimal::from(3), // 3x ATR
                lower_bound: Decimal::from(3), // 3x ATR
                bounds_type: BoundsType::VolatilityBased,
                auto_adjust: true,
                use_support_resistance: true,
            },
            risk_settings: GridRiskSettings {
                max_inventory: total_investment * Decimal::new(4, 1), // 40% max inventory
                stop_loss_pct: Some(Decimal::new(5, 2)), // 5%
                take_profit_pct: Some(Decimal::new(8, 2)), // 8%
                max_drawdown_pct: Decimal::new(8, 2), // 8%
                max_time_in_position: Some(48), // 48 hours
                dynamic_adjustment: true,
                volatility_pause_threshold: Some(Decimal::from(40)),
            },
            enable_rebalancing: true,
            rebalancing_interval: Some(8), // 8 hours
            ..Default::default()
        }
    }

    /// Create a geometric progression grid
    pub fn geometric(grid_levels: usize, total_investment: Decimal, multiplier: Decimal) -> Self {
        Self {
            grid_levels,
            total_investment,
            spacing: GridSpacing {
                mode: GridTradingMode::Geometric,
                geometric_multiplier: Some(multiplier),
                ..Default::default()
            },
            bounds: GridBounds {
                upper_bound: Decimal::new(15, 2), // 15% above center
                lower_bound: Decimal::new(15, 2), // 15% below center
                bounds_type: BoundsType::PercentageFromCenter,
                auto_adjust: false,
                use_support_resistance: false,
            },
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate grid levels
        if self.grid_levels < 2 {
            return Err("Grid levels must be at least 2".to_string());
        }

        if self.grid_levels > 100 {
            return Err("Grid levels cannot exceed 100".to_string());
        }

        // Validate investment amount
        if self.total_investment <= Decimal::ZERO {
            return Err("Total investment must be positive".to_string());
        }

        // Validate order sizes
        if self.min_order_size <= Decimal::ZERO {
            return Err("Minimum order size must be positive".to_string());
        }

        if let Some(max_size) = self.max_order_size {
            if max_size <= self.min_order_size {
                return Err("Maximum order size must be greater than minimum".to_string());
            }
        }

        // Validate spacing configuration
        match self.spacing.mode {
            GridTradingMode::Standard | GridTradingMode::Arithmetic => {
                if self.spacing.fixed_spacing.is_none() {
                    return Err("Fixed spacing is required for Standard/Arithmetic grid".to_string());
                }
                let spacing = self.spacing.fixed_spacing.unwrap();
                if spacing <= Decimal::ZERO {
                    return Err("Grid spacing must be positive".to_string());
                }
            }
            GridTradingMode::Geometric => {
                if self.spacing.geometric_multiplier.is_none() {
                    return Err("Geometric multiplier is required for Geometric grid".to_string());
                }
                let multiplier = self.spacing.geometric_multiplier.unwrap();
                if multiplier <= Decimal::ONE {
                    return Err("Geometric multiplier must be greater than 1".to_string());
                }
            }
            GridTradingMode::Dynamic => {
                if self.spacing.dynamic_base_pct.is_none() {
                    return Err("Dynamic base percentage is required for Dynamic grid".to_string());
                }
                if self.spacing.volatility_factor.is_none() {
                    return Err("Volatility factor is required for Dynamic grid".to_string());
                }
            }
            _ => {}
        }

        // Validate bounds
        if self.bounds.upper_bound <= Decimal::ZERO || self.bounds.lower_bound <= Decimal::ZERO {
            return Err("Grid bounds must be positive".to_string());
        }

        // Validate risk settings
        if self.risk_settings.max_inventory <= Decimal::ZERO {
            return Err("Maximum inventory must be positive".to_string());
        }

        if self.risk_settings.max_drawdown_pct <= Decimal::ZERO || self.risk_settings.max_drawdown_pct > Decimal::from(50) {
            return Err("Max drawdown percentage must be between 0 and 50".to_string());
        }

        // Validate market making settings
        if self.market_making.enabled {
            if self.market_making.spread_pct <= Decimal::ZERO {
                return Err("Market making spread must be positive".to_string());
            }

            if self.market_making.max_inventory_deviation <= Decimal::ZERO {
                return Err("Max inventory deviation must be positive".to_string());
            }
        }

        Ok(())
    }

    /// Calculate order size per grid level
    pub fn calculate_order_size_per_level(&self) -> Decimal {
        let base_size = self.total_investment / Decimal::from(self.grid_levels);

        // Ensure it meets minimum requirements
        base_size.max(self.min_order_size)
    }

    /// Get JSON schema for this configuration
    pub fn json_schema() -> Value {
        json!({
            "type": "object",
            "required": ["grid_levels", "total_investment"],
            "properties": {
                "grid_levels": {
                    "type": "integer",
                    "minimum": 2,
                    "maximum": 100,
                    "description": "Number of grid levels (buy + sell orders)"
                },
                "total_investment": {
                    "type": "number",
                    "minimum": 1,
                    "description": "Total investment amount"
                },
                "spacing": {
                    "type": "object",
                    "properties": {
                        "mode": {
                            "type": "string",
                            "enum": ["Standard", "Arithmetic", "Geometric", "Dynamic", "ZoneBased"],
                            "description": "Grid spacing mode"
                        },
                        "fixed_spacing": {
                            "type": "number",
                            "minimum": 0,
                            "description": "Fixed spacing percentage between levels"
                        },
                        "geometric_multiplier": {
                            "type": "number",
                            "minimum": 1,
                            "description": "Multiplier for geometric progression"
                        }
                    }
                },
                "bounds": {
                    "type": "object",
                    "properties": {
                        "upper_bound": {
                            "type": "number",
                            "minimum": 0,
                            "description": "Upper grid bound"
                        },
                        "lower_bound": {
                            "type": "number",
                            "minimum": 0,
                            "description": "Lower grid bound"
                        },
                        "bounds_type": {
                            "type": "string",
                            "enum": ["AbsolutePrice", "PercentageFromCenter", "VolatilityBased", "TechnicalLevels"],
                            "description": "Type of grid bounds"
                        },
                        "auto_adjust": {
                            "type": "boolean",
                            "description": "Auto-adjust bounds based on market conditions"
                        }
                    }
                },
                "risk_settings": {
                    "type": "object",
                    "properties": {
                        "max_inventory": {
                            "type": "number",
                            "minimum": 0,
                            "description": "Maximum inventory size"
                        },
                        "stop_loss_pct": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 50,
                            "description": "Stop loss percentage"
                        },
                        "take_profit_pct": {
                            "type": "number",
                            "minimum": 0,
                            "description": "Take profit percentage"
                        },
                        "max_drawdown_pct": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 50,
                            "description": "Maximum drawdown percentage"
                        },
                        "dynamic_adjustment": {
                            "type": "boolean",
                            "description": "Enable dynamic grid adjustment"
                        }
                    }
                },
                "min_order_size": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Minimum order size"
                },
                "enable_rebalancing": {
                    "type": "boolean",
                    "description": "Enable grid rebalancing"
                },
                "rebalancing_interval": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Rebalancing interval in hours"
                },
                "market_making": {
                    "type": "object",
                    "properties": {
                        "enabled": {
                            "type": "boolean",
                            "description": "Enable market making mode"
                        },
                        "spread_pct": {
                            "type": "number",
                            "minimum": 0,
                            "description": "Spread percentage for market making"
                        },
                        "inventory_target": {
                            "type": "number",
                            "description": "Target inventory level"
                        }
                    }
                }
            }
        })
    }
}