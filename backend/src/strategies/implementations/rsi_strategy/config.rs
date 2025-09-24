use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use validator::Validate;
use super::types::{RSISignalFilters};

/// RSI Strategy Configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RSIStrategyConfig {
    /// RSI calculation period
    pub rsi_period: usize,

    /// RSI overbought level (typically 70)
    pub overbought_level: Decimal,

    /// RSI oversold level (typically 30)
    pub oversold_level: Decimal,

    /// Enable long positions
    pub enable_long: bool,

    /// Enable short positions
    pub enable_short: bool,

    /// Position sizing configuration
    pub position_sizing: PositionSizingConfig,

    /// Risk management settings
    pub risk_management: RSIRiskManagement,

    /// Signal filters
    pub signal_filters: RSISignalFilters,

    /// Divergence detection settings
    pub divergence_config: DivergenceConfig,

    /// Exit strategy configuration
    pub exit_strategy: ExitStrategyConfig,

    /// Performance tracking settings
    pub performance_config: PerformanceConfig,
}

impl Default for RSIStrategyConfig {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            overbought_level: Decimal::from(70),
            oversold_level: Decimal::from(30),
            enable_long: true,
            enable_short: true,
            position_sizing: PositionSizingConfig::default(),
            risk_management: RSIRiskManagement::default(),
            signal_filters: RSISignalFilters::default(),
            divergence_config: DivergenceConfig::default(),
            exit_strategy: ExitStrategyConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

/// Position sizing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSizingConfig {
    /// Position sizing method
    pub sizing_method: PositionSizingMethod,
    /// Fixed position size (if using Fixed method)
    pub fixed_size: Option<Decimal>,
    /// Percentage of portfolio per trade
    pub portfolio_percentage: Decimal,
    /// Risk per trade as percentage of portfolio
    pub risk_per_trade: Decimal,
    /// Maximum position size
    pub max_position_size: Decimal,
    /// Minimum position size
    pub min_position_size: Decimal,
}

impl Default for PositionSizingConfig {
    fn default() -> Self {
        Self {
            sizing_method: PositionSizingMethod::PortfolioPercentage,
            fixed_size: None,
            portfolio_percentage: Decimal::new(5, 2), // 5%
            risk_per_trade: Decimal::new(2, 2), // 2%
            max_position_size: Decimal::from(1000),
            min_position_size: Decimal::from(10),
        }
    }
}

/// Position sizing methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PositionSizingMethod {
    /// Fixed position size
    Fixed,
    /// Percentage of available portfolio
    PortfolioPercentage,
    /// Risk-based sizing (Kelly criterion style)
    RiskBased,
    /// RSI strength-based sizing
    RSIStrengthBased,
}

/// RSI-specific risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIRiskManagement {
    /// Stop loss percentage from entry
    pub stop_loss_pct: Option<Decimal>,
    /// Take profit percentage from entry
    pub take_profit_pct: Option<Decimal>,
    /// Maximum drawdown before stopping strategy
    pub max_drawdown_pct: Decimal,
    /// Maximum consecutive losses before pause
    pub max_consecutive_losses: u32,
    /// Cool-down period after max losses (minutes)
    pub cooldown_period: u32,
    /// RSI-based stop loss (exit if RSI reverses beyond threshold)
    pub rsi_stop_loss: Option<Decimal>,
    /// Trailing stop loss configuration
    pub trailing_stop: Option<TrailingStopConfig>,
}

impl Default for RSIRiskManagement {
    fn default() -> Self {
        Self {
            stop_loss_pct: Some(Decimal::new(5, 2)), // 5%
            take_profit_pct: Some(Decimal::new(10, 2)), // 10%
            max_drawdown_pct: Decimal::new(15, 2), // 15%
            max_consecutive_losses: 3,
            cooldown_period: 30, // 30 minutes
            rsi_stop_loss: Some(Decimal::from(10)), // Exit if RSI moves 10 points against position
            trailing_stop: Some(TrailingStopConfig::default()),
        }
    }
}

/// Trailing stop loss configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingStopConfig {
    /// Enable trailing stop
    pub enabled: bool,
    /// Activation threshold (profit percentage to activate trailing)
    pub activation_threshold: Decimal,
    /// Trailing distance (percentage)
    pub trailing_distance: Decimal,
    /// Use RSI-based trailing (trail based on RSI levels)
    pub rsi_based_trailing: bool,
    /// RSI trailing buffer
    pub rsi_trailing_buffer: Decimal,
}

impl Default for TrailingStopConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            activation_threshold: Decimal::new(3, 2), // Activate at 3% profit
            trailing_distance: Decimal::new(2, 2), // Trail by 2%
            rsi_based_trailing: false,
            rsi_trailing_buffer: Decimal::from(5), // 5 RSI points buffer
        }
    }
}

/// Divergence detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceConfig {
    /// Enable divergence detection
    pub enabled: bool,
    /// Lookback periods for divergence analysis
    pub lookback_periods: usize,
    /// Minimum divergence strength required
    pub min_strength: Decimal,
    /// Enable regular divergence signals
    pub enable_regular_divergence: bool,
    /// Enable hidden divergence signals
    pub enable_hidden_divergence: bool,
    /// Price swing detection sensitivity
    pub swing_sensitivity: Decimal,
    /// Minimum price swing size for divergence
    pub min_swing_size: Decimal,
}

impl Default for DivergenceConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default as it's more complex
            lookback_periods: 50,
            min_strength: Decimal::new(6, 1), // 0.6
            enable_regular_divergence: true,
            enable_hidden_divergence: false,
            swing_sensitivity: Decimal::new(15, 1), // 1.5%
            min_swing_size: Decimal::new(2, 2), // 2%
        }
    }
}

/// Exit strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategyConfig {
    /// Exit strategy type
    pub strategy_type: ExitStrategyType,
    /// RSI exit levels
    pub rsi_exit_levels: RSIExitLevels,
    /// Time-based exit
    pub time_based_exit: Option<TimeBasedExit>,
    /// Profit target multiplier
    pub profit_target_multiplier: Decimal,
    /// Partial exit configuration
    pub partial_exits: Option<PartialExitConfig>,
}

impl Default for ExitStrategyConfig {
    fn default() -> Self {
        Self {
            strategy_type: ExitStrategyType::RSIReversal,
            rsi_exit_levels: RSIExitLevels::default(),
            time_based_exit: None,
            profit_target_multiplier: Decimal::new(2, 0), // 2:1 risk-reward
            partial_exits: None,
        }
    }
}

/// Exit strategy types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExitStrategyType {
    /// Exit when RSI reverses from extreme levels
    RSIReversal,
    /// Exit using fixed profit/loss targets
    FixedTargets,
    /// Exit using trailing stops
    TrailingStop,
    /// Combined exit strategy
    Combined,
}

/// RSI exit levels configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIExitLevels {
    /// Exit long positions when RSI crosses below this level
    pub long_exit_level: Decimal,
    /// Exit short positions when RSI crosses above this level
    pub short_exit_level: Decimal,
    /// RSI centerline (50) exit on trend change
    pub centerline_exit: bool,
}

impl Default for RSIExitLevels {
    fn default() -> Self {
        Self {
            long_exit_level: Decimal::from(70), // Exit longs at overbought
            short_exit_level: Decimal::from(30), // Exit shorts at oversold
            centerline_exit: true,
        }
    }
}

/// Time-based exit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedExit {
    /// Maximum hold time (hours)
    pub max_hold_time: u32,
    /// Enable weekend exit
    pub weekend_exit: bool,
    /// Exit before market close (minutes)
    pub market_close_exit: Option<u32>,
}

/// Partial exit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialExitConfig {
    /// Enable partial exits
    pub enabled: bool,
    /// First partial exit level (RSI)
    pub first_exit_rsi: Decimal,
    /// First exit percentage
    pub first_exit_percentage: Decimal,
    /// Second partial exit level (RSI)
    pub second_exit_rsi: Decimal,
    /// Second exit percentage
    pub second_exit_percentage: Decimal,
}

/// Performance tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Track detailed metrics
    pub detailed_tracking: bool,
    /// Calculate Sharpe ratio
    pub calculate_sharpe: bool,
    /// Risk-free rate for Sharpe calculation
    pub risk_free_rate: Decimal,
    /// Performance reporting interval (hours)
    pub reporting_interval: u32,
    /// Maximum trade history to keep
    pub max_trade_history: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            detailed_tracking: true,
            calculate_sharpe: true,
            risk_free_rate: Decimal::new(2, 2), // 2% annual
            reporting_interval: 24, // Daily reports
            max_trade_history: 1000,
        }
    }
}

/// RSI Strategy preset configurations
impl RSIStrategyConfig {
    /// Conservative RSI configuration for lower risk
    pub fn conservative() -> Self {
        Self {
            rsi_period: 21, // Longer period for smoother signals
            overbought_level: Decimal::from(75), // More extreme levels
            oversold_level: Decimal::from(25),
            enable_long: true,
            enable_short: false, // Long-only for safety
            position_sizing: PositionSizingConfig {
                portfolio_percentage: Decimal::new(3, 2), // 3% per trade
                risk_per_trade: Decimal::new(1, 2), // 1% risk
                ..PositionSizingConfig::default()
            },
            risk_management: RSIRiskManagement {
                stop_loss_pct: Some(Decimal::new(3, 2)), // 3% stop loss
                take_profit_pct: Some(Decimal::new(6, 2)), // 6% take profit
                max_consecutive_losses: 2,
                ..RSIRiskManagement::default()
            },
            ..Self::default()
        }
    }

    /// Aggressive RSI configuration for higher returns
    pub fn aggressive() -> Self {
        Self {
            rsi_period: 9, // Shorter period for faster signals
            overbought_level: Decimal::from(65), // Less extreme levels
            oversold_level: Decimal::from(35),
            enable_long: true,
            enable_short: true,
            position_sizing: PositionSizingConfig {
                portfolio_percentage: Decimal::new(10, 2), // 10% per trade
                risk_per_trade: Decimal::new(5, 2), // 5% risk
                ..PositionSizingConfig::default()
            },
            risk_management: RSIRiskManagement {
                stop_loss_pct: Some(Decimal::new(8, 2)), // 8% stop loss
                take_profit_pct: Some(Decimal::new(15, 2)), // 15% take profit
                max_consecutive_losses: 5,
                ..RSIRiskManagement::default()
            },
            ..Self::default()
        }
    }

    /// Scalping RSI configuration for short-term trades
    pub fn scalping() -> Self {
        Self {
            rsi_period: 5, // Very short period
            overbought_level: Decimal::from(80), // Extreme levels for quick reversals
            oversold_level: Decimal::from(20),
            enable_long: true,
            enable_short: true,
            position_sizing: PositionSizingConfig {
                portfolio_percentage: Decimal::new(15, 2), // 15% per trade
                risk_per_trade: Decimal::new(3, 2), // 3% risk
                ..PositionSizingConfig::default()
            },
            risk_management: RSIRiskManagement {
                stop_loss_pct: Some(Decimal::new(15, 1)), // 1.5% stop loss
                take_profit_pct: Some(Decimal::new(3, 2)), // 3% take profit
                max_consecutive_losses: 3,
                ..RSIRiskManagement::default()
            },
            exit_strategy: ExitStrategyConfig {
                strategy_type: ExitStrategyType::FixedTargets,
                ..ExitStrategyConfig::default()
            },
            ..Self::default()
        }
    }
}