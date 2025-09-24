use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// RSI signal types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RSISignal {
    /// RSI indicates oversold condition (potential buy)
    Oversold,
    /// RSI indicates overbought condition (potential sell)
    Overbought,
    /// RSI shows bullish divergence
    BullishDivergence,
    /// RSI shows bearish divergence
    BearishDivergence,
    /// No clear signal
    None,
}

/// RSI signal strength
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RSISignalStrength {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

/// Trade side for RSI operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// RSI strategy state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIStrategyState {
    /// Current RSI value
    pub current_rsi: Option<Decimal>,
    /// Previous RSI value
    pub previous_rsi: Option<Decimal>,
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
    pub last_signal: RSISignal,
    /// Last signal timestamp
    pub last_signal_time: Option<DateTime<Utc>>,
    /// Maximum consecutive wins
    pub max_consecutive_wins: u32,
    /// Maximum consecutive losses
    pub max_consecutive_losses: u32,
    /// Current win/loss streak
    pub current_streak: i32, // Positive for wins, negative for losses
    /// Average hold time per trade (in hours)
    pub average_hold_time: Option<Decimal>,
}

impl Default for RSIStrategyState {
    fn default() -> Self {
        Self {
            current_rsi: None,
            previous_rsi: None,
            position: 0,
            entry_price: None,
            realized_pnl: Decimal::ZERO,
            unrealized_pnl: Decimal::ZERO,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            last_signal: RSISignal::None,
            last_signal_time: None,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            current_streak: 0,
            average_hold_time: None,
        }
    }
}

/// RSI signal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIAnalysis {
    /// RSI signal type
    pub signal: RSISignal,
    /// Signal strength
    pub strength: RSISignalStrength,
    /// Signal confidence (0.0 to 1.0)
    pub confidence: Decimal,
    /// Current RSI value
    pub rsi_value: Decimal,
    /// RSI change from previous period
    pub rsi_change: Decimal,
    /// Price at signal time
    pub price: Decimal,
    /// Market conditions
    pub market_conditions: MarketConditions,
    /// Divergence information (if applicable)
    pub divergence_info: Option<DivergenceInfo>,
}

/// Market conditions snapshot for RSI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    /// Current price
    pub price: Decimal,
    /// 24h price change percentage
    pub price_change_24h: Option<Decimal>,
    /// Volume
    pub volume: Option<Decimal>,
    /// Price volatility (ATR or similar)
    pub volatility: Option<Decimal>,
    /// Support level
    pub support_level: Option<Decimal>,
    /// Resistance level
    pub resistance_level: Option<Decimal>,
    /// Market trend direction
    pub trend: Option<TrendDirection>,
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
            trend: None,
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

/// RSI divergence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceInfo {
    /// Type of divergence
    pub divergence_type: DivergenceType,
    /// Strength of divergence
    pub strength: Decimal,
    /// Time period of divergence analysis
    pub lookback_periods: usize,
    /// Price high/low involved in divergence
    pub price_extreme: Decimal,
    /// RSI high/low involved in divergence
    pub rsi_extreme: Decimal,
}

/// Types of RSI divergence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DivergenceType {
    /// Price makes higher high, RSI makes lower high
    BearishRegular,
    /// Price makes lower low, RSI makes higher low
    BullishRegular,
    /// Price makes lower high, RSI makes higher high
    BearishHidden,
    /// Price makes higher low, RSI makes lower low
    BullishHidden,
}

/// RSI execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIExecution {
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
    /// Execution price
    pub price: Decimal,
    /// Executed quantity
    pub quantity: Decimal,
    /// Trade side
    pub side: TradeSide,
    /// RSI value at execution
    pub rsi_value: Decimal,
    /// Signal that triggered execution
    pub signal: RSISignal,
    /// Signal strength
    pub strength: RSISignalStrength,
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

/// RSI performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIPerformanceMetrics {
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
    /// RSI effectiveness metrics
    pub rsi_metrics: RSIMetrics,
}

impl Default for RSIPerformanceMetrics {
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
            rsi_metrics: RSIMetrics::default(),
        }
    }
}

/// RSI-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIMetrics {
    /// Oversold signal accuracy
    pub oversold_accuracy: Decimal,
    /// Overbought signal accuracy
    pub overbought_accuracy: Decimal,
    /// Divergence signal accuracy
    pub divergence_accuracy: Decimal,
    /// Average RSI at entry
    pub avg_entry_rsi: Decimal,
    /// Average RSI at exit
    pub avg_exit_rsi: Decimal,
    /// Best performing RSI range
    pub best_rsi_range: (Decimal, Decimal),
}

impl Default for RSIMetrics {
    fn default() -> Self {
        Self {
            oversold_accuracy: Decimal::ZERO,
            overbought_accuracy: Decimal::ZERO,
            divergence_accuracy: Decimal::ZERO,
            avg_entry_rsi: Decimal::ZERO,
            avg_exit_rsi: Decimal::ZERO,
            best_rsi_range: (Decimal::ZERO, Decimal::ZERO),
        }
    }
}

/// RSI signal filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSISignalFilters {
    /// Minimum volume requirement
    pub min_volume: Option<Decimal>,
    /// Maximum spread percentage
    pub max_spread_pct: Option<Decimal>,
    /// Require SMA trend confirmation
    pub sma_trend_confirmation: bool,
    /// SMA period for trend confirmation
    pub sma_trend_period: usize,
    /// Minimum RSI change for signal validity
    pub min_rsi_change: Option<Decimal>,
    /// Price action confirmation required
    pub price_action_confirmation: bool,
}

impl Default for RSISignalFilters {
    fn default() -> Self {
        Self {
            min_volume: None,
            max_spread_pct: Some(Decimal::new(5, 3)), // 0.5%
            sma_trend_confirmation: false,
            sma_trend_period: 50,
            min_rsi_change: Some(Decimal::from(5)), // RSI must change by at least 5 points
            price_action_confirmation: false,
        }
    }
}