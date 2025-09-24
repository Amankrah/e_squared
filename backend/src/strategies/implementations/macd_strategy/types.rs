use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// MACD signal types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MACDSignal {
    /// MACD line crossed above signal line (bullish)
    BullishCrossover,
    /// MACD line crossed below signal line (bearish)
    BearishCrossover,
    /// MACD histogram shows bullish divergence
    BullishDivergence,
    /// MACD histogram shows bearish divergence
    BearishDivergence,
    /// MACD line crossed above zero line (trend confirmation)
    ZeroCrossBullish,
    /// MACD line crossed below zero line (trend confirmation)
    ZeroCrossBearish,
    /// No clear signal
    None,
}

/// MACD signal strength
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MACDSignalStrength {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

/// Trade side for MACD operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// MACD strategy state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDStrategyState {
    /// Current MACD line value
    pub current_macd_line: Option<Decimal>,
    /// Current MACD signal line value
    pub current_signal_line: Option<Decimal>,
    /// Current MACD histogram value
    pub current_histogram: Option<Decimal>,
    /// Previous MACD line value
    pub previous_macd_line: Option<Decimal>,
    /// Previous MACD signal line value
    pub previous_signal_line: Option<Decimal>,
    /// Previous MACD histogram value
    pub previous_histogram: Option<Decimal>,
    /// Current position (1 for long, -1 for short, 0 for none)
    pub position: i8,
    /// Entry price of current position
    pub entry_price: Option<Decimal>,
    /// Total realized profit/loss
    pub realized_pnl: Decimal,
    /// Unrealized profit/loss
    pub unrealized_pnl: Decimal,
    /// Total number of trades
    pub total_trades: u32,
    /// Number of winning trades
    pub winning_trades: u32,
    /// Number of losing trades
    pub losing_trades: u32,
    /// Last signal generated
    pub last_signal: MACDSignal,
    /// Last signal timestamp
    pub last_signal_time: Option<DateTime<Utc>>,
    /// Maximum consecutive wins
    pub max_consecutive_wins: u32,
    /// Maximum consecutive losses
    pub max_consecutive_losses: u32,
    /// Current win/loss streak
    pub current_streak: i32, // Positive for wins, negative for losses
    /// Trend state tracking
    pub trend_state: TrendState,
}

impl Default for MACDStrategyState {
    fn default() -> Self {
        Self {
            current_macd_line: None,
            current_signal_line: None,
            current_histogram: None,
            previous_macd_line: None,
            previous_signal_line: None,
            previous_histogram: None,
            position: 0,
            entry_price: None,
            realized_pnl: Decimal::ZERO,
            unrealized_pnl: Decimal::ZERO,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            last_signal: MACDSignal::None,
            last_signal_time: None,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            current_streak: 0,
            trend_state: TrendState::Neutral,
        }
    }
}

/// Trend state based on MACD analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendState {
    /// Strong uptrend (MACD > 0, MACD > Signal, increasing histogram)
    StrongBullish,
    /// Weak uptrend (MACD > Signal but weakening)
    WeakBullish,
    /// Neutral/sideways
    Neutral,
    /// Weak downtrend (MACD < Signal but strengthening)
    WeakBearish,
    /// Strong downtrend (MACD < 0, MACD < Signal, decreasing histogram)
    StrongBearish,
}

/// MACD signal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDAnalysis {
    /// MACD signal type
    pub signal: MACDSignal,
    /// Signal strength
    pub strength: MACDSignalStrength,
    /// Signal confidence (0.0 to 1.0)
    pub confidence: Decimal,
    /// MACD line value
    pub macd_line: Decimal,
    /// Signal line value
    pub signal_line: Decimal,
    /// Histogram value (MACD - Signal)
    pub histogram: Decimal,
    /// Histogram change from previous period
    pub histogram_change: Decimal,
    /// Price at signal time
    pub price: Decimal,
    /// Market conditions
    pub market_conditions: MarketConditions,
    /// Crossover information (if applicable)
    pub crossover_info: Option<CrossoverInfo>,
    /// Trend analysis
    pub trend_analysis: TrendAnalysis,
}

/// Market conditions snapshot for MACD analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    /// Current price
    pub price: Decimal,
    /// 24h price change percentage
    pub price_change_24h: Option<Decimal>,
    /// Volume
    pub volume: Option<Decimal>,
    /// Price volatility
    pub volatility: Option<Decimal>,
    /// Support level
    pub support_level: Option<Decimal>,
    /// Resistance level
    pub resistance_level: Option<Decimal>,
    /// Long-term trend (SMA-based)
    pub long_term_trend: Option<TrendDirection>,
}

impl Default for MarketConditions {
    fn default() -> Self {
        Self {
            price: Decimal::ZERO,
            price_change_24h: None,
            volume: None,
            volatility: None,
            support_level: None,
            resistance_level: None,
            long_term_trend: None,
        }
    }
}

/// Trend direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Sideways,
}

/// MACD crossover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossoverInfo {
    /// Type of crossover
    pub crossover_type: CrossoverType,
    /// Speed of crossover (how quickly lines crossed)
    pub crossover_speed: Decimal,
    /// Distance between lines at crossover
    pub crossover_distance: Decimal,
    /// Histogram momentum at crossover
    pub histogram_momentum: Decimal,
}

/// Types of MACD crossovers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrossoverType {
    /// MACD crossed above signal line
    BullishSignalCross,
    /// MACD crossed below signal line
    BearishSignalCross,
    /// MACD crossed above zero line
    BullishZeroCross,
    /// MACD crossed below zero line
    BearishZeroCross,
}

/// MACD trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Current trend state
    pub trend_state: TrendState,
    /// Trend strength (0.0 to 1.0)
    pub trend_strength: Decimal,
    /// Momentum direction
    pub momentum_direction: MomentumDirection,
    /// Momentum acceleration
    pub momentum_acceleration: Decimal,
    /// Trend duration (periods)
    pub trend_duration: u32,
}

/// Momentum direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MomentumDirection {
    Accelerating,
    Decelerating,
    Stable,
}

/// MACD execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDExecution {
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
    /// Execution price
    pub price: Decimal,
    /// Executed quantity
    pub quantity: Decimal,
    /// Trade side
    pub side: TradeSide,
    /// MACD line value at execution
    pub macd_line: Decimal,
    /// Signal line value at execution
    pub signal_line: Decimal,
    /// Histogram value at execution
    pub histogram: Decimal,
    /// Signal that triggered execution
    pub signal: MACDSignal,
    /// Signal strength
    pub strength: MACDSignalStrength,
    /// Position size before execution
    pub position_before: i8,
    /// Position size after execution
    pub position_after: i8,
    /// Realized PnL from this execution
    pub realized_pnl: Decimal,
    /// Market conditions at execution
    pub market_conditions: MarketConditions,
    /// Execution reason
    pub reason: String,
}

/// MACD performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDPerformanceMetrics {
    /// Total return percentage
    pub total_return_pct: Decimal,
    /// Win rate (successful trades / total trades)
    pub win_rate: Decimal,
    /// Average winning trade
    pub avg_win: Decimal,
    /// Average losing trade
    pub avg_loss: Decimal,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: Decimal,
    /// Maximum drawdown percentage
    pub max_drawdown_pct: Decimal,
    /// Sharpe ratio
    pub sharpe_ratio: Option<Decimal>,
    /// Total number of trades
    pub total_trades: u32,
    /// Average trade duration
    pub avg_trade_duration: Option<Decimal>,
    /// MACD effectiveness metrics
    pub macd_metrics: MACDMetrics,
}

impl Default for MACDPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return_pct: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            avg_win: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            max_drawdown_pct: Decimal::ZERO,
            sharpe_ratio: None,
            total_trades: 0,
            avg_trade_duration: None,
            macd_metrics: MACDMetrics::default(),
        }
    }
}

/// MACD-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDMetrics {
    /// Signal crossover accuracy
    pub signal_crossover_accuracy: Decimal,
    /// Zero line crossover accuracy
    pub zero_crossover_accuracy: Decimal,
    /// Divergence signal accuracy
    pub divergence_accuracy: Decimal,
    /// Average MACD value at entry
    pub avg_entry_macd: Decimal,
    /// Average MACD value at exit
    pub avg_exit_macd: Decimal,
    /// Best performing signal type
    pub best_signal_type: MACDSignal,
    /// Histogram effectiveness
    pub histogram_effectiveness: Decimal,
}

impl Default for MACDMetrics {
    fn default() -> Self {
        Self {
            signal_crossover_accuracy: Decimal::ZERO,
            zero_crossover_accuracy: Decimal::ZERO,
            divergence_accuracy: Decimal::ZERO,
            avg_entry_macd: Decimal::ZERO,
            avg_exit_macd: Decimal::ZERO,
            best_signal_type: MACDSignal::None,
            histogram_effectiveness: Decimal::ZERO,
        }
    }
}

/// MACD signal filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDSignalFilters {
    /// Minimum volume requirement
    pub min_volume: Option<Decimal>,
    /// Maximum spread percentage
    pub max_spread_pct: Option<Decimal>,
    /// Require trend confirmation with SMA
    pub sma_trend_confirmation: bool,
    /// SMA period for trend confirmation
    pub sma_trend_period: usize,
    /// Minimum histogram change for signal validity
    pub min_histogram_change: Option<Decimal>,
    /// Price action confirmation required
    pub price_action_confirmation: bool,
    /// Minimum crossover distance
    pub min_crossover_distance: Option<Decimal>,
    /// Filter signals in consolidation zones
    pub filter_consolidation: bool,
}

impl Default for MACDSignalFilters {
    fn default() -> Self {
        Self {
            min_volume: None,
            max_spread_pct: Some(Decimal::new(5, 3)), // 0.5%
            sma_trend_confirmation: true,
            sma_trend_period: 50,
            min_histogram_change: Some(Decimal::new(1, 4)), // 0.0001 minimum change
            price_action_confirmation: false,
            min_crossover_distance: Some(Decimal::new(1, 5)), // 0.00001 minimum distance
            filter_consolidation: true,
        }
    }
}