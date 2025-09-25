use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use validator::Validate;
use super::types::MACDSignalFilters;

/// MACD Strategy Configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MACDStrategyConfig {
    /// Fast EMA period for MACD calculation
    pub fast_period: usize,

    /// Slow EMA period for MACD calculation
    pub slow_period: usize,

    /// Signal line EMA period
    pub signal_period: usize,

    /// Enable long positions
    pub enable_long: bool,

    /// Enable short positions
    pub enable_short: bool,

    /// Position sizing configuration
    pub position_sizing: PositionSizingConfig,

    /// Risk management settings
    pub risk_management: MACDRiskManagement,

    /// Signal filters
    pub signal_filters: MACDSignalFilters,

    /// Signal configuration
    pub signal_config: SignalConfig,

    /// Exit strategy configuration
    pub exit_strategy: ExitStrategyConfig,

    /// Performance tracking settings
    pub performance_config: PerformanceConfig,
}

impl Default for MACDStrategyConfig {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            enable_long: true,
            enable_short: true,
            position_sizing: PositionSizingConfig::default(),
            risk_management: MACDRiskManagement::default(),
            signal_filters: MACDSignalFilters::default(),
            signal_config: SignalConfig::default(),
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
    /// Scale position size based on MACD strength
    pub scale_by_macd_strength: bool,
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
            scale_by_macd_strength: true,
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
    /// Risk-based sizing
    RiskBased,
    /// MACD momentum-based sizing
    MACDMomentumBased,
}

/// MACD-specific risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDRiskManagement {
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
    /// MACD-based stop loss (exit if MACD reverses)
    pub macd_reversal_stop: bool,
    /// Histogram-based stop loss
    pub histogram_stop_threshold: Option<Decimal>,
    /// Trailing stop loss configuration
    pub trailing_stop: Option<TrailingStopConfig>,
}

impl Default for MACDRiskManagement {
    fn default() -> Self {
        Self {
            stop_loss_pct: Some(Decimal::new(4, 2)), // 4%
            take_profit_pct: Some(Decimal::new(8, 2)), // 8% (2:1 risk-reward)
            max_drawdown_pct: Decimal::new(15, 2), // 15%
            max_consecutive_losses: 3,
            cooldown_period: 30, // 30 minutes
            macd_reversal_stop: true,
            histogram_stop_threshold: Some(Decimal::new(5, 4)), // 0.0005 threshold
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
    /// Use MACD-based trailing (trail based on MACD levels)
    pub macd_based_trailing: bool,
    /// MACD trailing buffer
    pub macd_trailing_buffer: Decimal,
}

impl Default for TrailingStopConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            activation_threshold: Decimal::new(25, 1), // Activate at 2.5% profit
            trailing_distance: Decimal::new(15, 1), // Trail by 1.5%
            macd_based_trailing: true,
            macd_trailing_buffer: Decimal::new(1, 4), // 0.0001 buffer
        }
    }
}

/// Signal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfig {
    /// Enable signal line crossover signals
    pub enable_signal_crossover: bool,
    /// Enable zero line crossover signals
    pub enable_zero_crossover: bool,
    /// Enable divergence signals
    pub enable_divergence: bool,
    /// Enable histogram signals
    pub enable_histogram: bool,
    /// Signal strength requirements
    pub signal_strength: SignalStrengthConfig,
    /// Confirmation requirements
    pub confirmation: ConfirmationConfig,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            enable_signal_crossover: true,
            enable_zero_crossover: true,
            enable_divergence: false, // More complex, disabled by default
            enable_histogram: true,
            signal_strength: SignalStrengthConfig::default(),
            confirmation: ConfirmationConfig::default(),
        }
    }
}

/// Signal strength configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalStrengthConfig {
    /// Minimum histogram value for strong signal
    pub min_strong_histogram: Decimal,
    /// Minimum crossover distance for valid signal
    pub min_crossover_distance: Decimal,
    /// Minimum momentum change for signal
    pub min_momentum_change: Decimal,
    /// Require increasing histogram for bullish signals
    pub require_histogram_acceleration: bool,
}

impl Default for SignalStrengthConfig {
    fn default() -> Self {
        Self {
            min_strong_histogram: Decimal::new(1, 4), // 0.0001
            min_crossover_distance: Decimal::new(5, 5), // 0.00005
            min_momentum_change: Decimal::new(1, 5), // 0.00001
            require_histogram_acceleration: true,
        }
    }
}

/// Confirmation requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationConfig {
    /// Require price confirmation (price moving in signal direction)
    pub price_confirmation: bool,
    /// Price confirmation period (bars)
    pub price_confirmation_bars: usize,
    /// Require volume confirmation
    pub volume_confirmation: bool,
    /// Volume increase threshold for confirmation
    pub volume_increase_threshold: Decimal,
    /// Require trend alignment with SMA
    pub trend_alignment: bool,
    /// SMA period for trend alignment
    pub trend_sma_period: usize,
}

impl Default for ConfirmationConfig {
    fn default() -> Self {
        Self {
            price_confirmation: true,
            price_confirmation_bars: 2,
            volume_confirmation: false,
            volume_increase_threshold: Decimal::new(2, 1), // 20% volume increase
            trend_alignment: true,
            trend_sma_period: 50,
        }
    }
}

/// Exit strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategyConfig {
    /// Exit strategy type
    pub strategy_type: ExitStrategyType,
    /// MACD exit conditions
    pub macd_exit_conditions: MACDExitConditions,
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
            strategy_type: ExitStrategyType::MACDReversal,
            macd_exit_conditions: MACDExitConditions::default(),
            time_based_exit: None,
            profit_target_multiplier: Decimal::new(2, 0), // 2:1 risk-reward
            partial_exits: None,
        }
    }
}

/// Exit strategy types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExitStrategyType {
    /// Exit when MACD shows reversal signals
    MACDReversal,
    /// Exit using fixed profit/loss targets
    FixedTargets,
    /// Exit using trailing stops
    TrailingStop,
    /// Combined exit strategy
    Combined,
}

/// MACD exit conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDExitConditions {
    /// Exit on opposite signal crossover
    pub opposite_crossover: bool,
    /// Exit on zero line cross against position
    pub zero_line_exit: bool,
    /// Exit on histogram reversal
    pub histogram_reversal: bool,
    /// Histogram reversal threshold
    pub histogram_reversal_threshold: Decimal,
    /// Exit on momentum weakening
    pub momentum_weakening: bool,
    /// Momentum weakening periods
    pub momentum_periods: usize,
}

impl Default for MACDExitConditions {
    fn default() -> Self {
        Self {
            opposite_crossover: true,
            zero_line_exit: false, // Can be too sensitive
            histogram_reversal: true,
            histogram_reversal_threshold: Decimal::new(3, 4), // 0.0003
            momentum_weakening: true,
            momentum_periods: 3,
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
    /// First partial exit trigger
    pub first_exit_trigger: PartialExitTrigger,
    /// First exit percentage
    pub first_exit_percentage: Decimal,
    /// Second partial exit trigger
    pub second_exit_trigger: PartialExitTrigger,
    /// Second exit percentage
    pub second_exit_percentage: Decimal,
}

/// Partial exit triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialExitTrigger {
    /// Trigger type
    pub trigger_type: PartialExitTriggerType,
    /// Trigger value (profit percentage, MACD value, etc.)
    pub trigger_value: Decimal,
}

/// Types of partial exit triggers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PartialExitTriggerType {
    /// Profit percentage reached
    ProfitPercentage,
    /// MACD histogram reached level
    HistogramLevel,
    /// MACD line reached level
    MACDLevel,
    /// Time-based (hours)
    TimeBased,
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
    /// Track MACD-specific metrics
    pub track_macd_metrics: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            detailed_tracking: true,
            calculate_sharpe: true,
            risk_free_rate: Decimal::new(2, 2), // 2% annual
            reporting_interval: 24, // Daily reports
            max_trade_history: 1000,
            track_macd_metrics: true,
        }
    }
}

/// MACD Strategy preset configurations
impl MACDStrategyConfig {
    /// Conservative MACD configuration for lower risk
    pub fn conservative() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 14, // Longer signal period for smoother signals
            enable_long: true,
            enable_short: false, // Long-only for safety
            position_sizing: PositionSizingConfig {
                portfolio_percentage: Decimal::new(3, 2), // 3% per trade
                risk_per_trade: Decimal::new(1, 2), // 1% risk
                scale_by_macd_strength: false,
                ..PositionSizingConfig::default()
            },
            risk_management: MACDRiskManagement {
                stop_loss_pct: Some(Decimal::new(3, 2)), // 3% stop loss
                take_profit_pct: Some(Decimal::new(6, 2)), // 6% take profit
                max_consecutive_losses: 2,
                ..MACDRiskManagement::default()
            },
            signal_config: SignalConfig {
                enable_zero_crossover: false, // Disable for less signals
                enable_histogram: false,
                ..SignalConfig::default()
            },
            ..Self::default()
        }
    }

    /// Aggressive MACD configuration for higher returns
    pub fn aggressive() -> Self {
        Self {
            fast_period: 8, // Shorter periods for faster signals
            slow_period: 21,
            signal_period: 5,
            enable_long: true,
            enable_short: true,
            position_sizing: PositionSizingConfig {
                portfolio_percentage: Decimal::new(10, 2), // 10% per trade
                risk_per_trade: Decimal::new(5, 2), // 5% risk
                scale_by_macd_strength: true,
                ..PositionSizingConfig::default()
            },
            risk_management: MACDRiskManagement {
                stop_loss_pct: Some(Decimal::new(6, 2)), // 6% stop loss
                take_profit_pct: Some(Decimal::new(12, 2)), // 12% take profit
                max_consecutive_losses: 5,
                ..MACDRiskManagement::default()
            },
            signal_config: SignalConfig {
                enable_zero_crossover: true,
                enable_histogram: true,
                enable_divergence: true,
                ..SignalConfig::default()
            },
            ..Self::default()
        }
    }

    /// Scalping MACD configuration for short-term trades
    pub fn scalping() -> Self {
        Self {
            fast_period: 5, // Very short periods
            slow_period: 13,
            signal_period: 3,
            enable_long: true,
            enable_short: true,
            position_sizing: PositionSizingConfig {
                portfolio_percentage: Decimal::new(15, 2), // 15% per trade
                risk_per_trade: Decimal::new(2, 2), // 2% risk
                scale_by_macd_strength: true,
                ..PositionSizingConfig::default()
            },
            risk_management: MACDRiskManagement {
                stop_loss_pct: Some(Decimal::new(1, 2)), // 1% stop loss
                take_profit_pct: Some(Decimal::new(2, 2)), // 2% take profit
                max_consecutive_losses: 3,
                macd_reversal_stop: true,
                ..MACDRiskManagement::default()
            },
            exit_strategy: ExitStrategyConfig {
                strategy_type: ExitStrategyType::MACDReversal,
                macd_exit_conditions: MACDExitConditions {
                    histogram_reversal: true,
                    histogram_reversal_threshold: Decimal::new(1, 4), // Very sensitive
                    ..MACDExitConditions::default()
                },
                ..ExitStrategyConfig::default()
            },
            ..Self::default()
        }
    }
}